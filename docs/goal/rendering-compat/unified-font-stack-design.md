# 设计：Unified Font Stack（fontdue+启发式 → chromium 对齐字体栈）

**版本**：v0.2（R1560/R1634 refresh）
**日期**：2026-07-08（v0.1）/ 2026-07-18（v0.2 refresh）
**作者**：AI Assistant（rendering-compat rally）
**状态**：scoping 文档（bounded rally exhausted 后的唯一 forward path；multi-month 架构投资）
**模式**：rally-pattern 设计文档（非 lei-spec-rfc skill —— 该 skill 需用户确认，与无人值守 rally 协议冲突；同 multicol Phase 2 spec 先例）
**关联**：master.md R1174/R1175/R1180/R1181/R1560/R1634；[`research-chromium-lineheight-normal-formula-2026-07-08.md`](./research-chromium-lineheight-normal-formula-2026-07-08.md)；[`research-font-backend-freetype-metric-consolidation-2026-07-08.md`](./research-font-backend-freetype-metric-consolidation-2026-07-08.md)；[`font-wall-cdep-scoping.md`](./font-wall-cdep-scoping.md)；[`skia-cdep-rfc.md`](./skia-cdep-rfc.md)

> **📍 v0.2 refresh（2026-07-18，R1560+R1634）**：① **R1560 实测 chromium 实际光栅器 skia-safe 0.80 A/B = net-negative**（css-text oracle −24，font-wall 簇真回归）→ **font-wall 不在光栅化 coverage**（FreeType 已 default-on R1159 +232，光栅层已对齐 chromium 的 FreeType）**，真在 layout/metric coherence**（advance 启发式 0.6 vs 实际 + generic/explicit line-height 区分）——**强化 §1「光栅已对齐」结论，光栅 C-dep 路径（Skia/HarfBuzz）全角度穷尽 ruled out，勿再试**。② **R1634 增量 reftest lever 彻底穷尽**（R1626-R1634 九轮：tables-overlap abandoned / float 005 多层 entangled / fresh scan plateau / 唯一净 land = R1626 color override net-0 spec-correct）→ **font-stack rebuild 是 rendering-compat 唯一剩余 major-arch unlock**，当前 reftest ~57% = 2026-07-17 用户裁决接受的阶段性 plateau。③ **directive 状态**：2026-07-17「font-stack rebuild 先列 RFC 不直接开工」→ 本 scoping RFC 已就绪（§3 分阶段 + §5 首切片），**implementation 待用户解锁**（R1634 飞书告知请裁决：① 解锁首切片 OR ② accept plateau）。**孤立 slice 均 net-negative 证伪**（R225/R375 advance / R1185 1.150 / R1560 Skia），须 C2'(per-font generic-vs-explicit metric)+C3(advance) 协同才 yield → multi-week。

> **📍 v0.2.1 addendum（2026-07-18，R1636）**：用户 directive 强化「不管遇到什么问题都要想办法解决，不要阻塞」→ 不把 font-stack 当待裁决阻塞，主动开工最小可测切片。**首切片 C2'（generic→1.150）post-R1263 重测 = R1185 结论 NOT stale，二次证伪**：A/B welcome @800 baseline 81512px/16.98% → treatment（`ZW_GENERIC_LH_1150=1`）83415px/17.38% = **+0.40pp 回归**（≈R1185 +0.39pp，几乎完全一致）。post-R1263 字形已对齐（sans-serif→Liberation Sans = chromium）但 metric 1.150 仍回归 → 根因 = welcome diff **由光栅化噪声主导**（FreeType-ZW vs FreeType+Skia-chromium 同字 Liberation Sans），1.2% line-height 扰动放大噪声。**font-wall line-height 常数 lever 三证收官**（R1175/R1185/R1636），1.164 全局最优，**C2' 永久关闭勿再试**（含 isolated generic-only）。代码已 revert（违「精准修改」不留二次证伪 dormant 死代码）。详见 [`evidence/r1636-...txt`](./evidence/r1636-c2-generic-1150-post-r1263-retest-refuted-2026-07-18.txt)。**forward 收窄**：line-height 层全闭；剩余 = **C3 advance-wall（R225/R375 refute 亦 pre-R1263 DejaVu-only，post-R1263 Liberation Sans advance 或匹配 chromium，stale-待复检；但 advance 非单 env-gate，须 plumbing）+ per-font 度量 coherence**，须 U1b-wiring（dormant FontLoader provider 接线，data-map 方案避 Rc-restructure）+ C1/C2/C3 协同。下轮 CONTINUE U1b-wiring dormant 首切片（rally 协议不阻塞）。**方法论沉淀**：pre-R1263 refute 结论须 post-R1263 重测（C2' 已复检证伪）；welcome diff 是度量 lever 灵敏**负向**指示器但**不可作正向 yield 证据**（光栅噪声主导）；度量 yield 须在**纯 Ahem corpus**（光栅噪声=0）验证。

> **📍 v0.2.2 addendum（2026-07-18，R1637–R1647）**：U1b-wiring + C3 font_id **dormant infra 全 LANDED**（零回归 verified），但 **R1639 战略结论 = font-stack 低 EV 端到端**，font-stack 不再作 active 推进方向，dormant infra 保留待用户授权 rebuild 时激活。★ **U1b-wiring 双路径 LANDED（dormant）**：R1637 切片 A（stored IFC 路径：`LayoutEngine.font_metric_provider` 字段 + setter + `compute_final_inline_layouts` provider 参数 + 递归 thread + 主 stored-IFC 站点 `.with_font_metric_provider`）+ R1638 切片 B（measure 路径：`measure_text_content` / `remeasure_text_with_float_exclusions` 加 provider 参数 + engine.rs 4 measure 调用点 thread）= **stored+measure 双路径 font_metric_provider threading 完整**，激活不再有 box-高 vs paint-行高 mismatch trap。★ **C3-enabling LANDED（dormant）**：R1639 `FontMetricProvider` trait 加 `font_id_of(family)->Option<u32>` + `FontLoader` impl（大小写不敏感）+ IFC `font_id_for_style` helper + 3 TextRun 构造点 populate `font_id`（替原 `None`）= 解 R223 font_id gap，advance_of 注入 AdvanceSource 时收真 font_id。三切片均 dormant（provider 默认 None → 字节等价旧路径，单测证机制 + make test 全绿 + product-smoke welcome 81512px/16.98% 字节一致）。★ **R1639 战略发现 = font-stack 三子杠杆全低 EV / blocked**：① **per-font line-height**（U1b 激活目标 1）：R1636 二次证伪（welcome 光栅噪声主导，post-R1263 Liberation Sans 字形已对齐但 1.150 metric 仍 +0.40pp 回归）；Ahem corpus per-font 度量 = 常数（无变化）→ 激活预期负向。② **C3 advance**（U1b 激活目标 2）：R225/R375 refute pre-R1263 DejaVu-only；**corpus 主体是 Ahem**（advance=font_size 精确，无 wall）→ 仅非-Ahem 子集受影响，且该子集光栅噪声 entangled；激活需 `AdvanceSource` 注入 = **FontLoader Rc-share**（reftest runner font_loader 被 painter `&mut` 占用，不可 Rc-share，data-map 方案不适用 advance 需字体字节）= 昂贵 refactor。③ **光栅化 C-dep**：R1560 real-Skia net-24 ruled out（font-wall 不在光栅化 coverage）。⇒ **font-stack rebuild 对 corpus pass-rate 是低 EV 路径**（line-height/advance/raster 三子杠杆全低 EV 或 blocked），font-wall 残余仅在非-Ahem 产品 fixture（welcome/morning/wintertc ~17%）且光栅噪声主导。★ **R1640–R1646 全 lever A/B 穷尽闭环**：per-font line-height activation nil（R1640）+ borders net-negative（R1642）+ vertical-mode 4 角度 net-negative（R1644/R1645/R1646）+ multicol frontier non-viable（R1593/R1594）+ struct-sweep 收口（R1576/R1578）+ 6 角度 plateau（R1579–R1585）= **in-environment rendering-compat DEFINITIVELY exhausted**（reftest ~57% = 2026-07-17 用户裁决接受的阶段性 plateau）。★ **裁决（v0.2.2）**：font-stack rebuild 作为「唯一理论 major-arch unlock」**需修正为低 EV**——对 corpus pass-rate 它是非 productive 路径（dormant infra 已就绪但激活路径三子杠杆全证伪/低 EV）。**dormant U1b/C3 infra 保留**（零回归 verified，待用户若授权 font-stack rebuild 时激活，复用 thread 模式 cheap）；**不主动推进 font-stack**（违 code-guidelines「不做零价值修改」+ R1639 低 EV 结论）。唯一剩余 forward = 待用户侧 chromium 环境修复（WSL2 SIGTRAP）解锁 legacy-html fixture oracle（用户 2026-06-26 优先级 DC-13 Tier 1）+ fresh corpus oracle 重抓，或用户给新渲染方向。

---

## 0. 执行摘要

> **⚠️ R1185 重大修正（reframe C1/C2）**：puppeteer 直测 chromium line-height:normal 揭示 **generic vs explicit family 区分**——generic sans-serif/serif = **1.150**（Blink 内部默认，非 resolved 字体度量；fc-match→NotoSansCJK 但 generic 渲染 1.15 ≠ NotoSansCJK explicit 1.45，像素指纹独立）/ explicit DejaVu 1.170 / NotoSans 1.360 / NotoSansCJK 1.450。fontdue hhea 对 **explicit** 字体精确匹配 chromium（DejaVu 1.1641 / NotoSans 1.3620 / NotoSansCJK 1.4480，探针实证），但 **generic family 走 Blink 默认 ~1.15**。**★ refute 原 C1 naive font-swap**：corpus 非-Ahem 多用 generic sans-serif（chromium 1.15），ZW font-swap→NotoSans + per-font hhea 1.36 必与 chromium generic 1.15 diverge（+18% 行高，灾难）。**★ refute 1.150 常数 A/B**：1.164→1.150 = css-text-decor +2 / normal-flow neutral / **welcome +0.39pp 回归**（81433→83347 px，trade 0.195 pp/flip 差于 R1175 的 1.164 0.136 pp/flip）→ **1.164 仍是全局最优**（介于 generic 1.15 与 per-font explicit 之间）。**修正后真路径**：C2' = per-font 度量须**区分 generic（~1.15 Blink 默认）vs explicit（fontdue hhea）**——替代原 C1 naive font-swap。font-wall line-height 常数 lever 收官（headroom 极小）。详见 master.md R1185 + [`evidence/r1185-chromium-generic-vs-explicit-lineheight-2026-07-08.txt`](./evidence/r1185-chromium-generic-vs-explicit-lineheight-2026-07-08.txt)。

> **为什么是这个**：bounded 增量 rally 已 exhausted（S1180/R1181 hard plateau definitive）。剩余 corpus gap（实际 ~45-48% oracle → 95% 目标）**主导 = font-wall**：ZW 字体栈（fontdue 加载 DejaVu + 启发式 advance 0.6 + 常数度量）≠ chromium 字体栈。R1175（line-height 1.164）+ R990（ascent 0.928）+ R1159（FreeType raster）对齐了 DejaVu 的常数度量，但 **advance 启发式（0.6 vs 实际）+ generic/explicit 度量区分** 两层未解，是 advance-wall / Phase A core / welcome 16.97% 的共同 root。

- **一句话目标（R1185 修正）**：把 ZW 字体栈统一到「chromium 对齐：per-font 度量**区分 generic（~1.15）vs explicit（fontdue hhea）** + 实际 advance in layout」，消除 advance-wall。**font-swap 非目标**（R1185 refute：generic family chromium 不用 resolved 字体度量）。
- **为什么 multi-month**：font-swap alone 经 R1180 实测 +1（~neutral）+ R1185 进一步 refute（generic/explicit 度量区分使 font-swap 反向）→ 须 per-font 度量 + generic/explicit 区分 + advance 协同才 yield，无 narrow yielding slice。
- **核心约束**：① 零回归（每 slice env-gated + A/B 守 welcome <20% + corpus oracle 不降）；② chromium-Oracle z_vs_chr 门禁；③ 单 `.rs` ≤2000 行；④ test-guard 包裹。

---

## 1. 根因（一手事实，R1174/R1175/R1180 确证）

| 层 | ZW 现状 | chromium 实际 | gap 后果 |
|----|---------|--------------|---------|
| 字体发现 | 硬编码路径加载 DejaVuSans + Ahem；sans-serif→DejaVu（loader.rs:246 sans_names 首选） | fontconfig 解析 sans-serif→NotoSansCJK SC（`fc-match sans-serif` 实测） | **font-mismatch**：welcome/corpus 非-Ahem 文本 ZW=DejaVu, chromium=NotoSansCJK，字形/度量不同 |
| 度量（line-height/ascent） | 常数（R1175 line-height 1.164 = DejaVu hhea；R990 ascent 0.928） | per-font hhea（DejaVu 1.164, NotoSans 不同） | font-swap 后 line-height 常数 confound（R1180：NotoSans 按 DejaVu 1.164 渲染 → +1 非 en masse） |
| advance（layout 换行） | `estimate_char_width` 启发式（非-Ahem Latin 0.6×fs） | 实际 advance（FreeType/HarfBuzz hmtx） | **advance-wall**（R627）：换行决策 ≠ chromium → 行数差 → 垂直累积偏移（welcome/morning 主因，R804 PIL 实测） |
| 光栅 | FreeType per-glyph（R1159 default-on）+ fontdue 回退 | FreeType/Skia per-glyph | 已对齐（R1159 +232） |

**关键证伪链**（为何 narrow 失败）：
- R225/R375：accurate DejaVu advance in layout → net-negative（DejaVu advance ≠ chromium NotoSansCJK 行数）。
- R803：font-swap NotoSansCJK（TTC）on welcome → -0.06pp（advance 未变，仅字形）。
- R1180：font-swap NotoSans on corpus → +1 css-fonts（line-height 常数 confound）。
- R627/R890：per-font metric 经 override-map wiring → no-op（paint Path B 空 styles gate）。
- → 三层（字体 + 度量 + advance）**须协同**，单层 zero-yield。

---

## 2. 目标组件（4 层，按依赖序）

### C1：字体发现对齐 fontconfig
- **现状**：硬编码 DejaVu/Ahem 路径（reftest_fonts.rs:22, renderer main.rs:1024）。
- **目标**：fontconfig-style 发现——枚举系统字体，按 CSS font-family 优先级匹配 chromium 同款（sans-serif→NotoSansCJK 或系统默认 sans）。
- **TTC 支持**：NotoSansCJK 是 .ttc（多 face），fontdue `from_bytes` 不支持 → 须 TTC 首选 face 提取器（R803 `extract_ttc_first_face` 已证技术可行，revert 非技术原因）。
- **范围**：~150 行（TTC 提取 + 字体发现枚举 + family 匹配）。

### C2：font-bridge 真触达 layout（per-font 度量）
- **现状**：R885 font-bridge（`FontMetricProvider` trait + IFC `font_metric_provider` 字段）dormant（R1005 零生产读取；R890 wiring no-op，paint Path B 空 styles gate）。
- **目标**：font-bridge 经 `store_font_sizes_from_ifc` override-map 模式（line_heights 字段已存 frag.height）真触达 layout + paint，使 line-height/ascent per-font（DejaVu 1.164/0.928，NotoSans 其本值）。
- **关键**：R890 发现的 bypass = `store_font_sizes_from_ifc`（inline_finalization.rs:330）已存 `text_node_line_heights`（per-frag height）；须让 layout IFC 用 font-bridge 算 per-font line-height（替 resolve_font_metrics 常数），存入 override-map，paint 消费。
- **范围**：~5 站点（compute_final_inline_layouts）wiring + override-map populate/consume，~200 行。

### C3：实际 advance in layout（替 0.6 启发式）
- **现状**：`estimate_char_width`（text_metrics.rs:40-58）启发式 0.6×fs（非-Ahem Latin）。
- **目标**：layout 换行用 fontdue/FreeType 实际 advance（hmtx），与 paint 同源。
- **风险**：R225/R375 证 accurate DejaVu advance net-negative（DejaVu ≠ chromium）；**仅当 C1（font=chromium 同款）+ C2（per-font 度量）协同后**，实际 advance 才匹配 chromium 行数。
- **范围**：advance-width plumbing（R223 trait seam 已有），~150 行 + 严格 A/B（advance 是换行决策，blast radius 大）。

### C4：FreeType per-font face cache（性能 + 度量同源）
- **现状**：FreeType per-glyph `new_memory_face2`（loader.rs:43，慢，无度量提取）。
- **目标**：per-font_id `Face` 缓存，供 raster + 度量（C2）共用。
- **范围**：~80 行（Face 缓存 + metrics 提取 ascender/descender/height）。

---

## 3. 分阶段计划（narrow slices + gates，每 slice env-gated 零回归）

> **原则**：每 slice **dormant-enabling**（默认关，零回归）或 **A/B net-positive 才留**。区别 R885/R900——本栈组件**单独 zero-yield**，须 C1+C2 协同首 yield（R1180 实证）。

### Phase U1：C1+C2 协同首 yield（font-swap + per-font 度量）
- **slice U1a**（dormant，低优先）：C1 TTC 提取器（R803 恢复）+ NotoSansCJK 加载 + fontconfig-style 发现（env `ZW_FONTCONFIG=1`）。**R1182 重排**：Latin 用 NotoSans-Regular.ttf 单 .ttf（无须 TTC），welcome font-swap -0.06pp（R803）→ U1a TTC 仅 CJK 后续，低优先。
- **slice U1b-core**（✅ LANDED R1184，dormant）：C2 首消费者——`resolve_font_metrics_with_provider` 在 layout IFC 2 调用点（text_metrics.rs + inline/mod.rs:602/887）消费 `font_metric_provider`，使 line-height:normal 用 per-font `ascent−descent+line_gap`。**关键洞察**：line-height override-map bypass 链路（`store_font_sizes_from_ifc` → `text_node_line_heights` → paint `with_line_height_overrides`）已完整存在，本 slice 仅改 line-height 源，per-font 值经既有链路自动触达 paint，绕 R890 空 styles 阻塞。provider None（生产默认）逐字节等价 `resolve_font_metrics` = 零回归（单测 `assert_eq!` + make test 12212/0 + product-smoke welcome 81433 px 精确一致 R1175）。4 新单测证 provider 被咨询。
- **slice U1b-wiring**（dormant，下一会话）：5 层 FontLoader 接线（app 创建 → WebView/RenderPipeline → LayoutEngine::set_font_metric_provider → compute_with_img_sizes → compute_final_inline_layouts 5 IFC 站点 `.with_font_metric_provider`）。R887 测绘路径，R889/R890 实验 revert clean。注入后 U1b-core 自动激活（per-font line-height 经既有 override-map 链路生效）。
- **slice U1c**（A/B，首 yield 测试）：U1b-wiring + font-swap（NotoSans-Regular.ttf + env `ZW_SANS_NOTO` resolve sans-serif→NotoSans）→ sans-serif→NotoSans + per-font line-height（NotoSans 本值，解 R1180 confound）+ advance 仍 0.6 → A/B corpus oracle（css-fonts/css-text）+ welcome。
  - **判定**：若 net-positive（>R1180 的 +1）→ font-swap + per-font 度量 yield（advance 非必需），留；若 ~neutral → 须 C3 advance 协同。

### Phase U2：C3 实际 advance（解 advance-wall）
- **前提**：U1 yield（font=chromium 同款）后，accurate advance 才匹配 chromium 行数（R225/R375 DejaVu-only 反例不再适用）。
- **slice U2a**（A/B）：advance 改 fontdue/FreeType hmtx，env-gated → A/B corpus + welcome。
- **判定**：net-positive → 解 advance-wall，welcome/morning 行数对齐；net-negative → advance 仍非 root（须再查）。

### Phase U3：C4 per-font face cache（性能 + 收尾）
- **slice U3a**：C4 Face 缓存（raster + 度量共用），性能 + 度量同源收尾。

---

## 4. 风险

| 风险 | 缓解 |
|------|------|
| Multi-month（无 narrow yielding slice） | 每 slice env-gated + A/B；U1c 是首 yield 决策点（若 ~neutral 须重评） |
| advance blast radius（换行决策全局影响） | U2a env-gated + 严格 A/B 守 welcome + corpus；R225/R375 先例 |
| fontconfig 跨平台差异（macOS/Windows） | C1 env-gated，默认关（CI 纯 Rust 不受影响）；fontconfig 仅 Linux |
| fontdue FreeType 度量不一致 | C4 per-font Face 缓存使 raster + 度量同源（FreeType） |
| R890 wiring no-op 重现 | U1b 用 store_font_sizes_from_ifc override-map bypass（R890 已识别） |

---

## 5. 第一步（下会话）

**slice U1b-core**（✅ LANDED R1184，2026-07-08）：font-bridge 首消费者（`resolve_font_metrics_with_provider` + layout IFC 2 调用点 + 4 单测），dormant 零回归。详见 [`evidence/r1184-u1b-core-per-font-lineheight-first-consumer-2026-07-08.txt`](./evidence/r1184-u1b-core-per-font-lineheight-first-consumer-2026-07-08.txt)。

**slice U1b-wiring**（下一会话）：5 层 FontLoader 接线（R887 测绘/R889/R890 revert clean），注入 provider 后 U1b-core 自动激活。**单独不可验证 yield**（须 U1c font-swap 协同）。

**slice U1c**（U1b-wiring + font-swap，首 yield A/B）：恢复 R1180 NotoSans-Regular.ttf 加载 + env `ZW_SANS_NOTO`，A/B corpus oracle（css-fonts/css-text）+ welcome，**net-positive 才留**（>R1180 +1）。

---

## 6. 何时止步（kill conditions）

- **U1c ~neutral**（≤ +3 corpus）+ U2a net-negative → font-stack 非 root，rally 真维护态（R1180/R1181 plateau 终局）。
- **U1c net-positive** → 本栈是 forward，按 Phase U2/U3 推进。
