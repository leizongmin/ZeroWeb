# R5338 证据 — flex 叶首行基线通道 + contain/multicol 基线守卫（css-flexbox-1 §8.5）

日期：2026-10-30 · 轮次：R5338 · 基线：R5337 交付态 有效 797F @ `886ef12bf` · 代码 commit：见轮次记录

## 0. 结论速览

- 叶节点 taffy `first_baselines` 从恒 `None`（→ border-box 底边合成基线，`align-items: baseline` 退化为底对齐阶梯）打通为真实首行基线：`measure_text_content_with_baseline` 测量附带首行基线（内容系）→ engine `leaf_first_baseline_border_relative` 补 border/padding 顶距换算 border 系 → taffy flex 基线对齐机制原样消费（`flexbox.rs` `unwrap_or(size.height)` 合成回退保留）。
- flexbox_align-items-baseline **✗1.77% → ✓0.00%**（chromium 真值：三项同基线盒顶相等；修复前 1em 步进反向阶梯）。
- 两条基线守卫（corpus 定谳补入）：① layout containment（`contain: layout/content/strict`）元素视作无基线（css-contain-1 §3.1，contain-layout-baseline-002 / contain-layout-suppress-baseline-002 回归守卫）；② multicol 容器（column-count/column-width 非 auto）不导出基线——css-align baseline-export 要求首列首行基线，ZW 列片断感知基线未实现（FIXME），保守回退合成。另：无字形首行（仅 `<br>` 空行）不导出基线（ZW 空行盒度量未按基线语义校准，multicol/baseline-001 族依赖）。
- corpus A/B：798F（base final3）→ **796F**：FIXED={flexbox_align-items-baseline, content-none-select-1（脆弱族 flake 本轮 ✓）}，REGRESSED=∅，通过率 95.2%（15794/16590）。
- kill-switch `ZW_FLEX_LEAF_BASELINE=0`：基线通道整体旁路（=0 → flexbox_align-items-baseline ✗1.77% 原值复现，四守卫页全绿），双向可判。

## 1. 择轻与 1.04% 簇预研（负结论）

从 R5337 有效 797F 账本分带扫描（0.00% 特性缺失 / 0.3–0.8 / 0.8–1.3 / 1.3–2.1）：**1.04% 等值簇**（multicol 8 页 + transforms + filter + cascade + contain）疑似同根因，取样三例像素 dump（`/tmp/r5338-cluster/`）：revert-layer-011 50×100=5000px（@property 注册自定义属性在暂停动画关键帧插值——深片）、multicol-fill-balance-003 25×200=5000px（列平衡高度分配——深片）、transform-generated-001 0px（flake）。等值面积 = 全绿填充测试区的巧合，**非同根因**，簇假设证伪，不立项。

转正选单：flexbox baseline 家族（账本常驻 align-self-016 ✗4.17 / css-flexbox-test1 ✗1.99 / flexbox_align-items-baseline ✗1.77 / baseline-synthesis 5 页）。选 flexbox_align-items-baseline（语义单点、chromium oracle 明确）。

## 2. chromium oracle 与机制链

- 探针（上轮 `/tmp/r5337-*` 管线同法）：三项 inline-block（32/48/64px 高，中项 margin-top 16px）+ `align-items: baseline` → chromium 全部盒顶相等（33px）：三项文本皆在盒顶，基线对齐 ⇒ 顶对齐形态；margin 参与基线偏移（css-flexbox-1 §8.5：行基线 = 各项 margin cross-start + 内部基线的最大者）。
- ZW 机制链定位：taffy flex `compute_flexbox_layout` 消费 `layout_output.first_baselines.y`（`:unwrap_or(size.height)` = border-box 底边合成）→ 叶测量 `compute_leaf_layout` 硬编码 `first_baselines: Point::NONE` → 任何叶都无真实基线 ⇒ baseline 对齐恒底对齐。算法机制本身正确，缺的是叶基线数据通道。

## 3. 实现（三层最小改）

- `inline_finalization.rs`：`measure_text_content` → `measure_text_content_with_baseline`（返回 `(Size, Option<f32>)`，首行基线 = `lines.first()` 的 `y + baseline_y`，内容系）；25 个既有调用方经 compat wrapper `measure_text_content` 不动。无字形首行（runs 无非空白文本）返回 None（见 §4-③）。行级垂直域（vertical writing-mode 臂）早退 None，central 域挂账。
- `taffy-local`（vendored 最小 diff，保持上游格式不整文件 rustfmt）：`compute_leaf_layout` MeasureFunction bound → 返回 `(Size, Option<f32>)`，最终 LayoutOutput `first_baselines.y = first_baseline`（ComputeSize 早退路径保持 NONE）；`taffy_tree.rs` 11 处 FnMut bound + 2 处默认闭包 + 测试 helper `size_measure_function` 同步；examples 4 文件（text/image/measure 闭包）跟随。缓存存完整 LayoutOutput（含 first_baselines），cache-safe。
- `engine.rs`：模块级 `leaf_first_baseline_border_relative(styles, dom_id, content_baseline)`——kill-switch `ZW_FLEX_LEAF_BASELINE=0` → None；contain/multicol 守卫（§4）；Px 顶距直取，Em/Percent/其他保守 0（基线略偏，不劣于既有合成；测试族 Px/无框盒，主面无影响）。3 处叶测量闭包（全量/增量/第三路径）解构接线；fragment-segment 臂维持 `None`（FIXME：try_measure_fragment_segment 出首行基线后接入）。

## 4. 回归归因链（corpus A/B 驱动，两轮收环）

第一轮全量 ON：800F，REGRESSED=4（contain-layout-baseline-002、contain-layout-suppress-baseline-002、multicol/baseline-001、multicol/baseline-002）。复跑 4/4 稳定红（非 flake）；`=0` 四页全绿（multicol-001 残 0.69% 计绿）——改动引入坐实。

- ① **contain 族**：css-contain-1 §3.1——layout containment 元素「treated as having no baseline」，flex 对齐须 border-box 合成。ZW 此前全体 None 恒合成，测试绿；真实基线导出后语义缺口暴露。守卫 ① 修复，两页 ✓0.00%。
- ② **multicol 族**：css-align baseline-export——multicol 容器基线 = 首列首行（"highest baseline of all the columns"）。ZW multicol = taffy 叶测量（单流）+ `adjust_multicol_layout` 后处理重写列片断，测量期无列感知 → 正确基线不可达（深片）。main 双绿 = 项1/项2 双合成巧合（同高盒底对齐 ⇒ 顶同）；项1（`<br>` 空行叶）导出真实基线后巧合破裂。dump 定谳（`/tmp/r5338-mc1-on.png`）：项2 合成基线顶到 y≈48、项1 真实基线沉到 y≈133。守卫 ②（multicol 容器 → None）后仍红——项1 的 `<br>` 真实基线（≈16）vs 项2 合成（=100）仍错位 → 引出 ③。
- ③ **无字形首行**：项1 首行仅 `<br>`（runs 无文本）。chromium 取 br 行基线，ZW 空行盒度量未按基线语义校准（br 行高/基线近似值有偏）→ 该行基线不导出（None → 合成），恢复双合成巧合。multicol 两页 ✓（0.69% 内）。chromium 行为差异记 FIXME（行度量校准后放开）。

终轮全量：796F，REGRESSED=∅。守卫全部落在 `leaf_first_baseline_border_relative` + 基线产出端单点（`filter` 无字形首行），blast radius = flex/grid 基线通道；grid 基线消费面经全量 corpus 覆盖（零新红）。

## 5. pin 测试（双向可判）

`r5338_flex_leaf_baseline_tests.rs` 2 例（夹具用内联 style——测试管线 `compute_styles(&doc, &[])` 不收集 `<style>` 元素，同目录既有用例同法）：

- `r5338_baseline_alignment_tops_equal`（驱动）：baseline 对齐 + 异高 + 中项 margin-top → 三项盒顶 spread < 0.5px；`=0` 时失败（tops=[32,16,0] 底对齐阶梯 = 旧 ✗1.77% 形状）。
- `r5338_flex_start_guard_unchanged`（守卫）：flex-start → margin 项独沉 16px、余两项顶相等，`=0` 时同绿（修复未外溢非 baseline 对齐）。

## 6. 门禁

- fmt：`cargo fmt -p zero-layout-engine` 0（taffy-local vendored 不整文件格式化，最小 diff 手排）。
- clippy：zero-layout-engine `--all-targets` 0 error；taffy `--all-targets` 0 error（含 examples 4 文件随签名更新）。
- corpus A/B：798F → 796F，FIXED=2（目标 + content-none flake），REGRESSED=∅，95.2%（15794/16590）。家族核查：align-self-016 ✗4.17 / css-flexbox-test1 ✗1.99 与账本同值（零位移）；baseline-synthesis 5 页 ✗（合成基线语义深片，另案挂账）。
- make test（guarded-test）：clippy quickjs 腿净（`--no-default-features --features quickjs -D warnings` 全绿；compat wrapper 补 `#[allow(clippy::too_many_arguments)]` 一处）；唯一 FAILED = zero-engine lib **3601P/9F**（js_dom_bridge fetch 契约红族，与 R5336/R5337 记录同族同数，与本轮 layout-engine/taffy 模块不相交）；其余 binary 全绿（3003P/874P/814P/372P 等 45 bin）。
- product-smoke：welcome 800px **14.59%** ≤ 20% + struct PASS（card:4/shortcut:6/link-tile:4/footer:1 + title/tagline 行数）；welcome 375/320 窄屏结构门 PASS；article 800/320 struct PASS；form-input-perf-gate PASS；product-smoke-legacy **42 fixtures 0 失败**。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-layout-engine`：**GATE PASS 25 指标 NEW=0**（layout/paint 主指标在预算内：morning paint_ms 62.29 vs 基线 54.43 / 预算 102.59）。

## 7. 挂账 / 下一步

- multicol 列感知基线导出（css-align baseline-export，首列首行）——依赖列片断感知基线，multicol 深片。
- baseline-synthesis 5 页（合成基线 interplay + vert-lr 写向）——合成基线语义族，另案。
- align-self-016（per-item override）、css-flexbox-test1（mega 页）账本常驻。
- `<br>` 行基线校准后放开无字形首行导出（ FIXME 已标）。
- content-none-select-1 脆弱族、行级 vertical writing-mode central 基线域维持挂账。
