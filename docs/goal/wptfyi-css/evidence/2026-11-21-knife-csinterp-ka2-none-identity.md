# 刀 CSINTERP-KA2：max-* 动画端点 none 槽归一＋serialize Auto→none（v1 物种映射收窄刀）

日期：2026-11-21
前置：CSINTERP-KA v1 fail-stop（[收账](2026-11-21-knife-csinterp-ka-keyword-roundtrip.md)，
净 +0 不入账 case 级 P2F=313 三池归因）——本刀为其收窄认证，不等 K-B/K-C 合成。
face：crates/engine/src/animation.rs（`parse_max_sizing_anim_value`：仅
max-width/max-height 两臂 `none→Px(INFINITY)`，其余 `parse_px_strict` 直通＝
b0 路径逐字节保留；width/height/min-* 四臂 HEAD 不动）＋
crates/engine/src/js_dom_bridge/computed_style.rs（`max_size_computed_norm`
Auto→`Px(INFINITY)` serialize hunk，v1 全语料实证零 P2F 块原样保留）
判别：FACE-MARKER `KA2-FACE max-none-identity`（b0=0/b1=1 实测）＋M2＋S1/S3 ride
电池：`.acceptance/wptfyi-css/knife-csinterp-ka2-battery.sh`（launcher
knife-csinterp-ka2-launcher.sh，systemd zw-knife-csinterp-ka2-launcher-2112）；
语料 values/transitions/sizing/flexbox/typed-om 五套件 ×双臂
bin：b0 `cd7e1e9d5b32191391f004b173328b71` / b1
`1537bfdbdd4ddf205638fda1db1930e2`；PATCH-MD5 `0275dd18e3862649da3402bb3a6f9fcb`
绑定；生成器 `knife-csinterp-ka2-face-gen.py`（temp-copy 构造型：HEAD 读入→
锚恰 1 命中断言→difflib→`git apply --check` 前向验证→单次 apply 落树）
落树时序：取对端 H 电池 legs 窗（21:19:35，其 bin 已拷/census 已采样完），
本刀 face 不入对端 build 扫树窗

## 物种映射定谳（v1 fail-stop → 收窄依据）

对 v1 全 10 腿 JSON case-name（`property <p> from [X] to [Y] at (t) should be [Z]`）
解析端点物种×回显侧：

- **P2F 侧：none 端点全表 P2F=0**。3 例 none-cell P2F（neutral→kw:max-content@to、
  neutral→kw:fit-content@to、kw:min-content→neutral@from 各 1）全由 keyword 侧
  变体解析引起。keyword 变体写入两类破形与 v1 三池归因吻合：
  1. min/max 臂槽形期望 `calc-size(min-content, 0px)`——b0 经 s2b 槽链串形巧过，
     变体串化破形＝K-C（calc-size slot 路由）域；
  2. width/height 臂 used-size "0px" 巧合＝K-B（used-px echo）域。
  at(0) fit-content 翻红根因＝css-parser R1018 bare fit-content→MaxContent 别名
  破恒等（helper 不解析 fit-content 即消除）。
- F2P 侧 echo=none cell：none→px@from 48＋none→max-content@from 18＋compositing
  add-none ~12 ⇒ 预注册 **~+78 F2P / 0 P2F**；keyword 侧增益（@to 臂 8、stretch
  14、neutral 4）不追（K-C/K-B 域）。

## 机制（规范锚）

- css-sizing-3 §5.2（#valdef-max-width-none）：max-* 无 auto 计算值，`none`
  槽表示归一 `Px(INFINITY)`——与 gCS `sizing_length_to_css` 无穷臂同口径，
  动画端点回读串化恒等 "none"；min-* auto 保 "auto"（§5.1）。
- serialize 侧：级联 `max-height: none` 声明写入径落 `Auto` 变体，computed 语境
  归一 none——b0 echo "auto"（CSS-wide keyword initial/unset 同径）破 §5.2，
  本刀归一后 "none"。
- 其余值保持 b0 `parse_px_strict` 失败即弃、consult 回落 cascaded 终值语义
  （不触 keyword 池 P2F）。

## 预注册（物种映射基准）

- values：none→px@from max-h/max-w 12+12＋none→max-content@from 9＋unparsed
  none ~6 ⇒ ~+39 F2P；sizing 同族 ~+39；transitions/flexbox/typed-om 三哨
  SET-IDENTICAL；P2F 红线：任何 P2F → fail-stop（预测 0）。

## 电池结果（GATE PASS：P2F=0 入账）

21:19:35 LAUNCH-GO（H 收官后 serialize 门自动放行）→ 21:50:49 GATE-DONE rc=0。
页级 judge：**values +89 / transitions +1 / sizing +91 / flexbox 0 / typed-om 0，
TOTAL gained=181 lost=0**。case 级键差分（页级对冲审计）：**F2P=178 / P2F=0 /
ASYM=3**。

- F2P 物种分解（全机制绑定）：other→px@from 96（=CSS-wide keyword
  initial/unset 族，serialize hunk §5.2 归一驱动，max-h/max-w×values/sizing
  各 24）＋none→px@from 48（apply none 臂）＋none→max-content@from 18＋
  compositing add-none ~7＋stretch→none@to 8＋unparsed none ~1。
- **剔除三笔（如实入账，净 +176）**：
  1. `calc-size/probe-h-diag.html` +3——对端 H 刀诊断探针页 21:29:29 创建
     （恰在本刀 b1 values 腿窗），b0=0/b1=6 ASYM 揭穿（6 子测 3P/3F）；
  2. `zw-probe-wa3b-census.html` +1——对端 WA-3b 探针页（transitions）；
  3. `random-computed.tentative.html` +1——`random(--identifier)` 种子页
     flake-suspect（与 face 零机制关联；单页 filter 探针未 engage，按
     URLSMOD +1 flake 剔除先例处置）。
- ASYM 其余两页：animate-with-color-mix 3→4、event-dispatch.tentative
  30→16（b1 断言数漂移，页级 P 零位移＝报告完备性不对称非回归；
  transitions corpus 事件时序型页既有 flake 域）。
- 门审计：FM b0=0/b1=1；M2 md5 异；S1/S3 ride 零扰动（ride=2 docs 文件）；
  腿 rc=1 为含失败用例正常退出码（腿时长 80s–8min 非 usage 同秒形）。
- **bin caveat（法证留档）**：对端 CLIENTWIDTH-PADBOX v3 shim part04.js
  mtime 21:19:49 落本刀 b0 build 窗（21:19:39–21:20:46）内——CLIENTWIDTH
  marker b0=3/b1=5 实证双 bin 携不同 shim。归因靠机制绑定闭环：7 个 gain 页
  全部 max-height/max-width interpolation/composition 域；clientWidth/
  scrollWidth 消费页（white-space-intrinsic-size/overflow/flexbox
  min-height-auto 族）零位移；gCS max-* 串化走 Rust computed_style 路径与
  part04.js 布局查询零交集；FM 门钉本刀 face b1-only。
- 对端 v3 电池 21:52 rc=8 自杀（其自身门 TREE-NOT-COMPILABLE/PATCH 门），
  其 face 域（part04.js＋style-system 双文件）自理，与本刀零交集。

## 后续

- keyword 大池仍待 K-B（used-px echo）/K-C（calc-size slot 路由）——物种表
  留 `.acceptance/wptfyi-css/NOTES-knife-csinterp-s2-size-lerp-claim.md`
  21:0x 节供其设计取用。
- 本刀入账后 max-* interpolation 页残余 fail＝数值 lerp 缺口（K-B）＋槽形
  期望（K-C），不再有 none/initial-unset 形缺口。
