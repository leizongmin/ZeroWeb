# R274 Evidence — 22,x oracle walk 提前终止归因（诊断轮：元素 wrapper 的 nextSibling 缺口）

**日期**: 2026-08-26
**切片**: M4——R274(a) element 端点跨容器族归因
**改动面**: 无生产代码（诊断轮）；「直接注入真实测试文件」方法论沉淀

## 一、诊断链（真实文件注入 + nextNode 追踪）

1. dual-iframe probe 复刻 22,x：**POST-A === POST-E**（两侧一致地保留了
   P#b/P#c 中段——oracle walk n=0）→ probe 环境与真实测试分歧。
2. **直接注入真实测试文件**（wpt-data gitignored——R222 模式）：在
   `Range-deleteContents.html` 的比较点注入 assert-dump，i===22 时 dump
   双侧文档树 → **POST-A === POST-E 仍成立**（文档树一致）——断言的
   「树根 First difference #text expected Äb got 全量」来自 **detached
   nodes 段**（testDeleteContents 的 actualAllNodes/expectedAllNodes
   逐节点 isEqualNode——被移除节点的孤儿副本比较）。
3. **nextNode 链追踪**（注入点内联 oracle 的爬升算法逐步）：
   `DIV->P`（首子 ✓）→ `P->#text`（✓）→ **`#text->NULL`**——
   `P#a.nextSibling` 在 expected iframe 克隆域返回 null/falsy → 爬升取
   `P#a.nextSibling`（应为 P#b）失败 → walk 终止 → nodesToRemove=0。

## 二、根因定性

**元素 wrapper 的 nextSibling 在克隆域断链**（R273 的 CDATA 兄弟 getter
同族，但在 part04 元素 proxy 的 sibling trap 域——克隆+setupRangeTests
重建后的融合视图解析缺口）。oracle walk 依赖元素链遍历；中段移除
（paras[1..2]）与 ec 头部处理全部不发生（双侧一致空转），detached 段
比较的孤儿节点集两侧不同 → 断言失败。

**修复面**（R275）：part04 元素 nextSibling trap 对克隆域 testDiv 的
paras 子解析（fusion view 的元素子枚举——`_handleChildren`/textEl/
_childNodeList 三源合并顺序或索引缺失）。

## 三、方法论沉淀

- **真实文件注入**（gitignored wpt-data 直接改 + assert-dump + 跑完
  restore）：当 probe 复刻与真实测试分歧时，一步消除环境差异
  （R222 模式复兴——比 dual-iframe 复刻快且保真）。
- **detached 段归因意识**：deleteContents 系列的 isEqualNode 失败有三层
  （文档树 / detached 孤儿集 / 位置）——文档树 dump 相同时下一步 dump
  allNodes 孤儿集。

## 四、验证

| 项 | R273 | R274（诊断轮） |
|---|---|---|
| Range-deleteContents | 115P/14F | 115P/14F（文件已 restore，零残留） |
| engine 单测 | 2411 | 2411（无代码变更） |

## 五、R275 靶点

- **(a) 克隆域元素 nextSibling 断链修复**：22/48/52/53,x 的共同前置
  （oracle walk 恢复后这些形态的中段/头段处理才能对齐）。
- (b) 28,x 深形态 + 49/50,x cursor-only。
- (c) extractContents 32F / cloneContents 29F 重聚类。
