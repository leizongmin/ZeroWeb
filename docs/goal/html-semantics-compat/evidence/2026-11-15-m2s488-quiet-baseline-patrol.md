# M2 s488 — 守成巡查轻覆盖轮第二轮（零入树同码树同二进制，连续第二轮）：**双腿完全一致**（8516 双 8307P/202F/6T/1U，disagree 0 leg-only 0，族 s487-leg2 形全族键入 stable 8514）vs s487 stable GONE 0 + NEW 4（全族 optional 4 子测键）+ flips 0——非族键逐键恒等，P 集 8307 持位 + picker 增益键第 11 轮保持 + 五观察哨零新翻红 + 全桥绿窗 1655P/0F 清单恒等

日期：2026-11-15。承前：s487 下一步「守成巡查（他流入树归
因轮或轻覆盖）」。开工 `git pull --rebase` 携入一笔 89343ee76
（rendering 流 R5669 docs，零代码）；码态核查 `git diff
492a0f2e0..HEAD -- crates/ apps/` 零行——ZeroWeb-3 三文件刀
per R5669 numstat 连续五轮恒等未落树，无归因对象 → 轻覆盖
轮连续第二轮。

## 对端错峰观察

起腿时 ZeroWeb-3 clone 内 b2-head/b2-wip runner A/B 在飞
（testharness-selection + testharness-input-events --json 对
比跑 + zero-wpt-runner-release 重编，四进程全在 ZW-3 树与
其独立 target 目录）——异树零干扰，load 3.5 带内起腿（
s482/s483 先例）。

## 二进制 provenance

- leg1 起跑 make 重建（`Compiling` ×6，runner mtime
  22:44:58，size 126909176 与 s486/s487 逐字节同值——码态未
  变同内容重链现象持续在册）；leg2 零 `Compiling` 同二进制。
- 码态核查：`git diff 492a0f2e0..HEAD -- crates/ apps/` =
  零行，HEAD = 891d1c704（s487 收口 docs，码态 492a0f2e0）。

## 双腿 join（891d1c704 树 = 码态 492a0f2e0，make testharness-html-semantics ×2）

- leg1/leg2 **均 8516 行（双 8307P/202F/6T/1U）——双腿完全
  一致 disagree 0 + leg-only 0**（s479/s484/s486 以来第四次
  全同形态），族 s487-leg2 形（detailed 整文件 T + optional
  4 子测 F）。双腿零截断零真 hang（rc=2 惯例失败退出）。
- join：**stable 8514（8305P/202F/6T/1U）**——双腿同相位，
  族键全数入 stable（s484 同款形）。
- **vs s487 stable（8510 = 8305P/198F/6T/1U）：GONE 0 + NEW 4
  + flips 0**——NEW 4 全族键（optional 4 子测键，F），**
  非族键逐键恒等**（非族 T 键集 5/5 comm 复核恒等）。
- T 键集 6 = 非族 5 恒等 + detailed 整文件 T（s487 同款）；
  **P 集账本口径 8307 持位**（双腿双 8307P 第 10 轮同形）；
  stable 机械口径 8305P 九轮同形。双发发射伪影 2 键照旧在册
  （8516 raw → 8514 unique，sort -u 去重吸收）。

## 观察哨复核（五组全零新翻红）

- onblur 哨组 stable 10 键全在册 8P/2F（following-label 双键
  Pass 持位，红池 parent-label 2F 照常）。
- dialog-overlay 哨第 17 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 20 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 11 轮保
  持）/ checkmark Fail（带形不变）。哨键组文件自带 s477 时
  代 status 列，对账取 join 第三列（当前 stable 状态），详
  见操作坑。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码树旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 18.42s；filtered 2389 同值）。
测试清单 vs s487：**set 恒等 1655，GONE 0**（码态未变零增
长预期兑现）。

## 操作坑（本轮新增两条，管线已修）

- **join.sh 尾行 rc 1 杀链**：`grep -Fxf` 空 match 返回 1 +
  `set -euo pipefail` 在打印 stable= 行之后终止脚本，调用方
  `&&` 链断裂——本轮将尾行改 awk key 集过滤（s487 轮已发现
  空产物问题，本轮补上 rc 修复），修复版随 /tmp/s488/join.sh
  在档。
- **哨键组文件自带 status 列**：s477-*.keys 每行为
  `key\ts477时代状态`，join 当前 stable 后旧状态在第 2 列、
  当前状态在第 3 列——detail 展示须取第 3 列（本轮误读第 2
  列曾致 picker-icon 假翻红惊魂，原始 log 双腿 Pass 复核后
  证伪）。

## 结论

s488 收口（守成巡查轻覆盖轮第二轮，零源码改动）：零入树同
码树同二进制（492a0f2e0 持位），双腿完全一致第四次（第 s479
/s484/s486 形），族键因双 leg 同相位全数入 stable（8514），
vs s487 stable 仅 NEW 4 全族键 + flips 0，非族键逐键恒等，
P 集 8307 持位，picker 增益键第 11 轮保持，五观察哨零新翻
红，全桥绿窗钉面 1655 恒等零破。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提示，
不重复通知）。

工件：/tmp/s488/{s488-leg1,s488-leg2}.log、s488-{l1,l2}.norm、
l{1,2}.canon、s488.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon,kts}、flips.txt、gone.txt、new.txt、
s487.stable.keys（s487.kts 派生）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用）、s488-bridge.log、
b488.list、b487-b488.diff（空 = 清单恒等）、
norm.sh/canon.sh/join.sh（s478 沉淀管线 + 本轮 join.sh rc
修复）。
