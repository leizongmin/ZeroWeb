# indent D 族跨空白并邻刀（R18）收口与 zw-r18 ABORT 打捞定谳（2026-11-18）

刀体 **2b21adf7f**（已推 origin/main）：part01c.js 两臂＋part139 钉测 5 形
（`test_indent_ws_merge_pins`）。

## 刀面（r17-indent.log 9F 中本刀 6 键）

- D 族 3 键（p 成员邻接合并缺失）：`<blockquote><p>foo</blockquote> <p>[bar]`、
  `<blockquote><p>foo</blockquote> <p>[bar]</p> <blockquote><p>baz</blockquote>`、
  `<p>[foo]</p> <blockquote><p>bar</blockquote>`——got 独立双壳、exp 单壳并邻。
- C 族 ws 形 3 键（列表域空白成员）：`<ol><li>foo</li><ol><li>bar</li> </ol> <li>[baz]</ol>`、
  `<ol><li>foo<ol><li>bar</li> </ol></li> <li>[baz]</ol>`、
  `<ol><li>foo<ol><li>bar</li></ol></li> <li>[baz]</ol>`（源含折叠 ws 的三形）。

## 根因与修复（详见 commit 2b21adf7f 正文）

1. collapsed whitespace 双向走查缺前向臂：回退遇非 ws 文本/img 断后须转前向走查
   （遇块/br＝行尾边界→折叠 true）。缺此臂时块界夹缝 ws（`</blockquote> <p>` 形）
   误判可见→wrap invisible 拉环不收→新壳不并邻（D 族三键 got 独立双壳）。
   spec：[collapsed-whitespace-node](https://w3c.github.io/editing/execCommand.html#collapsed-whitespace-node)。
2. ViaHtml 复用臂 blanket `(anyBr?'':'<br>')` 对块级成员首误垫行界（ws 拉入后成员首
   为 ws 文本、真界由块级 p 提供，Chrome 录形无 br）→换 DOM 臂同款 inline 垫界判据
   （前邻尾内联＋成员首/尾内联＋非 br）；A 族文本成员两条件恒真＝垫界保持。
   spec：[wrap](https://w3c.github.io/editing/execCommand.html#wrap)。

## zw-r18 首射 ABORT 定谳＝judge 缺陷（非引擎债）

- 首射（pid 1332910，04:24，head=2b21adf7f bin e9e63b211b8b05548fddb7a9ee5c5fb8）
  L1 ABORT「翻绿键形失配（2/1/1/1/1/1）」。写者轮 04:31 429 亡故未及入档。
- 数据面三硬门**全过**（ABORT 同时在册）：计数 1326/3/0 恰（1320+6/9−6）＋零新增红
  ＋resolved 恰 6 键且与 writer 预置期望 `r18b-indent-resolved-expected.txt`
  **cmp 恒等**（本席实证）。
- judge 根因＝`grep -cF '<blockquote><p>foo</blockquote> <p>'` 前缀撞 D1 双键行
  （两键共享该前缀子串）→ actual 2/1/1/1/1/1 vs 期望 1/1/1/1/1/1。同型先例：
  grep 子串撞腿标题静态文本假阳性。教训入账：翻绿 judge 用整行集合 diff，
  禁前缀/子串计数。

## 打捞批 zw-r18b ALL-DONE（2026-11-18T06:09:55+0800）

八腿全绿 @2b21adf7f bin e9e63b211b8b05548fddb7a9ee5c5fb8 全程恒一（与首射 L1
同 md5＝构建可复现，首射 L1 数据域闭合确认）：

- indent 1326/3/0（20s）翻绿恰 6 刀键整行集合 diff＋零新增红（本批 L1 重跑，
  修正 judge；独立复算 comm 双侧实证）。
- fb 5043/3（688s）＋fs 2419/3（54s）＋img 962/0（11s）＋event 176/4（3s）＋
  ie 1476/1852/3（140s）fail/timeout 集全部 SET-IDENTICAL 对 r17 fresh 锚
  （diff 恒等独立复算）。
- lib 4165 passed/0 failed（16.75s，`--skip test_get_computed_style_font_shorthand_r2761`
  外域红留痕）pin-in 8/8（新增 indent_ws_merge）。
- mt 10090/2/0 X37 双键恒一零暴露（multiplier2 终审，1847s）＝collapsed-ws
  走查共享面（indent/list/ip 全量）零扰动双哨兑现。

## 残 3F（r18b-indent.log，三族定刀序）

| 族 | 键 | got → exp |
|----|----|-----------|
| C 族 ol 域 1 键 | `<ol><li>foo<ol><li>bar</ol>[baz]</ol>` | got `<ol><li>foo</li><ol><li>bar</li><li>baz</li></ol></ol>`（baz 落内层 ol 新 li）→ exp `<ol><ol><li>foo<ol><li>bar</li></ol>baz</li></ol></ol>`（新壳包整 ol 外＋baz 并首 li 尾） |
| E-表 1 键 | `<table><tbody data-start=0 data-end=1><tr><td>foo<td>bar<td>baz</table><p>extra` | got 逐 td 各裹 bq → exp `<blockquote><table>…</table></blockquote>` 整表外包（tbody 全覆盖形；R17 升级臂 table 家族排除域遗留） |
| F li 尾 br 1 键 | `<ul><li>a<br>{<br>}</li><li>b</li></ul>` | got `<ul><ul><li>a<br></li></ul>…`（选中 br 被行界消费）→ exp `<ul><ul><li>a<br><br></li></ul>…`（占位 br＋选中 br 双存） |

下一步＝**C 族 ol 域刀（1 键，baz 越级归位＋整壳外包）**或 **E-表升级臂扩展
（tbody 全覆盖判据，R17 排除域收口）**；F 族 li 尾 br 可作小刀穿插。
