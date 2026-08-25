# R264 Evidence — insertBefore 超时族根因定位（text-ref 静默插入 + indexOf 死循环）

**日期**: 2026-08-26
**切片**: M4——R264(a) insertBefore 两段接线 + 超时根因定位（诊断轮，部分 land）
**改动面**: part04 insertBefore 两段调整钩子（remove 段前置 + insert 段收尾）+
诊断注记；无行为修复（hang 根因在 registry 插入缺口，R265 靶点）

## 一、接线（land 部分）

part04 insertBefore 的 handle 子路径补 R263 两段：
- remove 段在 wire 插入前（`__zwAdjustRangesForRemove(newNode)`——adopt 语义）；
- insert 段在 `_mo_notify`/`_zwLiveNLSync`/CE 传播收尾后（`ceAdded` 逐子）。

验证：appendChild 70P/0F、replaceChild 60P/0F、removeChild 20P/0F 全保持；
engine 2403 全绿；insertBefore 超时形态与 clean-HEAD 逐点一致（stash A/B）。

## 二、insertBefore 超时根因（探针链五轮，定位到单点死循环）

1. **表格二分**（38 条 insertBeforeTests 逐条循环 + 200ms 慢条打印）：
   0-6 条 32ms 全过、**第 7 条单点 90s 挂起**（`["paras[0]", "paras[1]",
   "paras[0].firstChild", ...]`——paras[0].insertBefore(paras[1],
   paras[0].firstChild)）。
2. **clean-HEAD 复现**：stash 后同样挂起——**预存项**，非 R262/R263 引入。
3. **PRE 视图探针**：p1.parentNode=testDiv(nt=1)、p1 在其 childNodes、
   firstChild(nt=3) 在 paras[0].childNodes——**前置状态全正确**。
4. **引擎直调探针**：`paras[0].insertBefore(paras[1], paras[0].firstChild)`
   **0ms 返回** ret=p1——引擎调用本身无循环。
5. **结论**：死循环在**调用方 common.js indexOf**（`while (node !=
   node.parentNode.childNodes[i]) i++`——无终止条件）。机制：引擎 insertBefore
   对 **refNode 是 textEl 包装**（textContent= 建的文本子，无 selector 无
   handle）且父是 handle 容器的形态，三个 wire 分支（refNode.__zwSelector /
   handle && refNode.__zwHandle / fragment）**全不命中**——插入静默不入
   registry，paras[0] 的 JS childNodes 视图不含 paras[1]（融合视图
   _zwLocalChildNodes 只有 textEl、_handleChildren 只有既有 handle 子）→
   随后 sim 的 modifyForRemove → indexOf(paras[1]) 在 paras[0].childNodes
   上无终止自旋。

## 三、修复方向（R265 靶点）

text ref 的 registry 插入：refNode 无 sel/handle 时经 `refNode.parentNode`
（textEl 的父 proxy）或 `_zwTextElsByEl`/`_zwTextElsByHandle` 反查父 handle，
把 newNode splice 进 `_handleChildren[handle]` 的**融合位次**（textEl 视图与
registry 的合并顺序——与 `_childNodeList` 融合读一致）。风险面：融合位次
错位会使 childNodes 顺序错（insertNode 套件的 text-ref 形态已 100%，须
stash A/B 防 P2F）。

## 四、验证

| 项 | R263 | R264（接线后） |
|---|---|---|
| Range-mutations-appendChild | 70P/0F | 70P/0F 持平 |
| Range-mutations-replaceChild | 60P/0F | 60P/0F 持平 |
| Range-mutations-removeChild | 20P/0F | 20P/0F 持平 |
| Range-mutations-insertBefore | 超时 | 超时（同点挂起，clean-HEAD 同形） |
| engine 单测 | 2403 | 2403 全绿 |
| fmt / clippy | 干净 | 干净 |

## 五、R265 靶点

- **(a) text-ref registry 插入**（本文件 §三方向）：解锁 insertBefore 套件
  （38 条 × 2 = 76 subtest 的整族验证面）。
- (b) deleteContents 49F / extractContents 32F / cloneContents 29F 重聚类。
- (c) replaceData/dataChange 超时（累积型，低 ROI）。
