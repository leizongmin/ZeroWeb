# R5516 轮次证据：匿名 cell margin 抑制＋BFC 隔离＋行高分配输入 aware floor（CSS2 §17.4/§17.2.1/§17.5.3）

日期：2026-11-13
轮次：R5516（R5515 后继）
择案：css-tables/anonymous-table-cell-margin-collapsing（✗2.77%，R5099 记档候选＋R1752
记档 3.40% characterization 在册——账本 grep 含 evidence/*.md 简称形）。
结论：**✗2.77% → ✓0.21%**（solo，残差＝cell B 幽灵列 9.6px 右带，亚阈在册）。三件套：
①tree.rs 匿名 cell（table-row 的块级子）margin 抑制（§17.4 taffy 输入源头清零，与
converter 显式 cell is_margin_suppressed 同口径）；②同点位 R5281 口径
item_margins_isolated＝true（子 margin 不塌穿 cell→row→table）；③table.rs
content_row_heights 分配输入加 cell_float_aware_content_height floor（分配度量与
R1390 装配度量同源）。**③对真 td 同病独立生效**（probe-real-td：rows 125/25 表 150
→100/0 表 100），非匿名 cell 特有。

## 一、取证链（临时插桩＋A/B 探针，已还原）

- 案面：display:table(height:100px,red) > row(green)[cellA(margin:50px 0)>内空
  div(margin:50px 0), cellB(margin:50px 0)] > row(green)[cellC(margin:50px 0)] > 空
  row。旧态 LAYOUT_DUMP：表 y=84 h=150 mt=50（**cell margin 塌至表顶**）、行
  100/50/0、cellA h=100 mt=50 dmt=50。
- 机制链（插桩 position_cells 入口实测）：匿名 cell 无合成盒——row 的块级子
  **直接当 cell 收编**（build_row 按列位），ComputedStyle display 仍 Block →
  converter `is_margin_suppressed`（TableCell 臂，margin-applies-to-005/006/007
  驱动）与 `item_margins_isolated`（R5281）双双旁路 → cell 自身 margin 入账 +
  子 margin 塌穿（taffy-local block 折叠：cellA margin＝max(自身,子)＝50 上提
  row→table 逃逸；cell 高塌 0）。
- A/B 探针（probe-ab：cellC margin:0）：行 2 高仍 50＝行高与 cell 自身 margin
  无关，来自分配 extras（target 100−content[0,0]→50/行）＋装配 aware 兜底差额
  ——**分配/装配度量分裂**实锤。
- 真 td 对照（probe-real-td：`<table height:100px>` td>空 div(margin:50px 0)+
  空行）：rows 125/25 表 150 与匿名案同病＝taffy 隔离盒高漏末子 margin-bottom
  （50 vs 真值 100）→ content_row_heights=[50,0] → extras 25/行 → 装配
  max(taffy,aware=100)+25=125。**margin-collapse-110/111 族无表 definite 高不
  触发分配，故在册绿**。

## 二、修复（三处，+42/−7）

1. **tree.rs build_subtree**（taffy 样式转换后）：父＝真表上下文内
   display:TableRow（复用 R5243 `row_in_table_context` 门）且自身
   Static/float None → `taffy_style.margin = 0`（§17.4 margin 不应用内部表元
   素；padding 不动＝§17.5 cell padding 始终应用）+ `item_margins_isolated =
   true`（R5281 同口径立 BFC）。kill-switch `ZW_IMPLICIT_CELL_MARGIN_SUPPRESS=0`。
2. **table.rs content_row_heights**：cell 度量加
   `.max(cell_float_aware_content_height(cb, styles))`——分配输入与装配行高
   （R1390 aware）同源，末子 margin-bottom 漏计不再双计。
3. **engine/tests/r1752_anon_table_margin_diag.rs**：旧 characterization（钉
   h≈150 已知 bug）按其文档自述（「anon cell 语义落地后更新为 ~100」）翻转为
   修复锚：h≈100＋表自身 margin_top=0。

## 三、锚测试（r5516_implicit_cell_margin_tests，2 针）

①匿名案最小形：表盒 h=100/mt=0、行 1=100/行 2=0（旧 150/50/100/50）；
②真 td 分配锚：table h=100、td1=100（BFC 内子 margin 留盒内）、td2=0
（extras 不错摊）。红态由值差算术确立（旧代码 150/125/25 可复核）。

## 四、验收

- lib：**2504P/0F/2I**（2502 基线＋2 新锚；r1752 翻转绿）。
- solo：驱动案 **✓0.21%**（旧 ✗2.77%；EXIT=0）。
- corpus 全量 A/B（16596 vs /tmp/r5515 轮 685F，LC_ALL=C）：**682F**——
  GONE=5＝驱动案＋tao-129/149/150/153（在册互翻族相位回绿）；NEW=2＝
  tao-127（在册互翻族，r5392 前科）＋insert-block-in-blocks-n-inlines-middle-003
  （在册批跑噪声族 middle-001 同族）。**收账补强（11-14 续轮双二进制）**：6 mover
  base/new 两二进制各 solo×3＝18/18 全绿（GONE 4 tao 案 base solo 亦绿＝互翻
  相位非真修；NEW 两案 new solo 全绿＝批跑 storm 相位 corpus 起跑 load≈250）。
  **真态 FIXED=1 REGRESSED=∅**。
- make reftest：**725/725**（521 layout＋204 text）。
- make test：3912P＋**7F＝js_dom_bridge 对端在册族恰集**（img/svg settle 2＋
  fetch 2＋websocket 2＋iframe-sw 1；R5513 同集先例口径，改动面 table/tree
  零涉网络桥面；workspace 腿止于 engine 包，layout-engine lib 单包复验
  2504P/0F，/tmp/r5516/make-test.log）。
- fmt 净＋clippy `-p zero-layout-engine --all-targets -D warnings` 净（wipe 后
  由 make test 重编译复盖）。

## 五、挂账与下一步

- **新残差（亚阈在册）**：cell B 幽灵列 9.6px（空 div 匿名 cell auto 宽＝9.6
  应 0）→ 驱动案残差 0.21%；案转绿不追。
- **新域记档（不点名不开工）**：taffy-local 隔离盒（margins_isolated）高
  度漏末子 margin-bottom（本表 50 vs 100）——ZW 侧 aware floor 已兜；taffy
  侧若补齐可拆 floor（动 block.rs 高度求和，深域）。
- R5099 记档候选 **销账**（本候选即驱动案）。R5418 挂账①后继面维持：IFC
  行进域（深域，不 piecemeal）。
- 下一步 R5517：①corpus FAIL 盘点续择轻（682F 清单 /tmp/r5516/fails-r5516.txt；
  避开行进新域/在册深域/待决策面/动态域/tao 互翻族；候选先查在册记录）②维持项
  ＋对端挂账观察。

## 六、本轮动作与卫生

- 取证插桩（position_cells 三处 eprintln）git 状态还原（手动摘除，真修复同
  文件保留）；探针页 3 枚用后删（wpt-data gitignored）。
- target-disk-guard 分级清理在 corpus 入口触发（97GB 超阈，增量缓存先行、
  全量后清）——零编译取证窗口教训再证（R5515 同）。
- 对端双 clone storm 期（load≈250）重腿纪律：单重腿串行（corpus→reftest→
  make test），无并行重腿撞窗。
