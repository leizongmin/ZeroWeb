# M2-S210：FORM 级约束校验双缺口——fieldset disabled 传染 + invalid 全遍历派发

- **日期**：2026-11-01
- **目标面**：`html/semantics/forms/constraints/form-validation-validate.html`
  3F（fm3 fieldset child 的 checkValidity/reportValidity 三态断言两键 +
  invalid 事件全遍历计数键 times1=4）
- **状态**：收口——靶面 8/8 Pass；forms 全域 A/B 净 F→P=3 恰 3 靶键，
  onblur flake 族 s209 摆走两键回摆 + 一键摆回（闭环归位），键漂移 0，
  其他位移 0

## 勘根

- **缺口一（fieldset 传染，fm3 两键）**：spec §4.10.5.1.1
  #concept-fe-disabled——form control disabled 判定含 disabled fieldset
  后代，其**首个 legend 元素子**的后代豁免。语料 fm3 三态钉面：
  初始 `checkValidity()` false（fieldset 直下 inp2 required 空）→
  `fs.disabled=true` 后 **true**（inp2 传染豁免 valueMissing；legend 内
  inp1 豁免传染）→ `inp1.type='url'; inp1.value='aaa'` 后 **false**
  （legend 豁免不传染，inp1 仍候选 url typeMismatch）。现状
  `_validityState`（part03:12825）只查**自身** disabled 属性，无祖先
  走查 → fs.disabled 置位后 inp2 valueMissing 仍 true → 期望 true 断
  言败。**可复用面**：s193 willValidate 臂（part04:6073-6096，
  disabled-001 语料 5 案钉态）已有同口径 fieldset 祖先走查 + 首
  legend 豁免；S197 焦点面（part06）复用同判据——读面基建齐备，仅
  validity 通道未接。
- **缺口二（invalid 全遍历，计数键）**：spec §4.10.21.2
  #dom-cva-checkvalidity → #interactively-validate-the-constraints——
  对 form owner 是本 form 的**每个** invalid 且未 barred 控件派发
  invalid 事件（cancelable 非冒泡）。语料 fm1 钉态：checkValidity 轮
  [test1(required 空), test2(email 'abc' typeMismatch)] 各 1 +
  reportValidity 轮再各 1 → times1=4。现状 FORM 臂（part04:5989 区）
  `if (!cv) return false` **短路**——test1 失败即返回，test2 从未检查
  → 每轮只派发 1 → times1=2（runner 实测 expected 4 got 2 实锤）。
  reportValidity 同段代码同形态。
- **归因小径**：R5328 首版误把语料 fm1（含 email 控件）与 fm3（纯
  fieldset form）揉成一个 form——腿 B 期望 true 被 email 'abc'
  typeMismatch 恒 false 掩盖；临时探针测试（fs.disabled getter 回读
  true + 属性真写 + inp2.parentNode 上行 FIELDSET + disabled 读 true +
  valueMissing 已豁免）证明施工 1 工作正常，败因在测试 form 设计——
  拆回语料双 form 形态后全绿。语料 form 拓扑不可合并是本次教训。
- **不能伤的键**：R57 disabled 语义（valueMissing 豁免、pattern/range
  仍校验——传染走同一豁免门不扩大）；willValidate 臂（s193 已有，不
  动）；`fs.disabled=false` falsy 真移除恢复候选（R2998 面）；FORM
  checkValidity 返回值语义（任一 invalid 即 false，全遍历不改变）；
  非 FORM 元素 checkValidity 控件级路径（同臂下方代码不动）。

## 施工

（shim 2 文件 + rust 测试 1 文件；语料零改动）

- **part03.js `_validityState` fieldset 祖先走查**（`_dis` 计算后）：
  自身 disabled 非命中时上行 parentNode 链（guard 64），disabled
  fieldset 命中 → 首 legend 元素子直子豁免（豁免后继续上行）→ 未豁
  免 `_dis = '1'`。R57 门不动（传染 barred 同样仅豁免 valueMissing）。
  变量 `_feD*` 前缀避免与既有 `_fs*` 撞名。与 willValidate 臂同构约
  25 行就地复制（跨文件提取共享帮手挂账——动 willValidate 臂扩大
  disabled-001 钉面回归风险）。
- **part04.js FORM 臂全遍历**：`_fmAnyInvalid` 旗标替换短路 return，
  全部控件检查完再 `return !_fmAnyInvalid`；spec 落点注释绑定
  #interactively-validate-the-constraints。
- **part09.rs R5328 新桥测试**
  （test_r5328_form_checkvalidity_fieldset_inherit_invalid_fanout）：
  六腿镜像语料——fm3 三态（初始 false / fs.disabled 后 true / legend
  内 inp1 url typeMismatch 后 false）；fm2 静态 disabled fieldset
  （legend 内 q1 仍候选 false + 直下 q2 豁免 true + q3 patternMismatch
  仍 true——R57 传染不豁免 pattern 钉面）；fm1 invalid 全遍历计数
  （checkValidity 轮 2 + reportValidity 轮累计 4 + 双返 false）；falsy
  移除恢复候选（valueMissing 回归）。

## 验证

- 拼接 shim（15 part include 序）node --check SYNTAX_OK。
- 桥测试 R5328 单测：**1 passed**（六腿全绿；首版 form 设计错经探针
  测试归因后拆分双 form 重写过）。
- 靶面：`FILTER=form-validation-validate` → **8/8 Pass**（重建
  release runner 后，3F 清零）。
- forms 全域 A/B（两侧各 4711 (key,state) 对，B 侧全域取 forms/ 子
  集）：**净 F→P=3 恰 3 靶键**（form-validation-validate 三键）；另 2
  键 F→P 落 onblur flake 族（checkable-active-onblur radio spacebar +
  with-click checkbox——**s209 轮 P→F 摆走的两键原位回摆**）；P→F=1
  落同族（active-onblur Buttons spacebar——s209 轮 F→P 摆上来的键回
  摆），族内三角闭环与 s209 evidence 的基线态重跑证据互证，非本刀；
  键漂移 0；其他位移（含 timeout）0。
- zero-engine lib：**3670P+0F**（R5328 计入）。
- fmt 本刀零 diff；guarded-clippy 过（0 警告）。

## 挂账

- part03 `_zwMElCheckValidity` FORM 分支（本地 _zwMEl 树路径，
  part03:18419 区）同构短路形态——validator.js 本地树承载面，无语料
  钉住，未动。
- fieldset 祖先走查跨文件共享化候选（part03 `_validityState` 与
  part04 willValidate 臂同构 ~25 行）——提取共享帮手需动 willValidate
  臂（disabled-001 钉面），独立小刀评估。
- button-click-submits-with-commandfor 3F（invoker commands
  commandfor 点击不提交门）+ button-click-submits 1F（fieldset
  disabled 传染对 button 提交门）——本轮域内 misc 残面盘点新浮现，
  后者与 s210 传染基建同区，次刀候选。
- onblur flake 域持续采样（本轮 +2 回摆 +1 回摆样本，s209 摆动闭环）。
