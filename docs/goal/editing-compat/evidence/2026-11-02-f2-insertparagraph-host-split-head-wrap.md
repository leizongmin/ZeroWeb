# F2 insertParagraph host-level split 前段包裹（① 臂双块输出）

日期：2026-11-02（editing 流，控制面下一步 ①，基于 g15 基线对账）

## 0. 背景与范围

普查轮定谳 F2：ipara 465F compare 主刀单机制——**host-level split 前段不包裹**
（`foo[bar]baz` got `foo<div>baz</div>` vs expected `<div>foo</div><div>baz</div>`；
`[]foo` got `<div><br></div>foo` vs expected `<div><br></div><div>foo</div>`）。
gate15 ipara 475F 重聚类：exp-wrapped-host 146（前段包裹 + 空段占位形）+
other-cmp 153（table/dl/blockquote 嵌套形 + `p→div` 断言面 + list 内段落 split
形）+ non-compare 174（in_array 63 多形态允许值 + equals 111 读面）。

## 1. 归因：Chrome delta 非 vendored 语义缺口

vendored insertParagraph（:7486）host 直子文本 caret 路径：splitText 后前段
text 留宿主直子**裸层**（extractContents 只摘 newLineRange 尾段）——vendored
输出即 `foo<div>baz</div>`。语料是 Chrome 行为录制（conformancerecording），
caret 行整体成块（前段也包 separator）——**Chrome delta 修复面**，同 blockExtend
两档（justify 簇先例）。

## 2. 实现（part01.js ① 臂一笔）

`__zw_ce_paragraph_split_at` ① 臂（宿主级拆分）boundaryText 路径重写双块输出：
- 前块（sepTag）= boundaryText 前兄弟（逆序 insertBefore）+ head 段（v1.slice
  (0, boundaryOff)）；空则占位 `<br>`（`__zwPlaceholderBr` 打标同款）。
- 后块（sep1）= tail 段（v1.slice(eo)）+ boundaryText 后兄弟；空则占位 `<br>`。
- boundaryText 摘除；caret → 后块内容起点。
- 宿主元素边界路径（boundaryText null，(host, so) 形）不动（其面已过：前子为
  块级元素形）；② 臂（块级后代同型拆分）不动（wss-ip 全绿面）。

multitest D2 重查：实为 typing×split 交互形（b 壳内夹 `<br>` caret 位
`G: <div>abc<b><br>d<br></b></div>` vs expected `<div>abc</div><div><b>d</b>
</div>`）——非纯 ① 臂面，挂账（typing 包裹 caret 夹位语义）。

## 3. 探针（zz-probe-ipara.html，gitignore）

六案逐字命中语料期望：`foo[]bar` → `<div>foo</div><div>bar</div>`；`foo[bar]
baz` 选区 → `<div>foo</div><div>baz</div>`；`[]foo` → `<div><br></div><div>foo
</div>`；`foo[]` → `<div>foo</div><div><br></div>`；块内形（② 臂恒等）`<div>
foo[]bar</div>` → `<div>foo</div><div>bar</div>`；p separator → `<p>foo</p><p>
bar</p>`。

## 4. 对账（gate16，systemd user unit 串行批）

gate16 串行批（systemd user unit + test-guard，2026-11-02 02:07–03:11，
transcript `/tmp/zw-edit-v/g16-*.log`，状态 `/tmp/zw-edit-v/g16-status.log`）：

| 语料 | g15/g13 基线 | gate16 | 位移 |
|---|---|---|---|
| run/insertparagraph | 6,720P/475F | 6,728P/467F | **F→P 8**，P→F 0 |
| run/inserttext | 2,777P/232F | 2,779P/230F | **F→P 2**，P→F 0 |
| run/multitest | 9,538P/554F | 9,538P/554F | 恒等 |
| other/wss-insertparagraph | 1,104P/0F | 1,104P/0F | 恒等 |
| other/wss-insertlinebreak | 552P/0F | 552P/0F | 恒等 |
| run/insertlinebreak | 1,641P/50F | 1,641P/50F | 恒等 |
| other/join-paragraphs | 3,456P/0F | 3,456P/0F | 恒等 |
| other/join-lrp（左行右段） | 2,607P/0F | 2,607P/0F | 恒等 |
| other/join-lpr（左段右行） | 1,455P/0F | 1,455P/0F | 恒等 |
| other/ip-host-cannot-have-div | 420P/0F | 420P/0F | 恒等 |
| other/ip-in-inline-host | 880P/0F | 880P/0F | 恒等 |
| other/ip-void / it-void | 72P/0F、48P/0F | 同 | 恒等 |
| run/forecolor | 2,279P/48F | 2,279P/48F | 恒等 |
| run/insertorderedlist | 1,619P/142F | 1,619P/142F | 恒等（F1 面保持） |
| run/insertunorderedlist | 1,910P/155F | 1,910P/155F | 恒等（F1 面保持） |
| engine lib | 3,683P/0F | rc=101 → 修正钉测后 3,684P/0F | 钉测同步更新 |

- ipara F→P 8 逐案核对：恰为裸文本 host 直子面 8 案（`foo[]bar`/`foo[bar]baz`/
  `[]foo`/`foo[]` 形及其变体），与 §2 改造面一致；P→F 零位移。
- engine lib 批内 rc=101 为 `test_insert_paragraph_split_r3254_m3_slice5` 锁旧
  SetChildText 序——钉测按 F2 双块语义更新（CreateElement×2 + CreateTextNode
  head/tail + RemoveChildAt 摘旧文本 + AppendChild），批外重跑 3,684P/0F。

## 5. 残余与挂账

- ipara other-cmp 153：table 跨块选区入格形（`fo[o<table>…<td>b]ar` 空段占位
  br）、dl/dt/dd 嵌套形、blockquote 形、list 内段落 split 形（嵌套轮）。
- multitest D2 typing×split 交互形（b 壳 caret 夹位 + 裸 `<br>` 占位形）。
- ipara non-compare 174（in_array 63 + equals 111 读面）。
- F6 delete 撤壳 → F5 尾 br 边界 → qV 残余 → list FIXME（控制面序不变）。
