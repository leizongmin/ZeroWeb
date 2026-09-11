# M4 — runner viewport 校准：shim viewport 真值桥接 + 差分重校准（2026-09-12）

**决策项**：goal 待用户决策清单「runner viewport 校准（1280×800 → 上游 WPT 校准 800×600）」
按建议执行——非「换一对硬编码常量」，而是结构修复（viewport 从 WebView 实际 config 桥接，
shim 常量仅作裸沙箱 fallback）+ 差分驱动重校准。

## 根因：三处真值互相矛盾（改动前）

| 面 | 值 | 出处 |
|---|---|---|
| testharness runner 实际布局 viewport | **800×600** | `testharness.rs` `WebView::new(WebViewConfig { width: 800, height: 600 })` |
| reftest 面布局 viewport | **800×600** | `reftest.rs` `ReftestConfig` 默认 |
| shim `innerWidth/innerHeight/outer*` stub | **1280×800** | `part01.js` 常量，全仓无任何桥接同步 |

后果：`documentElement.clientWidth/Height`（M1 切片 3c viewport 化，part04.js 直读
`globalThis.innerWidth/innerHeight`）返回 **1280×800 stub 值**，而 IO `rootBounds` 来自
800×600 真实布局——`rootBounds.bottom == documentElement.clientHeight` 型断言拿 600 比
800，系统性偏差。3c 的修复建立在错误真值上。

## 结构修复

`crates/webview/src/webview.rs`：

- `ensure_js_shim()`：shim 注入后执行 `zero_engine::script_user_resize(config.width,
  config.height)`（复用 R3254 既有桥，零新 JS 面）——`innerWidth/innerHeight/outerWidth/
  outerHeight ← config 真值`；`screen.*` 保持设备常量语义不动（viewport ≠ 屏幕）。
  - 时序安全性：安装时页面脚本未运行，无 resize 监听器/MQL 注册，事件派发与 matchMedia
    重评为 no-op（不产生 init resize 事件的可观测差异）。
  - **先置位 `js_shim_initialized` 再 sync**：sync 失败不可触发下次调用重装 shim（重复
    注入重置 `_nodeMap` 丢失监听器——幂等守卫的存在理由）；sync 仅在沙箱已损坏时失败，
    严格传播错误。
- `run_page_scripts_impl`：内联 shim 安装块（与 `ensure_js_shim` 重复的唯一另一处）收敛
  为 `self.ensure_js_shim()?` 调用——两处安装点收敛后 viewport sync 单点存在。

影响面：**仅 in-process 嵌入路径**（wpt-runner testharness / integration / webview-demo /
单进程模式）。生产多进程路径经 `external_script` 早 return 不触 `ensure_js_shim`，其
viewport 同步由既有链路承担（browser → IPC `SetViewport` → renderer `handle_set_viewport`
→ `script_user_resize`；headless/窗口化均在 renderer spawn 时发初始 SetViewport）。
裸沙箱（engine `js_dom_bridge_tests` 直接 execute shim）走 fallback 常量 1280×800，
既有断言不变。

## 差分验证（逐 subtest，test-guard 包裹，--json 双臂）

BEFORE = HEAD `8143701a1` release 二进制（存 `target/release/zero-wpt-runner.before`），
AFTER = 修复后二进制。差分脚本口径：`[path, [{name,status}]]` 逐 subtest 集合差。

| corpus | BEFORE | AFTER | 翻转明细 |
|---|---|---|---|
| intersection-observer | 94P/122F/2T（218） | **100P**/116F/2T | Fail→Pass ×6（`First rAF.` 簇：multiple-thresholds / same-document-no-root / same-document-with-document-root / same-document-zero-size-target / text-target / visibility-hidden）；`zw-probe` 校准后无回归 |
| resize-observer | 18P/33F/6T（57） | 18P/33F/6T | **零 delta** |
| dom 全量 | 54302P/90F/6PCF/15T（54,413） | 54303P/90F/6PCF/14T | 仅 `dom/events/handler-count.html?element` case 级 Timeout→完成（+1 Pass，贴超时边界的噪声级抖动，非 viewport 语义） |
| html（forms/focus） | 14P（14） | 14P（14） | **零 delta** |

成功标准（「Pass 净数不降 + 失败重新归因」）满足：IO +6 全部归因 viewport 真值一致化
（`First rAF.` 簇断言 rootMargin/clientHeight 型 viewport 相对几何），无未归因回归。

## 自写面重校准

- `tests/wpt-runner/wpt-data/intersection-observer/zw-probe.html`（本地诊断探针，非上游、
  不计 imported-testharness 账本）：原按 1280×800 stub 绝对校准（提拉后断言
  `gBCR.top === 608`，在旧视口 800 内、新视口 600 外）→ 改视口无关写法：body margin 归零
  + 提拉目标 `top = innerHeight - 100`（运行时按 `innerHeight` 计算）+ 断言区间
  `0 <= r.top < innerHeight`（相对断言替代绝对 px）。
- engine 裸沙箱测试（part01 `innerWidth=1280` 初值断言 / part02 matchMedia @1280 /
  part06 `innerHeight=800`）走 fallback 常量路径，**零改动零回归**（2673P/0F 实证）；
  需要非默认视口的既有测试本就用直接赋值覆盖（part09 IO 测试 `innerWidth=100`）。

## 质量门禁

- `cargo test -p zero-webview` 701P/0F；`cargo test -p zero-engine` 2673P/0F；
  `cargo test -p zero-integration-tests` 781P/0F（59 ignored 既有）
- `cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo fmt --all -- --check` 无 diff
- 全量 `make test` 见本切片提交后的 ②b ON 臂轮一并汇报（本轮单 crate + integration 面
  全绿先行记录）

## 后续效应（记录）

- 上游 WPT 用例本按 800×600 校准——此后 `make import-wpt` 的几何类用例可尽量原样落库，
  不再需要按 1280×800 改期望；越晚校准成本单调上涨的 trend 至此截断。
- 已知边界（不改，记录）：in-process 嵌入方直接调 `WebView::resize()` 不派发
  `__zw_user_resize`（innerWidth 不随 resize 更新）——生产多进程由 renderer
  `handle_set_viewport` 承担；in-process 嵌入方如需 JS 可见 resize 变化，经
  `zero_engine::script_user_resize` 显式注入。本切片不扩 `resize()` 行为（会致生产
  resize 事件双派发）。
