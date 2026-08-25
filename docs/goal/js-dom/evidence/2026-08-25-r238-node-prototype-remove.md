# R238 Evidence — Node.prototype.remove 泛型（assert_unreached 76F 簇主根因，+59P 纯增）

**日期**: 2026-08-25
**切片**: M4——R238 HRE 36 + assert_unreached 76 重评 → remove 缺失根因
**改动面**: `part03.js`（Node.prototype.remove 泛型）+ `part23.rs`（r238 单测）
**commit**: `2a18b40a2`

## 一、重评过程（R237 后重聚类）

228F 重聚类：assert_unreached 76（18/19/24,x）/ cDP 40 / HRE 36 /
INVALID_STATE 30 / differing 22。递归探针（R238-probe，已清理）对
19,1 dump 双引擎树：

- **host**：detachedPara1 = `[Ä-text, Op-text]`（新插入 + **原文本残留**）
- **sim**：`[Ä-text]`（原文本已移出）

根因：extractContents 与 surroundContents 路径 4 都经
`typeof kids[j].remove === 'function'` 守卫摘除 covered 子——
**iframe 子文档工厂文本（createTextNode 字面量）无 remove 方法**
（Text.prototype 占位链亦无），守卫静默跳过移除。

## 二、修复

`Node.prototype.remove` 泛型（spec `dom-child-remove`：父非空则
parent.removeChild(this)），own-property 版本优先（Element.prototype.remove
/ 工厂自有）。**过程坑**：首版直接补 `_zwMText`/`_zwMComment` 工厂——
实测 0 变化（失败节点来自 iframe createTextNode 工厂，非 detached
工厂）；泛型原型一次覆盖全部形态。

## 三、验证链（vs R237 基线）

| 项 | R237 | R238 | Δ |
|---|---|---|---|
| Range-surroundContents | 1612P/228F | **1671P/169F** | **+59，0 新失败** |
| Range-extractContents | 121P | **125P** | +4 |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-delete/clone | 67/155 | 67/155 | 0 |
| nodes（全目录） | 12661P | **12663P** | +2 |
| events（全目录） | 7F | 7F | **失败集逐条一致**（579/578 计数差为基线侧 flaky pass） |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1612→1671P（+59 一致）。
- **engine 单测**：**2385 全绿**（新增 r238_node_prototype_remove_generic）。
- fmt/clippy 干净；探针已清理。

## 四、R239 靶点（169F 重聚类）

| 簇 | 计数 | 行 | 备注 |
|---|---|---|---|
| cDP | 40 | 17,x/30,x | 绑 host foreignDoc surround 全序（R235 负结果——不可单独补方法面） |
| HRE | 36 | 24/25/26/28,x + 18/19,x 各 1 | 跨子区间（[testDiv,2,paras[4],1]）+ Document/Doctype 容器 |
| INVALID_STATE | 30 | 20/21/22/29/31,x 各 6 | 部分包含检查（20,x [paras[0].firstChild,0,paras[1].firstChild,0]） |
| differing | 22 | 28,x 17 + 13/14,x 4 | 28,x `[foreignDoc.body,0,foreignTextNode,36]` |
| assert_unreached | 18 | 24,x 16 | 跨子区间残余 |
| partial-msg 12 / startOffset 11 | 23 | 24,x / 16,x | message 形态 / index 算术 |

- **首选**：INVALID_STATE 30（20–22,x 同构三行——`[p0.firstChild,0,
  p1.firstChild,0]` 跨 text 边界部分包含检查，疑 host surround 入口
  partial-check 对多容器形态漏判——需对齐 sim 的 isPartiallyContained
  遍历起点/终点）。
- 次选：HRE 24,x（跨子区间 leaf-newParent 的 extract 全序——sim 对
  [testDiv,2,paras[4],1] 的 extract 涉及多子部分包含，需 generic
  cross-container extract）。
