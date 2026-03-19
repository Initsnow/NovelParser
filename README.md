# NovelParser

![logo](./src-tauri/icons/Square71x71Logo.png)

NovelParser 是一个基于 Tauri、Rust 和 React 的本地小说分析工具。它支持导入 EPUB 和 TXT，按章节调用大模型生成结构化分析，并在此基础上生成全书总结或快速提纲。

![NovelParser Preview](./preview.png)

## 功能

- 导入 EPUB 和 TXT，并自动拆分章节
- 按章节生成结构化分析，支持 API 模式和手动 Prompt 模式
- 批量处理章节，查看进度和耗时
- 基于章节结果生成全书总结
- 基于章节轻提纲生成全书大纲
- 导出分析报告或大纲文件
- 本地 SQLite 存储，数据默认保存在本机

## 导出内容

- 分析报告导出：
  - 全书分析 Markdown
  - 章节分析 Markdown
- 大纲导出：
  - `book-outline.md`
  - `chapter-outlines.md`

## 技术栈

- 后端：Rust、Tauri v2、rusqlite、tokio、async-openai
- 前端：React 19、TypeScript、Vite
- UI：Tailwind CSS、DaisyUI
- 状态管理：Zustand

## 开发

NovelParser 需要 Node.js 和 Rust 开发环境 (建议 Rust `1.80+`)。

1. 安装依赖

```bash
pnpm install
```

2. 启动开发环境

```bash
cargo tauri dev
```

3. 构建

```bash
cargo tauri build
```

## 说明

- 支持兼容 OpenAI API 的模型服务
- 快速提纲模式适合长篇大纲提取，不替代深度章节分析
- 章节列表已做虚拟渲染，适合上千章节规模使用

License: GPL-3.0
