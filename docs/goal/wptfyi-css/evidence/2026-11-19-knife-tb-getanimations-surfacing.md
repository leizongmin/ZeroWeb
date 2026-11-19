# 刀 TB：getAnimations 浮出面 + 管辖归并/取消面 + float/position 离散接入（2026-11-19）

## 目标

css-transitions-2 `transition-behavior.html` 残 5F（master.md 02:3x 在册
「transition-behavior 残 5F engine 离散转换行为面」，承接 A1c claim 残面）。
地面真值（v2-b1 账）：五子测 `getAnimations()` expected 1 but got 0。

## 根因（探针定谳，results-knife-tb/probe.json）

HEAD 探针（03:14 bin）实证残面**不在登记门**：t5 clip-path 离散对 gCS 返
from-value 覆盖（`circle(50.0001px)`——条目已登记 + query override 工作）
但 `getAnimations()`=0；t6 width 基线同样 0——**浮出面缺口是全 transition
面非离散专属**：

- shim `el.getAnimations()`（part04.js）只读 `_elementAnimations`（animate()
  机制专属填充，part03.js:16419 唯一写点）；
- CSS transition 条目活在 `transition_registry::ENTRIES`（Rust thread_local），
  生产链无任何通道浮出到 JS（legacy `TransitionClock::start_transitions` 无
  生产调用方，transitionrun 事件派发链同理 dead）；
- 初版哨兵 guard `&& sel` 分诊（探针 v2 d0）：addDiv 产物 `sel=null
  hnd=__n0`——handle 身份元素全跳；双身份路由（s161 同款）修复。

## 码面（两文件，均 clean 起步零他流 hunks）

**transition_registry.rs（+128/−54 摘）**：
1. `SURFACE_SENTINEL`（`__zw_transitions`）哨兵臂挂 `query_override_for_sel`
   /`query_override_for_handle`——经既有 gCS 通道返活跃条目快照
   （`prop\x1Ffrom\x1Fto\x1Fdur\x1Fdelay\x1Fstart\x1Fdisc`，`\x1E` 分隔；
   未完结过滤：local ≥ delay+duration 摘除）。缓存代际键（style_version/
   drain_gen）使哨兵首问即触发 `register_from_style_change` 重算（快照恒
   新鲜——class 驱动经 SetAttr mutation → version bump → cache MISS 覆盖）。
2. `register_from_style_change` 管辖归并重写（css-transitions-1 §starting：
   属性过渡参数取 transition-property 列表**末位**匹配层——overlap 正形
   S2 `[all(normal), float(allow-discrete)]` 启动 / S3 反序不启动）。
3. 取消面 `cancel_disqualified_entries`：只摘管辖位仍在但已失容的条目
   （duration≤0 / 离散条目 behavior 翻回 normal——S4/S6 断言面）；失管辖
   不取消（changing-while-004「Unsetting transition should not affect
   in-flight transitions」锚定；`transition-property` 清空早退分支零取消，
   fix E——初版 jurisdiction-loss 全取消＝004 过度取消根因）。
4. `DISCRETE_TRANSITIONABLE` 8→10：+float +position（仅 allow-discrete 位
   并入探测集，plain 面清单不变）。
5. FIXME(lazy-before-change)：纯样式表驱动且元素从未 gCS 读的形无
   before-change style 可 diff（S1 残红机制注记）。

**part04.js（+59/−2 摘）**：getAnimations 双身份路由（sel 通道 Gen /
handle 通道 View 回落）→ 哨兵咨询 → CSSTransition 形记录合流
（transitionProperty/playState/currentTime/cancel/effect.getComputedTiming；
per-record 工厂防 var 循环闭包共享末位绑定；cancel 本地摘除键防重复浮出）。

## 探针（zz-tb-probe.html，跑后归档出语料）

v4 终账（03:27 bin `8c6964d2`+handle 修 03:2x bin）：**7P/1F**——
d0 哨兵快照结构 / d1 id 身份 / t4 behavior 长手 CSSOM 写 / t5 clip-path
离散（gCS≈from + anims 1）/ **t3c behavior 翻转取消运行中离散** /
t6 width 基线 surfacing / t1b inline flush 双通道全绿；t1 结构性红
（= 页 S1 形，见 FIXME）。

## A/B 电池（results-knife-tb，b0=HEAD 两 face 剥除 / b1=HEAD+face patch）

双腿 bin md5 b0=`a0e050af` b1=`7d9cd973`（03:45-04:02 跑，fix E 落树前——
004 过度取消形在此窗内为真实红）：

- **css-transitions b1 vs b0：F2P=21 P2F=2 net=+19**。F2P 页级：
  CSSTransition-effect +5、transition-behavior +4（S2/S4/S5/S6 预期兑现）、
  AnimationEffect-getComputedTiming +2、CSSTransition-not-canceling +2、
  Element-getAnimations +2、CSSTransition-transitionProperty +1、
  Document-getAnimations +1、transition-duration-shorthand +1、
  zero-duration-multiple +1、currentcolor-animation-001 +1、
  starting-of-transitions-001 +1。P2F=2：changing-while-004（初版取消面
  jurisdiction-loss 全取消＝真回归，fix E 修复后分解电池绿）、
  transitionevent-interface tev（TB 侧确定性红，双电池同形，triage 见下）。
- **pva 哨兵：0/0/0**——TB face 无 @property 侧泄漏。

## 分解电池增量账（results-knife-decomp，09:40 GATE-DONE）

b1−b0s（TB+TS-RB 增量窗，其余 face 已在 b0s 沉淀）：F2P=20 P2F=1
net=+19——与本电池净账一致；页级分布见 recount-decomp.txt（+5
CSSTransition-effect、+4 transition-behavior、surfacing 族 +6、TS-RB
read-back 形 transition-computed +1 / transition-duration-shorthand +1）。
transitions GONE=16/NEW=2 全系 event-dispatch.tentative.html 页键不稳
（timeout 15→16，census 恒等中性）；css-animations GONE=1 页蒸发同页键
不稳（census 恒等，ΔP 中性不入账）。

**P2F=1 tev 定谳（transitionevent-interface「TransitionEventInit animation
property sets value」）**：确定性非 flake（TB 电池与分解电池双形同在 +
09:51 forensic-tev 单页探针 b0s Pass / b1 Fail 实证）。机制＝浮出面激活
后 TransitionEvent 构造走真 CSSTransition 对象，init dict 缺 `animation`
成员（css-transitions-2 §the-transitionevent-interface
`readonly attribute Animation? animation`；哨兵浮出前 anim=undefined
空洞通过，浮出后缺成员暴露）。修复＝part05.js TransitionEvent init
增 `['animation','animation',null]`（09:55 落树，spec 注记在档）；
R6 重验腿该键 Pass＝修复生效实证。分刀账仍引 battery 权威账
（P2F=1 按修复前形态挂账），修复笔随 TB 提交集入树。

**plain 门清偿注记（定谳收口）**：分解电池 b1 内 TB face governing
显式臂缺清单门＝css-masking 腿 mask-image plain 形 P2F=30 根因（定谳
与机制见 MK evidence；REGRESSED=60 形同 2026-11-18 A/B 档案）。清偿门
已落 transition_registry.rs（09:47：清单外属性不启动过渡，allow-discrete
位照常登记）。重验旁证：r3 重验（09:48 bin）mask-image-interpolation
P2F=30 全消；瞬态定谳维持（30 形 5 连 solo 不复现，MK evidence 详）。
重验账中 mask-valid.sub.html 16 P2F＝重验 bin mask.rs/mod.rs 接线缺位
残面伪形（10:2x 法证定谳），不入本刀账；battery bin event-dispatch
挂起形（GONE=16 截断）随 09:47/09:55 编辑消形（同页 30F 完整记录，
零 Pass 损失）。**账本权威＝battery#2 09:40 snapshot
（account-decomp.battery-snap.txt）**，重验腿仅作旁证。

## 预期与残面

- 预期：transition-behavior.html F2P=4（S2/S4/S5/S6），S1 残红挂账
  （lazy-before-change 结构性，1 子测），P2F=0 零容忍；哨兵 pva 净 0。
- 残面池：S1 形转正（push/seed 模型）、TransitionEnd/CSSTransition 事件
  系统大刀（master.md ~1000F+30F 在册）、cancel-003 flake 面。
