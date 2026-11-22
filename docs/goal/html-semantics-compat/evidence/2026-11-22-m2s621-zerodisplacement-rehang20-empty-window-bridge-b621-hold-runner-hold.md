# s621 收账证据——完全零位移链复挂第 20 连（空窗平凡形＝窗底账 0 commit）＋b621 keys 1737 持位（"不动"分支）＋runner a58c919e 三点恒持位

日期：2026-11-22（布防 10:04:47；候窗 ~12m；窗开 10:17:03；全毕 10:23:48）
环境：本机双 clone 并行；`/tmp/s621`；布防后同 session 内候窗 12m 开窗、三臂自动跑毕全链、当轮即收账。

## 窗与 legs

- 布防 10:04:47 三臂起动；候窗 ~12m16s（对端 kb146 判代 batch 退坡后 10:17:03 clean-soaked，procs=0 load<3 3×45s），工作树钉扎 `2319ac412`（本流 s620 收账提交）＝legs.head。
- leg1 10:17:03→10:19:49（2m46s）rc=2＝完整有效腿，Compiling ×6（mtime 失效重编形 ≡s615/s618 零码窗先例，md5 恒＝非码笔链形）；leg2 10:19:49→10:21:58（2m09s）rc=2 零编译。
- runner：pre=mid=post **`a58c919e`**/129,827,000 三点恒＝**a58c919e 持位观察兑现**（候选三十八在锚持位；候选三十九挂号未兑现＝空窗无 runner 依赖域直源）。
- 桥测（run-legs 内，同 s620 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，build 43.32s（engine 单笔 mtime 形）+test 24.38s，rcb=**101**。
- solo1 10:23:23→10:23:34（11s）／solo2 10:23:34→10:23:48（14s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s621＝发射形同值第 14 连。
- all.done 10:23:48；窗起至全毕 **6m45s** 零空窗（两臂齐全第 20 演，rearm 零触发）。

## join 判定（analyze.out）＝空窗零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s620）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗平凡形且为系列首个**全空窗**（窗底账 0 commit＝码 ×0 docs ×0，位移预判无码面无 docs 面可归因）。rc1=2 rc2=2＝失败数退出非截断。
- s621.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613=s614=s615=s616=s617=s618=s619=s620＝链复挂第 20 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 25 连 s597→s621；双腿恒第 11 演）。

## b621 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s621.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=b615=b616=b617=b618=b619=b620＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗桥触点核实：空窗＝js_dom_bridge 触点 ×0＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r55＝2P 双腿（flaky 键持位第 10 轮，GONE 形未现）；select-event r60＝279 keys 8F Fset ≡ se.577；password r60＝225 keys 0F；mpreload r83＝15/15P 双腿（持位第 20 轮）；waapi r142＝picker P/checkmark F 双腿；focus-dynamic r101＝121 all P；onblur r53＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 21 轮）；族带 r54＝band8 hits=8×2（聚合形持位第 38 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 16 演。轮次标签全对齐（dialog r55／select-event r60／password r60／mpreload r83／waapi r142／focus-dynamic r101／onblur r53／band r54＝与 s621 布防计划算术一致）。

## 窗底账（s621 窗＝2319ac412→2319ac412＝0 commit＝全空窗）

- 码 ×0；docs ×0；data ×0（系列首个全空窗；布防时 fetch 全范围核验 0 commit 与窗开实测一致＝tip-only 教训照记后首次干净捕获）。

## 基线换装

- join：s621.stable **8517**（f1aef7b7/057764d3，同 s610=s612=…=s620）。
- 全桥：b621 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**a58c919e**/129,827,000 在锚（候选三十八持位）。
