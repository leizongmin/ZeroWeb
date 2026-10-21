# M2-S4M — dispose 深簇收口 + 载入后 hash-setter push 语义 + ongoing 槽生命周期（2026-10-09）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4m-full-corpus.txt](2026-10-09-m2-s4m-full-corpus.txt)
**前序**: [2026-10-09-m2-s4l-transition-lifecycle.md](2026-10-09-m2-s4l-transition-lifecycle.md)

**注**：本文数字以最终修复态全量日志为准（轮内首跑含 traverse 提交双调用回归与
`?currententrychange` CCE 序微任务敏感翻转，均已处置/定性，见下）。

## 切片内容（C 类可切片——dispose 深簇）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **载入后 hash-setter 片段导航转 push**：window 'load' 派发置 `__zwDocCompletelyLoaded`（part06 dispatch 钩子）；hash-setter Navigation API 侧按标记分派——载入前 replace（location-api `?` 无变体面维持）/ 载入后 **push**（entries 增长 + forward-pruning dispose 面）。classic history 侧恒 push 不变 | `part06.js` + `part02.js`（`_setLocationHash`） | WPT dispose-same-document「entries = start+3」vs location-api「CCE replace + index 不变」——载入态区分 |
| **ongoing 槽生命周期**：navigate()/traverse 的 **commit 开始即清槽**（commit 不可再被抢占——dispose 处理器内触发的后续导航不被过期槽 abort，消 AbortError unhandled）；非 intercept 同文档导航 **committed/finished 同步立即结算**（spec「fulfill immediately」）+ success steps 微任务化 | `part02.js` | WPT dispose-same-document-navigate-during |
| **finished 标记 handled**：`ctrl.finished.catch(no-op)`（spec「Mark as handled finishedPromise」——被后续导航 abort 的导航其 finished 兑现/reject 均不产生 unhandledrejection） | `part02.js`（`_navNavResult`） | 同上 |
| **ordering `?currententrychange` CCE 序簇**：transition 创建前移至 `_navFireNavigate` dispatch 后（CCE 派发时 transition 已暴露——Recorder 断言同根）；_navRunIntercept 复用同一对象/结算钩子；顺修 traverse 提交双调用（ongoing-clear 插入误致 `_navCommitTraversal` ×2）回归（门禁捕获 r3065 scrollY 翻倍） | `part02.js` | WPT ordering `?currententrychange` 变体族 + r3065 回归钉 |

## 定位过程记录

- **traverse 提交双调用**：ongoing-clear 插入误落既有 `_navCommitTraversal()` 之前 → 提交
  ×2 → 滚锚 ×2（r3065 back() scrollY 1000）——门禁当场捕获，去重修复。
- **?currententrychange CCE 序微任务敏感**：S4K 展开后 ordering Recorder 断言对微任务边界
  敏感，轮间 -7 波动带——经 transition 前移 + commit 语义对齐后 11 案 CCE 变体全 Pass 复验
  （最终态全量）；残余微任务敏感 flake 根治挂账后续专项。

## 数字（最终态）

| corpus 域 | S4L 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api 全域 | 190/255 = 74.5% | **202/255 = 79.2%** | +12 |
| └ per-entry-events dispose 深簇 | 3 Fail | **3 Pass**（dispose-same-document / -intercept / -navigate-during；bfcache 1F + tentative 1T 挂账维持） | — |
| └ ordering `?currententrychange` 变体 | 11 Fail | **11 Pass** | — |
| 全量 | 329 P / 476 = 69.1% | **341 P / 476 = 71.6%** | +12 |

**零回归**：全量 per-subtest 精确 diff（S4L vs S4M-final）——Pass→Fail/Timeout 为 0。

## 挂账

- dispose-after-bfcache / dispose-cross-document / -for-navigation-in-child / -for-full-session-history：
  frame tree / dispatcher infra——M3 挂账。
- `?currententrychange` CCE 序微任务敏感 flake 根治——后续专项。
- dispose-same-document-navigate-during 1F（unhandled AbortError 源定位）——C 类余项。

## 质量门禁

- `make test`：全绿 **20,319 P / 0 F**（轮内 r3065 双 commit 回归被门禁当场捕获修复；
  renderer 时序钉负载 flake 两例 solo 复跑即绿——兄弟流 crate 归因同先例）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
