# R386 — 多进程生产 worker 沙箱接原生 DOM 绑定（DC-1 架构层收口补片）

**日期**: 2026-08-31
**HEAD**: `a3b2d7459`（基线 `a758023c7`，R385 收官态）
**性质**: DC-1 verify 补片——R385 verify 矩阵「DC-1 全项 ✅」声明与代码事实的一处相悖点修正

---

## 1. 缺口（R385 收官核对发现）

R382/R384 的双引擎 `native_dom` default-on flip 只覆盖 **webview 进程内路径**（`ensure_sandbox`
→ `install_native_dom_bindings`）。生产浏览器的真实页面脚本走**多进程 worker 沙箱**：

- `apps/browser/src/tab_js_worker.rs`（`PageScriptRunner → TabJsWorkerHandle::execute_page_script`）
- `apps/renderer/src/js_worker.rs`（renderer 进程页面脚本执行）

两个 worker 各自 `generate_js_dom_shim` + `register_dom_callbacks`，**从未调用
`install_native_bindings*`**（grep 计数 0）——webview `external_script` 路径在
`ensure_sandbox` / `execute_script` 早 return，原生绑定安装面不可达。即真实浏览器 app 的
页面 JS↔DOM 仍只走 polyfill 字符串桥，Mission 架构层「双引擎 native 为唯一生产路径」
仅对 WebView 进程内路径成立。

## 2. 实现

### 2.1 worker bootstrap 安装（tab_js_worker.rs + renderer js_worker.rs 镜像）

worker 的 DOM 真相是 `dom_html` 快照字符串（live `Rc<RefCell<Document>>` 属 webview 线程，
不能跨线程），故从快照 re-parse——与 RectBridge handler 的确定性同源。经
`Sandbox::install_native_bindings`（v8）/ `install_native_bindings_quickjs`（quickjs-only）
escape-hatch 在持久 Context 安装 `dom_bindings` / `quickjs_dom_bindings`。安装在 shim
bootstrap **之前**，shim `__zw_native_ce_*` 探针（part03，R384 合流面）即可命中。

### 2.2 快照换代 refresh-only 快路径（关键设计）

`SetDomSnapshot` 消费方调 `refresh_worker_native_dom_source`：

- **V8**：同代际走 `refresh_dom_source_from_html`（本轮新增 engine API，封装 parse，
  维持调用方不依赖 `zero_dom` 的边界）；跨代际全量重 install。
- **QuickJS**：同代际走 `refresh_quickjs_dom_source_from_html`（本轮新增）；**禁止**
  全量重 install——quickjs 全量 install 会把 `globalThis.Event`/`CustomEvent`/
  `DOMException` JS 胶水构造器重挂到全局，**覆盖 shim bootstrap 建立的同名全局**：
  shim Event 实例带 `_defaultPrevented` 私字段，native 胶水实例缺失 → shim
  `_dispatchWithBubble` 的 `return !event._defaultPrevented` 恒 `!undefined` = true →
  `form.reset()` 的 preventDefault 失效（renderer `prevented_reset_and_submit_skip_default_actions`
  quickjs 矩阵实测回归，探针逐层定位后修复）。

### 2.3 生命周期 / 悬垂防护（R3334/R74 家族的 worker 版）

- renderer `ResetDocumentState`（`sandbox.reset_context()` 销毁重建 context）→
  `native_installed = false` + quickjs `reset_quickjs_state()`（绑定线程局部持已释放
  context 的 Persistent 引用，`JS_FreeRuntime: list_empty(&rt->gc_obj_list)` abort 实证）；
  下一快照换代全量重 install。
- 两 worker `Shutdown` 退出前清原生绑定线程局部（v8 `reset_native_state` / quickjs
  `reset_quickjs_state`）——镜像 webview Drop 语义，进程内后续 worker 线程干净启动。

### 2.4 顺带修复（同文件既存债）

- renderer `js_worker_main` 的 v8+quickjs **组合态** `js_config` 双 move（E0382）——
  tab_js_worker R84 同款修法（v8 分支 clone + quickjs 分支 `not(v8)` 门控）；CI 单
  feature 矩阵掩盖的组合态编译断。

## 3. Feature 矩阵注记

`zero-browser` 默认 feature 不含 v8/quickjs：bare `cargo test -p zero-browser` 的沙箱是
test-cfg QuickJS（`all(test, not(feature="v8"))`）且 engine 无绑定模块——原生安装面
不存在（= R386 前的 polyfill-only 现状），故 browser 侧 r386 回归测试按
`cfg(any(feature = "v8", all(feature = "quickjs", not(feature = "v8"))))` 门控：
`--features v8` ad-hoc 验证 + `--features quickjs`（Windows quickjs 步骤）可跑。
renderer 默认 feature = v8，且其 lib 测试不在 `make test` 批内（workspace 批
`--exclude zero-renderer`，bin 步骤只跑 bin target）——renderer r386 测试经 ad-hoc
双 feature 验证。

## 4. 验证

| 门 | 结果 |
|----|------|
| `make test`（R386 gate，commit `a3b2d7459`） | **18504P / 0F**（与 R385 收官数恒等；EXIT 0；含 v8 workspace + quickjs 矩阵 + Xvfb zero-browser 411P） |
| `tab_js_worker_native_bindings_installed_r386`（`--features v8` / `--features quickjs` ad-hoc） | 1P（工厂在位 / native 读快照 DOM / 快照换代刷新 / stale 元素 null） |
| `renderer_js_worker_native_bindings_installed_r386`（bare v8 / `--features quickjs`） | 1P（同上 + reset_context 后重 install） |
| renderer `prevented_reset_and_submit_skip_default_actions`（quickjs，过程回归） | 初版 FAIL（native Event 覆盖 shim Event）→ refresh-only 修复后 PASS |
| engine v8 / quickjs | 2511P / 1484P 全绿 |
| renderer lib（v8 + quickjs） | 153P × 2 |
| clippy v8 / quickjs / v8+quickjs 组合（`-D warnings`） | 全零警告 |
| fmt | 无 diff |
| bench-gate 定向（`ZERO_WEB_BENCH_CRATES=zero-engine,zero-webview,zero-script-sandbox`，load1=0.5 近空窗） | **GATE PASS 42/42（NEW=0）**——首轮 load1≈6.0（并行流负载）跑出 2 FAIL（paint/compositing ns 级指标），按 R383/R385 噪声判据不采纳，空窗复跑全 PASS；R386 refresh-only 快路径在 JS 桥热路径上 net≥0 |

## 5. DC-1 状态影响

DC-1 的 default-on + kill-switch 子项此前已闭合（R382/R384/R384b），但「生产
`run_page_scripts` 路径默认安装并使用原生绑定」的 verify 之前只覆盖 webview 进程内
形态。本片把安装面扩展到多进程 worker 沙箱（tab + renderer 双入口、双 feature），
生产浏览器页面脚本的 JS↔DOM 桥的 native 覆盖与 webview 路径对齐。

**注记（萎缩语义）**：worker 侧 polyfill 桥（`register_dom_callbacks` + shim `__zw_*`
查询/变更回调）仍保留——它是 worker 的 mutation 队列通道（`DomMutation` 批应用）与
M4 基线消费面；native 绑定提供了同沙箱内的原生读/建/查/事件面（与 webview default-on
后「shim 持续萎缩的消费面」取舍一致，RFC v0.3 §修订历史）。

## 6. 提交链

- `513a1345d` — feat: worker bootstrap install + refresh-only 快路径 + 生命周期防护 + 组合态 js_config 修复 + 回归测试
- `ff092140b` — test: browser r386 测试按 feature 存在性门控（bare 构建无原生面）
- `a3b2d7459` — fix: `refresh_quickjs_dom_source_from_html` 维持 zero-dom 依赖边界（zero-dom 是 browser 的 dev-dep）
