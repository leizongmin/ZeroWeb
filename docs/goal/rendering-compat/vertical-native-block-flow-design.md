# Vertical native block-flow — rally-pattern 设计（R1544，2026-07-16）

> 承接 R1541（vertical empirical ground truth）+ R1542（postprocess 路三证 net-negative）。
> 本文档是 vertical-mode 最大未触 frontier 的 **native block-flow** 实现设计（rally-pattern，
> 非 lei-spec-rfc——后者需用户确认与无人值守 rally 协议冲突，见 master.md R896）。

## 问题（ground truth，R1541 PIL measured）

ZW 对 `writing-mode: vertical-rl/lr` 容器的 **block-level 子**用 horizontal-tb block-flow
（垂直堆叠），应水平排列：

| 变体 | chromium（正确） | ZW（bug） |
|------|------------------|-----------|
| V2 vertical-rl 2 block | B1 右 x[52-101]，B2 左 x[2-51]（right-to-left，同 y） | B1 上 x[4-53] y[4-153]，B2 下 y[154-303]（垂直堆叠） |
| V3 vertical-lr 2 block | B1 左 x[2-51]，B2 右 x[52-101]（left-to-right） | 同 V2（不区分 rl/lr） |

inline-flow（字符推进）ZW 已垂直（V1，纠正 R1050），故 **仅 block-flow 方向 + 容器 auto-width
两层是缺口**（R1541 缩窄 R1050「四层」判断）。

## 为何 postprocess 路不可行（R1542 confirmed）

converter `apply_vertical_writing_mode`（mod.rs:268）仅 per-element swap 轴；taffy `Block`
display 恒垂直堆叠子。postprocess 重定位子水平排列 → **taffy 已按垂直堆叠算 container
content-size**（width=max child, height=sum），水平重定位后 container box 与子位置矛盾
（sizing 不一致）→ 传播到 parent 致回归。R1043 mirror + R1047 grow + R1542 三证 net-negative。

## native 方案

在 ZW 自定义 block layout（engine.rs compute 主循环）增加 **vertical block-flow 分支**：
对 `writing-mode: vertical-rl/lr` 的 block 容器，**layout 期**（非 postprocess）计算子位置 +
container content-size 一致：

- **container content-width（inline-size）** = Σ child outer-width（+ column-gap，block-flow
  方向是水平故 gap 用 column-gap 经 axis-swap = 原 row-gap）。
- **container content-height（block-size）** = max child outer-height。
- **child 位置**：
  - vertical-rl：DOM 序子从右到左。child[i].x = content_width − Σ_{0..=i} width[i]；
    child[i].y = container content-top（block-start = top）。
  - vertical-lr：DOM 序子从左到右。child[i].x = Σ_{0..i} width[i]；y = content-top。
- **child 自身**：经 `apply_vertical_writing_mode` 已 axis-swapped（size/border/margin），
  其内部 IFC 已垂直（V1 证），故 child 内部不需改，仅 child 在父中的位置 + 父 content-size。

关键：**layout 期算 container content-size = Σ/max**（非 taffy 的 max/Σ），父用此值 sizing
（propagation 一致），消除 postprocess 的 sizing 矛盾。

## Phase 1（dormant，零回归，R885/R1350 模式）

`crates/layout-engine/src/vertical_block_flow.rs` 新模块：
- `VerticalBlockFlowLayout { child_offsets: Vec<(f32,f32)>, content_width: f32, content_height: f32 }`
- `compute_vertical_block_flow(children_outer_sizes: &[(f32,f32)], wm: WritingModeValue, gap: f32) -> VerticalBlockFlowLayout`
- 单测对 R1541 V2/V3 ground truth 验证：
  - V2 (rl, [50×150, 50×150], gap 0)：content_width=100, content_height=150, offsets=[(50,0),(0,0)]
  - V3 (lr, 同)：offsets=[(0,0),(50,0)]
- `#[allow(dead_code)]` dormant（Phase 2 wiring 待定）。
- 验收：fmt/clippy `-D warnings`/make test 全绿（+新测试），零生产行为变更（product-smoke welcome 字节一致）。

## Phase 2（wiring，多 session，紧 gate）

把 `compute_vertical_block_flow` 接入 engine.rs block-layout 主循环的 vertical 分支：
definite-wm 容器 + block-level 子 → 用本函数替 taffy 默认堆叠 + 重算 container content-size +
parent mark_dirty 重 layout（解 propagation）。env gate `ZW_VERTICAL_BLOCK_FLOW` + writing-modes
reftest-oracle 全量 A/B 守 net≥0（R1043/R1047 先例警示，预期需多轮调 gate）。

## 风险

- child 内部若含 nested block-flow 或 abspos，位置重算可能不一致（须 gate 排除复杂子树）。
- parent propagation 须 mark_dirty + 二次 layout（性能 + 一致性，R1013 two-pass 模式可参）。
- writing-modes reftest 多含 text-rotation / bidi / emphasis，block-flow-only fix 可能仍
  net-negative（须 A/B 实证，net<0 则回退记 entangled）。
