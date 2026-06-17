# RFC: advance-width 真实度量接入（layout-engine IFC）

**状态**: Draft（待渐进实施）
**日期**: 2026-06-17
**背景**: R221（DC-14 可信通过率 39.6%）+ R222（advance 估计误差 ±44-98%）
**目标 DC**: DC-2~5（chromium 一致率），瞄准 183-case 1-3% 系统性噪声桶

## 问题

`layout-engine` 的 IFC（行内格式化）用自由函数 `estimate_char_width(ch, font_size, is_ahem)`（inline/mod.rs:201）度量文本宽度，固定倍数：字母 0.55×fs、数字 0.5、标点 0.4、空格 0.25。R222 实测此启发式与真实字体 advance 误差**逐字符 ±44-98%**（W 0.99 欠估 44%、i/l 0.28 过估 98%、m 0.97、t 0.39、f 0.35）。

调用点（IFC 行内换行 + 定位 + 内在尺寸）：
- `inline/mod.rs`: 269（字符串宽度）、1207/1227（空格）、1304/1632（逐字符定位）
- `intrinsic_sizing.rs`: 108/131（内在宽度）
- `engine.rs`: 1997
- `paint/painter/text.rs`: 410/443（list marker 定位）

**根因**：`TextRun`（inline/mod.rs:28）只有 `font_size` + `is_ahem_font`，**无 `font_id`**——IFC 不知道用哪个字体，无法调 `FontLoader::measure_advance`（已存在于 render-foundation loader.rs:289）。

**约束**：`layout-engine` 不能依赖 `render-foundation`（依赖方向反了——render-foundation 是底层，layout-engine 不应向下耦合字体光栅化）。须用依赖反转（trait）。

## 设计

### 核心抽象：`AdvanceSource` trait（layout-engine 定义）

```rust
// layout-engine/src/inline/mod.rs（或新 advance.rs）
/// 字符 advance 宽度源。默认实现 = 现有 estimate_char_width 启发式（零行为变更）；
/// engine 可注入 FontLoader-backed 实现提供真实度量。
pub trait AdvanceSource {
    fn measure(&self, ch: char, font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32;
}

/// 默认：回退到 estimate_char_width（保持当前行为，零回归）。
pub struct EstimateAdvance;
impl AdvanceSource for EstimateAdvance {
    fn measure(&self, ch: char, _font_id: Option<u32>, font_size: f32, is_ahem: bool) -> f32 {
        estimate_char_width(ch, font_size, is_ahem)
    }
}
```

### TextRun 增加 `font_id`

```rust
pub struct TextRun {
    // ... 现有字段 ...
    pub font_id: Option<u32>,  // 新增：CSS font-family 解析结果（None = 未知，用 estimate）
}
```
run 构造时由 IFC 从 style 的 font-family 解析（需 font_resolver 注入，见下）。

### IFC 函数签名扩展

`InlineFormattingContext::layout(...)` 等接收 `&dyn AdvanceSource`（默认 `&EstimateAdvance`）。内部 `estimate_char_width(ch, fs, ahem)` 调用替换为 `source.measure(ch, run.font_id, fs, ahem)`。

### engine 注入 FontLoader-backed 实现

`zero-engine` 依赖两者（render-foundation + layout-engine）。在 RenderPipeline 渲染时，构造 `FontLoaderAdvance<'a> { loader: &'a FontLoader }` 实现 `AdvanceSource::measure` → `loader.measure_advance(font_id?, ch, size)`（font_id None 时回退 estimate），注入 IFC。

## 风险：R125 IFC 三路径死锁

R125 确证 paint IFC / remeasure IFC / compute_final IFC 三路径 font_size 解析不一致（任一统一都回归 multicol-fill-auto 等）。advance-width 接入须**三路径同源**：

- **缓解**：`AdvanceSource` 是纯度量函数（不改变 font_size 解析），三路径用**同一 source 实例** → 度量一致。区别于 R125 的 font_size 存储问题。但仍须每轮全量 reftest 守 multicol-fill-auto-001 / font-feature / BFC-004 等 R125 敏感用例。
- **自源中性**：reftest test/ref 同用 source → 同改 → 同源 439/490 应不变（除非换行点翻转）。chromium 一致率才反映真实改善（须 oracle 重测）。

## 渐进实施（每轮提交+全量验证）

| 轮 | 内容 | 行为变更 | 验证 |
|----|------|----------|------|
| R1 | 加 `AdvanceSource` trait + `EstimateAdvance` 默认实现 + `TextRun.font_id` 字段（构造处填 None） | **零**（默认=estimate） | make test 全绿、reftest 439/490 持平 |
| R2 | IFC 函数签名加 `&dyn AdvanceSource` 参数（默认传 `&EstimateAdvance`），内部调用改 `source.measure` | **零**（默认实现等价） | 同上 |
| R3 | engine 构造 `FontLoaderAdvance` 注入；TextRun 构造从 style 解析 font_id（需 font_resolver 透传 IFC） | **真实 advance 启用** | make test 全绿 + **reftest 必须 439/490 持平或 net≥0**（守 R125 敏感用例） |
| R4 | 扩展到 intrinsic_sizing + paint list marker | 同上 | 同上 |
| R5 | chromium oracle 重测，量化 183-case 噪声下降 | — | cross-validate 对比 R221 基线 |

## 成功标准

- R3 后 reftest 439/490 **不退化**（self-source 中性应持平；若净翻负则回退 R3，证 self-source 非中性）。
- R5 后 chromium 一致率 **严格真通过率从 39.6% 上行**（183-case 噪声桶下降）。
- 不破坏 R125 敏感用例（multicol-fill-auto-001 / font-feature-001/002 / BFC-004 / font-051）。

## 不做

- 不改 fontdue 光栅化（AA 基准已证 ≈ chromium）。
- 不改 rustybuzz shaping（paint glyph 定位用 IFC 的 x，非独立 shaping）。
- 不引入新 crate（FontLoader::measure_advance 已存在）。
- **不走「单点改 estimate_char_width 实测表」捷径**（R224 实验已证否）：曾用 DejaVu Sans 实测 advance 表替换固定倍数，全量 reftest 439→436 净 -3 回归。estimate 并非纯自源中性（test/ref 文本结构不同时换行点敏感度不同），单点扰动破坏同源对齐。必须完整接入 FontLoader（R2-R5，三处同源替换）。
