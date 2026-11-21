# s615 收账证据——完全零位移链复挂第 14 连＋b615 keys 1737 持位（对端未修）＋runner 候选三十四挂号未兑现＋限流断点零损续跑第 2 演

日期：2026-11-22（候窗 04:27–05:08:12；窗开 05:08:13；全毕 05:14:34）
环境：本机双 clone 并行（对端 wptfyi-css/rendering 流）；`/tmp/s615`。

## 限流断点与自动续跑（本轮特记，s605 无损补记模式第 2 演）

- 前一 session 04:26 完成本流 s614 收账提交（7503883e0）＋s615 布防（04:26 预装载全套、04:27:11 三臂起动），05:12:25 API 限流断点（leg2 尾声在飞，与 s614 断点在窗开前互补＝两形态均证臂内存活）。
- 断点时臂内 legs 已在飞（窗开 05:08:13），session 死亡不影响后台三臂：legs.done 05:13:52 → analyze.done 05:14:12 → solos/all.done 05:14:34 全链自动跑毕＝限流断点零损续跑第 2 演；收账由续 session 本轮补记。
- 窗开 05:08:12 clean-soaked（procs=0 load<3 3×45s），工作树已在 `7503883e0`（docs ×2 零码刀窗）＝legs.head 钉扎。

## 窗与 legs

- leg1 05:08:13→05:10:53（2m40s）rc=2＝完整有效腿，Compiling ×6（product-version→net→engine→page-runtime→webview→zero-wpt-runner 依赖链，窗码面零码刀＝git 操作 mtime 失效内容零变重编，md5 pre=mid=post 恒）；leg2 05:10:53→05:12:55（2m02s）rc=2 零编译。
- runner：pre=mid=post `18f1d1f5`/129,583,048 三点恒＝**候选三十四挂号未兑现**（窗码面零 runner 依赖域直源；持位观察形）。
- 桥测（run-legs 内，同 s614 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，21.94s，rcb=**101**。
- solo1 05:14:12→05:14:23（11s）／solo2 05:14:23→05:14:34（11s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s615＝发射形同值第 8 连。
- legs.done 05:13:52；all.done 05:14:34；窗起至全毕 **6m22s** 零空窗（两臂齐全第 14 演，rearm 零触发）。

## join 判定（analyze.out）

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s614）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现（零码刀窗，仅 docs ×2 入窗＝平凡兑现形，主预判兑现即窗底账完整性反证）；rc1=2 rc2=2＝失败数退出非截断。
- s615.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613=s614＝链复挂第 14 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 19 连 s597→s615；双腿恒恢复第 5 演）。

## b615 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s615.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续，飞书已通报）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗内桥触点核实：零码刀窗＝js_dom_bridge 零触点，桥测零噪声兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r49＝2P 双腿（flaky 键持位第 4 轮，GONE 形未现）；select-event r54＝279 keys 8F Fset ≡ se.577；password r54＝225 keys 0F；mpreload r77＝15/15P 双腿（持位第 14 轮）；waapi r136＝picker P/checkmark F 双腿；focus-dynamic r95＝121 all P；onblur r47＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 15 轮）；族带 r48＝band8 hits=8×2（聚合形持位第 32 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 10 演。轮次标签全对齐（dialog r49／select-event r54／password r54／mpreload r77／waapi r136／focus-dynamic r95／onblur r47／band r48＝与 s615 布防计划算术一致）。

## 窗底账（s615 窗＝b1fc0f8ea→7503883e0＝2 commit 全列）

- 码 ×0＝**零码刀窗**（join 零位移平凡形；runner 持位、桥测持位均与窗码面一致）。
- data ×0。
- docs ×2：对端/共享 1（`6e0201dcf` CHANGELOG 补记 typed-om/mathfn 续刀＋ROADMAP/architecture 口径刷新）＋本流 1（`7503883e0` s614 收账＋s615 布防）。

## 基线换装

- join：s615.stable **8517**（f1aef7b7/057764d3，同 s610=s612=s613=s614）。
- 全桥：b615 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**18f1d1f5**/129,583,048 在锚（候选三十四未兑现＝持位）。
