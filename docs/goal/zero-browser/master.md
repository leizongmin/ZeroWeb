# ZeroBrowser 运行时控制平面

**最后更新**: 2026-05-30
**当前活跃里程碑**: M1 — 项目骨架 + 渲染基础设施迁移
**执行状态**: 进行中

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | 空仓库（仅有文档） |
| Cargo workspace | 未创建 |
| 编译状态 | N/A（无代码） |
| 测试状态 | N/A（无测试） |
| 覆盖率 | N/A |
| WPT 通过率 | N/A |
| 性能基线 | N/A |
| CI | 未配置 |
| 文档 | ✅ 目标文档、Spec+RFC、技术调研已完成 |

### 已完成文档

- [x] `docs/goal/zero-browser.md` — 目标执行契约 v1.0
- [x] `docs/specs/zero-browser-spec-rfc.md` — Spec + RFC v1.0（完整 10 章节）
- [x] `docs/research/rust-cross-platform-browser-research.md` — 技术调研（四轮迭代）
- [x] `README.md` — 项目说明

### 文档控制平面状态

- [x] 入口文档（`docs/goal/zero-browser.md`）已就位
- [x] 运行时控制平面（本文件）已创建
- [x] 归档区域（`docs/goal/zero-browser/archive/`）已创建
- [x] Spec + RFC 文档已就位

---

## 活跃里程碑：M1 — 项目骨架 + 渲染基础设施迁移

**目标**: 建立项目结构，迁移 OmniTerm 渲染基础设施，在桌面平台上显示一个窗口并渲染文本。

### M1 交付物进度

| # | 交付物 | 状态 | 备注 |
|---|--------|------|------|
| 1 | 完整的 Cargo workspace 结构，所有 crate 骨架就位 | 🔲 待开始 | |
| 2 | `render-foundation` crate 从 OmniTerm 迁移并适配 | 🔲 待开始 | GPU/CPU 双路径、字体栈、图片缓存 |
| 3 | `host-runtime` crate 支持 winit 窗口创建和事件循环 | 🔲 待开始 | |
| 4 | 可以在 macOS/Linux/Windows 上创建窗口，使用 wgpu 渲染文本 | 🔲 待开始 | "Hello ZeroBrowser" |
| 5 | 所有 crate 编译通过，`cargo clippy` 无警告 | 🔲 待开始 | |
| 6 | `render-foundation` 单元测试（≥20 个测试用例） | 🔲 待开始 | |
| 7 | criterion 基准基础设施就位 | 🔲 待开始 | |
| 8 | `render-foundation` 首批基准（≥3 个） | 🔲 待开始 | |
| 9 | 覆盖率测量脚本就位 | 🔲 待开始 | |
| 10 | CI 管线就位 | 🔲 待开始 | |

### M1 验收标准

- `cargo build` 在三个桌面平台上成功
- `cargo test` 全通过，render-foundation 覆盖率 ≥ 50%
- `cargo bench` 可运行并输出结果
- 运行 demo 二进制可以看到窗口和渲染文本
- OmniTerm 渲染核心代码已迁移到本仓库

---

## OmniTerm 可复用资产清单

| OmniTerm 模块 | 行数(估) | 功能 | 迁移目标 |
|---------------|----------|------|----------|
| `omniterm-terminal-render` | ~1,500 | 场景/Primitive/Backend 分层架构 | `render-foundation` |
| `omniterm-terminal-render-wgpu` | ~2,300 | GPU glyph atlas、pane 缓存、wgpu 合成 | `render-foundation` |
| `omniterm-terminal-render-soft` | ~4,400 | fontdue + swash 字体栈、软件渲染后备 | `render-foundation` |
| `omniterm-terminal-image` | ~1,800 | 图片对象缓存与 GC 限制 | `render-foundation` |

**迁移策略**: 复用核心抽象（RenderBackend trait、scene/primitive 模式、glyph atlas、字体 fallback 链、图片缓存 GC），去掉终端特有逻辑（cell grid、terminal snapshot、kitty/sixel 协议解码）。

---

## 已确认的技术决策

| 决策 | 选择 | 状态 |
|------|------|------|
| 技术路线 | Route A — 自建内核 | 已确认 |
| CSS 解析方案 | 完全自建 | 已确认 |
| JS 页面引擎 | V8（rusty_v8） | 已确认 |
| JS 扩展沙箱 | QuickJS（feature-gated） | 已确认 |
| 布局基础 | taffy 扩展 | 已确认 |
| 渲染基础 | OmniTerm 复用 + wgpu | 已确认 |
| 进程模型 | 浏览器进程 + 多渲染进程 | 已确认 |

---

## 测试基线

**当前状态**: 无测试（项目无代码）

---

## 下一步计划

1. 初始化 Cargo workspace（含所有 15 个 crate 骨架）
2. 迁移 OmniTerm 渲染基础设施到 `render-foundation` crate
3. 实现 `host-runtime` crate（winit 窗口 + 事件循环）
4. 创建 "Hello ZeroBrowser" demo
5. 建立 CI 管线

---

## 未解决问题

| ID | 问题 | 优先级 | 状态 |
|----|------|--------|------|
| TBD-1 | MSRV（最低支持 Rust 版本）策略 | 重要 | 待定 |
| TBD-2 | OmniTerm 代码复用许可证确认 | 重要 | 假设同团队可复用 |
| TBD-3 | V8 二进制分发策略 | 重要 | 待定 |
| TBD-4 | CSS 解析器性能目标 | 重要 | 待定 |
| TBD-9 | 浏览器 UI 框架选型 | 重要 | 待定 |

---

## 归档记录

无已完成里程碑。
