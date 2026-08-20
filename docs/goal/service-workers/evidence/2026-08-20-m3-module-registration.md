# M3-18 Service Worker Module Registration Errors

**日期**：2026-08-20
**状态**：complete

## 实现

- module compiler 在 link 阶段校验 default 与 named import 对应的 export。
- export 查询可递归穿透 named/star re-export，缺失 binding 返回 compile error，
  不再把 dependency namespace 错当成 default export。
- top-level await 在当前同步 Service Worker module runtime 中明确以 compile failure
  拒绝，不产生部分安装。
- WPT runner 为 malformed worker 提供固定 parse/runtime/caught/TLA/instantiation
  响应，并把 invalid chunked fixtures 映射为网络失败。

## WPT

- 新增固定资产：
  [2026-08-20-m3-module-registration-assets.tsv](2026-08-20-m3-module-registration-assets.tsv)，
  6/6 asset 均校验固定 revision、字节数和 Git blob SHA。
- `registration-script-module.https.html`：10/10 Pass，覆盖：
  - invalid chunked encoding（含 flush）；
  - parse error、undefined access、uncaught exception；
  - top-level await；
  - missing default export instantiation failure；
  - missing export + top-level await；
  - missing script；
  - caught exception 成功注册。
- core baseline：
  [2026-08-20-m3-module-registration-baseline.json](2026-08-20-m3-module-registration-baseline.json)，
  23 case / 101 subtest，101 Pass，两轮 deterministic。
- disposition：23 core / 49 defer / 180 gated / 42 skip。

## 验证

- V8 `zero-script-sandbox`：link/re-export 回归通过。
- QuickJS Service Worker re-export runtime：通过。
- WPT asset verify 与 disposition audit：通过。
- workspace Clippy `-D warnings` 与 script-sandbox 定向性能门禁通过。
- 定向性能报告 `benchmark_20260820_220312.json`：1/1 microbench，
  startup 111.03 ms，peak RSS 155.72 MiB，page p95 20.19 / 540.40 / 112.14 ms，
  retained form p95 0.0370 ms，absolute budgets 通过。

## 下一步

- 审计 module registration/update 中尚未覆盖的 request metadata 与 cache policy。
- 继续 M3 MessagePort transfer 与多 client 枚举，或在依赖满足后启动 M2 fetch pipeline。
