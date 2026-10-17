# R5163 域核记录 — caption auto 宽在 size containment 下维持 fill（2026-10-17）

驱动案：`css/css-contain/contain-paint-size-001.html`（1.38%）+
`contain-strict-001.html`（1.38%）——同值对、共享同一 ref
（`contain-paint-size-001-ref.html`：绿条 206×40、无红）。

## 1. 域核

单测探针（r5163 probe，Ahem 环境 20px 字）：206 宽 fixed 表 + caption
（border green 1em、bg/color red、contain: paint size）：

- baseline（无 contain）：caption 206×86（content 高 46 = 一行文本）——内容驱动 ✓
- contain: paint size：caption **w=40** × h=40、content 高 0

块轴（高）size containment 生效（converter CIS-or-0 → content 高 0、只剩边框
40px），但**行内轴被错误塌 0**（40 = 左右边框）。chromium：caption 206×40 全绿。

## 2. 根因

converter containment 臂的 `block_fills_width` 分类集合
（Block/Flow/FlowRoot/ListItem/Flex/Grid − abspos，R2257）不含 `TableCaption`：
caption 的 auto 宽落入「非 block-level → CIS-or-0」臂塌 0。

CSS 2.2 §17.4.1：caption 宽「determined as if the caption were a block box child
of the table wrapper box」——fill 行为**非 content-based**；CSS Containment §3.1
size containment 只折叠 content 贡献，不改 fill-CB 解析（与 R2257 block 臂同语义）。
caption 由 taffy 作为 table 子布局（table.rs），fill = stretch 到表宽 206。

## 3. 实现

`converter/mod.rs`（kill-switch `ZW_CAPTION_CONTAIN_FILL=0`）：`block_fills_width`
并集追加 `DisplayValue::TableCaption`（同受 abspos 排除约束）。块轴 height 臂不动
（CIS-or-0 保持 content 高 0）。paint 裁剪（红字溢出不可见）无需改动——R2241
painter clip 对 caption 已生效，宽修复后双案即 0.00%。

## 4. 验证

- 单测 +3（r5163_caption_contain_fill_tests）：paint size 填表宽 / strict 同形 /
  无 containment baseline 保持内容驱动高。
- **OFF 反证**：`ZW_CAPTION_CONTAIN_FILL=0` 双案精确复红 1.38% / 1.38%（基线值）。
- 全 corpus 16,590：fail 1,121 → **1,119**（**+2 翻绿 0 新红 0 churn**，双案
  **0.00%**）；真通过-可信 9,802 → **9,804**。

## 5. 门禁记录

fmt 清；clippy workspace --all-targets -D warnings 0 告警；layout-engine lib 全测
**1958** 绿（1955+3）；make reftest **711/711**；make test **71 套全绿（21,058P）**；
product-smoke 6 页 struct PASS（welcome **17.07%** 与 R5162 同值）。
product-smoke-legacy 未跑：本修复仅在 `contain:size/strict` + caption + auto 宽
交点上生效，legacy 42 fixture 无 containment 使用面（按质量门禁按影响选择）。
bench-gate 定向 zero-layout-engine：ON 9 FAIL / OFF 10 FAIL（集合轮换：ON 页墙钟
first_paint 族、OFF 微基准 block/flex/grid/wide_tree 族，交集仅 first_paint×2）、
双跑 **NEW=0**、OFF 反更差 → 并行流（sibling `testharness-html-semantics` rustc
重载跑窗内）环境噪声族延续（R5149-R5162 同判），不 relax。

## 6. 限制/挂账

- 非 abspos 限制与 R2257 臂共享：abspos caption（罕见）仍 CIS-or-0（R4034b 语义）。
- 其余 table-internal 显示值（table-row-group/table-row/table-cell 等）未纳入
  fill 集合——无案驱动；若后续出现 cell/row + contain:size 的 fill 语义红案再议。
- converter 表宽双语义分叉（R5162 FIXME）、collapse 半宽 fidelity（R5162 FIXME）
  原样挂账。
