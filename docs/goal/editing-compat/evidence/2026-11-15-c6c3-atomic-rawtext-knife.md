# ip 非文本原子拆分边界刀——C3 注释原子 + C6 script rawtext（4 键域）

- 日期：2026-11-15
- 基线：gate 批 v3 ALL-DONE @f28e07366 bin 7a7f9657，ip 残 20F（族图见 [census](2026-11-15-ip-residual-census-20f.md)）
- 刀域：C3＋C6 共 4 键 2 形（`<script>foo[]bar</script>baz` dPS div/p ＋ `<p>foo[]<!--bar-->` dPS div/p）

## 根因

### C3 注释原子（corpus `<p>foo[]<!--bar-->`，got `<p>foo<!----></p><p><!--bar--><br></p>`）

`_zwIparaExtractRange`（part01f.js）sc===ec snap 臂与 contained 臂对边界 CharacterData 走
cloneSlice＋deleteData 切片语义；Comment/PI 虽属 `isCd`（nodeType 8/7 被判字符数据面），
但切片语义对原子节点 = `cloneSlice(comment, 0)` 产空克隆入左段 + 原节点残留
`<!---->` 空壳（got 实录左段空注释即此）。spec 语义：非 CharData 节点在 contained
面走 remove＋append 同型整移，不切片。

### C6 script rawtext（corpus `<script>foo[]bar</script>baz`，got `<div><script>foobar</script>baz</div><div><br></div>` 五步链）

1. caret 在宿主直子 script 壳内文本 → climb 停壳（`_zwJusIsEditable(host)=false`，
   part01c.js:348 壳即 host 直子链几何）；
2. M1 门（part01f.js gate walk）container tag 表只有 a/fmt 壳，script 被拒 → 落 wrap 面；
3. wrap nodeList 含宿主直子文本成员（`baz`）→ `_zwJusWrapViaHtml` 整树重建
   （host.innerHTML 赋值，part01c.js:931-1046）；
4. 拆分位引用脱挂 → c1InDom walk（C1 刀）兜底 `setStart(container, len)` 退化空拆分；
5. 全文留左块 + 空垫块（got 实录）。

## 修复（part01f.js 双臂）

- **C3**：extract 两分支边界判改 `isSplittableCd`（仅 Text/CDATA nodeType 3/4）；
  Comment/PI 落先摘后挂 move 臂（spec contained 子 remove＋append 同型）。
- **C6**：M1 门 container tag 表入 `script`（gate 直构，与 b/span 壳同径）；
  壳链循环克隆改 `ipCloneShellC` 辅助——script 元素走
  `createElement('script')`＋业务 attr 平移（setAttribute/attr 迭代 wire 齐
  备），其余壳 `cloneNode(false)` 原径。script `display:none`（UA 面）由既有
  `_zwJusIsVisible` 不可见门垫左块 br，exp `<script>foo</script><br>` 形成立。

### 探针定谳（游离 probe，验后即删）

首版仅扩门后 script 钉全空转（host 原样）：probe 实录 shim script 元素包装
**缺 `cloneNode`**（`typeof s.cloneNode === 'undefined'`）——gate 直构体
`ipE.cloneNode(false)` 抛 TypeError 静默中止整命令。同 probe 证
`appendChild`/`removeChild`/`setAttribute`/`hasAttribute`/`getAttribute`/attr
索引迭代（`.name`/`.value`）在 wire 全可用，且 `createElement('script')`＋
`appendChild(createTextNode)` 产物经 wired innerHTML round-trip 恰出
`<div><script>foo</script><br></div><div><script>bar</script>baz</div>`
exp 唯一形——`ipCloneShellC` 即按此落地。flex/grid 臂（同文件第二克隆位）
gate walk 拒非 a/fmt 壳，script 不入，未动。

## 钉测（part120_ip_atomic_rawtext_pins，6 钉）

| # | 钉 | exp |
|---|---|---|
| ① | c6_script_mid_div（corpus :390-394） | `<div><script>foo</script><br></div><div><script>bar</script>baz</div>` |
| ② | c6_script_mid_p（:395-399） | `<p><script>foo</script><br></p><p><script>bar</script>baz</p>` |
| ③ | c3_comment_atomic_div（:1250-1254） | `<p>foo</p><p><!--bar--><br></p>` |
| ④ | c3_comment_atomic_p（:1255-1259） | `<p>foo</p><p><!--bar--><br></p>` |
| ⑤ | ctrl b 壳中点（:763-767 M1 门绿键） | `<div><b>foo</b></div><div><b>bar</b>baz</div>` |
| ⑥ | ctrl 注释前界（:1260-1264） | `<p><!--foo--><br></p><p>bar</p>` |

## 验证

- 钉 6/6 GREEN（`cargo test -p zero-engine --lib insertparagraph_c` 35/35 含旧面）。
- enginelib 全量 **4032P/0F**（4022 基线＋g47 双刀 4＋本刀 6，名册零新增）。
- 安全面静态核：multitest.js 零 script-caret 形（mt 腿无涉）；insertlinebreak/
  delete 不走 ip 门；qV-dPS script-div before 行属挂账 qV-8 族（读面）本刀不动。
- 门禁批：五腿 fresh（ip/ab/mt/enginelib/ilb）judge vs v3 20F 基线，
  预期 F2P=4 恰等（4 compare 键）＋P2F/lost/newF=0＋residF=16 自洽——结果见控制面。
