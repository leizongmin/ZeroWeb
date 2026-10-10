# multicol-2 域收口立项评估（R5031 扫描——待用户拍板）

**状态**: 草案（待用户立项拍板）
**日期**: 2026-10-10（R5031）
**驱动**: fail-list 148 案含 `multicol` 路径（全量 16,594 案 = 0.89%，最大单一语义域）

## 1. 现状盘点

ZW 已有 multicol 机制（layout-engine 4,100+ 行）：
- `multicol.rs`（2,382 行）：`column-count/width/gap` 解析与列宽计算、`column-fill:
  auto` 顺序填充 + 列高限制（`column_height_limit`，容器明确高时每列上限）、
  balancing、spanner（直接子 / 嵌套 wrapper / bordered region fragments 871 行）、
  abspos 后代、嵌套 multicol。
- `multicol_balancing.rs`（245 行）：列均衡。
- 已收敛面：`multicol-rule-*` 大体绿、基础 balancing/fragmentation 多数绿。

## 2. 失败带结构（fail-list 148 案分族）

| 族 | 案数 | 根因域 | 工程量 |
|---|---|---|---|
| 基础 multicol（css-multicol/multicol-\*） | 26 | 列内行盒/fragmentation 精度 | 中 |
| `column-height`（css-multicol-2 属性，全库零解析零实现） | 14 | **新属性**：parser → computed → multicol 布局列高上限（区别于容器高度语义） | 中 |
| spanner 系（multicol-span-\* / spanner-\*） | 15+ | spanner 定位/跨列精度 | 中-深 |
| multicol-fill / breaking / width / nested / rule | 23 | 各属性边角 | 中 |
| fixedpos/abspos in multicol | 3-9 | containing block + 分片交互 | 深 |
| floats-clear-multicol（含 R5026 门控掩体案） | 4 | clear 与列分片交互 | 中 |
| 杂（contain-size 等经 multicol 路径） | 3+ | 域间交互 | 不定 |

## 3. 立项建议（三片，独立可回滚）

- **S1 column-height 属性面**（14 案）：css-parser `parse_column_height` +
  style-system computed + `column_height_limit` 消费扩展（属性语义：列内容高上限，
  区别于容器高度；与 `column-fill:auto` 组合驱动列换列）。**风险低**（新属性增量，
  零既有行为变更），建议先做。
- **S2 基础 multicol 行盒精度**（26 案）：列内 IFC 与 fragmentation 的行盒放置——
  与 R5026-5029 的 float band / auto-height 机制域相邻（floats-clear-multicol 掩体
  案在此解），可复用其取证方法（chromium headless oracle 逐案）。
- **S3 spanner/fixedpos 深域**（15-25 案）：spanner 定位精度 + 分片内 abspos CB。
  深结构域，**单独评估**。

预期合计收口 60-90 案（148 中约 20-30 案属 S3 深域或上游同样失败，不计入）。

## 4. 待拍板点

1. S1/S2 是否立项（S3 深域建议缓议）；
2. S1 若立项可并入常规轻量修复轮（无需独立 RFC）；
3. S2 的 oracle 取证法沿用 R5026-5030（chromium headless 逐案）。
