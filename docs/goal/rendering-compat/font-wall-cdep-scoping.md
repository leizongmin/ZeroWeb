# Font-Wall C-Dep Scoping Report — 字体渲染兼容性残余分析

> **日期**: 2026-07-16  
> **类型**: 技术调研（零源码改动）  
> **目标**: 为 font-wall C-dep 推荐最高收益的首个改进切片

## 1. 当前字体栈架构映射

### 1.1 核心组件定位

| 功能 | 当前实现 | 代码位置 | 配置 |
|------|----------|----------|------|
| **文本整形 (Shaping)** | `rustybuzz` (OpenType) | `crates/render-foundation/src/font/shaper.rs:74-132` | 默认启用；无 feature gate |
| **光栅化 (Rasterization)** | `FreeType` (优先) → `fontdue` (回退) | `crates/render-foundation/src/font/loader.rs:298-329` | `freetype-raster` feature (default-on) |
| **度量提取 (Metrics)** | `fontdue` (ascent/descent/line-gap) | `crates/render-foundation/src/font/loader.rs:418-443` | 从 `fontdue::Font` 提取 |

### 1.2 数据流详解

**整形路径 (Shaping Pipeline)**:
```text
Unicode 文本 → rustybuzz::shape → OpenType features (连字/kerning/GSUB/GPOS)
              → fontdue 逐字符回退 → ShapedGlyph 序列
```
- **关键文件**: `shaper.rs:74-132` (`shape_with_rustybuzz`)
- **回退条件**: 字体数据不可用或字体未加载
- **度量来源**: `fontdue.metrics_indexed()` (按 glyph_id 取 advance_width)

**光栅化路径 (Rasterization Pipeline)**:
```text
Glyph ID → FreeType (优先，freetype-raster feature)
         → FT_LoadGlyph(LoadFlag::DEFAULT) + FT_RenderGlyph(Normal)
         → fontdue 回退 → GlyphBitmap (灰度位图)
```
- **关键文件**: `loader.rs:298-329` (`rasterize_glyph`) + `loader.rs:37-92` (freetype_raster 模块)
- **配置**: `LoadFlag::DEFAULT` (full hinting)，`RenderMode::Normal` (灰度抗锯齿)
- **坐标约定**: `y_offset = bitmap_top - height` (见 `loader.rs:79-80` 注释)

**度量提取 (Metrics Pipeline)**:
```text
fontdue::Font → horizontal_line_metrics → (ascent, descent, line_gap)
               → line_metrics_full → (ascent, descent, line_gap)
```
- **关键文件**: `loader.rs:418-443`
- **当前用途**: Ahem 字体特殊处理 (`loader.rs:341-346`)
- **IFC 集成**: 度量未完整传导至 IFC (见 MEMORY.md R876/R1088)

### 1.3 Ahem 特殊处理

Ahem (WPT 标准测试字体) 通过 `rasterize_ahem_glyph` 生成完美填充方块:
- **触发条件**: `font_id == ahem_font_id && !whitespace`
- **方块尺寸**: `width = height = size.ceil()`
- **垂直偏移**: `y_offset = -ascent.ceil()`
- **验证**: R1066 prototype 证实 Ahem 在 fontdue vs FreeType 下几乎一致 (mean|Δ|=0.0000 @ 20px)

## 2. 残余分解：光栅化 vs 整形 vs 度量

### 2.1 当前状态量化

根据 `docs/goal/rendering-compat/master.md` (R1552):

| 指标 | 当前状态 | 主要障碍 |
|------|----------|----------|
| **Writing-modes** | 135/788 (17.1%) | font-wall (bidi 79案 + line-box 50案) |
| **Bidi** | 79 案 | font-wall (FreeType vs FreeType+Skia AA 差) |
| **Line-box** | 50 案 | font-wall (度量精度) |
| **10-dir aggregate** | 51.3% broad | font-raster 墙 (strict 个位数) |

**关键证据**:
- R1066 prototype: fontdue vs FreeType per-glyph 差异 **3-30%** (Latin) / **11-16%** (CJK)
- R1068: FreeType default-on 后全 corpus **+232** oracle，aggregate 36.2% → 51.3%
- R1088: Phase A line-box 度量修复 net-negative (-7 selectors)，根因 = 度量精度不足

### 2.2 残余分类

#### (A) 光栅化差异 (Rasterization Gap)
**根因**: FreeType (ZW) vs FreeType+Skia (Chromium) 抗锯齿/子像素渲染算法差

**证据**:
- R1066 prototype: Latin 'a'/'g' 差 **27-30%** (曲线字形最敏感)
- 当前配置: `LoadFlag::DEFAULT` + `RenderMode::Normal` (灰度，无子像素)
- Chromium 使用 Skia + LCD 子像素渲染 + 自定义 gamma

**影响范围**:
- **主要**: 非-Ahem 文本渲染 (909 个 writing-modes 案中的 bidi/line-box 残余)
- **次要**: css-text/css-text-decor/selectors (near-pass 带)

**修复复杂度**: 高 (须替换整个光栅化器或引入 Skia)

#### (B) 整形差异 (Shaping Gap)
**根因**: rustybuzz 仅用于 glyph 序列生成，复杂脚本/RTL/kerning 可能不完整

**证据**:
- 当前实现: `shaper.rs:74-132` 调用 `rustybuzz::shape` + `fontdue.metrics_indexed`
- 回退路径: 逐字符映射 (无 OpenType features)
- 无测试证据显示整形问题主导 diff (R1088/R1066 均聚焦光栅化)

**影响范围**: 潜在 (复杂脚本如阿拉伯文/梵文，但 WPT corpus 中覆盖有限)

**修复复杂度**: 中 (rustybuzz 已集成，可能需调优参数或补 pipeline)

#### (C) 度量差异 (Metrics Gap)
**根因**: IFC 用 `0.8·fs` / `1.2·fs` 近似而非真实 ascent/descent/line-gap

**证据**:
- R1088: Phase A line-box 修复 net-negative (首字母 36px 改变行盒高 → 级联偏移)
- R876: fontdue tight-ink bitmap 无 metric ascent → 三方补偿平衡 (ascent/baseline/paint)
- loader.rs 已暴露 `line_metrics_full` 但未接入 IFC

**影响范围**:
- **主要**: line-box 簇 (50 writing-modes 案)
- **次要**: ::first-letter (436 案，R1088 net-negative 回退)

**修复复杂度**: 低 (度量已提取，仅须 plumbing 到 IFC)

### 2.3 C-依赖边界

| 组件 | 当前依赖 | C-依赖候选 | 许可证 | 风险 |
|------|----------|-----------|--------|------|
| **FreeType** | `freetype-rs` (bundled) | ✅ 已 default-on | FTGL GPLv2/FTL | 低 (已验证 +232) |
| **Skia** | ❌ 无 | 🟡 大型 C++ | BSD 3-Clause | 中 (构建复杂度高) |
| **HarfBuzz** | ❌ rustybuzz 替代 | 🟢 可选 | Old MIT | 低 (rustybuzz 已够用) |

## 3. 方案收益/风险/成本对比

### 3.1 候选切片评估

| 方案 | 预期收益 | 风险 | 成本 | 验证复杂度 |
|------|----------|------|------|-----------|
| **(i) HarfBuzz/rustybuzz shaping 完善** | 低 (整形非主导 diff) | 低 (已有基础) | 2-4 人日 | 中 (须复杂脚本测试) |
| **(ii) FreeType subpixel/LCD + gamma 调优** | 中 (减少 ~10-20% rasterization diff) | 中 (可能引入回归) | 3-5 人日 | 低 (本地 A/B) |
| **(iii) fontdue→FreeType metric pipeline coherence** | 高 (解锁 line-box + ::first-letter) | 低 (度量已暴露) | 1-2 人日 | 低 (单测覆盖) |
| **(iv) Skia rasterization 完整替换** | 高 (接近 chromium rasterization) | 高 (大型 C++ 依赖) | 10-20 人日 | 高 (全平台 CI) |

### 3.2 收益量化估算

**方案 (iii) metric pipeline**:
- **直接收益**: line-box 50 案 + ::first-letter 436 案 (R1088 net-negative 根因)
- **间接收益**: 为 Phase A / 其他度量敏感修复铺路
- **估算 flip**: +20-50 oracle (保守估计 line-box 簇 20-30%，::first-letter 部分 10-20)

**方案 (ii) FreeType tuning**:
- **直接收益**: 减少 rasterization diff (bidi 79 案部分改善)
- **估算 flip**: +10-30 oracle (假设 AA 差改善 20-30%)

**方案 (iv) Skia**:
- **直接收益**: 接近 chromium rasterization (bidi/line-box/font-wall 主干)
- **估算 flip**: +100-300 oracle (rasterization 墙移除)

## 4. 推荐首切片：Metric Pipeline Coherence

### 4.1 选择理由

1. **最高 ROI**: 低成本 (1-2 人日) vs 高收益 (20-50 oracle + 解锁 Phase A)
2. **低风险**: 度量已暴露 (`line_metrics_full`)，仅须 plumbing
3. **可验证**: 单测覆盖 (loader.rs:755-785 已有 `test_line_metrics_full_exposes_line_gap`)
4. **无 C-依赖**: 纯 Rust 实现，符合「小切片验证」原则

### 4.2 实施细节

**修改文件**:
1. `crates/render-foundation/src/font/loader.rs` (度量导出接口)
2. `crates/layout-engine/src/inline_formatting.rs` (IFC 接收真实度量)
3. `crates/layout-engine/src/lib.rs` (pipeline 调整)

**关键改动**:
- 在 `FontLoader` 暴露 `get_font_metrics(font_id, size) -> (ascent, descent, line_gap)`
- IFC 用 `(ascent, descent, line_gap)` 替代当前 `(0.8·fs, -0.2·fs, 0.0)` 近似
- 保持 Ahem 特殊处理 (已验证正确)

**Kill-switch 设计**:
```rust
// 环境变量控制
env::var("ZW_REAL_FONT_METRICS").ok().as_deref() == Some("1")
```

**验证计划**:
1. **单测**: 复用 `test_line_metrics_full_exposes_line_gap` 验证度量提取
2. **集成测试**: `make test` (零回归)
3. **产品 smoke**: `make product-smoke welcome` (字节稳定)
4. **Reftest A/B**: `make reftest-oracle DIR=css-writing-modes` (预期 +10-20 案)
   - 重点监控: line-box 簇 (如 `line-box-*/vrl-*/vlr-*` 50 案)
5. **Sentinel**: `ORACLE_DUMP_ALL per-case` 逐案验证无恶化

### 4.3 预期结果

**保守估算**:
- Writing-modes: 135 → **145-155** (+10-20)
- 其中 line-box 簇: 50 案中 **20-30** flip 到 <1%
- ::first-letter: R1088 的 -7 回归风险消除 (metric 精度不再敏感)

**风险上限**:
- Worst-case: net 0 (度量精度 diff 与当前近似抵消)
- 回归风险: 低 (度量仅影响 line-height 计算，Ahem 路径不变)

**Gate 条件**:
- writing-modes A/B net ≥ 0
- welcome 字节一致 (metric 改动不应影响几何)
- 无新的 >=3% 恶化

## 5. 未解决问题

### 5.1 技术疑问

1. **度量传导路径**: `FontLoader → engine → IFC` 的具体插入点？
   - 需确认 `layout-engine` 中哪些模块依赖当前 `(0.8, 1.2)` 近似
   - 可能涉及 `inline_formatting.rs` 多处 `strut_ascent` / `half-leading` 计算

2. **Ahem 与非-Ahem 路径隔离**:
   - Ahem 当前方块生成 (`rasterize_ahem_glyph`) 是否依赖度量近似？
   - R1066 证实 Ahem 在 fontdue/FreeType 下一致，度量改动应零影响

3. **line-gap 使用**:
   - Chromium 的 `line-height: normal` = `ascent - descent + line_gap`
   - ZW 当前未用 line_gap，是否影响 line-height 断行？

### 5.2 后续路径

**如果方案 (iii) 成功**:
- **下一步**: 方案 (ii) FreeType tuning (subpixel/LCD)
- **长期**: Skia rasterization (须 RFC + 多平台验证)

**如果方案 (iii) net-neutral**:
- **诊断**: 用 `LAYOUT_DUMP` 确认度量是否传导到 IFC
- **备选**: 跳到方案 (ii) 直接改善 rasterization

### 5.3 外部依赖

- **Chromium 源码**: 当前无需 (rasterization 算法已通过 prototype 验证)
- **Skia 构建**: 方案 (iv) 时需要 (当前切片不需要)
- **系统字体**: Linux/macOS 已通过 `FontLoader` 加载，无需改动

## 6. 附录：关键证据索引

| 证据文件 | 核心结论 |
|----------|----------|
| `evidence/r1066-fontdue-vs-freetype-prototype-2026-07-06.txt` | FreeType vs fontdue 光栅化差 3-30% |
| `evidence/r1088-first-letter-phaseA-gate-2026-07-06.txt` | Phase A metric 精度 net-negative 根因 |
| `MEMORY.md R1068/R1067/R1056` | FreeType default-on +232；metric swap net-neutral |
| `master.md R1552` | Writing-modes残余分解：bidi 79 + line-box 50 |

---

**结论**: 推荐以 **方案 (iii) metric pipeline coherence** 为首切片，预期 +20-50 oracle，风险低，1-2 人日完成，无需 C-依赖新增，符合项目「小切片 A/B 验证」原则。验证通过后可考虑 FreeType tuning 或 Skia rasterization 作为后续路径。

---

## 7. R1552b 关键勘误（review 后追加）—— 勿盲从 §4 推荐

> **本节由主循环 review 追加**：§4 推荐「metric pipeline coherence (iii)」与既有强证据**冲突**，须勘误。

### 7.1 推荐 (iii) metric coherence **已被证伪（DEAD）**——勿重试

§4 推荐 plumb `line_metrics_full` 真实 ascent/descent 到 IFC 替代 `0.8·fs` 近似，称「低风险 +20-50」。但**该路径已在 FreeType default-on 之后被两轮 post-FreeType 复测证伪**（§2/§6 只引了 pre-FreeType 的 R1088，漏了 post-FreeType 复测）：

- **R1160**（post-FreeType）：Phase A line-box metric formula-only → **net −1**（已回退）。
- **R1206**（post-FreeType）：Phase A combined store-gate + 公式 → **net −22**（比 pre-FreeType R1090 的 −14 更差，已回退）。
- 加 pre-FreeType 的 R1090/R1095：**四轮（R1090/R1095/R1160/R1206）证 Phase A metric 放置杠杆死**（见 MEMORY.md `r1088-first-letter-phaseA-universal-gate`）。
- R1067：FreeType DejaVu metric swap（non-Ahem ascent 0.928→0.95）**net-NEUTRAL 第七证**（无正 yield）。
- R1056：CJK metric single-knob **net-negative 第六证**。

★ **根因**（R876）：font metric 改动须**三方同改**（rasterizer metric + baseline ratio + paint v_offset），单轴 plumb 必 net-negative——正是 §4.2 单轴提案的失败模式。**结论：metric coherence (iii) DEAD，勿重试**；本报告 §4 推荐作废。

### 7.2 §2.1 残余分类勘误：光栅化差是 **FreeType vs FreeType+Skia**（非 fontdue vs FreeType）

§2.1(A) 表述「fontdue vs FreeType」不准确——**FreeType 自 R1068/R1159 已 default-on**（fontdue 仅 fallback）。真实残余 = ZW FreeType（`LoadFlag::DEFAULT` full-hint + `RenderMode::Normal` 灰度 AA）vs chromium FreeType+**Skia**（Skia 自有 AA 算法 + LCD 子像素 + gamma）。R1066 的「fontdue vs FreeType 3-30%」是 R1068 之前的旧数据。**残余是 rasterizer 算法差，非 flag 可调**（R1069 证 LoadFlag DEFAULT 最优）。

### 7.3 实际可行的 font-wall 切片（修正后）

| 切片 | 状态 | 可行性 |
|------|------|--------|
| (iii) metric coherence | **DEAD**（§7.1 四轮证伪） | 勿重试 |
| LoadFlag tuning | 穷尽（R1069 DEFAULT 最优） | 勿改 |
| `RenderMode::Lcd` 子像素 | **未测**但高风险（色条纹；chromium oracle 子像素设置未知；显示相关） | 可一次性 env-gated A/B 验证，预期负/中性 |
| gamma-correct blend | 未测（R1069 仅测 FreeType flag，未测 composite gamma）；Rust 侧改动较小 | 可探（小切片） |
| **(iv) Skia rasterization** | **唯一真实 font-wall unlock**（FreeType→Skia 算法对齐） | 大 C-dep，须 RFC + 小切片（decision #1 授权） |

### 7.4 结论（修正）

font-wall **无安全 quick win**：metric DEAD、LoadFlag 穷尽、LCD/gamma 高风险未证。**真实路径 = Skia C-dep**（§3 iv），须 `lei-spec-rfc` 做 RFC（迁移设计 + 风险拆分 + 跨平台 CI 小切片），decision #1 已授权。本报告 §1-§2 font-stack map 有效；§3-§4 推荐**作废**，以本节为准。

## 8. R1553 实测：gamma-correct blend **net-neutral**（font-wall 非 compositing 问题）

> 主循环实测追加：`ZW_GAMMA_CORRECT_BLEND=1`（cpu/mod.rs `blend_pixel` linear-space 混合，仅影响 alpha<255 的字形路径）。

**A/B**（writing-modes 784 案 ORACLE_DUMP_ALL）：gamma ON **134** vs OFF **135**（NET **−1**，avg delta −0.001pp，0 改善 / 1 回归 logical-props-002 0.88→1.02）。line-box-height-vlr-003 单案 3.49→3.48（无变化）。

★ **结论**：gamma-correct blend **非 font-wall 杠杆**（net-neutral）。理论推导（black-on-white 直线 127 vs gamma 188，差 61px/像素）被实测推翻——font-wall AA 边缘差是 **FreeType coverage 算法 vs Skia coverage 算法**本身（位图生成），非 compositing 阶段。**已 revert**。

### 8.1 font-wall 小切片全排除——Skia 是唯一路径

| 小切片 | 状态 | 证据 |
|--------|------|------|
| metric coherence | DEAD | R1090/R1095/R1160/R1206 四轮 net-negative |
| LoadFlag tuning | 穷尽 | R1069 DEFAULT 最优 |
| gamma-correct blend | **net-neutral**（本节） | R1553 实测 −1 |
| `RenderMode::Lcd` 子像素 | 未测但预期负 | 显示相关 + chromium 设置未知 |

**font-wall 无 Rust 侧 quick win。真实 unlock = (iv) Skia C-dep**（FreeType→Skia coverage 算法对齐），须 `lei-spec-rfc` RFC + 跨平台小切片（decision #1 授权）。revert 干净（working tree = R1550 135）。
