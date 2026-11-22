# s623 收账证据——完全零位移链复挂第 22 连（docs/网站面窗平凡形＝窗底账 3 commit 码 ×0）＋b623 keys 1737 持位（"不动"分支）＋runner a58c919e 三点恒持位

日期：2026-11-22（布防 10:48:07；候窗 ~12m16s；窗开 11:00:23；全毕 11:06:10）
环境：本机双 clone 并行；`/tmp/s623`；布防后同 session 内候窗 12m16s 开窗、三臂自动跑毕全链、当轮即收账。

## 窗与 legs

- 布防 10:48:07 三臂起动（恢复 s620/s621 臂序＝waiter 直臂＋rearm 监督者＋post 分开臂；rearm.s623.log 不存在＝零真重臂）；候窗 ~12m16s（对端 kb147/kb148 判代 batch contention 退坡后 11:00:23 clean-soaked，布防期 host 15min load 6.53→1min 1.75 退坡，procs=0 load<3 3×45s），工作树钉扎 `8b22b0aa4`（本流 s622 收账提交，push 落定后窗开＝legs.head 干净钉扎）＝legs.head。
- leg1 11:00:23→11:02:46（2m23s）rc=2＝完整有效腿，Compiling ×6（mtime 失效重编形 ≡s615/s618/s621/s622 零码窗先例，release build 38.31s，runner md5 恒＝非码笔链形）；leg2 11:02:46→11:04:39（1m53s）rc=2 零编译。
- runner：pre=mid=post **`a58c919e`**/129,827,000 三点恒＝**a58c919e 持位观察兑现**（候选三十八在锚持位；候选四十一挂号未兑现＝窗码面 runner 依赖域直源 ×0）。
- 桥测（run-legs 内，同 s622 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，build 33.14s（test profile mtime 形）+test 20.31s，rcb=**101**。
- solo1 11:05:48→11:05:59（11s）／solo2 11:05:59→11:06:10（11s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s623＝发射形同值第 16 连。
- all.done 11:06:10；窗起至全毕 **5m47s** 零空窗（两臂齐全第 22 演）。

## join 判定（analyze.out）＝docs/网站面窗零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s622）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗平凡形（窗底账 3 commit 全 docs/网站面＝码 ×0，位移无码面可归因）。rc1=2 rc2=2＝失败数退出非截断。
- s623.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=…=s622＝链复挂第 22 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 27 连 s597→s623；双腿恒第 13 演）。

## b623 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s623.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=…=b622＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗桥触点核实：窗码面 Rust ×0（f6e8ddd51 R5902 收账 docs／e51257edb website 周报＝website/index.html+updates.json 静态站点面非产品码）＝js_dom_bridge 触点 ×0＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r57＝2P 双腿（flaky 键持位第 12 轮，GONE 形未现）；select-event r62＝279 keys 8F Fset ≡ se.577；password r62＝225 keys 0F；mpreload r85＝15/15P 双腿（持位第 22 轮）；waapi r144＝picker P/checkmark F 双腿；focus-dynamic r103＝121 all P；onblur r55＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 23 轮）；族带 r56＝band8 hits=8×2（聚合形持位第 40 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 18 演。轮次标签全对齐（dialog r57／select-event r62／password r62／mpreload r85／waapi r144／focus-dynamic r103／onblur r55／band r56＝与 s623 布防计划算术一致）。

## 窗底账（s623 窗＝f33e72a07→8b22b0aa4＝3 commit 全列）

- 码 ×0；data ×0（CODEFACE 过滤 rs/js/css/toml/lock/py/sh/scripts 零命中；e51257edb 面＝website/index.html+updates.json 静态站点面非产品码）。
- docs ×3：本流 1（`8b22b0aa4` s622 收账＋s623 布防）＋对端 2（`f6e8ddd51` R5902 收账〔携 96代 a58c919e 持位三证判代互证第六连＝与本流 a58c919e 持位独立互证；对端两读合本流 f33e72a07〕／`e51257edb` website 周报 2026-11-22〔全量 WPT 84.08%〕）。

## 基线换装

- join：s623.stable **8517**（f1aef7b7/057764d3，同 s610=s612=…=s622）。
- 全桥：b623 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**a58c919e**/129,827,000 在锚（候选三十八持位）。
