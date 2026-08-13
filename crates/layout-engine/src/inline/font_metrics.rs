//! Phase A font-metric 桥接 — 真实字体行度量（fontdue line_metrics）接入 IFC。
//!
//! 替代 IFC 硬编码 `0.8` ascent 近似（见 `inline/mod.rs::apply_vertical_alignment`
//! 中 `dominant_fs * 0.8` / `container_font_size * 0.8`）。本模块为 Phase A §12.6
//! step-1 的 enabling infra：仅定义 trait + `FontLoader` 实现 + IFC 可选字段，
//! **不改变 `0.8` 行为**（零回归）。step-2（三方协调）才在 `apply_vertical_alignment`
//! 中消费真实 ascent/descent/line_gap，替换 `0.8` 启发式。
//!
//! 设计参照 advance-width plumbing（`AdvanceSource` trait，R223），但本桥接针对
//! **行度量**（ascent/descent/line_gap，用于垂直/基线定位），**非** advance-width
//! （字符宽度/换行，R225–R375b 已 definitive 证伪为渲染差异根因，勿混淆）。
//!
//! 关键事实（§12.2 三方补偿）：`0.8` 常数对 Ahem 字体**恰好正确**（fontdue 实测
//! Ahem ascent=800/units_per_em=1000 = 0.8em），但对真实字体（system-ui / DejaVu /
//! NotoSansCJK）ascent ≈ 0.928em，故 `0.8` 对非-Ahem 文本基线偏低（welcome/morning
//! 文本度量残余主因之一）。本桥接暴露 per-font 真实度量，使 step-2 能按字体取真实
//! ascent（Ahem 仍 0.8em，非-Ahem 改用 ~0.928em）。

use std::rc::Rc;

use zero_render_foundation::font::{FontFamilyMetricMap, FontLoader};

/// 真实字体行度量（来自 fontdue OS/2 / hhea），按字号缩放后的 px 值。
///
/// 符号约定沿用 fontdue（与 chromium `line-height:normal = ascent − descent + line_gap`
/// 计算一致）：
/// - `ascent`：基线到字形网格顶部的距离，**正值**。
/// - `descent`：基线到字形网格底部的距离，**负值**（fontdue 约定）。故
///   `ascent − descent` 即 em-box 高度（Ahem: 0.8 − (−0.2) = 1.0em）。
/// - `line_gap`：字体推荐行间距（OS/2 sTypoLineGap / hhea lineGap），通常 0 或小正值。
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct LineMetrics {
    /// 基线到顶（px，正值）。
    pub ascent: f32,
    /// 基线到底（px，**负值**，fontdue 约定）。
    pub descent: f32,
    /// 行间距（px）。
    pub line_gap: f32,
}

/// IFC 字体度量提供者（依赖反转）。
///
/// 默认 IFC 不持有 provider（`font_metric_provider = None`），`apply_vertical_alignment`
/// 回退到 `0.8·fs` 启发式（保持当前行为，零回归）。`zero-engine` 可在构造 IFC 时注入
/// `FontLoader`-backed 实现（`Rc<dyn FontMetricProvider>`），供 Phase A step-2 在
/// strut baseline / half-leading 计算中消费真实度量。
///
/// 为什么用 trait 而非直接持有 `&FontLoader`：IFC 当前是无生命周期的 owned 结构，
/// trait 对象（`Rc<dyn>`）避免给 `InlineFormattingContext` 引入生命周期参数；同时允许
/// 单测用桩实现，且与 `AdvanceSource` 既有 seam 风格一致。
pub trait FontMetricProvider {
    /// 按 CSS `font-family` 列表 + 字号查询真实行度量。
    ///
    /// - `font_family`：CSS font-family 候选列表（与 `ComputedStyle.font_family`
    ///   同形，已展开为字符串），实现按优先级解析首个已加载字体。
    /// - `size`：字号（px）。
    ///
    /// 返回 `None` 表示无匹配的已加载字体（或字体无度量），IFC 应回退到 `0.8` 启发式
    /// （零回归）。调用方不得假设一定有值。
    fn line_metrics(&self, font_family: &[String], size: f32) -> Option<LineMetrics>;

    /// 解析 CSS `font-family` 列表到具体 font_id（C3 advance 解锁，R223 font_id gap）。
    ///
    /// IFC 在 `TextRun` 构造处经本方法 populate `run.font_id`（当前恒 `None`，致
    /// `advance_of` 即使注入 `AdvanceSource` 也收到 `None` → 回退 estimate）。默认
    /// `None`（保持现有行为 + 不破坏桩实现）；`FontLoader`-backed 实现覆写为真实
    /// `build_font_resolver` 解析。**dormant**：生产 IFC 的 provider 默认 `None`，
    /// 故 `font_id` 仍 `None` = 零回归；待 U1b-wiring provider 注入后自动 populate。
    fn font_id_of(&self, _font_family: &[String]) -> Option<u32> {
        None
    }

    /// 返回 first available font 的字体相对 metric aspect。
    fn font_metric_aspect(
        &self,
        _font_family: &[String],
        _metric: zero_style_system::FontSizeAdjustMetric,
    ) -> Option<f32> {
        None
    }

    /// 返回 first available face 经 `@font-face size-adjust` 缩放后的行度量。
    ///
    /// 未声明 descriptor 或 scale=100% 时返回 `None`，保持普通字体的全局
    /// `ZW_PERFONT_LINEHEIGHT` 策略不变。
    fn size_adjusted_line_metrics(&self, _font_family: &[String], _size: f32) -> Option<LineMetrics> {
        None
    }
}

/// 持有 `FontMetricProvider` 的 trait 对象句柄。
///
/// 单独定义（而非直接用 `Option<Rc<dyn FontMetricProvider>>` 字段）是因为
/// `InlineFormattingContext` 派生了 `Debug`，而 `dyn FontMetricProvider` 非自动
/// `Debug`（且 `FontLoader` 因含 `fontdue::Font` 不可 `Debug`）。本 newtype 提供
/// 手动 `Debug`，使 IFC 的 derive 不受影响。内部 `Rc` 允许 engine 与 IFC 共享同一
/// `FontLoader` 而不引入生命周期参数。
#[derive(Clone)]
pub struct FontMetricProviderHandle(pub(crate) Rc<dyn FontMetricProvider>);

impl std::fmt::Debug for FontMetricProviderHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontMetricProviderHandle").finish_non_exhaustive()
    }
}

impl FontMetricProviderHandle {
    /// 经由内部 provider 查询真实行度量。
    ///
    /// Phase A step-2 在 `apply_vertical_alignment` 中通过此方法消费真实度量
    /// （替换 `0.8` 启发式）；step-1 仅提供接口，IFC 默认 `None` 不调用本方法。
    pub fn line_metrics(&self, font_family: &[String], size: f32) -> Option<LineMetrics> {
        self.0.line_metrics(font_family, size)
    }

    /// 经由内部 provider 解析 family → font_id（C3 advance，R223 gap）。
    pub fn font_id_of(&self, font_family: &[String]) -> Option<u32> {
        self.0.font_id_of(font_family)
    }

    /// 查询字体相对 metric aspect。
    pub fn font_metric_aspect(
        &self,
        font_family: &[String],
        metric: zero_style_system::FontSizeAdjustMetric,
    ) -> Option<f32> {
        self.0.font_metric_aspect(font_family, metric)
    }

    /// 查询 first available face 经 `@font-face size-adjust` 缩放后的行度量。
    pub fn size_adjusted_line_metrics(&self, font_family: &[String], size: f32) -> Option<LineMetrics> {
        self.0.size_adjusted_line_metrics(font_family, size)
    }
}

/// `FontLoader`-backed 实现：解析 family → font_id → fontdue `line_metrics_full`。
///
/// `layout-engine` 已依赖 `zero-render-foundation`（Cargo.toml），故可直接为本具体类型
/// 实现 IFC 定义的 trait。family 解析复用 `FontLoader::build_font_resolver`（族名 → id）。
impl FontMetricProvider for FontLoader {
    fn line_metrics(&self, font_family: &[String], size: f32) -> Option<LineMetrics> {
        let resolver = self.build_font_resolver();
        // 按优先级解析首个已加载字体：先精确匹配，再大小写不敏感回退（与 IFC
        // `is_ahem` 检测的 `eq_ignore_ascii_case` 一致）。
        let font_id = font_family.iter().find_map(|fam| {
            let bare = fam.trim_matches('"').trim_matches('\'');
            resolver.get(bare).copied().or_else(|| {
                resolver
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(bare))
                    .map(|(_, v)| *v)
            })
        })?;
        let (ascent, descent, line_gap) = self.line_metrics_full(font_id, size)?;
        Some(LineMetrics {
            ascent,
            descent,
            line_gap,
        })
    }

    /// C3 advance（R223 font_id gap）：解析 family → font_id，复用 `line_metrics` 的
    /// 优先级 + 大小写不敏感匹配逻辑（build_font_resolver + 回退）。
    fn font_id_of(&self, font_family: &[String]) -> Option<u32> {
        let resolver = self.build_font_resolver();
        font_family.iter().find_map(|fam| {
            let bare = fam.trim_matches('"').trim_matches('\'');
            resolver.get(bare).copied().or_else(|| {
                resolver
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(bare))
                    .map(|(_, v)| *v)
            })
        })
    }

    fn font_metric_aspect(
        &self,
        font_family: &[String],
        metric: zero_style_system::FontSizeAdjustMetric,
    ) -> Option<f32> {
        let font_id = <Self as FontMetricProvider>::font_id_of(self, font_family)?;
        let metric = match metric {
            zero_style_system::FontSizeAdjustMetric::ExHeight => {
                zero_render_foundation::font::FontSizeAdjustMetric::ExHeight
            }
            zero_style_system::FontSizeAdjustMetric::ChWidth => {
                zero_render_foundation::font::FontSizeAdjustMetric::ChWidth
            }
            _ => return None,
        };
        zero_render_foundation::font::FontLoader::font_metric_aspect(self, font_id, metric)
    }

    fn size_adjusted_line_metrics(&self, font_family: &[String], size: f32) -> Option<LineMetrics> {
        let font_id = <Self as FontMetricProvider>::font_id_of(self, font_family)?;
        let scale = zero_render_foundation::font::FontLoader::font_size_scale(self, font_id);
        if (scale - 1.0).abs() <= f32::EPSILON {
            return None;
        }
        let (ascent, descent, line_gap) = self.line_metrics_full(font_id, size * scale)?;
        Some(LineMetrics {
            ascent,
            descent,
            line_gap,
        })
    }
}

/// HashMap-backed provider（字体相对 metric + 可选 per-font line-height）。
///
/// 持有 app/runner 启动期从 `FontLoader::build_line_metric_map()` 构建的 per-family
/// per-em 度量（拥有所有权，无生命周期/Rc-share 问题——runner 不能 Rc-share FontLoader
/// 因 painter &mut 占用）。`line_metrics` 按 family 匹配 + 按 `size` 缩放 per-em 比率
/// （fontdue 线性，等价 `FontLoader::line_metrics_full(id, size)`）；`font_id_of` 返回
/// family→id（启用 C3 font_id population）。family 匹配：精确 + 大小写不敏感（同 FontLoader impl）。
pub struct FontMetricMap {
    map: FontFamilyMetricMap,
    line_metrics_enabled: bool,
}

impl FontMetricMap {
    /// 创建 metric map；`line_metrics_enabled=false` 时仅暴露 font ID 与 aspect。
    pub fn new(map: FontFamilyMetricMap, line_metrics_enabled: bool) -> Self {
        Self {
            map,
            line_metrics_enabled,
        }
    }
}

impl FontMetricProvider for FontMetricMap {
    fn line_metrics(&self, font_family: &[String], size: f32) -> Option<LineMetrics> {
        if !self.line_metrics_enabled {
            return None;
        }
        let metrics = font_family.iter().find_map(|fam| {
            let bare = fam.trim_matches('"').trim_matches('\'');
            self.map.get(bare).or_else(|| {
                self.map
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(bare))
                    .map(|(_, v)| v)
            })
        })?;
        Some(LineMetrics {
            ascent: metrics.ascent * size,
            descent: metrics.descent * size,
            line_gap: metrics.line_gap * size,
        })
    }

    fn font_id_of(&self, font_family: &[String]) -> Option<u32> {
        font_family.iter().find_map(|fam| {
            let bare = fam.trim_matches('"').trim_matches('\'');
            self.map
                .get(bare)
                .or_else(|| {
                    self.map
                        .iter()
                        .find(|(k, _)| k.eq_ignore_ascii_case(bare))
                        .map(|(_, v)| v)
                })
                .map(|metrics| metrics.font_id)
        })
    }

    fn font_metric_aspect(
        &self,
        font_family: &[String],
        metric: zero_style_system::FontSizeAdjustMetric,
    ) -> Option<f32> {
        let metrics = font_family.iter().find_map(|fam| {
            let bare = fam.trim_matches('"').trim_matches('\'');
            self.map.get(bare).or_else(|| {
                self.map
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(bare))
                    .map(|(_, value)| value)
            })
        })?;
        match metric {
            zero_style_system::FontSizeAdjustMetric::ExHeight => Some(metrics.ex_height),
            zero_style_system::FontSizeAdjustMetric::CapHeight => Some(metrics.cap_height),
            zero_style_system::FontSizeAdjustMetric::ChWidth => Some(metrics.ch_width),
            _ => None,
        }
    }

    fn size_adjusted_line_metrics(&self, font_family: &[String], size: f32) -> Option<LineMetrics> {
        let metrics = font_family.iter().find_map(|fam| {
            let bare = fam.trim_matches('"').trim_matches('\'');
            self.map.get(bare).or_else(|| {
                self.map
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(bare))
                    .map(|(_, v)| v)
            })
        })?;
        if (metrics.size_adjust - 1.0).abs() <= f32::EPSILON {
            return None;
        }
        Some(LineMetrics {
            ascent: metrics.ascent * size * metrics.size_adjust,
            descent: metrics.descent * size * metrics.size_adjust,
            line_gap: metrics.line_gap * size * metrics.size_adjust,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    fn family_metrics(size_adjust: f32) -> zero_render_foundation::font::FontFamilyMetrics {
        zero_render_foundation::font::FontFamilyMetrics {
            font_id: 7,
            ascent: 0.8,
            descent: -0.2,
            line_gap: 0.1,
            ex_height: 0.5,
            cap_height: 0.7,
            ch_width: 0.6,
            size_adjust,
        }
    }

    /// Ahem.ttf 位于 workspace 根的 `tests/wpt-runner/fonts/`（WPT 标准正方形字体）。
    /// 本文件在 `crates/layout-engine/src/inline/`，故 4 级 `..` 回到 workspace 根。
    /// 编译期 `include_bytes!` 烘焙进测试二进制，避免运行期文件依赖。
    const AHEM_TTF: &[u8] = include_bytes!("../../../../tests/wpt-runner/fonts/Ahem.ttf");

    /// 构造一个加载了 Ahem 的 `FontLoader`。失败则跳过测试（与 render-foundation
    /// 既有 `load_system_font_data` 测试同的宽松降级风格）。
    fn ahem_loader() -> Option<FontLoader> {
        let mut loader = FontLoader::new();
        loader.load_font(AHEM_TTF).ok()?;
        Some(loader)
    }

    /// Trait seam：`FontLoader` 实现返回 Ahem 真实行度量。
    ///
    /// fontdue 实测 Ahem.ttf：ascent=800 / descent=−200 / line_gap=0 /
    /// units_per_em=1000（见 `text_metrics.rs` 注释）。故 size=40 时：
    /// ascent=32、descent=−8、line_gap=0，且 `ascent − descent = 40 = 1.0em`。
    #[test]
    fn font_loader_provider_returns_ahem_line_metrics() {
        let Some(loader) = ahem_loader() else {
            eprintln!("skipping: Ahem.ttf failed to load");
            return;
        };
        let m = <FontLoader as FontMetricProvider>::line_metrics(&loader, &["Ahem".to_string()], 40.0)
            .expect("Ahem must resolve via build_font_resolver");
        // ascent 正、descent 负（fontdue 约定）。
        assert!(m.ascent > 0.0, "ascent should be positive, got {}", m.ascent);
        assert!(m.descent < 0.0, "descent should be negative, got {}", m.descent);
        // Ahem ascent = 0.8·size（800/1000）。
        assert!(
            (m.ascent - 32.0).abs() < 0.5,
            "Ahem ascent ≈ 0.8·size = 32, got {}",
            m.ascent
        );
        // Ahem descent = −0.2·size（−200/1000）。
        assert!(
            (m.descent - (-8.0)).abs() < 0.5,
            "Ahem descent ≈ −0.2·size = −8, got {}",
            m.descent
        );
        // Ahem line_gap = 0。
        assert!(m.line_gap.abs() < 0.5, "Ahem line_gap ≈ 0, got {}", m.line_gap);
        // em-box = ascent − descent = 1.0·size = 40。
        assert!(
            (m.ascent - m.descent - 40.0).abs() < 1.0,
            "ascent − descent ≈ 1.0·size = 40, got {}",
            m.ascent - m.descent
        );
    }

    /// 未加载的 family 返回 `None`（IFC 须回退 `0.8` 启发式）。
    #[test]
    fn font_loader_provider_returns_none_for_unknown_family() {
        let Some(loader) = ahem_loader() else {
            eprintln!("skipping: Ahem.ttf failed to load");
            return;
        };
        assert!(FontMetricProvider::line_metrics(&loader, &["DoesNotExist".to_string()], 16.0).is_none());
    }

    /// C3 advance（R223 font_id gap）：FontLoader 解析 family → font_id（Ahem = 已加载）。
    /// 大小写不敏感（CSS font-family）。未加载 family 返回 None。
    #[test]
    fn font_loader_provider_font_id_of_resolves() {
        let Some(loader) = ahem_loader() else {
            eprintln!("skipping: Ahem.ttf failed to load");
            return;
        };
        let id = <FontLoader as FontMetricProvider>::font_id_of(&loader, &["Ahem".to_string()])
            .expect("Ahem must resolve to a font_id");
        assert_eq!(id, 0u32, "first loaded font (Ahem) has id 0");
        // 大小写不敏感。
        assert!(<FontLoader as FontMetricProvider>::font_id_of(&loader, &["aHeM".to_string()]).is_some());
        // 未加载 family → None。
        assert!(<FontLoader as FontMetricProvider>::font_id_of(&loader, &["DoesNotExist".to_string()]).is_none());
        // 多 family 列表：取首个已加载（Ahem 在第二位也解析）。
        assert!(
            <FontLoader as FontMetricProvider>::font_id_of(&loader, &["Missing".to_string(), "Ahem".to_string()])
                .is_some()
        );
    }

    /// 大小写不敏感匹配（CSS font-family 大小写不敏感；与 IFC `is_ahem` 检测一致）。
    #[test]
    fn font_loader_provider_matches_family_case_insensitively() {
        let Some(loader) = ahem_loader() else {
            eprintln!("skipping: Ahem.ttf failed to load");
            return;
        };
        assert!(
            FontMetricProvider::line_metrics(&loader, &["aHeM".to_string()], 40.0).is_some(),
            "family matching should be case-insensitive"
        );
    }

    /// Zero-regression 默认：`InlineFormattingContext::new()` 的 provider 为 `None`
    /// （`0.8` 启发式路径活跃，行为不变）；`with_font_metric_provider` 注入后为 `Some`
    /// 且可查询。证明 step-1 仅添加 dormant 字段，未触及 `apply_vertical_alignment`。
    #[test]
    fn ifc_font_metric_provider_defaults_none_and_is_injectable() {
        use crate::inline::InlineFormattingContext;

        // 默认 dormant → 0.8 启发式路径不变。
        let ctx = InlineFormattingContext::new(800.0);
        assert!(
            ctx.font_metric_provider.is_none(),
            "IFC must default to no provider (0.8 heuristic active = zero behavior change)"
        );

        // 注入 FontLoader-backed provider，字段变 Some 且可经 trait 查询到 Ahem 度量。
        let Some(loader) = ahem_loader() else {
            eprintln!("skipping injection check: Ahem.ttf failed to load");
            return;
        };
        let provider: Rc<dyn FontMetricProvider> = Rc::new(loader);
        let ctx = InlineFormattingContext::new(800.0).with_font_metric_provider(provider);
        let p = ctx
            .font_metric_provider
            .as_ref()
            .expect("provider should be set after with_font_metric_provider");
        let m = p
            .line_metrics(&["Ahem".to_string()], 40.0)
            .expect("injected provider should resolve Ahem");
        assert!(
            (m.ascent - 32.0).abs() < 0.5,
            "injected provider ascent ≈ 32, got {}",
            m.ascent
        );
    }

    // ── U1b：resolve_font_metrics_with_provider 首消费者测试 ────────────────

    /// 桩 provider：返回固定的 per-em 行度量（已按 size 缩放为 px），或 `None`（模拟
    /// 字体未加载）。用于证明 `resolve_font_metrics_with_provider` 在 provider 存在时
    /// 真正咨询了字体度量，而非静默回退常数。
    struct MockMetricProvider {
        ascent_per_em: f32,
        descent_per_em: f32,
        line_gap_per_em: f32,
        resolve: bool,
        aspect: Option<f32>,
    }

    impl FontMetricProvider for MockMetricProvider {
        fn line_metrics(&self, _font_family: &[String], size: f32) -> Option<LineMetrics> {
            if !self.resolve {
                return None;
            }
            Some(LineMetrics {
                ascent: self.ascent_per_em * size,
                descent: self.descent_per_em * size,
                line_gap: self.line_gap_per_em * size,
            })
        }

        fn font_metric_aspect(
            &self,
            _font_family: &[String],
            _metric: zero_style_system::FontSizeAdjustMetric,
        ) -> Option<f32> {
            self.aspect
        }
    }

    /// 构造一个 line-height:normal + 给定 font-family/font-size 的 ComputedStyle。
    fn normal_style(family: &str, size_px: f32) -> zero_style_system::ComputedStyle {
        use zero_css_parser::values::LengthValue;
        use zero_style_system::ComputedStyle;
        let mut s = ComputedStyle::default();
        s.font_family = vec![family.to_string()];
        s.font_size = LengthValue::Px(size_px as f64);
        s.line_height = zero_style_system::LineHeightValue::Normal;
        s
    }

    // ── R1192 font-size-adjust is_ahem-gated apply 测试 ────────────────────

    /// font-size-adjust:0.9 + Ahem → adjusted font_size = 40 × 0.9 / 0.8 = 45
    /// （chromium OS/2 sxHeight=800/upem=1000=0.8；ref font-size-adjust-001 adjusted=45px）。
    /// AHEM_FONT_SIZE_ADJUST_ASPECT=0.8。证 apply 触发 + 公式正确。
    #[test]
    fn resolve_font_metrics_font_size_adjust_ahem_scales() {
        use zero_css_parser::values::LengthValue;
        use zero_style_system::ComputedStyle;
        let mut s = ComputedStyle::default();
        s.font_family = vec!["Ahem".to_string()];
        s.font_size = LengthValue::Px(40.0);
        s.line_height = zero_style_system::LineHeightValue::Normal;
        s.font_size_adjust = zero_style_system::FontSizeAdjustValue::Adjust {
            metric: None,
            basis: zero_style_system::FontSizeAdjustBasis::Number(0.9),
        };
        let (fs, _lh) = super::super::resolve_font_metrics_with_provider(Some(&s), None);
        assert!(
            (fs - 45.0).abs() < 1e-3,
            "Ahem font-size-adjust:0.9 @40px → 45px (40×0.9/0.8), got {fs}"
        );
    }

    /// font-size-adjust < aspect（0.2 < 0.8）→ adjusted < font_size（font-size-adjust-002：
    /// blue < orange）。40 × 0.2 / 0.8 = 10。
    #[test]
    fn resolve_font_metrics_font_size_adjust_ahem_smaller() {
        use zero_css_parser::values::LengthValue;
        use zero_style_system::ComputedStyle;
        let mut s = ComputedStyle::default();
        s.font_family = vec!["Ahem".to_string()];
        s.font_size = LengthValue::Px(40.0);
        s.line_height = zero_style_system::LineHeightValue::Normal;
        s.font_size_adjust = zero_style_system::FontSizeAdjustValue::Adjust {
            metric: None,
            basis: zero_style_system::FontSizeAdjustBasis::Number(0.2),
        };
        let (fs, _lh) = super::super::resolve_font_metrics_with_provider(Some(&s), None);
        assert!(
            (fs - 10.0).abs() < 1e-3,
            "Ahem font-size-adjust:0.2 @40px → 10px (40×0.2/0.8), got {fs}"
        );
    }

    /// 非 Ahem 字体：font-size-adjust 暂不 apply（aspect 未知，Slice 3）→ font_size 不变。
    #[test]
    fn resolve_font_metrics_font_size_adjust_non_ahem_no_apply() {
        use zero_css_parser::values::LengthValue;
        use zero_style_system::ComputedStyle;
        let mut s = ComputedStyle::default();
        s.font_family = vec!["DejaVu".to_string()];
        s.font_size = LengthValue::Px(40.0);
        s.line_height = zero_style_system::LineHeightValue::Normal;
        s.font_size_adjust = zero_style_system::FontSizeAdjustValue::Adjust {
            metric: None,
            basis: zero_style_system::FontSizeAdjustBasis::Number(0.9),
        };
        let (fs, _lh) = super::super::resolve_font_metrics_with_provider(Some(&s), None);
        assert!(
            (fs - 40.0).abs() < 1e-3,
            "non-Ahem font-size-adjust should NOT apply (Slice 3), got {fs}"
        );
    }

    #[test]
    fn font_size_adjust_scales_normal_line_height_without_changing_run_size() {
        let mut style = normal_style("MetricFont", 20.0);
        style.font_size_adjust = zero_style_system::FontSizeAdjustValue::Adjust {
            metric: None,
            basis: zero_style_system::FontSizeAdjustBasis::Number(1.0),
        };
        let provider = MockMetricProvider {
            ascent_per_em: 0.0,
            descent_per_em: 0.0,
            line_gap_per_em: 0.0,
            resolve: false,
            aspect: Some(0.5),
        };
        let handle = FontMetricProviderHandle(Rc::new(provider));
        let (font_size, line_height) = super::super::resolve_font_metrics_with_provider(Some(&style), Some(&handle));
        assert_eq!(font_size, 20.0, "shaping must receive the specified size");
        assert!(
            (line_height - 40.0 * super::super::NORMAL_LINE_HEIGHT_RATIO).abs() < 1e-3,
            "normal line-height must use the adjusted 40px primary size, got {line_height}"
        );
    }

    #[test]
    fn font_metric_map_exposes_aspect_without_enabling_line_metrics() {
        let entries = std::collections::HashMap::from([("MetricFont".to_string(), family_metrics(1.0))]);
        let dormant = FontMetricMap::new(entries.clone(), false);
        let family = ["metricfont".to_string()];
        assert_eq!(dormant.font_id_of(&family), Some(7));
        assert_eq!(
            dormant.font_metric_aspect(&family, zero_style_system::FontSizeAdjustMetric::ExHeight),
            Some(0.5)
        );
        assert!(dormant.line_metrics(&family, 20.0).is_none());

        let enabled = FontMetricMap::new(entries, true);
        assert_eq!(
            enabled.line_metrics(&family, 20.0),
            Some(LineMetrics {
                ascent: 16.0,
                descent: -4.0,
                line_gap: 2.0,
            })
        );
    }

    #[test]
    fn size_adjust_descriptor_scales_normal_line_without_enabling_all_line_metrics() {
        let mut metrics = family_metrics(0.5);
        metrics.line_gap = 0.0;
        let entries = std::collections::HashMap::from([("AdjustedFont".to_string(), metrics)]);
        let provider = FontMetricProviderHandle(Rc::new(FontMetricMap::new(entries, false)));
        let style = normal_style("AdjustedFont", 20.0);

        let (font_size, line_height) = super::super::resolve_font_metrics_with_provider(Some(&style), Some(&provider));
        assert_eq!(font_size, 20.0, "computed font-size must remain unchanged");
        assert_eq!(
            line_height, 10.0,
            "normal line metrics must use the descriptor-adjusted 10px size"
        );
    }

    #[test]
    fn font_size_adjust_property_preempts_descriptor_normal_line_scale() {
        let mut metrics = family_metrics(0.5);
        metrics.line_gap = 0.0;
        let entries = std::collections::HashMap::from([("AdjustedFont".to_string(), metrics)]);
        let provider = FontMetricProviderHandle(Rc::new(FontMetricMap::new(entries, false)));
        let mut style = normal_style("AdjustedFont", 20.0);
        style.font_size_adjust = zero_style_system::FontSizeAdjustValue::Adjust {
            metric: None,
            basis: zero_style_system::FontSizeAdjustBasis::Number(0.5),
        };

        let (_, line_height) = super::super::resolve_font_metrics_with_provider(Some(&style), Some(&provider));
        assert!(
            (line_height - 20.0 * super::super::NORMAL_LINE_HEIGHT_RATIO).abs() < 1e-3,
            "property must preempt the descriptor scale, got {line_height}"
        );
    }

    /// font-size-adjust: None（默认）→ 不调整，font_size 不变。
    #[test]
    fn resolve_font_metrics_font_size_adjust_none_no_change() {
        let s = normal_style("Ahem", 40.0);
        // normal_style 默认 font_size_adjust = None
        let (fs, _lh) = super::super::resolve_font_metrics_with_provider(Some(&s), None);
        assert!((fs - 40.0).abs() < 1e-3, "font-size-adjust:None → no change, got {fs}");
    }

    /// provider 存在并解析字体时，line-height:normal 用 per-font 真实度量
    /// （`ascent − descent + line_gap`），**而非** DejaVu 常数 1.164。
    /// 选 distinctive 比率 0.9（≠1.0 Ahem、≠1.164 DejaVu）以证明 provider 路径被走。
    #[test]
    fn resolve_font_metrics_with_provider_uses_per_font_metric() {
        let style = normal_style("TestFont", 20.0);
        // ascent=0.6em / descent=-0.2em / line_gap=0.1em → ratio = 0.9
        let provider = MockMetricProvider {
            ascent_per_em: 0.6,
            descent_per_em: -0.2,
            line_gap_per_em: 0.1,
            resolve: true,
            aspect: None,
        };
        let handle = FontMetricProviderHandle(Rc::new(provider));
        let (fs, lh) = super::super::resolve_font_metrics_with_provider(Some(&style), Some(&handle));
        assert!((fs - 20.0).abs() < 1e-6, "font_size unchanged, got {fs}");
        // per-font line-height = 20 × (0.6 + 0.2 + 0.1) = 18.0
        assert!(
            (lh - 18.0).abs() < 1e-3,
            "line-height should be per-font 18.0 (0.9·fs), got {lh}"
        );
        // 关键：不等于常数回退 20×1.164=23.28
        assert!(
            (lh - 20.0 * super::super::NORMAL_LINE_HEIGHT_RATIO).abs() > 0.1,
            "per-font path must differ from constant fallback"
        );
    }

    /// provider 为 `None` 时逐字节等价于 `resolve_font_metrics`（常数回退）。
    /// 这是生产零回归保证：IFC `font_metric_provider` 默认 None。
    #[test]
    fn resolve_font_metrics_with_provider_none_is_byte_identical() {
        let style = normal_style("DejaVuLike", 20.0);
        let without = super::super::resolve_font_metrics(Some(&style));
        let with_none = super::super::resolve_font_metrics_with_provider(Some(&style), None);
        assert_eq!(without, with_none, "provider=None must equal resolve_font_metrics");
        // 非-Ahem Normal 回退 1.164
        assert!(
            (with_none.1 - 20.0 * super::super::NORMAL_LINE_HEIGHT_RATIO).abs() < 1e-3,
            "fallback ratio = NORMAL_LINE_HEIGHT_RATIO, got {}",
            with_none.1
        );
    }

    /// provider 存在但无法解析字体（`None`）时回退常数（与无 provider 相同）。
    #[test]
    fn resolve_font_metrics_with_provider_unresolved_falls_back() {
        let style = normal_style("MissingFont", 16.0);
        let provider = MockMetricProvider {
            ascent_per_em: 0.6,
            descent_per_em: -0.2,
            line_gap_per_em: 0.1,
            resolve: false, // 模拟字体未加载
            aspect: None,
        };
        let handle = FontMetricProviderHandle(Rc::new(provider));
        let (fs, lh) = super::super::resolve_font_metrics_with_provider(Some(&style), Some(&handle));
        assert!((fs - 16.0).abs() < 1e-6);
        // 回退常数 1.164，非 per-font 0.9
        assert!(
            (lh - 16.0 * super::super::NORMAL_LINE_HEIGHT_RATIO).abs() < 1e-3,
            "unresolved provider must fall back to constant, got {lh}"
        );
    }

    /// Ahem 字体：provider 解析时返回 per-font 1.0（与 AHEM_LINE_HEIGHT_RATIO 一致），
    /// 证明 provider 对 Ahem 也被咨询（结果恰好等于常数，但路径被走）。
    #[test]
    fn resolve_font_metrics_with_provider_ahem_per_font() {
        use zero_css_parser::values::LengthValue;
        use zero_style_system::ComputedStyle;
        let mut s = ComputedStyle::default();
        s.font_family = vec!["Ahem".to_string()];
        s.font_size = LengthValue::Px(40.0);
        s.line_height = zero_style_system::LineHeightValue::Normal;
        // Ahem 真实度量：ascent=0.8em / descent=-0.2em / line_gap=0 → 1.0
        let provider = MockMetricProvider {
            ascent_per_em: 0.8,
            descent_per_em: -0.2,
            line_gap_per_em: 0.0,
            resolve: true,
            aspect: None,
        };
        let handle = FontMetricProviderHandle(Rc::new(provider));
        let (_, lh) = super::super::resolve_font_metrics_with_provider(Some(&s), Some(&handle));
        assert!(
            (lh - 40.0).abs() < 1e-3,
            "Ahem per-font line-height = 1.0·fs = 40, got {lh}"
        );
    }
}
