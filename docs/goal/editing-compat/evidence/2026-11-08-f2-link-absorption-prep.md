# F2 link-absorption 36 开工预研（2026-11-08，只读；源码未动）

承 [普查分簇档](2026-11-08-knife11-v5fix2-mt-residual-census.md) §下一刀建议。
本轮在换基门在飞期间完成只读预研：F2 两路径语义定形 + delta carry 模型 +
集成点清单 + 待判别项。开工前置①（尾链落地后才动源码）未满足，本档不施工。

## F2a pending-link 面（26 键：裸 1 + 结构中置 13 + forecolor 互序 8 + underline 互序 4）

语料形 `foo[]bar` + `createlink` collapsed + 后继命令 + `inserttext`，compare
innerHTML。期望/实得采样（main 窗片，ab 页同形 shared-red）：

| 链 | 期望 | 实得 |
|---|---|---|
| `createlink>inserttext` | `foo<a href="…">a</a>bar` | `fooabar`（裸，link 全缺） |
| `forecolor(#0000FF/blue)…`（双序 :1994-:2012） | `foo<a href="…">a</a>bar`（无 font——≡参照） | `foo<font color="#0000ff">a</font>bar` |
| `forecolor(brown)…`（双序 :2014/:2019） | `foo<a…><font color="#a52a2a">a</font></a>bar`（**carry**） | 未逐键采样（同族 got） |
| `forecolor(black)…`（双序 :2024/:2029） | `foo<a…><font color="#000000">a</font></a>bar`（**carry**） | 同上 |
| `underline`（单/双应用、双序 :2033-:2052） | `foo<a href="…">a</a>bar`（全四案无 u） | `foo<u>a</u>bar` |
| `createlink>formatblock>inserttext` | `<div>foo<a…>a</a>bar</div>` | `<div>fooabar</div>` |
| `createlink>indent/hr/img/iol/linebreak/inserthtml…>inserttext` | 结构落位后 `<a…>a</a>` 仍在 | typed 段裸奔 |
| `createlink>delete>inserttext` | `fo<a…><font color="#000000">a</font></a>bar` | `foabar` |

语义定形（**统一 delta 法则**，2026-11-08 待判别④修正——初审「link armed 压制
样式」模型经 ：2014-:2030 brown/black 双序 carry 反证废止）：① collapsed
createlink 置 pending link 态，typed 段并入 `<a href>`；② typed 段在 link
上下文**仅 materialize ≠ 钉死 link 参照（#0000ff/underline）的属性**——
forecolor #0000FF/blue ≡ 参照 ⇒ 无 font；brown/black ≠ 参照 ⇒ font 内层
carry；underline/decoration ≡ 参照 ⇒ 全四案无 u；③ **override 读面与
materialization 解耦**：qV(forecolor)-after 全序照设（:1997 "rgb(0, 0, 255)"、
:2017 "rgb(165, 42, 42)"）、qS(underline)-after 照设 true（:2037）——knife
不动 override 置位，只在 typing insert materialization 层做 delta 过滤；
④ 结构命令不清 pending link（13 键期望 link 全存活）；⑤ delete 中置/既有-link
的 carry 同一法则（见 F2b 节）。

### 现状接线

- arm 点已存在：`part06.js:4893-4900` collapsed createlink 臂——**仅置
  `__zwTypingCmdArmed` 判别旗标，注释明言「本仓无消费臂（typed 段 link 化
  不在范围）」**。F2a = 补消费臂：新增 `__zwTypingLink`（href 槽）+ typing
  insert 侧 link 化；不动旗标判别语义（knife11 v5 ① 臂判别读依赖）。
- 消费点：`part01.js:9451` `_zwCeTypingInsert(tags, text)` 及四调用点
  （:9585/:9640/:9731/:9748）。link armed 时 `<a href>` 外裹 + **delta 过滤**
  （②法则的统一实现位）：色值归一化后 ≡ #0000ff 不 materialize、decoration
  类 tag（u/s?）在 link 上下文全滤（corpus 四案全裸）——非 link 上下文过滤
  不生效（:1153-:1157 backcolor 形照旧 materialize，现绿不动）。
- 清除点：与 typing 包裹态同生命周期（part06 `_zwSync` 选区变更清面，
  part01f.js:1190 注释锚指 part06 :10355），防跨子测泄漏。
- 判别注记：override 读面与 materialization 解耦（②③法则）——qS/qV 断言
  现绿（残键仅 innerHTML compare 行），knife 不动 override 置位层；delta
  过滤只在 typing insert 消费层。开工时逐键复核全 23 子测试数组（待判别④
  逐刀纪律）。

### 保全 wrap 与 delete 幸存（2026-11-08 三读补勘）

三读 part06 值/结构命令臂保全机制（:4942-:4964 `cmdPresOv` 门 + :4949-:4964
快照 + :5083-:5088 回写）补出集成表第 7/8 点：

- **第 7 点（结构命令保全 wrap）**：hr/img 臂尾部 sel.collapse → `_zwSync`
  清 typing 态（:4952-:4956 注释锚），而结构 13 键期望 link 全存活 ⇒
  `__zwTypingLink` 必须并入 `presTt` 同款快照/条件回写（null 保持式）。`cmdPresOv`
  门 :4942-:4948 已覆盖全部 13 键命令（formatblock/indent/hr/img/iol/iul/
  insertparagraph/insertlinebreak/justify×4/inserthtml），扩一个全局量零新门。
- **第 8 点（delete 幸存语义）**：:1170 `createlink>delete>inserttext` 期望
  `fo<a…><font color="#000000">a</font></a>bar`——pending link **幸存
  delete**，与 typing carrier 规范清除语义相异（:4931-:4941 cmd 门注释
  「delete 规范清 typing carrier」；final2 44 行 delete+inserttext compare 为
  carrier 清除证据面）。F2 knife 的 delete 臂不得清 `__zwTypingLink`；:1170
  的 `#000000` carry 走 F2b tcap delta 通道（被删 "o" 黑 vs link blue），
  与 carrier 保全互不相干。
- 消费面形态勘定（part01 四调用点）：D4 拆壳臂 ：9624（innerTags 链推导，
  非 typing tags）+ 文本 splice 臂 ：9679（ttags1）+ 元素边界两臂
  :9770/:9787（ttags2/ttags3）——后三臂统一经 `_zwCeTypingTagsAt` 过滤后进
  `_zwCeTypingInsert`；link 外裹 + delta 过滤做在 `_zwCeTypingInsert` 单点
  （读 `__zwTypingLink`）即可全覆盖，D4 臂语料无 link×toggle-off 交形暂不触。

## F2b 既有-link 面（delete 族 10 键）+ delta carry 模型

multitest.js :2138-:2172 连续族（链均 `delete>inserttext(a)`），七案全表：

| 行 | setup | 期望 |
|---|---|---|
| :2138 | `foo<a…><font color=blue>[bar]</font></a>baz` | `foo<a…>a[]</a>baz`（无 font） |
| :2143 | `foo<font color=blue><a…>[bar]</a></font>baz` | `foo<a…>a[]</a>baz`（外 font 剥） |
| :2148 | `foo<a…><font color=brown>[bar]</font></a>baz` | `foo<a…><font color="#a52a2a">a[]</font></a>`（carry） |
| :2153 | `foo<font color=brown><a…>[bar]</a></font>baz` | `foo<a…>a[]</a>baz`（外 font 剥） |
| :2158 | `foo<a…><font color=black>[bar]</font></a>baz` | `foo<a…><font color="#000000">a[]</font></a>`（carry） |
| :2163 | `foo<a…><u>[bar]</u></a>baz` | `foo<a…>a[]</a>baz`（u 剥，无 carry） |
| :2168 | `foo<u><a…>[bar]</a></u>baz` | `foo<a…>a[]</a>baz`（外 u 剥） |

**delta carry 模型（七案全闭合）**：carry = 删除内容 computed style **相对
link 上下文的 delta**——link UA 默认色 blue/下划线 ⇒ blue（:2138）与 u
（:2163/:2168）零 delta 不 carry；brown/black ≠ blue ⇒ carry 为 link 内层
font（:2148/:2158，色值 #a52a2a/#000000 与 corpus 逐字吻合）；外层壳一律剥
离（link 升出，:2143/:2153/:2168）；空 link 幸存接 typed 段。
`createlink>delete>inserttext` 的 `#000000` carry 同模型：被删 "o" 黑 vs
link blue ⇒ 有 delta。

### 现状与病灶

- `part01f.js:1145/:1150` `__zw_ce_capture_typing_chain` 链上行遇 `<a>` 即
  `hasA=true; break; return []`——**外链弃捕语义必须保持**（:2168 外 u 不捕，
  现绿），不可盲撤。
- 实得分裂：:2148 got `foo<a…>a</a>baz`（内 font 被删净、无 carry——缺 delta
  carry 通道）；:2143 got `foo<font…><a…>a</a></font>`（**外 font 未剥**——
  对比 :2168 外 u 已剥现绿：现有 fmt 壳剥离臂不含 style 壳 font）。
- 病灶拆分：① `part01f.js:1200` delta ⑤ `tcap` 捕获（非 collapsed 门）遇
  hasA 返空——改为 hasA 时改收**删除区内层 style props**（color/face/size，
  不含 b/i/u/s/sub/sup——:2163 u 不 carry 由此自然成立）作为 link 内层 carry
  快照；② 删除落定后空 link 外层 inline 壳剥离扩 font（现只剥 fmt 壳）；
  ③ typing insert 侧：link armed + carry 快照 ⇒ `<a><font …>a</font></a>`。

### 约束锚（现绿不可回退）

:2138（blue-in-link 零 delta）、:2163/:2168（u 双位形）、:2113（裸 link 删后
typed 段入空 link 现绿）、:2118-:2130（font 直亲壳删后幸存——非 link 形外壳
幸存语义）、:1164/:1174（createlink 后无 typing 无空 `<a>`）、:2173-:2182 邻族
（sub/fontsize 无 link 形，F6b 域——外壳幸存语义与 link 形相反，剥离规则必须
link 特异）。范围边界注记：:2053-:2062（subscript×fontsize 显式 collapsed
互序，qV fontsize-after 照设 "2" 而期望**无** font materialization）属 F6b
域独立机制（显式 fontsize 在 sub 上下文不 materialize 的 Chrome 面），F2 刀
不触及、F6b 开工时单独立判别。红线：每刀 F2P≥0 且 P2F=0/post-only=0/lost=0/
rows 恒一。

## 待判别项（2026-11-08 本轮闭合①②③；④为逐刀纪律）

1. **delta vs absolute——已闭合，delta 模型确立**：multitest.js 无壳
   `foo[]bar`+`delete>inserttext` 形**不存在**（:2083-:2130 全族均带壳），
   delta 模型在语料内无反例。另证 link 特异性：:2118-:2130 font face/size/
   color **直亲**壳删后幸存接 typed 段（`foo<font face="sans-serif">a[]</font>`，
   现绿）——与 :2143 font **包 link** 期望剥离相反 ⇒「外壳幸存/剥离」规则
   必须 link 特异，不可全局化。
2. **UA link 默认色——已闭合，钉死参照不读 computed**：本仓 UA 样式表无
   link 色规则（computed = 继承黑），若读 computed 则 :1170（黑 vs 黑零
   delta 需 carry）与 :2138（blue 需零 delta）双破。**参照钉死 #0000ff**
   （Chrome UA blue）：:2138 blue≡ref 零 delta ✓、:2148 brown、:2158 black、
   :1170 黑 均≠ref ⇒ carry ✓ 四案全合；carry 通道仅 color/face/size，
   永不含 text-decoration（:2163/:2168 u 不 carry 自然成立）。
3. **stray empty `<a>`——已闭合，pending-flag 模型确认**：:1164
   `createlink>delete`（无 typing）期望 `fo[]bar` **无空 `<a>`**；:1174
   `createlink>formatblock` 期望 `<div>foo[]bar</div>` 同证——createlink
   collapsed 只置态不插元素，a 仅在 typing 时插入。qV 断言
   `createlink:[false,false,"",…]`（qV=""、qS=false）现绿 ⇒ arm 面不得触
   override 读面（与 knife11 v5 旗标-only 臂一致）。
4. **全 36 键 qS/qV 数组复核——已闭合（2026-11-08 全普查完成）**：forecolor
   8（:1994-:2032 qV-after 全序照设）、underline 4（:2033-:2052 qS-after=true
   照设）、F2b 7（:2138-:2172 dict 全零）、pending-flag 邻域（:1159-:1177
   createlink 全零）、结构 13（:1194-:1310——indent/hr/inserthtml/img/
   linebreak/insertparagraph dict 全零，iol/iul qS-after=true，justify×4 qV
   left→center/justify/left/right）全部现绿仅 innerHTML 红；**pending-link
   态在全部 dict 零出现** ⇒ arm 不触 override 设计约束逐键验证成立。knife
   动源码前无剩余判别项。

## 集成点清单

| 位 | 文件:行 | 动作 |
|---|---|---|
| arm | part06.js:4893-4900 | 置 `__zwTypingLink`=href；②清 tags（读回 risks 见待判别④） |
| 消费+delta 过滤 | part01.js:9451 + :9585/:9640/:9731/:9748 | link armed ⇒ `<a>` 外裹；色 ≡#0000ff 不 materialize、decoration 全滤；carry 快照 ⇒ 内层 font |
| 清除 | part06 `_zwSync`（:10355 面） | `__zwTypingLink` 同 typing 态生命周期 |
| hasA 拆分 | part01f.js:1145-:1150 | 外链弃捕保持；内链 delta carry 新通道 |
| tcap 门 | part01f.js:1200（delta ⑤） | hasA 时改收内层 style props 快照 |
| 外壳剥离 | part01f.js 删除落定 strip 面 | 空 link 外层 inline 壳扩含 font |
| 保全 wrap | part06.js:4962-:4966 + :5083-:5088 | `__zwTypingLink` 并入 presTt 同款快照/条件回写 |
| delete 幸存 | part01f.js delete apply 面 | delete 臂不清 `__zwTypingLink`（:1170 面） |

## 运行注记

本预研为纯只读分析（语料行号、期望/实得取自 census 残键全名单
`.acceptance/dps/census-mt-residual-v5fix2-keys.txt`），与换基门尾链零冲突；
落档时换基门 mt 腿在飞（03:22），源码未动。三读补勘落档时门 mt 腿收尾中
（03:39），仍零源码触面；期间 origin 另进 shadow M3 系 2 提交
（ddb1e73cf/9360e07c5），本地 9 提交待 writer 落地 rebase 线性汇合。
