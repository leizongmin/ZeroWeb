# M2-S213：option/datalist 元素 API 面——Text isConnected + datalist.options IDL

- **日期**：2026-11-01
- **目标面**：option-text-setter（option.text setter 八断言，1F）+
  datalistoptions（datalist.options 集合断言，1F）合计 2 份语料 2 键在册 F
- **状态**：收口——两份靶语料全 Pass（2 键 F→P 闭合）；forms 全域 A/B
  净 F→P=3（2 靶键 + onblur flake 族 1 键摆回），P→F=0，共享键零漂移

## 勘根

- **入场**：s212 后 misc 残面盘点（s212-forms-wip-final.log 全域 F 逐一
  定性：customizable select 177F 挂起族、parser form element pointer 缓行
  族、override builtins 待决、click() 返回值高风险不入刀、渲染流域、
  onblur flake 族）——余下可施工孤面 = option/datalist 元素 API 2F。
- **option-text-setter 1F**：`option.text` setter 本体 R3907 已实现（
  part04 委托 textContent——spec #dom-option-text「string replace all」
  同步骤），失败点在断言 2 `textChild.isConnected` 返 **undefined**：
  `_wrapNodeEntry` 解析 text/comment 静态对象（part05）无 isConnected
  字段，其原型链 Text.prototype→CharacterData.prototype→Node.prototype
  亦无（Node.prototype 仅常量族/remove/cloneNode/getRootNode）。断言 4
  旧子 isConnected false 还受静态 `parentNode` 构造时指向的 stale 语义
  制约——string replace all 剔除后旧子 parentNode 仍指原父。
- **datalistoptions 1F**：`datalist.options` IDL 完全缺位（undefined →
  "Cannot read properties of undefined (reading 'length')"）。
- **spec 定谳**：datalist.options = [SameObject] HTMLCollection rooted
  at the datalist node，filter **朴素匹配 option elements（无排除条款）**
  （HTML §4.10.8 #dom-datalist-options）——语料 5 option 全在 datalist 内
  select 里，朴素后代全收才计 5。与 select.options 的
  concept-select-option-list 排除语义（`_zwListOfOptions`：select/hr/
  datalist 子树排除 + optgroup 嵌套整枝排除）**不同源，不复用**。
  isConnected = shadow-including root 是 Document（dom #dom-node-
  isconnected）；option.text setter = string replace all（已对齐）。
- **不能伤的键**：R3908 select options/selectedOptions（`_zwListOfOptions`
  排除语义原样）；R3907 option.text 既有 pass 面（option-text-spaces/
  recurse/option-label）；元素 proxy isConnected trap 臂（part04，先于
  原型链命中不受扰）；textContent sel 形 setter 的 R81/R189 replace-all
  记账（旧子 pending removed overlay 剔除——旧子身份不进融合视图是
  isConnected membership 核对的前提，WPT「Preexisting Text should have
  been removed」已钉）。

## 施工

（shim 1 文件 + rust 测试 1 文件；语料零改动——两份靶语料上游既有）

- **part03 Node.prototype.isConnected 访问器**（getRootNode 后）：
  parentNode 链上行 + **每跳 membership 核对**（`cur ∈ cur.parentNode.
  childNodes`——indexOf miss 即断链返 false，R84 sibling getter 同款
  live 判据，兜住 `_wrapNodeEntry` 静态 parentNode 的 stale 指向；视图
  不可得 best-effort 继续走链）+ nodeType 9 → true + p === cur 环守卫 +
  guard 64。enumerable:false（proto 面 for-in 零进 R3046 约定）。
  服务 plain 节点家族（_wrapNodeEntry 解析 text/comment、_zwMText/
  createTextNode 产物）；proxy 元素 get trap 臂先命中不受扰。
- **part03 `_zwDatalistOptionList`**：朴素后代走查（OPTION 收录、其余
  下钻、无排除条款；sel/handle 双形态根合并同 `_zwListOfOptions` 脚手架
  ；guard 64）。
- **part03 `_zwDatalistOptionsCollection`**：Proxy 镜像 R3908
  `_zwSelectedOptionsCollection` 形态（外层 [SameObject] WeakMap 缓存
  ——复用 `_zwOptionsCollCache`（键为元素 proxy，与 select 零碰撞）+
  每读重建内层 live 视图 `_zwMakeCollection`）；读面
  length/item/namedItem/索引/迭代器/constructor/toStringTag + has/
  getOwnPropertyDescriptor（R3908 同款 invariant 安全形态）。
- **part03 get trap 分发臂**（select 块前）：`prop === 'options' &&
  _realTag === 'DATALIST'` → `_zwDatalistOptionsCollection`。
- **part09.rs R5380**：腿 A = option-text-setter 八断言镜像（in-document
  option：isConnected true→false 翻转、旧子 nodeValue 不变、identity
  更替、新 firstChild nodeValue、`.text` getter、`''` 后 firstChild
  null）；腿 B = datalistoptions 六断言镜像（length 5、item/索引
  identity、label/value 数组、[SameObject]、willValidate barred）。

## 验证

- 拼接 shim（15 part include 序）node --check SYNTAX_OK。
- R5380 单测 **1 passed**（两腿一次过）。
- 两份靶语料全 Pass：option-text-setter **1/1**、datalistoptions
  **2/2**（终跑日志 evidence/logs/s213-forms-wip.log）。
- forms 全域 A/B（s212-forms-wip-final.log 为 B 侧，(key,state) 对全
  测试名整行键 sort -u join——LC_ALL=C；带空格测试名 awk 截断叉乘坑
  s139 在册）：共享键 **4700 对零漂移**；**净 F→P=3**（两靶键 +
  checkable-active-onblur-with-click 1 键——在册 onblur flake 族 s209
  ~s212 双向摆动承袭，本轮摆回 Pass，族内闭环非回归）；**P→F=0**。
  键集差 = beforeinput.tentative.html 单文件 16 对（B 侧文件级 Timeout
  无结果行——s202 起同态，A 侧跑全 16F 为既有失败面，beforeinput 事件
  面未实现，与两刀位无耦合）。
- zero-engine lib 全量 **3673P+0F**（3672+R5380）。
- guarded-clippy 0 警告；`cargo fmt -p zero-engine -- --check` EXIT=0。

## 挂账

- host 侧 `is_effectively_disabled` 平面走查与 shim 边界集模型统一
  （s212 挂账承袭，独立切片）。
- part04 willValidate 臂内联 fieldset 走查共享化（s211 承袭）。
- `__zw_has_attr` 非 lw 存量点位审计（s211 承袭）。
- onblur flake 族持续采样（本轮 with-click 摆回 Pass，族内双向闭环）。
- beforeinput.tentative.html 文件级 Timeout 方差 + 跑全 16F 既有面
  （beforeinput 事件未实现——录入事件族独立评估，本轮仅记账）。
- 上游 customizable select 特性面 177F 挂起待拍板承袭（GB-20261031）。
- form owner table / parser form element pointer 缓行族（用户决策承袭）。
- form-nameditem override builtins 待决（承袭）。
- input-type-button click() 返回值面（共享通路高风险，未入刀，承袭）。
