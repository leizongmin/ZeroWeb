# 2026-11-06 qState 读面 P10 刀——跨 tag 嵌套 after 面 8 行翻绿 + before/after 悖论定谳

## 债与刀

qState 读面债（控制面 ⑲ 后残余）：8 行 `queryCommandState after` 结构行（iol/iul
镜像各 4），全部为**跨 tag 嵌套同形**——iol 语料 ul-source 行、iul 语料 ol-source
行。极性双向：期望 false 6 行（after 树端点 li 含异型嵌套，如
`<ul><li>[foo]<ul><li>bar</ul><li>baz</ul>` → `<ol><li>[foo]<ul>…</li></ol>…`）、
期望 true 2 行（after 树无嵌套，如 `[foo<ul><li>bar]</ul>baz` →
`<ol><li>[foo</li><li>bar]</li></ol>baz`）。

刀（part01e.js `_zwListQueryStateBool`）：两端点最近列表祖先 tag 恰为本命令型，
且端点 li 无**异型**嵌套（`li.querySelector(opp)`）；mixed 判定仅归 indeterm。

## P5 REJECT 与悖论定谳

- **P5 首批（本席前轮）REJECT**：端点直读 + 双型嵌套罚。after 面拟合 286/286，
  但拟合只盖 after 面——before 面同 tag 嵌套真值行（:343
  `<ol><li>[foo<ol><li>bar]</ol>baz</ol>` 录 true）被误翻 false。门禁 iol 55 /
  iul 52（vs q15 F2P=7+6 / **P2F=14+14**）红线双破，批不可落地。
- **悖论定谳（参考算法判读，implementation.js）**：`getSelectionListState`
  every-ol 子句仅禁 ul（:3980-3987）——before 面 :343 录 true 与语料自洽（同型
  嵌套不进 mixed）；8 行 after 面 `qStateA=qIndA=F` 全 F 模式指向 post-action
  range 读 `'none'`（blockExtend nodeList 空＝参考算法 action 后选区停在列表
  外），而非 marker-range 推得的 'mixed'。**结论：after 录值含命令史态（选区恢
  复耦合），纯 (tree, selection) 读面不可双全**。impl-seat 06:45 独立双面拟合同
  判（before 286/286 + after 282/286，恰余 :478 wholesale 碰撞族）。
- **P10 取舍**：opp-only 嵌套罚——before 面全保（落地树旧逻辑本已满足该面，
  回落基线），after 面 282/286，翻绿恰为 opp-nest 6+2 中可直读的 4+4 行。

## 门禁（P10 批，gate 06:38:41 起四腿；enginelib 首跑 06:42 Terminated、06:47 重跑）

- iol 1761 行 fails 45→**41**（vs b10 F2P=4/P2F=0）；iul 2065 行 fails 42→**38**
  （vs b10 F2P=4/P2F=0，本席独立复核）。
- judge（judge-p10.txt 行级元组）：**vs b10 F2P=8 / P2F=0 / newF=0 / lost=0**；
  **vs q15 F2P=13 / P2F=0**（8 本刀 + 5 ⑲ compare 收口）。红线守。
- 翻绿 8 行逐名＝跨 tag（opp-nest）嵌套 after 面：iol
  `<ul><li>[foo]<ul><li>bar</ul><li>baz</ul>` /
  `<ul><li>foo<li>[bar]<ul><li>baz</ul><li>quz</ul>` /
  `<ul><li>[foo]<ul><li>bar</ul>baz</ul>` /
  `<ul><li>foo<ul><li>bar</ul>[baz]</ul>` 四行 + iul ol-source 镜像四行。
- enginelib **3758P/5F**，红集与 q16-reds.txt 五行逐名恒一（stylesheet 族已知
  红，非 editing 面）。probe 腿 rc=1 为已知旧探针聚合 assert（非本刀判据）。

## 残余债（账面核实：iol 5 行 + iul 7 行，fresh log 逐行点验）

结构 qState 行留债 7 行，两亚族并入选区恢复耦合刀：

1. 选区恢复亚族 ×3：期望 true 而我方 action 后选区停在列表外（端点直读不可
   达）——iol `[foo<ul><li>bar]</ul>baz`、
   `<ul><li>foo<ul><li>[bar]<li>baz</ul><li>quz</ul>`；iul 镜像
   `[foo<ol><li>bar]</ol>baz`。
2. :478 wholesale 同型嵌套亚族 ×4：期望 false 的命令史态（'none' range）纯读
   不可达行——iol `<ul><li>[foo<ul><li>bar]</ul>baz</ul>`、
   `<ul><li>[foo<ul><li>bar]</ul><li>baz</ul>`；iul 镜像
   `<ol><li>[foo<ol><li>bar]</ol>baz</ol>`、
   `<ol><li>[foo<ol><li>bar]</ol><li>baz</ol>`。

非本刀独立债：iul dps-div straddle 行
`<ul><li>foo<ul><li>b[ar</ul><li>b]az</ul>`（qState+qInd after 双行）；empty-body
write-face 行 ×3（`{} + inserttext` iol 1 + iul 2）。诊断工具留档：
zz-probe-qstate3.html（impl-seat 提案，挂 implementation.js + tests.js
setupDiv/setSelection 原语 dump action 后 sel 端点）。
