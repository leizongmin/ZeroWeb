# mask-border 刀：css-masking-1 §3.4–3.9 五长手插值/apply/串化

日期：2026-11-17
认领：`.acceptance/wptfyi-css/CLAIM-knife-mask-border.txt`
范围裁剪：composition 页（accumulate 通道）与本刀二期 FIXME(mask-border-discrete)（allow-discrete transition-behavior）不在此刀。

## 现象与基线

本地语料 `css/css-masking/animations/mask-border-*` 八页（outset/slice/width/【source】interpolation + slice-stability + 三 composition）基线 1791F/0P：style-system 无 mask-border-source/slice/width/outset/repeat 属性站，gCS 读回空、插值通道无臂。

## 实现（12 文件，style-system 七站 + engine 四面）

- style-system：
  - `property/computed_style.rs`：CSSStyle 五字段（复用 BorderImage*ComputedValue 类型——css-masking-1 §3.4–3.9 与 css-backgrounds-3 border-image 文法同构）。
  - `property/default_impl.rs` / `inherit.rs`：五初值（source=None、slice=0×4、width=auto×4、repeat=stretch×2、outset=0×4）与 reset 继承臂。
  - `property/apply_advanced.rs`：五 apply 臂（复用 parse_border_image_* + resolve_effect_length）；`border_image_source_to_computed`/`resolve_effect_length` pub 化（engine 动画通道同语义复用）；模块 `pub mod`（连带补 `apply_advanced_property_value` missing_docs）。
  - `shorthand/mod.rs`：`expand_border_image` 重构共享内核 + `ImageBorderShorthandSpec`；`mask-border` 简写（初值 slice "0"/width "auto"）+ 长手投影名单。
  - `matcher/supports.rs` / `property/registry.rs`：五长手 supports 门 + initial/known 清单。
- engine：
  - `js_dom_bridge/computed_style.rs`：六串化臂（五长手 + 简写 `mask_border_shorthand_to_css`，source=None→"none"）复用 border_image_*_to_css + box4 最小化折叠。
  - `js_dom_bridge/transition_registry.rs`：TRANSITIONABLE_PROPERTIES 60→65。
  - `animation.rs`：宽松臂五属性 + 严格臂一行委托（K1 契约：全局关键字端点通道前父链解析）+ apply_single_property 五臂。
  - `animation/clip_interp.rs`：`lerp_len` pub(super)。

## 插值契约（oracle 定谳，语料断言行直证）

- 四槽 TRBL 逐槽插值，**逐槽 type 对应一致（%↔%、number↔number、px↔px 等）才插值；任一槽失配 → 整条离散翻转**（expectFlip at<0.5→from、≥0.5→to）。修正点：早期认知「slice 跨域 %↔number 逐槽 lerp 单位随 from」是误读——语料四槽插值条每槽同构，no-interp 三条全为槽失配/fill 失配。
- slice 额外 fill 门：fill 不对称整条离散。
- 非负钳 0（slice/width/outset 三属性值域 [0,∞]，语料 Non-negative 注释 at:-0.3→'0'；inherit 端点外推 10px→2px at:1.5→'0px'）。
- width px↔% 走 calc 混合（`lerp_len` 同款 `calc(N% + Mpx)` 串形，刀 G inset 先例）。
- 裸无单位 0 双态歧义（css-values-4 §5.2：parser parse_length 先收 Px(0)）：与对侧 Number 归一同型参与 lerp（语料 '0'→'20' at:0.3 期望 '6'、'0'→'1' at:0.3 期望 '0.3'）——钉测抓住的伪离散缺陷。
- repeat/source 纯离散（shim `nonInterpolationExpectations` = expectFlip(0.5)）。
- 期望串匹配经 shim `normalizeValue`（roundNumbers+空白折叠），box4_str_min 单空格输出恒等。

## 钉测（animation.rs mod tests，7/7 绿）

期望表全取语料断言行：slice lerp/钳/混合槽/离散、width 三域 lerp/钳/失配离散/裸 0 归一、outset 四槽钳、repeat 翻转。全量 zero-engine lib 4158 passed / 0 failed。

## A/B 对账（verify-mask-border-ab.sh）

- A 腿 bin-mb-b0：HEAD 干净态重建（现 release bin mtime 20:07:31 provenance 无法定谳是否含前 session 半成品 mask-border 码面，弃用考古改 stash 12 文件精确清单重建；pathspec 不含 peer WIP es_module.rs 与 examples 删集）。
- B 腿 bin-mb-b1：工作树（12 文件刀码面）。
- 双腿五 corpus（css-gaps/css-multicol/css-typed-om/css-shapes/css-masking）+ reconcile-five-corpus.py。
- 双腿同基注记：不含 origin 6ea357800 fs 刀（editing 面与 css corpus 无关）。
- 判据：css-masking F2P 回收（CLAIM 预估 ~1454F）、P2F=0、余四 corpus SET-IDENTICAL。

结果（2026-11-17 21:09 GATE-DONE，十腿 rc=0，双 bin md5 b0=36291827…/b1=3877b8bd…）：

```
[css-gaps]      b0=3838  b1=3838  delta=+0    | F2P=0    P2F=0 | NEW/GONE 全零 | pages 75=75
[css-multicol]  b0=943   b1=943   delta=+0    | F2P=0    P2F=0 | NEW/GONE 全零 | pages 91=91
[css-typed-om]  b0=9947  b1=9947  delta=+0    | F2P=0    P2F=0 | NEW/GONE 全零 | pages 359=359
[css-shapes]    b0=5125  b1=5125  delta=+0    | F2P=0    P2F=0 | NEW/GONE 全零 | pages 145=145
[css-masking]   b0=2368  b1=3687  delta=+1319 | F2P=1319 P2F=0 | NEW F5 GONE O1 | pages 61=61
[TOTAL]         b0=22221 b1=23540 delta=+1319
```

- 判据达成：css-masking F2P=1319 / P2F=0，余四 corpus SET-IDENTICAL 全零位移，page-only 零容忍满足。CLAIM 预估 ~1454F 为按页断言上限估算，实测回收 1319（差值在 composition 页与 source 页残红，均属已挂账范围）。
- NEW F×5 ＋ GONE Timeout×1 归因（同一机制）：`mask-shorthand-subproperties-reset.html` 在 A 腿整页 Timeout 死态；B 腿（五长手可解析、mask 简写可展开）页活跑完，暴露 **mask 简写声明未重置五 mask-border-\* 长手**（css-masking-1 §3.10 reset 语义）5 个真红。FIXME(mask-shorthand-reset) 挂账，非本刀 P2F。

## 残面挂账

- composition 页（outset/slice/width -composition.html，accumulate 通道）残红二期（FIXME(mask-border-discrete) 同段）。
- allow-discrete transition-behavior：FIXME(mask-border-discrete)。
- **FIXME(mask-shorthand-reset)**：mask 简写声明应重置五 mask-border-* 长手至初值（css-masking-1 §3.10），A/B 新暴露 5F。
- `_zwAnimatableProps` JS 名单未加 mask-border 长手（border-image 同不在名单——该名单非 gCS 咨询通道必经门，现状一致不扩）。
- 运行注记：systemd service stdout 全缓冲致日志 LEG 行滞后落盘 20+ 分钟（伪停滞）；进度判读应以 results/transcript 文件为准。后续长跑脚本建议 `stdbuf -oL` 包裹。
