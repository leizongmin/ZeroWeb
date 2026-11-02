# M2 s233 — valueasnumber time 出界值 wrap（24h 双精度取模上卷）1F→P

日期：2026-11-02。切片号 s233（承 s232）。

## 入场定刀位

s232 下一步候选首选：valueasnumber time 出界 wrap——s232 预勘根已闭环
（setter time 臂 `new Date(vsN)` 超 JS Date ±8.64e15 ms 值域落 Invalid
Date → value 落空；fmod 24h 双精度取模 node 实证精确复现 WPT 期望
`"10:54:10.944"`），本轮零新勘根直接施工。

## 施工（+15 行：8 注释 + 7 代码，单文件单臂）

`crates/engine/src/js_dom_shim/part04.js` valueAsNumber setter 末分支拆
三路：NaN → ''（原样）；**time → 先 wrap 再构造 Date**
（`((vsN % 86400000) + 86400000) % 86400000`，注释记 spec 无
number→string 算法定义、浏览器实测语义、WPT 两例锚：-3600000→"23:00"
域内天然已过 / 2.734e26 必须先模）；其余 date/week/datetime-local 保持
`new Date(vsN)`（epoch 基语义不同不动）。getter 无需对称（value 串恒
域内）。

## 验证

- solo：input-valueasnumber 四文件 76 子测试全 Pass（64+1+7+4，修复前
  63P+1F=64）——目标 1F→P，stepping/typeerror/invalidstateerr 邻文件
  零回归。
- 域 A/B：全树 (key,state) join（对比 s232-ab.log 基线，8098 键）——
  lost/gained 各恰 2：目标键 F→P + checkable-active-onblur-with-click
  following-label F→P（在册 s196/197 flake 族幸运侧回翻——该键近四轮
  P→F→F→P 摆动，与本刀 time 格式化臂无通道，不归因）。全树
  7502P/518F/81T 与基线逐位同分布（日志 s233-ab.log）。
- 零插桩零探针。

## 散单预勘根新账（等待窗口产出，下轮候选）

- **input-type-change-value 2F 根定谳**：type-change 步（part01
  `case 'color'` 后通用段）把 color 模式默认净化值物化进存储并置
  `_inputValuesSet=true`——切回 text 携带 `"#000000"` 而非内容属性原值
  `""`。spec 语义 = color 默认 `#000000` 是**读侧合成不落存储**（dirty
  保持 false，change-out 按 spec 步骤回落内容属性）。修复方向三步：
  (a) change-into-color 不物化（存储留空）；(b) color 读侧只读合成
  `#000000`（存储空且非脏时）；(c) change-out 按 `_typeChangeTainted`
  标记携带 attr 值而非 store（s194 FIXME 镜像模型张力面）。
- **email-set-value 1F 面**：`send_keys` 键入未净化原文 → `input.select()`
  后 `window.getSelection().toString()` 应读回可见未净化文本，我方返
  `""`——文本控件 selection ↔ window.getSelection 集成面（内部
  selection 状态 R2844/R3906 未接 document selection 对象）。中面，非
  单行修。
- **input-type-button 1F**：`expected (undefined) undefined but got
  (boolean) true`——「button with no default behavior」断言，待读测试
  定位属性面（下轮勘根起点）。
- **inserted-or-removed 1F**：innerHTML 插入 SELECT 的 OPTION 时
  「last selected OPTION wins」未生效（parser/DOM API/jQuery append 三
  路全过仅 innerHTML 挂）——innerHTML setter 的 select 子树 selectedness
  处理缺位，疑连 s228 挂账「innerHTML sel 容器 identity 架构」域，下轮
  勘根先核对是否同根。

## 承袭挂账（更新）

- 上述散单四案签名与根面已录（本轮预勘根产出）；keyboard 余 2F 车道
  挂账；table/label 簇用户门禁 GB-20261031；parent-label 布局零盒 2F；
  checkable following-label flake 族在册（近四轮双向摆动实证）等不动。
- 环境注记：本轮 A/B 收尾时段并行车道进程同机在跑（reftest-upstream +
  另一 clone 的 testharness-form-url-encoding，ps 实证）——本日志在其
  启动前已稳定收尾（8101 行两次核对不变）互不污染；后续时序敏感单案
  判谳前可 ps 核对并行负载（s231 solo-F/全树-P 漂移疑点的旁证候选，
  已随 s232 修复闭合不再追查）。

## 方法论教训

- **预勘根写透账本 → 下轮零勘根施工**：连续第二轮验证该模式（s232→
  s233），从定刀到 A/B 全绿无仪器——「先模后构造 Date」的 node 数值
  先验（fmod 39250944 → 10:54:10.944 精确复现）让施工面只剩落位。
- **A/B 扰动键归因看通道不看方向**：本轮 flake 键 F→P 是「有利方向」
  但仍须核对改动通道（time 格式化臂与 :active checkbox 无任何共享面）
  + 历史摆动记录（近四轮双向）才能不归因——有利方向的扰动键更容易被
  顺手算进修复成果，虚增战果。
