# M2 s133 · form reset handle 身份枚举 + 清脏复位——reset-form-2 1F 收口 + form-nameditem 1 案搭车（真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s132（radio 组排他现象 4 type 变更信号接线）

## 入场定刀位

s132 收口后按控制面候选②入场：接线缺口 corpus 面勘根——defaultChecked
全语义树仅 reset-form-2.html 用例（1F「integration test on reset for a
created-from-script form」，首断言 `text.value` reset 后应「text
default」仍「text new value」）。刀位定性：脚本构造（detached）form
的 reset 算法面，非 defaultChecked/setAttribute 面本体。

## 勘根定谳

1. **根因 1（枚举盲区）**：`_formControls(sel)` 仅 sel 身份——walk 走
   `_wrapSelector(sel)`，`if (!sel) return []` 前置早退；纯
   createElement form（handle 身份，脚本构造不入文档）控件枚举恒空
   → reset 控件还原循环整体 no-op。part04 `_zwFormSubmitNavigate`
   注释已有同根已知限记（「handle 身份 form 的 _formControls 空列表
   → 空 entry list 导航」）——共享出口单点修复同时解除。
2. **根因 2（脏旗驻留）**：旧复位为赋值式（`c.value=c.defaultValue` /
   `c.checked=c.defaultChecked`）——IDL setter 重置 dirty 旗，复位后
   default 层变更不联动（L54「reset dirty value to false」/L55
   checkedness 同型断言面）。spec reset 置 dirty value / dirty
   checkedness 旗 false（值驻默认层）。
3. **R2998 捕获模型约束**：checkbox/radio 的 default 在 `_boolDefault`
   捕获不在 attr（前置 `.checked=false` 已移除 attr——reset-form.html
   rd1 案），复位必须经 `defaultChecked` 读捕获赋值后**再**清
   `_checkedDirtyState`——不能只清键（清键后回落 attr 读=false 翻红）。
4. **value getter 脏判谓词**：`_inputValues[key] != null &&
   (_inputValuesSet[key] || _stableValueKey(key))`——`delete
   _inputValues[key]`+`_inputValuesSet[key]` 即清脏（stable `#id`/`@`
   键 set 旗不参与判）；TEXTAREA 清键后落 child text 活文本读
  （R3906/s127），INPUT 落内容属性 latest-wins 读。
5. **基线误读排误（虚惊）**：resetting-a-form 定向 7 Fail 曾疑真翻红
   ——先前基线 grep 经 `sort | uniq -c | sort -rn | head` **截断**只见
   2 行；完整基线逐键比对证实 7 Fail 与 s131/s132 完全同额（click
   复位 6 面 + select-by-reset 1 面既红）。真翻红最终以全量键级 join
   为准（=0）。

## 施工（engine shim part04/05.js，无 Rust 变更）

1. part05 `_formControls(sel, handle)` 双身份：walk
   `sel ? _wrapSelector(sel) : _makeProxy(null, handle)`（嵌套 form
   整枝 / form= 属性跳过守卫对两身份同义——读经代理）；host 视图加
   `sel` 门（文档域查询，detached handle form 不在其中）。
2. part04 调用方 9 处传 handle：FORM get trap 区 7 处（elements
   迭代/get/namedItem/has、length、named access）+ `_zwFormSubmitNavigate`
   2 处（校验枚举 + entry list）。
3. reset 体：`if (sel)` → `if (sel || handle)`；TEXTAREA 赋值后清
   `_inputValues`/`_inputValuesSet`（OUTPUT 无脏键模型不动）；INPUT
   checkbox/radio 保留 `c.checked=c.defaultChecked` 赋值（R2998 捕获
   恢复；radio 组排他经 setter s125 臂照常触发）+ 清
   `_checkedDirtyState`；value-mode 赋值后清 `_inputValues`/`_inputValuesSet`。
4. 注释同步：part04 10968 既有局限 FIXME 标注 s133 解除；10687
   签名注更新。

## 验证

- 定向：`FILTER=resetting-a-form/reset-form-2` **1/1 Pass**（1F→0）。
- 定向：`FILTER=forms/resetting-a-form` **7 Fail 与 s131/s132 逐键
  同额**（虚惊排除后确认 0 翻红）；`FILTER=forms/form-control-
  infrastructure` **117P/4F 同额**；`FILTER=forms/the-form-element`
  **87P/21F**（基线 86P/22F，join 恰 1 F→P）；`FILTER=forms/
  the-input-element` **1512P/132F/3T 同额**；`FILTER=forms/constraints`
  **917P/3F 同额**。
- 全量 A/B × /tmp/s132-full.log 键级 join **恰 2 Fail→Pass**
（reset-form-2 主面 + form-nameditem「Input should only be a named
  property on the innermost form」——脚本构造嵌套 form 枚举面，共享
  出口修复合理外溢），**真翻红 0、new non-Pass 0、gone 0**；唯一键
  6785P/1021F/103T → 6787P/1019F/103T。
- engine lib **3570/0**；wpt-runner bin **256/0**（76 ignored）；
  clippy（zero-engine --all-targets -D warnings）零告警；本轮零 Rust
  变更（shim JS include_str!），fmt 不适用。shim 拼接（13 part）
  node --check 过。无探针页；语料零改动。全量 runner 以 corpus 已知
  Fail 非零退出（make 错误 1，前轮同形非异常）。渲染面零依赖。

## 挂账（本切片更新）

- **reset-form-2 1F 收口移出候选**；handle 身份 form 枚举盲区
 （form.elements/length/named access/submit entry list）**共享出口
  收口**。
- **reset 域既有异根显性入账**：①click 复位 6 面（reset-form.html
  input/textarea/select × click button/input）——reset 按钮激活路径
  仅派发 reset 事件**无控件还原臂**（part03:17793 click 通道；spec
  激活行为 = form.reset()），input/textarea 两对面接共享还原即可翻、
  select 对面需先收 ②；②select-by-reset 1 面（R3909 选择性复位面
  既红，select 断言前置）。③form-nameditem 残余 10F（命名 getter
  机制面异根，本轮 1 案搭车翻绿）。
- 承袭不变：button-events「type=button shouldn't trigger」与该文件
  级 Timeout、empty.html/empty-with-base.html Timeout、
  form-submission-0 186F、execution-timing 构成面、lazy-loading 26F、
  meta color-scheme 21F、写流盖章断链、part02 setAttribute 绕
  notify、041 nested write、deep clone 非 HTML ns、css-module
  corpus 门（待决）、www2 可达性、module-import-referrer TLA、
  dynamic import() with 透传、classic 外链 SRI、143 write、defer
  多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList
  缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面让位累积、s131
  插入步缺口细化、radio 现象⑤ becomes connected / checked 属性增设
  /defaultChecked 无独立 corpus 面（现象①覆盖）；**待决（须用户拍
  板）**：css/text 数据模块语料资源缺失、green-2x2.png 语料资源分
  发、parser form element pointer 4F 缓行（2026-10-23）。

## 过程教训

- **基线比对必须全量逐键**：`grep | sort | uniq -c | head` 类截断
  管道会造出「新增翻红」假信号（本轮 resetting-a-form 7F 误读，完整
  join 后排除）。真翻红判定唯一标准 = 全量键级 join 的
  Pass→nonPass 集合。
- **共享出口的已知限记是勘根捷径**：`_formControls` handle 盲区在
  `_zwFormSubmitNavigate` 注释已有记录（二轮 review note）——入场
  勘根时先翻同类 helper 的既有 FIXME/限记注释，可比探针更快定位根因。
- **清脏复位 ≠ 裸清键**：R2998 捕获模型下 checkbox/radio 的 default
  已不在 attr——顺序必须是「赋值恢复（捕获→attr）→ 清脏键（读回落
  attr）」，跳过赋值直接清键会把前置 dirty 写造成的 attr 缺失读成
  false（reset-form rd1 断言面）。
