//! # zero-ui-devtools
//!
//! 开发工具（spec §8.4.1 `zero-ui-devtools` / FR-016 / §8.4.1B UI 调试与回归）。
//!
//! 提供 [`Inspector`]（选中/悬停节点 + dev overlay 开关：layout bounds / semantics / paint regions，
//! 宿主据此在 scene 上叠加调试绘制）+ [`Timeline`]（帧耗时记录 + min/max/p99/fps/掉帧统计）。
//! dev/test feature gate，不影响 release footprint（spec §8.4.10）。

use zero_ui_core::widget::WidgetId;

// ── Inspector ─────────────────────────────────────────────────────────────────

/// Inspector：追踪当前选中/悬停节点 + dev overlay 开关。
///
/// 宿主在 paint 阶段读 Inspector 状态，对选中节点画 layout bounds 框、对整树画 semantics /
/// paint region 调试叠加。本结构只持有状态，不绘制（绘制走 `ui/render`）。
#[derive(Debug, Clone, Default)]
pub struct Inspector {
    pub selected: Option<WidgetId>,
    pub hovered: Option<WidgetId>,
    /// 画所有节点的外框（layout bounds 调试）。
    pub show_layout_bounds: bool,
    /// 画 a11y 语义区域边界 + role 标注。
    pub show_semantics: bool,
    /// 画每个 widget 的 paint 区域（脏区 / 重绘范围）。
    pub show_paint_regions: bool,
}

impl Inspector {
    pub fn new() -> Inspector {
        Inspector::default()
    }

    /// 选中节点（inspector 点击）。
    pub fn select(&mut self, id: WidgetId) {
        self.selected = Some(id);
    }

    /// 设置悬停节点（inspector 内移动）。
    pub fn hover(&mut self, id: Option<WidgetId>) {
        self.hovered = id;
    }

    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// 是否有任何 dev overlay 打开（host 据此决定是否需要额外调试 paint pass）。
    pub fn any_overlay(&self) -> bool {
        self.show_layout_bounds || self.show_semantics || self.show_paint_regions
    }

    /// 一键开关（测试/调试便捷）。
    pub fn toggle_layout_bounds(&mut self) {
        self.show_layout_bounds = !self.show_layout_bounds;
    }
}

// ── Timeline ──────────────────────────────────────────────────────────────────

/// 性能时间线：按帧记录耗时（ms）+ 派生统计。
#[derive(Debug, Clone)]
pub struct Timeline {
    pub frame_ms: Vec<f32>,
    /// 单帧耗时超过此阈值计为掉帧（默认 20ms ≈ <50fps）。
    pub jank_threshold_ms: f32,
}

impl Default for Timeline {
    fn default() -> Timeline {
        Timeline {
            frame_ms: Vec::new(),
            jank_threshold_ms: 20.0,
        }
    }
}

impl Timeline {
    pub fn new() -> Timeline {
        Timeline::default()
    }

    /// 设定掉帧阈值（ms）。
    pub fn with_jank_threshold(mut self, ms: f32) -> Timeline {
        self.jank_threshold_ms = ms;
        self
    }

    pub fn record(&mut self, ms: f32) {
        self.frame_ms.push(ms);
    }

    /// 最近 N 帧平均耗时（ms）。
    pub fn average_recent(&self, n: usize) -> f32 {
        let start = self.frame_ms.len().saturating_sub(n);
        let slice = &self.frame_ms[start..];
        if slice.is_empty() {
            0.0
        } else {
            slice.iter().sum::<f32>() / slice.len() as f32
        }
    }

    /// 全量最小帧耗时（ms）。无数据 → 0.0。
    pub fn min_ms(&self) -> f32 {
        if self.frame_ms.is_empty() {
            return 0.0;
        }
        self.frame_ms.iter().copied().fold(f32::INFINITY, f32::min)
    }

    /// 全量最大帧耗时（ms）。
    pub fn max_ms(&self) -> f32 {
        self.frame_ms.iter().copied().fold(0.0_f32, f32::max)
    }

    /// p 百分位帧耗时（0..=100，如 99 = p99）。无数据 → 0.0。
    pub fn percentile(&self, p: f32) -> f32 {
        if self.frame_ms.is_empty() {
            return 0.0;
        }
        let mut sorted: Vec<f32> = self.frame_ms.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p = p.clamp(0.0, 100.0);
        let idx = ((p / 100.0) * (sorted.len() - 1) as f32).round() as usize;
        sorted[idx]
    }

    /// 掉帧数：耗时 > `jank_threshold_ms` 的帧数。
    pub fn jank_count(&self) -> usize {
        self.frame_ms.iter().filter(|ms| **ms > self.jank_threshold_ms).count()
    }

    /// 估算平均 FPS（1000 / 全量平均帧耗时）。无数据 → 0.0。
    pub fn fps(&self) -> f32 {
        if self.frame_ms.is_empty() {
            return 0.0;
        }
        let avg = self.frame_ms.iter().sum::<f32>() / self.frame_ms.len() as f32;
        if avg > 0.0 { 1000.0 / avg } else { 0.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_select_hover_overlays() {
        let mut insp = Inspector::new();
        assert!(!insp.any_overlay(), "no overlay initially");
        insp.select(WidgetId::new("btn"));
        assert_eq!(insp.selected, Some(WidgetId::new("btn")));
        insp.hover(Some(WidgetId::new("row")));
        assert_eq!(insp.hovered, Some(WidgetId::new("row")));
        insp.toggle_layout_bounds();
        assert!(insp.show_layout_bounds);
        assert!(insp.any_overlay(), "overlay active after toggle");
        insp.clear_selection();
        assert!(insp.selected.is_none());
    }

    #[test]
    fn timeline_average_recent() {
        let mut t = Timeline::new();
        for ms in [16.0, 16.0, 32.0, 16.0] {
            t.record(ms);
        }
        let avg = t.average_recent(3); // 最近 3 帧：16, 32, 16
        assert!((avg - 21.333).abs() < 0.1);
    }

    #[test]
    fn timeline_min_max_percentile() {
        let mut t = Timeline::new();
        for ms in [10.0, 20.0, 30.0, 40.0, 50.0] {
            t.record(ms);
        }
        assert_eq!(t.min_ms(), 10.0);
        assert_eq!(t.max_ms(), 50.0);
        // p50 → 中位数 30。
        assert!((t.percentile(50.0) - 30.0).abs() < 1e-4);
        // p100 → 最大 50。
        assert!((t.percentile(100.0) - 50.0).abs() < 1e-4);
    }

    #[test]
    fn timeline_jank_and_fps() {
        let mut t = Timeline::new().with_jank_threshold(20.0);
        // 4 帧：16,16,16,33 → 1 帧 > 20ms 掉帧。
        for ms in [16.0, 16.0, 16.0, 33.0] {
            t.record(ms);
        }
        assert_eq!(t.jank_count(), 1);
        // 平均 = (16+16+16+33)/4 = 20.25 → fps = 1000/20.25 ≈ 49.4。
        assert!((t.fps() - 49.38).abs() < 0.5, "fps got {}", t.fps());
    }

    #[test]
    fn timeline_empty_stats_zero() {
        let t = Timeline::new();
        assert_eq!(t.min_ms(), 0.0);
        assert_eq!(t.max_ms(), 0.0);
        assert_eq!(t.percentile(99.0), 0.0);
        assert_eq!(t.fps(), 0.0);
        assert_eq!(t.jank_count(), 0);
    }
}
