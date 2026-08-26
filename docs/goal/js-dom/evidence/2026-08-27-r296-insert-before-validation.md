# R296 Evidence — Node-insertBefore 的 pre-insertion 校验族（文件级崩→32P/8F，+27P 净解锁）

**日期**: 2026-08-27
**切片**: M4——R296(b) Node-insertBefore 1F（文件级 apply 崩→subtest 级 8F + 32P）+ (a) tree-order 归因
**改动面**: `part03.js`（winning insertBefore 的 step-6 + ref 方向 + `_zwMEl` 校验 + body/generic 校验）+ `part04.js`（trap 的 WebIDL 参数 + 步骤 4 类型 + NotFound）+ `part23.rs`（+1 单测）

## 一、修复内容

### (a) pre-insertion 校验族（Node-insertBefore 文件级崩解锁）

旧版对非 child 的 refNode 直接推 host mutation → apply 时
`insert_before` 的 ref-not-child 报错**崩整文件**（probe B 形态实证）。按 spec
`concept-node-pre-insert` 步骤序补齐：
1. **WebIDL 参数序**（转换先于算法）：parameter 1 非节点 / parameter 2 缺省
   或非节点非 null → TypeError（trap + 泛型双入口）；
2. 步骤 1-2 HRE（`_r117GenValParentAncestor` 前置——顺序断言族）；
3. 步骤 3 NotFound（trap 版——refNode.parentNode identity）；
4. 步骤 4 类型 HRE（Document/DocumentType 只能入 Document——trap + `_zwMEl`）；
5. 步骤 6 doc-parent HRE（winning fn）：元素/doctype 冲突 + fragment
   2+元素/含text + ref 位方向。

### (b) 三处首版错误当场抓回（教训）

1. **NotFound 严格化回归**（Range-insertNode/surround 1840F×2 + 6 单测）：
   R117 注释早已警告「内部加载路径经 insertBefore 挂 pending ref 视图不完整
   会误抛」——四处 NotFound 全部回退（lenient 保留）。
2. **doctype ref 方向反了**（NodeIterator-removal 2F）：doctype 插到元素
   ref **之前是合法的**（spec「doctype 须在首元素前」——正是合法落位）；
   非法仅当 ref 在元素之后。`<=` 改 `>`。
3. **doctype 尾部冲突过宽**（同上）：`_r296wDt||_r296wEl` 全形态抛——
   收窄到**尾部追加**（refNode null）形态。
4. **同节点重插豁免**：NodeIterator-removal 恢复段 remove 后同 turn 重插
   （host 树仍含本节点）——identity 豁免（同名宽松键过宽曾误吞 dtDup 断言，
   回收到 identity-only）。

### (c) tree-order 4F 归因（未解，深结构记档）

probe 实证 iframe 子文档的 **traverse 不对称**：`traverse(doc)` 经
firstChild/nextSibling 只见 1 元素（工厂 docEl.childNodes 恒空——R220 评估
deferred），而 querySelectorAll("*") 返 313（detHtml 序列化 + filter_synthetic
剔 html）。两侧树视图结构性分歧——head/body 链入 docEl.childNodes 是
R220 记档的深结构（须 host Range.insertNode docEl 分支成对改）。

## 二、验证

| 套件 | R295 | R296 | Δ |
|---|---|---|---|
| Node-insertBefore | 1F（文件级崩） | **32P/8F** | +32P（subtest 级暴露） |
| NodeIterator-removal | 29P/0F | 29P/0F | 持平（回归当场修复） |
| Range-insertNode / surroundContents | 1841P/1840P 0F | 同 | 持平（回归当场修复） |
| Node-removeChild/replaceChild/prepend/appendChild/MO-childList/insertAdjacent 族 | 全基线 | 同 | 持平 |
| engine 单测 | 2433 | **2434** | +1（r296 校验序单测） |

## 三、R297 靶点

- **(a) Node-insertBefore 剩余 8F**：NotFound lenient 化后暴露的顺序断言族
  （"check child before node type" ×3）+ doc-parent 元素簇（内部流程与
  spec 冲突面——需要 L2 live 视图后收口，R117 原注）。
- **(b) tree-order 4F**（head/body 链入 docEl 的 R220 深结构——host 侧成对）。
- **(c) MO-document parser 3F**（parse-time record 基建）。
- **(d) selector 小簇**（mixed-case 1F + escapes 2F + scope 2F）。
