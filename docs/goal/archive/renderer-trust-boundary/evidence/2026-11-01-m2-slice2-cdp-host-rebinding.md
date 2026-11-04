# M2 切片 2：CDP 端口 DNS rebinding 防护（Host 头本机校验）

**日期**：2026-11-01
**HEAD（基线）**：`0fef0b3f8`（切片 1 提交后）
**范围**：`apps/browser` headless（本流域）；协议/自动化契约不变。

## 威胁与事实

CDP server（`--remote-debugging-port`）绑 127.0.0.1，默认无 token（`HeadlessSecurityConfig::default`，headless/mod.rs:125；`with_security` 生产零调用）。本机其他进程直连是既有产品形态（Chrome 同款）。真实缺口是 **DNS rebinding**：浏览器页面里 `ws://evil.example:9222`（解析到 127.0.0.1）发起的连接 Host 头为攻击者域名，修复前 `verify_origin` 空白名单=全放行，页面可借本机自动化通道全量驱动浏览器（`Runtime.evaluate` 等）。

## 修复（Chromium 同款策略）

- `HeadlessSecurityConfig::verify_host`（security.rs）：Host 头必须为 IP 字面量（v4/v6，可带端口，IPv6 按 RFC 3986 带方括号）或 `localhost`；缺失放行（裸 HTTP/1.0 客户端不发 Host，rebinding 必带）；畸形 fail closed。
- discovery.rs：`extract_host_header`（字段名大小写不敏感，跳过请求行防路径误取）。
- mod.rs：HTTP 发现与 WS 升级两分支先过 Host 校验；HTTP 拒绝回 `403 Forbidden`，WS 拒绝直接关闭。
- Chromium 参照：https://chromium.googlesource.com/chromium/src/+/refs/heads/main/net/server/http_server/

**不破坏面核对**：生产脚本消费者全部为 `127.0.0.1`/`localhost` Host 形态——`scripts/browser-launch-smoke.sh`（curl http://127.0.0.1）、`scripts/run-oracle-capture.sh`（localhost）、`scripts/compositor-animation-comp04-probe.sh`（node driver ws://127.0.0.1）、devtools frontend 同源自连。apps/webdriver 不经 CDP TCP 端口（直接 RendererHandle 管道）。

**待决（用户决策）**：默认强制 token 会破坏既有无 token 自动化客户端（=目标边界禁止的"关闭自动化能力"），不自主实施；已记 master 待决表。

## 验证

- 新增常驻回归（headless/tests.rs）：
  - `test_extract_host_header`（提取矩阵：常规/小写/请求行误取/缺失/空）
  - `test_verify_host_rejects_non_local_names`（放行矩阵 + 域名/畸形拒绝矩阵）
  - `test_http_discovery_rejects_non_local_host_header`（真实 socket：`Host: evil.example:9222` → `HTTP/1.1 403 Forbidden`；`Host: 127.0.0.1:PORT` → `200 OK` 不回归）
- `cargo test -p zero-browser --bin zero-browser`（test-guard，1800s）→ 717P/3F、716P/4F（两轮失败集漂移）
- **失败归因（干净 main `0fef0b3f8` stash 后连跑两轮）**：3F/5F 失败集同样漂移，含相同成员——
  - 确定性既有红：`tests::paste_and_go_blurs_and_clears_preedit`、`tests::form_fixture_complete_multiprocess_semantics`（两环境每轮必现）
  - 负载 flaky：`service_worker_owner_tests::committed_navigation_observes_top_level_service_worker_client`、`tests::page_context_menu_save_as_writes_page_html_to_download_dir`、`tests::default_actions_work_without_javascript`（干净树与本切片树均偶现，无本切片独有成员）
- `cargo clippy -p zero-browser --bins`：17 条 dead_code 警告均不在本切片文件（headless/security.rs、discovery.rs、mod.rs Host 路径零命中）。

## 清理

stash 归因两轮已 pop，工作树恢复本切片状态；无残留进程。
