# M2-S4O — C 类余项第二批：同文档 navState 承继 + reentrant 孤儿 transition + 提交期 abort + `:target` URL 同步 + window.stop abort 面（2026-10-09）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4o-full-corpus.txt](2026-10-09-m2-s4o-full-corpus.txt)
**前序**: [2026-10-09-m2-s4n-c-class.md](2026-10-09-m2-s4n-c-class.md)

## 切片内容（三个独立根因，各带 spec 锚）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **同文档导航 navState 承继**：`_navPushCurrent` 加 `carryState` 参数——hash-setter（载入后 push 支路）/ href-setter / assign / hash-only 锚导航四路 push 承继源 record navState（跨文档不承继）；并经 `_navInheritedNavState` 于 fire 时下发 `destState`（handler 内 `destination.getState()` 直读，先于 commit 绑 rec） | `part02.js`（`_navPushCurrent` + 四 fire 站点） | spec apply the push or replace history step——同文档新 entry navigation API state 承继 current entry（whatwg/html#apply-the-push-or-replace-history-step）；WPT state/updateCurrentEntry-method same-document-away-and-back-location-api + navigate-destination-getState-fragment-via-href |
| **transition 创建去重 + 抢占守卫**：删 `_navFireNavigate` dispatch 前恒死副本（S4L 遗留，intercept() 未调 `_zwIntercepted` 恒 false）；dispatch 后副本加 `!_zwErrored && !_zwSettled` 守卫——被抢占导航不再创建孤儿 transition 覆盖后继导航在位 transition（其 finished 永不结算 → Recorder finalExpectedEvent 永不达 → 4 变体全 Timeout） | `part02.js`（`_navFireNavigate` 尾部） | spec inner fire step 29（transition 于 apply step 创建，aborted 导航不建）；WPT ordering location-href-intercept-reentrant / navigate-same-document-intercept-reentrant 双变体 |
| **提交期 abort 不 reject committed**：`ev._zwCommitting` 印记（`_navCommitNav` / `_navCommitTraversal` 顶部置位）——提交块内 dispose/重入触发的新导航 abort 本导航时只 reject finished，committed 保持 pending 随后照常兑现（旧版 `ctrl.resolve` 在 commit 闭包返回后才调，提交中 abort 的 committed 仍 pending 即被 reject → `Promise.all` 抛 AbortError） | `part02.js`（`_zwAbortOngoing` + 两提交闭包） | spec：已提交导航 abort 只 reject finished；WPT dispose-same-document-navigate-during「the committed promise should still fulfill」+ intercept-and-navigate（意外收口） |
| **同文档导航 `:target` URL 同步**：内存导航不重载文档 → live doc `url` 槽与 shim `_zwFragmentUrl` 槽停留在载入值，`document.querySelector(':target')` 恒 miss。新增原生绑定 `__zw_native_set_document_url`（`with_dom_mut` → `Document::set_url`）+ shim `_zwSyncDocUrl` 双槽同步，接线 `_pushHistNav` / `_replaceHistNav` / `_histApplyNav`（push/replace 两支）三 chokepoint | `dom_bindings/factories.rs` + `dom_bindings/mod.rs` + `part02.js` | HTML URL and history update steps（文档 URL 随 session entry 更新）；WPT navigate-same-document ×2 + scroll-to-fragid target-pseudo-after-reinsertion（意外收口） |
| **window.stop abort 面三件套**：① `_zw_abort_signal` 触发 `onabort` 事件处理器属性（spec abort 事件派发——handler 本质 addEventListener 注册面）；② **跨文档 navigate() 结算推迟**——非 hash-only 的 committed/success 走微任务（同步块内 pending，`window.stop()` 可 reject committed；真跨文档的 committed 到新文档 commit 才兑现；同文档 hash-only 维持同步立即结算——ordering/dispose 簇断言基面）；③ `_zwCommitting` 印记**块末清零**（提交块内 abort 不 reject committed，块外照常 reject）；④ 导航 AbortError 附 `stack`（navigateerror filename/lineno/colno 提取源） | `part02.js`（`_zw_abort_signal` / navigate() 非 intercept 分支 / 两提交闭包 / `_navNavAbortError`） | spec stop loading + notify-about-committed 时序；WPT signal-abort-window-stop ×3 + signal-abort-preventDefault（意外收口） |

## 数字

| corpus 域 | S4N 后 | 本轮（终态） | Δ |
|---|---|---|---|
| navigation-api（NotRun 14 不计分母） | 203/255 = 79.6% | **218/255 = 85.5%** | +15 |
| 全量 | 342 P / 476 = 71.8% | **358 P / 476 = 75.2%** | +16 |

**16 翻全 Fail/Timeout → Pass，零回归**（全量 per-subtest 精确 diff，S4N vs S4O 终态）：
state/updateCurrentEntry-method away-and-back-location-api ×2（Timeout→Pass）、
location-href-intercept-reentrant 双变体 + navigate-same-document-intercept-reentrant
双变体 ×4（Timeout→Pass）、dispose-same-document-navigate-during（Fail→Pass）、
navigate-destination-getState-fragment-via-href（原 Fail 顺手收口）、
intercept-and-navigate（原 Fail——`_zwCommitting` 守卫正命中其断言「abort finished 但
不 abort committed」）、navigate-same-document ×2（Fail→Pass，`:target` 同步）、
target-pseudo-after-reinsertion（原 Fail→Pass，`:target` 同步连带收口）、
signal-abort-window-stop ×2 Fail + 1 Timeout（window.stop 三件套）、
signal-abort-preventDefault（原 Fail→Pass，连带收口）。

## 挂账（本轮诊断记录）

- **traverse 不 prune 记录面**：`history.back()` 后 `navigation.entries()` 保留 forward
  entries（away-and-back 族断言基面），dispose 于后续 push 截断时同步触发——与本切片
  fork 提交期 dispose 联动的 spec apply-the-traverse-history-step prune 语义未实现，
  现行为对全部在库用例成立，记账不动。
- replace-before-load 38F / bfcache / 跨文档链维持 M4 定性（B 类，runner 形态/M3 依赖）。

## 质量门禁

- `make test`：全绿（结果见 master.md 本轮行）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
