# M2-S4E — Navigation API focus-reset 面 + handler 同步起跑 + 模块静态导入实拉（2026-10-08）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-08-m2-s4e-full-corpus.txt](2026-10-08-m2-s4e-full-corpus.txt)
**前序**: [2026-10-08-m2-s4d-scroll-behavior.md](2026-10-08-m2-s4d-scroll-behavior.md)

## 切片内容（master 下一步计划 ①——④-D 残余 focus-reset 簇）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **handler 同步起跑**：intercept 链从 defer 任务起跑改为**同步**（spec——navigate event intercept commit handler steps 在 commit 事件 prepare-to-run-script 抑制段内执行，handler 于 navigate()/back() 返回前已调） | `part02.js`（`_navRunIntercept` 重构） | WPT change-focus-during-intercept（navigate() 同步返回后即调 intercept_resolve）+ S4D manual-scroll-after-dispatch 同根深化 |
| **focusReset 模式**：intercept({focusReset}) 'after-transition'（缺省）/'manual'；非法枚举值 TypeError（WebIDL enum——scroll 同面补齐） | `part05.js` | WPT basic「Invalid values for focusReset throw」 |
| **焦点变更追踪**：`_navFocusChanged`（dispatch 置位起点）+ el.focus()/el.blur() 经 `_navMarkFocusChanged` 记变；结算时**有变或 manual → 跳过重置** | `part02.js` + `part04.js` | WPT change-focus-*（focus() during intercept → 不重置） |
| **结算焦点重置**：finish event 步骤 4——先于 scroll 与 navigatesuccess/navigateerror；目标 = 文档序首个 [autofocus] 委托或 body；body 走**完整 focusing 序**（blur/focusout 落旧焦点——可重入导航；focus/focusin 落 body proxy，与 document.body 同源缓存 identity） | `part02.js`（`_navMaybeResetFocus`） | WPT focus-reset-timing（before navigatesuccess/navigateerror）+ reentry-from-focus-reset-navigate-api-tracker（blur 监听器内 pushState/navigate 重入） |
| **移除聚焦元素 → unfocus**：blur/focusout 落被移元素 + activeElement 回落 body（惰性 isConnected 校验）+ 记焦点变更（结算跳过重置、**无** body focus 事件）；removeChild 与 el.remove() 两路挂钩 | `part02.js`（`_zwUnfocusIfFocused`）+ `part03.js` + `part04.js` + `part06.js` | WPT change-focus-then-remove「Removing the element reset focus / onfocus shouldn't fire」 |
| **模块静态导入实拉**：fetcher 配置时 inline module 的静态 import 由**空存根**升级为递归拉真实源（单 spec 失败回落空存根保旧语义）——旧空存根使 ensure_module_export 报缺 export，模块化 corpus 命名导入全数 compile error | `crates/webview/src/webview.rs`（run_page_scripts is_module 分支） | WPT focus-reset basic/multiple-intercept（import { testFocusWasReset } from resources/helpers.mjs） |
| **fetch 资产**：`navigation-api/focus-reset/resources`（helpers.mjs，pin 315976933） | `tests/wpt-runner/scripts/goals/60-navigation-compat.sh` | 同上 |

## 定位过程记录（两处非显然根因）

1. **module compile error 与文件无关**：helpers.mjs 落地后仍报「does not provide an export」——
   根因在 webview 模块编译预注册：R3093 把静态 import 一律注册为**空串源**，ensure_module_export
   对空源必然失败。实拉修复后 basic/multiple-intercept 进入真实执行（余 Timeout 见挂账）。
2. **focus reset 的事件语义分叉**：body 聚焦须走完整 focusing 序（blur 落旧焦点——reentry 案
   靠 blur 监听器重入导航），而**移除**路径的 unfocus 不得给 body 发 focus 事件（then-remove
   unreached_func 守卫）——两条路径分立实现（`_navMaybeResetFocus` vs `_zwUnfocusIfFocused`），
   以「焦点变更标记」统一跳过判定。

## 数字

| corpus 域 | S4D 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api | 82/227 = 36.1% | **91/227 = 40.1%** | +9 Pass |
| └ focus-reset | 0/8（2 compile error + 6 Fail） | **9 Pass** / 2 Timeout / 7 NotRun | +9 |
| history/location/traversal | 91.8%/86.1%/93.3% | 不变 | 0 |
| 全量 | 211 P / 427 | **220 P / 435 = 50.6%** | +9（语料 +8：focus-reset 资产齐后 basic/multiple-intercept/autofocus 转可执行） |

**零回归**：全量 per-subtest 精确 diff——focus-reset 与 reentry-from-focus-reset 两簇外零状态变化。

## 挂账（focus-reset 余 9 案，均为 DOM 焦点/键盘域缺口）

- `basic.html` / `multiple-intercept.html` 主断言 **Timeout**：依赖 **Tab 键顺序焦点导航**
  （`test_driver.send_keys(TAB)` → tabindex 序下一个可聚焦元素获焦）——shim 无 tabindex
  焦点序（part04 R3247 documented 限制④），键盘焦点导航须独立切片。
- `autofocus.html` 7 案 **NotRun**：依赖 **autofocus 属性 load 期处理**（解析/load 后自动
  聚焦首个 [autofocus]）——promise_setup 卡死在「activeElement === 初始 autofocus 元素」
  前置断言。同属 DOM 焦点域切片。

## 质量门禁

- `make test`：全绿 **20,209 P / 0 F**。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check`：零 diff；`git diff --check` 零 diff。
