# WPT 95% · 07 Fetch 与网络安全策略

**状态** Active\
**立项** 2026-11-09，按用户要求扩展到 95% 冲刺，支持 Rally 无人值守推进。\
**父目标** [ZeroWeb](zero-web.md)\
**控制面** [master.md](wptfyi-network/master.md)\
**共同计分与运行规则** [95% 冲刺计划](../compat/wptfyi-95-plan.md)

## 基线与目标

11 月 7 日 [全量快照](../compat/wptfyi/coverage-2026-11-07.md) 已通过 5,562 / 44,598 项，缺口 39,036 项。`referrer-policy/gen` 缺 7,378 项，`fetch/api` 缺 5,987 项，`mimesniff/mime-types` 缺 2,541 项。复用 [网络测试基础设施](archive/runner-net-infra.md)、[网络 API 归档](archive/net-api-compat-v3.md) 和 [安全归档](archive/security-hardening.md)。

同口径净增 **33,000** 项，达到至少 **38,562** 项通过。固定 revision 的净增与当前周跑的上游漂移分别报告。

## 范围与协作

计分目录为 `fetch/`、`referrer-policy/`、`content-security-policy/`、`mimesniff/`、`websockets/`、`mixed-content/`、`xhr/`、`upgrade-insecure-requests/`、`cookies/`、`preload/`、`cors/`、`subresource-integrity/`、`resource-timing/`、`server-timing/`、`permissions-policy/`、`connection-allowlist/`、`secure-contexts/`。

负责 WPT 多源 HTTP(S)、证书和动态响应夹具。各协议的安全断言通过真实请求和浏览器策略验证。Service Workers 的计分归 08，HTML 子树归 05；浏览器自动化会话由 10 维护。与 zero-web 流协调网络代理和 IPC 共享写面。 共享文件按 [Rally 运行规则](../rally/run-rules.md) 分配归属。等待共享修复时，继续独立语料导入、对账和失败分类。

## 里程碑与完成条件

- **DC-1 · 基线与试跑**。用固定语料跑 `testharness-net-api`、`testharness-csp` 和 `testharness-mixed-content` 的代表性样本。补齐多源、HTTPS、重定向、动态响应和跨源预检的夹具对照，验证允许与拒绝各有真实网络结果，并核对生成路径可入账。 归档固定 revision、当前 commit、预期与实际执行路径，以及按路径和变体的差额。
- **DC-2 · 按簇收敛**。先按 Fetch API、Referrer Policy 生成语料、CSP 和 MIME Sniffing 分片，再补 Mixed Content、XHR、Cookie、WebSocket 和资源加载策略。逐轮验证重定向链、凭据、隔离与安全上下文。 修复随附常驻 WPT 或等价回归。
- **DC-3 · 全量对账**。接入周跑并验收净增至少 33,000 项，邻域无可归因回退。完成守卫下的构建、测试与 clippy，保存有效分片和全量结果，再归档。

每轮更新控制面与证据，最后一行按 [共同计划](../compat/wptfyi-95-plan.md#无人值守运行) 输出 `DONE`、`CONTINUE: <下一步>` 或 `BLOCK: <原因>`。
