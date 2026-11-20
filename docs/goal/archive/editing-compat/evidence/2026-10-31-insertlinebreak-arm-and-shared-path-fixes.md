# R5371：insertlinebreak execCommand 臂接入 + 共享路径双修——ilb 0P/552F→528P/24F，wss 同步 616P→848P/256F

**日期**: 2026-10-31
**前置**: [R5356b frag 种子与 live-list 同步](2026-10-31-r5356b-frag-seed-and-live-list.md)
（inline-host 880P/0F 全扫；ilb 0P/552F 为 M1 基线已知 deferred 簇）

## 0. 终账

| 语料 | 结果 | 对照 |
|---|---|---|
| insertlinebreak-with-white-space-style（ilb） | **528P/24F** | 0P/552F 起，主体清零 |
| insertparagraph-with-white-space-style（wss） | **848P/256F** | 616P/488F 起，+232 零回归 |
| inline-host | 880P/0F | 零位移（两轮复验） |
| join 三案 | 7,518P/0F | 零位移 |
| cannot-have-div | 420P/0F | 零位移 |
| engine lib | 3,616P/10F | 已知红集逐条 identical（fetch/Request 族 8 + iframe SW + s24） |
| webview lib | 830P/0F | 恒等 |

修复单提交 c81777413（part01 + part06，+35/-7），无插桩、净树全门禁。

## 1. 缺陷机制：execCommand 臂零变更（ilb 全灭根因）

`insertlinebreak` 在 `_zwExecCmdInputType` 映射表（part06）有
`insertLineBreak` inputType——R3254-M2 通用编辑命令臂对它照派
beforeinput/input 事件，但**实应用分派只有 format/delete/insertparagraph/
inserthtml 四族，无 insertlinebreak 臂** → 事件照派、零 DOM 变更，与失败签名
`expected "abc<br><br>" but got "abc"`（完全未插入）精确吻合。语料形态：
块级 `<div contenteditable>` + per-test `style.whiteSpace`（inline 面板）+
`execCommand("insertlinebreak")` 直驱，期望 ws-significant → `abc\n\n`/
`abc\n<br>` 数组，否则 `abc<br><br>`。

既有 `__zw_ce_enter`（键盘/selector 签名，runner testharness Enter 路径）
不能直接复用：自带完整事件序（beforeinput+textInput+input），接进
execCommand 臂会双派事件。

## 2. 修复三笔（c81777413）

1. **臂接入 + 行断模式**（part06 臂 + part01 核心）：`_zwCeParagraphMutate`
   增 `lineBreakOnly` 参数（跳过 ①宿主拆分/②同型拆分，直达 ③ 行断退化），
   白空间显著性（`_zwCeWsSignificant` 声明链）与可见性补断语义全量复用
   `_zwCeLineBreakAt`；新全局桥 `__zw_ce_line_break_mutate(host, range)`
   仅变更（事件口径同 insertparagraph 臂——beforeinput 已由通用路径派发）。
2. **`__zw_ce_insert` 元素边界 offset 位插入**（part01）：caret 容器为元素
   且两侧无邻接文本子时，旧版 `appendChild` 尾追使文本跳到元素尾——ilb
   现有 inline 样式组（`<b>abc<br>[]<br></b>` 位插 'def'，got
   `<b>abc<br><br>def</b>`）与 wss insertText 变体（+8）实证。改
   `insertBefore(node, kids[idx] || null)`；空宿主/caret 在尾的既有形态
   `kids[idx]` 为空 → 行为不变。
3. **同型拆分头段全切摘除空文本节点**（part01 ②）：caret 在内容起点
   （`[]abc`）时头段 `nodeValue=''` 滞留，`childNodes.length` 检查误判非空
   跳过「拆空侧 `<br>` 填充」→ 新块占位断缺失（wss got
   `<p></p><p>abc</p>`，期望 `<p><br></p><p>abc</p>`）。摘除后 wss
   起始占位族 +224 全清。

## 3. 探针/验证要点

- 臂缺失定位：`grep "cmd === "` 枚举 execCommand 分派臂——4165 copy 族/
  4192 text-control insert 族/4256 CE inserttext/4276 通用臂（内含
  format/delete/**insertparagraph**/inserthtml 四实应用点，无
  insertlinebreak）+ `insertlinebreak` 全文仅 `_zwExecCmdInputType` 表命中。
- wss +232 的两族增量分别由修复 2（+8，insertText 变体）与修复 3
  （+224，起始占位族）解释，无来源不明翻转。
- ilb 期望形态逐组核对：直子 `abc[]`/`[]abc`/`a[]bc` 与嵌套
  `<p style=ws>` 形全由 `_zwCeLineBreakAt` 输出恰合（含可见性第二断只在
  断点后无后继时补）。

## 4. 残余（两族，均属原 wss 488F 挂账面）

- **typing-state 族（ilb 24F + wss ~160F）**：`execCommand("italic"/"bold")`
  collapsed caret 的 typing state 不追踪，后续 insertText 不带 `<i>`/`<b>`
  包裹（含跨行断/跨拆分保留）。与 insertparagraph 488F 的 FIXME 同族
  （part01 9018 FIXME 留置）。
- **inline 盒 wrap 族（wss ~96F）**：caret 在 `display:inline/inline-block`
  样式段落内 insertParagraph，Chromium 把 inline 盒包进新匿名块
  （`<div><inline-div>…</div></div>` 形）；本引擎 ② 判非 blockish 落 ③
  行断退化（got `abc<br><br>` 在原盒内）。需独立的 wrap 拆分模型。
