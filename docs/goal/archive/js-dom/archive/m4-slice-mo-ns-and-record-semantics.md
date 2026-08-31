# M4 切片 R46 — MutationObserver NS record + no-mutation 语义 + classList record 对齐

**日期**: 2026-08-15
**里程碑**: M4 / DC-3（nodes MutationObserver）
**证据**: [../evidence/2026-08-15-r46-mo-ns-and-record-semantics.json](../evidence/2026-08-15-r46-mo-ns-and-record-semantics.json)

## 切片内容（R45 后剩 8F 的三簇全修）

### ① NS 属性 record（4F）

setAttributeNS/removeAttributeNS 原**委托** setAttribute/removeAttribute——record 带限定名（`xml:lang`）且 `attributeNamespace` null。spec `mutation-observer-attributes`：record.attributeName = **localName**（prefixed 拆解）+ attributeNamespace = ns。

修：NS 族自带 notify（attributeName=local + attributeNamespace=ns + 写前 old 捕获），直写 host 回调绕过 delegate 的无 namespace notify（初版双发 got 2，同轮修）。

### ② no-mutation guard（3F）

`removeAttribute`（及 NS 版）对**缺失属性**不再发 record——spec 仅在「已存在属性被移除」时 queue record（WPT "removal no mutation"：n71 无 class，`removeAttribute('class')` 后仅 id 改名 1 条）。presence 判定与 hasAttribute 同源（handle/sel latest-wins）。

### ③ classList record 语义（2F + classlist 回归修复）

- spec DOMTokenList update 步骤 8：新 token 集序列化与原值**相同仍 set attribute + 发 record**（real browser 对 `classList.add` 已存在 token 仍发 attributes record——"same value mutation" 期望 2 条）。R16 的「值相同 return」吞掉了它——移除该 early-return
- **例外**：remove 到空集且原属性**缺失**不写不 notify（remove 不得创建空 class 属性；WPT checkRemove(null,...) 期望 attribute 保持 null）。无此例外时 Element-classlist 回归 -10（同轮发现 + `_readClassRaw` absence 判定修复，1420P/0F 恢复）

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| MutationObserver-attributes | 30P/8F | **38P/0F（100%）双路径** |
| Element-classlist | 1420P/0F | **1420P/0F 维持**（中途 -10 同轮修）|
| dom/nodes polyfill | 2539P | **2547P（+8）** |
| dom/nodes native | 2509P | 2517P |

零回归：events 189P / collections 24P / traversal 9P / ranges 39P / childList 10P。

## 剩余聚类

childList fragment addition record 展开 + insertNode/surroundContents 几何 record（独立簇）。

## 验证门禁

- 单测 `test_mutation_observer_ns_and_no_mutation_r46`（NS record 三断言组 + no-record 两场景 + same-value add record）
- engine v8 2127 / quickjs 1415 全绿；quickjs 矩阵 14 crate 全绿
- clippy 双矩阵零警告，fmt 无 diff
