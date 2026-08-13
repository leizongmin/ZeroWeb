# HTML Behavior Compatibility Completion Audit

Date: 2026-08-13

## Functional Requirements

| Requirement | Evidence | Verdict |
|---|---|---|
| FR-001 product fixture | `form_fixture_complete_multiprocess_semantics`; `form_fixture_reports_missing_control_stage` | complete |
| FR-002 default-action transaction | `default_action_conformance_across_hosts`; `prevented_action_conformance_across_hosts` | complete |
| FR-003 text editing | page-runtime text action tests; three-host text constraint tests | complete |
| FR-004 focus model | scoped focus owner, Tab/Shift+Tab action tests, selected focus WPT | complete |
| FR-005 checkedness | page-runtime radio tests; three-host checked transaction tests | complete |
| FR-006 reset | reset tests across engine, renderer, TabWorker and WebView | complete |
| FR-007 submission | owner-aware entry list; GET/POST intent tests across hosts | complete |
| FR-008 interactive element families | anchor/fragment, details/summary, dialog/popover, select/option integration tests | complete |
| FR-009 resource element events | RFC priority is optional; M3d explicitly may be deferred and does not block M4 | non-blocking optional |
| FR-010 live renderer WebDriver | automation IPC contract; live HTTP form and stale-reference tests | complete |
| FR-011 WPT harness/testdriver | PASS/FAIL/TIMEOUT/UNSUPPORTED reporter; click/send_keys shared-action adapter; 8 upstream subtests | complete |
| FR-012 repository-owned coverage | `test-matrix.md`, integration tests, pinned WPT ledger | complete |

## Non-Functional Requirements

| Requirement | Evidence | Verdict |
|---|---|---|
| NFR-001 WPT is not sole oracle | every matrix row has repository-owned coverage | complete |
| NFR-002/003 determinism and bounded diagnostics | typed scenario DSL, wall-clock timeouts, bounded pending registries | complete |
| NFR-004 performance | retained form hard gate and absolute page-total budgets pass; cross-CPU relative comparison is rejected | complete |
| NFR-005 compatibility | no new third-party dependency; public WebView API remains backward compatible | complete |
| NFR-006 V8/QuickJS | `make test` exits 0 | complete |
| NFR-007 ownership | browser, renderer and WPT loops retain explicit state ownership | complete |
| NFR-008 security | loopback WebDriver, request/script/key size caps, enumerated operations, bounded pending/element registries | complete |

## Interfaces

| Interface | Evidence | Verdict |
|---|---|---|
| IF-001 stable node identity | `PageNodeHandle`, `PageNodeRef`, stale generation tests | complete |
| IF-002 pressed target | browser pressed-target reflow and generation tests | complete |
| IF-003 behavior classification | shared DOM/engine action snapshots | complete |
| IF-004 text shaping boundary | paint/IPC/caret/hit-test/IME shared UTF-16 boundaries | complete |
| IF-005 action transaction | `ActionPlan` mutations, events, effects and rollback tests | complete |

## Gates

- `make test`: pass; adapter-only GPU tests run only when the headless adapter probe succeeds.
- `make testharness-html`: pass, 8 upstream subtests.
- `cargo clippy --workspace --all-targets -- -D warnings`: pass.
- Full benchmark report: 16/16 microbench executors and retained form hard gate pass.
- `perf-gate.sh`: pass; absolute budgets pass and an incompatible i5 baseline is not applied to the Xeon host.
