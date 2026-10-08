# M2-S4H — anchor downloadRequest + 锚点击通用导航面（2026-10-09）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4h-full-corpus.txt](2026-10-09-m2-s4h-full-corpus.txt)
**前序**: [2026-10-09-m2-s4g-precommit-traverseto.md](2026-10-09-m2-s4g-precommit-traverseto.md)

## 切片内容（master 下一步计划 ①——anchor-download 2T + 锚点击族连带收口）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **anchor download 属性 → navigate downloadRequest**：A/AREA 点击读 `download`（expando 优先——IDL 非 反射、attr 桥回落 presence 判定）→ 线程 `_navFireNavigate`（读后即清）；非 hash download 锚走通用锚导航 | `part04.js` + `part02.js` | WPT navigate-anchor-download ×4（downloadRequest ''/filename） |
| **通用锚导航 helper `_navAnchorNavigate`**（A/AREA 非 hash href）：fire navigate（sameDocument=非 hash-only、**canIntercept=同源可重写**、sourceElement）→ preventDefault 取消 → intercept 同文档提交链 → download 未拦截 = 吞导航（永不结算）→ 其余 host 真导航；**同 URL 点击 → navigationType 'replace'** | `part02.js` + `part04.js` + `part03.js`（plain-node 激活路径同面） | WPT navigate-anchor-cross-origin（canIntercept false）/ -same-url（replace）/ anchor-download-intercept ×2 |
| **destination.sameDocument 按导航本源**：hash-only/pushState=true、非 hash href/assign/replace/navigate=false（原硬编码 true） | `part02.js` | WPT navigate-anchor-download「sameDocument false」 |
| **userInitiated ← 瞬态激活**：fire 时读 `_zwTransientActive`（testdriver click 签发）+ 读后清（一次激活归一次导航） | `part02.js` | WPT navigate-anchor-download-userInitiated / navigate-anchor-userInitiated |
| **锚激活 sourceElement 物化**：`_makeProxy(sel, handle)`（querySelector 形态漏 handle-identified 的 createElement 锚）；part03 plain-node `click()` 激活路径同面补线程 | `part04.js` + `part03.js` | WPT navigate-anchor-userInitiated「sourceElement === a」 |

## 定位过程记录（一处非显然根因）

test_driver 合成 click 的锚导航走 **plain-node `click()` 激活路径**（part03 R152——与 part04
代理侧分支并存），初版只线程了 part04 → testdriver 面 sourceElement/download 恒 null。
probe 实证「单次 fire、dl=null、se=null」后定位双路径，两处同面补齐。

## 数字

| corpus 域 | S4G 后 | 本轮 | Δ |
|---|---|---|---|
| navigation-api 全域 | 159/228 = 69.7% | **167/230 = 72.6%** | +8 |
| └ anchor-download-intercept{,-reject} | 2 Timeout | **2 Pass** | — |
| └ navigate-anchor-download ×4 | 1F+3T | **4 Pass** | — |
| └ navigate-anchor-same-url / -userInitiated / -fragment 等 | Timeout/Fail | Pass（same-url、fragment；userInitiated 余 sourceElement 断言见挂账） | — |
| 全量 | 298 P / 442 = 67.4% | **306 P / 442 = 69.2%** | +8 |

**零回归**：全量 per-subtest 精确 diff（S4G vs S4H）——Pass→Fail/Timeout 为 0。

## 挂账

- **host 激活路径锚线程**（下一步 S4I）：testdriver click 的 host `Activate` 管线
  （`PageEffect::SetFragment`/`Navigate`——`script_call_set_location_hash` / `anchor_click_target`）
  绕过 JS 锚分支 → sourceElement/downloadRequest 恒缺（navigate-anchor-userInitiated 余 1 断言、
  navigate-anchor-cross-origin / -same-origin-cross-document / -download-userInitiated 3T）。
  需 user_actions.rs 前置线程脚本 + `_navAnchorNavigate` 全局暴露——webview/engine 薄改。
- 锚激活余项（userInitiated 1F）+ form submit navigate 族（5T——form 面）随 S4I 一并评估。

## 质量门禁

- `make test`：全绿 **20,228 P / 0 F**（锚 20,225 → 20,228 含兄弟流新增）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
