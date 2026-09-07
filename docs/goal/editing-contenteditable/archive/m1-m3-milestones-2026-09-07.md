# M1-M3 里程碑归档（2026-09-07）

> 归档区域：只追加不修改。M1/M2/M3 的详细过程与证据（只读快照）——
> 运行时状态与后续决策见 `master.md`（控制面板）与 `evidence/`（验证证据）。

## M1 — WPT selection/editing 基线 + Selection 面摸底（2026-09-07）

**目标**：导入 `selection` + `editing` 用例记录基线；摸清 `window.getSelection()` 现状。

### 切片 1 — selection 用例导入 + 基线

- 20 用例导入（fetch-selection-subset.sh + `testharness-selection` 子命令 +
  Makefile 入口 `make testharness-selection`）
- 基线：**45P/2559F（1.7%）** @ WPT_REV 315976933870
- evidence：`evidence/2026-09-07-m1-slice1-selection-baseline.md`

### 切片 2 — Selection 面缺口修复（commit 679059d2f）

8 类修复（45P→**1824P，70.2%**）：

1. document.getSelection 绑定（与 window.getSelection 同一 Selection 单例）
2. instanceof 链（Selection.prototype 接通）
3. 空选区 collapseToStart/collapseToEnd 抛 InvalidStateError
4. getRangeAt 越界抛 IndexSizeError
5. removeRange 非成员抛 TypeError / NotFoundError 语义
6. collapseToStart 返回新 range 语义
7. collapsed 选区 toString 返空串
8. selectAllChildren / setBaseAndExtent / setPosition / deleteFromDocument 新增

单测 `test_selection_surface_r3254_m1` 九组断言。

**残余聚类**（defer 有据）：selectAllChildren 456F「createRange of undefined」=
runner host 视图 pending-tree 变动伪失败（js-dom 共享面已知限制）；deleteFromDocument
60F iframe 面依赖；selectionchange 派发；setBaseAndExtent 方向位。

### 切片 3 — editing 用例导入 + 失败聚类

- editing/event.html（beforeinput/input 事件面）+ other/ 两案导入
  （fetch-selection-subset.sh EDITING_CASES 段）
- 聚类结论：event.html 76F 全为 execCommand 不派发 input 事件 = E4/E5 缺口
  （M2/M3 领域）；组合面 1930P/854F

## M2 — 编辑行为管线（2026-09-07）

### 切片 1 — execCommand editing-host 事件序（commit 74a0c879b）

- 编辑类命令（_zwExecCmdInputType 表：format/insert/delete 族）在 editing host 内
  派发 beforeinput（cancelable + trusted，preventDefault 阻断 → 不派 input 不变更）
  + input（bubbles、不可取消、trusted、inputType 映射）
- 前提判定：整个选区在单一 editing host 内（R3187 空串 contenteditable ≡ true）
- event.html **104P/76F → 179P/1F（99.4%）**
- 单测 `test_execcommand_editing_host_events_r3254_m2` 六组断言
- 残余 1F = Changing-selection target 身份比较（上游 legacy 形态，记录不追）

### 切片 2 — contenteditable 键入/删除落 DOM（commit bf181ee60）

- shim `__zw_ce_insert` / `__zw_ce_delete`：Selection caret 处 insertNode/splice
  → SetChildText mutation 流转宿主；beforeinput/input 事件序
- webview user_actions.rs CE 分支：焦点元素为 CE 宿主时走 CE 管线（不落
  TextActionState 的 input/textarea value 模型）
- 单测 `test_contenteditable_typing_r3254_m2` 五组断言

### 切片 3 — CE Enter 换行落 DOM（commit c368e02df）

- shim `__zw_ce_enter`：caret 处 `<br>` 插入（innerHTML splice + 实体感知偏移扫描）
  → SetInnerHtml mutation
- InsertText{"\n"} 与无表单 Submit 双路由接通（webview 层分发）
- 单测 `test_contenteditable_enter_r3254_m2` 四组断言
- **限制记录**：insertParagraph 块级拆分 defer（SetInnerHtml 单域内块结构拆分需
  父域选择器）；跨节点回退/嵌套结构偏移映射 defer

## M3 — execCommand 基础面 + 收尾（2026-09-07）

### 切片 1 — format 四命令实应用（commit bf3338890）

- bold/italic/underline/strikethrough inline 包裹（`<b>/<i>/<u>/<s>`）：
  选区端点 → 宿主 innerHTML 串偏移（标签感知 + `&...;` 实体感知扫描）→ splice
  包裹 → SetInnerHtml mutation；caret 落包裹区末
- **flat 模型**：两端点均在宿主直子文本节点内才应用（嵌套结构偏移映射 defer）
- 单测 `test_execcommand_format_apply_r3254_m3`（flat/嵌套/collapsed 不应用/italic）

### 切片 2 — delete/forwardDelete 实应用（commit 743118629）

- 选区删除：SetChildText splice + 代理对安全；collapsed → back/forward 一单元
- caret 回落删除点（selection 单例直换）
- 单测并入 `test_execcommand_format_apply_r3254_m3`（④⑤ 两组）

### 切片 3 — queryCommandSupported/Enabled 真实反射（commit b8fe8eb78）

- `_zwQueryCommandState`：copy/cut/paste 恒 supported+enabled（ClipboardEvent 路径
  R2936）；编辑类 supported，enabled 与 execCommand 编辑分支同前提（选区在单一
  editing host 内）；未接命令 false；大小写不敏感；queryCommandValue 保持 ''
- 替换无条件 `return true` 桩——DC-4 首条硬缺口闭合
- 单测 `test_query_command_reflect_r3254_m3_slice3` 五组断言
- R2826 测试注释同步

**defer 有据**：toggle/queryCommandState、CSS 化命令（color 族）、queryCommandValue
值面、run/ 全量导入（330KB reference impl 依赖——43 用例，非本流域可独立闭环）。

## 最终验证状态（2026-09-07）

- WPT selection/editing 面：**2005P/779F**（20 selection + 3 editing 用例）
- engine 2631 全绿；webview 692 全绿；clippy -D warnings 零警告
- 每个 M2/M3 切片均带单测资产（12 个 r3254 系列 bridge 测试）
