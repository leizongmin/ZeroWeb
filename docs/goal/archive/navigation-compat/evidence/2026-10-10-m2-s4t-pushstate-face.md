# M2-S4T — pushState/replaceState navigate hashChange 面（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹）
**全量运行日志**: [2026-10-10-m2-s4t-pushstate-face.txt](2026-10-10-m2-s4t-pushstate-face.txt)（终跑）；per-subtest 精确 diff 对 [2026-10-10-m2-s4r-full-corpus.txt](2026-10-10-m2-s4r-full-corpus.txt)（S4R 终态）
**前序**: [2026-10-10-m2-s4r-fragment-indicated.md](2026-10-10-m2-s4r-fragment-indicated.md)

## 切片内容

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **pushState/replaceState 的 navigate 事件 hashChange 恒 false**（原按 `_navIsHashOnly(oldHref, abs)` 计算——url 仅差 fragment 时误 true）。spec hashChange = fragment **navigation** 面专属，pushState/replaceState 即便 url 只差 fragment 也不是片段导航 | `part02.js`（两处 fire） | WPT navigate-history-pushState / navigate-history-replaceState「pushState(1, null, '#1') → assert_false(e.hashChange)」 |

## 数字

| corpus 域 | S4R 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api（记账链延续） | 236/255 = 92.5% | **238/255 = 93.3%** | +2 |
| 全量 | 377 P / 476 = 79.2% | **379 P / 476 = 79.6%** | +2 |

**2 翻 Fail → Pass，零回归**（per-subtest 精确 diff，F/T 集合恰删 2 行零新增）：
navigate-history-pushState + navigate-history-replaceState。

## 首跑回归与挂账定稿（intercepted traverse popstate 时序）

首跑曾试**宏任务化** intercepted traverse 的 popstate/hashchange（spec popstate 为 task）——
收 intercept-popstate-no-handler 1F 的**同一跑**破 ordering-and-transition/
currententrychange-before-popstate-intercept 绿面（其要求 finished 结算时 popstate 已发）。
两案结构相同仅期望相反：ordering 要求 finished 前**已发**、no-handler 要求 finished 后**才发**
——Chromium 经其 task 管线排序同时满足；本沙箱「commit 后微任务 popstate + 空链 finished
同步结算」模型二者互斥，宏任务化收 no-handler 即破 ordering 族。**维持微任务、no-handler
记账**（同 navigate-multiple 先例：task 管线重做风险 > 1F 收益；注记已入
`_hist_dispatchPopState` 头）。

## 本轮新挂账

- **intercept-popstate-no-handler 1F**：见上——与 currententrychange-before-popstate-intercept
  绿面时序互斥，随 task 管线重做（navigate-multiple 同族）一并评估。
- **form-submit-and-window-stop 1F**：引用 `resources/helpers.js` 在 WPT pin 修订版与
  master 均 404——真资产偏斜，归入 same-url-replace 双案同族挂账（上游移除的文件）。
- **navigate-svg-anchor-fragment 1T**：`dispatchEvent(new MouseEvent('click'))` 于 svg:a 无
  激活行为（HTML 锚 dispatched click 走 R152 plain-node 面；svg:a 代理/dispatch 集成缺失）
  ——SVG 元素兼容域，随 svg-compat 域推进。

## 质量门禁

- `make test`：全绿 **20,363 P / 0 F**（实跑）。
- clippy/fmt：本轮 diff 纯 JS（part02.js）+ 文档，无 `.rs` 变更（S4R 片 clippy -D warnings
  干净、fmt 零 diff 维持）。
- `git diff --check`：零问题。
