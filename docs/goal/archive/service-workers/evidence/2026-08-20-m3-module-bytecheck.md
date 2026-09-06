# M3-15 Service Worker Module Update Bytecheck

**日期**：2026-08-20
**状态**：complete

## 实现

- WPT runner 增加 `bytecheck-worker.py` 与
  `bytecheck-worker-imported-script.py` 的状态化响应：
  - `default` query 每次返回相同 source bytes；
  - `time` query 每次真实请求返回不同 source bytes；
  - main fixture 按 `type=classic|module` 生成 `importScripts()` 或 static `import`。
- manager 对 classic 与 module 使用同一完整 script graph byte comparison：
  top-level source、script type 和全部 imported dependency source 任一变化都会创建
  replacement；完全相同时丢弃 candidate。
- module statement scanner 跳过字符串外注释，并按括号/块深度保留多行函数体，
  防止响应版本注释遮蔽后续 static import。

## WPT

- 新增固定资产：
  [2026-08-20-m3-module-bytecheck-assets.tsv](2026-08-20-m3-module-bytecheck-assets.tsv)，
  4/4 asset 均校验固定 revision、字节数和 Git blob SHA。
- `update-bytecheck.https.html`：8/8 Pass：
  - classic main/imported `default|time` 四种组合；
  - module main/imported `default|time` 四种组合。
- core baseline：
  [2026-08-20-m3-module-bytecheck-baseline.json](2026-08-20-m3-module-bytecheck-baseline.json)，
  21 case / 83 subtest，83 Pass，两轮 deterministic。
- disposition：21 core / 49 defer / 182 gated / 42 skip。

## 验证

- module dependency unchanged/changed manager test：通过。
- module comment/static import compiler regression：通过。
- WPT fixture asset verify 与 disposition audit：通过。
- M3-14 全量门禁继续有效：workspace、Clippy、GPU 94/94、CPU/GPU consistency 1/1、
  16/16 performance microbench。
- parser 定向性能门禁 `benchmark_20260820_192530.json`：1/1 microbench，
  startup 91.21 ms，peak RSS 155.66 MiB，page p95 15.00 / 458.44 / 114.70 ms，
  retained form p95 0.0306 ms，absolute budgets 通过。

## 下一步

- 补齐 module `export ... from` / `export * from` graph extraction 与转换。
- 评估 `registration-script-module.https.html` 和 cross-origin module bytecheck；
  CORS 响应策略必须保持 fail closed。
