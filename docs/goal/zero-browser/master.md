# ZeroBrowser 运行时控制平面

**最后更新**: 2026-05-30
**当前活跃里程碑**: M4 ✅ 已完成 | 下一活跃：M5 — 渲染管线集成
**执行状态**: M4 全部验收标准已满足，准备进入 M5

---

## 当前仓库事实

| 项 | 状态 |
|----|------|
| 仓库代码 | ✅ Cargo workspace + 16 个 crate（dom/css-parser/style-system/layout-engine 已实现） |
| 编译状态 | ✅ `cargo build --workspace` 通过 |
| 测试状态 | ✅ `cargo test --workspace` 450 个测试全绿 |
| 覆盖率 | ✅ css-parser 86.88% / style-system ≥85% / dom 85.45% / layout-engine ≥83% |
| WPT 通过率 | N/A |
| 性能基线 | ✅ 26 个 criterion 基准可运行 |
| CI | ✅ GitHub Actions 配置就位 |
| Clippy | ✅ 零警告（全 workspace） |

### 仓库结构

```
crates/           16 个 crate
  layout-engine/  ✅ 完整实现（types/converter/tree/engine）— 61 测试
  css-parser/     ✅ 完整实现（tokenizer/parser/selector/values）— 138 测试
  style-system/   ✅ 完整实现（cascade/inheritance/computed/matcher/property）— 101 测试
  dom/            ✅ 完整实现（node/document/query/parser/serializer/mutation）— 82 测试
  render-foundation/ ✅ GPU+CPU 渲染 — 53 测试
  其余 11 个      占位骨架
apps/             browser（占位）, webview-demo（占位）
tests/            wpt-runner, integration, benchmarks/results
scripts/          run-benchmarks.sh, check-coverage.sh
```

---

## 里程碑状态：M1 ✅ | M2 ✅ | M3 ✅ | M4 ✅ | 下一活跃：M5 — 渲染管线集成

### M4 交付物进度

| # | 交付物 | 状态 | 备注 |
|---|--------|------|------|
| 1 | `layout-engine` crate 实现布局树构建和布局计算 | ✅ 完成 | 基于 taffy 0.7 |
| 2 | Block layout（正常流块级布局） | ✅ 完成 | taffy Display::Block |
| 3 | Flexbox layout（含 gap、align-self、order） | ✅ 完成 | flex-direction/wrap/grow/shrink/basis/gap/alignment |
| 4 | CSS Grid layout（含 grid-template、grid-area、gap） | ✅ 完成 | grid-template-rows/columns, grid-row/column |
| 5 | Positioned layout（relative、absolute） | ✅ 完成 | Fixed/Sticky 标记（需宿主层处理） |
| 6 | Overflow 和 scrolling 布局 | ✅ 完成 | Visible/Hidden/Clip/Scroll |
| 7 | 布局输出为盒模型坐标 | ✅ 完成 | LayoutBox 含 position/size/border/padding/margin |
| 8 | **单元测试**（≥60 个，覆盖率 ≥ 70%） | ✅ 完成 | 61 个测试，各模块 ≥83% |
| 9 | **基准测试**（≥5 个） | ✅ 完成 | 6 个 criterion 基准 |

### M4 验收标准

- ✅ 给定 DOM + 计算样式，可以生成正确的布局盒树
- ✅ Block/Inline/Flexbox/Grid 布局通过测试验证
- ✅ Fixed/Sticky 定位标记正确（宿主层后续处理）
- ✅ 布局引擎有充足的测试用例覆盖各种布局场景
- ✅ `cargo bench` 输出各布局模式的基线数据

### layout-engine 已实现模块

| 模块 | 内容 | 覆盖率 | 测试 |
|------|------|--------|------|
| `types` | LayoutBox、OverflowClip、LayoutResult | 100% | 10 |
| `converter` | ComputedStyle → taffy::Style 映射 | 83.19% | 17 |
| `tree` | DOM + styles → taffy 树构建 | 98.58% | 14 |
| `engine` | LayoutEngine 编排器、布局计算和结果提取 | 99.81% | 20 |

---

## 覆盖率数据

### layout-engine 覆盖率（M4 测量）

| 模块 | Line Coverage |
|------|---------------|
| converter.rs | 83.19% |
| engine.rs | 99.81% |
| tree.rs | 98.58% |
| types.rs | 100.00% |

### css-parser 覆盖率（M3 测量）

| 模块 | Line Coverage |
|------|---------------|
| parser.rs | 86.33% |
| selector.rs | 88.89% |
| tokenizer.rs | 82.85% |
| values.rs | 82.51% |
| **整体** | **86.88%** |

### style-system 覆盖率（M3 测量）

| 模块 | Line Coverage |
|------|---------------|
| cascade.rs | 100.00% |
| computed.rs | 95.34% |
| inheritance.rs | 98.64% |
| lib.rs | 99.25% |
| matcher.rs | 85.20% |
| property.rs | 89.54% |

### dom crate 覆盖率（M2 测量）

| 模块 | Line Coverage |
|------|---------------|
| **整体** | **85.45%** |

---

## 已确认的技术决策

| 决策 | 选择 | 状态 |
|------|------|------|
| CSS Token 设计 | Delim(char) 用于 `.`, `!`, `>`, `+`, `*`, `~` | M3 |
| CSS 级联排序 | CascadeOrder 五维排序 | M3 |
| ComputedStyle | 60+ 类型化属性字段 + Default trait | M3 |
| 选择器匹配 | 右到左遍历 + DOM 树关系检查 | M3 |
| 布局引擎 | taffy 0.7 集成（TaffyTree API） | M4 |
| 布局输出 | LayoutBox 树（含 position/size/border/padding/margin） | M4 |
| 百分比处理 | taffy 百分比范围 0.0-1.0 | M4 |
| Fixed/Sticky | 在 LayoutBox 中标记，由宿主层处理 | M4 |

---

## 下一步计划

1. ~~M1: 项目骨架 + 渲染基础设施~~ ✅
2. ~~M2: HTML 解析 + DOM 树~~ ✅
3. ~~M3: CSS 解析器 + 样式系统~~ ✅
4. ~~M4: 布局引擎~~ ✅
5. M5: 渲染管线集成（首屏渲染）

---

## 归档记录

- **M1 — 项目骨架 + 渲染基础设施迁移** ✅ → [archive/m1-skeleton-render-foundation.md](archive/m1-skeleton-render-foundation.md)
- **M2 — HTML 解析 + DOM 树** ✅ → [archive/m2-dom.md](archive/m2-dom.md)
- **M3 — CSS 解析器 + 样式系统** ✅ → [archive/m3-css-parser-style-system.md](archive/m3-css-parser-style-system.md)
