# M2 s131 · radio 组排他现象 3（form owner 变更）四通道接线——radio-disconnected-group-owner 残余 2F 收口（真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s130（radio 组根域枚举）

## 入场定刀位

s130 收口后按控制面下一步入场：插入步 radio checkedness 同步族 2F
（radio-disconnected-group-owner 残余——「Removed elements」后半：
radio2 带 checked 属性 appendChild 进 detached form 后 radio3 应
uncheck；「Appending input radio input into a disconnect form」
全案：radio2 IDL checked=true 后 appendChild 进 detached form 后
radio1 应 uncheck + innerHTML 重建后 children[0] 应 uncheck）。

## 勘根定谳（含排误）

1. **spec 现象表定谳（先于施工拉原文）**：HTML §4.10.5.1.16 Radio
   Button state 现象表 5 项——①checkedness 置 true（s125 已接）②name
   属性设/改/删 ③**form owner 变更** ④type 变更信号 ⑤becomes
   connected。DIV/FORM 对照案（同文件子测 7 vs 8）的判别维度是现象
   ③而非 root/tree 变更：detached DIV 插入 owner 恒 null 不触发（子
   测 7 负案在账且当前 Pass，不得回翻），detached FORM 插入 null→form
   触发。⑤只覆盖连接文档面，两案都在 detached 树不沾。s130 账本
   「checked 属性同步不跑排他」的初步归因据此修正——checked 属性在
   两案都不是触发维度，触发维度是插入引发的 form owner 变更。
2. **JS 插入四通道拓扑（勘根主转折）**：首轮把钩子接在
   `Node.prototype.insertBefore`（R219 汇流点）→ 定向 2F 原样。
   排查定谳：appendChild 与 insertBefore 是 proxy handler 上**平行
   独立实现**（part04 6575 / 7952），appendChild 不委托 insertBefore
   trap；原型方法只是无 own/trap 实现者（轻节点）的兜底——handle
   proxy 主路径完全不经过。再加 R334 sel 子臂（removed sel 元素重挂，
   6871 起 `__zw_insert_adjacent_sel_element` 后提前 return）与
   innerHTML setter 各自成路，实为四通道。
3. **innerHTML 面时序（探针 B 三轮取证）**：桥调用点枚举取**旧
   registry**（探针实证：钩子内 qSA=item __n1/__n2 旧子，同 turn 紧
   随的测试面 qSA=__n3/__n4 新物化子）——t8r 物化
   `_handleChildren[handle] = _ihMat`（part04 setter 后段）才是同 turn
   断言面读到的子对象；钩子必须在 registry 重建之后。中途假设「桥读
   触发惰性刷新」被探针否证（`_realTag` 桥读先行后枚举仍旧）。
4. **R334 臂 form 解析（探针 A）**：radio2 remove 后为 sel 形
   （`sel=#radio2 handle=null`），appendChild 进文档内 form 走 R334
   `__zw_insert_adjacent_sel_element` wire；wire 后 `.form` 活解析即
   命中新 owner（探针 A1 formSame=true）——出口处比对 before/after 键
   可靠。sweep 组枚举走 document qSA（插入后 radio2 链顶回文档），
   radio3（文档内 form 组员）可见。

## 施工（engine shim part03/04/05.js，无 Rust 变更）

1. part05 顶层新共享出口 `_zwRadioUncheckGroupOthers(el)`：s125
   set-trap sweep 体原样收编（type=radio/name 非空/checkedness true
   门 + s130 根域枚举 + form owner 键比对 + 脏件写 false/净件移除
   checked attr + `_zwAttrInstanceRemoveKey`/MO notify）；
   set-trap 块改 3 行调用（现象①面行为逐字节等价，`key` 自比改
   `_elKey(el.__zwSelector, el.__zwHandle)` 自比——与 trap `key`
   同源同值）。FIXME 更新：form owner 变更已接线（s131），余 checked
   属性增设/defaultChecked/type 变更/becomes connected。
2. part04 appendChild trap：函数头 before 快照（form owner 键
   handle/sel/id 串键，同 helper 键形；非元素 undefined 哨兵）+ 唯一
   tail 出口 after 比对 sweep。
3. part04 insertBefore trap：同款 head/tail（校验抛错不到 tail 不
   触发；moveBefore 7949 委托本 trap 顺带覆盖）。
4. part04 R334 臂：出口前同款 after 比对（before 复用函数头快照）。
5. part04 innerHTML setter：`_handleChildren[handle] = _ihMat` 之后
   逆 registry 序 sweep（`nodeType===1 && .form` 门 + helper 全门；
   逆序使后案 checked 胜出——children[1] 先 sweep uncheck children[0]，
   children[0] 轮到时 checkedness 已 false 被门跳过）。
6. part03 `Node.prototype.insertBefore`：三出口汇单出口 + 同款
   before/after 钩子（轻节点原型路径；与 trap 钩子幂等双 sweep 无害
   ——sweep 只置 false/移除 attr，二次空转）。

## 验证

- 定向：`FILTER=the-input-element/radio-disconnected-group-owner`
  **8/8 全 Pass**（2F→0：Removed elements 后半 + Appending …
  disconnect form 全案）。
- 定向：`FILTER=forms/constraints` **917P/3F 与 s129/s130 完全同额**
  （3 Fail = form-validation-validate fieldset 族，在账异根不动）；
  `FILTER=forms/the-input-element` **1511P/132F/3T**（s130
  1509P/132F/3T，join 翻动恰 2 行全 F→P、new/gone 0——探针页 2 键
  参跑在删页前一轮，join 已剔）。
- 全量 A/B × /tmp/s130-full.log 键级 join **恰 2 Fail→Pass**
  （radio-disconnected-group-owner 两案），**真翻红 0、new non-Pass
  0**；other 5 = s130 探针页 Fail 键出清（页已删不参跑）。唯一键数
  6781P/1029F/103T → 6783P/1022F/103T。
- engine lib **3570/0**；wpt-runner bin **256/0**（76 ignored）；
  clippy（zero-engine --all-targets -D warnings）零告警；本轮零 Rust
  变更（shim JS include_str!），fmt 不适用。shim 拼接（js_dom_bridge.rs
  装配序 10 part）node --check 过。
- **首轮全量 guard 单进程内存杀一次**（4194576 KB > 4194304 KB，超
  272 KB，启动早期；同命令重跑全程通过无杀）——账本如实记，非本修复
  引入（阈值边缘抖动形态）。
- 探针页 zw-probe-s131.html（gitignored wpt-data，assert 面包屑形）
  四轮取证后**已删**；语料零改动。常驻语料
  radio-disconnected-group-owner 8 断言在账即回归面。渲染面零依赖
  ——reftest/product-smoke 未跑。

## 挂账（本切片更新）

- **插入步 radio checkedness 同步族 2F 收口移出候选**
  （radio-disconnected-group-owner 全文件 8/8 在账）。
- **现象接线缺口细化入账**（FIXME 已入码）：form= 属性变更重排
  （input.form IDL setter / form 内容属性变更面）、移除方向（form →
  detached）、子树整体插入的深层 radio、_zwMEl registry 树 own
  insertBefore、sel 面 innerHTML（R304/R309 pending overlay 域）、
  outerHTML/insertAdjacentHTML；becomes connected（现象⑤）未接
  （无 corpus 面，最小范围让位）。
- 承袭不变：button-events「type=button shouldn't trigger」与该文件
  级 Timeout、empty.html/empty-with-base.html Timeout、
  form-submission-0 186F、execution-timing 构成面、lazy-loading 26F、
  meta color-scheme 21F、写流盖章断链、part02 setAttribute 绕
  notify、041 nested write、deep clone 非 HTML ns、css-module
  corpus 门（待决）、www2 可达性、module-import-referrer TLA、
  dynamic import() with 透传、classic 外链 SRI、143 write、defer
  多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList
  缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面让位累积；
  **待决（须用户拍板）**：css/text 数据模块语料资源缺失、
  green-2x2.png 语料资源分发、parser form element pointer 4F 缓行
  （2026-10-23）。

## 过程教训

- **JS 插入是平行 trap 拓扑不是单管道**：appendChild/insertBefore 是
  handler 上两个独立实现，原型方法只是无 own 实现者的兜底——「在共享
  出口接线」的直觉（Node.prototype.insertBefore）对 handle proxy 主
  路径完全不生效（首轮定向 2F 原样的根因）。改造前先取证目标调用的
  实际通道（钩子内 breadcrumb：h= 值对比测试面读数），再逐通道接线。
- **innerHTML 同 turn 双视图时序**：桥写（host apply）与 JS registry
  物化（t8r `_ihMat`）解耦，钩子位置必须在目标视图重建点之后——
  「紧随桥调用」是最差位置。定位手段：钩子内 breadcrumb 与测试面读数
  对照，同一 qSA 两个答案即视图错位。
- **runner 不透 console.log**：探针取证用 assert 失败 detail 带值出
  账（每 test 首个 assert 截断后续——多值拼一条消息或分 test）。
