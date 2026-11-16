# ihc1 残 49F 全集普查（旁观轮，2026-11-16）

来源：`.acceptance/dps/ihc1-iht-a.log` / `ihc1-iht-b.log`（批 ihc1，锚 `8fb4e955d`，bin `cec0d8bde4ba1b6bdd59fc17848c26a2`，iht A/B 双腿各 1565 行 1516P/49F）。解析器按 `Fail <suite> :: <setup> <fixture>` 案级去重 + 首条 assert 明细提取，A/B 双腿各得 49 案恒等。原始 JSON：`/tmp/census39/fails.json`（临时工件，不入库）。

## 已认账桶核验（10 案）

- **span color 壳行内 ×8**＝案 1-8：形 `foo<span style=color:#aBcDeF>{bar}</span>baz` 等四选区形 × swcss 双态，payload `ab<b>c</b>d`。**与在飞轴二元素端点刀 part124 新增 8 钉（ihb-elm-in/around/cross-left/cross-right × true/false）选区形、swcss 态、期望值逐字恒等 8/8**（钉期望 `fooab<b>c</b>dbaz` / `ab<b>c</b>dbaz` / `fooab<b>c</b>d` ＝ corpus expected 串原样）——旁观核验通过，该刀钉测 corpus 忠实性无异议。
- **CE-false pre ×1**＝案 34（`<pre contenteditable="false"><span contenteditable>[1234]</span></pre>`，assert_in_array）。
- **multi-root div ×1**＝案 37（body root `{}` 插双 div 结构，got `""`）。

## 残 ×39 七族普查（案号按 iht-a 顺序）

| 族 | 案 | 数 | 签名（expected → got） |
|---|---|---|---|
| G. ws-pre 尾换行 caret 插载荷丢失 | 40-49 | 10 | `abc[]\n` / `abc []\n` / `[]\n` × ws:pre / pre-line，payload `X` 或 `X`+`Y`：exp 载荷落换行前（`abcX\n`）→ got 载荷整丢、结构不变（`abc\n`）。**文本端点（`[]` 在文本节点内），在飞元素端点臂不覆盖** |
| D. `{}` 元素边界坍缩选区载荷丢失/注释插入 | 22-27, 29, 30-33 | 12 | `<p>{}<br></p>` / `<p><!--foo-->{}<span><br></span>…` 形：payload `abc` → exp 占位 br 删除落文本（`<p>abc</p>`），got br 保留载荷丢；payload `<!--abc-->` → exp 注释落 caret 位，got 整丢。**机制可能与在飞元素端点归一臂部分重叠（{} 端点同为元素边界形），刀后需复普查确认覆盖面** |
| C. 全段选区 × 隐形载荷块壳丢失 | 18-21 | 4 | `<p>[foo]</p>` 插 `" "` / `<!--abc-->` / `<span style=display:none></span>`：exp 载荷落 p 内＋垫 br（`<p> <br></p>` 等）→ got 块壳整体被载荷替换（got 顶层 `" "` / `<!--abc-->`） |
| B. rawtext 壳内载荷解析 | 14-17 | 4 | `<xmp>f[o]o</xmp>` / `<script>f[o]o</script>bar` 插 `<b>abc</b>`：exp 元素化（`f<b>abc</b>o`）→ got 转义文本（`f&lt;b&gt;…`）。现役壳拼接臂保壳字面拼接语义与 corpus 元素化期望相反 |
| A. 载荷未闭合/退化壳吞并后续 | 9-13 | 5 | `""`→exp 仅删选区 `foobaz`，got 原文不动；`<b>` 开标签→exp 空 `<b></b>` 落位，got 后随文本被吞 `foo<b>baz</b>`；`<b>abc` 未闭合→exp 壳在载荷尾闭合 `foo<b>abc</b>baz`，got 吞并 baz `foo<b>abcbaz</b>` |
| E. 空行内壳载荷丢弃 | 35-36 | 2 | 插 `<span></span>` / `<span s1></span>b<span s2></span>`：exp 空壳原位保留（±载荷文本 b）→ got 空壳整丢，案 36 连载荷文本 b 一并丢（`ac`） |
| F. CE-false 邻接选区插入点外迁 | 38-39 | 2 | `<p><b>[X]<span ce=false>abc</span></b>…` 插 `<i>Z</i>`：exp 载荷落 <b> 外侧（`<i>Z</i><b>…`）→ got 载荷丢 X 原位保持；[Y] 后随形对称 |

计数 10+12+4+4+5+2+2＝39 ✓。

## 判读与下一步建议

1. **G 族 ×10 为最大独立族且文本端点**，与在飞刀机制正交，是轴二刀后的首选下一刀域；与 ip I5 pre 换行归一刀（cfd1a06af）同处 ws-pre 文本域，刀前需核对该刀垫底臂是否已部分改善此面（i5p1 批时点早于 ihc1，本普查日志即现状，无改善——I5 修的是 br 垫底，本族是载荷文本落位）。
2. **D 族 ×12 待在飞轴二刀落地后复普查**：若元素端点归一臂顺带修复部分 {} 形，残量收缩后再定刀界；注释载荷（`<!--abc-->` 6 案）与文本载荷（`abc` 6 案）各半。
3. C 族（块壳丢失 ×4）机制上疑似插入点解析把全段选区当块替换处理，与轴一块级臂（4f3469026）域相邻，刀前先核块臂旁路门。
4. B/A/E/F 族各 2-5 案，量小，适合攒批或搭车。

本普查为读侧分析（零码面改动），供轴二刀收口轮复普查与下一刀选面使用。
