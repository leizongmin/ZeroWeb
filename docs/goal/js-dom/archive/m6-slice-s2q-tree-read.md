# M6 S2q 续 — QuickJS 树读回 getter（R61）

**日期**: 2026-08-16
**commit**: `784aab22`
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal DC-7）第六切片
**证据**: [evidence/2026-08-16-r61-quickjs-s2q-tree-read.json](../evidence/2026-08-16-r61-quickjs-s2q-tree-read.json)

## 目标

R60b 的 mutation 族（create/append/remove）之后补**树读回**：childNodes/
parentNode/firstChild/lastChild——树构建与读回形成完整闭环。

## 实现

- `childNodes`：**Array 返回形态**（`rquickjs::Array::new` + `set(idx, v)`）。
  live NodeList 的 indexed props/own 枚举语义延 S1q 复合对象域（同 V8 侧分期）；
  快照数组与 V8 tests 断言面一致。子对象经 `get_or_build_node_value` 共享身份
  缓存包装——`childNodes[0] === appendChild(el)` 成立（spec identity）。
- `parentNode`：detached/根 → JS null（Value 返回形态）。
- `firstChild`/`lastChild`：共用 `first_last_child_getter` helper。

## PoC 闭环断言

append → childNodes.length/`[0]` 身份；child.parentNode 指回父（**双向一致**）；
firstChild/lastChild；remove → firstChild null + detached parentNode null。

## 验证

engine quickjs **1419** / v8 **2153** 零回归；webview quickjs wiring 绿；
clippy quickjs 矩阵零警告；fmt 无 diff。

## M6 累计元素面

2 全局工厂（element_for_id/create_element）+ 13 属性（6 setter）+ 6 方法。
树构建（append/remove）+ 树读回（childNodes/parentNode/first/last）闭环完成。

## 下一步

S3q 查询族（querySelector/getElementById 原生，消费 zero_dom 选择器）——元素
可达性大增；或 S4q EventTarget + DOMException 基建。
