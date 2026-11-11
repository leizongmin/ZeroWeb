# M2 切片 398 — OBJECT contentWindow 嵌套浏览上下文面（driving 2F→P）

日期：2026-11-11（承 s397 下一步：本流域散单——object.contentWindow 2F 中型切片）
commit：见 git log s398 提交

## 勘根

R5274（切片 67）定谳「runner 无 object 嵌套浏览上下文（不加载 data、不建 browsing context）→
contentWindow 恒 null」。driving `the-object-element/object-attributes.html` 要求分类型三态：

- obj1（`data=/common/blank.html`，text/html）→ contentWindow **非 null**；
- obj2（`type=image/png` 无 data）→ null（无资源可取，处理模型 nothing 尾步）；
- obj3（`data=missing.html`）→ null（fetch 404 → fallback，无嵌套 navigable）；
- t2：`obj1.contentWindow.name === "o"` 为**创建时点 name 内容属性快照**，
  setAttribute/removeAttribute 后仍 "o"（browsing context name 独立记账）。

基线 solo（/tmp/s398/solo-base-object-attributes.log）：2F = `object.contentWindow`（got null）
+ `object.contentWindow.name`（Cannot read properties of null）；width/height 2P（s396 hint 面）。

消费面清点（扩面回归核查）：可执行案中 object.contentWindow 断言仅 driving 一文件（object-events
为文件级 Timeout 不入键级账；historical/object-handler 断言 contentDocument——本切片不动该臂；
document-getters 断言 contentDocument/getSVGDocument null 且 data 为 SVG data URL——非 text/html
门同口径保 null）；无 `object.contentWindow === null` 断言案，扩面零回归面。

## 施工（两文件 + 契约桥测）

- `crates/engine/src/js_dom_shim/part01.js`：`_zwObjectNavCache`（独立缓存——不混入
  `_iframeDocCache`：object 嵌套 navigable 不走 iframe 的 SW 观察/history 账本/servedDoc 成熟
  翻转链，part02 SW refresh 遍历等 iframe 消费面零混入）+ `_zwLoadObjectNavigable`（首读惰性
  物化，R115 同步 fetch 契约同款）：无 data / fetch 缺席 / wire 空 / `__zw_fetch_error:` /
  非 2xx / Content-Type 非 text/html → error 态 null；text/html → `_zwMakeIframeDoc` +
  `_zwMakeIframeWin`（win.name = 创建时点 name 内容属性快照）。FIXME：类型判定按响应
  Content-Type 单源（无 MIME sniffing 全链）；嵌套文档不执行子文档脚本、不派 object load 事件
  （object-events 动态 data IDL setter 面散单另案）。
- `crates/engine/src/js_dom_shim/part04.js`：R5274 臂 OBJECT 分支拆臂——contentWindow 接
  `_zwLoadObjectNavigable`；contentDocument 保持恒 null（historical/object-handler 嵌套文档
  访问面散单另案）；getSVGDocument/EMBED 不动。
- `crates/engine/src/js_dom_bridge_tests/part100_s398_object_content_window.rs`：钉① 八面
  （text/html 物化非 null / name 快照 'o' / setAttribute 后仍 'o' / removeAttribute 后仍 'o'
  且 name IDL 反射 '' / 404 null / 无 data null / image/* null / 二次读同 identity /
  contentDocument 恒 null 边界）；钉② 无 fetch 通道 SVG data: URL 恒 null 保持（part87 钉①(b)
  语义新管线下两路保持）。

## 探针与陷阱（过程记录）

- 自建 iframe win.name 探针（`zw-probe-iframe-win-name.html` + helper.js）：实测 iframe win
  字面量无 name 槽（win.name 未实现——browsers 域 browsing_context_name 族欠账，不属本流域）；
  探针主源含 `<iframe` 字面被 skip 谓词静默排除零案 exit 1（s341 坑复现，helper 资产规避）；
  探针文件混入全域 sweep 案面 +1T 位移（跑完即删，记账口径排除）。
- **pairs 归一化连环伪影两轮**（s287/s292 教训重演）：日志行 `state key` 与 s397 基线 pairs
  `(state)\x1f(key)` 须同字节形态 join——`tr ' ' '\037'` 会把子测名内空格一并转掉、单侧
  `tr` 又造成字节形态不一致，均产 2.3MB 假位移；正确做法 = 两侧同用 `sed 's/ /\x1f/'`（仅
  首空格）或基线原样 sort -u 后 comm。

## 验证（串行管线，/tmp/s398/ 各段旗标+全量日志）

- driving solo 修复后（solo-fix-object-attributes.log）：object-attributes **exit 0 全绿
  4P**（2F→2P）；document-getters 3P 保持；the-object-element/ 目录 34 案对 s397 基线位移
  恰 driving 2 对（F→P），historical/object-events/object-handler Timeout 态零位移。
- 全树 sweep 双跑 join（基线 = s397 cur1-s397.pairs unique 8463，(state)\x1f(key) 同形态
  LC_ALL=C join）：
  - sweep1：unique 8464 = 8209P/217F/38T；gone 3 = driving 2F + radio-focused-and-scrolled-
    into-view 1F（子测名变化，见下）；new 4 = driving 2P + radio 探针案 1F + 本流已删探针 1T。
  - sweep2（稳定复跑）：unique 8464 = **8209P/218F/37T**；对 sweep1 仅 select-home-end-
    pagedown-pageup-detailed.optional.html T/F 翻转（s396 已定谳的负载时序 flake，真态 Fail
    在册红）。
  - 位移归因：driving 2F→P 精确落位；**radio-focused-and-scrolled-into-view 为共享 gitignored
    语料上的他流探针残留案**（上游断言体被替换为恒红 `assert_true(false, diag)` 滚动诊断，
    非本流变更面，按 s343 先例单列、headline 排除；归渲染流处置，本流不动他流探针文件）。
- 单元：`cargo test -p zero-engine -j 2`（test-guard 包裹）**3843P/7F**——7F 与 s395/s396/
  s397 在册红集完全同形（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 /
  fetch_filtered_response_type_matrix / iframe_service_worker_messages_dispatch /
  response_body_readable_stream_r2967 / t8k_websocket ×2），part100 两钉 ok 在列，零新增红；
  `cargo test -p zero-webview -j 2` **848P/2F** 与 s397 首记基线完全同形（image-decoder 子进程
  / CSP fetch mock）；`cargo test -p zero-script-sandbox -j 2` **442P/0F** 全绿。
- 产品 smoke：`make product-smoke` PASS（struct-check 0 issues）；`make product-smoke-legacy`
  PASS（42 fixtures，struct-check failures: 0）。
- fmt/clippy：`cargo fmt --check -p zero-engine` exit 0；`cargo clippy -p zero-engine
  --all-targets`（test-guard 包裹）Finished clean。
- shim 拼装检查：13 文件真实 include 序拼装 `node --check` 通过。

## 在册红口径

- 全域 unique 8464 = 8209P/218F/37T；**本流域口径 219F+37T → 217F+37T**（driving 2F→P）；
  全域 218F 含他流探针 1F 单列（radio-focused-and-scrolled-into-view，渲染流处置）。
- 本流已删探针（zw-probe-iframe-win-name）为测量期临时案，不入在册账。

## 残差与边界（钉死入账）

- object contentDocument 嵌套文档访问面（historical 2P+T / object-handler T——动态 data IDL
  setter 加载 + onload 派发 + contentDocument.location/replaceState 链）散单池留存。
- object-events（load/error 事件 + `obj.data = "/images/blue.png"` 动态加载 + 跨 realm
  instanceof）文件级 Timeout 散单池留存。
- iframe win.name 未实现（browsers 域 browsing_context_name 族欠账）——不属本流域，若后续
  browsers 域开工时与 window.name 语义一并勘根。
- EMBED contentWindow：HTMLEmbedElement 无此 IDL（fallthrough undefined）语义未动（spec 一致）。

## 下一步

- 在册红 217F+37T（本流域口径）。下一候选：本流域散单（focus scroll-into-view 1F、
  link-rel shadow DOM CSSOM 面）或 object contentDocument/object-events 中型切片（同管线
  扩 contentDocument 臂 + data IDL setter 加载链，historical/object-handler/object-events
  三案联动）；簇 D（customizable select 特性族 142F）仍待用户决策。
- 挂账池不变：`__zw_resolve_handle_selector` forward map 空返低优先勘根；渲染流 5 面→
  rendering-compat；form-submission-target 6T→navigation M3；sizes-auto lazy 半边待勘根；
  A2+D 簇立项征询待用户决策；新增他流 radio 探针残留案归渲染流处置。
