//! 页面级 retained 表单控件编辑状态。

use std::collections::HashMap;

/// 单个文本表单控件的 retained 编辑状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormControlState {
    /// 当前值。
    pub value: String,
    /// 选区起点（UTF-16 code unit offset，与 DOM selectionStart 一致）。
    pub selection_start: usize,
    /// 选区终点（UTF-16 code unit offset，与 DOM selectionEnd 一致）。
    pub selection_end: usize,
    /// 当前 IME preedit 文本；M2 接入平台 composition 生命周期。
    pub composition_text: Option<String>,
    /// 当前 IME composition 区间。
    pub composition_range: Option<(usize, usize)>,
    /// 当前值是否已偏离本次获焦时的值。
    pub dirty_value: bool,
    /// 控件当前是否拥有页面焦点。
    pub focused: bool,
    /// 每次可观察编辑递增的版本号。
    pub revision: u64,
    value_at_focus: String,
}

impl FormControlState {
    fn new(value: String, selection_start: usize, selection_end: usize) -> Self {
        let (selection_start, selection_end) = normalized_selection(selection_start, selection_end);
        Self {
            value_at_focus: value.clone(),
            value,
            selection_start,
            selection_end,
            composition_text: None,
            composition_range: None,
            dirty_value: false,
            focused: true,
            revision: 0,
        }
    }

    fn update(&mut self, value: String, selection_start: usize, selection_end: usize) {
        let (selection_start, selection_end) = normalized_selection(selection_start, selection_end);
        let changed =
            self.value != value || self.selection_start != selection_start || self.selection_end != selection_end;
        self.value = value;
        self.selection_start = selection_start;
        self.selection_end = selection_end;
        self.dirty_value = self.value != self.value_at_focus;
        if changed {
            self.revision = self.revision.saturating_add(1);
        }
    }
}

/// 失焦结果，供宿主决定是否派发 `change`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlurredFormControl {
    /// 稳定控件选择器。
    pub selector: String,
    /// 本次焦点会话内当前值是否变化。
    pub value_changed: bool,
}

/// 页面内所有文本表单控件的 retained 状态仓库。
#[derive(Debug, Default)]
pub struct FormControlStateStore {
    controls: HashMap<String, FormControlState>,
    focused: Option<String>,
}

impl FormControlStateStore {
    /// 创建空状态仓库。
    pub fn new() -> Self {
        Self::default()
    }

    /// 把指定控件设为焦点控件并建立新的焦点会话基线。
    pub fn focus(&mut self, selector: &str, value: String, selection_start: usize, selection_end: usize) {
        if let Some(previous) = self.focused.take()
            && let Some(state) = self.controls.get_mut(&previous)
        {
            state.focused = false;
        }
        if let Some(state) = self.controls.get_mut(selector) {
            let revision = state.revision;
            *state = FormControlState::new(value, selection_start, selection_end);
            state.revision = revision;
        } else {
            self.controls.insert(
                selector.to_string(),
                FormControlState::new(value, selection_start, selection_end),
            );
        }
        self.focused = Some(selector.to_string());
    }

    /// 用 JS/DOM 编辑后的最终快照原地更新控件状态。
    pub fn update(&mut self, selector: &str, value: String, selection_start: usize, selection_end: usize) {
        let state = self.controls.entry(selector.to_string()).or_insert_with(|| {
            let mut state = FormControlState::new(String::new(), selection_start, selection_end);
            state.focused = self.focused.as_deref() == Some(selector);
            state
        });
        state.update(value, selection_start, selection_end);
    }

    /// 结束当前焦点会话，返回 change 判断所需结果。
    pub fn blur_focused(&mut self) -> Option<BlurredFormControl> {
        let selector = self.focused.take()?;
        let state = self.controls.get_mut(&selector)?;
        state.focused = false;
        Some(BlurredFormControl {
            selector,
            value_changed: state.dirty_value,
        })
    }

    /// 返回当前焦点控件的稳定选择器。
    pub fn focused_selector(&self) -> Option<&str> {
        self.focused.as_deref()
    }

    /// 返回指定控件状态。
    pub fn get(&self, selector: &str) -> Option<&FormControlState> {
        self.controls.get(selector)
    }

    /// 清除整页状态；导航或替换文档时调用。
    pub fn clear(&mut self) {
        self.controls.clear();
        self.focused = None;
    }
}

fn normalized_selection(start: usize, end: usize) -> (usize, usize) {
    (start.min(end), end.max(start))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_state_tracks_value_selection_and_revision() {
        let mut store = FormControlStateStore::new();
        store.focus("#name", "ab".to_string(), 2, 2);
        store.update("#name", "a中b".to_string(), 2, 2);

        let state = store.get("#name").expect("state");
        assert_eq!(state.value, "a中b");
        assert_eq!((state.selection_start, state.selection_end), (2, 2));
        assert!(state.dirty_value);
        assert!(state.focused);
        assert_eq!(state.revision, 1);
    }

    #[test]
    fn blur_reports_change_and_preserves_last_state() {
        let mut store = FormControlStateStore::new();
        store.focus("#first", "a".to_string(), 1, 1);
        store.update("#first", "ab".to_string(), 2, 2);

        let blurred = store.blur_focused().expect("focused control");
        assert_eq!(blurred.selector, "#first");
        assert!(blurred.value_changed);
        assert!(!store.get("#first").expect("state").focused);
        assert_eq!(store.focused_selector(), None);
    }

    #[test]
    fn switching_focus_keeps_controls_independent() {
        let mut store = FormControlStateStore::new();
        store.focus("#first", "one".to_string(), 3, 3);
        store.update("#first", "one!".to_string(), 4, 4);
        store.focus("#second", "two".to_string(), 0, 3);

        assert_eq!(store.focused_selector(), Some("#second"));
        assert!(!store.get("#first").expect("first").focused);
        assert!(store.get("#first").expect("first").dirty_value);
        assert!(store.get("#second").expect("second").focused);
    }

    #[test]
    fn refocus_preserves_control_revision() {
        let mut store = FormControlStateStore::new();
        store.focus("#name", "a".to_string(), 1, 1);
        store.update("#name", "ab".to_string(), 2, 2);
        let _ = store.blur_focused();
        store.focus("#name", "ab".to_string(), 2, 2);

        let state = store.get("#name").expect("state");
        assert_eq!(state.revision, 1);
        assert!(!state.dirty_value);
    }
}
