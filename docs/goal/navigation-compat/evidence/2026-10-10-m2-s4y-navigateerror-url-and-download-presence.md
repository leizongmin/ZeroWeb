# M2-S4Y — navigateerror filename fire 时快照 + 锚 download presence own 门（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案迭代 + 全量两轮
per-subtest 精确 diff——fix#1 单独一轮隔离爆炸半径，再双 fix 终态轮）
**前序**: [2026-10-10-m2-s4x-readystate-host.md](2026-10-10-m2-s4x-readystate-host.md)

## 两个独立根因

### fix#1 — navigateerror filename 回退读派发时刻活 URL（**S4P 回归追认**）

`_navFireNavigateerror` 的 filename 自 `err.stack` best-effort 提取；页面内联脚本经匿名
eval 执行 → 帧文件为 `<anonymous>` → 回退 `__zw_get_page_url()`（**派发时刻活读**）。
S4P 批二给 navigate() API 路径加了提交时 page_url hash-only 同步（`_zwSyncDocUrl` 门）——
`navigation.navigate("#1")` 提交后 handler 拒绝晚于提交，回退读到带片段 URL。此前
location.href="#1" 通道 page_url 不同步故侥幸绿。

- 波及案：`intercept-multiple-times-reject`（S4B~S4O 连续 Pass → S4P 起 Fail 六轮未察）。
- **S4P 回归追认**：S4P md 称该案「连带收口」且 navigation-api 233/255，但其全量 txt
  原始 226/255、该案 Fail——md 与 txt 不符（S4V 同款中途读数问题第二例；S4W 起口径
  勘误「按 txt 原始行计」即为此设，本轮回归由 txt 逐轮追认定位）。
- 修法：`_navFireNavigate` 入口快照提交前 URL 挂 `ev._zwFirePageUrl`，
  `_navFireNavigateerror` 回退优先取快照、活 URL 兜底（无 ev 形态防回归）。
  提交前快照即四个 filename 断言案的共同期望（`start_href`/`start_url`/取消后未变的
  `location.href`）；lineno/colno 仍走 stack 提取不受影响。

### fix#2 — 锚 download presence 误判（proxy 反射空串）

`part04` 锚激活 download 读：expando 优先分支 `_p154d.download !== undefined/null`——
proxy get trap 对缺席 attr 经通用反射返回 `''`（非 undefined）→ 任何无 download 锚都
判 presence=true、`downloadRequest=''`。WPT `navigate-anchor-same-origin-cross-document`
「downloadRequest null」Fail（S4H 起）。

- 修法：href 同款 **own expando 门**（`Object.prototype.hasOwnProperty`——proxy 反射
  attr 不物化 own property，S4I 先例）：own 且非 undefined/null 才走 expando 分支；
  否则回落既有 `__zw_has_attr` presence 判定（`download=""` attr 同下载面不变——
  anchor-download ''/filename ×4 与 anchor-download-intercept ×4 复跑全绿）。
- part03 plain-node 路径同面检查：`node.download` 对缺席 attr 返回 undefined（无反射），
  行为正确，不触碰。

## 数字（按 evidence txt 原始 Pass 行）

| corpus 域 | S4X 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 395 P / 476 = 83.0% | **397 P / 476 = 83.4%** | +2（各归各根因） |
| navigation-api | 235 / 255 | **237 / 255**（92.9%） | +2 |

**零回归**：fix#1 单独轮 diff vs S4X 恰一行（intercept-multiple-times-reject
Fail→Pass）；双 fix 终态轮 diff vs fix#1 轮恰一行（navigate-anchor-same-origin-
cross-document Fail→Pass）。两轮 diff 均逐行比对全量 txt。

## 质量门禁

- `make test`：全绿（数字见本轮终态行；part02/part04.js 变更轮）。
- fmt/clippy 不适用（纯 .js shim 变更，无 .rs 改动）。
