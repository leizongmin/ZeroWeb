# M2-S8 — 垂直书写模式块起始边水平滚动（vertical-lr）（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案 + 探针页（用后
即删）+ 全量 per-subtest 精确 diff vs S7 终态）
**前序**: [evidence/2026-10-10-m2-s7-fragid-scroll-focus-faces.md](evidence/2026-10-10-m2-s7-fragid-scroll-focus-faces.md)

## 切片内容

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| `_scrollToAnchorForHash` 补垂直书写模式根的**块起始边水平对齐**：根 inline style `writing-mode: vertical-lr` → scrollX = 元素 border-box 左缘（native gBCR 文档绝对——S7 探针同约定；探针 rect.left=14 即期望位）。vertical-rl 原点在右（scrollX 负向）须滚动区宽度换算、inline-nearest 需内容高 clamp——均 headless 布局滚动范围缺，维持渲染域挂账。writing-mode 读法沿 part06 轴向判定同款 style 串正则（R35xx 先例） | `part02.js`（`_scrollToAnchorForHash` 单一 chokepoint） | spec scroll to the fragment 逻辑坐标（块轴随书写模式）；WPT scroll-position-vertical-lr「window.scrollX 14」 |

## 探针定谳（家族余项三分）

| 余案 | 探针/分析结论 | 归属 |
|---|---|---|
| scroll-position-vertical-lr | native rect.left=14 正确（vertical-lr abspos 布局对）——纯 shim scrollX 缺 | **本轮收口** |
| scroll-position-vertical-rl | scrollX 负向原点换算需滚动区宽度（headless 布局滚动范围缺） | 渲染域维持 |
| scroll-position-inline-nearest | inline 轴 nearest 需内容高 clamp（scrollHeight 近似为 client 尺寸，无内容范围） | 渲染域维持 |
| scroll-behavior reload 4F | manual-scroll 面（intercept scroll:'manual' 须抑制 reload 恢复滚）可 shim 化，但后继断言 scroll anchoring（buffer.remove() 位移 -1000）为渲染域——单点不可翻 | 渲染域维持 |

## 数字（按 evidence txt 原始 Pass 行）

| corpus 域 | S7 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 406 P / 476 = 85.3% | **407 P / 476 = 85.5%** | +1 |
| navigation-api | 237 / 255 | 237 / 255（92.9%） | 0（本案 html/browsers 域） |

**零回归**：全量 diff 恰一行（scroll-position-vertical-lr Fail→Pass）。

## 质量门禁

- `make test`：全绿（数字见本轮终态行；part02.js 变更轮）。
- fmt/clippy 不适用（纯 .js shim 变更，无 .rs 改动）。
