# M2 s237 — email-set-value select()↔document selection 桥 + 键入 raw 读（1F→P）

日期：2026-11-02。切片号 s237（承 s236）。两层根、两施工位（part04 select() 臂重构 + part01 键入/删除 raw 读）。

## 入场定刀位

s236 下一步候选：email-set-value 1F（文本控件 selection ↔ window.getSelection
集成面，中面非单行修）。

## 勘根（仪器：探针 assert_true(false, dump) + `__zw237dbg` 临时插桩，验后全撤）

失败面（email-set-value.html 单子测，入场 1F）。测试序列：send_keys
' foo@bar   '（11 键含首尾空格）→ assert validity.valid → `input.select()` →
assert value==='foo@bar'（value getter 读侧净化）→ assert
`getSelection().toString()===' foo@bar   '`（**可见键入原文**）→ value 再断
'foo@bar' → select() → toString 断 'foo@bar'。

- **根 1**：`el.select()` 只更新内部选区态（part04 `_textSelection`），从不映射
  document selection——`getSelection().toString()` 恒 ''。Chromium 口径：
  单行输入的可见文本选区就是 document selection 面，select() 全选生效即落
  document selection。
- **桥门设计（上游语料三测并存校验，Chromium 全绿不矛盾）**：
  selection-not-application.html 断言 email 的 text field selection API 门禁
  （selectionStart 恒 null、setter 抛 InvalidStateError）；selection-weekmonth.html
  断言 month/week 也支持 select()（select() 本身无 type 门）；
  email-set-value 断言 email 有可见文本编辑器。⇒ 桥适用 = 文本控件 ∪ email，
  内部选区分支 `_isTextControl` 门保持不动。
- **根 2（根 1 修复后暴露，双层根）**：store 脏值被打折——`__zw_text_input`
  每键 `cur = el.value` 读**净化 getter**，email/url/tel 净化族每键从净化形
  重建把可净化字符折叠：' foo@bar   ' 逐键实证折叠成 'foo@bar '（t2 'f' 拼在
  净化后空串上、t10/t11 尾随空格再折叠）。R3901 `__zwSkipValueSanitize` 旗标
  只保「本次写入不净化」，保不住「下次重建的读数」。修复 = 键入/删除路径读
  RAW `_controlValue`（store 脏值/内容属性回落）；value getter 读侧净化不变；
  text 族 raw===净化形零差异。

## 桥力学（part04 select() 臂）

- `_makeRange` 声明于 part06（**独立段作用域**）part04 不可见（typeof 守卫
  静默 null 一轮踩坑）——改经公开 API `document.createRange()` 取同一产物。
- live-view range：setStart/setEnd (el,0)-(el,0) 数据槽端点恒折叠；实例自有
  `toString` 属性覆写（走查落 `_zwRangeRenderedText` textarea/input 祖先臂
  range.toString() 回落，FIXME 在账）现读 `_textSelection[key]` 偏移截 RAW
  原文；email 无内部选区态（`useSelState=false` 门，先前 type 遗留 stale 偏移
  不会再被消费）→ 全文。
- 全覆盖特判：st.start===0 && st.end>=净化形长 → 返回完整 raw（内部选区偏移
  在净化值空间 R2844/R3906 既有模型，raw≠净化形 type 无逐点映射，部分选区按
  同 offsets 截 raw，语料无钉面 FIXME）。
- `collapsed` getter 自洽现读（= toString 空——数据槽端点恒折叠，否则与
  toString 非空矛盾）。
- 直接赋 `_ranges`/anchor/focus 不经 `_zwSync`：文本控件级 selectionchange 已
  按 changedSel 排程（上方既有臂），document 级不双派
  （textcontrols/selectionchange 事件计数面）；typing 包裹态不清（`_zwSync`
  显式变更才清）。email 无内部选区面不排程事件（既有 no-op 口径零回归，
  Chromium 行为语料无钉）。

## 施工（+86/-4，两文件）

1. part04 `prop === 'select'` 臂：`_selIsText` 内部选区更新保持；新增 document
   selection 桥（门 = `_selIsText || type==='email'`，s237 注释 + 语料链 +
   跨段作用域注）。
2. part01 `__zw_text_input`：cur 读 `_controlValue` raw（catch 回落 el.value），
   s237 注释记净化族折叠坑。
3. part01 `__zw_text_delete`：非 number 臂同 raw 读（number 编辑器臂原文双槽
   不动）。

## 验证

- solo email-set-value 全 Pass（修复前 1F；探针两轮：桥形态一轮 → store 打折
  双层根定谳一轮）。
- solo textfieldselection 目录 **784P/0F**（R2844/R3906 战阵面零回归）。
- solo selection 族 856P/1F——modal-dialog-selection『By default, text inside
  a modal dialog can be selected』**存量定谳**：s236 基线同键同态 Fail
  （同文件 3 键逐键对账同态），且该测走 testdriver pointer 拖拽
  （css-ui-4 content-selection 面）不触 select()/桥，无通道非回归，记基线态
  不归因。
- 全树 A/B（evidence/logs/s237-ab.log vs s236 基线 s236-ab.log，(key,state)
  对 sort -u join）：**8098 键 = 8098 键零增删，恰 1 键 Fail→Pass = 目标键
  email-set-value，零 P→F 零其他位移**（承 s232/s234 后第三个完美 A/B）；
  行级分布 7511P/509F/81T → 7512P/508F/81T，ΔP=+1/ΔF=−1 精确闭合。
  selection 族 solo 轮现形的 modal-dialog-selection 1F 双侧同态（基线键），
  join 不现位移——solo 单域跑与全树负载形差异记档（全树该键亦 Fail，
  基线态非新扰动）。
- node --check 全 shim concat（canonical 序 part01,01b,02…08d，js_dom_bridge.rs
  include_str! 序提取）语法干净；`__zw237dbg` 零残留；探针
  zz-s237-probe.html 验后删。

## 残余挂账（本切不动，FIXME/注释已注记）

- 部分选区 raw 截取按净化空间 offsets 直落 raw（raw≠净化形 type 无逐点映射，
  语料无钉面 FIXME，part04 桥 toString 注释已注）。
- `_zwRangeRenderedText` textarea/input 祖先臂 range.toString() 回落
  （既有在账 FIXME，本切搭桥受益其回落路径）。

## 下一步候选

color.window.js 语料缺口（补资产 flow）；plain value 死路补面（s236 新账，
与 s228 车道交叉评估）；modal-dialog-selection（css-ui content-selection
pointer 拖拽面，本轮定谳存量，是否立车道待评估）。
