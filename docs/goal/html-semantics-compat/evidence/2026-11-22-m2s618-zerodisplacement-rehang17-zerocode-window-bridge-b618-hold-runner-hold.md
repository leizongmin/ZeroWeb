# s618 收账证据——完全零位移链复挂第 17 连（零码窗＝docs ×7）＋b618 keys 1737 持位（"不动"分支）＋runner 96d18fc4 恒持位

日期：2026-11-22（布防 06:57:48；候窗 ~1h12m；窗开 08:09:36；全毕 08:15:55）
环境：本机双 clone 并行（对端 wptfyi-css/rendering 流）；`/tmp/s618`；限流断点跨 session 后台三臂自动跑毕全链，续 session 收账（≡s614/s615 无损续跑模式）。

## 窗与 legs

- 布防 06:57:48 三臂起动；候窗 ~1h12m（对端 contention 退坡后 08:09:36 clean-soaked，procs=0 load<3 3×45s），工作树钉扎 `4435dbfe4`（本流 s617 收账提交）＝legs.head。
- leg1 08:09:36→08:12:18（2m42s）rc=2＝完整有效腿，Compiling ×6（product-version/net/engine/page-runtime/webview/wpt-runner 全链＝mtime 失效重编形 ≡s615 先例，零码窗 md5 恒＝非码笔链形）；leg2 08:12:18→08:14:20（2m02s）rc=2 零编译。
- runner：pre=mid=post `96d18fc4`/129,515,616 三点恒＝**96d18fc4 持位观察兑现**（候选三十六在锚持位；候选三十七挂号未兑现＝零码窗无 runner 依赖域直源）。
- 桥测（run-legs 内，同 s617 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，build 44.23s（Compiling ×3 同 mtime 形）+test 23.37s，rcb=**101**。
- solo1 08:15:29→08:15:43（14s）／solo2 08:15:43→08:15:55（12s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s618＝发射形同值第 11 连。
- all.done 08:15:55；窗起至全毕 **6m19s** 零空窗（两臂齐全第 17 演，rearm 零触发）。

## join 判定（analyze.out）＝零码窗零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s617）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗平凡形（码 ×0＝docs ×7 零码窗，位移预判无码面可归因）。rc1=2 rc2=2＝失败数退出非截断。
- s618.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613=s614=s615=s616=s617＝链复挂第 17 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 22 连 s597→s618；双腿恒第 8 演）。

## b618 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s618.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=b615=b616=b617＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗桥触点核实：零码窗＝js_dom_bridge 触点 ×0＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r52＝2P 双腿（flaky 键持位第 7 轮，GONE 形未现）；select-event r57＝279 keys 8F Fset ≡ se.577；password r57＝225 keys 0F；mpreload r80＝15/15P 双腿（持位第 17 轮）；waapi r139＝picker P/checkmark F 双腿；focus-dynamic r98＝121 all P；onblur r50＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 18 轮）；族带 r51＝band8 hits=8×2（聚合形持位第 35 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 13 演。轮次标签全对齐（dialog r52／select-event r57／password r57／mpreload r80／waapi r139／focus-dynamic r98／onblur r50／band r51＝与 s618 布防计划算术一致）。

## 窗底账（s618 窗＝96cfa7790→4435dbfe4＝7 commit 全列）

- 码 ×0；data ×0。
- docs ×7：本流 1（`4435dbfe4` s617 收账＋s618 布防）＋对端 6（`0449856ab` R5896 收账／`f8feb2afb` R5897 收账／`973dc6021` FAM-F aspect-ratio 插值刀认领／`a41f0bc47` M4-a 判定笔 v1 弃置／`4dd0fa0b6` FAM-D1 探针判定／`7a3063ffc` CSSOM-INSETS 认领转移）。

## 基线换装

- join：s618.stable **8517**（f1aef7b7/057764d3，同 s610=s612=s613=s614=s615=s616=s617）。
- 全桥：b618 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**96d18fc4**/129,515,616 在锚（候选三十六持位）。
