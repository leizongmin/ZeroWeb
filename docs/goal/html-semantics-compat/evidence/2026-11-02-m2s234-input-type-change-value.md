# M2 s234 — input-type-change-value 变更链污染值不携带（taint-aware cur 派生）2F→P

日期：2026-11-02。切片号 s234（承 s233）。

## 入场定刀位

s233 散单预勘根已闭环根定谳：type-change 步（part01 `_zwApplyInputTypeChange`）
把 color 模式默认净化值物化进存储（`_inputValues[key]='#000000'` +
`_inputValuesSet[key]=true` + s194 `_typeChangeTainted[key]=true`），切回
text 时 `cur` 从 store 读 `"#000000"` 而非按内容属性回落 `""`。spec 语义 =
color 默认 `#000000` 是读侧合成不落 dirty 值（input type change 步骤 2：
非真脏 = value 内容属性值）。本轮零新勘根直接施工。

## 施工（+9 行：8 注释 + 1 代码，单文件单点）

`crates/engine/src/js_dom_shim/part01.js` `_zwApplyInputTypeChange` 的
`cur` 派生处插一行守卫：`_typeChangeTainted[key] === true` → `cur = null`，
强制走既有 attr latest-wins 回落分支。选 (c) 案（change-out 按 taint 标记
携带 attr 值）而非 (a) 不物化 / (b) 读侧合成：s194 标记机器现成（三写点
语义已核对——`setAttribute('value')` 污染态落 store 且清标记
part04:3825-3831、value IDL set 清标记 part04:14606），污染态仅由变更步
自身产生，修复面最小且不触碰 color getter 读路径。

语义闭环：真脏（value IDL set / setAttribute / 键入）写点已清标记 → 走
原 store 携带（dirty `'#ffffff'` 跨 type 保持，同测试步骤 4 断言）；
污染链（连续变更）每步回落内容属性。

## 验证

- solo：`FILTER=type-change` 前缀两族 626 子测试 **626P/0F/0T**——目标
  input-type-change-value 双子测试（Without focus / With focus）2F→P；
  邻域 type-change-state 全状态矩阵（text/email/password/datetime-local/
  color 等两两变更 ~350 断言）零回归，type-change-to-password、
  type-change-to-same-type、focus-dynamic-type-change 等全过（s194 既有
  钉面不动）。
- 域 A/B：全树 (key,state) join 对比 s233-ab.log 基线（8098 键两侧）——
  lost/gained 各恰 2 且同为目标键：`input-type-change-value` With/Without
  focus 双键 F→P，**零扰动键**（承 s232 后第二个完美 A/B）。分布
  7508P/512F/81T，对基线 7506P/514F/81T 差恰 +2P/-2F（日志
  s234-ab.log）。四轮链条 s231→s234 = 7502→7504→7506→7508P，每轮
  +2P/-2F 全部可归因。
- 零插桩零探针（承 s232→s233 预勘根闭环模式，连续第三轮）。

## 基线数字勘误（s233 账本）

s232/s233 两本账本把当轮全树分布误抄为 s231 的数字（"7502P/518F/81T"，
该对数字对 s230/s231 是对的——s231 净零扰动「同分布」成立）；日志现重
计：s233-ab.log = 7506P/514F/81T、s232-ab.log = 7504P/516F/81T（s231 =
7502P/518F/81T、s230 = 7502P/518F/81T，总数 8101 五轮一致，s232 起
+2P/-2F 单调链完整）。s233「逐位同分布」措辞与其自身「lost/gained 各
恰 2」键级结论本自相矛盾（有 2 翻转即非同分布）——正确表述 = 对 s232
基线差恰 +2P/-2F = 两翻转键。本轮修正两账本及控制文档引用数字，防止
后续轮按错基线对表。教训：落账分布数字必须从归档日志现算，不得从上一
轮账本转抄；键级结论与行级数字矛盾时以键级 join 为准回查行数。

## 承袭挂账（不动）

- email-set-value 1F（文本控件 selection ↔ window.getSelection 集成面，
  中面非单行修）；input-type-button 1F（`expected undefined got true`，
  测试未读）；inserted-or-removed 1F（innerHTML SELECT selectedness，疑
  连 s228 挂账同根，下轮勘根先核对）；color.window.js 语料缺口（补资产
  flow）。
- keyboard 2F 车道挂账；table/label 簇用户门禁 GB-20261031；parent-label
  布局零盒 2F；checkable following-label flake 族在册。

## 环境注记

本轮 A/B 期间并行车道进程同机在跑（另一 clone `/tmp/zw-enc-target` 的
testharness-form-url-encoding euckr 单测，ps 实证 11:31 起）——独立
target 目录与本车道无共享面；键级 (key,state) join + 扰动键逐键归因纪律
兜底时序扰动。
