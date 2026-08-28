# R335 — sel 域 parsed Text splitText（2026-08-28）

## 问题

R334 后 `MutationObserver-childList.html` 仅剩 1 pending：n91 `Range.insertNode children
insertion mutation`。该用例的 range 跨 parsed 首文本（"CHAE"）与 handle 尾文本（"D"），
`insertNode(f91)` 走 part06 R209 splitText 分支时，`_wrapNodeEntry`（R48，sel 域 parsed
文本包装）**没有 splitText 方法**——`typeof sc209.splitText === 'function'` 判定失败，
split 步骤静默跳过，尾节点 childList record 永不派发（WPT 期望 2 条 added record：
split 尾节点 + 插入的 f91）。

## 修复（part05.js `_wrapNodeEntry`）

`node.splitText(offset)`（spec `dom-text-splittext`）：

- 越界（offset < 0 或 > length）抛 IndexSizeError DOMException；
- 原节点保 `[0, offset)`（经 `_write` → host `SetChildText`，记录语义与 R48 一致），
  新 Text 含 `[offset, length)`；
- 新节点经**父的 insertBefore**（原节点 nextSibling 位，ref null 时 append）走标准 wire
  与 JS 记账，保证 host 树与融合视图同步；
- 返回新 Text 节点。后续 insertBefore/append 路径自然发出 WPT 期望的双 added record。

## 验证

- `MutationObserver-childList.html`：**38/38 全 Pass，文件级 Timeout 消除**
  （自 R37 导入上游用例以来首次全绿；此前 R301 曾到 25P/2F，R332 前 27P/13 pending）
- MO 全族 130P/4F/3T（fail 集与既存备档恒等：cross-realm 1F + document parser 3F；
  Timeout 4→3 即 childList 文件级消除）
- Range-insertNode 1841P、Text-splitText 2P（splitText 消费面零回归）
- engine v8 2473 / quickjs 1467 绿；tab 38P + renderer R2929/R2930 绿
- fmt/clippy 双矩阵干净

## 教训

1. 「A 域有、B 域没有」的方法面缺口（handle 域有 splitText、sel 域没有）是 shim 双源
   架构的持续风险——新增依赖某方法的调用点时须核对两域覆盖。
2. R209 的 `typeof === 'function'` 判定让缺口静默降级为「无 split 也继续」——降级路径
   本身是正确的（非 Text 容器走同分支），但 record 缺失要等 MO 观察面才暴露。
