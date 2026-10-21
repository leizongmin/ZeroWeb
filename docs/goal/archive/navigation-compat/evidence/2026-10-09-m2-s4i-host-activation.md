# M2-S4I — host 激活路径锚线程（2026-10-09）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4i-full-corpus.txt](2026-10-09-m2-s4i-full-corpus.txt)
**前序**: [2026-10-09-m2-s4h-anchor-download.md](2026-10-09-m2-s4h-anchor-download.md)

## 切片内容（master 下一步计划 ①——testdriver click 的 host Activate 管线锚线程）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **`__zwNavAnchorNavigate` 全局暴露**：`_navAnchorNavigate` 改**状态回传**（'canceled'/'intercepted'/'download'/'host'——host 导航决策外移给调用方）；part04 a.click() 路径按 'host' 回调 `__zw_request_navigate` | `part02.js` + `part04.js` | spec navigate event default 处理 |
| **SetFragment 锚线程**：`PageEffect::SetFragment` 执行前把触发元素 + download 属性线程进页面（`__zwNavSourceElement`/`__zwNavDownloadRequest` 前置脚本，执行后清除）——host hash 导航的 navigate 事件自此带 sourceElement/downloadRequest | `crates/webview/src/webview/user_actions.rs` | WPT navigate-anchor-userInitiated（sourceElement === a + userInitiated true） |
| **Navigate effect 锚预导航**：`PageEffect::Navigate`（锚激活时）先走 `__zwNavAnchorNavigate`——状态 ≠ 'host' → 跳过 host 导航（canceled/intercepted/download 页面已消化；intercepted 同步 host current_url）；='host' → 既有 host 导航。统一 `execute_dom_script`（内部沙箱/外部 executor 两态，DomScriptResult.value 读状态） | `user_actions.rs` + `crates/engine/src/js_dom_bridge.rs`（`anchor_activation_matches`/`anchor_download_request` 新 helper） | WPT navigate-anchor-download-userInitiated / navigate-anchor-cross-origin 面 |
| **锚 href expando-first 读**（part04 点击分支）：`a.href = v` IDL 在静态主文档锚上不落 attr（expando）→ own-property 判定优先读 expando，attr 桥回落；修复静态锚 href 读空导致的导航缺发 | `part04.js` | WPT navigate-anchor-cross-origin 前置（href 线程面） |

## 定位过程记录

1. **testdriver click 无事件**：host `Activate` 管线（`anchor_hash_target`/`anchor_click_target` →
   `PageEffect::SetFragment`/`Navigate`）完全绕过 JS 锚分支——probe 实证单次 fire 且
   sourceElement/downloadRequest 恒空 → host 层前置线程 + 状态回传设计。
2. **静态锚 href expando**：BODY dump 实证 `a.href` 赋值后读回空——`href` IDL 在主文档代理上
   无 setter 落 attr（expando 形态）；hasOwnProperty 门防误读原型 accessor 的 resolved URL
   （首版 expando-first 曾破 fragment 锚——hasOwn 门修复）。

## 数字

| corpus 域 | S4H 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api 全域 | 167/230 = 72.6% | **169/230 = 73.5%** | +2 |
| └ navigate-anchor-userInitiated | 1 Fail | **Pass** | — |
| └ navigate-anchor-download-userInitiated | 1 Timeout | **Pass** | — |
| 全量 | 306 P / 442 = 69.2% | **308 P / 442 = 69.7%** | +2 |

**零回归**：全量 per-subtest 精确 diff（S4H vs S4I）——Pass→Fail/Timeout 为 0。

## 挂账

- `navigate-anchor-cross-origin` 1T / `navigate-anchor-same-origin-cross-document` 1F：静态主
  文档锚 `a.href = v` IDL 赋值**不落 attr**（expando 已可读但 host `anchor_click_target` 走
  html 快照 attr 通道读不到）——A/AREA href IDL setter 落 attr 面回流 js-dom/element IDL 域。
- form submit navigate 族 5T：form submit → navigate 事件面（S4J 评估）。

## 质量门禁

- `make test`：全绿 **20,233 P / 0 F**（附带记录：renderer `js_worker::reset_purge_retains...`
  在全量并行负载下偶发（120ms sleep 时序面、solo 82/82 过）——兄弟流 crate 负载 flake，按
  归因纪律记录不修；本轮复跑全绿实证）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning（修 Navigate arm 穷尽
  match 后的 unreachable catchall）。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
