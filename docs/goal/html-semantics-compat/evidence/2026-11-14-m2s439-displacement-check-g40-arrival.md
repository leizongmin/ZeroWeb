# M2 s439 — 位移检查窗（g40 到货覆盖 + dialog 第三窗复验，零代码变更轮）：键面零真位移，三差异全在册振荡族

日期：2026-11-14（晚）。前情：s438 收口后渲染流 **g40 serialize-resolution
刀**（1382895dd，css-parser `<resolution>` 维度 + image-set() calc canonical
折叠）到货——s438 位移检查树（a80d3884b = anchor-size + ip C4）未覆盖
g40；本轮双跑覆盖 g40 到货 + dialog phase-flip 复验（s437 挂账）。

## 执行

组合树（0180a0a76）双跑 `make testharness-html-semantics`（无 FILTER，
release 增量重建；宿主 loadavg 回落带 1min 4.38 / 15min 21.42——zeroseed
风暴尾段，同窗 ZeroWeb-2 渲染流 bench 轻度并发）：

| 基线 | stable-P | stable-F | stable-T | s439 |
|---|---|---|---|---|
| s433 终稿 | 8309 | 202 | 3 | — |
| s437 fix | 8308 | 203 | 2 | — |
| s438 | 8308 | 199 | 2 | — |
| **s439（本轮）** | — | — | — | **8307 / 200 / 3** |

**vs s438 差集（P −1 / F +1 / T +1）全数归因在册振荡族，零真位移**：

- **modulepreload-referrerpolicy strict-origin P 沾染出集**（leg1 F / leg2
  P）：referrer 族 echo-referrer.py server-state 家族（s116 在册：
  modulepreload referrerpolicy 族多 job 并行 server 态交叠）；该键
  s433/fix437/s438 三基线均稳定 P，单窗翻转待下窗复验回位。
- **optional select 双文件文件级态置换**：detailed.optional 双腿文件级
  Fail（`page script threw: Execution timeout: 90000ms`——页脚本 90s
  超时→页级 Fail）；非 detailed leg2 文件级 Timeout（`loaded:false
  pending:4` 同 s438）且 4 子测试 leg2 全缺席、leg1 半缺席（End/Home 在、
  PageDown/PageUp 缺）＝负载族渐进退化形态。子测试态被文件级态置换，
  键面真态 = 红池在册无变化。
- **g40 serialize-resolution 零归因位移**：无新增稳定 F 可归因代码面
  （resolution 维度/image-set 折叠与 html/semantics 语料面不交叠，实测
  证实）；渲染流侧其 corpus 净 −4（R5565 补验在案）。

## dialog phase-flip 第三窗复验（未翻回）

leg1/leg2 双 F（stable-F 维持）。红相自 ~20:00 起三窗连续（fix437 双腿 →
s438 双腿 → s439 双腿，跨 ~2h），唯一绿窗 14:29（s436 收口轮）在 zeroseed
风暴起势前。**负载相关假设维持但窗数累积**：非代码定性不变（s437 bisect
父提交 solo×3 红在案，跨树恒红），若风暴完全平息（loadavg 持续 <5）后
下窗仍红，则重开归因面（候选拍点：focus-after-close 时序对 runner 帧
驱动泵的敏感窗、detailed.optional 同页族对照）。挂账维持、下窗续验。

## 结论

s439 收口（零代码变更轮）：g40 到货零波及、键面 8307/200/3 与前带等值
（差集全在册振荡族）。dialog 三窗红相累积待风暴平息后复验翻回；
modulepreload 单窗沾染待回位。基线带推移：s439 集合（/tmp/s439/s439.s*）
替代 s438 成当前基线。工件：/tmp/s439/{sweep-1,sweep-2}.log、
sweep-{1,2}.rc（rc=2）、join.sh、s439.{1,2}.ks、s439.s{P,F,T}、vs* 差集、
s439.taint。
