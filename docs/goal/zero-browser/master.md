# ZeroBrowser 运行时控制平面

**最后更新**: 2026-05-30
**当前活跃里程碑**: M3 ✅ 已完成 | 下一活跃：M4 — 布局引擎
**执行状态**: M3 全部验收标准已满足，准备进入 M4

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 16 个 crate（dom ✅、css-parser ✅、style-system ✅） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 390 个测试全绿（138 css-parser + 101 style-system + 82 dom + 53 render-foundation + 16 placeholder） |
| 覆盖率 | ✅ css-parser 86.88% / style-system 各模块 ≥85% / dom 85.45% |
| WPT 通过率 | N/A |
| 性能基线 | ✅ `cargo bench` css-parser 5 个 + style-system 5 个 + render-foundation 5 个 + dom 8 个基准可运行 |
| CI | ✅ GitHub Actions 配置就位 |
| Clippy | ✅ 零警告（全 workspace） |

### 仓库结构

```
crates/           16 个 crate
  css-parser/     ✅ 完整实现（tokenizer/parser/selector/values）— 138 测试
  style-system/   ✅ 完整实现（cascade/inheritance/computed/matcher/property）— 101 测试
  dom/            ✅ 完整实现（node/document/query/parser/serializer/mutation）— 82 测试
  render-foundation/ ✅ GPU+CPU 渲染 — 53 测试
  其余 12 个      占位骨架
apps/             browser（占位）, webview-demo（占位）
tests/            wpt-runner, integration, benchmarks/results
scripts/          run-benchmarks.sh, check-coverage.sh
.github/          CI 管线（三平台 build + test + clippy + 基准）
```

### 文档控制平面状态

- [x] 入口文档（`docs/goal/zero-browser.md`）已就位
- [x] 运行时控制平面（本文件）已创建
- [x] 归档区域（`docs/goal/zero-browser/archive/`）已创建
- [x] Spec + RFC 文档已就位（整体 v1.3 + M2 专项 v1.0 + M3 专项 v1.0）

---

## 里程碑状态：M1 ✅ | M2 ✅ | M3 ✅ | 下一活跃：M4 — 布局引擎

### M3 交付物进度

| # | 交付物 | 状态 | 备注 |
|---|--------|------|------|
| 1 | `css-parser` crate 实现完整的 CSS 语法解析（tokenizer + parser） | ✅ 完成 | Delim token、完整选择器解析、值解析 |
| 2 | 支持选择器解析（类型、类、ID、属性、伪类、伪元素、组合器、`:is()`/`:where()`/`:not()`） | ✅ 完成 | 含 nth-child(odd/even/2n+1)、:lang() |
| 3 | `style-system` crate 实现级联、继承、初始值、计算值 | ✅ 完成 | CascadeOrder 含 origin/layer/specificity/position/important |
| 4 | 支持 CSS 属性：display、width/height、margin/padding/border、color、background、font、position、overflow、visibility、opacity、z-index、box-sizing、min/max、flexbox 全量 | ✅ 完成 | ComputedStyle 含 60+ 类型化属性 |
| 5 | 样式系统与 DOM 集成，可以为 DOM 节点计算样式 | ✅ 完成 | StyleSystem::compute_styles() 全文档样式计算 |
| 6 | 支持 `@media`、`@supports`、`@layer`、`@import` 规则 | ✅ 完成 | |
| 7 | **单元测试**（≥80 个测试用例，覆盖率 ≥ 70%） | ✅ 完成 | 239 个测试（138 css-parser + 101 style-system），覆盖率 css-parser 86.88%、style-system ≥85% |
| 8 | **基准测试**（≥4 个基准） | ✅ 完成 | 10 个 criterion 基准（5 css-parser + 5 style-system） |

### css-parser 已实现模块

| 模块 | 内容 | 测试 |
|------|------|------|
| `tokenizer` | Token 枚举（含 Delim）、Tokenizer 迭代器、字符串/URL/数字/转义处理 | 49 |
| `parser` | Parser 结构体、样式表/规则/声明/选择器/@规则解析 | 29 |
| `selector` | Specificity 计算 (A,B,C) | 7 |
| `values` | 颜色/长度/display/position/overflow/flex/alignment/visibility/font 解析 | 53 |
| `ast` | Stylesheet/Rule/AtRule/Selector/Declaration AST 类型 | — |

### style-system 已实现模块

| 模块 | 内容 | 测试 |
|------|------|------|
| `property` | ComputedStyle（60+ 属性）、PropertyRegistry（初始值/继承标记） | 28 |
| `cascade` | CascadeOrder、级联算法（origin/!important/@layer/specificity/position） | 13 |
| `inheritance` | 属性继承、inherit/initial/unset/revert 关键字 | 14 |
| `computed` | 相对→绝对值转换（em/rem/vh/vw/ch）、var() 解析 | 16 |
| `matcher` | 选择器匹配（tag/class/ID/attribute/pseudo/combinator） | 16 |
| `lib` | StyleSystem 编排器、compute_styles() | 6 |

### M3 验收标准

- ✅ CSS parser 可以解析标准 CSS 文本并生成正确的 AST
- ✅ 选择器引擎可以正确解析 DOM 节点选择器（含复杂选择器）
- ✅ 级联规则正确应用（specificity、!important、继承、@layer）
- ✅ 计算值生成正确
- ✅ 自定义属性可以声明、引用、回退
- ✅ `cargo bench` 输出 CSS 解析和样式计算的基线数据（10 个基准已就绪）
- ✅ css-parser 覆盖率 ≥ 70%（86.88%）
- ✅ style-system 覆盖率 ≥ 70%（各模块 ≥85%）

---

## 覆盖率数据

### css-parser 覆盖率（M3 测量）

| 模块 | Region Coverage | Line Coverage |
|------|----------------|---------------|
| parser.rs | 85.74% | 86.33% |
| selector.rs | 78.86% | 88.89% |
| tokenizer.rs | 80.21% | 82.85% |
| values.rs | 77.12% | 82.51% |
| **css-parser 整体** | **86.92%** | **86.88%** |

### style-system 覆盖率（M3 测量，仅 style-system 自身代码）

| 模块 | Region Coverage | Line Coverage |
|------|----------------|---------------|
| cascade.rs | 99.71% | 100.00% |
| computed.rs | 95.30% | 95.34% |
| inheritance.rs | 98.94% | 98.64% |
| lib.rs | 98.80% | 99.25% |
| matcher.rs | 81.95% | 85.20% |
| property.rs | 89.94% | 89.54% |

### dom crate 覆盖率（M2 测量）

| 模块 | Region Coverage | Line Coverage |
|------|----------------|---------------|
| document.rs | 89.86% | 91.23% |
| mutation.rs | 100.00% | 100.00% |
| node.rs | 97.69% | 98.36% |
| query.rs | 95.15% | 93.98% |
| serializer.rs | 74.01% | 77.78% |
| parser.rs | 46.30% | 47.24% |
| **dom crate 整体** | **87.91%** | **85.45%** |

### render-foundation 覆盖率（M1 测量）

| Crate | Region Coverage |
|-------|----------------|
| render-foundation (整体) | 53.30% |

---

## M1、M2 交付物归档

- **M1 — 项目骨架 + 渲染基础设施迁移** ✅ 已归档 → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2 — HTML 解析 + DOM 树** ✅ 已归档 → [archive/m2-dom.md](archive/m2-dom.md)

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
| CSS Token 设计 | Delim(char) 用于 `.`, `!`, `>`, `+`, `*`, `~` | M3 已确认 |
| CSS 级联排序 | CascadeOrder (origin + important + layer + specificity + position) | M3 已确认 |
| ComputedStyle | 60+ 类型化属性字段 + Default trait | M3 已确认 |
| 选择器匹配 | 右到左遍历 + DOM 树关系检查 | M3 已确认 |

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
18. ~~归档 M2 里程碑~~ ✅
19. ~~实现 CSS Tokenizer 增强（Delim token）~~ ✅
20. ~~实现 CSS Parser 选择器解析（class/attribute/pseudo/combinator）~~ ✅
21. ~~实现 CSS 值解析（color/length/display 等）~~ ✅
22. ~~实现 style-system 级联算法~~ ✅
23. ~~实现 style-system 继承和计算值~~ ✅
24. ~~实现选择器匹配与 DOM 集成~~ ✅
25. ~~编写 ≥80 单元测试~~ ✅（239 个）
26. ~~编写 ≥4 基准测试~~ ✅（10 个）
27. ~~测量覆盖率达到 ≥70%~~ ✅（css-parser 86.88%、style-system ≥85%）
28. 归档 M3 里程碑
29. 开始 M4 — 布局引擎

---

## 未解决问题

| ID | 问题 | 优先级 | 状态 |
|----|------|--------|------|
| TBD-1 | MSRV（最低支持 Rust 版本）策略 | ~~已解决~~ | ✅ Rust 1.85 |
| TBD-2 | OmniTerm 代码复用许可证确认 | 重要 | 假设同团队可复用 |
| TBD-3 | V8 二进制分发策略 | 重要 | 待定 |
| TBD-4 | CSS 解析器性能目标 | ~~已解决~~ | ✅ 基准数据已记录 |
| TBD-9 | 浏览器 UI 框架选型 | 重要 | 待定 |
| TBD-10 | 选择器语法完整支持范围 | ~~已解决~~ | ✅ M3 已覆盖全部 Tier 1 选择器 |

---

## 归档记录

- **M1 — 项目骨架 + 渲染基础设施迁移** ✅ 已归档 → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2 — HTML 解析 + DOM 树** ✅ 已归档 → [archive/m2-dom.md](archive/m2-dom.md)
