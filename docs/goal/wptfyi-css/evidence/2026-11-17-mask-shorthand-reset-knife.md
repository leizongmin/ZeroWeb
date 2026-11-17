# mask 简写刀收口：css-masking-1 §3.10 reset 全投影 + 六长手站 + 裸 0 消歧 + inherit 通道五臂

日期：2026-11-17
承接：[mask-border 刀](2026-11-17-mask-border-knife.md) 挂账 FIXME(mask-shorthand-reset)（A/B 新暴露 5F：mask 简写声明未重置五 mask-border-* 长手）。
批次 11 文件：css-parser 1 + style-system 10（含 untracked shorthand/tests/mask.rs）。

## 现象与基线

- `mask-shorthand-subproperties-reset.html`：mask 简写无展开（registry 无 mask 简写臂），§3.10 reset 语义全缺；mask-border 刀后该页 5F（五 mask-border-* 长手 reset 读回旧值）。
- `inheritance.sub.html`：mask-border-* 五长手显式 `inherit` 关键字无通道（inherit_property 无臂 → fallback false 无操作），does-not-inherit 模板 target inherit 步读回旧值即 Fail。基线 8P/28F。
- mask-* 六长手（position/size/repeat/clip/origin/composite，css-masking-1 §5.3–5.6）无属性站。
- `border-image-outset` 裸 0 值形歧义：初值站 Number(0) 串化 "0"，apply("0") 走 parse_length 先收 Px(0) 串化 "0px"——reset 页 initialValue 与 reset 读回自比较恒等驱动，"0px" 形判 Fail。

## 实现

- css-parser `values/parse_extended_border.rs`：`parse_border_image_outset` 裸 0 双文法消歧（css-values-4 §5.2）——unitless 0 收 `<number>` 槽（判别器 `token.parse::<f32>() == Ok(0.0)`，"0px" f32 parse 失败自然区分；带单位 "0px" 与 calc 零叶仍 Length）。
- style-system 六长手站：`property/types.rs` PropertyValue 六臂；`types/image.rs` MaskGeometryBox/MaskClip/MaskComposite 三枚举（fill/stroke/view-box/no-clip/add/subtract/intersect/exclude，与 background 侧独立）；`computed_style.rs` 六字段；`default_impl.rs` + `registry.rs` 初值（position 0% 0%、size auto、repeat repeat、clip/origin border-box、composite add）+ known 清单 +8；`inherit.rs` apply_initial_value 六 reset 臂。
- `shorthand/mod.rs`：`expand_mask`（单值 `<mask-reference>` 形 13 投影 = mask-image 保形 + mask 七长手初值 + mask-border-* 五长手初值；组合形/多层形 FIXME(mask-shorthand-grammar) 拒收）。
- `inherit.rs` inherit_property 五臂：mask-border-source/slice/width/repeat/outset 显式 `inherit` 拷父（css-cascade-4 §inherit 对任何属性强制继承；transform 臂同口径先例）。
- 钉测：`shorthand/tests/mask.rs` 3 测（单 url 13 投影名序与初值、none/渐变接受、组合/多层拒收）；`tests/basic.rs` probe6 转正 `test_mask_shorthand_reset_cascade_wins`（18 声明照 cascade 真实 expand 输出序 → 断言五 reset 胜者 + mask-image 保形）。

## 5F 归因链（七层假说逐层实证排除）

parse 层保形 → tokenizer 保形 → expand 13 声明顺序正确 → apply 探测生效 → matching 无过滤 → canon_random_inputs 零改动 → cascade 胜者正确——真因两件套：

1. **裸 0 值形歧义**（上节第 4 点）：修复一收口。
2. **inherit_property 无 mask-border 臂**：修复二收口。apply_initial_value（置初值）语义本正确，初值臂=未声明非继承填充/initial/unset/revert 通道；显式 `inherit` 关键字走 inherit_property（与 apply_initial_value 是两个函数，无 parent 参数）。

诊断陷阱注记：探针页 `assert_true(false, 值)` 形态的 Fail message 即探针数据——「探针红」≠「被测行为红」。

## A/B 对账（bin-mm-b0 vs bin-mm-b1，2026-11-17 22:45–22:50）

五 corpus（reconcile-five-corpus.py）：

```
[css-gaps]      b0=3838  b1=3838  delta=+0    | F2P=0 P2F=0 | pages 75=75
[css-multicol]  b0=943   b1=943   delta=+0    | F2P=0 P2F=0 | pages 91=91
[css-typed-om]  b0=9947  b1=9947  delta=+0    | F2P=0 P2F=0 | pages 359=359
[css-shapes]    b0=5125  b1=5125  delta=+0    | F2P=0 P2F=0 | pages 145=145
[css-masking]   b0=3687  b1=4011  delta=+324  | F2P=325 P2F=3 | NEW P2 | pages 61=61
[TOTAL]         b0=23540 b1=23864 delta=+324
```

- 判据达成：css-masking F2P=325 / P2F=3（三例全归既有挂账面，见下），余四 corpus SET-IDENTICAL 全零位移，page-only 零容忍满足（NEW P=2 在 reset 页）。
- **P2F=3 归因**（mask-border-source-interpolation.html，Web Animations 通道 from [inherit] at -0.3/0/0.3 三例）：b0 双侧 inherit 均无臂同落 none 互证＝**假绿**；b1 期望侧经本刀 inherit_property 五臂正确解析父链 `url(green.png)`，实际侧 WAAPI/js_anim 通道父链不可达传 None 仍落 none——即 K1 刀已注记的 **FIXME(kf-inherit-waapi)** 面（CLAIM-knife-kf-inherit-waapi 已认领）。假绿揭真红挂账，非本刀行为回归；JSON 逐条核对 b0=Pass/b1=Fail 带 message 实证。
- `inheritance.sub.html` 8P/28F → **13P/23F**（+5P：mask-border-* 五长手 does-not-inherit inherit 步）。
- `mask-shorthand-subproperties-reset.html` 全 Pass（runner 归并 7 断言 / debug 形态 13 断言，两形态均 0F；mask-border 刀 A 腿整页 Timeout 死态→页活跑完）。
- 复证：本树 release bin（23:01）css-masking Fail=2336 与 mm-b1 Fail=2336 同账。
- css-backgrounds 语料（bin-mm-b1 vs 本树 bin）：4246F vs 4246F，日志 diff **逐字节零差**——裸 0 修复在 css-backgrounds 中性（其收益面 = css-masking reset 页 outset 断言）。
- B1 bin 含 regprop 刀面（engine registered_properties，同树在途）；css-masking 断言面不触发 registerProperty 通道，中性。
- style-system 常驻 2559 passed / 0 failed（含新钉 4 测）；fmt/clippy 净。

## 残面挂账

- **css-backgrounds parsing/border-image-outset-computed** calc 空格容错缺：`parse_border_image_outset` split_whitespace 把 `calc(0.5em + 10px)` 拆碎 → supported false（2 断言：'0 calc(…)' 钉面 + 'calc(10 + (sign(…)))' 叠 sign() 通道）。双腿同红（A/B 零位移实证既有），FIXME(border-image-calc-tokenize)。
- CSS.supports('mask', …) false：decl_supported 尾部无 mask 简写臂（探针实证）。
- mask-* 六长手 apply 写值通道 FIXME(mask-longhands-unimplemented)（仅初值/reset 通道生效，写值不生效）。
- mask 简写组合形/多层形 FIXME(mask-shorthand-grammar)；mask-valid.sub 多层形红（2336F 大头）。
- mask-border composition accumulate 通道、allow-discrete transition-behavior FIXME(mask-border-discrete)（承接 mask-border 刀）。
- inherit_property 同段 text-shadow/box-shadow/clip-path 等先例臂疑似同缺陷（既有红，未开刀）。

## 运行注记

mask-m 流首次发射（22:17）死于 BUILD-FAIL：regprop 刀 stash 时序造成 js_dom_bridge.rs mod 声明在树而 registered_properties.rs 不在（E0583），残骸污染共享树；regprop 流恢复刀面（registered_properties.rs 落树）后二次发射成功（22:45–22:50）。共享树 stash 循环的死因几乎全是「别人的半成品进了自己的 build」——发射前 mod 声明/文件配对完整性检查值得进 A/B 脚本前置位。
