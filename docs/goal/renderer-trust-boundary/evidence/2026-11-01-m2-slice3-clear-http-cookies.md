# M2 切片 3：ClearHttpCookies——clear/cookies 与 renderer 本地 jar 同步

**日期**：2026-11-01
**HEAD（基线）**：`c6a81d233`（切片 2 提交后）
**范围**：protocol 追加末尾变体 + renderer handler + browser 广播（本流域文件）。

## 缺口（M1 矩阵在册项 2 的收敛）

`zero://settings/clear/cookies`（app.rs `apply_settings_clear_cookies`）只清 browser 进程权威 jar。renderer 子进程本地 jar（`ipc_fetch.rs:65` 经 FetchResponse Set-Cookie 写入，`ws_handshake_cookie` 消费）不在清理面——已清除的 cookie 继续附着 renderer 侧 WS 握手（隐私残留；app.rs 原注释自认「跨进程清理需要新增 IPC 消息，属后续切片」）。

## 修复

- **协议**：`IpcMessageKind::ClearHttpCookies`（无参单元变体）追加枚举末尾（append-only 纪律，bincode 位置编码兼容）。
- **renderer**：`handle_clear_http_cookies` 清 `zero_net::shared_cookie_store()` 本进程 jar。
- **browser**：`ProcessTabBackend::broadcast_to_renderers`（无映射 tab 静默跳过）→ `TabManager::clear_renderer_http_cookies` → `apply_settings_clear_cookies` 接线；单进程路（process_backend=None）自然 no-op。

## 同片判定：Cookie 头伪造（矩阵在册项 ②）非缺口，不修

核验：JS 层 R3221（js_dom_shim/part02.js:3971-3976）已按 fetch spec forbidden-header-name 挡 `cookie`/`host`/`origin` 等（`_headersToWire` 出口过滤，XHR 同通道）；browser 侧 HTTP 无 jar 注入面（t8k-delta F1 摘除，FIXME 注释在案）。被攻破 renderer 若已越过 JS 沙箱即拥有进程网络栈（`default_net_ws_host` 直连 TCP），browser 侧再剥 FetchRequest 的 Cookie 头对该威胁模型无边界增益。矩阵更正：② 由「明显缺口」降级为「spec 已在 JS 边界强制；进程级攻击者无 IPC 面可护」。

## 常驻回归（三处）

- `zero-protocol`：`test_serialize_deserialize_clear_http_cookies`（roundtrip 判别值保持）
- `zero-browser`：`clear_cookies_broadcast_reaches_mapped_renderers`（两映射 tab 各收一条，TEST_RENDERER_OUTBOUND 桩）
- `zero-renderer`：`clear_http_cookies_empties_renderer_local_jar`（预置 jar 条目 → handler → 全空）

## 验证命令与结果（test-guard 包裹）

- `cargo test -p zero-protocol` → 333P/0F
- `cargo test -p zero-renderer` → 363P/0F
- `cargo test -p zero-browser --bin zero-browser` → 718P/3F（3F 归因：`paste_and_go_blurs_and_clears_preedit` 切片 1/2 已证既有红；`t28p37_menu_open_with_loading_keeps_chrome_animating`、`default_actions_work_without_javascript` 为切片 2 干净树对照轮已出现的负载 flaky，无本切片独有成员）
- `cargo clippy -p zero-protocol -p zero-renderer -p zero-browser`：本切片改动文件零警告

## 清理

无残留进程/目录。
