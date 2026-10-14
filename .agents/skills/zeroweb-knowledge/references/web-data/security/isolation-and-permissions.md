# Sandbox、权限、COOP 与 COEP

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取 iframe 受限能力、权限 origin 键、opener 与跨源隔离问题。本文说明 security 库当前判定与实现缺口，不把策略对象、页面 JS 反射或 OS 沙箱混成一个机制。

## Gotchas
- sandbox token 按 ASCII 大小写不敏感解析，未知 token 忽略；存在但为空的 sandbox 对应最严格限制，不能当作“未设置 sandbox”。 (`crates/security/src/sandbox.rs`)
- 无 allow-same-origin 时返回 SandboxOrigin::Opaque，但该变体没有唯一身份字段；不能依据注释中“唯一源”就把多个 Opaque 的派生 Eq 当作规范源身份。 (`crates/security/src/sandbox.rs`)
- 顶层导航允许条件是 allow-top-navigation，或用户激活加 allow-top-navigation-by-user-activation；弹窗检查只看 allow-popups，不能复用导航检查替代弹窗限制。 (`crates/security/src/sandbox.rs`)
- PermissionManager 未记录返回 Prompt，revoke 删除条目并回到 Prompt 而不是 Denied；其内存状态不自动产生系统权限请求或持久化文件。 (`crates/security/src/permission.rs`)
- 权限键始终序列化 scheme://host:port，含默认端口，按完整 origin 而不是 eTLD+1 隔离；不能拿省略端口的网络 origin 字符串直接绕过此键构造。 (`crates/security/src/permission.rs`, `crates/security/src/origin.rs`)

## Architecture
- evaluate_coop 是纯决策函数，同源分支直接 Allowed，跨源按双方策略组合决定；返回 Blocked 只是要求断开 opener，不会自己修改 browser 的窗口或浏览上下文组。 (`crates/security/src/coop.rs`)
- evaluate_coep 同源或已通过 CORS 直接放行；其 RequireCorp/Credentialless 分支按 CorpStatus 枚举判定，不实现实际请求删凭证，也不能当作完整 CORP 规范覆盖。 (`crates/security/src/coep.rs`)
- 当前 parse_corp 只识别 same-origin/cross-origin，未知值含 same-site 都落 NoPolicy；排查跨站资源时必须区分“没解析支持”和“真实响应没有 CORP”。 (`crates/security/src/coep.rs`)
- is_cross_origin_isolated 只接受 SameOrigin COOP 加 RequireCorp COEP，其他组合即便存在策略枚举也返回 false；该布尔值不能证明页面 SharedArrayBuffer 或浏览上下文已完成真实隔离。 (`crates/security/src/lib.rs`)
- SecurityContext 不持有 iframe sandbox、PermissionManager、COOP 或 COEP 状态，这些调用要逐产品路径核对；security_pipeline 的直接组合测试不等于所有运行路径都接线。 (`crates/security/src/context.rs`, `tests/integration/src/security_pipeline.rs`)
