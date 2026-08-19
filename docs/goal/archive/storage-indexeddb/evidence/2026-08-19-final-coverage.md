# IndexedDB 最终覆盖与 Done Criteria 报告

**日期**: 2026-08-19
**WPT revision**: `315976933870b34d6ea30e3f6643403edae678ba`

## 覆盖结论

- 上游范围：210 个 `IndexedDB/**/*.any.js`
- imported：168 文件（80.00%）
- imported 结果：1073 Pass / 0 Fail / 0 Timeout / 0 NotRun / 0 empty
- skip list：0 文件；没有从 210 分母排除任何用例
- 未导入：42 文件，仍计入分母，不用于抬高通过率

## Imported 分类

| 资产标签 | 文件数 |
|---|---:|
| `IDB-M1-factory` | 9 |
| `IDB-M1-getall-index-cursor` | 6 |
| `IDB-M1-index-cursor-expansion` | 5 |
| `IDB-M1-object-store-crud` | 6 |
| `IDB-M2-connection-queue` | 2 |
| `IDB-M2-continue-primary-key` | 3 |
| `IDB-M2-cursor-iteration` | 8 |
| `IDB-M2-cursor-mutations` | 7 |
| `IDB-M2-cursor-stepping-expansion` | 8 |
| `IDB-M2-cursor-surface` | 8 |
| `IDB-M2-get-all-options` | 10 |
| `IDB-M2-index-metadata` | 8 |
| `IDB-M2-key-range-binary` | 8 |
| `IDB-M2-key-semantics` | 8 |
| `IDB-M2-metadata-rollback` | 8 |
| `IDB-M2-object-store-ordering` | 8 |
| `IDB-M2-request-event-model` | 8 |
| `IDB-M2-request-lifecycle` | 11 |
| `IDB-M2-rust-cursor-stepping` | 3 |
| `IDB-M2-schema-rename` | 8 |
| `IDB-M2-store-metadata` | 8 |
| `IDB-M2-transaction-deactivation` | 3 |
| `IDB-M2-transaction-lifecycle` | 8 |
| `IDB-M2-transaction-scheduling` | 7 |

## 剩余范围分类

| 类别 | 文件数 | 处理 |
|---|---:|---|
| structured clone / Blob / value | 17 | 仍计入 210 分母，作为后续兼容扩展 |
| index / key generator / auto-increment | 11 | 仍计入 210 分母，作为后续兼容扩展 |
| scale / concurrency / request ordering | 9 | 仍计入 210 分母，作为后续兼容扩展 |
| IDL / platform integration | 5 | 仍计入 210 分母，作为后续兼容扩展 |

## Done Criteria 证据

| DC | 结论 | 权威证据 |
|---|---|---|
| DC-1 WPT | Pass | fetch / runner / ledger = 168 / 168 / 168；本报告；`indexeddb-skip-list.txt` 零排除 |
| DC-2 真实引擎 | Pass | `indexed_db_bridge.rs` + `indexed_db_host.rs`；factory/store/index/cursor/transaction host 路由；168 文件 WPT |
| DC-3 持久化 | Pass | `2026-08-18-m3-persistence-engine.*`、browser owner、embedded WebView owner；restart/I/O rollback tests |
| DC-4 质量 | Pass | `cargo fmt`、workspace Clippy、`make test`、168 文件 WPT 全绿；每次修复有 engine 回归与 ledger 资产 |

## 未导入文件

- `IndexedDB/bindings-inject-keys-bypass.any.js`
- `IndexedDB/bindings-inject-values-bypass.any.js`
- `IndexedDB/blob-composite-blob-reads.any.js`
- `IndexedDB/blob-contenttype.any.js`
- `IndexedDB/blob-delete-objectstore-db.any.js`
- `IndexedDB/blob-valid-after-abort.any.js`
- `IndexedDB/blob-valid-after-deletion.any.js`
- `IndexedDB/blob-valid-before-commit.any.js`
- `IndexedDB/clone-before-keypath-eval.any.js`
- `IndexedDB/crashtests/create-index.any.js`
- `IndexedDB/get-databases.any.js`
- `IndexedDB/historical.any.js`
- `IndexedDB/idbindex-multientry.any.js`
- `IndexedDB/idbindex_reverse_cursor.any.js`
- `IndexedDB/idbindex_tombstones.any.js`
- `IndexedDB/idbobjectstore-put-unique-index-constraint-is-atomic.any.js`
- `IndexedDB/idlharness.any.js`
- `IndexedDB/index_sort_order.any.js`
- `IndexedDB/interleaved-cursors-large.any.js`
- `IndexedDB/interleaved-cursors-small.any.js`
- `IndexedDB/keygenerator.any.js`
- `IndexedDB/large-requests-abort.any.js`
- `IndexedDB/nested-cloning-basic.any.js`
- `IndexedDB/nested-cloning-large-multiple.any.js`
- `IndexedDB/nested-cloning-large.any.js`
- `IndexedDB/nested-cloning-small.any.js`
- `IndexedDB/parallel-cursors-upgrade.any.js`
- `IndexedDB/reading-autoincrement-indexes-cursors.any.js`
- `IndexedDB/reading-autoincrement-indexes.any.js`
- `IndexedDB/reading-autoincrement-store-cursors.any.js`
- `IndexedDB/reading-autoincrement-store.any.js`
- `IndexedDB/request-event-ordering-large-mixed-with-small-values.any.js`
- `IndexedDB/request-event-ordering-large-then-small-values.any.js`
- `IndexedDB/request-event-ordering-large-values.any.js`
- `IndexedDB/request-event-ordering-small-values.any.js`
- `IndexedDB/storage-buckets.https.any.js`
- `IndexedDB/string-list-ordering.any.js`
- `IndexedDB/structured-clone-transaction-state.any.js`
- `IndexedDB/structured-clone.any.js`
- `IndexedDB/value.any.js`
- `IndexedDB/value_recursive.any.js`
- `IndexedDB/writer-starvation.any.js`
