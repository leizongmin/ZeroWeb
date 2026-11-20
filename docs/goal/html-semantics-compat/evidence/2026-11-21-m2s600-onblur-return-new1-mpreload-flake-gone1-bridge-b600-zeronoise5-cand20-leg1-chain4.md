# M2 s600 — **onblur「following label」Checkboxes 变体双腿 Pass 归位兑现＝NEW=1 回账（s599 flake 定谳终验）** + **mpreload「same-origin full referrer」单键单腿翻形 GONE=1＋disagree=1＝翻形定谳 flake（三证在案；leg1 norm c735d371 ≡ s599≡s598≡s597 腿恒整腿复现第 4 连，全轮位移被单条 leg2 行包含）** + **全桥 b600 零噪声第 5 连兑现轮（keys md5 dbf4ce88 ≡ b599 逐字节，NEW=0 GONE=0 fail=0 21.41s）** + runner 换代兑现轮（pre 35460320→mid=post **bed95e43**/129,346,152 +224＝**候选二十实例挂号**；leg1 Compiling ×8＝engine 下游全链＋wpt-runner 入链／leg2 零编译；桥测即跑于新代）+ 族带聚合形持位第 17 连兑现轮（band r33 hits=8×2）+ mpreload 持位第 25 连破面（翻形键同源）+ onblur r32 归位兑现轮（8P/2F Fset 双 IDENT）+ solo 发射形第 22 连兑现轮（4 Fail ×2 failsets IDENT rc=2 ×2）+ **警备史首例：前 session 因 API 限流终止后，分离 waiter/post 链零人工介入全自动兑现全程（arm 01:24 → all.done 01:42:54）**。

日期：2026-11-21（s599 收账/push 补账后接续轮；前 session 01:23:53 提交
3c82bb389 并完成 /tmp/s600 布防，01:37 前后因 API 限流终止；waiter/post
链以分离进程存活并全自动兑现，本 session 仅收账）：
布防 s600 三件套（/tmp/s600 预装载单基线 s599.stable 8516＋b599 换装；脚本
s600 化＋哨 r 注记 bump＋chmod +x＋join 前缀审计照规；waiter＋post.sh 分开
臂）。**候窗 ~12m**＝01:24 臂后对端批歇，01:36:32 clean-soaked（procs=0
load<3 across 3x45s）HEAD=**3c82bb389** 钉扎起腿 → leg1 01:36:32–01:39:32
（3:00，Compiling ×8）→ leg2 →01:41:28（1:56 零编译）→ 桥测 21.41s rcb=0
→ post 链 → solo ×2（11s ×2，rc=2 ×2）→ all.done 01:42:54；窗起至全毕
6m22s；**前 session 终止后零人工介入**。

## 窗底账（s600 窗＝d043d6f19→3c82bb389，7 commits）

- 对端 ×5＝码面 ×2（23bd3725b MOTION-1b css-motion 净 +22 engine＋
  style-system **逐文件核实不触 js_dom_bridge**＋f53710dfc
  CALC-SIZE-DEFINITE calc-size() definite-sum 尺寸槽 +41 style-system）
  ＋docs ×3（791de4fa4＋fcfee27e0＋92475c7f1）；本流 docs ×2（a6e9cc382
  s599 收账＋3c82bb389 s599 push 补账）。全列已于 s599 push 补账逐 commit
  核实；运行时零新捕获。
- runner 换代触发源＝MOTION-1b/CALC-SIZE-DEFINITE 两刀 → leg1 Compiling
  ×8（product-version/style-system/net/layout-engine/engine/page-runtime/
  webview/wpt-runner）＝engine 下游全链＋wpt-runner 入链；leg2 零编译。

## 双腿 join（HEAD 3c82bb389 树）＝**单键单腿翻形换位轮**

- raw canon：leg1 **8519**／leg2 **8519**（双发伪影 ×2 各腿恒）。
- join：**stable 8516（8304P/206F/5T/1U）+ disagree 1 + GONE 1 + NEW 1 +
  FLIPS 0 + l1only 0 + l2only 0**。
- **NEW=1（归位兑现）**＝`checkable-active-onblur-with-click.html ::
  Checkboxes should clear :active … while pressing the **following**
  label with a pointing device`：s600 双腿 Pass 回账＝s599 预判「该键
  双腿 Pass → NEW=1」精确路径兑现，s599 flake 定谳终验。
- **GONE=1＋disagree=1（同键新翻形）**＝
  `modulepreload-referrerpolicy.html :: Modulepreload with
  **same-origin** policy should send full referrer for same-origin
  requests`：s600-l1 Pass／s600-l2 Fail，键出 stable 基线；r1→r62 哨史
  首翻（此前持位 24+ 轮 15/15P；本轮 l1 仍 15/15P、l2 14P+1F）。
- **翻形定谳三证**：①同 binary 双腿（mid=post 恒 bed95e43）单腿翻形＝
  非确定性执行，码致确定性回归必双腿同翻；②**leg1 norm md5 c735d371
  逐字节 ≡ s599≡s598≡s597**＝leg1 整腿复现第 4 连（/tmp/s599/s599-l1
  .norm cmp 同源验证），全轮位移被单条 leg2 行包含（leg2 norm 0e3e06a0
  ＝与 s599-l2 恰换 2 行：onblur Fail→Pass＋mpreload Pass→Fail）；
  ③窗口两刀（css-motion offset-rotate／calc-size 尺寸槽）均不触
  modulepreload/referrer/网络面，同文件 14 兄弟键双腿 Pass 恒。归因＝
  **referrer 网络行为时序非确定性 flake**；s601 归位观察（双腿 Pass →
  NEW=1 回账定谳兑现；双腿 Fail → 升级真回归面排查；再单腿 → 摆动记形）。
- 实质链：s600.stable.canon md5 **2f618513**（≠80d67629＝断链第 2 轮，
  s599 断于第 13 连后未复位）；keys md5 57add179（≠0a1b9a63）。基线换装
  s600.stable **8516**（canon 2f618513／keys 57add179；mpreload 翻形键
  已出基线、onblur 归位键以 Pass 态入基线）。

## 全桥 b600 对账：零噪声第 5 连兑现

- keys **1734＝md5 dbf4ce88 ≡ b599 逐字节，NEW=0 GONE=0 fail=0**（21.41s
  rcb=0）。窗两刀不触 js_dom_bridge＝主预判零噪声兑现。b601 基线沿用
  **b600（≡b599 keys 1734 fail=0 全绿锚）**。

## runner 换代兑现与候选二十挂号（重点账）

- pre 35460320/129,345,928（候选十九值）→ mid=post **bed95e43**/129,346,152
  （+224）＝码面 ×2 换代兑现（s599 push 补账预判「必换代候选二十」精确
  兑现）。
- 桥测即跑于新代 bed95e43（bridge.runner.md5 同值）＝候选二十首测零噪声。
- **收敛候选二十实例＝本侧本代新值 bed95e43/129,346,152**，挂对端后续
  申报值对读（候选十六 a5b38f6e／十七 d9933fdc／十八 382131d2／十九
  35460320 对读随其申报窗口滚动）。

## 七哨 + 伪影 + solo

- dialog r34：2P 双腿。
- select-event r39a：279 键 8F，F-set ≡ se.577 双腿 IDENT。
- password r39b：225 键 0F 双腿。
- **mpreload r62：l1 15/15 P／l2 14P+1F＝持位第 25 连破面**（翻形键同源，
  见上定谳段）。
- waapi r121：picker P／checkmark F 恒形双腿（两刀无翻形）。
- focus-dynamic r80：121/121 P 双腿。
- onblur r32：10 键 8P/2F 双腿，Fset ≡ ob.s559 双腿 IDENT＝**s599 组史
  首现组内变体摆动归位兑现轮**（following-label Checkboxes 变体双腿
  Pass 回位）。
- 双发伪影 2 键双腿各 ×2 恒；dup 检查仅此 2 键。
- band r33：双腿各 hits=8 全 Fail 逐键同态，页级 T 零再现＝**聚合形持位
  第 17 连兑现**（两刀不触 select 键盘域）。
- solo ×2：4 Fail ×2 **failsets IDENT**（Home/End/PageUp/PageDown）＝
  **发射形第 22 连兑现**；rc=2 ×2。
- rc leg1=2/leg2=2/bridge=0；爆栈 0/0/0 惯例。

## s601 预判与布防

- **单基线布防**：对账基线 **s600.stable 8516**（canon md5 2f618513、
  keys 57add179；mpreload 翻形键已出基线、onblur 归位键以 Pass 态在
  基线）；全桥基线 **b600（≡b599 keys 1734）**；/tmp/s601 预装载全套
  （脚本 s601 化：analyze.sh 基线 s600.stable、bridge-extract b600.keys、
  哨 r 注记 bump：dialog r35／select-event r40a／password r40b／
  mpreload r63＝**翻形键归位观察**／waapi r122／focus-dynamic r81／
  onblur r33＝**following-label 变体续观察（已入基线 Pass，单腿翻 →
  GONE=1）**／band r34＝聚合形持位第 18 连观察／solo 发射形第 23 连
  观察；生成后 chmod +x＋join 前缀审计照规）。
- join 预判（**mpreload 翻形键回归路径主导**）：s601 双腿 Pass → 键
  回账 **NEW=1**（对 8516 基线）＝归位定谳兑现；双腿 Fail → NEW=1
  （Fail 态入基线）＝升级真回归面排查；再单腿翻 → disagree=1 持续
  摆动。次级 flaky 候选＝onblur following-label 变体（单腿翻 → GONE=1
  ＋disagree=1）。除该两键外 NEW/GONE/FLIPS 预判 0（窗码面按开窗 pull
  全列定；码刀 → 对应面归因照规）。
- runner 预判：s601 窗已见码面 ×1＝2b4300d26 css-parser
  font-feature-settings 槽 clippy 清理（非语义结果面；截至本收账
  HEAD=b24dc2066）→ **必换代候选二十一**（实例值 leg 时挂号）；
  docs-only 追加不变更该预判。
- 全桥 b601＝**零噪声第 6 连预判**（b596 起五连；窗带 js_dom_bridge
  新钉则条件观察逐键归因，b600 基线 1734）。
- **收敛对账挂起＝候选二十 bed95e43 挂对端申报值对读**（十六/十七/
  十八/十九同滚）。
- 三件套照规：开工 pull 捕获全列核实（完整 fetch 范围逐 commit 分类；
  窗＝3c82bb389→s601 legs HEAD）；waiter＋post.sh 分开臂、绝对路径
  起动＋臂后 ps 验证自排除串；对端 contention 浸泡窗可长等＝门拒窗非
  自锁勿 kill 重臂（s598 1h53m50s 长窗／s599 1m40s／s600 ~12m 先例；
  **s600 已证限流终止后分离进程全自动兑现＝布防后无需 session 在位**）；
  超时按 s580 预案重臂。
