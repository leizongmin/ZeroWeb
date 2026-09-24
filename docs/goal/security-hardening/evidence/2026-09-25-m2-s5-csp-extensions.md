# M2-s5 — connect-src 阻止族 + 外链 css 应用链归因（全绿 62→65）

**日期**: 2026-09-25
**corpus**: `make testharness-csp`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-25-m2-s5-csp-final.json](2026-09-25-m2-s5-csp-final.json)
**前序**: [M2-s4](2026-09-25-m2-s4-csp-extensions.md)（62/415，27.9%）

## 结果

| 指标 | M1 基线 | M2-s4 | M2-s5 | Δ |
|---|---|---|---|---|
| 全绿用例 | 30/415 | 62/415 | **65/415** | **+3 零丢失** |
| subtests Pass | 91/557 = 16.3% | 159/570 = 27.9% | **165/570 = 28.9%** | +6 |

新增全绿：connect-src 阻止族 3（eventsource-blocked / syncxmlhttprequest-blocked /
xmlhttprequest-blocked）。fetch/beacon blocked 变体仍红——shim 契约 divergence（host
错误 wire → resolve(ok:false) Response 而非 promise reject；记账 M2-s6）。

## 本切片落地（kill-switch 延续，default off）

1. **connect-src 检查点**（zero-security `check_connect` + csp.rs `has_directive`
   pub 只读面）：fetch/XHR/beacon/eventsource 统一桥（shim fetch → `__zw_fetch`）——
   导航面（r115iframe:）不受约束；指令口径 connect-src 显式上报 "connect-src"（connect
   面无 -elem 子分）、default-src 回退如实。
2. **共享 violation 队列**（`pending_csp_connect_violations: Arc<Mutex<Vec<_>>>`）：
   `__zw_fetch` 回调为 'static 闭包（同步原生栈，不能重入执行 JS）——被阻止请求推
   队列 + 返回 `__zw_fetch_error:`（fetch resolve(ok:false) / XHR error 语义）；
   run 尾与 style 面合并 document 站派发。

## 归因记账（跨域，不越界）

- **外链 stylesheet ID 选择器应用缺口**（stylenonce-blocked 期望 allowed.css green
  实得黑）：webview 层实证——default 配置（无 CSP）同样黑；external_css 27 字节正确
  取到并传 `load_html(css)`、styleSheets.length=1 但规则不生效——**pipeline/style 面
  预存缺口，属渲染流域 crates（css-parser/style-system）**，记账待协调不越界硬改。
- **fetch blocked 面契约**：shim host 错误 → resolve(ok:false)（非 reject）——
  fetch-keepalive/beacon blocked 族期望 reject 面，shim 语义 divergence 记账。

## 剩余缺口（M2-s6+）

| 簇 | 形态 | 切片 |
|---|---|---|
| script-src-attr | onclick 内联事件处理器（targeting block2/4） | M2-s6 |
| 运行时 img | createElement('img') src-set 钩子（securitypolicyviolation img 4 案） | M2-s6 |
| eval 检查点 | blockeduri-eval（is_eval_allowed 钩 sandbox 面） | M2-s6/s7 |
| fetch blocked 契约 | shim resolve(ok:false) vs 上游 reject 语义 | 记账（shim 面） |
| report-uri / Reporting API | report 端点族 | 记账（infra） |
