# br compare 族收口——enable 行粒度预遍（br 刀）

日期：2026-11-06。主刀：part01e enable 行粒度预遍 `_zwListEnableLineScope` + 三处挂臂。
基线：dps 后 iol 1747P/14F / iul 2047P/18F（2026-11-06 dps 轮证据）。目标族：?1-1000
br 边界三形 × iol/iul 共 6 失败行（归因见 [br 归因铺底](2026-11-06-br-family-attribution.md)
与勘误 9a579316b）。

## 契约（语料 :13/:18/:23 exp 实证）

Chrome 录制以 br 为行界做列表换型：选区所在行整体入列、行外内容不动、行间 br
消费为 li 切分、提取边界邻接 br 消费移除（A 前邻界 / B li 界+壳缘 / C 拆分点），
唯空行载体 br 不动。三形：

- A `foo<br>[bar]` → `foo<ol><li>[bar]</li></ol>`（前邻 br 移除）
- B `f[oo<br>b]ar<br>baz` → `<ol><li>f[oo</li><li>b]ar</li></ol>baz`（双 li 切分、
  尾 br 消费、baz 留外）
- C `<p>[foo]<br>bar</p>` → `<ol><li>[foo]</li></ol><p>bar</p>`（p 按 br 分裂、
  ol 拆出 p）

vendored blockExtend 无行概念，主循环 br 停聚臂把界外 br 吸进壳、wrap flat 重写臂
对 br/文本混合序列化失配——旧径三缺陷形（探针 got 六形）：A `foo<ol><br><li>`、
B bar 遗壳外+尾部幽灵壳、C `<p><ol><li>` ol 嵌 p。

## 实现（part01e.js，四处）

1. `_zwListEnableLineScope(capS, capE)`（`_zwListToggle` 前）：纯扁平块守卫（origin
   块直子全 inline/br ∧ brCount≥1 ∧ 双端点同块 ∧ 列表族语境排除 ∧ span 有可见内容）；
   行号 = 前方 br 计数；选区行 span 收临时 div（借主循环 p/div retag 臂成 li，绕开
   ViaHtml 臂）；行间与紧邻行界 br 移除（隔位 br=空行载体不动）；kept 出参供跨行
   剔除。
2. 调用点：blockExtend **之前**（预遍删 br/移行壳会位移块子索引，事后跑 skew
   newRange 致 nodeList 漏员）。
3. nodeList enable filter 后追加 kept 爬升过滤：块直子仅行壳放行（A foo 剔除、
   C bar 剔除），脱块/块外成员放行。
4. 壳臂 freshShell 后补 `_zwListFixDisallowedAncestors(newShell)`（vendored :5928
   等价，lineScope 门控）：C 形 ol 落 p 内，p ∈ inlineContents ∧ ol ∈ prohibited-
   paragraph-children → allowedChild 祖先链首检 false → splitParent 拆出，残段净
   `<p>bar</p>`；置于 dps 归并出口前（按新树位判归并）。

## 首批 P2F 定谳与守卫修复

首全腿批 iol 腿 P2F 2：`<ul><li>[foo]<br>bar<li>baz</ul>` /
`<ul><li>foo<br>[bar]<li>baz</ul>`——b 爬升命中 li（list-item 为 block）在既有列表
内误入预遍，破坏整壳 retag 域（期望 br 原位）。守卫修复：b 或任意祖先 ∈
{ol,ul,li,dl,dt,dd} 恒 null 走旧径。修后 zz-probe-br 6/6 复验逐字全中。

## 门禁

- 探针 zz-probe-br 6/6：got 逐字对语料 exp（A/B/C × iol/iul），sel 位点同 exp 标位
  （dump 式恒假断言投递，Fail 属机制）。
- 零扰动：zz-probe-qv13 26/26（other 入口）+ zz-probe-dps 11/11。
- 执行体证据：`grep -a -c _zwListEnableLineScope target/release/zero-wpt-runner` = 2、
  注释 marker = 1；构建行 0.18-0.22s no-op 复用（二进制含刀）。
- iol 腿：1747P/14F → **1750P/11F**（judge-dps：F2P 3 / P2F 0 / postF 0 / lost 0，
  rows 1761→1761 恒等，VERDICT OK）。
- iul 腿：2047P/18F → **2050P/15F**（F2P 3 / P2F 0 / postF 0 / lost 0，rows
  2065→2065 恒等，VERDICT OK）。
- enginelib：**3773P/5F 五红名册恒一**（p56_adopted_snapshot_restore_hook /
  p99s_rule_style_preserves_custom_property_case / stylesheet_add_rule_remove_rule_r3276 /
  stylesheets_rule_style_r2810 / stylesheets_write_r2809 逐名恒等；P 数 3766→3773 =
  bf838cd29 s306 新增 7 测试的基线时序差）。批脚本 ROSTER DRIFT 行为原稿 R 提取带
  4 空格缩进 vs EXPECT 无缩进的恒 drift 潜伏 bug（并行席已落盘 sed strip 修复），
  非行为漂移；真实名册自日志缩进名行直接复算恒等。
- mt 全文件腿：10,092 键恒等（shared F2P 0 / P2F 0 / postF 0 / lost 0，VERDICT
  OK）——br 预遍对 mt 语料零扰动（原始行计数漂移为多行键复用噪声，键级零翻转，
  与 dps 轮 mt 判读同口径）。

## 残差（本刀后）

- iol 11F：?1001-last rtl ×3 + nested ③ ×3 + blockquote ×2 + styled ×1 + noneditable ×1 + qVb 伪影 ×1（br ×3 除名）。
- iul 15F：nested ×2 + ?1001-2000 blockquote ×2 + nested ×2 + ?2001-last divbr straddle ×2 + rtl ×3 + div 边界 ×3 + noneditable ×1（br ×3 除名）。
- 窗界伪影族（?1001-last 首案状态重置）维持不追。

日志：.acceptance/dps/batch-br-legs.log（批汇总）、batch-{iol,iul}-post.log、
batch-enginelib.log、batch-mt-post.log、br-probe-round3.log（判读器 judge-dps.py 复用）。
