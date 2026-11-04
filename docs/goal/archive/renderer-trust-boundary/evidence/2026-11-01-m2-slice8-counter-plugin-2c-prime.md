# M2 切片 8：`__zw*` 特权 mutator counter-plugin（②A-env）+ 内部消费闭包化（②C'）

**日期**：2026-11-01
**HEAD（起点）**：`d251ea900`（决策包拍板提交，rebase 于 434e36f44 之上；本轮 `git pull --rebase` 同步后起点即拍板态）
**性质**：修复切片（DC-1/DC-2 决策面剩余项，拍板 ②C' 全收口）。

## 背景与实现要点

决策包 ②C' 拍板：②A-env counter-plugin 删 3 枚 + `__zwSetPermission` shim 内闭包化，全收口不留 fullscreen 权限路页面自授残余。实现落点：

- **新增 `crates/engine/src/counter_plugin.rs` + `js_dom_counter_plugin.js`**（本 goal 新文件）：删除面（`_zwMutators` 数组锚定恰 4 枚——`__zwUserActivate`/`__zwClipboardStoreWrite`/`__zwDeleteAllCookies`/`__zwSetPermission`）+ `shim_test_mode_enabled()`（`ZW_RENDERER_AUTOMATION=1`，`zero_runtime_config::enabled_when_true` 语义）。shim 三文件定义保留（幂等卫 + 测试模式 runner 直调）。
- **②C' 闭包化**：part02 外层 IIFE 作用域声明 `_zwPermSetState`/`_zwClipStoreWrite` 闭包别名，clipboard IIFE 内两钩子定义处赋值（幂等卫后取当前闭包，context 重建/重复装载随最新闭包刷新）；request() 自动授予（part02:9639-9641）与 part06 execCommand copy 桥消费点改走别名。**范围修正**：决策包原表述「part02 单文件 ~10 行」——实现发现 `__zwClipboardStoreWrite` 在 part06 有正常页面路径内部消费点（`document.execCommand('copy'/'cut')` defaultPrevented → 剪贴板 store 桥），只删全局不闭包化则非测试模式 copy 桥静默失效（功能性回归）。按拍板精神（特权面全收口 + 正常路径不回归）将别名消费一并落到 part06，触点为同一闭包化机制的第二处消费点，非新增删除面。
- **测试模式信号 ②A-env**：runner（`tests/wpt-runner/src/main.rs` main 首语句自置——runner 进程内直用 `zero_webview::WebView`，无派生链）；browser→renderer 经 `Command` 默认继承父 env（`RendererHandle::spawn` 无 env_clear，零协议/browser 代码改动）；renderer env 剥离（`apply_early_if_enabled`）只删 `LD_PRELOAD`/`LD_LIBRARY_PATH`，信号存活。wire 不变（②A-ipc 显式拒绝项未采）。
- **装配点 4 处**（每次 shim 装载后跟随，幂等）：apps/renderer js_worker bootstrap（:1686-1693）+ context 重建（:2321-2327）；apps/browser tab_js_worker bootstrap（:510-517，持久 context 一次覆盖）；zero-webview `ensure_js_shim`（webview.rs:3089-3096，`!cfg!(test)` 门控——本 crate 单测直调钩子（pointer_lock 激活门 3 用例），cfg(test) 保留，集成测试二进制走真实分支）。
- **`__zwModalGateBypass` 记接受残余**：part01 自管旗标，counter-plugin 删除会被 shim 复写（置 non-writable 破 fixup 写入已否决）；仅自影响本页模态焦点语义，非跨站/存储/权限特权。与二轮复核推荐一致，② 切片内定。
- 内部 reader 钩子（`__zwPermissionState`/`__zwNotifyPermissionChange`）保留——permissions.query、fullscreen 权限路（part04:4966）与 request() 状态读实现自身消费。

## 新增常驻回归（9 用例）

| 用例 | 面 | 位置 |
|---|---|---|
| `counter_plugin_deletes_privileged_mutators_2c_prime` | 非测试模式 4 枚 typeof undefined + reader 保留 + 幂等双执行 | engine `js_dom_bridge_tests/privilege_guard.rs` |
| `counter_plugin_permissions_request_auto_grant_survives_closure_2c_prime` | request() 'prompt'→granted 经闭包别名 + query 同注册表 | 同上 |
| `counter_plugin_fullscreen_activation_gate_not_bypassable_2c_prime` | 激活门 TypeError+fullscreenerror、fullscreenElement null（页面无激活/权限自授入口） | 同上 |
| `counter_plugin_exec_command_copy_bridge_survives_closure_2c_prime` | copy preventDefault+setData → readText 往返（闭包别名） | 同上 |
| `shim_without_counter_plugin_keeps_hooks_test_mode_invariance` | 测试模式钩子原样（runner 契约） | 同上 |
| `counter_plugin_targets_exactly_the_four_mutators` | 删除面恰 4 枚（数组字面量计数，无扩面） | engine `counter_plugin.rs` |
| `counter_plugin_library_wiring_non_test_then_test_mode` | 库路径真实装配两阶段（集成测试非 cfg(test)：删除面 + request() 非回归 → env 置位后钩子保留） | `crates/webview/tests/privilege_counter_plugin.rs` |

engine 用例复刻宿主装配序（shim → counter_plugin_script ×2），与 js_worker/webview 装载点同构。

## 验证（全部 test-guard 包裹，systemd scope 内存封顶）

- engine 定向：`counter_plugin` 6P/0F（1 单元 + 5 V8）；钩子邻接面 `clipboard` 4P、`fullscreen` 4P、`permission` 3P、`exec_command` 2P；钩子引用既有用例 `test_document_cookie_semantics_c2k`/`test_input_show_picker_r3903`/`test_select_show_picker_r3905` 各 1P——闭包改动在测试模式（钩子在场）零行为变化。
- zero-webview：`--lib pointer_lock` 8P/0F + 新集成测试 1P/0F。
- zero-renderer 全量：365P/0F（js_worker 装配在环，无回归）。
- zero-browser bin 全量（--test-threads=1）：728P/3F 与 729P/2F（三轮），失败集 `default_actions_work_without_javascript`（切片 7 今日 clean main stash 归因在册）、`paste_and_go_blurs_and_clears_preedit`（切片 1 归因在册）、`gpu_compositor_path_dispatches_input_events_to_form_controls`（负载 flaky，1/3 轮现）——无本切片独有成员；仓库无任何测试引用四枚钩子（repo 全 grep），browser 测试不经 tab_js_worker 装配面受删删除影响。
- clippy：v8 五 crate（engine/webview/renderer/browser/wpt-runner）+ quickjs 矩阵（engine/dom/script-sandbox/webview/net/security/storage/protocol）`-D warnings` 零警告（初版 renderer 2 处 collapsible_if 已改 let-chains）；fmt clean。
- 初版单元测试假红一轮：`contains("__zwPermissionState")` 被 JS 头注释提及命中——改数组字面量计数锚定（`matches("'__zw").count() == 4`），非行为缺陷。

## 范围与残余

- runner 既有套件原样绿由 html-semantics 流全量自证（决策包验收条款）；runner main 自置 env 后 shim 装载路径与改前逐字节一致（plugin 跳过），唯一行为面为 part02/part06 闭包别名（同一函数对象引用，测试模式语义等价——engine 定向已钉）。
- 外部自动化消费者迁移契约：`ZW_RENDERER_AUTOMATION=1`（counter_plugin.rs 模块文档 + 决策包 ② 记录）；仓内消费者仅 runner（自置）。product-smoke/reftest 经 runner main 自置信号，基线不受影响（本切片无渲染/布局变更，product-smoke 未跑）。
- 库嵌入外部自动化宿主若依赖钩子需显式置 env（secure default：未置位即删除）。
- M4 收口时随决策面增量复测补本切片生产接收方面（headless CDP 页面 mutator 不可见 + 非 automation renderer 日志）。
