# Origin、PSL、同站与站点隔离

<!-- verified_commit: 1217776cc9cf179b93aca8b1557f0dab44b8e498 -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

读取跨源 DOM、Cookie 同站分类、托管平台后缀与进程分组问题。Origin 是协议/主机/有效端口三元组，Site 是协议/注册域；PSL 是本域网络与安全的共同输入，不注册成跨三领域共享组件。

## Gotchas
- Origin::parse 归一化 HTTP/HTTPS 默认端口，但拒绝无 host 或无已知默认端口的 URL；file、data、null 的失败不能用空 host 的普通 origin 替代。 (`crates/security/src/origin.rs`)
- Origin::is_secure 只检查 scheme=https，而 Service Worker 注册另外允许 HTTP localhost 和 loopback；两者安全上下文口径不能互换。 (`crates/security/src/origin.rs`, `crates/page-runtime/src/service_worker_registration.rs`)
- Site 保留 scheme 却不保留端口，同站可以不同源；即使 SiteIsolationManager::None 也只有同源 iframe 能访问父 DOM，不隔离不意味着放宽同源策略。 (`crates/security/src/site_isolation.rs`)
- PSL 默认规则是项目有限列表，不嵌入完整上游数据；扩充数据须核对许可证及 from_rules 注入，不能把当前列表当成完整互联网公共后缀。 (`crates/psl/src/lib.rs`, `crates/psl/src/default_rules.txt`)
- 当前 *.github.io 规则把 user.github.io 本身视为公共后缀，www.user.github.io 的注册域是自身；不要只复述“用户子域隔离”而漏掉更深子域行为。 (`crates/psl/src/tests.rs`, `crates/psl/src/default_rules.txt`)

## Architecture
- PSL 按最长匹配后缀选择规则，普通/例外在同长度时优先于通配，例外回退一标签；未知 TLD 使用末两标签，IP 和单标签主机单独处理。 (`crates/psl/src/lib.rs`, `crates/psl/src/tests.rs`)
- PSL 只做 ASCII 小写和尾点处理，不执行完整 IDNA/Punycode 转换；调用者应先经 URL 规范化，不能把文档中 Unicode 小写说明当当前实现能力。 (`crates/psl/src/lib.rs`)
- SiteIsolated 根据 Site 复用逻辑进程，StrictOriginIsolated 根据完整 Origin 复用；这些是库内映射与策略判定，不单独证明 browser 已实现所有跨站 iframe 真进程分配。 (`crates/security/src/site_isolation.rs`, `crates/security/src/tests/site_isolation_coverage.rs`)
- browser 存储 origin 以已提交 renderer 导航映射为准，不能为兼容未知 URL 绕过提交前拒绝；该授权边界比页面自己序列化的 origin 更强。 (`apps/browser/src/process_backend.rs`)
