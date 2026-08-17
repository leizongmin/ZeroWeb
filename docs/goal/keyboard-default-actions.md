# 键盘默认动作 — WPT 驱动的 HTML 键盘交互正确性目标

**版本**: v1.0
**日期**: 2026-08-17
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/html-compat.md`（HTML 行为兼容——M0-M4 已完成，本专项为其键盘默认动作面延伸）

> **说明**
> 本文档是 ZeroWeb「键盘默认动作」专项目标执行契约。目标是把 HTML 控件的键盘默认动作
> （表单 Enter 提交 / Esc 重置与取消 / 空格与 Enter 激活按钮 / 方向键与 select 展开等
> UA default action）从缺失状态补齐到 spec 水平，以 WPT 真实用例通过率为验证标准。
> 本文定义 Mission、边界、Done Criteria、执行协议和文档治理规则，供后续 `rally run`
> 会话作为稳定输入。日常进展、evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-08-17 用户决策）**：form-validation 拆分时用户已点名「其次键盘默认动作
> （零冲突）」。html-compat M0-M4（FR-001~012）完成且 form-validation 立项后，该流域空闲。
> 键盘默认动作是表单可用性的最后一块：form-validation 管「提交前的校验与阻断」，本目标管
> 「键盘如何触达这些路径」（Enter 提交恰恰要走 form-validation 已建成的 interactive
> validation 管线——两专项天然衔接）。理由：① 零冲突——工作面在 host 输入事件派发层
> （engine + browser 输入路径），不碰 js_dom_shim 的 DOM 反射段；② 用户可直接感知
> （Tab/Enter/Esc/方向键手感）；③ FocusManager（Tab 导航 + tabindex 排序）与 html_actions
> （submit 路径）已有底座。
>
> **▶ 基线事实（2026-08-17 实测）**：
> - **FocusManager**：Tab 导航 + tabindex 排序 + 13 单元测试（M14 可访问性基础轮完成）。
> - **html_actions**：submit 路径 + interactive validation（form-validation M3 接入）已有。
> - **键盘默认动作**：Enter 触发提交、Esc 重置/取消、空格激活 button、方向键移动
>   radio/checkbox 之外的选择面——当前无系统性实现（输入事件经 js_dom_bridge 派发，但
>   keydown 的 default action 分发缺失或零散）。
> - **WPT 面**：无键盘默认动作专项导入（interactive/* 分类有内建用例但非上游真实用例）。

---

## Mission

以 WPT 真实用例通过率为验证标准（`html/semantics/forms/*` 键盘面 + `html/interaction/*`
激活面 + `uievents` 键盘事件面），把 HTML 控件键盘默认动作对齐到 Chromium 水平。分阶段
里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 第一阶段 | **基线建立** | 导入键盘交互相关 WPT 用例 + 通过率基线（当前无基线） |
| 中期 | **表单键全通** | Enter 提交（含 implicit submission 规则）/ Esc 重置与 dialog 取消 / 空格与 Enter 激活 |
| 长期 | **90%+（可校准）** | select 展开/选项导航、radio 方向键组内移动、accesskey、快捷键与修饰键语义 |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（同 canvas-2d /
form-validation——不允许手写 inline 用例替代或充数）。

覆盖范围：

1. **表单提交键** — text input Enter → form submit（implicit submission：单文本控件直接
   提交、多控件时须 submit button；`formnovalidate` 联动 form-validation 管线）
2. **按钮激活** — button/`<input type=button|submit|reset>` 的空格（keydown 默认动作，
   keyup 触发 click）与 Enter（keydown 即 click）语义差异
3. **Esc 语义** — reset 类控件的 Esc 不重置（只有 reset 按钮重置）；`<dialog>` open 时
   Esc 触发 cancel + close（与 R3290 dialog 状态机集成）；`<select>` 展开态 Esc 收起
4. **select 键盘导航** — 展开键（Alt+Down/Up、空格）、选项移动（方向键/Home/End）、
   输入首字符跳转、type-ahead 多字符缓冲
5. **radio/checkbox** — 空格切换 checkbox；方向键在 radio 组内移动并选中（不含点对点
   pointer 面——那是 html-compat pointer 域）
6. **焦点滚动** — 激活/聚焦可聚焦元素的 scroll into view（与兄弟目标
   keyboard-page-scrolling 的滚动管线衔接）
7. **事件序** — keydown → keypress（可选）→ keyup → default action 的顺序与
   `preventDefault()` 阻断语义（cancelable）

执行方式：**交替推进** — 每轮同时扩展 WPT 导入范围和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| 默认动作分发 | keydown 的 default action 分发层（按控件类型 + 按键） | host 输入事件派发层（engine/browser 输入路径） |
| 表单键 | Enter 提交/implicit submission/空格激活/Esc dialog 取消 | 与 form-validation 的 interactive validation 管线衔接 |
| select 导航 | 展开键/选项移动/type-ahead | headless 无真下拉 UI——JS 可观察面（value 变化 + change/input 事件）为验收面 |
| radio/checkbox 键盘 | 空格切换/方向键组内移动 | FocusManager 底座复用 |
| 事件序 | keydown/keyup/click 派发顺序 + preventDefault | 以 WPT 用例为准 |
| WPT 基础设施 | 键盘交互用例导入、通过率报告 | 复用 tests/wpt-runner + `make import-wpt`；已有 `interactive/*` 内建分类可作 smoke 但不作达标依据 |
| 单元测试 | 每项修复带单测 | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **文本编辑键**（光标移动/删除/输入插入）— 兄弟目标 `editing-contenteditable.md`
- **页面滚动键**（PageUp/PageDown/Space 滚页面/Home/End）— 兄弟目标
  `keyboard-page-scrolling.md`
- **指针输入**（鼠标/触摸点击激活）— html-compat pointer 域（已完成面）
- **快捷键/菜单键**（Ctrl+T 开标签等浏览器级）— browser-shell 域，已有 app_input_keys
- **IME 组合键** — 平台输入法域（host-runtime IME 已有独立工作面）

### 依赖约束

- **与 form-validation 衔接**：Enter 提交须走其已建成的 interactive validation 管线
  （requestSubmit 阻断）。该流若在 M3 收尾期活跃，先做非提交键面（激活/Esc/select）。
- **与 js-dom 流碰撞管理**：default action 若需触发 click 等合成事件，走既有事件派发
  管线；碰 `js_dom_shim` 事件段前先 `git log --since="14 days ago" --
  crates/engine/src/js_dom_shim/` 核对。

---

## 当前能力/缺口基线

**详见** [keyboard-default-actions/master.md](keyboard-default-actions/master.md)（运行时
控制面板，唯一真实状态来源）。

**关键摘要**（2026-08-17 实测）：

- ✅ **FocusManager**：Tab 导航 + tabindex 排序 + 13 单测
- ✅ **html_actions submit 路径**：form-validation M3 的 requestSubmit 阻断已接
- ⚠️ **缺口 1 — 默认动作分发层缺失**：keydown 按控件类型 + 按键的分发无系统性实现
- ⚠️ **缺口 2 — implicit submission 缺失**：Enter 提交规则（单/多文本控件）未实现
- ⚠️ **缺口 3 — 激活键语义缺失**：空格/Enter 对 button 类控件的 click 合成 + 两键差异
- ⚠️ **缺口 4 — select 键盘导航缺失**：展开/移动/type-ahead 全无
- ⚠️ **缺口 5 — WPT 覆盖为零**：无键盘默认动作上游用例导入

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT 用例导入与通过率基线

- [ ] 导入键盘交互相关的上游 WPT 真实用例（表单键/激活面/select 导航；范围与 skip list
      有据）
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经 `make import-wpt` 常驻断言集并记入 `imported-tests.txt`
- [ ] 通过率报告持久化到 `docs/goal/keyboard-default-actions/evidence/`，历史可追溯

### DC-2: 表单键与激活语义

- [ ] Enter 提交 + implicit submission 规则 + 与 interactive validation 联动（含
      formnovalidate）
- [ ] 空格/Enter 激活 button 类控件（两键的 keydown/keyup 差异）；preventDefault 阻断
- [ ] Esc 的 dialog cancel/close、select 展开态收起

### DC-3: select 与 radio/checkbox 键盘导航

- [ ] select 展开键/选项移动/Home/End/type-ahead（JS 可观察面验证）
- [ ] radio 方向键组内移动 + 选中；空格切换 checkbox

### DC-4: 事件序正确

- [ ] keydown → click（合成）→ keyup 顺序与 cancelable 语义与 spec 一致（WPT 为准）

### DC-5: 测试与质量不可退让

- [ ] `cargo test` 全绿，零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT 键盘交互基线建立

**目标**：导入键盘交互相关 WPT 用例，跑通执行，记录通过率基线。

**切片建议**：
1. 用例导入 + 分类通过率报告（零源码改动，纯资产）
2. 失败聚类 → 首个轻量修复队列
3. 默认动作分发层骨架（按控件类型 + 按键的分发表）

### M2 — 表单键与激活

**目标**：Enter 提交/implicit submission/空格与 Enter 激活/Esc 语义；与 form-validation
管线联动。

### M3 — select 导航 + radio/checkbox + 事件序收尾

**目标**：select 键盘导航全面、radio 方向键、事件序断言用例全通。

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

1. **自主探索**当前输入事件派发链路（browser app_input → engine 事件 → shim keydown）与
   default action 断点位置
2. **自主导入** WPT 键盘交互用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（分发层缺失？implicit submission？事件序？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`cargo test` + clippy + WPT 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 js-dom 共享面（js_dom_shim 事件段）前先 `git log` 核对；有活跃编辑则
   转零碰撞面（host 分发层、WPT 导入、Rust 侧）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。
2. **用例失败分析**：每个失败 case 必须分析根因（分发缺失？键语义？事件序？validation
   联动？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/keyboard-default-actions/master.md`：当前真实状态的唯一
  控制面板。治理规则：持续演进、不允许无限增长、各章节必须自洽。
- **归档区域** `docs/goal/keyboard-default-actions/archive/`：存储已完成里程碑的详细过程与
  历史证据，只追加不修改。
- **证据区域** `docs/goal/keyboard-default-actions/evidence/`：存储通过率报告、失败分析等
  验证证据，持续追加。
