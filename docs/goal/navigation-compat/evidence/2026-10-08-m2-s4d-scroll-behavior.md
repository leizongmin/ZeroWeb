# M2-S4D — Navigation API scroll-behavior 面：scroll 模式 / scroll() / 滚动保存恢复（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-08-m2-s4d-full-corpus.txt](2026-10-08-m2-s4d-full-corpus.txt)
**前序**: [2026-10-08-m2-s4c-traverse-face.md](2026-10-08-m2-s4c-traverse-face.md)

## 切片内容（master 下一步计划 ①——M2 ④-D scroll()/scroll-behavior 簇）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **滚动保存**：离开 entry 前（`_histApplyNav` push/replace + traverse 提交前）把 window 滚动位存入 session entry（scrollX/Y）——traverse 回访恢复基面 | `crates/engine/src/js_dom_shim/part02.js`（`_histSaveCurrentScroll`） | WPT after-transition-intercept-handler-modifies「state saved before handlers」 |
| **restore 规格**（两形）：① frag 形（navigate/reload intercept）——destination fragment 锚滚 | 无锚/无 fragment → 文档顶；② saved 形（traverse）——目标 entry 保存位。gen = 派发时刻滚动代次（「导航期间文档被滚 → 跳过」判定）。执行器 `_navApplyRestoreSpec` 为链尾 after-transition 与 `e.scroll()` 共用 | `part02.js`（`_navRestoreSpecForUrl`/`_navApplyRestoreSpec`） | WPT manual-scroll-resets-when-no-fragment / -fragment-does-not-exist / after-transition-* |
| **scroll 代次**：任意程序滚动 +1（`_zwApplyScroll` + 滚锚 fallback 写入）——after-transition 链尾恢复的 skip 判定基面 | `part01.js` | WPT after-transition-skips-restore-when-scrolled |
| **intercept({scroll})**：'after-transition'（缺省）/ 'manual' | `part05.js`（intercept 存储 `_zwScrollMode`） | WPT manual-basic |
| **e.scroll()**：dispatch 期抛 InvalidStateError（`_zwDispatching` 印记，`_navDispatchAny` save/restore 防嵌套串写）；重复调抛 InvalidStateError；preventDefault 后抛；导航结算后抛；否则立即按 restore 规格滚 | `part05.js` + `part02.js` | WPT manual-immediate-scroll / -repeated / scroll-after-preventDefault / manual-scroll-after-resolve / after-transition-explicit-scroll |
| **committed 结算时机**：intercept 链任务头（handler 起跑前、同任务）——handler pending 期间可 await；链尾仅 finished。handler 调用与 committed resolve 同任务 ⇒ `await committed` 续延晚于 handler 同步段（handler 内创建的 promise 对续延可见） | `part02.js`（`_navRunIntercept`） | WPT after-transition-push / after-transition-timing / manual-scroll-after-dispatch |
| **finishedSettle 显式 failed 标**：handler 拒绝原因原样透传（`Promise.reject()` → undefined 不误判成功） | `part02.js`（`_navNavResult`/`_navInterceptFail`） | WPT after-transition-reject（promise_rejects_exactly undefined） |
| **非 intercept traverse 滚动恢复**：scrollRestoration auto + 有保存数据 + 非 fragment 变更 | `part02.js`（`_histRestoreScroll`） | WPT scroll-to-fragid 族零回归（fragment 仍走 R3065 滚锚） |
| **非 intercept navigate('#frag') 滚锚**：navigate() 提交后（此前仅 hash-setter/hist-派 popstate 面有） | `part02.js` | WPT after-transition-basic「navigate('#frag') 后 scrollY ≠ 0」 |
| **链尾恢复序**：恢复先于 navigatesuccess；**不读** history.scrollRestoration（Navigation API restore 独立于 entry mode） | `part02.js` | WPT after-transition-timing（popstate 时未滚/onnavigatesuccess 时已滚）/ -with-history-scroll-restoration-manual / -during-promise |

## 定位过程记录（两处非显然根因）

1. **14 Timeout 同根**：S4B/S4C 的 committed 于 handler 链尾结算——scroll-behavior 族全部
   `await promises.committed` 于 handler pending 期间（spec：committed 随提交结算，链只挡
   finished）→ 悬挂。改链任务头结算后，`manual-scroll-after-dispatch` 仍 Timeout——它要求
   `await committed` 续延前 handler **已被调**（intercept_resolve 已赋值），故 resolve 与
   handler 调用须同任务且 handler 在 resolve 的同一同步段后被调。
2. **manual-immediate-scroll 假绿**：dispatch 期 `e.scroll()` 此前无 guard 不抛——assert 失败
   异常被 `_navDispatchAny` 的 listener try/catch 吞掉 → 用例假绿。加 `_zwDispatching`
   印记（save/restore）后 assert 真过（probe 实证 `scrollNoThrow:true` → 修后 InvalidStateError）。

## 数字

| corpus 域 | S4C 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api | 63/227 = 27.8% | **82/227 = 36.1%** | +19 Pass / -9 Timeout / -10 Fail |
| └ scroll-behavior | 1/26 | **22/26 = 84.6%** | +21 |
| history/the-history-interface | 45/49 = 91.8% | 45/49 = 91.8% | 0 |
| history/the-location-interface | 31/36 = 86.1% | 31/36 = 86.1% | 0 |
| traversal/history-traversal | 42/45 = 93.3% | 42/45 = 93.3% | 0 |
| 全量 | 192 P = 45.0% | **211 P = 49.4%** | +19 |

**零回归**：全量 per-subtest 精确 diff（427 案逐行，[全量日志](2026-10-08-m2-s4d-full-corpus.txt) vs
[S4C 日志](2026-10-08-m2-s4c-full-corpus.txt)）——scroll-behavior 域外零状态变化。

## 挂账（scroll-behavior 余 4F = reload 族，均为渲染域缺口）

- `after-transition-reload` / `manual-scroll-reload`（anchored 变体）：断言 `buffer.remove()` 后
  scroll anchoring 调整（1018→18）——引擎无 scroll anchoring（css overflow-anchor 面回流
  rendering-compat）。
- `after-transition-reload-no-scroll-anchoring` / `manual-scroll-reload-no-scroll-anchoring`：
  断言还原目标 = **移除 buffer 后重排版**的 #frag 文档位（1018-1000-10=8）——runner rect 快照
  载入时定格（`LayoutRectSnapshot` 无渲染循环不刷新），DOM 变更后几何过期（probe 实证
  `frag.top=1008` 应为 8）。回流 rendering-compat（rect 快照按需刷新面）。

## 质量门禁

- `make test`：全绿 **20,209 P / 0 F**。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check`：零 diff；`git diff --check` 零 diff。
