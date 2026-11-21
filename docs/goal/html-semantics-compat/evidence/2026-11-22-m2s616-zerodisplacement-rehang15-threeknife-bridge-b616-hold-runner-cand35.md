# s616 收账证据——完全零位移链复挂第 15 连（三码刀入窗非平凡形）＋b616 keys 1737 持位（对端未修）＋runner 候选三十五确定换代 18f1d1f5→25a331a3

日期：2026-11-22（候窗 05:29–05:44:13；窗开 05:44:13；全毕 05:51:35）
环境：本机双 clone 并行（对端 wptfyi-css/rendering 流）；`/tmp/s616`。

## 窗与 legs

- 布防 05:29 三臂起动（前 session 收账后即时布防）；候窗 ~15m（对端 probe contention 退坡后 05:44:12 clean-soaked，procs=0 load<3 3×45s），工作树已在 `9e0f7f415`（本流 s615 收账提交）＝legs.head 钉扎。
- leg1 05:44:13→05:47:02（2m49s）rc=2＝完整有效腿，Compiling ×9（40bf020a4 runner 码刀链形）；leg2 05:47:02→05:49:19（2m17s）rc=2 零编译。
- runner：pre `18f1d1f5`/129,583,048（候选三十四在锚）→ mid=post `25a331a3`/129,515,952（−67,096B）＝**候选三十五挂号转确定形兑现**（窗码面 runner 依赖域直源：40bf020a4 tests/wpt-runner main.rs +16／testharness.rs +56／testharness_py_routes +116＝DC-2 批十二 speculative-parsing 通道）；换代值与对端 rendering R5894 91代 判代预期 pre=25a331a3 独立互证。
- 桥测（run-legs 内，同 s615 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，23.44s，rcb=**101**。
- solo1 05:51:08→05:51:20（12s）／solo2 05:51:20→05:51:35（15s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s616＝发射形同值第 9 连。
- legs.done 05:50:56；all.done 05:51:35；窗起至全毕 **7m22s** 零空窗（两臂齐全第 15 演，rearm 零触发）。

## join 判定（analyze.out）＝三码刀入窗零位移非平凡形

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s615）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗非平凡（码 ×3 入窗：fa89cbb31 TEXT-INTERP 四属性九站注册／115adf54d OM-19 keyword 面／40bf020a4 runner 通道面）——CSS 属性注册/序列化臂与 runner 通道面均非 semantics 语料敏感面的预判兑现；与对端 rendering corpus 零回归互证（R5892/R5893/R5894 stable3 976 键字节恒等）。rc1=2 rc2=2＝失败数退出非截断。
- s616.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613=s614=s615＝链复挂第 15 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 20 连 s597→s616；双腿恒恢复第 6 演）。

## b616 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s616.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=b615＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续，飞书已通报）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗内桥触点核实：fa89cbb31 computed_style.rs +40＝to_css 序列化臂三函数零新增 #[test]；115adf54d computed_style.rs +1＝PointerEventsValue::BoundingBox 序列化臂零新增 #[test]＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r50＝2P 双腿（flaky 键持位第 5 轮，GONE 形未现）；select-event r55＝279 keys 8F Fset ≡ se.577；password r55＝225 keys 0F；mpreload r78＝15/15P 双腿（持位第 15 轮）；waapi r137＝picker P/checkmark F 双腿；focus-dynamic r96＝121 all P；onblur r48＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 16 轮）；族带 r49＝band8 hits=8×2（聚合形持位第 33 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 11 演。轮次标签全对齐（dialog r50／select-event r55／password r55／mpreload r78／waapi r137／focus-dynamic r96／onblur r48／band r49＝与 s616 布防计划算术一致）。

## 窗底账（s616 窗＝7503883e0→9e0f7f415＝14 commit 全列）

- 码 ×3：`fa89cbb31` TEXT-INTERP 入库（9 files +263＝text-justify/hyphenate-limit-chars/line-break/hyphenate-character 四属性九站注册；engine computed_style.rs +40 序列化臂零 #[test]）；`115adf54d` OM-19 入库（8 files +134＝typed-om keyword 面；bridge +1 序列化臂零 #[test]＋js_dom_shim part10 +22）；`40bf020a4` wpt-runner DC-2 批十二（9 files +1683＝main.rs +16/testharness.rs +56/py_routes +116＋Makefile +12＋goal docs）。
- data ×0。
- docs ×11：本流 1（`9e0f7f415` s615 收账＋s616 布防）＋对端 10（rendering 4＝`260056b49` R5891/`5f0735aa1` R5892/`7a26bc10d` R5893/`205aa79d4` R5894；wptfyi-css 6＝`53a670cad` TEXT-INTERP 收账/`a8c7c944f` 续开轮收账/`058e59b12` OM-19 复锚/`f0705ce72` r4 判定专行/`1c2e65407` r4 判死笔/`131086735` TEXT-INTERP-r2 解剖）。

## 基线换装

- join：s616.stable **8517**（f1aef7b7/057764d3，同 s610=s612=s613=s614=s615）。
- 全桥：b616 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**25a331a3**/129,515,952 在锚（候选三十五确定换代兑现）。
