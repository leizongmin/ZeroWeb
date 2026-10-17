# R5168 证据：§17.2.1 匿名 table-cell 网格预留 + 合成盒（anon-091/133/134 翻绿）

日期：2026-10-18。基线：R5167 tip（c148b9d9c）。kill-switch：`ZW_ANON_CELL_WRAP=0` 完整回退本行为集（合成 pass + build_row DOM 遍历网格预留一并关闭；LazyLock 构造期单读，判别经外部 env corpus OFF 轮验证）。

## 域核（088/091/092 缺陷定位，探针实证修正早期假设）

R5167 corpus A/B 外露 anon-088/091/092（1.23/1.05/1.73%）。探针（step-8 盒树 dump）修正早期假设：

1. **两形态**：shape A（088）——行内裸 inline span 走 R2160 multi-inline 跳过、step 8 **无盒**（step 12 才由 backfill 回填代理盒）；shape B（091）——inline 子在 step 8 **已有盒**。
2. **shape A 真缺陷 = 网格列分配**：旧 `build_row` 盒序遍历只见 boxed cell（裸 inline 无盒不可见）→ 显式 cell 挤到 col0（ref 在 col1）、列数 2 vs ref 3。非「盒合成缺失」。
3. **shape B 缺陷 = 无 display 过滤**：`build_row` 把行内每个 inline 盒当独立 cell 各占一列（4 列 vs ref 3），且裸 span 盒几何为 taffy 交换帧 junk。

## 实现（table.rs 减负：新模块 table_anon_cell.rs，table.rs 2553 行已超 2000 限）

1. **合成 pass**（`table_anon_cell.rs::synthesize_anon_table_cells`，`adjust_table_layout` 入口、一切 `layout_table` 之前）：对每个 display:table-row 盒（**水平域门**，vertical FIXME 挂账），把连续 inline 级子 drain 进合成 cell 盒插回原位——`node_id` 借宿主行（R109 block-mixed 片段盒先例）、`fragment_node_ids` 圈定被吞并子（collect_items.rs:334 现成接线）、`is_block_level` 放行 compute_final、`is_anon_table_cell` 语义旗标（防递归：walk 的 is_row 判定排除旗标盒）。
2. **build_row DOM 序遍历**（table_types.rs，盒序旧路径保留为 `build_row_box_order` fallback：无 node_id 行盒 / kill-switch OFF / vertical 行 / DOM 空而盒非空防御）：行内连续 inline 元素 + 非空白文本节点为一个匿名 cell 预留 1 列——占位 entry `child_index = usize::MAX`（get_cell_box 落 None，消费方 let-else/if-let 全 inert）+ `anon_max_width = fragment_inline_max_width(行样式, run ids)`；行内存在合成盒（shape B）时按 fragment 相交接真盒。
3. **列宽**（`compute_column_widths` Pass 1）：占位 entry 列宽 = `anon_max_width`（run 文本 max-content，§17.2.1 匿名 cell 内容参与列尺寸）。
4. **cell_x 推进**（`position_cells`）：占位 entry 无盒，在盒查找前按其列宽推进 `cell_x`（缺此步显式 cell 回落 col0——首轮 x=0 实证根因）。
5. **IFC/paint 对接**（inline_finalization.rs）：行子含合成 cell ⇒ 行的 `is_table_internal_with_text`（R978 臂）跳过（防整行 DOM 跑 IFC 双绘）；合成 cell 自身按盒子非 OOF 子放行 `has_text_children`。

## 验证

### 回归测试（engine/tests/r5168_anon_cell_wrap.rs，2 案）

- `r5168_row_explicit_cell_lands_after_inline_run_column`：088 形态 2 行孤儿 row（merge_orphan_table_run 路径），row1 = [inline span × 2, cell]——显式 cell 落 col1（x>100，修复前 0）。
- `r5168_anon_cell_stores_ifc_font_metrics`：合成 cell 存储行继承 32px IFC 字号度量（paint Path B 字号来源）。
- kill-switch 判别经 corpus OFF 轮外部 env 承担（LazyLock 进程级快照，进程内 set_var 不可靠——R3858 教训）。

### corpus ON/OFF A/B（16,590 案，同 binary 三轮）

| 轮 | fail | 备注 |
|---|---|---|
| OFF（`ZW_ANON_CELL_WRAP=0`） | 1,115 | anon-091 红（基线态）；uri-017 亦红（见抖动标注） |
| ON 首轮（无 vertical 门） | 1,118 | **inline-table-alignment-002..005 翻红 7.9-10.1%**（vertical 表合成未对接，R4470 bare-row 臂被扰）→ 收窄 |
| ON 终版（水平域门） | **1,114** | **终态** |

- ON 终版 ↔ R5167 基线 fail 清单 diff：**翻绿 3**（anon-091 1.05→0.00、anon-133、anon-134——133/134 系 R5167 挂账的 infer-second-row 行组变体，本轮匿名 cell 网格预留顺带修复）、**翻红 0**。
- ON ↔ OFF diff：ON-only 翻红 0、OFF-only 翻红恰 anon-091（ON 翻绿），kill-switch 判别成立。
- **抖动标注**：css/CSS2/syntax/uri-017.xht（5.85%）ON corpus 翻红，但 OFF corpus 与 standalone（ON/OFF 双态）均 fail——上下文相关抖动，同 R5167 white-space-collapsing-bidi-001 类，不计本轮回归。
- **088/092 未翻绿**（1.15/1.65%，较 R5167 态 1.23/1.73 微降）：网格列已对齐（lib 测试钉住），余差为匿名 run 文本绘制（marker cell 无盒、行级 IFC 串联绘制 vs ref 逐列定位）→ R5169 专项。

### 门禁

- fmt / clippy（-p zero-layout-engine --all-targets -D warnings）：清。
- layout-engine lib：1,980 全绿（r5168 2 案新入库）。
- `make reftest`：712/712（100%）。
- product-smoke：welcome oracle diff 17.07%（≤20，与 R5166/R5167 同值）+ 全部 struct PASS。
- `make test`（分腿跑全，工作区腿被预存红阻断后续 recipe，余腿逐条补跑）：
  - **唯一真实失败 = 预存红** `reftest::resources::tests::extract_css_urls_unterminated_at_style_close`（workspace 腿 223P/1F + quickjs 腿 154P/1F 同案）：**clean main（stash 反证）即红，与 R5168 无关**。根因已定位——zero-engine `extract_html_style_text`（extract.rs:809）给每个 `<style>` 块追加 `\n/* ) */\n`，**前导换行**落进未终止 quoted url 字符串（css-syntax-3：字符串内裸换行 → bad-string）→ EOF 闭合语义被毒化 → uri-017 的 swatch-green 资源抽不出（域核探针：style_text 正确、urls=[]）。R2134 单测期望（EOF 闭合）与现生产行为（bad-string）冲突；任何后缀（含尾部 `\n`）都会毒化 quoted 案 → 修复须 toknizer EOF-close + 后缀策略重设计，独立切片（本 slice 不越域）。
  - browser tests 腿 641P/2F：`window_surface_present_smoke` + `paste_and_go_blurs_and_clears_preedit`——环境面（headless display/剪贴板前置），isolated + xvfb-run 前缀双 PASS（2c2a675cb 先例同判；--tests recipe 行缺 --bin 腿的 ZW_GUARD_RUNNER_PREFIX，测试基建缺口另记）。
  - 其余腿全绿：renderer lib 272P；browser bin 643P；quickjs script-sandbox/webview/webview-demo/integration 240P+701P；quickjs renderer 270P。
- bench-gate 定向 zero-layout-engine：**ON 8 FAIL / OFF（`ZW_ANON_CELL_WRAP=0` = pre-R5168 行为）6 FAIL，双轮 NEW=0**；OFF 同族同量级（block_layout_1000 ON 3.89ms vs OFF 3.98ms——**OFF 反更差**；无表基准两态同 FAIL）→ 跑窗与并行流（html-semantics face 独立 clone testharness 重载）重叠噪声族延续（R5149-R5167 同判），代码中性，不 relax，空闲复跑挂账。

## 挂账与下一步

1. **R5169 专项**：匿名 cell run 文本绘制——marker 列文本经 cell 级 IFC（fragment_node_ids 圈定）逐列定位（088/092 直接受益；合成盒 Shape B 存储路径已接线）。
2. **uri-017 资源抽取预存红修复**（本轮域核已定位，测试基建/引擎 extract 域）：`extract_html_style_text` 的 `\n/* ) */\n` 后缀毒化 quoted-string EOF 闭合——tokenizer EOF-close 语义 + 后缀策略重设计（任何含换行后缀均毒化），修复后 R2134 单测与 corpus uri-017（抖动）同步受益。
3. vertical 表匿名 cell 合成/网格预留（inline-table-alignment 族同域勘察）。
4. 单行孤儿 row retrofit 路径（build_row 不经 merge、direct-cells accumulator 无 inline run 预留）；纯匿名 cell 行行高（marker 高 0）；marker 列文本换行按列宽（行级 IFC 用整行宽）。
5. browser tests 腿 recipe 缺 `ZW_GUARD_RUNNER_PREFIX`（--bin 腿有、--tests 腿无）——display 依赖用例在无头机恒红，测试基建补齐另案（test-infra 域）。
6. rtl 表列序（direction-applies-to-013/014）、bg-clip:text paint 域、既有挂账不变（row-2/multicol-2/动画 RFC 用户门、content-visibility、collapse 半宽 fidelity FIXME、converter 表宽双语义、019 行尾锚迁移 FIXME、R5166 自身挂账）。
