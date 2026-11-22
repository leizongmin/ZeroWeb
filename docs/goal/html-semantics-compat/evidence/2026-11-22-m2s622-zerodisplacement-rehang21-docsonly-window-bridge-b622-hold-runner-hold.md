# s622 收账证据——完全零位移链复挂第 21 连（docs-only 窗平凡形＝窗底账 8 commit 码 ×0）＋b622 keys 1737 持位（"不动"分支）＋runner a58c919e 三点恒持位

日期：2026-11-22（布防 10:31:10；候窗 ~5m15s；窗开 10:36:25；全毕 10:42:35）
环境：本机双 clone 并行；`/tmp/s622`；布防后同 session 内候窗 5m15s 开窗、三臂自动跑毕全链、当轮即收账。

## 窗与 legs

- 布防 10:31:10 两臂起动（本轮 waiter 由 rearm 监督者首迭代拉起＝rearm.s622.log "#1" 为首 spawn 固定措辞落账非真重臂，与 s620/s621 空日志形差异仅源于臂序不同；浸泡连续无 TIMEOUT 无中断）；候窗 ~5m15s（10:36:25 clean-soaked，procs=0 load<3 3×45s），工作树钉扎 `f33e72a07`（本流 s621 收账提交，push 落定后窗开＝legs.head 干净钉扎）＝legs.head。
- leg1 10:36:25→10:39:04（2m39s）rc=2＝完整有效腿，Compiling ×6（mtime 失效重编形 ≡s615/s618/s621 零码窗先例，release build 43.93s，runner md5 恒＝非码笔链形）；leg2 10:39:05→10:41:02（1m57s）rc=2 零编译。
- runner：pre=mid=post **`a58c919e`**/129,827,000 三点恒＝**a58c919e 持位观察兑现**（候选三十八在锚持位；候选四十挂号未兑现＝窗码面 runner 依赖域直源 ×0）。
- 桥测（run-legs 内，同 s621 recipe）：`./target/test-guard --compile-first --per-proc-mem 8 --total-mem 16 --time-limit 1200 -- cargo test -p zero-engine --lib js_dom_bridge`，build 34.30s（product-version/net/engine mtime 形 Compiling ×3）+test 28.51s，rcb=**101**。
- solo1 10:42:10→10:42:24（14s）／solo2 10:42:24→10:42:35（11s）均 rc=2；failset（`^Fail` 原始序）md5 `329e35ed7bd8b6e00f921f25671c57dd` 双 solo IDENT ≡s611–s622＝发射形同值第 15 连。
- all.done 10:42:35；窗起至全毕 **6m10s** 零空窗（两臂齐全第 21 演）。

## join 判定（analyze.out）＝docs-only 窗零位移

- `stable=8517 disagree=0 l1only=0 l2only=0`；8519 行 norm ×2＋canon ×2；集级 206 Fail/8305 Pass/5 Timeout/1 Unsupported（同 s621）。
- **NEW=0 GONE=0 FLIPS=0**＝完全零位移主预判精确兑现；本窗平凡形（窗底账 8 commit 全 docs/工具面＝码 ×0，位移无码面可归因）。rc1=2 rc2=2＝失败数退出非截断。
- s622.stable.canon md5 **`f1aef7b7d7eafeeef3dd0989ba6bc3d9`**（≡s610=s612=s613=s614=s615=s616=s617=s618=s619=s620=s621＝链复挂第 21 连）；keys md5 `057764d38ee311a47c34ecfc3e1e3192` 同值持位。
- 双腿 norm md5 双双 `c735d37188a0fa0ea338b47d2432a26f`（leg1 整腿复现第 26 连 s597→s622；双腿恒第 12 演）。

## b622 判定（bridge.out）＝条件双路径"不动"分支兑现

- keys=**1737** fail=1 NEW=0 GONE=0；s622.bridge.keys md5 `4e20b5df86bb5d08902d235c2e01e6ce`（≡b614=b615=b616=b617=b618=b619=b620=b621＝同值持位）。
- FAIL 同键持位＝`js_dom_bridge::computed_style::computed_calc_serialize_tests::min_max_single_arg_collapses_to_argument_terms`＝对端未修 v3 刀桥测期望（挂账续）；桥红待办仍归对端 v3 刀作者面（规则 9/10，本流不代修）。
- 窗桥触点核实：窗码面 Rust ×0（7a017f110 仅 goal 脚本 62 +23 bash 工具面＋json/md docs）＝js_dom_bridge 触点 ×0＝"不动"分支主预判兑现。

## 哨位（analyze2.out，SENTINELS_DONE）

dialog band r56＝2P 双腿（flaky 键持位第 11 轮，GONE 形未现）；select-event r61＝279 keys 8F Fset ≡ se.577；password r61＝225 keys 0F；mpreload r84＝15/15P 双腿（持位第 21 轮）；waapi r143＝picker P/checkmark F 双腿；focus-dynamic r102＝121 all P；onblur r54＝8P/2F Fset 双 IDENT ≡ob.s559（稳位第 22 轮）；族带 r55＝band8 hits=8×2（聚合形持位第 39 连，页级 T 零再现）；双发伪影 ×2 恒；爆栈 so1/so2/sob=0/0/0；'No such' 归零审计 analyze.out/analyze2.out 0/0 第 17 演。轮次标签全对齐（dialog r56／select-event r61／password r61／mpreload r84／waapi r143／focus-dynamic r102／onblur r54／band r55＝与 s622 布防计划算术一致）。

## 窗底账（s622 窗＝2319ac412→f33e72a07＝8 commit 全列）

- 码 ×0；data ×0（7a017f110 面码核验＝仅 `scripts/goals/62-wptfyi-html-infra.sh` +23 bash 工具面＋json/md docs；CODEFACE 过滤 rs/js/css/toml/lock 零命中＝Rust/wpt-runner 直源 ×0）。
- docs ×8：本流 1（`f33e72a07` s621 收账＋s622 布防）＋对端 7（`7a017f110` wptfyi-html 批十四 CI 周跑 infra 缺档定谳〔goal 脚本面〕／`c545fcfa6` 对端 R5901 收账〔携 96代 a58c919e 判代互证＝与本流 s620 候选三十八换代独立互证；py-routes faces allF=0 零存在独立核验〕／`1bf03aac6` 批十四勘误〔options-length-too-large 0 注册 TO 本地/CI 同形〕／`429329c55` learnings 动画时序域 flake 判别／`f5e69cd25`＋`38abd6dc9`＋`9437b0c66` wptfyi-css M4-b witness fail-stop 判定/发射方收账/勘误附注）。

## 基线换装

- join：s622.stable **8517**（f1aef7b7/057764d3，同 s610=s612=…=s621）。
- 全桥：b622 keys **1737**（4e20b5df 同值持位；GONE 1 挂账续＝对端面待修）。
- runner：**a58c919e**/129,827,000 在锚（候选三十八持位）。
