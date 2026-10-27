# R5302 — text-overflow: ellipsis 逐行适用 + 整 cell 收缩 + 贴排省略号（text-overflow-007 ✗1.30→✓0.00 翻绿，008 回归防护，013 结构定谳）

**日期**: 2026-10-27
**驱动案**: `css/css-ui/text-overflow-007.html` ✗1.30% → **✓0.00%**（R5301 家族地图 007/013 域）
**corpus**: 交付态 **16592/15737P/855F**；失败集 vs 基线（16592/15736P/856F）归一化
diff = {−text-overflow-007} 恰一案移出零翻转。
**基线说明**: 本轮起于跨 session WIP（前会话已写逐行骨架 + 3 单测后中断），本轮完成
并修正三处后定稿。

## 实现（text.rs ellipsis 后处理重写，无新 kill-switch）

- **逐行适用**（css-overflow-3/css-ui-3 §ellipsing：每条溢出行独立截断）：按 baseline y
  聚类分行（容差 0.5×font_size），逐行独立检测溢出、收缩、贴省略号。此前单趟从首溢出
  行截点清到切片末尾。
- **整 cell 收缩 + 贴排**（007 assert "no intervening gap"）：LTR 保留使「前缀末端 +
  '…'」装进容器的最大整 cell 前缀，'…' 贴前缀末端（非锚 content_right−宽）；检测改为
  行末端点（末字形 origin + 实测推进）越过 content_right——origin 判据会把 straddling
  末 cell 漏成不溢出。cell 推进量 = x_{i+1}−x_i（真实排版推进）；行末按字体实测。
- **RTL 逐行镜像**（R5301 语义保持）：检测 = 行首字形严格越过 content_left；保留使
  「后缀 + '…'」装进容器的最大后缀，'…' 贴后缀左缘；全不保时锚 content_left。
- **省略号装不进容器 → 不渲**（新增，008 防护）：`'…'宽 > 容器宽` 时整个 pass 跳过
  （不裁不渲，溢出交还 overflow:hidden 裁剪）。
- **省略号颜色 = 本盒 color**（前会话 WIP 曾取末保留字形色，本轮修正）：css-ui-3 语义
  '…' 随盒样式——008 assert「省略号出现即红（styled after the block）」、013 红 sub
  span 被替换后 '…' 仍绿。基线 y 仍取本行聚类首槽（013 「'…' 与块基线对齐」assert）。
- **fs=0 哨兵槽参与行聚类与行宽检测；不参与保留**：IFC/布局侧先把溢出盒外字形置零
  （x/y 原样保留，013 取证），排除哨兵会让行末端点缩到盒缘、溢出恒漏检；保留臂遇
  哨兵即止（已不可见，保留无意义）。

## 取证链（本轮三修正的证据）

1. **008 回归定位**（WIP 首跑 008 ✓0.00→✗2.08）：盒 0.5em=50px、单 cell 100px。旧
   origin 判据（`g.x >= content_right`）漏检 → 无巧合成正确；endpoint 判据正确判溢出
   后无条件渲 '…' → 半截红 '…' 露出（chromium：'…' 装不下 → 纯裁剪，008 assert
   「首字符 clipped rather than ellipsed」）。→ ell-fit 门。
2. **011 回归定位**（✗8.33→✗10.42）：字形探针定谳 '…' 被末保留字形染橙（ZW 行内
   RTL 无 bidi 重排，保留侧 4 个 orange x）→ diff 从 4 cell（8.33%）涨到 5 cell
   （10.42%）。→ 颜色改本盒 color 回 8.33 地板（该案剩余 = bidi item 裂解阻塞域，
   R5299 阻塞①维持）。
3. **013 无变化定谳**（✗10.42 → ✗10.42）：字形探针 + HEAD A/B——(a) IFC/布局侧先把
   盒外字形置零（哨兵槽 x/y 保留）→ 活字形行末端点恒 = 盒缘 → 溢出漏检、'…' 永不
   触发（新旧同病）；哨兵入聚类修 (a)。(b) **结构性阻塞**：013 全部字形由 span（IFC
   owner）绘制，text-overflow/overflow 非继承 → span 自身 overflow_x=Visible/
   text_overflow=Clip → **ellipsis pass 在 span 上不跑**；div（宿主）has_content=false
   同样不跑（gate 探针实证）。→ 需 ellipsis host 传播（R3768 line-clamp host 布局侧
   标记先例），**挂账深结构轮**。
4. **ruby 1.84→2.05（红页内变化，零翻转）**：reftest dump PNG A/B——test 侧渲染逐字节
   同（ruby 结构 glyph owner 不达 pass）；ref 侧（嵌套 span 结构）1436px 变化 = 逐行
   '…'（每行 "a…"）替换旧 origin 判据 sliver——**ref 侧行为更近 chromium**，ruby 结构
   test 侧（ruby 注释域）尚未跟进，|test−ref| 暂涨。corpus 口径零翻转（ruby 双侧皆红
   隔离域）。

## 家族地图更新（failset text-overflow 余案）

| 案 | 状态 | 缺口 |
|---|---|---|
| text-overflow-007 | **✓ 本轮闭合** | — |
| text-overflow-008 | ✓ 维持绿（本轮加 ell-fit 门防回归） | — |
| ellipsis-rtl-001 | ✓（R5301） | — |
| text-overflow-011 / with-selection | ✗8.33 / ✗0.06 | bidi item 裂解（R5299 阻塞①） |
| text-overflow-013 | ✗10.42 | **ellipsis host 传播**（见取证 3，结构轮） |
| scroll-001 / scroll-rtl-001 | ✗6.25 | scroll 容器语义 |
| text-overflow-ruby | ✗2.05（轮前 1.84） | ruby 注释域（ref 侧已更近 chromium，见取证 4） |

## 交付

- 007 **✗1.30%→✓0.00%**；corpus 交付态 **16592/15737P/855F**，失败集归一化 diff 恰
  {−007} 零翻转。全量二跑 diff 空（判定稳定）。
- 回归 +4（paint tests/visual_inline.rs）：`r5302_ellipsis_multiline_each_overflowing_
  line_ellipsized`（两溢出行各渲一 '…'）、`r5302_ellipsis_adjacent_to_kept_prefix_not_
  flush_right`（007 no-gap 贴排）、`r5302_ellipsis_straddling_last_cell_triggers_elision`
  （跨缘末 cell 整裁不留 sliver）、`r5302_ellipsis_wider_than_container_not_rendered_
  clip_only`（008 ell-fit 门）。

## 门禁

make reftest 716/716（512 layout + 204 text）、fmt 0 diff、clippy 0 告警（zero-engine
--all-targets + quickjs 组合 14 crate `-D warnings` 腿）、zero-engine lib **3557P/0F**
(+4)、full corpus 16592/15737P/855F 失败集恰 {−007} 零翻转、product-smoke PASS
（welcome oracle 15.50% 持平 + struct-check ×2 viewport）、bench-gate 定向
`ZERO_WEB_BENCH_CRATES=zero-engine` **GATE PASS**（19 指标 NEW=7 全在预算内）。
make test 全量未跑（改动面 = zero-engine paint ellipsis 后处理 + 同 crate 测试，
引擎消费面由 reftest/corpus/product-smoke 覆盖，R5301 同判例记录在案）。

## 挂账

- **ellipsis host 传播**（013 域，结构轮）：宿主块（overflow+text-overflow 盒）的省略
  号参数（content_right/container_width/is_rtl/color）需传播到 IFC owner inline 盒的
  paint pass——R3768/R3770b line-clamp host 布局侧标记先例可参照；取证 3 的 gate 探针
  结论可直接复用。
- R5299 翻转阻塞三项维持（bidi item 裂解、backdrop 合成语义、度量缓存）；R5298 遗留
  维持（容器 strut 域、ruby anon 继承）；ruby 域 test 侧跟进（ref 侧已前移）。
