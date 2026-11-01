# M2 切片 5：多进程 HSTS 注册（fetch_proxy document 响应落 per-tab SecurityContext）

**日期**：2026-11-01
**HEAD（基线）**：`b4f1b9079`（切片 4 提交后）
**范围**：`apps/browser/src/fetch_proxy.rs`（单文件）。

## 缺口

M1 矩阵在册「多进程 HSTS 不注册」（兼容缺口，非越权）：单进程导航路在 `webview::note_hsts_response`（webview.rs:2571，`mixed_content_enforcement` 门内）注册 STS 响应头；多进程模式页面网络经 browser `TabFetchProxy` 代理，响应头此前不落 HSTS 表——`check_resource_url` 的 HSTS 升级阶段只剩内置预加载表，站点下发的 `Strict-Transport-Security` 被丢弃。

## 修复

- `TabFetchProxy::note_hsts_response`（镜像 webview.rs 语义）：最终 URL 为 HTTPS 且携带 `Strict-Transport-Security` 头（大小写不敏感）时，按最终 URL 的 host 登记 per-tab `SecurityContext`（`register_hsts` → `HstsStore::register_from_header`），返回 true 时 `tracing::info!` 记录。
- 调用点 `drain()` 响应完成臂：`strip_spoofed_internal_headers` 之后、仅 `pending.resource_type == "document"`（导航身份由 renderer `AsyncFetchHost::fetch_document` 显式头声明，fetch_priority.rs P11 契约）时调用——对齐单进程仅导航响应注册的口径；子资源/流式图片路径不落表。
- 门语义差异记录：webview 侧有 `mixed_content_enforcement` kill-switch 门；browser 侧消费面 `check_resource_url` 无条件执行 HSTS 升级（无独立开关），故仅保留 https-only 门（与消费面一致，无「注册了不生效」或「生效了不注册」的错位）。
- HSTS 粒度观察（既有语义，本切片未改）：browser 侧 per-tab `SecurityContext` 各持独立 `HstsStore`（与单进程 per-WebView `security_context` 粒度一致）；真浏览器为全局存储——记为后续对齐项，不影响本缺口收口。

## 常驻回归（2 个）

- `hsts_document_response_registers_per_tab_security_context`：STS 头注册后 `check_resource_url` 强制升级（含 includeSubDomains 子域）；per-tab 隔离（未注册 tab 不升级）；非 HTTPS/无头/坏头不注册（对齐 `crates/webview/src/tests/mixed_content_gate.rs` 口径；http 页面源排除混合内容噪声）。
- `drain_document_response_does_not_register_hsts_over_http`：真实回环 HTTP 服务端发 STS 头的 document 响应端到端流经 drain 登记点，https-only 门拦截不注册；断言钉 `status==200` 防 Err 完成路径假绿。

## 验证命令与结果（test-guard 包裹）

- `cargo test -p zero-browser --bin zero-browser hsts` → 2 passed / 0 failed
- `cargo clippy -p zero-browser --bins` → 零警告
- 全量 `cargo test -p zero-browser --bin zero-browser` → 722P/2F（`paste_and_go_blurs_and_clears_preedit` 切片 1–4 已证既有确定性红；`default_actions_work_without_javascript` 切片 2/4 已归因负载 flaky——失败集与切片 4 完全一致，无本切片独有成员）

## 清理

无残留进程/目录。
