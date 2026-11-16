# M2 s513 — 守成巡查轻覆盖轮第十五轮（零入树零码变零换代）：族键摆动形（s512 detailed 回位键再现异相出位 leg1 F/leg2 T + base optional 整文件/子测形互换 l1only 1 + l2only 4——s508 两形异相同款）——vs s512 stable GONE 1（族 detailed 键出位）+ NEW 0 + flips 0 非族逐键恒等，stable 8509，P 集双腿双 8307P（持位第 35 轮）+ 四观察哨零新翻红 + 全桥绿窗 1661P/0F 清单恒等

日期：2026-11-16。承前：s512 下一步「守成巡查（对账基
线换 s512 stable 8510……他流入树归因轮或轻覆盖）」。
s512 收口 push 后 pull --rebase 携入对端 rendering-compat
R5713/R5714/R5715 三笔（全 docs-only；触发器基线
996@c19712be9 维持）；码面 `git diff c19712be9..HEAD`
空——对端刀均未落树，无归因对象 → 守成巡查轻覆盖轮
第十五轮。HEAD 6b48826a5 docs-only（本流 s512 收口）。

## 对端观察

- 双腿窗（10:55:10–11:01:51）内对端零构建级背景：
  leg1 起腿时 load ~3.8–5.4 回落态（容器视角）。
- 收口时（11:05 前后）对端新构建风暴在途（load 12.81
  1min 均值）——双腿与全桥窗已毕，窗内干净；风暴归属
  对端下一轮窗观察。

## 二进制 provenance

- runner size **127486704** 恒（s512 换代值持续，零换
  代）；leg1 make 增量重编 ×6（指纹 churn，非码变），
  leg2 同二进制零重编；bridge 收口跑 Compiling ×3 桥
  侧 churn。
- 码态 = c19712be9（HEAD 6b48826a5 docs-only）。

## 双腿 join（HEAD 6b48826a5 树 = 码态 c19712be9，make testharness-html-semantics ×2）

- leg1 10:55:10–10:59:00 / leg2 10:59:13–11:01:51，
  raw 8514/8517 → norm 8513/8516（双腿各丢 1 行 bare
  发射 link 面同形）——双腿零截断零真 hang（rc=2 惯
  例失败退出）。本轮管线全 LC_ALL=C 固化。
- **P 集账本口径双腿双 8307P**（持位第 35 轮）。
- join：**stable 8509（8305P/198F/5T/1U）+ disagree 1
  + l1only 1 + l2only 4**——族键摆动形全落带内：
  - disagree 1 = 族 detailed 整文件键 leg1 Fail/leg2
    Timeout（异相出位——s512 NEW 1 同键回位后再出
    位，形谱摆动无固定周期兑现）。
  - l1only 1 = 族 base optional 整文件键 leg1 Fail
    （leg2 零整文件发射）；l2only 4 = 族 base optional
    子测 End/Home/PageDown/PageUp 键 leg2 全发射
    （4 键首现全子测形）——**整文件/子测形互换两形异
    相，s508 同款再现**（s508：leg1 子测形/leg2 整文
    件形；本轮反向）。发射形谱漂移观察维持，全部落
    族带内不升级。
- **vs s512 stable（8510 = 8305P/199F/5T/1U）：GONE 1
  （族 detailed 键出位）+ NEW 0 + flips 0——非族逐键
  恒等；零刀零码变纯守成零语义位移**。T 集 5 恒 + U
  集 1 恒等。
- 双发发射伪影 2 键照旧在册（wrapping-transformation
  exact-len 双腿各 2 dup + execution-timing/149 WINdow
  双腿各 2 dup）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键
  红池照常）——flaky 键双腿双 Pass：零新例第二轮
  （s512 时钟重置后第 2 轮；s514 前再现方触发 <4 轮
  频率升级判定）。
- dialog-overlay 哨第 42 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 45 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 36 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1661P/0F rc=0**（test 阶段 34.26s；
filtered 2399 恒）。测试清单 vs s512（b512.k）：**set
恒等 1661，NEW 0 / GONE 0**（码态未变零增长预期兑现）。

## 结论

s513 收口（守成巡查轻覆盖轮第十五轮，零源码改动）：零
入树同码态同二进制（c19712be9 持位，runner 127486704
恒），族键摆动形全落带内（detailed 键回位后再出位
leg1 F/leg2 T + base optional 整文件/子测形互换两形异
相 l1only 1 + l2only 4——s508 同款，形谱漂移无固定周
期观察维持，不升级），stable 8510→8509 唯一位移 = 族
detailed 键 GONE；P 集双腿双 8307 持位第 35 轮，flaky
键零新例第二轮，picker 增益键第 36 轮保持，四观察哨零
新翻红，全桥绿窗钉面 1661 恒等零破。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445
已提示，不重复通知）。

下一步：守成巡查（对账基线换 s513 stable 8509——族键
摆动形：detailed 键出位 + base optional 整文件/子测形
互换两形异相；族键发射形谱漂移观察维持——回位/出位/
混合形均按形谱记账，全部落族带内不升级；flaky 键零新
例时钟顺延——s514 前再现方触发 <4 轮升级判定；他流
入树归因轮或轻覆盖；对端 ZW-3 querySelector 施工刀信
号 + css 文法刀后续刀信号 + editing 流 G 族 ws-pre 刀
信号 + generation-aware 挂起刀信号持续，落树后按他流
刀归因轮，join 实测为准）+ 候门决项等待用户拍板
（video P1+P2 立项 / 簇 D A2 立项 / s137）；rendering
流 dialog 16 红块协同面（归 rendering 流记账）。门开随
批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s513/{s513-leg1,s513-leg2}.log、
s513-leg{1,2}.rc、leg{1,2}.{start,end}、
s513-{l1,l2}.norm、l{1,2}.canon、flips.txt（零行）、
gone.txt（1 族键）、new.txt（零行）、
s513.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s512.stable.{keys,canon}（基线复用）+
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复
用，须剥状态后缀直查 canon）、s513-bridge.log、
s513-bridge.rc、b512.k（复用）+ b513.k（清单恒等对账）
+ b513.{new,gone}（双零行）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
