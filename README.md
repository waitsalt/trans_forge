# Trans Forge

一款基于 Tauri + Vue 3 的本地桌面翻译工具，用于配置多模型翻译服务、管理 Prompt 预设，并对文件/目录进行批量翻译与导出。

## 功能概览

- Provider 配置：支持 OpenAI / Google / Anthropic 格式，支持自定义 API Base、模型、超时、重试、限流与多 Key 轮换策略
- 项目管理：创建项目配置、选择输入/输出路径、设置源/目标语言与并发数
- Prompt 预设：按语言管理提示词，支持占位符 `{source}` / `{target}`
- 翻译运行：启动/暂停任务、错误重试、全部重试、实时进度统计
- 项目详情：按状态筛选条目，查看/编辑源文本与译文，支持导出结果
- 本地数据持久化：SQLite 存储 Provider / 项目 / 预设 / 翻译条目

## 支持的文件类型

- `txt`、`md`、`srt`、`ass`、`epub`、`xlsx/xls`、`json`

## 技术栈

- 前端：Vue 3 + Vue Router + Vite + TypeScript
- 桌面端：Tauri 2
- 存储：SQLite（`sqlx`）

## 开发与运行

### 依赖

- Bun（用于前端 dev/build 命令）
- Node.js（建议 LTS）
- Rust + Tauri CLI

### 安装依赖

```bash
bun install
```

### 仅启动前端（调试 UI）

```bash
bun run dev
```

### 启动桌面端

```bash
bun run tauri dev
```

### 打包构建

```bash
bun run tauri build
```

## 数据存储

默认数据目录为项目根目录下的 `.data`，数据库文件为 `.data/data.db`。在开发模式下可直接查看该文件以排查数据问题。

## 目录结构（简要）

- `src/`：Vue 前端页面与组件
- `src-tauri/`：Tauri 后端（Rust）命令与业务逻辑
- `src-tauri/tauri.conf.json`：Tauri 配置

## 备注

- Prompt 中的 `{source}` 和 `{target}` 会在翻译请求前由后端自动替换为语言代码。
- Provider 支持多 Key 以及按权重/可用性等策略进行轮换。
