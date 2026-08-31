# Evidence: R0 — polyfill vs native A/B 读路径行为等价基线

**日期**: 2026-08-13
**轮次**: R0
**Commit**: `c7cde09e`
**测试文件**: `crates/engine/src/dom_bindings/tests_ab_compare.rs`

## 测试命令

```bash
cargo test -p zero-engine --features v8 --lib dom_bindings::tests_ab_compare
```

## 结果

```
running 4 tests
test dom_bindings::tests_ab_compare::native_helper_document_bridge_works ... ok
test dom_bindings::tests_ab_compare::ab_query_selector_all_indexed_attribute ... ok
test dom_bindings::tests_ab_compare::polyfill_helper_shim_callbacks_works ... ok
test dom_bindings::tests_ab_compare::ab_read_operations_native_equals_polyfill ... ok

test result: ok. 4 passed; 0 failed; 0 ignored
```

## 对照用例清单（READ_CASES，9 条）

| # | 用例名 | 表达式 | native | polyfill | 一致 |
|---|--------|--------|--------|----------|------|
| 1 | query-selector-tagname | `document.querySelector('.row').tagName` | SPAN | SPAN | ✓ |
| 2 | query-selector-all-length | `document.querySelectorAll('.row').length` | 2 | 2 | ✓ |
| 3 | get-element-by-id-tagname | `document.getElementById('main').tagName` | DIV | DIV | ✓ |
| 4 | get-attribute | `document.getElementById('l').getAttribute('href')` | /p | /p | ✓ |
| 5 | get-attribute-missing-null | `String(...getAttribute('nope'))` | null | null | ✓ |
| 6 | has-attribute | `String(...hasAttribute('disabled'))` | true | true | ✓ |
| 7 | node-type | `document.getElementById('d').nodeType` | 1 | 1 | ✓ |
| 8 | id-reflected | `document.querySelector('p').id` | para | para | ✓ |
| 9 | query-selector-descendant | `document.querySelector('div span').getAttribute('data-x')` | 2 | 2 | ✓ |

（+ querySelectorAll 索引读 `[0].getAttribute('data-i')` = 0 一致）

## 结论

**native 读路径与 polyfill 读路径在 9 类核心读操作上行为完全等价**。

这是 M0 阶段的关键基线结论：在 L2（polyfill 桥改读 live Document）/ S6（高层 API 去字符串）/ QuickJS native（M6）等迁移切片推进前，建立了可回归的等价性证据。后续任何迁移切片若打破等价，本门会立即捕获。

## 双 feature 矩阵覆盖

- **v8**: zero-engine 2063 passed（含本门 4 个）
- **quickjs**: zero-engine 1405 passed（本模块 `#[cfg(feature="v8")]` 排除；M6 QuickJS native 落地后镜像 `run_native`，同一用例表将在 quickjs 矩阵复跑，对齐 DC-7）
