# M4 切片 R43 — HTMLCollection legacy 属性语义 + L2 live 视图诊断（未 land 部分）

**日期**: 2026-08-14
**里程碑**: M4 / DC-3（collections）+ M1 L2 前置诊断
**证据**: [../evidence/2026-08-14-r43-htmlcollection-legacy-props.json](../evidence/2026-08-14-r43-htmlcollection-legacy-props.json) + [../evidence/2026-08-14-r43-l2-live-view-diagnosis.md](../evidence/2026-08-14-r43-l2-live-view-diagnosis.md)

## Part 1（未 land）— M1 L2 live 视图最小切片：实现、实测、回退

按入口文档「L2-first 最小只读子集」建议实现 `with_query_doc_live`（快照 + pending mutations 合并视图，升级 `__zw_query_match/_all/parent/child_nodes` 四回调）。单测验证同块 appendChild→查询可见性成功，但 WPT 实测：

- dom/traversal 9P 持平（detached 树是 **handle-based**，走 JS 侧 `_handleChildNodes` registry，不经 host 查询回调——与 sel-based 查询正交）
- dom/ranges 39P 持平（剩余失败在 iframe mega-case/ShadowRoot）
- **dom/nodes 2503→2493（-10）**：case.html 在 iframe document 建元素、期望主文档 `getElementsByTagName` 返 `[]`——旧快照语义偶然正确，live 视图把跨文档 pending mutations 并进主视图

**决策：净负不 land，全部回退**。完整诊断（含 M1 设计输入：mutation 须带 document 作用域 / handle 树与 sel 查询须由 live Document 统一 / apply_dom_mutations handle 作用域强制全量重放）归档 `2026-08-14-r43-l2-live-view-diagnosis.md`。

## Part 2（已 land）— HTMLCollection legacy platform object 属性语义

WPT `HTMLCollection-delete.html`（0P/4F）+ `getElementsByClassName-32` 驱动：

1. **indexed 属性不可配置**：`_zwMakeCollection` 对 0..n-1 各设 `configurable:false` accessor（getter 读**包装前快照**的元素——`a===arr` 时读 `a[idx]` 会自递归；guard 跳过二次包装数组）。`delete c[0]` loose no-op（普通数组 delete 挖洞致永久 undefined——"before" 断言也炸的根因）/ strict 抛 TypeError
2. **named getter**：HTMLCollection 元素 id/name 暴露为 `configurable:false` accessor（`c.foo` 命中 `<i id=foo>`，文档序首个命中），delete 同语义
3. **纯数字 id 不经 named 暴露**：数组上 defineProperty 数字键 accessor 会把 length 推到 index+1（JS 语义），且 spec named/indexed 不混淆——跳过数字键

### 过程记录

- 初版 getter 读 `arr[idx]` 无限递归（RangeError）→ 元素快照
- 二次包装 redefine 抛 TypeError → guard
- **数字 id length 回归**（nodes 2503→2502，getElementsByClassName-32 "numeric IDs" 反转）：经 clean/after 全量 per-subtest diff 定位（4183 行输出 diff 出 3 处变化），根因 = 数组 defineProperty 数字键 length 增长 → 跳过数字键修复

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| HTMLCollection-delete | 0P/4F | **4P/0F（100%）** |
| dom/collections polyfill | 17P/31F（35.4%）| **21P/27F = 43.75%** |
| dom/collections native | 17P | **21P（对等差 0pp）** |

零回归：nodes 2503P / events 189P / traversal 9P / ranges 39P。

## 验证门禁

- 单测 `test_htmlcollection_indexed_named_props_r43`（7 断言组）
- engine v8 2124 / quickjs 1415 全绿；quickjs 矩阵 14 crate 全绿
- clippy 双矩阵零警告，fmt 无 diff
