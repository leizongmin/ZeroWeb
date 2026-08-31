# R321 Evidence — replaceWith/insertAdjacent 的 DocumentFragment 参数展开（配 R119 头插保序 + pending 记账 + JS 反链；视图可见性归 L2 深水区确证）

**日期**: 2026-08-28
**切片**: M4——R321(a) remove-next-sibling-during-replace-with 残余归因
**改动面**: `part05.js`（`_insertAdjacentVariadic` 的 fragment 展开分支 + pending 桶记账 + JS 反链）+ `part24.rs`（r321 事实域测试）

## 一、修复三层（探针逐层推进）

WPT remove-next-sibling-during-replace-with：`target.replaceWith(content.cloneNode(true))`
后 `container.querySelector('script')` 应命中（got null）。探针链定位：

1. **fragment 参数未展开**：`_insertAdjacentVariadic` 对 nodeType 11 参数走 `__zw_insert_adjacent_element(sel, position, fragmentHandle)` 整体挂载（host 端 fragment 语义未消费）——子不落地（`tags=DIV.B` 实证）。修：展开子按序逐个 INS + text 子走 adjacent_text。
2. **afterbegin 头插反序**：展开后逐子 INS 对头插位（prepend 经 afterbegin + reverseOrder）天然反序——`test_fragment_flatten_all_insertion_paths_e2e` 当场抓回（期望 `a<b<X` 得 `<b><a>X`）。修：afterbegin 时逆序遍历子（R119 prepend「逆序 unshift = 参数序」同理）。
3. **JS 侧记账缺失**：host wire（insert_adjacent_element）不写 JS 注册表——展开子无反链/pending 桶记录。修：每个子记 `_zwNodeParent[c] = {parentSel: sel}` + 批后 `_zwHCLiveInvalidate(ceInserted, [], sel, null)`（appendChild R51/R47 同款）。

## 二、已知限制（L2 深水区确证）

展开子对 **host 快照查询/视图**（querySelector(All) 的 host 侧、sel 父的 childNodes 融合视图）仍不可见（`kids=DIV.B`）——appendChild 对照（`apKids=3` 视图认、`apTags=2` 查询不认）证实这是**同一缺口的两面**：host 快照与 JS registry 的普通 append 归一。与 querySelector-mixed-case（R299 备档）同域，属 L2 identity 双源统一（深结构）。测试以当前事实域锁定（`kids=DIV.B` + `ck=3`），L2 落地后翻转。

## 三、A/B

| 项 | R320 | R321 | Δ |
|---|---|---|---|
| 全量 dom sweep | 54140P/58F/22T | 54140P/58F/22T | **Fail set 恒等零回归** |
| test_fragment_flatten_all_insertion_paths_e2e | 绿 | 绿（反序回归当场抓回后修复）| — |
| engine --lib（v8/quickjs）| 2459/1460 | **2460**/1460 | +1（r321 事实域测试）|
| fmt / clippy（v8 guarded + quickjs）| — | 干净/0 | — |

## 四、教训

1. **展开式修复的顺序语义**：把「整体插入」改「逐子插入」时，头插位的子序必须反向适配——
   既有 e2e（fragment flatten 全插入路径）是这类回归的正确守卫。
2. **跨层记账的完整性**：host wire 不回写 JS 注册表，每个新插入路径都要补全三件套
   （反链 + pending 桶 + live 失效）——appendChild 有、insertAdjacent 漏，正是「同语义多
   实现点」教训（R300）的另一实例。
