# 刀 F·neutral 边界帧通用化批（2026-11-17）：R5018 implicit keyframe 全属性通用臂 + compositor 双端点清偿

> 命名注记：并行流 evidence 已将「刀 F」规划为 mask-border 支持门+插值批
>（1,791F 靶面，`9693736da` docs-only）。本刀与其靶面不同、先手在跑，记为
> **刀 F·neutral**；mask-border 批保留原「刀 F」称谓，认领归并行流。

## 交付面（2 面 + 3 钉）

1. **通用 underlying 臂**（`crates/engine/src/animation.rs`）：R5018
   `underlying_individual_value` 原为逐属性手工臂（transform/clip-path/
   rule-width/rule-inset/offset 簇/individual 三属性）——shape-margin、
   shape-image-threshold、masking 全族等不在覆盖内，None 回退 = 边界帧缺键、
   插值段零输出或初值近似。新增 `generic_underlying_value`：逐属性臂 miss 后
   走 `serialize_computed_property_with_sibling(&style, name, None)`（gCS
   同源串化、E5 keyword 解析同通道——css-animations-1 §keyframes：缺失端点
   帧 = neutral key，即 underlying value）；空串（未实现属性）返 None 保持
   既有回退。三属性 individual 兜底语义不变（声明缺席仍回 0px/0deg/1 初值）。
2. **compositor 双端点清偿**（`animation/compositor_export.rs`）：from-only
   opacity 100% 边界帧经通用臂补 underlying（默认 1）——缺口期「导出拒绝
   （冻结同视觉）」保守形升级为双端点 [from, underlying] 正常导出（真淡入，
   规范语义）；钉测 `from_only_opacity_is_rejected` →
   `from_only_opacity_fills_underlying_and_exports` 断言重写。
3. **钉测 3 个**：`test_neutral_boundary_generic_shape_margin`（from-only
   边界帧补 '10px'、at(0.5) blend = '30.00px' 端到端）、
   `test_underlying_generic_number_and_fallback`（number 形 '0.6' 直出 +
   默认 '0px' 回填 + 未实现属性 None 回退）+ compositor 断言重写。
   engine lib **4,125/0F**、clippy 本刀文件过滤零告警、fmt 净。

## 权威账（bin-cssa-f1 `064154a2`，五 corpus；基线 gaps=v4 e4 账 3769、
## 其余=v2 e2 账——v3 SET-IDENTICAL、v4 仅动 rule-inset TRANS 通道）

| corpus | 基线 | f1 | 增量 |
|---|---|---|---|
| css-gaps | 3769 | **3792** | +23 |
| css-multicol | 753 | **757** | +4 |
| css-typed-om | 9873 | 9873 | 0 |
| css-shapes | 5088 | **5108** | +20 |
| css-masking | 2362 | 2362 | 0 |
| **合计** | 21845 | **21892** | **+47** |

**PAGE-ONLY 零容忍对账：NEW-PASS +47 / GONE-PASS 0 / NEW-FAIL 0 / 五 corpus
页集合全等**——纯增益零回归零丢页。Timeout 面持位（shapes/masking 各 1，
与 v2 恒等）。product-smoke struct-check PASS（0 issues）。

## 增益分布定性

- gaps +23 全落 rule-color/rule-width 列表族 **from neutral** 端点
  （rule-color-interpolation-repeaters 12 + row-rule-color 7 +
  multiple-values 4）——通用臂补键后列表插值臂真跑。
- shapes +20 全落 neutral 端点（shape-margin 5、shape-image-threshold 5、
  shape-outside 三页 10）——刀 E 10 GONE 定谳的 neutral 揭穿面回补兑现。
- multicol +4 同 rule-color neutral 族。
- masking 持位：其 neutral/clip 形揭穿账已在刀 E v2 计入；剩余缺口归
  mask-border 大刀（并行流认领）。
- `rule-color-interpolation-repeaters-001` CSSA at(1.5) 一案 f1 vs 刀 D 基线
  呈 NEW-FAIL 形——即刀 E v2 已揭穿的同一 vacuous 假绿（`repeat(auto,…)`
  上游 typo 非 parity 目标，刀 B3 外证闭环持位）；f1-gaps-vs-v4 对比零翻转
  证实非本刀新增位移。

## Provenance 与过程纪要

- A 基线 gaps=`results-cssa-e4-B-e1`（bin-cssa-e4 90762cf3 单腿）、其余=
  `results-cssa-e2-full`（bin-cssa-e2 b69a15ce）；B=`results-cssa-f1-full`
  （bin-cssa-f1 064154a2，shim 指纹 `zw-ruleinset-unitmix-2026-11-17` =1）。
  构建窗口 es_module s419 peer WIP 在树（同刀 E 形，抵消假设与 md5/指纹
  归档为准）。链 `launch-cssa-f1-full.sh`（transient service
  zw-r4-cssa-f1，cd $ROOT 内层，GATE-DONE 07:46:55 五腿全 rc=0）。
- 对账脚本双基线修正记录：初跑误用 `results-cssa-e1-A-base`（仅含 gaps 且
  旧 schema）——正确 A 侧 = 刀 D 归档 `results-transall-knife-d-base`
  （evidence「B0=knife-d」口径）+ v4/v2 增量基线；对比器 new/gone 标签同步
  订正。A 基线 JSON = list-of-[path, subtests] 旧 schema、f1/v2 = dict 新
  schema，loader 双形态兼容。
- 修改范围：animation.rs（通用臂 +2 钉）、compositor_export.rs（断言重写），
  共 2 文件；对 offset-path verbatim gCS 通道零触及（通用臂不影响 verbatim
  直出路径的既有离散语义）。

## 刀后残面队列（更新）

- **mask-border 支持门+插值批**（并行流认领「刀 F」称谓）：四插值页
  1,791F 前置支持门 + 三臂插值面——见 `9693736da` evidence 追加节。
- clip-path 跨单位 calc blend（lerp_len 同单位限定；clip-path/shape-outside
  有 canonicalize 通道可进 calc 混合，**offset-path verbatim 通道须保持
  离散**——emit calc 中间形会直出 gCS 与 chrome 求值形不符，需分通道处理）。
- inherit-keyframe 父样式通道（FIXME(kf-inherit) 池 40F/页残差）。
- masking 剩余 neutral/mask 簇缺口（并入 mask-border 批或独立小刀）。
- calc 端点形 quirk（TRANS 通道残余形态）。
