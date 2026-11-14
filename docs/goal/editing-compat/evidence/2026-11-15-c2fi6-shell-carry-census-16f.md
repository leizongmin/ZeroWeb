# ip C2/F/I6 方向性壳携族普查——16 键 got/exp 全解析（刀前归因铺底）

- 日期：2026-11-15；数据源：`.acceptance/dps/ipm3-b-v6-ip.log`（B 刀 v6 终判，ip 残 44F 中本族 16 键）；upstream：`insertparagraph.html` 语料。
- 族构成：C2 左空块壳携垫缺失 8 ＋ F 右空块 a 壳过携 6 ＋ I6 styled span 半携 2。

## 方向性壳携语义定谳（Chrome 录音，三族一致收敛）

**规则一句话：空块垫底 br 的壳链 = caret 祖先链中样式壳（b/i/span…）携全链，`<a>`（链接）壳弃携；左空块与右空块同规则。**

- 对照 C2 的 `<p><b><a href=foo>[]foo</a></b></p>`：exp 左块 `<p><b><br></b></p>`（b 携、a 弃）、右块 `<p><b><a href="foo">foo</a></b></p>`（a 携内容——右块非空时 a 是内容本体照携）。**a 弃携只在空块垫底面。**
- 对照 F 族 exp 备选两形右块均裸 `<p><br></p>`：右空块 a 弃携确认。

## 三族 got/exp（16 键，p/div 双变体与壳嵌套两方向逐形）

### C2 左空块壳携垫缺失（8 键，caret 位 0）

| 输入形 | exp（唯一形） | got（我方） |
|---|---|---|
| `<p><b>[]foo</b></p>` | `<p><b><br></b></p><p><b>foo</b></p>` | `<p><br></p><p><b>foo</b></p>` |
| `<p><b><i>[]foo</i></b></p>` / `<p><i><b>[]foo</b></i></p>` | `<p><b><i><br></i></b></p><p><b><i>foo</i></b></p>`（i<b 方向镜像） | `<p><br></p><p><b><i>foo</i></b></p>` |
| `<p><b><a href=foo>[]foo</a></b></p>` | `<p><b><br></b></p><p><b><a href="foo">foo</a></b></p>`（**b 携 a 弃**） | `<p><br></p><p><b><a href="foo">foo</a></b></p>` |

六键 assert_equals 唯一形；a-outer 两键同表。div 宿主变体镜像（`<div><b>[]foo</b></div>` 等）逐形同构。

### F 右空块 a 壳过携（6 键，caret 行尾 `foo&nbsp;[]<br>` 形）

| 输入形 | exp 备选两形（右块均裸） | got（我方过携） |
|---|---|---|
| `<p><a href=foo>foo&nbsp;[]<br></a></p>` | `<p><a>foo&nbsp;</a></p><p><br></p>` ／ `<p><a>foo&nbsp;<br></a></p><p><br></p>` | `<p><a>foo&nbsp;</a></p><p><a><br></a></p>` |
| `<p><a href=foo><b>foo&nbsp;[]<br></b></a></p>` | 同上右块裸（a 内 b 同弃） | `<p><a><b>foo&nbsp;</b></a></p><p><a><b><br></b></a></p>` |
| `<p><b><a href=foo>foo&nbsp;[]<br></a></b></p>` | 同上右块裸（b 外 a 同弃） | `<p><b><a>foo&nbsp;</a></b></p><p><b><a><br></a></b></p>` |

六键均 assert_in_array 双备选；a-outer/b-outer 两方向同弃。

### I6 styled span 半携（2 键，caret 行尾双壳形）

输入 `<div><span style=color:...><span style=bg:...>foo[]</span><br></span></div>`（及 ul>li 变体）：

- exp 四备选：右块垫底 br 均携**双壳全链** `<span style=color><span style=bg><br></span></span>`（br 位置三形：bg 壳内裸 br／foo 后同壳 br／foo+br 同层，差异在左块 br 摆位；右块壳链四形恒全携）。
- got：右块只携外层 color 壳（`<span style=color><br></span>`）＝**半携**（壳链下钻停在 caret 所在 text 的直接父外层，未含全链到内层 bg）。

## 刀向（三臂合一）

垫底 br 的壳包裹链统一在**空块物化点**推导：取 caret 祖先链过滤 `a`（及语义同族链接壳）后按序重建全链包裹垫底 br；左块（caret 位 0 形）与右块（退化物化形）同函数。预期 16 键全收（C2 8 由裸垫转携链、F 6 由过携转弃、I6 2 由半携转全携）；动刀前核 M1/M2 已收面回归窗（wrap 臂/重建链根垫底路径在 44 键其余族零暴露域，白名单门沿用 v6 批 judge 44F 基线）。

## v1 批 P2F 精化（2026-11-15，门禁批实测校正普查结论）

上文「a 过滤后携余链」的普查结论在 a-outer 形被门禁批证伪：v1 批 L1 judge F2P=16 恰等/residF=28 ✓ 但 **P2F=2**——`<p><a href=foo><b>[]foo</b></a></p>`（:1072）与 div 镜像（:1077）刀前裸垫通过、刀后携 b 失配，语料 exp 左块**裸垫** `<p><br></p>`。对照 b-outer 形（:1092 `<p><b><a>[]foo</a></b></p>` exp 携 b `<p><b><br></b></p>`）精化定谳：

**左空块携集 = caret 祖先链中最外层 `a` 之上的壳；`a` 及其内层（含非 a 壳）全弃；无 `a` 携全链。** 收集序（内→外）无法区分 `<b><a>` 与 `<a><b>`（过滤后同为 [b]），须以最外层收集壳之上的祖先扫描定夺。右空块维持「凡链含 a 即全裸」不变（F 6 键 v2 批全 F2P 实证）——**左右不对称属实**。修复见 [knife 收口](2026-11-15-c2fi6-shell-carry-knife.md)。
