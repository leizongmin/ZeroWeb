# DC-2 · editing/run 失败簇全量归类（2026-11-11）

方法：拉取 run 42947347500 全部 11 片产物 JSON，按失败消息归类。
editing/run 面合计 **3,414 Fail + 3 Timeout**（96,284 子测试）。

## 三族分布

| 族 | 数量 | 占比 | 形态 |
|---|---:|---:|---|
| queryCommand\* 断言（`Wrong result returned`） | **1,023** | 30% | queryCommandState/Indeterm/Value 期望值不匹配 |
| innerHTML/array 断言 | **2,342** | 69% | execCommand 后 DOM 输出与期望 HTML 长尾差异 |
| 其他（光标/undo/脚本错） | 49 | 1% | typing-text 光标行、undo &lt;br&gt;、4 例脚本 Runtime error |

## P1 共享路径：inline format + vertical-align 的 queryCommand 状态语义（~807）

按命令分布：bold 180 / underline 146 / strikethrough 144 / italic 138 /
subscript 100 / superscript 99；其余 defaultparagraphseparator 49、
stylewithcss 42、justify 四命令 44、零散 24。

同一共享路径：命令 **state/effective command value** 计算
（[execCommand 规范](https://w3c.github.io/editing/execCommand.html) state
定义——按 computed style 判定，非 DOM 标记形态）。当前实现在
`crates/engine/src/js_dom_shim/part06.js:5491-5560`
（queryCommandIndeterm/State/Value 三臂，state 按"选区起点所在 inline
包裹链"近似，R3254-M3 切片 4）——与规范口径的差距即本簇主因。失败上下文族：

- 字体计算值边界：`font-weight: 100..900` 数值边界、`font-style: oblique`
  计入 italic、`queryCommandIndeterm` 混合选区；
- 表格选区上下文：`<table><td>b[a]r` / `{<table>…}` 选区前后状态；
- 跨段落部分选区：`<p>[foo</p><p> <span>bar</span> </p><p>baz]</p>`；
- 不可编辑区：`<span contenteditable=false>` 相邻状态。

一条共享路径修复面 ~800 子测试，是 DC-2 最高杠杆项。

## P2 innerHTML 长尾（2,342，按命令分片开修）

片维度 er9 850 / er8 779 / er2 334 / er4 = er10 278 / er1 262 / er5 210 /
er3 206 / er7 126 / er6 89 / er0 2。命令维度大簇：inserttext 227、
outdent 173、bold 转换输出 197（379−182 queryCommand）、italic/strike/
underline 转换输出各 ~130、misc/delete 族零散。逐命令对
`Unexpected innerHTML` 期望/实际对开修，单命令杠杆低于 P1。

## P3 小簇（49）

- typing-text-in-direct-child-of-editing-host 12/16 光标行断言
  （white-space 四变体）；
- undo 后 &lt;br&gt; 移除、insertlinebreak &lt;br&gt; 位置各 ~4；
- 4 例 `Unhandled rejection Runtime error`（待看是否同根因）。
