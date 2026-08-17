# 表单校验（Form Validation）— WPT 驱动的 HTML 约束校验正确性目标

> **已归档（2026-08-17）**：目标 Done Criteria 全部满足（M1-M3 完成——
> constraints Pass 919 / Fail 0）。本入口文档移入归档区（只追加不修改）；
> 运行时状态见同目录 `master.md`，验证证据见 `evidence/`，里程碑过程见
> `archive/m1-m3-milestones-2026-08-17.md`。

**版本**: v1.0
**日期**: 2026-08-16
**状态**: Active
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/html-compat.md`（HTML 行为兼容——M0-M4 已完成，本专项为其约束校验面延伸）

> **说明**
> 本文档是 ZeroWeb「表单校验」专项目标执行契约。目标是把 Constraint Validation API 从
> permissive 基础（R2825：`checkValidity`/`setCustomValidity`/`validity` 全 valid）深化为
> **真实约束计算**（required/pattern/min/max/step/长度/type），以 WPT `html/semantics/forms/constraints`
> 真实用例通过率为验证标准。本文定义 Mission、边界、Done Criteria、执行协议和文档治理规则，
> 供后续 `rally run` 会话作为稳定输入。日常进展、evidence、active milestone 更新写入 `master.md`。
>
> **▶ 拆分动机（2026-08-16 用户决策）**：html-compat 已完成 M0-M4（FR-001~012，completion-audit
> 确认），但其 spec-rfc 范围内有未覆盖面——**表单校验**（FR 范围未提 validation）。现有 R2825
> 仅 permissive 基础（`_customValidity` 状态 + 反射，part01/04.js；原生约束 headless 不强制），
> 真实约束计算（required/pattern/min/max/step 的 validityState 位）缺失；WPT
> `html/semantics/forms/constraints` 无真实用例导入。用户建议：优先拆 **Form Validation 深化**
> （HTML 表单核心缺失面 + WPT 用例量大 + 独立验收面），其次键盘默认动作（零冲突）。
>
> **▶ 基线事实（2026-08-16 实测）**：R2825 提供 `checkValidity`/`reportValidity`/
> `setCustomValidity`/`validity`/`validationMessage`/`willValidate`（part04.js 反射 +
> part01.js `_customValidity` 状态 + `_userEdited` 用户编辑标记）；**原生约束 headless 不强制**
> （permissive valid——注释明言）；`validity` 默认全 valid、`checkValidity` 恒 true；
> customError 由 `setCustomValidity` 跟踪；invalid 事件在 checkValidity/reportValidity invalid
> 时派发（已有）。WPT `html/semantics/forms/constraints` 未导入（wpt-data 无 forms 目录）。

---

## Mission

以 **WPT `html/semantics/forms/constraints` 真实用例通过率为验证标准**，将 Constraint
Validation API 的约束计算与提交阻断行为对齐到 Chromium 水平。分阶段里程碑校准执行预期：

| 阶段 | 目标 | 说明 |
|---|---|---|
| 2026 年内 | **基线建立** | 导入 `html/semantics/forms/constraints` 用例 + 通过率基线（当前无基线） |
| 中期 | **80%** | 约束计算全（required/pattern/min/max/step/长度/type）+ 提交阻断 |
| 长期 | **90%+** | 覆盖非交互校验（novalidate/formnovalidate）、willValidate 全语义、badInput 边界 |

**关键约束**：所有验证必须基于从上游 WPT 仓库导入的**真实用例**（同 canvas-2d——不允许
手写 inline 用例替代或充数）。通过率统计的分母是上游 `html/semantics/forms/constraints`
目录中所有属于范围内、不在 skip list 中的用例。

覆盖范围：

1. **约束计算** — required（valueMissing）、pattern（mismatch）、min/max
   （rangeUnderflow/rangeOverflow）、step（stepMismatch）、minlength/maxlength
   （tooShort/tooLong——仅用户编辑值）、type 约束（typeMismatch/badInput）、
   `setCustomValidity`（customError）
2. **validityState 全位** — 各约束位的真实计算与联动（多约束同时失效的位组合）
3. **提交阻断** — form submit 的 interactive validation（invalid 控件聚焦 + invalid
   事件序列 + 提交中止）；novalidate/formnovalidate 跳过
4. **API 语义** — checkValidity/reportValidity（invalid 事件 + 返回 false）、
   validationMessage（约束消息）、willValidate（disabled/hidden/readonly 排除）、
   setRangeText 后的 tooLong 联动
5. **事件序列** — invalid 事件派发（checkValidity/reportValidity/submit 三条路径）
   与默认动作（报告 UI 聚焦——headless 面为事件 + 返回语义）

执行方式：**交替推进** — 每轮同时扩展 WPT constraints 导入范围和修复发现的缺口。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| 约束计算 | required/pattern/min/max/step/长度/type 的 validityState 位 | 核心面——host 层计算（Rust），shim 反射 |
| validityState | 全 12 位 + 联动（多约束组合） | 以 WPT 用例为准 |
| 提交阻断 | submit 的 validation 检查 + invalid 事件 + 聚焦 | 与 html_actions 的 submit 路径集成 |
| API 反射 | validity/validationMessage/willValidate/checkValidity/reportValidity/setCustomValidity | R2825 已有 permissive 基础——深化真实值 |
| WPT 基础设施 | constraints 用例导入、通过率报告 | 复用 tests/wpt-runner + `make import-wpt` |
| 单元测试 | 每项修复带单测（engine bridge 级 + 集成级） | CLAUDE.md 测试资产化规则适用 |

### 不在范围内（明确排除）

- **CSS 伪类**（`:valid`/`:invalid`/`:user-valid` 匹配）— 样式系统面（rendering-compat 域）
- **表单控件属性反射**（value/checked/selected 的 JS getter/setter）— js-dom 域
- **表单布局/外观** — rendering-compat 域
- **新 crate 依赖的大规模引入** — 最小化新依赖

### 依赖约束

- **与 js-dom 流碰撞管理**：表单控件的属性反射段（`js_dom_shim` part04/05.js 的表单段）与
  js-dom 流共享活跃面。开工前先 `git log --since="14 days ago" -- crates/engine/src/js_dom_shim/`
  核对；若该流最近编辑过相关段落（run-rules §9 碰头信号），先做零碰撞面（约束计算 host 层、
  WPT 导入、Rust 侧），碰头段等其告段落。
- **与 html-compat 流的关系**：html-compat 已完成（M0-M4）——本专项的提交阻断与
  `html_actions` 的 submit 路径共享（现有代码——不重建，深化其 validation 检查）。

---

## 当前能力/缺口基线

**详见** [form-validation/master.md](form-validation/master.md)（运行时控制面板，唯一真实状态来源）。

**关键摘要**（2026-08-16 实测）：

- ✅ **R2825 permissive 基础**：`checkValidity`/`reportValidity`/`setCustomValidity`/
  `validity`/`validationMessage`/`willValidate`（part04.js 反射 + part01.js 状态）；
  customError 跟踪；invalid 事件派发（checkValidity/reportValidity 路径）；
  `_userEdited` 标记（minlength/maxlength 用户编辑跟踪）
- ⚠️ **缺口 1 — 原生约束为零**：required/pattern/min/max/step/type 的约束计算未实现
  （permissive valid——注释明言「原生约束 headless 不强制」）
- ⚠️ **缺口 2 — WPT 覆盖为零**：`html/semantics/forms/constraints` 无真实用例导入
- ⚠️ **缺口 3 — 提交阻断缺失**：submit 的 interactive validation（invalid 聚焦 +
  事件序列 + 中止）未实现
- ⚠️ **缺口 4 — willValidate 真实化**：disabled/hidden/readonly 的排除未实现
- ⚠️ **缺口 5 — validationMessage 约束消息**：仅 customError 消息（原生约束消息缺失）

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

### DC-1: WPT constraints 用例导入与通过率基线

- [ ] 从上游 WPT 仓库 `html/semantics/forms/constraints` 目录导入**全部**范围内真实用例
- [ ] 建立分类通过率报告（文本 + JSON），记录基线
- [ ] 每项修复的 driving WPT 用例经 `make import-wpt` 常驻断言集并记入 `imported-tests.txt`
- [ ] 通过率报告持久化到 `docs/goal/form-validation/evidence/`，历史可追溯

### DC-2: 约束计算与 API 语义完整

- [ ] 全约束位真实计算：valueMissing/mismatch/rangeUnderflow/rangeOverflow/stepMismatch/
  tooShort/tooLong/typeMismatch/badInput/customError（联动组合）
- [ ] checkValidity/reportValidity/validationMessage/willValidate 与规范一致（WPT 为准）
- [ ] `setCustomValidity('')` 清除、非空设置、空串重置的语义

### DC-3: 提交阻断与事件序列

- [ ] form submit 的 interactive validation：invalid 控件聚焦 + invalid 事件 + 提交中止
- [ ] novalidate/formnovalidate 跳过 validation
- [ ] invalid 事件的三条路径（checkValidity/reportValidity/submit）语义一致

### DC-4: 测试与质量不可退让

- [ ] `cargo test` 全绿（含 engine bridge 级单测），零失败
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` 零警告
- [ ] 每项修复有对应单元测试 + driving WPT 用例资产化

---

## 活跃里程碑

### M1 — WPT constraints 基线建立（进行中）

**目标**：导入 `html/semantics/forms/constraints` 用例，跑通 testharness 执行，记录通过率基线。

**切片建议**：
1. 用例导入 + 分类通过率报告（零源码改动，纯资产）
2. 失败聚类分析 → 首个轻量修复队列（约束位计算）
3. 提交阻断（html_actions submit 集成）

### M2 — 约束计算完整化

**目标**：全约束位真实计算 + validityState 联动 + validationMessage + willValidate，
每项 kill-switch + A/B 零回归。

### M3 — 提交阻断与事件序列

**目标**：interactive validation 全链路（submit 阻断 + invalid 聚焦 + novalidate 跳过），
驱动用例全部通过。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进 | `CONTINUE: <下一步>` | **这是默认输出** |
| 真正的外部阻塞 | `BLOCK: <原因>` | 罕见使用 |

### DONE 允许条件

**同时满足**：DC-1~4 全部满足；验证基于上游真实 WPT 用例（无内建 inline 充数）；
`cargo build` + `cargo test` + `cargo clippy` 全通过；master.md 内部自洽，archive 已建立。

---

## Execution Protocol

### 自主执行原则

1. **自主探索**当前 validation 管线状态（part01/04.js 的 R2825 段 + host 约束计算入口）
2. **自主导入** WPT constraints 用例，扩大覆盖范围
3. **自主运行**用例，分析失败原因（约束位缺失？host 计算？事件序列？）
4. **自主修复**，不等待用户逐步指令；每修 net≥0 即 land
5. **自主添加测试**，新修复必须有对应单元测试 + WPT 用例资产化
6. **自主验证**：`cargo test` + clippy + WPT 通过率确认修复有效
7. **持续推动**，直到 Done Criteria 全部满足

### 轻量修复优先

1. **主线 = 轻量修复**：WPT 驱动、根因清楚、改动面小、A/B 无新失败。
2. **永不停**：遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复。
3. **碰撞管理**：碰 js-dom 共享面（表单控件属性反射段）前先 `git log` 核对；有活跃编辑
   则转零碰撞面（约束计算 host 层、WPT 导入、Rust 侧）。

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。遇到 flaky test、遗留失败、环境脚本问题时，
   当作当前任务的一部分修复。
2. **用例失败分析**：每个失败 case 必须分析根因（约束位缺失？host 计算错误？JS 反射？
   事件序列？）。
3. **技术决策**：在 master.md 中记录关键决策及其理由。

---

## Document Control / Archive Policy

- **入口文档**（本文件）：定义 Mission、Done Criteria、执行协议和文档治理规则。**修改条件**：
  仅在目标本身发生实质性变化时修改。**禁止行为**：每轮执行不重写本文件。
- **运行时控制平面** `docs/goal/form-validation/master.md`：当前真实状态的唯一控制面板。
  治理规则：持续演进、不允许无限增长（过时内容压缩或归档）、各章节必须自洽。
- **归档区域** `docs/goal/form-validation/archive/`：存储已完成里程碑的详细过程与历史证据，
  只追加不修改。
- **证据区域** `docs/goal/form-validation/evidence/`：存储通过率报告、失败分析等验证证据，
  持续追加。
