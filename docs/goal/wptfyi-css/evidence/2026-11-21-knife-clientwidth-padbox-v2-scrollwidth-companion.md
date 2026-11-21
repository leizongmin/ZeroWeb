# 刀 CLIENTWIDTH-PADBOX v2：clientWidth/Height＋scrollWidth/Height padding-box 同口径

日期：2026-11-21
face：crates/engine/src/js_dom_shim/part04.js 单文件（clientWidth/clientHeight 非
HTML 臂减 border-width floor 0；v2 增 scrollWidth/scrollHeight 同口径 companion）
判别：FACE-MARKER `CLIENTWIDTH-PADBOX`（v2 b0=0/b1=3）＋M2（bin md5 异）＋S1/S3
ride 零扰动（RIDE 18 文件 census）
电池：`.acceptance/wptfyi-css/knife-clientwidth-padbox-battery.sh`（text/values/
sizing/box/flexbox/overflow/typed-om/cssom-view/cssom 九语料×双臂；PATCH 绑定
v2 md5 599cbb2a）

## 机制（规范锚）

- clientWidth/clientHeight：CSSOM View §clientWidth＝padding-box 宽/高（含
  padding 不含 border）。旧实现直返布局 rect（border-box）。face 减左右/上下
  border-width，floor 0（内容小于边框的盒 padding-box 不得为负）。根元素 HTML
  臂 viewport 语义既有保持。
- scrollWidth/scrollHeight（v2 companion）：CSSOM View §scrollWidth＝scrolling
  area 居 padding-box 内不含 border。旧实现同布局 rect border-box 直返。
- 布局 rect 无 overflow 数据（不含滚动展开量）：scrollWidth/Height 保持
  client 尺寸近似（精度边界同旧注释声明），本刀语义修复点＝border 剔除，
  非滚动展开量精确化。

## v1→v2 演进（双错相消揭穿型 P2F 归因）

v1 电池（19:34，clientWidth/Height 单侧修复）主靶 +102/sizing +26 达成，但
flexbox-min-height-auto-003/004 共 5 P2F：断言 `scrollWidth == clientWidth`
（"no horizontal overflow"）b1 失败 `expected 40 but got 44`——scrollWidth
shim 同为布局 rect border-box 直返，b0 双错相消（44==44）假绿，face 修
clientWidth（40）后既有 scrollWidth border 膨胀被揭穿。配对修复入同 face
（v2），5 案清偿（详见 docs/learnings/bugs/2026-11/
2026-11-21-pair-cancelled-bugs-unmasked-by-single-side-fix.md）。

## 电池结果（v2 GATE-DONE 20:15，judge PASS）

- **净增 +150 / 零 P2F**（judge GRAND gained=150 lost=0）：
  - text：white-space-intrinsic-size-021 P 0→102（**主靶 +102，v1 复现**——
    `border: solid` medium 3px 的 inline-block 空盒 expected clientWidth 0，
    旧实现直返 border-box 恒 +6，192 测族大面积 F2P）
  - sizing：+29（contain-intrinsic-size auto-008/auto-012/030/031/032＋
    intrinsic-percent non-replaced-006/replaced-028＋stretch indefinite-1/2/3；
    v1 +26 基础上 cis-030 +3 增量）
  - overflow：+18（scrollable-overflow transform-005/-009＋padding-input＋
    overflow-empty-child-box＋overflow-outside-padding——scrollWidth 主消费
    语料零 P2F，companion 正兑现）
  - flexbox：+1（negative-overflow）；min-height-auto-003/004 双臂 4→4＝
    v1 的 5 P2F 清偿（b0 双错相消恒真、b1 双值修复恒真，case 级 8 Pass 全绿）
  - values/box/typed-om/cssom：SET-IDENTICAL（哨兵零位移）
  - cssom-view 腿 corpus 名笔误（`testharness-css-cssom-view` 不存在，正确名
    `testharness-cssom-view`）两臂空跑＝零 A/B 信息，判读跳过挂 ops 债；
    哨兵覆盖由 overflow/typed-om/cssom 补位
- 有效性：FM b0=0/b1=3、M2、PATCH-MD5 599cbb2a 绑定（v1 6ff847fa 作废）、
  RIDE 18 文件（peer faces+docs）S1/S2/S3 census 零扰动；G1b 入库提交
  （d50fb2f99，19:53:26）在双臂 build（19:51:18/19:52:06）之后且不触
  worktree 内容，A/B 归因纯度保持
- 钉测：part04.js 为 shim 面（无 Rust 钉测）；验证面＝电池九语料双臂＋
  v1 5 P2F 案 case 级 F2P 回归

## 消费域与余量

- clientWidth/Height/scrollWidth/scrollHeight 四 getter 语义统一 padding-box
  近似；offsetWidth/Height 保持 border-box（spec 语义）。scrolling area
  展开量（overflow 内容）精确化＝既有「布局 rect 无 overflow 数据」限制
  维持，挂账另刀。
- v1 电池（19:34 results-stale-v1-1936 留勘验源）：主靶 +102/sizing +26/
  flexbox -5；v2 全量取代入账。
