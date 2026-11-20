# 刀 WAC：WAAPI composition 准入扩面——mask-border 三长手 uniform+mixed op（2026-11-20）

## 池

刀 MB 落库后 mb3 电池 JSON 取证：css-masking composition 三页
（mask-border-{slice,width,outset}-composition.html）CSSA 通道 56/112 半账，
WAAPI 通道 0/56 空账，各 t 采样 got=underlying——不是采样缺口，是 shim WAAPI
准入把 mask-border 关键帧整体丢弃（gCS=underlying）。刀 MB 已把引擎侧
compose/预合成面修满（animation.rs 三臂逐槽同型求和 + §5.4.4 端点预合成 +
mixed-op ② 臂），引擎直接可消费 composition 声明，缺口只剩 shim 准入与传输
通道。本刀即补这条通道。

## 链路

shim keyframes → `_zwWaapiSpecKeyframes`（剥 composite 元数据）→ JSON
`{o, p}` 帧 → Rust `build_keyframes_rule` 合成 `@keyframes` CSS 文本 →
css-parser → engine point.composition。要保形透传 composition，需三处协同：
1. shim 白名单收 mask-border 三长手（否则进不了动画管线）；
2. 准入门放行 mask-border（uniform op 沿既有 `_zwUniOp` engine-comp 通道；
   mixed op（replace↔add 族）置 `_zwMbMixed` 新旗放行、`_zwEngComp` 置空；
   ④/单位集守卫对两模式跳过）；
3. spec 层帧级 `composite` 保形透传 `c` 字段，Rust 侧 `build_keyframes_rule`
   把 `c` 落成帧内 `animation-composition:` 声明（css-animations-2
   §animation-composition；与 CSSA 驱动的 @keyframes 文本同通道入
   point.composition，引擎刀 MB 臂直接消费——**引擎零改动**）。

## 码面（2 文件 +37/−9，patch md5 eef9e3823fdabca00466eecb996285a4）

- `crates/engine/src/js_dom_shim/part03.js`：
  `_zwAnimatableProps` +mask-border-slice/outset/width；准入门 `_zwIsMb`
  （`_zwMbMixed` 旗、engine-comp 条件 `_zwUniOp !== '' || _zwMbMixed`、
  `_zwEngComp = _zwMbMixed ? '' : _zwUniOp`）；④/单位集守卫加
  `!_zwMbMixed`；`_zwWaapiSpecKeyframes(keyframes, keepComp)` 帧级
  `composite` → `frame.c`；调用点 keepComp 按两模式置真。
- `crates/engine/src/js_dom_bridge/js_anim_registry.rs`：
  `build_keyframes_rule` 帧级 `c` ∈ {replace,add,accumulate} → 帧内
  `animation-composition:{c};`（白名单外值不落，帧回落动画级 op）。

## 探针

runner 86a3a5e4（b1 字节）跑 css-masking：三页 56/112 → 112/112 满贯
（WAAPI 56/56），全语料 +168 零页损。

## 电池（.acceptance/wptfyi-css/results-knife-wac，配方承 mb3）

- b0 = HEAD(1e1f8f8a4) 剥面，runner md5 f374ee17df52b65fe89935105c49445d；
  b1 = HEAD+面，runner md5 86a3a5e4d277f3377c57a12da07b2686（与探针 runner
  恒等＝跨构建确定性互证）。M2 异 md5 ✓；face marker `_zwIsMb` strings
  b0=0 / b1=3 ✓；RIDE 空（face 外树净，S1/S2/S3 免疫）。
- 腿 {css-masking 主靶, css-transitions, css-animations 哨兵} × {b0,b1}，
  CASE_WALL_S=60，test-guard 包裹（build 6/14/1800s，leg 4/8/3000s）。
- 原始对账：gained=168 lost=0 regressed=2 pages_gain=3——三页各 56→112
  全账；唯一 REG＝css-animations/event-dispatch.tentative.html P 2→0。

## REG 定谳：窗口伪影，不转抄

event-dispatch 在全部 6 次历史观测（mb3 双臂 + wac 双臂 + 仲裁复跑）均
页面级 Timeout（CASE_WALL_S=60 截断，子测只跑到一半，窗内条目数 12~19
波动）。b0 窗恰达第 17 子测（含 2 个偶发 Pass「Redundant change」），b1
窗 12 条即断未到达——Pass 集比对把「未跑到」记成回归。仲裁：双臂
CASE_WALL_S=240 复跑（/tmp/wac-arb-b{0,1}.json），ab-compare
**gained=0 lost=0 regressed=0 SET-IDENTICAL**——高墙下双臂恒等，REG 为
60s 窗口伪影。终判：**gained=168 lost=0 regressed=0，pages_gain=3**；
哨兵 transitions SET-IDENTICAL、animations SET-IDENTICAL（高墙口径）。

## 残面（挂账不扩面）

- mask-border *-interpolation 页 WAAPI 通道负外推采样缺口（mb3 已挂，
  WAAPI 采样不外推为预存域缺口）；
- 通用 mixed-op 面（非 mask-border 族）仍走 shim uniformity 门，后续刀
  候选（clip-path shape 加法 69F、text-shadow WAAPI 外推 15F、clip
  value-support 36F、池③ custom-property-animation ~150F）。

## 后续核验锚

patch/树 face/git diff 三方 md5 恒 eef9e3823（判读绑定 b1 blob：落库前
part03.js=fb30cb77d / js_anim_registry.rs=eb8acce2f 工作区字节，落库后以
`git rev-parse HEAD:<file>` 对照）。 battery 脚本 /tmp/knife-wac-battery.sh，
日志 .acceptance/wptfyi-css/knife-wac-battery.log。
