# s624 收账证据——完全零位移链复挂第 23 连（全空窗平凡形＝窗底账 0 commit）＋b624 keys 1737 持位（"不动"分支）＋runner a58c919e 三点恒持位＋legs 在飞 git 操作违规实录与教训条目

日期：2026-11-22（布防 11:09:44；候窗 ~1m30s＝系列最短；窗开 11:11:14；全毕 11:17:08）
环境：本机双 clone 并行；`/tmp/s624`；布防后同 session 内候窗 1m30s 开窗、三臂自动跑毕全链、当轮即收账（收账 commit 因窗态漏检落在 legs 在飞期间＝见教训条目）。

## 窗与 legs

- 布防 11:09:44 三臂起动（waiter 直臂＋rearm 监督者＋post 分开臂；rearm.s624.log 不存在＝零真重臂）；候窗 ~1m30s（机器空闲＝对端 kb148 batch 已退坡，11:11:14 clean-soaked procs=0 load<3），工作树钉扎 `8b22b0aa4`（本流 s622 收账提交）＝legs.head。
- leg1 11:11:14→11:13:07（1m53s）rc=2＝完整有效腿，**Compiling ×0**（release build 0.19s 全新＝s623 leg2 刚建的 mtime 新鲜形）。
- **legs 在飞期间本流 commit＋pull --rebase＋push 落地（~11:11:2x，窗开后 15-30s）＝钉扎纪律违规实录**：窗态检查与 commit 同命令串联非 gate 形＝漏检根因；变更面 docs-only（master.md＋evidence＋对端 rendering-compat docs）＝工作树产品面零接触；**HEAD-move 8b22b0aa4→0bdc97ab1 触发 build.rs git-hash 失效＝leg2 Compiling ×6**（release build 43.24s）＝HEAD-move 链形实证（≡对端 R5901/R5903 run1 Compiling ×6 HEAD-move 形逐字同构）；leg2 11:13:07→11:15:42（2m35s）rc=2。
- 结果级无扰旁证：join 双腿 norm md5 恒 c735d371＋canon f1aef7b7＋bridge keys md5 4e20b5df 全链持位＝内容级零漂移；runner pre=mid=post 三点恒不受 HEAD-move 影响（binary 输入未变）。
- runner：pre=mid=post **`a58c919e`**/129,827,000 三点恒＝**a58c919e 持位观察兑现**（候选三十八在锚持位；候选四十二挂号未兑现＝空窗无 runner 依赖域直源）。
- 桥测（run-legs 内，同 s623 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，build 35.79s+test 25.19s，rcb=**101**。
- solo1 11:16:44→11:16:56（12s）／solo2 11:16:56→11:17:08（12s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s624＝发射形同值第 17 连。
- all.done 11:17:08；窗起至全毕 **5m54s** 零空窗（两臂齐全第 23 演）。

## join 判定（analyze.out）＝空窗零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s623）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗平凡形且为系列第二个**全空窗**（窗底账 0 commit＝码 ×0 docs ×0）。rc1=2 rc2=2＝失败数退出非截断。
- s624.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=…=s623＝链复挂第 23 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 28 连 s597→s624；双腿恒第 14 演）。

## b624 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s624.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=…=b623＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗桥触点核实：空窗＝js_dom_bridge 触点 ×0＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r58＝2P 双腿（flaky 键持位第 13 轮，GONE 形未现）；select-event r63＝279 keys 8F Fset ≡ se.577；password r63＝225 keys 0F；mpreload r86＝15/15P 双腿（持位第 23 轮）；waapi r145＝picker P/checkmark F 双腿；focus-dynamic r104＝121 all P；onblur r56＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 24 轮）；族带 r57＝band8 hits=8×2（聚合形持位第 41 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 19 演。轮次标签全对齐（dialog r58／select-event r63／password r63／mpreload r86／waapi r145／focus-dynamic r104／onblur r56／band r57＝与 s624 布防计划算术一致）。

## 窗底账（s624 窗＝8b22b0aa4→legs.head 8b22b0aa4＝0 commit＝全空窗）

- 码 ×0；docs ×0；data ×0（系列第二个全空窗 ≡s621 形；本流 s623 收账 0bdc97ab1 与对端 R5903 f5ae04d39 落窗外＝归 s625 窗面）。

## 教训条目（窗底账完整性系列＋钉扎纪律）

布防后收账 commit 前的窗态检查与 commit 同命令串联（`ls window.judge … ; git commit …`）非 gate 形＝窗开后 15-30s 的 commit＋pull --rebase 在 legs 在飞期间落地（漏检根因＝检查结果未阻断后续命令）。后果良性（变更 docs-only＝工作树产品面零接触；HEAD-move 仅触发 build.rs git-hash 重验＝leg2 Compiling ×6，join/bridge 内容级 md5 全链持位＝结果无扰旁证），但两 lesson 固化：**（1）布防后 git 操作前窗态检查必须独立命令且对窗开情况阻断后续操作（窗开即挂起至 all.done 后补账〔既有钉扎纪律照记强化〕）；（2）HEAD-move 对 legs 的可观测效应＝build.rs git-hash 失效 leg Compiling ×6（bin 输入不变，结果面无扰）——legs 在飞 HEAD-move 形可用 leg 间 Compiling 分布反转作取证指纹。**

## 基线换装

- join：s624.stable **8517**（f1aef7b7/057764d3，同 s610=s612=…=s623）。
- 全桥：b624 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**a58c919e**/129,827,000 在锚（候选三十八持位）。
