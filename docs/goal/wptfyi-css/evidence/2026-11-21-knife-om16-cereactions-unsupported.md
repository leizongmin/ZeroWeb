# 刀 OM-16 · typed-om CEReactions 影子派发＋unsupported 名门 20F（2026-11-21）

## 认领与边界

- 池＝typed-om 通道残池 119F/64 页（OM-15 后 census b1r 12406 keys
  P 12287）分解首刀，主靶 **20 靶**：
  - `the-stylepropertymap/inline/attribute-changed-callback.html`
    9F（set/delete/clear/append/custom×2/shorthand/old-value/
    interleave 十子测族，恰集 9F）；
  - unsupported 名族 11F：get-invalid ×3 / getAll.tentative ×3 /
    has.tentative ×3 / delete-invalid ×2（declared/inline 双面页）。
- 域＝crates/engine/src/js_dom_shim/part03.js（style 代理写汇点）
  ＋part10-typed-om.js（typed-om 门域）双文件；零存储/读路变更
  （store 事件与内容属性写面维持现状，影子 Map 仅派发域）。
- 共享面提示：`_ce_dispatchAttrChange` 值门修正
  （absent(null)↔'' 现按 HTML attribute change steps 增删派发）为
  spec 正向修正，影响 custom-elements reactions 语料稳定窗，
  html-semantics 流（s611 布防）须预注册核对该门。

## 根因（探针 zz-om16probe/zz-om16probe2 定谳，已删）

1. **CEReactions 面（attribute-changed-callback 9 靶）**：
   css-typed-om-1 §5.3＋HTML custom-elements——StylePropertyMap
   set/append/delete/clear 每操作恰一次派发 attributeChangedCallback
   （shorthand 展开多写合一）。本仓 style 为 store 事件＋内容属性
   双源，写汇点直接落 store 不走属性变更步，callback 从不触发。
2. **unsupported 名族 11 靶**：`_zwOmPropSyntax`(225)/
   `_zwOmKnownExtra`(30) 是部分表非完整支持名单，get/getAll/has
   未知名（mask-position/padding/color-scheme 三实证）应抛
   TypeError（css-typed-om-1 §get/has unsupported property name），
   现返回 undefined/false；delete 未知名同抛。
3. **interleave 靶**：style 属性与 attributeStyleMap 交替写，
   callback old/new 值须反映属性串现态。

## face（part03.js 影子派发＋part10 门域）

1. **影子双轨**：`_zwStyleCeShadowByHandle`/`…BySel` 双 Map——
   handle 为**字符串**（createElement 产物 '__n0' 形），
   WeakMap 串键抛 TypeError 弃用（v2 教训，同 `_zwRawByHandle`
   Map 先例）；`el.style` getter 每次新 proxy 实例非 SameObject，
   影子记 last-wins 串化文本 `_zwStyleAttrText`。
2. **操作窗**：part10 set/append/delete/clear 四操作
   `__zwStyleCeWindow(true/false)` try/finally 包裹（set 保留
   `_zwOmExpandShorthand` 多写合窗）——window=end 时按首记 old
   ＋末写 text **单次** `_ce_dispatchAttrChange`（shorthand 五写
   合一派发）；窗口内 pending 记账首记捕获 old（影子优先＋attr
   回落）、后续只更 text；flush 先清深度防重入。
3. **7 写汇点改 `_zwStyleCeSyncDeferred`**：setProp ×4 /
   removeProperty ×2 / cssText ×1；cssText 臂保持 4 参传真 old
   （`_ceOldSt` pre-attr 捕获）。**参数分派陷阱**（v4 trace
   定谳）：转发函数恒绑定形参 → sync 以 `arguments.length >= 4`
   判分支恒真 → 显式传 undefined 把影子优先 old 查找短路成 null
   ——条件转发：未传 4 参时不得显式转发 undefined 第 4 参。
4. **共享门修正**：`_ce_dispatchAttrChange` 值门
   `o === nv && (oldVal == null) === (newVal == null)` ——
   absent(null) ≠ ''（HTML attribute change steps 增删须派发，
   clear 面吞派发根因）。
5. **part10 门域补遗**：`_zwOmGateKnownExtra`（mask-position/
   padding/color-scheme）入 `_zwOmStyleMapUnsupportedProp` 门
   （append.tentative 顶层 `computedStyleMap().get('mask-position')`
   抛型整页 script 死根因）；delete 头 unsupported 名 TypeError
   门保留。

## 电池（承 knife-om15 配方）

knife-om16-cereactions-unsupported-battery.sh（patch md5
`46ff137d495a51ce63fab4fd64524bab` v4 终版 face 快照）：
b0r（face 剥除＝HEAD 树）/b1r（face 重放）双臂，test-guard 包裹
（build per-proc 6/total 14/1800s，leg per-proc 4/total 8/3000s +
CASE_WALL_S=60），M2 异 md5＋FACE-MARKER token 增量门
（StyleCeSync b0r=0/b1r=12、UnsupportedProp b0r=0/b1r=3）。
腿：{css-typed-om 主靶, css-values, css-sizing, cssom} × 双臂
（cssom＝style 代理重消费哨，css-values＝typed-om 读径哨）。
judge：20F 恰集 F2P＋P2F=0＋new/gone=0 硬门。

## 返修链（v1 GATE-FAIL → v4）

- **v1**（battery.log 19:18）GATE-FAIL：F2P=16/expect 20、P2F=2、
  new=1、gone=14（append.tentative 整页 gone 14）。四根因：
  ①影轨键判 `typeof handle === 'object'` 对字符串 handle 全数落
  sel 轨 Map(null) 共键——跨元素泄漏＋同值短路吞派发；
  ②值门 absent(null) 归一 '' 与空串同值——clear 面吞派发；
  ③shorthand set 展开多写各派发（5 次）；④门域三真属性不在表。
- **v2**：v2 探针全零派发——`_zwStyleCeShadowByHandle` 用
  WeakMap，`WeakMap.set('__n0',…)` 串键抛 TypeError 被 try/catch
  吞，派发前中止 → 改 `new Map()`。
- **v3→v4**：interleave 靶 old=null 残留（19/20）。静态分析到
  极限后 `__zwStyleCeDbg` trace 插桩定谳：三次派发全 `a4=1`
  （见上参数分派陷阱）→ 条件转发。v4 探针 c3
  old='display: block;' 修复确认。

## 判读

**OM16-GATE-PASS**（battery-v3.log 20:21:32 build 起 20:33:42
收账 12 分钟；二进制绑定 b0r=`349af22a14b0d3e996213f78db93b25c`
/ b1r=`58ad4b28aecc1bf981b1a64a752ffcd3` M2 异 md5）：

- [css-typed-om] P 12287→**12307（+20）**：F2P=20 恰集精确兑现
  （attribute-changed-callback 9＋unsupported 名族 11），
  P2F=0、new=0、gone=0；
- [css-values] 954 keys SET-IDENTICAL（P 642→642）；
- [css-sizing] 5700 keys SET-IDENTICAL（P 2979→2979）；
- [cssom] 1630 keys SET-IDENTICAL（P 888→888）；
- GRAND F2P=20 P2F=0 new=0 gone=0；TARGET-GAIN om16-arms 20/20。

## 预注册对账

- 主靶 20/20 F2P 恰集，非主靶 F2P=0（恰集外零出列）；
- 三哨 SET-IDENTICAL（style 代理零存储/读路变更核）；
- FACE-MARKER 双 token 增量兑现（12＋3）；残池余 ~99F 负值/
  关键词回退/opacity percent/序列化 IDL 族为后续候选另刀。
