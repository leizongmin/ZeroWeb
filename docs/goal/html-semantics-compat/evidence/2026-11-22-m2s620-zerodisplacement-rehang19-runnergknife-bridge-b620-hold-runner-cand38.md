# s620 收账证据——完全零位移链复挂第 19 连（runner 码刀入窗非平凡形）＋b620 keys 1737 持位（"不动"分支）＋runner 候选三十八确定换代 9ef937e2→a58c919e

日期：2026-11-22（布防 09:51:39；候窗 ~2m43s；窗开 09:54:22；全毕 10:01:03）
环境：本机双 clone 并行；`/tmp/s620`；布防后同 session 内候窗 2m43s 开窗、三臂自动跑毕全链、当轮即收账。

## 窗与 legs

- 布防 09:51:39 三臂起动（前 session 收账 commit+push 后即时布防；窗开 09:54:22 晚于 push 落定＝legs.head 钉扎 `f8f39b35f` 干净态）；候窗 ~2m43s（09:54:22 clean-soaked，procs=0 load<3 3×45s）。
- leg1 09:54:22→09:57:07（2m45s）rc=2＝完整有效腿，Compiling ×6（runner 刀链形）；leg2 09:57:07→09:59:09（2m02s）rc=2 零编译。
- runner：pre `9ef937e2`/129,672,376 → mid=post **`a58c919e`**/129,827,000（**+154,624B**）＝**候选三十八条挂号转确定形兑现**（窗码面 runner 自域直源：3ad25d78c wpt-runner DC-2 批十三＝testharness.rs +7＋testharness_py_routes/misc.rs +326＋mod.rs +28＋goal 脚本 62 +11；leg1 Compiling ×6＝码刀链形）。
- 桥测（run-legs 内，同 s619 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，build 39.13s+test 28.13s，rcb=**101**。
- solo1 10:00:32→10:00:51（19s）／solo2 10:00:51→10:01:03（12s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s620＝发射形同值第 13 连。
- all.done 10:01:03；窗起至全毕 **6m41s** 零空窗（两臂齐全第 19 演，rearm 零触发）。

## join 判定（analyze.out）＝runner 码刀入窗零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s619）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗非平凡（码 ×1 入窗＝3ad25d78c runner 侧 .py 路由/备料刀，wptfyi-html query-encoding 面）——runner 基建刀对 html/semantics 语料零 sweep 影响定谳（sweep 走文件路径直读，新路由仅 wptfyi-html 案级注册）。rc1=2 rc2=2＝失败数退出非截断。
- s620.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613=s614=s615=s616=s617=s618=s619＝链复挂第 19 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 24 连 s597→s620；双腿恒第 10 演）。

## b620 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s620.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=b615=b616=b617=b618=b619＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗桥触点核实：3ad25d78c 纯 wpt-runner 域＝js_dom_bridge 触点 ×0＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r54＝2P 双腿（flaky 键持位第 9 轮，GONE 形未现）；select-event r59＝279 keys 8F Fset ≡ se.577；password r59＝225 keys 0F；mpreload r82＝15/15P 双腿（持位第 19 轮）；waapi r141＝picker P/checkmark F 双腿；focus-dynamic r100＝121 all P；onblur r52＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 20 轮）；族带 r53＝band8 hits=8×2（聚合形持位第 37 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 15 演。轮次标签全对齐（dialog r54／select-event r59／password r59／mpreload r82／waapi r141／focus-dynamic r100／onblur r52／band r53＝与 s620 布防计划算术一致）。

## 窗底账（s620 窗＝0d3073862→f8f39b35f＝4 commit 全列）

- 码 ×1：`3ad25d78c` wptfyi-html DC-2 批十三 query-encoding 四端点 .py 备料腿（wpt-runner 3 files +361＝testharness.rs +7/py_routes misc.rs +326/mod.rs +28；goal 脚本 62 +11＋evidence json/md＋master.md）。
- data ×0。
- docs ×3：`526a78cec` wptfyi-html 批十四备料核查／`ca9a46f02` 对端 R5900 收账〔携 95代 9ef937e2 判代互证＝与本流 s619 候选三十七换代独立互证〕／`f8f39b35f` 本流 s619 收账＋s620 布防。

## 教训条目（窗底账完整性系列）

布防时窗码面捕获犯 tip-only 错误：post-pull 仅核验了区间 tip commit（ca9a46f02）未跑 `git log A..B` 全 commit 核验，漏看同区间 526a78cec/3ad25d78c（码刀）——≡对端 R5895 同名教训重现；后果良性（布防 join/runner 预判均带条件分支覆盖＝非平凡形归因与候选三十八挂号按设计命中），但 lesson 固化＝**pull 携入判读必须 `git log A..B` 区间全 commit 核验，布防时窗码面捕获与收账判读同规**。

## 基线换装

- join：s620.stable **8517**（f1aef7b7/057764d3，同 s610=s612=…=s619）。
- 全桥：b620 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**a58c919e**/129,827,000 在锚（候选三十八确定换代兑现）。
