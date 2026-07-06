# 历史轮次归档 — R894–R990（multicol Phase 2 / harness JS vein / R109 §9.2.1.1 backfill / aspect-ratio / R990 ascent era）

> 内容 100% 保留自主控文档 `master.md`：`## 下一步` 节中 **R894–R990 升序逐轮详记块**逐字迁出（仅作归档，未去重、未重排）。下文为该 era 完整逐轮详记。回主控文档：[`../master.md`](../master.md)。

---

### R894 DC-12 feature 实测验证（2026-06-30，真进展：4 项 DC-12 checkbox 勾齐）

承接 R893 「转向 DC-11/12 feature 验证」，**实测验证 DC-12 高级视觉效果的真实状态**（不轻信 M9 声称）：
- **backdrop-filter 实测验证（渲染 with-vs-without）**：fixture = gradient 背景 + overlay `backdrop-filter:blur(15px)`；ZeroWeb 渲染 with.html vs without.html（无 overlay blur）diff = **15314 px 恰落在 overlay 盒 y[0,80] 带、带外 0 px** → blur 效果**正确限定在元素盒内**，backdrop-filter **真实工作**（painter/effects.rs）。
- border-image（painter/mod.rs + paint/tests/border_image_repeat.rs 单测）、clip-path（effects_indicators.rs + helpers.rs）、CSS mask（effects.rs）均有 impl + 单测（make test 绿），empirical 已验证。

**goal doc DC-12 checkbox 更新（4 项勾齐）**：border-image / clip-path / backdrop-filter / CSS mask 全部 ✅（原 stale 未勾）；残余未实现 = scroll-snap（宿主层）+ @media print（可选低优先）。同步纠正 goal doc 两处 stale 缺口表（Support Envelope + 已知关键缺口）+ milestone note 的「clip-path/backdrop-filter/mask 未实现」描述（→ 已实现 M9/R894），消除文档自相矛盾。

**★ DC 进度更新**：DC-12 现状 = text-shadow/多背景/重复渐变/border-image/clip-path/backdrop-filter/CSS mask 全 ✅（7/9），残余 2 项（scroll-snap 宿主层、@media print 低优先）。**渲染兼容性目标的 feature/infra DC 现大多达成**（DC-1/6/7/8/9/10/13/14 ✅，DC-11/12 大多 ✅）；**唯一远未达成的 = DC-2~5（WPT ≥95%）**，受 fontdue-vs-Skia 光栅 strict 噪声 + 结构性簇（multicol Phase 2 / R109 / baseline-export）主导 = 多会话硬核 + 字体光栅上限。下会话：继续 DC-11 残余（Float 完整 / sticky / scroll 容器，多需宿主层）或重启多会话硬核轨道（multicol Phase 2 spec-rfc）。

### R895 DC-11 Float 实测验证（2026-06-30，真进展：Float checkbox 勾齐）

### R896 multicol Phase 2 入口核查（spec-rfc skill 不适用 + subpixel 非窄 lever，2026-06-30）

承 R895「下一步 multicol Phase 2 spec-rfc」，核查后发现：
1. **lei-spec-rfc skill 与 unattended rally 冲突**：该 skill 要求「开发前需用户输入时，结束回复等待用户响应」+ 确认流程，与 rally 输出协议「不要向用户提问」+ 无人值守模式冲突。**裁决**：rally 的设计文档走既有 rally-pattern（如 multicol-fragmentation-design.md / phase-a-IFC-unification-design.md 直接产出，非 skill 的 confirmation flow）。
2. **subpixel-column-rule-width 非 narrow lever**：渲染实测 diff 23.39%；code 核查 column-rule-width 在 text.rs:196 用 `*w as f32` 不 round（gap 存在），但 column-rule 是细线、diff 主项 = Lorem ipsum 跨列文本分布（fragmentation），rule-rounding 修复仅影响细线像素（marginal）。**multicol 所有候选均由 Phase 2 fragmentation 主导**（nested/breaking/span-all），无 narrow 单点 lever。
3. **★ rally wpt-runner-reachable 杠杆穷尽定论**：feature/infra DC 已验证达成（DC-1/6/7/8/9/10/11-mostly/12-mostly/13/14 ✅）；reftest/oracle 侧 4 目录 7380 案 plateau 铁定（font-raster strict 噪声 + 结构性簇）；DC-2~5 strict ≥95% 受 fontdue≠Skia 光栅上限（per-glyph 0.1-3% 噪声聚合），<1% oracle 缺口 = 结构性硬核（multicol Phase 2 / R109 / baseline-export，均有 prior 证伪）。**残余可推进 = 多会话硬核架构**（非单 session）。welcome DC-13 gate PASS（16.11%<20%）。

**▶ 下会话**：multicol Phase 2 设计文档（rally-pattern，非 skill）——为 layout-side column-aware IFC 的**最窄可碎片化子集**（如单层 multicol + 单一 block 子元素 breaking 跨列，非 nested/balance）设计 `ColumnFragmentationContext` 接口 + Phase 1 死字段（净 0，守 multicol-fill-auto-001），作为多年硬核的首个 enabling slice；或转 R109 §9.2.1.1 匿名块（welcome P1，亦多会话）。勿以单 session 期望 DC-2~5 显著提升（受光栅 + 结构性双上限）。

承接 R894「feature 实测验证」纪律，核查 DC-11 Float 真实状态（master.md line 657 已记「table/float layout 已在 M4 实现」，但 goal DC-11 checkbox 未勾——stale）。**实测验证**（fixture = `float:left` 100×80 红盒 + 块级兄弟蓝 div 200px + 白文本）：
- **float 定位**：红 float 正确居容器左上 x[0,99]y[0,79]（100×80）✓
- **块级兄弟盒**：蓝 div border-box 全宽 x[0,799]y[0,199]（150678 px ≈ 800×200−float 区）在 float **之后**（CSS §9.5 正确——block border-box 不被 float 缩，仅 inline content 缩）✓
- **inline exclusion（文本绕排）**：y<80 带 x<95（float 区）无白文本、x≥100 有绕排文本 ✓
- **clear + containment**：`float_positioning.rs::adjust_float_positions(_with_context)` 的 `active_left/right_float_bottom` = clear 语义；`use_bfc_float_containment`（engine.rs）= float containment（master.md DC-11 BFC 行 ✅）。

**goal DC-11 Float checkbox 勾齐**（原 stale 未勾）。`float_positioning.rs` 是**原生 float 定位算法**（非 master.md 历史轮次 R857/R849「无原生 float」所述——那指 taffy-native-float vs ZW 手动后处理的冲突，非「float 不工作」；line 657 已正确记 M4 实现）。**残余** = CSS2 float reftest 边缘 case（结构性 plateau，R108b/R145/R881 已收主要 fix）+ sticky/scroll-snap/scroll 容器（需宿主层）。**DC-11 现状**：margin-collapse/BFC/fixed/replaced/%height/auto-margin/min-max/Float 全 ✅；残余 sticky（宿主）+ overflow:scroll-auto 真滚动容器（宿主）。

### R897 multicol Phase 2a Phase 1 死字段 LANDED（enabling slice，net 0，守 multicol-fill-auto-001）

承接 R896「下会话：multicol Phase 2 设计文档（rally-pattern，非 skill）+ Phase 1 死字段」。**完成三件**：

1. **R897 probe（A1 实证，evidence 已落地）**：单层 multicol + `column-fill:auto` + 明确高度 + **单一 block 子元素 breaking 跨列**（R109-independent slice）的真实缺口实测——Probe B（height=48px=3 行整数倍，12 行文本，column-count:3）ZW 渲染 col0/1/2 各 3 行，**第 10-12 行被静默丢弃**（不渲染不溢出），根因 = `assign_children_to_columns_with_breaking`（multicol.rs:336）产 3 fragments 覆盖 child y=[0,144]（前 9 行），循环在末列停止（`current_col+1 < col_count` 守卫）余量丢弃；Probe A（height=60px 非整数倍）每列 64px **超出 60px 列高**（overfill，列边界未裁断行）。**区别 nested case（R201「文本只在 col0」）**——本 slice 文本正确分布到各列，缺口 = assignment 丢余量 + 非整数高度 overfill（paint 垂直窗口后验裁剪无法精确算每列行盒）。**A1 假设✅实证**（区别 column-aware-IFC Phase 1 的 0-case 停止 R381）。

2. **设计文档（rally-pattern）**：[`multicol-phase2-column-fragmentation-context.md`](./multicol-phase2-column-fragmentation-context.md) v1.0——为 layout 侧 column-aware IFC 的最窄 R109-independent 可碎片化子集定义 `ColumnFragmentationContext` 接口（IFC 碎片化**输入**：列几何 + 列高预算 + 每列已占高度 + fill 模式）+ Phase 1 死字段切片（net 0，镜像 font-bridge R885）+ Phase 2a step-2 多会话接力路线（新模块 `column_fragmentation_flow.rs` 切片逻辑 + layout 注入 + 输出存 LayoutBox + env 门控 `MULTICOL_COLUMN_FRAG`）。spec-lint 23 Pass / 1 Warning（A2/A3 留 step-2，非阻塞）/ 0 Fail。

3. **Phase 1 死字段实施 LANDED（net 源码，零回归）**：
   - **新增** `crates/layout-engine/src/inline/column_fragmentation.rs`（~150 行）= `ColumnFragmentationContext` 数据结构（`col_count`/`col_width`/`col_gap`/`available_height: Option<f32>`/`col_filled_heights: Vec<f32>`/`fill_mode`）+ `ColumnFillMode { Balance, Auto }` 本地枚举 + 4 单测（字段构造 / balance 无高度预算 / IFC 默认 None 零回归 / builder 注入数据正确）。**纯数据无 trait/handle**（区别 font-bridge，纯数据可 derive(Debug)）。
   - **修改** `crates/layout-engine/src/inline/mod.rs`（+~12 行）：`mod column_fragmentation` + `pub use` + IFC `column_fragmentation: Option<ColumnFragmentationContext>` 字段（与 `font_metric_provider` 并列）+ `new()` 初始化 None + `with_column_fragmentation` builder。
   - **零生产读取**：grep 证 `column_fragmentation` 仅命中定义模块 + mod.rs 声明/字段/init/builder + tests，**无任何 layout/paint/engine 路径读取** → 渲染字节级不变。

**验证（全门禁绿）**：`cargo build -p zero-layout-engine` 绿；`make test` 全 workspace 绿（exit 0，0 FAILED，layout-engine 953 含 +4 新单测）；`cargo clippy --all-targets -- -D warnings` 干净；`cargo fmt` 干净；`make product-smoke` welcome **16.11%**（== baseline，<20% DC-13 gate）；`make reftest-oracle DIR=css-multicol` **104/452 (23%) == R893 baseline**（top-15 worst 与记录一字不差 = 零回归；multicol-fill-auto-001 sentinel 不在 worst 列表仍 <1% 通过）。

**意义**：multicol Phase 2 硬核（多年期）的**首个 enabling slice 落地**——为后续会话的 layout 侧 column-aware IFC 碎片化实施提供明确 seam（`ColumnFragmentationContext` 接口 + IFC dormant 字段），避免重写 IFC。R897 probe 把多会话硬核从「设计假设」推进到「A1 实证 + 接口基石」。与 font-bridge R885 同型 dormant 模式（net 0、grep 证零读取、守 sentinel），风险为零。

**▶ 下会话（Phase 2a step-2，多会话接力）**：① step-2 probe A2/A3（REFTEST_DUMP 单层 breaking 案 + chromium 余量 = overflow multicol 盒外语义验证）；② 新模块 `column_fragmentation_flow.rs::fragment_lines_into_columns`（消费 `ColumnFragmentationContext` 把 IFC 宽度换行行盒切片到列，整行不裁断，余量 overflow）env 门控；③ layout 侧为目标结构（单层+单 block 子+column-fill:auto+明确高度）构造 ctx 并 `with_column_fragmentation` 注入 + 输出存 LayoutBox 新字段 + paint 消费；④ 三态门禁 A/B 守 welcome<20% + scoped multicol oracle 零回归 + chromium-Oracle z_vs_chr 下降（净负即回退）。**勿以单 session 期望 DC-2~5 显著提升**（受光栅 + 结构性双上限）。

### R898 multicol Phase 2a step-2 commit 1 纯算法切片 LANDED（net 0，A2 探针确认 + 实测基线核对）

承 R897「下会话 step-2 探针 + commit 1」。**实测核对当前状态**（不信过时文档）：product-smoke welcome **16.11%**（<20% DC-13 gate PASS）；`make reftest-oracle DIR=css-multicol` = **104/452 (23.0%)** / credible 97 (21.5%，排除 7 退化) **== R893 baseline 一字不差**（plateau 确认）。

**A2 探针（read-only 代码分析，零风险）**：✅ **IFC 行盒可按列 budget 切片**——`LineBox`（`inline_types.rs:155`）携带每行 `y` + `height` + `runs`，IFC `layout()` 产出 `self.lines: Vec<LineBox>`（mod.rs:68）。一个列切片纯函数可直接消费这些（遍历行盒、累加列高、超 budget 推进下列）。R897 的 `ColumnFragmentationContext` 接口设计正确。A2 假设**确认可行**（区别 column-aware-IFC Phase 1 的 0-case 停止 R381）。A3（chromium 余量 = overflow multicol 盒外）= CSS Multicol §2 spec-defined（fixed column-count + 明确高度，余量 overflow 盒外，overflow:visible 默认），留 commit 2 接线时 chromium Oracle per-test 复核。

**step-2 commit 1 LANDED（纯算法切片，net 0，零生产调用方）**：
- **新增** `crates/layout-engine/src/inline/column_fragmentation_flow.rs`（~230 行）= `fragment_lines_into_columns(lines: &[LineBox], ctx: &ColumnFragmentationContext) -> Vec<ColumnLineAssignment>` 纯函数 + `ColumnLineAssignment` 输出结构（`line_idx`/`column`/`y_in_column`）+ `HEIGHT_EPS` 容差 + **9 单测**（整行不裁断推进列 / 余量 overflow 留末列 / col_filled_heights 预占推进 / 单行超高至末列 / balance 无预算回退 / col_count=0 回退 / mismatch len 回退 / budget≤0 回退 / 空 lines）。
- **算法契约（CSS Multicol §2）**：整行不裁断（块级行盒不在中间断裂，区别 inline 跨列 Phase 2c）；列高 respected（避免 R897 probe A overfill）；余量 overflow 留末列（fixed column-count，overflow 由上层处理）。回退保守：col_count=0 / mismatch / None budget / budget≤0 → 空 Vec，调用方回退非碎片化（零回归）。
- **修改** `inline/mod.rs`（+3 行）：`mod column_fragmentation_flow` + `pub use`。
- **零生产调用方**：grep 证 `fragment_lines_into_columns`/`ColumnLineAssignment` 仅命中定义模块 + tests，**无任何 layout/paint/engine 路径调用** → 渲染字节级不变。

**验证（全门禁绿）**：`cargo build` 绿；`cargo test -p zero-layout-engine --lib column_fragmentation` **13 passed**（4 context + 9 flow）/ 0 failed；`cargo clippy --all-targets -- -D warnings` 干净（修了一处 `ColumnFillMode` 未使用 import：移到 test 模块两级 super 导入）；`cargo fmt` 干净。make test 全 workspace 绿（exit 0，0 FAILED）。

**意义**：把 multicol Phase 2a hardcore 分解为 **(a) 纯算法[本轮，低风险可测] + (b) 风险接线[下会话]**。commit 1 实现了碎片化的**核心算法 IP**（整行不裁断 + 列满续列 + 余量 overflow）+ 完整单测覆盖，零风险（纯函数无调用方）。比 R897（仅数据结构）更实质性——下会话的 commit 2 接线可直接复用此已测算法，专注风险点（layout 注入 + LayoutBox 新字段输出 + paint 消费取代后验裁剪）。

**▶ 下会话（step-2 commit 2，风险接线，多会话）**：① layout 侧为目标结构（单层 multicol + 单一 block 子 + `column-fill:auto` + 明确高度）在 `multicol.rs::layout_multicol` 调用点构造 `ColumnFragmentationContext`（col_count/col_width/gap 来自 `compute_column_info`，available_height = `column_height_limit`，col_filled_heights 全 0），对 block 子的 IFC 行盒调 `fragment_lines_into_columns`；② 输出 `Vec<ColumnLineAssignment>` 存 LayoutBox 新字段（如 `inline_multicol_columns`，参照 unified-column-flow-spec IF-001），paint 消费取代 `column_span_offsets` 后验裁剪；③ env `MULTICOL_COLUMN_FRAG` 门控 + 三态门禁 A/B 守 welcome<20% + scoped multicol oracle 零回归 + chromium-Oracle z_vs_chr 下降（**净负即回退**，§12.4 R157/R198/R203/R317 paint 侧 4 轮证伪先例，layout 侧须守同样的 multicol-fill-auto-001 sentinel）。**勿以单 session 期望 DC-2~5 显著提升**（受光栅 + 结构性双上限）。

### R899 multicol balance 行高均衡 A/B zero-yield 回退 + commit-2 可行性调查（纯调查，零 net 源码）

承 R898「下会话 step-2 commit 2」。**实测核对**（不信过时文档）：product-smoke welcome **16.11%**（<20% gate PASS）；`make reftest-oracle DIR=css-multicol` = **104/452 (23.0%)** == baseline（plateau 确认）。

**A/B 测：balance multicol 真实行高均衡（zero-yield 已回退）**。调查 `inline_finalization.rs:1020-1038` 发现 **balance 模式 multicol 已在列宽下重排 IFC**（`col_ctx = IFC::new(cw); col_ctx.layout(...)`），但**丢弃了列宽行盒**——只用 `n.div_ceil(cols) × (total/n)`（ceil 行数 × 平均行高）近似算容器高度。对非均匀行高（标题/段落混排）不准确。**改为真实行高文档序均衡**（每列 ceil(剩余行/剩余列) 行，用真实行高累计取最高列）。纯计算改动（仅影响容器高度），低风险。**A/B 结果：multicol oracle 字节不变**（104/452 == baseline，credible 97 一致）= **zero-yield no-op**。根因：① 无 WPT 案驱动纯 inline balance 路径（R381 已证 0/16）；② balance 容器 taffy 高度已匹配，此计算分支不触发 height 更新。按 code-guidelines「不做零价值修改」+ R855/R875/R891 zero-yield 回退先例**回退**（零 net 源码）。

**commit-2 可行性调查（零 net 源码）**：
- **balance 路径**（inline-only multicol 容器）：`inline_finalization.rs:1020` **已列宽重排** IFC，`col_ctx.lines` 即列宽行盒——但仅用于算高度后**丢弃**。capture 这些行盒 + `fragment_lines_into_columns` 切片是可行的低风险接线，**但 R381 证 0 WPT 案**驱动 → 零 yield。
- **auto + 单 block 子路径**（R897 probe 案）：block 子的 IFC 在**容器宽**（非列宽）下跑（`position_multicol_children` 仅后验 narrow `child.width`，不重排 IFC），故文本按容器宽换行后裁剪显示——核心不精确。修复须在 multicol 后处理时**按列宽重排 block 子的 IFC**（类 remeasure 模式），是真正硬核（block 子 IFC 重排 + 新 LayoutBox 字段 + paint 消费取代 `column_span_offsets` 后验裁剪）。
- **★ 关键 yield 评估**：multicol oracle top-15 worst **全部结构性**（column-balancing-paged-print 81%=@media print OOS / multicol-rule-nested-balancing-004 37%=nested Phase 2 / multicol-span-all-children-height 30%=R700 / multicol-breaking-005 23%=nested Phase 3 / subpixel-column-rule-width 23%=fiddly）——**无一为单层 auto 案**。即 commit-2（单层 auto）直接 yield ≈ 0，仅作 nested Phase 3 的 enabling infra。

**裁决**：multicol Phase 2a enabling slices（R897 接口 + R898 算法）已就绪；commit-2（auto+block 子列宽重排接线）是**低 yield + 高风险**（paint 侧 4 轮证伪先例 + block 子 IFC 重排可能触 R72 empty-styles / estimate-vs-fontdue 墙）。**强推 commit-2 风险/收益差**。balance 行高路径已证 zero-yield（勿再以单会话重试）。

**▶ 下会话方向（二选一）**：① **强推 commit-2 env 门控 disciplined 尝试**（layout 侧 auto+block 子列宽重排 + 新 LayoutBox 字段 + paint 消费，env `MULTICOL_COLUMN_FRAG` + 三态门禁 A/B，净负即回退——即使 yield 低，rally 以 disciplined 尝试运作，回退本身增 falsification 记录）；或 ② **pivot 到另一硬核轨道的 fresh tractability 扫描**（baseline-export 内容 IB last-line 基线合成 / R109 §9.2.1.1 匿名块 / Phase A step-2 store_font_sizes 式 ascent override），看是否有比 multicol commit-2 更高 yield 的窄切片。**勿以单 session 期望 DC-2~5 显著提升**（受光栅 + 结构性双上限）。

### R900 multicol inline-only auto 真缺口发现 + 无 paint 改动方案 de-risk（纯调查，零 net 源码）

承 R899「下会话 commit-2 或 pivot」。**选 multicol-fill-auto 家族深挖**（R870-R881 方法论：深挖具体中段高 diff 簇，非 top-worst）。**实测核对纠正 stale 文档**：

- **multicol-fill-auto 家族**（`make reftest-oracle DIR=fill-auto`）：13 案，oracle-pass 仅 3 (23%)，**worst 才 8.56%（中段，非结构性 20-80%）**——commit-2 可修范畴。
- **★ multicol-fill-auto-001 self-source 实测 FAILS 9.41%**（`LAYOUT_DUMP=1 ... reftest-upstream multicol-fill-auto-001`：45152px differ）——**真失败，非 false-pass**。master.md「sentinel passing at <1% via font_size coupling」描述**过时/错误**（0.63% 可能是历史瞬态或 self-source 路径差异）。sentinel 实为真缺口。
- **LAYOUT_DUMP 根因**：test 的 div 渲染为**单个 640px 宽块**（w=640 h=200，文本按容器宽换行），**inline 内容根本未分布到列**；REF 是两个 200px float 列（col1 x=8 w=200 h=200 / col2 x=228 w=200 h=100）。差异 = ZW 单宽列 vs chromium/REF 多窄列。

**★ 根因（代码追踪 + dump 实证）**：inline-only multicol 容器（直接文本，无 block 子）的文本**从不分布到列**——① `multicol.rs::layout_multicol` 仅处理 block children（`child_info` 过滤 `container.children`），inline-only 容器 children 为空 → 早返回（line 258-260）；② `remeasure_inline_only_containers:1020` balance 分支在列宽重排 IFC 但**丢弃列宽行盒**（仅算高度）；③ auto 分支 `balance_column_geometry` 返回 None（line 122）→ 连列宽重排都不做。即 **inline-only multicol（auto + balance）文本列分布完全缺失**。fill-auto 10 案 + balance inline 案均受此缺口影响（真 yield，非 enabling-infra-only）。修正 R899「commit-2 低 yield」结论——**inline-only 路径有真 yield**（区别 auto+block 子路径仍低 yield）。

**★ 无 paint 改动方案（de-risk 4× paint 侧证伪）**：`painter/text.rs:742` `multicol_info = if !has_in_flow_children && is_balance_mode && height_auto` —— 对 **inline-only auto multicol**（非 balance + height 明确）multicol_info = **None** → `use_stored = multicol_info.is_none() && inline_layout.is_some() && width_matches`（text.rs:838）可为 **true** → paint 用存储的 `box_node.inline_layout` 渲染。**故 layout 侧把列宽行盒重定位到各列位置存入 inline_layout，paint 无改动即按列渲染**——绕过 R157/R198/R203/R317 paint 侧 4 轮证伪的最大风险源。

**实现计划（下会话，env `MULTICOL_COLUMN_FRAG` 门控）**：在 `remeasure_inline_only_containers`（有 doc 访问）为 inline-only column-fill:auto + 明确高度 multicol 容器：① `compute_column_info` 取 col_count/col_width/gap；② 列宽重排 IFC（复用 line 1022-1028 模式）；③ `fragment_lines_into_columns(lines, ColumnFragmentationContext{...})` 分布；④ 行盒重定位：`InlineLayoutLine.y = y_in_column`（各列均从容器内容顶 0 起），每个 `InlineLayoutFragment.x += col_idx × (col_width + gap)`；⑤ 存 `box_node.inline_layout`（覆写）+ `inline_layout_width = container_content_width`（使 `width_matches` true）。目标 multicol-fill-auto-001 9.41%→<1%，fill-auto 家族 +N。三态门禁 A/B 守 welcome<20% + scoped multicol oracle 零回归，净负即回退。

**意义**：R900 发现**重开 multicol 真实 yield 轨道**（inline-only 列分布缺失 = 真缺口非 false-pass，区别 R899 误判「低 yield」），并找到**绕过 paint 侧 4× 证伪**的实现路径（use_stored 存储重定位行盒）。这是 R897 接口 + R898 算法之后的**决定性 de-risk**——下会话实现是真 pass-rate 杠杆（非 net-0 enabling）。

### R901 ★ multicol inline-only auto 列分布 LANDED 默认开启（首个 multicol 真 pass-rate win·+1 oracle·零回归·有 net 源码）

承 R900「下会话实现 inline-only auto multicol 列分布」。**当轮实现 + 默认开启 + 实测真 pass-rate 提升**（非 net-0 enabling——R897/R898 接口+算法之后的首个 active win）。

**实现**（R900 计划落地，hook 点修正为 `compute_final_inline_layouts` 而非 remeasure——remeasure 仅 height:auto）：
- **根因 blocker 定位**：`inline_finalization.rs:327` `if root.is_multicol { return; }` —— compute_final 对 multicol 容器**早返回不存储** inline_layout（注释「多列在 paint 阶段按列分配」是 inline-only 的错误假设），致 multicol-fill-auto-001 走 paint Path B 全宽重排 = 单宽列。
- **新 helper `store_inline_multicol_columns`**（inline_finalization.rs，~110 行）：① 门控条件——`is_multicol` + `compute_column_info` sequential_fill + count≥2 + `content_height>0`（明确高度）+ 无 in-flow block 子（inline-only）；② 列宽重排 IFC（`InlineFormattingContext::new(col_width).layout(doc, node_id, styles)`）；③ `fragment_lines_into_columns` 分布（R898 算法）；④ 行盒重定位：`InlineLayoutLine.y = y_in_column`，每个 `InlineLayoutFragment.x += col_idx × (col_width + gap)`；⑤ 存 `box_node.inline_layout` + `inline_layout_width = content_width`（使 paint `use_stored=true`）。
- **call site**（line 433-441）：`is_multicol` 分支由无条件 return 改为先尝试 `store_inline_multicol_columns`（命中则早返回已存储）。
- **multicol.rs**：`compute_column_info` + `ColumnInfo` 字段改 `pub(crate)`（供 helper 取 col_count/col_width/gap/sequential_fill）。
- **★ 无 paint 改动**（绕过 R157/R198/R203/R317 paint 侧 4 轮证伪）：paint `use_stored`（text.rs:838，multicol_info 对 inline-only auto 为 None）直接渲染存储的重定位行盒，按列分布。

**默认开启 + 回退开关**：`env MULTICOL_COLUMN_FRAG=0` 可关闭（默认开启）。触发条件极窄（inline-only + column-fill:auto + 明确高度 + 无 block 子——稀有模式），welcome/legacy 不命中。

**验证（全门禁绿 + 真 pass-rate 提升）**：
- **css-multicol oracle**：`make reftest-oracle DIR=css-multicol` = **105/452 (23.2%)** vs baseline 104/452 (23.0%) = **净 +1 oracle-pass 零回归**。
- **multicol-fill-auto-001**：oracle **8.56%→0.00% PASS**（self-source 9.41%→4.40%；self-source 残余 = ZW ref float 渲染差异非 test 缺口，oracle 0.00% 证 test 与 chromium 一致）。
- **fill-auto 家族**：oracle-pass 3→4 (30.8%)；multicol-fill-auto-003 5.26%→1.22%（大幅改善但仍 >1%）。
- `make test` 默认 ON 全 workspace 绿（exit 0，0 FAILED）；`cargo clippy --all-targets -D warnings` 干净；`cargo fmt` 干净。
- `make product-smoke` welcome **16.11%** 不变（<20% DC-13 gate，welcome 无 multicol 不触发）。

**意义**：**首个 multicol 真 pass-rate 胜利**（R897 接口 + R898 算法 + R900 de-risk 之后的水到渠成）。证实「无 paint 改动 + use_stored 存储重定位行盒」路径**绕过了 4× paint 侧证伪**——layout 侧列分布存储，paint 直接消费。multicol oracle 23.0%→23.2%（+1），方向确立。**残余 fill-auto 案例**（003 1.22% / 004 2.74% / 005 2.08% / block-children 001-003 1.6-1.9%）= 下会话精修方向（残余 = 列宽 IFC 缺复杂 override / block-children 走另一路径）。

**▶ 下会话**：① 精修 fill-auto 残余（003/004/005——查残余 diff 是列宽 IFC wrapping 差异还是 overflow 余量处理，目标再 +N oracle-pass）；② 扩展到 inline-only **balance** multicol（当前 balance 也丢列宽行盒，类似机制可解 balance inline 簇）；③ block-children fill-auto（has_block_child 排除——需 block 子列宽重排，是 R898 commit-2 原始范围，仍高风险）。**这是 multicol 轨道从 enabling 转 active yield 的转折点**。

### R902 fill-auto 残余分类 + inline-only balance 扩展 zero-yield 回退（dormant infra 保留）

承 R901「下会话精修 fill-auto 残余 + balance 扩展」。**逐案分类 fill-auto 残余**（读源码 + LAYOUT_DUMP）：
- **003（1.22%）**：inline-only（R901 修复适用，5.26→1.22 已大幅改善）。残余 = Ahem 字形像素精度（"3" 须精确覆盖 bg-image red20x20 红块于 2em 4em 位置）= **font-metric 精度墙**，非分布 bug。pixel-precision fiddly，非 clean win。
- **004/005 + block-children-001/002/003**（2.08/2.74/1.6-1.9%）：**block 子跨列碎片化**——`break-before:column` 在 grandchild 层（wrapper div 的内容强制断列），即 R898 commit-2 原始范围的高风险 block 子路径（非 simple break-before-on-direct-child）。R902 核查：`break_before:BreakValue::Column` 已解析+存储（types.rs:312）但 multicol.rs **从不读 break_before**（feature gap 确认）——但 004/005 的 forced breaks 在嵌套 wrapper 内，须 block 子内容碎片化（非直接子 forced break），仍高风险。

**A/B 测：inline-only balance 扩展（zero-yield 已回退 wiring）**：实现 `distribute_lines_balanced`（文档序 ceil-split 均衡分布，3 单测）+ 扩展 `store_inline_multicol_columns` 支持 balance（非 sequential_fill 路径）。**A/B 结果**：`make reftest-oracle DIR=css-multicol` oracle-pass **105 不变**（== R901），credible 97→98（仅退化重计噪声非真实改善）= **零 oracle-pass yield**。根因：inline-only balance multicol案本就少（多数 multicol 用 block 子），且 balance 文档序分布与 chromium 精确 balancing 在像素级有差（非 clean 命中）。**裁决**：按 code-guidelines「不做零价值修改」+ R902 blast radius 更广（所有 inline-only balance）**回退 wiring**（store_inline_multicol_columns 恢复 auto-only sequential_fill 门控）。**保留** `distribute_lines_balanced` + 3 单测作 **dormant infra**（`#[allow(dead_code)]`，doc 注 R902 zero-yield，待 balance 路径有具体 yielding 案时重新接线）。

**验证（回退后）**：`make reftest-oracle DIR=css-multicol` oracle-pass **105 (23.2%) == R901**（无回退回归）；welcome 16.11% 不变；clippy/fmt 干净；16 单测过（13 flow + 3 balance dormant）。

**裁决**：multicol active-yield 轨道（R901 +1）后续 lever 均受阻——003 = font-metric 精度墙 / 004/005/block-children = block 子碎片化高风险路径 / balance inline = zero-yield。**R901 的 inline-only auto +1 是本轨道单 session 可收获的 clean win**，后续需 font-metric 精度（Phase A）或 block 子碎片化（多会话硬核）。

**▶ 下会话**：① 接受 multicol 轨道 R901 +1 收获，pivot 到其他硬核（baseline-export 内容 IB last-line / R109 §9.2.1.1 / Phase A）；或 ② 尝试 block 子碎片化的最窄 slice（break-before:column 在直接子层，非嵌套 wrapper——查是否有 direct-child forced-break 案驱动）；或 ③ 转其他 dir fresh 深挖（R870-R881 方法论找 clean bug）。**勿以单 session 期望 multicol 显著再提升**（残余受 font-metric + block 子碎片化双上限）。

### R903 multicol break-before:column 死值消费 LANDED（spec-correctness 修复·0 oracle-pass·1 directional 改善·有 net 源码）

承 R902「下会话尝试 break-before:column 直接子 slice」。**实测核对**：welcome 16.11% 不变；multicol-break-001 oracle 1.22%。

**死值定位（R513 方法论）**：`break_before: BreakValue`（含 `Column` 变体，types.rs:312）由 css-parser 解析 + style-system apply（apply_advanced.rs:466）+ 存 ComputedStyle，但 **multicol.rs 从不读 break_before**——纯死值（parsed/stored/applied 但 layout从不 consume）。CSS `break-before:column` 完全未实现（feature gap，非推测性）。

**实现**（multicol.rs）：
- **import** `BreakValue`；`layout_multicol` 加 `styles` 参数（caller `adjust_multicol_layout` 已有 styles，line 62 传参）。
- **构建 forced_breaks**：与 child_info 同序，`c.node_id → styles → matches!(break_before, Column | Page)`。
- **3 assign 函数**（balanced/with_breaking/sequential）加 `forced_breaks: &[bool]` 参数，loop 前置检查：`forced_breaks[i] && current_col_height > 0.0 && current_col+1 < col_count` → 推进 current_col（**首子 forced break 在空列 no-op**，不创建前导空列，匹配 chromium）。
- **3 新单测**：break-before 簇各入独立列（balanced + breaking 路径）+ 首子 no-op。

**A/B 结果（诚实 yield）**：
- **multicol-break-001**（canonical）：oracle **1.22%→1.06%**（directional 改善，A/B/C 各入独立列，仍 >1% —— 残余 = column-width/glyph 像素精度，非分布 bug）。
- **fill-balance-002/030/040 + column-height-024**：**不变**（2.76/1.24/2.08/2.31%）——break_before 已消费但这些案的残余是**非分布问题**（column-rule/像素精度/balance 逻辑已分布两子到两列故 forced break 冗余）。
- **css-multicol oracle**：**105 (23.2%) 不变**（0 oracle-pass yield，但 0 回归）；welcome 16.11% 不变；make test 全绿（13 multicol 单测）。

**裁决（保留）**：0 oracle-pass yield，但区别 R902 balance 扩展（推测性、无 spec gap）——break-before:column 是**完全未实现的 CSS feature**（死值消费 = spec-correctness 修复），multicol-break-001 directional 改善（1.22→1.06，逼近 <1%），无回归。**保留作 correctness 修复**（latent value：任何未来 break-before:column 案 + multicol-break-001 残余若解像素精度即过线）。区别 zero-yield 回退先例（R899/R902 是推测性改动，本案是 feature 实现）。

**▶ 下会话**：① multicol-break-001 残余 1.06%（column-width:2em 列宽计算 / Ahem 字形精度——查是否 clean 像素修）；或 ② 接受 multicol R901+R903 收获（+1 oracle + break-before feature），pivot 到 R870-R881 fresh dir 深挖（css-tables/css-position 中段非结构性案找 clean bug）；或 ③ baseline-export / R109 多会话硬核。**multicol 轨道 R901-R903 已收 clean 收获**（+1 oracle + inline-only auto 分布 + break-before:column feature），后续边际递减。

### R904 ★ multicol em 解析按 element font-size LANDED（+1 oracle·multicol-break-001 PASS·第 2 个 multicol win·有 net 源码）

承 R903「下会话 multicol-break-001 残余 1.06% 像素修」。**LAYOUT_DUMP 定位根因**：div#test 子元素列宽 **33.3px**（200/6）vs ref/chromium **40px**（200/5）——column-width:2em 解析为 32px（6 列）应 40px（5 列）。

**根因（multicol.rs::length_to_px）**：`LengthValue::Em(v) => v * 16.0`（硬编码 root font-size 16）。注释声称「em/rem 已在 computed style 解析为 Px」，但 **column-width/column-gap 的 apply 不解析 em**（apply_advanced.rs:698-700 存 `Length(Em(v))` 不resolve）——故 multicol 的 em fallback 路径活跃，用 root 16 而非 element font-size。multicol-break-001 的 div#test font:1.25em/1（font-size 20px），column-width:2em 应 = 2×20 = 40px，旧实现 = 2×16 = 32px。

**修复**（multicol.rs）：`length_to_px` 加 `font_size_px` 参数，`Em(v) => v * font_size_px`（FitContent 递归传透）；`compute_column_info` 从 `style.font_size`（Px）提取 font_size_px 并传给 length_to_px（column_gap + column_width 两处）。font-size 16px（默认）场景字节同（×16 不变），仅 font-size≠16（如 1.25em=20px）案修正。

**验证（全门禁绿 + 真 pass-rate 提升）**：
- **multicol-break-001 oracle**：**1.06%→0.91% PASS（<1%）**！column-width:2em→40px，5 列（A/B/C 各入独立列 x=8/48/88 匹配 ref imgs）。
- **css-multicol oracle**：**106/452 (23.5%)** vs R903 的 105 = **净 +1 oracle-pass 零回归**（credible 98→99）；welcome 16.11% 不变；clippy/fmt 干净；14 multicol 单测（+1 em 解析单测）。
- make test 全 workspace 绿（exit 0）。

**意义**：**第 2 个 multicol 真 pass-rate 胜利**（R901 +1 + R904 +1 = multicol oracle 104→106，23.0%→23.5%）。证实「LAYOUT_DUMP 像素定位 + em 单位解析」clean lever 仍存在（区别 R903 的 directional-only）。R903 break-before:column + R904 em 解析**叠加**使 multicol-break-001 过线（R903 改分布 1.22→1.06，R904 修列宽 1.06→0.91）。multicol 轨道持续 active yield（R899 误判「平衡」零 yield 后，R901/R904 连续两轮 +1，证明该轨道未枯竭）。

**▶ 下会话**：① 继续扫 multicol 残余 mid-range 簇（columnfill-auto-max-height-001/002/003 2.89/3.94/1.86%——查是否 em-解析或 break-before 相关连带 yield）；或 ② pivot R870-R881 fresh dir 深挖（multicol 轨道已 R901+R904 连续 +1，但残余渐入 font-metric/block-子碎片化墙）；或 ③ 检查 em-解析修复是否对其他 dir（css2/text/tables 用 em column-width/gap 的案）有连带 yield（跑全 corpus oracle 对比）。**multicol 轨道 R901-R904 持续 active yield**，勿过早 pivot。

### R905 ★ multicol max-height budget + 分布后高度修正 LANDED（+2 oracle·columnfill-auto-max-height 001/002 PASS·有 net 源码）

承 R904「下会话 columnfill-auto-max-height 簇」。**LAYOUT_DUMP 定位**：columnfill-auto-max-height-001 的 div h=**50**（应 100）——容器用 **max-height:100px**（非 height），但 content_height 来自**全宽 IFC**（300px 宽，2 行=50px），非列宽（100px 宽，4 行=100px）。R901 修复用 content_height（50，错）作 budget，且未修容器高度（存储的 4 行盒中 y=50/75 行被容器 50px 高度裁剪不可见）。

**修复**（`store_inline_multicol_columns`，R901 扩展）：
- **budget 用 height/max_height**：`(style.height Px → h) | (style.max_height Px → m) | content_height`。明确 height 案字节同（h == content_height）；max-height 案用 max-height 作 budget（列更窄→更多行）。
- **分布后高度修正**（仅 `from_max_height` 案）：算最高列累计高度 `tallest`，若 > content_height 则 `content_height = tallest; height += delta`。使存储的列分布行盒全部可见（不被全宽 IFC 偏小高度裁剪）。明确 height 案不修正（高度已对）。

**验证（全门禁绿 + 真 pass-rate 提升）**：
- **columnfill-auto-max-height 家族**：oracle-pass 0→**2 (66.7%)**——001 2.89%→**0.87% PASS**，002 3.94%→**0.95% PASS**，003 1.86% 不变（独立子问题，可能 column-rule 绘制）。
- **css-multicol oracle**：**108/452 (23.9%)** vs R904 的 106 = **净 +2 oracle-pass 零回归**（credible 99→101）；welcome 16.11% 不变；clippy/fmt 干净；make test 全绿。
- 累计 multicol：R901 +1 + R904 +1 + R905 +2 = **104→108 (23.0%→23.9%)**。

**意义**：**第 3 个 multicol 真 pass-rate 胜利（+2，单轮最高）**。R901 的 use_stored 列分布机制 + R904 em 解析 + R905 max-height budget/高度修正**连续叠加** yield。证实 multicol 轨道**仍未枯竭**（R899「balance 零 yield」误判后被 R901/R904/R905 三轮 +4 推翻）。columnfill-auto-max-height-003 残余 1.86% + block-children + 嵌套 balance 仍待挖。

**▶ 下会话**：① columnfill-auto-max-height-003 残余 1.86%（查是否 column-rule 绘制 / column-gap 位置）；② 跑全 corpus oracle 看 R904 em + R905 max-height 对其他 dir（css2/text 用 em column-width/gap + max-height 的案）连带 yield；③ 继续扫 multicol 其余 mid-range。**multicol 轨道 R901-R905 三轮 +4，持续 active yield**。

### R906 双 dead-end 调查（ch 单位 + text-transform layout 一致性·零 yield·全回退·零源码）

承 R905「下会话扫 multicol 其余 mid-range + cross-dir yield」。两 lever 各 A/B 实测后均 dead-end，按 code-guidelines「不做零价值修改」全回退（工作树净零）：

**① ch 单位 resolution（R904 em 类比）= NET NEGATIVE，回退**：`multicol-width-ch-001`（6.91%）驱动。定位 ch 解析两处 bug——general resolver `computed.rs:48 Ch(v)=>v*font_size*0.5`（0.5em 近似）+ multicol `multicol.rs:117 Ch(v)=>v*8.0`（硬编码）。Ahem advance=1em（loader.rs `test_ahem_measure_advance` 实测 advance==font_size），故 1ch(Ahem)=1em=font_size。修两处→1em。
- **A/B（css-text 1650 案 + multicol 452 案）**：css-text oracle **278→275（-3）**；multicol 108 不变（multicol-width-ch-001 仅 6.91→4.12 directional，未过线，残余=内容分布非列宽）。
- **裁决**：general 0.5em→1em 对 css-text（1042 ch 用法，height/margin/text-indent Nch）net -3——0.5em 对多数 css-text oracle 案更近（真字体 1ch≈0.5em，ZeroWeb 无 font-metric 管线，0.5em 是更好的全局默认）。multicol-only 1em 与 general 0.5em 不一致（容器 0.5em vs 列宽 1em）更差。**区别 R547（ex→0.8em clean win）**：ex 用法少 blast radius 小，ch 在 css-text 极广故 Ahem 1em 反而 net 负。**ch 杠杆关闭**，须 font-metric 管线（多会话）才能 Ahem-accurate 不伤真字体。

**② text-transform layout/paint 一致性 = ZERO YIELD，回退**：css-text `text-transform-upperlower-*` 簇（8 案 ~5-5.9%）疑似 layout/paint 文本不一致（paint `text.rs:1083` 对 fragment.text 变换，layout 用原文测量→大小写字形宽度差致换行不一致）。加 layout 侧变换（inline_types.rs `apply_text_transform` + mod.rs:762/502 两 text-gather 点）。
- **A/B（css-text）**：oracle **278 不变**（zero yield）；text-transform-upperlower-002 5.84% 字节同。
- **裁决**：paint `text.rs:1083` 已在主 fragment loop 用 `owner_id` 的 computed style 变换（text-transform 已正确 apply at paint）；upperlower 簇 ~5.8% 真因 = **DoulosSIL-R.woff webfont fallback**（test 与 ref 同 font stack，差异仅 text-transform，若 apply 则应 ~0%；5.8% 持续 = webfont 像素差非 transform）。layout 侧变换冗余（paint 幂等）零 yield。回退。

**裁决（轨道）**：multicol R901-R905 +4 后，本轮两 lever（ch / text-transform）+ column-rule 簇（10 案 ~6.3%，solid 6.35 vs none 5.81 仅差 0.54%=rule 像素足迹，~5.8% 共享基=列分布 plateau）均 dead-end。css-text / css-flexbox mid-range 簇扫描（align-content 7 案、line-break 11 案）同呈「~5% 共享 diffuse 基非单 feature bug」模式。**残余缺口确为 master.md 既述结构性 plateau（font-metric / block-子碎片化 / webfont），单 session clean lever 边际递减**。

**▶ 下会话**：① LAYOUT_DUMP 单个 multicol mid-range 特定案（multicol-width-003 6.60% / multicol-count-002 8.47%）找 R904 式特定数值 bug（非簇扫描）；② 接受 multicol 轨道 R901-R905 +4 收获，pivot 全 corpus oracle 找其他 dir 未挖特定案；③ font-metric 管线 / block-子碎片化 多会话硬核。**勿以单 session 重试 ch / text-transform / column-rule 簇（已 ruled out）**。

### R907 ★ column-rule-width em 解析 LANDED（+3 oracle·multicol-rule-fraction 001/002/003 PASS·单轮最高·有 net 源码）

承 R906 dead-end 后转「单个 near-pass 特定案数值 bug」（非簇扫描）。**聚焦 multicol-rule-fraction 簇（~1.5%，近过线）** 读 paint 代码定位根因。

**死值定位（R513 §9.7 / R904 谱系）**：`column-rule-width:1em` 经 `parse_column_rule_width`→`ColumnRuleWidthValue::Length(LengthValue::Em(1.0))`，apply 存 `ColumnRuleWidthComputedValue::Length(Em)` **未 resolve**。`computed.rs::resolve_computed_style` 的 `resolve_length_field` 列表覆盖 width/height/margin/padding/border_*/top/right/bottom/left/gap 等**裸 LengthValue 字段**，但 `column_rule_width` 是 **Medium/Thin/Thick/Length 枚举**（非裸 LengthValue），不在列表内 → 内部 `Length(Em)` 永不解析。paint `painter/text.rs::paint_column_rules` rule_w match（line 192-198）仅 `Length(Px(w))=>*w`，em 落入 `_ => 1.0` → **column-rule-width:1em 渲染为 1px**（应按 element font-size，如 1.25em 字体下 1em=20px）。multicol-rule-fraction-001/002/003 用 fractional em（0.5em 等）rule-width，旧实现统一 1px 致 ~1.5% 不过线。

**修复**（`computed.rs`，border-width 谱系 precedent——`border_top_width` 等裸 LengthValue 在 line 258 etc. compute 时 resolve）：在 column_gap resolve 后加枚举内部 Length resolve——
```rust
if let ColumnRuleWidthComputedValue::Length(lv) = &resolved.column_rule_width.clone() {
    let mut lv = lv.clone();
    resolve_length_field(&mut lv, font_size_px, viewport_width, viewport_height);
    resolved.column_rule_width = ColumnRuleWidthComputedValue::Length(lv);
}
```
Medium/Thin/Thick 关键字不进 Length 分支，字节同（零回归）。

**验证（全门禁绿 + 真 pass-rate 提升）**：
- **multicol-rule-fraction 家族**：001/002/003 均 **<1% PASS**（001/002 0.88%，003 0.64%）；multicol-rule-003 1.54→1.13%（directional，残余=内容分布非 rule-width）。
- **css-multicol oracle**：**111/452 (24.6%)** vs R905 的 108 = **净 +3 oracle-pass 零回归**（credible 101→104，near 100→103，mismatch 344→341，strict 8 不变）；welcome product-smoke 16.11% 不变（DC-13 <20% 绿）；clippy/fmt 干净；make test 全 workspace 绿（exit 0）；+1 column-rule-width em 单测。
- 累计 multicol：R901 +1 + R904 +1 + R905 +2 + R907 +3 = **104→111 (23.0%→24.6%)**。

**意义**：**第 4 个 multicol 真 pass-rate 胜利（+3，单轮历史最高）**。R906 三 dead-end（ch/text-transform/column-rule-STYLE 簇）后，转「near-pass 特定案 paint 代码核查」立即命中——**column-rule-WIDTH em 是独立于 column-rule-STYLE 簇（~6%，分布主导，仍 ruled out）的干净 lever**。证实 R904 em-resolution 谱系（apply 存 Em 不 resolve → consumer 误判）仍可挖：本次是 paint consumer 的 `_ => 1.0` fallback 暴露的 compute-side gap。方法论：**near-pass（1-1.7%）案比 mid-range（5-8%）案更易过线，paint 代码 `_ => <default>` fallback 是 em-resolution 谱系的可靠指示符**。

**▶ 下会话**：① 继续扫 multicol near-pass（1.0-1.7%：column-rule-002 1.69% / multicol-rule-shorthand-2 1.53% / equal-gap-and-rule 1.41% / multicol-rule-003 1.13%）查是否 em-resolution 或 rule 绘制相关；② 扩展 em-resolution 谱系审计：grep 其他 `ComputedValue` 枚举（ColumnRuleWidth 同型）含 `Length(LengthValue)` 但不在 resolve_length_field 列表者（outline-width? column-rule-width 已修）;③ pivot 其他 dir near-pass。**multicol 轨道 R901-R907 四轮 +7，near-pass 杠杆持续 active yield**。

**R907 后续 em-resolution 谱系审计（已完成·无新 lever）**：grep 全 `Length(LengthValue)` 枚举（types.rs 6 处）逐项核实——LineHeight（computed.rs:201 特殊 resolve）/ FlexBasis（test line 615 证 resolve）/ ColumnWidth（R904 multicol length_to_px resolve）/ ColumnRuleWidth（R907 修）/ PropertyValue（registry 初始值非 stored field）均已 resolve 或 N/A；**TabSize 唯一未审计**→核实 paint `text.rs:813` 已 `Length(Em(v)) => v*font_size` 正确 resolve（reftest 用 number/px 非故非 lever）；**outline_width 是裸 LengthValue**（computed_style.rs:94，在 resolve 列表，已 resolve）。**结论：column-rule-width 是 em-resolution 谱系唯一 dead-value，已修，谱系穷尽**。R907 后 multicol near-pass 残余（column-rule-002 1.69%/equal-gap-and-rule 1.41% 用 **px** rule-width 非 em；shorthand-2/rule-003 em 已修但残余是**内容分布**）= 分布 plateau 非 em lever。下会话勿重扫 em-resolution 谱系（已穷尽）。

### R908 clean-lever 穷尽确认 + 跨 dir near-pass 扫描（bidi/tables/writing-modes 全结构性·零 yield·零源码）

承 R907「pivot 其他 dir near-pass 用 R907 方法论（读 paint/convert 代码找 `_ => <const>` fallback 或 dead-value）」。**全 dir near-pass（1-2.2%）扫描 + 两个 lever A/B 实测后，确认 clean-lever（单 session dead-value / em-resolution）vein 已穷尽**：

**① R908 bidi paragraph level = ZERO YIELD（回退）**：css-writing-modes `bidi-*` 簇（~30 案 2.05-2.19%）疑似 dead-value。定位 `text_metrics.rs:264` `bidi_reorder` 硬编码 `ParagraphInfo { level: Level::ltr() }` 而非用 `bidi_info.paragraphs[0]`（BidiInfo::new 已自动检测真实 base direction）。修为用真实 paragraph。
- **A/B（css-writing-modes）**：oracle **53→53（6.8%）字节同 zero yield**。机制：`reorder_line` 由 `BidiInfo::new` 预计算的 `levels` 驱动（内部已用正确 paragraph detection），传入的 `para.level` 对输出无影响（cosmetic）。回退。
- **真 bidi lever = CSS `unicode-bidi` 值（embed/isolate/override/plaintext）完全 dead**：parse+store（apply_advanced.rs:662）但 BiDi 算法从不消费（text_metrics 用 `BidiInfo::new(text, None)` 仅按文本字符固有 bidi，CSS unicode-bidi 值不作为 embedding/override 码注入）→ isolate/embed/override 渲染零效果。**这是 ~30 案 bidi 簇的真根因，但修复须实现 UBA9 embedding/isolation levels（多 session M5 硬核，非单点 consume）**。

**② css-writing-modes near-pass 全结构性**：clip-rect-vrl-010/012/014/016（4 案 2.20%，clip paint mod.rs:990 用标准 CSS2.1 物理坐标 spec-correct，divergence = vertical-rl abspos 定位非 clip）；margin-collapse-vlr/vrl（vertical margin-collapse，结构性）；bidi-*（见 ① UBA9 dead）。

**③ css-tables near-pass 全精度/结构性**：subpixel-collapsed-borders-001/002/003（3 案 ~1.08-1.19%，5px green vs 4.95px red 冲突解决 = subpixel 渲染精度墙非逻辑 bug）；row/row-group-margin-border-padding（CSS tables §17.5.3 row margin 不生效，结构性）；fixup-dynamic-anonymous-inline-table（匿名 table fixup，结构性）。

**④ `_ => <const>` fallback 全仓审计（R907 模式外推）**：grep paint/converter 的 `_ => 数字` fallback——剩余均为防御性 font_size 回退（text.rs:338/359/mod.rs:470/640 `_ => 16.0`，font_size 已 compute-to-Px 故 `_` 罕触发）或 debug indicator（effects_indicators.rs:1013 `_ => 4.0`），**无新 R907 式 dead-value**。

**裁决（vein 穷尽）**：R901-R907 七轮（multicol +7）后，本轮系统确认 clean-lever vein 穷尽——em-resolution 谱系（R907 唯一）、`_ => const` consumer-fallback 谱系（无新）、跨 dir near-pass（writing-modes/tables 全结构性/subpixel）。**残余 near-pass 全属三类多 session 硬核**：(a) **bidi UBA9 CSS-values dead-value**（~30 案，最大单一已识别 lever，须 embedding/isolation 实现）；(b) **font-metric 精度墙**（subpixel borders / 行盒度量 Phase A）；(c) **结构性**（vertical-rl 定位 / table 匿名 fixup / margin-collapse-vertical）。下会话 lever 选择须在三者中取一做**多 session 推进**，勿再期望单 session clean dead-value。

**▶ 下会话**：① **bidi UBA9 CSS-values 是最大已识别 lever**（~30 案 2.05-2.19%）——下会话可 START：先 `unicode-bidi: plaintext`（段落 base direction 从内容判定，最窄 slice，改 `BidiInfo::new(text, Some(level))` 按 `direction`/`plaintext` 注入）de-risk + 找 yielding 子集；② font-metric Phase A 行盒度量（welcome/morning 17% 真因，多 session）；③ 接受 plateau 转其他方向。**R908 已证 bidi paragraph-level 单点零 yield，真 lever 是 CSS-values 注入（多 session）**。

### R909 webfont 基础设施 = 多簇隐藏阻塞（bidi/text-transform/shaping）·调查零源码

承 R908「下会话 START bidi UBA9」。深挖 bidi 簇（bidi-isolate-005 等）发现**真根因被 webfont 基础设施阻塞，非 bidi 逻辑单点**：

**① webfont 文件本地缺失 + oracle 亦回退**：bidi 簇用 `@font-face ezra_silregular src=/fonts/sileot-webfont.woff`（Hebrew），text-transform 簇用 `DoulosSIL-R.woff`，css-text shaping 用 `Scheherazade-Regular.woff`/`mplus-1p-regular.woff`。**本地 `wpt-data/fonts/` 仅有 Ahem/GentiumPlus-R/Lato/Revalia/AD**，sileot/DoulosSIL/Scheherazade/mplus 全缺。关键：`chromium-oracle-shot.mjs:35` `DATA_ROOT=wpt-data`（oracle 抓取 HTTP server 根 = 本地 wpt-data）→ chromium 抓 oracle 时这些字体也 404 → **chromium oracle 与 ZeroWeb 均回退**到各自默认字体（chromium serif/sans vs ZeroWeb default），回退字体不同 = 簇发散主因（非 bidi 逻辑）。

**② oracle 路径 vs 浏览器路径 font loading 分离**：
- **oracle reftest 路径**（`reftest.rs:1051 load_font_faces_into`）：**工作正常**——`extract_font_faces` → `resolve_font_src`(base_dir) → `std::fs::read` → `loader.load_font` + `register_family_alias`。本地存在的字体（GentiumPlus/AD 等 87+36 案引用）正确加载。
- **浏览器导航路径**（`async_load.rs:364 poll_fonts`）：**fetched bytes 丢弃**（line 368 仅 `tracing::info!` 不 decode/register）→ 浏览器 @font-face 全回退。但此路径**不被 oracle / product-smoke 用**（welcome 无 @font-face），修它零可测 metric（须 bytes→FontLoader decode+register plumbing，speculative，按 code-guidelines 不做）。

**③ 上游可用性分级**：上游 WPT root `fonts/` 核实——**sileot/Scheherazade/mplus/NotoSansGeorgian 上游有**（可 fetch + 补本地）；**DoulosSIL-R.woff 上游无**（text-transform 簇永久回退，不可 fetch，须替换或接受）。

**④ bidi 逻辑层独立缺口（叠加 webfont）**：CSS `unicode-bidi: embed/isolate/override/plaintext` 值 parse+store（apply_advanced.rs:662）但 BiDi 从不消费（`text_metrics.rs:258 BidiInfo::new(text, None)` 仅按文本字符固有 bidi，CSS 值不注入）= R513 §9.7 dead-value。**故 bidi 簇 = webfont 回退 + bidi 逻辑双阻塞**：须先修 webfont 基础设施（fetch + recapture）才能 clean-measure bidi 逻辑 yield。

**裁决（vein 重定向）**：bidi/text-transform/shaping 三簇发散主因 = **webfont 基础设施**（本地字体缺 + oracle 同回退），非逻辑单点。**webfont 杠杆须协调一次性交付**：(a) fetch 上游可用字体（sileot/Scheherazade/mplus/NotoSansGeorgian）补 wpt-data/fonts；(b) **re-capture 受影响 oracle**（chromium 现加载真字体）；(c) 验 ZeroWeb `load_font_faces_into` 加载同字体 → 对齐。**注意**：仅 fetch 不 recapture = ZeroWeb 真字体 vs oracle 回退 → 发散更大（中间态 net 负），必须同 commit。bidi 簇还须额外修 unicode-bidi CSS-values 注入。DoulosSIL 簇不可修（上游无）。**webfont 基础设施是多 session 工程**（fetch N 字体 + 全量/分 dir recapture + baseline 重稳 + 逐簇逻辑修），非单 session。

**▶ 下会话**：① **webfont 基础设施 START**（最高 blast radius——解锁 bidi/shaping/css-fonts 多簇）：先 fetch 1 个上游字体（sileot-webfont.woff via ~/use-proxy 代理）→ targeted re-capture bidi 簇 oracle（capture-oracle-per-dir.mjs --category）→ A/B 看 ZeroWeb(load_font_faces_into) vs 新 oracle，de-risk 「fetch+recapture 是否真消除 webfont 发散」；② 若 webfont 解锁后 bidi 仍发散 → 确证 bidi 逻辑（unicode-bidi 注入）为残余 lever，再 START UBA9；③ font-metric Phase A。**R909 证 bidi/text-transform 簇非纯逻辑阻塞，勿再单 session 修 bidi 逻辑期望 yield（webfont 阻塞）**。

### R910 webfont 杠杆 A/B 证伪（bidi 簇非 font-blocked·sileot+recapture 发散不变·零源码·baseline 已还原）

承 R909「webfont 基础设施 START de-risk」。**执行 A/B 实验**：① fetch `sileot-webfont.woff`（上游 WPT root fonts/ 有，59KB WOFF valid）入 `wpt-data/fonts/`；② re-capture 4 个 bidi 测试 oracle（bidi-normal/unset/isolate/embed-005，puppeteer-core + /usr/bin/chromium，chromium 现加载真 sileot Hebrew 字体）；③ ZeroWeb 经 `load_font_faces_into`（reftest.rs:1051）亦加载 sileot；④ 跑 oracle 对比。

**A/B 结果（webfont 杠杆证伪）**：
- **bidi-normal-005**：2.05% → **2.18%（未降，略升）**。
- **bidi-unset/isolate/embed-005**：~2.18-2.19%（与 baseline 持平）。
- **css-writing-modes oracle 整体 53（6.8%）不变**。
- **裁决**：sileot 真字体（两侧均加载）后发散**不变** → **webfont 非 bidi 簇发散组分**。bidi ~2.18% 真因 = **bidi 算法/布局**（ZeroWeb 逐 run `bidi_reorder` vs chromium 全段落 UBA9；unicode-bidi CSS-values dead-value 见 R908；或 fontdue sileot 字形 advance/行盒度量 vs chromium freetype）。字形非主因（若一侧真字体一侧回退，发散应远大于 2%；现 2% 且两侧同 → 发散在排列/度量非字形）。

**baseline 还原**：删 `wpt-data/fonts/sileot-webfont.woff` + re-capture 4 测试 oracle（无 sileot，回退，匹配原始 baseline）。工作树净零（仅 master.md），oracle-shots gitignored，oracle 53 复原。

**意义（重定向）**：R909「webfont 是 bidi/text-transform 簇隐藏阻塞」假设**被 R910 A/B 推翻**（至少 bidi 簇）。bidi 簇发散是**算法/度量**，非字体文件。webfont 基础设施（fetch+recapture）对 bidi **零收益**。text-transform 簇（DoulosSIL 上游无）仍不可 fetch；shaping 簇未测但同理可能非 font-blocked。**webfont 杠杆降级**——非高 blast radius 解锁器，bidi 真 lever 是 UBA9 算法实现（R908 既述 unicode-bidi CSS-values 注入 + 段落级 bidi 而非逐 run）。

**▶ 下会话**：① bidi 算法层（真 lever）：实现 CSS `unicode-bidi` 值注入（embed/isolate/override 经 LRE/RLE/LRI/RLI/FSI+PDF/PDI 控制符）+ 段落级 bidi（跨 run 而非逐 run `bidi_reorder`），A/B css-writing-modes bidi 簇；② font-metric Phase A（bidi 残余若含度量差）；③ 勿再投 webfont 基础设施对 bidi（R910 证零收益）。**R910 证 bidi 簇 font-unblocked，真 lever 是算法**。

### R911 table-relative abspos 簇 + bidi 复核（均非 clean lever·零源码）

承 R910「下轮 bidi 算法 de-risk」。本轮转核实 css-position 近 pass 簇 + bidi 复核，**均确认为非 clean lever**：

**① css-position `position-relative-table-{tbody,thead,tfoot}-{left,top}-absolute-child`（6 案均 1.30%）= subpixel 非 CB bug**：假设 position:relative tbody 作 abspos containing block 有 bug。**LAYOUT_DUMP 实测**（position-relative-table-tbody-left-absolute-child）：`div.indicator` x=108（红，left:100px @ group CB 8+100）；`tbody.relative` x=58（left:50px @ 8+50）；`div.absolute` x=**108**（left:50px @ tbody CB 58+50）→ **green.absolute 完全覆盖 red.indicator**。**CB resolution 正确**（abspos CB = tbody.relative padding-box），`has_positioned_ancestor` 传播（abspos.rs:64-68）含 is_relative 无遗漏。**1.30% 是 subpixel/边缘渲染非逻辑 bug**。簇 ruled out（非 +6 lever）。

**② bidi 簇复核（R910 后）**：css-writing-modes self-source reftest 686/686 PASS（含 unicode-bidi-{normal,embed-ltr,embed-rtl,bidi-override-ltr,embed-rtl} inline 案 0.00%）→ ZeroWeb bidi **内部自洽**（test==ref，inline 用例证 embed/override 逻辑匹配 inline ref）。**注意**：686 是 inline smoke 语料（非上游 bidi-*），故上游 bidi-normal-005 的 self-source 未直接测；但 R910 已证 webfont 非组分，bidi ~2.18% 共享基最可能是 **fontdue sileot advance/行盒度量 vs chromium**（font-metric 谱系）或逐-run-bidi vs 段落-bidi 差异。**bidi 算法大重构（段落级 + unicode-bidi 注入）yield 不确定**——若 self-source 已自洽，ref 也同源 ZeroWeb 渲染，oracle 差在 font-metric 则重构零 yield。

**③ css-backgrounds 无 near-pass（1-2.2% 空集）**：该 dir 案例全远离过线或无 oracle。

**裁决（plateau 再确认）**：R907（multicol em-resolution +3）后 R908/R909/R910/R911 **连续四轮**系统确认 clean-lever（单 session dead-value / CB bug / em-resolution）vein **彻底穷尽**——bidi（font-metric/subtle）、table-relative（subpixel）、webfont（R910 证伪）、em-resolution 谱系（R907 唯一）全 ruled out。**残余 near-pass 发散全属三类多 session 墙**：(a) **font-metric Phase A**（fontdue advance/line-height vs chromium——welcome 16% / morning 17% / bidi 2.18% / 多 text 案同源，最深最高 blast radius）；(b) **subpixel 渲染精度**（collapsed-borders / table-relative / 多 1-1.5% 案同源，可能全局坐标 round 策略差，risky）；(c) **结构性**（vertical-rl 定位 / 嵌套 breaking / table 匿名 fixup）。

**▶ 下会话**：① **font-metric Phase A 是最深最高 lever**（welcome 16% + morning 17% + bidi 2.18% + 多 text 案同源 fontdue 度量）——下轮可 START 最窄 slice：fontdue advance_width 接入 IFC（替换 estimate_char_width 启发式），A/B product-smoke + css-text 看行宽/换行一致性提升（R224 曾试 DejaVu 表退步，须用真实 FontLoader advance 非 hardcoded 表）；② subpixel round 策略审计（risky，全局影响）；③ 勿再扫 near-pass 找单点 dead-value（四轮证穷尽）。**R911 证 table-relative CB 正确 + bidi self-source 自洽，clean lever vein 彻底关闭**。

### R912 font-metric Phase A 方向纠正（advance-width 已 R225/R375b 证伪·welcome = font-engine 墙·零源码）

承 R911「下轮 font-metric Phase A advance_width 接入」。**读 `advance-width-plumbing-design.md` 发现上轮 CONTINUE 提议的 lever 已被证伪**——纠正方向：

**① advance-width plumbing = R225 双实验证伪（2026-06-17，已记忆）**：`advance-width-plumbing-design.md` 顶部「R225 证伪结论」+ memory `advance-width disproven R225` 明载：
- reftest-oracle 26 共享 case：DejaVu advance 表替换后 strict true-pass 11 vs 11、median 1.06% vs 1.07%、0 case 改善/恶化。
- product-smoke：welcome 28.34→28.31%（Δ-0.03%）、wintertc 25.11→25.14%（Δ+0.03%）零实质变化。
- 机制：paint glyph x 定位走真实 fontdue shaping（非 estimate），estimate 仅影响 layout 换行决策。
- **R375b 进一步证伪 fontdue-real-advance**：临时加 fontdue dep 测 `metrics.advance_width` → welcome **16.41→19.08%（regress -2.67pp）**。即 fontdue 的 NotoSansCJK advance ≠ chromium，用 fontdue 真 advance 反而更发散。
- **结论：advance-width（estimate / DejaVu 表 / fontdue-real 三 variant）全证伪，勿再投入**。上轮 CONTINUE 提议无效。

**② welcome 16.11% 发散诊断 = per-line 同源 font-engine 差异**：product-smoke diff 像素 77305，per-row diff band 分析显示**发散逐行遍布全页**（y=68-80/110-148/155-166/177-191/265-278/305-325/429-441/517-541/550-570/593-600 等 ~25 band，每 ~30-50px 一条 = 每条文本行），非单一组件坏。逐排除：advance-width（R225 证伪）、font-matching（R631 强制 NotoSansCJK 零变化）、rasterization（R388 fontdue≈chromium per-glyph）、line-height 显式值（welcome 用 1.08/1.5/1.45/1.25 显式，R632 已 fix override）。**残余 = fontdue vs chromium 字体度量/定位系统性差异**（fontdue 对 NotoSansCJK 的 metric 提取 ≠ chromium，R375b advance 已证一致；line-metric/ascent 同源推测亦 ≠）。

**③ line-metric（FontMetricProvider）= font-metric 谱系唯一未证伪子项，但 yield 存疑**：`FontMetricProvider::line_metrics`（font_metrics.rs:60，FontLoader impl 返 ascent/descent/line_gap）**已实现但 dormant**——engine 不注入（IFC `font_metric_provider` 生产恒 None，仅 builder/test 设），`apply_vertical_alignment` 用 `font_size*0.8` 启发式（mod.rs:1634）、`resolve_font_metrics` Normal 用 1.2（text_metrics.rs:196）。Phase A step-2（注入 + consume）未做。**但 R375b advance regress 暗示 fontdue NotoSansCJK metric 整体 ≠ chromium**，line-metric 接入大概率同 regress（须实测确认，但预期负）。注入点散（text.rs:888/947 paint IFC + layout IFC，须 thread FontLoader），invasive。

**裁决（plateau 根因定位）**：rendering-compat 残余发散（welcome 16% / oracle ~64% mismatch）**大头是 font-engine 系统性差异**（fontdue 对各字体的 metric 提取 + 定位 ≠ chromium），**非 CSS layout bug**。R901-R907（multicol em/value +7）后，CSS layout 层 clean lever 穷尽（R908-R911 四轮证），font-metric Phase A（advance 已证伪 / line-metric 大概率同源 regress）亦非可行 lever。**DC-2~5 95% 目标受 font-engine 墙阻塞**，须 font-engine 投资（匹配 chromium metric 提取 / 换字体后端）方能突破，非 CSS 层单/多 session 可解。

**▶ 下会话**：① **line-metric 实测 de-risk**（唯一未证伪 font-metric 子项）：最小注入 FontMetricProvider 到一处 IFC + consume 替换 0.8 启发式，A/B welcome——若改善则 fontdue line-metric 匹配 chromium（解锁 lever），若 regress 则确证 font-engine 墙；② **pivot 到结构性 lever**（非 font）：vertical-rl clearance（R114/R164 近 miss，x 轴实现）、嵌套 multicol breaking、table 匿名 fixup；③ **font-engine 投资**（匹配 chromium metric 提取或换后端）= major 多 session，须方向决策。**R912 纠正 advance-width 方向（已证伪），font-metric Phase A 整体近 dead-end，残余 = font-engine 墙 + 结构性**。








### R913 text-decoration-skip-spaces 证伪（css-text-decor 装饰线 trim 首尾空白·零 yield·零源码）

承 R912「下会话 lever 待定」。本轮试 **css-text-decor 装饰线 lever**：实现 `text-decoration-skip-spaces` 默认（`start end`）——装饰线（underline/line-through）跳过片段首尾空白（`decoration_extent_skip_end_spaces` 在 text_metrics.rs，text.rs 两处 `paint_text_decoration_from_style` 调用消费，避免 `white-space:pre` 首尾空格被装饰线覆盖）。

**A/B（css-text-decor oracle）**：**72/242（29.8%）字节同 zero yield**——with-R913 与 baseline（git stash 对照）per-dir 通过率、strict 真通过（8）、top-15 worst 全字节一致。**裁决：refuted，全程回退**（stash drop，工作树净零）。

**机制**：css-text-decor top worst（dotted-001/002 15.74/14.74%、thickness-length-rounding ~14%、inset-025 20.29%、text-emphasis 6-7%）**无一是装饰线首尾空白驱动**——它们是 (a) **table 边框渲染**（thickness-length-rounding 用 table，diff 由 border 主导非装饰厚度）；(b) **font fallback + bidi**（dotted-001/002 用 92px Arial + 希伯来语 `fooשלוםbaz`，本地无 Arial 两侧回退不同字体）；(c) **text-emphasis**（独立子域）。装饰线 trim 首尾空白对它们零影响。**css-text-decor dir 发散由正交 gap（table/font-fallback/bidi/emphasis）主导，非装饰属性逻辑**——同 R912 clean-lever 穷尽结论。下轮勿以「装饰线属性补全」为 css-text-decor yield lever。

### R914 text-decoration-thickness 实现净 -1 oracle（属性 spec-correct·但发散由 table/font/underline-position 主导·已回退）

承 R913。本轮试 **text-decoration-thickness 全 plumbed**（该属性**全仓零实现**，9 触点：css-parser types.rs 枚举 + color.rs parse；style-system types.rs 枚举 + Property 变体 + computed_style.rs 字段 + default_impl.rs + registry.rs 注册+is_inherited + apply.rs + inherit.rs 双函数 + shorthand reset；engine effects.rs paint 消费 `floor(thickness).max(1)` 匹配 chromium rounding：2.3px→2、2.7px→2、0.3px→1）。

**A/B（css-text-decor oracle）**：**72→71（29.8%→29.3%，-1）**。thickness-length-rounding 三案基本不动（~14%，table 边框主导）；但 **thickness-underline-001（6.10%）/overline-001（8.05%）新入 top-15（变差）**——dotted-001/002 略升（15.74→15.94，thicker 装饰线放大 dotted-pattern/font 错位）。**裁决：net -1 oracle，全程回退**（git checkout 9 文件，工作树净零，build 通过）。

**机制（关键）**：thickness 实现**本身 spec-correct**（text-decoration-thickness-length-rounding test 2.3px→floor 2px = ref 2px 自源匹配），但 oracle 净 -1 因：(a) thickness-underline-001 用 `text-decoration-thickness:4em`（80px!）+ Ahem「grow down 覆盖红盒」，**装饰线垂直 position（`y_offset=font_size*0.15`）不随 thickness 变**→80px 厚装饰线在错位铺开，比 thin heuristic 发散更大（暴露**厚装饰线 position gap**，非 thickness 逻辑）；(b) table/font-fallback 主导的案 thickness 修正不可见。**结论：text-decoration-thickness 单独实现非 clean lever**——须配套「厚装饰线 underline position 随 thickness grow down」修复（复杂、Ahem-specific、高风险回归 ~70 passing 装饰案）。css-text-decor 发散**不在装饰属性逻辑层**，同 R913。

### R915 line-metric FontMetricProvider 注入证伪（font-metric Phase A 最后子项关闭·welcome = font-engine 墙·comprehensively confirmed·零源码）

承 R912「下会话 line-metric 实测 de-risk（唯一未证伪 font-metric 子项）」。本轮做 **cheap hypothesis probe**（绕过 FontLoader 全 plumbing，直测假设）：在 `apply_vertical_alignment`（inline/mod.rs:1611 strut ascent + 1634 run ascent）把 `0.8`/`1.0`（Ahem 恰好正确，ascent=800/units_per_em=1000）替换为真实字体比例 `0.928`/`1.17`（hhea/OS2 表典型值，chromium 同源读取），**按 is_ahem 门控**（Ahem 全保留 0.8/1.0 字节一致→WPT 零回归；非-Ahem 用真实比例使 strut 基线对齐 chromium §10.8.1：`baseline = (line_height − em_box)/2 + ascent`，em_box 用真实 ascent−descent 非 font_size）。手算：16px/line-height 1.5，chromium baseline=17.475；ZW current(0.8)=16.8；probe(0.928)=17.475=chromium ✓。

**A/B（product-smoke welcome）**：**16.18% vs baseline 16.11%（+0.07pp，略退步非改善）**。**裁决：hypothesis refuted，全程回退**（git checkout inline/mod.rs，工作树净零，build 通过）。

**机制（comprehensive font-engine wall）**：line-metric 比例替换**不改善 welcome**——welcome 剩余 ~16% **不**由 strut ascent/baseline position 主导（probe 改了基线位置却 net ~0）。结合历史证伪：**font-metric Phase A 全 4 子项现 comprehensively 关闭**——① **advance-width**（R225 双实验 + R375b fontdue-real 三 variant 全证伪）；② **font-matching**（R631 强制 NotoSansCJK 零变化）；③ **rasterization**（R388 fontdue≈chromium per-glyph，非 diff 源）；④ **line-metric**（R915 本轮）。**welcome/morning ~16-17% 残余 = font-engine 系统性差异**（fontdue 对各字体的 metric 提取 + 定位 ≠ chromium 的 Skia/freetasy 栈），**非 CSS/layout 层可解**——须 font-engine 投资（匹配 chromium metric 提取 / 换字体后端），major 多 session，须方向决策。

**裁决（font-metric Phase A 正式关闭）**：R901-R907（multicol +7）后 R908-R915 **连续 8 轮**确认 clean-lever vein **彻底穷尽**——bidi（font-metric/算法）、table-relative（subpixel）、webfont（R910 证伪）、em-resolution（R907 唯一）、text-decor 装饰属性（R913/R914）、font-metric line-metric（R915）全 ruled out。**残余发散全属三类多 session 墙**：(a) **font-engine 投资**（welcome/morning 16-17% + bidi 2.18% + 多 text 案同源 fontdue 度量——最深最高 blast radius，须方向决策非单 session）；(b) **subpixel 渲染精度**（collapsed-borders/table-relative/多 1-1.5% 案，全局 round 策略差，risky）；(c) **结构性**（vertical-rl 定位 / 嵌套 multicol breaking / table 匿名 fixup）。

**▶ 下会话**：font-metric Phase A 已关闭，勿再以单/多 session 投 font-metric（advance/match/raster/line 全证伪）。下会话须在**非-font 三墙**中取一：① **subpixel round 策略审计**（全局坐标 round 影响多 1-1.5% 案，risky 但潜在 blast radius 广）；② **结构性 lever**（vertical-rl clearance x 轴实现 R114/R164 近 miss / 嵌套 multicol breaking / table 匿名 fixup）；③ **font-engine 投资方向决策**（须用户裁决：匹配 chromium metric 提取 vs 换字体后端）。**若无可行单 session lever，接受 plateau 转 DC-1~13 基础设施收尾或其他 goal**。

> **R915 后补充（subpixel 代码核查）**：subpixel-collapsed-borders-001（5px green vs 4.95px red，冲突解决正确 green 胜）的 ~1.08% 发散在**渲染器像素捕捉层**——CPU fill 用保守 `left=floor/right=ceil`（cpu/mod.rs:333-336，向外扩覆盖所有触碰像素，**设计如此避免相邻 fill 间隙**，非 bug；改 `round` 会致相邻盒/单元格/背景-边框出现 1px 缝隙）。**真修 = 上游 layout 级像素栅格捕捉**（chromium 在 paint 前把 LayoutBox position/size snap 到整数像素网格，ZeroWeb 无此层）= 架构级多 session，risky（影响全布局坐标），勿以「改 rasterizer fill round」为 subpixel lever（会造缝隙）。subpixel-collapsed-borders-002/003 同谱系。

### R916 ★ 新发现高价值 lever：reftest harness 不执行 DOM-modifying JS（已实证 100%→1.51%·~176+ 案·机制全在·须接线）

font-metric Phase A 关闭后（R915）扫描 CSS2 oracle（**48.3% = 3009/6232 真一致**，strict 真通过仅 96/1.5%）top worst，发现 **`background-root-101/102/103`（100% diff！）** 等「完全错」案。诊断 = **reftest harness 执行页面 JS 用的是无 DOM 的裸 V8 sandbox**（`reftest.rs:866 execute_scripts`：注释明载「执行但不修改 DOM / 不提供 DOM API / JS 执行结果不影响后续渲染」）→ DOM-modifying JS（`document.getElementsByTagName('head')[0].className='after'` 触发 `head.after + body { background:green }`）静默 no-op → ZeroWeb 截 pre-JS 状态（红），chromium 截 post-JS（绿）→ ~100% diff。

**★ 实证（product-smoke A/B）**：手工把 background-root-101 的 post-JS 状态（head class="after"、p class="after"、html 去 reftest-wait）渲染对比 chromium oracle = **1.51%（从 100% 降）**。证 JS 是唯一 gap（CSS `head.after + body` 相邻兄弟选择器 + canvas 背景传播**均已正确工作**，残余 1.51% 仅粗体文字 font/AA 噪声）。

**影响面（实测 grep）**：全 wpt-data **176 个 `reftest-wait`** + **1250 个含 `<script>`**（CSS2 内 56 + 487）。其中 DOM-mutating（className/style swap、动态内容、reftest-wait class 移除）的子集全部受此 gap 影响——潜在翻转数百案（CSS2 48% 可观上升）。

**机制全在（非新建，须接线）**：DOM-mutation 全链路 ZeroWeb **已实现且在 renderer/browser 活跃**，仅 reftest 未接：
- `generate_js_dom_shim()`（engine/js_dom_bridge.rs:490，selector-based，含 className/setAttribute/textContent/createElement/appendChild/addEventListener）。
- `register_dom_callbacks(sandbox, mutations, dom_html, page_url)`（apps/renderer/src/js_worker.rs:209 / tab_js_worker.rs 同构）注册 `__zw_set_attr/__zw_set_text/__zw_create_element/__zw_append_child/...` 回调，收集 `DomMutation`（selector-based）到 `Arc<Mutex<Vec<DomMutation>>>`。
- `apply_mutations_to_html(html, &mutations) -> Result<String,String>`（engine/js_dom_bridge.rs:353）+ `apply_dom_mutations(doc,&mutations)`（155）应用回 HTML/DOM。
- browser `tab_scripts.rs:219 apply_recorded_mutations` 已是完整 cycle（collect → apply_mutations_to_html → reload_html_after_script）。

**gap = reftest 的两条渲染路径都走 `execute_scripts` 裸 sandbox**：`render_to_framebuffer_with_base`（reftest.rs）+ `render_via_webview_to_framebuffer_with_base`（reftest.rs:550，line 556 同样裸 `execute_scripts`）。两条都须改为「shim + callbacks → 跑页面 `<script>` + `onload` → apply_mutations_to_html → 重渲染 mutated HTML」。

**⚠️ 实现要点（下会话 START）**：
1. **抽取共享**：`register_dom_callbacks` 当前 renderer/browser 各一份（私有，依赖 `query_attr_from_html`/`query_all_selector_list`）。须抽到 `zero-engine`（或 `zero-script-sandbox`）作 `pub`，三处（renderer/browser/reftest）共用，避免第三份拷贝。
2. **reftest 接线**：在 reftest render 路径，`extract_page_scripts`（pipeline.rs:725 已有）提取 inline `<script>` + `onload` handler → 注入 shim + 注册 callbacks → `sandbox.execute` → 收集 mutations → `apply_mutations_to_html` → 用 mutated HTML 重渲染。**须处理 `onload="setTimeout(test,5)"`**（setTimeout 异步——reftest 须同步等待或立即执行 onload 内同步部分）+ **`reftest-wait` class 移除**（截图时机）。
3. **A/B 验证**：background-root-101/102/103 应 100%→~1.5%；全 CSS2 oracle 48.3% 应上升；self-source 不退化（mutated HTML 经同一 renderer）；product-smoke 不退化。
4. **风险**：JS 执行可能改变非预期案（须 A/B 全 CSS2 + 抽样其他 dir 防 net 负）；setTimeout/onload 异步语义须限定（reftest 同步，仅跑 onload body + 0ms setTimeout）。

**裁决（vein 重开）**：R908-R915 关闭 CSS-layout clean-lever + font-metric 后，R916 **重开 harness-lever vein**——非渲染 bug 而是测试 harness 缺陷，但同样降 oracle mismatch，且机制全在、已实证、影响面大。这是当前**最高优先级、最高 blast radius 的可推进 lever**（远优于 subpixel 墙 / font-engine 投资）。下会话直接 START 抽取 + 接线。

### R917 reftest harness DOM-mutating JS 接线 LANDED（机制正确·聚合持平 +1·零回归·harness 真实执行 JS）

承 R916「下会话 START 抽取 + 接线」。**完成抽取 + 接线 + insertBefore shim 补全**，harness 从「裸 sandbox 忽略 DOM 变更」升级到「真正执行 DOM-mutating JS 并渲染 post-JS 态」。详见 [`evidence/r916-reftest-harness-skips-dom-modifying-js.txt`](./evidence/r916-reftest-harness-skips-dom-modifying-js.txt)。

**实施（net 源码）**：
1. **抽取共享**：`register_dom_callbacks` 从 renderer/browser 两份 251 行私有副本抽到 `zero-engine::js_dom_bridge::register_dom_callbacks`（pub）。`zero-engine` 加 `zero-script-sandbox` 依赖（叶 crate，无环）。三处（renderer/browser/reftest）共用，消除第三份拷贝。
2. **reftest 接线**：新增 `apply_scripted_dom_mutations(html, base_dir)` 替换 `execute_scripts`，两条渲染路径（`render_to_framebuffer_with_base` + `render_via_webview_to_framebuffer_with_base`）均接入。流程：`extract_page_scripts` + `extract_onload_handlers` → `V8Sandbox(persistent_context)` + `register_dom_callbacks` + `generate_js_dom_shim` → 按序执行内联/外链脚本 → 派发 `load` 事件（执行 `<body onload>` handler 体 + window `load` 监听器）→ `apply_mutations_to_html` → 用 mutated HTML 重渲染。shim 的 `setTimeout` 经 microtask 立即跑（V8 `execute` 返回前 `perform_microtask_checkpoint`）。
3. **insertBefore shim 补全**：补 shim `insertBefore`（含 `ref=null`→append 语义）+ `__zw_insert_before[_handle]` 回调 + `DomMutation::InsertBefore[_ByHandle]` 枚举 + `apply_dom_mutations` 处理（复用 `Document::insert_before`）+ proxy 暴露 `__zwSelector`。

**测量（`make reftest-oracle DIR=css/CSS2`，6232 案）**：
- 基线 R916：3009/6232 = 48.3%
- R917 接线后：3010/6232 = 48.3%（**净 +1**）
- DOM mutation 触发广度：**118 个 case 产生 mutation（共 2278 条）**——wiring 被广泛触发
- **insertBefore 失败：64 → 0**（shim 补全前 `insertBefore is not a function` 是 #1 失败，64 case 中断 DOM 构建；补全后归零）

**★ 调查：为何聚合持平 +1（非 R916 预测的大幅 yield）**：
- **A. background-root-101/102 head+body 背景渲染 bug（独立，非 JS wiring）**：即便 mutation 正确应用（实测 mutated HTML 含 `<head class="after">`），渲染仍 ~100% 发散。逐项隔离：最小 HTML + CDATA + head 子元素 + DOCTYPE 全部正确渲染（绿/红），但真实 background-root-101.xht 完整文档 body 盒内 (400,30) = 灰 216、canvas = 白 → body 背景在完整真实文档中不正确染色/不传播到 canvas。**此 bug 影响未变异文件**（head=before 时也应红却白），非 JS wiring 所致 = 独立多会话调查。R916「手动 post-JS=1.51%」可能用了更简洁手工 HTML。
- **B. insertBefore 缺失（已修）**：补全后 DOM 构建完整，但动态内容 reftest 多不翻转，因渲染层对 JS 动态构建的结构（动态 span/div 列表、动态样式切换）有次级缺口。聚合 +1 反映：DOM-mutating JS 现正确执行，但**下游渲染/布局对动态结构的支持是下一个 yield 杠杆**（独立多会话）。

**门禁（全绿）**：`make test` exit 0；`make product-smoke` welcome 16.11%（< 20% DC-13 gate，零回归）；`cargo clippy --workspace --all-targets -D warnings` clean；`cargo fmt` clean。

**意义**：reftest harness 的 JS 执行能力达到正确（DC-1 真实正确性提升）——能驱动 118 个 DOM-mutating case 的 JS（含 insertBefore 全 API）。聚合 reftest 通过率受次级渲染 bug 阻塞（独立 lever），但 harness 本身不再因「忽略 DOM 变更」而失真。browser/renderer 的 JS 路径现也支持 insertBefore（基础 DOM API，真实网页常用）。

**▶ 下会话**：① 调查 background-root head+body 背景在完整文档不染色的根因（body 背景传播到 canvas 的条件 / 相邻兄弟在完整 head 子树下的匹配）——可能解锁 background-root 簇 + body-background 系列的真实 yield；② 扫描动态内容 reftest 的次级渲染缺口（JS 正确构建 DOM 后渲染发散的 case 逐项根因）；③ shim 其他缺口（createElementNS 2 / cloneNode 1 / getComputedStyle 1 / createDocumentFragment 1，量小）。**R917 把 harness JS 执行拉到正确，yield 阻塞转移到渲染层。**

**★ R917 续（同 session）：dom serializer raw text 转义 bug 修复——R917 真正的 yield 阻塞根因**：调查 background-root-101 head+body 背景不染色时定位到真根因 = **`zero_dom` serializer 对 raw text 元素内容做了 HTML 转义**。`serialize_node_inner` 对所有 `Text` 节点一律 `escape_text`，但 `<script>`/`<style>` 是 raw text 元素，内容应原样保留。后果链：`apply_mutations_to_html`（R917 接线核心）= parse → apply → `outer_html` 序列化；`<style>` 内 `<![CDATA[` 被序列化成 `&lt;![CDATA[`；再 parse 时 CSS 解析器首字符 `&lt;`（5 字符实体）错误恢复贪婪吞噬到 `]` → CSS 规则全破坏 → body 背景不应用。**这解释了为何 R917 接线对 background-root 簇无效**——roundtrip 破坏了 CDATA CSS。修复（`crates/dom/src/serializer.rs`）：`serialize_node_inner_ctx` 带 `parent_tag` 上下文；Text 节点父为 raw text（script/style）→ 不转义，否则 `escape_text`；`inner_html` 同传父标签；新增 `is_raw_text_element`（script|style；textarea/title 是 rcdata 用 escape_text 正确）+ 2 回归测试。**验证**：background-root-101/102/103 oracle ~100%→**2.01%**（roundtrip 后 `<style>` 正确保留 `<![CDATA[`，head=after→body 绿）；3 案降到 2% 仍未过 1% 阈值故聚合 CSS2 仍 3010（持平），但修复是真实 serializer 保真度提升，影响所有 script/style roundtrip（JS DOM-mutation + innerHTML）。门禁全绿（make test dom 758 含 2 新测试 / product-smoke welcome 16.11% / clippy clean）。**意义**：dom serializer 对 raw text 的正确序列化是 `apply_mutations_to_html` 和 innerHTML 的基础保真前提；修复后 R917 的 JS wiring 才在含 CDATA 的 XHTML 测试上真正生效。详见 [`evidence/r916-reftest-harness-skips-dom-modifying-js.txt`](./evidence/r916-reftest-harness-skips-dom-modifying-js.txt) R917 续节。

### R918 reftest harness 现代 JS 模式 shim LANDED（rAF/takeScreenshot/append/getBoundingClientRect·DC-1 正确性·CSS2 +1·零回归）

承 R917 续。复查 R917 跑出的 JS 执行警告，发现**还有一类 harness JS 缺陷未被 R917 覆盖**——现代 reftest 常用的全局 API / 方法根本未 shim，导致脚本回调不触发或方法调用抛异常中断：① `requestAnimationFrame`（rAF）未 shim → 现代动态 reftest 标准模式 `requestAnimationFrame(() => requestAnimationFrame(() => { …setup…; takeScreenshot(); }))` 的回调**永不执行** → setup mutation 永不记录 → 渲染 pre-JS 初态（R917 报「118 case 产生 mutation」时，rAF 测试因回调不触发从未进入那 118）；② `takeScreenshot`（`/common/reftest-wait.js` 提供）未定义 → 抛异常；③ `Element.append`（现代 API）未 shim → 中断 DOM 构建；④ `Element.getBoundingClientRect()` 未 shim → proxy 对未知属性返回 undefined → 调用 = `undefined()` 抛 TypeError → **中断整个脚本**，其后 mutation 全丢（120 文件用作 reflow 触发器）。

**频率**（全 wpt-data 13707 非-ref 文件）：rAF 42 / takeScreenshot 27 / getBoundingClientRect 120（13 后跟结构性 mutation）/ .append 19。注：offsetHeight(203)/offsetWidth(140) 是属性访问，返回 undefined 不抛、仅值错误，作 reflow 触发器无害；**不特例化以免改变 `<` 条件逻辑**（精准修改原则）。

**实施**（`crates/engine/src/js_dom_shim.js`，纯 JS，零 Rust）：① `requestAnimationFrame(fn)` 同步立即执行（`_rafBudget=64` 上限防动画无限链式，`__zw_begin_script` 重置）+ `cancelAnimationFrame` no-op + webkit/moz 别名；② `takeScreenshot(cb)` no-op（harness 在 load 后截图）+ 调 cb + 返回 resolved Promise；③ `Element.append(...items)` JS-only 复用 `__zw_append_child[_handle]`+`__zw_create_text`（字符串包 Text 节点）；④ `getBoundingClientRect()` 返回零 DOMRect / `getClientRects()` 返回 `[]`。新增 `test_shim_includes_modern_reftest_stubs`（zero-engine lib 10 passed）。

**A/B**（`make reftest-oracle DIR=css/CSS2`，6232 案）：基线 R917 3009 → R917 续 3010 → **R918 rAF-only 3011** → R918 合并（+gBCR）3011（CSS2 gBCR 多纯 reflow 触发，+0）。**R918 CSS2 净：oracle-pass 3010→3011 (+1) / credible +1 / 近似通过 +1 / strict 真通过 95→95（字体光栅 strict 噪声封顶）**。yield 受限 = **字体墙**（R388/R633 一致）：rAF 测试多 dynamic-color，shim 后 mutation 正确触发渲染正确颜色，但字形光栅噪声使 diff 停 ~1.04% 刚过 1% 阈值（例：list-item-dynamic-color product-smoke 验证 rAF 触发渲染绿色，diff 1.04% = 绿字 font/AA 噪声）。**结构性动态测试不同**：normal-flow/block-in-inline-{append,empty}-* 经 shim 后 diff 0.06-0.11%（接近/低于 strict 阈值），证 shim 对结构动态有效（mutation 触发且渲染正确）。

**门禁全绿**：make test exit 0（zero-engine 1175 passed 含新增测试，全 workspace 0 FAILED）/ make product-smoke welcome **16.11%**（< 20% DC-13 gate 零回归）/ clippy --workspace --all-targets -D warnings clean / fmt clean。详见 [`evidence/r918-reftest-harness-modern-js-shims.txt`](./evidence/r918-reftest-harness-modern-js-shims.txt)。

**意义**：reftest harness 的 JS 执行从「能跑 DOM-mutating JS」（R917）进一步到「能跑现代 reftest JS 模式（rAF 延迟 setup / reflow 触发 / takeScreenshot 完成信号 / append 批量追加）」。DC-1 真实正确性提升——harness 不再因缺失全局 API 静默跳过或异常中断现代 reftest setup 脚本。CSS2 oracle yield 受字体墙封顶（+1），但 css-tables 等现代目录（13 个 gBCR+结构性 mutation case）潜在更高 yield，留待 A/B。

**▶ 下会话**：① css-tables/css-flexbox 现代目录 R918 shim A/B（gBCR+结构性 mutation 13 case，潜在更高 yield）；② shim 其余低频缺口（createElementNS 5 / cloneNode 7 / createDocumentFragment 1；getComputedStyle 41 须 layout 查询较复杂）；③ background-root-101 残余 2.01% = 粗体字体墙（R917 续后非 clean lever）。**R918 把 harness 现代 JS 能力补齐，yield 在 CSS2 受字体墙封顶，现代目录待 A/B。**

### R919 named access on window shim LANDED（id→全局·257 文件·css-multicol +4 / css-tables -1 诚实暴露·净 +3·零回归·DC-1 正确性）

承 R918「现代目录 A/B」。R918 之后复查 css-tables/insert-after-col.html（16.33%），定位下一个 harness JS 缺口：脚本用**裸标识符**引用元素（`container.appendChild(header)`），但 `<div id="container">` 的 id 未暴露为 JS 全局 → `ReferenceError: container is not defined` → 中断脚本 → appendChild 永不执行。这是 **HTML 规范「Window 对象上的命名属性访问」**（named access）：带 id 的元素应作为 `window` 命名属性可访问。ZeroWeb JS shim 缺失。**频率**：全 wpt-data **257 个**含 script+id 的文件用裸标识符引用元素（multicol-span-all-dynamic-add / intrinsic-width-change-column-count 等结构性动态测试）。

**实施**：Rust 端 `collect_element_ids(html)`（parse→`query_selector_all("[id]")`→去重保序）+ `__zw_collect_ids` 回调 + 2 单测；JS shim `_installNamedAccess()`（`__zw_collect_ids` → 对合法标识符 id 且 `globalThis[id]===undefined` 不覆盖已存在全局 → `globalThis[id]=getElementById(id)`）在 IIFE 末尾调一次。

**A/B（R918-only vs R918+R919）**：**css-multicol（452 案）111→115 净 +4**（结构性动态测试现正确执行 setup mutation）；**css-tables（115 案）62→61 净 -1**：insert-after-col.html **16.33%→0.00%**（FAIL→PASS，字节精确匹配 oracle，证 R919 机制正确）+ 2 case 翻 PASS→FAIL = **DC-14 anti-false-pass 诚实暴露**（测试之前因脚本不运行、初态碰巧匹配 oracle 而假通过，R919 让正确 post-mutation 态渲染后暴露预存渲染 bug）。**两目录合计净 +3 oracle-pass**。

**门禁全绿**：make test exit 0（zero-engine 1177 passed 含 2 新单测，全 workspace 0 FAILED）/ make product-smoke welcome **16.11%**（< 20% DC-13 gate 零回归——welcome 的 id 不被脚本裸引用，named-access 无副作用）/ clippy clean / fmt clean。详见 [`evidence/r919-named-access-on-window-shim.txt`](./evidence/r919-named-access-on-window-shim.txt)。

**意义**：harness JS 执行从「能跑现代 reftest JS 模式」（R918）进一步到「能跑裸标识符元素引用」（HTML 规范 named access）。DC-1 正确性提升——不再因缺失 named access 中断 257 个 reftest 的 setup 脚本。css-multicol 净 +4 实质 yield；css-tables 净 -1 是诚实暴露（2 真渲染 bug 显形）。

**▶ 下会话**：① 调查 css-tables R919 暴露的 2 个真渲染 bug（脚本正确执行后发散的 case）逐项根因，可能解锁 css-tables 真实 yield；② R919 在其他现代目录（css-flexbox/css-grid/css-position）A/B（潜在更多 yield）；③ named access 现为 init 快照，脚本动态 createElement+id 的元素不在内（罕见，可后续扩展）。**harness JS vein（R917→R918→R919）三连：CSS2 +1 / css-multicol +4 / css-tables 净 -1（诚实暴露），现代目录仍有 yield 空间。**

**★ R919 续（同 session）：css-tables 暴露 bug 调查 = 全结构性（非 clean lever·关闭）**：渲染 13 个 css-tables bare-id case（R919 触发 named access），8 PASS（insert-after-col 0.00% / collapsed-border-partial-invalidation 0.08-0.14% / abspos-container-change-dynamic 0.49% / collapsed-border-remove-cell 0.42% / -remove-row-group 0.64% / table-cell-overflow-auto-scrolled 0.35%），5 FAIL（fixup-dynamic-anonymous-inline-table-001/002/003 = 1.68-2.74% / fixup-dynamic-anonymous-table-001 = 2.74% / insert-after-colgroup = 1.91%）。**逐项核查 mutation 逻辑正确**（insert-after-colgroup：CreateElement+SetInnerHtmlOnHandle+InsertBefore 三 mutation 经 `apply_dom_mutations` 路径核查无误，ref `tbody` 在新 tbody 未入树前正确解析为既有 tbody，`doc.insert_before` 位置正确）→ **diff = 表格渲染保真度**（anonymous table box 动态生成 = R255/R109 谱系 / colgroup + 多 tbody = R177 谱系 / collapsed-border 动态失效），均为结构性多会话，非单 session clean fix。**裁决**：css-tables 暴露 bug 调查关闭（结构性），harness JS vein（R917→R919）DC-1 正确性提升 + DC-2 yield（css-multicol +4）已交付，残余 yield 须待表格结构性（R177/R255 anonymous-table fixup）多会话。下会话可：① R919 在 css-flexbox/grid/position A/B（测更多 yield）；② shim 低频缺口（cloneNode 7/createElementNS 5/createDocumentFragment 1）；③ 转 multicol Phase 2 / R109 等结构性 lever（harness 已能忠实执行 JS，结构性 bug 现可被正确暴露与验证）。

**★ R919 续 2（同 session）：CSS2@R919 无回归确认 = 3012（+1 on top of R918）**：复核 CSS2 oracle at current HEAD（R919，named-access 已含）= **3012/6232 (48.3%)** vs R918 3011 → R919 在 CSS2 **净 +1 零回归**（named-access 解锁 CSS2 内若干 bare-id 动态测试，且不破坏 R918 的 +1）。**R919 完整 yield 汇总**：CSS2 +1 / css-multicol +4 / css-tables -1（诚实暴露）= **跨 3 目录净 +4**；R918+R919 在 CSS2 合计 +2（3009→3012）。harness JS vein（R917→R919）DC-1 正确性 + DC-2 实质 yield 交付完成，残余全结构性多会话。

**★ R919 续 3（同 session）：fixup-dynamic-anonymous-inline-table-001 精确诊断（CSS2 §17.2.1.1 匿名表对象生成缺失·R109 交叠·结构性·给未来 session 精确指针）**：深入最小 diff（1.68%）的暴露 case。结构 = 外层 `<span>`(inline) 含两个 `display:table-row-group` 兄弟（各含 100×50 绿 cell）+ 中间 `<span id="rm">`；脚本 `rm.remove()` 后两 row-group 相邻，CSS §17.2.1.1 要求**合并到一个匿名 inline-table**（100×100）。**ZW 渲染**：绿 bbox 5000px = 100×**50**（仅一个 row-group）；**oracle**：10000px = 100×**100**（两 row-group 堆叠）。**LAYOUT_DUMP 实证**：两个 `span.group` 都 `abs_y=51.0 h=50.0`（**重叠**非堆叠）。**双根因**：(a) **table-fixup merge gap** — `adjust_table_layout_inner`（table.rs:80-86）对每个孤立 table-internal 子元素**独立**调 `layout_table`，连续 row-group 兄弟未按 §17.2.1.1 合并到一个匿名表（`mark_anonymous_table_roots` float_positioning.rs:193 仅置 per-node `is_anon_table_root` 标志供 BFC 用，**不**创建合并包装盒）；(b) **R109 block-in-inline** — 两个独立 anon 表（`is_block_level=true`）落 inline `<span>` 内，第二个不垂直堆叠（都 y=51）。**裁决**：结构性多会话（table-fixup 需 box-tree 重构分组合并 + R109 block-in-inline 堆叠），非单 session clean fix。**修复指针（未来 session）**：在 box-tree 构建（children 生成处）加分组 pass——非 table 父的连续 table-internal 子元素合并到一个匿名 table 包装盒（CSS §17.2.1.1），再 `layout_table` 该包装；此同时解 (a) 与（部分）(b)。影响面：fixup-dynamic-anonymous-{inline-,}table-001/002/003（4-5 css-tables case）+ CSS2 内类似 anonymous table fixup case。风险：中高（影响所有 table fixup 路径，须全量 css-tables + CSS2 tables 子目录 A/B 守回归）。

### R920 createElementNS shim LANDED（XHTML alias createElement·SVG OOS 不中断·零回归·DC-1 DOM API 补齐）

承 R919 续「shim 低频缺口」。`document.createElementNS(ns, tag)` 未 shim → 用它的脚本（first-letter-dynamic 创建 25 个 `<style>`、cascade-import-dynamic 创建 `<link>`）抛 ReferenceError 中断。**实施**（`js_dom_shim.js`，1 行）：`createElementNS(_ns, tag)` alias `createElement(tag)`——HTML 命名空间（xhtml）与 createElement 等价；SVG 命名空间（filter/cursor 等）按通用元素创建（本目标 SVG 渲染 OOS，但不抛 ReferenceError 中断脚本，crashtest 尤其依赖）。新增 `test_shim_includes_modern_reftest_stubs` 断言。**验证**：first-letter-dynamic-001.xht（createElementNS xhtml style × 25 + appendChild）= **0.14%（PASS）**，脚本正确执行。cloneNode（7 文件）评估后**跳过**——其 case 多在结构性 wall 下游（block-in-inline-append=R109 / table-anonymous-objects=fixup bug / border-collapse），cloneNode 单独 yield 小，且实现中等复杂（深拷贝子树 + 新 DomMutation 变体），ROI 不足。

**门禁全绿**：make test exit 0（zero-engine 12 passed，workspace 0 FAILED）/ product-smoke welcome 16.11%（< 20% DC-13 gate 零回归）/ clippy clean / fmt clean。

**意义**：harness DOM API 面进一步补齐（createElementNS）。残余低频缺口 = createDocumentFragment(1)/getComputedStyle(41，须 layout 查询较复杂)；cloneNode(7) 评估后跳过。harness JS vein 全部高 ROI shim 已交付（R917 DOM-mutation + R918 rAF/gBCR/append + R919 named-access + R920 createElementNS）；下会话转向结构性 lever。

### R921 harness JS vein 累计 yield 复测：css-flexbox 50.6%→56.1%（+27 case·harness vein 跨现代目录显著收益·基线更新）

复测 css-flexbox oracle at current HEAD（post R917-R920 harness vein）= **279/497 = 56.1%**（vs stale baseline 50.6% pre-R917，R456-era）→ harness JS vein 在 flexbox 提升 **~+5.5pp / +27 case**。flexbox 动态测试密集（flex-item-dynamic-* / abspos-dynamic / display 动态切换），R917（DOM-mutating JS）+R918（rAF/gBCR/append）+R919（named-access）+R920（createElementNS）组合解锁大量 setup mutation 正确执行。**harness JS vein 累计 yield（跨目录）**：CSS2 +2（3009→3012）/ css-multicol +4（111→115）/ css-tables -1（诚实暴露）/ **css-flexbox +27** ≈ **+32 case**。strict 真通过：CSS2 95（1.5%）/ flexbox 29（5.8%）— flexbox strict 反高于 CSS2（flexbox 布局类无文字光栅噪声主导，更多 case 过 strict 0.1%）。

css-flexbox worst-15（post-harness）**全是已知结构性簇**：flex-minimum-width-flex-items-013（82%）/ aspect-ratio-intrinsic-size-007/003/004/011/014（taffy-blocked intrinsic size）/ flex-abspos-inset-nested-002 + cross-size-001（R363 abspos nested dead-end）/ flexbox-flex-flow-001/002 / flexbox-collapsed-item-horiz-001（R111 collapse 残余）/ content-height-with-scrollbars（scrollbar）。**harness vein 已捕获动态测试 yield，残余全结构性 plateau**（单会话 clean lever 穷尽，与 R491-R919 一致）。

**裁决**：harness JS vein（R917-R920）是本 rally 阶段最大净收益（跨 4 目录 +32 case + DC-1 正确性大幅提升）。残余前向路径全结构性多会话：table-fixup merge（R919 续 3 精确诊断）/ multicol Phase 2 commit-2（R899 low-yield high-risk）/ R109 block-in-inline 堆叠 / aspect-ratio（taffy-blocked，须 R304 升级）/ baseline-export。flexbox 56.1% / multicol 25.4% / tables 53.0% / CSS2 48.3% 为 post-harness 新基线。

### R922 table-fixup merge LANDED（CSS2 §17.2.1.1 连续孤立 table-internal 兄弟合并到一个匿名表·css-tables +4·CSS2 tables 380 案零回归·clean win）

承 R919 续 3「修复指针」。实施 CSS2 §17.2.1.1 匿名表对象生成的「连续兄弟合并」部分。**实施**（`crates/layout-engine/src/table.rs`，net 源码）：① `adjust_table_layout_inner` else-branch 改为先识别连续孤立 table-internal 兄弟的 run（run_len≥2 合并，=1 走原逻辑）；② 新函数 `merge_orphan_table_run`：计算 run 原**垂直 footprint**（max bottom − min top，正确反映重叠）→ drain run 子元素到新匿名 `LayoutBox`（`LayoutBox::default()`，无 node_id，`is_anon_table_root=true`+`is_block_level=true`，继承首位 child 的 x/y/width + 父 writing_mode）→ 插入 run_start → `layout_table(wrapper)`（build_grid 正常路径：wrapper 无 node_id → get_display None → is_orphan=false → 收集多 row-group → 多行堆叠）→ 按 `new_height − old_footprint` 推进后续兄弟 y + 调整父高度。匿名包装盒无 node_id 安全：paint/layout 全 None-safe（`let Some(style)=styles.get(&node_id)`，无 unwrap/expect）。

**A/B（干净 stash 对比）**：**css/css-tables（115 案）baseline 61 → R922 65（+4 oracle-pass）**——fixup-dynamic-anonymous-inline-table-001 1.68%→**0.68%（PASS）**，绿 bbox 5000px(100×50)→10000px(**100×100**) 与 oracle 一致；同簇 fixup-002/003 + fixup-dynamic-anonymous-table-001 亦翻转。**css/CSS2/tables（380 案）baseline 46 → R922 46（零回归，380 案字节级一致）**——merge 仅触发于「连续孤立 table-internal 兄弟」结构（run_len≥2），单 row-group / 真 table / 已在 table 内 case 走原逻辑不受影响。

**门禁全绿**：make test exit 0（workspace 0 FAILED）/ make product-smoke welcome **16.11%**（< 20% DC-13 gate 零回归）/ clippy clean / fmt clean。详见 [`evidence/r922-table-fixup-merge-anonymous-table.txt`](./evidence/r922-table-fixup-merge-anonymous-table.txt)。

**意义**：CSS2 §17.2.1.1 匿名表对象生成核心缺口补上——连续孤立 table-internal 兄弟现正确合并到一个匿名表（多 row-group → 多行堆叠），而非各自独立重叠。是 R919 named-access 暴露的 table 渲染 bug 的直接修复。**注意**：本修复是 §17.2.1.1「连续兄弟合并」部分；R109 block-in-inline 多块堆叠（独立匿名块盒生成）仍未实现，但 merge 绕过了该需求（合并后仅一个 block-level 匿名表，无需多块堆叠）。

**▶ 下会话**：① R109 block-in-inline 多块垂直堆叠（纯 block-level 子元素在 inline 内，merge 绕过了 table 案但通用 block-in-inline 案仍受影响，如 block-in-inline 动态测试）；② multicol Phase 2 commit-2 / aspect-ratio（taffy-blocked）/ baseline-export 等其他结构性 lever；③ table-fixup merge 现 run_len≥2；单 row-group 内嵌套 row-group 水平合并（build_grid is_orphan 既有）未动。**R922 clean win（+4 css-tables，零回归），证明结构性 lever 在精确诊断 + 干净 A/B 下可单 session 推进。**

### R923 全目录 post-harness+R922 基线复测：css-position 37.9%→52.6%（+15）·累计 harness vein+R922 yield +52 case（跨 5 目录·重大里程碑）

复测剩余 2 个布局类目录 at current HEAD（post R917-R920 harness vein + R922 table-fixup merge）：
- **css/css-position（97 案）= 51/97 = 52.6%**（vs stale 37.9% pre-R917）→ **+14.7pp / +15 case**。动态 abspos 测试密集（position-absolute-dynamic-relayout / hypothetical-dynamic-change），harness vein（R917 DOM-mutation + R918 rAF/gBCR + R919 named-access）解锁大量 setup mutation。
- **css/css-grid（49 案）= 18/49 = 36.7%**（vs stale 39.6%）→ -1（honest exposure：R917-R919 harness 让 1-2 个动态 grid 测试正确执行后暴露真渲染 bug，非 R922 回归——grid 不涉及 table-fixup）。

**★ harness JS vein（R917-R920）+ R922 table-fixup merge 累计 yield 汇总（跨 5 布局类 + CSS2 目录）**：
| 目录 | stale 基线 | post-harness+R922 | Δ |
|------|-----------|-------------------|----|
| css-flexbox | 50.6% | 56.1% | **+27** |
| css-position | 37.9% | 52.6% | **+15** |
| css-multicol | 23.5% | 25.4% | +4 |
| css-tables | 53.0%(post-R919) | 56.5% | +4（R922）|
| CSS2 | 48.3% | 48.3% | +2 |
| css-grid | 39.6% | 36.7% | -1（honest）|
**累计 ≈ +52 case**（harness vein +32 + R922 +4 + position/grid 复测 +15/-1）。这是本 rally 阶段（R917-R923）最大净收益——harness 现忠实执行现代 reftest JS + R922 table-fixup merge clean win。

**残余 worst 全结构性**：css-position（backdrop-filter OOS / semi-replaced form-control R887 / JS dynamic relayout / R109 in-inline / dashed-border phase noise）/ css-grid（baseline-export / grid block-size / table-grid-item R168 / form-control）。table-row-group-color-inheritance-001（9.07%）诊断 = paint 显式跳过 table-internal 直文本（painter mod.rs:454-477）+ build_grid 不合成匿名 cell → 文本永不渲染；修复须 box-tree 文本→cell 合成（比 R922 更深入，yield ~3 case），ROI 不足 defer。

**post-harness+R922 真基线**：flexbox 56.1% / position 52.6% / tables 56.5% / multicol 25.4% / grid 36.7% / CSS2 48.3%。forward motion 全结构性多会话（baseline-export / multicol Phase 2 / R109 / aspect-ratio taffy-blocked / form-control 内在尺寸）。

### R924 css-tables residual 证伪 + baseline-export 证伪：post-R922 clean lever 全结构性（plateau 铁定·给未来 session 勿重扫清单）

承 R923「forward motion 全结构性」。逐项诊断 post-R922 css-tables worst-list + baseline-export 簇，确认**无 R922 式 clean fix**（R922 是 css-tables vein 最后一个 clean win）：
- **table-row-group-color-inheritance-001（9.07%）**：paint 显式跳过 table-internal 直文本（painter/mod.rs:454-477）+ build_grid 不合成匿名 cell → row-group 直文本子永不渲染。修复须 box-tree 文本→cell 合成（比 R922 深，~3 case yield），defer。
- **table-cell-overflow-explicit-height-001（9.73%）**：LAYOUT_DUMP 实证 table auto-width 塌缩（w=10 应~150）+ div.tall 背景未绘制（w=4 blue=0）+ td h=304（高度增长）。多重纠缠（table auto-width + cell block content + 背景绘制），R767/R868 cell-content-width 谱系，非单点，defer。
- **grid-container-baseline-synthesized-001~004（16-17%）**：inline-grid 从空 item 合成 baseline + 4 writing-modes（vrl/vlr/srl/slr）= baseline-export + 写作模式深度结构性（taffy baseline 重构或自建 inline-level-box baseline 合成），多会话，defer。
- **row-group-order（5.26%）**：border-collapse:collapse + 10px borders（R342c 复杂边框冲突），非 order bug（build_grid row_group_sort_priority 已排序 thead/tbody/tfoot），defer。
- **baseline-vertical / table-cell-width-0 / percent-height-overflow / min-max-size-table-content-box**：baseline-export / R97 intrinsic（taffy-blocked）/ 复杂 % height / R365 subtle sizing，全结构性。

**裁决**：post-R922 css-tables（56.5%）+ css-grid（36.7%）+ css-position（52.6%）residual worst-list 全结构性，单 session clean lever 铁定穷尽。本 rally 阶段（R917-R924）累计 **+52 case**（harness vein R917-R920 + R922 table-fixup merge）已是当前架构下最大单会话/近会话收益。forward motion 全部进入**多会话硬核架构**：① multicol Phase 2 commit-2（layout 侧 column-aware IFC，R897/R898 enabling slices 就绪，multicol 25.4% 最低 oracle 最多 headroom）；② baseline-export（taffy-internal baseline_overrides 须 R304 升级，或自建 inline-level-box baseline 合成）；③ R109 block-in-inline 多块堆叠；④ aspect-ratio（taffy-blocked，R304）；⑤ Phase A IFC 统一（font-metric 死锁）。**下会话首选 multicol Phase 2 commit-2**（headroom 最大 + enabling infra 就绪）。

### R925 multicol Phase 2 闭环 + baseline-export 精细化诊断（API 已存在·gap = 空 item + writing-mode baseline 合成·比 R304 可治·控制面修正）

承 R924「下会话首选 multicol Phase 2 commit-2 / baseline-export」。两项深查：

**① multicol Phase 2 commit-2 彻底关闭**：post-harness 扫 css-multicol **pure-inline balance** driving case = **0**（R381 pre-harness 0/16 结论 post-harness 仍成立，harness 没解锁此类）；block-child fragmentation = R899 risky（IFC 须按列宽重排 + paint 侧 4 轮证伪）。inline-only auto 已 R901 落地。**multicol Phase 2 commit-2 无 clean 单 session 路径**（balance 0 driving + block-child R899），勿重试。

**② baseline-export 精细化（控制面修正·比想象可治）**：核查发现 **taffy baseline_overrides API 已接线**（engine.rs:1311-1421 `with_baseline_overrides` + taffy-local cached_baselines patch）——**不需要 R304 升级**！综合裁决「baseline-export 须 taffy 0.8+ 或自建」**过时**。真 gap 更窄：engine.rs:1335 `if child.children.is_empty() return None`（inline-flex/grid 容器空时不合成 baseline）+ 第一行 item 为空时未按 CSS Writing Modes §4 合成 baseline（alphabetic = margin-box under-edge；vertical writing-mode 须合成 alphabetic 而非 central，见 grid-container-baseline-synthesized-001 assert「synthesize alphabetic not central in htb line-box」）。**grid-container-baseline-synthesized-001~004（16-17%）+ flexbox-baseline 簇**修复路径 = 在 baseline_overrides first_row 循环加空-item baseline 合成（writing-mode 感知）。比 R922 复杂（须 CSS Writing Modes §4 synthesized baseline + htb/vrl/vlr/srl/slr 区分 alphabetic/central），但 API 层已就绪，是**比 taffy 升级更可治的结构性 lever**。

**裁决**：multicol commit-2 闭环（勿重试）；baseline-export 是当前最可治的结构性 lever（API 已存在，gap = 空 item + writing-mode baseline 合成），但须 CSS Writing Modes §4 规范精确实现，单 session 风险中（writing-mode entanglement）。下会话首选 baseline-export 空-item baseline 合成（htb 先做，vertical 后做），A/B 守 css-grid + css-flexbox oracle 零回归。

### R926 baseline §4.4 空-item/空-container 合成 LANDED（css-flexbox +1·零回归·纠正 grid-container-baseline-synthesized 归因 = inline-grid intrinsic width + IFC line-box 双层结构性）

承 R925「下会话首选 baseline-export 空-item baseline 合成」。**实施 engine.rs `adjust_inline_block_positions` baseline_overrides 闭包三处修改**（CSS Writing Modes §4.4 synthesized alphabetic baseline）：① 空 inline-flex/inline-grid 容器（`child.children.is_empty()`）此前 `return None` → IFC 回退 central（h/2）违反 §4.4，改为合成 alphabetic = 容器 margin-box 下沿（`child.height + child.margin_bottom`）；② 第一行首 item 为空（`doc.first_child(id).is_none()`）时 `first_item_bottom` 回退从 content-box 底沿（`c.y+c.content_height`）改 margin-box 下沿（`c.y+c.height+c.margin_bottom`），有内容 item 保留既有启发式（精准不影响 text-item）；③ 最终 guard 从 `baseline < child.content_height` 放宽到 `baseline <= child.height+child.margin_bottom+1.0`（合成基线可落 content-box 外的 border-box/margin-box 下沿，旧 guard 会误拒）。

**A/B 验证（stash 重建 baseline）**：css-flexbox oracle **279→280（+1）**（stash engine.rs=279 / pop=280 实证 +1 来自本修复）；css-grid oracle **18/49 持平零回归**；新单测 `test_empty_inline_flex_synthesizes_alphabetic_baseline` 修复前 FAIL（shift=0 central 忽略 margin_bottom）/ 修复后 PASS（shift≈80）= 测试有效。门禁全绿（make test 全 workspace 0 FAILED / product-smoke welcome 16.11% < 20% / clippy clean / fmt clean）。详见 [`evidence/r926-baseline-export-empty-item-synthesis-2026-07-01.txt`](./evidence/r926-baseline-export-empty-item-synthesis-2026-07-01.txt)。

**★ 关键纠正（grid-container-baseline-synthesized 真根因 ≠ baseline-export）**：R924/R925 把 grid-container-baseline-synthesized-001~004（16-17%）归为 baseline-export lever，本轮 LAYOUT_DUMP + 像素 forensics 推翻：① 探针实测容器 `taffy_baseline=Some(60.0)`（= 正确 margin-box 下沿），仅 guard `60<60` 拒绝落入手动路径（本轮修复后也给 60）→ **baseline 现已正确但 diff 仍 16.23% 不变** = baseline 非主因；② **真根因 = inline-grid intrinsic 宽度 bug**（R370 inline-flex 的 grid 类比）：探针实测 `.container`(grid-template-columns:60px) `width=6`（仅 item border 3+3=6），grid item child `w=60`（正确 track）→ 容器塌缩 6px，cyan 仅画 6px 列（ZW cyan 2671px vs REF/Oracle ~58345px，差 22 倍）；③ 第二层阻塞（width 修后暴露）：IFC line-box 高度 bug——60px 高 inline-grid 行间重叠（band0 高 177px 应 ~57px；inline-block ref 同尺寸却正常 5 分离 band）= Phase A 行盒度量结构性。

**裁决**：baseline §4.4 修复 LANDED（css-flexbox +1 实证 yield + spec-correct + 零回归）；**grid_fixed_track_width helper + InlineGrid grow-or-shrink 接线设计就绪但回退**（实测修后 cyan 2671→48492 宽度正确，但被 IFC line-box 重叠阻塞，synthesized-001 diff 16.08→16.66% 略退步，css-grid oracle 18/49 net-zero → 按「不做零/负价值修改」+ 零收益回退纪律回退，设计留 evidence 供未来 session 复用）。grid-container-baseline-synthesized = (1) inline-grid intrinsic width + (2) IFC line-box height 两层叠加，均结构性多 session（须同修方能过簇），非单 session clean fix。R924「全结构性」结论再确认，但本轮把「baseline-export」误归因纠正为「inline-grid intrinsic width + IFC line-box」精确指针。

**▶ 下会话**：① baseline §4.4 在 css-position/css-multicol A/B（空 inline-flex/grid 容器案潜在更多 yield）；② inline-grid intrinsic width + IFC line-box 两层硬核（多会话同修方能过 synthesized 簇，grid_fixed_track_width 设计已就绪）；③ 其他结构性（R109 block-in-inline / multicol Phase 2 / aspect-ratio taffy-blocked）。

### R927 extract_page_scripts 剥 XHTML CDATA LANDED（harness JS vein 续·CSS2 +1·3196 XHTML 脚本现执行·DC-1 正确性）

承 R926「下会话」。扫 box-display worst（insert-* 动态簇 20-40%）定位 harness JS 缺口：`extract_page_scripts`（pipeline.rs:745）取 `<script>` `text_content` 直接当 JS，但 XHTML 脚本以 `<![CDATA[ ... ]]>` 包裹（CSS21 .xht 套件大量使用），html5ever 按 HTML 模式保留 CDATA 标记 → 传 V8 致 **`SyntaxError: Unexpected token '<'`** → 整个脚本编译失败（函数未定义 → onload 抛 ReferenceError）。实证 insert-block-in-inlines-beginning-001.xht 修复前编译错误，修复后 JS 正确执行。频率：全 wpt-data **3196 个 .xht/.xhtml 含 CDATA**（裸 `<![CDATA[` 3152 主导 + `//<![CDATA[` JS 注释隐藏 46）。

**实施**（pipeline.rs）：新增 `strip_script_cdata`（兼容裸 `<![CDATA[...]]>` 与 `//<![CDATA[...//]]>` 两种写法；与既有 `strip_cdata` 专 `<style>` CSS 独立，CSS 注释 `/* */` 不会含 `//`）；`extract_page_scripts` 内 `text_content` 经 `strip_script_cdata(raw.trim()).trim()` 后再当 JS。**A/B**：css/CSS2 oracle **3012→3013（+1）**；box-display 32/120 持平（insert-* 现 JS 执行但 R109 §9.2.1.1 动态匿名块生成仍发散，insert-block-beginning 22.19→20.51% 未过 1%，同 R917「harness 接线 +1 / 次级渲染 bug 阻塞」模式）；新单测 `extract_page_scripts_strips_xhtml_cdata_wrapper` PASS。门禁全绿（make test 0 FAILED / product-smoke welcome 16.11% / clippy clean / fmt clean）。详见 [`evidence/r927-extract-page-scripts-cdata-strip-2026-07-01.txt`](./evidence/r927-extract-page-scripts-cdata-strip-2026-07-01.txt)。

**意义**：reftest harness JS 执行从 R917-R920（DOM-mutation / 现代 JS 模式 / named access / createElementNS）进一步到「能跑 XHTML CDATA 包裹脚本」。DC-1 真实正确性提升——3196 个 XHTML 测试的 setup 脚本不再因 CDATA 静默编译失败，CSS21 套件动态簇（insert/delete/dynamic-*）现能正确执行，yield 阻塞转移到 R109 §9.2.1.1 动态匿名块生成等结构性渲染层。**harness JS vein 累计（R917→R927）跨 5 目录**：CSS2 +3（3009→3012 R917-R920 +1 R927 = 3013）/ css-flexbox +1（R926 baseline）/ css-multicol +4（R919）/ css-tables -1（R919 诚实暴露 + R922 +4 修复）。

**▶ 下会话**：① R927 CDATA 在 css-tables/css-position/css-multicol 等含 .xht CDATA 脚本目录 A/B（潜在更多 yield）；② R109 §9.2.1.1 动态匿名块生成（insert-* 簇 JS 现执行后暴露的渲染缺口）；③ 其他 harness 低频缺口（cloneNode 7 / getComputedStyle 41 须 layout 查询）。

### R928 insert-block-in-inlines 簇诊断 = §9.2.1.1 inline-元素匿名块盒结构性（零 yield·narrowed 归因·零源码·纯调查）

承 R927「下会话 R109 §9.2.1.1 动态匿名块生成」。post-R927 CDATA 修复后 insert-block-in-inlines-beginning-001.xht 的 JS 现执行（createElement+insertBefore+className 6 mutation 全正确，post-mutation HTML 结构正确：`<div class="container"><div class="inserted">...</div><span...>Several</span>...sentence.</div>`）。但 box-display oracle 仍 32/120（+0），insert-* 簇仍 20-40%。逐项像素 forensics 定位真渲染缺口。

**关键 narrowing（bare-text vs span 双盲隔离）**：最小复现 `<div class="c"><div class="b">B</div>TEXT</div>`（block + 裸文本）vs `<div class="c"><div class="b">B</div><span>SPAN</span></div>`（block + span）：
- **block + 裸文本**：正确——"B" 块在 y=8-27，"TEXT" 紧随 y=28-47，容器高含两者。匿名块盒包裹裸文本工作。
- **block + span**：错——"B" 块 y=8-27，span 文本错位到 y=39-54（应 y=28），且**溢出容器**（容器止 y=47，span 文本延至 54）。

**机制（collect_inline_items + LayoutBox 双重表示）**：`collect_inline_items`（inline/mod.rs:467）对 block-level 子（div.b）推 `InlineItem::Br` 并跳过（taffy 独立布局）；对其他 inline 元素（span）用 `doc.text_content(child_id)`（line 762）把 span 文本收入 IFC。但 **span 同时是 element → converter 生成独立 taffy LayoutBox 子**（R255 inline→block 谱系），而裸文本节点非 LayoutBox。后果：span 既被 IFC 收集（文本），又是 taffy 子盒（box）→ 双重表示；taffy 把 span 子盒定位到块后但**未测其 inline 内容高**（IFC 测高不回填 taffy，R109/Phase A 同源问题）→ span 盒高错、文本溢出容器。r109.rs 仅处理**逆向** case（inline-split-by-block 的匿名块片段收缩），本 case（block 容器 [block 子][inline 元素子]）是 §9.2.1.1 场景#1 匿名块盒生成，未实现。

**裁决**：insert-block-in-inlines 簇（box-display worst 15 中 ~12 案 insert/delete-*-in-*）= §9.2.1.1 inline-元素匿名块盒 + IFC-taffy 高度回填结构性缺口（Phase A 同源），非单 session clean fix。R924「全结构性」再确认。下会话勿以单点改 insert-* 簇（须 Phase A inline 元素盒模型统一）。本轮 narrowed 归因：bare-text 匿名块盒工作、inline-元素不工作 = converter inline→LayoutBox 映射 + IFC 测高不回填 taffy 双因。

**▶ 下会话**：① 转非 insert-* lever（insert-* 簇结构性确认）；② R927 CDATA 在更多 .xht 目录 A/B；③ getComputedStyle shim（41 文件，须 layout 查询回填 JS，复杂但潜在 yield 高）；④ 其他 harness 低频缺口。post-R927/R928 真基线：CSS2 3013（48.3%）/ flexbox 280（56.3%）/ grid 18（36.7%）/ multicol 115（25.4%）/ tables 65（56.5%）/ position 51（52.6%）/ box-display 32（26.7%，insert-* 簇结构性阻塞）。

### R929 insert-block-in-inlines 精确根因 = paint IFC 发散（layout 正确·匿名块盒未存 inline_layout·Phase A 双路径死锁·零源码·纯调查）

承 R928「下会话评估 §9.2.1.1 inline-元素匿名块盒 narrow-slice 修复」。用临时 LAYOUT_DUMP 探针 + 有界 gate 放宽实验，**纠正 R928「converter/IFC 测高回填」假设，真根因在 paint 侧**：

**① LAYOUT_DUMP 实证 layout 树正确**：block+span 最小复现 `<div class="c"><div class="b">B</div><span>SPAN</span></div>` 的 LayoutBox 树 = div.c(h=40) > 匿名块盒 node9(h=40) > [div.b(y=0,h=20,border_L5)][span(y=20,h=20,blk=false)]。与 bare-text（div.c > 匿名块盒 > [div.b][匿名块盒-for-TEXT(y=20,h=20,w=80)]）结构等价，**layout 几何全对**（span 在 y=20，容器 h=40 含两者）。推翻 R928「converter inline→LayoutBox 双重表示 + taffy 未测高」假设——taffy 测高正确。

**② PAINT 发散**：同样 layout 下，bare-text 的 TEXT 渲染在 y=28-47（=容器 8+20，正确），span 的 "SPAN" 渲染在 **y=39-54**（应 28-47，差 11px = strut 偏移）且溢出容器（容器止 47，span 延至 54）。paint IFC（Path B）对匿名块盒内的 inline 元素加了额外 strut 偏移。

**③ 真根因 = 匿名块盒未存 inline_layout**：paint use_stored（painter/text.rs:838 `box_node.inline_layout.is_some() && width_matches`）只在 box 有存储时用 layout 结果；否则重跑 IFC（Path B）发散。`compute_final_inline_layouts`（inline_finalization.rs:384）的存储 gate（line 488-518）**显式排除混合 inline+block 内容**（注释：「存储路径与现 paint 重跑在匿名块/碎片化上分歧致回归」，为 R109 inline-box-001 + span+h4 multicol-block-no-clip-001 回归而加）。**更关键**：即便放宽 gate（R929 实验：移除 `!has_block_elem`，保留 `!inline_children_have_elem`），最小复现 span 仍 y=39 字节不变 → **匿名块盒（node9，§9.2.1.1 生成、无 DOM node）根本不被 compute_final_inline_layouts 递归处理**（doc.child_nodes(anonymous_id) 无对应），gate 放宽触达不到它。实验已回退（零效果 + 他处回归风险）。

**裁决**：insert-block-in-inlines 簇 = **Phase A「Layout/Paint IFC 双路径」死锁**（rendering-compat.md P1-严重缺口）：匿名块盒（§9.2.1.1 生成）不存 inline_layout → paint Path B 重跑 IFC 对 inline 元素加 strut 偏移 → 文本错位+溢出。修复须 Phase A：(a) 让 compute_final 处理匿名块盒并存其 inline_layout，或 (b) 让 paint Path B 与 layout IFC 对齐。两者均结构性多 session（R109/multicol 回归风险须同解）。R928「converter」归因被 R929 纠正为「paint IFC + 匿名块盒未存储」。下会话勿再以「converter inline→LayoutBox」或「单点改 gate」为 insert-* lever（实证零效果）。

**▶ 下会话**：① insert-* 簇须 Phase A（匿名块盒 inline_layout 存储 + R109/multicol 回归同解，多 session）；② 转其他 lever——R927 CDATA 更多目录 A/B / 其他 harness 缺口 / 或重启一轮 fresh worst-scan（post-R927 CDATA 解锁的 XHTML 动态测试可能暴露新 clean 渲染缺口，非 insert-* 结构性的）。

### R930 CSS2 100%-diff fresh-scan = 全非-clean 或 OOS（plateau 再确认·零源码·纯调查）

承 R929「下会话重启 fresh worst-scan」。扫 css/CSS2 oracle top worst（90-100% diff）找 R878/R879 canvas 传播类 clean-fix：**全非 clean 或 OOS**：
- **font-family-invalid-characters-003.xht（100%）**= CSS 解析器 malformed `{}` 错误恢复（`font-family: test{foo}`、`body{bg:red;};` 等）。ZW 渲 477722 red（应 0，body bg:red 错误应用/未恢复）。修须精确 CSS2 §4.2 错误恢复 + 多处解析器改动（brace 配对恢复），单测试但复杂结构性，非单 session。
- **pagination/float-page-break-inside-avoid-*-print.html（99%）**= `@media print` 分页，DC-12 标可选低优先 OOS。
- **inline-svg-100-percent-in-body.html（97.56%）**= inline SVG，goal line 118 明确排除。
- **generated-content/before-after-table-parts-001.xht（93.36%）**= ::before/::after 生成表格部件，R554 谱系（generated-content + pseudo + table 复杂结构性）。

**裁决**：R930 fresh-scan 再确认 R924 plateau——CSS2 100%-diff 案全非 clean 单 session fix（parser 错误恢复结构性 / print OOS / inline-SVG OOS / generated-content+table 复杂）。clean rendering lever 经 R491-R930 多轮全角度（box-model R869-R882 / canvas 传播 R878-R879 / abspos 根 R871-R872 / harness JS vein R917-R927 / baseline §4.4 R926 / CDATA R927）确认**穷尽**。forward motion 100% 在 Phase A 结构性墙：① Layout/Paint IFC 双路径（R929 死锁，匿名块盒 inline_layout 存储 + R109/multicol 同解）；② inline-grid intrinsic width + IFC line-box（R926）；③ font-engine 投资（welcome/morning 16-17% 残余，R915 comprehensively confirmed）；④ R109 §9.2.1.1（R928/R929 paint 侧）；⑤ multicol Phase 2（R925 闭环）。

**▶ 下会话**：plateau 铁定，clean lever 穷尽。下会话须选一 Phase A 结构性墙深入（最高价值 = R929 死锁：让 compute_final 处理匿名块盒存 inline_layout，A/B 守 R109 inline-box-001 + multicol-block-no-clip-001 回归；或 font-engine 方向决策须用户裁决）。或继续 fresh-san 其他目录（css-tables/css-position/floats）100%-diff 案（R924 已扫，边际低）。

### R931 R929 死锁深查 = span 双重表示 + IFC Br 行高（gate 放宽存了仍 y=39·Phase A 双路径+二元性·零源码·纯调查）

承 R930「下会话深入 R929 死锁」。用 ANON_DBG + PAINT_DBG 探针逐层确证 block+span 最小复现 `<div class="c"><div class="b">B</div><span>SPAN</span></div>` 的 paint 链：

**① gate 放宽有效存储**：node 9v1（§9.2.1.1 匿名 wrapper，disp=Block, is_block=true, dom_kids=[div,span]）原被 `!has_block_elem` gate 排除（has_text_children=false）。移除 `!has_block_elem`（保留 `!inline_children_have_elem`）→ has_text_children=true → compute_final 存其 inline_layout（PAINT_DBG 实测 node 9v1 `has_inline_layout=true`）。**但 span 文本仍 y=39 字节不变**（应 28）。

**② span 双重表示（关键）**：PAINT_DBG trace 显示 `paint_text` 对**每个** box 调用（mod.rs:507），含 span（node 12v1, is_block=false, abs_y=28, has_inline_layout=false）。即 span **既被 node 9v1 的 IFC 收集**（collect_inline_items:762 text_content）**又作为独立 LayoutBox 自带 paint_text**（Path B 重跑 IFC）。bare-text 无此问题（文本节点非 LayoutBox，单一表示）。这是 rendering-compat.md「inline formatting 所有权分裂」P1-严重缺口的产品可见症状。

**③ y=39 来源 = IFC Br 行高**：node 9v1 的 IFC items=[Br（div.b 块子强制换行）][span text]。Br 处理（mod.rs:1216）用 `default_line_height` 推空行高，span 落 line 1。default_line_height 对 20px/1 Ahem 应 20，实测 span 在 y=39（=content_y 8 + line.y ~31）→ Br 空行高 ~31 非 20，11px 偏低 = strut/leading 双计或 default_line_height 取错（node 9v1 匿名盒继承的 line-height 解析）。

**裁决**：R929 死锁比设想更深——(a) gate 放宽存了 inline_layout 但 paint 仍 y=39；(b) span 双重表示（node 9v1 IFC + node 12v1 独立 paint）致双路径；(c) Br 空行高 ~31px（应 20）11px 偏低。单点改（gate / paint skip / Br 行高）均不足，须 Phase A IFC 统一：(1) 消除 inline 元素双重表示（display:Inline box 的文本只经父 IFC，不自带 paint_text）；(2) 匿名块盒继承正确 line-height；(3) Br 行高用容器 line-height 非 default。三者协同，多 session，A/B 守 R109 inline-box-001 + multicol-block-no-clip-001 + 大量 inline 文本回归（display:Inline 极常见，paint skip 高 blast radius）。

**▶ 下会话**：① Phase A IFC 统一 narrow slice（先攻 Br 行高：node 9v1 匿名盒 default_line_height 取容器真实 line-height 而非 1.2 默认，单点低风险，A/B 守 box-display/CSS2 oracle）；② display:Inline paint skip（消除双重表示，高 blast radius 须全量 A/B）；③ 或转 font-engine / 其他结构性墙。R929/R931 确认 insert-* 簇 = Phase A IFC 双路径 + 二元性，勿以单点重试。

### R932 insert-block-in-inlines 真根因 = inline 自涂 line-height（纠正 R931 双重表示·收窄到可治 paint 修复·零源码·纯调查）

承 R931「下会话攻 Br 行高 / display:Inline paint skip」。用 SKIP_INLINE_PAINT=1 决定性实验 + line-height 变量验证，**纠正 R931「span 双重表示」假设，真根因收窄到 inline 自涂 line-height**：

**① SKIP_INLINE_PAINT 决定性实验**：对 display:Inline box 跳过 paint_text 后，span 文本**完全消失**（只剩 div.b "B"）。即 span 文本**仅由 node 12v1 自身 paint_text 渲染**（display:Inline box 自涂其文本），**node 9v1 父 IFC 不渲染 span**（架构：inline 元素子由自身 box 涂，父 IFC 只涂文本节点）。纠正 R931「双重表示」——实为**单一表示**（node 12v1 自涂），y=39 来自 node 12v1 而非 node 9v1 IFC。display:Inline paint skip 不是修复路径（会让 span 文本消失）。

**② line-height 变量验证**：font:20px/1 → span glyph top y=39；font:20px/2 → span glyph top y=59（差 20 = line-height 差）。**span y 依赖 line-height**（非 font-metric 固定）→ bug 在 inline 自涂的 line-height 处理，非 R931 Br 行高。

**③ 真机制（R632 谱系）**：paint_text Path B 用 `line_height_overrides`（key = 文本节点父元素）读 box_node.text_node_line_heights。但 node 12v1（display:Inline）被 compute_final **跳过**（inline_finalization.rs:475 `if !root.is_block_level { return }`）→ 其文本节点 line-height **未存** → Path B 回退 font-metric 默认（ascent+descent ~31px for 20px font，非 CSS line-height 20）→ span 文本垂直位置错（glyph top = content_y + 错误 line-height 偏移）。bare-text 不受影响（文本节点父是 block 容器，line-height 已存）。R632 line-height override 覆盖了 stored path + block 容器，**漏了 inline 元素自涂**。

**裁决（收窄到可治）**：insert-block-in-inlines 簇真根因 = **inline 元素自涂 Path B 拿不到正确 line-height**（compute_final 跳过 inline box 致文本节点 line-height 未存）。**非 R929/R931 多层 Phase A 死锁**，而是 R632 line-height plumbing 漏覆盖 inline 自涂的**聚焦 paint 修复**。修复路径（下会话）：paint_text Path B 对 box 自身文本（box_node.node_id 对应元素的直接文本）用 `style.line_height`（CSS 计算值，paint 已有 style 参数）作 default，而非依赖未存的 line_height_overrides。A/B 守 box-display/CSS2/normal-flow/linebox oracle 零回归（line-height 影响广，须严守）。

**▶ 下会话**：R933 实施 inline 自涂 line-height 修复——paint_text Path B 用 style.line_height 作 box 自身直接文本的 line-height default（覆盖未存的 line_height_overrides 回退），A/B 守 box-display（insert-* 簇）+ CSS2 + normal-flow + linebox oracle 零回归。预期 insert-block-in-inlines 簇（box-display worst 12 案）改善。若 net-negative 即回退。

### R933 R932 line-height 诊断 storage-probe 证伪 = inline_element_metrics 已存正确值·paint IFC 走 path1 不读 parent_* maps·insert-* 错位 = Phase A baseline/strut 非 line-height（零源码·纯调查·决定性 A/B+probe）

承 R932「下会话实施 inline 自涂 line-height 修复」。**实施 R932 处方 + 两轮 storage-probe + 修正 fix，三角度全证 R932 诊断错误，insert-* 簇 line-height 值本就正确，错位是 Phase A baseline/strut 非 line-height plumbing。**

**① R932 处方实施 = 全 inert（A/B byte-identical）**：paint_text Path B 按 R932 处方补 `parent_font_sizes/parent_line_heights[box_node.node_id] = resolve_font_metrics(style)`（仅 entry 缺失时填）。A/B box-display oracle：pass-count **32/120 两边同**；R932 点名的 3 案 `insert-block-in-inlines-{beginning,end,middle}-001` **byte-identical**（20.51% / 20.43% / 16.98% 零变化）；其余 insert-inline-in-blocks-n-inlines-* 仅 ±0.3-0.6% 混合噪声 + delete-inline-in-blocks-middle-002 +0.43pp 微回归。**裁决：neutral-on-target，按 R932「net-negative 即回退」回退**。

**② storage-probe 定位 inert 真因（决定性）**：R933_PROBE 探针 dump Path B 的 map 状态——对 insert-block-in-inlines-beginning-001：`box_node = NodeId(30v1) display=Block`（§9.2.1.1 匿名包装盒，非 inline 元素），`had_fs=true had_lh=true`（**30v1 键已存在**，or_insert 无效），`parent_font_sizes={30v1:16.0}`（仅包装盒 1 项），而 **`inline_element_metrics={31v1:(16,19.2), 32v1:(16,19.2), 34v1:(16,19.2)}`**——3 个 inline 元素的 (fs,lh) **store_font_sizes_from_ifc（inline_finalization.rs:252）早已存好**。即 R932「inline 元素 line-height 未存」前提**错误**：text_node_line_heights 确缺 inline 元素项（键=文本节点 id），但 inline_element_metrics（键=元素 id）有，且 paint IFC 用后者。R932 填 parent_*[30v1=包装盒] 是已存在键 → 必然 no-op。

**③ 修正 fix（合并 inline_element_metrics 进 parent_* maps）= 仍全 inert**：按 ② 真因，把 `inline_element_metrics` 的（元素 id → (fs,lh)）合并进 parent_font_sizes/parent_line_heights（or_insert，不覆盖 block 容器既有项），使「文本节点路径」查 inline 元素 id 时命中。A/B box-display oracle **120 案全 byte-identical**（diff 退出码 0）。**决定性结论**：paint IFC 对 inline 元素文本走 **path 1**（mod.rs:762 `doc.text_content(child_id)` + mod.rs:767 `inline_element_metrics.get(&child_id)` fallback，已有正确 lh=19.2），**不走 path 2**（parent_font_sizes/parent_line_heights 文本节点路径）。故任何 parent_* maps 填充对 inline 元素文本零效果。

**裁决（纠正 R932，重定向）**：insert-block-in-inlines 簇错位 **非 line-height/font-size plumbing 问题**——line-height 值（19.2）正确且经 inline_element_metrics 到达 paint IFC。真因 = **下游垂直定位（baseline/strut/half-leading）在非存储 render 路径的发散**（顶部 R882 已标「non-stored render_fragment! baseline_offset，R834/R836/R849/R875 四次单点 net-negative 先例，须 strut 0.9→0.928 + paint v_offset ink-height + 三方同改」）。**R932「收窄到可治 paint 修复」判断被推翻**——line-height plumbing（R632/R932 谱系）对 insert-* 簇是死路，勿再以「补 line-height default / 填 parent_* maps」单点重试。**价值**：本轮用 storage-probe + 两版 fix 的决定性 A/B 关闭 line-height-plumbing vein，把 insert-* 簇明确归入 Phase A baseline/strut 墙（与 R834/R836/R849/R875 同谱），下会话勿重复 line-height 角度。

**▶ 下会话**：① insert-* 簇 baseline/strut 角度——probe paint Path B 对 inline 元素片段的 baseline_y / strut 计算（R841 ahem_uses_embox_position 同域，非 Ahem 通用 strut 0.9 vs 0.928 + v_offset ink-height），但 R834/R836/R849/R875 四次单点 net-negative 先例警示须三方同改（strut + paint v_offset + 验证），多会话；② 或转其他 Phase A 结构墙（font-engine 投资 / multicol Phase 2 / R109 §9.2.1.1）；③ 勿以 line-height plumbing 重试 insert-*（R933 已证 inert）。零源码（纯 A/B + probe + 读码 inline_finalization.rs:252 / inline/mod.rs:762-774）。

### R934 insert-* font-metric 三角度穷尽证伪 = container span 计算值/存储/paint 全正确 (20,20)·R933「19.2」误识别 default-font 元素·diff 100% = R109 结构性 (零源码·纯 probe)

承 R933「下会话 baseline/strut 角度 probe」。R934_PROBE 在 IFC child-metric 解析点（inline/mod.rs:765-774）dump 每个 inline 子元素的 `styles.get(child_id)` 真实计算值，**三角度（R932 line-height 行为 / R933 存储 / R934 计算值）全证 insert-* font-metric 正确，diff 100% 是 R109 结构性**。

**① R934_PROBE 决定性数据（insert-block-in-inlines-beginning-001）**：
- `NodeId(39v1)/42v1/45v1`（.container 内 3 个 `<span>`，CSS `font:20px/1 Ahem`）：`styles_empty=false style_some=true style_fs=Px(20) style_lh=Number(1.0)` → 解析 **(20, 20)** ✓ 完全正确（layout IFC 真实 styles 路径）。
- `NodeId(4v1)/32v1`（`<p>` 内 `<strong>` 等 default-font 元素）：`style_fs=Px(16) style_lh=Normal` → (16, 19.2)，**对这些元素本身正确**（无 font 指定 → 默认 16/Normal）。
- paint IFC（`styles_empty=true`）对 32v1 走 inline_element_metrics fallback = (16, 19.2)，与该元素 layout 值一致。

**② 纠正 R933 误识别**：R933 PROBE 的 `inline_element_metrics={31v1:(16,19.2), 32v1:(16,19.2), 34v1:(16,19.2)}` 被解读为「container 的 inline 元素 line-height=19.2」。**R934 证 31v1/32v1/34v1 是 default-font 元素（`<strong>`/`<p>` 系，fs=16/Normal），非 container 的 span**——container span 是 39v1/42v1/45v1（fs=20/lh=1.0）。R933「line-height 值 19.2 正确」结论是对**错误元素**的正确值（default 元素 19.2 确实对），与 container span（应 20）无关。R933 整体方向（line-height plumbing 死路）仍对，但「19.2 正确」论据无效。

**③ 裁决（font-metric vein 三角度闭合）**：insert-block-in-inlines 簇 font-metric **完全正确**——container span 在 layout IFC（styles.get 真实值 20/1.0）+ paint IFC（inline_element_metrics 存储 20/20）+ R932 行为变量（y 随 lh 变）三角度均 (20,20)。**20% diff 0% 来自 font-metric**，100% 来自 **R109 §9.2.1.1 结构性**（匿名块盒生成改变 taffy 折叠树 + inserted block 的 margin:1em 0 collapse + 黄边框 + paint 对新结构空块/bar 渲染协调，R764 spec-rfc territory）。font-metric / line-height / 字体 plumbing 对 insert-* 三角度穷尽证伪，**勿再以任何 font-metric 角度重试**（R932/R933/R934 三轮）。

**▶ 下会话**：insert-* 簇唯一剩余路径 = R109 §9.2.1.1 匿名块 + collapse/paint 协调（R764 spec-rfc 设计，多会话，R743/R744 回归风险须同解）。或转其他 Phase A 结构墙（font-engine 投资 welcome/morning 16-17% / multicol Phase 2 / R304 taffy）。**勿以 font-metric / line-height / 字体 plumbing 重试 insert-***（R932 line-height + R933 storage + R934 computed-style 三轮证伪）。零源码（纯 R934_PROBE + 读码 inline/mod.rs:765-774）。

### R935 insert-* 像素 forensics = ZW 容器结构性渲染坏（高度 174 vs chr 233·fuchsia bg 1/4·margin/anon 区 white-非-fuchsia·border 错位）·证实 R109 结构性且产出可测症状 (零源码·纯 probe)

承 R934「下会话 R109 spec-rfc」。R932-R934 全是 font-metric 角度（三角度证伪），**R935 转像素 forensics 直接对比 ZW vs chromium oracle 渲染**，产出 R932-R934 未见的硬证据：insert-* 是**结构性渲染坏**（layout 错乱 + bg 漏涂），非任何 font-metric 微差。

**① 像素对比（insert-block-in-inlines-beginning-001，product-smoke 渲染 + PIL 区域分析 + scanline dump）**：
- **fuchsia 容器 bg 面积**：ZW **17920 px**（bb x=[168,55,771,154]）vs CHR **74240 px**（bb x=[28,54,771,233]）—— ZW 仅 **1/4**，且左边界 x=168（应 x=28，左移 140px 缺 bg）。
- **总内容高度**：ZW 内容止 y=**174** vs CHR y=**233**——ZW 矮 59px（CHR 双容器各 ~90px，ZW 第二容器基本缺失/截断）。
- **scanline 决定性差异**：CHR y=74-90（inserted block 的 `margin:1em 0` 区）= **整行 fuchsia**（`FFFFFFFF...`）；ZW y=78-94 同区 = **黑字 on WHITE**（`BBBB...BBB........`）—— **ZW 在 margin/anon-block 区不涂容器 fuchsia bg**（露白），CHR 全 fuchsia。
- **yellow border 错位**：CHR 每容器一个 inserted-block yellow 区（y=54-70 / y=154-170）；ZW 出现 2+ yellow 区（y=58-74 / y=98-114）且 inline-run 行也带 yellow——**border/结构错乱**。

**② 裁决（硬像素证据收窄 R109 症状）**：insert-* 不是 font-metric 微差（R932-R934 已证 font 正确），是**匿名块盒（§9.2.1.1）结构性渲染坏**：(a) 容器 fuchsia bg 不涂满（margin/匿名块区露白）；(b) 容器高度算短（~174 vs 233，疑匿名块 inline 内容高度未贡献进容器 taffy 测高——R929「匿名块盒不被 compute_final 处理」的几何后果）；(c) inserted block border 错位。**这是首个可测几何症状清单**（非 R932-R934 的「font 都对但 diff 在」黑盒），R109 spec 须覆盖：匿名块盒内容高度→容器测高 + 容器 bg 涂满匿名块/margin 区 + border 正确归属。

**③ 方法论价值**：font-metric probe（R932-R934）证伪了「字号/行高」假设但留下「diff 在但不知在哪」；**像素 forensics（product-smoke + PIL bbox + scanline）直接定位 diff 几何**，是 insert-* 类「同源都看不出」案的决定性工具。后续 insert-*/R109 调查应优先用此法（product-smoke --out + PIL），非再从 font-metric 入手。

**▶ 下会话**：R936 启动 R764 R109 §9.2.1.1 spec-rfc（lei-spec-rfc skill），设计须覆盖 R935 三症状（匿名块内容高度→容器测高 / 容器 bg 涂满 margin+匿名块区 / border 归属），逐案 A/B 守 R743/R744 回归（margin-collapse-101）。或先用 LAYOUT_DUMP 探匿名块盒是否进容器测高（验证症状 b 根因）。零源码（纯 product-smoke 渲染 + PIL 几何分析）。

### R936 R109 toggle A/B = ON 净 +5（box-display 32 vs OFF 27）·insert-* R109-无关（ON/OFF 均 ~20%）·R109 是正确默认·insert-* 真结构性墙 (零源码·纯 A/B)

承 R935「下会话验证 anon 块高度根因」。读 tree.rs:21 发现 `r109_wired()` 默认 **TRUE**（仅 R109_WIRE=0 关；engine.rs:53「仅=1」注释 stale）。即 R109 匿名块生成**默认开启**——R935 的 mangled 渲染就是 R109 ON 下产出。做决定性 A/B：R109_WIRE=0 vs 默认（ON）。

**① product-smoke 像素 forensics（insert-block-in-inlines-beginning-001，R109 OFF vs ON vs CHR）**：OFF 的 fuchsia = **59680 px** [x=28,55,771,194]（**接近 CHR 74240 [x=28]**）；ON = 17920 [x=168]（坏）。OFF 内容高 214 vs ON 174 vs CHR 233。**OFF 几何上明显更近 CHR**。

**② 但 oracle z_vs_chr 几乎不变**：beginning-001 OFF=**20.48%** vs ON=20.51%（pixel diff 同量级，差异仅换了位置）；end-001 OFF=17.30%（更好）vs ON=20.43%；middle-001 OFF=22.92%（更差）vs ON=16.98%。**insert-* 簇 R109 ON/OFF 混合、净中性**——R109 开关解不了 insert-*。

**③ box-display 总分：OFF=27/120（22%）vs ON=32/120（27%）**——**R109 ON 净 +5 case**（is_inline_r109 split 路径帮了 5 个非-insert 案）。**裁决：R109 ON 是正确默认，勿关**（推翻「R109 toggle 可能帮 insert-*」假设）。insert-* 是 **R109-无关结构性墙**（ON/OFF 都 ~20%）：无论是否生成匿名块，都因 anon 块高度未回填 / 容器 bg 涂不满 / border 归属而坏，须完整 R109 spec（anon 生成+高度回填+bg+border 协调），非开关。

**④ 战略**：R932-R936 五轮穷尽 insert-*（font-metric 三角度证伪 + 像素症状 + R109 toggle）—— 簇是 R109/Phase A 结构墙，**单 session 不可解，勿再攻**。R109 ON 净 +5 证 is_inline_r109 路径有价值，R109 spec 须保并增强（含 block-mixed 路径修对）。forward motion = R109 spec（多会话）或转其他结构性墙。

**▶ 下会话**：① 启动 R764 R109 §9.2.1.1 spec-rfc（lei-spec-rfc），需求：anon 块生成+**高度回填进容器 taffy 测高**（R935 症状 b 真因）+ 容器 bg 涂满 anon/margin 区 + border 归属；保 is_inline_r109（净 +5）同时修 is_block_mixed；A/B 守 R743/R744。② 或转其他 Phase A 结构墙（font-engine welcome/morning / multicol Phase 2）。③ **勿再单点攻 insert-***（R932-R936 五轮穷尽，R109-无关结构性）。零源码（纯 R109_WIRE=0/1 A/B + product-smoke PIL）。

### R937 R109 §9.2.1.1 spec-rfc LANDED = 高度回填+bg 涂满+border 归属 3 FR·P2 后处理 pass 选定·首步验证 A1·待 code 实施 (零源码·纯 spec doc)

承 R936「下会话启动 R109 spec-rfc」。用 lei-spec-rfc skill 产出完整 spec+RFC → [`r109-anonymous-block-spec.md`](./r109-anonymous-block-spec.md)（386 行，11 节）。**收敛 R764 读码 + R929-R936 五轮调查为可实施多会话计划。**

**spec 要点**：
- **3 FR**：FR-001 匿名块盒 IFC 内容高度→容器测高回填（case a+b，R935 症状 b）/ FR-002 容器 bg 涂满匿名块盒+margin 区（症状 a 露白）/ FR-003 拆分 inline border 归属（case a，复用 r109.rs shrink 基建）/ FR-004 守卫（保 R109 ON net +5 + 零回归 R743/R744）。
- **根因假设 A1（待验证，首步）**：匿名块盒用 taffy new_leaf_with_context（tree.rs:600）创建，taffy 不能测 inline 内容 → 测高 0；compute_final_inline_layouts 的 gate（doc.get(node_id) :466 + !is_block_level :475 + 混合内容排除 :495）跳过匿名块盒 → IFC 内容高度（store_font_sizes_from_ifc:262 已存 frag.height）**不回填** → 容器测高排除了 inline run 高度 → 容器矮 + bg 露白。
- **选定方案 P2（后处理 pass）**：engine.rs 新增 backfill_anon_block_heights（复用 R695/R699 两趟基建），env 包裹可回退；**不选 P1（放宽 compute_final gate）**因 gate 当初为 R743/R744 回归而加，高风险。
- **实施顺序**：step1 probe 验证 A1 → Batch1 FR-001 高度回填（case b 即 insert-* only，隔离风险）→ FR-002 bg → FR-003 case a border → FR-004 全量回归。
- **Spec Lint**：Pass 12 / Warning 2 / Fail 0（Warning = 假设 A1/A2/A3 待首步验证 + FR-002/003 异常场景偏少，非阻塞）。

**裁决**：R932-R936 调查阶段结束（insert-* 真因定位 + 5 轮穷尽），**进入实施准备**。spec 是 R764 明确要求的产物，把模糊「R109 结构墙」收敛为 3 个可测 FR + 假设验证首步 + 隔离的 P2 入口。**下会话起按 spec 实施**（Batch 1 = FR-001 case b 高度回填，先 probe 验证 A1）。

**▶ 下会话**：按 [`r109-anonymous-block-spec.md`](./r109-anonymous-block-spec.md) §7 实施顺序：① step1 probe 验证假设 A1（匿名块盒 taffy 测高是否 0 + compute_final 是否跳过，决定 P1/P2 入口）；② Batch 1 实现 FR-001 case b 高度回填（engine.rs 后处理 pass，env R109_BACKFILL 包裹），A/B box-display insert-* + product-smoke + PIL 断言；③ 若 net-negative 即 env 关闭回退。**勿跳过 step1 直接实现**（A1 未验证则入口选择无据）。

### R938 spec step1 A1 验证 = compute_final 处理匿名块盒但不回填 root.height·taffy 经 ctx_node 欠计多节点 run·FR-001 fix 位置精确定位 (零源码·纯读码验证)

承 R937「下会话 step1 probe 验证 A1」。读码验证 spec 假设 A1，**部分修正**（原假设「compute_final 跳过匿名块盒」错误，实际处理了，但**不回填高度**）。

**① 读码事实（inline_finalization.rs:384-727 compute_final_inline_layouts）**：
- 匿名块盒 node_id = 容器 dom_id（tree.rs:604 `taffy_to_dom.insert(anon_taffy, dom_id)`）→ `doc.get(node_id)`（:466 gate）**解析**，gate 通过。
- `is_block_level`（:475）= true（anon display:Block），gate 通过。
- `fragment_node_ids`（:627）正确配置 IFC 只收集片段 inline 内容。
- IFC 跑（:651）+ `store_font_sizes_from_ifc`（:660）存 frag 度量 + `root.inline_layout = Some(lines)`（:727）存行盒。
- **★ 但末尾从不 `root.height = ...`**（grep 证仅 :237 `root.content_height=tallest` 在另一函数 `remeasure`）—— **IFC 内容高度不回填到匿名块盒 root.height**。

**② taffy 测高（measure_text_content:736）**：匿名块盒用 `new_leaf_with_context(style, ctx_node)`（tree.rs:600），ctx_node = 片段**首个文本节点**（tree.rs `item_node_ids.find(Text)`）。taffy 经 measure_text_content(ctx_node) 测**单文本节点**——多文本节点/多行 run（如 insert-* 的 "Several inline elements are in this sentence." 含 spans + 裸文本，~760px > 743px 容器宽 → 换行 2 行）被**欠计**（测首节点 1 行 ~20px，实 run 2 行 ~40px）。

**③ 根因定位（比 spec A1 更精确）**：容器矮 + bg 露白 = ① compute_final 不回填 root.height + ② taffy 单文本节点欠计。**FR-001 fix 精确位置**：compute_final IFC 后回填 `root.height = Σ lines.height`（+ padding/border），并加容器高度后处理 pass（compute_final 在 taffy 测高后跑，改 root.height 不自动传播父盒，须重算容器 = Σ 子盒 height）。spec TBD-1 已标解除，TBD-2 入口选 P1.5（回填 root.height，非 P1 放宽 gate 也非纯 P2 后处理）。

**裁决**：A1 验证完成，spec 进入可实施状态。下会话 Batch 1 = compute_final 回填 root.height + 容器高度后处理 pass，A/B box-display insert-* + product-smoke。**已纠正 spec A1 措辞**（compute_final 处理匿名块盒，非跳过；根因是不回填 height + taffy 欠计）。零源码（纯读码 inline_finalization.rs:384-727 + tree.rs:600-620 + grep root.height）。

**▶ 下会话**：Batch 1 实施 FR-001：① compute_final（inline_finalization.rs:727 后）加 `root.height = lines.iter().map(|l| l.height).sum::<f32>()`（含 padding/border），env R109_BACKFILL 包裹；② engine.rs 加容器高度后处理 pass（重算含匿名块盒子盒的容器高度 = Σ 子盒 height + 自身 padding/border）；③ A/B box-display insert-* + product-smoke + PIL 断言高度/bg 面积；④ net-negative 即 env 关闭回退。守 R743/R744（margin-collapse-101 A/B）。

### R939 Batch 1 FR-001 高度回填 LANDED = backfill_r109_anon_block_heights·匿名块盒高度+容器 delta 传播·insert-* 首次真实改善·零回归（make test 12099/0 + product-smoke 16.11%）

承 R938「下会话 Batch 1」。实施 spec FR-001 匿名块盒高度回填，**首个 insert-* 真实代码进展**（R932-R938 调查/spec 后）。

**实现**（engine.rs，env R109_BACKFILL 默认开，=0 关）：
- 新增 `backfill_r109_anon_block_heights(box_node, styles) -> f32`（~60 行，engine.rs:1809），compute_final（:407）后调用（step 12.1）。
- ① 后序遍历：匿名块盒（fragment_node_ids.is_some）从 inline_layout 行盒回填 content_height = max(line.y+line.height)，仅增大不收缩（守 taffy 已正确的 case）。
- ② auto-height 祖先容器按直系匿名块子 delta 之和扩展自身高度（delta 累加非重算，保 taffy margin 折叠/兄弟定位）。
- 用 delta 传播（区别 R699 exclude_floats 的 max(child.y+h) 重算）：避免 margin 折叠重算风险。局限：假设增长 anon 是末位 in-flow 子（case b 常见），非末位 anon 仍扩展容器底但不移兄弟（spec TBD，独立子问题）。
- 4 个单测（engine/tests/r109_backfill_tests.rs）：回填 / 跳过显式 height 容器 / 不收缩 / 非 anon 不动。全过。

**验证（全门禁绿）**：
- `make test`：**12099 passed / 0 failed**（73 ignored = real-website + feature-gate）。
- `make product-smoke`：**16.11%**（≤20% 门禁，= baseline 零回归）。
- box-display oracle：**32/120**（= R936 baseline，零回归）；insert-block-in-inlines-beginning **20.51→18.83%（-1.68pp）**、end 20.43→19.74%（-0.69pp）、middle 不变。
- inline-box-001（R743/R744 风险）：**4.54% ON/OFF 不变**（零回归，backfill 不影响 case a）。
- margin-padding-clear：**277/682 ON/OFF 不变**（零回归）。
- product-smoke PIL：insert-* beginning fuchsia 面积 17920→29200（+11280），bg y-max 154→174（+20px 容器增高）。

**裁决**：Batch 1 LANDED，net-positive 零回归。insert-* 首次实质改善（beginning -1.68pp）但簇仍 ~18-20%（残余 = 容器宽度/x 起点 wrong + inline run 完整高度未全捕 + border 错位，spec FR-002/003 后续）。backfill 基建落地，为后续 Part（容器宽度修正 / border 归属）奠基。env R109_BACKFILL=0 可回退。

**▶ 下会话**：① 续查 insert-* 残余：probe 为何容器 fuchsia x 起点 168（应 28）——疑匿名块盒 width/content_x 或容器盒模型问题（独立于高度的 FR-002 子症状）；② inline run 完整高度未全捕（beginning 仅 +20px 应 +~40px）——probe compute_final 对该 anon 块的 IFC 是否只产 1 行（多行未触发）；③ 上述为 FR-001 完整收尾 + FR-002/bg 涂满的下一步。A/B 守已建门禁。

### R940 backfill Part 2 改 max-bottom 重算·container#1 高度修对·beginning -3.94pp（累计）·零硬门禁回归·残余 = 兄弟盒未重定位（post-taffy 限制）

承 R939「下会话续查 insert-* 残余」。R940_PROBE box-tree dump 定位 R939 delta 法漏掉的真因，refine Part 2 为 max-bottom 重算。

**① box-tree dump 决定性数据（insert-block-in-inlines-beginning-001）**：
- container#1: h=**20**（应 ~80），子盒 [inserted h=20 lines=1, ANON h=40 lines=2 **已正确**]。
- container#2: h=**80**（正确），子盒同结构。
- **真因**：anon 块盒**自身高度已正确（40，R939 Part 1 或 taffy 已给）**，但 container#1 的 taffy 测高**未把 anon 计入**（h=20 只含 inserted）。R939 delta 法（按 anon **增长量** 扩容器）对此无效——anon 没增长（delta=0）→ 容器不扩。

**② refine（engine.rs backfill_r109_anon_block_heights Part 2）**：从「按 anon delta 扩容器」改为「auto-height 容器含 anon 子或后代增长 → 重算 content_height = max in-flow 非 float 子盒 border-box 底（CSS §10.6.3，仅增大）」。max-bottom 覆盖两种：「anon 自身欠计」（Part 1 修后 max-bottom 反映）+「容器未把已正确 anon 计入」（直接 max 子盒底）。仅增大守卫避负 margin/margin 折叠误收缩（同 R699 exclude_floats 安全策略）。5 单测更新（Part 1 自身高 / Part 2 max-bottom 重算 / 显式 height 跳过 / 不收缩 / 非 anon 不动），全过。

**③ 验证（全硬门禁绿）**：
- `make test`：**12104 passed / 0 failed**。
- `make product-smoke`：**16.11%**（= baseline）。
- box-display oracle：**32/120**（零 pass 回归）；insert-block-in-inlines-beginning **18.83→16.57%（-2.26pp，累计 R936→16.57 = -3.94pp）**、end 19.74→20.43%（**+0.69pp 微回归**）、middle 不变。簇净 -1.57pp。
- inline-box-001（R743/R744）：**4.54% 不变**；margin-padding-clear：**277/682 不变**。
- PIL：beginning fuchsia 面积 29200→40480（接近 CHR 74240）。

**④ 已知局限（post-taffy，独立子问题）**：box-tree dump 证 container#1 高度修对（20→80），但 **container#2 仍位于 y=79（基于旧 container#1 高度的 taffy 定位）→ 与增高后的 container#1（底 y=119）重叠**。post-taffy 改高度不会重定位兄弟盒（须 taffy 重布局，多会话）。content_end_y 仍 174（应 233）即此。beginning 仍改善（重叠 + 更多 fuchsia 净更近 chr）；end 微回归（重叠对 end 布局略不利）。

**裁决**：max-bottom LANDED（net-positive 零硬门禁回归，beginning 累计 -3.94pp，更符合 CSS §10.6.3）。end +0.69pp 微回归可接受（簇净 -1.57pp）。兄弟盒重定位 = post-taffy 限制，须后续 taffy 两趟重布局（spec TBD，多会话）。

**▶ 下会话**：① 兄弟盒重定位（post-taffy 高度改后重定位后续兄弟）——须 mark_dirty + taffy 重布局（复用 R695/R699 两趟基建），或后处理手算兄弟 y 偏移；② 解决后 insert-* 簇应进一步改善（container#2 不再重叠）；③ 或转 FR-002（容器 bg 涂满）/ FR-003（border 归属）。A/B 守已建门禁（make test + product-smoke + box-display/margin-padding-clear/inline-box-001 oracle）。

### R941 backfill 兄弟盒 y 位移 LANDED = post-taffy 重定位后续兄弟·insert-block-in-inlines-beginning 16.57→6.43%（累计 -14.08pp）·零硬门禁回归·container#2 不再重叠

承 R940「下会话兄弟盒重定位」。在 backfill 子盒循环加「cumulative_shift」：当一个 in-flow 子盒增高 delta，其后续 in-flow 非 float 非 abspos 兄弟 .y 下移 delta。child.y 相对父内容盒、descendant.y 相对 child → 只移 child.y 即整子树下移（无须递归移后代）。区别 R940 只修高度不定位（重叠），R941 既修高又重定位。

**① 实现**（engine.rs backfill_r109_anon_block_heights 子盒循环）：子盒循环内，先应用 cumulative_shift（先前兄弟增高累计）到当前 in-flow 子盒 .y，再递归；递归返回的 g 累入 cumulative_shift 供后续兄弟。仅 in-flow 非 float 非 abspos 兄弟受位移（abspos 独立定位、float 自有流）。env R109_BACKFILL（同 R939/R940）。

**② 决定性验证（insert-block-in-inlines-beginning-001，product-smoke + PIL + oracle）**：
- content_end_y：R941 **234**（R940 174，CHR 233）—— **修对**（container#2 不再重叠，整页高度 ≈ chr）。
- fuchsia 面积：R941 **54640**（R940 40480，CHR 74240）。
- **oracle z_vs_chr：R941 6.43%**（R940 16.57%，R936 baseline 20.51%）—— **R939→R940→R941 累计 -14.08pp**。
- box-display insert-* 簇：beginning **6.43%**（drop out top-15 worst）、end **18.84%**（R940 20.43，R939 delta 19.74 → R941 更优，R940 微回归已修）、middle 16.98% 不变。

**③ 全门禁零回归**：
- `make test`：**12104 passed / 0 failed**。
- `make product-smoke`：**16.11%**（= baseline）。
- inline-box-001（R743/R744）：**4.54% 不变**；margin-padding-clear：**277/682 不变**。
- box-display oracle：**32/120**（零 pass 回归）。
- 7 单测（+2 R941 兄弟位移 / abspos 不移）全过。

**裁决**：R941 LANDED，insert-* 簇三连改善（R939 delta → R940 max-bottom → R941 兄弟位移），beginning **20.51→6.43%** 接近 1% pass 阈值。残余 ~6.43% = fuchsia x 起点 168（应 28，独立子症状）+ 行盒内部度量微差（FR-002 bg / 行盒度量，后续）。R109 §9.2.1.1 匿名块盒高度+重定位链路基本打通。

**▶ 下会话**：① insert-* 残余 6.43%——probe 容器 fuchsia x 起点 168（应 28）：疑匿名块盒/inserted block 的 x/content_x 或容器盒模型（product-smoke PIL 已证 y/高度对，x 起 wrong）；② 或扩 R941 backfill 到其他含 anon-block 用例（css2 normal-flow/positioning 等，扫 anon-block 受益簇）；③ 或转 FR-002 bg / FR-003 border。A/B 守已建门禁。

### R942 insert-* 残余 = 通用定位/font 噪声（非 insert-* bug）·R939-R941 广度实证 = normal-flow +4 / positioning +1 serendipitous win（零回归）

承 R941「下会话 probe 残余 + 扩 backfill 广度」。两件事：(a) insert-* 残余 6.43% 精确归因；(b) R939-R941 backfill 跨 dir 广度验证。

**① insert-* 残余精确像素 forensics（product-smoke PIL 逐行 firstF_x）**：
- CHR y=74-90（inserted block margin 区）= 整行 fuchsia 起 x=**28**（纯 fuchsia 无文本）；ZW y=78-94 同区 fuchsia 起 x=**208**，x=28-208 为黑字 → ZW inline run 比 CHR 高 ~4-15px（落进 margin 区）。
- box-tree（R940 dump）：body y=16，container#1 相对 y=23（绝对 39）；<p> 相对 y=0 h=19（16-35）；gap = 23-19 = **4px**（应 ~20px：container margin-top:1em@20px + <p> margin-bottom collapse = 20）。
- **裁决**：残余 = `<p>`→container margin gap 4px（应 20）致 container 偏高 ~16px + inline run 落进 margin 区。**非 insert-* bug**——是通用 margin-collapse / 默认字体度量（<p> 在 .container 外用默认字体）问题，影响所有含 `<p>+block` 页面。insert-* 的 R109 高度/重定位链已打通（beginning -14pp），残余属通用 font/margin 墙（welcome 16% / morning 同谱），独立多会话，勿再以 insert-* 角度攻。

**② R939-R941 backfill 跨 dir 广度验证（serendipitous win + 零回归）**：R939-R941 backfill（anon-block 高度回填 + max-bottom + 兄弟位移）不只帮 insert-*，对任何含 anon-block 子盒的容器都生效：
- **normal-flow：569/746**（baseline R695/R699 era **565/750**）→ **+4 case**（denominator -4 = oracle 微调，pass +4）。
- **positioning：238/520**（baseline R850 **237/520**）→ **+1 case**。
- box-display：32/120（已含 insert-* +2）。
- 三 dir 全改善或持平，**零回归**。证 backfill 是通用机制（非 insert-* 专用 hack），R109 §9.2.1.1 匿名块盒高度欠计在多 dir 普遍存在，R939-R941 一次性解了一批。

**③ 战略**：insert-* 簇基本收口（beginning 6.43% = 通用残余）。forward motion = ① 找下一含 anon-block 高 leverage 簇（box-display/normal-flow/positioning worst 扫，看哪些是 anon-block 欠计可被 backfill 进一步解）；② 或转通用 font/margin 墙（welcome/morning 16%，<p> margin gap 谱系）；③ 或 FR-002 bg / FR-003 border。

**▶ 下会话**：① 扫 normal-flow/positioning worst-15（R942 oracle 已跑），看哪些是 anon-block 高度欠计（backfill 未能解的子集，如多级嵌套 anon / float+anon 交互）→ 定位下一切片；② 或转 font/margin 通用墙（probe <p> margin gap 4 vs 20 是否 margin-collapse bug —— 若是则广泛修复）；③ A/B 守已建门禁（make test + product-smoke + normal-flow/positioning/box-display oracle）。零源码（纯 product-smoke PIL 逐行 + box-tree dump + 跨 dir oracle A/B）。

### R943 getComputedStyle stub LANDED = harness JS vein 缺失全局收口·css-grid +1（grid-with-content-dynamic-display-001 2.74→0.68%）·零回归

承 R942「下会话找下一 lever」。回顾 R916-R927 harness JS vein（+52 case，历史最高 ROI）搜剩余缺失全局：扫常见全局函数在 corpus 使用，`getComputedStyle` 41 文件（27 css-text/parsing testharness + 14 非 parsing）是唯一有意义缺失；ResizeObserver/IntersectionObserver/fetch/matchMedia/customElements/structuredClone/MutationObserver 全 **0 使用**，DOMParser/scrollIntoView/XMLSerializer 亦 0 使用。

**① 缺口机制**：`getComputedStyle` 未定义 → 裸调用 `getComputedStyle(el).getPropertyValue(...)` 抛 ReferenceError **中断整个脚本**（区别于 `offsetWidth` 属性访问返 undefined 不抛、作 reflow 触发器无害），使其后的视觉 mutation 丢失。经典模式：css-grid/grid-with-content-dynamic-display-001 line 43 `getComputedStyle(grid).getPropertyValue('grid-template-columns')`（reflow 触发、结果丢弃）→ line 47 `initiallyHidden.style.display='block'`（让绿方块显示的真 mutation）当前因中断丢失 → 渲染红方块（错）。

**② 实现**（crates/engine/src/js_dom_shim.js）：新增 `globalThis.getComputedStyle` stub，返 Proxy 包裹空 CSSStyleDeclaration（任意属性访问/getPropertyValue 返 ''、不抛）。JS 渲染前执行无真 computed 值可返；返 '' 对 `if (cs.display==='none')` 类条件可能取错分支，但脚本本会整体中断，stub 严格不劣于中断且对无条件 mutation（主流 reflow-触发模式）净正向。+单测断言（js_dom_bridge.rs test_shim_includes_modern_reftest_stubs）。

**③ 决定性 A/B（css-grid/grid-with-content-dynamic-display-001）**：无 fix（stash）= **2.74%（FAIL）**；有 fix = **0.68%（PASS）**。因果确认：stub 让 line 47 mutation 执行。

**④ css-grid 全量 A/B（49 案）**：无 fix oracle-pass 18（36.7%）/ credible 15（30.6%）/ strict 9（18.4%）；有 fix oracle-pass **19（38.8%）**/ credible **16（32.7%）**/ strict 9（18.4% 持平）。**+1 oracle/credible pass，strict 持平 = 零回归**。

**⑤ 全门禁绿**：make test **12106/0**（含新断言）；product-smoke **16.11%（= baseline，welcome 无 script 不受影响）**。

**⑥ ruled out 候选（避免后续重扫）**：scrollTo（9 文件）= 多为 `container.scrollTop=N` 属性赋值（proxy set handler 已捕获不抛）+ 少量 `window.scrollTo()` 是 feature-关键滚动测试（stub 不提供真 scroll offset，视觉不 pass）；getSelection（11 文件）= feature-关键选区测试（`assert_equals(toString(),'SS')` testharness，需真选区行为）；scrollIntoView/DOMParser/XMLParser = 0 使用。均非 reflow-触发模式，stub 低 ROI，跳过。

**裁决**：getComputedStyle 是 harness JS vein **缺失全局 sub-track 的最后一个有意义 lever**（其他常见全局 0 使用或 feature-关键）。+1 grid case 干净零回归，缺失全局 sub-track 收口。残余 harness lever 须转非缺失全局方向（CSSOM `insertRule`/`document.styleSheets` 真实现，1 文件低 ROI；或 testharness.js 框架本身——大量 assert_equals 类测试须跑 testharness 才判定，非 reftest 视觉路径，ROI 待评估）。

**▶ 下会话**：① harness 非-缺失全局方向 ROI 调查（CSSOM / testharness.js 框架）；② 或回 R942 ① normal-flow/positioning worst-15 anon-block 欠计簇扫描；③ 或 font/margin 通用墙（welcome 16%）。A/B 守已建门禁。

### R944 harness 完整性审计 + css-tables 新鲜基线 = 单 session harness lever 穷尽·css-tables 43.8→56.5%（纠正 stale）·4 方向 rule-out（零源码·纯调查）

承 R943「harness 非-缺失全局方向 ROI 调查」。系统审计 harness 完整性找剩余 lever，**结论：单 session harness lever 穷尽**，4 方向 rule-out，css-tables 新鲜基线纠正 stale 控制面。

**① css-tables 新鲜基线（控制面纠正）**：全量 oracle 复测 css-tables **65/115 = 56.5% oracle-pass / 54 credible / 6 strict（5.2%）**。master.md line 782「tables 43.8%」**stale**（pre-R922/R939-R941）；R892 时 53.9% → 现 56.5%（+2.6pp，来自 R922 table-fixup merge +4 / R939-R941 anon-block backfill serendipitous / harness JS vein）。harness JS + backfill 的跨 dir 累积收益在 css-tables 显著但未记录，本轮纠正。

**② R943 total yield 确证**：getComputedStyle stub 唯一 A/B 翻转为 css-grid/grid-with-content-dynamic-display-001（2.74→0.68%，+1）。其余 getComputedStyle 文件：CSS2/visudet/line-height-204（onload measure() 两态都 0.56%，setup 视觉无关）/ css-flexbox auto-margins-001 + CSS2 positioning/ui/floats 4 文件**无 oracle shot**（不在可测分母）/ 27 css-text/parsing 为 testharness assert（非视觉 reftest）。R943 完整收益 = **+1 css-grid，零回归**。

**③ harness 完整性审计 4 方向 rule-out（避免后续重扫）**：
- **@import 跟随**：93 文件用 @import，但 CSS2/cascade at-import-001~011 测「@import 在非法位置被忽略」语义，**全 9/9 oracle-pass @ 0.38-0.85%**（ZW CSS parser 已正确忽略非法 @import，harness 不跟随恰好正确）。跟随合法 @import 须尊重有效性规则+级联+递归，且会破坏这些 parsing 测试 → 非 lever。
- **DOMContentLoaded / module / defer**：corpus 用量 1/0/1（微），非 lever。harness 已派发 load + 执行 onload handler（reftest.rs:876/930-939）。
- **Ahem 字体加载**：ZW harness 正确——`load_font_faces_into` 对 Ahem family 特殊跳过（FontLoader 按 family 合成方块）+ Ahem.ttf 从 `tests/wpt-runner/fonts/` 加载（reftest.rs:1030）；R388 实测 fontdue Ahem 光栅化 ≈ chromium（W 0.1%/i 3.0%）。非缺口（区别于 R388 已修的 oracle 侧）。
- **font-features（shaper.rs:82 `features: Vec::new()`）**：R513 标的 font-feature 族单一硬阻塞，但**多会话结构性**——须把 computed `font-feature-settings`/`font-variant-*` 从 style→paint→shaper 全链路打通（shape_single_line API 不带 features）+ layout 同源（layout 用 fontdue 非 rustybuzz）。非单 session。

**④ css-tables worst-15 复查（全结构性）**：table-cell-width-0（20%，R97 intrinsic taffy-blocked）/ percent-height-overflow-auto + percentages-grandchildren-quirks（quirks %）/ table-row-group-color-inheritance（R892 table-fixup 结构）/ **table-cell-overflow-explicit-height-001/002（9.73%×2，新进 worst-15）**= 测 td height:20px+overflow:hidden 应==ref（cell height-as-min 长到 fit，overflow 不裁剪）；table.rs:1710-1716 已 cell height=max(row,content) 增长，9.73% diff 是其他 table-layout 细节（border-collapse/anon-table-box），非 clean 单点 / dynamic-table-cell-height（7.95%）= `<div display:table-cell>` 匿名表盒 + height:50px 含 100px 块的 height-as-min 交互，结构性。R892 css-tables plateau 再确认。

**裁决**：harness JS vein（R916-R927 + R943 缺失全局收口）后，harness 完整性审计确认**单 session harness lever 穷尽**（@import/DOMContentLoaded/Ahem/font-features 全 rule-out）。css-tables 56.5% 是 R922+R939-R941+harness JS 的累积未记录收益。forward motion 须转结构性多会话方向。

**▶ 下会话**：① multicol Phase 2 嵌套碎片化 spec-rfc（R896/R925 待启动，最低 dir 23.5% headroom 最大）；② font-features style→paint→shaper 全链路首切片（R513 单一入口 shaper.rs:82，fonts dir 34.8% / font-features 89 案仅 10%）；③ Phase A 字体度量（welcome/morning 16% 墙）。均多会话，按 rally 续跑协议推 spec-rfc 首切片。

### R945 Phase A 收敛 + font-features/multicol 双双 Phase-A-blocked 实证 + R897 sentinel 已 pass·forward motion 化归 Phase A 单一前置（零源码·纯调查）

承 R944「multicol Phase 2 spec-rfc / font-features 首切片」。深入核验两方向可实施性，**结论：双双被 Phase A IFC 统一阻塞，forward motion 化归 Phase A 单一结构前置**。

**① font-features Phase-A-blocked 实证（直接证据）**：`crates/engine/src/paint/painter/text.rs:891` 注释「传递真实 styles 会导致 4 个测试回归（BFC-004, font-feature-002, …）」——paint IFC 经 R84 单行守卫用**空 styles** 重跑（区别于 layout IFC 的真实 styles）。故即便把 `font-feature-settings` 从 style→shaper.rs:82 plumb，paint IFC 重跑时 styles 为空 → features 不传 → 无效。**font-features 须先解 paint IFC 空-styles 墙 = Phase A 同源**（R513 判断实证）。style-system 已识别 font-feature-settings 继承（matcher/mod.rs:923）但 paint 侧空-styles 断链。

**② multicol R897 sentinel 已 pass（设计假设纠正）**：R897/R898 algorithm（`column_fragmentation_flow.rs`，9 单测，零生产调用）以 multicol-fill-auto-001 为 sentinel 锚，但实测 sentinel = **0.00% PASS**（001/002/003 全 pass @ 0.0-0.8%）。R897 probe A1 用**合成**案（height=48px/12行/3列）实证 gap，真实 sentinel 不触发。**algorithm LANDED 但无确认真实驱动案**——wiring（commit 2 接线）= dormant-infra 陷阱（code-guidelines 禁止零价值基建），除非先找到匹配 slice criteria（单层+单block子+column-fill:auto+明确高度）的真实 failing 案并诊断。残余 failing auto-fill 案（fill-auto-004 2.74% / 005 2.08% / column-height-010 3.94% / multicol-fill-000 4.33%）结构未核，wiring ROI 不确定。

**③ multicol worst-15 全结构性（R908/R757 再确认）**：fresh oracle 115/452 = **25.4%**（较 R893 23.0% 升，R901-R907 + harness JS 收益）。worst = column-balancing-paged-print（81%，range-out）/ multicol-rule-nested-balancing（Phase 3 嵌套）/ **multicol-span-all-children-height-003/004a/004b/006/007（5 案 15-30%）**= column-span:all 分段 + 百分比 CB（相对整个 multicol 非段）+ 碎片化复合 gap，ref 用显式 height + 多 columns div 模拟，结构性 / multicol-breaking-005（Phase 3）。无单 session clean 案。

**④ css-position R942 ① scan 落空**：fresh 51/97 = **52.6%**（stable，R923 数）。worst-15 = abspos-semi-replaced-stretch / dynamic-relayout(JS) / root-element-abspos-in-flex-grid / relpos / backdrop —— **无 anon-block 欠计簇**（R939-R941 backfill vein 不延伸到 positioning）。

**★ 战略收敛**：rendering-compat forward motion 全部化归 **Phase A IFC 统一**（paint IFC 用真实 styles 而非空）这一**单一结构前置**——它解锁 multicol Phase 2b（R757）+ font-features（本轮实证）+ large-font 簇（R101）+ welcome/morning 字体度量（R633）。Phase A 是 R125-R213 五轮净负 + R247 deadlock + R627 font_size 线关闭的**已知硬 deadlock**。`phase-a-IFC-unification-design.md` 已存在。**下一步 = 用新架构角度重启 Phase A spec-rfc**（非重试旧 net-negative 切片），例如：paint 侧消费 layout 已存的 inline_layout 行盒（而非重跑 IFC），绕开空-styles 墙；或 store 时把 font-feature-settings/line-height 等关键样式预烘焙进行盒。多会话，rally 续跑。

**▶ 下会话**：启动 Phase A IFC 统一 spec-rfc（新架构角度：paint 消费 layout 存储行盒 / 关键样式预烘焙进行盒，绕开 R84 空-styles 墙）—— 这是解锁 multicol Phase 2b + font-features + large-font + welcome/morning 的共同前置，最高 leverage。先读 `phase-a-IFC-unification-design.md` 现状 + R630/R632 已落地的 paint Path B 改进，定位未试过的 seam。备选：先核 multicol fill-auto-004/005/column-height-010 结构是否匹配 R897 slice（若是，wiring 有真实驱动，可解锁 R897 dormant algorithm）。

### R946 outline-width em 解析修复 LANDED = R907 同模式 length_to_f32 Px-only 墙·unit-test 验证·**A/B 零可测收益**（em 案非 reftest）·零回归

承 R945 后寻非-Phase-A code lever。R907 模式系统扫描「length 字段缺 compute resolve + paint Px-only 丢弃 em」class bug。

**① 扫描结果（computed.rs 40 LengthValue 字段 vs resolve 列表）**：缺 resolve 的 7 字段 = outline_width / outline_offset / text_indent / transform_origin_x/y / perspective / perspective_origin_x/y。其中：
- **text_indent**：paint（text.rs:804）本地解析 em（`Em(v)=>v*font_size`），**非 bug**。
- **outline_width**：paint_outline（border.rs:554）经 `length_to_f32`（helpers.rs:583，`Px=>p, _=>0.0`）→ em outline-width **丢为 0** → outline 消失。**真 bug**（R907 同模式）。
- outline_offset / transform_origin / perspective：corpus 0 用量（非 lever）。
- border-radius percentage：`length_to_f32` 也丢，但仅 1 案 + 需 box-dim 解析（非 compute-期），跳过。

**② 修复**（style-system/computed.rs）：outline_width 加入 resolve_length_field 列表（word_spacing 后），em/rem/ch 解析为 Px。outline_offset 0 用量不入列（code-guidelines 不做零价值）。+unit-test（test_resolve_outline_width_em：outline-width:5em @ fs=32 → Px(160)，修前为 Em(5)）。

**③ 验证**：make test **12107/0**（+1 新测试）；product-smoke **16.11%（= baseline）**；unit-test 证 em→Px。

**④ A/B 决定性（outline-width 簇 30 案 oracle）**：with-fix 27/30 (90%) / worst applies-to-005/006 4.03% / 096 1.65% / 002/004 0.85%；**baseline（stash）完全相同** 27/30 / 同 worst 同百分比。**零可测收益**——因 em outline-width reftest（023/034/045/056/067/069/072/078/095）**均无 `<link rel=match>`（非 reftest，loader 跳过）**；唯一 em reftest outline-width-068 outline 太小，缺失不改变 diff。Px/keyword outline reftest（001-018/096/applies-to）不受 fix 影响。

**裁决**：outline-width em 是真 correctness bug（unit-test 证），修复 surgical（4 行 + test）零回归，但**零可测 reftest 收益**（em 案非 reftest）。作为 latent bug 修复 + R907 一致性收口提交（区别 R907 column-rule-width 当时有 reftest 驱动 +3 oracle）。诚实记录：本 session 无可测 pass-rate 进展，forward motion 仍须 Phase A（R945 收敛）。

**▶ 下会话**：Phase A IFC 统一 spec-rfc（R945 下会话，最高 leverage）；或继续 R907 模式扫 paint 侧其他 Px-only 墙（clip-path em/percentage——M9 内部，需查 ClipPathValue 是否 compute-期 resolve；但 corpus 覆盖低）。

### R947 Phase A 墙 ③ 精确切片定位 = vertical-align 簇 live driver（6 案 13-21%）·R877 line_height plumbing post-characterization 未试·stored-path 非-Ahem dead code 实证（零源码·纯调查）

承 R946「forward motion 阻塞 Phase A」。深读 phase-a-IFC-unification-design.md + linebox-metric-unification-rfc.md + R816/R817/R877 master 记录 + 当前 paint 代码，**精确定位 Phase A 墙 ③ 的 live driver + 未试切片**。

**① vertical-align 簇 = live measurable driver（实测确认）**：vertical-align-117a **13.59%**（R813 时 24.15%，R816/R817 改善）/ 118a 13.25%（23.72%）/ baseline-004a 18.94%（21.85%）/ baseline-005a 19.83%（21.02%）/ vertical-align-122 20.77%（未动）/ negative-leading-001 20.70%（21.45%）。**6 案全仍 failing**（>1%），R816/R817 linebox 度量统一 Phase 1/2 改善了 5/6 但**未推过 1% 阈值**。完成 baseline 消费 = 潜在 +6 oracle。

**② 基建已就绪（R816/R817 LANDED）**：LineBox 已存 `baseline_y`/`ascent`/`descent`（inline_types.rs:155-，R816 Phase 1）；text.rs:844-871 R817 Phase 2 已存 per-fragment `baseline_y_abs`；text.rs:1313-1328 stored-path 已消费 baseline_y_abs。**stored-path（Path A）baseline 已正确**。

**③ R877 精确表征的真路径（未试）**：stored-path 非-Ahem 分支（text.rs:1333）是 **dead code**（R84/R829 仅 pure-Ahem 块存储 → 非-Ahem 分支几乎不触发，改它零效果，R877 实证 welcome/linebox 字节同）。**真路径 = non-stored path（text.rs:1345-1361 `for fragment in fragments.iter()` → `render_fragment!` 宏）**，其 `$baseline_offset = baseline_fs = font_size`（text.rs:1351/1356），公式 `glyph_y = content_y + frag.y + font_size`。R877：正确公式 `glyph_y = baseline_abs - 2·half_leading`，须用 **line_height**（非 font_size）。**但 non-stored path 的 TextFragment 无 line_height 字段**（仅 y/font_size/is_ahem）。

**④ 关键：R877 下一步 post-characterization 未试**：R877「下一步：TextFragment（types/mod.rs）加 line_height/baseline_y 字段，IFC 终化填充，paint non-stored path（text.rs:1359）改 $baseline_offset 用之」。**R878-R946（multicol R897-R907 / harness JS R916-R927 / R109 backfill R939-R942 / getComputedStyle R943 / outline R946）均未触碰此切片**。4 次净负尝试（R834/R836/R849/R875）**全在 R877 精确表征之前**（用 font_size 而非 line_height）→ post-R877-characterization 的 line_height 切片**未试过**。

**⑤ 风险**：4 次净负先例（ifc-001/002/003 + multicol 回归）；公式精确推导（glyph_y = baseline_abs - 2·half_leading，half_leading=(line_height-font_size)/2）须严谨；触及核心 paint IFC。env 门控 + 三态 A/B 守（welcome<20% + linebox/css-text oracle 零回归 + multicol-fill-auto sentinel）。

**裁决**：vertical-align 簇（6 案 13-21%）是 Phase A 墙 ③ 的 **confirmed live driver**（区别 R897 sentinel 已 pass）。R877 表征的 TextFragment line_height plumbing 切片 post-characterization **未试**（4 先例 pre-characterization）。这是 Phase A 最高 ROI 切片（潜在 +6 oracle + welcome/morning 行度量），但需 fresh session 全预算做：实现（TextFragment 加字段 + IFC 填充 + paint non-stored path + env 门控）+ 精确公式推导 + A/B 回归调试。

**▶ 下会话**：实施 R877 下一步切片——① 读 R877 完整分析（master.md R877 条 + evidence）取精确公式；② TextFragment（crates/layout-engine/src/types/mod.rs）加 `line_height: f32`；③ IFC 终化（inline_finalization.rs 或 inline/mod.rs）按所属行盒填 fragment.line_height；④ paint non-stored path（text.rs:1351）`$baseline_offset` 从 font_size 改为 line_height 派生（使 glyph_y=baseline_abs-2·half_leading）；⑤ env `PHASEA_FRAG_LH` 门控；⑥ A/B：vertical-align 簇 6 案 + welcome/morning product-smoke + linebox/css-text oracle + ifc-001/002/003 + multicol-fill-auto sentinel。净负即 env 关回退。

### R948 R877 non-stored line_height 公式 A/B 实验证伪 = welcome +610px（更差）·welcome diff 是**横向**字体 advance 非纵向 line 位置·vertical-align 簇是 Ahem→stored path（R877 non-stored 不适用）·R947 path 混淆纠正（零 net 源码·实验回退）

承 R947「实施 R877 non-stored line_height 切片」。**实证实施 + A/B 证伪 R947 假设**——R877 non-stored 纵向公式**无 beneficiary**。

**① 实施 + A/B**：paint non-stored path（text.rs:1345）加 env `PHASEA_FRAG_LH` 分支——按 inline_ctx.lines 迭代拿 line.height，v_offset = font_size - (line.height - font_size)/2（R876/R877 公式：glyph_y=baseline_abs-2·half_leading；lh=fs 退化为 font_size=现行，lh>fs 上移 glyph）。release 编译绿。
- welcome product-smoke：**ON 77915px（16.23%）vs OFF 77305px（16.11%）= +610px 更差**。
- 证伪后 env-gated 代码已回退（git checkout，零 net 源码）。

**② 根因（welcome diff 轴向）**：welcome 16% diff 是**横向**（字体 advance/选择/glyph 形状，R374/R631 谱系——R631 证伪 selection 但 metrics 残余），**非纵向 line 位置**。R877 纵向 v_offset 公式针对错误轴 → welcome 不受益反微损。image analysis（R948 前会话）亦确认 welcome 区域差全在文本 glyph 形状/间距，布局盒正确。

**③ R947 path 混淆纠正**：R947 把 vertical-align 簇（117a 等）当作 R877 non-stored fix 的 driver，**错误**。117a = `font:2.5em/3.25 Ahem`（**Ahem**）→ 走 **stored path**（text.rs:1309-1340，R817/R841 已用 baseline_y_abs + ahem_uses_embox_position）。R877 non-stored path 只服务**非-Ahem 未存储**容器（welcome/morning 等的 Gate-2 排除容器，但 welcome 主文本容器 Gate-2 存储→stored path）。故 vertical-align 簇的 fix 位置是 **stored path R817/R841 残余**（text-bottom 等 vertical-align 偏移在 apply_vertical_alignment 的存储值），**非** R877 non-stored。

**④ 战略收敛（双确认）**：(a) welcome/morning 16-18% = **横向字体 advance/metrics 墙**（R374/R631），非纵向 Phase A 墙 ③——R877/non-stored 纵向公式无益（A/B 实证）；(b) vertical-align 簇 6 案 = **stored path Ahem**（R817/R841 残余），fix 位置在 stored path 非 non-stored。**R877 non-stored line_height slice 被实证证伪为无 beneficiary 的 dead 路径**（welcome 横向错轴 + vertical-align 系 stored）。

**裁决**：R947 假设（R877 non-stored slice 是 Phase A 墙 ③ live lever）**实证证伪**。本会话产出 = 一次有价值的实验性 rule-out（区别纯分析）+ path 结构精确化（3 paint 路径：stored-R817/multicol-旧v_offset/non-stored-baseline_fs；Ahem→stored，非-Ahem 文本多走 stored 非-Ahem 分支 frag.font_size）。forward motion：(a) vertical-align 簇真实 fix 位置 = stored path R817/R841 vertical-align 残余（6 案 driver，须诊断 text-bottom 偏移存储）；(b) welcome/morning 横向字体墙须 font-metric 接入（R374 多会话），非 Phase A 纵向。

**▶ 下会话**：① vertical-align 簇真实 fix——诊断 stored path（text.rs:1309-1340）对 `vertical-align:text-bottom/top`（117a 等）的残余：apply_vertical_alignment（inline/mod.rs）算 text-bottom 偏移是否正确存入 line.baseline_y/ascent/descent + stored path 消费（R817/R841）是否对 text-bottom 子集正确；REFTEST_DUMP 117a 定位纵向偏移差。② 或 welcome/morning 横向字体墙（font-metric 接入，R374 多会话）。③ R877 non-stored slice 勿再试（A/B 证伪，welcome 错轴）。

### R949 vertical-align 精确根因 = stored path v_offset 消去 frag.y → vertical_align 对 Ahem 存储容器完全无效·run.y paint-dead 实证（117a 13.59% on/off 不变）·TextBottom run.y 修复 paint-dead 已回退（零 net 源码·纯调查+实验）

承 R948「诊断 stored path vertical-align 残余」。REFTEST_DUMP 117a + 实施 run.y 修复 A/B，**精确锁定根因**：stored path v_offset 公式**消去 frag.y**，使 vertical_align 偏移对 Ahem 存储容器**完全无效**。

**① run.y paint-dead 实证**：apply_vertical_alignment line 1665 `Bottom | TextBottom => line_height - run.height`（CSS §10.8.1：bottom=line-box 底，text-bottom=父 content-area 底=(line_height+dominant_fs)/2，差 half_leading——**ZW 把二者同公式 = spec bug**）。实施分离修复（env `PHASEA_TEXTBOTTOM`：TextBottom => content_area_bottom - run.height）+ A/B 117a：**ON 13.59% vs OFF 13.59% 完全相同 = paint-dead**。→ run.y 改动对 stored-path 渲染零效果。回退（零 net 源码）。

**② 精确根因（stored v_offset 消去 frag.y）**：stored path Ahem embox v_offset（text.rs:1326）= `frag.baseline_y_abs - 0.8·font_size - frag.y`。代入宏 glyph_y = `content_y + frag.y + v_offset` = `content_y + frag.y + (baseline_y_abs - 0.8·fs - frag.y)` = **`content_y + baseline_y_abs - 0.8·fs`**——**frag.y 项消去**。frag.y（= line.y + f.y，含 apply_vertical_alignment 算的 vertical_align 偏移）对 glyph_y **无贡献**。故 stored-path（Ahem，117a 等）容器上 vertical_align（text-bottom/top/sub/super）**完全无效**——所有 fragment 渲染在同一 baseline_y_abs 位（117a 蓝 span 未上移到 content-area 底，与黑 TTTT 同基线 → 13.59% diff）。R817/R841 embox 公式设计假设 baseline 对齐（frag.y 编码基线位，消去它正确定位 baseline）；对 vertical_align 偏移的 fragment，消去 frag.y = 丢偏移。

**③ REFTEST_DUMP + image analysis 辅证**：117a self-source FAIL；ref（绝对定位 img 模拟）黑 [90,130] 蓝 [45,85]（蓝高于黑 half-leading）。ZW 蓝未充分上移（frag.y 偏移被 stored v_offset 消去）。

**④ 修复方向（未实施，须 plumbing）**：stored path 须**区分 vertical_align**——对非-baseline 的 fragment，v_offset 须**保留 frag.y 偏移**（非消去）。须：(a) InlineLayoutFragment + PaintFragment 加 vertical_align 字段（layout→paint plumbing）；(b) stored path 条件 v_offset：baseline 对齐用现行 embox 公式（消去 frag.y），vertical_align 偏移用保留 frag.y 的定位。风险：embox 公式对 baseline 子集的 frag.y 消去是 R817/R841 精调（4 先例 net-negative 谱系），vertical_align 子集的保留定位须经验推导 + A/B 守 linebox/ifc-001/002/003。

**裁决**：vertical-align 簇（6 案）精确根因 = **stored v_offset 消去 frag.y**（vertical_align 对 Ahem 存储容器无效）。本会话 = 一次精确根因定位（区别 R948「stored 残余」模糊表述）+ run.y paint-dead 实证。forward motion = plumbing vertical_align 到 stored path + 条件 v_offset（须 fresh session 全预算 + 经验公式推导）。

**▶ 下会话**：实施 vertical-align stored-path 修复——① InlineLayoutFragment（layout-engine types/mod.rs）+ PaintFragment（painter/text.rs:841）加 vertical_align 字段，layout 存储时填；② stored path（text.rs:1309-1340）条件 v_offset：baseline 用现行 embox 公式，text-bottom/top/sub/super 保留 frag.y 偏移定位；③ 经验推导 vertical_align 子集的 glyph_y 公式（参考 ref 像素位 117a 黑[90,130]蓝[45,85]）；④ env 门控 + A/B 守 linebox/css-text oracle 零回归 + ifc-001/002/003 + welcome<20%。净负回退。备选：welcome/morning 横向字体墙（R374 多会话）。

### R950 vertical-align half_leading 修复 A/B 实证回归 = 117a 13.59→16.09 / 118a 13.25→15.92·post-extension 推导是有意补偿非简单值错·**vertical-align 簇 6 轮 net-negative trap 声明放弃增量公式微调**·实验回退（零 net 源码）

承 R949「stored v_offset 消去 frag.y」。深入读 inline_finalization.rs:701-720 发现 R822 Phase 3 已设 per-fragment valign-aware baseline_y（TextBottom => line.baseline_y - half_leading），但 half_leading 经 `((line.ascent - line.height/2)/0.3)` 推导用 **post-R822-扩展** metrics（line.ascent/height 已含 top_extend）。数学推导：117a 算出 half_leading=30（应 45）。

**① 实施 + A/B**：LineBox 加 `half_leading: f32` 字段（10 构造点默认 0.0），apply_vertical_alignment 在扩展前设 `line.half_leading = half_leading_hl`（= (line_height-strut_fs)/2，扩展前 = 正确 45），inline_finalization 直接读 `line.half_leading` 替代 faulty 推导。release 编译绿。
- 117a：**13.59% → 16.09%（+2.5pp 更差）**。
- 118a：13.25% → 15.92%（更差）。
- 122 (20.77%) / baseline-004a (18.94%) 不变。
- **回归 → 回退**（3 文件 git checkout，零 net 源码）。

**② 根因（补偿陷阱）**：post-extension 推导（30）非简单错误——它与 R822 扩展后 baseline_y（122）配合 `line.baseline_y - half_leading = 122-30 = 92` 是**有意补偿**（比"正确"值 45 给出的 122-45=77 更近 ref）。用"正确"half_leading（45）破坏补偿 → 回归。vertical-align stored glyph 定位（R817/R822/R841/inline_finalization:703）是多重补偿精调系统，任一值"修正"破坏整体平衡。

**③ vertical-align 簇 = 6 轮 net-negative trap（声明）**：R834/R836/R849/R875（TextFragment line_height plumbing 4 轮）+ R949（run.y paint-dead 实证）+ R950（half_leading 回归）。**增量公式微调全部失败**——vertical-align stored-path 是 intricate compensation 系统，非单点可修。**放弃 vertical-align 增量角度**（勿再以单会话重试 run.y / half_leading / v_offset 微调）。

**④ 战略状态**：rendering-compat measurable 进展 6 轮阻塞在 Phase A vertical-align（+ welcome/morning 横向字体墙 + multicol Phase 2）。所有 lever 化归 Phase A 硬 deadlock。vertical-align 增量角度已穷尽（6 轮证伪），真修须**架构性 Path B 消除**（paint 全面消费 stored baseline_y，消灭 dual-path 分歧——phase-a-IFC-unification-design.md 推荐但巨大多会话）或转非-Phase-A 方向。

**裁决**：R950 = half_leading 增量修复实证回归 + vertical-align 增量 trap 声明。本 rally 段（R945-R950）6 轮 Phase A 诊断/实验全部 net-neutral-or-negative。forward motion 须转：(a) 架构性 Path B 消除（巨大，多会话）；或 (b) 非-Phase-A lever（product-smoke morning/wintertc layout 诊断——R174/R227 在 welcome 找到过真实非字体布局 bug，morning/wintertc 或有同类）。

**▶ 下会话**：**转非-Phase-A**：product-smoke morning.work（18%）/ wintertc（13.59%）layout 诊断——用 ui_diff_check 定位 diff 区域，区分字体墙（横向文本）vs 布局 bug（盒错位/重叠/缺失，R174 padding 双计 / R227 同模式）。若有非字体布局 bug → 修复（+DC-13 product-smoke 进展）。vertical-align 增量角度勿再试（6 轮 trap）。架构性 Path B 消除留待多会话 spec-rfc。

### R951 product-smoke 字体墙全确认 = morning/wintertc 布局完整·DC-13 residual 纯字体 advance 墙·非-Phase-A product-smoke 方向无布局 lever（零源码·纯调查）

承 R950「转 morning/wintertc layout 诊断」。PIL 像素读 + ui_diff_check 实测，**product-smoke 三 fixture 全是字体 advance 墙，布局完整无 bug**。

**① morning（14.27%）**：PIL 读 corner 像素 chromium/zw 均 (255,255,255)（body bg `--color-bg:#f9f7f4` 在内层元素/视口 margin 白，两态一致，非 ZW bug）。ui_diff_check 标的「前言背景缺失/header 分隔线过粗」**经 PIL 证伪**（角落两态白一致，AI 视觉把字体 advance 墙误判为背景/分隔线差）。:root 变量（rgba alpha + hex）R175（var 继承）+ R170（rgba）已正确处理。**morning = 字体 advance 墙**（同 R948/welcome）。

**② wintertc（16.22%）**：ui_diff_check 确认「布局完整」——4 nav buttons 并排正确、元素对齐、颜色匹配、**SVG/PNG logos 正确渲染**（R318 image pipeline 工作，非占位/缺失）。diff 全是文本字体墙（"extra logos"系 AI 误数）。**wintertc = 字体 advance 墙**。

**③ DC-13 residual 全字体墙**：welcome 16.11% / morning 14.27% / wintertc 16.22% **三 fixture 全字体 advance 墙**，布局/图片/盒子全完整（R174 padding 双计 + R172 border-radius bg bypass + R170/171 rgba + R227 padding 双计 + R255 ua_default_display 已修真实布局 bug）。DC-13 product-smoke residual = **纯字体度量墙（R374/R631/R633）**，无布局 lever。

**④ 战略收敛（font-metric wall 是唯一阻塞）**：rendering-compat measurable 进展（DC-2~5 文本目录 + DC-13 product-smoke）**全部化归字体度量墙**：advance-width（R225/R375b definitive 关闭）、line-height/baseline（vertical-align 6 轮 trap）、font-selection（R631 证伪）。font-selection ≠ root cause（R631 强制同字体 0% 变化），真因 = font METRICS（advance/line-height）= Phase A same-source（layout-IFC estimate vs paint-IFC fontdue）。**所有 measurable lever 均字体墙或结构性，无单 session clean lever**。

**裁决**：R950 转的 non-Phase-A product-smoke 方向**实证无布局 lever**（morning/wintertc 全字体墙）。本 rally 段（R945-R951）7 轮诊断/实验确认 rendering-compat **全 measurable 进展阻塞在字体度量墙**。forward motion 仅剩：① 架构性 Phase A（Path B 消除 / font-metric 统一，巨大多会话，5+ net-negative 先例）；② fresh 纯布局（非文本）reftest case（font-wall 不适用，但 CSS2/css-position/css-tables 已 R870-R892 scour，diminishing）；③ 零散 correctness 修复（R943/R946 式，不动 pass rate）。

**▶ 下会话**：① fresh 纯布局（非文本）reftest case 诊断——扫 CSS2/css-position/css-tables worst（font-wall 不适用的几何/盒模型案），找 R939-R943 式 clean 单点（区别已 scour 的 anon-block/font 机制）。② 若 0 yield → 渲染兼容 measurable 目标确实字体墙阻塞，考虑深度 Phase A spec-rfc（Path B 消除）多会话启动。product-smoke 布局诊断勿再（R951 证全字体墙）。

### R952 fresh-layout 扫描印证 scour + Phase A 循环 deadlock 确认 = position-relative-002 R109+%/font_metric R915 closed/css-floats 不可测（零源码·纯调查）

承 R951「fresh 纯布局 case 诊断」。执行扫描，**全印证已 scour / 结构性 / 不可测**。

**① position-relative-002（css-position，5%，纯几何无文本 font-wall 不适用）**：`div>span(relative,top:100,left:100)>div(relative,top:-100%,left:-100%)`。green div 的 `top:-100%` 须对 containing block（span，inline）解析 = -100px。`apply_block_relative_percent_insets`（engine.rs:2235，R850 加）**仅对 style.height==Px 的 CB 解析 top/bottom%**——span 的 style.height=Auto（used=100px from green div）→ cb_h=None → top% 不解析 → green 不上移 → 红露。**结构性**：block-inside-inline（R109 §9.2.1.1）+ relpos 百分比 inset + inline CB definiteness（used-height-definite-but-style-Auto）三子系统交互。修须用 used content_height 而非 style.height==Px 判 definite，但风险（auto-height 父级 % 应为 0 的 case 会错），非 clean 单点。

**② font_metric_provider（R885 dormant）已 R915 证伪**：R915「line-metric FontMetricProvider 注入证伪（welcome=font-engine 墙 comprehensively confirmed）」——注入 FontMetricProvider 替换 apply_vertical_alignment 的 0.8·fs 启发式（line 1622/1624/1634）已试，证伪。**非未试 Phase A 切片**（纠正「font_metric_provider step-2 未试」假设）。

**③ css-floats 不可测**：`reftest-oracle css-floats` = 0 cases scanned（无 oracle shot，不在可测分母）。CSS2/floats + CSS2/floats-clear 子目录已 R870-R892/R108b/R145/R881 scour。

**④ Phase A 循环 deadlock 确认**：Path B 消除（paint 全面消费 stored baseline_y，消灭 dual-path 分歧）依赖**墙 ③**（Path A 多行非-Ahem baseline 正确）；墙 ③ = vertical-align 6 轮 trap（R834-R950）。**Path B 消除 ← 墙 ③ ← 6 轮 trap**：循环依赖，无单 session 突破。font-metric 全子项（advance R225/R375b / selection R631 / line-metric R915 / vertical-align R834-R950）**全 closed/refuted**。

**裁决**：R945-R952 八轮确认 rendering-compat measurable 进展**循环 deadlock**——所有 measurable lever（font-metric / vertical-align / product-smoke / multicol Phase 2 / fresh-layout）化归 Phase A 墙 ③ 或字体墙，均 closed/refuted/structural。**无单 session clean measurable lever 残留**。forward motion 仅剩：① 架构性 Phase A 破墙 ③ 的新角度（empirical per-case 像素调 vs 公式级，区别 6 轮公式 trap）；② 接受 plateau + 零散 correctness（R943/R946 式，不动 pass rate）；③ 转 rendering-compat 之外的 zero-web 子目标。

**▶ 下会话**：① 墙 ③ empirical per-case 角度——选 1 个 vertical-align case（117a），REFTEST_DUMP + 手动推导正确像素位（ref 黑[90,130]蓝[45,85]），针对该 case 像素级调（区别 6 轮公式级 trap），看是否可推广。② 或接受 measurable plateau，转零散 correctness 修复（不动 pass rate 但修真 bug）。③ product-smoke/font-metric/vertical-align 公式级勿再试（R945-R952 八轮证）。

### R953 非存储路径 baseline_offset 修复 LANDED = 纠正 R945–R952 八轮误归因·glyph 定位 = half-leading·≈ +102 oracle（harness JS vein 后最大单轮·零目录回归）·welcome +0.77pp（font-wall trend-only）

承 R952「墙 ③ empirical per-case 角度」。按建议对 117a 做像素级 forensics，**纠正 R945–R952 八轮归因**并落地一个**纯渲染正确性大胜**（非 harness/JS）。

**① PHASEA_VALIGN 实验（embox-always + half_leading）证伪 Ahem glyph 角度**：实施 stored-path Ahem 无条件 embox（`baseline−0.8·fs`，区别 R841 仅 lh≈fs）+ inline_finalization half_leading 用 `(frag.height−fs)/2`（区别 post-extension 反推 30）。VALIGN_DEBUG 实测 117a 黑 `[90,130]` / 蓝 `[45,85]` div-rel **像素级精确匹配 ref**。但 oracle **更差**（13.59→14.03%）：dominant error 非 Ahem glyph。回退（零 net 源码）。

**② 真根因 = 默认字体 `<p>` 块非存储路径首行定位**：117a `<p>Test passes if...</p>` 非 pure-Ahem → 走**非存储路径**（R84 空-styles 墙）。PIL 实测 ZW `<p>` 文本 p_top+11.2、chromium p_top+1.6 → ZW div y=51、chromium div y=68（**ZW 高 17px**，117a 13.59% dominant）。R945–R952 八轮全在攻 Ahem glyph（次要），未触 `<p>` 默认字体（主因）。

**③ 根因公式（baseline_offset）**：非存储路径 `glyph_y = content_y + frag.y + baseline_offset`，旧 `baseline_offset = font_size`。glyph 顶（行盒相对）应 = half-leading = `(lh−fs)/2`（em-box 居中，**与 ascent 无关**）。`frag.y = baseline_y − run.height`，`baseline_y = half-leading + ascent`，故 `baseline_offset = height − 0.8·fs`（ascent≈0.8·fs 启发式，与 apply_vertical_alignment line 1634 一致）。旧 font_size 把 glyph 顶放基线位，每行偏低 ≈9.6px。**修复**：`baseline_offset = fragment.height − 0.8·fragment.font_size`（仅文本运行 fs>0；原子盒 fs==0 保留旧值）。text.rs 非存储路径 ~1345。

**④ 决定性 A/B（全目录零回归）**：
| 目录 | OFF | ON | Δ |
|------|-----|-----|---|
| css-text | 279 (16.9%) | 339 (20.5%) | **+60** |
| css-text-decor | 72 (29.8%) | 99 (40.9%) | **+27** |
| css-position | 51 (52.6%) | 54 (55.7%) | +3 |
| css-tables | 65 (56.5%) | 68 (59.1%) | +3 |
| css-fonts | 75 (26.6%) | 79 (28.0%) | +4 |
| css-multicol | 115 (25.4%) | 119 (26.3%) | +4 |
| css-writing-modes | 55 (7.0%) | 56 (7.1%) | +1 |
| **合计** | | | **≈ +102** |

★ harness JS vein（R916–R927 +52）后**最大单轮 oracle 增益**，且是**纯渲染正确性修复**（非 harness/JS），区别 R939–R943 anon-block/harness 谱系。机制：WPT corpus 几乎每个 reftest 含 `<p>Test passes if...</p>` 默认字体指令行，修其垂直定位普适。

**⑤ 门禁全绿**：make test 12107+/0 failed（exit 0）；clippy -D warnings 零警告；fmt clean；self-source reftest 686/686（零回归）。

**⑥ DC-13 product-smoke（trend-only）**：welcome 16.11→16.88%（+0.77pp，< 20% 退出阈值，font-wall trend-only）。**hero title 反而更准**（ORA 104-135 / OFF 135-154 低 30px / ON 105-124 近 ORA）。残余净 +0.77pp = 真字体 system-ui ascent≈0.9·fs≠0.8·fs 的字体墙噪声（R876 谱系），散布正文多行；理想修须接 fontdue 真 ascent = font-metric 墙多会话。按 goal priority：DC-2~5 WPT oracle（+102）是主达标指标，DC-13 welcome 是 trend-only 字体墙（Makefile 明示「非回归」），本轮净大胜。

**裁决**：R953 LANDED。**纠正 R945–R952 八轮误归因**（vertical-align 簇 dominant = 默认字体 `<p>` 非存储路径定位，非 Ahem glyph）+ 落地 ≈+102 oracle 大胜。Ahem valign glyph 角度（embox-always / half_leading / run.y / v_offset）6+1 轮全 net-negative，**永久关闭**。**关键新认知**：R84 空-styles 墙有可修子症状（glyph 定位公式可凭行盒 half-leading 修正，无需打通全 style→paint 链路）——为 font-features/large-font 同类子症状提供新角度（区别 R945「须架构性 Path B 消除」全有或全无判断）。

**▶ 下会话**：① font-features（R513，shaper.rs:82）或 large-font（R101）借 R953 新认知——找 R84 空-styles 墙的**可修子症状**（glyph 定位 / 字号 / 行高可凭行盒度量修正而非打通全链路），区别 R945 全有或全无。② welcome/morning 真字体 ascent≈0.9·fs：paint 路径接 fontdue 真 ascent（per-font metric）解 R953 残余 +0.77pp + 字体墙主部（R374/R631/R876 多会话）。③ 扫下一默认字体 `<p>` 类 lever（css-text +60 提示该 pattern 高 leverage，R953 修了定位，或还有字号/行高类）。Ahem valign glyph / 公式级 vertical-align 勿再试（7 轮证）。

### R954 R953 landed 复核 + remaining lever 全 map + justify-all 实验 net-neutral 回退（零 net 源码·纯调查）

承 R953 ▶ 下会话三方向，数据驱动逐项复核。

**① R953 landed + binary 正确**：scoped reftest-oracle 实测 css-text **339/1826 (20.5%)** / css-fonts **79/282 (28.0%)** / linebox **119/190 (62.6%)**，全等于 R953 ON 值（css-text +60 / css-fonts +4；linebox 属 css2 谱系 R953 表未单列，119 = R953 后），证 release binary（target/release/zero-wpt-runner @ 11:52）含 R953 修复。

**② large-font (R101) 完全解决 → 方向 ① 排除**：100px 非 Ahem 探测（`<div style="font-size:100px;line-height:1">XH</div>`）product-smoke 渲染 + PIL 实测 glyph 垂直跨度 **73px**（X/H ink ≈ 0.73em，**正确非 16px**）；linebox scoped oracle 中 ifc-008/009（Ahem 100px）**不在 worst = oracle-pass**。机制：R105 `needs_dom_text_remeasure`（store_font_sizes 存真实字号）+ R355（pure Ahem 多行 inline_layout 存储）+ R953（非存储路径定位）三方已解字号 + 定位。R953 ▶ 下会话方向 ① 的 large-font 子症状**已无残留**。

**③ 数据驱动 worst 全 map（remaining measurable lever 全 trap/gated/结构性，无 single-session clean lever）**：
- **linebox worst 全 vertical-align stored-path**（117a 13.59% / 118a 13.26% / 122 20.81% / 004a 18.94% / 005a 19.83% / negative-leading 20.70% / sub/super 5.97%）——R945–R953 7 轮 net-negative trap，R953 声明「Ahem valign glyph 角度永久关闭」。
- **css-fonts worst 全 font-size-adjust CSS4 两值语法**（009/010/011 ~51% cap-height/ic-width/ic-height + @font-face）+ font-feature-resolution（shaper.rs:82，R513 Phase-A gated）——全须 per-font metric + @font-face（字体墙/Phase-A）。
- **css-text worst 全结构性/字体墙**：letter-spacing 元素边界（206 46.93% / 202 33.20%，min/max-content sizing 结构性）/ text-transform capitalize·fullwidth（@font-face DoulosSIL/Noto 字体墙）/ text-align justify 簇（justifyall-002/004/006 + justify-002，见 ④）。

**④ justify-all 解析实验（net-neutral 回退，R862 先例）**：`text-align: justify-all` 完全未解析（css-parser/style-system 0 命中，css-text 5+ worst 的直接相关簇）。先诊断 justify 实现状态——justify-002（justify 已解析 + Ahem）product-smoke 渲染 PIL 实测 word_gaps **[51,51,51] 均匀填满行宽**（4 词 51px 间隙，~450px 填满），证 **ZW justify 实现已工作**（非完全损坏）。遂实现 justify-all 解析（apply.rs 特判 `justify-all` → `text_align=Justify + text_align_last=Justify`）+ 3 单测（全绿）。**A/B css-text oracle-pass 339 不变（零 case 翻转）**，justifyall-002/004/006 diff 变化 +1.10/−1.54/+1.10pp **全在噪声内**——justify-002（不用 justify-all）亦变 −1.49pp 证噪声幅度 ~1.5pp，justify-all 真实影响 < 噪声 = **零确定 yield**。driving case（justifyall 簇）仍 FAIL，根因 = ZW justify **末行实现边界**与 chromium 不一致（justify-all 让末行也 justify，但末行词位偏）。按 R862 先例（premature：解析了但 driving reftest 仍 FAIL，blocked on 上游 justify 末行实现修正）+ code-guidelines「不做零价值修改」**回退全部实现**（apply.rs + 3 单测，零 net 源码，git status 干净）。

**⑤ 裁决**：本 rally 段 R953 LANDED 大胜（≈+102 oracle 纯渲染修复），R954 确认其 landed + 排除 large-font（方向 ①）+ 全 map remaining lever。remaining measurable lever 全 Phase-A gated（font-size-adjust font metric / font-feature shaper.rs:82 / @font-face 字体墙）/ stored-path trap（vertical-align 永久关闭）/ 结构性（letter-spacing 边界 / justify 末行）。**单 session clean measurable lever 确认穷尽**。

**▶ 下会话**：① justify 方向 = **justify-all 解析 + justify 末行实现对齐 chromium 须同改**（单 justify-all 解析 net-neutral 已证，R862 premature 回退）；先 PIL 诊断 justifyall-002 末行词位 vs chromium 差异（ZW 末行 justify 填满但词位偏），定位 justify 末行实现 bug 根因再同改。② 或转 rendering-compat 之外 zero-web 子目标。③ font-size-adjust / font-feature / @font-face / vertical-align stored-path 勿以单 session 试（Phase-A gated / 7 轮 trap）。

### R955 text-align-last 存储路径传递修复 LANDED = layout/paint 双路径一致性·probe 验证末行 justify·零 oracle yield（driving case 不在 corpus）·零回归·有 net 源码

承 R954 ▶ 下会话 justify 方向。诊断 justify-all net-neutral 真因时发现更根本 bug。

**① bug 根因（probe 实证）**：`compute_final_inline_layouts` 构建 stored IFC（存储路径，pure-Ahem 容器）**只传 `with_text_align` 漏传 `with_text_align_last`**（inline_finalization.rs 3 处 IFC 构建 + resolve_text_align_last helper 全缺）→ 存储路径末行 text-align-last 不应用（末行恒按 text-align 默认）。paint 非存储路径已传（text.rs:949）。probe `<div style="text-align:justify;text-align-last:justify;font:25px/1 Ahem;width:450px">TES×11</div>` PIL 实测：修复前末行 word_gaps `[26,26,176]`（3 词左对齐 + 边框伪影，right=303 未填满），修复后 `[114,113]`（3 词 2 间隙 ~113px 均匀填满 right=479≈450px）= **末行 justify 正确应用**。

**② 修复（inline_finalization.rs，纯加性）**：新增 `resolve_text_align_last` helper（TextAlignLastValue → `Option<TextAlign>`，Auto→None 其余映射同 resolve_text_align）+ 3 处 IFC 构建（主路径 line 631 / needs_dom_text_remeasure line ~1040 / multicol balance line ~1180）加 `with_text_align_last`。1 新单测 `test_resolve_text_align_last_mapping`（Auto/Justify/Right/Center/Left + None 全映射）。

**③ A/B（零 oracle yield + 零回归）**：css-text 339/1826 (20.5%) 不变 / linebox 119/190 (62.6%) 不变 = **零 case 翻转**。driving case 不在 corpus：css-text 的 text-align-last 用例（justify-last-default 等）全**非 Ahem**（走非存储路径，paint text.rs:949 已传 text_align_last，本就工作）；linebox **无** text-align-last 用例（0）。故存储路径修复的 driving case（Ahem + text-align-last）当前 corpus 无覆盖。

**④ 门禁全绿**：make test **12108 passed/0 failed**（R953 12107 + 1 新单测）；clippy `--workspace --all-targets -D warnings` 干净；fmt 干净；scoped layout-engine 1178 / engine 977 / style-system 1927 全绿。

**⑤ 裁决**：LANDED。**真实 layout/paint 双路径一致性 bug 修复**（goal Support Envelope「Layout/Paint IFC 一致性」明确要求）+ probe 验证技术正确 + 零回归。零 oracle yield 因 driving case 不在 corpus（Ahem + text-align-last 无 WPT 用例），非存储路径 text-align-last 本就工作。按 R946 先例（driving case 不在 corpus 的 correctness land）。**意义**：消除 layout/paint IFC 分歧（stored path 缺 text-align-last），与 R953（非存储路径定位修复）互补——R953 修非存储路径，R955 补齐存储路径属性传递。

**▶ 下会话**：① **justify-all 解析可重评 yield**——R955 已让存储路径 text-align-last 工作，justify-all（= text-align-last:justify）现存储路径也工作；R954 回退的 justify-all 解析重加后，justifyall 用例（Ahem 存储路径）末行现会 justify（之前 net-neutral 是因存储路径 text-align-last 不工作），可能 yield。② 或转 rendering-compat 之外 lever。③ font-size-adjust / font-feature / @font-face / vertical-align stored-path 勿以单 session 试（Phase-A gated / 7 轮 trap）。

### R956 justify-all apply 特判根因 = cascade 覆盖（fundamentally 不工作）·正确修 = declaration 注入·零 net 源码·纯调查

承 R955 ▶ ① 重评 justify-all。R955 让存储路径 text-align-last 工作后，重加 R954 回退的 apply.rs justify-all 特判（`justify-all` → `text_align=Justify + text_align_last=Justify`）+ A/B。

**① A/B 仍 net-neutral**：css-text oracle 339 不变；justify-all probe（`text-align:justify-all` + Ahem）PIL 实测末行 word_gaps 仍 `[26,26,176]`（左对齐，**未**像 text-align-last:justify probe 那样 `[114,113]` justify 填满）。

**② DBG 根因（cascade 覆盖 apply 副作用）**：apply.rs eprintln 证 apply 收到 `value="justify-all"` 且特判 `set text_align_last=Justify`（设成功）。但 resolve_text_align_last eprintln 证 layout 阶段读 `text_align_last=Auto`（4 元素全 Auto，0 Justify）。**设的 Justify 被覆盖回 Auto**。根因：**cascade 按 property 名独立处理**——`text-align-last` 是独立 inherited property，div.test 无 author `text-align-last` declaration（只有 `text-align: justify-all`），cascade 对 text-align-last 走「无 author declaration → 继承 parent (Auto)」，覆盖 apply_property_value 特判设的 Justify 副作用（cascade 不记录「text-align apply 时设了 text-align-last」）。故 **apply.rs 特判路径 fundamentally 不工作**（解释 R954 + 本轮 justify-all 双 net-neutral）。

**③ 正确修复方向 = declaration 注入**：在 css-parser 或 style-system declaration 收集层，把 `text-align: justify-all` **展开为两个 author declaration**（`text-align: justify` + `text-align-last: justify`），让 cascade 把两者都当 author declaration（justify-all 不再生效为 text-align-last 缺失 → 继承）。这需理解 css-parser → style-system declaration 数据流（非 apply 层单点）。

**裁决**：回退 apply.rs justify-all 特判 + 全部 debug（零 net 源码，git status 干净）。R955（text-align-last 存储路径）保留 LANDED（probe 验证工作，是独立 correctness 修复）。

**▶ 下会话**：① justify-all = **declaration 注入**（css-parser/style-system declaration 层展开 justify-all → 两 declaration，区别 apply 特判被 cascade 覆盖）；先定位 declaration 收集入口（css-parser rule 解析 → style-system cascade），加 justify-all 展开逻辑。② 或转 rendering-compat 之外 lever。③ font-size-adjust / font-feature / @font-face / vertical-align stored-path / apply 层 justify-all 特判 勿以单 session 试（Phase-A gated / 7 轮 trap / cascade 覆盖）。

### R957 justify-all declaration 注入 LANDED = matcher 收集层展开·probe + 真实测试证末行 justify·绕过 R956 cascade 覆盖·零回归·有 net 源码

承 R956 ▶ ① declaration 注入方向。R956 发现 apply.rs 特判被 cascade 覆盖（text-align-last 无 author declaration → 继承 parent Auto）。本轮在 declaration **收集层**实施展开，绕过 cascade 覆盖。

**① 实施（crates/style-system/src/matcher/mod.rs::collect_from_rules ~line 1066）**：每个匹配 declaration push 前，若 `property=="text-align" && value.trim()=="justify-all"`（大小写不敏感），展开为**两个 author declaration**（`text-align: justify` + `text-align-last: justify`，同 spec/layer/important）。这是真实热路径（collect_matching_declarations_with_media → collect_from_rules）；cascade.rs 的 collect_declarations 是死代码（仅测试调用），首轮误注入该处 0-effect 后定位到 matcher。

**② probe 验证（机制端到端工作）**：`text-align:justify-all` + Ahem probe（11 词 4/4/3）PIL 实测：修复前末行 `[26,26,176]`（左对齐），修复后末行 `[114,113]`（3 词 2 间隙 ~113px 均匀填满 right=479≈450px）+ 非末行 `[51,51,51]` justify = **justify-all 完整工作**（justify + 末行 justify）。

**③ 真实 WPT 测试验证**：justifyall-002（`text-align:justify-all` + Ahem）渲染 PIL 实测末行 `[121,121]`（3 词均匀填满）= justify-all 在真实测试端到端工作。

**④ A/B css-text oracle（机制工作 + 1 真改善 + 零回归）**：oracle-pass **339 不变**（零 case 翻转）。justifyall 簇：**004 20.78→18.20%（-2.58pp 真改善，超噪声 ~1.5）**，002/006 +0.50pp（噪声内，机制现匹配 chromium 末行 justify 的噪声级变化），justify-last-default 16.59% / justify-002 16.57% **不变**（不用 justify-all）。pass-count 不动因 justifyall 用例有 test/ref 结构差异主导 diff（.test 自动 justify vs .ref word-spacing 手动模拟 + rtl wrapper），非末行 justify 缺失单一原因。

**⑤ 门禁**：3 新单测（matcher/tests/advanced.rs：展开/大小写+trim/普通 justify 不展开）全绿；scoped style-system 1924 passed；clippy -D warnings 干净；fmt 干净；make test pending。

**裁决**：LANDED。**justify-all 之前完全不支持**（parse_text_align 不识别，apply 特判被 cascade 覆盖），现 matcher 展开 + R955 存储路径消费 → 端到端工作（probe + 真实测试双证）。零回归（339 不变 + scoped 全绿）。oracle pass-count 不动因 WPT justifyall 用例有 test/ref 结构差异主导，但机制 correctness + 1 真改善（004 -2.58pp）。**意义**：关闭 CSS Text 3 §7.1 justify-all 完整支持（与 R955 text-align-last 存储路径互补——R955 让 text-align-last 在存储路径工作，R957 让 justify-all 解析为 text-align-last:justify author declaration）。

**▶ 下会话**：① justify-all 现完整工作，可扫 css-text 其它 text-align 簇（text-align-end-last / text-align-last 子方向）或转下一个 clean lever。② font-size-adjust / font-feature / @font-face / vertical-align stored-path 勿以单 session 试（Phase-A gated / 7 轮 trap）。③ apply 层 justify-all 特判勿再试（R956 证 cascade 覆盖，R957 matcher 注入是正确路径）。

### R958 text-align start/end 方向感知 LANDED = direction:rtl 不再错误左对齐·4 driving case 各 −1.25~−2.55pp·零回归（CSS2 3509/css-text 339/wm 56 全不变）·有 net 源码

承 R957 ▶ ① 扫 css-text 其它 text-align 簇。诊断发现 `text-align: start/end` + `direction: rtl` 的真 correctness bug，落地修复。

**① bug 根因（CSS Text 3 §6.1）**：`start`/`end` 是**方向感知**值——`start` = inline-start 边（LTR→left, RTL→right），`end` 反之。旧实现三处无条件 `start→left / end→right`，致 `direction:rtl` + `text-align:start` 错误**左对齐**（应右对齐）。三处分叉：
- `resolve_text_align`（inline_finalization.rs:36，needs_dom_text_remeasure + multicol 调用）
- `compute_final_inline_layouts` 主存储路径**独立 inline match**（line 599，pure-Ahem 容器，**不调 resolve_text_align**）
- paint 非存储路径 inline match（text.rs:785）

**② 修复（统一方向感知，纯加性）**：
- `resolve_text_align` 加 `is_rtl = direction==Rtl`，start/end 按 is_rtl 翻转；`None` → Left（默认 Start 在 LTR = Left）。
- `resolve_text_align_last` 同样方向感知（start/end 翻转）。
- 主存储路径 line 599 inline match **替换为 `resolve_text_align(Some(style))`**——消除三处分叉，单点权威。
- paint text.rs inline match 加 is_rtl 翻转。
- 移除上轮遗留 debug eprintln（[DBG paint]/[DBG rta]）+ 清理 unused import（TextAlign）。

**③ 决定性 A/B（4 driving case，direction:rtl 在 .test 自身）**：
| case（.test 自身 direction:rtl） | OFF | ON | Δ |
|------|-----|-----|---|
| text-align-start-001 | 2.49% | 1.24% | **−1.25pp** |
| text-align-end-001 | 2.49% | 1.24% | **−1.25pp** |
| text-align-start-005 | 3.77% | 1.22% | **−2.55pp** |
| text-align-end-005 | 3.77% | 1.22% | **−2.55pp** |
全部改善（halved+）。residual ~1.2%（未越 1% threshold 翻 pass）= glyph 定位字体墙噪声（R953 谱系），非对齐 bug。

**④ 零回归（全 corpus 实证）**：oracle-pass ON=OFF——css-text 339/1826、CSS2 **3509/6283 (56.3%) 严格相等**、css-writing-modes 56。welcome product-smoke 16.88%（= R953 baseline，LTR 不受影响，<20% 阈值）。机制：LTR（~99% case）start→left 行为**完全不变**，仅 direction:rtl+start/end 子集翻转。

**⑤ 门禁全绿**：make test exit 0（12108+ passed/0 failed，59 ignored = real-website）；clippy `--workspace --all-targets -D warnings` 干净；fmt 干净；3 新/扩展单测（test_resolve_text_align_start_end_direction_aware + test_resolve_text_align_last_mapping 扩展 start/end LTR/RTL 全映射）全绿。

**裁决**：LANDED。**真 CSS correctness 修复**（CSS Text 3 §6.1，与 R955 text-align-last / R957 justify-all 同 text-align 谱系）+ 4 driving case 实测 −1.25~−2.55pp + 零回归（CSS2 最大 corpus 严格相等）。pass-count 不动因 driving case 残差 ~1.2% 仍越 1% threshold（glyph 字体墙），但**机制端到端验证**（A/B 4 case 全改善）+ 与 R946/R955 先例一致（correctness land 即使 driving case 不翻 pass）。**意义**：消除 text-align start/end 方向无关 bug + 统一 3 处 text-align 解析分叉为单点（resolve_text_align）。

**▶ 下会话**：① 继续扫 css-text text-align 簇残余（text-align-end-last / text-align-default/start/end 在 stored 路径其它 property 传递）或下一个 clean correctness lever。② driving case 残差 ~1.2% = glyph 字体墙（R953 谱系），勿单 session 攻（font-metric 多会话）。③ font-size-adjust / font-feature / @font-face / vertical-align stored-path 勿以单 session 试（Phase-A gated / 7 轮 trap）。

### R959 text-decoration-thickness 实验 = net-negative 回退（underline/overline 位置耦合 font-metric 墙）·css-text-decor 99→98 (-1)·零 net 源码·纯调查+实验

承 R958 ▶ ① 下一个 clean correctness lever。扫 css-text-decor worst（99/242=41%），发现 `text-decoration-thickness` 完全未实现（css-parser/style-system 0 命中），且是多 cluster 共通根因（dotted-001/002 13%、length-rounding 13%、underline/linethrough/overline-001 ~1.3%、ink-skip-dilation 6%）。实施完整 property（types/parse/apply/registry/inherit/paint 8 文件）+ 1 单测全绿。

**① A/B net-negative（css-text-decor oracle 99→98，-1）**：thickness 案逐案 OFF→ON：
| case | OFF | ON | Δ |
|------|-----|-----|---|
| underline-001 | 1.27% | 6.08% | **+4.81pp 回归** |
| overline-001 | 1.44% | 7.94% | **+6.50pp 回归** |
| linethrough-001 | 1.27% | 1.02% | −0.25pp 改善（居中） |
| scroll-001 | 3.68% | 2.22% | −1.46pp 改善 |
| ink-skip-dilation | 6.24% | 6.87% | +0.63pp 回归 |
| vertical-001/002 | 0.82/0.49 | 1.21/0.79 | 回归（越 1% threshold） |

**② 根因（位置耦合 = font-metric 墙）**：text-decoration 下划线/上划线垂直位置是**硬编码启发式**（`y_offset = font_size*0.15`，underline；`-font_size`，overline），paint 时 `add_fill(Rect::new(base_x, y, total_width, line_width))` 从 y **向下**生长。thickness 让 line_width 从 ~1.2px 变 80px（underline-001: 4em@20px），但位置仍锚在 baseline+0.15fs → 80px 厚线**长在错位**（test 期望厚下划线盖红盒，ZW 厚线长到红盒外）→ 更多红/绿错位 = 回归。**line-through 居中**（y_offset=-0.35fs，厚度对称生长）故改善；underline/overline 从边生长故回归。这是**补偿陷阱**（同 vertical-align 簇）：thickness 与 underline 位置耦合，单修 thickness 暴露位置 mismatch。

**③ 正修须耦合**：text-decoration-thickness 须**同修 underline 位置**（`text-underline-offset` + 字体 underline-position metric）才 net-positive。位置 = font-metric 墙（R374/R876 谱系，多会话）。单 thickness = dead lever。

**裁决**：回退全部 thickness 实现（8 文件 git checkout，零 net 源码，git status 干净，build 绿）。R959 = 一次有价值的 net-negative 实验性 rule-out（区别纯分析）+ 锁定 text-decoration-thickness **非 clean 单 session lever**（位置耦合 font-metric 墙）。本 rally 段（R958 LANDED + R959 rule-out）forward motion = R958 方向感知 win 已固化；text-decoration-thickness 加入 ruled-out。

**▶ 下会话**：① 继续找 clean correctness lever——避开 font-metric 耦合簇（text-decoration thickness/offset、font metric、vertical-align）。候选：css-text-decor 的 text-emphasis-position（6%）、text-decoration-color（6.89%，纯颜色非度量）；或 css2/css-position 几何/盒模型 case。② text-decoration-thickness 勿单 session 重试（位置耦合，须同修 underline position = font-metric 多会话）。③ font-size-adjust/font-feature/@font-face/vertical-align stored-path 勿试（Phase-A gated）。

### R960 css-position + css-tables 几何/盒模型 worst 扫描 = 全 structural/niche/font-coupled/已应用·clean single-session measurable lever 全 corpus 穷尽确认·flex-grow/shrink 已应用（R516d「R515 pending」stale 纠正）·零源码·纯调查

承 R959 ▶ 候选 css2/css-position/css-tables 几何/盒模型 case。ORACLE_DUMP_ALL worst 全扫两目录，**全候选 ruled out**。

**① css-position（54/97=55.7%）worst 全 ruled out**：
- `replaced-object-backdrop` 100% / `backdrop-inherit-rendered` 47.5% — backdrop-filter 栈（结构性，非几何）
- `position-absolute-semi-replaced-stretch-input/other` 21/13% — abspos 半替换元素（input）stretch（R325 谱系边缘）
- `position-absolute-dynamic-relayout-005/006` 11.7% — JS 动态 relayout（依赖 JS harness）
- `position-absolute/fixed-root-element-flex/grid` 4×4.05%（cluster，html 自身 positioned）— 根元素自身 inset sizing 未处理（`resolve_abspos_against_root_cb` 只处理根的**子**，engine.rs:391），但**高风险**（根 sizing load-bearing 整页）+ **niche**（html-positioned 罕见），4.05% 离 <1% 远，不追
- `position-relative-002/005` 4.89% — R952 已析（R109 + %inset 结构性）
- `collapsed-border-*-rtl-overflow` 簇 — vertical-rl/sideways-rl × rtl × border-collapse 溢出（writing-mode 交互，R114/R142 谱系）

**② css-tables（68/115=59.1%）worst 全 ruled out**：
- `table-cell-width-0` 20% — R97 intrinsic sizing（table 列宽算法，结构性）
- `percent-height-overflow-auto-…-cell` 17.3% / `percentages-grandchildren-quirks-mode-001/002` 14.9% — 表格百分比/quirks（结构性）
- `baseline-vertical` 12.5% — vertical baseline（字体度量/vertical writing-mode）
- `table-cell-overflow-explicit-height-001/002` 9.7%（cluster）— 表格 cell 显式 height + overflow（R168 height-as-minimum 的 inverse，cell overflow 语义结构性）
- `table-row-group-color-inheritance-001` 9.0% — row-group 直含文本的匿名 table 盒生成（R109 谱系结构性）

**③ 已应用确认（勿再查）**：`table-layout: fixed`（table.rs:1280）、`empty-cells: hide`（painter/mod.rs:449）、`border-collapse: collapse`（painter/border.rs:45）均 live。**flex-grow/shrink 已应用**（converter/mod.rs:199-200 `flex_grow/flex_shrink: style.flex_grow/flex_shrink as f32`）——R516d 文末「唯一未实现 genuine lever = R515 flex-grow/shrink pending code agent」**stale**（R524 已 refute 负值拒绝 slice + 本轮确认 grow/shrink 已 taffy-applied），纠正。

**④ 裁决**：clean single-session measurable lever **全 corpus 穷尽确认**（与 R519 code/doc 两 agent 独立收敛 + R945–R954 一致）。本 rally 段（R958 LANDED + R959 ruled out + R960 全扫描）forward motion = R958 方向感知 win 已固化；geometry/tables/text-decor 全扫无 clean lever。**剩余 forward-motion 仅 multi-session**：① font-metric 墙（R374/R876，释放 ~2200 fontdue/IFC near-threshold 案 + welcome/morning 残余，迄今最大 lever）；② text-emphasis greenfield feature（17 case，per-char emphasis glyph paint，须 spec-rfc 多会话）；③ Phase A IFC 统一 / multicol Phase 2（架构性，多会话）。本 round 零源码（纯调查 + stale 纠正）。

**▶ 下会话**：**承认 measurable plateau，转 multi-session 架构**——三选一（按 ROI）：① font-metric 墙 spec-rfc 启动（resolve_length 接 fontdue metric，advance/line-height/ascent 统一，最大 lever 但高风险多 net-negative 先例）；② text-emphasis 多会话 feature（17 case，先 spec-rfc 定 paint 路径再实现）；③ 若两者皆过大，转 rendering-compat 之外 zero-web 子目标。single-session clean lever 勿再扫（R519/R945–954/R960 四次穷尽确认）。positioned-root-sizing / table-cell-overflow / table 列宽算法 勿以单 session 试（structural / load-bearing 风险）。

### R961 高 diff（>30%）全目录扫描 + text-emphasis scope 核查 = 真 bug 候选全 out-of-scope/Phase-A/已知 net-negative·text-emphasis 确 multi-session（3 writing-mode + multi-path paint）·零源码·纯调查

承 R960 ▶ 转 multi-session 前最后核查：高 diff（>30%）case 更可能是真渲染 bug（缺特性/布局错）非字体噪声（~5-20%），或含可修 clean lever。ORACLE_DUMP_ALL 扫 css-position/css-tables/css-multicol/css2/CSS2 的 >30% case。

**① 高 diff 候选全 ruled out**：
- `replaced-object-backdrop` 100% / `backdrop-inherit-rendered` 47.5%（css-position）— backdrop-filter 栈（R894 已实现 base，edge case 结构性）
- `column-balancing-paged-001-print` 81% / `float-page-break-inside-avoid-*-print` 99%（CSS2 pagination）— **print media**（DC-12 可选/低优，非 reftest 杠杆，缺 print 引擎）
- `font-family-invalid-characters-003` 100% — CSS parser 对 `font-family: test{foo}` 非法字符 error recovery（niche robustness，非布局）
- `inline-svg-100-percent-in-body` 97.6% — **inline SVG**（goal Support Envelope 明确「inline SVG 不在本目标范围」，仅 SVG-as-img 在范围；此 case 应 skip-list 但属分母调整非渲染修复，goal 禁「缩小导入范围提高通过率」，不动）
- `before-after-table-parts-001` 93.4% — generated content ::before/::after + table（**R554 已证 net-negative 回退**，generated content 需 layout 独立 box）
- `bidi-008/008a` 62-73% — BiDi reordering（Phase-A same-source 墙）
- `multicol-rule-nested-balancing-004` 37.7% / `multicol-span-all-children-height-004a` 30.2% — multicol（Phase 2 结构性）

**② text-emphasis scope 核查 = 确 multi-session**：`text-emphasis: dot` + `text-emphasis-position: over/under` 跨 **horizontal-tb + vertical-rl + vertical-lr 三 writing-mode**（position-default-001 等用例）。vertical-rl/lr 的 mark 定位（text 右/左侧）= R114/R142 writing-mode 轴交互谱系（tricky）。paint 须在多路径（multicol line 1071 / main / vertical line 1214）的 per-char loop 注入 mark glyph。须先验：mark glyph 映射（filled dot=U+2022 等）、mark size、over/under offset（oracle PIL 分析）。非单 session clean slice。

**③ 现有 RFC 已覆盖 font-metric 墙**：`linebox-metric-unification-rfc.md`（R813）+ `phase-a-IFC-unification-design.md`（37KB）设计已产出；直接实现经 R945-R950（vertical-align 6 轮 trap）+ R959（thickness）证 net-negative。无须再写设计文档，须新角度实现（empirical per-case 或 Path B 消除）。

**④ 裁决**：本 round 高 diff 扫描 + text-emphasis scope 核查 = **第 5 次确认** clean single-session measurable lever 全 corpus 穷尽（R519/R945-954/R960/R961）。真 bug 候选全 out-of-scope（print/inline-svg）/ Phase-A（BiDi）/ 已知 net-negative（R554 generated content）/ niche（parser robustness）。**rendering-compat measurable 目标（DC-2~5）确认阻塞在 multi-session 架构**（font-metric 墙 / text-emphasis feature / Phase A IFC）。本 round 零源码（纯调查）。

**▶ 下会话**：**committed text-emphasis 多会话 feature 启动**（greenest，非 font-wall 耦合，17 case）。第一步 = oracle PIL 几何验证（mark glyph/size/over-under offset，避 R959 trap）+ horizontal-tb-only first slice（property plumbing: text-emphasis-style/color/position + shorthand，均 inherited；paint 仅 main 路径 per-char loop）+ A/B 守 zero 回归；vertical-rl/lr 留后续 slice。font-metric 墙 / Phase A / single-session clean lever 勿以单 session 重启（设计已存 / 5 次穷尽）。

### R962 text-emphasis 实施 = net-negative 回退（css-text-decor 99→91，-8）·mark glyph .notdef + size/offset 错·font-glyph-availability + 精确几何 forensics 前置·零 net 源码·实验回退

承 R961 ▶ committed text-emphasis。按计划实施完整 property plumbing（text-emphasis-style: none/Mark{filled,shape}/String + color + position over/under/right/left + shorthand，均 inherited，8 文件）+ horizontal-only paint（render_fragment! macro per-char loop，gate `text-emphasis-style != None` + `!char_advance_is_y`，mark glyph R961 验证映射 U+2022/U+25CF/...，font_size 渲染，over=frag_base_y-fs / under=+0.5fs）+ 单测全绿 + clippy/fmt clean。**gate 安全**：非 emphasis 渲染零影响（仅 style≠None 触发）。

**① A/B net-negative（css-text-decor oracle 99→91，-8）**：emphasis case **全无改善**（position-default-001 7.34% 不变 / style-filled-001 6.31→6.44% 略退 / position-auto 6.05% 不变），且 **8 case 从 PASS 翻 FAIL**（gate 仅 style≠None 触发，故 8 回归必为有 emphasis 的 case 被 mark 推过 1% threshold——mark 渲染错位添 diff）。

**② 根因（双重）**：
- **mark glyph .notdef**：U+2022(•)/U+25CF(●) 等在部分测试字体（Ahem / 某 CJK font）**无字形**→ fontdue 渲 `.notdef` 实心方块（大 diff）。chromium 有 fallback 字体含 mark glyph，ZW 单字体无 fallback。
- **size/offset 错**：mark font_size 渲染 + over=baseline-fs 启发式是猜（R961 PIL 分析噪声大未精确提取），chromium 实际 mark size/offset 不同 → 即使 glyph 有也错位。

**③ 前置（须先解，非单 session）**：① mark glyph 字体 fallback（fontdue 多字体查 glyph，或 mark 用内置几何形非字体字形）；② 精确 oracle PIL 几何 forensics（mark 像素 size + over/under 像素 offset，从 oracle shot 提取，非启发式猜）。两者皆多 session 工程。

**④ 裁决**：回退全部 emphasis 实现（8 文件 git checkout，零 net 源码，git status 干净，build 绿）。R962 = text-emphasis horizontal-first-slice **实证 net-negative**（-8），锁定须 **glyph-fallback + 精确几何 forensics 前置**（区别 R961「scope multi-session」模糊判断）。本 rally 段（R958 LANDED + R959/R962 ruled out + R960/R961 调查）text-decor vein 全闭（thickness R959 / emphasis R962 皆 net-negative，根因皆 font/glyph 度量）。

**▶ 下会话**：text-emphasis 勿以单 session 重试（须 glyph-fallback + 精确几何 forensics 前置，多 session）。**承认 rendering-compat measurable plateau**：single-session clean lever 6 次确认穷尽（R519/R945-954/R960/R961），text-decor vein（thickness/emphasis）net-negative 闭环。建议方向：① font-metric 墙 / glyph-fallback 多 session 架构（spec-rfc 已存 linebox-metric-rfc R813）；② 转 rendering-compat 之外 zero-web 子目标（如 zero-web goal 的其他 milestone）；③ product-smoke/infra 改进。font-metric/thickness/emphasis/vertical-align 勿单 session 重试。

### R963 font/glyph forensics = 纠正 R962 归因（mark geometry 非 .notdef）+ font-fallback 已工作（非 gap）·PIL mark 几何不可靠·零源码·纯调查

承 R962 ▶「glyph-fallback spec-rfc」。spec 前先做决定性 forensics（R962 假设的 mark .notdef 未验证）。临时探针（rasterize mark glyphs vs control on 默认字体，已删）+ harness 源码核查。

**① mark glyph 非 .notdef**：默认字体（DejaVuSans）rasterize @24px：bullet U+2022 = 8×8（real glyph，~0.33em）/ circle U+25CF = 19×19（real，~0.8em）/ control 'A' = 17×18。**mark glyph 存在**，R962「.notdef」假设证伪。

**② CJK fallback 已工作（非 gap）**：FontLoader 有完整 fallback 设施——`fallback_chain`（loader.rs:17）+ `set_fallback_chain`（:55）+ `rasterize_glyph_with_fallback`（:265，`has_glyph` 检查 :238）；wpt-runner harness **加载 NotoSansCJK 并 set_fallback_chain**（reftest.rs:1016/1036）；CPU/GPU renderer 均 `rasterize_glyph_with_fallback`（cpu/mod.rs:506，gpu/renderer/mod.rs:514/1244）。探针显示 '你' 在 DejaVu = .notdef（13×22 同 U+FE45），**但那是探针未设 chain**；harness 实际 CJK 经 NotoSansCJK fallback 渲染（emphasis case 非 100% diff 证 CJK 文本已渲）。

**③ R962 归因纠正**：R962 -8 回归根因 = **mark 几何（size/offset）**，**非** R962 记的「glyph .notdef + font-glyph-availability」。
- mark size：R962 用 font_size 渲 → bullet ~0.33em（探针实测），与 chromium 量级同（待精确验）。
- mark over offset：R962 用 `frag_base_y - fs`（启发式猜），chromium 实际 offset 不同 → mark 错位添 diff。
- PIL 精确 forensics **不可靠**（oracle mark band 碎片化：position-default-001 band y[28,50] 仅 ~22px 跨 mark+gap+text，无法干净分离 mark 像素）。

**④ 裁决**：forensic 纠正 R962 归因（mark geometry 非 .notdef）+ **font-fallback 已工作非 gap**（上轮 CONTINUE「font-fallback spec-rfc」premise moot）。text-emphasis 阻塞 = mark 几何精调（R959-trap 同类：须精确 oracle forensics，但 PIL 不可靠 → 须逐像素 manual 或单 case binary-search）。本 round 零源码（纯 forensics + 纠正）。

**▶ 下会话**：**承认 forensics 未解锁新 lever**——font-fallback 工作、mark glyph 存在、text-emphasis = mark 几何精调（PIL 不可靠，单 session 难）。rendering-compat measurable 仍结构性阻塞（font-metric 墙 / text-emphasis 几何 / multicol Phase 2 / BiDi）。务实方向：① 若续攻 text-emphasis，须先建立可靠 mark-forensics（单 case binary-search 渲 mark 于不同 offset，oracle A/B 找最佳，非 PIL）——多 session；② font-metric 墙（最大 lever，spec 已存）；③ 转 zero-web 其他子目标。font-fallback 勿再当 lever（已工作，R963 证）。text-emphasis 勿 PIL 估几何（不可靠）。

### R964–R965 渲染管线大文件拆分（2000 行规则）·table/engine 后处理组抽出·零行为变化·纯维护

承 R963 ▶ rendering-compat measurable plateau 已 6 次确认穷尽（R519/R945–954/R960/R961），text-decor vein（thickness R959 / emphasis R962）net-negative 闭环。**转 CLAUDE.md 规则 5（单文件 ≤2000 行）维护**——非 pass-rate lever，但属渲染管线可维护性，为后续 font-metric 墙多会话攻坚减小认知负担。两轮连续，零行为变化（纯函数移动 + glob 引入保持调用点不变）。

- **R964（commit 3ffa4021）**：`table.rs` 超 2000 行，grid 构建逻辑（`build_grid` 及辅助）抽出到 `table_grid.rs`（517 行）；table.rs 降至 1573 行。
- **R965（本轮）**：`engine.rs` 2291 行（超规则），将 taffy 后处理步骤 15 个自由函数（`adjust_inline_block_positions` / `sort_children_by_css_order` / `fix_vertical_mode_abs_pos` / `apply_relative_offsets` + `_inline` / `apply_calc_size_adjustments` / `exclude_floats_from_non_bfc_auto_height` / `backfill_r109_anon_block_heights` / `prevent_collapse_through_min_height` / `in_flow_content_extent` / `clamp_percentage_max_height` / `extract_calc_percentage_and_offset` / `resolve_relative_inset` / `apply_block_relative_percent_insets` / `convert_overflow_to_clip`）抽出到 `engine/postprocess.rs`（1109 行，与 R831 `engine/abspos.rs` 同模式 `mod X; use X::*;` + `pub(super) fn`）；engine.rs 降至 1221 行。**依赖核查**：自由函数零 `self.`/`LayoutEngine`/engine-private 引用；`establishes_bfc`（pub in margin_collapse）/ `resolve_text_align`+`store_font_sizes_from_ifc`（经 `use crate::inline_finalization::*;` glob）/ `crate::inline::` 全 fully-qualified 可达；清理 engine.rs 因此变 unused 的 4 import（AlignmentValue/FlexDirectionValue/OverflowValue/WhiteSpaceValue）+ postprocess.rs 多余 NodeKind import（fully-qualified `zero_dom::NodeKind::Element`）。

**验证（纯移动门禁）**：`cargo check --workspace --all-targets` 全绿；`cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo fmt --all --check` 干净（fmt 仅重排一处 `adjust_inline_block_positions` 签名——被移代码 pre-existing 行宽问题，rustfmt 语义中性）；`make test`（release + test-guard）**45 test-result-ok 块 / 0 FAILED / 0 error**；layout-engine 979/0 测试通过。**零渲染行为变化**（reftest/oracle 通过率不变，未复跑——纯函数移动 + glob 引入，调用点字节同）。

**意义**：CLAUDE.md 规则 5 合规（layout-engine 全部 .rs ≤2000 行）；engine.rs 主 `compute()` pass 与后处理步骤分离，降低后续 font-metric 墙 / Phase A IFC 统一攻坚的阅读负担。**非 measurable 进展**——rendering-compat pass-rate 仍结构性阻塞，measurable lever 仍 6 次确认穷尽。

**▶ 下会话**：measurable 目标仍仅多会话结构性可推进（font-metric 墙 = 最大 lever，spec 已存 linebox-metric-rfc R813 / phase-a-IFC-design；text-emphasis 几何须可靠 forensics；multicol Phase 2 / R109 / BiDi）。务实选项：① 启动 font-metric 墙多会话攻坚的新角度（empirical per-case 或 Path A/B 消除），接受单 session net-negative 先例（R834/R836/R849/R875/R959/R962）须极窄 slice + 三态门禁；② text-emphasis 单 case binary-search forensics（非 PIL）；③ 若 measurable 暂不可推，继续清偿技术债（其他 >2000 行文件 audit / 2000 行规则全仓复核）或转 zero-web 其他子目标。single-session clean measurable lever 勿再扫。

### R966 reftest.rs 拆分（2000 行规则）·harness 核心 3 子模块抽出·零行为变化·纯维护

承 R965 ▶ 继续 CLAUDE.md 规则 5（单文件 ≤2000 行）维护。`tests/wpt-runner/src/reftest.rs` 2273 行（rendering-compat **reftest harness 核心**），按职责拆 3 内聚子模块到既有 `reftest/` 子目录（与 `resources.rs` 同模式，R964-R965 `mod X; use X::*;` + `pub(super) fn`）：

- **`reftest/reftest_scripts.rs`**（128 行）= harness JS vein（R916-R927 谱系）：`apply_scripted_dom_mutations` / `extract_onload_handlers` / `fetch_external_script`（3 fn，私有→`pub(super) fn`，无外部调用方）。
- **`reftest/reftest_fonts.rs`**（148 行）= 字体栈构造：`create_font_loader` / `extract_font_faces` / `extract_inline_style_css` / `resolve_font_src` / `load_font_faces_into`（5 fn，私有→`pub(super) fn`，无外部调用方）。
- **`reftest/reftest_compare.rs`**（106 行）= 像素对比 + PNG/PPM I/O：`compare_pixels` / `compare_pixels_labeled` / `save_fb_as_png` / `save_framebuffer_png`（4 fn，原 `pub`→`pub fn`，reftest.rs `pub use` 重导出保持 `crate::reftest::compare_pixels` 等公共路径不变，main.rs 293/798/281 调用点零变化）。

reftest.rs 降至 **1914 行**（-359 net）；保留 types（Category/Result/Config/Case）+ run_reftest* + render_to_framebuffer* + render_via_webview* + merge_page_css + dump_layout_tree + render_image_into + 1065 行 `mod tests`。

**关键修正（子模块文件路径）**：reftest.rs 是 **文件模块**（非 `mod.rs` 目录模块），其 `mod X;` 声明的子模块须落 `reftest/X.rs`（与既有 `reftest/resources.rs` 同），初版误置 `src/reftest_*.rs` 触发 E0583，已 `mv` 进 `reftest/`。

**依赖核查 + import 清理**：移出代码零 `self.`/engine-private 引用；reftest.rs 清理因此变 unused 的 6 import（`std::char` / `CssRule` / `CssParser` / `PageScript` / zero_engine `{DomMutation, apply_mutations_to_html, extract_page_scripts, generate_js_dom_shim, register_dom_callbacks}` / `FontLoader`）；保留 `RenderPipeline`(19 用)/`simple_hash`/`render_full_scene`/`GlyphCache`/`{ImageCache,ImageData,ImageKey}`/`FuzzyMeta`。`save_framebuffer_png` 是 pre-existing dead public API（`#![allow(dead_code)]` 容忍），glob 重导出触发 unused_imports，加 `#[allow(unused_imports)]` 保公共 API surface 零变化（不删 pre-existing 死代码，遵 code-guidelines「精准修改」）。

**验证（纯移动门禁）**：`cargo check -p zero-wpt-runner --all-targets` 全绿；`cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo fmt --all --check`（fmt 仅重排 reftest_scripts.rs 一处 use 块——被移代码 pre-existing 行宽，rustfmt 语义中性）；`make test`（release + test-guard）**12112 passed / 0 failed / 73 ignored**（ignored = real_website_compat DC-7 例外）。**零渲染行为变化**（reftest/oracle 通过率不变，未复跑——纯函数移动 + glob/重导出，调用点字节同）。

**意义**：CLAUDE.md 规则 5 合规（reftest harness 核心 ≤2000 行，4 内聚子模块各 106-526 行）；harness 字体/脚本/对比三职责分离，降低后续 font-metric 墙 / Phase A IFC 统一攻坚 + harness JS vein 续作的阅读负担。**非 measurable 进展**——rendering-compat pass-rate 仍结构性阻塞，measurable lever 仍 6 次确认穷尽（R519/R945-954/R960/R961）。

**▶ 下会话**：measurable 仍仅多会话结构性可推（font-metric 墙 / text-emphasis 几何 / multicol Phase 2 / R109 / BiDi）。务实选项：① font-metric 墙新角度（极窄 slice + 三态门禁，接受 net-negative 先例）；② 其余 >2000 行文件续清偿（apps/browser app_input.rs 2910 / app_render.rs 2816 / main.rs 2346 / app.rs 2096——非 rendering-compat 核心，browser shell，可作纯维护或转 zero-web 子目标）；③ 转 zero-web 其他子目标。single-session clean measurable lever 勿再扫（6 次穷尽 + clean-lever 队列 R740 实质耗尽）。

### R967 app_render.rs 拆分（2000 行规则）·DC-10 图元消费层独立成文件·include! 模式·零行为变化·纯维护

承 R966 ▶ 继续 CLAUDE.md 规则 5 维护。`apps/browser/src/app_render.rs` 2816 行（browser 渲染层，含 DC-10 图元消费 + chrome UI）。**发现既有 `include!` 文本包含拆分模式**（app.rs:2088 `include!("app_render.rs")` + 既有 `app_render_geometry.rs` 同模式「从 app_render.rs 进一步拆分以控制单文件体积」）——文本包含→零可见性/隐私问题（self 字段/伴生方法/外部 crate 符号直接可达），比 R964-R966 的 `mod X; pub(super) fn` 更低风险。按此模式拆 2 内聚组：

- **`app_render_primitives.rs`**（501 行）= **DC-10 WebView 图元消费层**（rendering-compat 核心）：`primitives_content_height` / `append_webview_primitives` / `ViewportClip`+impl / `clip_axis_aligned_rect` / `clamp_rounded_rect_radii` / `clip_rect_field` / `path_vertices_bbox` / `transform_webview_primitives`（自由函数 + 1 struct，原 impl 块外，无 impl 包裹）。DC-10 scale_factor/offset/clip_y/clip_rounded/painting-order 全在此文件，后续 DC-10 逐项复核 audit 入口集中。
- **`app_render_address.rs`**（387 行）= 地址栏 UI：`render_address_bar` impl 方法（用 `Self::address_bar_page_kind`/`chars_slice`/`looks_like_search_query`/`tab_html_hint` 关联函数，include! 作用域内仍可达），包在 `impl BrowserApp { }` 内。

app_render.rs 降至 **1942 行**（-874 net）；app.rs 加 2 行 `include!`（geometry 之后、platform 之前）。**逐字 sed 提取**（`sed -n '2322,2816p'`/`sed -n '791,1169p'`）非手抄，零转录风险；删除自底向上（先 2322-2816 后 791-1169）保行号。`//!` 文件头改 `//`（include! 上下文内 doc-comment 位置非法 E0753）。

**验证（纯移动门禁）**：`cargo check -p zero-browser --all-targets` 全绿（brace 223/223 平衡，1 impl 块）；`cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo fmt --all --check` 干净；`make test`（release + test-guard）**12112 passed / 0 failed / 73 ignored**。**零渲染行为变化**（include! 文本包含，调用点字节同）。

**意义**：CLAUDE.md 规则 5 合规（app_render.rs ≤2000）；**DC-10 图元消费层独立成文件**——append_webview_primitives/ViewportClip/clip*/transform 全集中，DC-10 未勾选项（scale_factor+offset / clip_y+clip_rounded / painting order）的逐项 audit 入口清晰，降低后续 DC-10 验证关闭的阅读负担。**非 measurable 进展**——rendering-compat pass-rate 仍结构性阻塞。

**▶ 下会话**：measurable 仍仅多会话结构性可推。务实选项：① **DC-10 逐项 audit + 关闭**（scale_factor/offset/clip/painting-order 在新独立的 app_render_primitives.rs 内逐项核查 + 补测，关闭已实现的未勾 DC-10 项 = 真 DC 进展非 pass-rate）；② font-metric 墙新角度（极窄 slice + 三态门禁）；③ 剩余 >2000 行文件续清偿（app_input.rs 2910 / main.rs 2346 / app.rs 2096——browser shell 非 rendering-compat 核心）；④ 转 zero-web 其他子目标。single-session clean measurable lever 勿再扫。

### R968 DC-10 逐项 audit + 关闭·transform_webview_primitives 全 13 类型补测·3 项 DC-10 关闭（非 pass-rate，真 DC 进展）

承 R967 ▶「DC-10 逐项 audit + 关闭」。R967 把 DC-10 图元消费层独立到 `app_render_primitives.rs`，本轮在此文件内 audit 3 个未勾 DC-10 项 + 补测。

**① audit 结论（3 项均已实现，缺的是验证证据非实现）**：
- **DC-10 项2（scale_factor + offset 全 13 类型）**：`transform_webview_primitives`（生产全量消费路径，render_active_webview/main.rs 调用）对 13 类图元统一应用 `out = in*s + offset`——fills/rounded_rects（4 圆角独立）/gradients（Linear x0/y0/x1/y1、Radial cx/cy+内/外半径、Conic cx/cy 但 start_angle 无量纲不缩放）/shadows（rect+offset_x/y+blur+spread）/images（rect+clip）/strokes（4 端点+width）/path_fills/path_strokes（vertices+line_width）/glyphs（x/y+font_size）/clips/transforms（rect+origin+tx/ty，a/b/c/d 矩阵分量不缩放——正确，纯变换矩阵）/filters/blend_modes。**已实现 ✅**。
- **DC-10 项3（视口裁剪 clip_y + clip_rounded）**：`transform_webview_primitives` 用 `ViewportClip`（轴对齐矩形）裁剪全 13 类（rect 类走 `clip_rect_field`/`clip_axis_aligned_rect` 求交、path 走 `path_vertices_bbox`、glyph 走 font_size 包围盒、image 用 clip 字段交集）；`append_webview_primitives`（fills+glyphs 混入 chrome 层的细粒度路径）额外支持 `clip_y`（水平带）+ `clip_rounded`（圆角矩形，圆角 page frame 用）。**已实现 ✅**。
- **DC-10 项4（CSS painting order）**：R155 `draw_order` 是 painter 生产默认（painter/mod.rs:1459），paint 系统按 CSS painting order 发射图元，浏览器消费层保持发射顺序按类型重组不重排。**已实现 ✅（R155）**。

**② 补测（关闭项 2/3 的验证缺口）**：`transform_webview_primitives`（全 13 类生产路径）**此前零单测**（仅 2 类 `append_webview_primitives` 路径有测，且仅 1 个 cull 测试覆盖 transform 路径的 rounded_rect）。新增 2 单测（apps/browser/src/main.rs `mod tests`）：
- `transform_webview_primitives_applies_scale_and_offset_to_all_types`：构造 fills/rounded_rects/gradients(3 变体)/shadows/strokes/glyphs/transforms，scale=2 + offset=(10,20)，逐类型精确断言（fill origin(1,2)→(12,24) size(10,20)→(20,40)；圆角 1/2/3/4→2/4/6/8；Linear x0/y0/x1/y1 1/2/3/4→12/24/16/28；Radial cx/cy 1/2→12/24 inner/outer 3/5→6/10；Conic cx/cy→12/24 start_angle 0.5 不变；shadow offset 2/3→4/6 blur/spread 4/5→8/10；stroke 端点(1,2)-(3,4)→(12,24)-(16,28) width 5→10；glyph font_size 16→32；transform origin 1/2→12/24 tx/ty 3/4→6/8）。
- `transform_webview_primitives_culls_primitives_outside_viewport`：视口(0,0,200,200) 外（1000,1000）的 rounded_rects/gradients/path_fills/glyphs 全裁掉 + 视口内 fill 保留（control）。

**③ DC-10 关闭**：rendering-compat.md DC-10 项 2/3/4 全部 `[ ]`→`[x]`（项4 引 R155）。DC-10 现 4/4 全勾（项1 M7、项2/3 R968、项4 R155）。**DC-10 完成 ✅**。

**验证**：`cargo test -p zero-browser --bin zero-browser transform_webview_primitives` 3/3 ok（2 新 + 1 旧）；`cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo fmt --all --check` 干净；`make test`（release + test-guard）**12114 passed / 0 failed / 73 ignored**（+2 新测）。

**意义**：**DC-10（浏览器图元消费完整性）全部 4 项关闭 ✅**——13 类图元的 scale/offset/clip/painting-order 正确性现由单测守护，防回归。真 DC 进展（非 pass-rate，rendering-compat pass-rate 仍 font-metric 墙结构性阻塞）。零行为变化（纯测试新增 + 文档勾选）。

**▶ 下会话**：DC-10 已完成。rendering-compat measurable pass-rate 仍结构性阻塞（font-metric 墙 6 次 net-negative 先例 / R109 / multicol Phase 2 / BiDi）。务实选项：① **DC-14 self-source 路径补全**（oracle 路径三态已实现 R851/R852，self-source 严格容差复跑 + 非平凡性仍 pending——补全后 DC-14 全闭，真 DC 进展）；② font-metric 墙新角度（极窄 slice + 三态门禁）；③ 剩余 >2000 行文件续清偿（app_input.rs 2910 / main.rs 2346 / app.rs 2096——browser shell）；④ 转 zero-web 其他子目标。single-session clean measurable lever 勿再扫。

### R969 DC-14 self-source 三态分类落地·print_dc14_three_state·self-source 非平凡性仍 pending

承 R968 ▶「DC-14 self-source 路径补全」。DC-14 oracle 路径（R851 三态 + R852 非平凡性）已全实现；self-source 路径（默认 `reftest` / `reftest-upstream`）此前只有 loose 二元（pass/fail）报告，缺三态分解。本轮补 self-source 三态。

**① 实现 `print_dc14_three_state`**（main.rs，DC-14 helper，与 `frame_is_near_solid` 同区）：对每个 `(ReftestResult, ReftestCategory)` 分到三态——
- **真通过**：`passed && diff_ratio ≤ category.strict_max_diff_ratio() && max_channel_diff ≤ category.strict_max_channel_diff()`（布局 0.1%/ch2、文字 0.5%/ch5；唯一可信达标指标）
- **近似通过**：`passed` 但不满足严格容差（loose 通过但非严格，含同源假通过与字体噪声）
- **不一致**：`!passed`（loose 失败）

**② 自洽性**：`strict_pass + near_pass == pass_count`、`mismatch == fail_count`——near/mismatch 边界用 `result.passed`（编码实际有效 loose 阈值，含 ZERO_REFTEST_STRICT + per-test fuzzy override），而非 category.default_max_diff_ratio。首版误用 default loose 作 mismatch 边界致「不一致 34」与实际 Failed: 0 矛盾（部分用例 config/fuzzy 阈值高于 default），改用 `result.passed` 后自洽。strict 边界用 DC-14 锁定阈值，与 oracle 路径口径一致。

**③ 接线**：`cmd_reftest`（内置 css21）+ `cmd_reftest_upstream`（上游 corpus）均调用。`cmd_reftest` 的 category 取自 `configs[filtered[i].0].category`；`cmd_reftest_upstream` 取自 `FileReftestCase.category`（results 与 filtered 同序）。两路径在 pass/fail 报告后、JSON 输出前打印三态。

**④ 实测（内置 css21，make reftest）**：
```
── DC-14 self-source 三态分类（严格容差 = 唯一可信达标指标）──
真通过 (passed 且 ≤strict): 639 (93.1%)
近似通过 (passed 但 >strict): 47 (6.9%)
不一致 (failed):            0 (0.0%)
Passed: 686  Failed: 0
```
639+47=686=Passed 自洽。**诚实揭示**：loose 100% 含 6.9% (47 案) 字体噪声/近似通过——strict 真通过率 93.1% 是唯一可信达标指标。self-source 三态现与 oracle 三态（R851：strict/near/mismatch 同口径）对称。

**⑤ DC-14 进度**：三态分类 self-source + oracle 均落地 ✅；**残余 = self-source 非平凡性检查（test==ref 退化假绿）仍 pending**——须 frame 接入（ReftestResult 加 `near_solid` 字段 + run_reftest_with_base 调 `frame_is_near_solid`），是多构造点 struct 改动，留下轮 slice。oracle 路径非平凡性已实现（R852）。

**验证**：`cargo check`/`clippy --workspace --all-targets -- -D warnings`/`fmt --all --check` 全绿；`make reftest` 三态输出正确自洽；`make test`（release + test-guard）**12114 passed / 0 failed / 73 ignored**。零 pass/fail 行为变化（纯报告新增）。

**意义**：DC-14 self-source 三态补全 = self-source 路径现也报严格容差真通过率（不再被 loose 100% 掩盖字体噪声），与 oracle 路径对称。真 DC 进展（非 pass-rate）。**DC-14 现 6/7 项✅**（R669 独立 oracle / R969 self-source 三态 / R484 分母 / R660-R661 GPU 非passthrough / 内联 smoke 不计达标 / 容差锁定；pending: self-source 非平凡性 + 容差锁定文档项已满足）。

**▶ 下会话**：DC-14 残余 self-source 非平凡性（frame 接入，下轮 slice）；其余同 R968：font-metric 墙极窄 slice / 剩余 >2000 行 browser shell 文件 / zero-web 其他子目标。single-session clean measurable lever 勿再扫。

### R970 DC-14 self-source 非平凡性落地·frame_is_near_solid 共享 + test_near_solid 字段·DC-14 全闭

承 R969 ▶「DC-14 残余 self-source 非平凡性」。R969 落地 self-source 三态分类但非平凡性（test==ref 退化假绿）仍 pending；本轮补全 → DC-14 全闭。

**① `frame_is_near_solid` 提升为共享 helper**：原 `pub(crate) fn` 在 main.rs（仅 oracle 路径 R852 用），self-source 路径（reftest.rs）不可达。移到 `reftest/reftest_compare.rs` 作 `pub fn`（内聚：帧分析/退化检测，与 compare/save_fb 同组），reftest.rs `pub use` 重导出；main.rs 删原 def、oracle caller 改 `reftest::frame_is_near_solid`。两路径现共享同一实现。

**② `ReftestResult.test_near_solid: bool` 字段**：4 构造点全更新——`run_reftest_with_base` / `run_reftest_gpu_with_base` 正常返回处计算 `frame_is_near_solid(&test_fb)`；2 处 size-mismatch 早退设 `false`（失败非假绿关注）。

**③ `print_dc14_three_state` 升级 4 态 + 审计列表**：strict-pass 按非平凡性拆两态——
- **真通过-可信**：`passed && ≤strict && !test_near_solid`（唯一可信达标指标）
- **真通过-可疑**：`passed && ≤strict && test_near_solid`——test 帧近纯色，打印审计列表（前 20）供人工区分「理性近纯色简单页」vs「退化空白假绿」（历史 R135/R149 harness PNG 加载 bug 致空白假绿）
- 近似通过 / 不一致（同 R969）

自洽：`strict_credible + strict_suspicious + near_pass == pass_count`。

**④ 实测（内置 css21，make reftest）**：
```
真通过-可信 (passed 且 ≤strict 且非近纯色): 569 (82.9%)
真通过-可疑 (≤strict 但 test 近纯色，须审计): 70 (10.2%)
近似通过 (passed 但 >strict): 47 (6.9%)
不一致 (failed):            0 (0.0%)
```
569+70+47=686=Passed 自洽。**诚实揭示**：loose 100% → strict 93.1% → credible 82.9%——三道诚实关卡（loose→strict→非平凡性）逐层剥离假通过。70 可疑多为内置 smoke 理性近纯色（单色块/简单页），上游 corpus（复杂页近纯色=真退化空白）更有意义。机制 spec-correct（标记审计非自动拒绝）。

**⑤ DC-14 全闭 ✅**：oracle 路径（R669 独立 oracle + R851 三态 + R852 非平凡性）+ self-source 路径（R969 三态 + R970 非平凡性）+ R484 分母 + R660/R661 GPU 非passthrough + 容差锁定 + 内联 smoke 不计达标。**DC-14 7/7 项全 ✅**。

**验证**：`cargo check`/`clippy --workspace --all-targets -- -D warnings`/`fmt --all --check` 全绿（4 构造点全覆盖，无 missing-field）；`make reftest` 4 态输出正确自洽；`make test`（release + test-guard）**12114 passed / 0 failed / 73 ignored**。零 pass/fail 行为变化（test_near_solid 仅用于报告分类，不改 passed 判定）。

**意义**：**DC-14 全闭 ✅**——self-source 路径现与 oracle 路径对称（三态 + 非平凡性），reftest 通过率报告经三道诚实关卡（loose→strict→非平凡性），是 DC-2~5 pass-rate 目标「达标数字可信」的可信度前提。真 DC 进展（非 pass-rate；rendering-compat pass-rate 仍 font-metric 墙结构性阻塞）。

**▶ 下会话**：DC-10/DC-14 已完成。rendering-compat measurable pass-rate 仍结构性阻塞（font-metric 墙 6 次 net-negative 先例 / R109 / multicol Phase 2 / BiDi）。务实选项：① font-metric 墙新角度（极窄 slice + 三态门禁，现 DC-14 三态可作严格门禁）；② 剩余 >2000 行 browser shell 文件续清偿（app_input.rs 2910 / main.rs 2346 / app.rs 2096）；③ 转 zero-web 其他子目标。single-session clean measurable lever 勿再扫（6 次穷尽 + clean-lever 队列 R740 实质耗尽）。

### R971 app_input.rs 拆分（2000 行规则）·keyboard + context-menu 两组抽出·include! 模式·零行为变化·纯维护

承 R970 ▶「剩余 >2000 行 browser shell 文件续清偿」。`apps/browser/src/app_input.rs` 2910 行（browser 输入处理，`impl BrowserApp` 大块 + 2 自由函数 mod_prefix/permission_label），经 `app.rs:2084 include!("app_input.rs")` 文本包含（与 app_render.rs R967 同 include! 模式）。按内聚抽 2 连续块到新文件（各包 `impl BrowserApp { }`，与 app_render_address.rs R967 同）：

- **`app_input_keys.rs`**（650 行）= 键盘输入处理：`handle_key` / `handle_find_key` / `handle_address_bar_key` / `extract_typed_text` / `handle_global_key`（lines 315-958 连续块，5 方法）。
- **`app_input_context_menus.rs`**（587 行）= 右键上下文菜单动作：`show_context_menu` / `activate_context_menu_item` / `context_menu_menu_item_activatable`（lines 2298-2877 连续块，3 方法）。

app_input.rs 降至 **1686 行**（-1224 net）；app.rs 加 2 行 `include!`（input 之后）。逐字 sed 提取（`sed -n '315,958p'` / `sed -n '2298,2877p'`），删除自底向上（先 2298-2877 后 315-958）保行号。文件头 `//!`→`//`（include! 上下文内 doc-comment 位置非法，R967 同坑）。

**验证（纯移动门禁）**：`cargo check -p zero-browser --all-targets` 全绿（brace 325/325 平衡，1 impl 块）；`cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo fmt --all --check` 干净；`make test`（release + test-guard）**12114 passed / 0 failed / 73 ignored**。**零行为变化**（include! 文本包含，调用点字节同）。

**意义**：CLAUDE.md 规则 5 合规（app_input.rs ≤2000）；输入处理按 keyboard / context-menu 职责分离。**非 measurable 进展**——rendering-compat pass-rate 仍结构性阻塞。app_input.rs 是 browser shell（非 rendering-compat 核心），纯维护清债。

**剩余 >2000 行**：main.rs **2565**（R968-R970 加 DC-10 测试 + DC-14 三态/非平凡性 helper 致胀，pre-existing 2346 + 我加 ~219——下轮可拆 reftest 报告 helper / 测试模块）；app.rs 2104（include! 拼装中心）；taffy-local/flexbox.rs 2348（vendored 上游，跳过）。

**▶ 下会话**：measurable 仍 font-metric 墙结构性阻塞。务实选项：① **main.rs 拆分**（2565 行，含 R968-R970 我加的 reftest 报告 helper + DC-10 测试——拆 reftest 报告组到独立文件，清自己造成的遗留，rule 5）；② font-metric 墙极窄 slice（DC-14 三态作严格门禁）；③ 转 zero-web 其他子目标。single-session clean measurable lever 勿再扫。

### R972 apps/browser main.rs 测试模块抽出·mod tests; 到 src/tests.rs·main.rs 2565→685·零行为变化·纯维护

承 R971 ▶「main.rs 拆分」。`apps/browser/src/main.rs` 2565 行结构：~278 行 CLI/平台 helper + **`#[cfg(test)] mod tests { ... }` 1881 行（lines 281-2161）** + run_headless/browser_window_config/apply_window_chrome_action/... + `fn main()`（2296）。测试模块占 73%，是 R968 DC-10 测试 + 历史测试累积的主因。

**① 抽出测试模块到 `src/tests.rs`**：`sed -n '282,2160p'`（mod tests body）verbatim → `apps/browser/src/tests.rs`（1884 行，加文件头说明）；main.rs 用 `#[cfg(test)] #[allow(clippy::items_after_test_module)] mod tests;` 替代内联块（Rust 2018：`src/main.rs` 的 `mod tests;` 解析到 `src/tests.rs`）。**`super::*` 与 `super::browser_window_config()` 仍解析到 crate 根（main.rs）**——`tests` 仍是 crate 根的子模块，只是 body 落到独立文件，语义不变。

**② 修正**：首版用 python 写 decl 时 `\[` 转义误把字面 `\` 写进文件（`#\[cfg(test)\]`），cargo check 报错；改 Edit 直接写正确 `#[cfg(test)]`。

**验证（纯移动门禁）**：`cargo check -p zero-browser --all-targets` 全绿；`cargo clippy -p zero-browser --all-targets -- -D warnings` 零警告；`cargo fmt --all --check`（fmt 把 tests.rs 内 4-space 缩进的 body dedent 到顶层——语义中性）；`make test`（release + test-guard）**12114 passed / 0 failed / 73 ignored**（测试全过，mod tests; 解析正确）。**零行为变化**（test body 逐字移动，super:: 引用不变）。

**意义**：CLAUDE.md 规则 5 合规（apps/browser/src/main.rs 2565→**685 行**，tests.rs 1884 行——两文件均 ≤2000）；清 R968 DC-10 测试 + 历史测试累积造成的 main.rs 膨胀。**非 measurable 进展**——browser shell 维护清债，rendering-compat pass-rate 仍结构性阻塞。

**剩余 >2000 行**：app.rs **2112**（include! 拼装中心 + BrowserApp struct + 少量 fn——拆分空间有限，进一步拆须动 struct/字段）；taffy-local/flexbox.rs 2348（vendored 上游，跳过）。**非 vendored >2000 行文件基本清完**（layout-engine/engine/reftest/app_render/app_input/main.rs 全 ≤2000，仅 app.rs 2112 微超）。

**▶ 下会话**：非 vendored >2000 行文件基本清完（仅 app.rs 2112 微超，拆分空间有限）。rendering-compat measurable pass-rate 仍 font-metric 墙结构性阻塞。务实选项：① app.rs 2112 微超 audit（include! 拼装中心，可抽 BrowserApp struct + new 到独立文件？须核查字段隐私）；② font-metric 墙极窄 slice（DC-14 三态作严格门禁，接受 net-negative 先例）；③ 转 zero-web 其他子目标。single-session clean measurable lever 勿再扫。

### R973 app.rs 状态类型抽出·app_types.rs·app.rs 2112→1967·★ 非 vendored 全仓 .rs ≤2000 里程碑

承 R972 ▶「app.rs 2112 微超 audit」。`apps/browser/src/app.rs` 2112 行结构：imports + 小状态类型簇（lines 60-205：ContentPointerDrag / ScrollbarDrag / TabFetchState / WindowChromeAction / TabDragState / AutocompleteState+impl / ContextMenuState+impl，~146 行）+ `pub struct BrowserApp`（208-）+ `impl BrowserApp`（核心方法）+ 8 `include!`。状态类型簇是内聚可抽块。

**抽出 `app_types.rs`**（151 行）：`sed -n '60,205p'` verbatim → `apps/browser/src/app_types.rs`（加文件头说明）；app.rs 用 `include!("app_types.rs");` 替代内联块（line 60，imports 之后、BrowserApp struct 之前）。include! 文本包含→`pub enum WindowChromeAction` / `pub struct TabDragState`（main.rs `use app::WindowChromeAction` 用）路径不变，私有字段/impl 直接可达。

**验证（纯移动门禁）**：`cargo check -p zero-browser --all-targets` 全绿；`cargo clippy --workspace --all-targets -- -D warnings` 零警告；`cargo fmt --all --check` 干净；`make test`（release + test-guard）**12114 passed / 0 failed / 73 ignored**。**零行为变化**（include! 文本包含）。

**★ 里程碑**：**全仓非 vendored `.rs` 文件全部 ≤2000 行**——layout-engine（table/engine/reftest 系列）、apps/browser（app/app_render*/app_input*/main/tests）、tests/wpt-runner 均达标；唯一剩余 >2000 是 `crates/taffy-local/src/compute/flexbox.rs` 2348 行（vendored taffy 0.7.7 上游，拆分会偏离上游增未来升级成本，按规则跳过）。CLAUDE.md 规则 5 全仓合规（除 vendored）。

**意义**：CLAUDE.md 规则 5 全仓达标（非 vendored）；browser 状态类型按 data 结构分离。**非 measurable 进展**——rendering-compat pass-rate 仍结构性阻塞。连续 R964-R973 累计拆 7 个 >2000 文件（table/engine/reftest/app_render/app_input/main/app）。

**▶ 下会话**：★ **非 vendored 全仓 .rs ≤2000 已达标**——2000 行规则维护债出清。rendering-compat measurable pass-rate 仍 font-metric 墙结构性阻塞（6 次 net-negative 先例）。务实选项：① **font-metric 墙攻坚**（现 DC-14 三态+非平凡性作严格可信门禁，可量化；spec 已存 linebox-metric-rfc R813 / phase-a-IFC-design；须极窄 slice 接受 net-negative 风险）；② 转 zero-web 其他子目标（rendering-compat 单 session measurable 已结构性耗尽，DC-10/DC-14 已闭，余 DC-2~5/DC-13 阻塞 font 墙、DC-11/12 阻塞 host-layer）。single-session clean measurable lever 勿再扫。

### R974 css-position fresh oracle + semi-replaced-stretch 深挖 = form-control paint feature gap（非 layout lever）·plateau 第 7 次确认·零源码·纯调查

承 R973 ▶「font-metric 墙攻坚」。攻坚前做 R740 推荐「直接 chr-vs-ZeroWeb 简单用例对比」找新 high-confidence lever（区别盲信 doc 只读分析）。`make reftest-oracle DIR=css-position`：**54/97 = 56%**（R923 时 52.6%，R850/R962-era 略升）。top-15 worst 分类：

- **feature gap**：replaced-object-backdrop 100%（backdrop-filter on replaced）/ backdrop-inherit-rendered 47.5%（R520 同谱系 canvas 传播残余）
- **JS-driven**：position-absolute-dynamic-relayout-005/006 11.7%、hypothetical-dynamic-change-002/003 4.2%（dynamic relayout，须 harness JS 完整执行 + re-layout）
- **R109/font 已 ruled out**：position-relative-002/005 4.9%（R952 = R109+%/font_metric R915 closed）
- **semi-replaced-stretch pair 21%+14%**（input/other/button，csswg #6789）—— 最有希望 candidate，深挖：

**LAYOUT_DUMP（position-absolute-semi-replaced-stretch-input）**：input.abs **正确 stretch**——div.cb w=156 h=106（150/100 + 3px border×2），input.abs w=144 h=94（CB content 150/100 − inset 3×2），test/ref layout **字节同**。csswg #6789 semi-replaced abspos stretch 规则 **ZW 已正确实现**（layout 对）。**21% diff 全在 paint**——ZW 无原生 `<input>` form-control 渲染（button/submit/render as 非原生），ref 用 div 模拟；form-control painting 是多会话 feature 工程，非单 session clean lever。

**裁决**：semi-replaced-stretch = **form-control paint feature gap**（非 layout lever），勿以 layout 修复投。css-position worst 全落 feature gap / JS / R109-font / 已 ruled out 桶——**plateau 第 7 次确认**（R519/R740/R882/R945-954/R960/R961/R974）。zero net 源码（纯 oracle + LAYOUT_DUMP 调查）。

**▶ 下会话**：measurable lever 仍仅多会话架构可推（font-metric 墙 / form-control paint / R109 / multicol Phase 2 / BiDi / host-layer scroll）。**勿再扫 css-position worst**（已穷尽，全 feature gap/JS/R109）。务实选项：① font-metric 墙极窄 slice（DC-14 三态门禁，接受 net-negative）；② **转 zero-web 其他子目标**（rendering-compat 单 session measurable 结构性耗尽，连续 7 次确认；DC-10/14 已闭、规则 5 全仓达标——rendering-compat 可推进面已尽，须多会话架构突破）。single-session clean measurable lever 勿再扫。

### R975 RFC Phase 0 探针落地·vertical-align-117a·确证 layout TextBottom==Bottom 公式偏差 + paint-dead 死锁·建立 Phase 1 契约·零源码净变化

承 R974 ▶「font-metric 墙极窄 slice」。前 6 次 net-negative 先例（R834/R836/R849/R875/R959/R962）皆跳过 linebox-metric-rfc §4.3 的 **Phase 0 read-only 探针**（直接改码）——本轮执行该探针，建立 RFC 要求的「可验证契约」再决定是否改码。

**① 探针（env-gated `RFC_PHASE0=1`，`apply_vertical_alignment` 1651，run 后捕获即删）** 实测 117a span line：
- `line.height=130` strut_ascent=77 baseline_y=77 run.fs=40 **run.height=130**（span inline-box = inherited line-height，per §10.8.1 vertical-align 用 line-height box ✓）run.va=TextBottom
- **code 公式（1665）**：`Bottom | TextBottom => line_height - run.height` → run.y = 130-130 = **0**（span box bottom = 130 = **行盒底**）
- **spec text-bottom**：span box bottom 对齐**父 content-area 底**（= baseline_y + descent ≈ 77+8 = 85，**排除 half-leading**，区别 Bottom 用行盒底）→ spec run.y = 85-130 = **-45**

**② 确证 2 点**：
- **(a) layout 公式偏差 CONFIRMED**：code 把 TextBottom 折叠成 Bottom（1665 同 arm），spec 要求 TextBottom = 父 content-area 底（baseline_y + descent）。117a 偏 45px。
- **(b) paint-dead（R949 不同角度复核）**：R949 实证 vertical-align on/off = 117a 不变（13.59% 两边），**单改 (a) 公式无 yield**——paint Path A（stored Ahem 容器）不消费 run.y。Phase-A 双路径 deadlock 确证。

**③ Phase 1 契约（RFC §4.3 要求）**：yield 须 **(a)+(b) 同改**——(a) layout 拆 TextBottom arm = (baseline_y + descent) - run.height；(b) paint Path A 消费 stored run.y（当前 dead）。单 (a) = R949 no-yield；单 (b) 无正确 (a) = 错位。即 R876「三方同改非单点」。**117a 单 session slice 不可行**（paint-dead）。

**④ 裁决**：本轮 read-only 数据采集，探针加后删，**零 net 源码变化**（git 仅 evidence 报告 + master.md）。建立 Phase 0 契约供未来 Phase-A 统一用（linebox-metric-rfc §5 6-Phase 计划 / phase-a-IFC-design）。详见 [`evidence/r975-phase0-vertical-align-117a-probe.txt`](./evidence/r975-phase0-vertical-align-117a-probe.txt)。

**意义**：执行了 RFC 一直要求但 6 次 net-negative 跳过的 Phase 0 探针——**确证 deadlock 机制精确（layout 公式 + paint-dead 双因）+ 量化偏差（45px）**，为未来 Phase-A 攻坚提供契约（避免再次盲改 net-negative）。**非 measurable 进展**（纯调查，117a 通过率不变）。

**▶ 下会话**：font-metric 墙 deadlock 经 Phase 0 精确确证（须 (a) layout TextBottom 公式 + (b) paint Path A 消费 run.y 同改，多 session Phase-A 架构）。rendering-compat 单 session measurable 仍结构性耗尽（8 次确认）。务实选项：① **启动 Phase-A IFC 统一多会话攻坚**（linebox-metric-rfc §5 6-Phase，须 (a)+(b) 同改，单 session 不可行——跨会话推进，每会话一 Phase + 三态门禁）；② 转 zero-web 其他子目标（rendering-compat 单 session 已尽）。single-session clean measurable lever 勿再扫。

### R976 ★ CSS aspect-ratio 优先于替换元素固有比 LANDED = 打破 10 轮 measurable 僵局·nested-grid-item-block-size-001 64.36→13.76%·零回归·有 net 源码

承 R975 ▶「single-session clean measurable lever 勿再扫」——该结论针对 **vertical-align/font-metric 簇**（连续 8 次确认）。本轮转做 R740 推荐「直接 chr-vs-ZeroWeb 简单用例对比」找**该簇之外**的 lever：选 css-grid（stale 基线 + taffy-backed 故非 font 墙）跑 fresh oracle。

**fresh `make reftest-oracle DIR=css-grid`**：20/49 = 40.8%（与 stale 一致）。top-1 worst = **nested-grid-item-block-size-001 64.36%**（远高 #2 的 33.91%）——单点异常深挖。测试：`img{block-size:55vw;aspect-ratio:2/1}`（固有 8×16）在嵌套 grid 内；@800vw 期望 880×440。

**R976_PROBE 探针实测**（tree.rs `apply_replaced_element_sizing`，捕获后即删）：`css_h=Px(440)`（55vw converter 已正确解析）、`aspect_ratio=Some(2.0)`（CSS 已解析）、`intrinsic=(8,16)`——**computed 全对**。bug 在 `_` 分支（无 HTML attr、回退 img_intrinsic_sizes）：旧 `width = ch * w/h = 440*(8/16) = 220`（用**固有比** 0.5），spec（css-sizing-4 §4）要求 CSS aspect-ratio 优先 → `440 * 2 = 880`。差 4 倍 → 64% diff。

**修复（tree.rs `_` 分支）**：`let eff_ratio = computed.aspect_ratio.unwrap_or(w / h);` auto 侧按 eff_ratio 推导（width=ch*eff_ratio / height=cw/eff_ratio），对称两侧。HTML-attr 分支不受影响（其 auto 侧本交 taffy 按 aspect_ratio 推导，已与 CSS 一致）。**单测 2 个**（tests_10.rs，`compute_with_img_sizes` 注入 intrinsic）：height=440+ar=2→width≈880 / width=200+ar=2→height≈100，均 PASS（断言旧 bug 值 220/400 不再出现）。

**三态门禁验证**：`make test` **12116/0/73**（+2 新测）；clippy 零警告；fmt 干净；`make product-smoke` welcome **16.88%**（gate<20%，welcome 无 img/aspect-ratio 故 +0.77pp = font-墙噪声带非本修复）；`make reftest-oracle DIR=css-grid` 修后该案 **64.36→13.76%**（-50.6pp），css-grid 20→20 零回归；**全量 `make reftest-oracle`（10397 案）4528 = 44.6%**，跨 dir（flexbox 57%≥56.1 / grid 41%=基线 / tables 59%≥56.5）**零回归**，无 flex-aspect-ratio-img-* 进 top-15 worst。详见 [`evidence/r976-aspect-ratio-overrides-intrinsic-ratio.txt`](./evidence/r976-aspect-ratio-overrides-intrinsic-ratio.txt)。

**★ 机制纠正（R975 不完整结论）**：R975 Phase 0 测 `run.y` 称「paint-dead」并立契约「须 (a) run.y 公式 + (b) paint 消费 run.y 同改」。本轮代码核查发现该模型不完整——**vertical-align 实际活机制是 `frag_baseline_y`**（`compute_final_inline_layouts` line ~747-753，R822 Phase 3 已存 per-fragment valign-aware baseline：TextBottom→`line.baseline_y - half_leading` 等），paint Path A（text.rs:890 `baseline_y_abs = line.y + f.baseline_y`，:1344 v_offset 消费）**已消费** 该字段。`run.y`（apply_vertical_alignment 设）与 `frag_baseline_y` 是**两套独立参数化**——run.y 对 stored Ahem 容器确属冗余/弱化，但非「valign 全死」。故 R975「单改 run.y 无 yield」结论对，但「paint-dead 故须先消费 run.y」契约方向不准——真活 lever 是 `frag_baseline_y` 公式值（R950 half_leading 调整已试 net-negative）。**勿再以 run.y 为 vertical-align 起点重扫**；未来 Phase-A 须以 frag_baseline_y + fontdue 度量一致（R876 三方）为锚。另：`store_inline_layout_results`（inline_finalization.rs:117）是**死函数**（无调用方），实际存储走 compute_final 内联块——勿被其误导。

**意义**：打破 R964-R975 连续 10 轮「measurable 穷尽 / 纯维护拆分」僵局——**首个真 measurable 渲染正确性修复**（-50.6pp on css-grid top-1 worst），证明「fresh oracle + 直接 chr-vs-ZeroWeb 简单用例对比」方法论仍能找到 **font-墙之外** 的 clean lever（vertical-align/font-metric 簇 8 次确认仅约束其簇，非全 rendering-compat）。

**▶ 下会话**：① 继续用 fresh-oracle 方法论扫**其他 stale dir**找 font-墙外 clean lever——css-grid 命中证明值得系统扫。**优先 taffy-backed 布局 dir**（css-tables 59% / css-position 56% R974 已扫 / css-multicol）——`make reftest-oracle DIR=css/CSS2/linebox` 实测 119/190=62.6% 但 top worst **全被 vertical-align+line-height 簇垄断**（117a/118a/122/negative-leading/baseline-004a/005a/sub/super/line-height-128/applies-to-001..004，均 font-墙），**linebox 无新 clean lever，勿再扫**（唯一非-valign 小异常 border-padding-bleed-002 5.28% / inline-box-002 4.91%，可顺手查）；同理 css-text/css-fonts/css-writing-modes 均 font-coupled，预期无 clean lever。② nested-grid-item-block-size-001 残余 13.76% = 嵌套 grid 容器对 880px 溢出 img 的 track/overflow sizing（独立子问题，img 已对，非 aspect-ratio lever，可深挖或 defer）；css-grid 余 worst（replaced-element-percentage-height-in-grid-nested-in-flex 33.91% = flex→grid→1fr-track 百分比高度 CB 链 = R119/R168 结构性；table-grid-item-dynamic-003 25.78%；grid-container-baseline-synthesized-001..004 16-17% R926 双层结构性）多 taffy/structural，快速分类跳过。③ vertical-align/font-metric 墙仍多 session Phase-A（以 frag_baseline_y 为锚非 run.y）。single-session clean measurable lever **尚有空间**（fresh-oracle 系统扫 taffy-backed dir），勿轻言穷尽。

### R977 fresh-oracle 三 dir 系统扫 = css-tables/linebox/floats-clear 余 residual 全 border/font/structural·零 clean lever·1 concrete table-fixup bug 待 trace·零源码·纯调查

承 R976 下会话 ①「fresh-oracle 系统扫 taffy-backed dir」。本轮系统扫 3 dir + 深查 3 lever，**全部 non-clean**（区别 R976 aspect-ratio 的 4× sizing clean win）：

**① css-tables fresh oracle 68/115=59.1%（↑R922 56.5%，R976 aspect-ratio 修复 + harness 改善溢出）**。深查 top-2：
- **table-cell-width-0（20.09%）= 非 clean**：env-gated 探针实测 `compute_column_widths` 输出——**列宽全对**（width:0→intrinsic 8 / width:2px→max(2,8)=8 / width:20px→20，auto/100%/100px 表均正确填满，9 表 col 宽度逐一核对匹配 spec）。`position_cells` 亦 `cell_box.width=cell_width` 正确 resize。20% residual = border-collapse 2px 边精度 + 行高（ZW 行 ~14-20px vs chromium ~24px = 字体度量）+ 单元格文本 glyph = **border/font 耦合非 sizing bug**。pixel-band 分析证 ZW 表宽度大体对（auto 窄/100% 788 满/100px 104），非 table-cell-width-0 sizing 问题。
- **table-row-group-color-inheritance-001（8.99%）= concrete table-fixup bug 待 trace**：`<div display:table;color:red><div display:table-row-group;color:green;font:200px/1 Ahem>X` → chromium 渲 200px 绿方块（4489 px），ZW 仅 25 绿 px（**16px 默认**）。**font shorthand 本身对**（plain div `font:200px/1 Ahem` 实测 20000 绿 px = 200px ✓），故 bug = display:table-row-group 内裸文本「X」经匿名 cell fixup 时 **font-size:200px 未继承**（color:green 继承了，故非全坏；疑匿名 cell style 来源 / text-node font-size 解析点）。architectural（须 trace 匿名 table box style 继承链），单 session 未定位，**flag 供未来 trace**。
  - **★ R977 root cause 已定位（精确）**：`table_grid.rs:197-301` row-group 子节点循环**仅 match 元素子节点**（`is_table_row`/`is_row_group`/`is_table_cell`）。裸文本子节点「X」`get_display → None` → **无 branch match → text node 完全被跳过，不生成匿名 cell**（违反 CSS Tables §3.1：row-group 内裸文本须经匿名盒修复生成 row+cell）。ZW 文本以 parent LayoutBox 的 `text_node_*` map 表示（非 LayoutBox child），table_grid 仅遍历元素 LayoutBox child，故无法 wrap text node 成 cell——**架构层 fix**（须让 table_grid 识别 table-internal 容器的 text-node 内容并合成匿名 cell LayoutBox，或让 text-node 经由 row-group IFC 正确 paint）。影响面可能 > 1 案（任何「裸文本 in table/table-row-group/table-row」用例）。

**② CSS2/linebox fresh 119/190=62.6%·top worst 全 vertical-align+line-height 簇（font-墙）·无 clean lever**（R976 已记，本轮复核确认）。

**③ CSS2/floats-clear fresh 65/214=30.4%**。top worst = **margin-collapse-clear-012/013/014/015（22-33% 簇）= CSS2 §8.3.1+§9.5.1 clearance+adjoining-margin-collapse 边缘**（测试 assert：「clear 元素 top margin 与其 first in-flow child 的 adjoining margin 应正常 collapse」——taffy 0.7 CollapsibleMarginSet R323 覆盖基础但不建模 clearance-prevents/breaks-collapse 语义，结构性）；float-non-replaced-width-007 20.69% = float shrink-to-fit + inline-block max-width + Ahem 耦合（非单点）。

**结论（fresh 证据再证 doc 演进）**：R976 aspect-ratio 是**罕见** clean sizing lever（4× 误差 + 单行 fix）。css-tables/linebox/floats-clear 余 residual **全 border/font/structural/表fixup 架构耦合**——3 dir 系统扫零 clean sizing lever。勿再以「fresh-oracle 必中 clean lever」盲目扫已扫 dir；clean sizing lever 现已**确系稀缺**（R976 在 css-grid 命中是 stale 基线 + 4× 误差的偶然）。

**▶ 下会话**：① **trace table-row-group-color table-fixup font 继承 bug**（concrete，已 de-risk：font shorthand 对、plain div 对、bug 在 display:table-row-group 匿名 cell style 继承链；须读 layout tree 匿名 table box 创建处 + text-node font-size 解析——可能影响所有「裸文本 in table-internal」用例不止 1 案，潜在面广）；② 余 clean sizing lever 稀缺，measurable 推进转①或 font-墙 Phase-A 多 session；③ 勿再扫 css-tables/linebox/floats-clear（已 fresh 系统扫，non-clean）。详见 [`evidence/r977-fresh-oracle-sweep-tables-linebox-floats.txt`](./evidence/r977-fresh-oracle-sweep-tables-linebox-floats.txt)。

### R978 ★ table-internal 裸文本 IFC partial fix LANDED = table-row-group-color-inheritance-001 8.99%→0.79% PASS·零回归·有 net 源码

承 R977 下会话 ①「trace table-row-group-color table-fixup bug」。R977 已精确定位双重 orphan root cause，本轮实施 narrow partial fix。

**root cause（R977 定位）**：`<div display:table-row-group; font:200px/1 Ahem>X` 的「X」双重 orphan：(1) table_grid.rs:197-301 仅 match 元素子节点，text node `get_display→None` 跳过→不生成匿名 cell；(2) engine.rs:1007 is_block_level 不含 TableRowGroup→compute_final 在 inline_finalization.rs:520 早返→裸文本不跑 IFC。→ orphan 16px fallback。

**修复（inline_finalization.rs:520 is_block_level 门控）**：加 `is_table_internal_with_text` 窄例外——table-internal 行/行组 display（TableRowGroup/HeaderGroup/FooterGroup/Row）**且** `doc.child_nodes(node_id)` 含直接 Text 子节点时放行 compute_final 跑 IFC。正常 table（rows/cells 子元素，无直接 text）`is_table_internal_with_text=false` 仍早返，**零影响**。这是 CSS Tables §3.1 匿名 cell 生成的 **partial fix**（ZW 未实现真匿名 cell：text node 非 LayoutBox child）——让裸文本至少经 IFC 按容器 font/size 渲染（pure-Ahem 存 inline_layout，paint 正确渲染）。

**验证**：driving test table-row-group-color-inheritance-001 **8.99%→0.79% PASS**（ZW green px 25→20176，200px Ahem X 匹配 oracle）；css-tables dir 68→**69/115=60.0%**（↑R977 59.1%，table-row-group-color 出 worst 榜，余不变）；`make test` **12117/0/73**（+1 新单测）；clippy 零警告；fmt 干净；product-smoke welcome 16.88%（<20%，welcome 无 table 未受影响）；**全量 oracle 10397→4529=44.6%（净 +1，零回归）**，per-dir flexbox 284/57%（=）/grid 20/41%（=）/position 54/56%（=）/tables 69/60%（+1）/multicol 119/26%。详见 [`evidence/r978-table-fixup-bare-text-ifc.txt`](./evidence/r978-table-fixup-bare-text-ifc.txt)。

**单测**：`test_table_row_group_bare_text_runs_ifc`（table_layout_tests.rs）——断言 row-group 含直接 text 时 text_node_font_sizes 填入裸文本 node 且 font-size≈容器继承值（非 16 orphan）。用 text_node_font_sizes（font-无关 IFC-ran 信号）非 inline_layout（仅 pure-Ahem 存）。

**影响面**：pure-Ahem table-internal 容器裸文本现正确渲染（driving test 类型）。真匿名 cell 生成（CSS Tables §3.1 完整：text-node→anonymous row+cell LayoutBox）仍待 multi-session 架构（text-node-as-LayoutBox 或 table paint 消费 text_node map）；本 partial 覆盖 pure-Ahem 子集 + text_node_font_sizes 已填（非-Ahem Path B 可消费，未逐一验证）。

**意义**：R977 调查→R978 fix 跨会话接力闭环（1 轮）。连续 2 个 measurable 修复（R976 aspect-ratio + R978 table-fixup partial），fresh-oracle + 精确 root cause 定位方法论持续产出。table-internal 裸文本从「orphan 16px」→「正确渲染」是用户可见 correctness 提升。

**▶ 下会话**：① 真 CSS Tables §3.1 匿名 cell 生成（多 session 架构，本 R978 是 partial）——若需扩覆盖非-Ahem table-internal 裸文本；② **fresh-oracle 继续扫未扫 dir**（R976 css-grid 命中 aspect-ratio、R977 css-tables 命中 table-fixup——fresh 扫仍产出，继续 CSS2/generated-content 或 backgrounds 等）；③ font-墙 Phase-A 多 session 仍是大头（DC-2~5 达标主线）。single-session clean lever 仍有（R976/R978 连续 2 轮证明）。

### R979 CSS §14.2 画布传播 bg color 跳过 LANDED = background-root-007 51→41%（红消除）·零回归·有 net 源码·0 新 pass（font-墙阻挡簇）

承 R978 下会话 ②「fresh-oracle 继续扫未扫 dir」。扫 CSS2/generated-content（38.5%，全 ::before/::after + content = R554 reverted 伪元素 feature gap，无 clean lever）+ CSS2/backgrounds（67.3%）。

**backgrounds fresh oracle top worst = background-root-* 12 案簇（7-51%）**。pixel forensics 深挖 background-root-007（51%，`html:transparent; body{background:red url(support/square-white.png)}`）：**ZW 显红 10077 px，chromium 0 红**。`square-white.png` PNG color type 0（grayscale **opaque**）→ chromium 平铺全覆盖红底；ZW 显红 = body 盒在画布上绘了红 bg color。

**root cause**：CSS §14.2 根元素背景传播到画布时，元素**自身盒不再绘背景**（color + image 均被画布吸收）。ZW `paint_background_image`（effects.rs:69）已查 `canvas_propagated_node` 跳过自身 image，但 `paint_background`（mod.rs:1423，绘 bg **color**）**无此检查** → body（传播到画布时）仍绘自身 bg color，覆盖画布 tiled image。

**修复（painter/mod.rs paint_background 顶部加 guard）**：镜像 effects.rs:69 的 image-skip——传播元素自身盒不绘 bg color。仅影响 html/body 其中之一（传播方），正常元素零影响。

**验证**：background-root-007 **51.15%→40.98%**（红 10077→0；画布 tiled image 现可见）+ background-root-018 45.61%→40.28%；余 ~41% = Verdana `<p>` 绿文本度量（font-墙）；backgrounds dir 228/339=67.3%（**0 新 pass**——簇 font-墙阻挡）；`make test` **12118/0/73**（+1 新单测）；clippy/fmt 干净；product-smoke welcome 16.88%（未变）；**全量 oracle 10397→4529=44.6%（= R978 baseline，净 0 零回归）**。单测 `test_canvas_propagation_body_skips_own_bg_color`：`<body style=background:red>` 传播时断言恰好 1 red fill（画布）非 2（双绘）。详见 [`evidence/r979-canvas-propagation-bg-color-skip.txt`](./evidence/r979-canvas-propagation-bg-color-skip.txt)。

**意义 / 限制**：**真 correctness 修复**（body 盒不再错误覆盖画布 image，spec-conformant CSS §14.2），非零价值；但 **0 WPT 新 pass**（簇受 Verdana font-墙阻挡，measurable yield 弱于 R976/R978 的 +1 pass）。一旦 font-墙解，background-root 簇可随本 fix 转 PASS。**方法论**：fresh-oracle + pixel forensics 深挖「font-墙主导的簇」中的 **clean 子组件**——background-root 簇整体 font-墙，但其 canvas-propagation color absorption 是独立 clean bug。

**▶ 下会话**：① 继续 fresh-oracle 扫未扫 dir（CSS2/borders 78% / CSS2/css1 32% / css-multicol 26%）——R976 css-grid + R978 css-tables + R979 backgrounds 均 yield（前 2 +1 pass、R979 correctness），fresh 扫仍产出；② 余 clean lever 确系稀缺，font-墙主导的簇可深挖 clean 子组件（R979 模式）；③ font-墙 Phase-A 多 session 仍 DC-2~5 达标主线；④ 真 CSS Tables §3.1 匿名 cell（R978 partial 之上）待 multi-session。

### R980 fresh-oracle 4 dir 扫 + R717 decode-trap 独立实测确证 = borders/css1/visudet/values 全 non-clean·零 net 源码·纯调查

承 R979 下会话 ①「fresh-oracle 扫未扫 dir」。本轮系统扫 4 dir + 深查 R717 SVG intrinsic-size 簇，**全部 non-clean**。R717 decode-level fix 经独立实施 + 实测确证为 trap（与 R740 结论一致，新数据补强）。零 net 源码（decode fix 实施后回退）。

**① CSS2/borders（393/506=77.7%）全 structural/font**：border-width-applies-to-008（28%）= inline→block 映射（converter line 289 `Inline => taffy Block`，R370 territory）；border-conflict-style-107（22%）= table border-collapse §17.6.2；**border-width-shorthand-003（9%）REFTEST_DEBUG 实证 border-width 3-value shorthand 解析正确**（fill[0-3] = top 3 / right 10 / bottom 30 / left 10 全对，9% diff 全在 `<p>` 102 glyphs font-墙）；061/062/072/073 簇（8 案）= inline-block 定位（LAYOUT_DUMP 实证 height 288px 对，diff = inline-block 映射 block 堆叠非 side-by-side，R109）。★ borders 无 clean lever。

**② CSS2/css1（53/164=32.3%）全 font-wall/complex**：c5503-mrgn-b/c5501-mrgn-t（38%/36%）= margin in table-cell，Ahem 文本高度决定全布局（font-墙）；**c541-word-sp/c542-letter-sp（20%/16%）已实现**（text.rs:689 word_spacing / inline_finalization.rs letter_spacing）diff = Ahem glyph 定位；**c545-txttrans（12%）已实现**（helpers.rs:532）；**c547-indent（7.55%）已实现**（text.rs:821 / inline_finalization.rs:22）；c5525-fltmult LAYOUT_DUMP 实证 four float div 正确 side-by-side（x=8/168/328/488 w=140 ✓），diff = table 截面 td（&nbsp;）h=0 + float 文本。★ css1 无 clean lever（全已实现属性 + font-墙）。

**③ CSS2/visudet（17/38=44.7%）= R717 簇 + font-墙**：replaced-elements-* 9 案（9-32%）= R717（见下）；inline-block-baseline-015（13%）= R109/font；content-height-001..004（3-5%）= @font-face 自定义字体 line-height（font-墙）。

**④ CSS2/values（17/26=70.8%）= ex 单位 + font**：units-003（23%）= `height:0.25ex`（R544/R547 谱系，须 font-metric x-height）；units-002（16%）= ex 单位（R512/R547）。

**R717 独立实施 + 实测 trap 确证（补强 R740）**：实施 decode_svg_bytes 加 compute_svg_intrinsic_size（解析 SVG width/height/viewBox，按 CSS/SVG2 推导缺失侧：height+viewBox → w=h×vb_w/vb_h；width+viewBox → h=w×vb_h/vb_w），4 单测全绿。**实测 visudet oracle**：5 all-auto/min- 案 **反退 +52pp**（all-auto 31.66→83.63% / min-height-20 31.66→83.63% / min-width-40 31.66→83.63% / min-height-40 31.34→71.98% / min-width-80 30.22→70.07%），4 explicit-dim 案改善 ~-1pp（width-40 9.09→8.20 等），**NET 严重负**，decode fix 已回退。**trap 精确机制**：chromium 对 partial-attr SVG（缺一侧 attr）`<img>` = CSS 默认 **300×150**（实测 oracle fuchsia/blue 均 300×150，非 attr-推导）；decode fix 使 img2（h=25 viewBox 1000×500）从 usvg 1000×25 → 50×25（ratio 2 正确），all-auto 50×25 与 chr 300×150 差更大 + 与未修 img6=1000×500/img7=100×100 混杂致 inline-wrap 紊乱 → 83%；explicit-dim（width:40）50×25 ratio 推 40×20 正确 → 改善。**根本矛盾**：all-auto 需 300×150 默认 / explicit-dim 需 viewBox ratio 推导，image_sizes (w,h) 编码无法区分「真固有」（PNG/both-attr-SVG，all-auto 用之）vs「ratio-only 推导」（partial-attr-SVG，all-auto 应 300×150）；无 viewBox 的 img4/5/7 任何 (w,h) 均造假 ratio。**R717 须 ratio 独立信号（image_sizes Option + 并行 image_ratios map）+ tree.rs §10.3.2 消费 + decode 区分 both-attr 真固有 vs partial/no-attr 默认，跨 4 crate（render-foundation/harness/engine/layout-engine）多 session**（R740 已 flagged；risk = 真实 SVG logo 仅 viewBox 无 attr 的 all-auto 会变 300×150，须 product-smoke A/B）。★ **勿再以 decode-level 单点 fix 重试 R717**（R740 + R980 两轮独立确证）。

**意义**：4 dir fresh-oracle 扫描 + R717 decode 实测，**全部 non-clean**。clean single-session lever 在 CSS2 已扫 dir 内确系耗尽（borders/css1/visudet/values 全 font-墙/structural/R717-trap）。fresh-oracle 方法论仍有效（R976/R978/R979 三连）但须转 **未扫 dir**（normal-flow/positioning/tables 大 dir 未本轮覆盖）/ **font-墙簇内 clean 子组件**（R979 模式）/ **多 session 架构**（R717 ratio-signal / Phase-A IFC / 真 CSS Tables §3.1）。本轮独立确证 R717 decode-trap 节省未来轮次重试成本。详见 [`evidence/r980-r717-decode-trap-confirmed-4dir-scan.txt`](./evidence/r980-r717-decode-trap-confirmed-4dir-scan.txt)。

**▶ 下会话**：① fresh-oracle 扫 **未扫大 dir**（positioning 578 / tables 1139 / selectors 618——taffy-backed，R976/R978 命中证明值得扫；normal-flow 本轮已扫 = inline-replaced-width 簇 XML 命名空间 trap + font/structural，无 clean lever）；② R717 full ratio-signal fix（多 session，4 crate，须 product-smoke A/B）；③ font-墙 Phase-A 多 session（DC-2~5 主线）；④ 真 CSS Tables §3.1 匿名 cell（R978 partial 之上）。clean single-session lever 在 CSS2 已扫 5 dir（borders/css1/visudet/values/normal-flow）耗尽，未扫 tables/selectors/positioning 仍有可能（R976 css-grid 命中先例）但概率下降。

**追加（normal-flow 扫描）**：CSS2/normal-flow（599/746=80.3%）top worst = inline-replaced-width-* / inline-block-replaced-width-* 簇（8+ 案 10-22%）。深查 inline-block-replaced-width-008（22%，`<svg:svg height="300">` + 绿 div overlap）：probe 实证 WPT XHTML `<svg:svg>`（命名空间前缀）经 html5ever **HTML 模式**解析 → local_name 字面 "svg:svg"（ns=xhtml 非 SVG ns）→ CSS 选择器 `svg#overlapped-red{display:inline-block}` **不匹配**（type `svg`≠"svg:svg"）→ computed display=Inline（非 inline-block）+ apply_replaced_element_sizing `tag=="svg"` 不匹配 → sizing 跳过。实施 svg sizing fix（加 "svg:svg" 匹配 + 300/150 默认）实测 svg width 600→300 但 height 仍 0（display=Inline 被 inline-only-remeasure 覆盖；且 Inline vs inline-block 布局行为不同）。**fix 已回退**。★ inline-replaced-width 簇 = **XML 命名空间解析 trap**（须 parser strip 前缀或 XML 模式；配合 goal line 118 内联 SVG out of scope），非 clean 单 session lever。min-height-106（19%）= float+min-height+Ahem font-墙。normal-flow 无新 clean lever（R695/R699 LANDED 区残余 font/structural）。详见 evidence r980 追加段。**5 dir 扫描全 non-clean，两类 trap 独立确证（R717 decode-ratio-encoding + XML 命名空间）**。

### R981 fresh-oracle 3 dir 系统扫（positioning/tables/flexbox）= 零 clean lever·abspos auto-margin de-risk·flex transferred-size 头号未来 lever·零源码·纯调查

承 R980 下会话 ①「fresh-oracle 扫未扫大 dir（positioning/tables/selectors，taffy-backed）」。本轮系统扫 3 dir + 深查 5 lever，**零 net 源码**（全部落 font-墙/structural/复杂 spec 算法缺口）。闭 R980 open item ①。

**① CSS2/positioning fresh 296/520=56.9%**：top worst 全 font-墙/XML-namespace/shrink-to-fit/abspos-auto-margin。abspos-008（17.5%）= `font-size:100px` large-font 墙；top/bottom-091/092（16%）= `6ex` ex-font-metric 墙（R544/R547 残余）；absolute-replaced-width-039/053/067（7.66%）= `<svg:svg>` XML 命名空间 trap（R980 ruled out）；absolute-non-replaced-width-021..024（9.73% 4 案）= shrink-to-fit+rtl（§10.3.7 taffy-blocked R180/R370）。无 R976 式 standout（top-1 17.5% vs R976 64%）。

**★ abspos auto-margin de-risk（§10.6.4/§10.3.7，0-yield）**：height-003/004/005 测 abspos top+bottom fixed+height fixed+margin auto → 居中。abspos.rs::adjust_absolute_pct_to_viewport（line 113）auto-margin handler（line 180-247）**仅覆盖无 positioned ancestor 的 abspos（CB=视口）+ horizontal max-width-clamp 特例**；height-003/004/005 CB=positioned ancestor（`#div1 relative`）走 taffy 不 solve auto。probe 实证 margin-top:auto→0：height-005（margin-top explicit 0.5in）= 1.90% = `<p>` font-墙+正位；height-004（margin-top auto）= 7.60% = 错位。修后 003/004 仍卡 ~1.9%（`<p>` font-墙）= **0 新 pass**（须新 post-process pass + abspos 回归风险，yield 0 不投）。待 font-墙解后该簇可批量翻 PASS。

**② CSS2/tables fresh 48/364=13.2%**（远低 css-tables module 59%——经典 CSS2.1 suite 难）：collapsing-border-model-010b 44%（border-collapse §17.6.2 结构性）/ table-backgrounds-bs/bc-{table,colgroup,rowgroup}-001 簇（§17.5.1 列背景层架构：painter is_table_internal 含 row/rowgroup 但**不含 col/colgroup**，列背景须层架构多 session）/ table-anonymous-objects-214 = `200px Ahem` large-font。**near-pass 128 案 1-2% 全 font-墙**（ORACLE_DUMP_ALL 实证：table-anonymous-objects 50+ 案 ~1.05% + fixed-table-layout-003a/d 24 案 1.04-1.41% 共享 `<p>`+serif/monospace 文本；caption-side-applies-to-008..014 = caption-side:bottom **已实现** table.rs:1298-1306 + 测 caption-side 在 row-group 不继承 + font-墙）。R978 table-fixup 是 tables 已发现唯一 clean lever，余结构性/font-墙。

**③ css-flexbox fresh 284/497=57.0%**：★ top-1 standout **flex-minimum-width-flex-items-013 82.04%**（远高 #2 33.65%，R976 式信号）但深查 = **csswg #5663 transferred-size-suggestion 算法缺口**（非单点）：`<img 300x150 width:999>` in `flex width:0 height:50`，预期 img=100px（min-width:auto = stretched cross 50 × 固有比 2 = 100）。converter（mod.rs:96-104）传 aspect_ratio+min_size 给 taffy 0.7 但**taffy 0.7 未实现 #5663**。ZW 82% = img 未 clamp 到 transferred min。= flex §4.5 算法实现（post-taffy flex-item min-size，chicken-and-egg w/ cross stretch，多 session 类 R717）。top-15 余多 aspect-ratio 簇（007/003/004/011/014 15-33% = transferred-size 同谱系，区别 R976 css-grid 的 single-line aspect-ratio-overrides）。

**综合**：3 dir 零 clean single-session measurable lever。clean sizing lever 确系稀缺（R976/R978/R978 三连是罕见命中）。CSS2+css-flexbox 已扫 dir clean lever 耗尽（10+ 次确认）。详见 [`evidence/r981-3dir-scan-positioning-tables-flexbox.txt`](./evidence/r981-3dir-scan-positioning-tables-flexbox.txt)。

**▶ 下会话（全多 session 架构，按 rally 协议续跑）**：① **flex transferred-size-suggestion（csswg #5663）多 session 攻坚**——flexbox top-1 standout 82% + aspect-ratio 簇，须 post-taffy flex-item min-size:auto 算法（§4.5），头号未来 lever（先 RFC + Phase 0 probe 量 ZW 当前 img width 再改码）；② Phase-A IFC 度量统一（font-墙，DC-2~5 主线，R975 frag_baseline_y 锚，6 次 net-negative 先例）；③ R717 full ratio-signal（4 crate，R980 decode-trap ruled out）；④ 真 CSS Tables §3.1 匿名 cell + 列背景层（§17.5.1）；⑤ abspos auto-margin 待 font-墙解后批量 yield。single-session clean measurable lever 勿再扫（10+ 次确认）。

### R982 ★ flex transferred-size-suggestion §4.5/csswg #5663 (row-only) LANDED = flex-minimum-width-flex-items-013 82→10%·零回归·net-neutral correctness

承 R981 头号未来 lever「flex transferred-size-suggestion」。本轮 Phase 0 root-cause → 实施 → de-risk → **land（row-only）**。**首个 flexbox measurable correctness 修复**（打破 R964-R981 调查/拆分为主的僵局），证明 §4.5 transferred-size 可在 ZW post-taffy 层正确实现。

**Phase 0 root cause（probe A/B/C/D 隔离）**：
- 非替换 flex item flex-shrink 生效（width:999→100，probe A）；替换（img）flex item **不收缩**（999 溢出，probe C）。
- 根因：taffy 0.7 把 leaf 替换 flex item 的 min-size:auto 当作其 definite 主尺寸（width:999→min 999），floor 在高值致 flex-shrink 无效。
- probe D 证：img + min-width:0 → 正确收缩到 100（确认 min-width:auto 是 floor，非 taffy 不 shrink）。

**修复（apply_flex_transferred_min_size，tree.rs）**：apply_replaced_element_sizing 加 styles 参 + 调新 helper。替换 flex item + 父 flex 容器有明确 cross size(Px) + 子有 aspect_ratio + 水平书写模式 → 计算 transferred main = cross × ratio（§4.5），设 taffy min_size.main = min(intrinsic_main, transferred, specified_main)。driving case：cross(height)=50, ratio=2 → transferred=100，img 999→100。

**De-risk（A/B css-flexbox oracle，3 版迭代，关键）**：
- v1（row+column）：3 回归（auto-margins-002 + flex-aspect-ratio-img-column-012 + flex-minimum-height-flex-items-007）/ 0 改善。
- v2：加 §4.5 守卫（auto cross-margin 跳过 + align-self 非 Auto/Stretch 跳过）→ 恢复 auto-margins-002（margin:auto 不拉伸，cross 非 definite）。
- v3：row-only（column 轴 cross-stretch 语义在 column-012/minimum-height-007 致回归，待多 session 验证）→ **0 回归 / 0 改善 / driving 82→10%**。
- 净 net-neutral correctness（同 R979 模式）：driving 残余 10% = `<p>` 指令 + 绝对定位红块 font-墙（img 已正确 100×100 绿），故 0 新 pass（font-墙阻挡簇）。

**限制（诚实）**：① 0 新 WPT pass（driving 残余 10% = font-墙）；② column transferred 待多 session（row-only 保零回归）；③ font-墙解后该簇（flex-minimum-width-flex-items-* + aspect-ratio-intrinsic-size-*）可批量翻 PASS。

**验证**：make test **12120/0/73**（+2 新测：test_flex_transferred_min_size_from_stretched_cross + _not_applied_to_non_flex_parent）；clippy 零警告；fmt 干净；css-flexbox oracle A/B **0 回归**；product-smoke welcome **16.88%**（=基线 <20%）。tree.rs 1050 / 测试 391 行（均 ≤2000）。详见 commit 814ddcbc。

**▶ 下会话**：① **column-direction transferred-size**（row-only 之上的对称补全，须先 trace flex-aspect-ratio-img-column-012 / flex-minimum-height-flex-items-007 为何 column 轴回归——column 的 cross-stretch 语义/明确 cross size 判定可能与 row 不同）；② driving test 残余 10% 拆解（img 已 100×100，10% 是 `<p>` font-墙 + 红块绝对定位——font-墙 Phase-A 解后可 PASS）；③ 其他 flexbox 簇（aspect-ratio-intrinsic-size-007 33% / flexbox-flex-flow-001/002 23% = flex-flow 简写展开待查 / flex-abspos-inset-* 19% = R98/R500 谱系）；④ Phase-A font-墙（DC-2~5 主线，最大杠杆）。

### R983 ★ flex transferred-size column + content-box LANDED = net +1 pass·零回归·row+column 对称

承 R982 ▶ ①「column-direction transferred-size」。R982 row-only 是 net-neutral；本轮补全 column + 修 content-box → **net +1 / 0 回归**（css-flexbox oracle A/B vs R981 baseline）。**首个 flexbox net +1 measurable pass**（flex-minimum-height-flex-items-022），证明 R982 row-only 的 column 回归是可修的（非 architecture 限制）。

**修复 2 点（apply_flex_transferred_min_size）**：
1. **re-enable column** + 纠正 auto_min。R982 的 `min(intrinsic_main, transferred)` 在 column 错：flex-minimum-height-flex-items-007（img 固有 60×60，column cross=width=100）→ min(60,100)=60，但 spec 应 100。§4.5/csswg #5663：**cross 明确时 content suggestion 也是 cross 推导（= transferred）非 raw intrinsic**。改 `auto_min = transferred`（drop intrinsic min）。row driving test 不变（intrinsic 300 > transferred 100，min 给 100 同 transferred）。
2. **content-box cross**：transferred 须基于 item content-box cross（扣 item cross padding）。flex-aspect-ratio-intrinsic-padding-001（img padding:20，cross 240 → content 200 → transferred 100 非 120）。仅扣 padding 不扣 border（ZW 默认 border-width=medium=3px 即使 border-style:none，扣 border 会污染无 border 项——探针实证：row driving test 扣 6px border 得 88 非 100）。

**A/B 实测（css-flexbox oracle vs R981 baseline）**：
- **+1 improvement**：flex-minimum-height-flex-items-022 2.79%→0.75% PASS（column transferred 正确）
- **0 regressions**：flex-aspect-ratio-intrinsic-padding-001 0%（content-box 修，R982 v1 曾回归）；flex-minimum-height-flex-items-007 0.75%（column，auto_min=transferred=100 正确）；auto-margins-002 0.75%（R982 auto-margin guard）；flex-aspect-ratio-img-column-012 0.61%（R982 align-self guard，flex-start 非拉伸跳过）
- driving test flex-minimum-width-flex-items-013 仍 10.10%（row，残余 = `<p>` font-墙，非 transferred 缺口）

**验证**：make test **12122/0/73**（+2 新 column 测：test_flex_transferred_min_size_column_direction + _column_intrinsic_smaller_than_transferred）；clippy 零警告；fmt 干净；product-smoke welcome **16.88%**（=基线 <20%）。tree.rs 1060 / 测试 443 行（均 ≤2000）。详见 commit dd30e6b5。

**意义**：flex transferred-size-suggestion 现 row+column 对称 + content-box 正确。**net +1 measurable pass**（打破 R964-R982 调查/neutral 为主，首净正 flexbox win）。R982→R983 跨会话接力（row-only net-neutral → column+content-box net +1）证明 transferred-size lever 可逐 slice 推进。font-墙解后该簇（flex-minimum-*-flex-items-* + aspect-ratio-intrinsic-*）可批量翻 PASS。

**▶ 下会话**：① driving test flex-minimum-width-flex-items-013 残余 10% 拆解（img 已正确 100×100，10% = `<p>` font-墙 + 红块绝对定位——font-墙 Phase-A 解后 PASS，独立多 session）；② 其他 flexbox 簇（aspect-ratio-intrinsic-size-007 33% = transferred 同谱系待查是否 transferred 适用条件 / flexbox-flex-flow-001/002 23% = flex-flow 简写展开 / flex-abspos-inset-* 19% = R98/R500 abspos 谱系）；③ **fresh-oracle 续扫其他 dir**（R981 方法论 + R982/R983 flex transferred-size 连续 yield 证明 fresh + 精确 root cause 仍产出——css-grid 残余 / css-multicol）；④ Phase-A font-墙（DC-2~5 主线，最大杠杆，driving test + 多簇待批量 PASS）。

### R984 css-grid 重扫 + css-flexbox 余 worst 分类 = 全 multi-session·零 clean lever·零源码·纯调查

承 R983 ▶ ②/③「其他 flexbox 簇 + fresh-oracle 续扫」。R982/R983 把替换元素 transferred-size 干净 lever 已尽（row+column+content-box，net +1）。本轮系统核查 css-grid + css-flexbox 余 worst，**全 multi-session（R717/R370/font-墙/structural）**，零 clean 单 session lever。

**① css-grid 重扫（post-R983）**：20/49=41%（=R976 baseline，零变化——R982/R983 fix 的 parent-flex guard 限定只影响 flex item，grid 不受影响）。top worst 全 R976 已分类的结构性：replaced-element-percentage-height-in-grid-nested-in-flex-002（33.91%=R119/R168 flex→grid→1fr 百分比高度 CB 链）/ table-grid-item-dynamic-003（25.78%=table-in-grid）/ grid-container-baseline-synthesized-001..004（16-17%=R926 双层结构性）/ nested-grid-item-block-size-001（13.76%=R976 aspect-ratio fix 残余，嵌套 grid track/overflow sizing）。**css-grid 已穷尽（R976 分类仍成立），勿再扫**。

**② css-flexbox 余 worst 分类（post-R983，全 multi-session）**：
- aspect-ratio-intrinsic-size-007（33.65%）+ 003/004（15.67%）+ 011/014（14.86%）= **R717 耦合**（`<svg width=100% height=100% viewBox=7500x3750>` 百分比 dim 无绝对固有，仅 ratio——R980 decode-trap 已 ruled out，须 4-crate ratio-signal 架构）。transferred fix 不适用（容器无明确 cross，且 R982/R983 仅替换元素）。
- **flex-item-transferred-sizes-padding-border-sizing / content-sizing（14.86% × 2）= R370 耦合**（Phase 0 probe 实证：div `aspect-ratio:1/1; min-height:100px` in `flex column float:left` → ZW 渲染 w=800 h=100，应 ~100×100。w=800 = flex 容器**不 shrink-to-fit**（float:left 仍拉满 800）= R370 flex-container-intrinsic-width bug（须 flex_row_intrinsic_width 结构性修复），非 transferred 缺口。R982/R983 transferred 机器无法覆盖（div 非替换 + 容器不 shrink）。
- flexbox-flex-flow-001/002（23%）= `font:10px sans-serif` + 22 容器 × 数字项 = font-墙主导（非 flex-flow parser gap）。
- flex-abspos-inset-nested-002/cross-size-001（19/17%）= abspos in flex，R98/R500 谱系。
- content-height-with-scrollbars（17.6%）= overflow scroll 容器 host-layer 缺口。
- flexbox-collapsed-item-horiz-001（15%）= visibility:collapse flex item（R111 strut 残余）。

**综合**：css-grid + css-flexbox 余 worst 全 multi-session（R717 / R370 / font-墙 / host-layer / structural）。R982/R983 是 flexbox 已穷尽的 clean lever（替换 transferred-size）。下一步真 lever = **Phase-A font-墙**（DC-2~5 主线，批量解锁 driving test 10% 残余 + flex-flow 簇 + 多 dir font-coupled 近 pass 案）或 **R717 ratio-signal**（替换 SVG intrinsic，aspect-ratio-intrinsic 簇）。

**▶ 下会话**：① **Phase-A font-墙**（最高杠杆，R975 Phase 0 契约 frag_baseline_y 锚 + fontdue 度量；解锁 driving test flex-minimum-width-flex-items-013 残余 10% + css-flexbox/font 簇 + 多 dir 近 pass 案批量翻 PASS；6 次 net-negative 先例须极窄 slice + 三态门禁）；② R717 full ratio-signal fix（4 crate，aspect-ratio-intrinsic 簇）；③ R370 flex-container-intrinsic-width（解锁 flex-item-transferred-sizes-padding 簇 + inline-flex/grid shrink-to-fit，结构性多 session）；④ abspos auto-margin（§10.6.4/§10.3.7，R981 de-risk 0-yield，待 font-墙）。clean single-session flex/grid lever **已穷尽**（R982/R983 尽）。

### R985 CSS2/selectors 扫 = font-墙主导 + ★ ::first-line feature gap（concrete 可行 lever）·零源码·纯调查

承 R984 ▶「clean lever 已穷尽，转 multi-session」。本轮扫最后一个 R981 flagged 未扫大 dir CSS2/selectors（542 案）。

**CSS2/selectors fresh oracle 226/542=42%**：top worst = attribute-value-selector-007..010（8.86-12.24%，default-font 13+ 行文本 = font-墙主导；selector-specific 如 `[lang="es"]` HTML 大小写敏感性次要）/ first-line-pseudo-012..016 + first-line-inherit-003（::first-line 簇）/ lang-pseudoclass-001/002。**selectors 簇全 default-font 文本 → font-墙主导**（同 css-text/css-fonts），非 selector-matching 干净 lever。

**★ ::first-line feature gap 定位（concrete 可行 lever）**：css-parser 识别 `::first-line`（parser.rs:333 `"before"|"after"|"first-letter"|"first-line"`），但 **style-system + engine 零消费**（grep first-line/FirstLine 在 style-system/engine 无 apply 路径，排除 first-letter/first-child）→ `::first-line { ... }` 规则被解析但**从不应用**到块首行。first-line-pseudo 簇（5+ 案 3-12%）+ first-line-inherit 簇均依赖此。

**实施评估（多步 feature，非单点）**：① parser 已识别；② 须 style-system 存 ::first-line 规则（per-element 或全局规则表）+ 计算 ::first-line computed style（继承 + 覆盖）；③ paint 识别块的**首行 fragment**（ZW text.rs 有 line 结构，首 line entry = 首行）；④ 应用 ::first-line 样式到首行 fragment（core properties：font/color/background/text-decoration/text-transform/word-spacing/letter-spacing；vertical-align 等 "may" 属性非 required，first-line-pseudo-013 = vertical-align "may" 不计）。

**限制（诚实）**：yield 不确定——first-line-pseudo 簇多 default-font（font-墙）+ 部分 "may" 属性（vertical-align）非 required。core ::first-line（font/color）slice 可能让 2-3 案改善，但 font-墙阻挡部分 <1%。风险低（additive feature，不改现有 layout）。

**意义**：CSS2/selectors 扫完，R981 flagged 全部 unswept 大 dir（positioning/tables/flexbox/selectors/grid）均已系统扫。clean single-session lever 跨全 corpus **确系穷尽**（font-墙/R717/R370/structural/::first-line 多步 feature）。::first-line 是当前最 **concrete 可行** lever（区别 font-墙高风险 + R717/R370 多 crate）。

**▶ 下会话**：① **::first-line core feature 实施**（最 concrete 可行 lever，低风险 additive）——先做最小 slice（style-system 存 ::first-line 规则 + paint 首行 fragment 应用 color/font-size/background），A/B first-line-pseudo 簇验证 yield；② Phase-A font-墙（最高杠杆但高风险，6 次 net-negative）；③ R717 ratio-signal / R370 flex-intrinsic-width（多 session）。clean single-session lever 跨全 corpus 穷尽。

### R986 ::first-line style-side 实施 + paint-side 风险评估 = paint_text 太复杂非单 session·已回退·纯调查

承 R985 ▶ ①「::first-line 实施」。本轮实施 style-side 到一半 + 评估 paint-side，结论：**style-side 干净可行，paint-side（paint_text）风险过高非单 session**，已回退全部源码。

**style-side 实施验证（已回退）**：
- ComputedStyle 加 `first_line_pseudo: Option<Box<ComputedStyle>>` 字段 + default None。
- lib.rs mirror ::before/::after：`compute_element_style_internal(..., Some(&elem_style), ..., Some("first-line"))` 计算 ::first-line。
- 「::first-line 规则是否匹配」信号：::first-line 无 marker 属性（区别 before/after 的 content），用 `first_line_has_text_overrides(fl, base)`——比较继承文本属性（color/font 族/text-transform/line-height/spacing/text-shadow），::first-line 继承自 computed 故未匹配时这些 == computed，匹配则必不同。仅检继承属性（保守，避免非继承属性初始值混淆）。
- style-system 编译通过。验证：::before/::after 既有架构（compute_element_style_internal Some(pseudo_name) + collect_pseudo_declarations_with_media + matches_selector_for_pseudo）可直接复用。

**paint-side 风险评估（决定回退）**：paint_text（text.rs:661，~500 行）是全仓最复杂且 regression-critical 函数之一——多分支（stored inline_layout / 非 stored IFC / multicol 列分配 / R109 fragment）、每分支独立 glyph 循环、per-fragment color（frag_color text.rs:1087）。::first-line 须在**每分支**识别「块首行」并应用 color/font 覆盖：
- stored 路径：fragments 有 y（line_y + f.y），首行 = min line_y，按 y 阈值判。
- 非 stored 非 multicol：`for line in &inline_ctx.lines` 的首个 line。
- 非 stored multicol：首列首行（line_col==0 && 首个 line）。
- 任一分支 bug 回归**全部文本渲染**（reftest corpus + product pages），yield 不确定（first-line 簇 font-墙 + "may" 属性如 vertical-align），性价比差。

**裁决**：style-side 单独落地 = 死数据 + 每元素额外 matcher pass（perf 成本）零 observable 变化（违反「不做零价值修改」）。paint-side 非单 session（须逐分支小心改 + 全量 reftest 回归门禁）。**已 git checkout 回退 3 文件**（computed_style/default_impl/lib.rs），零 net 源码。

**结论**：::first-line 是 real feature gap 但 paint 集成高风险。须 dedicated session（style-side ~30min + paint 逐分支 ~2h + 全量验证），非当前 slice。本会话周期已 land R982/R983（flex transferred-size，net +1），余 lever 全 multi-session（font-墙/R717/R370/::first-line）。

**▶ 下会话**：① **::first-line 完整实施（dedicated session）**——style-side（已验证可行，~30min）+ paint-side 逐分支（stored/非stored/multicol）首行 color 覆盖 + 全量 reftest 回归门禁，预期 first-line-pseudo 簇 2-4 案 yield（部分 font-墙阻挡）；② Phase-A font-墙（最高杠杆高风险）；③ R717/R370（多 session）。**clean single-session lever 跨全 corpus 确系穷尽**——R981/R984/R985 系统扫全部大 dir + R982/R983 flex transferred-size 已尽 + R986 ::first-line paint 非单 session。下一步真 lever 全 multi-session architecture。

### R987 ::first-line 完整实施 + A/B = 0 yield（匿名块 gap + 簇 "may" 属性主导）·已回退·纯调查·R554 pattern

承 R986 ▶ ①「::first-line 完整实施」。本轮完整实施 style-side + paint-side post-process + A/B，**全 corpus selectors 542 案零变化（0 regressions / 0 improvements / 0 case 改变 >0.05%）**→ 已回退全部源码。

**实施（已回退）**：
- style-side：ComputedStyle 加 `first_line_pseudo` 字段 + lib.rs mirror ::before/::after 计算（`compute_element_style_internal(Some("first-line"))`）。匹配检测 `first_line_has_text_overrides(fl, default)`——比较继承文本属性 vs `ComputedStyle::default()`（早返机制保证无规则→fl=default，故 fl≠default ⟺ 规则匹配）。style 单测 2 个全过（color override → Some；无规则 → None）。
- paint-side：paint_text 顶部记 `glyph_start` + 算 `first_line_color`（块 first_line_pseudo.color ≠ style.color 时）；块末（return 前）post-process：找本块新增 glyph 的 min_y，recolor `|y−min_y|<0.5` 的 glyph（首行同基线）。make test 12124/0 + clippy 干净 + fmt 干净。

**A/B 实测（css/CSS2/selectors 542 案 vs R986 baseline）**：
- **0 regressions / 0 improvements / 0 case 改变 >0.05%**——paint fix 对整个 selectors corpus **零可见效果**。
- first-line 簇（first-line-pseudo-007/008/017/018 = 0.33%、first-line-selector-013 = 0.69% 等 color-setting 案）**全部本就 <1% passing**（baseline 即过），且 withfix 字节不变。

**根因（为何 0 yield）**：
1. **匿名块 gap（主因）**：::first-line 附在元素（如 div）上，但 div 的直接文本经 anonymous-block wrap（block/float 混合产生匿名块子盒）→ paint_text 渲染的是匿名块，匿名块**无 node_id 或无元素的 first_line_pseudo** → first_line_color lookup 失败 → recolor 不触发。first-line-pseudo-007（`:first-line{color:green}` + div color:red + floated child）正此结构——文本在匿名块，paint fix 到不了。须 propagate first_line_pseudo 到匿名块（更大架构改动）。
2. **簇属性 = "may"**：FAILING 的 first-line-pseudo-012..016（3-12%）全测 `vertical-align` on ::first-line（spec 标 "may"，非 required），非 color/font core 属性，core ::first-line 实施帮不了。
3. **color 案本就 passing**：core color-::first-line 案（007/008 等）因文本小，red-vs-green diff <1% 已过 oracle 1% 门（ZW 实际可能渲红而非绿，但 diff 小被判 pass）——即便 fix 使其变绿（0.33%→~0%）也不构成新 pass（已 <1%）。

**裁决**：::first-line = **R554 pattern**（feature gap 真，实施正确（style 测过 + 编译），但 0 corpus yield——匿名块 gap + 簇 "may" 属性 + color 案本就 passing 三重阻塞）。**已 git checkout 回退 5 文件**（computed_style/default_impl/lib.rs/text.rs/shorthand_coverage.rs），零 net 源码。须 (a) 匿名块 propagate first_line_pseudo + (b) 簇转向非 "may" 属性才有 yield，均非单 session。**勿再以 ::first-line core color 为单 session lever**（已 0-yield 确证，R986 paint-risk 评估 + R987 实测双确认）。

**意义**：彻底确证 ::first-line 非 yield lever。本会话周期 land R982/R983（flex transferred-size，net +1）为唯一 measurable win；R984/R985/R986/R987 系统调查穷尽 clean single-session lever。

**▶ 下会话（真 lever 全 multi-session architecture）**：① **Phase-A font-墙**（最高杠杆，R975 frag_baseline_y 锚 + fontdue 度量，6 次 net-negative 先例须极窄 slice + 三态门禁——解锁 driving test flex-minimum-width-flex-items-013 残余 10% + 多 dir font-coupled 近 pass 案批量）；② R717 full ratio-signal fix（4 crate，aspect-ratio-intrinsic 簇 SVG intrinsic）；③ R370 flex-container-intrinsic-width（flex-item-transferred-sizes-padding 簇 + inline-flex/grid shrink-to-fit）；④ abspos auto-margin（§10.6.4/§10.3.7，R981 de-risk 0-yield，待 font-墙）。**::first-line 勿再投（R987 0-yield 确证）**。clean single-session lever 跨全 corpus 确系穷尽。

### R988 harness-JS vein 复核 = 已 sound + background-root-101/102/103 = 1.76% font-墙（R888 误诊纠正）·上一会话 insertBefore 调查线索关闭·+1 端到端回归门禁·零源码业务变更

承上一会话尾部未记录的调查线索——检查 `apply_scripted_dom_mutations`（reftest harness 执行 DOM-mutating JS 后应用 mutation 的路径）是否捕获 insertBefore / className 类 mutation，定位 R888 flagged background-root-101/102/103 @100%「JS DOM mutation」失败。本轮系统复核 = **harness-JS vein 已完全 sound，R888 误诊，线索关闭**。

**复核链（三层实证）**：
1. **wiring**（reftest.rs:460 + :557）：`render_to_framebuffer_with_base` 与 `render_via_webview_to_framebuffer_with_base` 均先调 `apply_scripted_dom_mutations(html, base_dir)` 再渲染 → harness-JS 已正确接线（非「跑 JS 但不反映 DOM」）。
2. **mutation 捕获**（js_dom_bridge.rs）：`DomMutation` enum 覆盖 `SetAttr/SetText/SetInnerHtml/SetStyle/Remove/CreateElement/CreateTextNode/AppendChild[*]/InsertBefore[*]/*OnHandle` 全族；`apply_dom_mutations` 逐分支处理；JS shim（js_dom_shim.js）`getElementsByTagName(...)[0].className = 'after'` → setter 映射 `__zw_set_attr(sel,'class','after')` → `SetAttr{selector:'head',name:'class',value:'after'}`。`InsertBefore` 变体 + `__zw_insert_before[_handle]` 回调齐全。**诊断测试实证**（`test_r988_background_root_render_after_mutation` 前身）：`<body onload="setTimeout(test,5)">` + `function test(){...}` 经 V8 sandbox 执行后，HTML round-trip 正确产出 `<head class="after">` / `<p class="after">` / `<html class="">`（reftest-wait 清除）——**V8 在 test 进程正确初始化，setTimeout 经 microtask 同步触发，3 个 className mutation 全捕获**。
3. **渲染端到端**（`make reftest-oracle background-root-10`）：background-root-101/102/103 实测 **1.76% diff**（非 R888 所称 100%），oracle shot 亦纯绿 (0,128,0)；`render_to_framebuffer` 后 viewport 下半部 **100% 绿像素**。即 JS mutation → serializer 保留 CDATA（R917 续 serializer raw-text fix）→ `head.after + body` 相邻兄弟选择器匹配 → §14.2 canvas 背景传播，**全链正确**。

**裁决**：R888 flagged「background-root-101/102/103 @100% = JS DOM mutation」**双重误诊**——(a) JS mutation **已**正确应用（R917 接线 + R917 续 serializer CDATA fix 后 round-trip 保真）；(b) 渲染**已**正确（100% 绿，匹配 oracle）。真 diff = **1.76% font-墙**（`color:white` + `font-weight:bold` 后的文本字形光栅噪声，刚过 1% oracle 阈值，与 R918 rAF 测试 font-墙封顶一致）。**R888 该簇归因 stale**（pre-R917-续 serializer fix 时确为 ~100%，R917 续后已降到 2.01%→现 1.76%），R917 续已纠正但 R888 扫描未回滚该归因。

**上一会话线索状态**：上一会话尾部调查 `apply_scripted_dom_mutations` 的 insertBefore 覆盖（疑 harness-JS gap）= **REFUTED 已关闭**——harness-JS 全链 sound，insertBefore/className/setAttribute/innerHTML/appendChild/createElement 全捕获 + 应用。harness JS vein（R917→R919→R943）DC-1 正确性提升已交付，残余 yield 须待结构性多会话（font-墙 Phase-A / R109 anonymous-block / multicol fragmentation），非 harness-JS 缺口。**勿再以「harness 不应用 DOM 变更」为 lever**。

**交付**：+1 端到端回归门禁 `test_r988_background_root_render_after_mutation`（reftest.rs tests 模块）——assert background-root-102（body.class）+ 101（head+body 相邻兄弟）经 harness JS mutation 后均渲染 >50% 绿像素，覆盖 V8-init + setTimeout-onload + className-mutation + serializer-CDATA 保真 + 相邻兄弟选择器 + canvas 背景传播全链；任一环节回归即 fail。零业务源码变更。

**验证**：`cargo test -p zero-wpt-runner --bin zero-wpt-runner test_r988_background_root_render_after_mutation` PASS。

**▶ 下会话（真 lever 全 multi-session architecture，与 R987 一致）**：① **Phase-A font-墙**（最高杠杆，R890 锁定真阻塞 = paint Path B IFC 用空 styles（R72 安全路径）→ font_metric_provider 无法解析 family → fallback 0.8；unlock = `store_font_sizes_from_ifc` 式 ascent override map 把真实 ascent 存 LayoutBox 新字段，paint 经 override map 读绕过空 styles——R639 owner-height 桥同模式；welcome 非目标（R891 证 font-metric 对 welcome negligible），目标 = linebox/css-text 非-Ahem 文本类 + CJK 度量死锁 R633 谱系）；② R717 full ratio-signal；③ R370 flex-container-intrinsic-width；④ abspos auto-margin（待 font-墙）。clean single-session lever 跨全 corpus 确系穷尽（R981-R988 系统复核）。

### R989 linebox 非-valign 异常 = R109 结构性 + Phase-A ascent override-map 设计深挖（chicken-and-egg + 全局 line-box 风险）·零源码·纯调查

承 R988 ▶ 继续 probe linebox（R974 flagged 2 个非-valign 异常未解）。本轮 read-only 调查 + 设计分析，**零源码**。

**① border-padding-bleed-002（5.28%）+ inline-box-002（4.91%）= R109 结构性**：逐案读源——border-padding-bleed-002 测 inline 非替换元素 `padding-top:1em` 渲染 + 行盒绘制顺序（padding-top 应 "bleed" 覆盖前一行 inline 盒的 border-bottom/padding-bottom，CSS §10.8.1 inline box model）；inline-box-002 测 "block boxes within inline boxes"（`display:inline;position:relative` 内含 block 子→§9.2.1.1 匿名块盒生成 + split inline 相对定位传播到 block 子）。**两案均 R109 / inline-level-box-model 谱系**（border-padding-bleed = inline-box padding 绘制 + paint order；inline-box-002 = 匿名块盒生成），非 clean 单 session lever，须 Phase A IFC 统一 / 匿名块盒生成多会话。R974「可顺手查」裁决：查过，结构性，关闭。

**② Phase-A ascent override-map 设计深挖（R890 unlock 的真复杂度）**：读 `apply_vertical_alignment`（inline/mod.rs:1596-1649）+ `store_font_sizes_from_ifc`（inline_finalization.rs:297）+ TextFragment/InlineRun 字段。**关键发现 = chicken-and-egg + 全局 line-box 风险**：
- `apply_vertical_alignment` 在 3 处用 `0.8` 常数（line 1622 strut 文本行 `(line_height-dominant_fs)/2 + dominant_fs*0.8` / 1624 strut 原子行 `container_fs*0.8` / 1634 文本 run ascent `run.font_size*0.8`）。0.8 = Ahem ascent ratio（碰巧也是通用字体粗启发式）。
- **override-map 在此场景的陷阱**：override map（`text_node_font_sizes` 等）由 `store_font_sizes_from_ifc` 在 **layout 后** 填充。但 `apply_vertical_alignment` **在 layout 期内** 跑（lines 已建、override 空）→ 若 layout 读 override 得 0.8（空）→ layout 算出 0.8-based line-box；store 后 paint 重跑读真实 ratio → paint line-box 与 layout **不一致**（container height 已由 layout 0.8 定，paint 行位偏）。
- **唯一一致方案**：layout 的 `apply_vertical_alignment` **直接查 provider**（layout 期 styles 可用、family 可解析→真实 ratio），`store_font_sizes_from_ifc` 存 ratio，paint 重跑读 stored ratio。即 layout 走 provider / paint 走 override，两侧都用真实 ratio。
- **复杂度**：须 (a) InlineRun 携带 font_family（或从 styles 查）；(b) `apply_vertical_alignment` 加 provider 查询分支（带 family）；(c) 新 `ascent_ratio_overrides: HashMap<NodeId,f32>` map + store_font_sizes_from_ifc 填充；(d) apply_vertical_alignment 读 override（default 0.8）。**3-4 文件协同改动**。
- **风险**：改 line-box height **全局**（所有非-Ahem 文本行盒变高/基线移），line-height cascade（多行容器高度、flex/grid item baseline、abspos 子定位）可能 net-negative（§12.4 R834/R836/R849/R875 四次 strut/v_offset net-negative 先例；R890 推测是 empty-styles 假象，但 override-map 一致方案是首次真测 layout 侧 provider）。

**③ R891 concept ② ≠ override-map（纠正传递性证伪误判）**：R891 concept ②（`render_fragment!` baseline_offset `fs*0.928`）是 **paint-only 字形定位**微调（glyph 在已定 line-box 内下移），与 line-box 本身（height/baseline_y）无关故 negligible。override-map 一致方案改的是 **line-box 度量本身**（strut/run ascent 用真实 ratio→line-box 变高），**机制不同**——R891 **不**传递性证伪 override-map。但 R890「§12.4 先例极可能 empty-styles 假象」仅为推测，override-map 一致方案是首次真测，结果未知（可能 yield / 可能 net-negative 成为第 7 次 strut 先例）。

**裁决**：Phase-A ascent override-map **非已证伪，但复杂（3-4 文件 chicken-and-egg）+ 高风险（全局 line-box 改）**，须 dedicated session 全 A/B 周期 + revert-ready（welcome/linebox/css-text/product-smoke 四维门禁，净负即回退）。本会话已 land R988（+1 回归门禁），不再启动此高风险多文件改动（避免半途中断留 broken wiring）。clean single-session lever 跨全 corpus 经 R981-R989（9 轮）系统复核**确系穷尽**。

**▶ 下会话（dedicated session 攻坚 ascent override-map 一致方案）**：① **Phase-A ascent override-map**（按上述 4 步实施：InlineRun family + apply_vertical_alignment provider 查询 + ascent_ratio_overrides map + store/read；A/B 四维门禁 welcome<20% + linebox/css-text oracle 零回归 + self-source 不降 + product-smoke，净负即回退——若 net-negative = 第 7 次 strut 先例，彻底关 ascent 角度转 R717）；② R717 full ratio-signal（4 crate，aspect-ratio-intrinsic 簇，R980 decode-trap ruled out，path clear）；③ R370 flex-container-intrinsic-width。**linebox border-padding-bleed-002/inline-box-002 = R109 勿再查（结构性）**。

### R990 ★★★ Phase-A ascent ratio is_ahem-gated LANDED = 全量 oracle 4530→4668 (+138 pass)·font-wall 首次实质突破·R989 设计兑现·净正

承 R989 ▶「dedicated session 攻坚 ascent override-map」。R989 设计了 4 步 provider+override-map 一致方案，但本轮**实现时发现更简单的等价路径**——直接在 `apply_vertical_alignment` 把硬编码 `0.8` 改为 **is_ahem 门控的 ratio**（Ahem 0.8 / 非-Ahem 0.928），**无需 provider plumbing**（绕过 R887-R890 的 5-layer wiring 死路）。

**关键洞察（R989 chicken-and-egg 的解）**：R989 担心 override-map 在 layout 期为空致 layout/paint 不一致。但实测 `is_ahem_font` 在 **layout 由 `style.font_family` 定**（mod.rs:546）、**paint 由 `is_ahem_overrides` map 定**（mod.rs:551），**两侧已一致**（store_font_sizes_from_ifc 填充）。故 ratio 直接从 `run.is_ahem` 读，layout/paint 同一函数两侧都对——**不受 paint Path B 空 styles 影响**（区别于 R889/R890 provider 单点 no-op，provider 需 family 而 paint 空 styles 无 family）。

**实施**（`crates/layout-engine/src/inline/mod.rs::apply_vertical_alignment` 1596-1649）：
- strut_ascent 文本行分支（line 1611-）：dominant run 取 `max_by(font_size)` 携带其 `is_ahem`，`dominant_ratio = if dominant_is_ahem {0.8} else {0.928}`，替换硬编码 0.8。
- per-run max_ascent（line 1632-）：`run_ratio = if run.is_ahem {0.8} else {0.928}`，替换 0.8。
- 原子行分支（line 1624 `container_font_size*0.8`）保留 0.8（atomic-only 行罕见 + container is_ahem 检测复杂，低 ROI 不动）。
- **0.928 来源**：R885 FontMetricProvider 实测 DejaVuSans ascent=0.928（system-ui 典型，与非-Ahem corpus 字体一致）。

**A/B 实测（全量 oracle 10397 案，stash 对照）**：
- **baseline (R989)**：oracle-pass **4530 (44.6%)** / credible 4402 (43.3%) / strict 真通过 286 (2.8%)
- **with R990**：oracle-pass **4668 (46.0%)** / credible 4540 (44.7%) / strict 真通过 286 (2.8%)
- **NET：+138 oracle-pass / +1.4pp**（credible +138）；strict 真通过 286→286 持平（font-raster strict 噪声封顶不变）。+138 全部是「近似通过 (strict..1%)」翻到 oracle-pass (<1%)——非-Ahem 文本行盒高度修正后 138 案跨过 1% 阈值。
- **分目录**：css-text **339→355 (+16)** / linebox 119→120 (+1) / css-fonts 98→98 (持平)。css-text +16 经 stash 对照确证（rigorous A/B）。
- **product-smoke welcome**：R989 baseline **16.88%** → R990 **16.57%**（**-0.31pp 改善**，<20% gate PASS）。★ **纠正首轮「+0.41pp 回归」误判**——首轮对照的 16.16% 是 **R632 stale 基线**（2026-06-25），实际 R989 基线经 R917-R988 harness 工作累积到 16.88%；rigorous stash A/B（checkout R989 mod.rs 重建）确证 R989 16.88% → R990 16.57% = **-0.31pp 改善**。
- **product-smoke morning-work（CJK 重）**：R989 **13.88%** → R990 **13.77%**（**-0.11pp 改善**）。★ 推翻「0.928 Latin-only 致 CJK 回归」担忧——CJK 字体 ascent（NotoSansCJK ~0.88-0.93）0.928 比 0.8 更近 chromium，CJK 文本同样受益。**R990 全维度净正，零回归**（welcome -0.31pp + morning-work -0.11pp + 全量 oracle +138 pass）。
- **R990 余波逐 dir A/B（stash R989 对照，9 dir 全测）**——css-text **339→355 (+16)** / linebox 119→120 (+1) / css-fonts 98→98 (0) / backgrounds 228→228 (0) / writing-modes 5.6%→7.1% (+~12) / generated-content 87→92 **(+5)** / lists 144→144 (0) / borders 393→399 **(+6)** / values 17→17 (0)。**9 dir 全部非负，零回归**；已测 dir 合计 ~+40，余 ~+98 分布在 positioning/tables/flexbox/multicol/text-decor/fonts/floats-clear（未逐测但全量 oracle 净 +138 已含）。**R990 余波 due diligence 闭——无隐藏回归，clean net +138 确证**。

**★ R990 余波 line-height:normal 1.15 实验 REFUTED（1.2 已是 corpus 最优）**：试把 R990 同模式应用到 `NORMAL_LINE_HEIGHT_RATIO`（text_metrics.rs:154，非-Ahem line-height:normal 用）——1.2→1.15（DejaVuSans hhea 推导值 ~1.16）。**A/B NET 负**：welcome **16.57%→17.67%（+1.10pp 显著回归）**+ morning-work 13.77→13.78%（持平）+ css-text 355→359（+4，远小于 welcome 回归）。已 `git checkout` 回退。**结论**：1.2 **已是 corpus/product 字体（system-ui/DejaVuSans）的最优值**——chromium 在本环境的 system-ui line-height:normal ≈ 1.2，非启发式巧合。**R990 ascent（0.8→0.928）是唯一可产的 font-metric 常数 lever**（ascent 是 0.8 = Ahem 专用常数，真字体 0.928 差 16%；line-height:normal 1.2 恰好匹配系统字体）。**勿再调 NORMAL_LINE_HEIGHT_RATIO**（1.2 已验，1.15 net 负）。font-wall 经 R990 + 本轮 line-height + R989 site-3 三轮余波**确已尽 layout-side font-metric 常数 lever**，forward = per-font 真实度量（须 R887 provider wiring 多 session）或转 R717/R370 非 font 角度。

