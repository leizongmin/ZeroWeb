# M2 s583 — **band 聚合形首现轮（族带第 16 轮＝错开分裂终结：detailed 页级 T 消失 GONE 1＋8 子测 Fail 双腿聚合 NEW 8，stable 8510→8517 双腿全同 disagree 0/l1only 0/l2only 0）+ 全桥 b582 零噪声兑现轮（keys 1731 fail=0 NEW=0 GONE=0＝d63 callbacks/css_wire 双钉未改桥测键面，改判预期解消；b583≡b582 逐字节恒）+ runner 必换代 ×12 连轮（pre a528acd8→mid=post ae5baec4，+253,008 bytes；leg1 Compiling ×6／leg2 Fresh 0）+ solo 发射形复现第 5 连轮（2/2 IDENT 4 子测 Fail 恒形，第 5 形未现）+ 七哨持位轮（mpreload 第 8 连兑现；waapi 恒形对 WAAPI face 三码落窗零响应）**：join **stable 8517（8305P/206F/5T/1U）disagree 0 + l1only 0 + l2only 0**，vs s582 stable（8510）**NEW 8 + GONE 1 + flips 0＝位移全落族带（聚合形），非带键逐键零位移**。

日期：2026-11-20。跨 session 双断点接管轮：前 session s582
收口后接管（05:18 捕获＋基线核对＋三件套布防 05:15 waiter），
07:45 waiter #1 满程超时（对端 d63 runner 压窗全程无净窗）→
07:46:23 s580 预案重臂第二臂后 session 终止（API 限额）；本
session 08:2x 接管：开工 pull 捕获 ×6 全列核实（17d7c44bf→
743e1bc59：code ×3＝9996cbebf WA-2 computed timing＋b1562f1d2
WA-3a CSSTransition 对象面＋169fdf61f 对端 d63 刀 D3 caret 重
映射＝**js_dom_bridge callbacks.rs+css_wire.rs 双钉＝全桥零噪声
预期改判「或现 NEW/GONE」**）→ 补账二 cd5472a15 提交推送
（pull 再进 ×2＝2e567a891 mask-border animation.rs＋1e1f8f8a4
收账；animation.rs 零桥触）→ 三件套在循核对（waiter 1250460
第二臂＋post.sh 974404 原链完好，无 double-fire）→ 开窗
**08:44:42 clean-soaked**（procs=0 load<3 3×45s）HEAD=
**4923a4398** 钉扎起腿 → leg1 08:44:42–08:47:15（2:33，rc=2
惯例）→ leg2 08:47:16–08:49:14（1:58，rc=2 惯例）→ 桥测
22.05s rcb=0 → analyze → solo1 08:50:16–08:50:27＋solo2
08:50:27–08:50:37（rc=2 ×2，11s+10s）→ 收口。

## 窗底账（s583 窗＝2483ee96e→4923a4398）

- 窗＝**27 commits：code ×11**（fix ×10＋test ×1）＝
  c503a585e overlay 增量合并与 NA 采集收窄（DOM 变异超线性
  热点修复）＋7513d2c8a NA 采集墙钟护栏（test）＋641656d54/
  4fc4cb49f/8d947bf96 named 候选集三臂＋ea98cb8ae WA interface
  簇 face＋9996cbebf WA-2 computed timing＋b1562f1d2 WA-3a
  CSSTransition 对象面＋7dae507ef SV2 typed-om 矩阵＋
  169fdf61f 对端 d63 caret 重映射 fresh 查询通道（**js_dom_
  bridge callbacks.rs+css_wire.rs 双钉**）＋2e567a891 mask-
  border animation.rs；docs ×14（本流接管/补账三笔＋对端
  rendering R5845＋wptfyi-css 七笔＋全仓对账＋PR #313/#314
  系 merge ×2 复合计入）；test ×1 见上。
- 捕获三批全列核实（接管续核 05:18 ×7／接管续核二 ×6／
  push 补账 ×2），runner 预判 Fresh→必换代改判（ea98cb8ae
  起）全程兑现；全桥改判「或现 NEW/GONE」→ 实测零噪声
  解消（见下）。

## 双腿 join（HEAD 4923a4398 树）

- raw canon：leg1 **8519**／leg2 **8519**（uniq 8517 双腿
  ＝双发伪影 ×2 各腿恒）＝**双腿逐键全同（s577 完全收敛
  后第 2 例）**（s582 8516/8513 错开形后回归全同）。
- join：**stable 8517（8305P/206F/5T/1U）+ disagree 0 +
  l1only 0 + l2only 0**——vs s582 stable（8510，8305P/
  198F/6T/1U）：**NEW 8 + GONE 1 + flips 0**。状态拆分：
  P 8305 恒、F 198→206（+8＝NEW 子测）、T 6→5（−1＝
  detailed 页级 T）、U 1 恒。**NEW/GONE 全部落在族带
  select-home-end-pagedown-pageup（detailed+optional 双文
  件），非带键逐键零位移**——窗带 named 候选集/overlay/
  WAAPI face/mask-border/d63 caret 五股码面，sweep 语义面
  仅族带响应（族带变化归因见下节，非五股直接语义靶）。

## 族带第 16 轮：聚合形首现（错开分裂终结）

- **GONE 1＝detailed 整文件页级 Timeout 键**
  （select-home-end-pagedown-pageup-detailed.optional.html
  页级键；s582 双腿 stable 锚）。
- **NEW 8＝8 子测 Fail 双腿聚合**：detailed ×4（Home and
  End／…starting at the top／…starting at the bottom／
  PageUp and PageDown）＋optional ×4（Home/End/PageUp/
  PageDown）全 Fail，两腿逐键同态。
- 直证（新旧 leg log）：s582 leg1 `Timeout …detailed
  .optional.html :: <页级键>` vs s583 leg1 同页 4 条
  `Fail <子测键>`；余 5 页级 T 键（style-error-01/
  audio-loading-lazy/sizes-auto/update-media/video-
  loading-lazy）两轮逐键恒＝非全局计时漂移，单页解锁。
- 候选归因：**c503a585e overlay 增量合并与 NA 采集收窄
  （DOM 变异超线性热点修复）**——变异重的 detailed 页
  此前触页级 90s 执行超时（s578 在案「band page 级 90s
  Execution timeout 形」），热点移除后页完成并上报子测
  （子测 Fail＝键盘导航未实现语义，非回归）。单窗相关
  归因，机制一致；定谳可后补定向 A/B（暂不扩轮）。
- 形态谱记：谱内新形＝**聚合形**（8 子测 Fail、页级键
  全消、双腿恒）；腿全同先例＝s577 并存形回带轮（双整
  文件键同相，故聚合形为新形而非腿同态首例）；序列
  s577 并存形→s578-s579 第 4 形（整文件 Fail）→s580
  并存形→s581-s582 错开分裂 2 连→**s583 聚合形**。
  band566（5 键）跟踪集退役 → **band8（8 子测键）跟踪
  集 s584 起**。

## 全桥 b582 对账：零噪声兑现（改判解消）

- keys **1731 fail=0 NEW=0 GONE=0**（rcb=0，1731 passed
  22.05s；2501 filtered）＝b582 恒形。d63 caret 双钉
  （callbacks.rs+css_wire.rs）**未改桥测键面**（行为内
  部通道，无新 pin）——「或现 NEW/GONE」改判预期解消，
  按实测记零噪声。
- **b583≡b582 逐字节恒**（md5 66995b33 双侧同）＝s584
  桥基线无需换装实质（装载改名即用）。

## runner 换代与编译档

- pre a528acd8/128,664,848（s582 值）→ mid=post
  **ae5baec4/128,917,856（+253,008）**＝必换代 ×12 连；
  mid=post 同值＝重建稳定。跨 clone 收敛候选挂对端申报
  （对端 kb87/kb88 候窗；先例序列 s561 69967b2e／s579
  f37f570e／s580 0b5c2914／s581 95926afc／s582 a528acd8）。
- leg1 Compiling ×6／Fresh ×0；leg2 Compiling ×0／Fresh
  ×0＝engine+下游六档精确（低于 s581 三码刀 ×9 全链档，
  与「Compiling 档数预期少于全链」预判同向）。
- rc leg1=2/leg2=2/bridge=0 惯例；爆栈 0/0/0 惯例。

## 七哨 + 伪影 + solo

- dialog r17：2P 双腿（16→17 连）。
- select-event r22a：279 键 8F，F-set ≡ se.577 双腿 IDENT。
- password r22b：225 键 0F 双腿。
- **mpreload r45：15/15 P 双腿＝持位第 8 连兑现**。
- waapi r104：picker P／checkmark F 恒形双腿——**WAAPI
  face 三码（ea98cb8ae/9996cbebf/b1562f1d2）落窗零响应**
  （伪元素动画键语义面未动）。
- focus-dynamic r63：121/121 P 双腿。
- onblur r15：10 键 8P/2F，F-set ≡ ob.s559 双腿 IDENT。
- 双发伪影 2 键双腿各 ×2 恒（dup(2) 计数行核）。
- solo ×2（FILTER=select-home-end-pagedown-pageup.optional，
  s578 定规）：2/2 发射形 IDENT＝**第 5 连**（4 子测 Fail
  恒形；单文件 11s/10s）；第 5 形未现（聚合形本体已入
  sweep stable，solo 通道恒形续观察）。

## s584 预判与布防

- 对账基线换装 **s583 stable 8517**（聚合形 8 子测 Fail
  为新常）；全桥基线 **b583**（≡b582 逐字节）。
- /tmp/s584 已预装载（stable keys/canon＋b583＋band8＋
  七哨 keys＋全套脚本 s584 化）；开工核 diff 后即可臂
  waiter（超时按 s580 预案重臂，kill 旧实例防 double-fire）。
- runner 预判按窗码面定（纯 docs 窗＝Fresh；带 engine/
  下游码面＝必换代）；窗带 js_dom_bridge 新钉则全桥
  NEW/GONE 对账（b583 基线）。
- 族带 r17＝**聚合形持位观察**（8 子测 Fail 双腿恒＝聚合
  持位；再现页级 T 或新形＝谱内摆动重开）；solo 发射形
  第 6 连观察；mpreload r46＝第 9 连观察。
