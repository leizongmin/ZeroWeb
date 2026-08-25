# R259 Evidence — leaf-HRE 空覆盖子 extract-first（16,x 全簇 +11P，surround 100%）

**日期**: 2026-08-25
**切片**: M4——R259(a) 16,x startOffset 11F（surround 最后一簇）
**改动面**: `part06.js`（leaf-newParent 路径 kids===0 分支补 extract-first）+
`part23.rs`（+1 回归单测）
**commit**: 29b8fe636

## 一、形态与诊断链

16,x = `[document.body,4,document.body,5]` × Text/Comment/PI newParent
（j=1,2,3,5,7,14–19 共 11F）——「startOffset expected 2 but got 4」，DOM 断言全过。

探针四连（harness 内嵌 assert 强制失败法，R248/R256 方法论）：
1. **PRE**（sim 运行前）：A/E 双侧完全一致——`(BODY N=1[DIV], so=4, eo=5)`
   （ZeroWeb iframe body 只有 testDiv 一个子，offset 4/5 为真浏览器 body 形态
   下的索引；两侧同形 → 分歧纯算法路径非环境）。
2. **POST（E 侧）**：sim 终态 `(BODY N=2[DIV,#text], so=2, eo=2)` +
   HIERARCHY_REQUEST_ERR。
3. **POST（A 侧）**：host 终态 `(5,5)`（旧版）——边界漂移。
4. **机制解码**：sim 的 mySurroundContents 步骤 3 **无条件**调
   myExtractContents——其 setStart/setEnd 折叠对写 (body,4→4)；步骤 4
   myInsertNode 尾步 setEnd(body,2)（newOffset = 插入前 nodeLength+1）经
   **shim 的 R203 crossing 重设**（end < start 时把 start 拉平）自然得到
   (2,2)。host 的 kids===0 分支只调 insertNode：range 保持 (4,5) 未折叠，
   R219 的 start===end 守卫跳过 setEnd，边界漂移 (5,5)。

## 二、首版教训（镜像回归）

首版在 HRE 抛出前加「start 镜像到 end」——12–14,x leaf 簇回归 +87F（机制
错误：跨容器形态的 sim 终态不经 crossing 重设）。正确修复 = 按 sim 序
**kids===0 也先 extract**（折叠对使 R219 生效，crossing 重设自然产出双边界）。

## 三、验证（vs R258 基线）

| 项 | R258 | R259 | Δ |
|---|---|---|---|
| Range-surroundContents | 1829P/11F | **1840P/0F（100%）** | **+11**（16,x 全簇） |
| ranges 上游 set-diff | — | — | **+11 F2P / 0 P2F** |
| engine 单测 | 2398 | 2399 | +1 回归单测全绿 |
| fmt / clippy | — | 干净 | — |

**里程碑注**：Range-surroundContents 自 R210（823P/1017F）起经 R210–R259
共 26 轮聚类驱动修复，现 **1840P/0F 100%**。R255 的「异步 fetch-rebuild
时序」假设被本轮推翻——16,x 真因是 leaf 路径算法序（无环境时序因素）。

## 四、R260 靶点

- insertNode / extractContents / cloneContents 套件残余重聚类（R256–R259
  行为面四轮变化后取样）
- 深项：customElements 多 registry / :scope query-root
