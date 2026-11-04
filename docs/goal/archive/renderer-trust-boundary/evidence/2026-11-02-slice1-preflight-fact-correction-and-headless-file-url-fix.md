# ① 预执行核查：file:// 直读面事实修正 + headless file:// 直读臂修复

**日期**：2026-11-02
**HEAD（起点）**：`a8e0cb438`（切片 6 media-host 交付态）
**性质**：预执行核查切片（拍板 ①A「拍板时补强核查」条款执行）+ 先存缺陷修复（headless file:// 导航恒失败，修复解锁本核查的实证面）。
**代码提交**：`5a240188e`（fix(browser)，headless file:// 直读臂 + 常驻回归）、`9cf255acf`（docs(renderer)，沙箱模块文档事实修正）。

## 背景与范围

拍板 ①A（2026-11-02，[决策包](../2026-11-02-decision-package-media-host-devtools.md)）：browser 预解析 file:// → LoadHtml + 模块脚本改代理路径，然后翻 `ZW_RENDERER_LANDLOCK` 默认。拍板时核查条款：「file:// 子资源已走 browser fetch_proxy，renderer 直读仅主文档+模块脚本」。本切片执行该条款：逐链核对多进程产品代码路径 + 真实进程 landlock on/off 双腿实证。

核查中发现 headless 侧先存缺陷：`headless::session::proxy_fetch` 缺 file:// 直读臂（GUI 侧 `fetch_proxy::enqueue` 一直有，apps/browser/src/fetch_proxy.rs:274-287），文档与子资源 file:// 全落 reqwest builder error 错误页——headless CDP 导航本地文件恒失败，双腿实证无法进行，先修（`5a240188e`）。

## 事实核查结果（逐链代码核对，修正拍板前提）

多进程 renderer 文档加载链：`handle_navigate` → `AsyncPageLoad::start` → tick 经 `IpcAsyncFetchHost` → IPC `FetchRequest` → browser `fetch_proxy.enqueue`（file:// 臂 → `read_file_url`）。据此：

| 面 | 拍板时表述 | 实际链路 | ①A 对应动作 |
|---|---|---|---|
| file:// 主文档 | renderer 直读 | browser fetch_proxy（IPC）直读 | 「browser 预解析 → LoadHtml」**为 no-op**（已 browser 侧） |
| file:// 子资源（css/img） | 已走 fetch_proxy | 同上（enqueue file:// 臂） | 无动作（已符合） |
| 模块脚本**静态** import 依赖 | renderer 直读 | `script_prefetch.rs` 经 `host.fetch_text_meta` → IPC → browser 直读（prefetch 展开静态 import） | 「模块脚本改代理路径」**静态面为 no-op**（已代理） |
| 模块体**动态** import() | （未单列） | renderer worker 本地直读：`__zw_dynamic_import` → `__zw_load_module` → `__zw_compile_module` 回调 fetch 闭包（apps/renderer/src/js_worker.rs:2630）→ `zero_net::HttpClient::get` file:// 臂（crates/net/src/client.rs:643 `read_file_url`） | **①A 唯一剩余执行面** |

结论：**renderer file:// 本地直读面收窄为唯一一臂——模块体动态 import**。翻 landlock 默认只需处理该面，无需做主文档预解析迁移。

### 连带观察（非本 goal 面，记档移交）

- **classic-script 动态 import 在 renderer 恒「Not supported」**：V8 host resolver `set_dynamic_import_resolver` 仅单进程 webview.rs:4127 注册，renderer 多进程路径未注册（先存单/多进程能力差，与本核查无关，移交 script/renderer 兼容流参考）。
- **页面 `fetch()` file:// 不经 file:// 臂**：`HttpClient::send/send_async*` 无 file:// 分叉（仅 `get_with_headers` 有），页面 fetch file:// 走 reqwest 恒败（先存，Chrome 亦禁，记档不追）。

## headless 缺陷修复（5a240188e）

`proxy_fetch`（apps/browser/src/headless/session.rs:769）补 file:// 直读臂（session.rs:844-859），形态与 GUI `fetch_proxy::enqueue` 同款（`is_file_url` → `zero_net::read_file_url`，旁路 HTTP 缓存/调度；同步完成 inline 形态同「线程创建失败」兜底）。常驻回归 `apps/browser/tests/cdp_file_url_navigation.rs` 两用例钉产品链：默认腿 + `ZW_RENDERER_LANDLOCK=1` 腿，各断言 title / 静态模块链（`window.__modOk == "dep-42"`，fixture index.html→main.js→dep.js）/ 样式子资源 / 非 `error://` 错误页。该回归在 landlock 默认翻转后仍成立（两腿覆盖的链路均 browser 侧直读），是 ① 验收前提的钉。

## landlock 双腿实证（一次性行为判别，不落仓）

fixture：index.html（module src=main.js + classic script）+ main.js（静态 import dep.js 置 `__staticMod=42`；模块体内 `import("./dyn.js")` 置 `__dynResult`/`__dynMod`）+ dep.js + dyn.js + pixel.png。headless 真实进程（CDP navigate + evaluate）双腿：

| 断言 | 腿 A 默认（landlock 关） | 腿 B `ZW_RENDERER_LANDLOCK=1` |
|---|---|---|
| document.title | 正常加载 | 正常加载 |
| `__staticMod`（静态 import 链） | 42 | 42 |
| dep.js 已执行 | true | true |
| pixel.png naturalWidth（子资源） | 1 | 1 |
| **`__dynMod`（模块体动态 import）** | **7** | **null（被拒）** |

判别式成立：landlock 在生产 renderer 完整生效；landlock 开启时唯一被拒面 = 模块体动态 import 本地直读；「打开本地文件」主链（主文档/子资源/静态模块依赖）双腿全通——与逐链代码核对结论互证。

## 新待决：模块体动态 import file:// 面取舍（翻默认前须拍板）

模块体动态 import 走 worker 本地直读是当前「能用」行为；landlock 默认开将拒绝该面。代理化受死锁约束：worker 线程（`renderer-js-N`）执行 sandbox 回调，主循环以 `reply_rx.recv_timeout(TAB_JS_CHANNEL_TIMEOUT)` 同步等待——worker→host 的同步 IPC fetch 代理会死锁，须先泵化等待通路（中等规模改造）。选项：

- **①-a 接受收窄翻默认**：landlock 默认开，file:// 模块体动态 import 被拒（`ZW_RENDERER_LANDLOCK=0` 操作员豁免）。Chrome 对 file:// 页面模块加载同样 CORS 拒绝，收窄即 parity；代价是当前可用行为回归。
- **①-b 先泵化代理再翻默认**：worker→host fetch 走可泵等待，动态 import 经 browser 直读，行为不回归；改动面 = renderer 主循环等待机制 + fetch 代理接线，风险与工作量均高于 ①-a。

## 验证（test-guard 包裹）

- `cargo test -p zero-browser --test cdp_file_url_navigation`：**2P/0F**（0.69s）。
- 邻近 cdp_* 6 套件全绿：navigate_evaluate 9P（一次并行负载 flaky `default_async_probe_discovers_iframe_before_busy_arm_blocks`——4 二进制并行首跑红，clean-main 隔离单跑 + 本树全量复跑双绿，归因负载非本改动；本改动仅新增 HTTP 测试不走分支的 file:// 臂）/ fetch_cache 6P / history_traverse 2P / focus_governance 2P / input_events 4P / t27p35_click_latency 2P。
- fmt clean（zero-browser + 全仓 check）；guarded-clippy `-p zero-browser --all-targets -D warnings` 清零（新测试 1 处 collapsible_if let-chains 化）、`-p zero-renderer --all-targets -D warnings` 清零（注释修正）。

## 范围与残余

- ① 状态：拍板 A 的主文档迁移与静态模块代理两半经核查均为 no-op；**执行被新待决阻塞**（①-a/①-b），landlock 维持 opt-in 直至拍板。
- 环境受限残余不变：macOS 真机、android 编译腿 + service bootstrap 接线。
- classic-script 动态 import renderer 缺口与页面 fetch() file:// 面为连带观察，移交对应流，本 goal 不追。
