---
date: 2026-10-07
modules: engine,js-dom-shim
---

# DOM shim 每次访问打宿主/重扫树的模式在批量循环下退化为 O(n²)

## 问题描述

bilibili 首页稳态轮询回调（~5s 周期）单次 700-860ms（Chrome 上同回调几乎静止）；真实站点 append 探针单次 1.3-2.6ms/节点、removeChild 20.5ms/节点。页面脚本大量使用 `createElement`+`appendChild` 循环建树与批量属性访问，任何「每属性访问都触发宿主查询或 live 视图重建」的 shim 实现在此类循环下都是 O(n²)。

三处实例（同模式不同载体）：

1. **tag 解析打宿主线性扫**：get trap 每属性读执行 `_realTag(sel, handle)==='CANVAS'` gate，handle 路径调宿主 `__zw_get_tag_handle` → `query_tag_from_mutations` 全队列逆序线性扫。mutation 队列随结构操作只增 ⇒ 每属性读 O(队列长)，append 循环 O(n²)。沙箱实测 200 append 触发 444,221 次宿主查询。
2. **Range 边界调整循环内重读 live childNodes**：`_zwAdjustRangesForInsert/Remove`（part03.js）newIndex 扫描循环内每迭代读 `parent.childNodes`（2 次/迭代）；R140 live NL 每次属性读都 refresh（registry slice+concat，O(n)）⇒ 每次 insert/remove O(n²)。
3. **（未修，疑似）live collection 逐索引访问**：`getElementsByTagName` 等若每次索引访问重新查询，迭代即 O(n²)——hotfn 探针包装失败对该面不可见，e() 大块疑案候选。

同族既有判例：R-baidu3 TAG_MEMO（sel 侧 tag 查询 O(1) 化）——本例 handle 侧漏改，不对称导致回归面残留。

## 根因分析

共性根因：**shim 侧对象的访问成本被实现成了「队列/树规模的函数」**，而页面代码假设 DOM 访问是 O(1)（Chrome 语义保证）。三个放大器：

- get trap 是**所有**属性读的必经点——一个看似局部的 gate（如 CANVAS 判定）会把成本乘到每次访问上；
- mutation 队列只在 drain 时清空，稳态轮询页面队列持续增长；
- live 视图（live NodeList/HTMLCollection/Range 表）的 refresh 若挂在每次属性读上，读循环内每读都付 O(n)。

诊断配方（三件套，blanket 包装会因 `performance.now`/`_perfNow` 别名链递归爆栈，不可用）：

1. **targeted 宿主回调计数**：只重写具体宿主回调名（如 `globalThis.__zw_get_tag_handle`），计数+计时；
2. **`new Error().stack` 抓点**：在计数命中时采栈，定位 shim 内调用链（配合拼接 shim 的 part 行号映射）；
3. **消融实验**：将嫌疑函数置空（`__zwAdjustRangesForInsert=null`）看循环耗时是否变为恒定，锁定剩余线性项归属。

宿主 census 分桶（`ZW_JS_WORKER_CENSUS`）可远程定性：`ResolveAsyncCallback[<1KB]` 大时长 = 沙箱内纯 JS 慢；`Execute` 大时长 = 宿主回调慢。本轮 e() 疑案即靠此排除宿主面。

## 解决方案

原则：**访问成本必须与队列/树规模解耦**——要么 memo（队列纯函数量可增量维护），要么外提（循环不变量移出循环），且必须保语义等价。

1. **`HANDLE_TAG_MEMO` 增量备忘**（callbacks.rs）：键 = (mutations epoch Arc, drain 代际)。tag 是队列纯函数 ⇒ 同键下队列只追加 ⇒ 水位 `scanned` 增量扫 `[scanned, count)`；CreateElement/NS 正序 insert 覆写 = 逆序首中的 latest-wins；drain（MUT_DRAIN_GEN 递增+清队列）或换 epoch 全量重建；R342 竞窗水位取 min 防越界；miss 落旧回落链语义不变。效果：真站 append 1334→178µs（7.5×）。
2. **R263/R262 承载数组外提**（part03.js）：边界调整扫描循环内树不变、首读已含本次变更 ⇒ `childNodes` 提到循环外读一次。效果：消融验证 append 分块由线性增长变恒定。
3. 变异自查钉死备忘语义：drain 漏判 → 钉红；insert-if-absent（latest-wins 破坏）→ 钉红（`js_dom_bridge_tests/part40.rs` + callbacks.rs 单元钉常驻）。

**审查清单**（新增 shim 访问路径/修性能时过一遍）：该访问的成本是否依赖队列长度/子树规模/兄弟数？循环体内是否重读 live 视图？sel 侧与 handle 侧是否对称修过（一侧 O(1) 化后另一侧必查）？
