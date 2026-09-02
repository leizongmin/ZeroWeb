# M3 Module Service Worker Dynamic Import Rejection

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/no-dynamic-import-in-module.any.js` 从
  `defer-module-worker-dynamic-import` 提升到 core runner。
- `serviceworker-module` `.any.js` runner 现在按 `// META: global=serviceworker-module`
  使用 `{ type: 'module' }` 注册 Service Worker，并以静态 module import 方式加载
  `/resources/testharness.js`。
- Service Worker runtime 对 classic 与 module worker 中的动态 `import(url)` 都返回
  rejected `TypeError` promise；module worker 不再在图构建阶段因源码包含 `import()` 而
  直接注册失败。
- 固化 module no-dynamic-import wave asset manifest：
  [2026-09-02-m3-module-no-dynamic-import-assets.tsv](2026-09-02-m3-module-no-dynamic-import-assets.tsv)。

## 验证

- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo test -p zero-script-sandbox dynamic_import_rejects -- --nocapture`：
  3 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo run --release --bin zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root no-dynamic-import.any.js`：
  1 case / 4 subtests / 4 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 600 -- cargo run --release --bin zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root no-dynamic-import-in-module`：
  1 case / 4 subtests / 4 Pass
- `make test-wpt-service-workers-module-no-dynamic-import-wave-assets`：PASS

## 结论

core Service Worker baseline 从 45 case / 176 subtest 提升到 46 case / 183 subtest。
该切片不改变 CacheStorage 分母，只补齐 Service Worker module runner 与动态 import
负面语义。
