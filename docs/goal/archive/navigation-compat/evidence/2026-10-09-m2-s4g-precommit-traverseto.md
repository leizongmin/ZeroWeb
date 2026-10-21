# M2-S4G — precommitHandler + traverseTo + ongoing-abort 事件序（2026-10-09）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4g-full-corpus.txt](2026-10-09-m2-s4g-full-corpus.txt)
**前序**: [2026-10-08-m2-s4f-location-exotic.md](2026-10-08-m2-s4f-location-exotic.md)

## 切片内容（M2 ④ 残余最大簇——precommit-handler 30F + ordering-and-transition 28F + navigation-methods traverseTo 面）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **precommitHandler**：intercept({precommitHandler}) 分列存储；全部**同步调起**（controller 注入）→ wait-for-all 成功 → 提交 → 起链；任一 reject/throw → 失败路径（无提交、双 reject、navigateerror） | `part02.js`（`_navRunIntercept` 重构）+ `part05.js` | spec nav-event commit handler steps；WPT precommitHandler-push/replace/reload/traverse 三模式 |
| **提交延迟化**：navigate()/reload()/pushState/replaceState 的提交动作闭包化（commitFn）——precommit 存在时延迟到其结算；traverse 的 cursor/CCE/popstate 块同样闭包化延迟 | `part02.js` | WPT precommitHandler-rejectBeforeCommit「committed 不 fulfill + 无 entry」 |
| **NavigationPrecommitController**：redirect(url, {history, state, info})——destination URL/state 单元格改写（提交前 getState 即反映）+ push/replace 切换（省略成员不覆写）+ addHandler 追加 handler 链 | `part02.js` + `part05.js`（`_zwSetUrl`/`_zwSetState`） | WPT precommitHandler-redirect-push / -redirect-options |
| **navigate({state}) → navState 槽**：entry navState 与 classic history.state 分槽（pushState 只入 classic）；destination/currentEntry.getState() 提交前后一致 | `part02.js` | WPT navigate-destination-getState-navigate / navigate-info-and-state |
| **traverseTo(key)**：按 key 反查 record → session entry 位；越界双 reject InvalidStateError；key 即当前双 fulfill；否则入 traverse 队列 | `part02.js` | WPT traverseTo-same-document / -multiple-steps |
| **NavigationHistoryEntry / NavigationTransition 接口对象** + 实例 prototype 链接（instanceof 面）+ transition.from/to | `part02.js` | WPT return-value helpers「fulfillment value must be a NavigationHistoryEntry」/ transition-to |
| **进行中导航抢占**（spec inform-abort **while 循环**）：任意新导航派发前中止 ongoing（非 intercept 同样参与）；abort 序 = signal abort → committed（未结算才）/finished reject → navigateerror → transition.finished reject → 清；dispatch 期中止 = canceled 标记（调用方 cancel 分支去重） | `part02.js`（`_navOngoing` + `_navFireNavigate` 入口循环） | WPT navigate-multiple-navigation-navigate 事件序 / navigate-canceled |
| **window.stop()**：中止进行中导航（双 reject AbortError + navigateerror + 无提交） | `part02.js` | WPT precommitHandler-window-stop-before-commit / traversal-window-stop |
| **非 intercept 同文档导航 success steps**：navigatesuccess 照发（空 handler 链）；被抢占则跳过；committed 提交即结算（success 微任务前） | `part02.js` | WPT ordering navigate-same-document / back-same-document |
| **transition 生命周期**：navigatesuccess/navigateerror 派发时 transition 仍暴露（Recorder 于监听器内挂 transition.finished）；settled 后 resolve.finished、最后清；+ transition.committed | `part02.js` | WPT ordering-and-transition Recorder 序 |
| **traverse 步骤入 task 队列**：back/forward/go 从 queueMicrotask 改 setTimeout（spec session history traversal queue——back() 后同任务排队的微任务先于 traverse 事件） | `part02.js`（`_hist_queueTraversal`） | WPT ordering back-same-document「promise microtask 先于 navigate」 |
| **dispose 事件**：replace 让位 / push 截断的 entry 在 CCE 后派 dispose（ondispose + addEventListener('dispose')） | `part02.js`（`_navFireDispose`） | WPT currententrychange-dispose-ordering |
| **scroll() precommit 期抛**：interception state 未 committed（WPT manual-scroll-in-precommit-handler） | `part05.js` | 同左 |
| **资产**：ordering-and-transition/resources（Recorder helpers.mjs） | fetch 脚本 | 同左 |

## 定位过程记录（三处非显然根因）

1. **非 intercept navigate 丢提交**：precommit 改造把 `_histApplyNav` 移入 intercepted 分支后，
   非 intercept 分支漏回补——全语料 navigate() 不再落 entry（probe：entries 恒 1）。提交动作
   闭包化（`_navCommitNav`）统一三分支后收敛。
2. **window.stop 假中止**：stop 走 ev 级 abort（只置 `ev._zwSettled`），链闭包 `settled` 未落 →
   precommit promise resolve 后 doCommit 照跑 → 提交 + navigatesuccess。修为 `_navOngoing.abort`
   升级为**链感知 wrapper**（先落链 settled 门再走 ev 级 abort 序）。
3. **abort 期二连 navigateerror**：_navInterceptFail 内同步 abort signal → 测试监听器 reject
   precommit → Promise.all failure 分支再派一次。failure 分支加 `settled` 早退去重。

## 数字

| corpus 域 | S4F 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api 全域 | 91/227 = 40.1% | **159/228 = 69.7%** | +68 |
| └ precommit-handler | 0/30（全 Fail） | **38/38** | — |
| └ ordering-and-transition | 1/30 | **28/30**（余 anchor-download ×2 Timeout） | — |
| └ navigation-methods | — | traverseTo/getState 簇收口 | — |
| html/browsers + iframe 域 | 139 | 139 | 0 |
| 全量 | 230 P / 435 = 52.9% | **298 P / 442 = 67.4%** | +68（语料 +7：ordering resources 齐后 case 转可执行） |

**零回归**：全量 per-subtest 精确 diff（S4F vs S4G-final）——Pass→Fail/Timeout 为 0。

## 挂账

- `anchor-download-intercept{,-reject}` 2T：anchor download 属性 → downloadRequest 面与
  canIntercept 语义——独立小切片（downloadRequest 非 null 时不拦截的导航路径）。
- 余 91F 集中在 replace-before-load（load 前 replace 语义）、scroll-to-fragid（几何/编码）、
  navigate-event 深簇——M4 判定的定性输入。

## 质量门禁

- `make test`：全绿 **20,225 P / 0 F**（锚 20,209 → 20,225 含兄弟流新增）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
