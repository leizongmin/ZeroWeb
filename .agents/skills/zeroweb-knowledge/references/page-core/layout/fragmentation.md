# 多列、碎片化与分页

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查多列 block 分片、inline 行分列、nested spanner 装饰段与 Print 分页。列分配与绘制裁剪共同实现视觉碎片化，不等于拆成独立 DOM/LayoutBox；表格经后处理求解尺寸后仍参与这条分片链。

## Gotchas
- block column breaking 重用同一 child_idx，以 fragment_y_offset 取不同垂直窗口，绘制要按每片偏移与裁剪执行；不能只看 child 的单个 x/y 判断多列可见几何。 (`crates/layout-engine/src/multicol.rs`, `crates/engine/src/paint/painter/mod.rs`)
- inline 列分配有两套余量规则：`fragment_lines_into_columns` 留在末列，生产 inline auto 存储调用 overflow 变体创建额外列；不能把基本纯函数的规则当生产结论。 (`crates/layout-engine/src/inline/column_fragmentation_flow.rs`, `crates/layout-engine/src/inline_finalization.rs`)
- overflow 变体只有整行能装入空列时才继续创建列，超高单行留当前列，防止不停创建空列；列预算不存在、非正或 filled-heights 长度不匹配则返回空分配让 caller 回退。 (`crates/layout-engine/src/inline/column_fragmentation_flow.rs`)
- inline multicol balance 仍保存 font/line-height override，但不保存列分布行盒，让 paint 按 multicol_info 重跑；auto 路径预算按 definite height、max-height、content_height 优先取值。 (`crates/layout-engine/src/inline_finalization.rs`)
- `ColumnFragmentationContext` 和 flow 模块的“零生产读取”头注释已经过时，当前终化存在 overflow 调用；`multicol_balancing` 的闭式经验模型则没有对应生产调用，不能混同。 (`crates/layout-engine/src/inline/column_fragmentation.rs`, `crates/layout-engine/src/inline_finalization.rs`, `crates/layout-engine/src/multicol_balancing.rs`)
- nested spanner 有框 wrapper 的首区域拥有顶边、末区域拥有底边，侧边与背景按 region×column cell 裁剪；非零圆角回落单盒模型，不代表完整曲线分片已支持。 (`crates/layout-engine/src/multicol/bordered_region_fragments.rs`)
- count=1 spanner 装饰段仅处理 sequential-fill 且非嵌套 multicol wrapper；它不生成 child fragment、不改 column_span_offsets，与多列 wrapper 路径不是同一重定位规则。 (`crates/layout-engine/src/multicol/bordered_region_fragments.rs`)
- Print 分页只在块流根的一层直接 in-flow block 子上累加 gap，嵌套 forced break 近似提升整单元，oversized 单元真跨页拆分尚未实现，不能宣称表格已有完整跨页 row/header 分片。 (`crates/layout-engine/src/print_pagination.rs`)

## Architecture
- spanner 区域 inline 平衡先于 multicol 定位，多列容器收缩后先拉回后续兄弟，再按最新子底回收 auto-height 祖先，交换顺序会留旧高度。 (`crates/layout-engine/src/engine.rs`)
- multicol 兄弟 restack 用最终 gap 模型而不保存跨 pass delta；合法 clearance/relative 间隙跳过，避免重测与终化重复消费陈旧位移。 (`crates/layout-engine/src/multicol.rs`)
- nested spanner 段矩形与子 fragment 的位置/clip 都是 wrapper border-box 相对坐标，painter 直接消费；通用 content-band 裁剪不可替换 cell 裁剪。 (`crates/layout-engine/src/multicol/bordered_region_fragments.rs`, `crates/layout-engine/src/types/mod.rs`)
- Print 根布局宽取 page width 减水平 margin，Screen 仍取 viewport；分页高与页序列共享页尺寸和 margin 退化规则，内容区不足 1px 回退零 margin。 (`crates/layout-engine/src/print_pagination.rs`, `crates/layout-engine/src/engine/tests/r2013_print_layout_width_tests.rs`)

## Decisions
- `multicol_balancing` 闭式末区域模型明确保留 L 型 outlier 与 overflow 容器总高未解；其注释提出 min-space-shortage 迭代方向，不是已实现的完整 LayoutNG 平衡器。 (`crates/layout-engine/src/multicol_balancing.rs`)
- 打印分页依靠子绝对 extent 增大文档高而不硬扩 body.height，后代通过父相对坐标整体跟随；不能逐后代无条件加分页偏移。 (`crates/layout-engine/src/print_pagination.rs`)
