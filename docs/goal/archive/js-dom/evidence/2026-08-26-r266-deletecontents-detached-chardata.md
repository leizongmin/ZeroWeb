# R266 Evidence — deleteContents 同节点 CharData（detached）放宽（+12P）

**日期**: 2026-08-26
**切片**: M4——R266(a) deleteContents 重聚类首切片
**改动面**: part06 deleteContents 的 R213 分支 guard 放宽 + part23.rs（+1 回归单测）

## 一、重聚类发现（R265 后基线 80P/49F）

按 range 形态聚类，12F 簇共享签名 `[detachedTextNode, 0, …, 8]` 族
（32-37,x：detachedTextNode/detachedForeignTextNode/detachedXmlTextNode/
detachedComment/detachedForeignComment/detachedXmlComment 的同节点区间）：
「differing nodeValue expected "" but got "Uvwxyzab"」——deleteContents 整体
空转，data 原样。

## 二、根因与修复

deleteContents 的 R213 同节点分支 guard 要求 `_r213sc.parentNode` 非空
（异节点中段 remove 循环的遗留门）——detached 节点 parentNode=null 整族跳过。
extractContents 的 R228 已在 R228 轮放宽同款（`sc===ec || (双父非空且相等)`），
deleteContents 漏了对齐。

修两处：
1. 外层 guard：`(_r213sc === _r213ec) || (双父非空且相等)`——同节点无需父
   容器（spec `dom-range-delete-contents` 的 replace-data 段不依赖 parent）；
2. 内层 `si>=0` 门：同节点形态恒进（detached 时 kids 空 si=-1，但同节点
   分支只做 deleteData + collapse，无中段兄弟需移除）——`_r213sc === _r213ec
   || (si>=0 && ei>=si)`。

## 三、验证（vs R265 基线）

| 项 | R265 | R266 | Δ |
|---|---|---|---|
| Range-deleteContents | 80P/49F | **92P/37F** | **+12**（32-37,x 六形态 × DOM/position） |
| Range-extractContents | 160P/32F | 160P/32F | 持平 |
| Range-cloneContents | 162P/29F | 162P/29F | 持平 |
| Range-surroundContents | 1840P/0F | 1840P/0F | 持平（100%） |
| engine 单测 | 2404 | **2405** | +1（r266 回归单测）全绿 |
| fmt / clippy | 干净 | 干净 | — |

## 四、剩余聚类（R267 靶点）

deleteContents 残余 37F 按形态：
- **ancestor-element 族**（23,x `[paras[0],0,paras[0].firstChild,7]` 等）：
  sc 是 ec(text) 的父元素——须削 text 头部 [0,so) 保留 remainder（期望
  "̈efgh\n" got 全量）——R236 extract 的 ancestor 分支在 delete 侧的对应缺口；
- document/documentElement 容器族（17/25/26/49/50/51,x）：doc 级 child 数
  分歧（expected 1 got 2——doctype/html 摘除不完整）；
- 跨容器族（20-22/24/28-31/48/52/53,x）：R240-R242 extract 系列分支的
  delete 侧对应。
- extractContents 32F / cloneContents 29F 独立聚类。

优先级：ancestor-element 族（数量大、与 R236 extract 分支对称、模式已知）。
