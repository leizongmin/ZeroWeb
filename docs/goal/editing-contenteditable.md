# 编辑与 contenteditable — WPT 驱动的富文本编辑基础目标

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/html-compat.md`（HTML 行为兼容——本专项为其编辑面延伸）

> **说明**
> 本文档是 ZeroWeb「编辑与 contenteditable」专项目标执行契约。目标是把 `contenteditable`
> 元素从属性反射（R3187：枚举状态求值 true/false/inherit——**纯状态读取，无编辑能力**）
> 深化为可用的富文本编辑基础：光标（Selection/Range 可观察）、文本输入（键入/删除/换行）、
> `document.execCommand` 基础命令面、`beforeinput`/`input` 事件。以 WPT `editing` / `selection`
> 真实用例通过率为验证标准。本文定义 Mission、边界、Done Criteria、执行协议和文档治理
> 规则，供后续 `rally run` 会话作为稳定输入。日常进展、evidence、active milestone 更新
> 写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：与 keyboard-default-actions 同批拆出（键盘/编辑
> 方向三拆之二）。理由：① contenteditable 是「可输入网页」的基础（评论框/富文本编辑器/
> Note 类应用全依赖），当前 ZeroWeb 只有属性反射——页面看似可编辑、实际键入无反应；
> ② 工作面清晰（Selection/Range + 文本变更管线），与 js-dom 流的 Range/TreeWalker
> （zero-dom range.rs 952 行——deep-review 已确认健壮）天然衔接但不撞其 dom_bindings 面；
> ③ 上游 WPT `selection` + `editing` 目录用例量厚，独立验收面。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **contenteditable 反射**：R3187（part01.js:328）——枚举状态求值（true/false/inherit，
>   空串≡true）getter/setter 完整。**无任何编辑行为**（键入不落到 DOM）。
> - **execCommand**：part06.js:1432 附近有 `document.execCommand` 桩（copy/cut 返 true
>   语义；**format 类命令不真应用**——注释明言）。
> - **Selection/Range**：zero-dom `range.rs`（952 行）Range API 健壮（R3377 deep-review
>   确认；跨容器 branch 已知简化记录在案）；JS 侧 `window.getSelection()` 面未核实——
>   M1 首项即摸清。
> - **page_selection.rs**：browser 侧已有文本选区基础设施（页面选择/复制的宿主侧底座）。
> - **WPT 面**：`editing` / `selection` 目录未导入，无基线。

---

## Mission

以 **WPT `selection` / `editing` 真实用例通过率为验证标准**，把 contenteditable 元素从
属性反射深化为可用编辑基础：光标可观察（Selection/Range）、键入/删除/换行落到 DOM、
`beforeinput`/`input` 事件真实派发、`execCommand` 基础命令真应用。分阶段里程碑校准执行
预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 导入 `selection` + `editing` 范围内用例 + 通过率基线（当前无基线） |
| 中期 | **选区与光标全通** | Selection/Range 的 JS 可观察面（getSelection/collapse/extend/rangeCount/anchorNode 等） |
| 长期 | **可编辑落地** | 键入/删除/换行改 DOM + beforeinput/input 事件 + execCommand 基础命令（bold/italic/insertText 等） |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（同 canvas-2d /
form-validation——不允许手写 inline 用例替代或充数）。

覆盖范围：

1. **Selection API** — `window.getSelection()`（rangeCount/anchorNode/anchorOffset/
   focusNode/focusOffset/isCollapsed/collapse/collapseToEnd/collapseToStart/extend/
   selectAllChildren/toString/removeAllRanges/addRange/type）
2. **Range 与选区联动** — Range 在选区中的进出（addRange/removeAllRanges 后的
   getRangeAt）；zero-dom range.rs 既有能力接到 JS 面
3. **contenteditable 编辑行为** — 聚焦 + 键入/删除/Backspace/Enter 换行落到 DOM
  （文本节点分裂/合并——range.rs insert_node 已有字符偏移分裂底座）；光标位置随编辑移动
4. **编辑事件** — `beforeinput`（inputType/delete/insertText 等 + cancelable 阻断）/
  `input` 事件派发序
5. **execCommand 基础面** — queryCommandSupported/queryCommandEnabled 反射真实化；
  bold/italic/underline/insertText/insertParagraph/delete 的最小真应用（format 类从
  最常用子集起步，不追求全命令面）
6. **可编辑宿主联动** — 键入触发重渲染（编辑后的 DOM 变更走既有重渲染管线）与
   IME 组合输入的最小兼容（compositionstart/end 不破坏状态——host-runtime IME 已有底座）

执行方式：**交替推进** — 每轮同时扩展 WPT 导入范围和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| Selection JS 面 | getSelection 全 API 接到 zero-dom 选区模型 | page_selection.rs 宿主侧底座 + range.rs 模型复用 |
| 编辑行为 | 键入/删除/换行的 DOM 变更管线 | 文本节点分裂/合并复用 range.rs 既有能力 |
| 编辑事件 | beforeinput/input 派发与阻断 | 事件派发走既有管线 |
| execCommand | 基础命令真应用（最小常用集） | part06.js 桩替换；全命令面非目标 |
| WPT 基础设施 | `selection`/`editing` 用例导入、通过率报告 | 复用 tests/wpt-runner + `make import-wpt` |
| 单元测试 | 每项修复带单测 | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **表单控件键盘默认动作**（Enter 提交/空格激活）— 兄弟目标 `keyboard-default-actions.md`
- **页面滚动键** — 兄弟目标 `keyboard-page-scrolling.md`
- **剪贴板集成**（copy/cut/paste 的 OS 剪贴板）— navigator.clipboard 已有独立面（R2817/
  R2964）；paste 进编辑器的管线属后续扩展
- **完整富文本命令面**（justify*/formatBlock/createLink 全量、styleWithCSS、undo 栈）—
  长期扩展，基础面达标即可
- **Spellcheck / 拼写下划线** — 远期
- **`<textarea>`/`<input>` 内部编辑模型**（光标绘制/选区高亮渲染）— 渲染域；本目标只保证
  JS 可观察面 + DOM 状态正确

### 依赖约束

- **与 js-dom 流碰撞管理**：Selection/Range 的 JS 绑定若进 `js_dom_shim` DOM 反射段，开工前
  先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/` 核对；该流活跃则先做
  零碰撞面（zero-dom 选区模型补强、WPT 导入、编辑管线 host 层）。
- **与 keyboard-default-actions 的边界**：本目标只处理**编辑宿主内**的键（键入/删除/换行）；
  非编辑键（Tab 走焦点、Enter 在非编辑宿主走提交）归兄弟目标。分发顺序：编辑宿主优先
  消费，未消费才走默认动作。

---

## 当前能力/缺口基线

**详见** [editing-contenteditable/master.md](editing-contenteditable/master.md)（运行时
控制面板，唯一真实状态来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **contenteditable 反射**：R3187 枚举状态求值（true/false/inherit）getter/setter
- ✅ **Range 模型**：zero-dom range.rs（952 行）健壮（R3377 确认；insert_node 有文本
  节点字符偏移分裂底座）
- ✅ **宿主选区底座**：page_selection.rs（browser 侧文本选区基础设施）
- ⚠️ **缺口 1 — 编辑行为为零**：键入/删除/换行不落到 DOM（无编辑管线）
- ⚠️ **缺口 2 — Selection JS 面未核实**：`window.getSelection()` 可观察面待 M1 摸底
- ⚠️ **缺口 3 — beforeinput/input 事件缺失**
- ⚠️ **缺口 4 — execCommand format 桩**：命令不真应用（注释明言）
- ⚠️ **缺口 5 — WPT 覆盖为零**：`editing`/`selection` 未导入，无基线

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 用例导入与通过率基线

- [ ] 导入上游 `selection` + `editing` 范围内真实用例（skip list 有据）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经 `make import-wpt` 常驻断言集并记入 `imported-tests.txt`
- [ ] 通过率报告持久化到 `docs/goal/editing-contenteditable/evidence/`，历史可追溯

### DC-2: Selection API 可观察面

- [ ] getSelection 全 API（collapse/extend/selectAllRanges/toString 等）与 spec 一致
      （WPT 为准）；与 Range 进出联动正确

### DC-3: 编辑行为落地

- [ ] contenteditable 宿主：键入/删除/Backspace/Enter 的 DOM 变更正确（文本分裂/合并），
      光标随编辑移动
- [ ] beforeinput（cancelable）/input 事件按 spec 派发

### DC-4: execCommand 基础面

- [ ] queryCommandSupported/queryCommandEnabled 真实反射
- [ ] bold/italic/underline/insertText/insertParagraph/delete 最小真应用

### DC-5: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT selection/editing 基线 + Selection 面摸底

**目标**：导入用例记录基线；摸清 `window.getSelection()` 现状（有/无/近似面）。

**切片建议**：
1. `selection` 用例导入 + 基线（Selection 面是编辑的前置——先立可观察面）
2. getSelection 全 API 接 zero-dom 选区模型（缺失则建）
3. `editing` 用例导入 + 失败聚类

### M2 — 编辑行为管线

**目标**：键入/删除/换行落 DOM + 光标移动 + beforeinput/input 事件。

### M3 — execCommand 基础面 + 收尾

**目标**：命令真应用（最小集）+ 剩余用例修复 + 通过率达标（阈值按 M1 基线校准）。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~5 全部满足；验证基于上游真实 WPT 用例（无内建 inline 充数）；
`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**Selection/Range 的 JS 可观察面现状与 zero-dom 选区模型的差距
2. **自主导入** WPT selection/editing 用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（API 缺失？编辑管线？事件序？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`cargo test` + clippy + WPT 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 js-dom 共享面（js_dom_shim DOM 反射段）前先 `git log` 核对；有活跃
   编辑则转零碰撞面（zero-dom 模型、WPT 导入、编辑管线 host 层）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。
2. **用例失败分析**：每个失败 case 必须分析根因（Selection 面？文本变更？事件？命令？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/editing-contenteditable/master.md`：当前真实状态的唯一
  控制面板。治理规则：持续演进、不允许无限增长、各章节必须自洽。
- **归档区域** `docs/goal/editing-contenteditable/archive/`：存储已完成里程碑的详细过程与
  历史证据，只追加不修改。
- **证据区域** `docs/goal/editing-contenteditable/evidence/`：存储通过率报告、失败分析等
  验证证据，持续追加。
