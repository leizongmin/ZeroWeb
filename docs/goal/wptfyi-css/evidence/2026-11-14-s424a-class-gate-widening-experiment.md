# s424a：class 属性族门拓宽实验——受控 A/B 定谳与回退

日期：2026-11-14。承接 s423 收窄（evidence 2026-11-14-g36-border-coord-s423.md）
在册挂账：anchor-name-mutation 15F（classList 增删不在放行族）。**本刀全程
未提交**：engine 三文件已回 HEAD 零残留，补丁与 A/B 工件存 `.acceptance/`
（s424a-class-gate-widening.patch / results-s424a-classattr /
results-s424a-postrevert）。

## 1. 刀口与实施

假设：s423 selector rect 臂放行族拓宽到 class 属性族（`classList.add/remove/
toggle`、`className` setter 经 shim `_zwSetAttr` 全值 `SetAttr/RemoveAttr`），
anchor-name-mutation 15F 同 turn 断言即可翻绿。

实施（回退前态）：

- `js_dom_bridge.rs` 新增 `is_class_attr_family`（SetAttr/RemoveAttr 及
  handle 变体 name=="class"）；
- `callbacks.rs` s423 臂 gate 改 `all(is_inline_style_family ||
  is_class_attr_family)`；
- part115 集成单测（class 写 → anchor() 解析新锚 140/150；add→remove 重放
  回落静态位）——engine lib 3959 全过，clippy 净。

## 2. 判别探针三连（机制定谳）

单测首跑 `#t.offsetTop=13`（静态流位）→ 三层判别：

1. `<style>` 块级联 ✓：基座类样式（`.sz` 22px）在视图槽读面生效
   （`collect_stylesheets_with_parsed_external(doc, …)` 收 doc 内表）。
2. anchor() 显式名 inset 视图槽解析需**定位 CB**：`apply_inset_walk`
   （anchor_pos.rs）首门 `cb_is_positioned = b.is_abspos_cb`——基座挂静态
   body 下整段跳过；包 `position:relative` 容器后单测全绿（corpus 页与
   s423 position-area 测试同形）。
3. 插曲：`format!("…('{sel>').{prop}")` 形（`'`+双格式槽组合）触发 rustc
   format 串解析误报（"expected alignment specifier after `:`"）——改字符
   串拼接绕开。

## 3. 受控 A/B（同 peer g38 在途基座，二进制 provenance 见 §5）

基座说明：两轮 corpus 二进制同树同 peer g38 在途态（style-system/layout-
engine 五文件 19:02:37 快照，见 §5 provenance）；对照组 = engine 门回退
HEAD，实验组 = 门拓宽。**账面基线注意**：g38 轮已在 master 定谳 g39 时代
12,061P JSON 系过期 provenance 二进制所测，提交态权威账 11,199P/1,654F
（HEAD+g38）——本节对比为「同基座 ± 门」受控面，不与提交态账混算。

**实验组（拓宽）**：265 页 11,199P/1,654F/4T。

- **+3 真增益**：anchor-position-dynamic-001 `#target 1` /
  anchor-position-dynamic-004 `.target 1`（两页唯一变异 =
  `container.classList.add('after')`，老门闭臂陈旧读 Fail → 视图真读
  Pass）/ anchor-fallback-invalidation `Correct invalidation…`
  （expected 40 got 50 → Pass）。三页均单条 classList、无 inline style
  写——老门从未开臂，翻转面即拓宽面。
- **−865 揭真（非回归）**：anchor-position-writing-modes-001 **864 案
  P→F**（页面全部变异 classList 增删 8 处、无 inline style 写，老门恒闭
  臂 1296P 全为快照陈旧读伪过——g36 轮金丝雀探针 zz-g36probe-wm001-drain
  已定谳该群伪过、s423 收窄注释在册真态缺口 ~864F）+ 金丝雀页 1 案 P→F
  同步翻转。新失败形态 "expected 200 but got 100" = 视图真读暴露
  writing-mode × anchor 组合未建模真实几何。
- **anchor-name-mutation 维持 15F（假设证伪）**：页面 `inflate()` 经
  `<template>.content.cloneNode(true)` + `append` 运行时创建元素——队列 =
  [结构性 Append, SetAttrOnHandle class ×N]，`all()` 谓词被结构性变异否决
  门仍闭；元素带 id 走 selector 身份（非 `__n{n}` handle 臂），快照无
  运行时创建元素 → 0。清偿需 selector 臂结构性放行（= s421 handle 臂的
  无门全重放语义 + 等值性实证，s423 收窄所求），挂账。

**对照组（回退）**：265 页 **12,061P/792F/4T**。定向复验：writing-modes-001
1296P/0F（伪过池复原）、dynamic-001 0P/1F（回基线）、anchor-name-mutation
15F（不变）。逐案 diff 与总账互证：拓宽净效应 = +3 − 865 = **−862**。

## 4. 回退决策

净 −862 违背 rally 指标方向（口径计通过数）；s423 收窄是已确立约定
（「门关闭回落快照旧行为不扩大暴露面」）；865 揭真转真增益的前提是
writing-mode × anchor 组合建模落地（master g38 段挂账「修复面另立 VRL
in-flow 布局刀」，anchor_pos.rs peer 活跃在途）。

处置：engine 三文件 `git checkout` 回 HEAD（零残留）；补丁留
`.acceptance/` 供组合建模落地后复用（届时揭真桶转真增益、本刀 +3 同回）。

## 5. 二进制 provenance

- 实验组二进制 19:03:33 链出（构建窗内 peer g38 五文件 19:02:37 批量落盘
  已扫入——同基座对照法证排除 peer 干扰：翻转面全部 class-门机械一致，
  与 peer 编辑域无重叠）。
- 对照组二进制 19:10:37 链出（同基座，engine 回退）。
- 对照组全量面貌（12,061P/792F）与 g39 时代过期二进制 JSON 恰同——peer
  在途 VRL in-flow WIP 复现该面貌，提交态权威账（11,199P）与两者均不同；
  三态关系已并入 §3 基线注意。
