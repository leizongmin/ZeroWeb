# d48 前置诊断：join 壳吸收深化（master ①(a) 燃点族零码变静态定谳）

日期：2026-11-17。口径：/tmp/d46-ie2.json（d46 A/B 迭代窗全语料腿，
probe 排除 1098P/2162F/5T，与 d46 终态 1135P/2193F/3T 差 37P 系 d46 后续
迭代未达终态——**记档：本诊断只取该 JSON joining-dl/dil 变体 Fail 形样本
与臂位静态追踪，计数与正式对账无关**；d46 终态账见 d46 证据）。
方法：失败键 expected/got 逐形读 + part01 join 臂静态追踪（零探针、零码变）。
当前树（d46 + 并行流 ilb 刀 3312189d7）对 delete/join 面无涉（ilb 仅
insertLineBreak dt/dd/pre caret 重映射），诊断结论对当前树有效。

## 主簇 A：join 接收容器缺壳下降（~1500F 大头）

两形同根，got 逐键复现于 d46 账：

1. **li→p 内层壳**（dil Backspace/Delete,ol/ul 变体）：`<ol><li><p>list-item1
   </p></li><li>[]list-item2</li></ol>` Backspace 期望
   `<ol><li><p>list-item1list-item2</p></li></ol>`，got
   `<ol><li><p>list-item1</p>list-item2</li></ol>`。
2. **跨 dl 容器**（joining-dl-elements Backspace/Delete）：`<dl><dt>list-item1
   </dt></dl><dl><dt>[]list-item2</dt></dl>` 期望
   `<dl><dt>list-item1list-item2</dt></dl>`，got
   `<dl><dt>list-item1</dt>list-item2</dl>`（文本落左 **dl**，壳外）。
   同族 `<ul><li>…</li></ul><dl><dt>[]…` 期望并入 li、got 落 ul
   （joining-dl-element-and-another-list）。

**根因（静态追踪）**：块界 join 臂接收容器解析
`bdst = _zwCeIsBlockEl(bp) ? bp : bc`（part01.js:12546；FD 镜像 12721
`fdst`、select-boundary 双臂 12700/12575 同构）——bp 是**块**即直接收。

- 跨 dl 形：bp = 左 dl（`_zwCeIsBlockEl` 宽表含 DL，part01.js:11619）→
  dst=dl 容器 → `_zwCeJoinMutate` 首行内容作 dst 直接子追加 → 落壳外。
  正确接收者 = dl 的**行块**（末 dt/dd）。
- li>p 形：bp = 左 li（行块）→ dst=li → 内容落 li 层。正确接收者 = li 的
  **唯一内容壳**（p）。

**下降规则草案**：dst 定后两级下降——①dst 为列表容器（ul/ol/dl）→ 降其
末行块（li/dt/dd）；②行块可见内容被唯一内层壳元素包裹 → 降入壳。守卫取
d46 直取门同款思路：**壳内须有非空活内容才降**（空壳/多子不降，落行块层），
防 d45b/d46 空白装饰族回红；纯空白文本 `/^\s*$/` 不算内容。下降只改接收
容器解析，不动 `_zwCeIsBlockTagName` 窄表（ip 拆分边界语义，3312189d7 同
教训）。

## 副簇 B1：行尾/行首 br 挡界（join no-op 族）

`Delete at "<ol><li>list-item1[]<br></li><li>list-item2</li></ol>"` 期望
`<ol><li>list-item1list-item2</li></ol>`（消费行尾 br 并入次 li），got
**no-op**（beforeinput expected 1 but got 0）。根因：FD 块界臂链上找后邻
`fnx = _zwCeSkipWs(fn.nextSibling)` 命中 br（非块）→ `fn === sc &&
!_zwCeIsBlockEl(fnx)` → return（part01.js:12720）。行尾 br（所在行块可见
末子）应视作块界界标：消费 br 后按块界 join。Backspace 镜像同查（12545
前邻方向）。

## 副簇 B2：左空摘除把空文本节点计作内容

`Delete at "<dl><dt>[list-item1</dt></dl><dl><dt>}list-item2</dt></dl>"`
期望左 dl+dt 整体剪除（`<dl><dt>list-item2</dt></dl>`），got
`<dl><dt></dt></dl><dl>…`（空 dt 壳残留）。根因：select-delete 核左无残余
路径 `!leftBlock.childNodes.length`（part01.js:12343）——摘除只截空
nodeValue 不移除文本节点 → childNodes=[空text] → 摘除门不中。修法：判空
改「无可可见内容」（空白/空文本不计），命中时连带移除空文本子再摘壳，左
侧容器剪枝复用 12326 右剪枝循环方向镜像。

## 副簇 B3：同块全选键删路径缺占位 br

`Backspace at "<ol><li>[list-item1]</li></ol>"` 期望 `<ol><li><br></li>
</ol>`，got `<ol><li></li></ol>`。根因：sc=(text,0)/ec=(text,len) 形走
`_sdFour` 文内 splice 路径（12520 带 _sdFour 门），不经过 12352
`_zwCePlaceholderBr(leftBlock)`（该行只在 `_zwCeSelectDelete` 同块臂）。
修法：文内 splice 摘空后（sc 所在行块无可可见内容）补占位 br；键入替换面
（typingData）不占位（d46 语义不变）。

## gtr 端点族（挂②不动）

`<ol><li>list-item1</li><li>[]list-item2<br>…` 期望 `(<li>,1)-(#text…,0)`
got `(text,10)-…`——跨 child-block gtr 端点规范化，master ② dwuf 余臂
持位，与 d46 记录同族，本刀不动。

## 下刀切分建议

切片 1 = A（两下降 + 守卫）预期 ~1500F 大头；切片 2 = B1；切片 3 =
B2+B3 小簇。每片独立 A/B（全语料 + html-semantics/selection 双哨），A 臂
守卫以 d45b/d46 空白装饰族 P 案为反证哨（join-different-white-space 三族
4,312P 是既有绿面，下降守卫破一面即回滚级信号）。
