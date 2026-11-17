# M2 s557 — **零码面轻覆盖轮 + 族带部分发射形 legs 首现轮（disagree 腿间分歧）**：窗内零码面刀（f677918c6→18486fa9a 全 docs-only ×1＝对端 rendering-compat R5764「selection 刀窗 reftest-中性认证＋runner 重编注记」收账 docs，非触发器），码面零变，runner 十一代 47699964/128,108,992 **恒**（leg1 Compiling ×6 确定性重编 md5/size 恒＝零换代，s551/s554 先例）——v3.2 浸泡门判净 22:49:50 HEAD=18486fa9a 钉扎起腿（**waiter 自锁两连根因修复后首跑**：①相对路径 `./wait-and-run.sh` 启动致自身 cmdline 不含自排除串 `/tmp/s557/wait-and-run`→check_clean 永数自己；②对端 CI 守护 node 进程 cmdline prompt 文本含「cargo/clippy」字样被 cmdline grep 误数——check_clean 改 v3.2 按 `ps -eo comm=` 进程名匹配重进程＋args 匹配自排除，两坑先后修复后判净），双腿常态全绿（leg1 3:29 含确定性重编/leg2 2:41 Fresh 同二进制，rc=2 惯例 0 爆栈）——join **stable 8513（8305P/201F/6T/1U）disagree 1 + l1only 0 + l2only 0**（raw 双腿恒 8516 行＝8514 键）——vs s556 stable **NEW 3 + GONE 1 + flips 0 + disagree 1，位移全落 customizable-select home-end 族带**：non-det 变体呈 **probe2 同款部分发射形 legs 首现**（End/Home/PageUp 子测 F×3 NEW 回潮入 stable＋PageDown 双腿零报告＋整文件键 Timeout 持位——超时消息形新变体「testharness page timeout completed the file with 1 test(s) unreported」pending:1＝s556 探针捕获的谱内中间态在 legs 全量负载下复现）；detailed 变体整文件键 GONE 1＝**disagree 1 腿间形态分歧**（leg1 harness-no-report pending:4 T 形 / leg2 90s 0-declared F 形——族带两超时模式腿间摆动，分歧键不入 stable）；**归因：窗零码面＋runner md5 恒＋双腿同二进制，位移纯族带时序谱内摆动（probe2 中间态 legs 复现＝谱内形态再添实证）——按 s556 收口新规族带位移回归常态记账，升级路径不触发**；password 11 键 stable 持位第 5 轮兑现

日期：2026-11-17（s556 收口同日第六轮；判净
22:49:50 起腿，legs 22:56:00 跑毕，本轮会
话内完成全流程——上轮收口 s556 后即备腿
起 waiter，本会话续轮修复 waiter 自锁后
判净＋双腿＋分析＋收口一气完成）。
承前：s556 收口轮（双刀窗归因 + 族带谱内
摆动探针定谳轮）。

## waiter 自锁两连根因修复（v3.1→v3.2）

- 坑①（相对路径自锁）：s557 waiter 以
  `cd /tmp/s557 && nohup ./wait-and-run.sh`
  相对路径启动——自身 cmdline `/bin/bash
  ./wait-and-run.sh` 含 `wait-and-run` 但
  不含自排除串 `/tmp/s557/wait-and-run`
  →check_clean 永远数到自己→静窗 ~80 分
  钟不判净（既有教训「waiter 绝对路径启
  动」复踩）。修复：kill 后以绝对路径
  `/tmp/s557/wait-and-run.sh` 重启。
- 坑②（对端守护 cmdline 文本误匹配）：对
  端 CI 守护 node 进程（rally query，pid
  57127）cmdline prompt 文本含「cargo
  fmt/clippy」等字样——v3.1 check_clean
  按 `ps aux | grep -E '...|cargo|rustc|...'`
  全 cmdline 匹配把它误数为重进程→绝对
  路径重启后仍静窗不判净（node 守护常驻
  ＝永久锁死）。修复：check_clean 升
  **v3.2**——重进程按 `ps -eo comm=` 进程
  名匹配（`^(cargo|rustc|cargo-clippy|
  clippy-driver|clippy|zero-wpt-runner|
  test-guard)`，node 守护 comm='node' 不
  命中），waiter/run-legs 自身仍按 args
  匹配＋绝对路径自排除。v3.2 首跑判净
  22:49:50 兑现。

## 窗底账（零码面）

- 窗 f677918c6→18486fa9a：18486fa9a（
  rendering-compat R5764 收账 docs——
  对端 selection 刀窗 reftest-中性认证＋
  runner 重编注记）全 docs-only 非触发器
  ；码面零变。
- runner 审计（刀 G 工法）：pre＝mid＝
  post 47699964/128,108,992（十一代恒）
  ——leg1 Compiling ×6 确定性重编（同
  s551/s554 先例：md5/size 恒＝零换代
  ），leg2 Fresh；provenance 零异常。

## 双腿 join（HEAD 18486fa9a 树，零码面）

- v3.2 判净 22:49:50 HEAD=18486fa9a 钉扎
  起腿。leg1 22:49:50–22:53:19（3:29 含
  确定性重编）/ leg2 22:53:19–22:56:00
  （2:41 Fresh 同二进制），rc=2 惯例 0
  爆栈，双腿时长常态带。
- raw canon：双腿恒 8516 行（8307P/201F
  /6T/1U＋双发伪影 2 行；唯一键 8514）
  ——双腿 raw 逐行同数；U 恒 1。
- join：**stable 8513 + disagree 1 +
  l1only 0 + l2only 0**（stable canon
  8305P/201F/6T/1U）。
- **vs s556 stable（8511）：NEW 3 + GONE 1
  + flips 0 + disagree 1**，位移全落族带：
  - **non-det 变体＝probe2 同款部分发射
    形 legs 首现**：End/Home/PageUp 子测
    F×3 NEW 回潮入 stable，PageDown 双腿
    零报告（norm 零行），整文件键 Timeout
    持位——超时消息形新变体「testharness
    page timeout completed the file with
    1 test(s) unreported」（state
    pending:1）＝s556 探针 probe2 捕获的
    谱内中间态在 legs 全量负载下复现，
    族带谱再添实证形。
  - **detailed 变体整文件键 GONE 1＝
    disagree 1 腿间形态分歧**：leg1
    harness-no-report（"testharness
    completed without reporting
    registered tests" pending:4，T 形）
    / leg2 90s 0-declared（"page script
    threw (declared tests: 0) Execution
    timeout: 90000ms"，F 形）——族带两
    超时模式腿间摆动，分歧键不入 stable
    ＝GONE 出账。
  - listbox 变体本轮探针 FILTER 外（legs
    内 F/F 恒带形不变，账目在 stable 内
    无位移）。
  - **归因判据**：窗零码面＋runner 十一
    代 md5 恒＋双腿同二进制——腿间分歧
    与跨轮位移必为时序态；probe2 中间态
    同二进制已实证（s556 探针）——**判
    族带时序谱内摆动非码面，按 s556 收
    口新规族带位移回归常态记账，升级路
    径不触发（探针工法已定谳谱内摆动，
    反事实 A/B 豁免维持）**。
- stable counts：P **8305 恒**（第 5 轮
  恒等）；F 199→**201**（+3 子测键回潮
  −1 detailed 出账）；T 6→**6 恒**（
  non-det 整文件 T 持位＋惰载 5 键：
  style-error-01/audio-loading-lazy
  -source-deferred/sizes-auto/update
  -media/video-loading-lazy-source
  -poster-deferred）；U 恒 1。
- password 11 键：select-event 两腿
  270P/0F 恒——**stable 持位第 5 轮兑
  现**。
- 双发发射伪影 2 键照旧在册（双腿恒）：
  wrapping-transformation.window.js ::
  exact len + execution-timing/149.html
  :: for=' WINdow。

## 观察哨复核（五哨持位 + mpreload 常规计数第 19 轮）

- onblur 哨组 10 键双腿 join 零 flip（8P
  /2F parent-label 红池照常，**F 集与
  s556 逐键 IDENT（diff 零行实证）**）
  ——flaky 键双腿双 Pass：零新例第 46 轮
  （s512 时钟顺延）。
- dialog-overlay 哨第 86 轮：双键双腿双 P
  常规持位。
- **modulepreload 哨：15 键双腿全 P/P
  ——常规连轮计数第 19 轮零翻红**（s556
  排程兑现）。
- waapi 哨 2 键：picker-icon 双腿双 P（
  s478 刀增益第 80 轮保持）/ checkmark
  F/F 恒 F 带形不变。
- focus-dynamic 121 键双腿全 Pass：收窗
  后第 39 轮零再现。

## 全桥套件绿窗收口跑

guard 包裹（--compile-first --per-proc-mem 8
--total-mem 16 --time-limit 1200）`cargo test
-p zero-engine --lib js_dom_bridge`（仓根执
行，guard rc=0）：**1694P/0F**（test 阶段
28.25s）。清单 vs s556（b556.k 1694）：
**SET-IDENTICAL（NEW 0 + GONE 0）**——窗
零码面零伴钉预期兑现。基线 b556.k→b557.k
（1694，set 恒等）。

## 对端观察

- 对端 rendering 流 R5764 收账 docs 落窗
  （docs-only 非触发器）——hs 面「申报=
  实测」零位移清偿谱系延续（轻覆盖轮，
  零码变零位移）。
- 对端刀信号持续：css 文法刀后续（
  shapepos 残面队列余项）+ wptfyi-css 残
  面（mask-border 家族支持门定谳、B3 残
  面 348F 余项、jsanim flip Steps(Start)
  ）+ editing 流 B4-S2 残面刀 + rendering
  dialog 16 红块协同面，join 实测为准。

## 结论

s557 收口（零码面轻覆盖轮 + 族带部分发射
形 legs 首现轮）：窗全 docs-only 零码面，
runner 十一代恒（确定性重编零换代）；浸泡
门 v3.2 判净（waiter 自锁两连根因修复：
相对路径自排除失效＋对端守护 cmdline 文
本误匹配，check_clean 改 comm 进程名匹配
）双腿常态全绿零爆栈；join stable 8513
（8305P/201F/6T/1U）disagree 1 + l1only 0
+ l2only 0；vs s556 NEW 3 + GONE 1 + flips
0 + disagree 1——**位移全落族带＝non-det
probe2 同款部分发射形 legs 首现（3 子测 F
回潮＋PageDown 零报告＋整文件「1 test(s)
unreported」Timeout 新消息形）＋detailed
腿间两超时形分歧（l1 harness-no-report/l2
0-declared）GONE 出账；判族带时序谱内摆
动非码面，按 s556 新规常态记账升级路径不
触发**；P 集 8305 第 5 轮恒等；password
持位第 5 轮；flaky 零新例第 46 轮，五哨持
位；全桥 b557.k SET-IDENTICAL（1694）。挂
用户门维持：GB-20261114-VIDEO + GB-20261031
（簇 D）+ s137（不重复通知）。

下一步：守成巡查（**对账基线换 s557
stable 8513——族带位移常态记账延续**：
s558 起族带 NEW/GONE/flip/disagree 形均按
摆动谱常态记账（probe2 部分发射形与
disagree 腿间分歧已实证在谱），升级路径不
预挂；s558 若出非带非哨键位移→刀 G 工法
查码面 diff 定归因；password 11 键 stable
持位确认（第 6 轮）；全桥清单基线 b557.k
1694 持位；flaky 键零新例时钟顺延第 47 轮
；**mpreload 哨常规连轮计数第 20 轮**；
dialog 哨常规持位（第 87 轮）；他流入树
归因轮或轻覆盖——对端刀信号持续同上（css
文法刀后续+wptfyi-css 残面+editing B4-S2
+rendering dialog 协同面），起腿前 runner
provenance 异常即查码面 diff（刀 G 工法）
，join 实测为准；起腿一律 v3.1 浸泡门（
waiter 绝对路径＋v3.2 comm 匹配版脚本，
s558 起以 /tmp/s557/wait-and-run.sh 为模
板 sed 改造））；候门决项等待用户拍板（
video P1+P2 立项 / 簇 D A2 立项 / s137）
；rendering 流 dialog 16 红块协同面（归
rendering 流记账）。门开随批落预案不变：
簇 D 开 → customizable-select-styles.css
×47 随批落。

工件：/tmp/s557/{s557-leg1,s557-leg2}.log、
s557-leg{1,2}.rc、leg{1,2}.s557.{start,end}
、legs.done、legs.head（18486fa9a）、
runner.{pre,mid,post}.{size,md5}（十一代
恒 47699964 在册）、window.s557.judge（
22:49:50 clean-soaked v3.2）、waiter.s557
.log + wait-and-run.sh（v3.2 comm 版）+
s557-{l1,l2}.norm、l557-{1,2}.canon、
s557.{stable.keys,stable.canon,disagree
.keys,leg1only.keys,leg2only.keys}、
s557.new.keys（3 族带子测键）、s557.gone
.keys（1 族带整文件键）、s557.flips（0）
+ s556.stable.{keys,canon}（基线复用）+
b556.k（复用）+ b557.k（1694）+ s557
.bridge.{new,gone}（0/0）+ s557-bridge
.log、s557-bridge.rc（guard rc=0）、s557
-bridge.list.log + s477-{onblur,dialogov
,mpreload,waapi}.keys + fd121.keys +
s557.onblurF.l1（与 s556 IDENT diff 实证
）+ norm.sh/canon.sh/join.sh（s488 修复版
管线，export LC_ALL=C 前缀调用）。
