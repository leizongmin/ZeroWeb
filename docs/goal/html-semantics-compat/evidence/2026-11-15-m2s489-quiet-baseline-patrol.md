# M2 s489 — 守成巡查轻覆盖轮第三轮（零入树同码树同二进制，连续第三轮）：leg1 onblur following-label 键单腿瞬态 F（leg2 P，s486-s488 三轮 stable P 持位）致 stable 8513 vs s488 GONE 1 + NEW 0 + flips 0——非族键逐键恒等零稳定回归，族键全数持位，P 集 leg2 8307P 持位 + picker 增益键第 12 轮保持 + 哨组 9 键 stable + 全桥绿窗 1655P/0F 清单恒等

日期：2026-11-15。承前：s488 下一步「守成巡查（他流入树归
因轮或轻覆盖）」。开工 `git pull --rebase`「已经是最新的」
（本地已携 R5670-R5672 三笔 rendering docs，零代码）；码态
核查 `git diff 492a0f2e0..HEAD -- crates/ apps/` 零行——
ZeroWeb-3 三文件刀 per R5670-R5672 施工面 185/16/36 持位未
落树（R5670 testharness.rs +28→+36 增长＝落地窗口临近信
号），无归因对象 → 轻覆盖轮连续第三轮。

## 对端负载窗错峰

起腿前 ZW-3 clone b2 工作 cargo debug 全量编译在飞（wgpu-core
rustc，load 9.72）——错峰等待 4 分钟回落 load 2.06 后起腿；
b2-head testharness-selection 长跑（guard 2400s 档）跨双腿窗
在飞，leg1 处其负载窗内、leg2 已 quieter。

## 二进制 provenance

- leg1 起跑 make 重建（`Compiling` ×6，runner mtime
  23:02:31，size 126909176 与 s486-s488 逐字节同值——同内
  容重链现象持续）；leg2 零 `Compiling` 同二进制。
- 码态 = 492a0f2e0（HEAD 3a8e93565 docs-only）。

## 双腿 join（3a8e93565 树 = 码态 492a0f2e0，make testharness-html-semantics ×2）

- leg1 **8516（8306P/203F/6T/1U）** + leg2 **8516（8307P/
  202F/6T/1U）**，双零截断零真 hang（rc=2 惯例失败退出）。
  leg1 -1P/+1F 来自 onblur following-label checkbox 键单腿瞬
  态（见下）。
- join：**stable 8513（8304P/202F/6T/1U）disagree 1 +
  leg-only 0/0**——族形双腿同相位（optional 4 子测 F +
  detailed 整文件 T，s488 同款全族键 stable 持位）。
- **vs s488 stable（8514 = 8305P/202F/6T/1U）：GONE 1 + NEW 0
  + flips 0——非族键逐键恒等零稳定回归**。GONE = onblur 哨
  组 checkbox following-label 键（leg1 瞬态 F / leg2 P，
  s486/s487/s488 三轮 stable P 持位；同文件 radio
  following-label 双腿 Pass 旁证非稳态迁移）。归因：leg1 起
  跑处对端 b2 负载窗（:active/点击交互时序敏感型用例），带
  内单腿瞬态、非稳定回归；下轮双腿复核，再现则升级观察。
- T 键集 6 = 非族 5 恒等 + detailed 整文件 T（s488 同款）；
  **P 集账本口径 leg2 8307P 持位第 11 轮**（leg1 8306P 瞬态
  -1 单键）；stable 机械口径 8304P（8305P 基 − 瞬态键退出）。
  双发发射伪影 2 键照旧在册。

## 观察哨复核（零稳定翻红；一键单腿瞬态退 stable）

- onblur 哨组 **9 键 stable 7P/2F**（parent-label 双键 F 红
  池照常）+ checkbox following-label 键 leg1 瞬态 F/leg2 P
  退 stable（带内单腿瞬态，非稳定回归）。
- dialog-overlay 哨第 18 轮零翻红（双键 stable Pass）；
  modulepreload 哨第 21 轮零翻红（15 键 stable 全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 12 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码树旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem 16
--time-limit 1200）`cargo test -p zero-engine --lib js_dom_bridge`：
**1655P/0F rc=0**（test 阶段 15.23s；filtered 2389 同值）。
测试清单 vs s488：**set 恒等 1655，GONE 0**（码态未变零增
长预期兑现）。

## 工序注记

- join.sh rc 修复版（s488 轮改 awk 尾行）首次实战：stable
  canon 8513 正常产出、调用链零中断。
- leg1 负载窗错峰记录在案（b2 编译峰 9.72 → 回落 2.06 起
  腿）；瞬态键归因以此负载窗为背景。

## 结论

s489 收口（守成巡查轻覆盖轮第三轮，零源码改动）：零入树同
码树同二进制（492a0f2e0 持位），唯一位移 = onblur 哨组一
键单腿瞬态退 stable（GONE 1 全账可归因，非族其余键逐键恒
等 + flips 0 零稳定回归），族键全数持位，P 集 leg2 8307P
持位，picker 增益键第 12 轮保持，全桥绿窗钉面 1655 恒等零
破。挂用户门维持：GB-20261114-VIDEO + GB-20261031（簇 D）+
s137（s445 已提示，不重复通知）。

工件：/tmp/s489/{s489-leg1,s489-leg2}.log、s489-{l1,l2}.norm、
l{1,2}.canon、s489.{l1u,l2u,s1,s2,k1,k2,both.keys,
leg1only.keys,leg2only.keys,disagree.keys,stable.keys,
stable.canon}、flips.txt、gone.txt、new.txt、
s488.stable.keys（s488.kts 派生）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用）、s489-bridge.log、
b489.list、b488-b489.diff（空 = 清单恒等）、
norm.sh/canon.sh/join.sh（s488 修复版管线复用）。
