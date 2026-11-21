# 刀 CALC-SIZE-RESOLVED-S3：gCS min-width/min-height 'auto' resolved 0px 门 7 靶（2026-11-21 14:2x 收账）

## 池

恰集单页 7F：`css/cssom/getComputedStyle-resolved-min-size-auto.html`
（csswg #11716 + css-sizing-3 §valdef-width-auto："auto resolves to zero
when no box is generated"）。b1r（S1 后现树）census：19 元素页中 7 元素
期望 0px 全红（block/inline、display-none 自身、display-none+aspect-ratio、
display:none 祖先×aspect-ratio/flex/grid 三变体），9 元素 preservesAuto
已 Pass（四形 aspect-ratio `1/1`/`0/1`/`auto 1/1`/`auto 0/1`＋inline×2＋
flex-item-row/column＋grid-item）。census 排除面：animation/interpolation/
composition（插值域）、keyword-sizes-*（layout intrinsic 域）、
parsing/min-*-computed（calc 负钳＋fit-content() 参量串化域）、
calc-size-*-box-sizing（S2 深值域）、button-min-width（layout used 宽）、
serialize-values（`.1em` inline 声明序列化域——门不触 inline 声明串化）。
设计 [.acceptance/wptfyi-css/knife-calcsize-resolved-s3-design.md]
（patch `bb0f6c495548a1993f9525ea8b63ef96`，engine 双文件 +118/−3；v1
`b63de6a4…` rustfmt 修两处链式调用后重封）。

## face

- **computed_style_cache.rs**：`min_size_auto_resolved_for_selector`（纯
  styles-map 咨询零布局；门序①自身/祖先 display:none 走查 → 0px（无盒
  生成例外置前，压过②③——css-sizing §valdef-width-auto）②aspect-ratio
  指定（computed `aspect_ratio: Option<f32>` None=未声明，默认 `auto`
  不入例外；inline 声明探针 `inline_aspect_ratio_declared` 兜 R3797 退化
  形解析拒收面）→ 保 `auto` ③直接父 display flex/inline-flex/grid/
  inline-grid → 保 `auto` ④否则 0px）。非 `auto` computed 值不进门
  （px/% 静态串化即 resolved）。FIXME: stylesheet 退化形 aspect-ratio
  不识别——保 auto 需渲染 lane declared ratio slot。
- **callbacks.rs**：gCS selector 通道 cache-hit 臂 + miss 臂接线，S1
  echo 之后、静态级联串化之前（spec 序不扰；cache-hit 臂先算后返——
  entry 被 map.insert move 前消费 &entry.0，同 S1 镜像）。

## 电池（knife-calcsize-resolved-s3-battery.sh，log knife-calcsize-resolved-s3-battery.log）

基线 217739b80（S1 bb41f58b4 后 rebase 拉 sibling GLBREAK/VRL-STACK/
CHMETRIC 钉三笔；b0r 四语料 TOTAL 与 S1 b1r 完全相等＝sibling 两笔
layout 刀在本域零位移旁证）。b0r＝face 剥除、b1r＝树面＋face，M2 异
md5 ✓，FACE-MARKER min_size_auto_resolved/inline_aspect_ratio 双 token
0→1（符号入二进制实证）。14:13:57 build 起、14:27:30 收账（13m33s，
全程 test-guard 包裹：build per-proc 6/total 14/1800s＋leg per-proc
4/total 8/3000s＋CASE_WALL_S=60）。corpus 同 S1 四腿（css-values/
css-sizing/cssom/css-variables 哨兵）。

判读（knife-calcsize-resolved-s3-judge.py）：

- cssom **P 881→888（+7）**，主靶页恰集 7 靶全翻（TARGET-GAIN 7＝
  census 7 恰合）。
- css-values **P 608→608（+0）**、css-sizing **P 2905→2905（+0）**、
  css-variables **P 281→281（+0）**：页级 P 计数门零降（flex-basis 残余
  35F 留 S2 深值域不动；parsing computed 域不动）。
- css-variables 哨兵 **SET-IDENTICAL**（56 页）。
- **GRAND F2P 7 / P2F 0**；**CALCSIZE-S3-GATE PASS rc=0**。TRAP-NOOP
  state=done rc=0（b1 face 态即提交态）。

## 验证面注记

face 域 cargo check（release runner 依赖图）净；rustfmt 净（v1 两处
let/闭包链漂移已修；既有无关漂移 css_wire.rs/js_dom_bridge.rs 未触碰）；
clippy face 域零告警；engine lib unit 4246P/1F——clip_interp
test_offset_path_m2_canonical_lerp 既有红（MOTION-2 lane 挂账，同 S1
基线）。

## 残面

- **S2**：calc-size 非 definite 值进 style-system 槽＋layout 期
  basis→px 解析——flex-basis 页残 35F＋calc-size box-sizing 页 11F
  （min-content/max-content/fit-content/stretch basis 深值面）。
- handle 通道 echo（s284 clone-img 期望面雷区，独立 battery）。
- parsing/min-*-computed：calc 负值钳 0（2F）＋fit-content(…) 参量
  保形串化（8F）＝style-system computed 串化域非 gCS 读门。

[控制面](../master.md)
