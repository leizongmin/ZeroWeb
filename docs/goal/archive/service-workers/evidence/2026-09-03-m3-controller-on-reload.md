# M3 Service Worker controller on reload

- Date: 2026-09-03
- WPT revision: `04067ce9c7c2165e71ad7d0dde10a4c5cb394a83`
- Case: `service-workers/service-worker/controller-on-reload.https.html`
- Support: none

## Result

`controller-on-reload.https.html` is promoted to the Service Worker core runner.
The case covers the controlled iframe reload boundary: after an iframe registers
an active Service Worker, its current document remains uncontrolled, and the
reloaded iframe document observes `navigator.serviceWorker.controller` as a
Service Worker object.

## Runtime Notes

Iframe reload now removes the old nested Service Worker window client before
creating the replacement iframe document. This keeps same-document registration
semantics unchanged while allowing the reloaded cross-document client to be
observed as a fresh nested client and acquire the active controller for its
matching scope.

The WebView regression verifies that the reloaded iframe exposes an iframe-realm
`ServiceWorker`, that its `scriptURL` matches the active registration, and that
the parent registration wrapper is not reused across realms.

## Verification

- `BINDGEN_EXTRA_CLANG_ARGS='-isystem /usr/lib/gcc/x86_64-linux-gnu/13/include' ./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo test -p zero-webview iframe_reload_observes_active_service_worker_controller -- --nocapture`:
  1 test / 1 passed
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 240 -- cargo run -p zero-wpt-runner -- testharness-service-workers --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root controller-on-reload.https.html --json`:
  1 case / 1 subtest / 1 Pass
- `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- python3 tests/wpt-runner/scripts/run-service-workers-core-baseline.py --runner ./target/release/zero-wpt-runner --wpt-data tests/wpt-runner/wpt-data/.service-workers-tier-a-root --output docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-reload-baseline.json --summary docs/goal/archive/service-workers/evidence/2026-09-03-m3-controller-on-reload-baseline.md`:
  65 cases / 249 subtests / 249 Pass, deterministic
- `python3 tests/wpt-runner/scripts/audit-service-worker-disposition.py`:
  PASS, `core=78 defer=22 fetch=3 gated=149 skip=42`

## Asset Contract

- [2026-09-03-m3-controller-on-reload-assets.tsv](2026-09-03-m3-controller-on-reload-assets.tsv)
- [2026-09-03-m3-controller-on-reload-baseline.md](2026-09-03-m3-controller-on-reload-baseline.md)
- [2026-09-03-m3-controller-on-reload-baseline.json](2026-09-03-m3-controller-on-reload-baseline.json)

## Conclusion

The Service Worker core baseline expands from 64 cases / 248 subtests to
65 cases / 249 subtests. This fixes the WPT disposition lane from `defer` to
`core` for iframe reload controller acquisition.
