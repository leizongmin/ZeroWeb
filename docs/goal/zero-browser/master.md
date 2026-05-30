# ZeroBrowser 运行时控制平面

**最后更新**: 2026-05-30
**当前活跃里程碑**: M2 ✅ 已完成 | 下一活跃：M3 — CSS 解析器 + 样式系统
**执行状态**: M2 全部验收标准已满足，准备进入 M3

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 16 个 crate（dom 已实现） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 149 个测试全绿（82 dom + 53 render-foundation + 14 placeholder） |
| 覆盖率 | ✅ dom 85.45% / render-foundation 53.30% |
| WPT 通过率 | N/A |
| 性能基线 | ✅ `cargo bench` render-foundation 5 个 + dom 8 个基准可运行 |
| CI | ✅ GitHub Actions 配置就位 |
| Clippy | ✅ 零警告（含 dom crate） |

### 仓库结构

```
crates/           16 个 crate
  dom/            ✅ 完整实现（node/document/query/parser/serializer/mutation）
  render-foundation/ ✅ GPU+CPU 渲染
  其余 14 个      占位骨架
apps/             browser（占位）, webview-demo（占位）
tests/            wpt-runner, integration, benchmarks/results
scripts/          run-benchmarks.sh, check-coverage.sh
.github/          CI 管线（三平台 build + test + clippy + 基准）
```

### 文档控制平面状态

- [x] 入口文档（`docs/goal/zero-browser.md`）已就位
- [x] 运行时控制平面（本文件）已创建
- [x] 归档区域（`docs/goal/zero-browser/archive/`）已创建
- [x] Spec + RFC 文档已就位（整体 v1.3 + M2 专项 v1.0）

---

## 里程碑状态：M1 ✅ | M2 ✅ | 下一活跃：M3 — CSS 解析器 + 样式系统

### M2 交付物进度

| # | 交付物 | 状态 | 备注 |
|---|--------|------|------|
| 1 | `dom` crate 实现完整的 DOM 节点类型 | ✅ 完成 | Element、Text、Comment、Document、DocumentType、DocumentFragment、ProcessingInstruction |
| 2 | HTML 解析器集成 html5ever，生成 DOM 树 | ✅ 完成 | DomBuilder + TreeSink 实现，支持错误恢复 |
| 3 | DOM 修改 API（appendChild、removeChild、insertBefore 等） | ✅ 完成 | 含循环检测、重新挂载、cloneNode（深/浅） |
| 4 | Mutation Observer 基础框架 | ✅ 完成 | MutationRecord、MutationObserver、childList + attributes 变更记录 |
| 5 | 单元测试（≥50 个测试用例，覆盖率 ≥ 70%） | ✅ 完成 | 82 个测试全绿，覆盖 11 个维度 |
| 6 | 基准测试（≥3 个基准） | ✅ 完成 | 8 个 criterion 基准（树构建、查询、批量操作、解析吞吐量） |

### dom crate 已实现模块

| 模块 | 内容 | 测试 |
|------|------|------|
| `node` | NodeId、NodeKind、NodeData、ElementData、TextData、CommentData 等 | 8 |
| `document` | Document 结构体、所有 DOM 操作 API（创建/追加/移除/插入/替换/克隆/属性/查询/遍历） | 60 |
| `query` | SimpleSelector 解析器、querySelector/querySelectorAll | 7 |
| `parser` | DomBuilder（html5ever TreeSink）、parse_html() | 7 |
| `serializer` | HTML 序列化（outer_html、inner_html）、void 元素、转义 | 6 |
| `mutation` | MutationRecord、MutationType、MutationObserver | 4 |
| `attributes` | 属性操作辅助（功能已在 ElementData 和 Document 中实现） | 0 |

### M2 验收标准

- ✅ 可以解析标准 HTML5 文档并生成正确的 DOM 树
- ✅ DOM 树操作（增删改查）全部通过测试
- ✅ 解析器能处理错误恢复（malformed HTML）
- ✅ `cargo clippy` 零警告
- ✅ `cargo bench` 输出 DOM 操作的基线数据（8 个基准已就绪）
- ✅ dom crate 覆盖率 ≥ 70%（85.45% line coverage，87.91% region coverage）

---

## 覆盖率数据

### dom crate 覆盖率（M2 首次测量）

| 模块 | Region Coverage | Line Coverage |
|------|----------------|---------------|
| document.rs | 89.86% | 91.23% |
| mutation.rs | 100.00% | 100.00% |
| node.rs | 97.69% | 98.36% |
| query.rs | 95.15% | 93.98% |
| serializer.rs | 74.01% | 77.78% |
| parser.rs | 46.30% | 47.24% |
| **dom crate 整体** | **87.91%** | **85.45%** |

注：parser.rs 覆盖率较低是因为许多 TreeSink 边界情况（如 reparent_children、append_based_on_parent_node 等）仅在极端 HTML 结构中触发，正常 HTML 解析路径已完全覆盖。

### render-foundation 覆盖率（M1 测量）

| Crate | Region Coverage |
|-------|----------------|
| render-foundation (整体) | 53.30% |
| ├ color | 92.41% |
| ├ geometry | 98.24% |
| ├ surface | 92.86% |
| ├ font/cache | 89.34% |
| ├ primitive | 87.10% |
| ├ font/loader | 64.84% |
| ├ gpu/atlas | 92.21% |
| └ gpu/renderer | 15.40% |

---

## M1 交付物归档

M1 已完成并归档 → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)

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
| DOM 节点存储 | slotmap（稳定 NodeId + O(1) 查找） | M2 已确认 |
| html5ever 集成 | DomBuilder（RefCell 内部可变性） | M2 已确认 |

---

## 下一步计划

1. ~~初始化 Cargo workspace~~ ✅
2. ~~创建 render-foundation 核心抽象~~ ✅
3. ~~实现 host-runtime（winit 窗口 + 事件循环）~~ ✅
4. ~~建立 CI 管线~~ ✅
5. ~~创建 "Hello ZeroBrowser" 渲染 demo~~ ✅
6. ~~将 CPU 渲染 demo 升级为 wgpu GPU 渲染~~ ✅
7. ~~迁移 OmniTerm wgpu 渲染器~~ ✅
8. ~~提交并推送代码~~ ✅
9. ~~测量 render-foundation 覆盖率~~ ✅
10. ~~归档 M1 里程碑~~ ✅
11. ~~实现 dom crate 核心类型和操作~~ ✅
12. ~~集成 html5ever TreeSink~~ ✅
13. ~~实现查询 API 和属性操作~~ ✅
14. ~~实现 MutationObserver 框架~~ ✅
15. ~~编写 ≥50 单元测试~~ ✅（82 个）
16. ~~编写 ≥3 基准测试~~ ✅（8 个）
17. ~~测量 dom crate 覆盖率~~ ✅ 85.45%
18. 归档 M2 里程碑
19. 开始 M3 — CSS 解析器 + 样式系统

---

## 未解决问题

| ID | 问题 | 优先级 | 状态 |
|----|------|--------|------|
| TBD-1 | MSRV（最低支持 Rust 版本）策略 | ~~已解决~~ | ✅ Rust 1.85 |
| TBD-2 | OmniTerm 代码复用许可证确认 | 重要 | 假设同团队可复用 |
| TBD-3 | V8 二进制分发策略 | 重要 | 待定 |
| TBD-4 | CSS 解析器性能目标 | 重要 | 待定 |
| TBD-9 | 浏览器 UI 框架选型 | 重要 | 待定 |
| TBD-10 | 选择器语法完整支持范围（复杂选择器） | 重要 | M3 时确定 |

---

## 归档记录

- **M1 — 项目骨架 + 渲染基础设施迁移** ✅ 已归档 → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
