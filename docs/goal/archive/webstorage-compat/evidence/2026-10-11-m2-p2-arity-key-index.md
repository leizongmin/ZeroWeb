# M2-P2 — 方法缺参 TypeError + key() unsigned long 取模

**日期**: 2026-10-11
**改动**: `crates/engine/src/js_dom_shim/part02.js` `Storage.prototype.key/getItem/
setItem/removeItem`
**A/B**: 1221/1251 = 97.6%（M2-P1 后）→ **1239/1251 = 99.0%**（+18P，Fail 30→12）

## 修齐语义

1. **WebIDL arity 校验**（https://webidl.spec.whatwg.org/#es-operations）：`key()`/
   `getItem()`/`removeItem()` 零参、`setItem()` 少于 2 参 → TypeError（missing_arguments
   0/10 → 10/10）。
2. **`key(index)` unsigned long 转换**：`index >>> 0`（ToUint32 取模 2^32）——
   `key(4294967296)` 读回 index 0、`key(-1)` → 4294967295 越界 null（storage_key
   14/22 → 22/22）。

## A/B 分案

| 案 | P1 后 | P2 后 |
|---|---|---|
| missing_arguments.window.js | 0/10 | **10/10** |
| storage_key.window.js | 14/22 | **22/22** |

余 Fail 12：quota 4（P3）+ StorageEvent 构造/init 8（M3）。

## 回归验证

`cargo test -p zero-engine --lib`（test-guard 包裹）2977 passed / 0 failed
（`--skip test_pipeline_deeply_nested_html`，M2-P1 已 A/B 归因预存）。
