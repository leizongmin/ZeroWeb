# M2-S4L — transition 派发时机对齐（dispatch 后 commit 前）+ ordering CCE 变体簇收口（2026-10-09）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4l-full-corpus.txt](2026-10-09-m2-s4l-full-corpus.txt)
**前序**: [2026-10-09-m2-s4k-activation-variants.md](2026-10-09-m2-s4k-activation-variants.md)

## 切片内容（master 下一步计划 ①——ordering `?currententrychange` 变体 Fail 簇 ~11 案）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **transition 创建时机**：由 `_navRunIntercept`（commit 后）**前移至 `_navFireNavigate` dispatch 后**（spec inner fire step 29——CCE 派发时 `navigation.transition` 须已暴露）；`_navRunIntercept` 复用同一对象/结算钩子（防御兜底缺省创建）；顺修 head 内重复 `var settled` 声明与丢失的 `var idx` | `part02.js` | spec inner navigate event firing；WPT ordering `?currententrychange` 变体族（Recorder 于 CCE 监听器读 navigation.transition） |

## 定位过程记录

初版插入点误置 **dispatch 前**——`intercept()` 尚未调用、`_zwIntercepted` 恒 false、transition
恒不创建（probe 实证 nav/cce/after 三点全 null）；head 修复版又漏 `var idx` 声明致 handler 链
全断（18 Timeout）。两轮 probe 定位后正解：dispatch 后创建 + 复用。

## 数字

| corpus 域 | S4K 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api 全域 | 190/255 = 74.5% | **200/255 = 78.4%** | +10 |
| └ ordering `?currententrychange` 变体 | 11 Fail | **11 Pass**（余 reentrant 双变体 2T 见挂账） | — |
| └ ordering 基础面 | — | intercept-async?cce 等连带收口 | — |
| 全量 | 329 P / 476 = 69.1% | **339 P / 476 = 71.2%** | +10 |

**零回归**：全量 per-subtest 精确 diff（S4K vs S4L）——Pass→Fail/Timeout 为 0。

## 挂账

- `location-href-intercept-reentrant` / `navigate-same-document-intercept-reentrant`
  双变体 2T：re-entrant 嵌套 fire 的 CCE/transition 时序（嵌套导航于 handler 内再 fire
  —— ongoing-abort 循环 + transition 覆盖序），C 类余项。
- state same-document-away-and-back ×2、dispose 深簇：同前记账。

## 质量门禁

- `make test`：全绿 **20,317 P / 0 F**（run 1 中 renderer R2946 onload 时序钉负载 flake
  solo 复跑即绿——兄弟流 crate，同 S4I/S4J 归因先例；复跑全绿）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
