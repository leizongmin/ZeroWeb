# M2 s504 — 他流刀归因轮（单刀）：对端 editing 流 ihtml 轴一块级拆分插入刀 4f3469026 落树（js_dom_shim part06.js +497 + part124 ihtml block pins 新文件 +211 + bridge_tests.rs +1）——html-semantics 面**零语义位移清偿，「申报=实测」谱系第十一例**；码态换新 4f3469026（runner 换代 126999128 → 127019640 +204,512，leg1 真重建 Compiling ×6）——vs s503 stable GONE 5（全族键出位，leg1 整文件形 / leg2 子测形 = s500 两形发射形差重放）+ NEW 1（flaky 键双腿 Pass 回位）+ flips 0，stable 8509 与 s500/s502 stable 键集+状态双恒等，桥面 1660P/0F（+1 恰 test_ihtml_block_pins，增长预期兑现）

日期：2026-11-16。承前：s503 下一步「**他流刀归因轮**（刀时序
注记：s503 收口 push rebase 携入对端 editing 流 ihtml 轴一
块级拆分插入刀 4f3469026 落树——js_dom_shim part06.js
+497 + part124 ihtml block pins 新文件 +211 +
bridge_tests.rs +1，桥面 1659 将增；对账基线 s503 stable
8513，join 实测归因零语义位移清偿「申报=实测」谱系）」。
开工 `git pull --rebase`「已经是最新的」（HEAD 108817951 =
s503 注记版）；码态核查 `git diff 4f3469026..HEAD --
crates/ apps/` 零行——刀码即域内 HEAD，单刀无叠加。对端
editing-compat 控制面头仍 I5 收口态（ihtml 刀申报账待对端
落），本流按刀 commit + 实测归因。part124 = 30 钉单测函数
1 个（test_ihtml_block_pins）→ 桥面 1659→1660 预期。

## 二进制 provenance

- runner 换代 **126999128 → 127019640**（+204,512 刀载
  荷；leg1 真重建 `Compiling` ×6）；leg2 零 Compiling 同
  二进制（Finished 0.18s 全缓存）。
- 码态 = 4f3469026（HEAD 108817951 docs-only 注记版）。

## 对端观察

leg1 起腿前对端一笔 zero_engine/zero_webview release 构
建在飞（ZW-3 施工持续，05:0x 清场），轮询干净窗起腿——
双腿期间对端零构建进程，负载干扰零。

## 双腿 join（HEAD 108817951 树 = 码态 4f3469026，make testharness-html-semantics ×2）

- leg1 8514 raw（8308P/200F/5T/0U/1U）/ leg2 8517 raw
  （8308P/202F/6T/0U/1U）——双腿零截断零真 hang（rc=2 惯
  例失败退出）。本轮管线全 LC_ALL=C 固化。（过程注记：
  首次 leg2 误自 /tmp/s504 起跑 make 报 No rule 立即退出
  零测试效果，自仓库根重跑为准。）
- join：**stable 8509（8305P/198F/5T/1U）+ disagree 1 +
  leg1only 1 + leg2only 4——全差异 6 键均为在册带内键**：
  - 族键 5 键两形发射形差：leg1 整文件形（optional 整文
    件 Timeout + detailed 整文件 Fail，s487/s498 形）/
    leg2 子测形（optional 4 子测 Fail + detailed 整文件
    Timeout，s501 形）——**s500 两形发射形差重放**；
  - flaky 键 1 键（onblur checkbox following-label）：双
    腿 Pass，s503 leg2 瞬态 F 经一轮往返回位。
- **vs s503 stable（8513 = 8304P/202F/6T/1U）：GONE 5
  （全族键出位）+ NEW 1（flaky 键回位）+ flips 0**。
- **stable 8509 与 s500 stable / s502 stable 键集+状态双
  恒等（comm 双零）——刀致语义位移零，全部位移落在在册
  带内谱（族相位谱 1 轮周期往返摆 s503 入 → s504 出 +
  flaky 摇摆 P↔F 非单向第 4 轮回位）→「申报=实测」谱系
  第十一例（单刀）零语义位移清偿**。
- **P 集账本口径双腿双 8307P 持位第 26 轮**；stable 机械
  口径 8305P。双发发射伪影 2 键照旧在册（双腿各 2
  dup）。T 集 6 恒等。

## 观察哨复核

- onblur 哨组 10 键**双腿双 8P/2F**（parent-label 双键红
  池照常）——flaky 键双腿 Pass，**3 例窗口满后零新例第一
  轮**（窗口持位 3 例：s489 leg1 / s493 leg2 / s503
  leg2；后续新例即第 4 例按频率升级判定，政策维持）。
- dialog-overlay 哨第 33 轮零翻红（双键双腿 Pass）；
  modulepreload 哨第 36 轮零翻红（15 键双腿全 Pass）。
- waapi 哨 2 键：picker-icon Pass（s478 刀增益第 27 轮保
  持）/ checkmark Fail（带形不变）。
- hang 观察项静默哨维持（双腿零真 hang，零截断）。

## 全桥套件绿窗收口跑（刀码同源旁证，增长预期兑现）

guard 包裹（--compile-first --per-proc-mem 8 --total-mem
16 --time-limit 1200）`cargo test -p zero-engine --lib
js_dom_bridge`：**1660P/0F rc=0**（test 阶段 15.36s；
filtered 2394 恒）。测试清单 vs s503：**1659→1660 +1 = 
`js_dom_bridge::tests::test_ihtml_block_pins`（part124
pins，GONE 0）——s503 注记「桥面 1659 将增」精确兑现**。

## 结论

s504 收口（他流刀归因轮，单刀 4f3469026）：码态换新
4f3469026（runner 换代 127019640，leg1 真重建 Compiling
×6），html-semantics 面**零语义位移清偿**——vs s503
stable GONE 5 全族键出位（两形发射形差 s500 形重放）+
NEW 1 flaky 键回位 + flips 0，stable 8509 与 s500/s502
双恒等，全部位移落在在册带内谱零刀致位移（申报=实测谱系
第十一例）；P 集 8307 持位第 26 轮，flaky 窗口满后零新例
第一轮，picker 增益键第 27 轮保持，四观察哨零新翻红，全
桥绿窗 1660（+1 pins 增长预期兑现）零破。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137（s445 已提
示，不重复通知）。

下一步：守成巡查（对账基线换 s504 stable 8509——族键出
位形 + flaky 键回位形、s500/s502 构成同款；族相位谱 1 轮
周期往返摆观察维持——下轮预期回位入位形；flaky 窗口满
后新例观察维持——新例即第 4 例按频率升级判定；他流入树
归因轮或轻覆盖；对端 ihtml 刀申报账落定信号 + 对端
ZW-3 querySelector 施工刀信号持续，落树后按他流刀归因
轮，join 实测为准）+ 候门决项等待用户拍板（video P1+P2
立项 / 簇 D A2 立项 / s137）；rendering 流 dialog 16 红
块协同面（归 rendering 流记账）。门开随批落预案不变：簇
D 开 → customizable-select-styles.css ×47 随批落。

工件：/tmp/s504/{s504-leg1,s504-leg2}.log、
s504-{l1,l2}.norm、l{1,2}.canon、flips.txt（零行）、
gone.txt（5 族键）、new.txt（1 flaky 键）、s504.{l1u,l2u,
s1,s2,k1,k2,both.keys,leg1only.keys,leg2only.keys,
disagree.keys,stable.keys,stable.canon}、s503.stable.{keys,
canon}（基线复用）+ s500.stable.keys/canon、
s502.stable.keys（构成恒等对账）、s477-{onblur,dialogov,
mpreload,waapi}.keys（哨键组复用，须剥状态后缀直查）、
s504-bridge.log、b503.k（复用）+ b504.k（清单 +1 对
账）、norm.sh/canon.sh/join.sh（s488 修复版管线复用，全
LC_ALL=C）。
