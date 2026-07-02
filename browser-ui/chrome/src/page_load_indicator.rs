//! PageLoadIndicator — 页面加载进度（spec §8.4.1A）。
//!
//! 作为 toolbar 或 tab 内的领域组件，绘制走通用 [`ProgressIndicator`]（不绕过 ui/render）。
//! navigation progress / loading state 来自 browser-shell navigation state。

use zero_ui_widgets::progress::ProgressIndicator;

/// 页面加载进度状态（props）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PageLoadIndicator {
    pub loading: bool,
    /// `0.0..=1.0`；`None` = 不确定进度（仅知道在加载）。
    pub fraction: Option<f32>,
}

impl PageLoadIndicator {
    pub fn idle() -> PageLoadIndicator {
        PageLoadIndicator {
            loading: false,
            fraction: None,
        }
    }

    pub fn loading(fraction: Option<f32>) -> PageLoadIndicator {
        PageLoadIndicator {
            loading: true,
            fraction,
        }
    }

    /// 构造通用进度指示器；非 loading 时返回 `None`（chrome 不显示）。
    pub fn build_indicator(&self) -> Option<ProgressIndicator> {
        if !self.loading {
            return None;
        }
        Some(match self.fraction {
            Some(f) => ProgressIndicator::determinate(f.clamp(0.0, 1.0)),
            None => ProgressIndicator::indeterminate(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_has_no_indicator() {
        assert!(PageLoadIndicator::idle().build_indicator().is_none());
    }

    #[test]
    fn loading_without_fraction_is_indeterminate() {
        let ind = PageLoadIndicator::loading(None).build_indicator().unwrap();
        assert!(ind.is_indeterminate());
        assert!(ind.fraction().is_none());
    }

    #[test]
    fn loading_with_fraction_is_determinate_and_clamped() {
        let ind = PageLoadIndicator::loading(Some(0.5)).build_indicator().unwrap();
        assert!(!ind.is_indeterminate());
        assert_eq!(ind.fraction(), Some(0.5));

        // 超界 clamp 到 [0,1]。
        let clamped = PageLoadIndicator::loading(Some(2.0)).build_indicator().unwrap();
        assert_eq!(clamped.fraction(), Some(1.0));
        let clamped_lo = PageLoadIndicator::loading(Some(-1.0)).build_indicator().unwrap();
        assert_eq!(clamped_lo.fraction(), Some(0.0));
    }
}
