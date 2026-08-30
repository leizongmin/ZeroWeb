# R376 — dom 根目录散用例导入 + Element 反射族 gate（M4/DC-3 基线扩展）

**日期**: 2026-08-30
**切片**: M4 基线扩展——dom/ 根目录 9 个散用例导入（第 8 个扫描条目）+
historical/interface-objects/attributes-are-nodes 三域修复
**改动面**: `testharness.rs` + `fetch-dom-subset.sh`（SUBDIRS 加 "dom"）+
`js_dom_shim/part04.js`（text/comment 反射 gate + Attr 插入 gate）+
`part06.js`（window 级 attachEvent/detachEvent 移除）+ `part07.js`（WebIDL
接口全局属性形态归一）

## 1. 导入与修复

**dom 根目录 9 用例**（historical / interface-objects / window-extends-event-
target / attributes-are-nodes / xpath-result-single-node-value-nullable /
eventPathRemoved / svg-insert-crash / historical-mutation-events[imported 未
注册?] / slot-recalc[2 ref 用例不跑]）——collect 为单层扫描，"dom" 条目只扫
根目录 .html（子目录各自条目覆盖）。

1. **text/comment 反射 gate**（part04 get trap）：attributes/hasAttributes/
   isSupported 对 text/comment handle 返 undefined（spec 这些 IDL 成员在
   Element；WPT historical "Node member must be removed"——旧 proxy get trap
   对任意 handle 服务元素反射面）。**PI 豁免**（R123 PI attribute layer 的
   hasAttributes 是已锁定 shim 面）。
2. **Attr 插入 gate**（appendChild/insertBefore/replaceChild）：Attr 子 →
   HierarchyRequestError（spec `concept-node-tree`；WPT
   attributes-are-nodes 全族——旧静默穿透/误挂）。
3. **window 级 attachEvent/detachEvent 移除**（part06）：IE 专有遗留已从
   Window 删除（WPT historical "Window member must be removed"）；元素级
   proxy 的 attachEvent（part04）保留——legacy 元素调用面不受影响。
4. **WebIDL 接口全局属性形态归一**（part07）：22 个接口全局
   （Event…DOMTokenList）统一 enumerable:false + configurable:true（for-in
   不枚举 + delete 可删——WPT interface-objects 两族断言）+ NodeIterator/
   TreeWalker/DOMTokenList 缺失接口占位构造器补位（工厂产物不经构造器，接口
   对象本身须在 window）。

## 2. 域定性备档

- **historical 3F（namespaceURI/prefix/localName）不追**——WPT 该文件断言这
  三成员已从 Node 移除，但现行 spec 已回归 Node（Chromium: text.namespaceURI
  === null），shim 的 null 语义正确且被 in-repo 单测锁定（part07
  test_element_local_name_r11 Text.localName === null）——stale 期望。
- **window-extends-event-target 2F（EventTarget 继承域）**——window 的
  addEventListener 须继承 EventTarget.prototype 且 this 语义正确；shim 的
  window 事件路由是定制实现（_globalAddEventListener），原型继承改造 = 深结构
  转档。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 目标域 | historical **90P/0F→87P/3F（3F = stale 期望备档不追）**；interface-objects 23P/0F；attributes-are-nodes 4P/0F；xpath/eventPathRemoved/svg-insert 全 Pass |
| 哨兵 | Node-appendChild 11P / Node-insertBefore 40P / Node-replaceChild 58P 恒等（Attr gate 零回归）；MO 族 135P/3F 恒等 |
| 全量 dom sweep（polyfill，TIME_LIMIT=2400） | **55804P（+115）/真实 Fail 文件集 = 6 已知 + historical 3F[stale 备档] + window-extends 2F[深结构转档] 零意外新增** |
| engine 单测 | v8 2500 / quickjs 1475 全绿；integration 784P |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

## 4. 后续

- 已知 Fail 集合计 6 原有 + historical 3F[stale 不追] + window-extends 2F[转
  档]，全域定性维持。
- M4 基线扩展候选：dom 子分类已全量覆盖（nodes/events/collections/traversal/
  ranges/abort/lists/根散用例）；observable 全 tentative 跳过。
- 主线剩余：M5/M7 default-on（待用户点名，改 Mission 级单向门）；M3 已达成；
  M4 基线持续维护；M2 已收口；M8/DC-8 已收敛。
