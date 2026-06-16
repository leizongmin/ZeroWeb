# 设计草图：flex/grid 容器两趟固有宽度布局（shrink-to-fit）

**版本**：v0.1（设计草案，待实施）
**日期**：2026-06-16
**状态**：设计完成，待分轮实施
**关联**：rendering-compat master.md R180/R180d； unlocks collapsed-item(chr20.5%)/table-grid-item-003(chr29%)/child-border-box-and-max-content(1.52%×2)/flex-container-max/min-content/css-flexbox-row

---

## 0. 执行摘要

- **一句话目标**：让 `width:auto`/`width:max-content` 的 flex/grid 容器（inline-float/inline-grid/float:flex/float:grid）正确 shrink-to-fit 到内容固有宽度，而非被 taffy 拉伸到可用宽度（800px）。
- **核心问题**：post-hoc 收缩（R129/R134/R138/R180 谱系）对 block/inline-block 有效（子元素有独立宽度不增长），但对 flex/grid **无效**——因为 taffy 在 definite(800) 可用宽度下布局时，flex item（`flex:1`）和 grid item（auto/1fr track）**已增长填满容器**，post-hoc 读到的 `child.width` 是增长后的值（772px），无法据此收缩且无法 re-layout 已增长子元素。
- **推荐方案**：在 `compute()` 的 taffy 首趟布局与 `extract_layout` 之间插入「固有宽度测量趟」——对 shrink 候选容器，用 `AvailableSpace::MaxContent` 重测其内容固有宽度，回写到 taffy 节点的 `size.width`，`mark_dirty` 后重跑 `compute_layout_with_measure`，再提取。
- **首个落地步骤**：实现 `measure_container_intrinsic_width()` 工具（从子元素 base size 计算 flex 行和/列固有宽度，纯计算无布局副作用）+ 单元测试，**不接线**（先证明测量正确）。

---

## 1. 现状与根因（已实证，见 master.md R180/R180d）

| 机制 | 证据 |
|------|------|
| taffy 给 flex grid item 拉伸宽度 | FLEXSHRINK 探针 `child.width=774 child_w=[0,772]`（collapsed-item）；TBLW_DBG `table box.width=278 parent=Grid`（table-grid-item-003） |
| post-hoc 收缩读增长值失效 | R129 float-shrink `content_max_w=max(child.width)` 对 flex:1 item 读到 772→不收缩 |
| max-content 被解析为 0 | `computed.rs:68` `MinContent|MaxContent => 0.0` → 容器塌缩 |
| converter 映射 | `Table/InlineFlex/InlineGrid => taffy::Block`（converter/mod.rs:254）；flex/grid item 在 taffy 内由其 flex/grid 算法布局 |

**关键约束**：`compute()` 流程 = build taffy tree → `taffy.compute_layout(Definite(viewport))` → `extract_layout` → 后处理（steps 4-12）。taffy 树在 compute() 末尾移入 `cached_state`。post-hoc 后处理只能改 `LayoutBox`，**无法重跑 taffy**，故 post-hoc 修 flex/grid 宽度后子元素仍处于增长态→溢出/错位。

## 2. 目标状态：两趟布局

```
compute():
  1. build taffy tree（含 measure_text_content 回调）
  2. taffy.compute_layout_with_measure(root, Definite(viewport))   # 首趟：定宽
  3. extract_layout → root_box（临时）
  4. 【新增】identify_shrink_candidates(root_box, styles)
       → Vec<(dom_id, intrinsic_width)>
       intrinsic_width = measure_container_intrinsic_width(...)
  5. 【新增】若候选非空：
       for (dom_id, w) in candidates:
           taffy.set_size(dom_to_taffy[dom_id], width=w)  # 设 definite 固有宽度
           taffy.mark_dirty(...)
       taffy.compute_layout_with_measure(root, Definite(viewport))  # 第二趟
       root_box = extract_layout(...)  # 重新提取
  6. 后处理 steps 4-12（不变）
```

第二趟：候选容器宽度 = 固有宽度（definite），其 flex/grid item 在该宽度下重新分布（flex:1 无 free space→保持 base size；grid track 收缩）。

## 3. `measure_container_intrinsic_width` 设计

输入：`box_node: &LayoutBox, styles`（首趟提取后的 LayoutBox，子元素已布局但可能增长）。
**不用** `child.width`（增长值）；改读 **base size**：

- **flex item base size**：`style.flex_basis`（Px）→ 用之；否则 `style.width`（Px）→ 用之；否则读子元素 intrinsic content（难，先用 0 占位，下轮补）。
- **flex 行容器固有宽度** = Σ item base size + gaps + 容器 padding/border。
- **flex 列容器固有高度** = Σ（同上，主轴换列）。
- **grid item base size**：item 的 max-content（item 含显式宽子元素→该子元素宽 + item frame；复用增强版 `block_max_content_width`，须修正「叶节点显式宽」返回 0 的 bug——见下）。
- **grid 固有宽度** = Σ track base size（column flow）/ max（row flow）。

**已知子问题**：`table_shrink::block_max_content_width` 对「叶 block 显式 width」返回 0（只读子元素不读自身 width）。须增强：当 box 无更宽子元素时，回退到 `box` 自身 `style.width`（Px）。此增强须验证不回归 R138 table-shrink（table 的 block 子元素若显式宽会改变收缩测量）——独立单测 + css-tables 全量回归。

## 4. 候选识别（shrink candidates）

仅对**应 shrink-to-fit 的容器**触发，避免回归填满宽度的正常 block：

| 容器 | 触发条件 | 当前处理 | 本设计 |
|------|----------|----------|--------|
| `display:flex` block | width:auto | 填满 800（正确，block 语义）| **不触发** |
| `display:inline-flex` | width:auto | taffy 拉伸 800 | 触发两趟 |
| `float:flex` | width:auto | R129 post-hoc（flex 失效）| 触发两趟（替代 R129 flex 路径）|
| `display:grid` block | width:auto | 填满（正确）| **不触发** |
| `display:inline-grid` | width:auto | 拉伸 | 触发 |
| `float:grid` | width:auto | R129（失效）| 触发 |
| 任意 flex/grid | `width:max-content/min-content` | computed.rs:68→0 塌缩 | 触发（且须先修 computed.rs:68 保留信号，见 §6.3）|

守卫：仅当 `intrinsic_width < current_width - 0.5` 才设宽（避免对已正确窄容器无谓重算）。

## 5. 风险与缓解

| 风险 | 等级 | 缓解 |
|------|------|------|
| 第二趟 taffy 重算性能（每个 shrink 候选触发全树重算）| 中 | 候选通常少（页面中 inline-flex/float flex 不多）；可按子树 dirty 而非全树；先正确再优化 |
| base size 测量不准（flex-basis:auto + 内容宽难测）→ 过/欠收缩 | 高 | 守卫 `intrinsic < current`；对测不准的 item 用 0→容器不收缩（no-op，安全）；逐 reftest 验证 |
| R138 block_max_content_width 增强回归 css-tables | 中 | 独立 commit + css-tables 51 用例全量回归门禁 |
| computed.rs:68 max-content→0 改保留信号回归 R97 标的 8 通过用例 | 中 | 先单独验证 8 用例是否依赖→0 行为 |
| 第二趟与后处理（adjust_float/adjust_table/multicol）交互 | 高 | 第二趟仅设 taffy 宽度+重算，后处理不变；全量 reftest 门禁（434/490 不得回退）|

## 6. 实施顺序（分轮，每轮独立验证 + commit）

1. **Round A（零布局风险）**：`measure_container_intrinsic_width()` + `block_max_content_width` 叶节点增强 + 单元测试。不接线。验证：make test 全绿。
2. **Round B**：`identify_shrink_candidates`（仅 inline-flex/inline-grid width:auto）+ 接入 compute() 两趟。验证：全量 reftest 434/490 不回退 + collapsed-item/baseline-align-self 改善。
3. **Round C**：扩展到 float:flex/float:grid（替代 R129 flex 失效路径）。验证同上 + floats-clear 不回退。
4. **Round D**：修 `computed.rs:68` max-content/min-content 保留信号 + 触发两趟。验证 child-border-box-and-max-content/flex-container-max-content。先验证 R97 的 8 用例。

## 7. 验证标准

- 全量上游 reftest 同源 ≥ 434/490（零回退硬门禁）。
- chromium Oracle：collapsed-item-horiz-001 chr 20.5%→<5%；table-grid-item-003 29%→<10%；child-border-box-and-max-content 1.52%→PASS。
- 单元测试覆盖 `measure_container_intrinsic_width`（flex row/col、grid column/row、显式宽子元素、flex-basis 优先级）。
- clippy/fmt/make test 全绿。

## 8. 不在本设计范围

- multicol 列感知 IFC 碎片化（R131，独立里程碑）。
- 表单控件原生外观渲染（semi-replaced paint gap，R180b）。
- taffy 0.7 升级（若上游修复 auto track 扩展可整体替代本设计，须评估）。
