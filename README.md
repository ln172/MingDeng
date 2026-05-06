# MingDeng

MingDeng 是一个轻量的桌面学习 TODO 挂件。它常驻在桌面上，默认是一个可拖动的小型今日任务浮窗，点击后可以下拉展开完整的学习助手功能。

## 当前形态

- 默认悬浮小窗：显示今日 TODO 摘要和勾选状态
- 下拉展开：查看日历、生成计划、资源库、统计、设置和 AI 对话
- 本地优先：结构化数据写入 JSON，长期记忆写入 Markdown
- 每日梦游：启动时检查长期记忆是否需要按天更新
- 跨平台：基于 Tauri v2，可在 macOS、Windows、Linux 构建

## 功能

- 今日任务：快速查看、完成、添加、编辑、删除任务
- 学习日历：周视图、月视图、每日任务弹窗
- AI 计划生成：把学习目标拆成可执行任务并写入日历
- AI 任务重排：根据完成率和长期记忆重新安排未完成任务
- AI 对话：结合 Markdown 长期记忆回答学习问题
- 资源库：保存视频、文章、论文和碎片资料，并自动关联任务
- 学习统计：总任务、完成数、完成率和难度分布
- 备份恢复：备份 JSON 数据和 `memory/` 目录，最多保留 10 个备份

## 技术栈

- Desktop：Tauri v2
- UI：Vite + Vanilla TypeScript + CSS
- Local service：Rust Tauri commands
- AI：OpenAI-compatible Chat Completions API
- Storage：JSON + Markdown

MingDeng 不再启动 Python 后端，也不依赖 FastAPI、mem0、qdrant 或 Python virtualenv。

## macOS 使用

开发运行：

```bash
npm install
npm run dev
```

构建安装包：

```bash
npm run build
```

构建产物位于：

```text
src-tauri/target/release/bundle/macos/MingDeng.app
src-tauri/target/release/bundle/dmg/MingDeng_0.2.0_aarch64.dmg
```

## Windows 使用

可以在 Windows 上使用，但需要在 Windows 机器上构建 Windows 安装包。

### Windows 开发依赖

根据 Tauri v2 官方文档，Windows 开发需要：

- Node.js
- Rust
- Microsoft C++ Build Tools，并勾选 `Desktop development with C++`
- Microsoft Edge WebView2 Runtime

Windows 10 1803 及更新版本通常已经安装 WebView2；如果没有，需要安装 WebView2 Evergreen Runtime。

官方文档：

- [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)
- [Tauri Windows installer](https://v2.tauri.app/distribute/windows-installer/)

### Windows 构建步骤

在 Windows PowerShell 中执行：

```powershell
git clone https://github.com/ln172/MingDeng.git
cd MingDeng
npm install
npm run build
```

构建完成后，Windows 安装包会生成在：

```text
src-tauri\target\release\bundle\
```

Tauri 在 Windows 上通常会生成 `.msi` 或 NSIS 的 `setup.exe` 安装包，具体取决于当前 Tauri bundle 配置和本机环境。

### Windows 直接开发运行

```powershell
npm run dev
```

首次运行需要配置 OpenAI-compatible API：

- API Base URL，例如 `https://api.openai.com/v1`
- API Key
- 模型名，例如 `gpt-4o-mini`、`deepseek-chat`、`qwen-plus`

## 数据位置

应用数据写入系统的 app data 目录，由 Tauri 自动选择。数据结构如下：

```text
config.json
todos.json
library.json
memory/
  long-term.md
  conversations/YYYY-MM-DD.md
  dreamwalk-log.md
backups/
```

说明：

- `config.json`：API 和用户设置
- `todos.json`：计划和任务
- `library.json`：资源库
- `memory/long-term.md`：长期记忆，带 frontmatter
- `memory/conversations/`：每日对话记录
- `memory/dreamwalk-log.md`：每日梦游日志

## 常用命令

```bash
npm install
npm run dev
npm run build:web
npm test
npm run build
```

## 开发原则

- 轻便优先：不引入 React/Vue/Svelte，不启动本地 HTTP 后端
- 本地优先：用户数据只写入本机
- 记忆透明：长期记忆使用可读写的 Markdown
- 跨平台优先：桌面能力通过 Tauri/Rust 封装
