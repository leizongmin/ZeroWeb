# M2 s579 — **出带回摆轮（族带第 12 轮第 3 连未达成：band 5 键全跌出 stable＝GONE 5，leg1 双整文件 T 形 vs leg2 第 4 形×发射形双腿分裂不收敛＝谱内摆动结案候选触发兑现）+ 第 4 形 sweep 常规腿再现轮（leg2 detailed 整文件 Fail＝page 级 90s Execution timeout；solo 窄语境 2/2 未再现）+ runner 必换代 ×8 连兑现轮（pre 8c621236→mid=post f37f570e；engine 单 crate ×6 档精确兑现；跨 clone 同值收敛 ≡ rendering 流 R5833/R5834 申报值）+ 等窗接管轮（前轮 21:41 布置 waiter，压窗等待 64 分钟 22:45:05 开窗自放）+ solo 谱形第 5 形未现轮（2/2 发射形 IDENT 复现，FILTER= 变量传递定规首执行）+ 七哨持位轮（mpreload 第 4 连）**：join **stable 8509（8305P/198F/5T/1U）disagree 1 + l1only 1 + l2only 4**；vs s578 stable（8514，8305P/202F/6T/1U）**NEW 0 + GONE 5 + flips 0**＝非带键逐键零位移。

日期：2026-11-19（s578 收口同日第八轮；
跨 session 接管续轮：前轮 21:53 API
限额断（布置 /tmp/s579 三件套＋
waiter 21:41 臂毕）；本 session
22:07 接管：pull 捕获核实＋核对
waiter/基线/哨资产在位（s578
stable 8514 净版直写无漂移，diff
0）→ 压窗等待（peer test-guard
uievents 12min＋构建风暴 load 峰值
8.75）→ 22:45:05 clean-soaked
（procs=0 load<3 3×45s）HEAD=
0c14ef2e2 钉扎起腿 → leg1
22:45:05–22:48:16（3:11，rc=2 惯
例）→ leg2 22:48:16–22:50:52
（2:36，rc=2 惯例）→ analyze 22:51
→ solo1 22:53:29–22:54:08 ＋ solo2
22:54:08–22:54:47（FILTER= 变量传
递，rc=2 ×2）→ 收口。

## 窗底账（s579 窗＝bee607db9→0c14ef2e2）

- 窗＝**6 commits：code ×1＝
  2daef5ede fix(engine) 刀 G2-a set
  门 number-unit famName 族表直取
  （js_dom_shim/part10-typed-om.js
  +11/-3，对端 wptfyi-css 流，
  css-typed-om 净 +25）＝**shim 行
  为面非桥测钉**（git show --stat
  核：零 js_dom_bridge 路径）→ 全
  桥对账不触发，**b570（1728P/0F）
  基线持位**；docs ×5＝本流 s578
  收口 9fe38a085＋rendering
  R5832/R5833/R5834＋wpt.fyi
  weekly snapshot 5a5e34a7c。
- **runner 必换代 ×8 连兑现**：pre
  8c621236（128538096）→mid=post
  **f37f570e（128538248，+152）**
  ；leg1 Compiling ×6（
  product-version/net/engine/
  page-runtime/webview/wpt-runner
  ）＝**engine 单 crate 档精确兑现
  **（2daef5ede 实改面 engine）；
  leg2 Fresh ×0；爆栈 0/0；rc=2/2
  惯例。
- **跨 clone 同值收敛第二实例**：
  mid=post f37f570e ≡ rendering 流
  R5833/R5834 申报 runs f37f570e
  （×3/×4 恒）＝两 clone 同码态同
  工具链位产同 md5（s561 69967b2e
  先例第二例）。
- **等窗接管注记**：前轮限额断前
  已臂 waiter（21:41），peer
  test-guard uievents（d61-base
  runner，2400s 档）＋对端构建风
  暴（load 峰 8.75）压窗 64 分钟，
  22:45:05 三判 clean 自放＝s577
  预案「重臂不必要、既有 waiter
  持轮」路径首兑现。

## 双腿 join（HEAD 0c14ef2e2 树）

- raw canon：leg1 **8513**（uniq
  8511＝双发伪影 ×2）／leg2
  **8516**（uniq 8514＝双发伪影
  ×2）＝**双腿非 IDENT（band 相位
  分裂轮）**。
- join：**stable 8509（8305P/198F
  /5T/1U）+ disagree 1（detailed
  整文件键）+ l1only 1（optional
  整文件键）+ l2only 4（optional
  4 子测键）**——vs s578 stable
  （8514）：P 8305 恒；F 202→198
  （−4＝optional 4 子测）；T 6→5
  （−1＝detailed 整文件）；U 恒；
  **NEW 0 + GONE 5 + flips 0**＝位
  移全落 band 5 键，非带键逐键零
  位移。

## 族带第 12 轮：出带回摆＝第 3 连未达成（持位止于第 2 连）

- leg1 band 形＝**双整文件 T 形**
  ：detailed 整文件键 Timeout（
  s577/s578 持位形）＋optional 整
  文件键 Timeout＝**s554
  harness-no-report 形**（
  testharness completed without
  reporting，state={loaded:false,
  pending:4,phase:4,tests:4}），4
  子测未发射。
- leg2 band 形＝**第 4 形×发射形
  并存**：detailed 整文件键 Fail（
  page script threw declared tests
  : 0：Execution timeout:90000ms）
  ＋optional Home/End/PageUp/
  PageDown 4 子测 Fail 发射。
- **band 5 键双腿全出 stable（
  GONE 5）＝完全出带回摆轮**：
  r11/r12 既定判定「双腿恒延续＝
  码面候选信号；**回摆＝谱内摆动
  结案候选**」——回摆兑现，**d59
  码面假设出局**（子系统错配＋跨
  s577/s578 两代 runner 恒后第 3
  代回摆＝码面相关性证伪方向），
  band 定谳**谱内相位摆动结案候选
  **（context-phase，s574 先例谱内
  语境相位）。
- **深化归因实验未执行＝双重依据
  结案**：①第 3 连判定条件未触发
  （实验前置不成立）；②pre-d59
  runner 工件核实无存档（/tmp 全
  存档 20+ 二进制 md5 扫描 0 匹配
  29e1d30d/8ffe9a69；pre-d59 二进
  制 s576 solo 16:20 重建覆盖即无
  存档）＝「复用旧产物不重编旧树
 」路径不可达。若后续需码面定谳
  只剩旧树重编（worktree 冷编译高
  成本）——不立项，候用户门。
- **PageDown 发射观察收束**：本轮
  leg2 4 子测全 Fail 发射（含
  PageDown）＝发射相位延续；leg1
  未发射——发射/静默随语境摆，无
  独立 PageDown 单发射形再现。

## 第 4 形追踪兑现：sweep 常规腿再现，solo 窄语境不复现

- detailed 整文件 Fail（page 级
  90s Execution timeout）＝s578
  leg3 计划外全量首现后，**s579
  leg2 常规 sweep 腿再现**＝非一次
  性形，**sweep 语境在册形**。
- **solo/窄语境 2/2 未再现**（
  solo1/solo2 皆发射形，零 page
  script threw）＝第 4 形 sweep 语
  境相关性成立候选；leg2 同腿另见
  select-picker-popover-open-
  invalid.tentative 整文件 Fail（
  ReferenceError: test_invalid_
  selector is not defined）＝持位
  恒形非新形（stable F 桶在册键，
  双腿恒）。

## optional solo：2/2 发射形 IDENT＝谱形轮转第 5 形未现

- solo1＋solo2（**FILTER=
  select-home-end-pagedown-pageup
  .optional 变量传递定规首执行兑
  现**，仅主文件）＝**两跑发射形
  IDENT**（Home/End/PageUp/
  PageDown 4 子测 Fail ≡ sweep 键
  ），rc=2 ×2，39s+39s，runner
  f37f570e 同值。
- **谱形轮转第 5 形未现＝发射形
  复现轮**：在册四形（s574 静默／
  s576 发射／s554 整文件 T（s577
  全轮＋s578 solo1）／s578 leg3 整
  文件 Fail page-90s）本轮回到发
  射形（s576 先例同形）；sweep 双
  腿分裂（T 形 vs 第 4 形×发射）
  与 solo 双跑恒定（发射 IDENT）
  并存＝语境相位谱继续展宽。

## 观察哨复核（七哨持位全绿＋mpreload 第 4 连兑现）

- dialog 哨：**第 13 轮 key1/key2 双
  P 持位**。
- password 哨：**第 18 轮持位兑现
  **（select-event 279 键 8F 恒形，
  F 集 ≡ se.577 双腿逐键 IDENT；
  password 225 键 0F 双腿）。
- onblur 哨组 10 键（第 11 轮）：
  **8P/2F 恒**——F 集 ≡ ob.s559 双
  腿 IDENT。
- modulepreload 哨：**15/15 双 P
  全绿——第 41 轮＝回带后持位第 4
  连兑现**（s575 首红→s576 回带→
  s577–s579 续绿）。
- waapi 哨 2 键：**第 100 轮**
  picker-icon 双 P（第 100 轮）/
  checkmark F/F 恒形。
- focus-dynamic 121 键 stable 全
  Pass：第 59 轮零再现。
- 双发伪影 2 键（wrapping exact
  len＋149.html for=' WINdow）双
  腿各 ×2 恒；rc=2/2 惯例 stack
  overflow 0/0。

## 下一步（s580）

- 守成巡查续轮：**对账基线换 s579
  stable 8509（出带后首基线，带键
  5 键不在基线内）**；全桥 b570
  （1728P/0F）持位；开工 pull 捕
  获全列核实（窗＝0c14ef2e2→
  s580 legs HEAD，本笔 push 捕获归
  s580 开工续核）；runner 预判
  docs 面为主＝Fresh 预期＋mtime
  churn 单列备注；窗带
  js_dom_bridge 新钉则全桥 NEW/
  GONE 对账（键基线 b570.keys 在
  /tmp/s572）；dialog 第 14 轮；
  password 第 19 轮；**mpreload 第
  42 轮＝持位第 5 连观察**；waapi
  第 101 轮；focus-dynamic 第 60 轮
  ；onblur 组第 12 轮；**族带第 13
  轮＝出带后首轮回带观察**（五谱
  形跟踪：s577 并存形回带／发射／
  s554 整文件 T／第 4 形／出带持
  位；双腿恒回带＝并存形回持信号
  ，谱内摆动结案维持）；solo ×2
  记形续（FILTER= 变量传递定规）
  ；第 4 形语境追踪续（solo 窄 vs
  sweep 双腿）；双发伪影恒；爆栈
  0/0 惯例；起腿三件套照规（基线
  换装净版直写；peer 构建风暴时按
  s577 预案重臂 waiter，kill 旧实
  例防 double-fire）。
- 候门决项等待用户拍板维持（video
  P1+P2 立项 / 簇 D A2 立项 / s137
  ；对端 ZRG-2026-11-18-01 归用户
  ）。

账本=evidence/2026-11-19-m2s579-band-swing-out-4th-form-sweep-recur-mpreload-hold4.md
