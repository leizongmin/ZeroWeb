# ip 残 20F 按因普查——C1 批 v3 后族图闭合

- 日期：2026-11-15
- 数据源：`.acceptance/dps/ipm3-c1-v3-ip.log`（gate 批 v3 @f28e07366 bin 7a7f9657，ip 腿 fresh 7175P/20F/0T wall 880s）；20 条 Fail 的 got/exp 逐条解析，与 C1 批前预测（控制面 2026-11-15 行）**精确闭合，零漂移键**。
- 判读对照：judge F2P=8（bq 4＋td 4）恰 C1 刀域，P2F=0/lost=0/newF=0——翻转面零外溢。

## 族图（12 compare＋8 qV）

### qV-dPS 读面 8（挂账，域决策维持）

`queryCommandValue("defaultparagraphseparator")` before/after 断言，签名 `expected "p" but got "div"`（我们回放 execCommand 设置值，Chrome oracle 恒回默认 "p"——dPS 写面生效但查询面不反映，Chromium 特有读值语义）。分布：script dPS-div before、blockquote dPS-div before、styled span dPS-p before/after＋{bar} 形 before、ul>li>div dPS-p after、div>p dPS-div before、div class dPS-div before。M1 前在册，**认领前需域决策**（匹配 Chromium 写不反映 quirk vs 维持设置值语义），本普查不动。

### compare 面 12（六族）

| 族 | 键/形 | got → exp（v3 log 实录） |
|---|---|---|
| **C4 越界 3**（容器链） | 3/3 | ①`<dl><dt>foo<dd>bar<dl><dt>{}<br><dd>baz</dl></dl>`：got `<dt></dt>`→exp `<dt><br></dt>`（空 dt 无垫，拆分已发生仅缺 pad）；②`<blockquote><p>foo[]<p>bar</blockquote>` dPS div：got `<p>foo</p><p>bar</p>`（**命令 no-op**，未插空块）→exp `<p>foo</p><p><br></p><p>bar</p>`；③`<ul><li><div>foo[]</ul>` dPS p：got `<li><div>foo</div></li>`（**no-op**）→exp `<li><div>foo</div></li><li><div><br></div></li>`。 |
| **C5 根层裸 br 形** | 2/1 | `<div>[abc]</div>` dPS div/p 双面：got `<div><br></div><div><br></div>`（左空块包块）→exp `<br><div><br></div>`（左退化根层裸 br）。M3-A 左段留根层语义的空左段特形。 |
| **C6 script rawtext 拆分** | 2/1 | `<script>foo[]bar</script>baz` dPS div/p：got `<div><script>foobar</script>baz</div><div><br></div>`（rawtext 文本不拆、整体左移）→exp `<div><script>foo</script><br></div><div><script>bar</script>baz</div>`（拆进 rawtext 双 script 元素）。 |
| **C3 注释克隆** | 2/1 | `<p>foo[]<!--bar-->` dPS div/p：got `<p>foo<!----></p><p><!--bar--><br></p>`（左段克隆出空注释）→exp `<p>foo</p><p><!--bar--><br></p>`（注释原子整体留右段）。 |
| **I4 CE 列表项块化** | 2/2 | `<ol contenteditable=false><li contenteditable>ab[]cd</li></ol>`＋dl/dd 同形：got `ab<br>cd` 行内（**no-op**）→exp `<div>ab</div><div>cd</div>`（接受形含 `cd<br>` 变体）。CE=false 祖先下 CE li 内写面被门禁吞。 |
| **I5 pre 换行归一** | 1/1 | `<pre>foo&#10;[]</pre>`：got `<pre>foo\n<br><br></pre>`（换行文本＋双 br）→exp 接受形 `<pre>foo\n<br></pre>`（换行留＋单 br）。 |

## 机制分组与刀选

- **非文本原子拆分边界（C3＋C6，4 键 2 形）**：同一根域——ip 拆分边界对非文本节点（comment 原子节点、script rawtext 元素）的处理：C3 克隆臂对 comment 产空克隆（按 character data 切片语义切 comment 本体 offset 0）；C6 文本拆分臂拒绝拆 rawtext 元素内文本（整体并入左段）。单刀双臂可覆盖。
- **容器链垫/劈裂（C4 越界 3）**：三形异质（1 缺垫＋2 no-op），no-op 面需先探针定位命令吞没位（blockquote/ul 容器链内新块创建路径），独立刀。
- **根层空左段退化（C5 2）**：M3-A 左段不包块臂的空段特形，小刀。
- **I4（2）**：跨 CE=false 边界写面，域敏感（CE 边界纪律），缓。
- **I5（1）**：pre 上下文单 br 垫，小刀随域顺带。

## 方法注记

- 20 条 Fail 逐条对照 `.acceptance/dps/ipm3-c1-v3-ip.log`（assert_equals 12＋assert_in_array 2＋qV 8 中 assert_equals 6——实录见上文）；本档为人工逐条解析，未走 census-m3 脚本（键集小，机械分簇无增益）。
