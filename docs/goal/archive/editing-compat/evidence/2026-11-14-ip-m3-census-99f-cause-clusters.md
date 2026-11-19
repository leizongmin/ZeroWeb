# ip 残 99F 按因普查——M3 选面定谳（A 根层裸行内拆分）

- 日期：2026-11-14（gate v3 终判日志 `ipm2-v3-ip.log` 全量归因，census 预警「签名聚类只到形不分因」的按因重普查）
- 数据源：`.acceptance/dps/ipm2-v3-ip.log`（gate v3 @6c8457a36 bin bf718734，99F=91 compare innerHTML＋8 qV-dPS 读面）；99 条 Fail 的 got/exp 逐条解析（assert_equals 47＋assert_in_array 44 接受形数组按 br-Δ 最近形对齐），零未解析。
- 工具：`.acceptance/dps/census-m3-cabinet.py`（解析+机械分簇）＋ `census-m3-final.py`（按因族标定）；产出 `ipm2-v3-fail-entries.json` / `ipm2-v3-census-families.json`。

## 机械分簇（br 计数差）

| 簇 | 条数 |
|---|---|
| structure-only（br 数相等，结构差） | 66（34 inarray＋32 assert_equals） |
| missing-br（got−1） | 15 |
| inarray br-Δ(−1) | 10 |
| qV-dPS 读面 | 8 |

零 extra-br（got 无多垫 br）——M2-B pad 免垫后过垫面已清。

## 按因族（13 族全归因，99 条闭合）

| 族 | 条/案 | 机制（got vs exp 定谳） |
|---|---|---|
| **A 根层裸行内拆分** | **22/11**（×dPS div/p） | 可编辑根层裸行内内容 ip：got 左右两段都包块（`<div>foo<br>bar</div><div>…</div>`）；exp **左段留根层不包块**（`foo<br>bar<div><br><span>baz</span></div>`、`foo<br><br><div>bar…`），仅右段落块。11 案 got/exp 形全一致，单机制。 |
| C4 块边界空块垫缺失 | 17/10 | 块边界拆分右段空块该垫 br 未垫：表跨选区（`fo[o<table…b]ar` 4）、块后 bare-br（`</p>{}<br>` 8）、整块选 h1 后 p 嵌套违规（`{<h1>bar</h1>}` got `<p><p>baz</p></p>` 2）、dt 空块无垫、blockquote 内 `<p>foo[]<p>` 不插空块、li 尾不劈新 li。含 **p 内容模型违规子族**（got 块嵌块合法化未执行）。 |
| B 拆点空白/nbsp 归一 | 15/15 | 拆点处折叠空格未按块边界规则转 `&nbsp;`（`a[] b`→exp `a&nbsp;`/`&nbsp;b`；`abc[] `→exp 右块退化 `<div><br></div>`）。与 2026-10-30 white-space join 域同源不同位（join 管填充物化，此管拆分边界归一）。 |
| C1 容器尾拆分空块序反转 | 8/4 | td/blockquote 尾拆分：got 空块落内容块前（`<td><div><br></div><div>fo</div>`），exp 应在后（`<td><div>fo</div><div><br></div>`）。 |
| R1 qV-dPS 读面（挂账不动） | 8/7 | queryCommandValue("defaultparagraphseparator") 读值缺陷，M1 前在册。 |
| C2 左空块壳携垫缺失 | 8/4 | `<p><b>[]foo</b></p>`：exp 左空块携壳垫 `<p><b><br></b></p>`，got 裸 `<p><br></p>`。M2-C 只下钻重建链右根，左段（extract 侧）壳携垫缺失。 |
| F 右空块 a 壳过携 | 6/6 | `<a href=foo>foo&nbsp;[]<br></a>`：exp 右空块弃 a 壳（`<p><br></p>`），got 携 a（`<p><a><br></a></p>`）。 |
| C5 根层裸 br 形 | 6/3 | `{<table>…}` 整选 / `[abc]` 整选 / `{}<br><span display:block>`：exp 根层裸 br＋块形（`<br><div><br></div>`），got 双块或嵌套违规形。Chrome oracle 本身产根层裸 br。 |
| C6 script rawtext 拆分 | 2/1 | `<script>foo[]bar</script>`：exp 拆进 rawtext（双 script 元素），got 合并右移。 |
| C3 注释节点克隆 | 2/1 | `foo[]<!--bar-->`：got 克隆空注释（`foo<!---->`），exp 注释整体留右段。 |
| I6 styled span 右空块半携 | 2/2 | color+bg 双 span：exp 空右块携**全壳链**，got 只携外层（内层 bg span 丢）。 |
| I4 CE 列表项块化 | 2/2 | `contenteditable=false` ol/dl 内 `contenteditable` li/dd：exp li 内容块化（`<div>ab</div><div>cd<br></div>`），got 留行内 `ab<br>cd`。 |
| I5 pre 换行归一 | 1/1 | `<pre>foo&#10;[]</pre>`：got 保留换行文本＋双 br，exp 接受形换行归一为 br 形。 |

**方向性壳携语义（C2/F/I6 合并洞察）**：空 continuation 块的壳携是方向与类型双敏——样式壳（b/i/span 链）应携入且携全链（C2 左侧缺失、I6 链不全），链接壳（a）应弃（F 右侧过携）。单一「壳携策略」修可覆盖三族 16 条。

## M3 起刀面定谳

**A 族（22 键，最大单机制）**：根层裸行内拆分，part01f.js ip 拆分 host 边界路径——左段不包块留可编辑根层，仅右段落块。预期 F 99→77；C4 若随拆分边界路径同修可再收表跨 4 条（不强求，刀域以 A 为界）。B 族（15）与 white-space join 域交叠，择期独立刀并先核 join 域回归面。

## 方法注记

- assert_in_array 条目按 br-Δ 最近接受形对齐归因；全部 99 条 got/exp 已存 `ipm2-v3-fail-entries.json` 供复核。
- 族标定规则与正则边界见 `census-m3-final.py`；UNTAGGED 3 条人工归并（b>a 左壳 2→C2、dd CE 1→I4）后零未定。
