# R276 Evidence — 22,x 比较遍历的 wrapper data 源分歧（诊断轮：R275 假设修正 + 单对象源确认）

**日期**: 2026-08-26
**切片**: M4——R276(a) 双 iframe body 绑定假设验证（推翻）+ 比较遍历归因
**改动面**: 无生产代码（诊断轮）

## 一、R275 假设推翻（post-delete 伪影）

R275 的「expected iframe 扁平 body」是 **post-delete 读取伪影**：probe dump
发生在 myDeleteContents 之后（range 已被 oracle 塌缩/重指）。本轮 PRE dump
实证：**双侧 doc 完全同构**（DIV kids=7：6P+comment，scE.parentNode=P 正确）
——双 iframe body 绑定（R221）无缺口，R275 假设作废。

**教训**：dump range 相关状态必须在 mutation **前**（post-delete 的 sc
父链是塌缩产物，不是树结构证据）。

## 二、真根因（比较遍历 dump）

注入 isEqualNode 的比较循环起点（actualRoots[0] 起 nextNode 双侧同步
walk，dump 每节点 nodeValue+data）：

```
A: ... P #text(nv=Äb̈,data=Äb̈)   ← FULL（未修剪）
E: ... P #text(nv=Äb,data=Äb)    ← 修剪后
```

**同一棵树的两个遍历面**：
- childNodes 遍历（我此前的 dump）：P#a 的 text = 修剪后（"Äb"）；
- **比较器用的 nextNode 遍历**（firstChild/nextSibling 导航）：P#a 的
  text = **完整数据**。

即 `paras[0].firstChild`（作为 range 端点被 deleteData 修剪的对象）≠
`paras[0].firstChild`（遍历导航返回的对象）——**firstChild 导航每次构造/
返回 data 源不同的 wrapper**（host 未更新：_regWrite 对无 sel 的 handle 父
是纯本地写，host text 保持完整；导航 wrapper 读 host 源）。

这正是 R272 最初假设的 **wrapper identity churn** 的实体形态（当时方向对、
层次浅了）：不是 identity 不同，是 **data 源不同**的平行 wrapper。

## 三、R277 修复方向

P（handle 元素）的 firstChild/childNodes 融合视图的 text wrapper 必须与
textEl 注册表的 node **同一对象**（单一对象源）：
- `_handleChildNodes`/融合视图对 text 子的包装改为从 `_zwTextElsByEl`
  （handle 键）取注册 node（textContent= 建的形态已如此——R84 路径）；
- 本形态的 P#a text 来自 **克隆树**（_zwDeepCloneEl 的 _zwMText）+
  setupRangeTests 的 appendChild 入 registry——deleteData 修剪的是 range
  拿到的那个（_zwMText 域），导航读到 host 源 wrapper——须统一为 registry
  的 _zwMText 对象。

## 四、验证

| 项 | R273 | R276（诊断轮） |
|---|---|---|
| Range-deleteContents | 115P/14F | 115P/14F（文件 restore 零残留） |

## 五、R277 靶点

- **(a) text wrapper 单一对象源**（22/48/52/53,x 共同根因的修复面）：
  导航（firstChild/nextSibling 路径）与 range 端点共享同一 textEl/_zwMText
  对象。
- (b) 28,x / 49/50,x；extract/clone 重聚类。
