# M4 切片 R45 — MutationObserver 批内 id 追链 + attributeOldValue 预捕获

**日期**: 2026-08-14
**里程碑**: M4 / DC-3（nodes MutationObserver）+ M2 S6 前置评估
**证据**: [../evidence/2026-08-14-r45-mutation-observer-semantics.json](../evidence/2026-08-14-r45-mutation-observer-semantics.json)

## 切片动机

本轮计划为 M2 S6 最小切片评估（MutationObserver 去 `__zw_*` 字符串化）。评估结论：**S6 正体依赖 L2**——native 模式下页面侧 proxy 仍是 polyfill selector/handle，把 notify target 换成 native NodeId 需 L2 建立的 per-node 身份桥。评估过程暴露两个具体 MO 语义 bug，本轮修复。

## 修复

### ① 批内 id 重命名追链（Rust `apply_dom_mutations`）

`el.id='abc'; el.className='x'` 两条 mutation 的 selector 均取自 proxy 建立时（`#old`）。第一条 SetAttr(id) 应用后文档中 `#old` 消失，第二条失配 → 旧 `?` 硬错中止整批 → `page script threw: apply mutations: set_attr: no match` 整用例崩（WPT MutationObserver-attributes 0P）。

**`rewrite_pending_id_selectors`**：id 改名成功后遍历剩余队列（apply 循环改 `while let` + `VecDeque`），把 `#旧id` 选择器重写为 `#新id`（精确匹配或 `#old > …` 后代前缀；nth-child 结构路径不受 id 改动影响）。其他 stale selector 仍走原错误路径——不掩盖真 bug。spec 语义：两 mutation 引用同一元素都应生效。

### ② attributeOldValue 写入前捕获

IDL 反射 setter（`el.id=`/`className=`/`title=`/`lang`/`type`）与 classList write 的 notify 在**写入之后**——oldValue 永远读不到（恒 null，WPT "oldValue didn't match" 全族）。修：part04 set trap 头按 IDL 名预判内容属性名（id/className→class/title/lang/type），有 observer 请求 old 时先读暂存；part05 尾部 notify 携带；classList write（part03）在 `__zw_set_attr` 前捕获。`setAttribute`/`removeAttribute` 路径 R3025 已有。

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| MutationObserver-attributes | 0P（整用例崩）| **30P/8F**（双路径）|
| dom/nodes polyfill | 2507P | **2539P（+32）** |
| dom/nodes native | — | 2509P |
| MutationObserver-childList | 10P | 10P（fragment 展开/insertNode 几何为独立簇）|

零回归：events 189P / collections 24P / traversal 9P / ranges 39P。

## S6 延后诊断

S6 正体（MO target 用 native node 对象、去 ser/deser）需 L2 live-Document 身份桥——记为 M2 前置条件。本轮修复是既有路径的纯语义正确性。

## 剩余聚类（下轮候选）

- MutationObserver-attributes 剩 8F：setAttributeNS/removeAttributeNS 的 `attributeNamespace` 字段 + same-value/no-mutation 的 records 数量语义
- childList fragment addition record 展开（addedNodes 应为 fragment 子节点）

## 验证门禁

- 单测 `test_mutation_observer_id_chain_and_oldvalue_r45`（id 改名双 mutation 入 record + oldValue 写前捕获；microtask 轮询 flush）
- engine v8 2126 / quickjs 1415 全绿；quickjs 矩阵 14 crate 全绿
- clippy 双矩阵零警告（含 apply 循环 while-let 重构的借用调整），fmt 无 diff
