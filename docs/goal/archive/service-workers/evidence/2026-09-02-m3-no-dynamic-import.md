# M3 Classic Service Worker Dynamic Import Rejection

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/no-dynamic-import.any.js` 从
  `defer-worker-dynamic-import` 提升到 core runner。
- 固化 classic no-dynamic-import wave asset manifest：
  [2026-09-02-m3-no-dynamic-import-assets.tsv](2026-09-02-m3-no-dynamic-import-assets.tsv)。
- 该用例确认 classic Service Worker global 中 `import(url)` 返回 rejected promise，
  维持动态 import 禁用语义；module worker 版本已在 M3-47 跟进纳入 core。

## 验证

- `make testharness-service-workers-core FILTER=no-dynamic-import --always-make`：1 Pass
- `make audit-wpt-service-workers-no-dynamic-import-wave`：4 assets verified
- `make test-wpt-service-workers-no-dynamic-import-wave-assets`：PASS
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- ./target/release/zero-wpt-runner testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --json no-dynamic-import`：
  1 case / 1 subtest / 1 Pass
- 2026-09-02 follow-up：`.any.js` worker wrapper 扩展到 plain `.any.js` 后重跑
  `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo run --release --bin zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root no-dynamic-import.any.js`：
  1 case / 4 subtests / 4 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/archive/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json`：
  45 cases / 176 subtests / 176 Pass，double-run deterministic

## 结论

core Service Worker baseline 从 44 case / 175 subtest 提升到 45 case / 176 subtest。
该切片不改变 CacheStorage 分母，只收敛 classic Service Worker global 的动态 import
负面暴露面。
