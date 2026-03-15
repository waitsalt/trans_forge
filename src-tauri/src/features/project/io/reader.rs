//! 多格式文件读取器

use std::fs;
use std::path::Path;

use crate::features::translation::{FileType, TranslationItem};

/// 从输入路径读取文件（支持文件或目录）
pub fn read_files_from_path(input_path: &str) -> Vec<TranslationItem> {
    let mut items = Vec::new();
    let path = Path::new(input_path);

    if !path.exists() {
        tracing::warn!("路径不存在：{}", input_path);
        return items;
    }

    if path.is_file() {
        // 单个文件
        let parent = path
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        let file_items = read_file(path, &parent);
        items.extend(file_items);
        tracing::info!("已从文件 {} 读取 {} 条内容", input_path, items.len());
    } else if path.is_dir() {
        // 目录：递归读取所有文件
        walk_directory(path, input_path, &mut items);
        tracing::info!("已从目录 {} 读取 {} 条内容", input_path, items.len());
    } else {
        tracing::warn!("无效路径：{}", input_path);
    }

    items
}

fn walk_directory(dir: &Path, base_path: &str, items: &mut Vec<TranslationItem>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk_directory(&path, base_path, items);
            } else if path.is_file() && path.extension().is_some() {
                let file_items = read_file(&path, base_path);
                items.extend(file_items);
            }
        }
    }
}

/// 读取单个文件并返回翻译条目
pub fn read_file(path: &Path, base_path: &str) -> Vec<TranslationItem> {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let file_type = FileType::from_extension(&ext);
    let file_path = path
        .to_str()
        .unwrap_or("")
        .strip_prefix(base_path)
        .map(|s| s.trim_start_matches('/').to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    match file_type {
        FileType::Txt => read_txt(path, &file_path),
        FileType::Md => read_md(path, &file_path),
        FileType::Srt => read_srt(path, &file_path),
        FileType::Ass => read_ass(path, &file_path),
        _ => {
            tracing::warn!("不支持的文件类型：{}", ext);
            Vec::new()
        }
    }
}

/// 读取纯文本文件
fn read_txt(path: &Path, file_path: &str) -> Vec<TranslationItem> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("读取 txt 文件失败 {}：{}", path.display(), e);
            return Vec::new();
        }
    };

    let mut items = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            items.push(TranslationItem::new(
                FileType::Txt,
                file_path.to_string(),
                i as i32,
                trimmed.to_string(),
            ));
        }
    }

    items
}

/// 读取 Markdown 文件
fn read_md(path: &Path, file_path: &str) -> Vec<TranslationItem> {
    // MVP 阶段先按纯文本处理
    // TODO: 增加更完整的 Markdown 解析（标题、代码块等）
    read_txt(path, file_path)
}

/// 读取 SRT 字幕文件
fn read_srt(path: &Path, file_path: &str) -> Vec<TranslationItem> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("读取 srt 文件失败 {}：{}", path.display(), e);
            return Vec::new();
        }
    };

    let mut items = Vec::new();

    // 按空行切分字幕块
    let blocks: Vec<&str> = content.split("\n\n").collect();

    for block in blocks {
        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            continue;
        }

        // 跳过序号与时间轴行，提取文本
        // 格式：序号\ntimecode\n文本
        if lines.len() >= 3 {
            let text = lines[2..].join(" ").trim().to_string();
            if !text.is_empty() {
                let index: i32 = lines[0].trim().parse().unwrap_or(0);
                items.push(TranslationItem::new(
                    FileType::Srt,
                    file_path.to_string(),
                    index,
                    text,
                ));
            }
        } else if !lines.is_empty() {
            // 兼容不规范字幕块（可能只有文本）
            let text = lines.join(" ").trim().to_string();
            if !text.is_empty()
                && !text
                    .chars()
                    .all(|c| c.is_ascii_digit() || c == ':' || c == '-' || c == ' ')
            {
                items.push(TranslationItem::new(
                    FileType::Srt,
                    file_path.to_string(),
                    items.len() as i32,
                    text,
                ));
            }
        }
    }

    items
}

/// 读取 ASS/SSA 字幕文件
fn read_ass(path: &Path, file_path: &str) -> Vec<TranslationItem> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("读取 ass 文件失败 {}：{}", path.display(), e);
            return Vec::new();
        }
    };

    let mut items = Vec::new();

    let mut in_events = false;
    let mut index = 0;

    let tag_regex = regex::Regex::new(r"\{[^}]*\}").ok();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.eq_ignore_ascii_case("[Events]") {
            in_events = true;
            continue;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_events = false;
            continue;
        }

        if in_events
            && trimmed
                .get(..9)
                .is_some_and(|s| s.eq_ignore_ascii_case("dialogue:"))
        {
            // 格式：Dialogue: Layer,Start,End,Style,Name,MarginL,MarginR,MarginV,Effect,Text
            if let Some(text) = trimmed.splitn(10, ',').last() {
                let text = text.trim();
                if !text.is_empty() {
                    // 去除 ASS 格式标签，如 {\pos(x,y)}、{\an8} 等
                    let clean_text = tag_regex
                        .as_ref()
                        .map(|r| r.replace_all(text, "").to_string())
                        .unwrap_or_else(|| text.to_string());

                    if !clean_text.trim().is_empty() {
                        items.push(TranslationItem::new(
                            FileType::Ass,
                            file_path.to_string(),
                            index,
                            clean_text.trim().to_string(),
                        ));
                        index += 1;
                    }
                }
            }
        }
    }

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_type_from_extension() {
        assert_eq!(FileType::from_extension("txt"), FileType::Txt);
        assert_eq!(FileType::from_extension("md"), FileType::Md);
        assert_eq!(FileType::from_extension("srt"), FileType::Srt);
        assert_eq!(FileType::from_extension("ass"), FileType::Ass);
        assert_eq!(FileType::from_extension("unknown"), FileType::Unknown);
    }
}
