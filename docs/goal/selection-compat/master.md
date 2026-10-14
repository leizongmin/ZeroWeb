# selection-compat — 运行时控制面

> 状态头（2026-10-14——M1 完成，M2 切片 1、2 完成）

**当前状态**：M2 切片 2 完成——`anchor-removal.html` ×2 转绿（2966 →
**2968 Pass / 2 Fail**）。拖选默认动作落地（mousedown collapse /
pointermove extend，坐标命中 + sel 键回落 + body/html 零布局门），
`__zwSelectionFollowRemoval` 匹配器补 NA 族桥（match-only），吸附目标保持
`oldParent262` 原对象；part04 R3K 换代刷新修同名 global 世代滞留（测试 2
remove 不触达活树的根因，跨段共享根因）。家族分立与回归修复链见
[evidence/2026-10-14-m2s2-anchor-removal.md](evidence/2026-10-14-m2s2-anchor-removal.md)。
engine lib 3044/0。切片 1（toString rendered text，2965 → 2966）与已知
Chromium 换行分叉记档
[evidence/2026-10-14-m2s1-tostring.md](evidence/2026-10-14-m2s1-tostring.md)。

**卡点**：本切片无。uievents 暴露检查按 2026-10-11 fileapi 行记账以
TIME_LIMIT=2400 复跑（无基线侧，结果补记 evidence 门禁节）。

**下一步**：M2 余量收口评估——Document-open（跨域挂账，待
document.open 落地复测）与 editing/event（排除域）之外 corpus 已全绿；
评估是否进 M3（分级通过率两轮不下行 + 账本纪律 + make test 全绿 +
make reftest 零回归）。

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
| M2 语义收敛 | 进行中（切片 1 ✅ 2026-10-14：2966/4；切片 2 ✅ 2026-10-14：2968/2） | [evidence/2026-10-14-m2s1-tostring.md](evidence/2026-10-14-m2s1-tostring.md)、[evidence/2026-10-14-m2s2-anchor-removal.md](evidence/2026-10-14-m2s2-anchor-removal.md)；余：分级通过率两轮不下行 → M3 |
| M3 收口归档 | 未开始 | DC-3：账本纪律 + make test 全绿 + make reftest 零回归 |

## 碰撞管理（run-rules §9）

本 goal 工作面：`crates/engine/src/js_dom_shim/part06.js`（Selection 段）、
`tests/wpt-runner/wpt-data/selection/`（只增）、`docs/goal/selection-compat/`、
`docs/compat/trends/wpt-suites.csv`（本 goal 行）。已归档 js-dom（Range 遗产）
与 editing（编辑面）不再动工；渲染面（选区高亮 ::selection）归
rendering-compat（R5052~R5086 已有系列修复），本 goal 不触碰 paint 路径。
