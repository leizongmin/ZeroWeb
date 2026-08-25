# R257 Evidence — self-surround HRE（清子后 inclusive-ancestor 检查，18,0/19,6 簇全解 +4P）

**日期**: 2026-08-25
**切片**: M4——R257(a) 18,0/19,6 self-surround 4F
**改动面**: `part06.js`（surroundContents 元素主路径清子循环后新增 inclusive
ancestor 检查 + HRE 先变更后抛）+ `part23.rs`（+1 回归单测三场景）
**commit**: 702d6d80c

## 一、形态与根因

- **18,0** `[paras[0],0,paras[0],1]` + paras[0]、**19,6**
  `[detachedPara1,0,detachedPara1,1]` + detachedPara1：**self-surround**
  （newParent === range 容器）。spec `dom-range-surroundcontents` 步骤 4
  insertNode 的 pre-insertion validity（`concept-node-pre-insertion-validity`
  「node 是 parent 的 inclusive ancestor → HRE」）必抛。
- host 元素主路径（R237 clone 循环 + 直接 insertBefore）**不经 insertNode**，
  无任何拦截：清 newParent 子（把自身内容误删）+ R248 newParent.remove()
  （把自己摘出旧父）后静默成功——WPT `assert_throws_dom "A
  HIERARCHY_REQUEST_ERR must be thrown"` 两连失败。

## 二、时序关键（首版教训 → 19,9 回归修正）

首版把 ancestor 检查放在**清子循环前**：19,9
（`[detachedPara1,0,…]` + detachedDiv——newParent 是容器的**父**）翻绿成
fail——sim 不抛而 host 误抛。根因：sim 序（common.js mySurroundContents）
步骤 3 extract → 步骤 2 **清 newParent 子** → 步骤 4 insertNode 校验。当
range 容器是 newParent 的**子**（19,9 族），步骤 2 清子已把容器从
newParent 摘出——sc→newParent 父链断，步骤 4 的 inclusive-ancestor 上行
walk 不再命中，wrap 合法成功（sim 期望树 = 容器内 [div[text]]）。

**正确时序 = 检查在清子之后**：只有 true self-surround（newParent === sc
或经不经 newParent 子列表的父链命中）仍抛。抛出前先移出 covered 子
（extract 等价——18,0 的清子循环因 newParent===sc 已顺带清空）+ range
塌缩，对齐 sim 的「先变更后抛」中间态。

## 三、验证（vs R256 基线）

| 项 | R256 | R257 | Δ |
|---|---|---|---|
| Range-surroundContents | 1822P/18F | 1826P/14F | **+4**（18,0/19,6 四连） |
| ranges 上游 set-diff | — | — | **+4 F2P / 0 P2F** |
| engine 单测 | 2396 | 2397 | +1 三场景回归单测全绿 |
| fmt / clippy | — | 干净 | — |

（19,9 在 R256 基线已是 Pass——R256 sibling 重接线已解其树分歧，本轮仅
防误抛回归，单测场景三锁定。）

## 四、R258 靶点

- 16,x startOffset 11F（异步 fetch-rebuild 时序深项）
- 28,0 1F / 30,4 + 30,11 2F 残余
- customElements 多 registry / :scope query-root
