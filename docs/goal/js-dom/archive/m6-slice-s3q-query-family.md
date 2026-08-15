# M6 S3q — QuickJS 查询族（R62）

**日期**: 2026-08-16
**commit**: `e438af32`
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal DC-7）第七切片
**证据**: [evidence/2026-08-16-r62-quickjs-s3q-query-family.json](../evidence/2026-08-16-r62-quickjs-s3q-query-family.json)

## 目标

S3q 查询族：`querySelector`/`querySelectorAll` 原生（全局工厂 + 元素级方法），
消费 zero_dom 全量选择器引擎——QuickJS native 元素可达性大增（此前只能经
`element_for_id` 定点访问）。

## 实现

- `__zw_native_query_selector(sel)` / `__zw_native_query_selector_all(sel)` 全局
  工厂（与 V8 同名同 wire——A/B 对照门双引擎复用同一脚本）。
- 元素级 `querySelector`/`querySelectorAll` 方法（子树作用域）。
- 查询结果经 `get_or_build_node_value` 共享身份缓存包装——**跨工厂 identity**：
  `qs('#main') === element_for_id('main')`。
- miss/空/非法选择器 → null / 空 Array（parse 失败返 None 无 panic，V8 同语义）。

## PoC 断言

id/class/复合选择器（`div#main`）；miss 语义；Array 文档序；跨工厂身份一致；
class 选择器跟随 live 状态（className setter 改值后 `.c` 不再命中）；元素级
子树作用域查询。

## 验证

engine quickjs **1419** / v8 **2153** 零回归；clippy quickjs 矩阵零警告
（needless_question_mark 修复一处）；fmt 无 diff。

## 本 session（R58→R62）累计

M6 从 S0q 骨架推进到可用元素面：
- **4 全局工厂**：element_for_id / create_element / query_selector /
  query_selector_all（全部与 V8 同名同 wire）
- **13 属性**（6 setter：id/className/title/lang/accessKey/textContent；7
  getter-only：nodeType/tagName/nodeName/namespaceURI/localName/childNodes/
  parentNode/firstChild/lastChild）
- **8 方法**：get/set/remove/hasAttribute + appendChild/removeChild +
  querySelector/querySelectorAll

## M6 剩余

S4q EventTarget（addEventListener/dispatchEvent + DOMException 基建——补
appendChild DomError→DOMException 对齐）→ S5q customElements 五件套 + lifecycle
→ S0q 续 weak/finalizer → S1q 复合对象（attributes/classList 二级身份缓存）。
