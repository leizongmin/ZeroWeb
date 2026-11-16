# 刀 F·neutral 隐式边界帧通用化（2026-11-17）：R5018 通用 gCS 串化臂 + compositor 双端点

## 交付面

1. **通用臂**（`animation.rs` 新 `generic_underlying_value`）：`underlying_individual_value`
   逐属性手工臂（transform/clip-path/rule-width 双列表/rule-inset 八槽/offset 簇/
   individual 三属性）miss 后，一律落 `serialize_computed_property_with_sibling`
   （E5 keyword 解析同通道）——css-animations-1 §keyframes neutral key = underlying
   value 的**全属性**兑现；空串（未实现属性）返 None 保持既有回退。
   individual 三属性命中定制臂即返回（既有初值近似语义保持）。
2. **compositor 导出双端点**（共享路径语义变化）：`from_only_opacity_is_rejected`
   缺口期保守形（100% 空帧不补 → 导出拒绝冻结）随通用臂自然翻转——
   `from_only_opacity_fills_underlying_and_exports` 新钉：双端点 `[0.0, 1.0]`
   正常导出（真淡入替代冻结；缺口期旧注释「只补 transform/individual」清偿）。

## A/B 终账（B=bin-cssa-f1 `064154a2`，shim 指纹沿用 zw-ruleinset-unitmix=1）

基线口径：gaps=v4 探针账（bin-cssa-e4 `90762cf3`，3769P）、其余=v2 全账
（bin-cssa-e2 `b69a15ce`；v3 SET-IDENTICAL、v4 仅动 rule-inset TRANS 通道不触及）。

| corpus | v2/v4 基线 | f1 | 增量 |
|---|---|---|---|
| css-gaps | 3769 | **3792** | **+23** |
| css-multicol | 753 | **757** | **+4** |
| css-typed-om | 9873 | 9873 | 0 持位 |
| css-shapes | 5088 | **5108** | **+20** |
| css-masking | 2362 | 2362 | 0 持位 |
| **合计** | 21829 | **21892** | **净 +47** |

**PAGE-ONLY 零容忍**：NEW-PASS +47 / GONE-PASS 0 / NEW-FAIL 0 / 五 corpus 页集合全等。
增益分布定性：gaps +23 全落 rule-color/rule-width 列表族 **from neutral** 端点
（row-rule-color-interpolation 7 + rule-color-repeaters 12 + multiple-values 4）；
shapes +20 落 shape-margin 等通用臂新覆盖属性的 neutral 面——即刀 E 揭穿的
neutral implicit boundary 缺口（GONE 定谳第 2 条 + 18 dn 揭穿面）的兑现。
typedom 持位符合预期（Typed OM 不走隐式边界帧）；masking 持位（clip-path
underlying 已在逐属性臂，剩余面非隐式边界帧形态）。
2 timeout（shape-outside-018 / mask-shorthand-subproperties-reset）= A 侧同形
持位 flake，非本刀新增。

## 与并行流刀 F（mask-border 批）撞名处置

并行流 evidence（9693736da）已认领「刀 F」靶面 = mask-border 家族支持门+插值
批（1,791F，css-parser/style-system/shim 跨站）。本刀记为**刀 F·neutral
通用化**以区分；mask-border 批继续按其规划执行，两刀靶面零交集
（本刀 animation.rs underlying 通道，彼刀 parser/registry/shim + interpolate
三臂），同文件编辑风险限 interpolate_property_value 邻域。

## Provenance

- B `bin-cssa-f1` 064154a2ca55c8e0a2f4affe5284e082（HEAD 9693736da 窗构建，
  es_module s419 peer WIP 在树，与刀 E 同形态——归档副本 md5 + 指纹为准）；
  腿链 `launch-cssa-f1-full.sh` / results-cssa-f1-full / transcript-cssa-f1-full*。
- 测试：engine lib **4,125/0F**（+2 钉 + 1 钉更新）、animation:: 135/0F；
  clippy 本刀文件零告警；fmt 两文件净。
- 对账脚本口径修正记录：flip（Fail→Pass）必须计入 NEW-PASS，A/B 独有键方向
  以 B 为基准——首跑脚本漏 flip 账显示全零，corpus 行计数（3792-3769=+23）
  与 SET 账交叉核出后修正，两口径最终对齐（+47 三方一致）。
