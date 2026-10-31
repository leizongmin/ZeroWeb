# M2-S185 form.elements 集合层——HTMLFormControlsCollection/RadioNodeList 接口面 + fieldset.elements + form indexed 属性面 + owner 域文档序，11F 恰靶翻绿，shim 3 零 rust

## 入场定刀位

s184 账本「下一步候选」首位：form.elements 集合层 12
（HTMLFormControlsCollection/RadioNodeList 接口 + nameditem/matches/
filter/indexed/fieldset 关联面）。

## 勘根（六形态，靶案逐一对位）

1. **nameditem-02**（1F）：part04 form.elements ad-hoc Proxy 的 has
   trap 只有 canonical 索引臂——`"length" in c` false，
   assert_array_equals 入门探测（`"length" in actual`）即失败。
2. **matches**（1F）：`form.elements["03"]` 非 canonical 数字串名
   miss——ad-hoc Proxy get/has 无 named 字符串臂（parseInt+String
   归一后全落 `list[p]` 对象键 miss）。
3. **interfaces-01**（2F）：`"HTMLFormControlsCollection" in window`
   false；ad-hoc Proxy 无接口原型 `form.elements instanceof FCC/HC`
   恒 false。
4. **nameditem-01**（3F）：`"RadioNodeList" in window` false；named
   读无多匹配臂（返 undefined）；取集后 `removeChild` 集合不收缩
   （快照语义，live 断言面缺）。
5. **filter**（1F）：`_formControls` walk-first 使 form 子树控件恒排
   前，`form=` 跨树关联控件（before-*/after-*，host 文档序枚举面）
   错序——expected before-button1 首、got button1 首。
6. **form-indexed-element**（1F）：元素 proxy gOPD 无 FORM indexed 臂
   （`Object.getOwnPropertyDescriptor(form, 0)` 返 undefined）；deleteProperty
   无 indexed 拒绝臂（sloppy 返 true ≠ false 面）。
7. **HTMLFieldSetElement**（2F）：`fieldset.elements` getter 整体缺位
   （undefined；elements HTMLCollection constructor + controls root
   at fieldset 两案）。

## 施工（shim 3 零 rust）

- **part03**：`HTMLFormControlsCollection`/`RadioNodeList` 接口占位
  （`|| function` 惯例）+ RadioNodeList.prototype→NodeList.prototype
  直链（R198 已链 Array.prototype）。FCC.prototype 接线延后 part04
  构造点惰性（_zwHCPrototype 单例在 part05，加载序不依赖）。
- **part05 `_zwMakeHTMLCollection`**：① `liveSpec.protoCtor`——target
  直链接口原型（instanceof 面），缺省回落 `_zwHCPrototype()` 既有
  调用零变化；② `liveSpec.namedFn` 覆盖内建首匹配 namedFor（var
  重绑定，get/set/defineProperty/deleteProperty/gOPD/has 六 trap
  named 命中共用，HTMLCollection 本体不受影响）。
- **part05 新 `_zwMakeRadioNodeListLive(lsel, lhandle, lname)`**：每次
  属性读 `_formControls` 现取按 id/name 过滤（live）；target
  `{__zwNL: true}` 链 RadioNodeList.prototype（R140 Symbol.hasInstance
  认印记）；trap：get（length/item/canonical 索引，余落 target 链保
  Array 迭代器）、has（length 恒真/索引在界/链透传）、gOPD（索引在界
  → {value, writable:false, enumerable:true, configurable:true}，
  assert_array_equals hasOwnProperty 面）、ownKeys（索引表）。
  RadioNodeList.value 无语料面 FIXME。
- **part05 新 `_zwFieldsetListed(sel, handle)`**：fieldset 子树 pre-order
  walk（childNodes 读链同 _formControls walk 形）收 `_formControlTags`
  成员，无 form-owner/form-attr 条件、无嵌套 form 整枝、legend 子不
  豁免、input[type=image] 排除。spec #dom-fieldset-elements。
- **part04 form.elements**：ad-hoc Proxy 整体替换 `_zwMakeHTMLCollection
  ([], {rebuild, protoCtor, namedFn})`——rebuild=R5267 活集合形态
  （每次读现取 `_formControls`，集合对象仍 `_zwFormCollCache` 恒等）；
  FCC.prototype→_zwHCPrototype 惰性接线（首次构造接通）+ FCC
  namedItem 定义（恰 1→元素、>1→`_zwMakeCollection(hits,false)` 快照
  ——live 形无语料面 FIXME、miss→null、''→null 空串守卫同 part03）；
  namedFn：恰 1→`_zwFormNamedLookup` 元素、>1→live RNL、''→undefined
  （R38 非 supported property name）。新 **FIELDSET gate**：cached
  `_zwMakeHTMLCollection([], {rebuild: _zwFieldsetListed})`（通用 HC
  原型链，无 protoCtor）。
- **part05 `_formControls` 重排**：host 列表（`__zw_form_controls`
  文档序 owner 枚举，含 form= 跨树关联）**先推**，walk 后推补
  same-turn 新增（t8f live 面，push 键去重）；host 条目过
  `_zwPendingRemoved` 过滤（R50 同款——same-turn removeChild 的
  stale host 条目消除，nameditem-01 live 收缩依赖此 + walk 读链）。
- **part05 元素 proxy**：gOPD 加 FORM indexed 臂（expando 臂后、named
  臂前——WebIDL #LegacyPlatformObjectGetOwnProperty 序；canonical 在
  界 → {value: els[n], writable:false, enumerable:true, configurable:
  true}；R26 成本：仅 canonical 数字串进 `_formControls`，非索引键
  零新增调用）；deleteProperty 加 FORM indexed 臂（②b，expando 后
  named 前；命中恒 false——V8 sloppy 静默 no-op + strict TypeError，
  臂形与 ③ FORM named 同款；#LegacyPlatformObjectDelete 序）。

## 验证

- **定向 the-form-element：105P/3F**——11 靶案全 Pass；3F 全在册
  承袭（「Named elements should override builtins」1F fix#19c 挂
  待决 s137 起；form-action-reflection-with-base-url 2F 独立面）。
- **定向 the-fieldset-element：HTMLFieldSetElement.html 3P/1F**——
  elements HTMLCollection + controls root 两案翻红；「type attribute
  must return 'fieldset'」在册独立面承袭。
- **forms 全树 join：F2P = 11 恰靶案、P2F = 0**（4337P/363F/12T →
  **4348P/352F/12T**，12T 不变）——11 键 = filter 1 + interfaces 2 +
  matches 1 + nameditem-01 3 + nameditem-02 1 + indexed delete 1 +
  HTMLFieldSetElement 2。(key,state) 对 sort -u 去重后 join。
- **interactive-elements 483 键零位移**（439P/44F bit-exact）。
- **engine lib 3641P/1F**（1F = ua_display_li 渲染流在册同键 s184
  同报；form.elements 4 契约测试 R2829/t8f live/可迭代 image 排除/
  form-attr 三态全过——host-first 重排逐位兼容，t8f held ref 走
  walk 补充、三态走 host+walk 合并）。
- **shim node --check SYNTAX_OK**（114358 行，+203）；cargo fmt
  --all --check 过 + guarded workspace clippy 零告警。

## 复验（push 后 rebase 并入 origin 4 提交——site-opt nth-child 修复
## PR #263 两 fix + 三 docs——合并树快速复核）

- 定向 the-form-element **105P/3F 逐位一致**（3F 同上在册承袭）；
  engine lib 复跑 3/4 次 **3641P/1F**（1F 同在册键；run 1 出现 1 例
  未命名瞬态 flaky 2F，连续 3 轮不可复现，非本刀面——shim 契约测试
  4 键全过）。

## 残留与边界

- RadioNodeList.value（checked radio 值聚合）与 FCC namedItem 多匹配
  live 形：语料无面 FIXME 挂（随面补）。
- form.elements named 多匹配每读新 live RNL 实例（`form.elements.b !==
  form.elements.b` 身份不稳定）——无语料面；真浏览器跨读同实例，
  挂 [SameObject] 形语料出现时补缓存。
- fix#19c「Named elements should override builtins」1F 挂待决承袭
  （s137 起在册）。
- `_formControls` host-first 后 walk-only 新增控件排于 host 全列之后
  （真浏览器为文档序插位）——same-turn 插入场景无语料面，挂注。
- FORM indexed has 臂（`"0" in form`）未加：语料无面，get/gOPD/delete
  三面已闭合。

## 下一步候选

disabled click 8 + click 消费激活 7（同 click 分派域可合刀）；
color-attributes input.alpha/colorSpace 2F 小反射面可搭车；
submission-checks 三兄弟（连接性面）在册。
