//! 翻译结果文件写入器

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::Result;

use crate::shared::error::AppError;
use crate::features::translation::{FileType, TranslationItem};

/// 将翻译条目写回文件
pub fn write_files(items: &[TranslationItem], output_dir: &str) -> Result<usize> {
    if items.is_empty() {
        return Err(AppError::NoWritableTranslationItems.into());
    }

    // 按文件路径分组
    let mut files: HashMap<String, Vec<&TranslationItem>> = HashMap::new();
    for item in items {
        if item.status == crate::features::translation::ItemStatus::Processed {
            files.entry(item.file_path.clone()).or_default().push(item);
        }
    }

    let mut written_count = 0;

    for (file_path, file_items) in files {
        let mut sorted_items = file_items;
        sorted_items.sort_by_key(|item| item.index);

        let output_path = Path::new(output_dir).join(&file_path);

        // 创建父目录
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 根据文件类型写入
        let Some(first_item) = sorted_items.first() else {
            continue;
        };
        let result = match first_item.file_type {
            FileType::Txt => write_txt(&file_path, &sorted_items, &output_path),
            FileType::Md => write_md(&file_path, &sorted_items, &output_path),
            FileType::Srt => write_srt(&file_path, &sorted_items, &output_path),
            FileType::Ass => write_ass(&file_path, &sorted_items, &output_path),
            _ => {
                tracing::warn!("不支持写入的文件类型：{:?}", first_item.file_type);
                Ok(())
            }
        };

        match result {
            Ok(_) => {
                written_count += 1;
                tracing::info!("写入成功：{}", output_path.display());
            }
            Err(e) => {
                tracing::error!("写入失败 {}：{}", file_path, e);
            }
        }
    }

    Ok(written_count)
}

/// 写入纯文本文件
fn write_txt(_file_path: &str, items: &[&TranslationItem], output_path: &Path) -> Result<()> {
    let content: String = items
        .iter()
        .map(|item| item.translated_text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    fs::write(output_path, content)?;
    Ok(())
}

/// 写入 Markdown 文件
fn write_md(_file_path: &str, items: &[&TranslationItem], output_path: &Path) -> Result<()> {
    // MVP 阶段先按纯文本处理
    // TODO: 保留 Markdown 结构
    write_txt(_file_path, items, output_path)
}

/// 写入 SRT 字幕文件
fn write_srt(_file_path: &str, items: &[&TranslationItem], output_path: &Path) -> Result<()> {
    // 为了兼容性保留简单输出
    // MVP 阶段仅输出序号与翻译文本
    let mut content = String::new();

    for item in items {
        content.push_str(&format!("{}\n", item.index));
        content.push_str("00:00:00,000 --> 00:00:00,000\n");
        content.push_str(&item.translated_text);
        content.push_str("\n\n");
    }

    fs::write(output_path, content)?;
    Ok(())
}

/// 写入 ASS 字幕文件
fn write_ass(file_path: &str, items: &[&TranslationItem], output_path: &Path) -> Result<()> {
    // 读取原文件以尽量保留结构
    let original_path = output_path
        .file_name()
        .map(|_| file_path.replace("output/", ""));

    // 建立序号到翻译文本的映射
    let mut translations: HashMap<i32, String> = HashMap::new();
    for item in items {
        translations.insert(item.index, item.translated_text.clone());
    }

    // MVP 阶段若原文件不可读，则输出简化版本
    let content = if let Some(orig) = original_path {
        let orig_path = Path::new(&orig);
        if orig_path.exists() {
            // 读取并替换文本
            let original = fs::read_to_string(orig_path)?;
            replace_ass_text(&original, &translations)
        } else {
            // 生成简化输出
            create_simple_ass(&translations)
        }
    } else {
        create_simple_ass(&translations)
    };

    fs::write(output_path, content)?;
    Ok(())
}

/// 替换 ASS 文件中的对白文本
fn replace_ass_text(original: &str, translations: &HashMap<i32, String>) -> String {
    let mut result = String::new();
    let mut current_index = 0;
    let mut in_events = false;

    for line in original.lines() {
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("[Events]") {
            in_events = true;
            result.push_str(line);
            result.push('\n');
            continue;
        }

        if trimmed.starts_with('[')
            && trimmed.ends_with(']')
            && !trimmed.eq_ignore_ascii_case("[Events]")
        {
            in_events = false;
        }

        if in_events
            && trimmed
                .get(..9)
                .is_some_and(|s| s.eq_ignore_ascii_case("dialogue:"))
        {
            // 解析并替换文本
            let parts: Vec<&str> = trimmed.splitn(10, ',').collect();
            if parts.len() >= 10 {
                let mut new_line = parts[..9].join(",");
                if let Some(translation) = translations.get(&current_index) {
                    new_line.push_str(&format!(",{}", translation));
                } else {
                    new_line.push_str(&format!(",{}", parts[9]));
                }
                result.push_str(&new_line);
                result.push('\n');
                current_index += 1;
            } else {
                result.push_str(line);
                result.push('\n');
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    result
}

/// 创建简化 ASS 文件
fn create_simple_ass(translations: &HashMap<i32, String>) -> String {
    let mut content = String::from(
        "[Script Info]\nTitle: Translated\nScriptType: v4.00+\nCollisions: Normal\nPlayDepth: 0\n\n[V4+ Styles]\nFormat: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\nStyle: Default,Arial,20,&H00FFFFFF,&H000000FF,&H00000000,&H00000000,0,0,0,0,100,100,0,0,1,2,2,2,10,10,10,1\n\n[Events]\nFormat: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    let mut ordered: Vec<_> = translations.iter().collect();
    ordered.sort_by_key(|(index, _)| *index);

    for (_, text) in ordered {
        content.push_str(&format!(
            "Dialogue: 0,0:00:00.00,0:00:00.00,Default,,0,0,0,,{}\n",
            text
        ));
    }

    content
}
