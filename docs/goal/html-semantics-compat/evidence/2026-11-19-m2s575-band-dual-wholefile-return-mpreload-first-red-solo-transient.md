# M2 s575 — **新 session 接管布置轮（waiter 在飞 34 min 判净）+ runner 必换代 ×4 连兑现轮（Compiling ×9 全链＝三 crate 预判精确兑现 41ce096c→eab9ca64）+ 族带双腿收敛轮（双整文件键 Timeout 并存形回归 NEW 2、leg1 三子测发射缺 PageDown）+ mpreload 哨 37 轮首红轮（referrerpolicy 键 leg P×F disagree 出带→solo ×2 全 Pass 定谳 sweep 时序瞬态非码面）+ s574 optional solo 静默形未复现轮 + 七哨持位轮**：join **stable 8510（8304P/198F/7T/1U）disagree 1 + l1only 3 + l2only 0（raw 双腿 8516/8513）**；vs s574 stable（8509）**NEW 2 + GONE 1 + flips 0**。

日期：2026-11-19（s574 收口同日第四轮；
新 session 13:03 滚入接管：/tmp/s575
全链前 session 已布置（waiter 12:56 起
在飞），接管后 pull 零捕获（55b61dd84
已是远端最新＝s575 窗即 s574 push 补
账时已全列核实的 ×5 批）→ 13:37:45
v3.2 判净（对端 d58 扫描＋ZeroSeed 侧
cargo 让位 34 min）HEAD=55b61dd84 钉
扎起腿 → leg1 13:37:45–13:41:09
（3:24，rc=2 惯例）→ leg2 13:41:09–
13:44:01（2:52，rc=2 惯例）→ analyze
13:44 自治 → solo 定相 ×4 → 哨复核。

## 窗底账（s575 窗＝b50817198→55b61dd84）

- 窗＝pull 捕获 ×5（code ×2＝cabfb6ce6
  fix(engine) font 车道三面＋dc3a5e01b
  fix(style-system,engine,css-parser)
  typed-om grid/logical+break 写门扩
  面 face A；docs ×3）＋本流 s574 两
  docs commit。
- runner 预判＝必换代全链档（engine＋
  style-system＋css-parser 三 crate
  码面）——×4 连兑现：leg1 Compiling
  ×9 全链 pre 41ce096c → mid=post
  **eab9ca64**；leg2 Fresh ×0；爆栈
  0/0。
- **零桥测钉**（cabfb6ce6/dc3a5e01b
  桥测 diff ×0）→ 全桥对账不触发，
  **b570（1728P/0F）基线持位**。

## 双腿 join（HEAD 55b61dd84 树）

- raw canon：leg1 **8516**/leg2
  **8513**；双发伪影 2 键双腿各 ×2 恒
  Pass 形。
- join：**stable 8510（8304P/198F/7T
  /1U）+ disagree 1 + l1only 3 +
  l2only 0**——vs s574 stable（8509）
  ：F/U 桶逐项恒，P 桶 8305→8304（
  −1＝mpreload 翻红键出带），T 桶
  5→7（+2＝双整文件键回带）。
- **NEW 2 + GONE 1 + flips 0**：
  NEW 2＝detailed＋optional 双整文
  件键 Timeout 回带；GONE 1＝
  modulepreload-referrerpolicy no-ref
  errer 键（哨面，非带）。

## 族带第 8 轮：双腿收敛＝并存形回归＋PageDown 零发射首现

- s574 分裂形未持续：双腿整文件键同
  相 Timeout（s572 并存形回归，stable
  面 2 现位）→ **升级路径不触发**。
- **leg1 三子测发射（End/Home/
  PageUp Fail×3）缺 PageDown＝
  PageDown 双腿零发射首现**（leg2 整
  文件发射零子测）；4 子测键未回带
  （3 l1only＋PageDown 零发射）。
- **勘误（s573/s574 账本记法更正）
  **：multiple:/single: 2 持位键属
  **customizable-select-listbox 兄弟
  文件**（select-listbox-home-end-
  pagedown-pageup.optional），非主
  optional 文件子测——「同文件」表述
  有误，本轮更正；两键双腿恒 Fail 跨
  四轮持位不变。

## mpreload 哨第 37 轮首红：solo 定谳 sweep 时序瞬态非码面

- **referrerpolicy no-referrer 键
  leg1 Pass × leg2 Fail disagree 出
  带**（15 哨键 37 轮首红）：leg2 断
  言形＝`assert_equals expected ""
  got "https://wpt.test/.../modulepre
  load-referrerpolicy.html"`＝no-ref
  errer 预载请求被断言到携带完整
  referrer。
- **solo 定相（FILTER=modulepreload-
  referrerpolicy ×2）＝全 7 键 Pass、
  两跑 IDENT、rc=0**——腿形不重现
  ＝**sweep 8-job 时序瞬态（断言与请
  捕获竞速），非码面**；窗 code ×2
  （font 车道/typed-om）零网络/
  preload 面佐证。
- 14/15 在带全 P 持位；挂 s576 回带
  观察（回 stable P＝瞬态收敛；持续
  出带＝深化归因）。

## optional solo 定相（filter 为宽 substring——与 s574 窄 filter 差异注记）

- 本轮 filter＝`home-end-pagedown-
  pageup.optional`（无 select- 前缀
  ＝substring 兼中 listbox 兄弟文件
  ；s574 solo 为 select- 前缀窄
  filter 仅主文件）——两轮 solo 不
  严格可比，注记在案。
- optional solo ×2＝**listbox 2 子测
  Fail（End expected option 49 got
  option 0）＋主文件整文件 Timeout
  （harness-no-report pending:4＝
  s554 在册形）×2 IDENT rc=1**——
  **s574 静默形未复现（2/2 发射）**
  ；s554 谱形 solo 语境首现（谱内
  形非新极）。
- detailed 键本轮双腿同相 Timeout 入
  带＝s573/s574 异相期消解（solo
  定相轮兑现后首轮回归）。

## 观察哨复核（七哨持位＋mpreload 首红）

- dialog 哨：**第 9 轮 key1/key2 双 P
  持位**。
- password 哨：**第 14 轮持位兑现**
  （select-event 279 键 8 非 P 恒形
  ；password 键 0F）。
- onblur 哨组 10 键（常规连轮第 7 轮
  ）：**8P/2F 恒**——F 集 ≡ s559
  逐键 IDENT。
- modulepreload 哨：**14/15 在带全 P
  ——第 37 轮首红（1 键 disagree
  出带，solo 定谳瞬态）**。
- waapi 哨 2 键：picker-icon 双 P（
  第 96 轮）/checkmark F/F 恒形。
- focus-dynamic 121 键 stable 全 Pass
  ：第 55 轮零再现。
- 双发伪影 2 键双腿各 ×2 恒；rc=2/2
  惯例 stack overflow 0/0。

## 下一步（s576）

- 守成巡查续轮：**对账基线换 s575
  stable 8510**；开工 pull 捕获批全
  列核实；runner 预判按 code 面＋同
  值重建规重估；全桥 b570（1728P/
  0F）持位——窗带 js_dom_bridge 新
  钉则 NEW/GONE 对账（键基线
  b570.keys 在 /tmp/s572）；dialog
  第 10 轮；password 第 15 轮；
  mpreload 第 38 轮（**翻红键回带观
  察**：回 stable P＝瞬态收敛定谳；
  持续出带＝深化归因）；waapi 第 97
  轮；focus-dynamic 第 56 轮；onblur
  组第 8 轮；**族带第 9 轮＝并存形持
  位判定**（双整文件键双腿恒＝持位
  ；回摆子测形＝谱内再摆；**PageDown
  零发射复现观察**）；**optional
  solo 静默形复跑（s574 同款 select-
  前缀窄 filter 对齐 ×2 记形）**；
  双发伪影恒；爆栈 0/0 惯例；起腿三
  件套照规（基线换装净版直写为首选
  ）。
- 候门决项等待用户拍板维持（video
  P1+P2 立项 / 簇 D A2 立项 / s137；
  对端 ZRG-2026-11-18-01 归用户）。

账本=evidence/2026-11-19-m2s575-band-dual-wholefile-return-mpreload-first-red-solo-transient.md
