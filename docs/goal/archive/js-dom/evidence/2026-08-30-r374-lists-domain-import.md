# R374 — dom/lists 域导入 + DOMTokenList 品牌化（M4/DC-3 基线扩展）

**日期**: 2026-08-30
**切片**: M4 基线扩展——`dom/lists` 子目录导入（第 7 个 dom/ 子分类）全绿 +
DOMTokenList 品牌/迭代器/反射族语义补全
**改动面**: `testharness.rs` + `fetch-dom-subset.sh`（SUBDIRS 扩展）+
`js_dom_shim/part03.js`（`_classListProxy` attr 参数化 + 品牌 + Array.prototype
方法）+ `part04.js`（DOMTokenList 反射族 gate + undefined 块）

## 1. 基线与修复

`dom/lists`（5 个 DOMTokenList 用例）基线 **36P/153F** → **189P/0F 全绿**。
失败三簇与修复：

1. **品牌面**（coverage-for-attributes ~150 + stringifier）：
   ① `Symbol.toStringTag = 'DOMTokenList'`（assert_class_string 全族——旧
   '[object Object]'）；② **DOMTokenList 反射族 gate**（part04 get trap）——
   relList（HTML ns a/area/link + SVG/MathML ns a）、sandbox（HTML iframe）、
   sizes（HTML link）、htmlFor（output）经参数化 `_classListProxy(sel, handle,
   attrName)` 服务（rel/sandbox/sizes/for 属性 backing）；gate-miss（错误元素/
   错误 ns）显式返 undefined（generic 反射回落属性串 "" 不可接受；label.htmlFor
   字符串反射例外保持）。
2. **迭代器协议**（iteration + Iterable）：spec iterable<T> 声明的方法与
   Array.prototype 同源——get trap 对 keys/values/entries/forEach/Symbol.iterator
   返 Array.prototype 对应 generic 函数（经 this=length/indexed 作用于 proxy）；
   旧自定义迭代器对象不可展开（`[...list.values()]` not-iterable）且 identity
   断言（`list[Symbol.iterator] === Array.prototype[Symbol.iterator]`）不满足。
3. **has/set trap**：`'length' in list` 等 membership 断言（旧无 has trap 恒
   false）；`list.value = '…'` 字面值写（spec value setter **不规范化**——
   "assigning value should set the literal value" 期望 " foo bar foo " 原样；
   区别于 add/remove 的 runUpdate 规范化）。

**过程回归**（当场抓回同轮修）：`_classListProxy` attr 参数化初版把默认
'class' 的缓存键也加了后缀——`_classCache` 其余写点（className setter/host
merge/R358 清桶）全用无后缀键 → 同脚本内 class 读/写分裂两个缓存条目
（Element-classlist length 全族 0，1420P→675P）。修复：默认 'class' 保持无
后缀键，仅非 class 列表加 `:attr` 后缀隔离。**NS 元素 tag 解析**：coverage
全部经 createElementNS 建——pending 未 apply 时 `_realTag` 回落 DIV，改从
`_nsHandles[handle].qualifiedName` 取 + namespace gate（`null || XHTML` 误判
bug：null ns 归一空串）。

## 2. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 目标域 | dom/lists **189P/0F 全绿**（基线 36P/153F，净 +153） |
| classList 哨兵 | Element-classlist 1420P/0F 维持（回归修复后）；MO 族 135P/3F 恒等 |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55689P（+191）/6 已知 Fail 文件恒等零新增** |
| engine 单测 | v8 2500 / quickjs 1475 全绿；integration 784P |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

## 3. 后续

- 已知 Fail 集合余 6（全部深结构/架构域定性，R373 已备档）。
- M4 基线扩展候选：dom/observable（Observable API——未实现域）。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛。
