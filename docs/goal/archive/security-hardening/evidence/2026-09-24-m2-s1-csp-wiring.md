# M2-s1 — CSP 接线骨架（meta 装配 + script 检查点 + violation 事件）

**日期**: 2026-09-24
**corpus**: `make testharness-csp`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-24-m2-s1-csp-final.json](2026-09-24-m2-s1-csp-final.json)
**基线**: [2026-09-24-m1-security-wpt-baseline.md](2026-09-24-m1-security-wpt-baseline.md)

## 结果

| 指标 | M1 基线 | M2-s1 | Δ |
|---|---|---|---|
| 全绿用例 | 30/415 | **33/415** | **+3 零丢失** |
| subtests Pass | 91/557 = 16.3% | **109/557 = 19.6%** | **+18** |
| Fail / Timeout | 332 / 134 | 316 / 132 | −16 / −2 |

新全绿案：`securitypolicyviolation/constructor-required-fields`（14/14，SPV 构造器面）、
`script-src/scripthash-changed-2`、`script-src/scriptnonce-changed-2`（blocked 语义
正确化——被阻止脚本不再执行 + violation 派发）。

## 本切片落地（kill-switch `WebViewConfig::csp_enforcement`，default **off**）

1. **SecurityPolicyViolationEvent 构造器**（engine dom_bindings/event.rs）：type 必需
   （TypeError）+ 全字段可选缺省（字符串 ""、数值 0、disposition "enforce"）+
   instanceof Event（inherit Event 模板）。WPT constructor-required-fields 14/14 对齐。
2. **meta CSP 装配**（engine extract.rs `extract_meta_csp_policies`）：head 域限定 +
   http-equiv 大小写不敏感 + 多政策并集。FIXME：spec meta「插入点」语义在「全解析后
   统一执行脚本」管线形态下不可分，s1 全文档生效近似。
3. **script 检查点**（webview.rs `run_page_scripts_impl` 循环头）：inline
   （nonce/hash/unsafe-inline 判定 + sha256 内容 hash，`script_hash_sha256_base64`）+
   外链（nonce 放行或 script-src-elem URL 源匹配）。被阻止脚本跳过执行。
4. **violation 事件派发**（shim part06.js `__zw_dispatch_securitypolicyviolation` +
   engine script_gen 生成器）：document 站 `_dispatchWithBubble`（tgt='doc' AT_TARGET +
   bubble 上行 window），`_makeEvent` 事件 + 违规字段自有属性（native 构造实例过不了
   shim 站内检查——探针实测，见 master.md R2 记录）。lineNumber/columnNumber 置 0
   （FIXME M2-s2：extract 侧携带源位置）。
5. **runner 装配**（testharness.rs）：`csp_enforcement: true`（runner = 实验臂；其他
   corpus 无 meta CSP 页，实测零外溢）+ 注入脚本 nonce 戳记（meta CSP 首个 nonce 源）
   + `data-zw-harness` 标记豁免。

### 中途归因修正（两轮探针闭环）

- **hash 缺位误拦**（−4 全绿假象）：首跑 33→29 回退案全数归因「hash-allowed 内联被
  误拦 + runner 注入/内联 helper 在 no-nonce 政策下被拦」。修：inline sha256 hash
  传入检查 + `data-zw-harness` 投递形态豁免（上游同源外链过 'self'，本地内联是
  harness 基础设施而非页面内容）。修后 **+3 零丢失**。
- **派发路由探针**：document 级 listener 不达 shim EventTarget 链（doc 槽位独立）、
  native `__zw_native_get_document()` 返原始 id 无 dispatchEvent、window 站经
  `__zw_dispatch_event('html',…)` 可达——最终走 document 站专用 shim 全局。

## 剩余缺口（M2-s2+ 修齐方向）

| 簇 | 形态 | 切片 |
|---|---|---|
| 源位置定位 | blockeduri-inline `expected 15 but got 0`（lineNumber/columnNumber） | M2-s2 |
| img/style/connect 资源检查点 | blockeduri-inline img 族 / style-src 阻止族 / connect-src 全簇 0 绿 | M2-s2/s3 |
| eval 检查点 | blockeduri-eval（eval 门禁钩 sandbox 面） | M2-s3 |
| 元素站 target | targeting.html（violation target = 被阻止 script 元素；现为 document 站） | M2-s2 |
| report-uri / Reporting API | .sub.html report 族（runner 无 HTTP server 报告端点） | 记账（infra） |
| Trusted Types / worker 面 | source-file / inside-worker 族 | 挂账 |
