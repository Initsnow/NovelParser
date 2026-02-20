---
description: NovelParser 项目开发指南，包含架构、技术栈、模块说明和开发规范
---

# NovelParser 开发指南

## 项目概述

Tauri v2 桌面应用，使用 AI 逐章分析小说（EPUB/TXT），支持 8 种可配置分析维度。

## 技术栈

| 层 | 技术 |
|----|------|
| 前端框架 | React 19 + Vite 7 + TypeScript |
| 样式 | TailwindCSS 4 + DaisyUI (night/emerald 主题) |
| 状态管理 | Zustand |
| 路由 | react-router-dom |
| 图标 | lucide-react |
| 后端 | Rust + Tauri v2 |
| LLM 调用 | async-openai (支持自定义 base_url) |
| 数据存储 | SQLite (rusqlite, bundled) |
| EPUB 解析 | epub-rs v2 + html2text |
| TXT 编码 | encoding_rs (UTF-8/GBK/GB18030) |

## 项目结构

```
src/                        # React 前端
├── types/index.ts          # TypeScript 类型定义（镜像 Rust models）
├── store/novelStore.ts     # Zustand 全局状态 + Tauri IPC 封装
├── pages/
│   ├── HomePage.tsx        # 首页：小说列表 + 导入
│   ├── NovelPage.tsx       # 分析主页：章节列表 + 分析视图
│   └── SummaryPage.tsx     # 全书汇总报告
├── components/
│   ├── layout/AppLayout.tsx
│   ├── LlmConfigModal.tsx  # LLM 设置弹窗
│   ├── DimensionSelector.tsx
│   ├── ManualPromptPanel.tsx
│   └── ChapterAnalysisView.tsx
├── App.tsx                 # 路由配置
├── main.tsx                # 入口
└── index.css               # TailwindCSS + DaisyUI 配置

src-tauri/src/              # Rust 后端
├── lib.rs                  # Tauri 入口，注册 19 个 commands + plugins
├── models.rs               # 所有数据结构 (Novel, Chapter, Analysis, LlmConfig)
├── storage.rs              # SQLite 持久化 (novelparser.db)
├── epub_parser.rs          # EPUB → 章节列表
├── txt_parser.rs           # TXT → 章节列表 (自动编码检测 + 章节正则)
├── token_utils.rs          # Token 估算 + 内容分段
├── prompt.rs               # 动态 Prompt 组装
├── llm.rs                  # async-openai API 客户端
└── analysis.rs             # JSON 解析容错 + 多段合并
```

## 核心架构要点

### 前后端通信
- 前端通过 `@tauri-apps/api/core` 的 `invoke()` 调用 Rust commands
- 所有 IPC 都封装在 `store/novelStore.ts` 的 Zustand actions 中
- 类型在 `types/index.ts` 和 `models.rs` 之间手动同步

### 数据库 Schema (SQLite)
```sql
novels (id, title, source_type, enabled_dimensions, created_at)
chapters (id, novel_id, chapter_index, title, content, analysis)
novel_summaries (novel_id, summary)
summary_cache (id, novel_id, layer, group_index, content)
settings (key, value)  -- 存储 LlmConfig 等
```
- WAL 模式，ON DELETE CASCADE
- 章节内容按需加载（`load_chapter_content` 只取 content 列）
- analysis 字段存储为 JSON 字符串

### 8 种分析维度 (AnalysisDimension)
`characters` | `plot` | `foreshadowing` | `writing_technique` | `rhetoric` | `emotion` | `themes` | `worldbuilding`

- 用户可多选，存储在 `novel.enabled_dimensions`
- Prompt 按选中维度动态拼装（见 `prompt.rs`）
- `ChapterAnalysis` 所有字段为 `Option<T>`，只填充选中的维度

### LLM 交互两种模式
1. **API 模式**：`llm.rs` 通过 async-openai 调用，支持任意 OpenAI 兼容 API
2. **手动模式**：生成 prompt → 用户复制到外部 AI → 粘贴 JSON 回来解析

### Token 管理
- 估算：中文字符 × 1.5，ASCII 字符 × 0.3
- 超长章节自动按段落边界分段 → 分段分析 → 合并结果
- 全书汇总采用树归约策略（尚未完整实现 command）

## 开发命令

// turbo-all

1. 安装依赖
```bash
pnpm install
```

2. 开发模式运行
```bash
cargo tauri dev
```

3. Rust 类型检查
```bash
cd src-tauri && cargo check
```

4. TypeScript 类型检查
```bash
npx tsc --noEmit
```

5. 生产构建
```bash
cargo tauri build
```

## 开发规范

### Rust 端
- 所有数据结构在 `models.rs` 集中定义，实现 `Serialize` + `Deserialize`
- Command 错误统一返回 `Result<T, String>`，通过 `.map_err(|e| e.to_string())` 转换
- Database 通过 `State<AppState>` 注入，用 `Mutex` 保护
- 新增 command 需要在 `lib.rs` 的 `generate_handler![]` 中注册

### 前端
- 新增 Rust 类型时同步更新 `types/index.ts`
- 新增 Tauri command 时同步在 `store/novelStore.ts` 添加 action
- UI 组件使用 DaisyUI 类名（`btn`, `card`, `badge`, `modal` 等）
- 主题色由 DaisyUI 语义色控制：`primary`, `secondary`, `accent`, `base-*`

### 新增分析维度流程
1. `models.rs`: 在 `AnalysisDimension` enum 和 `ChapterAnalysis` 添加字段
2. `prompt.rs`: 在 `dimension_instruction()` 和 `generate_json_schema()` 添加模板
3. `analysis.rs`: 在 `merge_segment_analyses()` 添加合并逻辑
4. `types/index.ts`: 同步 TypeScript 类型
5. `ChapterAnalysisView.tsx`: 添加对应维度的渲染区块
6. `DimensionSelector` 自动从 `get_all_dimensions` command 读取，无需改动

## 已实现的重要功能 (Recent Features)
- [x] 全书汇总的完整 command（树归约调用链与前端展示）
- [x] 批量分析与多选批处理（一键分析/删除选取章节）
- [x] 分析进度实时事件推送（Tauri events: `analysis_progress`, `batch_progress`）
- [x] 导出报告（支持 Markdown 导出）
- [x] 深色/浅色主题切换 UI (DaisyUI themes)
- [x] 更精确的 EPUB 子章节层级合并 (TOC Flattening)
- [x] 流式输出展示 (Streaming JSON) 与时间预估统计
