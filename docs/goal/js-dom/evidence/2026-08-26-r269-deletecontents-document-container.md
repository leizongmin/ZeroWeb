# R269 Evidence — deleteContents document 容器分支（+6P，四轮累计 +28P）

**日期**: 2026-08-26
**切片**: M4——R269(a) deleteContents document 容器族 + element 端点族分析
**改动面**: part06 deleteContents 新增同容器 Document 删除分支 + part23.rs（+1 单测）

## 一、document 容器族（已修）

**25/26,x**：`[document,0,document,1/2]`——doc 的 contained 子（doctype/
html）旧 `_coveredChildren` 融合视图对主文档 proxy 恒空 → 无移除
（「expected 1/0 got 2」）。修：**同容器 Document 限定分支**——contained
子 [so,eo) 逆序 rm（remove + removeChild 兜底）+ 塌缩 (doc, so)。
**限定 nodeType 9**：元素/fragment 容器已被 _coveredChildren 回落正确处理
（防行为面漂移——R268 教训的模式复用）。doctype 走主文档 removeChild 的
本地标记路径；html 移除 host 不支持但 JS 视图按记录反映。

连带 +2（51,x `[document,1,document,2]` 同族）。

## 二、element 端点族分析（本轮定性，未修）

- **24,x** `[testDiv,2,paras[4],1]`：probe 检验「expected 7」= **expected 侧
  oracle 抛异常后回退的未触碰树**（7 子）——myDeleteContents 在我方引擎的
  树遍历（nextNode/isContained 对 CDATA 子树）上失败。非 delete 算法缺口，
  是 **oracle 侧树遍历的 shim 缺口**（6,x CDATA 族同源）。修复面在遍历
  原语（compareDocumentPosition 缺方法：17,x 的 nodeB 错误同族）。
- **22/48/52/53,x**：element 端点跨容器（部分同源）。
- **17,x**：foreignDoc.documentElement 无 compareDocumentPosition 方法
  （oracle 崩）——shim 方法面补齐项。

## 三、验证（vs R268 基线）

| 项 | R268 | R269 | Δ |
|---|---|---|---|
| Range-deleteContents | 102P/27F | **108P/21F** | **+6**（25/26/51,x 六 subtest） |
| Range-extractContents | 160P/32F | 160P/32F | 持平 |
| Range-cloneContents | 162P/29F | 162P/29F | 持平 |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%） |
| Range-insertNode | 1841P/0F | 1841P/0F | 持平（100%） |
| Range-mutations-insertBefore | 76P/0F | 76P/0F | 持平（100%） |
| engine 单测 | 2407 | **2408** | +1（r269 单测）全绿 |
| fmt / clippy | 干净 | 干净 | — |

deleteContents 四轮累计：80P/49F → **108P/21F（+28）**。

## 四、R270 靶点（残余 21F）

- **oracle 遍历原语族**（17/24/6,x 同源）：foreignDoc 域
  compareDocumentPosition 方法面补齐（17,x 直修；24/6,x 的 oracle 回退
  可连带解锁）。
- **element 端点跨容器**（22/48/52/53,x）：方向分支 contained 递归。
- **28/29,x**（`[testDiv,0,comment,5]` 深形态）/ 49/50,x（cursor-only 差异）。
- extractContents 32F / cloneContents 29F 独立聚类。
