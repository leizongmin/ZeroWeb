# lrlp 残余 264F 收口——壳保留等值降级，join 三案 7,518P/0F 全扫（100%）

**日期**: 2026-10-31
**前置**: [lrlp 壳丢失两笔 + R383×CE walk 撞车仲裁](2026-10-31-lrlp-shell-loss-and-r383-collision-adjudication.md)
（fix13 = 7,254P/264F，残余定性为 normal 白空间壳降级簇）。

## 1. 根因修正：等值判定，非无条件 normal 降级

上轮 §4 预判「有效白空间为 normal 时壳降级为不包裹」——本轮读语料源码
（`tests/wpt-runner/wpt-data/editing/other/join-different-white-space-style-paragraphs.html`）
发现该预判**不完整**：paragraphs 案的 `expectedHTML` 是
`aAttrsInLeftBlock => [...]` 参数化函数，accept 数组恒为
`style="${rightWhiteSpace}"`——right=normal 变体期望 `abc<span
style="white-space:normal">def</span>`（落进非 normal 左块，normal 语义须
保留壳）。

真正的规则：**保留壳只在源行有效 ws 与落位上下文有效 ws 不同时才需要**。

- paragraphs（块+块）：壳落进左块（ws=left），源行 ws=right，语料恒异
  → 恒壳（现行实现已正确，不受本轮影响）；
- lrlp（左行内行+右块）：壳落进宿主（corpus CE host 无 ws 声明 → normal），
  right=normal 时等值 → **裸**（`def`/`<b>def</b>`）——264F 唯一 delta
  （fix13 log 逐行核对：actual `style="white-space:normal"` 印 vs expected 裸，
  结构/落位/移除全对）；right=非 normal 时异 → 壳（已过面不变）。

无条件 normal 降级会倒灌 paragraphs（X→normal 全变体期望恒带壳）。

## 2. 落地三笔（全在 `part01.js` `_zwCeJoinApply`）

1. **落位修正块上提**至壳构造前（`_shellDst`/`_shellRef` 判定只读树结构，
   无提取时序依赖）+ `_dstWsVal = _zwCeWsOf(_shellDst)` +
   `_wsNeedsShell`（源行 `_wsVal` 与 `_dstWsVal` toLowerCase 等值比较）；
2. **`_stampWs` 等值早退**；per-run 文本节点等值时**裸迁入**（push 原文本
   节点，不再造带印 span）——单元素行经 `_stampWs` 同门；
3. **序列化文本形态臂**：shim 文本节点无 `outerHTML`（此前 `String(undefined
   || '')` = 空串 → `continue` 静默跳过序列化）——`nodeType === 3` 走
   `&<>` 转义 textContent → `__zw_insert_adjacent_html` 解析落引擎为文本
   节点，与元素壳「本地落位 + 序列化 copy」同 move 语义。

调用面封闭性：`_zwCeJoinApply` 仅 CE delete 家族四臂调用（backspace 块界/
select-boundary + forwarddelete 两臂），execCommand 路径不经此函数。

## 3. 验证账

| 门 | 结果 |
|---|---|
| lrlp 单案（/tmp/lrlp-normal-degrade.log） | 2,607 行 **0F**（fix13 = 88F/方法） |
| join 三案（/tmp/join-fix14-normal-degrade.log） | **7,518P/0F（100%）**；对 fix13 逐测试 ID **F→P 264 / P→F 0**，7,518 行恒等 |
| insertparagraph 语料（attr 门收益保全） | cannot-have-div 420P/0F ✓、inline-host 80P/800F ✓、white-space-style 616P/488F ✓、undo-moving-split-nodes 4F（既有挂账）零位移 |
| engine lib | 3,611P/11F，失败清单与 fix13 make-test 账逐条 diff 恒等 |
| webview lib | 822P/1F 已知红唯一 |
| fmt | `cargo fmt --all --check` clean（本轮仅 JS 改动） |

## 4. 时序安全注记

落位修正上提后 `_shellRef` 在部分几何由「提取后读」（startNode 已脱离 →
null → append 兜底）变为「提取前读」（attached 通过校验）：

- `srcParent === dst` 臂：ref=startNode 被提取脱离 → `insertBefore` 抛错 →
  既有 catch 回落 appendChild，终态同旧路径；
- ref=提取对象自身的后邻几何（sn1=span 且非末子）：ref 从 null（尾 append）
  变为真实后邻位——join 三案 7,518 行零翻转实证该形态不在语料面或终态一致。

## 5. 下一步

join 三案全扫收口。残余按语料账重排：① inline-host 800F 残余簇（attr 门
已采样：跨子测文本渗漏 + D1 族 text-view 生命周期）；② white-space-style
488F（italic/bold temporary-inline 保留组）；③ execCommand 簇序。
