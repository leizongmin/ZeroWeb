# M2-S208：:valid/:invalid 伪类候选门——output-validity 1F 收口

- **日期**：2026-11-01
- **目标面**：`html/semantics/forms/the-output-element/output-validity.html`
  1F（output 应既不匹配 ：valid 也不匹配 ：invalid——barred from
  constraint validation；含 setCustomValidity 后仍两不匹配）
- **状态**：收口——靶面 1/1 Pass（6 断言全过），forms 全树 keyed A/B
  净 F→P=2（恰 1 靶键 + 1 onblur flake 域上摆键），P→F=0，键漂移 0

## 勘根

- **spec 落点**：HTML §4.10.22.2（#selector-valid/#selector-invalid）——
  `:valid`/`:invalid` 仅匹配 **candidates for constraint validation**
  （submittable 且非 barred）；output 属 barred → 两伪类均不匹配，
  setCustomValidity 不改变（候选性不因 custom error 产生）。
- **双通道形态**：Rust 样式系统路径已正确——matcher/mod.rs:331
  `"valid" => doc.is_valid_element`，dom/document/validation.rs
  `is_validation_candidate` 仅 input/select/textarea（非 disabled、input
  非 readonly），output 注释明示 barred 两不匹配。**失败走 shim matches
  通道**：part04.js matches 臂 `:valid`/`:invalid` 直接返
  `_validityState(key, sel, handle).valid`，无候选门——output 落通用
  汇总（无约束旗标）→ `valid:true` → `matches(':valid')` 误 true；
  setCustomValidity 后 `customError → valid:false` → `matches(':invalid')`
  误 true。
- **语料断言面**（6 断言 1 测试）：前置双 matches false；setCustomValidity
  后 validationMessage 空（`_wvSubmittable` 已排除 output，本就 Pass）、
  validity.customError true / validity.valid false（validity IDL getter
  无门，通用汇总已 spec-true）、双 matches 仍 false。首断言失败中断，
  修门后 6 断言一次全绿（后 5 断言形态勘根时已核对既有实现正确）。
- **不能伤的键**（语料 matches 用面全查）：input（pattern_attribute*.html、
  constraints/input-number-*）、select（customizable-select validity 面）、
  **FORM 聚合**（form-requestsubmit.html:167 `form.matches(':invalid')`
  true——`_validityState` 的 FORM 聚合臂，R57 FV M3 所建）。button/
  fieldset 语料无 matches 断言。→ 门集 = INPUT/SELECT/TEXTAREA/FORM。

## 施工

（shim 1 文件 + rust 测试 1 文件；语料零改动）

- **part04.js matches 臂候选门**（:5380 区）：`:invalid`/`:valid` 合并
  判定——`_realTag` 非 INPUT/SELECT/TEXTAREA/FORM 直接返 false，不落
  `_validityState` 通用汇总；候选内保持原 valid/!valid 语义（FORM 聚合
  臂经 `_validityState` 自然保留）。候选集注释绑定 Rust 侧
  `is_validation_candidate` 同源（双通道同语义）。
- **part09.rs R5326 新桥测试**
  （test_r5326_output_matches_valid_invalid_candidate_gate）：output 前置
  双 matches false + setCustomValidity 后 validationMessage 空/
  customError true/valid false/双 matches 仍 false + input sanity 匹配
  :valid——八断言镜像语料六断言 + 候选对照（语料 gitignore，常驻桥
  测试等价承载，s205/s206/s207 先例）。

## 验证

- 拼接 shim（14 part include 序）node --check SYNTAX_OK。
- 靶面：`make testharness-html-semantics FILTER=output-validity` →
  **1/1 Pass**（重建 release runner 后）。
- forms 全树 A/B（FILTER=forms 4711 键，基线侧复用 s207
  forms-wip.log=已提交 HEAD 809a14134 shim 态）：**净 F→P=2，其中恰
  1 靶键**（output-validity）；另 1 键 F→P 落 **onblur flake 域**
  （checkable-active-onblur-with-click，在册双向摆动族再采样）；P→F=0；
  键漂移 0；其他位移（含 timeout）0，两侧 (key,state) 对各 4711。
- zero-engine lib：**3667P+1F**（1F = ua_display_li 渲染流域在册键，
  基线同败非本刀；R5326 新测计入 3667）。
- fmt 本刀零 diff；guarded-clippy 过（0 警告）。

## 挂账

- shim `_validityState` 的 disabled/readonly 语义与 Rust 候选门存在
  既有分歧（R57 按 validator.js 迭代语义：disabled 仅 valueMissing 豁免、
  pattern/range 仍校验——WPT constraints 语料钉住；Rust 候选门则
  disabled/readonly input 整体非候选）——双通道伪类结果对 disabled 控件
  可能分歧，无语料钉住，不动（R57 语义为语料所建）。
- `:in-range`/`:out-of-range` matches 臂同区同构（非候选元素潜在同形
  误匹配）——无语料驱动，未勘未动。
- onblur flake 域持续采样（本轮 +1 上摆样本）。
