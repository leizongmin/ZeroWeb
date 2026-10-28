# R5313 — inline-block 背景盖字 paint-order 修复（CSS2.1 App E）——FLAT_CHILD_WALK 降级路径文本片段容器趟让位（kill-switch `ZW_IB_FLAT_FRAG_DEFER`）；outline-028 ✗0.50%→✗0.31%；corpus 840F→834F（FIXED=5 零新增红 + content-none-select-1 既有 flake 摆动）

**日期**: 2026-10-28
**驱动案**: `css/css-ui/outline-028.html`（R5312 余差归因命中：ref 侧 ib 白 bg 压字形）+ 本轮最小复现 `r5313-v4`（outline-028-ref 第 4 行片段，grid + 前置 span 行 + ib 行）
**corpus**: 同一二进制 kill-switch 双跑 A/B——`ZW_IB_FLAT_FRAG_DEFER=0`（旧行为）**840F** vs 修复态 **834F**，逐案 diff = FIXED 5 案（4× `css-writing-modes/inline-block-alignment-orthogonal-{vlr-003,vlr-005,vrl-002,vrl-004}` + `css-flexbox/flexbox-baseline-align-self-baseline-vert-001`，全部 inline-block 域同源）+ content-none-select-1（既有双向摆动 flake 账，单跑 ✓0.00）；**零新增红**。840 vs R5312 基线 839 = content-none-select-1 负载相位摆入（R5309/R5308 同款记账）。

## 定性（诊断链）

1. **现象**：shrink-to-fit inline-block（width:auto）带 background + padding，自身文本被背景矩形覆盖（outline-028 ref 页 4/7/8 案字形缺失；R5312 五连 probe 立档）。
2. **机制链**（ZW_DBG 插桩 + v4 PNG 复现定谳）：FLAT_CHILD_WALK（collect_items）对**无有效尺寸**的 inline-block 降级为零宽度 TextRun → ib 自身文本片段被扁平进**父容器** stored IFC（归因 node_id = ib 元素 id）→ 容器 paint_text **先**绘字形（render_fragment! 宏内 `painted_inline_nodes.insert(frag_nid)`）→ 子盒遍历**后**绘 ib bg 盖字 → ib 自身 paint_text 经 painted_inline_nodes dedup（text.rs 1286）跳过——CSS2.1 App E「ib 背景先于自身行内内容」违例。
3. **kill-switch A/B 因果**（REFTEST_DUMP_PASS 逐像素）：switch-off v4 test 黑字形 220px（ib 行 X 缺失）vs switch-on 320px（X 恢复，与 ref 逐像素同位）。注记：REFTEST_DUMP 只落失败案，**pass 案 dump 须 `REFTEST_DUMP_PASS=1`**——首两轮 A/B 同值系 stale PNG 假象，险些误判开关无效。
4. **引擎 harness 触发域注记**：RenderPipeline 最小 probe 形（container > ib）**不触发降级**（插桩 gate-enter 零命中）——须 v4 全片段（grid + 前置 span 行 + Ahem + outline-container 结构）。回归钉初版用最小形时断言恒真（vacuous green），换 v4 markup 后 gate 命中方达成判别力。

## 实现（paint 单侧一处门 + 两 helper，布局零改动）

`crates/engine/src/paint/painter/text.rs` stored 片段渲染循环（render_fragment! 调用前）：

```rust
if let Some(styles) = styles
    && defer_flattened_atomic_inline_fragment(box_node, frag.node_id, doc, styles)
{
    continue; // 跳过绘制且不进宏——宏内 insert(frag_nid) 恰是 ib 自身 paint_text 被 dedup 的来源
}
```

`defer_flattened_atomic_inline_fragment` 判据（全部命中才让位）：

1. kill-switch `ZW_IB_FLAT_FRAG_DEFER`（`LazyLock` 构造期单次读取，`=0` 回退旧序）；
2. 片段 owner（Text 归父，镜像 r4332_owner 口径）`display` ∈ {InlineBlock, InlineFlex, InlineGrid, InlineTable}——正常契约下父容器 stored IFC 不携带 ib 文本（App E 空占位），唯降级路径例外；
3. 盒子树内（depth 限 8）存在该 owner 的真实原子盒——盒不存在不让位（否则无人绘文本）；
4. 盒自带可见装饰（非零 border ×4 或 bg 色 ≠ Transparent 或可绘 bg 图）——裸 ib 现行为无覆盖问题，收窄爆炸半径。

让位后 ib 自身盒路径按 App E 序完成：子盒遍历绘 bg（draw_order 位点先）→ 自身 paint_text 不再被 dedup、自绘文本。multicol（~text.rs:2053）与 Path B（~4341）两处片段循环**未挂门**——corpus 零翻红暂无必要，挂观察项。

## 回归钉（engine 级，mutant 红在案）

`crates/engine/tests/r5313_ib_flat_frag_defer.rs`：v4 markup 三色互斥锁归因（span 蓝 / `<p>` 绿 / ib 文本黑 / ib bg 品红），断言 **draw_order 中品红 Fill 位点 < 黑 ib Glyph 位点**（R5086 同源 draw_order 口径 = 真 raster 序）+ 蓝 2 / 绿 5 归因锚。

`crates/engine/tests/r5313_ib_flat_frag_defer_kill_switch.rs`：独立测试二进制（开关经 LazyLock 构造期读取，set_var 须先于本进程首次渲染——同进程双渲染第二腿被 LazyLock 缓存吞掉的教训在案），`ZW_IB_FLAT_FRAG_DEFER=0` 断言旧序（文本位点 < bg 位点），同时证明 harness 确实走降级路径（原子路径两态同序，无法区分）。

**mutant 证据**：helper 源级短路（env-gated `return false`）→ 主钉红于 App E 断言（bg 位点 7 > 文本位点）；恢复绿。kill-switch=0 双跑 = 同等旧序（runner 侧 + 引擎钉双确认）。

## 驱动案与余差

- `r5313-v4`（临时 probe，已删）：✓0.02% 翻绿，PNG 黑字形逐像素同 ref。
- `outline-028`：✗0.50%→**✗0.31%**（1500/480000px）。余差 char-map 归因：3 簇 lime 色域块差（test 侧 lime 200px vs ref 60px 同域），**非字形缺失域**——属 ref 页 `outline: auto` + `width: max-content` 容器在 ZW 的绘制外延差，归 outline 深域切片（fuzzy meta totalPixels=0-4 收口另轮）。
- 附带翻绿 5 案全为 inline-block 域（正交 writing-mode 对齐 ×4 + flexbox baseline vert）：ib 文本改由自身盒路径绘制后基线/对齐快照同源，与机制预测一致。

## 门禁

- fmt 0 diff；clippy `zero-engine --all-targets -D warnings` 0 告警；engine lib **3570P/0F**。
- corpus A/B 同二进制 kill-switch 双跑（见题头）零新增红；5 案单跑复核稳定 ✓。
- product-smoke：struct ×6 全 PASS；welcome oracle **15.68%（75254/480000px）与 R5308-R5311 零位移同值**。
- make test：workspace 腿 abort 于 `image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight`（:317/:233 双态 panic，隔离单跑绿）——**stash 干净 HEAD 复跑同位同 panic 定谳既有负载脆弱族、非 R5313**（R5310/R5311 记账同款）；残余腿补跑：renderer --bin 0P、renderer --lib **350P/2F**、browser --bin **704P/2F/1I**、browser --tests cdp 族 31P 全绿（bin harness 重跑 704P/2F）、quickjs no-browser-renderer **186P/0F**、quickjs renderer --bin 0P、quickjs renderer --lib **347P/3F**（第三红 `mutation_arm_does_not_apply_across_document_reset_window` 同 page_scripts form/reset 域）——**全部失败名与 s125 form fixture 族记账吻合**（`form_interaction_fixture_complete_sequence` / `javascript_disabled_skips_listeners_not_default_actions` / `shared_checked_and_form_actions...` / `form_fixture_complete_multiprocess_semantics`），移交对端维持。
- bench-gate 定向 zero-engine（静窗）：**GATE PASS**，19 指标 NEW=0 全在预算内（page/morning/paint_ms 54.86 / 预算 102.59——stored 循环逐片段门 O(1) 早退，ib 族 owner 才进盒搜索）。
- 临时 probe 页（r5313-*.html ×9）与 reftest-manifest.json 探针登记已清零（manifest 自 /tmp 备份还原、wpt-data 无残留）；ZW_DBG_R5313/ZW_DBG_R5313T/ZW_MUTANT_R5313 插桩全清。

## 挂账

- outline-028 残余 0.31%（lime 域 outline: auto 外延差，非字形域）——归 outline/绘制外延深域，须活体 chromium 对比定性。
- multicol / Path B 两处 stored 片段循环未挂让位门（观察项：corpus 零翻红；若未来 multicol 页暴露同缺陷再扩）。
- float walk y 双补偿、Path B border-box 带 mr 口径核对（R5312 挂账延续）；form fixture 族移交对端（s125）维持；image_batch_e2e + service_worker_owner 负载脆弱双族维持；text-wrap:balance 9F、welcome 15.68% 对端流、corner-shape 族维持。

## 下一步 R5314

①834F 账本续扫择轻（避开 R5313 域刚翻绿面）②outline-028 残余 lime 域定性（outline: auto 外延，候选深域轮）③balance 供给评估维持。
