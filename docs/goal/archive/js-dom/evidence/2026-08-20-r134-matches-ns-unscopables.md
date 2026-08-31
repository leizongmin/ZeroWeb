# R134 — M4 nodes：matches NS type selector + [Unscopable] 表 + on* 缓存失效（+9 净）

**日期**: 2026-08-20
**Driving WPT**: `dom/nodes/Element-matches-namespaced-elements.html`（6F→0F 双路径 100%）；
`dom/nodes/remove-unscopable.html`（0P→3P，剩 3F 记深结构）
**账本**: `tests/wpt-runner/imported-tests.txt`（R134 条目）

## 根因与修复（三层）

### ① matches 的 NS type selector（6F→0F）

`createElementNS('', 'element')` 等产物是 **handle-only**（sel=null）——matches 分支
`!sel` 短路恒 false。新增 `_r134MatchTypeSelector`（part03）：

- 裸 type：**不比较 ns**（浏览器 API matches 的默认 ns 语义——WPT 空 ns 与 urn:ns
  两形态都以裸 type 断言真）
- `ns|type` ns 全等 / `*|type` 任意 ns / `|type` 显式无 ns / `*` 任意元素
- ASCII 大小写不敏感（HTML 文档语义，`DIV` 匹配 div）
- 复合选择器（伪类/属性/组合器/空白）返 null → 调用方 false（保守——detached 元素
  的复合匹配属选择器引擎深结构）
- ns/localName 源：`_nsHandles`（createElementNS 登记）；miss 时 local=tag 小写、
  ns=null

### ② Element.prototype [Unscopable] 表（remove-unscopable 前置条件）

spec ChildNode 四方法 + ParentNode prepend/append 均 [Unscopable]——`with(element)`
词法域不可见。挂 `Element.prototype[Symbol.unscopables]` 表 + proxy get trap 的
`Symbol.unscopables` 分支透出（V8 with(proxy) 语义消费——has trap true 后经 get
trap 读 unscopables 表对属性排除 → 裸 `remove` 继续向外层解析到 window）。

### ③ on* setAttribute 的 handler 缓存失效 + 重编译

旧仅 body/frameset 转发路径失效缓存——普通元素首编译**永久缓存**（`_onHandlers`
+ `_listenerStore` 双表都不更新），WPT 六变体逐个 `setAttribute('onclick', ...)`
只跑首个体。修：setAttribute on* 一律剔除 listener store 旧条目 + 缓存置 undefined
+ 立即按新 attr 重编译入 store（setAttribute 即编译——派发路径零改动）。

## 剩余缺口

remove-unscopable 的 before/after/replaceWith 3F：探针实证直接 `new Function`
编译的 with(this) 双向语义全对（裸 name → window string、this.name → function），
但经静态标记 + setAttribute + dispatchEvent 的完整链路时三方法 result1 恒
undefined——inline handler 派发时序（listener 注册与编译的先后/缓存路径）深结构，
下轮可归因（记 R135 候选 c）。

## A/B 验证

- **Element-matches-namespaced**：6F→**0F（6P 双路径 100%）**。
- **remove-unscopable**：0P→**3P**（remove/prepend/append）。
- **dom/nodes 全量**：polyfill 8429→**8438P（+9）** fail 196→**187（零新增）**；
  native 7649→**7658P（+9 同步）**。
- **回归面**：events 422P/27F、collections 49P、traversal 1589P/15F 与 R133 逐项
  一致。
- **单测**：engine `test_matches_ns_and_unscopables_r134`（NS 三形态 + ns|/*|/错
  ns + * + 大小写 + 复合保守 false + unscopables 表 + proxy 透出）。

## 教训

1. **`!sel` 短路是 handle-only 元素的系统性盲区**——matches/closest/webkit 同族
   API 对 detached createElement 产物全 false；handle 元素的 JS 侧能力面要按 API
   逐个补（本切片 type selector 是最小面）。
2. **with(proxy) 的 unscopables 语义经 get trap**——has trap 返回 true 后 V8 会经
   get trap 读 `Symbol.unscopables`，表上的属性名被排除出 with 作用域；proxy 侧
   symbol 分支透出原型表即可，无需改 has trap。
3. **双表缓存（编译 fn + listener 注册）的失效要同步两处**——只清 `_onHandlers`
   不清 `_listenerStore` 则 dispatch 仍跑旧 fn；setattr 即重编译使派发路径零改动。
