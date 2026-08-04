# P1a 架构侦察 + master.md「文档 vs 代码」一致性纠偏（2026-08-04）

> **状态**：侦察报告（recon），无代码变更。结论：**master.md 顶部对 P1a「fetch/MutationObserver/事件循环为 stub」的描述对生产路径已 stale**；P1b S1-S5 已在生产 worker 路径真实化 fetch/MO/timer。真正的 P1a 前沿 = 布局几何反馈（`getBoundingClientRect` 真实化 + IntersectionObserver/ResizeObserver），非重新实现 fetch/MO/timer。

## 背景

2026-08-04「主线切换裁决」恢复 P1a（事件循环补全 + fetch/MutationObserver 真实化，主要改 `dom_bridge.rs`）。R2640 落地首个切片（`generate_dom_api_polyfill` 的 data: URL fetch 真实解码）。本侦察在规划下一切片时核对实际架构，发现 **两条并行 JS bridge 路径**，且 master.md 描述与生产路径不符。

## 发现 1：两条并行 JS bridge 路径

| 维度 | 旧 polyfill 路径 | P1b worker shim 路径 |
|------|------------------|----------------------|
| JS 源 | `generate_dom_api_polyfill()`（`crates/engine/src/dom_bridge.rs`） | `generate_js_dom_shim()`（`crates/engine/src/js_dom_shim.js`） |
| 宿主桥 | 无（同步桩） | `FetchBridge`(S3) + `TimerBridge`(S5) + `AsyncResolver`(S1) + `register_dom_callbacks` |
| 异步 | 否（`setTimeout` 同步即触发；`fetch` 桩返回空 200） | 是（`__zw_setTimeout` 子线程 sleep + `__zwResolveCallback`；`__zw_fetch` 异步抓取） |
| 谁用 | **WebView 嵌入**（`crates/webview/src/webview.rs:1017`）+ 全部 polyfill 测试 | **Browser / Renderer 生产**（`apps/browser/src/tab_js_worker.rs:11,216-217` 构造 AsyncResolver+FetchBridge+TimerBridge 并注入 shim） |
| fetch | 桩（R2640 起 data: URL 真实） | **真**（P1b S3 incr-c） |
| MutationObserver | 桩（不触发回调） | **真**（P1b S2 incr1/incr2：JS 侧拦截 + microtask 派发 `obs._callback(records,obs)`） |
| setTimeout | 同步即触发 | **真**（P1b S5：子线程 sleep + resolve） |

**证据**：
- 生产 worker：`tab_js_worker.rs:11` `use ... AsyncResolver, FetchBridge, TimerBridge, generate_js_dom_shim, register_dom_callbacks`；`:216-217` 注释「P1b S1/S3：AsyncResolver + FetchBridge（`__zw_fetch` 注册 + handler cell）」。
- shim 真实 fetch：`js_dom_shim.js:51-71` `fetch(url)`→`__zw_fetch(id,url)`→异步 resolve→`_makeResponse(body)`（spec `ok/status/text()/json()`，`__zw_fetch_error:` 前缀 → `ok:false`）。
- shim 真实 MutationObserver：`js_dom_shim.js:136-208`（S2，`obs._callback(records, obs)` @ :184）。
- 旧 polyfill 仅 WebView + 测试用：`generate_dom_api_polyfill` 调用方 = `webview.rs:1017` + `dom_bridge_tests.rs` / `dom_bridge_extended_tests.rs` / `tests/integration/src/dom_bridge_polyfill.rs`（全测试，除 webview.rs 一处）。

## 发现 2：master.md「fetch/MO/timer stub」对生产路径 stale

`zero-web/master.md` 顶部（用户 2026-08-04 刷新）称：「Observer 类型为 stub 不触发回调，fetch() 为 stub 返回空 Response，事件循环为简化版（非 spec-compliant microtask/task queue）」。

- 对**旧 polyfill/WebView 路径**：准确。
- 对**生产 browser/renderer worker 路径**：**stale**——fetch（S3）、MutationObserver（S2）、setTimeout（S5）、异步 resolve（S1）均已真实化。microtask 经 `perform_microtask_checkpoint`（`v8_runtime.rs:307/374`）drain。

R2640 的 data: URL fetch 切片改的是**旧 polyfill（WebView 路径）**，非生产 browser 路径。对 WebView 嵌入者仍有效，但**不是**生产 fetch 的实现位置。

## 发现 3：真正的 P1a 剩余缺口（生产 worker 路径）

侦察确认 shim 已覆盖 fetch/MO/timer，**未覆盖**：

1. **`getBoundingClientRect` 是桩**（`js_dom_shim.js:776-783`：返回零 `DOMRect` 不抛，注释「布局测量 API：动态 reftest 极常用 … 返回零 DOMRect 不抛」）。这是**真缺口**——基于布局测量的 JS（动态 reftest、动画、视口检测）被阻断。
2. **IntersectionObserver**：shim 无真实实现（需布局几何反馈计算 intersection ratio）。
3. **ResizeObserver**：shim 无真实实现（需布局尺寸变化反馈）。
4. **路径分裂**：WebView（polyfill 桩）vs browser（shim 真）——嵌入者与浏览器行为不一致，长期应统一。

三者共同依赖 **布局几何反馈 plumbing**（host 把 layout 结果喂回 JS），这是 P1a 真正的前沿与高价值方向，非重新实现已完成的 fetch/MO/timer。

## 结论与建议

1. **纠偏 master.md**（待用户工作区未提交编辑 land 后）：P1a 描述应区分两条路径；生产 worker 路径 fetch/MO/timer 已真（P1b S1-S5），旧 polyfill/WebView 路径仍桩。
2. **P1a 下一最高价值方向 = 布局几何反馈**：
   - 首切片候选：`getBoundingClientRect` 真实化（host 在 mutation 应用重渲染后回填真实 DOMRect 到 shim），解锁布局测量 JS——高 driving-test 价值（动态 reftest）。
   - 续：IntersectionObserver / ResizeObserver（基于同一 layout-rect 反馈基建）。
3. **勿在旧 polyfill 重复实现 fetch/MO/timer**（生产 shim 已有）；旧 polyfill 仅 WebView 用，按需补齐或长期统一到 shim。
4. **R2640 data: URL fetch 保留**（WebView 路径净正向，8 测试），但标注其非生产 browser 路径。

## 下一轮

- 读 `js_dom_bridge.rs` + `tab_js_worker.rs` 核验 layout-rect 反馈接入点（mutation 应用 → 重渲染 → DOMRect 回填的 plumbing 是否已部分存在）。
- 出 `getBoundingClientRect` 真实化可回退切片设计（kill-switch + 三态 A/B + driving test）。
- 待用户 zero-web 文档编辑 land 后，纠偏 master.md P1a 描述。
