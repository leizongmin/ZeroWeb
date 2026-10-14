# selection-compat — 运行时控制面

> 状态头（2026-10-14——M1 完成，DC-2 首切片动工）

**当前状态**：M1 导入与基线完成。`make testharness-selection`（test-guard
包裹，release + test-guard 入口）全量跑通：**2965 Pass / 5 Fail / 0 Timeout
（2970 subtests，99.83%）**。5 个失败已逐一定性（见缺口清单）：1 个属本
goal（Selection.toString 渲染文本语义，DC-2 首切片）、1 个属本 goal 深水区
（Actions 拖选无选区构建 + live-range 边界吸附）、1 个跨域挂账
（document.open 缺失）、1 个显式排除域继承（execCommand input 事件语义，
editing 域）。测试资产已就位（`tests/wpt-runner/wpt-data/selection/` 25 文件
+ `editing/`，fetch 通道 `make fetch-wpt-selection`，固定上游 rev 见
`tests/wpt-runner/scripts/goals/lib.sh` WPT_REV）。csv planned 行已回填。

**卡点**：无。

**下一步**：DC-2 首切片——`Selection.toString()` 渲染文本语义
（selection/script-and-style-elements.html）：display:none 子树 Text 剔除 +
空白折叠 + 块级边界折行（`crates/engine/src/js_dom_shim/part06.js`，
Selection 层专用路径，不动 Range.toString 的 raw data 语义）。修复后余 4 失败
（缺口清单分型推进）。

## 缺口清单（M1 基线定性，2026-10-14）

| 失败用例 | 定性 | 归属 | 处置 |
|---|---|---|---|
| `selection/script-and-style-elements.html` | Selection.toString 未按「rendered text」语义：未剔除 display:none 子树文本、未做空白折叠与块级折行 | 本 goal（DC-2 首切片） | 修 part06.js Selection.toString |
| `selection/anchor-removal.html` ×2 | testdriver Actions 拖选（pointer 按下-移动-抬起）未构建文本选区（runner Actions shim 无坐标→文本偏移 caret 命中能力，engine 无 caretRangeFromPoint）+ 节点移除时选区未按 live range 规则把边界点吸附到父节点（DOM Standard §mutate 后 range 边界更新） | 本 goal 深水区 | M2 后续切片：先做 live-range 边界吸附（shim mutation 路径），拖选命中测试另立切片评估 |
| `selection/Document-open.html` | `document.open()` 缺失（`iframe.contentDocument.open is not a function`）；用例本体断言 document.open 后 Selection 单例不变 | 跨域挂账（HTML 文档生命周期，navigation/html-syntax 域基建） | 记账不修，待 document.open 落地后复测 |
| `editing/event.html :: Changing selection from handler: input event` | execCommand 触发的 input 事件 target 语义（期望 handler 内改选区后 target 落到最终编辑宿主） | **显式排除域**（goal 边界：execCommand 命令语义归 editing 域，已归档） | 记账不修（继承 editing-contenteditable 已知失败，与 csv 历史行一致） |

## 里程碑

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M1 导入与基线 | ✅ 完成（2026-10-14） | [evidence/2026-10-14-m1-baseline.md](evidence/2026-10-14-m1-baseline.md) |
| M2 语义收敛 | 进行中 | DC-2：toString 首切片 → live-range 吸附切片 → 分级通过率两轮不下行 |
| M3 收口归档 | 未开始 | DC-3：账本纪律 + make test 全绿 + make reftest 零回归 |

## 碰撞管理（run-rules §9）

本 goal 工作面：`crates/engine/src/js_dom_shim/part06.js`（Selection 段）、
`tests/wpt-runner/wpt-data/selection/`（只增）、`docs/goal/selection-compat/`、
`docs/compat/trends/wpt-suites.csv`（本 goal 行）。已归档 js-dom（Range 遗产）
与 editing（编辑面）不再动工；渲染面（选区高亮 ::selection）归
rendering-compat（R5052~R5086 已有系列修复），本 goal 不触碰 paint 路径。
