# M1-5c Registration URL Contract

**日期**：2026-08-19
**状态**：complete

## 实现

- page-runtime 新增 browser/WebView 共享 registration URL validator。
- scriptURL 与 scope fragment 在安全校验前规范化移除。
- scriptURL/scope 的 `%2f`、`%2F`、`%5c`、`%5C` 编码路径分隔符统一拒绝。
- scope 必须位于 script directory 下；显式 `scope: null` 按 WebIDL 转为 `"null"`，
  并因越过 script directory 以 `SecurityError` 拒绝。
- 页面区分 scope absent/undefined 与显式空值，不再用 truthy conversion 抹平。
- IPC 追加 `Security` error code，既有 0–6 判别值不变，新值为 7；renderer 将
  validation error 投影为 TypeError 或 SecurityError DOMException。
- DOMException 原型与 QuickJS 既有行为对齐，实例同时满足 `instanceof DOMException`
  与 `instanceof Error`。

## WPT 结果

M1-5c 收敛最后 6 个红项：

- scope null：1 Pass；
- scope fragment：1 Pass；
- scope encoded slash/backslash：2 Pass；
- scriptURL fragment：1 Pass；
- rejection DOMException/Error brand：1 Pass。

固定 core baseline 达到 12/12 case、36/36 subtest Pass、0 Fail、0 Timeout、
0 Unsupported；连续两轮 `(case, subtest, status)` 一致。

## 回归

- shared validator：fragment、encoded separator、script-directory scope；
- browser owner：normalized fetch plan 与 Type/Security 分类；
- protocol：ServiceWorkerErrorCode append-only discriminant；
- V8/QuickJS WebView：URL normalization、null/encoded/cross-origin rejection shape；
- V8/QuickJS DOMException dual brand；
- `make testharness-service-workers-core` 与双轮 baseline；
- `make test`：fresh peers、workspace V8、94/94 adapter GPU、QuickJS WebView 568 项、
  QuickJS WPT runner 113/113、QuickJS renderer；
- replacement identity 全包并行回归使用 20 秒宿主预算并记录 Promise 阶段，避免 5 秒负载假红；
- `make bench-gate`：16/16；page total p95 18.20/472.82/152.23 ms；retained form
  p95 0.0312 ms、jank 0；绝对预算通过。
