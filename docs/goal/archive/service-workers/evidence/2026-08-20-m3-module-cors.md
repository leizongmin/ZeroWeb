# M3-16 Service Worker Module CORS

**日期**：2026-08-20
**状态**：complete

## 实现

- static module URL 仍在 manager 中拒绝 credentials、fragment、非 HTTP(S) 和 HTTPS
  downgrade，但不再错误地把所有跨源 URL 在 fetch 前拒绝。
- browser owner 与 WebView response adapter 对跨源 module 响应执行 CORS 校验：
  `Access-Control-Allow-Origin` 必须为 `*` 或 registration origin。
- classic `importScripts()` 保持 no-cors 行为；同一响应校验函数由显式 `is_module`
  区分策略，不改变已有 classic WPT。
- WPT runner 渲染 `get-host-info.sub.js` 的固定 host 模板，并为 imported bytecheck
  fixture 返回标准 JavaScript MIME 与 ACAO。

## WPT

- 新增固定资产：
  [2026-08-20-m3-module-cors-assets.tsv](2026-08-20-m3-module-cors-assets.tsv)，
  case 的 revision、字节数和 Git blob SHA 均固定。
- `update-bytecheck-cors-import.https.html`：8/8 Pass：
  - cross-origin classic main/imported `default|time` 四种组合；
  - cross-origin module main/imported `default|time` 四种组合。
- core baseline：
  [2026-08-20-m3-module-cors-baseline.json](2026-08-20-m3-module-cors-baseline.json)，
  22 case / 91 subtest，91 Pass，两轮 deterministic。
- disposition：22 core / 49 defer / 181 gated / 42 skip。

## 安全边界

- module 跨源响应缺失或不匹配 ACAO 时 fail closed。
- final URL 继续接受 CORS 允许的跨源 redirect，但禁止 secure-context downgrade。
- response MIME、UTF-8、单脚本 16 MiB 与完整 graph 64 MiB 上限保持不变。

## 验证

- browser response policy：classic cross-origin 无 ACAO 通过；module 无 ACAO 拒绝；
  module `ACAO: *` 通过。
- WebView navigator module 无 ACAO：注册 Promise 拒绝并包含 CORS 诊断。
- workspace Clippy `-D warnings`、asset verify 与 disposition audit 通过。
- M3-15 workspace、GPU 94/94 与 performance 门禁未受渲染无关的 response policy
  变更影响。

## 下一步

- 补齐 module `export ... from` / `export * from` graph extraction 与转换。
- 审计 `registration-script-module.https.html` 的 parse/runtime/instantiation/TLA
  错误分类与动态 fixture。
