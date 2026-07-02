//! # zero-ui-devtools
//!
//! 开发工具（spec §8.4.1 `zero-ui-devtools` / FR-016）。
//!
//! M1 skeleton：inspector（选中节点）+ timeline（帧时间记录）。
//! 完整 inspector/layout bounds/perf timeline 随 ui/testing snapshot 在后续里程碑填实。

use zero_ui_core::widget::WidgetId;

/// Inspector：追踪当前选中节点 + 是否显示 layout bounds。
#[derive(Debug, Clone, Default)]
pub struct Inspector {
    pub selected: Option<WidgetId>,
    pub show_layout_bounds: bool,
}

impl Inspector {
    pub fn new() -> Inspector {
        Inspector::default()
    }
    pub fn select(&mut self, id: WidgetId) {
        self.selected = Some(id);
    }
    pub fn clear(&mut self) {
        self.selected = None;
    }
}

/// 性能时间线：按帧记录耗时（ms）。
#[derive(Debug, Clone, Default)]
pub struct Timeline {
    pub frame_ms: Vec<f32>,
}

impl Timeline {
    pub fn new() -> Timeline {
        Timeline::default()
    }
    pub fn record(&mut self, ms: f32) {
        self.frame_ms.push(ms);
    }
    /// 最近 N 帧平均耗时。
    pub fn average_recent(&self, n: usize) -> f32 {
        let start = self.frame_ms.len().saturating_sub(n);
        let slice = &self.frame_ms[start..];
        if slice.is_empty() {
            0.0
        } else {
            slice.iter().sum::<f32>() / slice.len() as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspector_select_clear() {
        let mut insp = Inspector::new();
        insp.select(WidgetId::new("btn"));
        assert_eq!(insp.selected, Some(WidgetId::new("btn")));
        insp.clear();
        assert!(insp.selected.is_none());
    }

    #[test]
    fn timeline_average() {
        let mut t = Timeline::new();
        for ms in [16.0, 16.0, 32.0, 16.0] {
            t.record(ms);
        }
        let avg = t.average_recent(3); // 最近 3 帧：16, 32, 16
        assert!((avg - 21.333).abs() < 0.1);
    }
}
