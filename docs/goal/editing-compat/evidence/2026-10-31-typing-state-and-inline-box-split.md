# R5392：typing-state 追踪 + div 宿主 inline 盒拆分 + 占位 br 消费——ilb 528P/24F→552P/0F 全扫，wss 848P→912P/192F

**日期**: 2026-10-31
**前置**: [R5371 insertlinebreak 臂接入](2026-10-31-insertlinebreak-arm-and-shared-path-fixes.md)
（ilb 528P/24F，残余 typing-state 族 24F + wss wrap 族）

## 0. 终账

| 语料 | 结果 | 对照 |
|---|---|---|
| insertlinebreak-with-white-space-style（ilb） | **552P/0F 全扫** | 528P/24F，+24 全清 |
| insertparagraph-with-white-space-style（wss） | **912P/192F** | 848P/256F，+64 零回归 |
| inline-host | 880P/0F | 零位移 |
| join 三案 | 7,518P/0F | 零位移 |
| cannot-have-div（chd） | 420P/0F | 零位移（中途 16F 已修，见 §3） |
| bold run 面 | 1,138P/1,910F | 对现行基线逐位恒等（format 臂 collapsed 新分支零影响） |
| engine lib | 3,617P/10F | 已知红集逐条 identical（fetch/Request 族 8 + iframe SW + s24）；+1P 为并行流 s175 新测试 |
| webview lib | 830P/0F | 恒等 |

提交 14045bc7c（part01 + part06，+151/-10），rebase 到并行流 aa4e9d190
（webcrypto s175，part05 unique_selector——文件级零重叠）之上；lib 复验在
合并基座完成，语料账出自合并前同内容 js（part01/part06 rebase 零改动）。

## 1. typing-state 机制（draft execCommand "state of the command"）

collapsed caret 的 format 命令（italic/bold）不改 DOM，只切换
`globalThis.__zwTypingTags` 数组；后续 `__zw_ce_insert`（inserttext）按
记录嵌套包裹插入文本。设计三要点：

1. **状态存储**：`__zwTypingTags`（string 数组，undefined/空 = 无）。
   format collapsed 臂（part06）：`_zwQueryFormatState(cmd)` 为真时从数组
   splice 掉同标签（二次 toggling），为假且不在数组时 push。
2. **清除点单点化**：`_zwSync` 入口（11 个调用点全为页面可见 Selection
   方法——`removeAllRanges`/`addRange`/`collapse` 等）置 null。语料
   setupEditingHost 每 case 走 removeAllRanges+addRange，天然防跨子测泄漏。
   内部 typing 重锚走 `_getSelection()._ranges = [nr]` 直接赋值不经
   `_zwSync`，state 存活。
3. **消费**：`_zwCeTypingInsert(tags, text)` 由内向外嵌套 createElement
   包裹，`__zw_ce_insert` 三插入臂（text-splice / element-target /
   element-fallback）typing 分支均三段式：头段留原节点 + wrap 元素 +
   尾段新文本节点，caret 锚 wrap 内文本节点。preserving 组（无 format
   前置）state 空走原臂，输出不带包裹。

## 2. ②b inline 盒拆分与占位 br 消费

- **②b 臂**（`_zwCeParagraphMutate` ② 内非 blockish 分支最前）：宿主
  div（`hostTag === 'div'` 且 display ''/block）且 caret 在宿主直子非块
  标签元素内（如 `<b>abc[]</b>`）→ 原盒截尾段保留，新 separator 块内
  `cloneNode(false)` 复制 inline 壳包尾段（wss preserving bold 组
  `<b>abc[]</b>` → `<b>abc</b><sep><b>def</b></sep>`）。非块宿主与
  cannot-have-div 宿主仍落 ③ 行断。
- **占位 br 打标**：拆分产生的空块 `<br>` 占位四处打标
  `__zwPlaceholderBr = true`（① 臂 phbr1a/1b/1c + ②b phbr2b）。
- **占位消费**：inserttext 内容进场后，从 caret 直接父层逐层上行到块级
  或宿主，扫描移除带标 br。**不门 typing state**——preserving 组无
  format 前置同样消费（首版错误门在 typing state，preserving bold 组
  16F：got `<b>abc</b><div><b>def<br></b></div>` 反推消费与 typing 无关，
  去门后 wss 896P→912P）。

## 3. chd 16F 回归归因与守卫修复（本轮唯一反复）

②b 首版守卫只看 `dispH2b === '' || 'block'`，chd 语料
`host=p/span display:block + 内层 <span ws:normal>a[]b</span>` 形（16F）
被截胡：期望 `<span>a<br>b</span>` 行断（cannot-have-div 宿主不做块
拆分），got `<span>a</span><div><span>b</span></div>`。归因：① 臂既有
守卫是 `hostTag === 'div'`（part01:9211），②b 漏了对齐。修复 = ②b 守卫
加 `hostTag === 'div'`，复验 chd 420P/0F 恢复、ilb/wss/inline-host 保持。

## 4. 探针/验证要点

- wss 256F→192F 分族核对：temporary italic 48F（typing state）+
  preserving bold block 16F（②b + 占位消费）全清；残余 192F 断言消息
  全部为 wrap 族三形态（"latter one should be wrapped by a new `<div>`" /
  "at start" / "at end" 各 64）。
- bold run 面（1,910 案）作为 format 臂 collapsed 新分支影响面兜底：
  分母与计数对现行基线逐位恒等——collapsed 分支只切 state 不动 DOM，
  run 面无 state 消费组合。
- join 目录非基线三文件（join-pre-and-other-block 194P/232F、
  keeping-attributes 80P/132F、typing-around-link 0P/48F）失败签名全在
  Backspace/delete 合并域，不经过本轮改动点（insertParagraph 臂/
  inserttext 臂/format 臂），属既有 delete 簇挂账面，非本轮回归。

## 5. 残余

**wrap 族（wss 192F 单族）**：caret 在 `display:inline/inline-block` 样式
段落内 insertParagraph，Chromium 把 inline 盒包进新匿名块
（`<div><inline-div>…</div></div>` 形）；本引擎落 ③ 行断退化。需独立的
匿名块 wrap 拆分模型（新簇设计前按流程读 lei-spec-rfc）。
