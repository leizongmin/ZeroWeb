# 来源、策略与隔离

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖 origin/site 身份、PSL、安全资源检查、sandbox、权限与 COOP/COEP。OS 进程沙箱、IPC 输入格式和页面 JS 通用绑定分别归 runtime/scripting；这里保留影响网络与存储授权的具体策略。

## Gotchas
- 安全库“存在策略 API”与“产品路径实际强制执行”要分别取证；SecurityContext 资源 URL 入口不会自动执行 CSP/CORS/COOP/COEP。 (`crates/security/src/context.rs`, `crates/security/src/cors.rs`, `crates/security/src/coep.rs`)

## Architecture
- Origin、Site 与 Service Worker 注册安全上下文各有不同口径；权限按 origin 隔离、站点分组按 PSL、SW 另允许 loopback，不应抽成一个仅看主机名的安全判断。 (`crates/security/src/origin.rs`, `crates/security/src/site_isolation.rs`, `crates/security/src/permission.rs`, `crates/page-runtime/src/service_worker_registration.rs`)

## Child Knowledge Nodes
- [Origin、PSL、同站与站点隔离](origin-and-site.md) — 默认端口、有限后缀数据与同站/同源差异。
- [CORS、CSP、混合内容与 HSTS](resource-gates.md) — 实际执行检查点、政策装配与响应升级。
- [Sandbox、权限、COOP 与 COEP](isolation-and-permissions.md) — 受限能力、权限键、opener 和隔离判定的实现边界。
