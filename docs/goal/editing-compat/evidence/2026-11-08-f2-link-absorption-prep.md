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
| `forecolor(X)>createlink>inserttext` | `foo<a href="…">a</a>bar` | `foo<font color="…">a</font>bar` |
| `createlink>forecolor(X)>inserttext` | `foo<a href="…">a</a>bar`（**无 font**） | 同上 |
| `createlink>underline>inserttext` | `foo<a href="…">a</a>bar`（**无 u**） | `foo<u>a</u>bar` |
| `createlink>formatblock>inserttext` | `<div>foo<a…>a</a>bar</div>` | `<div>fooabar</div>` |
| `createlink>indent/hr/img/iol/linebreak/inserthtml…>inserttext` | 结构落位后 `<a…>a</a>` 仍在 | typed 段裸奔 |
| `createlink>delete>inserttext` | `fo<a…><font color="#000000">a</font></a>bar` | `foabar` |

语义定形：① collapsed createlink 置 pending link 态，typed 段并入 `<a href>`；
② **createlink 清空既有 typing style**（forecolor 前置序期望裸 link）；③ link
armed 期间**显式样式命令不再上 carry**（forecolor/underline 后置序期望同样裸
link，双序一致）；④ 结构命令不清 pending link（13 键期望 link 全存活）；⑤
delete 中置例外：delete 重引字体 carry，复合于 link **内层**（见 delta 模型）。

### 现状接线

- arm 点已存在：`part06.js:4893-4900` collapsed createlink 臂——**仅置
  `__zwTypingCmdArmed` 判别旗标，注释明言「本仓无消费臂（typed 段 link 化
  不在范围）」**。F2a = 补消费臂：新增 `__zwTypingLink`（href 槽）+ typing
  insert 侧 link 化；不动旗标判别语义（knife11 v5 ① 臂判别读依赖）。
- 消费点：`part01.js:9451` `_zwCeTypingInsert(tags, text)` 及四调用点
  （:9585/:9640/:9731/:9748）。link armed 时 link-only 应用 + 抑制
  `__zwTypingTags`（②③的统一实现位）。
- 清除点：与 typing 包裹态同生命周期（part06 `_zwSync` 选区变更清面，
  part01f.js:1190 注释锚指 part06 :10355），防跨子测泄漏。
- 判别注记：③的抑制若放在 override 置位层会动 qS/qV-after 读——这些键的
  qS/qV 断言现绿（残键仅 innerHTML compare 行），抑制必须**只在 typing
  insert 消费层**，override 读面不动。开工时逐键复核全 23 子测试数组。

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

:2138（blue-in-link 零 delta）、:2163/:2168（u 双位形）、:2173-:2182 邻族
（sub/fontsize 无 link 形，F6b 域——外壳幸存语义与 link 形相反，剥离规则必须
link 特异）。红线：每刀 F2P≥0 且 P2F=0/post-only=0/lost=0/rows 恒一。

## 待判别项（开工第一步，动源码前完成）

1. **delta vs absolute**：plain `delete>inserttext`（foo[]bar 无壳无 link）的
   corpus 期望是否带 `font color="#000000"`——带 ⇒ absolute carry（delta 模型
   被否，:2138 需另解）；不带 ⇒ delta 模型确立。multitest.js 普查该链形。
2. **UA link 默认色依赖**：delta 参照 = link computed style。核本仓 UA 样式
   link 色是否 blue（与 corpus 参照一致）；不一致则参照面须 corpus 钉死值，
   不读 computed。
3. **stray empty `<a>`**：createlink collapsed 后**无** typing 的语料形
   （选区移走/仅结构命令链）是否出现空 `<a>` 幸存期望——pending-flag 模型
   （typing 时才插 a）与立即插空 a 模型分界。扫 createlink.js 及 multitest
   相关链形。
4. **全 36 键 qS/qV 数组复核**：普查残键仅 innerHTML compare 红，qS/qV 全绿
   ——link arm/清除设计不得动 override 读面。

## 集成点清单

| 位 | 文件:行 | 动作 |
|---|---|---|
| arm | part06.js:4893-4900 | 置 `__zwTypingLink`=href；②清 tags（读回 risks 见待判别④） |
| 抑制+消费 | part01.js:9451 + :9585/:9640/:9731/:9748 | link armed ⇒ link-only；carry 快照 ⇒ 内层 font |
| 清除 | part06 `_zwSync`（:10355 面） | `__zwTypingLink` 同 typing 态生命周期 |
| hasA 拆分 | part01f.js:1145-:1150 | 外链弃捕保持；内链 delta carry 新通道 |
| tcap 门 | part01f.js:1200（delta ⑤） | hasA 时改收内层 style props 快照 |
| 外壳剥离 | part01f.js 删除落定 strip 面 | 空 link 外层 inline 壳扩含 font |

## 运行注记

本预研为纯只读分析（语料行号、期望/实得取自 census 残键全名单
`.acceptance/dps/census-mt-residual-v5fix2-keys.txt`），与换基门尾链零冲突；
落档时换基门 mt 腿在飞（03:22），源码未动。
