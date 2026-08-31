# R230 Evidence — Text 容器 leaf-newParent 的 extract 先行（extract 边界精度簇第一片）

**日期**: 2026-08-25
**切片**: M4——R230(a) surround 剩 650F 的 Text differing ~180 簇首片
**改动面**: `part06.js`（leaf 分支 3/4 容器 extract→insert 序）+ `part23.rs`（回归测试扩展 R230 断言）

## 一、根因

Text/CDATA 同节点容器的 leaf-newParent（Text 型 newParent，WPT
Range-surroundContents 9,x `[detachedPara1.firstChild, 2, 8]` + text newParent
族）：sim（common.js mySurroundContents）序 = 步骤 3 extract（**源 text 削为
前缀**"Op"，切片"qrstuv"进 frag）→ 步骤 4 insertNode(newParent) → 步骤 5
appendChild(frag) 抛 HRE。host 的 leaf 分支（R229 只对 7/8 容器补了
extract-first）对 3/4 容器仍只 insertNode——源 text 保留原文 "Opqrstuv"（got
"qrstuv" 族）。

## 二、修法

leaf 分支 3/4 容器同节点路径补 `this.extractContents()` 于 insertNode 之前
（与 7/8 的 R229 序对齐——sim 步骤 3-5 完整序）。

https://dom.spec.whatwg.org/#dom-range-surroundcontents

## 三、验证链（vs R229）

| 项 | R229 | R230 | Δ |
|---|---|---|---|
| Range-surroundContents | 1190P/650F | **1269P/571F** | **+79，0 新失败**（diff 79 fixed / 0 new） |
| Range-insertNode | 1840P/0F | 1840P/0F | 0（100% 保持） |
| Range-extractContents | 115P | 115P | 0 |

- **engine 单测**：**2380 全绿**（回归测试扩展 `r230:Op,HierarchyRequestError`
  ——源 text 削前缀 + HRE 上抛）。
- **fmt / clippy**：零警告。

## 四、R231 靶点（当前 571F 聚类）

| 簇 | 计数 | 备注 |
|---|---|---|
| assert_unreached | 133 | fresh-doc 跨轮残留族 |
| cDP 缺方法面 | 108 | R219 开关（fresh-doc 深项绑定） |
| HRE / INVALID_STATE must-thrown | 37+30 | sim 全序残余（newParent=祖先的 ensure 在塌缩后 parent_ 判定） |
| endOffset expected 8/9 got 2（2,x/27,x 等） | ~93 | **最大可动簇**：sim 的 myExtractContents 对工厂 text 的塌缩行为与 spec 不同（疑 extract 中途 catch 使 range 未动）——需逐行对齐 sim 的 early-return 形态 |
| Text differing 残余 | ~16 | R230 同族边缘 |

## 五、commit

c0a850782
