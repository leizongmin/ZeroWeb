# ES module、import map 与动态脚本

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查裸说明符、动态 import 失败、Vite 能力探测与脚本预取不收口。转换核心在 `es_module.rs` / `import_map.rs`，消费者是 renderer、tab worker 与 WebView；网络策略见 [资源取回](../web-data/network/fetch-and-navigation.md)。

### User Task Navigation

`es_module.rs` / `import_map.rs` 的内嵌测试覆盖转换及解析，`script_prefetch.rs` / `tick_per_task_tests.rs` 钉宿主动态脚本成功、失败和通道关闭。未实跑，转换测试不能证明原生模块规范全面实现。

## Gotchas
- 现行 module 是源码转换为 IIFE/exports 的实现，不是直接使用引擎原生 module loader；`source.contains("await")` 等启发式分支不能当完整语法分析或完整 live-binding 语义。 (`crates/script-sandbox/src/es_module.rs`)
- runtime prelude 要先定义 `__zw_load_module` / `__zw_dynamic_import` 再求值 registry IIFE；模块体顶层 import 探测会在注册表达式中立即执行，反序会先抛 helper 未定义。 (`crates/script-sandbox/src/es_module.rs`)
- dynamic import helper 捕获同步 fetch/compile 错误并返回 rejected Promise，否则 `import(x).catch(...)` 在挂 catch 前同步抛，站点能力探测被提前中断。 (`crates/script-sandbox/src/es_module.rs`)
- 动态模块缓存键采用 parent×specifier 解析后的 URL，宿主回传 `resolved\\u001fcode`；按原始 `./config.js` 缓存会让不同目录导入错误地共享副作用与 namespace。 (`crates/script-sandbox/src/es_module.rs`, `apps/renderer/src/js_worker.rs`, `apps/browser/src/tab_js_worker.rs`)
- import map 必须装到静态依赖预取、主 registry 和动态 compile 新建 registry；只修页面入口 registry，动态 import 仍会把裸说明符错误解析成相对 URL。 (`apps/renderer/src/script_prefetch.rs`, `apps/renderer/src/js_worker.rs`, `apps/browser/src/tab_js_worker.rs`, `git:9786defd3ac7087154bffea875c5c58466d1290d`)
- `resolve_mapped` 把 Blocked 降成 None，部分消费者随后回退普通 URL 解析；当前 scopes/prefix 的遍历也不能保证规范最长前缀，存在 import map 实现边界，不能按解析器结果枚举推出端到端合规。 (`crates/script-sandbox/src/import_map.rs`, `crates/script-sandbox/src/es_module.rs`, `apps/renderer/src/script_prefetch.rs`)
- 预取仅抽静态 import，动态 `import("_")` 探测留运行时；全量抽取会把探测当网络依赖，产生 404 与 cache 污染。 (`apps/renderer/src/script_prefetch.rs`, `crates/script-sandbox/src/es_module.rs`)
- 预取 Receiver::Disconnected 是完成失败而不是继续等待；导航/StopLoading 丢 Sender 后不移除 inflight，会让 `is_active()` 永真、脚本阶段不收口。 (`apps/renderer/src/script_prefetch.rs`)

## Architecture
- 动态经典 `<script src>` 经宿主 ResourceFetchMeta::SCRIPT 取回，再执行并派元素 load/error；不能用页面 fetch() 代替资源脚本取回，因为无 ACAO 的脚本源可能在页面 CORS fetch 中被拒。 (`apps/renderer/src/script_prefetch.rs`, `apps/renderer/src/tick_per_task_tests.rs`)
- JavaScript MIME helper 接收的是 MIME essence，大小写不敏感并保留 legacy 类型；它不剥离参数，调用方须先解析 Content-Type，不能把完整带 charset 字符串直接当 essence。 (`crates/net/src/script_mime.rs`)
- v8+quickjs feature 并集时 EsModuleSandbox 优先 V8，QuickJS 构造分支带 not(v8)；两个引擎分支不能无互斥同时声明同一个 sandbox。 (`crates/script-sandbox/src/es_module.rs`, `crates/script-sandbox/src/lib.rs`)
