# R334 — sel 子移动族 childList record（2026-08-28）

## 问题

R332 后 `MutationObserver-childList.html` 仍 11 个 async_test 挂起：
n30-n32/n35（insertBefore 族）、n40-n42（appendChild 族）、f34/f44（fragment removal）、
n53（replace-self）、n91（insertNode children）。

探针实证根因：**sel 子**（静态页面元素，有 `__zwSelector` 无 `__zwHandle`）的
appendChild/insertBefore 全通道 miss——`recs=0, d40parent=#dummies, kids=1`（无 record、
parent 不变、视图不变）。R182 只覆盖了 insertAdjacent 家族。

## 修复（part04.js）

1. **appendChild/insertBefore sel 子分支**：wire 复用 `InsertAdjacentSelElement`
   （append = `'beforeend'` on 父 sel；insertBefore 有 ref = `'beforebegin'` on ref sel——
   host `insert_nodes_at_position` 自带 reparent 移动语义）。
2. **同步可见性**：`_zwSelPendingParent` 槽 + `_zwSiblingBaseInvalidateAll`。
3. **record 对**：removed 归旧父（wire 前兄弟 getter 快照作 prev/next——首版缺字段被 WPT
   `previousSibling didn't match expected span got null` 当场抓回）+ added 归新父；
   **同父移动两条都发**（spec move = remove+insert 两步，WPT n42 期望 2 record——
   首版 `OldSel !== sel` 门漏发，`n42recs=2` 修正）。
4. **fragment flatten 补 fragment 自身 removed record**（f34/f44 的 observer 挂在
   fragment 上，`target = fragment`，先于新父 added）。
5. **replace-with-self 补 removed+added 双 record**（n53；spec replace = 前插 + 移除，
   树序不变但 record 序列不变）。

## 验证

- `MutationObserver-childList.html`：**36P/0F/1T**（pending 11 → 1）
- 剩 1 pending：n91 `Range.insertNode children insertion`——需 sel 域 parsed-text 包装
  （`_wrapNodeEntry`）补 splitText（现只 handle 域有）+ split 尾节点 record + 双 added
  record。独立小片转下轮。
- MO 全族 129P/4F/4T（fail 集与既存备档恒等）
- Node-normalize 4P / Element-children 2P / ParentNode-children 1P / single-activation
  132P / getElementsByTagName 68P 全持平
- tab_js_worker 38P + renderer js_worker R2929/R2930 全绿
- engine v8 2473（+1 回归测试 `test_sel_child_move_mo_records_r334`）/ quickjs 1467 绿
- fmt/clippy 双矩阵干净

## 教训

1. WPT 失败消息（`previousSibling didn't match expected span got null`）直接指路缺的
   字段——先读 assert 消息再做探针。
2. pending 计数是挂起测试的精确收敛信号：`13 → 4 → 2 → 1` 每步修复可独立验证。
3. spec 的 move = remove + insert 两步各发 record，同父移动也不例外。
