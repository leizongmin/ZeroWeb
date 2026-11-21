# 刀 CLIENTWIDTH-PADBOX v3 收账——table 元素 clientWidth/Height offset 口径豁免（+1，零回归）

日期：2026-11-21 21:19 发射–22:48 GATE-DONE · lane cwcv3 · ZeroWeb-2 clone · 续开轮采彼落库
域：CSS2 §11.5 tables separated model——table/inline-table 元素 clientWidth/Height == offsetWidth/Height（含自身 border）
预注册/挂账源：v2 收账 evidence（2026-11-21-knife-clientwidth-padbox-v2.md）table-with-border 缺 internal-table border 特例挂账

## face（part04.js 单文件追加，+18 行）

`clientWidth`/`clientHeight` 两臂顶部 table 豁免：`__zw_get_computed_style(sel,'display')`
∈ {table, inline-table} → 直接返回 `r.w`/`r.h`（offset 口径，不减 border）。
v2 统一减 border 对 table 元素二次减除（16px＝8px×2）即本刀清偿的 P2F 特例。

## 电池（log `knife-clientwidth-padbox-v3-battery.log`）

- 21:19 launcher 排队（SERIALIZE-WAIT 至 22:26，让位 H/KA2/cwcv-v2 窗）；22:26–22:48 八腿 ×双臂。
- b0 md5 `2382287511` / b1 md5 `ae97150f`（与 G1a 电池 b1r 同 md5＝G1a face ride 双臂相消互证）；S1 ride 窗内零扰动。
- FACE-MARKER-V3 b0=0 / b1=2 ✓；TRAP-NOOP state=done rc=0（face 在树＝发射态，落库即提交态）。

## RECONCILE（22:48:19）

- CWV3-GRAND gained=1 lost=0：cssom-view `table-with-border-client-width-height.html` P 0→1（expected 37.6 == offsetWidth 实证）。
- 哨兵：values/sizing/box/flexbox/overflow/typed-om/cssom 七腿全 SET-IDENTICAL（gained=0 lost=0）。

## 判定

v2 挂账特例恰集清偿：主靶 1F 全翻、零 P2F、零哨位移。净 +1 入账（累计 +12965→+12966）。
