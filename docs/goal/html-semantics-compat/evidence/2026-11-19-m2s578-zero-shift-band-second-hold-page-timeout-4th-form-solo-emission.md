# M2 s578 — **限额断点接管轮（前轮 17:46 API 限额断，本 session 21:16 接管）+ 完全零位移轮（vs s577 stable 8514 NEW 0/GONE 0/flips 0，双腿 raw 8516/8516 IDENT）+ runner 必换代 ×7 连兑现轮（pre df57d278→mid=post 8c621236；engine 单 crate ×6 档精确兑现，mtime churn 未触发）+ 族带第 11 轮并存形持位第 2 连兑现（跨代恒注记）+ solo 谱形轮转第 4 形兑现（整文件 T→发射形）+ 计划外 leg3 整文件 Fail 新第 4 形（page 级 90s Execution timeout）+ 七哨持位轮**：join **stable 8514（8305P/202F/6T/1U）disagree 0 + l1only 0 + l2only 0**；vs s577 stable（8514）**NEW 0 + GONE 0 + flips 0**，逐键零差。

日期：2026-11-19（s577 收口同日第七轮；
跨 session 断点接管：前轮 session
17:27 布置 /tmp/s578（基线净版直写
s577 stable 8514）→ waiter 17:29:22
v3.2 判净（procs=0 load<3 3×45s
soak）HEAD=bee607db9 钉扎起腿 →
leg1 17:29:22–17:32:14（2:52，rc=2
惯例）→ leg2 17:32:14–17:34:31
（2:17，rc=2 惯例）→ analyze 17:41
自治 → solo1 17:42–17:43 → **17:46
API 限额断** → 本 session 21:16 接
管续：哨复核＋solo2 narrow（21:27
–21:28）＋leg3 计划外全量（21:21
–21:25， solo2 误记先于 solo2 narrow
）＋收口。

## 窗底账（s578 窗＝3519e00ed→bee607db9）

- 窗＝**7 commits：code ×1＝
  b449da8f0 fix(engine) 刀 ARITH
  typed-om CSSNumericValue 算术
  API+类型代数（js_dom_shim/
  part10-typed-om.js +197，对端
  wptfyi-css 流，numeric-objects 净
  +84）＝**shim 行为面非桥测钉**
  （git show --stat 核：零
  js_dom_bridge 路径）→ 全桥对账不
  触发，**b570（1728P/0F）基线持
  位**；docs ×6＝rendering
  R5830/R5831＋wptfyi-css 五笔（刀
  ARITH 收口系）＋本流 s577 收口
  bee607db9。
- runner 预判＝必换代（engine 单
  crate 码面）——**必换代 ×7 连兑现
  ：pre df57d278（128530096）→
  mid=post 8c621236（128538096）**
  ；leg2 Fresh ×0；爆栈 0/0；rc=2/2
  惯例。
- **档位精确兑现＋mtime churn 未触
  发**：leg1 Compiling ×6（
  engine/net/page-runtime/
  product-version/webview/
  wpt-runner）＝engine 单 crate 非
  全链档精确命中；css-parser/
  style-system/layout-engine 零
  Compiling 行——s577 注记的「docs
  rebase 全树触 mtime churn」本轮未
  再现（pull 后树为快进态，无全树
  checkout），s579 预判档位继续以
  code 面为主、mtime churn 单列备
  注。

## 双腿 join（HEAD bee607db9 树）

- raw canon：leg1 **8516**/leg2
  **8516**＝**双腿逐键 IDENT（完全
  收敛二连）**；双发伪影 2 键双腿各
  ×2 恒 Pass 形（wrapping-
  transformation exact len＋149.html
  for=' WINdow）。
- join：**stable 8514（8305P/202F/6T
  /1U）+ disagree 0 + l1only 0 +
  l2only 0**——vs s577 stable（8514
  ）：四桶全恒（P 8305/F 202/T 6/U
  1），**NEW 0 + GONE 0 + flips 0
  ＝逐键完全零位移轮**（s573 以来首
  个 NEW/GONE/flips 三零轮）。

## 族带第 11 轮：并存形持位第 2 连兑现（跨代恒注记）

- 双整文件键双腿恒延续：detailed 整
  文件键 Timeout ×双腿＋optional
  Home/End/PageUp/PageDown 4 子测
  Fail ×双腿（**PageDown 双腿发射延
  续**）——r11 判定条件「双腿恒延续
  ＝d59 码面候选信号」兑现，**并存
  形持位第 2 连**。
- **跨代恒注记**：本轮 runner 换代
  （df57d278→8c621236，ARITH face
  b449da8f0）下 band 键形不变——
  band 并存形跨两代 runner 持位，
  d59 码面归因进一步弱化（码面假设
  弱化＋谱形语境相位假设增强）。
- **深化归因挂 s579**：若第 12 轮双
  腿恒延续（第 3 连），启动码面假设
  检验对照实验（d59 pre-face runner
  复跑 band 窄 filter 对照；对照面
  需临时 runner 复用旧产物，不重编
  旧树）。

## optional solo 谱形轮转第 4 形兑现

- solo1（前轮 17:42–43，s574 同款窄
  filter，runner 8c621236 fresh）＝
  **整文件 Timeout harness-no-report
  s554 形**（state={loaded:false,
  pending:4,phase:4,tests:4}），rc=2
  。
- solo2 narrow（本轮 21:27–21:28，
  38s，runner 8c621236）＝**发射形
  ：4 子测 Fail（Home/End/PageUp/
  PageDown，≡ sweep 双腿键）**，rc=2
  。
- **谱形轮转第 4 形兑现**：solo 语
  境在册形累计四形——s574 静默 /
  s576 发射 / s554 整文件 T（s577 全
  轮＋s578 solo1）／发射形（s578
  solo2）。**同 runner 同 face 连跑
  两形**（solo1 整 T→solo2 发射）＝
  solo 语境自身不稳定谳加强，非码面
  非文件内在行为（s574 先例延续）。

## leg3 计划外全量第三跑（21:21–21:25）：整文件 Fail 新第 4 形

- 误记在案：solo2 首跑以裸 goal 参数
  传 filter（`make
  testharness-html-semantics
  select-home-end-pagedown-pageup
  .optional`），Makefile FILTER 变量
  未透传→全量 8514 键跑（**make 入
  口＋test-guard 包裹照规，非裸跑
  runner**）；s579 起定规：filter 一
  律 `FILTER=` 变量传递。
- 结果：8513 键 8307P/200F/5T/1U，
  vs stable **NEW 1 + GONE 4 + flips
  1**——**band 双文件整文件 Fail＝新
  第 4 形**：两文件 wholefile Fail 均
  带 `page script threw (declared
  tests: 0): Script error: page
  script: Execution timeout:
  90000ms`＝**page 级 90s 执行超时**
  （非 harness-no-report T 形）；
  detailed T→F flip 1、optional 4
  子测 GONE→optional 整文件 F NEW 1
  ；其余 8510 键与 stable 恒。
- runner 8c621236 **同值重建恒等象**
  （cargo 重编 md5 不变，R5830 先例
  同象）。
- 语境注记：21:21 host load ~2.6＋
  对端 mykernel cargo clippy 在飞
  （非本仓 target 锁）——context 相
  位非码面（s574 先例）；整文件 Fail
  形是否在 solo/窄语境再现挂 s579
  追踪。

## 观察哨复核（七哨持位＋mpreload 第 3 连全绿）

- dialog 哨：**第 12 轮 key1/key2 双
  P 持位**。
- password 哨：**第 17 轮持位兑现**
  （select-event 279 键 8F 恒形，F
  集 ≡ s577 逐键 IDENT；password
  225 键 0F）。
- onblur 哨组 10 键（常规连轮第 10 轮
  ）：**8P/2F 恒**——F 集 ≡ s559
  逐键 IDENT（双腿，ob1/ob2 各 2F
  同集）。
- modulepreload 哨：**15/15 双 P 全
  绿——第 40 轮＝回带后持位第 3 连**
  （s575 首红→s576 回带→s577/s578
  续绿）。
- waapi 哨 2 键：picker-icon 双 P（
  第 99 轮）/checkmark F/F 恒形。
- focus-dynamic 121 键 stable 全 Pass
  ：第 58 轮零再现。
- 双发伪影 2 键双腿各 ×2 恒；rc=2/2
  惯例 stack overflow 0/0。

## 下一步（s579）

- 守成巡查续轮：**对账基线换 s578
  stable 8514**；开工 pull 捕获全列
  核实（窗＝bee607db9→s579 legs
  HEAD）；runner 预判按 code 面＋
  mtime churn 单列备注（s578 未触
  发，快进态 pull 不全树触）；全桥
  b570（1728P/0F）持位——窗带
  js_dom_bridge 新钉则 NEW/GONE 对
  账（键基线 b570.keys 在 /tmp/s572
  ）；dialog 第 13 轮；password 第
  18 轮；**mpreload 第 41 轮＝持位第
  4 连观察**；waapi 第 100 轮；
  focus-dynamic 第 59 轮；onblur 组
  第 11 轮；**族带第 12 轮＝并存形
  持位第 3 连判定＋深化归因启动**
  （双腿恒延续→码面假设检验对照实
  验：d59 pre-face runner 复跑 band
  窄 filter 对照，临时复用旧产物不
  重编旧树；回摆＝谱内摆动结案候选
  ；PageDown 发射持续观察）；**
  optional solo 复跑记形（窄 filter
  ×2，FILTER= 变量传递定规，谱形
  轮转第 5 形观察）**；**band 整文件
  Fail（page 级 90s Execution
  timeout）形追踪**（leg3 新形是否
  在 solo/窄语境再现）；双发伪影恒
  ；爆栈 0/0 惯例；起腿三件套照规
  （基线换装净版直写）。
- 候门决项等待用户拍板维持（video
  P1+P2 立项 / 簇 D A2 立项 / s137；
  对端 ZRG-2026-11-18-01 归用户）。

账本=evidence/2026-11-19-m2s578-zero-shift-band-second-hold-page-timeout-4th-form-solo-emission.md
