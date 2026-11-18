# R5524 border-collapse 域 chromium CDP 几何捕获（R4031-N 放行流程执行）

**日期**: 2026-11-14
**chromium**: Chrome/156.0.8078.73（Debian 13 系统 chromium，`manifest.json` 含 userAgent）
**工具**: [capture-collapse-geometry.mjs](../../../../../tests/wpt-runner/scripts/capture-collapse-geometry.mjs)（新增，CDP Runtime.evaluate 盒几何提取，无删除路径）
**视口**: 800×600（与 runner `reftest.rs` / oracle-shots 捕获同口径）
**授权**: 待用户决策清单「table collapse 几何 RFC 批」——R4031-N 六轮像素逆向穷尽后用户 2026-09-08 批复「点名执行」捕获流程；**本轮执行**。

## 捕获范围

授权时族 = ~16 案（1.3-9.9%）；当前真态（R5521 基线 681F）族仅剩 2 案（14 案已被后续轮次修复），捕获按现族执行：

| 案 | 当前差 | 产物 |
|---|---|---|
| `css/CSS2/borders/border-conflict-style-107.html` | 在册红 | `css_CSS2_borders_border-conflict-style-107.json` |
| `css/css-backgrounds/box-shadow-table-border-collapse-001.html` | 2.34%（R5264 记 2.82→2.34 附带改善） | `css_css-backgrounds_box-shadow-table-border-collapse-001.json` |
| 两案 ref（green-200px-square / 同名 -ref） | — | 各 1 JSON |

## 地面真值要点（修复切片 ②③④ 的量化目标）

### 中心线模型实证（半边框外伸）

两案一致证实 CSS2.1 §17.6.2 collapsed 模型的**中心线盒**几何：table border-box 边 =
塌陷赢家边框**中心线**，渲染边框带以赢家宽度居中于该线、半宽外伸出 table 盒：

- **box-shadow-table-border-collapse-001**: table rect `(8,68,160×160)`；外缘赢家宽 30px
  （tr 声明 30px solid，宽度劣先胜过 table 声明的 29px）→ cell 网格原点 `(23,83)` =
  `8 + 30/2`；td 网格 2×2 每格 `65×65`（第二格 x=88、第二行 y=148），tbody rect
  `130×130`；table 盒 160 = `15 + 130 + 15`。
- **border-conflict-style-107**: 每表 winner 25px → tbody 原点 = table 原点 + `12.5`
  （如首表 table `(8,50,50×50)` → tbody `(20.5,62.5,25×25)`，0.5 亚像素值如实捕获）；
  table 盒 50 = `12.5 + 25 + 12.5`。

ZW 侧当前差（box-shadow 案 2.34% 残差）对应带定位/盒边界与该模型的小偏差；中心线
迁移四切片（①cell intrinsic 去拉伸 ②collapse 边框中心线行/列 sizing ③cell 盒几何
④paint 边框定位）的目标几何自此有实测锚。

### 冲突消解（107 案矩阵）

- 同宽（25px）同式（solid）冲突：**cell 声明色胜 tr 声明色**（首表 tr=red vs td=green →
  渲染 green），与 §17.6.2.1 tie-break「cell > row > row group > column > column group >
  table」一致；ZW `resolve_collapsed_borders` 的 tie-color 单测口径与此相符，案级残差
  在盒几何而非消解逻辑（与 R5203 像素定谳「冲突消解/tie-break 逻辑正确、差值纯几何」一致）。
- 宽度劣先实例：table 声明 29px < tr 声明 30px → 外缘渲染 30px（赢家宽进几何，各元素
  computed 仍保留自身声明值——extract 的 border 三元组为声明值，渲染赢家宽从盒原点差反推）。

## 修复轮状态注记

- **后置约束已解**：layout-perf-pullback 专项 2026-09-11 §4 解阻判据落地（六微基准
  p50 全部入预算，R44xx 记账），「collapse 修复排在 perf 收敛之后」条件满足。
- **迁移本体仍等点名**：四切片为 multi-session 深切片（engine/table 域 RFC 级），
  按清单纪律不自主开工；捕获产物为其前置输入，随本提交入库。
- 授权范围注记：条目原文「~16 案所需几何」按现族 2 案执行；若未来 WPT 批导入或
  回归使族重新扩大，工具可直接对案清单复跑（`node … capture-collapse-geometry.mjs <相对路径>…`）。

## R5525 补充验证（2026-11-14）

- **工具确定性成立**：SL-008 修复后的脚本（产物回执只写仓库相对路径）复跑同案清单
  4/4，四个几何 JSON 与 evidence 归档副本**字节级一致**（`cmp` 全 IDENTICAL）——
  chromium 对这些案的 layout 确定性与工具复现性均成立，条目内「工具可对案清单复跑」
  的口径自此有验证背书。
- **ZW 侧几何量化推迟到迁移轮**：两案均依赖 Ahem（cell 高度 = 50px 内容 + 中心线
  偏移，字体度量与模型偏差交织）；layout-engine 临时单测探针不带 corpus 字体
  harness，跑出的差值会把字体归因和模型归因混在一起（estimate 字体 vs 窗口真值
  教训：半混淆数据比无数据更误导）。ZW-vs-chromium 逐盒差值表留给被点名的迁移轮
  以正规 harness 产出。

## R5526 族内容级普查扩展（2026-11-14）

- **普查方法与结果**：对当前真态 681F（R5521 基线）逐案 grep 源码 `border-collapse`——
  4 案命中：已知 2 族案 + **border-right-width-applies-to-006**（新入族，账本归因
  「table-column border collapse 绘制整特性缺失（96×96 全块）＋resolve cell>row/rowgroup
  与 table-column border 臂输」，同属迁移切片 ②④+resolve 覆盖链域；R5524 按案名
  grep 划族时漏计——案名不含 collapse/conflict 关键词）+ ruby-inlinize-blocks-002
  （ruby 带在册案，collapse 系 incidental，**非族成员**）。族界定＝账本归因过滤后
  **3 案**。
- **工具盲区二修复**：006 为 CSS 表布局案（div+display:table-column/table-cell，无真
  table 标签），tag 选择器首轮 0 盒——提取段扩为真标签 ∪ computed display∈table-*
  族（保持文档序）。**扩展不扰动既有捕获**：原 4 案复跑仍与归档副本字节级一致。
- **006 地面真值（Chrome/156.0.8078.73）**：chromium 给 `display:table-column` 生成
  phantom 盒（div#test rect `(8,50,48×96)`，computed border-right 96px solid）；表盒
  96×96 无自身边框、cell 网格双列各 48×48 **无边框内缩**——96px 列边框作为 collapse
  赢家带绘制在列界 x=56 中心：带域 8..104 恰好覆盖全表盒（56±48）→ 全黑方块＝ref
  （ref-filled-black-96px-square）期待。迁移切片 ②（列 sizing）④（paint 边框定位）
  + resolve 覆盖链的目标几何自此含 table-column 臂实测。
- 产物：+2 JSON（006 + 其 ref）+ manifest 刷新为 6 案全量；捕获命令同 R5525 复跑口径。
