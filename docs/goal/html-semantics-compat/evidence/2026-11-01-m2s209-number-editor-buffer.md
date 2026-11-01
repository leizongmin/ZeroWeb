# M2-S209：number 编辑器双槽——键入 veto + caret 臂 + badInput 接线收口

- **日期**：2026-11-01
- **目标面**：number 键入交互三语料——
  `number-invalid-scientific-key-events`（4 测：`1e-+1`/`1e++1`/`+1e+1`
  否决、`-` 起点接受 → valueAsNumber −10）、
  `number-constraint-validation`（5 测：键入 `'1.e'`→''+badInput true、
  `'1.'`→'1'+badInput false、setter 双臂净化无 badInput、required+键入
  '1.e' → valueMissing）、`defaultValue-clobbering` number 腿（键入
  `'123e'` → validity.valid false 且 defaultValue 赋值不扰动）
- **状态**：收口——三语料 11/11 Pass；forms 全域 A/B 净 F→P=4（恰 4
  靶键）；P→F=2 全落 onblur flake 族（基线态当下重跑同 F，非本刀）；
  键漂移 0

## 勘根

- **根因**：引擎 number 键入**无 veto、无 caret 臂**。`__zw_text_input`
  把键入字符按文本控件通用拼接写 value 槽（raw slot），科学计数三测
  1-3 经 `parseFloat` 前缀解析**意外通过**（`'1e+1-'` → 10）；`Insert -
  at start`（caret 移位插入）无 caret 模型 → 从尾部拼 → `1e+1-` 可
  valueAsNumber 但 validity 断言败（基线 3P+1F）。constraint 测 1
  `'1.e'` 键入后 value 仍 `'1.e'`（无净化换算）→ badInput 断言败；
  测 3 `'1.'` 同理。clobbering number 腿 `'123e'` 同败。
- **双通道形态**：part04 value setter 已有 R3901
  `__zwSkipValueSanitize` 旗标（键入组合绕行 `_zwSanitizeInputValue`
  净化）与 M2-S203 number 严格文法净化——**setter 面正确**，缺口纯在
  键入面（编辑器模型）与 validity badInput 臂。
- **语料模型三角定位**（HTML §4.10.5.1.10 number state 用户交互面）：
  键入原文保留于**编辑器层**（可能长于暴露值）；value 槽写**暴露换算
  值**（严格合文法且 isFinite → 原样；仅尾点 `^(-?\d+)\.$` → 剥点；
  否则 ''）；badInput = 编辑器原文非空且换算 ''；不合编辑器文法
  （`^-?\d*\.?\d*(?:[eE][+-]?\d*)?$`）的键入**整键否决**（编辑器/值/
  事件全不动）。ArrowLeft/Right 移 caret 于编辑器空间。
- **不能伤的键**：setter 双臂（constraint 测 2/4——净化无 badInput，
  键入 skip 通道不清编辑器，须区分）；R3906 value setter 选区复位臂
  （编辑器清后暴露长度变化，caret 语义不变仍成立）；text/其他 type 控
  件键入（`_nTy === 'number'` 窄门）；maxlength 截断序（编辑器候选拼
  接在截断之后， veto 不吃截断产物）。

## 施工

（shim 3 文件 + rust 3 文件；语料零改动）

- **part01.js 编辑器双槽**：`_numberEditor = {}` 表 +
  `_zwNumberEditorGrammar`/`_zwNumberEditorValue` 帮手；`__zw_text_input`
  number 分支——caret 拼候选→文法否决→编辑器写候选→value 槽写暴露
  值（skip 旗标通道），事件 `data` 仍为键入原字符，caret 推进 = 键入
  长度；`__zw_text_delete` number 分支——删除走编辑器原文空间、不否
  决、删后 value 槽写暴露值；新增 `globalThis.__zw_text_caret_move`——
  ArrowLeft/Right ±1 UTF-16（代理对安全）、选区折叠、方向
  backward/forward，返 true=消费；`__zw_reset_form_state` 清编辑器表。
- **part03.js `_validityState` badInput 臂**：INPUT+number+编辑器有原
  文时 `badInput = 原文 !== '' && 换算 === ''`，并入 valid 汇总与
  `_vs` 对象（setter 路径无编辑器原文 → badInput false 自然成立）。
- **part04.js 三处程序化清槽**：value setter 主 else 臂、valueAsNumber
  setter、valueAsDate setter——非 skip 旗标写（即真实 IDL set）删
  `_numberEditor[key]`；clone 路径与 removeAttribute('value') 不清
  （dirty 旗标守卫下暴露值语义不变，勘定时排除）。
- **script_gen.rs**：`script_text_caret_move(selector, key)` 桥生成
  （同 script_text_input 形态）。
- **testharness.rs scroll_key 分支**：keydown 未阻止后
  select_consumed（s129 SELECT/radio）之后插 **text_caret_consumed**
  门——input/textarea 目标经 `__zw_text_caret_move` 消费 ArrowLeft/
  Right（移 caret 而非滚动，真实浏览器语义）；两者皆未消费才落滚动。
- **part09.rs R5327 新桥测试**
  （test_r5327_number_editor_buffer_keystroke_veto_badinput）：八腿
  镜像三语料——键入 '1.e'→''+badInput true+valid false；'1.'→'1'+
  badInput false；编辑器空间续键（ArrowLeft+5 → '15.' 暴露 '15'）；
  setter 三连净化无 badInput；'-' 起点接受 → '-1e+1'/−10；'e' 后插
  '-' 否决 → '1e+1'/10/valid true；'123e'→valid false + defaultValue
  扰动免疫；required+valueMissing；text 控件不受扰 + caret 臂消费
  语义（非文本目标返 false 落滚动）。

## 验证

- 拼接 shim（15 part include 序）node --check SYNTAX_OK。
- 桥测试 R5327 单测：**1 passed**（八腿全绿；首跑暴露 part01 键入臂
  误用 `key`（应为 `target.key`）ReferenceError，修正后过）。
- 靶面三定向（重建 release runner 后）：
  `FILTER=number-invalid-scientific-key-events` → **4/4 Pass**；
  `FILTER=number-constraint-validation` → **5/5 Pass**；
  `FILTER=defaultValue-clobbering` → **2/2 Pass**。
- forms 全域 A/B（B 侧全域跑 9080 行取 `html/semantics/forms/` 子集，
  与 s208 基线 forms-wip.log 4711 键对齐）：**净 F→P=4，恰 4 靶键**
  （constraint 测 1 + 测 3、scientific 'Insert - at start'、clobbering
  number 腿）；另 1 键 F→P 落 onblur flake 族（active-onblur Buttons
  spacebar，在册摆动族回摆）；P→F=2 全落同族（checkable-active-onblur
  Radios spacebar / with-click Checkboxes pointing device）——**flake
  判定证据**：WIP 态单文件重跑 Checkboxes 腿回摆 Pass；stash WIP 后
  基线态当下重跑 spacebar 双腿亦 Fail（环境时序敏感，基线同败非本
  刀）；s208 轮 A/B 同族已有双向摆动记录。键漂移 0，两侧 (key,state)
  对各 4711。
- zero-engine lib：**3669P+0F**（预期 1F 的 ua_display_li 本轮转绿；
  R5327 计入）。
- fmt 本刀零 diff；guarded-clippy 过（0 警告）。

## 挂账

- ArrowUp/Down textarea 视觉行 caret 模型未实现（当前 caret 臂仅
  Left/Right；宿主 scroll_key 分支对 Up/Down 仍落滚动）。
- `script_set_text_control_state` skip 旗标程序化写不清
  `_numberEditor`（既有 t2-pd2a 通道，与 IDL setter 不同路；无语料
  钉住，未动）。
- caret 臂与 `_zwReconcileSel` 交互已勘安全（`__zw_text_input` 每键
  整体替换 `_textSelection` 对象、so.val 失配不钳制；caret 臂原对象
  改字段时 so.val 同源不触发钳制）——若后续 reconcile 语义变更需复核。
- onblur flake 域持续采样（本轮 +2 下摆 +1 上摆样本，已附基线态重跑
  对照证据）。
