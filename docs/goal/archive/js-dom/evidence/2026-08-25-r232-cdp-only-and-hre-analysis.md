# R232 Evidence — cDP-only 开关拆解实验 + HRE 簇 sim 考古（负结果轮）

**日期**: 2026-08-25
**切片**: M4——R232(a) surround 剩 455F（1385P 基线）
**改动面**: 无 land 代码（两个实验均回退，工作树零 diff）

## 一、455F 当前聚类

| 簇 | 计数 |
|---|---|
| assert_unreached | 133 |
| cDP 缺方法面（node2 91 + nodeB 17） | 108 |
| HRE must-thrown | 37 |
| INVALID_STATE must-thrown | 30 |
| startOffset expected 2 got 4（16,x） | 11 |
| Text/differing 残余 | ~130 |

## 二、两个 A/B 负结果

1. **cDP-only 开关拆解**（R227 三开 -28P 的归因实验）：仅开
   `Node.prototype.compareDocumentPosition`（contains/hasChildNodes 不开）→
   **1385→1357（-28）**——与三开完全相同。有害轴就是 cDP 单独本身：解锁 sim
   深入 iframe 合成树后，跨轮残留使后续 subtest 树形态与 host 分歧。R219
   开关族整体保持关，绑定 fresh-doc 深项不变。
2. **leaf 分支 kids>0 的 extract-first**（16,x/24,x 形态）：实现后 **0 变化**
   （diff 0 fixed / 0 new）——`_coveredChildren` 对这些形态返回 null 或空，
   分支不可达。已回退（CLAUDE.md 最小改动）。

## 三、HRE 37F 簇 sim 考古（R233 靶点分析）

24,x（`[testDiv,2,paras[4],1]` + Text newParent）：期望 HRE 但 host 的 R210
部分包含检查应先抛 INVALID_STATE——而 sim 两查都未触发（partial check 未命中
insertNode validity 抛 HRE）。推断：sim 的 partial 检查 nextNode 遍历**提前
终止**（stop = nextNodeDescendants(cac)），部分包含的 paras[4] 未被扫到；
16,x 的 startOffset 2 涉及 body 内 harness iframe 的 index 算术。两簇都需
沙箱直跑 mySurroundContents 全源对照（R231 方法论可复用——注入 sim 全源 +
dump 中间态逐行对齐）。

## 四、R233 靶点

- **(a) mySurroundContents 全源注入探针**（R231 方法论复用）：对 24,x/18,0/
  16,x 形态 dump sim 的 partial-check 终止点与 insertNode validity 触发位，
  逐行对齐 host。
- **(b) assert_unreached 133**（fresh-doc 深项入口评估）。
- **(c) 深项清单不变**：fresh-doc 残余 / customElements 多 registry /
  :scope query-root / lone-surrogate wire / MO-document parser 记录。

## 五、commit

无代码 land（实验已回退）。
