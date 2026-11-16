# M2 s506 — 守成巡查轻覆盖轮第十二轮（零入树同码态同二进制，连续第十二轮）：**双腿完全一致第十次**（双 8517 raw，disagree 0 / leg-only 双零）——vs s505 stable GONE 0 + NEW 4（族 optional 4 子测键 Fail 回位）+ flips 1（族 detailed 整文件键 Fail→Timeout 回位，全部位移落族带内发射形谱），stable 8514 = s501/s503 形完全恢复，P 集双腿双 8307P 持位第 28 轮 + 四观察哨零新翻红 + 全桥绿窗 1660P/0F 清单恒等

日期：2026-11-16。承前：s505 下一步「守成巡查（对账基线
换 s505 stable 8510——族键 detailed F 入位形；族键带内发
射形谱第三形观察维持，1 轮周期往返摆下轮相位开放；
……他流入树归因轮或轻覆盖）」。开工 `git pull --rebase`
fast-forward 携入对端 wptfyi-css/rendering-compat docs 三
笔（HEAD ff29d95be，docs-only）；码态核查
`git diff 07fcc9e0e..HEAD -- crates/ apps/ engines/` 零
行——对端 css 文法刀后续刀 / ZW-3 querySelector 施工刀 /
editing 流 ihtml 刀均未落树，无归因对象 → 轻覆盖轮连续
第十二轮。

## 对端观察

起腿时 load 0.26 回落干净窗，对端零构建进程（peer procs
= 0）；leg2 起腿时同形。双腿均为干净窗零进程背景。本轮
对端源码刀零落树（双克隆静默）。

## 二进制 provenance

- runner size **127411968** 恒（s505 换代值持续，刀码同
  二进制）；leg1 make relink Finished 40.31s 同内容形态。
- 码态 = 07fcc9e0e（HEAD ff29d95be docs-only）。

## 双腿 join（HEAD ff29d95be 树 = 码态 07fcc9e0e，make testharness-html-semantics ×2）

- leg1 08:14:30–08:17:42 / leg2 08:17:52–08:20:16，
  双 8517 raw（双 8308P raw）——双腿零截断零真 hang
  （rc=2 惯例失败退出）。本轮管线全 LC_ALL=C 固化。
- norm 双 8516（两腿同形各丢 1 行 bare 发射）/**双腿双
  8307P**。
- join：**stable 8514（8305P/202F/6T/1U）+ disagree 0 +
  leg-only 双零 = 双腿完全一致第十次**（s501 第九次后
  首现）。
- **vs s505 stable（8510 = 8305P/199F/5T/1U）：GONE 0 +
  NEW 4 + flips 1**：
  - **NEW 4 = 族 optional 4 子测键 Fail 双腿同相位回
    位**（select-home-end-pagedown-pageup.optional.html
    Home/End/PageDown/PageUp 四子测）——s501/s503 形恢
    复。
  - **flips 1 = 族 detailed 整文件键 Fail→Timeout 回
    位**（select-home-end-pagedown-pageup-detailed
    .optional.html 整文件；s505 形 Fail → s506 形
    Timeout，s501/s503 形恢复）。
  - 全部落**族键带内发射形谱**（本形谱四形：
    子测形 T / 整文件形 T / detailed F / 完全回位形），
    非族非哨键逐键恒等；码态零入树，**刀致语义位移
    零**。族相位谱：s500 出 → s501 入 → s502 出 →
    s503 入 → s504 出 → s505 半入位形 → **s506 完全回
    位形**——上轮「1 轮周期往返摆下轮相位开放」兑现
    为回位。
- **P 集账本口径双腿双 8307P 持位第 28 轮**（raw 8308 →
  norm 8307 两腿同形各丢 1 行 bare 发射；unique 8305 =
  stable 机械口径 8305P）。双发发射伪影 2 键照旧在册
  （wrapping-transformation exact-len +
  execution-timing/149.html WINdow，双腿各 2 dup）。
  T 集 6 恒等（5 恒 + detailed 回位）；U 集 1 恒等。

## 观察哨复核（四组全零新翻红）

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键红
  池照常）——flaky 键（checkbox following-label）双腿
  Pass，**3 例窗口满后零新例第三连轮**（窗口持位 3 例：
  s489 leg1 / s493 leg2 / s503 leg2；后续新例即第 4 例
  按频率升级判定，政策维持）。
- dialog-overlay 哨第 35 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 38 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 29 轮
  保持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（同码态旁证，清单恒等）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1660P/0F rc=0**（test 阶段 19.24s；
filtered 2399 恒）。测试清单 vs s505：**set 恒等 1660，
GONE 0**（码态未变零增长预期兑现）。

## 结论

s506 收口（守成巡查轻覆盖轮第十二轮，零源码改动）：零入
树同码态同二进制（07fcc9e0e 持位，runner 127411968 恒），
双腿完全一致第十次（stable 8514），族键完全回位形（
s501/s503 形恢复——optional 4 子测 Fail + detailed 整文
件 Timeout，全部位移落族带内发射形谱），P 集双腿双 8307
持位第 28 轮，flaky 窗口满后零新例第三连轮，picker 增益
键第 29 轮保持，四观察哨零新翻红，全桥绿窗钉面 1660 恒
等零破。挂用户门维持：GB-20261114-VIDEO +
GB-20261031（簇 D）+ s137（s445 已提示，不重复通知）。

下一步：守成巡查（对账基线换 s506 stable 8514——族键完
全回位形；族键带内发射形谱观察维持（s505 半入位形 →
s506 完全回位形，下轮相位开放——出位形重现则 1 轮周期
往返摆维持，连续回位则摆动假说降权）；flaky 窗口满后新
例观察维持——新例即第 4 例按频率升级判定；他流入树归因
轮或轻覆盖；对端 css 文法刀后续刀信号 + ZW-3
querySelector 施工刀信号 + editing 流 ihtml 刀申报账落
定信号持续，落树后按他流刀归因轮，join 实测为准）+ 候
门决项等待用户拍板（video P1+P2 立项 / 簇 D A2 立项 /
s137）；rendering 流 dialog 16 红块协同面（归 rendering
流记账）。门开随批落预案不变：簇 D 开 →
customizable-select-styles.css ×47 随批落。

工件：/tmp/s506/{s506-leg1,s506-leg2}.log、
s506-leg{1,2}.rc、leg{1,2}.{start,end}、
s506-{l1,l2}.norm、l{1,2}.canon、flips.txt（1 族键）、
gone.txt（零行）、new.txt（4 族键）、
s506.{l1u,l2u,s1,s2,k1,k2,both.keys,leg1only.keys,
leg2only.keys,disagree.keys,stable.keys,stable.canon}、
s505.stable.{keys,canon}（基线复用）、
s477-{onblur,dialogov,mpreload,waapi}.keys（哨键组复用，
须剥状态后缀直查 canon）、s506-bridge.log、b505.k（复
用）+ b506.k（清单恒等对账）、norm.sh/canon.sh/join.sh
（s488 修复版管线复用，全 LC_ALL=C）。
