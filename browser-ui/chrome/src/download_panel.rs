//! DownloadPanel / DownloadItemView — 下载面板（spec §8.4.1A）。
//!
//! 组合通用 [`Popover`] + [`ListView`] + [`ProgressIndicator`]（+ 每项 Button/Menu 由 shell 组合）；
//! 下载状态来自 browser-shell download model；打开/取消/显示通过 action dispatch。

use crate::browser_action::BrowserAction;
use zero_ui_core::geometry::Rect;
use zero_ui_widgets::list_view::ListView;
use zero_ui_widgets::popover::{Popover, PopoverPlacement};
use zero_ui_widgets::progress::ProgressIndicator;

/// 下载状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadState {
    /// 进行中。
    InProgress,
    /// 已完成。
    Completed,
    /// 已取消。
    Cancelled,
}

/// 单个下载项视图（props）。
#[derive(Debug, Clone, PartialEq)]
pub struct DownloadItemView {
    pub id: String,
    pub filename: String,
    pub received_bytes: u64,
    /// `None` = 未知总大小（用不确定进度）。
    pub total_bytes: Option<u64>,
    pub state: DownloadState,
}

impl DownloadItemView {
    pub fn in_progress(id: &str, filename: &str, received: u64, total: Option<u64>) -> DownloadItemView {
        DownloadItemView {
            id: id.to_string(),
            filename: filename.to_string(),
            received_bytes: received,
            total_bytes: total,
            state: DownloadState::InProgress,
        }
    }

    pub fn completed(id: &str, filename: &str) -> DownloadItemView {
        DownloadItemView {
            id: id.to_string(),
            filename: filename.to_string(),
            received_bytes: 0,
            total_bytes: None,
            state: DownloadState::Completed,
        }
    }

    /// 组合通用进度指示器：
    /// - 进行中且已知总大小 → 确定性 fraction；
    /// - 已完成 → 1.0；
    /// - 其余（未知大小/已取消）→ 不确定。
    pub fn build_progress(&self) -> ProgressIndicator {
        match (self.state, self.total_bytes) {
            (DownloadState::Completed, _) => ProgressIndicator::determinate(1.0),
            (DownloadState::InProgress, Some(total)) if total > 0 => {
                ProgressIndicator::determinate((self.received_bytes as f32 / total as f32).clamp(0.0, 1.0))
            }
            _ => ProgressIndicator::indeterminate(),
        }
    }

    /// 完成比例（确定进度时为 Some）。
    pub fn fraction(&self) -> Option<f32> {
        self.build_progress().fraction()
    }
}

/// 下载面板（props）。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DownloadPanel {
    pub items: Vec<DownloadItemView>,
}

impl DownloadPanel {
    pub fn new(items: Vec<DownloadItemView>) -> DownloadPanel {
        DownloadPanel { items }
    }

    /// 组合通用 popover（锚定下载按钮下方）。
    pub fn build_popover(&self, anchor: Rect) -> Popover {
        Popover::new(anchor, PopoverPlacement::Below)
    }

    /// 组合通用 ListView（每项一个 DownloadItemView）。
    pub fn build_list(&self) -> ListView {
        ListView::new(self.items.len())
    }

    pub fn on_open(&self, idx: usize) -> Option<BrowserAction> {
        self.items.get(idx).map(|d| BrowserAction::OpenDownload(d.id.clone()))
    }

    pub fn on_cancel(&self, idx: usize) -> Option<BrowserAction> {
        self.items.get(idx).map(|d| BrowserAction::CancelDownload(d.id.clone()))
    }

    pub fn on_show(&self, idx: usize) -> Option<BrowserAction> {
        self.items.get(idx).map(|d| BrowserAction::ShowDownload(d.id.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_known_size_determinate() {
        let item = DownloadItemView::in_progress("d1", "a.zip", 50, Some(200));
        let p = item.build_progress();
        assert!(!p.is_indeterminate());
        assert_eq!(p.fraction(), Some(0.25));
    }

    #[test]
    fn progress_completed_full_and_unknown_indeterminate() {
        let done = DownloadItemView::completed("d2", "b.pdf");
        assert_eq!(done.fraction(), Some(1.0));
        let unknown = DownloadItemView::in_progress("d3", "c.iso", 100, None);
        assert!(unknown.build_progress().is_indeterminate());
    }

    #[test]
    fn panel_builds_popover_and_list() {
        let panel = DownloadPanel::new(vec![
            DownloadItemView::in_progress("d1", "a.zip", 1, Some(10)),
            DownloadItemView::completed("d2", "b.pdf"),
        ]);
        assert_eq!(panel.build_popover(Rect::ZERO).placement, PopoverPlacement::Below);
        assert_eq!(panel.build_list().item_count, 2);
    }

    #[test]
    fn actions_map_to_correct_download() {
        let panel = DownloadPanel::new(vec![DownloadItemView::in_progress("d1", "a.zip", 1, Some(10))]);
        assert_eq!(panel.on_open(0), Some(BrowserAction::OpenDownload("d1".into())));
        assert_eq!(panel.on_cancel(0), Some(BrowserAction::CancelDownload("d1".into())));
        assert_eq!(panel.on_show(0), Some(BrowserAction::ShowDownload("d1".into())));
        assert!(panel.on_open(5).is_none(), "越界");
    }
}
