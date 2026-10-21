# M2-S4B — navigate 事件 + navigation.navigate()/reload() + intercept 拦截面（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-08-m2-s4b-full-corpus.txt](2026-10-08-m2-s4b-full-corpus.txt)
**前序**: [2026-10-08-m2-s4-navigation-read-side.md](2026-10-08-m2-s4-navigation-read-side.md)

## 切片内容（master 下一步计划 ①——M2 ④-B 第一刀：同文档 navigate 面）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| `NavigateEvent`（destination/signal 必填 TypeError；defaults navigationType 'push'/canIntercept false/info undefined/sourceElement null 等；cancelable=true UA 面）+ `NavigationDestination`（url/sameDocument/key=''/id=''/index=-1 同文档 push 面） | `crates/engine/src/js_dom_shim/part05.js` | WPT event-constructor 四断言 + navigate-anchor-fragment |
| `intercept()`：synthetic 事件抛 **SecurityError**（isTrusted 面）、!canIntercept / 已 preventDefault 抛 InvalidStateError、handler 显式 null 抛 TypeError、显式 undefined 合法；**多次 intercept 合法 = handler 顺序链**（finished 依序 await） | 同上 | WPT intercept-on-synthetic-event / intercept-handler-null-or-undefined / intercept-multiple-times(-reject) |
| `scroll()`：synthetic 抛 SecurityError；未 intercept 抛 InvalidStateError；intercept 后 headless no-op | 同上 | WPT scroll-on-synthetic-event / scroll-without-intercept |
| navigate 事件在导航算法内**同步**派发（pushState='push'、replaceState='replace'、hash-setter/href/assign='push'、location.replace='replace'）+ preventDefault 中止（无 entry/无 CCE/无 popstate）+ intercept 跳过默认 fragment 滚锚并起 handler 生命周期 | `crates/engine/src/js_dom_shim/part02.js`（五 hook + `_navFireNavigate`/`_navRunIntercept`） | WPT navigate-history-pushState / navigate-history-replaceState / intercept-history-pushState（URL/state/length 同步应用 + 无 popstate）/ intercept-handler-throws |
| `navigation.navigate(url, {state, history, info})`：同文档提交（push/replace + CCE）、committed/finished 双 Promise（取消 → AbortError reject——WPT navigation-navigate-preventDefault）；`navigation.reload({info})` 同面（默认 no-op reapply） | 同上 | WPT navigation-methods/navigate-history-state（history.state 面）/ currententrychange-event/navigation-navigate-* |
| navigateerror ErrorEvent（error/message/filename/lineno/colno——err.stack 末帧解析，空帧/`<anonymous>` 回落页面 URL） | 同上 | WPT intercept-handler-throws / intercept-reject |
| anchor 片段 click 的 sourceElement 线程（R154 路径 sel 选择器经 querySelector 物化保身份） | `crates/engine/src/js_dom_shim/part04.js` | WPT navigate-anchor-fragment `e.sourceElement === getElementById('a')` |

**已知限制（记录，④-C）**：traverse（back/forward/go）不派 navigate（intercept-popstate /
navigate-history-back-after-fragment 等面缺席）；destination.getState()/index 动态重算面
缺席（navigate-destination-getState-* / -dynamic-index）；userInitiated 恒 false（runner 无
user-activation 模型——navigate-anchor-userInitiated 记 runner 缺口）；跨文档 navigate 面
（intercept-cross-origin/-same-origin）未建模；preemption 面（defaultPrevented-navigation-
preempted / intercept-and-navigate）缺席。

## 数字

| corpus 域 | S4 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api | 19/222 = 8.6% | 55/225 = **24.4%** | +36 Pass |
| history/the-history-interface | 45/49 = 91.8% | 45/49 = 91.8% | 0 |
| history/the-location-interface | 31/36 = 86.1% | 31/36 = 86.1% | 0 |
| traversal/history-traversal | 42/45 = 93.3% | 42/45 = 93.3% | 0 |
| 全量 | 148 P = 35.1% | **184 P = 43.3%** | +36 |

零回归：S4 全部 Pass 案本轮全保持（逐案对照；首轮曾因 scroll() 缺 guard 翻掉
scroll-on-synthetic-event 一案——补 SecurityError/InvalidStateError 面后复绿，终轮零回归）。

## 质量门禁

- `make test`：三轮运行各折在**不同的一个**负载敏感用例（双流共用机器并发实测）——
  ① `testharness::tests::send_keys_dispatches_*`（2s 硬编码 case 超时；solo 1.05s 恒过）
  ② `js_worker::tests::reset_purge_*`（renderer 域；solo --lib 2P 过）③
  `webdriver_screenshot_error_and_success_paths`（workspace 腿中止）。三个失败互不重叠、
  均不在本 goal 触碰路径（engine shim / wpt-runner），solo 复跑全绿——归因双流机器负载
  （run-rules §10 归因条款）。**本切片触碰 crate 定向全绿**：zero-engine 2,895 P / 0 F +
  zero-wpt-runner 213 P / 0 F（test-guard 包裹）。
- `cargo fmt --all -- --check`：零 diff（本切片仅 .js 变更）
