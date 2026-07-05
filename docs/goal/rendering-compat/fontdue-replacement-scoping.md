# Scoping：fontdue → chromium-matching 字体光栅化替换

**版本**：v0.1（scoping，待 lei-spec-rfc 升级为完整 RFC + lei-deep-research 候选对比）
**日期**：2026-07-06
**作者**：AI Assistant（rally R1064）
**状态**：scoping 草稿（rally 自主模式；假设显式标注；待用户确认范围/优先级）

> 本 doc 把 R1056（CJK ascent 第六证）+ R876（fontdue tight-ink 三方补偿）+ R1064（font-wall 笼罩 CSS2 测试四证穷尽）收敛为可实施的多会话计划前置 scoping。R1064 确证：rendering-compat clean single-session lever 已穷尽，残余失败 100% 受阻于 fontdue≠chromium 字体墙。

---

## 0. 执行摘要

- **一句话目标**：把 ZeroWeb 字体加载/度量/光栅化从 `fontdue 0.9` 替换为与 chromium 字体管线（FreeType + Skia）像素级匹配的方案，消除 1-3% font-wall diff baseline，unblock css-backgrounds/borders/box-display/text/text-decor 等簇 + welcome/morning 产品页残余。
- **驱动**：R1064 实证 WPT css21 测试标准化 `<p>` 指令文本 fontdue vs chromium 字形渲染差贡献 ~1-3% diff baseline，笼罩全 CSS2 测试（clean lever 四证穷尽）；R1056/R876 实证 layout-side font-metric 单点改动 net-negative（六证），须 fontdue 替换 + 全 pipeline coherence。
- **明确排除**：font-engine 完全重写（仅替换光栅化 + 度量 API）；字体子集化/加载策略变更；多进程字体沙箱。
- **核心约束**：① 不得回归 Ahem WPT 测试（当前 ~80% pass，fontdue 对 Ahem 方块度量精确）；② 单个 .rs ≤2000 行；③ 须保持 Linux/macOS/Windows 跨平台；④ 须 A/B 零回归 + product-smoke <20%。
- **候选方案**（待 lei-deep-research 对比）：① **FreeType via `freetype-rs`**（chromium Linux 同栈，理论像素级匹配；C 依赖）；② **`swash`**（纯 Rust，font-kitin/MSDF 风格，chromium 接近度待测）；③ **`ab_glyph`**（纯 Rust，API 简洁，精度待测）。
- **首个落地步骤**：先用 lei-deep-research 对比三候选的 chromium 像素一致度（在 ZeroWeb 现有 fontdue 调用点做 prototype，对同一字体同一字形渲染 chromium vs 候选，像素 diff 排序），再定首选 + 起 RFC。

---

## 1. 背景与目标

### 1.1 背景

ZeroWeb 当前字体栈：`crates/render-foundation/src/font/loader.rs` 用 `fontdue 0.9` 加载 sfnt/WOFF 字体（R1006 WOFF 解码），提供：
- `fontdue::Font::from_bytes`（加载）
- `fontdue::Font::rasterize`（字形光栅化 → alpha bitmap，Skia 替代）
- `fontdue::Font::horizontal_line_metrics` / `line_metrics_full`（ascent/descent/line_gap，R847 暴露 line_gap）

fontdue 是纯 Rust、tight-ink 包围盒光栅化器。与 chromium（Linux 用 FreeType + Skia，macOS 用 CoreText，Windows 用 DirectWrite + Skia）的字体管线在两处发散：
- **光栅化位图**：fontdue tight-ink vs FreeType/Skia AA，per-glyph alpha bitmap 像素差。
- **行度量**：fontdue OS/2/hhea vs chromium 同源，但 line-height:normal 计算 + half-leading 分布 + baseline 位置累积发散（R834/R836/R849/R875/R1052/R1056 六证）。

### 1.2 真实代价（R1064 量化）

WPT css21 测试标准化 `<p>Test passes if...</p>` 指令文本。fontdue vs chromium 对该文本的字形渲染差贡献 **~1-3% diff baseline**，笼罩全 CSS2 测试：
- backgrounds-001-022 簇 1.15% identical（~10 案同指令文本）
- backgrounds-026-053 簇 1.46% identical（~6 案）
- border-top-width-012-078 簇 3.10% identical（7 案同 "Filler Text" 文本）

case 须 layout diff > ~0.5% 才有 fix 后越过 font-wall <1% 阈值的可能。backgrounds/borders 簇 layout diff≈0（纯 background/border 正确）故全卡 font-wall。welcome/morning 产品页 16-17% / 13% 残余同样主因 fontdue（R174 证）。

### 1.3 目标

- **业务目标**：css-backgrounds/borders/box-display/text/text-decor chromium-Oracle 一致率显著提升；welcome product-smoke <5%。
- **用户目标**：水平 CJK 文本 + 产品页（morning-work）渲染与 chromium 视觉一致。

### 1.4 范围边界

- **在范围内**：
  - `render-foundation/src/font/` FontLoader 的 fontdue 调用替换（from_bytes / rasterize / line_metrics）。
  - 度量管线 coherence（layout-engine inline/font_metrics.rs + paint v_offset + half-leading 三方同改，R848 路线图）。
  - 逐案 A/B 守 Ahem WPT + product-smoke。
- **不在范围内**：
  - 字体子集化 / 懒加载策略变更。
  - 多进程字体沙箱 / 跨进程字体缓存。
  - shaping 引擎替换（harfbuzz 集成，独立多 session）。
  - macOS CoreText / Windows DirectWrite 原生后端（统一用跨平台光栅化器）。

### 1.5 关键假设（待验证）

- **假设 A1**：`freetype-rs`（绑定 FreeType 2）在 Linux 上能像素级匹配 chromium（同 FreeType 库 + 同字体）；macOS/Windows 用 FreeType 也能接近（chromium 在那些平台用原生 + Skia，非纯 FreeType，但 Skia 光栅化模型接近）。
- **假设 A2**：`swash`（纯 Rust）的光栅化对拉丁/CJK 字体精度 > fontdue，接近 chromium 度（须 prototype 测）。
- **假设 A3**：替换 fontdue 的度量为 FreeType/Skia 同源（OS/2 sTypo + hhea）后，layout strut_ascent + paint v_offset + half-leading 三方用真实度量 coherence，可消除 fontdue≠Skia 累积发散（R848 路线图）。
- **假设 A4**：Ahem 字体在三候选下光栅化与 fontdue 一致（Ahem 是简单方块字体，光栅化器差异最小），故不回归 Ahem WPT。

---

## 2. fontdue API surface（待替换）

`crates/render-foundation/src/font/` 调用点（grep 实证）：

| fontdue API | ZW 调用点 | 用途 | 替换复杂度 |
|---|---|---|---|
| `Font::from_bytes(bytes, FontSettings)` | `loader.rs`（load_font）+ `woff.rs:214`（WOFF→sfnt 后加载） | 字体加载 | 低（三候选都支持 from_bytes） |
| `Font::rasterize glyph, size` | `loader.rs::rasterize_glyph`（非 Ahem 路径）+ `rasterize_ahem_glyph`（Ahem 专用） | 光栅化 → alpha bitmap | 中（位图格式/坐标约定差异） |
| `Font::horizontal_line_metrics(size)` | `loader.rs::line_metrics` | ascent/descent（layout IFC strut + paint v_offset） | 中（返回结构 + line_gap 处理） |
| `Font::line_metrics_full(size)` | `loader.rs::line_metrics_full`（R847） | ascent/descent/line_gap（chromium line-height:normal） | 中 |
| `Font::metrics glyph` | `loader.rs`（glyph advance/边距） | 字形 advance（换行决策）+ 定位 | 中 |
| `Font::glyph_glyph_horizontal_advance` | `loader.rs` | 字形水平 advance | 中 |

**关键文件**：
- `crates/render-foundation/src/font/loader.rs` — FontLoader 主实现（fontdue 直接调用集中处）。
- `crates/layout-engine/src/inline/font_metrics.rs` — FontMetricProvider bridge（消费 line_metrics_full）。
- `crates/layout-engine/src/inline/text_metrics.rs` — text 估计（部分用 fontdue 度量）。
- `crates/engine/src/paint/painter/text.rs` — paint 字形定位（消费 rasterize + 度量）。

---

## 3. 候选方案对比（待 lei-deep-research 实证）

| 候选 | 类型 | chromium 一致度（理论） | C 依赖 | 维护度 | 风险 |
|---|---|---|---|---|---|
| **`freetype-rs`**（绑定 FreeType 2） | C 绑定 | ★★★★★（Linux 同栈 FreeType + Skia 光栅化模型） | 是（FreeType 2，跨平台） | 高（FreeType 工业标准） | C 依赖构建复杂度（CI 三平台）；macOS/Windows chromium 用原生后端非纯 FreeType，仍可能差 |
| **`swash`** | 纯 Rust | ★★★☆（待测；font-kitin/CosmicText 用） | 否 | 中（CosmicsText 生态） | 光栅化模型可能与 Skia 差；CJK shaping 支持有限 |
| **`ab_glyph`** | 纯 Rust | ★★☆（API 简洁但光栅化简单） | 否 | 高 | 光栅化精度低（无 hinting/AA 精细控制）；可能不如 fontdue |

**推荐评审顺序**：① freetype-rs（最高一致度，验证 A1 是否成立）→ ② swash（若 freetype-rs C 依赖被拒）→ ③ ab_glyph（兜底）。

---

## 4. 推荐 multi-session 切片

| 切片 | 内容 | 验证 | 会话估算 |
|---|---|---|---|
| **Phase 0**（research） | lei-deep-research 对比三候选：在 ZeroWeb fontdue 调用点 prototype，对 Ahem + DejaVuSans + NotoSansCJK 同字形渲染 chromium vs 候选，像素 diff 排序，定首选 | 候选选定 + 像素 diff 数据 | 1-2 session |
| **Phase 1**（from_bytes + line_metrics） | 替换 from_bytes + line_metrics_full（最小 slice，光栅化仍用 fontdue），度量管线 coherence（layout strut + paint v_offset + half-leading 三方同改，R848 路线图） | welcome/morning/linebox/css-text A/B + Ahem WPT 零回归 | 2-3 session |
| **Phase 2**（rasterize） | 替换 rasterize_glyph（非 Ahem）+ rasterize_ahem_glyph（Ahem），paint 字形定位协调 | 全 CSS2 oracle A/B + product-smoke <20% | 2-3 session |
| **Phase 3**（清理） | 移除 fontdue 依赖，文档更新 | cargo build --workspace 绿 + clippy | 1 session |

---

## 5. 风险与开放问题

- **R1（FreeType C 依赖）**：CI 三平台（ubuntu/macos/windows）构建 FreeType 2；Linux 通常系统装，macOS/Windows 须 vendored 或 vcpkg。Cargo `freetype-rs` 的 `bundled` feature 可编译 C 源。**待用户决策**：是否接受 C 依赖换 chromium 像素级匹配。
- **R2（跨平台 chromium 一致度）**：Linux chromium 用 FreeType+Skia；macOS 用 CoreText；Windows 用 DirectWrite+Skia。单一光栅化器无法三平台像素级匹配 chromium。**建议**：Linux 优先（CI oracle 跑 Linux），macOS/Windows 接受 ~1% 差异。
- **R3（Ahem WPT 回归）**：Ahem 是简单方块字体，三候选光栅化应与 fontdue 一致（A4），但须 Phase 1 A/B 守。
- **R4（shaping 缺口）**：fontdue 无 shaping（连字/复杂脚本）。chromium 用 harfbuzz。替换光栅化器不补 shaping（独立多 session）。ZW 当前简单脚本（拉丁/CJK）不依赖 shaping，故非阻塞。
- **R5（layout-side font-metric 净负六证）**：Phase 1 度量管线 coherence 须**三方同改**（R848 路线图：layout strut_ascent + paint v_offset −real_ascent + half-leading (lh−(asc−desc))/2），单点改 net-negative（R834/R836/R849/R875/R1052/R1056 六证）。Phase 1 须严格按 R848 路线图全链协调，A/B 守 welcome/morning/linebox/css-text oracle，净负则回退。

---

## 6. 开放问题（须用户决策）

1. **是否接受 C 依赖（FreeType 2）换 chromium Linux 像素级匹配？** 若否，用纯 Rust（swash/ab_glyph），接受 ~1-2% 残余。
2. **平台优先级？** Linux（CI oracle）优先 → macOS/Windows 接受差异；还是三平台并重（须多后端）？
3. **是否启动 Phase 0 lei-deep-research 候选对比？** 还是先评估其他 ZeroWeb 目标（如 zero-web.md）优先级？

---

## 7. 下一步

- **若用户确认启动**：Phase 0 lei-deep-research（候选对比 + prototype 像素 diff）→ Phase 1 RFC（lei-spec-rfc 完整 RFC）。
- **若用户调整优先级**：本 scoping 入 archive，转其他目标。
- **当前 rally 状态**：rendering-compat clean single-session lever 四证穷尽，font-wall 结构性平台期，须用户决策方向。
