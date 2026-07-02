//! ProgressIndicator — 进度指示器（spec FR-009）。
//!
//! `value = None` 表示不确定（indeterminate）进度；`Some(f)` 为 0..=1 的完成比例。

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressIndicator {
    value: Option<f32>,
}

impl ProgressIndicator {
    pub fn determinate(fraction: f32) -> ProgressIndicator {
        ProgressIndicator { value: Some(fraction) }
    }
    pub fn indeterminate() -> ProgressIndicator {
        ProgressIndicator { value: None }
    }

    /// 归一化到 0..=1 的进度；indeterminate 返回 None。
    pub fn fraction(&self) -> Option<f32> {
        self.value.map(|f| f.clamp(0.0, 1.0))
    }

    pub fn is_indeterminate(&self) -> bool {
        self.value.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_fraction() {
        assert_eq!(ProgressIndicator::determinate(-0.5).fraction(), Some(0.0));
        assert_eq!(ProgressIndicator::determinate(0.4).fraction(), Some(0.4));
        assert_eq!(ProgressIndicator::determinate(2.0).fraction(), Some(1.0));
    }

    #[test]
    fn indeterminate_has_no_fraction() {
        let p = ProgressIndicator::indeterminate();
        assert!(p.is_indeterminate());
        assert_eq!(p.fraction(), None);
    }
}
