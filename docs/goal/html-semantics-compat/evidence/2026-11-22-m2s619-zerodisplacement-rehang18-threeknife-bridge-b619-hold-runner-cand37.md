# s619 收账证据——完全零位移链复挂第 18 连（三码刀入窗非平凡形）＋b619 keys 1737 持位（"不动"分支＝桥触点 ×1 零新钉）＋runner 候选三十七确定换代 96d18fc4→9ef937e2

日期：2026-11-22（布防 09:26:00；候窗 ~13m；窗开 09:39:16；全毕 09:45:43）
环境：本机双 clone 并行（对端 wptfyi-css/rendering 流）；`/tmp/s619`；布防后同 session 内候窗 13m 开窗、三臂自动跑毕全链、当轮即收账（非断点续跑形）。

## 窗与 legs

- 布防 09:26:00 三臂起动（前 session 收账后即时布防；sed 生成脚本丢执行位致首臂未起动＝ps 双进程验证环节抓获，chmod 修复重臂后两臂齐全——既有「浸泡门五坑」记忆条目命中）；候窗 ~13m16s（对端 kb/R5900 contention 退坡后 09:39:16 clean-soaked，procs=0 load<3 3×45s），工作树钉扎 `0d3073862`（本流 s618 收账提交）＝legs.head。
- leg1 09:39:17→09:42:06（2m49s）rc=2＝完整有效腿，Compiling ×8（三码刀 runner 依赖域链形）；leg2 09:42:06→09:44:03（1m57s）rc=2 零编译。
- runner：pre `96d18fc4`/129,515,616 → mid=post **`9ef937e2`**/129,672,376（**+156,760B**）＝**候选三十七挂号转确定形兑现**（窗码面三刀 runner 依赖域直源全覆盖：68dd6dda3 style-system hyphens 初值＋0174f69f4 engine gCS inset〔js_dom_bridge 7 files＝computed_style_insets.rs 新 213 行＋cache +214＋callbacks +75＋bridge.rs +5＋computed_style.rs +3＋part03.rs 断言翻面＋shim part04.js +35 嵌入〕＋54d967087 layout-engine abspos.rs +56；与对端 kb144 94代 c95e7b1d88〔双码笔窗〕不同代＝本窗多含 layout-engine 刀，各流各记）；leg1 Compiling ×8＝码刀链形。
- 桥测（run-legs 内，同 s618 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，build 50.00s（net/layout-engine/engine 链）+test 23.02s，rcb=**101**。
- solo1 09:45:21→09:45:32（11s）／solo2 09:45:32→09:45:43（11s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s619＝发射形同值第 12 连。
- all.done 09:45:43；窗起至全毕 **6m27s** 零空窗（两臂齐全第 18 演，rearm 零触发）。

## join 判定（analyze.out）＝三码刀入窗零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s618）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗非平凡（码 ×3 入窗：style-system 初值刀／engine gCS 回读刀／layout-engine abspos stretch 刀）——三刀均非 semantics 语料敏感面的预判兑现（≡s616 三码刀/s617 双码刀先例续形；abspos stretch 几何面无位移＝STRETCH-ABSPOS 刀对 semantics 语料零 sweep 影响定谳）。rc1=2 rc2=2＝失败数退出非截断。
- s619.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613=s614=s615=s616=s617=s618＝链复挂第 18 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 23 连 s597→s619；双腿恒第 9 演）。

## b619 判定（bridge.out）＝条件双路径"不动"分支兑现（零新钉形）

- keys=**1737** fail=1 NEW=0 GONE=0；s619.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=b615=b616=b617=b618＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗桥触点核实：0174f69f4＝js_dom_bridge 触点 ×1，**零新增 #[test]**（全桥触点 diff 计数 0；part03.rs hyphens 断言 none→manual＝既有键值层面翻面非新钉，与 style-system 初值刀同窗同步）＝"不动"分支主预判兑现（≡OM-20/OM-18 零新钉形）。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r53＝2P 双腿（flaky 键持位第 8 轮，GONE 形未现）；select-event r58＝279 keys 8F Fset ≡ se.577；password r58＝225 keys 0F；mpreload r81＝15/15P 双腿（持位第 18 轮）；waapi r140＝picker P/checkmark F 双腿；focus-dynamic r99＝121 all P；onblur r51＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 19 轮）；族带 r52＝band8 hits=8×2（聚合形持位第 36 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 14 演。轮次标签全对齐（dialog r53／select-event r58／password r58／mpreload r81／waapi r140／focus-dynamic r99／onblur r51／band r52＝与 s619 布防计划算术一致）。

## 窗底账（s619 窗＝4435dbfe4→0d3073862＝13 commit 全列）

- 码 ×3：`68dd6dda3` hyphens 初值 none→manual（style-system default_impl/registry/types/extended/apply_coverage 5 files +32/-8）；`0174f69f4` gCS inset 族 resolved=used 回读＋gBCR/clientHeight 视图现算（engine js_dom_bridge 7 files +535＝桥触点 ×1 零新 #[test]）；`54d967087` STRETCH-ABSPOS abspos stretch margin-box 填充（layout-engine abspos.rs +56）。
- data ×0。
- docs ×10：本流 1（`0d3073862` s618 收账＋s619 布防）＋对端 9（`7a877f4ba` M4-a r2 勘误／`a744218f4` M4-a r2 收账笔／`c05e6de9f` CSSOM-INSETS-USEDECHO 收账／`d94aa355d` R5898 收账／`347349577` FAM-B 证伪笔回插／`4d4cc9fe8` ZRG 扩面立案／`21e6adb7a` insets census 定谳／`fb3d2d1b4` FAM-D1-b v2 fail-stop 判定／`489792c8c` R5899 收账）。

## 基线换装

- join：s619.stable **8517**（f1aef7b7/057764d3，同 s610=s612=s613=s614=s615=s616=s617=s618）。
- 全桥：b619 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**9ef937e2**/129,672,376 在锚（候选三十七确定换代兑现）。
