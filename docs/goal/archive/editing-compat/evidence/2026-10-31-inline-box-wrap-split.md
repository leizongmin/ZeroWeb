# R5395：②c inline 盒 wrap 拆分——wss wrap 族 192F 全清，语料全扫 1104P/0F

**日期**: 2026-10-31
**前置**: [R5392 typing-state + ②b inline 盒拆分](2026-10-31-typing-state-and-inline-box-split.md)
（wss 912P/192F，残余全 wrap 族）

## 0. 终账

| 语料 | 结果 | 对照 |
|---|---|---|
| insertparagraph-with-white-space-style（wss） | **1104P/0F 全扫** | 912P/192F，wrap 族 +192 全清 |
| insertlinebreak-with-white-space-style（ilb） | 552P/0F | 零位移 |
| cannot-have-div（chd） | 420P/0F | 零位移 |
| inline-host | 880P/0F | 零位移 |
| join 三案 | 7,518P/0F | 零位移 |
| bold run 面 | 1,138P/1,910F | 逐位恒等 |
| engine lib | 3,617P/10F | 已知红集 md5 identical |
| webview lib | 830P/0F | 恒等 |

提交 b380a98d6（part01，+31，单分支一次过无反复）。

## 1. 规范事实（语料注释即 spec）

insertparagraph-with-white-space-style.tentative.html:388-395：inline 段落
直接拆分会「duplicate the inlined element」，Chromium 把拆分段落**包进新匿
名块**；且 **separator 恒用 `<div>`**——壳可能本身是 p 或 div（display:
inline），`<p>` 不能包含 p/div，故即使 defaultParagraphSeparator 是 p 也
用 div。期望集（assert_in_array）三种 caret 位：

- at end（`abc[]`）2 可，取最简 E1：`<inline>abc</inline><div><inline><br></inline></div>`
- at start（`[]abc`）4 可，取最简 S1：`<div><inline><br></inline></div><inline>abc</inline>`（新段落在**前**，原盒全留）
- mid（`a[]bc`）4 可，取最简 M1：`<inline>a</inline><div><inline>bc</inline></div>`

192F = 3 断言 × 64 case（display inline/inline-block × paragraph div/p ×
ws 4 值 × separator div/p × command 2 臂）。

## 2. 实现（②c 臂）

`_zwCeParagraphMutate` ② 内、②b 之后 ③ 之前：`hostBlockish2b`（hostTag
=== 'div' 且 display ''/block，与 ②b 共用）且 `_zwCeIsBlockTagName(tag2)`
且 `boundaryText`（元素 caret 形不在语料面，落 ③）：

- **at start**（`boundaryOff === 0 && eo === 0`）：新 div 块（壳副本
  `cloneNode(false)` + 占位 br 打标 `__zwPlaceholderBr`）`insertBefore`
  原盒，原盒内容不动。
- **其余**：头段留原盒（`nodeValue` 切到 boundaryOff），尾段进壳副本，
  尾空补占位 br；新块插原盒后。
- caret 重锚壳内（尾段非空锚文本节点后，否则锚 0），`_ranges` 直赋。
- 非 div 宿主仍落 ③ 行断（chd 期望 `a<br>b` 形，与 ②b 守卫同源）。

insertText `'\n'` 等价路由进同一 `_zwCeParagraphMutate`，两臂一并覆盖。

## 3. 验证要点

- wss 一次过全扫（1104 = 语料总分母），三种 caret 位输出逐一与期望集
  E1/S1/M1 吻合，无来源不明翻转。
- chd 420P/0F 保持——②c 的 hostTag==='div' 守卫与 ②b 同源（chd 的
  `host=p + 内层 div display:inline` 期望行断，非 div 宿主不触发）。
- 占位 br 沿用 R5392 打标消费链（wrap 语料无后续 inserttext 断言，打标
  与既有 ①/②b 臂一致防泄漏）。
