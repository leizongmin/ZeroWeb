# CSINTERP-S2A r4 判死笔（battery 实证 72 P2F fail-stop）

- 日期：2026-11-22 05:05
- face：knife-csinterp-s2a-face-r4.patch（md5 a85b00a1，5 文件：engine/animation.rs、
  engine/js_dom_bridge/transition_registry.rs、style-system/property/{registry,inherit,computed_style}.rs）
- battery：zw-knife-csinterp-s2a-r4（systemd service），log
  `.acceptance/wptfyi-css/knife-csinterp-s2a-r4-battery-0508.log`，快照
  `snap-knife-csinterp-s2a-r4-0505/`。B0 md5 827497ef / B1 md5 eec6f16d，
  FACE-MARKER b0=0 b1=1，六腿全 rc=0，EXIT rc=0 05:04:57，TRAP-NOOP。
- 基线 HEAD：fa89cbb31（TEXT-INTERP 落库后）。

## 判定

**TOTAL gained=0 lost=72 → P2F=72 > 0 → 预注册 fail-stop 红线触发，r4 face 判死弃置。**

- values：max-height 页 P 230→196（−34）、max-width 页 225→215（−10）、
  min-height 页 110→82（−28）；gained=0 lost=72。
- transitions / sizing：净 0（ASYM 2 行＝animate-with-color-mix b0=5 b1=4、
  event-dispatch b0=11 b1=17，case 数漂移属 runner flake，不入账）。

## 机制取证（b0/b1 JSON message 探针）

失败单根因：**hunk B（query_core s2a 臂输出直返）劫持了本已被 canonical
calc-size 路径正确解析的配对，直返符号形字符串，绕过解析回程。**

- kw↔px 组（min-content→neutral，t=0 期望解析形）：
  - b0 Pass（canonical 双侧同解析，端点吻合）。
  - b1 `assert_equals: expected "100px" but got "calc-size(min-content, 0px + size * 1)"`
    ——actual 恰为符号形，参照元素仍解析为 100px → 整行全跌。
- kw↔kw 组（stretch↔fit-content，allow-discrete/plain 各组）：
  - b0 Pass（discrete 翻转＋端点解析）。
  - b1 actual = `calc-size(max-content, 0px + size * 0)`（符号形），连 t=0/t=1.5
    端点也不匹配 → 六点全跌。

hunk A（`pair_lerp(from,to,0.0).is_some()` 收紧旗标）不设防：插值表对
kw↔kw、kw↔px 配对全覆盖，探针恒 Some → 全部放行进直返臂。

## 预注册勘误

预注册主判据「max-height 页 12 P 复位＋min/max 混对行 F2P（calc-size 期望形）」
前提不成立：b0（HEAD fa89cbb31）账面上这些行**本就是 P**——canonical
calc-size 插值路径（先期刀已落）已覆盖 kw↔px 混对与 kw↔kw discrete 行，
S2A 在本窗**无增益空间**。r3 判定笔的「12 P 复位」推演只对
「flag 过松误伤行复位」成立，未核实 b0 既有覆盖面。S2A 增益前提证伪。

## 家族处置

- S2A 族 r1–r4 连续四轮 fail-stop，全弃置；本语料窗内家族关闭。
- 若未来重开：s2a 臂必须只接管 canonical 路径确有 F 行的配对（以 b0 账面
  探针圈域），且输出必须走 canonical 解析回程（不得直返符号形）——
  在本窗两条件交集为空。
- 后续刀候选不变：SIZE-ROUND-AXIS（判据页需 css-values 净窗）、
  TEXT-INTERP 遗留 4 条（verdict 2026-11-22-knife-text-interp-verdict.md）。

## 树态处置

r4 face 5 文件 `git checkout --`（显式路径）还原 HEAD；s419（es_module.rs）
按协议继续留树不提交；快照与 log 存 .acceptance/（不入库）。
