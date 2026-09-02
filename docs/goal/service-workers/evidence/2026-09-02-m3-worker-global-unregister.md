# M3 Service Worker Worker-Global Unregister

**日期**：2026-09-02
**上游 revision**：`04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
**状态**：complete；promoted to core baseline

## 变更

- `service-workers/service-worker/ServiceWorkerGlobalScope/unregister.https.html` 从
  `defer-worker-global-unregister` 提升到 core runner。
- Service Worker runtime 新增 worker-global `registration.unregister()` host bridge：
  worker 同步发起 typed request，browser/page-runtime 返回 `Promise<boolean>` 语义的
  removal result 或 DOMException-shaped failure。
- renderer/browser IPC 补齐 `UnregisterRequested` 与 `CompleteUnregister` wire contract，
  并保持 request id / error payload 校验。
- page-runtime manager 支持 worker 自身在 evaluation/install/activate/message 期间调用
  `registration.unregister()`：移除后续 scope matching，使 `getRegistration()` 返回
  `undefined`；install/activate 中的调用让 worker 进入 `redundant`；active worker 注销时
  保留既有受控 client 的 controller，同时不再控制新 client。
- 固化 worker-global unregister wave asset manifest：
  [2026-09-02-m3-worker-global-unregister-assets.tsv](2026-09-02-m3-worker-global-unregister-assets.tsv)。

## 验证

- 初始红线：`make testharness-service-workers-core FILTER=ServiceWorkerGlobalScope/unregister TIME_LIMIT=300`
  失败，`self.registration.unregister is not a function`，且 case 因 pending test 超时。
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-protocol service_worker_message_port_and_update_wires_round_trip -- --nocapture`：
  1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-script-sandbox worker_registration_unregister_round_trips_through_host -- --nocapture`：
  1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 300 -- cargo test -p zero-page-runtime worker_unregister -- --nocapture`：
  3 passed
- `make testharness-service-workers-core FILTER=ServiceWorkerGlobalScope/unregister TIME_LIMIT=420`：
  1 case / 4 subtests / 4 Pass
- `make baseline-wpt-service-workers-core OUTPUT=docs/goal/service-workers/evidence/2026-08-19-m1-wpt-core-baseline.json TIME_LIMIT=900`：
  49 cases / 193 subtests / 193 Pass / deterministic true
- `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include cargo clippy --workspace --all-targets -- -D warnings`：
  pass
- `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-protocol -p zero-script-sandbox -p zero-page-runtime -p zero-wpt-runner -- --nocapture`：
  pass
- `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-browser --bin zero-browser -- --test-threads=1`：
  411 passed / 1 ignored
- `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test -p zero-render-foundation gpu::renderer:: -- --test-threads=1`：
  94 passed
- `BINDGEN_EXTRA_CLANG_ARGS=-I/usr/lib/gcc/x86_64-linux-gnu/13/include ZERO_NOPROXY=1 ./target/test-guard --compile-first --per-proc-mem 4 --total-mem 8 --time-limit 900 -- cargo test --no-default-features --features quickjs -p zero-script-sandbox -p zero-webview -p zero-webview-demo -p zero-integration-tests -p zero-wpt-runner`：
  pass

## 结论

core Service Worker baseline 从 48 case / 189 subtest 提升到 49 case / 193 subtest。
该切片补齐 worker-global `registration.unregister()` 在 evaluation、install、activate 与
controlled-client message 场景下的基础语义，不改变 CacheStorage window/SW 分母。
