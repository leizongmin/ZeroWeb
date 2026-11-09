# M2 切片 362：SW 宿主 import.defer 拒型面——prelude stub 归正

**日期**: 2026-11-09
**目标对**: 非 webview 宿主 import.defer 面（s361 账本挂账）——ServiceWorker classic/module 两路
**结果**: `import.defer(` 动态形式在 SW 宿主从 ReferenceError 破损面归正为 TypeError 拒绝（与 dynamic import 同款 fail-closed）；sandbox 427 单测全绿（含新 2 件红态回归守卫）。

## 勘根定谳

1. **宿主面盘点**（s360/s361 施工后）：import.defer 重写发射面有两档——webview 页面 lane（Module 档 → `__zw_import_defer.call`，part01.js 提供运行时）+ classic lane（ClassicDeferOnly 档 → `__zwClassicImportDefer`）；SW 宿主 5715 classic 路径与 es_module build_module_script 语句路径（module SW 经由）共用 `rewrite_dynamic_imports`（Module 档）→ `import.defer(` 也被重写为 `__zw_import_defer`。
2. **真根因**：SW prelude（SERVICE_WORKER_DYNAMIC_IMPORT_PRELUDE）只提供 `__zw_dynamic_import` 拒型 stub（spec fail-closed：classic SW 禁一切动态模块取回，w3c ServiceWorker #service-worker-script-request），无 `__zw_import_defer` → SW 源含 `import.defer(...)` 时调用点 `ReferenceError: __zw_import_defer is not defined` 抢在 TypeError 拒绝面前——拒型面破损，错误类背离宿主契约。
3. **顺带核对（零改动项）**：SW module 静态 `import defer *` 声明面已被 s361 顺带传导——SW `compile_module_script` 委托 `compile_module_script_with_credentials`（s361 async 传染点）、图收集 `extract_static_module_import_specifiers`（s361 归一点）均 lib 共享；SW 主模块 async 判定由此归正。动态 defer 的求值语义（真加载 vs 拒）在 SW 宿主无 spec 钉面（w3c SW 禁动态模块取回的 fail-closed 现状沿用），挂账不触。

## 施工

1. **SERVICE_WORKER_DYNAMIC_IMPORT_PRELUDE 追加 stub**（service_worker.rs 29 行起）：`globalThis.__zw_import_defer = function() { return Promise.reject(new TypeError('dynamic import is unavailable in Service Worker scripts')); };`——与 `__zw_dynamic_import` 同文案同拒型，classic/module 两路消费同一 prelude。
2. **单测两件**（mod tests，红态回归守卫）：`import_defer_rejected_with_typeerror_classic_s362` / `..._module_s362`——两段式（源内 `.then(fulfilled, rejected)` 记账 → 第二段 evaluate 同步断言 `instanceof TypeError`），依托 v8 sandbox `execute` 末尾 `perform_microtask_checkpoint` 逐跳排空（v8_runtime.rs 1009 先例）。

## 红态轨迹

1. 无 stub（施工前现状）：双测 FAILED——`other:ReferenceError: __zw_import_defer is not defined` ≠ `TypeError`（拒绝回调捕获 ReferenceError，账面值 `other:` 前缀）——红态证据复现（临时摘 stub 复跑，双测红；回补 stub 后双绿）。
2. 加 stub 后：双测 Pass（classic/module 两路均 TypeError 拒绝）。

## Blast radius / 回归定谳

- **零 import.defer 的 SW 源零变化**：stub 仅新增一个全局函数属性，无既有行为消费点；SW 全局形状断言测试（scope brand / fetch prototype 位）不查全量 keys，不受影响。
- **拒型语义与宿主契约同构**：文案与 `__zw_dynamic_import` 一致（同一 fail-closed 理由），错误类 TypeError 与 dynamic import 拒型逐字同构。
- sandbox 427 单测全绿（425 + 新 2）；clippy --all-targets 净；cargo fmt 净。
- make test 全量：见下「日志」（基线 9324P/7F 逐名同构核对）。

## 验证

- **单测红态对**：摘 stub 双红（ReferenceError 捕记账）→ 回补双绿，bug 回归证据完整（旧版本因目标行为失败、修复后通过）。
- **make test**：9325P/7F，失败 7 名与 s356-s361 基线逐名同构，零新增红。P 较 s361（9324）+1 属并行流组合态计数漂移（js_dom_bridge 3819→3820，ok 名单 sort -u 同构，本刀仅 script-sandbox 面不在该 target 覆盖集）。
- 域跑不适用：corpus 无 SW defer 案（非 webview 宿主面，验收面为等价本地测试——SW runtime 单测，符合挂账纪律）。

## 挂账 / 下一步

- SW module 静态 `import defer *` 求值语义（真加载 vs fail-closed 拒）——w3c SW spec 无钉面，上游实现差异大，维持现状挂账。
- mixed eager/defer 同 URL 记录共享语义（webview 面，spec 语义决策先行）；TLA 入口体延迟属性访问挂起语义。
- 存量在册：143/052 SVG 族运行时面+流式物化长挂；142 xlink:href write 注入面；probe12/13 自建单列；T=5 长尾。

## 日志

- evidence/logs/s362-make-test.log（全量回归）
