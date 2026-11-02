# M2 s231 — forms misc 散单分型 → switch 反射布尔 IDL 面（INPUT 门 get/set 双臂）+ radio-focus 双视图分歧勘根挂账

日期：2026-11-02。切片号 s231（承 s230）。

## 入场定刀位

s230 下一步候选：forms misc 散面续勘（checkbox-switch 3F 或散单）。散单聚类
（s230-ab.log 数据源）：email-set-value 1F、input-type-change-value 2F、
checkbox-switch 3F、input-valueasnumber 1F、input-type-button 1F、
inserted-or-removed 1F。定刀 checkbox-switch 3F——三子测试
（setter/getter/cloning）失败签名同根概率最高。

## 勘根

测试 `input-type-checkbox-switch.tentative.window.js`（36 行，tentative
开关外观提案的 IDL 面）：

- it1 setter：`input.switch = true`（typeless input）→ hasAttribute/getAttribute
  `""`、type 保持 text；
- it2 getter：innerHTML `<input type=checkbox switch>` → `input.switch === true`；
- it3 cloning：switch=true → cloneNode → 属性/type/switch 三断言。

spec：HTML §switch control（tentative）
https://html.spec.whatwg.org/multipage/input.html#switch-control——`switch`
内容属性反射布尔 + IDL `dom-input-switch`（`[Reflect]` 语义 presence 读）。
shim 全库 grep 无 `switch` 臂——setter/getter/clone 全断。

施工面选型核对 s187 先例（`alpha`、`colorSpace`）：**不入
`_REFLECTED_BOOL` 通用表**（part01）——通用表会给非 INPUT 元素造 `.switch`
假面 + CE 反射映射（`_zwCeReflectionAttr`）波及；改在 part03 get 陷阱 /
part05 set 陷阱各加显式臂，`_realTag(sel, handle) === 'INPUT'` 门。
descriptor 面（gOPD）不动：s187 alpha 亦无 gOPD 臂，断言面不探 descriptor。

## 施工（+30 行，两文件，纯新增）

- `crates/engine/src/js_dom_shim/part03.js`：get 陷阱 colorSpace 臂后加
  `switch` 臂——INPUT 门；presence 读走 `__zw_has_attr_handle` /
  `__zw_has_attr_lw`（R136 latest-wins：同批 setAttribute 后 IDL 读
  latest-wins）→ `=== '1'` 返 boolean；非 INPUT 无臂（陷阱 fallthrough
  天然返 undefined，spec 真）。
- `crates/engine/src/js_dom_shim/part05.js`：set 陷阱 colorSpace 臂后加
  `switch` 臂——truthy → presence 设空（`__zw_set_attr[_handle](
  handle|sel, 'switch', '')`）；falsy → 真移除（`__zw_remove_attr[_handle]`
  + `_zwAttrInstanceRemoveKey` 同步 + moAttr 记账）；detached 亦真移除
  （R3039 同款语义——旧 generic fallthrough 写 `attr="false"` → 读恒真
  bug 面不复现）。

## 验证

- solo：checkbox-switch 3 子测试 Fail → **3×3 全 Pass**（终验 3 轮复跑
  确认）。
- 域 A/B：全树 (key,state) join（对比 s230-ab.log 基线，8098 键）——
  gained/lost 各 6 行：
  - **目标翻转恰 3**：switch setter/getter/cloning F→P；
  - **扰动 3**（P→F，全部证伪 diff 因果，见下节）。
  全树 7502P/518F/81T（与基线逐位同分布；日志 s231-ab.log）。
- 插桩验后撤净：testharness.rs 临时 [td-cmd] 命令级 eprintln 一处，
  git diff 零残留；探针 HTML 验后删；Rust/JS 零临时改动入库。

## A/B 洁净度核查（扰动键三分定谳）

三个 P→F 键逐个 stash A/B（改动搁置 → HEAD 纯态连跑 3×）：

1. **checkable-active-onblur-with-click :: following label（checkbox）**
   HEAD 纯态 3×3 恒 F——与 diff 无关；本文件族即 s196/s197 专勘的在册
   Actions flake 族（挂账「following-label 2F」），s228-s230 基线该键
   P 是 flake 幸运侧，本轮翻回常规侧。
2. **radio-focus-navigation-checked × 2**（Focus the checked radio in
   the group. / Focus the new checked radio in the focused radio group.）
   HEAD 纯态 3×3 恒 F——与 diff 无关，但**非在册 flake**：结构根，勘根
   见下节，转新挂账。

## 新挂账：radio-focus-navigation-checked 双视图分歧（本轮勘根闭环，未修）

失败签名：`Unhandled rejection "Error: testdriver target not found:
input[checked]"`（it1/it3，两测试都含 Tab 落动态 checked radio `c` 的
导航步；it4 落有 id 的 b 不触发）。勘根链（命令级插桩 + 探针 2 轮）：

- **锚定链**：stub `Actions.send()` 对纯键盘链取
  `element = document.activeElement`（send 时点）——Tab#3 send 时
  activeElement 已是 `c`（createElement 产物、无 id、IDL
  `c.checked = true`）→ `queuedElements[id] = c`。
- **生成链**：宿主出队调 `__zw_td_selector(id)` → stub `selectorFor(c)`：
  arm1 `__zwSelector` 缺（handle 族元素）；arm2 `__zw_selector_for_handle`
  返**空串**（探针实证）；arm3 无 id；arm4 属性唯一化——
  `getAttributeNames()` 含 `checked`（**IDL checked 写入落属性表**，
  `hasAttribute` 实证）且页内 qSA `input[checked]` 命中且唯一（探针实证）
  → 返 `'input[checked]'`（arm4 产出后无复验）。
- **解析链**：宿主 `page_node_ref_for_selector('input[checked]')` →
  pipeline `cached_doc` **native live Document** Rust 侧 query_selector
  → **miss**（native 树无该 attr-instance 投影）→ error → R347 重入队
  20 次 → 全 miss → promise reject → 子测试 Fail。
- **降级臂跳过原因**：M2-S195 键盘链动态焦点降级臂只在 selector **为空**
  时接管；本页 selectorFor 非空但宿主不可解析——降级臂被绕开，设计盲区。
- **分歧本质**：shim 视图（mutTree attr-instance，IDL 写入即时可见）vs
  native doc（DOM mutation 投影面）对同一 IDL checked 写入可见性不一致；
  stub arm4 校验走 shim qSA、宿主解析走 native doc——跨视图断裂。
- **漂移未定谳**：solo 恒 F（7/7，含 stash 纯态 3）而 s228-s230 三轮全树
  基线 P——疑 runner 8-job 进程内跨文件状态或文件分配序扰动；输入位
  （源/语料/旗标）逐位一致已实锤，机制留探。
- **修复方向候选**（下轮勘根起点）：(a) stub arm4 产出后补宿主可解析性
  复验，不济落 nth-of-type 形；(b) IDL checked 写入同步投影 native doc
  （mutation 面对齐）；(c) 宿主 resolve-miss 时键盘臂回落 activeElement
  通道（M2-S195 原意图延伸；限 keydown/keyup，不动指针步 R347 重入队
  语义）。

## 承袭挂账（更新）

- 散单余 6F 各自独立根（签名已录）：email-set-value（可见值净化面）、
  input-type-change-value 2F（type 切换 value 脏旗）、
  input-valueasnumber（**time 出界值 wrap 语义**——spec 数字→time 串按
  ms 对 24h 取模、负值上卷：`-3600000`→`"23:00"`、`2.73e26`→浮点取模
  `"10:54:10.944"`；域内值已过，仅出界例挂）、input-type-button
  （default behavior undefined vs true）、inserted-or-removed（select
  最后选中 OPTION innerHTML 面）。
- radio-focus-navigation-checked 双视图分歧（本节新账）。
- 其余承袭不动（keyboard 余 2F 车道挂账；table/label 簇用户门禁；
  parent-label 布局零盒 2F；color.window.js 语料缺口补资产流等）。

## 方法论教训

- **A/B 扰动键必须逐键归因**：目标翻转之外的同轮翻转不能以「时序 flake」
  一笔带过——stash A/B（改动搁置→纯态复跑）是零成本定谳手段，本轮 3 键
  全部实跑归因，其中 2 键挖出结构性新账。
- **「sel=None 的键盘命令报 target not found」＝ send() activeElement
  锚定链信号**：错误 selector 不是 Actions 目标而是 `__zw_td_selector`
  现场解析产物；追错误串先 grep 构造点（唯一）再反推调用臂，panic 面
  （unwrap None）即可排除构造点。
- **探针控制组要按文档序语义设计**：querySelector 返回首个匹配——双
  checked radio 控制组取「第二实例」必假红，本轮探针 2 假 fail 即此
  设计错误（已废，不影响定谳）。
