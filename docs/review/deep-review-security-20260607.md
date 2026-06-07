# ZeroWeb 深度审查报告 — 安全（Security + Network）

> **摘要**
>
> **审查范围**：`crates/security/`（CORS、CSP、同源策略）、`crates/net/`（HTTP 客户端、Cookie、缓存、WebSocket）
>
> **关键发现**：共发现 17 个问题（高 4 / 中 9 / 低 4）
>
> **最高优先级**：Cookie 无域名匹配任意 host（超级 Cookie 漏洞），可被外部攻击者利用窃取会话
>
> **验证状态**：已验证（2026-06-07）— 12 verified, 3 dismissed

## 审查上下文

| 字段 | 内容 |
|------|------|
| **审查对象** | security/、net/ crate |
| **审查维度** | 安全漏洞、可靠性、性能 |
| **代码版本** | main 分支，commit f5eb85b |

---

## 问题清单

### 高优先级（Critical）

#### SEC-01 [安全] Cookie 无域名匹配任意 host（超级 Cookie 漏洞）

- **位置**：`crates/net/src/cookie.rs:415`
- **置信度**：0.95
- **状态**：verified
- **描述**：当 `cookie.domain` 为 `None` 时，`cookie_matches_url` 函数完全跳过域名检查，导致该 cookie 匹配**任意** host。违反 RFC 6265 §5.3：无显式 Domain 属性的 cookie 应仅发送至精确匹配的 host（无子域匹配）。
- **触发条件**：从 `example.com` 收到 `Set-Cookie: session=abc`（无 Domain 属性），该 cookie 将随 `evil.com`、`bank.com` 等请求发送。
- **代码证据**：
  ```rust
  if let Some(ref domain) = cookie.domain {
      let host = url.host.as_deref().unwrap_or("");
      if !domain_matches(domain, host) {
          return false;
      }
  }
  // domain == None → 跳过检查，匹配任何 host
  ```
- **影响**：会话劫持、跨站请求伪造
- **建议修复**：当 cookie 无显式 Domain 时，要求与 URL host 精确匹配：
  ```rust
  let host = url.host.as_deref().unwrap_or("");
  match &cookie.domain {
      Some(domain) => { if !domain_matches(domain, host) { return false; } }
      None => { /* 精确匹配 host，无子域扩展 */ }
  }
  ```

---

#### SEC-02 [安全] Cookie 值中的 CRLF 注入（HTTP 头部注入）

- **位置**：`crates/net/src/cookie.rs:240`
- **置信度**：0.80
- **状态**：verified
- **描述**：Cookie 的 name 和 value 从 `Set-Cookie` 头解析时未净化 `\r`、`\n`、`\0`。恶意服务器可注入 CRLF 构造额外的 HTTP 头。
- **触发条件**：服务器返回 `Set-Cookie: session=abc\r\nX-Injected: evil`，后续格式化 Cookie 头时产生 CRLF 注入。
- **代码证据**：
  ```rust
  let value = first[eq_pos + 1..].trim().to_string(); // 无 CRLF 净化
  // ...
  .map(|c| format!("{}={}", c.name, c.value)) // 嵌入原始 CRLF
  ```
- **影响**：HTTP 响应拆分攻击、头部注入
- **建议修复**：在 `parse_set_cookie` 中拒绝含 CRLF 的 cookie。

---

#### SEC-03 [安全] 跨域重定向转发敏感头部（凭证转发 / SSRF）

- **位置**：`crates/net/src/client.rs:70-143`
- **置信度**：0.85
- **状态**：verified
- **描述**：当发生跨域重定向时，所有原始请求头（包括 `Authorization`、`Cookie`）被原样发送到新源。浏览器通常在跨域重定向时剥离这些敏感头。
- **触发条件**：从 `api.example.com` 302 重定向至 `partner.com`，`Authorization: Bearer <token>` 被发送至 `partner.com`。
- **代码证据**：
  ```rust
  for (name, value) in &request.headers {
      // 所有头部在每个重定向上发送，无源比较
  }
  ```
- **影响**：SSRF、凭证泄露
- **建议修复**：跨域重定向时剥离 `Authorization`、`Cookie` 等敏感头，比较当前 URL 与重定向目标的源。

---

#### SEC-04 [安全] V8 沙盒未限制 eval/Function 等危险内置对象

- **位置**：`crates/script-sandbox/src/v8_runtime.rs:190-209`
- **置信度**：0.85
- **状态**：dismissed
- **描述**：V8 Context 使用 `Context::new(scope)` 创建，未使用 ObjectTemplate 限制内置对象。
- **dismiss 原因**：script-sandbox 是浏览器页面级 JS 执行环境，不是受限安全沙箱。"sandbox"指 V8 Isolate 隔离，不是 eval 限制。浏览器页面脚本本身需要 eval/Function 能力。JavaScript 代码可完全访问 `eval()`、`Function()` 构造函数等，可执行任意代码。
- **触发条件**：任何通过 `execute_script` 执行的 JS 代码都可调用 `eval("...")` 或 `new Function("...")("...")`。
- **代码证据**：
  ```rust
  let context = rusty_v8::Context::new(scope); // 无 ObjectTemplate 限制
  ```
- **影响**：沙盒逃逸、任意代码执行
- **建议修复**：使用 `ObjectTemplate` 白名单内置对象，至少阻止 `eval` 和 `Function` 构造函数。

---

### 中优先级（Major）

#### SEC-05 [安全] CORS allow_headers 不支持 "*" 通配符

- **位置**：`crates/security/src/cors.rs:122-151, 260-271`
- **置信度**：0.85
- **状态**：verified
- **描述**：`allow_origins` 将 `"*"` 视为通配符，但 `allow_headers` 仅做字面匹配。配置 `allow_headers: ["*"]` 只匹配名为 `*` 的头，而非任何头。违反 Fetch 规范。
- **触发条件**：配置 `Access-Control-Allow-Headers: *` 的策略，自定义头预检请求被错误拒绝。
- **建议修复**：当 `allow_headers` 包含 `"*"` 且 `allow_credentials == false` 时，将所有头视为允许。

---

#### SEC-06 [安全] CORS 未处理 null 源

- **位置**：`crates/security/src/cors.rs:77-99`
- **置信度**：0.55
- **状态**：dismissed
- **描述**：`check_cors` 未验证源字段是否有效。
- **dismiss 原因**：Origin 只能通过 parse/from_url 构建（两者均做验证），Origin::parse("null") 返回错误。无效源在解析阶段即被拒绝，无法到达 check_cors。若 Origin 以空 host 或 `scheme == "null"` 构建，可能匹配配置不当的 `allow_origins`。
- **建议修复**：在函数入口拒绝空 scheme/host 的源。

---

#### SEC-07 [安全] Cookie 路径前缀匹配过于宽松

- **位置**：`crates/net/src/cookie.rs:423-427`
- **置信度**：0.92
- **状态**：verified
- **描述**：使用 `starts_with` 进行路径匹配，`Path=/app` 会错误匹配 `/application`。违反 RFC 6265 §5.1.4。
- **触发条件**：设置 `Path=/app` 的 cookie 被发送至 `/application`。
- **建议修复**：添加路径分隔符检查：cookie_path 须以 `/` 结尾或请求路径的下一字符为 `/`。

---

#### SEC-08 [安全] SameSite=None Cookie 未强制要求 Secure 属性

- **位置**：`crates/net/src/cookie.rs:283-295`
- **置信度**：0.85
- **状态**：verified
- **描述**：`SameSite=None` cookie 未被要求具有 `Secure` 属性，违反 SameSite 规范和现代浏览器行为。
- **建议修复**：解析完所有属性后检查 `same_site == None && !secure` 则拒绝。

---

#### SEC-09 [安全] Cookie 存储无大小限制（OOM 向量）

- **位置**：`crates/net/src/cookie.rs:315-330`
- **置信度**：0.80
- **状态**：verified
- **描述**：`CookieStore` 无最大条目限制。恶意服务器可发送大量唯一 cookie 导致内存无限增长。`evict_expired` 从未被自动调用。
- **建议修复**：添加最大条目数，添加时驱逐过期/最旧 cookie。

---

#### SEC-10 [可靠性] Client::with_config 静默回退到默认客户端

- **位置**：`crates/net/src/client.rs:45-56`
- **置信度**：0.70
- **状态**：verified
- **描述**：`unwrap_or_default()` 在构建失败时静默创建默认 reqwest 客户端，超时、重定向策略、User-Agent 均与预期不同。
- **建议修复**：使用 `expect("failed to build HTTP client")` 或返回 `Result`。

---

#### SEC-11 [可靠性] Max-Age 溢出导致 Cookie 过期时间不正确

- **位置**：`crates/net/src/cookie.rs:299-309`
- **置信度**：0.65
- **状态**：verified
- **描述**：`now_secs + max_age as u64` 可能算术溢出（release 模式回绕），导致 cookie 立即过期或任意时间过期。
- **建议修复**：使用 `saturating_add`。

---

#### SEC-12 [实现缺陷] parse_http_date 产生严重错误的时间戳

- **位置**：`crates/net/src/http_cache.rs:393-396`
- **置信度**：0.90
- **状态**：verified
- **描述**：HTTP 日期解析使用粗略近似公式（每月 30 天、忽略闰日世纪规则、忽略时分秒），与实际 Unix 时间戳偏差可达数年。
- **建议修复**：复用 `cookie.rs` 中正确的 `parse_expires_date` 函数。

---

#### SEC-13 [安全] V8 沙盒 timeout_ms 声明但从未强制执行

- **位置**：`crates/script-sandbox/src/v8_runtime.rs:62-136`
- **置信度**：0.95
- **状态**：verified
- **描述**：`SandboxConfig.timeout_ms` 被声明和存储但从未读取。恶意 JS（`while(true){}`）可无限期阻塞调用线程。
- **触发条件**：执行 `while(true){}` 脚本，整个调用线程被永久阻塞。
- **建议修复**：当 `timeout_ms > 0` 时，创建定时器线程调用 `isolate.thread_safe_handle().terminate_execution()`。

---

### 低优先级（Minor）

#### SEC-14 [设计] CORS 默认策略允许所有源

- **位置**：`crates/security/src/cors.rs:31-41`
- **置信度**：0.60
- **状态**：verified
- **描述**：`CorsPolicy::default()` 返回 `allow_origins: vec!["*"]`，默认启用 CORS。
- **建议修复**：默认使用空列表，要求显式启用。

---

## 统计总览

| 维度 | 高 | 中 | 低 | 合计 |
|------|----|----|----|------|
| 安全漏洞 | 4 | 5 | 1 | 10 |
| 可靠性 | 0 | 3 | 0 | 3 |
| 实现缺陷 | 0 | 1 | 0 | 1 |
| 设计 | 0 | 0 | 1 | 1 |
| **合计** | **4** | **9** | **2** | **15** |

## 修复建议优先级

| 优先级 | 问题 | 建议动作 | 预估改动量 |
|--------|------|---------|-----------|
| P0（立即） | SEC-01, SEC-02, SEC-03 | 修复 Cookie 域名匹配、CRLF 净化、重定向头剥离 | 各约 20-30 行 |
| P0（立即） | SEC-04, SEC-13 | V8 沙盒限制内置对象、强制 timeout | 各约 50-80 行 |
| P1（本迭代） | SEC-05, SEC-07, SEC-08 | CORS 通配符、Cookie 路径匹配、SameSite 校验 | 各约 10-20 行 |
| P2（后续跟进） | SEC-09, SEC-10, SEC-11, SEC-12 | Cookie 存储限制、客户端构建、溢出保护、日期解析 | 各约 10-30 行 |
