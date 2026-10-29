# R5331 — column flex 容器内在主尺寸 item 内容贡献探针（css-flexbox-1 §9.9.1 × §4.5）

- **日期**: 2026-10-29
- **驱动案**: `css/css-flexbox/flex-minimum-height-flex-items-029.xht` ✗8.33% → **✓0.00%**（树形与 ref 完全一致：outer 100 / inner 100 / 两孙项各 50）
- **corpus**: 803F（R5330 交付态）→ **802F**（16590 案；**FIXED=1**，REGRESSED=0），通过率 15788/16590 = **95.2%**
- **域**: flex 容器内在主轴定尺 × taffy InherentSize style 注入
- **代码**: `deaa29eff`（fix(layout): column flex 容器内在主尺寸探针——item specified 主轴尺寸不注入内容测量）

## 定性

029 结构：OuterFlexbox（column，auto 高）> InnerFlexbox（column，`flex:1 0 0px`，
**height:500px**）> 两个 50px 内容子项。期望外层 auto 高 = Σ item 内容贡献 =
100；ZW 渲 500——外层被中层 item 的 specified 高顶替。（R5330 挂账时定性 029 为
「嵌套 column flex 容器 min-content 高另臂」，本轮实证真根因不在 min-content
测量臂本身，见下——挂账定性修正。）

**手工 taffy 推演三度落空**（三种 available_space 分支都无法推出 500）后改单点
观测：extract_layout 阶段临时插桩（`ZW_R5331_DIAG` eprintln，已回退）证明
**500 是 taffy 原生输出**——不是 ZW 后处理回填（与 R5330 的 postprocess 根因
不同族）。读 `compute_flexbox_layout` 入口闭环根因链：

1. 容器 auto 主轴定尺 → `determine_container_main_size` 内在臂
   （MaxContent/MinContent 分支）按 item 内容贡献求和；
2. 测量 item 用 `SizingMode::InherentSize` → `compute_flexbox_layout` 的
   `clamped_style_size`（仅 InherentSize 时计算）把 item 显式主轴定尺
   （height:500px）经 `styled_based_known_dimensions = known_dimensions.or(...)`
   注入；
3. 中层 item 以 known height 500 布局，测量返回 500 非 content 100；
4. 外层 auto 高 = Σ = 500 ✗。

规范（css-flexbox-1）：**§9.9.1** 容器内在主尺寸 = Σ item **hypothetical main
size**；item specified size 只能经 **automatic minimum**
（min(specified, content)，§4.5）进入，不得经 style 覆盖顶替 content 贡献。
column 臂贡献公式：`content_main_size.max(item.flex_basis).maybe_clamp(style_min,
style_max)`。

## 实现

三件套（kill-switch `ZW_FLEX_INTRINSIC_MAIN_PROBE=0` 回退，default-on，
LazyLock 进程级单读——R5108 教训）：

1. **`LayoutInput::flex_intrinsic_main_probe` 旗标**
   （tree/layout.rs）：真值时被测节点自身的 specified main size 不注入
   ComputeSize 结果——`compute_flexbox_layout` 的 `clamped_style_size`
   在主轴 `size.set_main(dir, None)` 失效。**cross 轴照常注入**（wrap 分栏
   宽度解析依赖）；后代节点不受影响（各自 inputs 不携带旗标）。
2. **窄门 + 探针测量**（flexbox.rs `determine_container_main_size`）：
   `use_intrinsic_main_probe = !is_row ∧ FLEX_INTRINSIC_MAIN_PROBE ∧
   item.size.main(dir).is_some() ∧ flex-basis definite（CompactLength
   LENGTH|PERCENT|CALC_TAG）`——命中走新 ext 方法
   `measure_child_intrinsic_main_size`（tree/traits.rs，旗标置位）。
   **row 臂不开放**：row（inline 轴）容器内在宽已按 Chrome/WebKit/Firefox
   校准（csswg#1435、flex_basis_unconstraint_* gentests），specified 宽注入
   是该共识行为的一部分。
3. **min 钳**：探针测得 true content 后
   `clamped_main_size = measured.maybe_min(item.size.main(dir))`——
   automatic minimum = min(specified, content)。

**min 钳是 030 回归驱动**：首轮 corpus（裸 content 探针）**804F、REGRESSED=1**
`flex-minimum-height-flex-items-030`——其外层 `width:100%` + item
`height:70px` × content 200px，期望贡献 min(200, 70) = 70，裸探针给 200 ✗。
加 maybe_min 钳后 030 保持 ✓0.00%、029 ✓0.00%。二环缺一不可，构成双 mutant
判别面。

`block.rs` 一处 LayoutInput 字面量补 `flex_intrinsic_main_probe: false`；
`traits.rs` 现有 measure_child_size / measure_child_base_size /
measure_child_size_both / perform_child_layout 四处同补（缓存键 PartialEq
新增字段自动参与 compute_cached_layout 判等）。

## A/B 与回归

- 基线 = R5330 交付态 803F（fail-list 排序版 `/tmp/r5330-base-clean.txt`），
  同树终态 **802F**（`/tmp/r5331-corpus-full2.log` → `/tmp/r5331-after2.txt`）：
  集合 diff 恰 {−flex-minimum-height-flex-items-029}，REGRESSED=0。
- kill-switch =0 复现基线红（029 回 ✗），双向可判。
- 首轮（无 min 钳）804F REGRESSED={030} 实证第二环必要性，钳后收口。

## 回归 pin

+4 pin（`crates/layout-engine/src/engine/tests/r5331_flex_intrinsic_main_probe_tests.rs`，
compute helper：parse_html → StyleSystem 800×600 → LayoutEngine::compute）：

1. `r5331_col_flex_item_specified_height_probe_clamps_container`——029 形：
   嵌套 column，item height 500 × content 100，outer 高 ≈100、inner 高 ≈100
   （无探针时 outer=500，杀 ① 腿）；
2. `r5331_col_flex_item_specified_height_below_content_takes_min`——030 形：
   content 200 × height 70，outer ≈70、叶 ≈100 的 min(specified, content)
   语义（无 min 钳时 outer=200，杀 ② 腿）；
3. `r5331_col_flex_item_auto_basis_keeps_specified_height`——**守卫腿**：
   flex-basis auto（非 definite）时探针不启用，item height 70 保持
   specified 注入行为（两态均过）；
4. `r5331_row_flex_item_specified_width_contributes`——**守卫腿**：row 容器
   float flex（shrink-to-fit 触发同一内在宽路径）specified 宽 200 照常贡献
   （两态均过；row 被窄门排除）。

**双 mutant 判别**：只关探针测量（窄门短路）→ ① FAIL；只去 min 钳 → ② FAIL
（首轮 corpus REGRESSED=030 独立实证）。守卫腿两态均过属窄门单侧判别面。

## 门禁

- fmt：本轮 6 文件 0 漂移（现存 `crates/engine/src/js_dom_bridge/text_encoding.rs`
  drift = 并行流移交项，阻塞 `make test` fmt 前置门，维持移交）。
- clippy：`cargo clippy --workspace --all-targets -- -D warnings` 0（guarded）。
- **zero-layout-engine lib 2258P/0F**（2254+4，2 ignored 既有）。
- workspace 测试腿（guarded `--compile-first --per-proc-mem 4 --total-mem 8`）：
  - 首轮全 workspace：fmt 前置门被并行流 text_encoding.rs drift 阻塞（共享面
    不越界修），按 R5330 口径直跑 workspace 腿；
  - 二轮 8 binary 截断 + **zero-webdriver 11F「未就绪」**（spawn 的
    zero-webdriver 5s TCP 未就绪，`http_session.rs:47` panic）→ clean-HEAD
    stash 实证同红 = **既有/并行流新家族**（疑 PR #254 落地或环境态），与本刀
    无关——移交账本；
  - 终轮 `--exclude zero-webdriver` 跑至底：45 binary、3598P，唯一红 =
    zero-engine fetch 10F 逐项一致（既有家族零位移）。
- product-smoke：welcome **15.68%** ≤ 20% 零位移 + struct PASS + legacy
  42 fixtures struct 0 失败。
- **bench-gate 定向 zero-layout-engine：GATE PASS，25 指标全 PASS，NEW=0**
  （taffy-local 无自有 BENCH_SPECS 条目，端到端布局开销由 zero-layout-engine
  layout_bench 覆盖；flex_layout_1000_elements 3.10ms 预算内）。

## 挂账与菜单

- **收口**：flex-minimum-height-flex-items-029（R5329/R5330 两轮挂账）；
- **菜单（未驱动，勿臆断同修）**：
  - flex-minimum-width-flex-items-015 ✗6.25% = aspect-ratio transferred
    minimum × % min-height 解析（R3859 谱系延伸，与本刀不同臂）；
  - **taffy automatic min 短板维持**：029 修复只覆盖容器内在主尺寸臂——
    automatic min 本身的 content size suggestion 测量仍走 basis-sums 分支
    （min-content 测量未真回 content），flex-aspect-ratio-026 的回填补偿位
    依赖仍在，撤销须先补 content-suggestion 臂；
  - **zero-webdriver 11F「未就绪」**：workspace 腿 spawn 5s TCP 超时
    （clean-HEAD stash 实证同红），疑 PR #254 落地或环境态，移交定性；
  - **inline-flex 内在宽 0 观察**：守卫腿首选形态（span>div inline-flex
    shrink-to-fit）端到端宽 0，kill-switch 两态一致 = 既有现象，与本刀无关，
    另立定性；
  - 维持：zero-engine fetch 10F + details 闪断、R5327 fragment 臂结构域、
    safe/unsafe overflow-position 域、content-none-select-1 脆弱族、
    c44-003 残差、va 族余案、c548 待用户决策、fmt drift（text_encoding.rs）
    移交。

## 下一步 R5332

1. 802F 账本续扫择轻（避开 va 域与 OOF 锚域）；
2. flex-minimum-width-flex-items-015 分诊驱动择机；
3. multicol 嵌套分片专项预研（8×1.04% 聚簇）。
