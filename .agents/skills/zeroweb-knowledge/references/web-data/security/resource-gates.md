# CORS、CSP、混合内容与 HSTS

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取资源被允许/阻止、CSP 违规事件、HSTS 升级和页面加载安全接线问题。SecurityContext 是组合状态与检查 API，不是所有策略都自动执行的总入口。

## Gotchas
- check_resource_url 实际只依次执行 HSTS 与混合内容；不会调用文档 CSP、CORS、COOP/COEP，尽管文件开头摘要包含 CSP，调用者仍须接独立检查点。 (`crates/security/src/context.rs`)
- 页面 origin 不存在或不是 HTTPS 时混合内容阶段放行；set_page_origin 解析失败得到 None，不能在坏 URL 后沿用“安全上下文已建立”的假设。 (`crates/security/src/context.rs`)
- HTTP scheme 检测大小写不敏感，script/style/connect/font 等被阻止，img/audio/video/media 尝试 HTTPS 升级；相对 URL 本身不会命中，检查前必须核对解析位置。 (`crates/security/src/mixed_content.rs`)
- CSP 多政策不是任一放行即通过，而是任一阻止即违规；set_document_csp 整组替换，导航到无政策文档需清除旧政策，不能泄漏上一文档约束。 (`crates/security/src/context.rs`)
- CORS 凭证请求不能用 wildcard origin，非简单 Content-Type 必须被允许；库内策略判断和实际浏览器预检网络流程不是同一证据层。 (`crates/security/src/cors.rs`)

## Architecture
- WebView 在文档加载处装配 meta CSP，并在 script、style、image、connect、eval 和 script attr 路径调用对应检查点；只检查 fetch_proxy 的 URL gate 不能覆盖这些执行时约束。 (`crates/webview/src/webview.rs`, `crates/security/src/context.rs`)
- CSP 元素面违规报告使用 effective_*_directive，不应统一改回裸 script-src/style-src；nonce/hash 的允许结果和事件 blockedURI/originalPolicy 都有页面断言。 (`crates/security/src/csp.rs`, `crates/webview/src/tests/csp_gate.rs`)
- HstsStore 自身 register_from_header 不校验传输安全，WebView note_hsts_response 才限制 HTTPS 响应注册；直接调用底座登记函数不能证明不可信 HTTP 头已被拒。 (`crates/security/src/hsts.rs`, `crates/webview/src/tests/mixed_content_gate.rs`)
- HSTS max-age=0 删除策略，includeSubDomains 传播到父域匹配；当前 store 是内存 HashMap，模块摘要的“持久化”不能替代实际磁盘 owner 证据。 (`crates/security/src/hsts.rs`)

## Decisions
- CSP source summary 在首次检查时用 OnceLock 派生且保留源值下标，避免把热路径成本挪到只解析不检查的调用；修改公开 directives 后也必须考虑派生缓存是否失效。 (`crates/security/src/csp.rs`)

## Patterns
- security_pipeline 集成测试直接组合安全库 API，WebView gate 测试验证页面检查点和违规事件；前者不能代替真实 renderer 网络、CORS 预检或全产品安全验收。 (`tests/integration/src/security_pipeline.rs`, `crates/webview/src/tests/csp_gate.rs`, `crates/webview/src/tests/mixed_content_gate.rs`)
