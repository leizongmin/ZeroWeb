# R229 Evidence — sim 全序复刻三步（leaf-newParent extract-first / detached 不塌缩 / 同节点 self-collapse）

**日期**: 2026-08-25
**切片**: M4——R229(a) surround 剩 897F 重聚类 → 三个可轻量修复簇
**改动面**: `part06.js`（leaf-newParent comment/PI 先 extract / R228 detached 分支不塌缩 / R211 同节点 self-collapse）+ `part23.rs`（R228 测试扩展三断言）

## 一、重聚类（943P/897F 基线）

| 簇 | 计数 | 处置 |
|---|---|---|
| assert_unreached | 275 | 跨轮残留深项（fresh-doc 族） |
| cDP 缺方法面 | 108 | R219 kill-switch（fresh-doc 深项绑定） |
| HRE/INVALID_STATE must-thrown | 43+30 | 部分本轮收（leaf-newParent 族） |
| Stuwxyz 残余（35,1–2 / 36,x） | 22 | **本轮修**（簇 1） |
| endOffset expected 8 got 0（32,x） | 18 | **本轮修**（簇 2） |
| startOffset expected 0 got 3（39,x） | 17 | **本轮修**（簇 3） |
| Text/Comment/PI differing | ~180 | extract 边界精度残余 |

## 二、三修复（逐簇对齐 sim 语义）

1. **leaf-newParent（Text/Comment 型 newParent）对 comment/PI 容器**：sim 序 =
   extractContents（**变更容器 data**：中段切片 deleteData）→ myInsertNode 抛
   HRE。旧版 leaf 分支只 insertNode（3/4 容器）或直接抛（7/8 容器）——容器 data
   不变。修：7/8 同节点容器先 extract 再抛（+55P，Stuwxyz 族全消）。
2. **detached 全区间 extract 不塌缩**：sim 的 myExtractContents 塌缩步走
   `parent_` 定位——无父时**不重设边界**（range 保持 (node,0)-(node,8)）。旧版
   R228 分支强 collapse 到 (容器, a)（endOffset 8→0，18F）。修：去掉
   setStart/setEnd（+69P）。
3. **同节点区间的 self-collapse**：sim 塌缩首分支 `isAncestorContainer(start,
   end)` 对同节点 self 命中——collapse 到 **(容器, startOffset)** 而非 (父,
   si+1)（else 分支才走父定位）。旧版 R211 同节点一律 (父, si+1)（PI 落
   (xmlDoc,3)，17F）。修：`sc===ec` 分设（+117P 累计，含 39,x 全消）。

https://dom.spec.whatwg.org/#dom-range-extractcontents
https://dom.spec.whatwg.org/#dom-range-surroundcontents

## 三、验证链（vs R228）

| 项 | R228 | R229 | Δ |
|---|---|---|---|
| Range-surroundContents | 943P/897F | **1190P/650F** | **+247，0 新失败**（失败集 diff 247 fixed / 0 new） |
| Range-insertNode | 1840P/0F | 1840P/0F（全量 1841） | 0（100% 保持） |
| Range-extractContents | 111P | 115P | +4 |
| Range-deleteContents / cloneContents | 65 / 152 | 65 / 152 | 0 |

全量套件：ranges 37740→**37991（+251）**；nodes +1、events 577→579（flake 带
回归）、collections 49 / traversal 1602 稳。净 **≈ +254P**。

分步增量：+55（簇 1）→ +69（簇 2）→ +117（簇 3，累计 1190）。

- **engine 单测**：**2380 全绿**（R228 回归测试扩展三断言：PI 同节点
  startOffset=0 + HRE、leaf newParent 先切片（"Stuwxyz"）再抛 HRE）。
- **fmt / clippy**：零警告。

## 四、R230 靶点

- surround 剩 650F：assert_unreached ~253 / cDP 108（fresh-doc 深项）/ HRE 43 +
  INVALID_STATE 30 残余 / Text differing ~180（extract 边界精度）。
- 深项：fresh-doc 残余 / customElements 多 registry / :scope query-root /
  lone-surrogate wire / MO-document parser 记录。

## 五、commit

21686866a
