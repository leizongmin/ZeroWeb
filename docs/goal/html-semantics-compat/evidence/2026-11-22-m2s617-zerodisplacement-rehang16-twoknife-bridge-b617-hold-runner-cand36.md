# s617 收账证据——完全零位移链复挂第 16 连（双码刀入窗）＋b617 keys 1737 持位（OM-20 零桥触点）＋runner 候选三十六确定换代 25a331a3→96d18fc4

日期：2026-11-22（候窗 05:58–06:44:13；窗开 06:44:13；全毕 06:51:10）
环境：本机双 clone 并行（对端 wptfyi-css/rendering 流）；`/tmp/s617`。

## 窗与 legs

- 布防 05:58 三臂起动（前 session 收账后即时布防）；候窗 ~46m（对端 kb140 batch＋probe contention 退坡后 06:44:12 clean-soaked，procs=0 load<3 3×45s），工作树已在 `96cfa7790`（本流 s616 收账提交）＝legs.head 钉扎。
- leg1 06:44:13→06:47:05（2m52s）rc=2＝完整有效腿，Compiling ×8（双码刀依赖链）；leg2 06:47:05→06:49:10（2m05s）rc=2 零编译。
- runner：pre `25a331a3`/129,515,952（候选三十五在锚）→ mid=post `96d18fc4`/129,515,616（−336B）＝**候选三十六挂号转确定形兑现**（窗码面双码刀依赖链：330d7f6f0 OM-20 js_dom_shim part10 +21 嵌入 runner 二进制＋f3001b563 TEXT-INTERP-F2 style-system parse.rs +50；leg1 Compiling ×8＝码刀链形；leg2 零编译）。
- 桥测（run-legs 内，同 s616 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，27.08s，rcb=**101**。
- solo1 06:50:47→06:50:58（11s）／solo2 06:50:58→06:51:10（12s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s617＝发射形同值第 10 连。
- all.done 06:51:10；窗起至全毕 **6m57s** 零空窗（两臂齐全第 16 演，rearm 零触发）。

## join 判定（analyze.out）＝双码刀入窗零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s616）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗非平凡（码 ×2 入窗：330d7f6f0 OM-20 min-size computed 'auto' 分面保形＝typed-om shim JS 层／f3001b563 TEXT-INTERP-F2 hyphenate-limit-chars calc() 分量接受＝style-system parse 层）——shim/parse 值层面刀均非 semantics 语料敏感面的预判兑现（≡s616 三码刀先例续形）。rc1=2 rc2=2＝失败数退出非截断。
- s617.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613=s614=s615=s616＝链复挂第 16 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 21 连 s597→s617；双腿恒恢复第 7 演）。

## b617 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s617.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=b615=b616＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续，飞书已通报）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗内桥触点核实：330d7f6f0 OM-20 零 js_dom_bridge 文件触点、零新增 #[test]（typed-om shim JS 层 ≡OM-18 形）；f3001b563 仅 style-system parse.rs＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r51＝2P 双腿（flaky 键持位第 6 轮，GONE 形未现）；select-event r56＝279 keys 8F Fset ≡ se.577；password r56＝225 keys 0F；mpreload r79＝15/15P 双腿（持位第 16 轮）；waapi r138＝picker P/checkmark F 双腿；focus-dynamic r97＝121 all P；onblur r49＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 17 轮）；族带 r50＝band8 hits=8×2（聚合形持位第 34 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 12 演。轮次标签全对齐（dialog r51／select-event r56／password r56／mpreload r79／waapi r138／focus-dynamic r97／onblur r49／band r50＝与 s617 布防计划算术一致）。

## 窗底账（s617 窗＝9e0f7f415→96cfa7790＝9 commit 全列）

- 码 ×2：`330d7f6f0` OM-20 入库（3 files +77＝engine js_dom_shim part10-typed-om.js +21／evidence +54／master +3；零 rust 桥触点）；`f3001b563` TEXT-INTERP-F2 入库（style-system property/parse.rs +50＝hyphenate-limit-chars calc() 分量接受）。
- data ×0。
- docs ×7：本流 1（`96cfa7790` s616 收账＋s617 布防）＋对端 6（`a288f55aa` FAM-A 证伪附笔／`3cd9d1faa` 插值鲸池定稿／`5e01f4645` 插值鲸池侦查／`a750b1202` TEXT-INTERP-F2 收账／`2a95906e0` rendering R5895／`881d5969f` learnings daily index）。

## 基线换装

- join：s617.stable **8517**（f1aef7b7/057764d3，同 s610=s612=s613=s614=s615=s616）。
- 全桥：b617 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**96d18fc4**/129,515,616 在锚（候选三十六确定换代兑现）。
