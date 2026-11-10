# Test262、Encoding 与 Wasm控制面

**2026-11-11 · ✅ 已完成（goal 归档，模式 A）**。DC-1~3 全部完成。终局快照
[coverage-2026-11-10](../../../compat/wptfyi/coverage-2026-11-10.md)
（run [42843305476](https://github.com/leizongmin/ZeroWeb/actions/runs/42843305476)
aggregate 落库，HEAD `91ab28c56`）：三目录合计 **1,393,984 ≥ 验收线
1,393,955**（余 +29），对 11-07 基线 1,317,955 **净增 76,029 ≥ 76,000**
——encoding +22,979（分母不变，未覆盖 23,847→868）、wasm +1,122（含
−7 上游漂移）、test262 0→51,928（run-1 +90 source-phase 切片 F2P）。
run 整体红仅系 workers 腿 4G 帽口杀（不进三目录，6G 修复已落）；邻域
回退终局复核全部定性（三目录零回退；workers/* −784 帽口面、dom/ranges
−93 与 shadow-dom/declarative −372 执行截断、css/selectors −4 与
css/values −3 跨流组合态位移归各流，[终局证据](evidence/dc3-final-reconciliation-2026-11-11.md)）。
收口取证：全仓 clippy 干净、`make build` 全绿、`make test` v8 workspace
腿 3833P/7F（7F 为 11-09 起各流在册的组合态既有集逐名同构，非本流引入；
中止语义后续腿未跑）、fmt 4 文件跨流漂移统一修复，见
[收口证据](evidence/goal-closure-2026-11-11.md)。

**卡点** 无。残余定性不入账不变（streams 16、wasm 引擎 ~56、
test262 已定性簇、engine 挂账 2 项，见下方 DC-2 行与各证据文）。

[执行契约](../wptfyi-runtime.md) · [全量预算](../../../compat/wptfyi-90-plan.md)

**2026-11-10 · Active**。DC-1 完成（test262 全量基线 53,166 案 96.39%、CI 三分片、
runner 探针两形态缺陷修复）；DC-2 encoding + wasm worker 变体面收口（runner
`f57d91099` +11,728 过、`c181cccab` +1,129 过）+ test262 三大失败簇定性
（eval-code 早期错误 / source-phase imports / import defer 均 V8 引擎侧，
[证据](evidence/dc2-test262-cluster-triage-2026-11-10.md)）+ export default
具名函数声明簇修复（+27 过）+ expressions 负例桶 import 形态严格化修复
（+67 过，[证据](evidence/dc2-test262-import-shape-fix-2026-11-10.md)）+
module 早期错误双通道预检＋import.meta 词法重写修复（+181 过，`42ab0ee46`，
[证据](evidence/dc2-test262-module-early-error-fix-2026-11-10.md)）+ TLA 判定
单调不减注入修复（+97 过，`59091ef2e`）+ TLA 深度门只认大括号修复（+30 过，
`5b8080ae2`，均见
[证据](evidence/dc2-test262-tla-injection-2026-11-10.md)）+ iframe realm JS
内建转发面修复（scoped eval＋50 内建转发＋Proxy 可构造化，`d92788bb1`，
+121 过，[证据](evidence/dc2-test262-realm-intrinsics-2026-11-10.md)）+
$262.agent SAB 管线切片（SAB 报告环形缓冲替换 postMessage 载体＋backing
store 跨 isolate registry＋双 isolate 前台任务泵＋runner 真线程 worker 宿主，
+116 过零翻红、Timeout 6 全回收，[证据](evidence/dc2-test262-sab-pipeline-2026-11-10.md)）+
ShadowRealm 特性切片（V8 harmony flag＋宿主 context 回调＋realm 转发＋
importValue 步序补丁，64 案 0P→59P，importValue 真加载 4 案划界 skip，
+59 过零翻红，[证据](evidence/dc2-test262-shadowrealm-2026-11-10.md)）+
encoding iframe 面收口（py-routes charset 端点族＋encoding 通道 `.window.js`
形态开＋fallback-encoding 父继承＋meta 引号语义，unsupported-labels 158/158P、
single-byte-decoder-iframe 168/168P，+326 过零翻红，[证据](evidence/
dc2-encoding-iframe-2026-11-10.md)）：净增账 **76,056 / 76,000——账面越过
76,000 验收线**。

**卡点** 无阻塞性卡点。净增 76,000 账构成：test262 ~51,528 + encoding 自动
+10,925 + encoding worker +11,728 + wasm worker +1,129 + export-default 簇
+27 + import 形态簇 +67 + realm 内建簇 +121 + SAB 管线簇 +116 + ShadowRealm
簇 +59 + encoding iframe 面 +326 = **76,056**。残余定性不入账：streams
16 案（realms 12T——srcdoc+postMessage→eval 跨 realm 泵面缺；invalid-realm
4F——detached realm 流写语义）+ wasm 引擎小簇 ~56 + test262 已定性簇
（[归簇证据](evidence/dc2-test262-realm-intrinsics-2026-11-10.md)、
[SAB 证据](evidence/dc2-test262-sab-pipeline-2026-11-10.md)、
[ShadowRealm 证据](evidence/dc2-test262-shadowrealm-2026-11-10.md)：
ShadowRealm 残余 1 案 kernel 侧 + 4 划界 skip；tco 34 引擎侧不可回收——V8
未实现规范尾调用；TLA 残余 15；gc 15；Atomics coerce 残余 8）。
engine 挂账 2 项（timer 异常无 window error 报告、unhandledrejection 未实现）
见证据文。

**下一步（11-10 在飞账，已完成）** DC-3 四步对账（快照落库核验/净增
对账/邻域回退复核/守卫证据）已由终局证据文收口，入上方 DC-3 里程碑行；
gc 桥 +11、模块加载顺序 +18 两簇 defer 不入账（见
[source-phase 证据](evidence/dc3-source-phase-imports-2026-11-10.md) 挂账节）。

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | **完成** | [DC-1 全量基线](evidence/dc1-full-baseline-2026-11-10.md)——53,166 案 96.39%，变体/归簇定量，CI 三分片 + filter 多段语义，探针缺陷两形态修复；[通道试跑](evidence/dc1-channel-smoke-2026-11-09.md)；发布快照基线 1,317,955 / 1,398,399 |
| DC-2 按簇收敛 | **完成（账面 76,056/76,000）** | [encoding 定量](evidence/dc2-encoding-gap-2026-11-10.md) + [encoding worker 收口](evidence/dc2-worker-variants-2026-11-10.md)（`f57d91099`，+11,728 过、组残余 ~1,194）+ [wasm 定量与 worker 收口](evidence/dc2-wasm-gap-2026-11-10.md)（`c181cccab`，+1,129 过、残余 1,674 = 引擎 ~56 + 划界 ~1,369）+ [test262 三簇定性](evidence/dc2-test262-cluster-triage-2026-11-10.md)（eval-code 早期错误 / source-phase / import defer 均 V8 引擎侧，~250 案不回收）+ [export default 具名声明簇修复](evidence/dc2-test262-export-default-fix-2026-11-10.md)（+27 过）+ [expressions 负例桶 import 形态严格化](evidence/dc2-test262-import-shape-fix-2026-11-10.md)（+67 过：63 import-defer + 4 importcall；8 strict callexpression 三引擎一致定性引擎侧）+ [module 早期错误双通道预检＋import.meta 词法重写](evidence/dc2-test262-module-early-error-fix-2026-11-10.md)（`42ab0ee46`，+181 过：module-code +63、import.meta 面 +17、expressions 救回 +101，零翻红；HostCallback Plain/Scoped enum＋scoped 回调窗转借 HandleScope＋has_unpublished_static_import 豁免＋RewriteCtx 状态机镜像）+ [TLA 判定单调不减注入](evidence/dc2-test262-tla-injection-2026-11-10.md)（`59091ef2e`，+97 过零翻红：扫描器命中恒保持＋V8 async/classic 双腿探针仲裁 false 面＋TLA_OVERRIDES 线程表注入 4 判定点；两轮翻红回滚教训在案——export/import 源函数体包裹必拒、`await <模板/正则>` sloppy 语义误合法）+ [TLA 深度门只认大括号](evidence/dc2-test262-tla-injection-2026-11-10.md)（`5b8080ae2`，+30 过零翻红：括号不引入函数作用域，括号内 await 命中——上轮「for-await 19」勘误定性为 export-class extends 实参簇＋调用实参形）+ [iframe realm JS 内建转发面＋三簇定性](evidence/dc2-test262-realm-intrinsics-2026-11-10.md)（`d92788bb1`，+121 过零翻红：scoped eval＋50 内建转发本体＋Proxy 可构造化——-realm 面 33P/189F→132P/90F、staging/sm 面 +22；归簇定性入册：tco 34 引擎侧不可回收（V8 未实现 PTC）/SAB 106 可修（backing store 跨 isolate 管线方案已勘明）/ShadowRealm 61 特性切片；createRealm 281 宽收勘误为 eval/intrinsics 族 ~50 本轮已修）+ [$262.agent SAB 管线切片](evidence/dc2-test262-sab-pipeline-2026-11-10.md)（+116 过零翻红：SAB 报告环形缓冲替换 postMessage 载体（结构性 report 竞态根因——忙等自旋与消息派发互斥）、通道门＋`__zwSabRef` token＋Rust backing store registry＋worker 派发物化、双 isolate 前台任务泵＋微任务检查点、runner 真线程 worker 宿主；Atomics 面 258P/118F/6T→374P/8F/0T、Timeout 全回收；dom 锚点非 Range 族翻红 0）+ [ShadowRealm 特性切片](evidence/dc2-test262-shadowrealm-2026-11-10.md)（+59 过零翻红：harmony flag＋宿主 context 回调＋realm 转发＋importValue 步序补丁；64 案 0P→59P/1F/0T、importValue 真加载 4 案 SIGTRAP 划界 skip；-realm 面 +11 翻红 0、sm 面 1313P/91F 零漂移、engine lib 7F 集逐字一致）；剩余簇：TLA 残余 15（template-literal 3/语义时序 12）、gc（15 案）、Atomics coerce 残余 8、ShadowRealm 残余 5（1 kernel 侧 + 4 skip）、escaped-await 探针结构残余）+ [encoding iframe 面收口](evidence/dc2-encoding-iframe-2026-11-10.md)（+326 过零翻红：py-routes charset 端点族＋encoding 通道 `.window.js` 形态开（`extra_forms`＋4 源放行＋worker 腿 `.any.js` 门限修复同键双跑）＋fallback-encoding 同源祖先继承＋meta 引号感知正则；unsupported-labels 158/158P、single-byte-decoder-iframe 168/168P、encoding 全通道 23,800→24,126、dom-nodes/R141 双侧零漂移；streams 16 案挂账定性——realms 12T 跨 realm 泵、invalid-realm 4F detached realm 流写） |
| DC-3 全量对账 | **完成（净增 76,029/76,000）** | 首跑快照 1,393,894（差 61）+ [source-phase imports 切片](evidence/dc3-source-phase-imports-2026-11-10.md)（`91ab28c56`，+90 过零翻红）+ [workers 帽口归因](evidence/dc3-workers-shard-cap-attribution-2026-11-11.md)（`434dd2f89` 6G 修复）+ [终局对账](evidence/dc3-final-reconciliation-2026-11-11.md)：run [42843305476](https://github.com/leizongmin/ZeroWeb/actions/runs/42843305476) aggregate 落库 [coverage-2026-11-10](../../../compat/wptfyi/coverage-2026-11-10.md)——1,393,984 ≥ 1,393,955（余 +29）、净增 76,029 ≥ 76,000；三 test262 分片文件数两轮逐字一致无截断；邻域回退文件级全归因（三目录零回退） |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
（2026-11-09：`resources/test262/` 为本目标新占用 fetch 面，`wpt-data/third_party/test262/`
vendored 子树为本目标独占，无共享写面冲突。）
（2026-11-10 碰头：html-semantics-compat 流把 3 个 probe `.html` 提交进 gitignored
语料树 `wpt-data/html/semantics/scripting-1/the-script-element/module/`
（`42274e6e9`），触发 fetch lib.sh:62-67 幂等快路径「目录已含 .html 即跳过列
目录」——新 CI checkout 不再拉取该目录 64 个上游 module 用例，CI html-sem
分片 −126 Pass（本流三目录不计，邻域核对见
[dc3 证据](evidence/dc3-source-phase-imports-2026-11-10.md)）。修复归该流
（移位 probe 或 fast-path 计数门），本流不动；42843305476 快照 html 邻域将
复现同缺口，对账按已归因处理。）
