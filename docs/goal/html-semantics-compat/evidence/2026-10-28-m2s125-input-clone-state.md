# M2 s125 · input 克隆状态传播收口（deep clone HTML-ns 物化 + form 状态 walk + radio 组排他——clone.html 19F→19/19 全 Pass，radio.html 搭车 4F→0，真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s124（modulepreload 取回面收口）

## 入场定刀位

s124 下一步候选为挂账序复核（form-submission-0 / execution-timing / meta
color-scheme），均为大基建/跨流面。全量基线（/tmp/s124-full.log）翻拣
forms 域单根面：`forms/the-input-element/clone.html` 整页 19F（4 checked
+ 15 value）——克隆状态传播单根候选，入场。

## 勘根定谳（探针页 gitignored corpus 验后删）

1. **deep clone 子树是 `_zwMEl` plain 视图（无 handle、无 IDL 面）**——
   host 元素 deep cloneNode 走 innerHTML 序列化重解析
   （`__zw_get_inner_html` → `_zwFragmentAdded`），重解析产物是 plain 解析
   视图。克隆 input 的 `.value`/`.checked` IDL get 无脏缓存可读、无键入
   `_inputValues`/`_checkedDirtyState` 表——form 状态传播断链。19F 主簇：
   4 checked（checkedness 不随克隆）+ 15 value（value IDL 值不随克隆）。
   浅克隆臂（R3901/R3902b）状态传播正常（隔离探针 B1/B2 Pass 对照）。
2. **第 19 失败根因（修复 18/19 后余 1 定谳）**：corpus setup 跳过
   `inputs[3].checked = false`（只设 inputs[2].checked = true）——
   HTML §4.10.5.1.16 Radio Button state 现象表「The element's checkedness
   state is set to true (for whatever reason)」要求同组其余 radio
   checkedness 置 false，引擎 checked IDL setter（part05）无组排他逻辑
   （grep `_checkedDirtyState` 全 shim 仅 setter/getter/浅克隆臂三处）。
   inputs[3] 的 checked 属性存留 → 序列化带 checked → 克隆读 true。
3. **序列化面不含 pending mutation**：`__zw_get_inner_html` 只含已 apply
   面——`.checked = false`/组排他的 RemoveAttr、`.value =`（value 族不
   反射属性）均不在序列化产物——克隆侧需显式状态对齐，不能靠属性复制。

## 施工（engine shim part04.js + part05.js，无 Rust 变更）

1. **deep clone HTML-ns 物化**（part04 proxy cloneNode deep 块）：
   `_zwFragmentAdded` 产物逐顶层子经 `_wcRebuildAsHandle` 物化为 detached
   真 handle 树（t8r innerHTML setter 同款——createElement + 属性复制 +
   子递归 + `__zwFragNoExec` 章传递 s120），`_zwNodeParent` 反链挂到克隆
   host（parentHandle = nh）。
2. **form 状态配对传播 walk**：源侧优先 JS registry
   （`_handleChildren[handle]`，回落 `_childNodeList` 文档序配对）——
   INPUT/TEXTAREA 复制 `_inputValues`/`_inputValuesSet`；INPUT 按
   `_zwInputValueMode` 分派：default/default-on 族目标补写 value 属性
   （getter 读属性、存储即内容属性——spec 同义），value 族只写脏缓存
   （不反射属性）；复制 `_checkedDirtyState`（undefined 守卫）+
   `_indeterminate`。
3. **checked 属性对齐**（walk 内 INPUT 臂）：克隆属性集与源当前属性集
   同步（源侧 `hasAttribute` 走 latest-wins 读含 pending；克隆侧
   `__zw_has_attr_handle` 只见序列化面）——源有克隆无则补写、源无克隆有
   则移除（`_zwAttrInstanceRemoveKey` 实例层同步）。脏克隆 checkedness 由
   脏缓存裁决（getter 脏值优先），本对齐只纠 attr 面。
4. **radio 组排他**（part05 checked IDL setter INPUT 臂）：value truthy +
   type=radio 时枚举同组（document qSA 同名 radio——R3254-K5 键盘臂先例；
   form owner 比对经 `.form` 祖先链 + 选择器串键，不同 form 不互斥），
   其余元素 checkedness 置 false：脏件写 `_checkedDirtyState = false`
   （dirty 旗标保持——spec 只清 checkedness，旗标在则后续属性变更不联
   动）；净件移除 checked 属性（latest-wins 即刻可见）+
   `_zwAttrInstanceRemoveKey` + `_mo_notify`（spec 属性移除面 MO 记录）。
   FIXME: 仅 IDL setter 面触发；checked 属性增设/defaultChecked/激活行为
   等其余置真现象未走组排他。
5. **模板子树豁免物化**：含 TEMPLATE 后代的顶层子保持 plain 视图路径
   （WC-M1 切片 2a 85d3987cc 契约——模板 content 以惰性视图承载，拷贝语
   义在有环装配下无限递归；`wc_m1_factory_shadow_mutation_and_clone_
   template_content` 的 `inShadow:ok` 形态哨兵盯 content 产物无 Element
   全 API）。首版全量物化翻红该哨兵（importNode 产物带 closest）+ 把
   slot 落成模板常规子——模板 content 子是 inert 非表单控件无传播需求，
   整树豁免后哨兵复绿。

## 验证

- 定向 `FILTER=the-input-element/clone`：**19/19 Pass**（19F→0F）。
- **全量 A/B ×2**（修正前后各一轮，/tmp/s124-full.log →
  /tmp/s125-full.log、/tmp/s125-full2.log 逐行 status join）：终版
  **真翻红 = 0、Fail→Timeout 漂移 0、新增/消失行 0**；翻绿 23 =
  clone.html 19 + radio.html 4（组排他 sweep 搭车：「only one control
  of a radio button group…」「non-ASCII name attributes…」「changing
  the name…」「moving radio input element out of or into a form…」）。
  总量 6668P/1141F/103T → **6691P/1118F/103T**（干净配平 +23/−23）。
- engine lib **3570/0**（连续两轮；首版模板物化翻红 wc_m1 哨兵 1 例，
  豁免修复后复绿。中间一轮单例未复现失败未捕获名字，后续两轮全绿
  记为偶发并行摆动，续观察）；wpt-runner 全量 **256/0**（76 ignored）；
  guard 内 clippy（engine --all-features --all-targets）零告警；fmt 净；
  shim 拼接（真实装配序）node --check 过。
- 语料零改动（探针页 zw-probe-s125.html 验后删）。runner 侧无代码改动
  ——常驻语料 clone.html 19 断言 + radio.html 组断言即回归面（与修复
  同提交）。渲染面零依赖——reftest/product-smoke 未跑。

## 挂账（本切片更新）

- **input 克隆状态传播收口移出候选**（deep clone form 状态 + radio 组
  排他 IDL 面）。
- **radio 组排他其余置真现象**（checked 属性增设/defaultChecked/激活行
  为）——corpus 无断言需求，FIXME 注记保留。
- **deep clone 非 HTML ns 面**：`_r185SrcNs !== ''` 保持 plain 视图路径
  （R190 ns 修复面向视图形态）——SVG 深克隆 form 状态无语料需求。
- 承袭不变：form-submission-0 219F、写流执行身份盖章断链 FIXME、
  part02 setAttribute 绕 notify 臂、execution-timing 构成面、041
  nested write、lazy-loading 26F、meta color-scheme 21F、css-module
  corpus 门（待决）、www2 可达性、module-import-referrer TLA、
  dynamic import() with 透传、classic 外链 SRI FIXME、143 write、defer
  多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺
  索引访问、live 树 document 级逗号列表 qSA 根缺口；**待决（须用户拍
  板）**：css/text 数据模块语料资源缺失、parser form element pointer
  4F 缓行（2026-10-23）。

## 下一步候选（s126）

承袭挂账序复核：form-submission-0 186F POST 传输契约勘根（大基建评估
先行）/ execution-timing 构成面单根（056/063+051/052 增量解析）；或
全量基线翻拣下一 forms 域单根面（本轮 clone.html 同款刀法——域挂账外
的单页整簇 Fail 勘根优先级高于大基建挂账）。
