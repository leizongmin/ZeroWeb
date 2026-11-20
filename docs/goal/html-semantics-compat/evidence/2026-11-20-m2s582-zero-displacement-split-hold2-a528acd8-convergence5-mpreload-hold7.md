# M2 s582 — **完全零位移轮（stable 集逐字节恒 s581＝NEW 0/GONE 0/flips 0）+ band 错开分裂持位第 2 连兑现轮（leg1 并存形 vs leg2 双整文件 T 形逐腿复刻 s581）+ b581 全桥基线首战零位移轮（keys 1731 fail=0 NEW=0 GONE=0）+ runner 必换代 ×11 连兑现＋跨 clone 收敛第五实例确认轮（pre 95926afc→mid=post a528acd8 ≡ 对端 kb83 重建值）+ solo 发射形复现第 4 连轮（2/2 IDENT，第 5 形未现）+ 七哨持位轮（mpreload 第 7 连兑现）**：join **stable 8510（8305P/198F/6T/1U）disagree 0 + l1only 4 + l2only 1**，stable.keys 与 s581 **cmp 逐字节恒**。

日期：2026-11-20（s581 收口同日凌晨第三轮；
跨 session 延续轮：前 session s581 收口
03:34 布防 s582 waiter＋post 链，守窗
约 76 分钟 → 本 session 05:0x 接管：
all.done 已立（post 链自放走完）→
核产 → 收口。开窗
**04:50:39 clean-soaked**（procs=0
load<3 3×45s）HEAD=2483ee96e 钉扎
起腿 → leg1 04:50:39–04:53:51（
3:12，rc=2 惯例）→ leg2
04:53:51–04:56:19（2:28，rc=2 惯例
）→ 桥测 34.22s rcb=0 → analyze →
solo1 04:58:12–04:58:51 ＋ solo2
04:58:51–04:59:34（rc=2 ×2，39s+
43s）→ 收口。

## 窗底账（s582 窗＝57aa5e0be→2483ee96e）

- 窗＝**12 commits：code ×1**＝
  d70660fae cargo fmt 修复 main 侧
  格式漂移（10 文件 79+/55−：
  css-parser ×6＋style-system
  apply.rs＋engine grid_cssom.rs
  ＝engine+css-parser 码面 →
  runner 必换代兑现）；test ×0；
  docs ×11＝本流 s581 收口
  28ef435c8＋s581 push 阶段补账
  9df93b217＋对端 rendering
  R5841/R5842/R5843/R5844 四笔＋
  wptfyi-css 池④/语料 css/support
  修复收账/ffix2 注记/WA face 落码
  纪要四笔＋PR #314 merge。
- 开工 pull 捕获 ×12 全列核实（
  fmt 笔码面 → 必换代预判 9df93b217
  改判入账；kb83 重建值 a528acd8 →
  本轮 runner 值收敛预判，均兑现）。

## 双腿 join（HEAD 2483ee96e 树）

- raw canon：leg1 **8516**（uniq
  8514＝双发伪影 ×2）／leg2
  **8513**（uniq 8512＝双发伪影 ×2
  ＋l2only 1）＝与 s581 逐轮同值同
  形。
- join：**stable 8510（8305P/198F
  /6T/1U）+ disagree 0 + l1only 4
  （optional 4 子测键）+ l2only 1
  （optional.html 整文件键）**——
  vs s581 stable（8510）：**NEW 0
  + GONE 0 + flips 0＝完全零位移
  轮**；**s582.stable.keys 与
  s581.stable.keys cmp 逐字节恒**
  （分裂形逐轮同态故 stable 集连
  字节序都未动，最强持位表述）。
  状态拆分 8305P/198F/6T/1U 全
  恒；l1only/l2only 键集逐键复刻
  s581。

## 族带第 15 轮：错开分裂持位第 2 连兑现

- leg1 band＝**并存形**（detailed
  整文件 Timeout＋optional Home/End
  /PageUp/PageDown 4 子测 Fail＝5
  hits＝s580/s581 leg1 持位形）。
- leg2 band＝**双整文件 T 形**（
  detailed 整文件 Timeout＋
  optional.html 整文件 Timeout＝
  band566 grep 1 hit＋l2only 1＝
  s554 harness-no-report 形）＝
  **与 s581 逐腿复刻＝错开分裂形
  持位第 2 连达成**（新组合首现
  s581，本轮守形）。
- s581 预判「双腿恒回并存形＝聚合
  信号」未现——双腿仍错开分裂，
  但分裂形本身逐腿恒＝错开形持位
  信号；detailed T 双腿恒＝
  stable 锚持位维持。谱内摆动结
  案维持（窗 fmt 笔非 select 键盘
  域，带形与码面无对应方向）。
- 第 4 形（detailed 整文件 Fail
  page 90s）本轮双腿零再现。

## 全桥 b581 基线首战零位移

- keys **1731**（b581=1731）＝
  **NEW 0 / GONE 0 / FAIL 0**；
  rcb=0（1731 passed; 0 failed，
  34.22s）＝换装基线后首战全绿
  零噪声兑现。
- fmt 笔 d70660fae 触
  css-parser/style-system/engine
  码面而桥测键集零响应（纯格式
  无语义）＝桥面稳态佐证。

## runner 必换代 ×11 连＋跨 clone 收敛第五实例

- pre **95926afc**（128665024）→
  mid=post **a528acd8（
  128664848，−176）**＝必换代
  ×11 连兑现；leg1 Compiling ×9
  （fmt 闭包 10 文件跨
  css-parser/style-system/engine
  ＝product-version/css-parser/net
  /style-system/layout-engine/
  engine/page-runtime/webview/
  wpt-runner）/leg2 Fresh ×0；
  爆栈 0/0/0；rc=2/2 惯例。
- **跨 clone 收敛第五实例确认**：
  a528acd8 ≡ 对端 kb83 重建值
  （R5842 报 fmt 笔重建象
  95926afc→a528acd8 ×3 恒，
  R5843/R5844 ×4 恒持位）＝两
  clone 同 fmt 树态独立产出同值
  （同树确定性 ×5：s561 69967b2e
  ／s579 f37f570e／s580 0b5c2914
  ／s581 95926afc／**s582
  a528acd8**）。

## optional solo：2/2 发射形 IDENT＝复现第 4 连

- solo1+solo2（FILTER= 定规）＝
  **两跑发射形 IDENT**（Home/End
  /PageUp/PageDown 4 子测 Fail ≡
  sweep leg1 4 键，rc=2 ×2，39s+
  43s，runner a528acd8 同值）。谱
  形轮转第 5 形未现（发射形复现
  第 4 连）；sweep 双腿错开分裂与
  solo 双跑恒发射并存＝语境相位
  谱续展宽。

## 观察哨复核（七哨持位全绿＋mpreload 第 7 连兑现）

- dialog 哨：**第 16 轮 key1/key2 双
  P 持位**。
- password 哨：**第 21 轮持位兑现**
  （select-event 279 键 8F 恒形，F
  集 ≡ se.577 双腿逐键 IDENT＋
  password 225 键 0F 双腿）。
- onblur 哨组 10 键（第 14 轮）：
  **8P/2F 恒**——F 集 ≡ ob.s559 双
  腿 IDENT。
- modulepreload 哨：**15/15 双 P
  全绿——第 44 轮＝持位第 7 连兑现
  **（s576 回带起 s577–s582 续绿）。
- waapi 哨 2 键：**第 103 轮**
  picker-icon 双 P／checkmark F/F
  恒形。
- focus-dynamic 121 键 stable 全
  Pass：第 62 轮零再现。
- 双发伪影 2 键双腿各 ×2 恒（dup
  计数行核：wrapping round-trips
  exact len＋149.html for='
  WINdow）；rc=2/2 惯例 stack
  overflow 0/0。

## 下一步（s583）

- 守成巡查续轮：**对账基线持
  s582 stable 8510（与 s581 逐字节
  恒，基线无换装必要）**；全桥基
  线换装 **b582（/tmp/s583/
  b582.keys＝s582.bridge.keys
  1731P/0F 全绿集，键集与 b581
  同值故预期零噪声）**；开工
  pull 捕获全列核实（窗＝
  2483ee96e→s583 legs HEAD）；
  runner 预判 **Fresh**（
  2483ee96e 纯 docs，s583 窗若无
  新码笔则 a528acd8 Fresh；对端
  kb86 候窗＝54代 持位＋ZW-3
  css_wire dirty=4 扩，若对端发
  码刀则改判必换代——开工捕获后
  定谳）；窗带 js_dom_bridge 新
  钉则全桥 NEW/GONE 对账；
  dialog 第 17 轮；password 第
  22 轮；**mpreload 第 45 轮＝
  持位第 8 连观察**；waapi 第
  104 轮；focus-dynamic 第 63 轮
  ；onblur 组第 15 轮；**族带第
  16 轮＝错开分裂聚合观察**（双腿
  恒回并存形＝聚合信号；错开形
  第 3 连＝分裂持位续证；detailed
  T 恒＝stable 锚持位）；solo ×2
  记形续（FILTER= 定规）；第 4 形
  语境追踪续；双发伪影恒；爆栈
  0/0 惯例；起腿三件套照规（
  waiter 超时按 s580 预案重臂，
  kill 旧实例防 double-fire）。
- 候门决项等待用户拍板维持（video
  P1+P2 立项 / 簇 D A2 立项 / s137
  ；对端 ZRG-2026-11-18-01 归用户
  ）。

账本=evidence/2026-11-20-m2s582-zero-displacement-split-hold2-a528acd8-convergence5-mpreload-hold7.md
