# M2 切片 363：mixed eager/defer 同 URL 记录共享——双向钩子按 module map 单键归一

**日期**: 2026-11-09
**目标对**: s361 账本挂账首项——webview 页面 lane 同 URL 先 eager 后 defer / 先 defer 后 eager 的记录共享语义（HTML spec module map 单键，模块求值恰一次）
**结果**: 双向共享钩子落地；sandbox 428 单测全绿（新 1）+ engine V8 级单测（part98 新文件）；cyclic TLA 死锁案保持 Pass；域 join 1129 键零翻转。

## 勘根定谳

1. **现状**：eager 面（`__zw_load_module`）记账 `__moduleCache[resolved]`（值 = 导出面对象）+ `__zwModuleFetchKeys[fk]`；defer 面（part01.js `__zw_load_module_deferred`）记账 `__zwDeferRecords[resolved]`（三臂对象 {exports, depsPromise, runBody}）+ `__zwDeferFetchKeys[fk]`——两表独立（源内 FIXME 明示「同 URL 混用未建模」）。
2. **背离面**：①eager 先、defer 后 → defer 表无记录 → 宿主重编译 + defer 求值，**体跑两次**（违反求值恰一次）；②defer 先、eager 后 → eager 表无记录 → 同上双求值；且 defer 门面触发的是第二份编译产物的守卫，与首份无关联。
3. **spec**：module map 按 resolved URL 单键——两视图同一记录；eager import 语义 = **全图求值完成**后取导出；import.defer 门面属性访问触发的是导出绑定求值（延迟面）。

## 施工

1. **es_module.rs prelude**（module_dynamic_import_helpers_prelude）：新 `globalThis.__zwEagerFromDefer(drec)` 等待 helper——`Promise.resolve(drec.depsPromise).then(() => { drec.runBody(); return drec.exports; })`。三接入点（同形态统一）：①fk 缓存快速路径 defer 表命中 → helper；②宿主调用后 resolved 层钩子（defer 表命中 → **同步记账** `__moduleCache[resolved] = drec.exports` + fk 表 → return helper）；③resolved 层缓存复取 defer 表命中 → helper。守卫 `globalThis.__zwDeferRecords &&`（契约 eval 上下文无 part01、表缺位落穿零漂移）。
2. **part01.js defer 侧**（`__zw_load_module_deferred` resolved 段）：eager 缓存命中（体已求值）→ 以导出面构造「已求值三臂」`{exports, depsPromise: Promise.resolve(), runBody: no-op}` 入 defer 表 + fk 记账——门面 get trap 触发 runBody（no-op）无害，属性访问直取导出值。
3. **engine 测试**：新 `js_dom_bridge_tests/part98_s363_mixed_eager_defer.rs`（persistent context 三段 execute：shim + prelude + 调用体；eager 先取 → defer 取用 → 断言 exports 引用同源 + 体恰一次 + facade 直取值零再求值）。

## 红态轨迹（三轮收敛）

1. **首版钩子同步 runBody**（域跑抓出 P→F）：cyclic 案 `module A's body should never run expected undefined but got true`——死锁时序破坏：defer 记录 deps 挂起时（A 的 deps IIFE await memoized B 挂死）钩子同步触发 A 体。教训：**eager import 命中 defer 记录必须先等 depsPromise**（spec 全图求值语义），域跑基线 join 是钩子类变更的必要验证（单测抓不到死锁时序交互）。
2. **Promise 版钩子后测试 a 链断**（settled=None 三段定位）：`kind:object:then:function:runs:0` 定谳钩子命中（typeof Promise = 'object'）；`entry-runs:1` 定谳 F1 已被首段 checkpoint 排空；`f2-entry` 定谳 F2 跑、**内层二次 import `.then` 炸**——复 import 走 fk 缓存快速路径同步返回对象，与首次钩子 Promise 形态不一致，裸 `.then` 挂非 thenable 抛 TypeError 静默 reject。诊断探针（probe_checkpoint_ticks + 分段 report）跑后已删。
3. **定稿**：同步记账（exports 引用入缓存，getter 已首装）+ 三接入点统一 helper（复 import 与首次同形态 Promise，等图完成语义在复取路径保持）；测试记录的 runBody 补产物内建恰一次守卫（真实 deferred 编译产物带 `__zwBodyRan` 守卫，测试造裸函数致二次触发 runs=2 红态，非实现 bug）。

## Blast radius / 回归定谳

- **无 defer 记录面零位移**：守卫落穿，域 join 1129 键（A=s361-b vs B=s363-b 终版）A-only/B-only 双零、STATE-DIFF=0（cyclic P 保持）。
- **消费面形态兼容**：生产 `__zw_dynamic_import` 发射面（es_module 625）以 `Promise.resolve(...)` 包裹——同步对象与 Promise 两形态通吃；静态依赖 memoized 面不走 `__zw_load_module`（编译期 inline 展开）不受扰；既有同步直调单测（t8j 缓存键 / s87 四态）无 defer 表记录落穿。
- **微任务时序**：钩子路径较原同步返回多一层 then tick——promise 链幂等 tick 数，域 join 零翻转实证。
- sandbox 428 全绿（含新 test_mixed_eager_hits_defer_record_s363 两段 persistent 式）；clippy zero-script-sandbox + zero-engine --all-targets 净；cargo fmt 净。
- make test：见下「日志」。

## 验证

- **solo**：cyclic-tla-defer-eval 恢复 Pass；engine mixed_defer_hits_eager_record_s363 Pass；sandbox 新测 Pass。
- **域 join**（终版）：1129 键双零零翻转（基线 s361-b）。
- **make test**：9326P/7F（较 s362 9325 +1 = part98 新测进 zero-engine 大 target 3827→3828，归因自洽），失败 7 名与 s356-s362 基线逐名同构，零新增红。
- **诊断物清理**：probe_checkpoint 临时测试已删（源内零残留）；wpt-data 零改动。

## 挂账 / 下一步

- TLA 入口体延迟属性访问挂起语义（钩子 resolve 的 exports 在 TLA 体完成前 getter 值未定——与既有 async 入口面 FIXME(R5182) 同族）。
- SW module 静态 defer 求值语义；143/052 SVG 族运行时面+流式物化长挂；142 xlink:href write 注入面；probe12/13 自建单列；T=5 长尾。

## 日志

- evidence/logs/s363-b-script-element.log（域跑 B 面，join 基线 s361-b-script-element.log；首版钩子 P→F 的中间态日志被终版 tee 覆盖，红态现场见「红态轨迹」引文）
- evidence/logs/s363-make-test.log（全量回归）
