# R171 Evidence — L2-d3d 评估（element 上下文回退）+ 解析器提取

**日期**: 2026-08-22
**Commit**: `8a825479b`
**切片**: M1 L2-d3d 重评估——element/fragment 上下文本树化实验（回退）+ compound 解析器共用化提取（保留）

## 一、d3d 实验（两轮，均回退）

| 实验 | 形态面 | 结果 |
|----|--------|------|
| 1 | element 上下文 compound 全形态本树前置 | ParentNode 33→35F（+2 identity） |
| 2 | 收缩到纯 tag/`*` | 仍 +2F/-2F 持平互换 |

**+2 的形态**：`Detached Element.querySelector: :enabled`——`querySelector` 产物与
`querySelectorAll[0]` **对象不等**。机制：`:enabled` 形态含空白**本不走本树门**，
但 tag 前置改变了**相邻查询**的产物形态时序——归一缓存（`_zwMWrapCached`）对
两路径的键命中不同步。

**0 改善**：element 上下文没有任何 subtest 因本树化转 Pass——R165 的「902F
wrapper 依赖」结论在 R170 key 修复后**部分幸存**：element 消费面对产物形态的
敏感度高于 doc 上下文（doc 侧 gate 已无回归地落地）。

## 二、保留物

- **`_zwParseCompoundSel` 共用解析器**（模块级）：queryBody 的内联解析提取，
  行为等价（全量 9522P/343F/18T = R168 逐计数一致）——未来任何 gate 消费者
  （element 上下文重启 / fragment）复用同一形态判定。

## 三、结论（d3 域推进状态）

| 上下文 | gate 状态 | 依据 |
|--------|-----------|------|
| doc | **compound 已落地**（R170） | 三簇基线 + 双路径零差异 |
| element | **JSON 往返维持**（本轮回退） | 0 改善 + identity 边缘；重启前置 = querySelector/QSA 产物归一路径统一 |
| fragment | 未实验（element 结论的外推） | 同 element 前置条件 |

L2-d3 剩余：element/fragment 的产物归一统一（深水区）——或视为「doc 侧已获
本树化收益、element 侧 JSON 往返成本可接受」的**实用收口点**（RFC d3e 组合器
本树化的前提同样在此）。

## 四、验证

| 门 | 结果 |
|----|------|
| 全量 dom WPT polyfill | **9522P/343F/18T**（= R168 逐计数一致） |
| `make test` | 66 套件 **18087P/0F** |
| fmt / clippy | 干净 |

## 五、下一步（R172）

- **方向重估**：d3 主线（查询面本树化）已到实用收口点——转 **M6 域**
  （native dom_bindings 补齐）或 **Element-matches 剩 3F / ParentNode 剩 33F
  聚类**（轻件收口）。
- M1 的收口判据重估：L2「查询读 live 树」在 doc 上下文达成（tag+compound），
  element/fragment 维持 JSON——记录到 RFC 修订。
