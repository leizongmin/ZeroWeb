# M2 s132 · radio 组排他现象 4（type 变更信号）接线——radio-morphed 1F 收口（真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s131（radio 组排他现象 3 form owner 变更四通道接线）

## 入场定刀位

s131 收口后按控制面下一步入场：候选①「form= 属性变更 radio 组重排」
勘根先翻 corpus 断言面——**无 radio 组重排面**（全语义树
`setAttribute('form')` 共 5 文件：radio-keyboard-navigation-order /
form-control-infrastructure/form_attribute（Pass）/ the-form-element
form-elements-filter（Fail=form.elements 成员重排，异根）/ form-nameditem
（Fail=命名 getter 机制面，异根）/ radio.html）；候选②全量翻拣前，
radio 域残余单根面显性：**radio-morphed.html 1F**——「Setting type
attribute must unset checkedness of other elements」，恰为 spec 现象表
现象 4「A type change is signalled for the element」（s131 FIXME 已入
码的接线缺口之一），定刀位收窄为现象④单根接线。

## 勘根定谳

1. **spec 定谳（先拉原文）**：HTML §4.10.5.4 input type change steps
   步骤 6「Signal a type change for the element. (The Radio Button
   state uses this, in particular.)」= 现象④的触发信号；type 变更
   算法本身无 checkedness 重置步骤——morphed text（带 checked 属性）
   的 checkedness 非脏回落属性读即 true，「occurrence 后置真」门自然
   成立。现象②（name 属性设/改/删）无独立 corpus 面：radio.html:88
   「changing the name …」案由现象①（checkedness 置 true，s125 已接）
   在新组员资格下 sweep 覆盖（12/12 已 Pass），不入本轮范围。
2. **接线点单点覆盖双通道**：input.type IDL setter（part05 type 分支）
   与 setAttribute 路径（part04 R3901 臂）**均汇入**
   `_zwApplyInputTypeChange`（part01:2137，R3901 既有共享出口）——
   尾部单点接线即覆盖双通道，无需 s131 式四通道拓扑（type 变更不走
   插入管道）。s131 的 part04→part05 跨 part 调用已证实拼接单闭包内
   声明提升跨 part 可调用（part01→part05 helper 同理）。
3. **BUTTON 边界自查**：`_reflectedTypeEnum` 对 BUTTON 也返非 null
   （oldTy 合法），setAttribute 设 BUTTON type=radio 时 newTy 亦归一
   'radio'——sweep 的 checked 门自兜（BUTTON 无 checked 属性读
   undefined 即返），不新增 tag 门（最小范围）。

## 施工（engine shim part01/05.js，无 Rust 变更）

1. part01 `_zwApplyInputTypeChange` 尾部：`oldTy !== newTy && newTy
   === 'radio'` 门 + try/catch 调 `_zwRadioUncheckGroupOthers(
   _makeProxy(sel, handle))`（同态再设不触发——spec「changes state」
   语义；注释含 spec 双链接与通道覆盖说明）。
2. part05 s125 FIXME 更新：type 变更移出「未走组排他」清单（form
   owner 变更 s131、type 变更信号 s132 已接线）。

## 验证

- 定向：`FILTER=the-input-element/radio-morphed` **1/1 Pass**（1F→0）。
- 定向：`FILTER=forms/the-input-element` **1512P/132F/3T**（s131
  1511P/132F/3T，+1 全为 radio-morphed）；`FILTER=forms/constraints`
  **917P/3F 与 s129/s130/s131 同额**（异根不动）。
- 全量 A/B × /tmp/s131-full.log 键级 join **恰 1 Fail→Pass**
  （radio-morphed 单案），**真翻红 0、Pass→Timeout 0、new non-Pass
  0、gone 0**；唯一键 6784P/1022F/103T → 6785P/1021F/103T。
- engine lib **3570/0**；wpt-runner bin **256/0**（76 ignored）；
  clippy（zero-engine --all-targets -D warnings）零告警；本轮零 Rust
  变更（shim JS include_str!），fmt 不适用。shim 拼接（js_dom_bridge.rs
  装配序 13 part）node --check 过。
- 本轮勘根为代码阅读 + spec 原文取证，未建探针页；语料零改动
  （git status 仅 shim 两文件）。渲染面零依赖——reftest/product-smoke
  未跑。全量 runner 以 corpus 已知 Fail 非零退出（make 错误 1，s131
  同形非异常）。

## 挂账（本切片更新）

- **radio-morphed 1F 收口移出候选**；the-input-element radio 族
  残余 Fail 全为 focus-navigation/keyboard 家族（交互面异根，另账）。
- **现象接线缺口收窄**（FIXME 已同步）：余 checked 属性增设 /
  defaultChecked / becomes connected（现象⑤）未接；现象② name
  设/改/删无独立 corpus 面（由现象①覆盖），不单独立项。
- 承袭不变：button-events「type=button shouldn't trigger」与该文件
  级 Timeout、empty.html/empty-with-base.html Timeout、
  form-submission-0 186F、execution-timing 构成面、lazy-loading 26F、
  meta color-scheme 21F、写流盖章断链、part02 setAttribute 绕
  notify、041 nested write、deep clone 非 HTML ns、css-module
  corpus 门（待决）、www2 可达性、module-import-referrer TLA、
  dynamic import() with 透传、classic 外链 SRI、143 write、defer
  多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList
  缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面让位累积、s131
  插入步缺口细化（form= 属性变更重排无 radio corpus 面、移除方向、
  子树深层 radio、_zwMEl own insertBefore、sel 面 innerHTML、
  outerHTML/insertAdjacentHTML）；**待决（须用户拍板）**：css/text
  数据模块语料资源缺失、green-2x2.png 语料资源分发、parser form
  element pointer 4F 缓行（2026-10-23）。

## 过程教训

- **接线前先翻 corpus 断言面再定刀位**：控制面候选①（form= 属性
  变更）勘根后无 radio corpus 面，若硬上即为无断言面的投机实现；
  同域残余 Fail 翻拣（radio-morphed=现象④）才是有验证闭环的刀位。
- **共享出口先于多通道拓扑**：s131 四通道接线是因为插入无共享出口；
  type 变更已有 `_zwApplyInputTypeChange` 单汇点，单点接线即双通道
  覆盖——勘根先找既有汇点，没有才逐通道布线。
