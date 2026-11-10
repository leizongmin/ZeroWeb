# Test262、Encoding 与 Wasm控制面

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
+121 过，[证据](evidence/dc2-test262-realm-intrinsics-2026-11-10.md)）：
净增账 ~75,555 / 76,000。

**卡点** 净增 76,000 账：test262 ~51,528 + encoding 自动 +10,925 + encoding
worker +11,728 + wasm worker +1,129 + export-default 簇 +27 + import 形态簇
+67 + realm 内建簇 +121 ≈ 75,555，余 ~445：encoding 残余 ~1,194 中的可回收
项（iframe 面 ~330——unsupported-labels 158 + single-byte-decoder-iframe 168
需 iframe charset→document.encoding→TextDecoder 管道；缺语料 2 源已关闭——
pin 404 不可补拉，DC-3 同 revision 归一自动出分母）+ wasm 引擎小簇 ~56 +
test262 剩余簇已定性（[归簇证据](evidence/dc2-test262-realm-intrinsics-2026-11-10.md)：
SAB clone 106 可修——真线程 worker 已落地、缺 backing store 跨 isolate 管线；
ShadowRealm 61 特性切片——构造器＋evaluate＋包装语义；tco 34 引擎侧不可回收
——V8 未实现规范尾调用；TLA 残余 15——template-literal 3 + 语义/时序 12；
escaped-await IIFE 探针结构残余；expressions 负例桶已关闭）。engine 挂账 2 项
（timer 异常无 window error 报告、unhandledrejection 未实现）见证据文。

**下一步** ① SAB 管线切片（~106：通道能力门＋`{"__zwSabRef"}` token＋Rust
backing store registry＋worker 派发侧物化，[方案已勘明](evidence/dc2-test262-realm-intrinsics-2026-11-10.md)）；
② ShadowRealm 特性切片（61，s376 内建转发面即地基）；③ encoding iframe 面
~330 评估；④ DC-3 全量对账（同 revision 归一、验收线 1,393,955）。

[执行契约](../wptfyi-runtime.md) · [全量预算](../../compat/wptfyi-90-plan.md)

| 里程碑 | 状态 | 缺口和证据 |
|---|---|---|
| DC-1 基线与试跑 | **完成** | [DC-1 全量基线](evidence/dc1-full-baseline-2026-11-10.md)——53,166 案 96.39%，变体/归簇定量，CI 三分片 + filter 多段语义，探针缺陷两形态修复；[通道试跑](evidence/dc1-channel-smoke-2026-11-09.md)；发布快照基线 1,317,955 / 1,398,399 |
| DC-2 按簇收敛 | 进行中 | [encoding 定量](evidence/dc2-encoding-gap-2026-11-10.md) + [encoding worker 收口](evidence/dc2-worker-variants-2026-11-10.md)（`f57d91099`，+11,728 过、组残余 ~1,194）+ [wasm 定量与 worker 收口](evidence/dc2-wasm-gap-2026-11-10.md)（`c181cccab`，+1,129 过、残余 1,674 = 引擎 ~56 + 划界 ~1,369）+ [test262 三簇定性](evidence/dc2-test262-cluster-triage-2026-11-10.md)（eval-code 早期错误 / source-phase / import defer 均 V8 引擎侧，~250 案不回收）+ [export default 具名声明簇修复](evidence/dc2-test262-export-default-fix-2026-11-10.md)（+27 过）+ [expressions 负例桶 import 形态严格化](evidence/dc2-test262-import-shape-fix-2026-11-10.md)（+67 过：63 import-defer + 4 importcall；8 strict callexpression 三引擎一致定性引擎侧）+ [module 早期错误双通道预检＋import.meta 词法重写](evidence/dc2-test262-module-early-error-fix-2026-11-10.md)（`42ab0ee46`，+181 过：module-code +63、import.meta 面 +17、expressions 救回 +101，零翻红；HostCallback Plain/Scoped enum＋scoped 回调窗转借 HandleScope＋has_unpublished_static_import 豁免＋RewriteCtx 状态机镜像）+ [TLA 判定单调不减注入](evidence/dc2-test262-tla-injection-2026-11-10.md)（`59091ef2e`，+97 过零翻红：扫描器命中恒保持＋V8 async/classic 双腿探针仲裁 false 面＋TLA_OVERRIDES 线程表注入 4 判定点；两轮翻红回滚教训在案——export/import 源函数体包裹必拒、`await <模板/正则>` sloppy 语义误合法）+ [TLA 深度门只认大括号](evidence/dc2-test262-tla-injection-2026-11-10.md)（`5b8080ae2`，+30 过零翻红：括号不引入函数作用域，括号内 await 命中——上轮「for-await 19」勘误定性为 export-class extends 实参簇＋调用实参形）+ [iframe realm JS 内建转发面＋三簇定性](evidence/dc2-test262-realm-intrinsics-2026-11-10.md)（`d92788bb1`，+121 过零翻红：scoped eval＋50 内建转发本体＋Proxy 可构造化——-realm 面 33P/189F→132P/90F、staging/sm 面 +22；归簇定性入册：tco 34 引擎侧不可回收（V8 未实现 PTC）/SAB 106 可修（backing store 跨 isolate 管线方案已勘明）/ShadowRealm 61 特性切片；createRealm 281 宽收勘误为 eval/intrinsics 族 ~50 本轮已修）；剩余簇：TLA 残余 15（template-literal 3/语义时序 12）、$262.agent（112 案——SAB 管线 dependent）/gc（15 案）、SAB clone 106、ShadowRealm 61、encoding iframe ~330、escaped-await 探针结构残余 |
| DC-3 全量对账 | 待执行 | 验收线为 1,393,955 项通过，证据待补 |

碰撞记录暂无。遇到共享文件或语料 pin 冲突时，记下对应 goal、范围和可并行工作。
（2026-11-09：`resources/test262/` 为本目标新占用 fetch 面，`wpt-data/third_party/test262/`
vendored 子树为本目标独占，无共享写面冲突。）
