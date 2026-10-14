# selection-compat — 运行时控制面

> 状态头（2026-10-14——M1 完成，M2 切片 1 完成）

**当前状态**：M1 导入与基线完成（2965/2970，99.83%）；M2 切片 1 完成——
`Selection.toString` rendered text 语义落地（display:none 剔除 + 空白折叠 +
块级边界折行 + pre 族不合并 + 控件例外回落），`make testharness-selection`
**2966 Pass / 4 Fail**（+1，`script-and-style-elements.html` 转绿）。
Range.toString 保持 raw 语义未动；2 个 bridge 断言随语义迁移更新。
已知互操作分叉（现代 Chromium 对同文档少 2 处换行）记档 evidence。

**卡点**：本切片无。环境面（非本 goal）：integration `css_properties` 8 个
「渲染指示器」测试在 HEAD 既有红（stash A/B 实证与本切片无关，疑似
578bb8c75 波及）——make test workspace 无法全绿，归 render 面处理，见
[evidence/2026-10-14-m2s1-tostring.md](evidence/2026-10-14-m2s1-tostring.md)
门禁节。

**下一步**：DC-2 续切片——`anchor-removal` ×2 的 live-range 边界吸附
（节点移除时选区边界点按 DOM Standard 吸附到父节点；shim mutation 路径）；
拖选命中测试（坐标→文本偏移）体量另估。Document-open 跨域挂账不动。

## 缺口清单（M1 基线定性，2026-10-14；切片 1 后余 4）

| 失败用例 | 定性 | 归属 | 处置 |
|---|---|---|---|
| ~~`selection/script-and-style-elements.html`~~ | ~~toString 未按 rendered text 语义~~ | 本 goal | ✅ **已修复**（M2 切片 1，2026-10-14） |
| `selection/anchor-removal.html` ×2 | testdriver Actions 拖选（pointer 按下-移动-抬起）未构建文本选区（runner Actions shim 无坐标→文本偏移 caret 命中能力，engine 无 caretRangeFromPoint）+ 节点移除时选区未按 live range 规则把边界点吸附到父节点（DOM Standard §mutate 后 range 边界更新） | 本 goal 深水区 | M2 后续切片：先做 live-range 边界吸附（shim mutation 路径），拖选命中测试另立切片评估 |
| `selection/Document-open.html` | `document.open()` 缺失（`iframe.contentDocument.open is not a function`）；用例本体断言 document.open 后 Selection 单例不变 | 跨域挂账（HTML 文档生命周期，navigation/html-syntax 域基建） | 记账不修，待 document.open 落地后复测 |
| `editing/event.html :: Changing selection from handler: input event` | execCommand 触发的 input 事件 target 语义（期望 handler 内改选区后 target 落到最终编辑宿主） | **显式排除域**（goal 边界：execCommand 命令语义归 editing 域，已归档） | 记账不修（继承 editing-contenteditable 已知失败，与 csv 历史行一致） |

## 里程碑

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M1 导入与基线 | ✅ 完成（2026-10-14） | [evidence/2026-10-14-m1-baseline.md](evidence/2026-10-14-m1-baseline.md) |
| M2 语义收敛 | 进行中（切片 1 ✅ 2026-10-14：2966/4） | [evidence/2026-10-14-m2s1-tostring.md](evidence/2026-10-14-m2s1-tostring.md)；余：live-range 吸附切片 → 分级通过率两轮不下行 |
| M3 收口归档 | 未开始 | DC-3：账本纪律 + make test 全绿 + make reftest 零回归 |

## 碰撞管理（run-rules §9）

本 goal 工作面：`crates/engine/src/js_dom_shim/part06.js`（Selection 段）、
`tests/wpt-runner/wpt-data/selection/`（只增）、`docs/goal/selection-compat/`、
`docs/compat/trends/wpt-suites.csv`（本 goal 行）。已归档 js-dom（Range 遗产）
与 editing（编辑面）不再动工；渲染面（选区高亮 ::selection）归
rendering-compat（R5052~R5086 已有系列修复），本 goal 不触碰 paint 路径。
