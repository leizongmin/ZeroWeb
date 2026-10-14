# selection-compat — 运行时控制面

> 状态头（2026-10-15——**goal 收口**：M1/M2/M3 全部完成，DC-1/2/3 满足）

**当前状态**：M3 收口完成（[evidence/2026-10-15-m3-close.md](evidence/2026-10-15-m3-close.md)）。
分级通过率连续两轮不下行（2966 → 2968 → 2968，收口复跑与切片 2 失败集
完全一致）；make test 全绿（71 腿 0 失败——既有环境红 css_properties 由
upstream 8216f9919 修复解除）；make reftest 100.0% 零不一致；product-smoke
welcome 15.35% ≤ 20%（struct 全 PASS + form-input-perf PASS）。终态
**2968/2970（99.93%）**。M2 切片 2（anchor-removal ×2 拖选默认动作 +
R3K 换代刷新）与切片 1（toString rendered text）证据：
[evidence/2026-10-14-m2s2-anchor-removal.md](evidence/2026-10-14-m2s2-anchor-removal.md)、
[evidence/2026-10-14-m2s1-tostring.md](evidence/2026-10-14-m2s1-tostring.md)。

**卡点**：无。残余 2 失败均不属本 goal 语义面：Document-open 待
document.open 落地后由 navigation/html-syntax 域复测；editing/event 为
goal 边界排除域（execCommand 归 editing 域）。uievents 套件基线为跨 goal
记账（csv fileapi 行），不入本 goal 关账条件。

**下一步**：无——goal 关账。Document-open 复测触发条件（document.open
落地）与 uievents 基线记账归各自域；本控制面转存档态，不再滚动更新。

## 缺口清单（M1 基线定性，2026-10-14；切片 2 后余 2）

| 失败用例 | 定性 | 归属 | 处置 |
|---|---|---|---|
| ~~`selection/script-and-style-elements.html`~~ | ~~toString 未按 rendered text 语义~~ | 本 goal | ✅ **已修复**（M2 切片 1，2026-10-14） |
| ~~`selection/anchor-removal.html` ×2~~ | ~~Actions 拖选未构建文本选区 + 节点移除时边界点未吸附~~ | 本 goal | ✅ **已修复**（M2 切片 2，2026-10-14——拖选默认动作 + R3K 换代刷新 + NA 族桥） |
| `selection/Document-open.html` | `document.open()` 缺失（`iframe.contentDocument.open is not a function`）；用例本体断言 document.open 后 Selection 单例不变 | 跨域挂账（HTML 文档生命周期，navigation/html-syntax 域基建） | 记账不修，待 document.open 落地后复测 |
| `editing/event.html :: Changing selection from handler: input event` | execCommand 触发的 input 事件 target 语义（期望 handler 内改选区后 target 落到最终编辑宿主） | **显式排除域**（goal 边界：execCommand 命令语义归 editing 域，已归档） | 记账不修（继承 editing-contenteditable 已知失败，与 csv 历史行一致） |

## 里程碑

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M1 导入与基线 | ✅ 完成（2026-10-14） | [evidence/2026-10-14-m1-baseline.md](evidence/2026-10-14-m1-baseline.md) |
| M2 语义收敛 | ✅ 完成（切片 1：2966/4；切片 2：2968/2；收口复跑 2968/2 两轮不下行） | [evidence/2026-10-14-m2s1-tostring.md](evidence/2026-10-14-m2s1-tostring.md)、[evidence/2026-10-14-m2s2-anchor-removal.md](evidence/2026-10-14-m2s2-anchor-removal.md) |
| M3 收口归档 | ✅ 完成（2026-10-15：make test 全绿 71 腿 + reftest 100.0% + product-smoke 15.35% ≤ 20% + 账本纪律） | [evidence/2026-10-15-m3-close.md](evidence/2026-10-15-m3-close.md) |

## 碰撞管理（run-rules §9）

本 goal 工作面：`crates/engine/src/js_dom_shim/part06.js`（Selection 段）、
`tests/wpt-runner/wpt-data/selection/`（只增）、`docs/goal/selection-compat/`、
`docs/compat/trends/wpt-suites.csv`（本 goal 行）。已归档 js-dom（Range 遗产）
与 editing（编辑面）不再动工；渲染面（选区高亮 ::selection）归
rendering-compat（R5052~R5086 已有系列修复），本 goal 不触碰 paint 路径。
