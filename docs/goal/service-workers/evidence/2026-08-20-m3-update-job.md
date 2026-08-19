# M3-6 Service Worker Update Job

**日期**：2026-08-20
**状态**：complete

## 实现

- `ServiceWorkerRegistration.update()` 通过 typed `Update` operation 请求 browser owner；
  production 和 embedded WebView 都重新抓取当前 version 的 canonical script URL。
- manager 为 live version 保存经过 UTF-8 校验的原始 top-level script bytes。相同 bytes 返回
  existing version，不创建 runtime/version；变化 bytes 复用 `start_evaluation()` 创建同
  origin/scope 的 installing replacement。
- update target 按 scope-keyed registration 的当前 waiting/active version 解析；持有旧
  version ID 的 renderer 仍抓取并比较当前 script URL，no-op 时静默刷新本地 version 投影。
- changed update 在顶层 evaluation 成功后 resolve 同一个 `ServiceWorkerRegistration` JS
  identity，随后按既有 task/cursor 路径派发一次 `updatefound` 和 lifecycle statechange。
- fetch、redirect、size、UTF-8、capacity 与 script evaluation failure 复用 register 的
  fail-closed 路径。失败 update reject `TypeError`，不改变现有 active/waiting 页面投影。
- protocol 追加 `Update` operation 10 和 `Updated` result 8；既有 operation 0–9、result
  0–7 判别值不变。

## 回归

- page-runtime：相同 bytes 不增加 runtime，不创建 installing slot；变化 bytes 创建新
  version，旧 active 保持。
- browser owner：三次 browser-owned fetch 返回 v1/v1/v2，分别验证 register、
  `changed=false` 与 `changed=true/new version`。
- WebView V8/QuickJS：no-op update 保持 registration/active identity 且不派发
  `updatefound`；changed update 保持旧 active、创建 waiting replacement，并仅派发一次
  `updatefound`；compile failure 不改变 active/waiting。
- fresh renderer：真实 browser/renderer IPC 和 HTTP fetch 路径完成 v1/v1/v2 更新，
  registration identity、active/waiting slot 与 updatefound 次数正确。
- `update-result.https.html` 固定 revision asset 通过；core baseline 扩为 13/13 case、
  37/37 subtest Pass，连续两轮 deterministic。
- `make test`：V8 WebView 620/620、QuickJS 573/573、QuickJS WPT runner 113/113、
  adapter GPU 94/94 和 fresh peers 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。

## WPT 资产

- manifest：`2026-08-20-m3-update-assets.tsv`，5/5 asset 按 bytes 和 Git blob SHA 固定。
- `make test-wpt-service-workers-update-wave-assets` 覆盖 restore、verify-only、篡改修复与
  缺失拒绝。

## 边界

- 本阶段比较 classic worker 的 top-level script bytes；`importScripts()` 依赖图比较、
  module worker graph 和 `updateViaCache` cache mode 留待 update follow-up。
- 周期性 soft update、跨浏览器重启调度与 registration persistence 留待 persistence 阶段。

## 性能

`make bench-gate` 报告 `benchmark_20260820_055653.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- page total p95：15.29 / 430.30 / 115.92 ms；
- retained form p95：0.0430 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。
