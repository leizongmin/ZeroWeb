# M2 s502 — 守成巡查轻覆盖轮第十轮（零入树同码态同二进制，连续第十轮）：族键两形异相位出位——vs s501 stable GONE 5（全族键）+ NEW 0 + flips 0 非族逐键恒等，stable 8509（8305P/198F/5T/1U）与 s500 stable 键集+状态双恒等（族相位谱 1 轮周期往返摆 s500 出 → s501 入 → s502 出），leg1 与 s501 leg1 逐键恒等、leg2 仅族键 5 键发射形差，P 集 8307 持位第 24 轮 + picker 增益键第 25 轮保持 + 四观察哨零新翻红 + 全桥绿窗 1659P/0F 清单恒等

日期：2026-11-16。承前：s501 下一步「守成巡查（对账基线
换 s501 stable 8514——族键入位形；他流入树归因轮或轻覆
盖；……对端 ihtml 轴一拆分插入刀信号持续 + 后续刀信号持
续，落树后按他流刀归因轮）」。开工 `git pull --rebase`
fast-forward 仅对端 rendering-compat docs 三笔（R5688/
R5689/R5690）；码态核查 `git diff cfd1a06af..HEAD --
crates/ apps/` 零行——ihtml 轴一拆分插入刀（对端申报
「下一步入账」）与对端 ZW-3 querySelector 施工面（对端巡
查记录 unstaged 4 文件）均未落树，无归因对象 → 轻覆盖轮
连续第十轮。

## leg1 残档复用（同码态同二进制同语料 pin 三条件核验）

上一 session 在 429 限流中断前已起 leg1（约 03:53 起，
04:04 收尾），中断发生在腿完成之后（session 最终 API 错
误 04:08:46）。残档三条件核验：

1. 日志完整：8517 raw 状态行 + 惯例 `make 错误 1`（rc=2）
   收尾，无截断；
2. 码态同：04:01 make 重链发生时 crates/ 未变（本 round
   开工核查 cfd1a06af..HEAD 零行），runner size
   **126999128** 恒（s500/s501 换代值），同内容重链形态
   （s495/s497/s501 leg1 先例同款）；
3. 语料 pin 同：fetch 完成标记 315976933 在，FORCE=0 跳过
   重列。

残档 breakdown 8308P/202F/6T/0U/1Unsupported 与 s501
leg1 完全一致 → leg1 有效复用。leg2 等干净窗现跑。

## 对端观察

leg2 起腿前对端一笔 ZW-3 querySelector WIP input-events
探查在飞（`/tmp/zw-runner-b3u` WIP runner，test-guard 包
裹，04:15–04:18），轮询约 2.7 分钟干净窗（load 回落
1.53）起腿——负载干扰零。本轮对端源码刀零落树（双克隆
静默）。

## 双腿 join（HEAD 232b636a5 docs-only = 码态 cfd1a06af，make testharness-html-semantics）

- leg1 8517 raw（8308P/202F/6T/0U/1U）/ leg2 8514 raw
  （8308P/199F/6T/0U/1U）——双腿零截断零真 hang（rc=2 惯
  例失败退出）。本轮管线全 LC_ALL=C 固化。
- join：**stable 8509（8305P/198F/5T/1U）+ disagree 1 +
  leg1only 4 + leg2only 1——全差异 5 键均为族键**
  （select-home-end-pagedown-pageup 两文件）：leg1 子测形
  （optional 4 子测 Fail + detailed 整文件 Timeout）/
  leg2 整文件形（optional 整文件 Timeout + detailed 整文
  件 Fail，s487/s498 形）——两形异相位，非全同形。
- leg 级对账：**leg1 与 s501 leg1 逐键恒等（comm 零差
  异）**；leg2 仅族键 5 键发射形差（detailed T→F +
  optional 子测 4F → 整文件 T）。
- **vs s501 stable（8514 = 8305P/202F/6T/1U）：GONE 5
  （全族键：optional 4 子测键 + detailed 整文件键）+
  NEW 0 + flips 0——非族键逐键恒等**。
- **stable 8509 与 s500 stable（8509 = 8305P/198F/5T/1U）
  键集+状态双恒等（comm 零差异）**——族相位谱在册维持观
  察：s494 出 → s495 入 → s496 出 → s497 入 → s498 整文
  件形双腿并现 → s499 整文件形第二连轮 → s500 两形 →
  s501 入（全同）→ s502 出，1 轮周期往返摆，不升级。
- **P 集账本口径双腿双 8307P 持位第 24 轮**（raw 8308 →
  norm 8307 两腿各丢 1 行 bare 发射，两轮同形；unique
  8305 = 双发发射伪影 2 键 dedup 照旧在册：
  wrapping-transformation.window.js exact-len +
  execution-timing/149.html WINdow）。stable 机械口径
  8305P。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 **10 键双腿双 8P/2F**（following-label 双键
  双腿 Pass 持位，parent-label 2F 红池照常）——**flaky
  键 3 例窗口零新例连续第八轮**（当前 2 例：s489 leg1 /
  s493 leg2）。
- dialog-overlay 哨第 31 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 34 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 25 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1659P/0F rc=0**（test 阶段 15.51s；
filtered 2394 恒）。测试清单 vs s501：**set 恒等 1659，
GONE 0**（码态未变零增长预期兑现；对账时剥 `test ` 前缀
+ LC_ALL=C 重排后 comm 零差异）。

## 结论

s502 收口（守成巡查轻覆盖轮第十轮，零源码改动）：零入树
同码态同二进制（cfd1a06af 持位，runner 126999128 恒，
leg1 残档三条件核验复用），族键两形异相位出位（GONE 5 全
族键 + NEW 0 + flips 0 非族逐键恒等），stable 8509 与
s500 stable 双恒等（1 轮周期往返摆 s500 出 → s501 入 →
s502 出），leg1 与上轮逐键恒等、leg2 仅族键发射形差，P 集
8307 持位第 24 轮，flaky 键 3 例窗口零新例连续第八轮，
picker 增益键第 25 轮保持，四观察哨零新翻红，全桥绿窗钉
面 1659 恒等零破。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s502 stable 8509——族键出位
形、s500 构成同款；族键相位谱 1 轮周期往返摆观察维持——
下轮预期回位入位形；他流入树归因轮或轻覆盖；对端 ihtml
轴一拆分插入刀信号持续 + 对端 ZW-3 querySelector 施工刀
信号持续，落树后按他流刀归因轮）+ 候门决项等待用户拍板
（video P1+P2 立项 / 簇 D A2 立项 / s137）；rendering 流
dialog 16 红块协同面（归 rendering 流记账）。门开随批落
预案不变：簇 D 开 → customizable-select-styles.css ×47
随批落。

工件：/tmp/s502/{s502-leg1,s502-leg2}.log（leg1 残档复
用）、s502-{l1,l2}.norm、l{1,2}.canon、s501.l{1,2}.canon
（腿级对账）、flips.txt（零行）、gone.txt（5 族键）、
new.txt（零行）、s502.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon}、s501.keys.s 复用（跨轮对账基线）、
s501.stable.{keys,canon}（基线复用）、s500.stable.keys+
s500.sc（s500 构成同款对账）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用；waapi 键组存 s477 旧
形带 \tFail 后缀，picker-icon 需剥后缀直查）、
s502-bridge.log、b501.sorted（复用）+ b502.sorted/b502.k
（清单恒等对账，剥前缀重排后 comm）、dups-l{1,2}.txt、
norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
