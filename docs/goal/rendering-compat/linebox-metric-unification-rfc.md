# RFC：行盒度量统一（baseline / ascent / half-leading 一次计算、paint 复用）

**版本**：v1.0（草稿，read-only 设计产出）
**日期**：2026-06-29（R813）
**状态**：草稿（rally 模式：无用户逐阶段确认，按控制面 master.md 推进）
**关联**：`docs/goal/rendering-compat/master.md` R805-R812；`phase-a-IFC-unification-design.md`（Path-A/IFC 存储，本 RFC 为其下游度量子域）；DC-2/5 文本类、DC-13 welcome、DC-14 chromium Oracle
**驱动数据**（实测 `reftest-oracle`）：vertical-align 簇 `vertical-align-117a` 24.15% / `118a` 23.72% / `baseline-004a` 21.85% / `negative-leading-001` 21.45% / `baseline-005a` 21.02% / `122` 20.77%；welcome 16.11%；linebox dir 33.7%（64/190）

---

## 0. 执行摘要

- **一句话目标**：让 inline 文本的 baseline / ascent / descent / half-leading 在 **layout IFC 一次计算并存储**，paint 直接复用——根除 layout vs paint 两趟 IFC 在垂直度量上的分歧，解 vertical-align 簇（6 案 20-24%）+ welcome 文本行位 + 父子 margin 传播的共性根因。
- **本期范围**：仅产出设计文档 + 分阶段实施计划。**不落地代码**（本轮为 read-only 设计；R805/R807/R811 三轮证单点尝试 net-negative，须先有可验证契约再改码）。
- **明确排除**：Path-A/IFC 存储路径删除（`phase-a-IFC-unification-design.md` 已覆盖，本 RFC 不重复）；multicol column-aware IFC（独立 RFC）；writing-mode 轴交换（R114/R164 证否）；taffy 升级（R304 deferred）。
- **核心约束**：① 任一 Phase 必须 `make test` + `make reftest`（loose + strict）+ `make product-smoke`（welcome diff>20% 退出 2，R541 教训）+ scoped `make reftest-oracle` **零 count 回归**；② 单文件 ≤2000 行（`inline/mod.rs` 现 ~? 行，触及处须评估拆分）；③ paint 不得以改变布局语义的方式重排 glyph（goal DC-13）；④ 不引入新 `#[ignore]`。
- **推荐方案**：**baseline-resolved 单一权威行盒**——`InlineLayoutLine` 增 `baseline_y` / `ascent` / `descent`，`InlineLayoutFragment` 增 `baseline_y`；compute_final 用真实字体度量算出并存储；paint Path A 直接消费 `baseline_y`，逐步用其替换 `is_ahem?0:font_size` 启发式。
- **首个落地步骤**：Phase 0（read-only 探针）= 对 `vertical-align-117a`（`vertical-align:text-bottom` + `font:2.5em/3.25 Ahem`）用临时插桩实测 layout 算出的 `baseline_y`/`half-leading` vs paint 实际用值 vs chromium，确证分歧点与正确公式，再决定 Phase 1 字段语义。

---

## 1. 背景与目标

### 1.1 背景

ZeroWeb 的行内排版（IFC）垂直度量目前在 layout 与 paint 两阶段**各算一次**且输入不同：

- **layout IFC**（`crates/layout-engine/src/inline/mod.rs::apply_vertical_alignment`）用真实 ComputedStyle 计算 `strut_ascent = (line_height - dominant_fs)/2 + dominant_fs*0.8`（R800 half-leading + ascent），run ascent 等，得到行盒与片段的垂直位置。
- **paint IFC**（`crates/engine/src/paint/painter/text.rs`）对未存 `inline_layout` 的容器**重跑 IFC**（Path B，空 styles + override maps），用 `v_offset = is_ahem?0:font_size`（text.rs:1293）或 `baseline_fs`（text.rs:1310）启发式定位 glyph 基线。

两趟在 **half-leading / baseline / vertical-align 对齐** 上分歧。vertical-align 簇（`text-bottom`/`text-top`/`super`/`sub`/`baseline` + `line-height` 变化）直接依赖这些度量，故 20-24% 发散。

R800 修了 baseline 公式（half-leading + ascent，welcome +0.05pp 小正）但只触及 layout 侧；paint Path B 仍用旧启发式。R805 复读 `phase-a-IFC-unification-design.md` 确认：narrow Phase-A slice 全 net-negative（墙①Gate2 多行 / 墙②multicol 反向依赖 / 墙③v_offset-baseline 语义分歧），唯一 clean slice = R207 已 LANDED。**墙③（v_offset/baseline 语义分歧）正是本 RFC 要解的度量分歧**——R805 推荐的「full 统一」须先解 R72 保留 Path B 规避的回归，但 R806 实证 BFC-004 非 Path A/B（float bug），故 Path B 删除前提部分瓦解，本度量统一是独立可推进的子域。

### 1.2 目标

- **业务目标**：ZeroWeb 行内文本垂直位置（baseline / vertical-align / half-leading）与 Chromium 一致，vertical-align 簇 chromium-Oracle z_vs_chr 下降。
- **用户目标**：welcome / 真实静态页正文行间距、标题基线、`vertical-align` 图标/上下标位置正确。
- **可验证成功标准**：① vertical-align 簇（117a/118a/baseline-004a/005a/122/negative-leading）chromium-Oracle z_vs_chr 下降；② welcome product-smoke diff 下降或持平；③ 全量 `make reftest` loose 不退、strict 不退、scoped `make reftest-oracle` 真一致率不降。

### 1.3 范围边界

- **在范围内**：`InlineLayoutLine` / `InlineLayoutFragment`（`types/mod.rs`）的度量字段；`apply_vertical_alignment`（`inline/mod.rs`）的存储；`compute_final_inline_layouts`（`inline_finalization.rs`）的存储条件；`paint_text` Path A（`text.rs`）对 `baseline_y` 的消费；度量计算函数（ascent/descent/half-leading 来源）。
- **不在范围内**：Path-B 删除（→ `phase-a-IFC-unification-design.md`）；multicol column-aware IFC（→ `multicol-fragmentation-design.md`）；writing-mode 轴（→ R114/R164）；intrinsic sizing（→ R97/R301）；taffy 升级（→ R304）。

---

## 2. 功能需求（FR）

### FR-001：单一权威 baseline 度量
- **描述**：当 `inline_layout` 被存储时，每个 `InlineLayoutLine` 必须携带**已解析的绝对 baseline_y**（= line.y + strut_ascent），paint Path A 必须直接用该 `baseline_y` 定位 glyph，禁止用 `is_ahem?0:font_size` 启发式推断。
- **优先级**：必须
- **来源**：R800（baseline 公式已对但仅 layout 侧）+ R805 墙③

**验收场景**：
```
场景: vertical-align:text-bottom 行 baseline 正确
  假设 `vertical-align-117a`（`font:2.5em/3.25 Ahem`，span vertical-align:text-bottom）
  当 ZW 渲染并对比 chromium oracle
  那么 blue span 底边对齐父 content-area 底边（span 完全在黑条上方），z_vs_chr 下降
  验证: `make reftest-oracle DIR=vertical-align-117a`（z_vs_chr < 起始 24.15%）

场景: baseline 公式对 pure-Ahem 单行子集退化正确（不回归 R207）
  假设 `font-051`（pure-Ahem 单行 100px，R207 已 PASS 0.00%）
  当 启用存储 baseline_y 后渲染
  那么 font-051 仍 0.00% PASS（baseline_y 对该子集退化为旧 v_offset=0 语义）
  验证: `make reftest-oracle DIR=font-051`（仍 PASS）

场景: 字体度量不可得时的回退
  假设 某 fragment 的 ascent 无法由 fontdue 度量解析
  当 paint 读取 baseline_y
  那么 回退到 `font_size*0.8` 近似 ascent 并 `tracing::warn!`，渲染不崩
  验证: 单测 `baseline_y_fallback_uses_font_size`
```

### FR-002：half-leading / 行盒高度一致性
- **描述**：存储的 `InlineLayoutLine.height` 必须等于 `ascent + descent`（含 half-leading），与 layout IFC 算出的行盒高度同源；paint 不得用不同 line-height 重算。
- **优先级**：必须
- **来源**：R632（line_height override 已存但仅 Path B）+ welcome 行间距

**验收场景**：
```
场景: 多行非-Ahem 文本行间距响应 CSS line-height
  假设 `line-height:2.0` 的多行 div（R632 修后已响应）
  当 存 baseline_y 后渲染
  那么 行间距仍 = CSS line-height（不回退到 R632 前 fallback 19.2）
  验证: 全量 `make reftest` loose 不退（R632 收益保持）

场景: 异常 — line-height:normal 退化
  假设 line-height:normal（1.2/Ahem 1.0）
  当 渲染
  那么 行盒高度 = font_size * normal_ratio（NORMAL_LINE_HEIGHT_RATIO），不回归
  验证: 单测 + `make reftest` strict 不退
```

### FR-003：零 count 回归硬门禁
- **描述**：每个 Phase 落地必须以全量 `make reftest`（loose + strict）+ `make product-smoke`（welcome）+ scoped `make reftest-oracle` 三态不退为合并条件。
- **优先级**：必须
- **来源**：R541（product-smoke 缺失藏回归 14 轮）+ R807（root-stretch +22pp welcome）

**验收场景**：
```
场景: 任一 Phase 三态门禁
  假设 Phase N 代码改动
  当 跑 `make test` + `make reftest` + `make product-smoke` + scoped `make reftest-oracle`
  那么 全绿 + welcome diff≤起始 + loose/strict count 不退 + Oracle 真一致率不降
  验证: 命令组合；任一退步即 `git revert` 该 commit
```

---

## 3. 约束与假设

### 3.1 必须约束（Must）
- 每 Phase 落地前 `make test` 全绿、`cargo clippy --workspace --all-targets -D warnings` 干净。
- 触及 `inline/mod.rs`（apply_vertical_alignment）须评估 2000 行拆分。
- 修改「禁止修改」路径须停止并说明。

### 3.2 禁止约束（Must Not）
- 不允许放宽容差掩盖 vertical-align 回归（DC-14 容差锁定）。
- 不允许 paint 对 glyph 做改变布局语义的整行重排（DC-13）。
- 不允许引入新 `#[ignore]`（除 real_website_compat.rs）。

### 3.3 已定决策
- 复用 `compute_final_inline_layouts` + `paint_text` 现有架构，不重写 IFC 引擎。
- 字体度量优先复用 fontdue 现有 ascent；若 fontdue 不提供 per-glyph ascent，layout 侧用 `font_size*0.8` 近似（R800 已用），**不引入 FontLoader 全量预解析**（更大独立 RFC）。

### 3.4 假设
- **A1（Phase 0 已证真）**：fontdue/font-size/ascent 基础应用正确——孤立 `<div style="font:40px/3.25 Ahem">` line-box span=129px（=line-height 130）。line-height 计算无误。— 状态：✅ 已验证（R814）。
- **A2（Phase 0 部分推翻 + 扩范围）**：vertical-align 簇发散**非仅** layout vs paint baseline 分歧。va-117a 实测：`height:2em;width:2em`（80×80 固定盒）+ 换行 `TTTT<span..>TTTT`，ZW black TTTT 仅 50px 高（line-box 应 ~130/行）→ **连单行 line-box 高度都算错**（换行 + 固定高度约束下）。故 RFC 须扩到**换行多行盒 per-line baseline + height 约束交互**，非单 baseline_y 字段可解。— 状态：⚠️ 部分验证，范围须扩（R814）。
- **A3**：pure-Ahem 单行子集（R207）在新 baseline_y 下退化到旧 v_offset=0 语义仍 PASS。— 状态：需 Phase 1 验证。

### 3.5 代码变更边界
- **允许修改**：`crates/layout-engine/src/types/mod.rs`（字段）、`crates/layout-engine/src/inline/mod.rs`（apply_vertical_alignment 存储）、`crates/layout-engine/src/inline_finalization.rs`（存储转换）、`crates/engine/src/paint/painter/text.rs`（Path A 消费 baseline_y）、`crates/layout-engine/src/inline/text_metrics.rs`（度量来源）。
- **禁止修改**：`crates/taffy-local/**`（vendored，R304）、`crates/render-foundation/**`（渲染器光栅化，与 IFC 度量无关）、`tests/wpt-runner/**`（reftest harness）。

---

## 4. 技术设计（RFC）

### 4.1 目标状态架构

```
compute_final_inline_layouts（真实 styles IFC，apply_vertical_alignment）
   │  对所有过 Gate 1 容器存 inline_layout
   │  每行存 (y, height, baseline_y, ascent, descent)
   │  每片段存 (x, y, baseline_y, width, height, font_size, is_ahem, ...)
   ▼
paint_text
   use_stored = inline_layout.is_some() && width_matches（与现一致）
   │
   ▼
Path A：渲染 stored fragments，glyph baseline = frag.baseline_y（取代 is_ahem?0:font_size）
Path B（重跑）：仅 Gate 1 显式跳过的容器（flex/grid/table）保留
```

**核心变更**：`InlineLayoutFragment.y` 语义保持（片段框顶 = baseline_y - height）；**新增 `baseline_y`** 字段供 paint 直接消费；paint Path A 用 `baseline_y` 取代 `v_offset = is_ahem?0:font_size`。

### 4.2 数据模型变更（`types/mod.rs`）

```rust
pub struct InlineLayoutLine {
    pub y: f32,
    pub height: f32,
    pub baseline_y: f32,   // 【新增】行基线绝对 y = y + strut_ascent
    pub ascent: f32,       // 【新增】行 strut ascent（含 half-leading 上半）
    pub descent: f32,      // 【新增】行 strut descent（含 half-leading 下半）
    pub fragments: Vec<InlineLayoutFragment>,
}
pub struct InlineLayoutFragment {
    pub x: f32,
    pub y: f32,            // 保留（片段框顶 = baseline_y - height）
    pub baseline_y: f32,   // 【新增】片段基线绝对 y（baseline 对齐时 = line.baseline_y + vertical_align_offset）
    pub width: f32,
    pub height: f32,
    pub font_size: f32,
    pub is_ahem: bool,
    pub text: String,
    pub node_id: Option<NodeId>,
}
```

**实现来源（§6.5A）**：ascent/descent 由 `inline/text_metrics.rs::resolve_font_metrics`（已有，R800 用）+ fontdue 度量；half-leading = (line_height - font_size)/2。仓内自实现，无新依赖。

### 4.3 Phase 0 探针（read-only，前置）

对 `vertical-align-117a`（`font:2.5em/3.25 Ahem`，span `vertical-align:text-bottom`，`margin:auto -1em`）：
1. 临时插桩 layout `apply_vertical_alignment` 打印 strut_ascent / line.height / span frag.y / span baseline_y。
2. 临时插桩 paint Path A 打印实际用的 v_offset / glyph_y。
3. 对比 chromium oracle 几何（PIL 测 span 底边 vs 父 content-area 底边）。
→ 产出探针报告，确证：layout 算出的 baseline 是否正确？paint 用的是 layout 值还是启发式？vertical-align:text-bottom 的 offset 公式对否？据此定 Phase 1 字段语义（避免在 shaky 前提上编码，code-guidelines「先思考再编码」）。

**裁决依据**：R306 Phase 0 探针曾证伪「geometric baseline（frag.y+height）可作 render baseline」（font-051 16.67% FAIL），故本 Phase 0 须先实证 baseline_y 的正确语义（render baseline ≠ 几何 baseline，差 fontdue-metric 常量）。

### 4.4 影响范围

| 影响项 | 程度 | 说明 |
|--------|------|------|
| types/mod.rs | 中 | 两结构加 3+1 字段 |
| inline/mod.rs apply_vertical_alignment | 高 | 存储 baseline_y/ascent/descent |
| inline_finalization.rs | 中 | IFC→InlineLayoutLine 转换带新字段 |
| text.rs Path A | 高 | 用 baseline_y 取代 v_offset 启发式 |
| inline/mod.rs 文件行数 | 中 | 须评估 2000 行拆分 |

### 4.5 替代方案

| 方案 | 描述 | 优点 | 缺点 | 决定 |
|------|------|------|------|------|
| A：baseline_y 字段 | 存绝对 baseline_y，paint 直接用 | 解耦 layout/paint 度量分歧 | 须先解 R306 证伪的 baseline 语义 | ✅ 选定（Phase 0 先实证） |
| B：存 render glyph_y | compute_final 用同款 offset 算出 glyph_y，paint 直接消费 | 绕过 baseline 语义分歧 | 与 Path-A v_offset 耦合，难维护 | ❌ 拒绝（R805 §6.3B 评） |
| C：仅扩 Gate 2（不改度量） | 放宽存储覆盖更多容器 | 改动小 | 不解度量分歧，vertical-align 仍错 | ❌ 拒绝（R209/R213/R327 证 net-negative） |

**最终选择**：方案 A（Phase 0 探针先行定语义）。

### 4.6 回归风险与缓解

| 风险 | 概率 | 缓解 |
|------|------|------|
| baseline_y 对普通字体 ascent 近似不准 → 文本类大面积漂移 | 高 | Phase 0 先在 Ahem 子集验证退化正确；普通字体分阶段，每类目 set-diff + product-smoke |
| Path A 改 baseline_y 破坏 R207 pure-Ahem 子集 | 中 | A3 验证；font-051 单测守卫 |
| multicol / flex-baseline 交互回归 | 中 | Gate 1 显式跳过 multicol/flex/grid/table，不触发新路径 |
| welcome 回归（R807 教训） | 中 | 每 Phase 必跑 product-smoke（diff>20% 退出 2） |

---

## 5. 实施交接（Implementation Handoff）

### 推荐修改顺序（6 Phase，每 Phase 独立可合并）

0. **Phase 0（read-only 探针）**：vertical-align-117a 度量插桩 + chromium 对比。→ 验证：探针报告，无代码。
1. **Phase 1（加字段，行为不变）**：types/mod.rs 加 baseline_y/ascent/descent 字段（默认值，compute_final 填充但 paint 不读）。→ 验证：全量三态不退（纯加字段）。
2. **Phase 2（paint Path A 读 baseline_y，Gate 2 不变）**：text.rs Path A 用 baseline_y，仅对纯-Ahem 子集启用（is_ahem 守卫）。→ 验证：font-051 等 R207 子集仍 PASS（A3）。
3. **Phase 3（扩到非-Ahem）**：去掉 is_ahem 守卫，paint 全用 baseline_y。→ 验证：vertical-align 簇 z_vs_chr 下降，product-smoke 不退。
4. **Phase 4（收尾）**：清理 v_offset 启发式死代码。→ 验证：全量三态不退 + clippy/fmt 干净。
5. **Phase 5（文件拆分，若需）**：inline/mod.rs 超 2000 行则抽 vertical_alignment.rs。→ 验证：纯移动，全量三态不退。

### 首批提交建议

| 批次 | 范围 | 预期 | 验证 |
|------|------|------|------|
| Phase 0 | 探针插桩 | 探针报告 | vertical-align-117a 度量对比 |
| Phase 1 | 加 baseline_y 等字段 | 零行为变化 | `make test` + `make reftest` + product-smoke 三态不退 |

---

## 6. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 含目标/范围/排除/约束/方案/首步 |
| 场景存在性 | ✅ Pass | FR-001~003 各有 ≥1 验收场景 |
| 异常路径覆盖 | ✅ Pass | FR-001 含 fallback、FR-002 含 normal 退化、FR-003 含 revert |
| 测试绑定 | ✅ Pass | 每场景绑 `make reftest-oracle`/单测/命令 |
| TBD 清零 | ⚠️ Warning | A1/A2/A3 标注「待 Phase 0 验证」（非阻塞，降级为假设） |
| 实施交接完备 | ✅ Pass | §5 含文件清单/顺序/首批提交 |
| 首步可执行性 | ✅ Pass | Phase 0 探针明确 |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ Pass | FR 用「必须携带/必须用/禁止」具体动词 |
| 非确定性措辞 | ✅ Pass | 无「应该/可能」（A1-A3 标为假设非需求） |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 在/不在范围无交集 |
| 约束冲突 | ✅ Pass | §3.1/3.2 Must/Must Not 无矛盾 |
| 方案漂移 | ✅ Pass | §4 设计未引入与范围冲突依赖 |
| 实现来源闭合 | ✅ Pass | §4.2 ascent/descent 来源（text_metrics + fontdue）已写 |
| 代码边界完备 | ✅ Pass | §3.5 允许/禁止修改声明 |

**汇总**：13 Pass / 1 Warning / 0 Fail / 0 Skip
**门禁判定**：Fail = 0 → 允许进入实施（本轮为设计，下一轮起 Phase 0 探针）

---

## 7. 修订历史

| 版本 | 日期 | 变更 |
|------|------|------|
| v1.0 | 2026-06-29（R813） | 初始 read-only 设计产出（linebox 度量统一 RFC） |
