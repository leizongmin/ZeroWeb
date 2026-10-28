# R5311 — taffy 测量 IFC 同源 text-indent（CSS2 §10.3.1）——c547-indent-001 ✗1.17%→✓0.43% 翻绿，corpus 15752P/840F→15753P/839F 逐案 diff 恰 {−驱动案} 零新增红；测量/布局双 IFC 缩进分歧定位链

**日期**: 2026-10-28
**驱动案**: `css/CSS2/css1/c547-indent-001.xht`（R5310 下一步①840F 账本续扫择轻命中）
**corpus**: 修复后两次独立复跑 **16592 / 15753P / 839F**，失败集逐案一致；归一化 diff vs R5309/R5310 稳定账本（840F）**恰 {−c547-indent-001}**，零新增红。

## 定性（诊断链）

驱动案：div `text-indent:50%; width:25%`（→196px/98px）**3 行 vs chromium 4 行**，逐带 PNG 比对前 3 行逐像素一致，ref 第 4 行 "block."（x9-46）整行缺失——末词在 ZW 渲染中不存在。

1. **双端隔离**：引擎 RenderPipeline（估算度量）同页结构 4 行正确；runner（注册 fontdue 真实度量，main.rs set_char_measure_fn 等）复现 3 行。
2. **runner 内最小 .xht 三变体 bisect**（临时 probe 页 + layout-dump）：`div.a`(196px+indent98px) h=54 复现；`div.b`(196px 无缩进) h=54 但逐词墨带 16 词全在（合法 3 行）；`div.c`(294px+indent98) 16 词全在（2 行合法）。触发域 = **真实度量 + text-indent + 末词须换新行**。
3. **逐词墨带走查**（RGBA 暗像素列扫描 + ≤4px gap 词合并）：div.a line1 "X The first X"（x107 起，缩进已施）line2 "in this sentence should be" line3 "indented to the center of this"（末端 x189）——"block."（~37px）应换行 line4，实际全页无该墨迹（含 div.b 行带重叠位核查）。
4. **断行层插桩**（临时 env 门控 trace，已撤）：`break_items_into_lines` 对 div.a **全部趟次产出 4 行含 "block."**（line3 y=54 txt=block.）——断行层完全正确，截断在下游。
5. **测量面插桩**：taffy 测量回调 `measure_text_content` 的测量 IFC **nlines=3 / total_h=54**，且该趟 **indent=0**（同 div 的布局趟 indent=98）→ 分歧定位。
6. **根因**：`measure_text_content` 构造测量 IFC 时**漏设 `.with_text_indent()`**（全仓唯一赋值点在布局 IFC inline_finalization:1813；paint Path B text.rs:1725 亦有）→ taffy 内容高按「无首行缩进」行数计 = 3 行 54 高；布局 IFC 带缩进折 4 行，第 4 行 y=54 起于盒底之外，paint 按盒几何裁除 → 丢词。测量/布局/paint 三 IFC 中唯独测量面缺缩进。

## 实现（inline_finalization.rs 一处补设，无新 kill-switch）

`measure_text_content` 在测量 IFC 构造链补：

```rust
if let Some(s) = measure_style {
    let indent = resolve_text_indent(
        &s.text_indent, &s.font_size,
        if width.is_finite() { width } else { 0.0 },
    );
    if indent != 0.0 {
        inline_ctx = inline_ctx.with_text_indent(indent);
    }
}
```

- 与布局/paint 双路径共用既有 `resolve_text_indent`（Px/Em×fs/Percentage×容器宽，CSS §10.3.1）。
- intrinsic 测量（Min/MaxContent，width 可为 0/无穷）：width=0 时百分比自然为 0；width=INFINITY 时按「循环百分比=0」惯例取基准 0（**真实长度缩进照用**——min/max-content 首行仍含确定缩进）。
- 测量与布局同源后，盒高容纳全部折行行，paint 无越盒裁除。

## 交付

- 全量 corpus 两次独立复跑 **16592/15753P/839F**，失败集两次一致；vs 稳定账本归一化 diff 恰 {−c547-indent-001}，零新增红零意外翻转。
- 驱动案 **✗1.17% → ✓0.43%**（≤strict 真通过-可信）；同族 c547-indent-000 保持 ✓0.00。
- 回归钉 `crates/engine/tests/r5311_measure_text_indent.rs`（引擎级 RenderPipeline，Ahem 16px 确定性）：同文两 div，`width:196px + text-indent:98px`（首行余 98px 装 3 词 → 17 词 4 行）vs `width:197px` 无缩进（6 词/行 → 3 行）——断言缩进版测量高比无缩进版多一个行盒。**stash mutant 复验红**（修复前 ind=48=plain=48 等高，失败信息即预期签名），修复后 ind=64/plain=48 绿。
- 临时 probe 页 / 插桩 / 引擎探针测试已全部清除，工作树仅剩修复 + 回归钉。

## 门禁

- fmt 0 diff（首跑 make test fmt 腿红——修复行超长，fmt 修正后复跑）。
- clippy：make test clippy 腿（zero-dom/engine 全家 15 crate 含 zero-layout-engine + zero-engine，`--all-targets -D warnings`）0 告警。
- make test 全量：793P/**1F**——`image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight` 翻红，隔离复跑 **3/3 绿**——与 R5310 记账同款多进程并发限流 e2e 负载脆弱签名（对端 renderer/net 域），不阻断。
- product-smoke：struct ×6 全 PASS + form-perf PASS；welcome oracle **15.68%**（75254px 与 R5308-R5310 同值零位移；welcome 无 text-indent 容器，无因果路径）。
- bench-gate 定向 zero-layout-engine：**GATE PASS，25 指标 NEW=0 全在预算内，suspect=false**（medium/layout_ms p50 148.52，预算 181.9；与 R5310 gated 同窗 157.77 同量级）。当前树含 R5307-R5311 全部改动正式过门——**R5307 残余 + R5308 + R5309 + R5310 空闲窗并账挂账一并清账**（门为树态判定，基准零改动零 relax）。
- 新增回归测试套件（cargo test -p zero-engine --test r5311_measure_text_indent）1/1 绿 + stash mutant 红证据在案。

## 挂账

- **跨流发现**：本地 720 套件 `css-float/float-shrink-inline-children-zw-001` ✗3.68% **既有红**（stash A/B 无本修复同值红；case 引入于 d68f3aa16「收缩量测三根因」轮——对端流自加回归钉当前 HEAD 即红）。移交对端流知悉。
- image_batch_e2e 负载脆弱维持记账（本轮 make test 再现 + 隔离 3/3 绿）。
- R5310 挂账全维持：outline-028 第二层（IFC float+inline 行布局）；image_batch_e2e 负载脆弱 + fetch-wpt-data 语料 footgun 双移交对端；text-wrap:balance 9F（中复杂度供给评估）；welcome 15.68% 对端流；R5307/R5308 项。
- flake 账本无新增（content-none-select-1 / size-adjust.tentative / multicol-fill-auto-block-children-002 维持）。

## 下一步 R5312

①839F 账本续扫择轻 ②outline-028 第二层评估（候选结构轮）③bench-gate 空闲窗并账 ④balance 供给评估（若账本轻案不足）。
