# R1094（2026-07-06）：freetype-raster feature-on 全 corpus Oracle A/B 实测 + tuning 缺口精确定位

承 R1093（autonomous plateau exhaustive 完成）。R1092 估计 C-dep（FreeType）batch unlock font-wall instruction cluster +200~400，但属 probe 推断非直接测量。本轮：**首次直接全 corpus A/B 实测验证估计** + 深挖 R1068②「paint v_offset 与 FreeType 度量 coherence」的下一个 tuning 杠杆。零源码 net 变化（freetype-raster 仍 default-off；临时 probe 已回退）。

## Linux 验证（WSL2，gcc 14.2，bundled FreeType2+libpng）

- `cargo build --features freetype-raster` → OK（11.49s，bundled C 编译通过）
- `cargo test --features freetype-raster` → **523 passed / 0 failed**（含 cfg-gated `freetype_rasterize_ahem_glyph_end_to_end` 坐标守卫测试）
- `cargo clippy --features freetype-raster --all-targets -D warnings` → 零 warning
- 浏览器 CLI 直转验证 `cargo check --bin zero-browser --features zero-render-foundation/freetype-raster` → OK（apps/browser 直依赖 render-foundation，零代码改动即可本地用 FreeType）

## A/B 结果（reftest-oracle，z_vs_chr < 1.0% = oracle-pass）

**css-text 子集（1650 案）**：
- baseline(feature-off) oracle-pass **359** (21.8%) / credible 346 / strict 84 / near 275
- treatment(feature-on) oracle-pass **383** (23.2%) / credible 370 / strict 88 / near 295
- Δ = **+24** oracle-pass / +24 credible / +4 strict / +20 near（精确复现 R1068）

**全 corpus（10397 案）**：
- baseline(feature-off) oracle-pass **4759** (46.8%) / credible 4631 / strict 295 / near 4464
- treatment(feature-on) oracle-pass **4991** (49.1%) / credible 4862 / strict 302 / near 4689
- Δ = **+232** oracle-pass (+2.23pp) / +231 credible / +7 strict / +225 near

**零回归**：25 目录 improved / 0 regressed / 47 unchanged；逐 dir Δ 之和 = +232 与总数对账。

**Top 改善目录（feature-on 全 corpus）**：

| Δ | 目录 | (baseline→treatment) |
|---|---|---|
| +129 | css/CSS2/selectors | (299→428) 主导，~55% 收益 |
| +22 | css/CSS2/text | (213→235) |
| +13 | css/CSS2/tables | (48→61) |
| +12 | css/CSS2/sec5 | (0→12，新通过) |
| +9 | css/css-text-decor | (108→117) |
| +7 | css/css-text/line-breaking | (61→68) |
| +6 | css/CSS2/positioning | (296→302) ← abspos instruction cluster |
| +4 | css/CSS2/floats-clear | (74→78) |
| +3 ×4 | css-multicol / css-fonts / generated-content / white-space | |
| +2 ×4 | css-flexbox / CSS2 normal-flow / fonts / backgrounds | |
| +1 ×8 | word-spacing / text-transform / hyphens / CSS2 syntax / margin-padding-clear / linebox / box-display / borders | |

**★ R1092 的 +200~400 batch unlock 估计首次实测确证**（+232 落中段）。单 dir 分布与估计不同（selectors 主导，非假设的 margin-padding-clear +126——实际该 dir 仅 +1），但总量吻合。机制 = font-wall batch unlock：文本密集 reftest 在 FreeType 修正光栅化后批量跨过 1% 阈值。

## tuning 缺口精确定位（paint ↔ renderer glyph 放置契约深挖）

**契约分析**：
1. FreeType `rasterize`（`crates/render-foundation/src/font/loader.rs:80`）算出正确 `y_offset = bitmap_top − height`（真 per-glyph ascent metric），存入 GlyphBitmap。
2. 但 reftest 渲染走 **GlyphPrimitive 路径**（`crates/render-foundation/src/cpu/mod.rs:485-516` → `blit_glyph_bitmap:523`），该路径把位图左上角 **raw-blit 到 `(glyph.x, glyph.y)`，完全忽略 `GlyphBitmap.x_offset/y_offset`**。
3. 另一条 `draw_glyph`（cpu/mod.rs:435）用 `glyph_top_left + y_offset` 正确放置，但它是 GlyphDraw（GPU 侧）路径，非 reftest CPU 路径。
4. paint 侧（`crates/engine/src/paint/painter/text.rs` `render_fragment!` macro）按片段（per-fragment）发单一 `glyph.y = content_y + frag.y + v_offset + ty`；非-Ahem stored 用 `v_offset = frag.font_size`（1.0·fs 启发式），Ahem 用 `baseline_y_abs − 0.8·fs`，non-stored 用 R953 的 `height − 0.8·fs`。

**GLYPHPROBE 实测**（env-gated，white-space 目录，15 fragment 全 Ahem）确认 Ahem 路径 `glyph.y = baseline − 0.8·fs`（heuristic bitmap-top），renderer raw-blit。已回退。

**★ 核心结论：+232 全部来自 FreeType 位图数据质量**（hinted ink 形状 vs fontdue tight-ink），**metric 放置（真 per-glyph bitmap_top）完全未用**——paint 仍用 0.8/font_size 启发式定位。

**tuning 下一步 = 协调改动**：paint 发 baseline_y + renderer 用 glyph_top_left 应用 bitmap.y_offset（renderer 有 per-glyph bitmap_top，应用即得 per-glyph 正确放置，paint 仍可 per-fragment 发 baseline）。但 **非-Ahem 的 baseline_y_abs 受 R1090 store-gate 门控**（`inline_finalization.rs:776` `if !is_pure_ahem` 把 valign-aware baseline_y 存储限 pure-Ahem）→ 移除 gate = linebox −47（R1090 实测）→ 与 baseline 公式 + renderer offset 三方互锁（R876 谱系）。

**EV 评估**：这是 8 次 font-metric 单点实验（R834/R836/R849/R875/R1052/R1056/R1067/R1090）**未测的新假设**——彼等皆 feature-off（fontdue）+ 常数公式（R1090 的 0.928 是常数，非 per-glyph bitmap_top 应用），feature-on + 真 per-glyph bitmap_top renderer 应用从未测过；但与 store-gate/strut/half-leading 互锁，单 session 风险高。

## 裁决

- C-dep 价值从「推测 +24 css-text」（R1068）/「估计 +200~400」（R1092）升级为「**实测 +232 全 corpus 零回归**」，default-flip 决策 evidence 进一步强化。
- freetype-raster feature 仍 default-off（CI 纯 Rust 不变）；本轮零源码 net 变化。
- 浏览器本地用 FreeType 零代码改动：`cargo run --release --features zero-render-foundation/freetype-raster --bin zero-browser`。
- **forward 决策（用户）**：① 多 session 协调改动攻 metric-placement 墙（先解 store-gate 非-Ahem 存储 + paint 发 baseline + renderer 应用 offset 三位一体）；② 翻 default-on（CI 6-target bundled-C 编译验证 + 计费决策）；③ 停在 +232（C-dep 价值已 decisive）。

**门禁**：纯测量 + 诊断，零 net 源码（临时 GLYPHPROBE 已回退，tree clean）。Linux 三关（build/test/clippy with feature）全绿。
