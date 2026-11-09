# M2-S336：data: URL 报告位置保真 + 微任务检查点序（承 s335）+ 内联模块解析基恢复

日期：2026-11-07。控制文档：`docs/goal/html-semantics-compat.md`（L5 链 s336 段）。
本切片在 s335（微任务检查点语料缺口 + strict classic 抛错报告面）挂账的两条线上开工：
**data: URL strict 抛错三根因** 与 **message 面检查点序（R3）**，期间勘根出第三个面
（**内联投递模块解析基**）一并收口。

## 定谳

### F1：data: URL strict 抛错报告面（三根因）

WPT `the-script-element/data-url.html` 判据：`<script src="data:,…">` 内抛错时
`window.onerror` 五参 `(message, source, lineno, colno, error)` 的
`source` 须为 **data: URL 原文**（spec：脚本 URL 即请求 URL）。三处缺陷串联：

- **R1a-后继**（script_gen.rs strict tail）：`reportError(__zw_e)` 单参调用，
  无 filename/lineno/colno 穿入——报告位置丢失。
- **R2932 面**（shim part06 `_defineWinOnHandler`）：`window.onerror` 以
  EventHandler 单参注册——缺 OnErrorEventHandler legacy 五参转换面
 （spec：window.onerror 是唯一五参 handler，其余 on* 均单参）。
- **R2932 报告臂**（shim part01 `reportError` shim）：单参硬编码
  `filename:''/lineno:0/colno:0`。

### F2：message 面检查点序（R3 定谳）

`microtasks/checkpoint-after-window-onerror{,-module}`：postMessage 监听器在
宿主 task（JS 栈空）派发——spec `clean up after running scripts` 检查点逐监听器
执行。shim `_dispatchToListeners` 纯 JS 同步 fire 循环无检查点 → 序错
`[h1,h2,h1p,h2p]`（期望 `[h1,h1p,h2,h2p]`）。

### F3：内联投递模块解析基（新勘根）

`microtasks/evaluation-order-3` Fail 勘根三级递进（探针实证）：

1. **表症**：`import("./evaluation-order-3.2.mjs")` 报
   `Failed to fetch module`（①态空契约文案）。
2. **第一层**：sandbox 层探针（stub `__zw_compile_module` 捕获三参）——
   helper/重写/parent 穿入全数干净，parent = `_importMeta.url` 正确。病灶不在
   sandbox，在宿主臂。
3. **第二层**：运行时探针（`__zw_compile_module` 回调 warn 三参）——parent =
   **页面 URL**（`…/microtasks/`）而非模块 URL（`…/resources/…`）。runner
   内联投递把外链模块体转 InlineModule，engine 内联模块默认基 = 页面 URL；
   s101 静态面靠 runner spec 重写（root-absolute `/…`，基无关）恢复，**动态
   import() 不在重写面**（runner s101 注释自认挂账）——兄弟 spec 对页目录解析
   → 404 → TopFetch。
4. **第三层**：`data-inline` 存**语料根相对裸路径**（`html/…`，无 scheme 无
   前导斜杠）——第一版修复直接 `resolve_document_url(doc, src)` 产生页目录双拼
   （探针实证 parent=`…/microtasks/html/semantics/…`）；裸路径须先归一
   root-absolute。

### F4：动态 import 同步求值序（order-3 残差定谳）

F3 修复后 order-3 残差为**纯序偏差**：期望
`[3.1-1,3.1-2,3.1-3,microtask-3.1,3.2-1,3.2-2,microtask-3.2,import-catch,error]`，
实得 3.2 体插进调用点同步块（`[3.1-1,3.1-2,3.2-1,3.2-2,3.1-3,…]`）。spec
fetch-a-single-module-script 的求值属**后续 job**——`__zw_load_module` 同步
取回+求值把子模块体塞进 `import()` 调用点，检查点序全错。

## 施工

| 面 | 位置 | 内容 |
|---|---|---|
| R1a-后继 | `script_gen.rs` `STRICT_UNCAUGHT_REPORT_TAIL` | `reportError(__zw_e, <src_expr>, 1, 1)` 四参——src 由 webview 穿入（`script_run_classic_page_strict` 新第三参），IIFE 内 `currentScript.src`→`location.href` 回落 |
| 传递 | `webview.rs` strict 臂 | `strict_report_src` = 外链/内联原始 URL 解析（InlineModule 走 `script_csp_info[i].inlined_src`，绝对 data: URL 原样） |
| R2932 | `shim/part06.js` `_defineWinOnHandler` | onerror 特例：适配器五参展开 `(ev.message, ev.filename, ev.lineno, ev.colno, ev.error)` 注册 addEventListener；原 fn 存 `_winOnHandlers` 保 getter 身份；setter remove 用适配器 |
| R2932 报告 | `shim/part01.js` `reportError` | 四参 `(reason, filename, lineno, colno)`——`filename==null→''`，`lineno/colno==null→0`（单参调用面零漂移） |
| R3 | `shim/part03.js` `_dispatchToListeners/_dispatchWithBubble` | 新第六参 `asyncDrain`——排空臂改 `fire(entry)` + `Promise.resolve().then(drainNext)` 链（V8 原生微任务 FIFO ≡ spec 检查点）；7 调用点穿参；**仅 postMessage 臂启用**（`dispatchEvent(ev, {zwAsyncDrain:true})`），同步 window.event 面不动 |
| F3 | `webview.rs` `module_dep_base` | `InlineModule` + `script_csp_info[i].inlined_src` 臂：裸路径归一 `/{src}` 后解析（绝对 URL 原样）；独立 `external_module_error_face` bool 保 R2944 错误派发判定不被基语义污染 |
| F4 | `es_module.rs` `__zw_dynamic_import` | 尾改 `Promise.resolve().then(function(){ return Promise.resolve(__zw_load_module(...)); })`——求值跳入微任务；取回/缓存/失败记账/错误对象身份面零漂移（`__zwModuleFetchKeys` 时序面：同轮并行 import 串行化仍单次取回） |

## 验证

- **契约**：engine part09 `test_window_onerror_legacy_five_arg_adapter_and_report_position_s336`（五参序 + ev.filename 断言）；script-sandbox `test_dynamic_import_evaluation_deferred_microtask_s336`（order-3 全序列钉 + 宿主取回恰一次）；webview `inlined_module_dep_base_restored_from_data_inline_s336`（基恢复 + 兑现断言）。sandbox 401P/0F、webview 844P/1F（红 = `proxy_mode_decodes_png_correctly` image-decoder 进程环境红，s168c/s256 两账在案 HEAD 同挂）、runner bins 290P/0F。
- **microtasks 域**（域终态 log `evidence/logs/s336-script-element-after.log`）：11 face = 10 Pass + 1 Timeout——checkpoint-after-window-onerror ×2、checkpoint-after-window-onerror-module ×2、evaluation-order-1 ×4、order-2、order-3 全 Pass；order-4 Timeout 挂账。s335 基线（`s335-microtasks-after.log`）8 红面（checkpoint ×4 Fail/Timeout + order-2/3 红 + order-4）。
- **the-script-element 域 A/B**：`s336-script-element-after.log` **1057P/54F/12T** vs `s335-script-element-after.log` 1052P/57F/14T——**+5P/-3F/-2T，P→F 零**。翻绿五文件：`data-url.html`（F1）、`checkpoint-after-window-onerror{,-module}.html`（F2）、`evaluation-order-2.html`（s335 语料 + F2）、`evaluation-order-3.html`（F3+F4）。
- **engine lib**：`s336-engine-lib.log` 3805P/6F——红集 = 已知六红精确（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / websocket_error_nonterminal / websocket_event_flow_open_message_close），零新增。
- **clippy/fmt**：`-D warnings` 三 crate 全绿；rustfmt 全绿。

## 挂账 s337

`evaluation-order-4`（TLA `await import()`）Timeout 三子面（勘根已毕）：

1. **load 投递尾时序**：TLA 入口模块的 `script-load` 期望落在
   `microtask-4.1` 之后、`step-4.2-1` 之前——现行投递尾在体尾同步 fire（TLA
   async IIFE 早于微任务排空）；需 TLA 面 load 延迟（取回完成语义）。
2. **TLA rejection → window 上报**：`await import()` 拒绝（4.2 体 throw）→
   async IIFE Promise rejected 无人接——`testDone`（window.onerror）永不触达
   → Timeout。需 TLA 面错误上报桥（async_body 包装拒臂 reportError）。
3. 依赖 F4 已修面（本切片收口）。

另挂账：`window.onerror = fn` 直接赋值后 spec 要求的 per-fire
`window.event` 语义偏差（F2 施工时 event-global 面维持既有语义，未扩大）。

## 证据索引

- `logs/s336-microtasks-after-fix-mid.log`——R3 drain 链落点（mt4：checkpoint ×4 全 Pass 中间态）
- `logs/s336-microtasks-after-base.log`——F3 基修复后（mt5：order-3 序残差现形）
- `logs/s336-script-element-pre-hop.log`——F4 前全域基线（se2：1056P/55F/12T）
- `logs/s336-script-element-after.log`——F4 后全域终态（se3：1057P/54F/12T）
- `logs/s336-engine-lib.log` / `logs/s336-webview-lib.log`——crate 全量
