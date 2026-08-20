# M3-9 Service Worker importScripts Graph

**日期**：2026-08-20
**状态**：complete

## 实现

- classic Service Worker global 暴露真实同步 `importScripts()`。worker engine 线程通过
  typed request ID 暂停 top-level evaluation，宿主完成抓取后一次性回填全部 source；
  多参数脚本在同一 worker global 中按调用顺序执行。
- production JS runtime 位于 renderer 的独立 Service Worker host 线程；import request/event
  与 completion command 走 typed browser↔renderer IPC，不依赖 renderer 主循环。browser
  manager 仍是 registration graph 和网络决策的单一所有者。
- manager 按 main script URL 解析相对 specifier，移除 fragment，并只接受 canonical
  `http`、`https` 和 `data` URL。每个 version 保存 URL→原始 UTF-8 bytes 的 imported
  script graph。
- production browser owner 通过 `TabFetchProxy` 和 `ResourceLoader` 并发抓取同一次调用的
  全部 URL。全部响应完成并通过 status、final URL、secure-context downgrade、CORS、
  JavaScript MIME、UTF-8 和 size 校验后才恢复 runtime；任一失败时不执行该批任何 source。
- imported fetch cache mode 使用 registration 的 `updateViaCache`：`none` bypass，
  `imports`/`all` 允许正常 cache reuse。初次 main script 仍无条件 bypass。
- update job 即使 main bytes 相同也创建隔离 candidate runtime并重新加载 startup imports。
  candidate 完成后比较 main bytes 与完整 imported URL/source map；graph 相同则回收 candidate，
  返回现有 version 且不派发 `updatefound`；任一 imported byte 变化才进入 replacement lifecycle。
- active registration persistence 保存排序后的 imported graph。browser restart 使用 snapshot
  直接恢复 startup imports，不重新联网，也不重放 install/activate。
- `ResourceLoader` 增加标准 `data:` URL 解码；JavaScript MIME predicate 由 `zero-net`
  统一提供给 production browser 与 embedded WebView。

## 资源边界

- 每次 `importScripts()` 最多 64 个 URL；URL 最多 64 KiB。
- 每个 imported script 最多 16 MiB；单次 runtime response batch 最多 16 MiB。
- 每个 version 最多 1,024 个唯一 imported URL，main + imported source 总计最多 64 MiB。
- runtime shutdown 和 tab disconnect 都会主动解除阻塞请求，不等待超时，也不遗留 runtime。
- fetch 失败在 worker global 中以 `NetworkError` 名称抛出。

## 回归

- runtime V8/QuickJS：多参数顺序、global binding、host failure、`NetworkError` 和 shutdown
  cancellation 通过。
- page-runtime：relative/absolute/data URL canonicalization、policy projection、graph
  persistence/restart、main 相同的 imported-byte no-op/changed comparison 通过。
- browser owner：三轮 register/update graph、`none` bypass、MIME/CORS/downgrade 拒绝、
  queued-plan disconnect cleanup 通过。
- WebView V8/QuickJS：真实页面 `register()` 顺序执行 imports；仅 dependency bytes 改变时
  `update()` 产生一次 `updatefound`。
- fresh renderer：production browser/renderer 真实 HTTP 链抓取 `/sw.js` 和
  `/dependency.js`，imported global 被 message handler 使用。
- fresh browser restart：首次抓取 main + dependency 并持久化；第二组 browser/renderer
  在服务器关闭后仍从 snapshot 恢复 activated controller。
- core WPT：`import-scripts-data-url.https.html` 从 defer 提升为 core；14/14 case、
  38/38 subtest Pass，连续两轮 deterministic。
- disposition：294 source / 331 URL 确定性重建为 14 core / 49 defer /
  189 gated / 42 skip。
- `make test`：V8 WebView 622/622、QuickJS 575/575、QuickJS WPT runner 113/113、
  adapter GPU 94/94、CPU/GPU consistency、fresh peers 和 QuickJS Clippy 全过。
- `cargo clippy --workspace --all-targets -- -D warnings` 通过。
- static-wave 4/4 asset restore、verify-only、篡改/缺失 fail-closed 回归通过。

## 边界

- 本阶段 graph 是 classic worker startup evaluation 中实际调用的 `importScripts()`。
  install/activate/message handler 执行期间首次出现的新 import 尚未绑定长期 browser fetch
  context。
- dynamic MIME/redirect/stash/cross-origin WPT 仍依赖动态 WPT server adapter；本阶段已实现
  production response validation，但未提升这些 gated case。
- module Service Worker、静态 module import graph、dynamic `import()`、周期性 soft update
  和 M2 FetchEvent/Cache pipeline 不在本阶段。

## 性能

`make bench-gate` 报告 `benchmark_20260820_113222.json`：

- 16/16 crate、94 个微基准完成，报告未标记 suspect；
- startup：117.56 ms，peak RSS：153.66 MiB；
- page total p95：21.71 / 585.37 / 129.06 ms；
- retained form p95：0.0568 ms，jank 0；
- 当前主机与固定基线 CPU 不同，relative gate 不可比较；absolute page-total 与 retained-form
  budgets 通过。
