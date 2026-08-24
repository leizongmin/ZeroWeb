# R235 Evidence — leaf-newParent extract-first 扩展两形态（+88P）+ cDP own-prop 负结果

**日期**: 2026-08-25
**切片**: M4——R235(a) foreignDoc cDP 残余（负结果回退）+ R235(b) differing 簇子切片（正结果）
**改动面**: `part06.js`（surroundContents leaf 路径两分支）+ `part23.rs`（r235 单测）
**commit**: `625e67e62`

## 一、R235(a) cDP own-prop 负结果（已回退）

给 `_makeDetachedDocument` 内部 docEl/headEl/body 附加 contains/cDP
own-property（镜像 R234 诊断——foreignDoc =
`implementation.createHTMLDocument` 路径的 documentElement/body 是
plain object 无 cDP）：

- **实测**：fixed 6 / new **34**（净 -28）——17,x「resulting DOM」转绿
  但「resulting range position」翻红。
- **结论**：与 R227/R232 同现象——**解锁方法面使 sim 完成度变化，
  position 断言翻转面大于 DOM 断言修复面**。17,x/30,x 的 cDP 40F 簇
  绑定在更深的 host 语义缺口上（host 对 foreignDoc.docEl 容器的
  surround 全序未对齐），单独补方法面必 -28。回退，记深项。

## 二、R235(b) leaf-newParent extract-first 两形态（land）

surroundContents 的 leaf-newParent（Text/Comment/PI/CDATA 作 newParent）
路径只覆盖 R230 的同节点 Text/CDATA 容器形态。两缺失形态在
「extract 先行」上与 sim（common.js mySurroundContents 步骤 3 无形态
分支）分歧——host 直接抛 HRE 使树保留区间原文：

1. **异节点同父 CharData 区间**（6,x `[paras[5].firstChild,2,
   paras[5].lastChild,4]`——CDATA#1→text 同父 46F assert_unreached 簇）：
   extract 先削首尾切片（deleteData）→ insertNode → HRE。
2. **元素容器含覆盖子**（18,x `[paras[0],0,paras[0],1]` + Text
   newParent differing 簇）：extract 先移出 covered 子（容器清空）→
   insertNode 插 newParent（树内留 'z'）→ HRE。

**过程坑**：首版第二分支未排除同节点形态（`sc.parentNode ===
ec.parentNode` 对 sc===ec 恒真），劫持 R229/R231 的 PI 同节点流 →
39,x 翻红 104。加 `this.startContainer !== this.endContainer` 守卫后
fixed=88 / new=0。

## 三、验证链（vs R234 基线）

| 项 | R234 | R235 | Δ |
|---|---|---|---|
| Range-surroundContents | 1421P/419F | **1509P/331F** | **+88，0 新失败**（6,x 46 + 18,x 30 + 12–14,x 残余 12） |
| Range-insertNode | 1841P/0F | 1841P/0F | 0（100% 保持） |
| Range-extract/delete/clone | 118/67/155 | 118/67/155 | 0 |
| **ranges 全量**（除 probe） | 40080 行 | 40080 行 | set-diff **88 Fail→Pass，0 Pass→Fail** |

- **native 同值**：ZW_NATIVE_DOM=1 surround 1421→1509P（+88 与 polyfill 一致）。
- **engine 单测**：**2382 全绿**（新增 r235_leaf_newparent_extract_first_variants：
  xnode 削尾/削头 + elem 容器清空留 newParent 双断言）。
- fmt/clippy 干净。

## 四、R236 靶点（331F 重聚类）

| 簇 | 估计 | 备注 |
|---|---|---|
| differing | ~136 | 12–14,x（docEl 容器 host/sim extract 差异）+ 23,x（node-_descendant 区间 split）+ 19,x detached |
| assert_unreached | ~63 | 克隆子树内部残留（R234 dump 只证 doc 级无累积） |
| cDP | 40 | 17,x/30,x——绑定 host surround 全序（本轮负结果实证不可单独解） |
| HRE / INVALID_STATE | 37/30 | R233 归因修正后需重评独立性 |
| partial-selected 12 / startOffset 11 / other 2 | 25 | 24,x message 形态 + 16,x index 算术 |

- **首选**：23,x `[paras[0],0,paras[0].firstChild,7]`（node-与-后代区间）
  ——host `_coveredChildren` 对 sc≠ec 恒 null 使 extract 空转；spec 需
  split 后代边界（end 在 text 内 → splitText + 部分移动）。32F 簇。
- 次选：12–14,x 残余 differing（R234 动态 getter 后 host 真实提取的
  树形态与 sim 首差节点）。
