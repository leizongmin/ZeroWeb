# M2 切片 397（s397）——module parse 失败缓存语义施工（tag-5 契约面，driving 1F→P）+ current-pixel-density 域分诊定谳路由

日期：2026-11-11（承 s396 下一步：forms 树散单/分诊推进或按域 solo 分诊；本切片 = 分诊轮转溢出的 script-sandbox 施工 + embedded-content 域分诊）

域：html/semantics/scripting-1/the-script-element/module/dynamic-import（driving）+ embedded-content/the-img-element 域分诊（路由）

性质：**施工轮（script-sandbox/webview 契约单面，shim prelude 一臂 + 宿主发射一臂）+ 域分诊定谳（零源码改动部分）**

> 交接说明：本 session（429 限流恢复后）先收口 s396 残件（commit 4a0d1ba65，rebase origin/main 5 笔渲染流提交零冲突），再做全域分诊后转溢出候选施工。

## 分诊定谳（非 forms 大池全域扫荡，s392 方法论）

在册红 220F+37T（s396 sweep2 基线 /tmp/s396/cur2-s396.pairs）域分布：forms 164F+10T（其中 the-select-element 146F 里 **142F = customizable-select 特性族**——s389 簇 D appearance:base 特性门同族，立项待用户决策不可自主施工）、embedded-content 31F+14T、interactive-elements 16F、tabular-data 4F、document-metadata 4F、scripting-1 1F。非 forms 可行动池逐一定谳：

- **current-pixel-density 10F → 渲染流路由（两根新钉）**：
  - basic.html 3F 单根：`crates/layout-engine/src/engine/sizing.rs:3354-3355` R695 谱系 `iw.max(1.0)/ih.max(1.0)` 钳制把本流 s219 密度折算产出的合法亚像素/零固有尺寸（256/10000=0.0256、/∞=0）抬回 1px → clientWidth 1≠0（三案同签名 `expected 0 but got 1`）。本流 extract.rs 折算面（`Some(+Inf)`/`Some(10000)` 语义）正确，断点在 layout replaced 补设臂。归属 layout-engine（渲染流 crate，规则 9；R5498 未触 sizing.rs 但 crate 归属不动）。
  - error.html 7F 单根：broken img 无固有尺寸供给面（alt 文本/default object size 渲染语义缺失）→ box 0×0 → `ref.width`=0 → 7 案齐红于 `assert_not_equals(expected_width, 0)`。layout 侧 alt 文本行内盒供给归渲染流；shim width IDL rendered-width fallback 臂随 layout 修复配套（本流）。
  - svg-img-density-vs-zoom 3F → CSS `zoom` 属性面（zoom:2 rendered size 缩放/naturalWidth 不变），归属渲染流 CSS zoom 特性（css-parser/render-foundation 面），不入本流挂账。
- **dialog 15F = 既有 R-s31 挂账确认**：居中簇 8F（abspos-dialog-layout 6F + multiple-centered 1F + scroll-height 1F「centered before computing overflow」）由 UA 定位注入回退先例（lib.rs:2110 R-s31 备注——依赖布局侧 abspos 收缩/fit-content 语义）覆盖，无新信息；余 7F 为 light-dismiss bounds/corner-cases（s374 谱系）、top-layer stack、inert（在册）散面。
- **tabular-data 4F**：col-span-limits 2F = 表布局亚像素精度（53 vs 51.99985）、rowspan-0 2F = rowspan=0 展开 clientHeight 面——table 布局面归渲染流。
- **document-metadata 4F**：render-blocking link stylesheet 2F = render-blocking 机制立项级；link-rel shadow DOM 1F + non-parser-doc style sheet 1F = CSSOM 混合面散单。
- **the-input-element 4F**：`:active` label 传递 2F、readonly date keyboard 1F、focus scroll-into-view 1F——交互散单池。
- **scripting-1 1F（本切片 driving）**：本流域 script-sandbox 面，转施工。

## 勘根（driving 1F 失败签名）

`dynamic-imports-script-error.html :: import() must reject with the same error object for each import when there is a parse error`：同 URL 两次 `import(spec+"?x")` 必须以**同一错误对象** reject（module map 失败记账语义，HTML spec fetch-a-single-module-script）。红态实证（s396 sweep2.log）：两 reject 均为 `SyntaxError: SyntaxError: Unexpected token '%'`（双前缀=态④ `new SyntaxError(宿主消息)` 包装形态）——s373 module goal parse 预检（webview `compile_module_contract` gate 臂）把 parse 失败发 **tag 3**，shim `__zw_load_module` 态④语义=逐次新 SyntaxError（不入缓存）。而 tag 3 的逐次新对象语义只对**链接/instantiation 失败**正确（WPT instantiation-error 判 differentErrorObjects）；parse 失败属 module map 失败记账，应与态③ parse-eval 抛错同侧（缓存复抛同对象）。

## 施工（契约单面：宿主发射 + shim 消费 + 桥测）

- `crates/webview/src/webview.rs` compile_module_contract gate 臂：parse 预检失败 `3\u{1f}…` → **`5\u{1f}{resolved}\u{1f}{msg}`**（tag 5 = parse 失败缓存语义）。tag 5 仅此臂发射——native resolver 路径 parse_gate=None，v8_runtime 契约解析器不暴露新态；defer 面（compile_module_contract_deferred）无 parse gate 不接新 tag；renderer/tab 旧宿主无 tag 发射零漂移。
- `crates/script-sandbox/src/es_module.rs` `__zw_load_module`：tag-3 臂后增态⑤——`5` 载荷解析 resolved/msg，首抛 `new SyntaxError(msg)` 记入 `__moduleFailures[resolved]`、同键命中复抛同对象（与 tag 3 同序前置：resolved 通用提取前分派）；s87 失败四态文档注释扩为五态。
- `crates/engine/src/js_dom_bridge_tests/part99_s397_module_parse_failure.rs` 新增（include 于 js_dom_bridge_tests.rs）：mock 宿主契约钉死两面——tag 5 同 URL 复 `__zw_load_module` 抛**同一** SyntaxError 对象（`e1 === e2` identity + instanceof）；tag 3 对照臂保持逐次新对象（态④ differentErrorObjects 面不回退）。

## 验证（串行管线，/tmp/s397/ 各段旗标+全量日志）

- driving solo 修复后（solo.log）：`dynamic-imports-script-error.html` **11P/0F exit 0**（1F→0F 文件全绿；同文件其余 4 态失败语义案不受扰）。红态证据=s396 sweep2.log 同文件 1F 行。
- 全树 sweep 双跑 join（基线 s396 sweep2 unique 8463；(state,key) 对 0x1F 分隔 LC_ALL=C join）：
  - sweep1：8463 unique = **8207P/219F/37T**；gone/new 恰为 driving 1 对（F→P），零意外位移、零 Timeout 位移（渲染流 origin 5 笔 rebase 合入同树亦零位移）。
  - sweep2（稳定复跑）：同分布同 join 定谳。**在册红 220F+37T → 219F+37T**。
- 单元：`cargo test -p zero-script-sandbox -j 2` exit 0 全绿（含 prelude 自测）；`cargo test -p zero-webview -j 2` **848P/2F**——2F（proxy_mode_decodes_png_correctly image-decoder 子进程启动失败 / csp_connect_gate_blocks_fetch_sh1_m2s5 fetch mock env）**stash 归因旧树复跑同红 = 预存 env 红，零新增**；`cargo test -p zero-engine -j 2` **3840P/7F**——7F 与 s395/s396 在册红集完全同形（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch / response_body_readable_stream_r2967 / t8k_websocket ×2；s396 轮临时命中的 r5296 渲染流 env 竞态本轮未复现），新增 part99 桥测在列，零新增红。
- 产品 smoke：`make product-smoke` PASS；`make product-smoke-legacy` PASS（42 fixtures）。
- fmt/clippy：`cargo fmt --check -p zero-script-sandbox -p zero-webview -p zero-engine` exit 0；`cargo clippy -p zero-script-sandbox -p zero-webview -p zero-engine --all-targets`（test-guard 包裹）Finished clean。
- shim 拼装检查项：本切片无 js_dom_shim 13 文件变更（prelude 由 es_module.rs Rust 生成，桥测直接驱动），无拼装 check 项。

## 残差与边界（钉死入账）

- current-pixel-density 10F + svg-zoom 3F 归渲染流（上文分诊定谳），本流挂账池新增三路由条目。
- webview 2F 预存 env 红（image-decoder 子进程 / CSP fetch mock）——首次发现记录，webview 套件基线 848P/2F，后续轮次对比用。
- s396 轮 r5296 env 竞态本轮未复现（渲染流在册，本流零债）。

## 下一步

- 在册红 219F+37T。forms 主池 142F customizable-family 卡簇 D 用户决策；散单池各面已定谳（dialog 居中簇=渲染流 R-s31 在册、input 交互散单、tabular/document-metadata 归渲染流或立项级）。下一候选：本流域剩余散单（object.contentWindow 2F 中型切片、focus scroll-into-view 1F、link-rel shadow DOM CSSOM 面）或等簇 D 决策开 customizable select 特性施工。
- 挂账池更新：+current-pixel-density basic 3F（sizing.rs 钳制根，→渲染流）、+current-pixel-density error 7F（broken img 尺寸供给根，layout 主体→渲染流 + shim IDL 臂配套本流）、+svg-zoom 3F（→渲染流 CSS zoom 立项）。不变：`__zw_resolve_handle_selector` forward map 空返低优先勘根；渲染流 5 面→rendering-compat；form-submission-target 6T→navigation M3；sizes-auto lazy 半边待勘根；A2+D 簇立项征询待用户决策。
