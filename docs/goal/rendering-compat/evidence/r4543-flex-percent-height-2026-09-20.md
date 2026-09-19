# R4543 证据：flex item 子元素百分比高度塌 0——taffy 层排除、ZW 集成面定位（P 轮 0 net code）

- 日期：2026-09-20（R4543 P 轮——R4542 下轮方向③，+5 条 taffy 层基线锚测试）
- 谱系：R4541 记账的 flex item 子元素不绘制独立 bug（border-shape-overflow-child-clip 1.64% 的真实病因）。

## 最小探针归因链（逐层收窄）

1. flex 容器 + item（黄底）+ 块级子 div（绿底，100%×100%）→ **子不绘制**；同构 block
   容器（非 flex）→ 子正常铺满（flex1/flex2 探针）。
2. 子固定 50×50 + 文本 → 正常绘制（flex3）→ paint 遍历无缺，**纯尺寸解析问题**。
3. 子 `width:100%; height:50px` → 正常（flex5，宽 % 解析 ✓）；子 `width:50px;
   height:100%` → 消失（flex6）→ **百分比 HEIGHT 塌 0**（LAYOUT_DUMP 实证：item
   h=100 definite、child w=50 ✓ h=0）。
4. **容器 height:auto（indefinite）+ item definite → 塌 0**；容器 height:100px
   definite → 正常（flex7）→ 分歧点 = 容器 cross 尺寸 indefinite。

## taffy 层排除（关键定谳）

纯 taffy API（crates/taffy-local，`TaffyTree` 直接构树，非 ZW 管线）复现同构四形态 +
measure 管线变体——**全部正确解析 100**（容器 auto + item definite 也通过）。
converter 侧 `convert_length_to_dimension` 对 `height: 100%` → `Dimension::percent(1.0)`
映射正确。⇒ **bug 不在 taffy、不在 converter 的高度映射，在 ZW 的 taffy 树构建/
后处理链（tree.rs 集成面）对「flex 容器 cross indefinite + item definite height」
形态的处理**（疑点：flex 子树的 taffy style 覆写或 extract 侧高度改写）。

## 本轮产出（资产化）

- `crates/layout-engine/tests/r4543_flex_percent_height_baseline.rs`：5 条 taffy 层
  基线锚（基线形态/块父控制组/容器 definite/item stretch/measure 管线）——当前全绿，
  双重作用：①锚定 taffy fork 行为（fork 改动致转红 = taffy 层回归先修）；②下轮
  ZW 集成面修复的对照组（修复后补「ZW 集成面」断言测试）。

## 下轮方向（修复路径已铺好）

1. **ZW 集成面定位**：在 tree.rs 构建/extract 链对探针页 dump flex 子树的 taffy
   Style（item 的 size.height、align_self 与容器的 align_items）与最终 LayoutBox
   高度来源，定位覆盖点并修复（预期窄修）；
2. 修复后 border-shape-overflow-child-clip（1.64%）预期同步翻绿（flex 子 bug 是其
   唯一残差）；
3. 或 corner-shape 余 6 fail 深域 / 资产激活新红二案勘察 / 守成轮。
