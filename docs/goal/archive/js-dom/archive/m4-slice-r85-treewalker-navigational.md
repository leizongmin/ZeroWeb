# M4 切片 R85 — TreeWalker 导航式重写 + previousNode 规范镜像（WPT 驱动）

**日期**: 2026-08-17
**基线**: R84 后（traversal 1556P/24F）
**Evidence**: [../evidence/2026-08-17-r85-treewalker-navigational.json](../evidence/2026-08-17-r85-treewalker-navigational.json)

## 驱动用例簇

- `TreeWalker-basic.html`（"Walk over nodes" 全序列断言——9 步导航链）
- `TreeWalker-traversal-skip.html` / `traversal-reject.html` / `traversal-skip-most.html`（previousNode/nextSibling 的 SKIP/REJECT/0 三态语义）
- `TreeWalker.html` 的 `createTreeWalker(document, 0xFFFFFFFF, null)` previousSibling 簇

## 四重根因

1. **accepted/parentAcceptedIdx 扫描模型无法表达方法分叉的 0 语义**：WPT oracle 对 filter 返 false(→0)：children 循环只横移不 dig；sibling 循环经 firstChild dig（类 SKIP）；父站 truthy 即止。R84 的 check() 统一归一是过度概化。
2. **previousNode 的 order-scan 逆向不剪 REJECT 子树**：逆向候选序中子树内节点先于被拒祖先命中（traversal-reject B3→B2 期望 A1，C1 错返）。
3. **html.previousSibling 恒 null**（host 对 html 无父）：WPT oracle 的 expected 计算与 walker 实现走同一导航链——html.prev=null 使 expected null、walker 返 doctype → 分歧。
4. **父站判定**：`if (filterNode(node))` truthy 止过宽——DOM spec 仅 FILTER_ACCEPT 止（skip-most 实证：SKIP 父不止单步）。

## 修复

- **navKids/navSiblings 导航式层级方法**：四方法 + parentNode 全部改经真实 firstChild/lastChild/nextSibling/previousSibling/parentNode getter 步进（R84 兄弟链修复后导航可靠），精确镜像 WPT oracle 循环。check() 返回原始值，消费方按方法解释。
- **previousNode 拆两态**：TreeWalker 用 DOM 规范算法（sibling=previousSibling；REJECT→sibling=prev；SKIP/0→sibling=lastChild||prev；兄弟尽→parent 续）；NodeIterator 保持结构序逆向（迭代集合结构性不剪）。
- **nextNode 显式分叉**：TreeWalker REJECT(2)+0 剪枝 / NodeIterator 仅 2 剪（保持 R84 通过集）。
- **filterNodeTruthy → 仅 ACCEPT 止**。
- **html 兄弟走 document.childNodes 派生**（previousSibling=doctype、nextSibling=null）。

## 过程否决记录

- 初版统一归一（0→2）→ NodeIterator -12P 回归（迭代器不剪）后按接口分叉（R84）。
- 本轮再发现方法级分叉：单步前驱（hasChildNodes→lastChild）→ basic prevN 错返 i——前驱绝不能是自身子节点（树序子在后）；最终落到 DOM 规范 previousNode 算法镜像（sibling 循环 + parent 续）。
- structPrev 最深右端下钻跳过中间祖先 check → traversal-reject 仍错返 C1——规范算法要求逐步 check。

## 验证

| 项 | 结果 |
|----|------|
| dom/traversal | 1556P/24F → **1565P/15F（+9 净）**（basic/skip/reject/skip-most 全修，TreeWalker.html 8F→1F） |
| dom/nodes | 6643P → **6650P（+7 净）** |
| dom/events / collections | 189P / 48P-0F 零回归 |
| 单测 | part18 +3（导航序列全链 / REJECT 剪 prevNode + skip-most / html-doctype 兄弟）；engine v8 **2179** / quickjs **1427** 全绿（R82/R83/R84 旧 walker 测试零回归） |
| fmt / clippy | 无 diff / workspace 默认 + quickjs 矩阵零警告 |

## 剩余（traversal 15F）

- cross-realm/realm 7F（跨 realm 深结构）
- currentNode 2F + walking-outside-a-tree 1F（root 外重定位/嫁接 lazy-resume，M1 L2 域）
- NodeIterator removal 族 3F（live 集合移除跟踪）
- TreeWalker.html Recursive-filters 1F + previousNodeLastChildReject 1F

## 教训

1. **WPT oracle 的 expected 在我们引擎里同源计算**——实现与 oracle 走同一批导航 getter 时，修 getter（html.previousSibling）= 同时修 expected 与 actual 两端；只修 walker 一侧会制造假分歧。
2. **filter 返回 0 的语义随遍历方法分叉**（nextNode 剪 / sibling dig / children 横移 / 父站仅 ACCEPT 止）——统一归一必错，须按 spec 各算法分别镜像。
3. **树序前驱绝不是自身子节点**（子在后）——previousNode 的正确形态是 DOM 规范的 sibling 循环 + parent 续，而非「下钻最深」的镜像直觉。
