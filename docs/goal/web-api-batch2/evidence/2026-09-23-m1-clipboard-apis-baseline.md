# M1 / DC-1 — clipboard-apis window 子集通过率基线

**日期**: 2026-09-23
**套件**: `make testharness-clipboard-apis`（WPT 315976933870b34d6ea30e3f6643403edae678ba）
**原始数据**: [2026-09-23-m1-clipboard-apis-baseline.json](2026-09-23-m1-clipboard-apis-baseline.json)

## 结果（WAB2-M1 基建修复后口径）

| 指标 | 值 |
|---|---|
| 导入用例 | 35 案 fetch / 33 案执行（2 案内容级 skip：iframe 依赖） |
| subtests | **15/71 = 21.1% Pass** |
| 全绿用例 | 5/33（clipboard-events-synthetic、data-transfer-file-list、permissions/readText-granted、permissions/writeText-granted、text-write-read/async-writeText-readText） |
| 非 Pass 状态 | Fail × 54、Timeout × 1、Unsupported × 1 |

修复前口径（基建修复前首跑）：10/77 = 13.0%——24 案折在
`ReferenceError: trySetPermission/tryGrantReadPermission is not defined`（helper 装配层，
非 API 面）。修复后同案推进到真实 API 缺口（`ClipboardItem is not defined` 等）。

## M1 基建修复（WAB2-M1，随本基线同轮落地）

**根因**：`crates/engine/src/js_dom_bridge/script_gen.rs` `script_run_classic_page` 的
strict-eval 顶层声明全局发布扫描（R147/R198/R201/R3254-E2 序列）只认行首
`function`/`class`/`var`/`const`/`let`——**`async function`/`function*` 形态不在清单**，
strict 间接 eval 下困在 eval 作用域不落 globalThis。WPT
`clipboard-apis/resources/user-activation.js` 顶层四 helper（waitForUserActivation /
trySetPermission / tryGrantReadPermission / tryGrantWritePermission）恰为全 `async
function` 形态 → 24 案未触 API 面先折在 helper 装配。

**修复**：`async` 前缀剥离 + 可选 `*`/空白容忍后与 `function` 分支共路导出；
`functional(` 等伪前缀由「剥离后须出现过 `*` 或空白」守卫排除。单测
`test_classic_script_strict_async_generator_globals_wab2m1`（part21.rs）；R147/R198/R201
邻接测试全绿。注记：探针甄别期间「同页非 strict 变体声明同名函数的全局遮蔽」会使
shared-name 探针假绿，多文件矩阵须用唯一名逐案断言。

## 失败聚类（修复后，M2 修齐方向）

| 簇 | 案数 | 形态 | 归属 |
|---|---|---|---|
| `ClipboardItem is not defined` | 13 | ClipboardItem 构造面缺位（write 路径主簇） | M2 |
| `Clipboard is not defined` | 2 案 17 subtest | Clipboard 接口构造/静态面缺位 | M2 |
| read() 返回 undefined 形态 | 2 | read() 返回值/类型化面 | M2 |
| blob 回读类型丢失（`expected "image/png" but got ""`） | 2 案 4 subtest | write(blob) → read() types/Blob 回读 | M2 |
| 权限拒绝不生效（`Should have rejected`） | 3 | readText/writeText denied 桌面缺 NotAllowedError 拒绝 | M2/P3（security-hardening DC-4 对齐点） |
| copy 事件 `isTrusted` false | 1 | 信任链事件语义 | M2 |
| DataTransfer clearData 计数 | 1 | DataTransfer 面小缺口 | M2 顺带 |
| `/common/subset-tests.js` fetch 失败 | 1 | 跨 corpus `/common/` 绝对路径，runner fetcher 不服务 | 记账（runner infra） |
| Unsupported（testdriver 越白名单） | 1 | async-svg-read-write | 记账 |

已实现的真面（基线绿）：writeText→readText 文本回环（含非 Latin-1）、permissions
granted 查询路径、synthetic clipboard events 大半、DataTransfer items 大半。

## 导入面与排除（fetch 脚本显式清单）

- 拉：top-level 26 + events/copy-event + permissions ×4 + text-write-read ×4 + resources ×4
- 不拉（记账）：`*-manual` ×3（需真实用户手势/物理剪贴板）、`detached-iframe/` ×6
  （iframe 面，重入 = iframe 管道）、`permissions-policy/`（Permissions-Policy 头
  infra）、`drag-multiple-urls.html`（DnD 挂账）、`idlharness.https.window.js`
  （wrapper 形态不拉，observers 先例）
- 运行面双保险 skip：`clipboard_apis_case_skipped`（resources/manual/iframe 内容/DnD/
  detached-iframe/permissions-policy）

## 下一步（M2）

1. ClipboardItem + Clipboard 接口面（两主簇 15 案）
2. blob write→read 回读类型
3. 权限 denied 拒绝语义（P3 最小权限查询面，security-hardening DC-4 对齐）
