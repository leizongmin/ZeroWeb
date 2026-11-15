# M2 s497 — 守成巡查轻覆盖轮第八轮（零入树同码态同二进制，连续第八轮）：**双腿完全一致**第八次（双 8516 = 双 8307P/202F/6T/1U，disagree 0 leg-only 0），s496 族键发射形相位带内往返——vs s496 stable GONE 0 + NEW 5（optional 4 子测键 Fail 回子测形 + detailed 整文件键双腿 T 入 stable，s495/s488/s490/s492 形恢复）+ flips 0 非族键逐键恒等 + T 集 5→6 恢复，onblur flaky 键双腿 Pass 3 例窗口零新例连续第三轮，P 集 8307 持位第 19 轮 + picker 增益键第 20 轮保持 + 四观察哨零新翻红 + 全桥绿窗 1655P/0F 清单恒等

日期：2026-11-16。承前：s496 下一步「守成巡查（对账基线换
s496 stable 8509；他流入树归因轮或轻覆盖；onblur flaky 键
3 例窗口观察维持；ZW-3 后续刀信号持续，落树后按他流刀归因
轮）」。开工 `git pull --rebase`「已经是最新的」；码态核查
`git diff 4427b121e..HEAD -- crates/ apps/` 零行——ZW-3
后续刀与对端新刀均未落树，无归因对象 → 轻覆盖轮连续第八轮。

## 对端观察

起腿前对端活跃开发循环在飞：`/tmp/zw-runner-b2-wip
testharness-html-semantics --json` 快检探查（timeout 590
有界，PID 2406678）+ 数笔小域 cargo probe（zeroseed-storage
rename_operation_probe 等）交替——b2-wip runner 探查
html-semantics 面为刀准备期信号（未落树，origin 零新
commit）。等待干净窗约 7.5 分钟（对端快检收尾 + probe 编译
结束），起腿时对端零构建进程在飞、load 2.15 回落——负载
干扰零。

## 二进制 provenance

- runner size **126982592** 恒（s496 换代值，刀码同二进制）。
- leg1 起跑 make 重链（`Compiling` ×6 同内容重链形态，s495
  leg1 先例同款）；leg2 零 `Compiling` 同二进制。
- 码态 = 4427b121e（HEAD e795899a6 docs-only）。

## 双腿 join（e795899a6 树 = 码态 4427b121e，make testharness-html-semantics ×2）

- leg1/leg2 **均 8516 raw（双 8307P/202F/6T/1U）——双腿完
  全一致 disagree 0 + leg-only 0**（s479/s484/s486/s488/
  s490/s492/s495 以来第八次全同形态），族 s495/s488/s490/
  s492 形（optional 4 子测 F + detailed 整文件 T）。双腿零
  截断零真 hang（rc=2 惯例失败退出；截断 grep 唯一命中为
  maxlength emoji 测试名内「truncated」词，非真截断）。
- join：**stable 8514（8305P/202F/6T/1U）**——族键全数入
  stable。
- **vs s496 stable（8509 = 8305P/198F/5T/1U）：GONE 0 +
  NEW 5 + flips 0**——NEW 5 全族键：optional 4 子测键
  Fail（双腿恢复子测发射形）+ detailed 整文件键 Timeout
  （双腿 T 同相位入 stable）——**s496 归因轮判定的族带内
  发射形差（leg1/leg2 两形 disagree）经一轮往返收案**（
  s496 两形带 → s497 双腿一致子测形 + detailed T 形，
  s495/s488/s490/s492 形恢复；族相位谱 s464 起在册维持
  观察：s494 出 → s495 入 → s496 出 → s497 入，1 轮周期
  往返摆，不升级）；**非族键逐键恒等**，T 键集 5→6 恢复
  （新增即 detailed 整文件 T）。
- **P 集账本口径双腿双 8307P 持位第 19 轮**（stable 机械
  口径 8305P）。双发发射伪影 2 键照旧在册（8516 raw →
  8514 unique：wrapping-transformation.window.js exact-len
  + execution-timing/149.html WINdow）。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 **10 键双腿双 8P/2F**（following-label 双键
  双腿 Pass 持位，parent-label 2F 红池照常）——flaky
  checkbox following-label 键双腿 Pass，**3 例窗口零新例
  连续第三轮**（当前 2 例：s489 leg1 / s493 leg2）。
- dialog-overlay 哨第 26 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 29 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 20 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 17.03s；filtered 2394 恒）。
测试清单 vs s496：**set 恒等 1655，GONE 0**（码态未变零增
长预期兑现）。

## 结论

s497 收口（守成巡查轻覆盖轮第八轮，零源码改动）：零入树同
码态同二进制（4427b121e 持位，runner 126982592 恒），双腿
完全一致第八次，s496 族键两形发射形差相位带内往返收案（
GONE 0 + NEW 5 族键回位 + flips 0 非族逐键恒等），族键全
数入 stable 8514，P 集 8307 持位第 19 轮，flaky 键 3 例窗
口零新例连续第三轮，picker 增益键第 20 轮保持，四观察哨零
新翻红，全桥绿窗钉面 1655 恒等零破。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提
示，不重复通知）。

下一步：守成巡查（对账基线换 s497 stable 8514；他流入树归
因轮或轻覆盖；onblur flaky 键 3 例窗口观察维持；对端
b2-wip runner html-semantics 探查信号持续 + ZW-3 后续刀信
号持续，落树后按他流刀归因轮）。

工件：/tmp/s497/{s497-leg1,s497-leg2}.log、s497-{l1,l2}.norm、
l{1,2}.canon、s497.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon}、flips（零行）、gone.txt（零行）、new.txt
（5 族键）、t496.keys/t497.keys（T 集恢复对账）、
s496.stable.{keys,canon}（基线复用）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用）、s497-bridge.log、
b497.list、b496-b497.diff（空 = 清单恒等）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用）。
