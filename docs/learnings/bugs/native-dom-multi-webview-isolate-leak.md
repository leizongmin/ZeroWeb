# native-dom 路径多 WebView 同线程顺序创建触发 disposed-Isolate panic（R3332 实测定位）

- 日期：2026-08-13
- 相关模块：`crates/engine/src/dom_bindings/gc.rs`（线程局部 DOM-source / element_template 缓存）、`crates/webview/src/webview.rs`（`install_native_dom_bindings`）
- 相关切片：R3332 native-dom 路径 parity 回归门（单 WebView 粒度，规避本 bug）

## 问题描述

P1b S0–S5 原生 dom_bindings（`crates/engine/src/dom_bindings/`，19 文件）默认关（`WebViewConfig.native_dom=false`），经 `native_dom=true` flag 入口（webview `install_native_dom_bindings` → `install_dom_bindings`）安装。R3332 建 parity 回归门时发现：**同一线程内顺序创建多个 native-dom WebView**（如 parity 门跨用例循环、或多标签页生产场景），第二个 WebView 执行脚本时 panic：

```
thread '...' panicked at .../v8-150.2.0/src/handle.rs:628:9:
attempt to access Handle hosted by disposed Isolate
```

## 根因（初步定位）

`gc.rs` 用**线程局部**（thread-local）存 DOM-source（`Rc<RefCell<Document>>`）+ element_template 缓存（`element_template_local`）+ native element 身份映射。首个 native WebView drop 时，其 V8 Isolate 被销毁，但线程局部缓存仍持有**指向已销毁 Isolate 的 V8 Handle**（global template / persistent）。第二个 WebView 建新 Isolate 后，`install_dom_bindings` / getter 经线程局部读到旧 Isolate 的 Handle → access disposed Isolate → panic。

对比：shim 路径（`native_dom=false`）无此问题——polyfill 全在 JS 字符串层，无 Rust↔V8 Handle 生命周期耦合。

## 复现

```
ZW_NATIVE_DOM=1 或 WebViewConfig{native_dom:true} 同线程顺序建 2 个 WebView，各 load_html + run_page_scripts_strict
→ 第 2 个 panic（v8::handle.rs:628）
```

R3332 单 WebView parity 门（`native_dom_path_parity_*_r3332` 三个测试，各建 1 WebView）**不触**此 bug——故 native parity 仍可安全锁（单页面单 WebView 场景 native 路径行为对等）。

## 影响

- **生产**：当前 `native_dom` 默认关 → 生产零影响。但「多标签页 = 多 WebView」是 native 路径默认开（S6/S7）后的核心场景，此 bug 是默认开前的**阻塞项**。
- **测试**：parity 门只能用「每测试 1 WebView」粒度，无法在单测试内跨用例循环（限制了 native 路径 WPT 覆盖广度）。

## 解决方案（当前：记录，未修）

闭合需：gc.rs 线程局部缓存的**生命周期与 Isolate 绑定**——WebView drop 时清理线程局部 V8 Handle（或改为 per-Isolate 缓存，非 per-thread）。属 native bindings 内部（连接 shim→native 的生产路径，rule 11 深结构风险），非自主可 land 切片。

**待 P1b S6/S7（shim 萎缩 / 默认开）用户点名时，本 bug 作默认开前的必修阻塞项**——届时 native 路径须支持多 WebView 同进程共存（多标签），否则默认开会 break 多标签。

## 如何避免

- 写 native-dom 路径测试时，**每测试建 1 个 WebView**（勿在单测试内循环多 WebView），否则触本 panic（误判为 native 绑定 parity bug）。
- 若需跨用例 native 覆盖，拆成多个 `#[test]`（每个 = 1 WebView），由 cargo 并行调度（各测试独立线程，不共享线程局部缓存）。
- 评估 P1b 默认开前，先闭合本 bug（多 WebView 同进程），否则多标签生产 break。
