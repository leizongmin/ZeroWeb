# 字体后端 FreeType metric 合并可行性评估

**版本**：v1.0（R1172，2026-07-08，rendering-compat rally 自主模式）
**模式**：deep-research 源码深潜（聚焦可行性 go/no-go）
**作者**：AI Assistant
**状态**：可行性评估（inform 架构投资决策）

> **📌 来源说明（全文）**
> - **一手事实**：ZW 源码（crates/render-foundation/src/font/{loader,shaper,cache,mod}.rs + layout-engine/src/inline/font_metrics.rs）+ master.md R834-R1171 round 记录 + memory。
> - **💡 推理/⚠️ 假设**：标注于各节。
> - 无外部搜索（源码深潜模式；外部 Rust 字体后端 landscape 见 §6 forward）。

---

## 30 秒速览

- ZW 字体后端**分裂**：fontdue（加载 + line metric + 回退光栅）+ FreeType（主光栅，R1159 `freetype-raster` feature default-on）。
- **font-wall 主导剩余 gap**（41.3% corpus，R1171）：fontdue line metric（ascent/descent/line_gap）≠ chromium（FreeType/Skia）→ line-box 高度/基线发散。
- **唯一 en masse lever** = 把 line-box metric 从 fontdue `line_metrics_full` 切到 **FreeType `face.metrics`**（chromium Linux 同栈，R1159 已验证 raster 对齐）。
- R1159（FreeType raster）已证 FreeType 路径在 ZW 可工作（+232）。metric 是同一 face 的另一查询，技术可行。
- **风险**：R834-R891 font-metric single-point 全 net-negative/negligible（R1090 fontdue per-font -14 / R891 baseline_offset -0.01pp）。但 **FreeType per-font line-height（line-box 高度）未被直接测过**（R1090=fontdue / R1095=per-glyph offset / R1160=FreeType 常数 / R891=baseline_offset）。
- **go/no-go**：**tentative GO** 一个窄 A/B（FreeType line metric 经 override-map 接 line-box 高度，gated），高回退概率（precedent），但 line-box 高度是多行文本累积 diff 主导，与 baseline 单点不同，值得一次实证。

---

## 1. 现状字体后端架构（一手事实）

### 1.1 职责分裂

| 职责 | 实现 | crate/文件 | feature |
|------|------|-----------|---------|
| 字体加载（sfnt parse） | fontdue `Font::from_bytes` | render-foundation/font/loader.rs | default |
| Line metric（ascent/descent/line_gap） | fontdue `line_metrics_full` | layout-engine/inline/font_metrics.rs（R885 font-bridge） | default（**dormant**） |
| Advance width（layout IFC 估算） | `estimate_char_width`（启发式 0.6×fs） | layout-engine/inline/ | default |
| 光栅化（glyph bitmap） | **FreeType** `FT_Render_Glyph`（非-Ahem 主）+ fontdue 回退 | render-foundation/font/loader.rs `rasterize_glyph` | `freetype-raster`（R1159 default-on） |
| Shaping（GSUB/GPOS） | rustybuzz（shaper.rs，**零渲染路径调用**） | render-foundation/font/shaper.rs | default（**dead**） |

### 1.2 FreeType 路径细节（loader.rs:16-94 `freetype_raster` mod）

- 每 glyph 创建 `freetype::Face`（`new_memory_face`）→ `set_char_size` → `load_glyph` → `render_glyph`。
- 已提取 `advance = glyph.advance().x / 64.0`（loader.rs:81）+ bitmap_left/bitmap_top 坐标。
- R1159 验证：welcome -0.28pp + css-text +24，零回归。

> **📌 来源说明（§1）**
> - 一手事实：loader.rs / font_metrics.rs 源码 + master.md R1159/R885。

---

## 2. font-wall metric 根因（一手事实 + 推理）

### 2.1 line-box metric 发散

ZW line-box 高度（默认 line-height:normal）经 fontdue `line_metrics_full`（ascent/descent/line_gap）。fontdue tight-ink metric ≠ chromium（FreeType `face.metrics` ascender/descender + OS/2 sTypoLineGap）→ 多行文本 line-box 累积发散（welcome/morning 行间距 + 文本垂直定位 diff，R633 谱系）。

### 2.2 为何 R834-R891 single-point 全 net-negative

| 轮 | 切片 | 结果 | 为何不收敛 |
|----|------|------|-----------|
| R1090 | fontdue per-font ascent + store-gate + 0.928 | -29 | fontdue metric ≠ chromium |
| R1095 | FreeType per-glyph bitmap_top | net-negative | per-glyph offset 破 compensating-errors |
| R1160 | FreeType + 常数 0.928 | -1 | 常数非 per-font |
| R891 | baseline_offset（概念②真路径） | -0.01pp negligible | baseline 单点，非 line-box 高度 |

**💡 推理**：四证均**未直接测「FreeType per-font line-box 高度（ascent+descent+line_gap）」**。R1090 是 fontdue（错源），R1095 是 per-glyph（非 line-box），R1160 是常数（非 per-font），R891 是 baseline 定位（非 line-box 高度）。line-box 高度是多行累积 diff 主导，与单点 baseline 不同量级。

### 2.3 R890 wiring no-op 根因

R890 wire font-bridge（fontdue metric）+ override-map → welcome 16.11% **一字不变**。诊断：paint Path B 用空 styles（R72）→ font_metric_provider 无法解析 family → fallback 0.8。R890 发现 bypass（`store_font_sizes_from_ifc` override-map 模式）但**未完成填充+消费接线**。故 FreeType/任何 per-font metric 经 override-map 的真实效果**未被测过**。

> **📌 来源说明（§2）**
> - 一手事实：master.md R1090/R1095/R1160/R891/R890 + font_metrics.rs。
> - 💡 推理：四证未覆盖 line-box 高度切片。

---

## 3. FreeType metric 合并方案（作者综合）

### 3.1 改动点

1. **per-font FreeType face 缓存**：freetype_raster 现每 glyph 建 face（慢）。加 per-font_id `Face` 缓存（loader.rs），供 metric + raster 共用。
2. **FreeType metric 提取**：`face.metrics().ascender / .descender / .height`（font units，scale × size/upem）→ `LineMetrics`。
3. **font-bridge 切 FreeType**：font_metrics.rs `FontLoader::line_metrics` 改用 FreeType metric（替代 fontdue `line_metrics_full`）。
4. **R890 override-map 完成接线**：layout IFC 经 provider 算 per-font line-box 高度 → 存 `text_node_ascent_ratios`/`line_height_overrides`（store_font_sizes_from_ifc 模式）→ paint 经 `ascent_ratio_for`/`line_height_overrides` 消费（绕空 styles gate）。
5. **env gate**：`FT_LINE_METRIC=0` 回退 fontdue（A/B + 紧急回滚）。

### 3.2 范围/风险

- **范围**：render-foundation/font/{loader,font_metrics} + layout-engine/inline/{font_metrics,mod,inline_finalization} + paint text.rs override-map 消费。~6 文件，每 < 80 行。
- **风险**：① R834-R891 precedent（高回退概率）；② override-map 接线 subtle（R890 no-op 教训）；③ per-font face 缓存内存；④ line-box 高度变 → 多行布局重排（welcome/morning 行数可能变 → 大 blast radius）。
- **sentinel**：welcome product-smoke（DC-13，diff ≤ 20%）+ css-text oracle（loose 不降）+ self-source 不降。

### 3.3 A/B 验证计划

- **主变量**：welcome diff（line-box 高度变 → 多行重排，最敏感）+ css-text/css-text-decor oracle pass-rate（多行文本累积 diff 主战场）。
- **判定**：net ≥ 0（welcome 持平或改善 + 文本类 oracle 不降）即留作 dormant foundation；net < 0（welcome 退步，疑多行重排）即关 gate 回退，记录为 font-metric 第 N 证。
- **预期**：line-box 高度对齐 chromium → 多行文本行间距 diff 缩小 → welcome/morning 改善（区别 R891 baseline 单点 negligible）。但 blast radius 大（多行重排），回退概率 ≥ 50%。

> **📌 来源说明（§3）**
> - 作者综合：基于 §1 现状 + §2 根因设计的合并方案。

---

## 4. go/no-go 裁决

**tentative GO** 一个窄 A/B 实验（§3 方案，env-gated）。

**理由**：
- ✅ 唯一 en masse font-wall lever（line-box 高度是多行文本累积 diff 主导，区别 baseline 单点）。
- ✅ FreeType 路径已在 ZW 验证（R1159 raster +232），metric 是同 face 另一查询，技术低风险。
- ✅ 四证 precedent 未直接覆盖此切片（§2.2 推理）。
- ⚠️ 高回退概率（R834-R891 font-metric 谱系 + R890 no-op + 多行重排 blast radius）。
- ⚠️ 若 net-negative，记录为 font-metric 第 N 证，font-wall 确认彻底死墙 → rally 维护态。

**前置**：先完成 R890 override-map 接线（populate + consume），用 fontdue metric 验证 wiring 通（welcome 应有变化，区别 R890 no-op），再切 FreeType metric。两步分离降风险。

---

## 5. 结论

font-wall（ZW corpus 41.3% → 95% 主导 gap）唯一 en masse lever = FreeType line metric 合并。技术上 R1159 已铺路（FreeType raster），metric 合并可行（~6 文件，env-gated）。R834-R891 precedent 强（font-metric net-negative 谱系），但 **line-box 高度切片未被直接测过**，且是多行累积 diff 主导（与 baseline 单点不同量级）。建议一次窄 A/B 实验（高回退概率），若 net-negative 则 font-wall 确认死墙、rally 转维护态；若 net ≥ 0 则作 foundation 逐步扩。

---

## 6. forward（外部 landscape + 实施）

- **外部 deep-research（建议下轮）**：Rust 字体后端 landscape（skrifa/swash/cosmic-text font stack/Servo 字体栈 vs fontdue+FreeType），评估是否有比「fontdue+FreeType 合并」更优的统一后端（如 skrifa 提供 metric+raster 一致源）。本评估聚焦内部可行性，未覆盖外部选型。
- **实施（若 GO）**：按 §3.3 两步（先 fontdue wiring 通，再切 FreeType metric），每步 A/B 守 welcome + css-text。
- **若 net-negative**：font-wall 确认死墙（第 N 证），rally 转低频维护，待重大架构投资（如 skrifa 全替换）决策。

---

## 参考资料

| # | 来源 | 类型 | 引用章节 |
|---|------|------|----------|
| 1 | crates/render-foundation/src/font/loader.rs（fontdue + freetype_raster mod） | 一手事实 | §1 |
| 2 | crates/layout-engine/src/inline/font_metrics.rs（R885 font-bridge） | 一手事实 | §1, §2 |
| 3 | master.md R834/R849/R876/R885/R890/R891/R1060/R1090/R1095/R1160 | 一手事实 | §2 |
| 4 | master.md R1159（freetype-raster default-on +232） | 一手事实 | §1, §3 |
