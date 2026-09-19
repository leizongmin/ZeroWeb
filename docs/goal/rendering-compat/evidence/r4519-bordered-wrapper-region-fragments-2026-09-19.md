# R4519 证据：R1473 step-2 painter slice ①——bordered wrapper 区域×列 fragment 分段模型（006 翻绿 8.79→0.00）

- 日期：2026-09-19（R4519 C 轮——4 code files + 新模块·net code）
- 用例：`css/css-multicol/multicol-span-all-children-height-006`（article column-count:2 > div.container[height:250 + pink bg + border:20 purple + margin 1em] > [block1 200][spanner 50][block2 200]）
- 方法：`REFTEST_DUMP=1 REFTEST_DUMP_PASS=1 zero-wpt-runner reftest-upstream`（test/ref 双页 ZW 渲染）→ python 逐像素横/纵扫描带分析 + 文本 glyph band 定位。

## 关键定性：PASS 目标 = ZW(ref 页) 渲染（自源 reftest），逐像素解码其几何

ref 页把 spanner 拆分静态化为两个独立 bordered container（border-bottom:none / border-top:none），
走 ZW **基础 multicol 分片路径**（balance target = (border-box + 非折叠 margin)/2、cso cell 裁剪、
整盒按 fragment 位置重绘 + clip）。逐像素实测其渲染（y 绝对坐标，列宽 192 stride 208 的
**double-subtraction 段盒宽 152** = column_width − bl − br，position_multicol_children 列宽约束终值）：

| 元素 | ref 实测 |
|---|---|
| article 绿 | [8,219] 全宽（h=211） |
| region0 col0 | 盒绘于 24：top 边框 [8,160]×[24,44]、left [8,28]×[24,126]、right [140,160]×[24,126]；bg content [28,140]×[44,126]（pink 82 高，黄块裁至 126） |
| region0 col1 | 盒绘于 **−94**（offset = cell 118）：左/右边框 [8,126]、**无 top**（在 cell 外）、bg pink [8,126]、黄块 [8,126] |
| spanner | [8,408]×[126,176]（x=8，**article content 帧**） |
| region1 col0 | 盒绘于 176（cell 43）：left/right [176,219]、底边框被裁空（盒底 246 在 cell 外）；黄块 [176,219] |
| region1 col1 | 盒绘于 133（offset 43）：left/right [176,203]、**底边框 [183,203]**（盒底 203 落 cell 内）；bg pink [176,183]；黄块溢出 [176,219] 覆底边框 |

## 区域×列 cell 模型（本轮落地，`multicol/bordered_region_fragments.rs`）

- 区域 block 总量 = 区域内容渲染量 + 所属盒边/内边距 + 首区域 mt / 末区域 mb
  （边框只归拥有该盒边的区域实例——spanner 相邻边 skip 的几何表达）；
- 区域 cell = 区域总量 / 列数（006：118 = 236/2、43 = 86/2）；
- 末区域内容渲染量 = 剩余 CSS content 预算（R4514 同律：250 − 200 = 50，block2 溢出照绘）；
- 总 advance = Σ cell + Σ spanner = 211 → article h、wrapper 盒 = 211 − mt = 195；
- 段盒宽 = column_width − bl − br = 152（与 ref 页容器 double-subtraction 终值 byte-一致）；
- painter：分段装饰矩形（layout 预裁剪到 cell）+ 子元素逐 fragment paint/cell 裁剪
  （取代 R4506 单次整绘 + R1359 strip）。

## 中间回归教训（首轮 +20 文本偏移）

region 盒 content 原点误用全盒 bt（region1 黄块文本 [201,212] vs ref [181,192]，恰 +20 =
wrapper border-top）：**区域盒的顶边框只属 region0 实例**，区域内坐标须用 r_bt/r_bb。
修正后 006 test/ref **0 diff pixels**（文本 glyph band 逐带一致：[49,60]/[131,145]/[181,192]）。

## A/B

- css-multicol 目录：fail-list diff **唯一 006**（141→140），311→312 pass，零翻转。
- 全 corpus：fail-list diff **唯一 006**（1489→1488），15105/16594（91.03%；单跑 15106 = box-shadow flake ±1 带既有噪声）。
- kill-switch `ZW_BORDERED_FRAG_SEG=0`：006 回落 8.79%（R4517 态）✓。
- 004a/004b（无框 wrapper）不触新路径（wrapper_has_box gate）✓。

## 残余与下轮

006 已清零。同族余账：直连 spanner 路径（无 wrapper 中间层）预算化（span-all 族 002/003 27%/23%）、
008（column-count:1，gate 外）、bordered × 多 spanner（N≥2 region 中间区域上下均 skip 已建模，实测缺 driving 案）。
