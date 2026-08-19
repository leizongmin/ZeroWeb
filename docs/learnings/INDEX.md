# Learnings 索引

> 本文件由 `scripts/gen-learnings-index.py` 生成，勿手改；
> 新增 learning 后运行 `make learnings-index` 重建。
> 布局契约：`<分类>/<YYYY-MM>/<YYYY-MM-DD>-<topic>.md`，日期以 frontmatter 为准。
> 方法论蒸馏层见 `.agents/skills/zeroweb-guidelines/SKILL.md`。

## Bugs — 踩坑记录（根因 + 修复 + 如何避免）（81）

- 2026-08-19 [IndexedDB terminal state wait](bugs/2026-08/2026-08-19-indexeddb-terminal-state-wait.md) — apps/browser/src/process_backend/indexed_db_owner_tests.rs
- 2026-08-19 [IndexedDB listener 异常必须结合 transaction 状态处理](bugs/2026-08/2026-08-19-indexeddb-listener-exception-transaction-state.md) — engine, wpt-runner
- 2026-08-19 [IndexedDB key range canonical conversion](bugs/2026-08/2026-08-19-indexeddb-keyrange-canonical-conversion.md) — crates/engine/src/js_dom_shim/part02.js
- 2026-08-19 [IndexedDB get-all options overload 不能复用旧 query/count 路径](bugs/2026-08/2026-08-19-indexeddb-getall-options-overload.md) — engine, wpt-runner
- 2026-08-18 [Shorthand Component Classification](bugs/2026-08/2026-08-18-shorthand-component-classification.md)
- 2026-08-18 [Process backend drop can race multiprocess compositor tests](bugs/2026-08/2026-08-18-process-backend-drop-compositor-test-race.md) — apps/browser/src/process_backend.rs, apps/browser/src/tests.rs
- 2026-08-18 [Media Query Trailing Token Rejection](bugs/2026-08/2026-08-18-media-query-trailing-token-rejection.md) — crates/css-parser/src/media_query.rs
- 2026-08-18 [Media Query Keyword Token Boundary](bugs/2026-08/2026-08-18-media-query-keyword-token-boundary.md) — crates/css-parser/src/media_query.rs
- 2026-08-18 [IndexedDB schema rename wrapper rollback](bugs/2026-08/2026-08-18-indexeddb-schema-rename-wrapper-rollback.md) — crates/engine/src/js_dom_shim/part02.js
- 2026-08-18 [IndexedDB native detached binary key](bugs/2026-08/2026-08-18-indexeddb-native-detached-binary-key.md) — crates/engine/src/js_dom_shim/part02.js
- 2026-08-18 [IndexedDB metadata tasks and UTF-16 wire names](bugs/2026-08/2026-08-18-indexeddb-metadata-task-and-utf16-wire.md) — engine, page-runtime, storage
- 2026-08-18 [IndexedDB keyPath own-property and sparse-array wire](bugs/2026-08/2026-08-18-indexeddb-keypath-own-property-wire.md) — engine, page-runtime, storage
- 2026-08-18 [IndexedDB deferred operation active check](bugs/2026-08/2026-08-18-indexeddb-deferred-operation-active-check.md) — crates/engine/src/js_dom_shim/part02.js
- 2026-08-18 [IndexedDB cursor transaction view](bugs/2026-08/2026-08-18-indexeddb-cursor-transaction-view.md) — crates/page-runtime/src/indexed_db_host/cursor.rs
- 2026-08-18 [IndexedDB cross-renderer stale schema after upgrade](bugs/2026-08/2026-08-18-indexeddb-cross-renderer-stale-schema.md) — engine/js_dom_shim, browser/process_backend
- 2026-08-18 [IndexedDB compound object-store key path schema drift](bugs/2026-08/2026-08-18-indexeddb-compound-object-store-key-path.md) — engine, page-runtime, storage
- 2026-08-18 [Font Style Oblique Token Boundary](bugs/2026-08/2026-08-18-font-style-oblique-token-boundary.md) — crates/css-parser/src/values/parse_misc.rs, crates/css-parser/src/parser/at_rules.rs
- 2026-08-18 [Exact Font Family Matching For Ahem](bugs/2026-08/2026-08-18-exact-font-family-matching-for-ahem.md) — crates/layout-engine/src/table_types.rs
- 2026-08-17 [QuickJS 专属测试必须排除 V8 feature union](bugs/2026-08/2026-08-17-quickjs-tests-must-exclude-v8-feature-union.md) — zero-webview, workspace 测试矩阵
- 2026-08-17 [JS worker 空闲计数应使用 steady-state 基线](bugs/2026-08/2026-08-17-js-worker-idle-count-needs-steady-state-baseline.md) — apps/renderer
- 2026-08-17 [Image Stream File Descriptor Exhaustion](bugs/2026-08/2026-08-17-image-stream-fd-exhaustion.md) — apps/browser/src/fetch_proxy.rs, crates/net/src/client.rs
- 2026-08-17 [DMA-BUF 测试必须关闭 compositor scroll transform](bugs/2026-08/2026-08-17-dmabuf-test-scroll-transform-precondition.md) — apps/compositor, zero-protocol
- 2026-08-17 [Active Tab IPC Poll Starvation](bugs/2026-08/2026-08-17-active-tab-ipc-poll-starvation.md) — apps/browser/src/process_backend.rs, apps/browser/src/compositor_client.rs
- 2026-08-16 [RenderPrimitives 路径图元顶点格式契约：段序列 vs 点序列](bugs/2026-08/2026-08-16-render-primitives-path-vertex-format.md) — zero-canvas, zero-render-foundation（primitive/gpu mesh）
- 2026-08-16 [GPU clip「擦白」语义 vs canvas clip()「持续裁剪」语义差异](bugs/2026-08/2026-08-16-gpu-clip-erase-vs-canvas-clip-state.md) — zero-canvas（gpu_path 测试）, zero-render-foundation（gpu/renderer）
- 2026-08-16 [dma-buf 导入偏移硬编码 (0,0) 导致页面覆盖浏览器 chrome](bugs/2026-08/2026-08-16-dmabuf-import-dst-offset-covers-chrome.md) — apps/browser（app_platform.rs / process_backend.rs / tab_snapshot.rs）, crates/render-foundation（GPU import blit）
- 2026-08-15 [184bbffc6 默认翻转 Linux 侧验证：四个修复与三个已知缺口（2026-08-15）](bugs/2026-08/2026-08-15-seccomp-default-on-blocks-dmabuf-fd-socket.md) — apps/compositor/src/seccomp_linux.rs, crates/render-foundation/src/gpu/renderer/mod.rs, apps/browser/src/app_platform.rs, crates/webview/src/image_decoder.rs
- 2026-08-15 [FreeType measure_advance hinting 取整导致英文文本字距与水平位置错乱](bugs/2026-08/2026-08-15-freetype-measure-hinting-advance.md) — crates/render-foundation/src/font/loader/freetype_raster.rs, loader.rs, crates/engine/src/paint/painter/text/
- 2026-08-14 [Variable font shaping must share axes with rasterization](bugs/2026-08/2026-08-14-variable-font-shaping-needs-raster-axis.md) — render-foundation/font, layout-engine, engine/paint, WPT runner
- 2026-08-14 [并行测试中的空闲端口 TOCTOU](bugs/2026-08/2026-08-14-test-server-free-port-toctou.md) — zero-webdriver
- 2026-08-14 [Synthetic small-caps 需要在断行前展开](bugs/2026-08/2026-08-14-synthetic-small-caps-needs-pre-line-break-expansion.md) — zero-layout-engine::inline, zero-engine::paint
- 2026-08-14 [Rust 并行测试中的进程级 FD 计数误报](bugs/2026-08/2026-08-14-rust-test-process-global-fd-count.md) — zero-protocol
- 2026-08-14 [Multiprocess tests require fresh peer binaries](bugs/2026-08/2026-08-14-multiprocess-tests-require-fresh-peer-binaries.md) — protocol, browser, renderer, compositor, test tooling
- 2026-08-14 [Italic/oblique matching 依赖 inline face ownership](bugs/2026-08/2026-08-14-italic-oblique-matching-needs-inline-face-ownership.md) — zero-render-foundation::font, zero-engine::paint
- 2026-08-14 [GPU HiDPI 二次缩放与 compositor 启动缺失](bugs/2026-08/2026-08-14-gpu-hidpi-double-scaling-and-compositor-launch.md) — apps/browser, scripts/browser*.ps1
- 2026-08-14 [Fallback face 必须独立解析 feature descriptor](bugs/2026-08/2026-08-14-fallback-face-feature-descriptor-ownership.md) — zero-render-foundation::font
- 2026-08-14 [Compositor owned-present 导致 GPU 启动白窗](bugs/2026-08/2026-08-14-compositor-owned-present-gpu-white-window.md) — apps/browser, apps/compositor
- 2026-08-14 [Compositor IPC 合法帧超过通用管道上限](bugs/2026-08/2026-08-14-compositor-ipc-frame-size-limit.md) — crates/protocol, apps/browser, apps/compositor
- 2026-08-14 [Compositor 空闲后首帧被误判超时](bugs/2026-08/2026-08-14-compositor-idle-first-frame-false-timeout.md)
- 2026-08-14 [CJK fallback exposes paint IFC contract gaps](bugs/2026-08/2026-08-14-cjk-fallback-exposes-paint-ifc-contract-gaps.md)
- 2026-08-14 [blocking reqwest 不可在 Tokio worker 内析构](bugs/2026-08/2026-08-14-blocking-reqwest-drop-inside-tokio.md) — zero-net 的 HttpClient, 资源调度器 async 迁移
- 2026-08-14 [Author font layout and paint must share ordered face advances](bugs/2026-08/2026-08-14-author-font-layout-paint-advance-ownership.md) — layout-engine/inline, engine/paint, engine/pipeline
- 2026-08-13 [WPT Webfont Resource False Green](bugs/2026-08/2026-08-13-wpt-webfont-resource-false-green.md) — WPT reftest 资产, 字体加载, Chromium Oracle
- 2026-08-13 [WPT testharness 需要显式 timer owner](bugs/2026-08/2026-08-13-wpt-testharness-timer-owner.md) — tests/wpt-runner, zero-webview DOM shim
- 2026-08-13 [wgpu 30 Window Surface Requires a Display Handle](bugs/2026-08/2026-08-13-wgpu30-window-surface-display-handle.md) — render-foundation, browser
- 2026-08-13 [Webfont indexed glyph 必须按 face 边界校验](bugs/2026-08/2026-08-13-webfont-indexed-glyph-bounds.md) — zero-render-foundation, zero-wpt-runner
- 2026-08-13 [Table Text Metadata And UA Defaults](bugs/2026-08/2026-08-13-table-text-metadata-ua-defaults.md) — style-system UA declarations, layout final IFC, table paint
- 2026-08-13 [Shaping cache key 必须包含 face 元数据](bugs/2026-08/2026-08-13-shaping-cache-font-face-metadata.md) — zero-render-foundation, zero-wpt-runner
- 2026-08-13 [Root font-relative units need metric identity](bugs/2026-08/2026-08-13-root-font-relative-units.md) — css-parser, style-system
- 2026-08-13 [Oracle Webfont Ready Race](bugs/2026-08/2026-08-13-oracle-webfont-ready-race.md) — WPT Chromium oracle capture, CSS Font Loading
- 2026-08-13 [Nonspacing mark 不应贡献独立 layout advance](bugs/2026-08/2026-08-13-nonspacing-mark-layout-advance.md) — zero-layout-engine, zero-wpt-runner
- 2026-08-13 [native-dom 路径多 WebView 同线程顺序创建触发 disposed-Isolate panic（R3332 实测定位）](bugs/2026-08/2026-08-13-native-dom-multi-webview-isolate-leak.md) — crates/engine/src/dom_bindings/gc.rs（线程局部 DOM-source / element_template 缓存）, crates/webview/src/webview.rs（install_native_dom_bindings）
- 2026-08-13 [lh/rlh require a used line-height provider](bugs/2026-08/2026-08-13-lh-rlh-needs-used-line-height-provider.md) — style-system, layout-engine
- 2026-08-13 [Host 默认动作与 Microtask 顺序](bugs/2026-08/2026-08-13-host-default-action-microtask-order.md) — apps/renderer, crates/engine, crates/script-sandbox
- 2026-08-13 [Placeholder Caret 与 Legacy 合成测试](bugs/2026-08/2026-08-13-form-placeholder-caret-and-legacy-compositor-test.md) — zero-engine paint, zero-browser 多进程合成测试
- 2026-08-13 [font-language-override needs Chromium-aligned glyph positioning](bugs/2026-08/2026-08-13-font-language-override-needs-chromium-glyph-positioning.md) — style-system, render-foundation, engine
- 2026-08-13 [Font Face Size Adjust Advance](bugs/2026-08/2026-08-13-font-face-size-adjust-advance.md) — css-parser, render-foundation/font, layout advance, engine paint
- 2026-08-13 [First Available Font Metrics](bugs/2026-08/2026-08-13-first-available-font-metrics.md) — css-parser, style-system, render-foundation, webview
- 2026-08-13 [Dynamic WPT fixtures need a fresh Oracle before import](bugs/2026-08/2026-08-13-dynamic-wpt-fixtures-need-fresh-oracle-before-import.md) — tests/wpt-runner, engine
- 2026-08-13 [CJK per-character breaks must not change advances globally](bugs/2026-08/2026-08-13-cjk-contiguous-advance-wall.md) — layout-engine, engine
- 2026-08-13 [canvas shadowBlur 极大值致 region padding i32 溢出 panic](bugs/2026-08/2026-08-13-canvas-shadow-blur-pad-i32-overflow.md) — crates/canvas/src/context/raster.rs（shadow_blur_geom）+ crates/canvas/src/context/context_impl.rs（draw_shadow_rect / draw_shadow_path）
- 2026-08-13 [canvas ImageData 尺寸计算的 u32 溢出回绕](bugs/2026-08/2026-08-13-canvas-image-data-u32-size-overflow.md) — crates/canvas/src/context/context_impl.rs（CanvasContext::new / get_image_data / create_image_data）
- 2026-08-13 [Author font fallback 不应共用 generic 性能门](bugs/2026-08/2026-08-13-author-font-fallback-must-not-share-generic-perf-gate.md) — engine::paint::text_shaping, render-foundation::font
- 2026-08-12 [Multi-process GUI tests must rebuild the renderer child](bugs/2026-08/2026-08-12-multiprocess-gui-tests-stale-renderer.md) — apps/browser, apps/renderer, GUI integration tests
- 2026-08-12 [headless JS-DOM 行为差异 backlog（R3323 WPT 行为锁实测定位）](bugs/2026-08/2026-08-12-headless-js-dom-divergence-backlog.md) — crates/engine/src/js_dom_shim/（B-gen shim）, crates/engine/src/js_dom_bridge/callbacks.rs（host 回调）
- 2026-08-12 [headless handle-only 元素 childNodes 读取限制](bugs/2026-08/2026-08-12-headless-handle-childnodes-limit.md) — crates/engine/src/js_dom_shim/part05.js（_childNodeList）, part04.js（childNodes get-trap）, part02.js（__zw_child_nodes host 回调）
- 2026-08-12 [表单 live value 被全量重载清空](bugs/2026-08/2026-08-12-form-live-value-lost-by-full-reload.md) — zero-engine, zero-webview, zero-renderer
- 2026-08-12 [@font-face unicode-range 被忽略](bugs/2026-08/2026-08-12-font-face-unicode-range-ignored.md) — css-parser, render-foundation/font, engine, webview, browser, renderer, wpt-runner
- 2026-08-12 [CPU/GPU 双链路分叉：测试全绿 ≠ 用户所见正确（排查基线）](bugs/2026-08/2026-08-12-cpu-gpu-path-divergence.md) — crates/render-foundation/src/gpu/renderer/mod.rs, src/cpu/*, tests/wpt-runner/src/reftest.rs, apps/browser/src/app_platform.rs, apps/compositor/src/*
- 2026-08-11 [Shaped advance must preserve the paint base](bugs/2026-08/2026-08-11-shaped-advance-must-preserve-paint-base.md) — zero-engine, zero-layout-engine, zero-render-foundation
- 2026-08-11 [表单控件事件链不能依赖渲染缓存](bugs/2026-08/2026-08-11-form-controls-must-not-depend-on-glyph-cache.md) — host-runtime, apps/browser, apps/renderer, crates/engine
- 2026-08-11 [BrowserShell 持久化设置测试的 TOCTOU 竞争](bugs/2026-08/2026-08-11-browser-shell-persisted-settings-test-toctou.md) — crates/browser-shell
- 2026-08-11 [BiDi range 顺序不能替代 shaping direction](bugs/2026-08/2026-08-11-bidi-range-order-is-not-shaping-direction.md) — layout-engine/inline, engine/paint
- 2026-08-09 [User-Agent 导致服务端返回降级页面](bugs/2026-08/2026-08-09-user-agent-server-content-negotiation.md) — zero-net, zero-engine, 浏览器导航
- 2026-08-09 [JS-in-Rust-string 测试断言：`{...}` 大括号与 `//` 行注释陷阱](bugs/2026-08/2026-08-09-js-in-rust-string-assert-braces-and-comments.md) — crates/engine/src/js_dom_bridge_tests/part*.rs（js_dom_bridge 测试, 含大量内联 JS）
- 2026-08-09 [Canvas background and false horizontal overflow](bugs/2026-08/2026-08-09-canvas-background-and-scroll-width.md) — zero-engine paint, zero-browser page scroll
- 2026-08-08 [Renderer 等待图片期间重复重绘](bugs/2026-08/2026-08-08-renderer-pending-image-repaint-loop.md) — zero-webview, zero-renderer
- 2026-08-08 [Renderer deferred IPC 消息自旋](bugs/2026-08/2026-08-08-renderer-deferred-ipc-spin.md) — zero-renderer, zero-protocol
- 2026-07-31 [make test 在 http_proxy 设定时假失败（localhost fetch 被代理路由）](bugs/2026-07/2026-07-31-make-test-proxy-breaks-localhost-fetch.md) — apps/browser/src/tab_js_worker.rs（tab_js_worker_default_fetch_handler_real_http）, make test / rally 无人值守执行流程
- 2026-07-29 [reftest-upstream 大目录触发 test-guard OOM 杀进程（fail-list 捕获空致误判）](bugs/2026-07/2026-07-29-reftest-upstream-large-dir-testguard-oom.md) — tests/wpt-runner（cmd_reftest_upstream）, scripts/test-guard.rs（OOM 包裹器）
- 2026-07-25 [product-smoke 输出 PNG 路径陷阱（stale 文件致假 bug 误判）](bugs/2026-07/2026-07-25-product-smoke-png-stale-trap.md) — tests/wpt-runner（cmd_product_smoke）, legacy/product smoke 诊断流程

## Patterns — 可复用代码模式与最佳实践（11）

- 2026-08-18 [WebIDL readonly interface shape](patterns/2026-08/2026-08-18-webidl-readonly-interface-shape.md) — crates/engine/src/js_dom_shim/part02.js
- 2026-08-18 [IndexedDB transaction latest view](patterns/2026-08/2026-08-18-indexeddb-transaction-latest-view.md) — crates/storage/src/indexed_db/types.rs, crates/page-runtime/src/indexed_db_host/cursor.rs
- 2026-08-18 [IndexedDB persistence requires a single writer](patterns/2026-08/2026-08-18-indexeddb-single-writer-persistence.md) — zero-storage, zero-page-runtime, browser/renderer IPC
- 2026-08-14 [一致性验收应直接观测 live page](patterns/2026-08/2026-08-14-parity-live-page-observation.md) — apps/browser/src/parity_smoke.rs, zeroweb-browser-chrome-parity skill
- 2026-08-11 [Ordered font fallback must preserve the resolved primary face](patterns/2026-08/2026-08-11-ordered-font-fallback-primary-anchor.md) — zero-layout-engine, zero-engine
- 2026-08-11 [行内片段元数据必须贯通 stored paint 路径](patterns/2026-08/2026-08-11-inline-fragment-metadata-stored-path.md) — layout-engine, engine/paint
- 2026-08-09 [使用构建日期生成产品版本](patterns/2026-08/2026-08-09-build-date-product-version.md) — zero-product-version, zero-net, zero-engine, zero-browser, 打包脚本
- 2026-08-08 [后台轮询与窗口重绘解耦](patterns/2026-08/2026-08-08-background-poll-without-redraw.md) — zero-host-runtime, zero-browser
- 2026-08-05 [本地 Chromium 作 getComputedStyle 序列化 oracle](patterns/2026-08/2026-08-05-local-chromium-getcomputedstyle-oracle.md) — zero-engine getComputedStyle 序列化（crates/engine/src/js_dom_bridge/computed_style.rs）, reftest oracle 工具链
- 2026-08-05 [回调闭包 Send+Sync 约束：不能缓存 Document](patterns/2026-08/2026-08-05-callback-closure-send-sync-no-document.md) — zero-engine（js_dom_bridge.rs）, zero-script-sandbox（register_callback）, zero-dom（Document）
- 2026-07-20 [经验：reftest 布局诊断必须用 empirical ZW-output 验证，不能只靠 code-trace](patterns/2026-07/2026-07-20-reftest-layout-diagnosis-empirical-verification.md) — tests/wpt-runner（reftest harness）, crates/layout-engine（multicol 等）

## Performance — 性能优化经验（35）

- 2026-08-19 [bench-report.sh 编译/测量相位分离：批量 cargo 调用消掉串行编译开销](performance/2026-08/2026-08-19-bench-report-phase-split.md) — scripts, ci
- 2026-08-18 [Text-Only Font Overrides](performance/2026-08/2026-08-18-text-only-font-overrides.md) — crates/layout-engine/src/font_resolution.rs, crates/layout-engine/src/inline/mod.rs, crates/layout-engine/src/inline_finalization.rs
- 2026-08-18 [Snapshot Leaf Measurement Flags](performance/2026-08/2026-08-18-snapshot-leaf-measurement-flags.md) — crates/layout-engine/src/inline_finalization.rs, crates/layout-engine/src/inline/runtime_flags.rs
- 2026-08-18 [在布局 pass 边界快照环境开关](performance/2026-08/2026-08-18-snapshot-layout-env-at-pass-boundaries.md) — zero-layout-engine
- 2026-08-18 [Skip redundant final IFC after post-order finalization](performance/2026-08/2026-08-18-skip-redundant-final-ifc-after-post-order-finalization.md) — zero-layout-engine inline finalization
- 2026-08-18 [Reuse post-order inline-block IFC results](performance/2026-08/2026-08-18-reuse-post-order-inline-block-ifc.md) — zero-layout-engine inline finalization and postprocessing
- 2026-08-18 [按字体链分组 hmtx 字符缓存](performance/2026-08/2026-08-18-group-hmtx-cache-by-font-chain.md) — zero-render-foundation
- 2026-08-18 [受信任的 NodeId 热点可使用专用哈希](performance/2026-08/2026-08-18-fast-hash-trusted-node-ids.md) — zero-layout-engine, zero-engine
- 2026-08-17 [默认上下文应使用稀疏映射](performance/2026-08/2026-08-17-sparse-default-font-context.md) — zero-layout-engine 字体解析与行内布局
- 2026-08-17 [进程级 Paint 开关不应在 fragment 热路径读取](performance/2026-08/2026-08-17-snapshot-process-lifetime-paint-flags.md) — zero-engine text paint 与 variable-font metrics
- 2026-08-17 [热路径中的进程策略应统一快照](performance/2026-08/2026-08-17-snapshot-process-lifetime-ifc-flags.md) — zero-layout-engine
- 2026-08-17 [热路径运行开关应按布局快照](performance/2026-08/2026-08-17-snapshot-hot-runtime-flags-per-layout.md) — zero-layout-engine DOM-to-taffy 构树
- 2026-08-17 [大值映射应按精确键空间预留](performance/2026-08/2026-08-17-preallocate-exact-element-map.md) — zero-dom, zero-style-system
- 2026-08-17 [hmtx 文本测量缓存热路径](performance/2026-08/2026-08-17-hmtx-measurement-cache-hot-path.md) — zero-render-foundation, zero-layout-engine, zero-engine
- 2026-08-17 [昂贵子树扫描应先判断消费模式](performance/2026-08/2026-08-17-gate-expensive-scan-by-consuming-mode.md) — zero-layout-engine 最终 IFC
- 2026-08-17 [热路径映射应先确认消费方](performance/2026-08/2026-08-17-disable-unconsumed-hot-map.md) — zero-layout-engine DOM-to-taffy 构树
- 2026-08-17 [已规范化字体 alias 应直接查表](performance/2026-08/2026-08-17-direct-font-face-alias-lookup.md) — zero-render-foundation 字体 face matching
- 2026-08-17 [连续 last-write-wins 写入可延迟去重](performance/2026-08/2026-08-17-dedup-adjacent-last-write-metrics.md) — zero-layout-engine
- 2026-08-17 [布局构树应借用只读 ComputedStyle](performance/2026-08/2026-08-17-borrow-readonly-computed-style.md) — zero-layout-engine DOM-to-taffy 构树
- 2026-08-17 [递归继承应借用 owned 父样式](performance/2026-08/2026-08-17-borrow-owned-parent-style-across-recursion.md) — zero-style-system
- 2026-08-17 [ASCII 快路应先于 Unicode 属性查表](performance/2026-08/2026-08-17-ascii-before-unicode-property-lookup.md) — zero-layout-engine 行内文本度量
- 2026-08-15 [R3424-F 默认开启后 layout 10x 回归：每 IFC 全文档 collect/clone 的 O(n²)](performance/2026-08/2026-08-15-layout-advance-overrides-on2-collect.md) — layout-engine（inline_finalization / font_resolution / engine）
- 2026-08-14 [系统字体按需解析：避免启动时展开全量字形轮廓](performance/2026-08/2026-08-14-lazy-font-outline-parsing.md)
- 2026-08-12 [GPU 图片资源复用与 compositor 图片生命周期](performance/2026-08/2026-08-12-gpu-image-resource-reuse.md) — render-foundation/gpu, compositor
- 2026-08-12 [表单当前值更新应走 paint-only 路径](performance/2026-08/2026-08-12-form-value-paint-only.md) — zero-engine 渲染管线, renderer 表单输入
- 2026-08-12 [表单输入流畅度需要双层自动门禁](performance/2026-08/2026-08-12-form-input-smoothness-gate.md) — engine, page-runtime, browser, 性能门禁
- 2026-08-12 [页面输入事件的帧合并与 IPC latest-wins](performance/2026-08/2026-08-12-event-frame-coalescing.md) — page-runtime, renderer, browser process_backend
- 2026-08-09 [V8 isolate 懒创建 + 初始堆限制降低 WebView 常驻内存](performance/2026-08/2026-08-09-v8-isolate-lazy-create-and-initial-heap.md) — zero-webview（webview.rs）, zero-script-sandbox（lib.rs / v8_runtime.rs / worker.rs）
- 2026-08-08 [ViewPainted 积压帧合并](performance/2026-08/2026-08-08-view-painted-backlog-coalescing.md) — zero-browser, zero-renderer, zero-protocol
- 2026-08-08 [滚动 translate-blit 实现与 region 语义坑（M1-S1，2026-08-08）](performance/2026-08/2026-08-08-scroll-blit-region-culling.md)
- 2026-08-08 [CSS 解析器规则数 O(n²) 超线性缩放（已修复，2026-08-08）](performance/2026-08/2026-08-08-css-parser-quadratic-scaling.md) — zero-css-parser（tokenizer）, 性能门禁体系
- 2026-08-07 [reftest 的 --gpu 路径是 CPU 回退 stub（非真 GPU，且曾是 jobs=1 footgun）](performance/2026-08/2026-08-07-wpt-reftest-gpu-cpu-stub.md) — tests/wpt-runner/src/reftest.rs（render_to_framebuffer_gpu_with_base）, tests/wpt-runner/src/main.rs（effective_jobs）, crates/render-foundation/src/gpu/renderer/mod.rs（GpuRenderer / GPU_CREATE_MUTEX）
- 2026-08-07 [WPT reftest 单案成本 85% 是 fontdue 字体重解析（缓存默认字体后 ~100× 加速）](performance/2026-08/2026-08-07-wpt-reftest-font-parse-cost.md) — tests/wpt-runner/src/reftest.rs（render_with_layout_inner, BASE_FONT_LOADER）, tests/wpt-runner/src/reftest/reftest_fonts.rs（create_font_loader）, crates/render-foundation/src/font/loader.rs（FontLoader::load_font → fontdue::Font::from_bytes）
- 2026-08-07 [WPT reftest @font-face loader 缓存：键必须等于构造函数输入（+Arc 共享解析结果）](performance/2026-08/2026-08-07-wpt-reftest-font-face-cache.md) — tests/wpt-runner/src/reftest.rs（FRESH_LOADER_CACHE）, crates/render-foundation/src/font/loader.rs（FontLoader::duplicate, fonts: HashMap<u32, Arc<fontdue::Font>>）
- 2026-08-07 [CJK 字形栅格化重尾优化：FreeType face 缓存 + 采样哈希](performance/2026-08/2026-08-07-cjk-raster-face-cache.md)

## Platform — 平台与环境相关经验（14）

- 2026-08-18 [git worktree 共享 CARGO_TARGET_DIR 导致构建指纹污染](platform/2026-08/2026-08-18-worktree-shared-target-dir-fingerprint-collision.md) — 工具链 / cargo / git worktree / 性能 A/B 验证
- 2026-08-16 [Windows 同排标签栏与 Snap Layout](platform/2026-08/2026-08-16-windows-native-titlebar-tabs.md) — apps/browser, Win32 non-client hit test
- 2026-08-15 [test-guard --compile-first 直接执行测试二进制的 cwd 语义](platform/2026-08/2026-08-15-test-guard-compile-first-cwd.md) — scripts/test-guard.rs, Makefile（make test）, apps/browser, render-foundation 测试
- 2026-08-14 [Windows 上 bindgen 找不到 libclang](platform/2026-08/2026-08-14-windows-libclang-bindgen.md) — QuickJS feature, rquickjs-sys, Windows 开发环境
- 2026-08-13 [性能基线的平台分类必须区分 CPU](platform/2026-08/2026-08-13-perf-gate-platform-class-cpu-mismatch.md) — scripts/bench-report.sh, scripts/perf-gate.sh, docs/perf/baselines/
- 2026-08-13 [GPU 与性能门禁需要能力匹配](platform/2026-08/2026-08-13-gpu-and-benchmark-capability-gates.md) — Makefile, zero-render-foundation, zero-compositor, 性能门禁
- 2026-08-12 [Windows 全量测试的数据与端口隔离](platform/2026-08/2026-08-12-windows-test-data-and-port-isolation.md) — zero-wpt-runner, zero-webview, Windows 测试门禁
- 2026-08-12 [Windows WPT case id 不能使用宿主路径分隔符](platform/2026-08/2026-08-12-windows-msys-wpt-case-id.md) — wpt-runner, run-reftest-smoke.sh
- 2026-08-12 [Windows GUI 自动测试的进程与字体隔离](platform/2026-08/2026-08-12-windows-gui-test-isolation.md) — apps/browser, apps/renderer, tests/wpt-runner, Makefile
- 2026-08-12 [browser 多进程测试 spawn 陈旧 renderer 二进制导致误判回归](platform/2026-08/2026-08-12-stale-renderer-binary-browser-tests.md) — apps/browser（多进程 GUI 测试）, apps/renderer, crates/protocol/src/process.rs
- 2026-08-12 [多进程二进制查找与并行 GUI 测试的子进程竞争](platform/2026-08/2026-08-12-multiprocess-binaries-and-parallel-gui-tests.md) — zero-browser（compositor_client / process_backend / tests.rs）
- 2026-08-12 [`@font-face src: local()` 必须保持精确 face 身份](platform/2026-08/2026-08-12-font-face-local-exact-face-identity.md) — css-parser, render-foundation/font, WPT runner
- 2026-08-09 [macOS 上 product-smoke 的系统字体加载](platform/2026-08/2026-08-09-macos-wpt-runner-fonts.md) — tests/wpt-runner
- 2026-08-08 [macOS 子进程使用嵌套 Helper app](platform/2026-08/2026-08-08-macos-helper-app-bundle.md) — scripts/package-macos.sh, apps/browser/src/process_backend.rs, apps/browser/src/compositor_client.rs, crates/webview/src/image_decoder.rs
