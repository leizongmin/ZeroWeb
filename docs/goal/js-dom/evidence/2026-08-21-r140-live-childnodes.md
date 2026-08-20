# R140 — live childNodes（Node-childNodes 簇 5F→0F，双路径 100%）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/nodes/Node-childNodes{,-cache,-cache-2}.html`（8 subtest）

## 根因

spec [dom-node-childnodes](https://dom.spec.whatwg.org/#dom-node-childnodes) 返回 **live NodeList**，六断言族：

1. **同一性（caching）**：同节点重复读 `childNodes` 返回**同一对象**（Node-childNodes-cache / cache-2）。
2. **活性（live）**：append/insert/remove 后**旧引用**的 length/索引反映新状态。
3. `item(i)` 方法。
4. 迭代器 identity：`list[Symbol.iterator] === Array.prototype[Symbol.iterator]`。
5. `instanceof NodeList`。
6. Document/DocumentFragment 形态同语义。

旧实现每次读都新建数组（proxy get trap 直接返回 `_childNodeList(...)` 快照/融合视图），1/2/3/5 全挂。

## 修复（part03/04/05）

- **真数组承载**：live 数组是真 Array（原生 keys/values/entries/forEach/Symbol.iterator 天然满足断言 4——不换原型）。
- **identity 缓存** `_zwLiveNLCache`（elKey → 数组）：同节点重复读返同一对象（断言 1）。
- **`_r140Refresh(arr)`**：读时重查融合视图（容器 handle → registry；sel 路径 → `_childNodeList(sel, null)` + pending overlay；handle 元素 → `_zwLocalChildNodes` + registry + detached 物化缓存融合），splice 级同步 length+索引（保持非索引属性）。
- **mutation 入口同步** `_zwLiveNLSync`（elKey → refresh 回调）：`_recordHandleChild` / `_unrecordHandleChild` / appendChild / insertBefore / removeChild / replaceChild / remove / replaceWith / prepend / replaceChildren 记账后调用。
- **`instanceof NodeList`**：NodeList 构造器挂 `Symbol.hasInstance` 认 `__zwLiveNL` 标记数组（part03；原型链保持 Array）。
- **`item(i)`**：就地挂在承载数组 + detached doc 的 childNodes/children。
- **换代失效**：`_zwChildBaseInvalidateAll`（register_dom_callbacks 重注册时）同批清空 live 缓存与 sync 表。

### R140 当轮修正两处（上轮 429 中断 WIP 收口时发现）

1. **命中缓存分支永不刷新**：原写法在缓存命中分支直引 `_r140Refresh(...)`——该 `var` 在同一函数作用域内**声明于分支之后**，hoisting 后此处是 `undefined`，调用抛 TypeError 被 `catch` 吞掉 → 命中路径返回 stale。此前 sel 路径靠 `_zwLiveNLSync` 兜底掩盖；`prepend`（`_prependHandleVariadic` 直接 unshift registry，无 sync 钩子）暴露——R119 单测（`prepend(null)` 期望 `[null 文本, text 文本]` 双子）回归红灯定位。修：refresh 闭包挂承载数组自身 `__zwRefresh`，命中时经属性调用。
2. **replaceWith 绕过入口**：R117 的 handle 路径直接 splice `_handleChildren`（不经 `_recordHandleChild`/`_unrecordHandleChild`），须手动双路同步（`_zwLiveNLSync` + `__zwRefresh`——sync 表可能被换代清空，双路最稳）。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| Node-childNodes{,-cache,-cache-2} | **8 subtest 双路径全 Pass（0F）** |
| dom/nodes 全量 | 4799P/874F，fail 集与 R139 基线**逐条一致**（characterSet-normalization 654F 既存 + realm/adopt 簇既存）零回归 |
| dom/events | 413P/32F（含 Timeout shadow-relatedTarget），fail 集与基线**完全一致** |
| dom/traversal | 50P/6F fail 集一致 |
| dom/collections | 49P/0F 全绿 |
| `make test` | 66 套件全绿（双矩阵） |
| fmt / clippy | 零 diff / 零警告 |

注：本轮 nodes 全量数字（4799P/874F）与 master.md 账面（8464P/188F）口径差异来自本轮 log 含 characterSet-normalization 两文件 654F（此前轮次未计入的 wpt-data 滚动新增文件）——非本轮回归，fail 集 diff 为证。

## 单元测试

`test_live_childnodes_r140`（part20.rs，10 断言段）：handle 元素 caching / append live + item / fragment 容器 / sel 元素 caching+append+remove live / instanceof NodeList / 迭代器 identity / detached doc item / prepend 文本双子（R119 回归）/ replaceWith 原位替换 live（R117 回归）。
