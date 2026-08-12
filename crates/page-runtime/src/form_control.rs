//! 页面级 retained 表单控件编辑状态。

use std::collections::HashMap;

/// 页面输入路由的单一焦点所有者与最新指针目标。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageInteractionState {
    focus_owner: Option<String>,
    pointer_target: String,
}

impl Default for PageInteractionState {
    fn default() -> Self {
        Self {
            focus_owner: None,
            pointer_target: "body".to_string(),
        }
    }
}

impl PageInteractionState {
    /// 创建空焦点、body 指针目标的页面状态。
    pub fn new() -> Self {
        Self::default()
    }

    /// 最新命中的指针目标。
    pub fn pointer_target(&self) -> &str {
        &self.pointer_target
    }

    /// 更新指针目标，不隐式改变键盘焦点。
    pub fn set_pointer_target(&mut self, selector: String) {
        self.pointer_target = selector;
    }

    /// 当前键盘/IME 焦点所有者。
    pub fn focus_owner(&self) -> Option<&str> {
        self.focus_owner.as_deref()
    }

    /// 更新唯一焦点所有者，返回此前所有者。
    pub fn set_focus_owner(&mut self, selector: Option<String>) -> Option<String> {
        std::mem::replace(&mut self.focus_owner, selector)
    }

    /// 导航时清空页面交互状态。
    pub fn clear(&mut self) {
        self.focus_owner = None;
        self.pointer_target.clear();
        self.pointer_target.push_str("body");
    }
}

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
            self.composition_text = None;
            self.composition_range = None;
        }
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
        if let Some(previous) = self.focused.take() {
            if let Some(state) = self.controls.get_mut(&previous) {
                state.focused = false;
            }
            // R3254-L10：隐式 blur（焦点直接切换，如页面 JS focus() 到另一控件）——
            // previous 状态一并移除（change 由调用方按需报告；重聚焦重建基线）。
            if previous != selector {
                self.controls.remove(&previous);
            }
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
    ///
    /// R3254-L10：失焦后移除控件状态（change 已报告、基线已无用途）——`controls` 不再
    /// 随页面生命周期内不断获焦的新控件无限增长。重聚焦时 `focus()` 重建基线（revision
    /// 从 0 开始；paint 失效判断只看 revision 变化，跨 blur 保留无必要）。
    pub fn blur_focused(&mut self) -> Option<BlurredFormControl> {
        let selector = self.focused.take()?;
        let state = self.controls.get_mut(&selector)?;
        state.focused = false;
        state.composition_text = None;
        state.composition_range = None;
        let value_changed = state.dirty_value;
        self.controls.remove(&selector);
        Some(BlurredFormControl {
            selector,
            value_changed,
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

    /// 更新焦点控件的 IME preedit；空文本表示取消合成。
    pub fn update_focused_composition(&mut self, text: String) -> bool {
        let Some(selector) = self.focused.as_deref() else {
            return false;
        };
        let Some(state) = self.controls.get_mut(selector) else {
            return false;
        };
        let next_text = (!text.is_empty()).then_some(text);
        let next_range = next_text.as_ref().map(|_| (state.selection_start, state.selection_end));
        let changed = state.composition_text != next_text || state.composition_range != next_range;
        state.composition_text = next_text;
        state.composition_range = next_range;
        if changed {
            state.revision = state.revision.saturating_add(1);
        }
        changed
    }

    /// 取消焦点控件尚未提交的 IME preedit。
    pub fn clear_focused_composition(&mut self) -> bool {
        self.update_focused_composition(String::new())
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
        // R3254-L10：失焦后状态移除（change 已报告，基线无用途）。
        assert!(store.get("#first").is_none());
        assert_eq!(store.focused_selector(), None);
    }

    #[test]
    fn switching_focus_keeps_controls_independent() {
        let mut store = FormControlStateStore::new();
        store.focus("#first", "one".to_string(), 3, 3);
        store.update("#first", "one!".to_string(), 4, 4);
        store.focus("#second", "two".to_string(), 0, 3);

        assert_eq!(store.focused_selector(), Some("#second"));
        // R3254-L10：焦点切换时前控件已 blur（change 报告后状态移除）。
        assert!(store.get("#first").is_none());
        assert!(store.get("#second").expect("second").focused);
    }

    #[test]
    fn refocus_rebuilds_baseline_after_blur() {
        let mut store = FormControlStateStore::new();
        store.focus("#name", "a".to_string(), 1, 1);
        store.update("#name", "ab".to_string(), 2, 2);
        let _ = store.blur_focused();
        // R3254-L10：blur 已移除状态——重聚焦重建基线（新基线 = 当前值，不 dirty）。
        store.focus("#name", "ab".to_string(), 2, 2);

        let state = store.get("#name").expect("state");
        assert!(!state.dirty_value);
    }

    #[test]
    fn composition_is_temporary_and_cleared_on_blur() {
        let mut store = FormControlStateStore::new();
        store.focus("#name", "base".to_string(), 2, 2);
        assert!(store.update_focused_composition("拼音".to_string()));
        let state = store.get("#name").expect("state");
        assert_eq!(state.value, "base");
        assert_eq!(state.composition_text.as_deref(), Some("拼音"));
        assert_eq!(state.composition_range, Some((2, 2)));

        let _ = store.blur_focused();
        // R3254-L10：blur 已移除状态（composition 随状态清除）。
        assert!(store.get("#name").is_none());
    }

    #[test]
    fn disabled_composition_does_not_commit_text() {
        let mut store = FormControlStateStore::new();
        store.focus("#name", "base".to_string(), 4, 4);
        assert!(store.update_focused_composition("未提交".to_string()));
        assert!(store.clear_focused_composition());

        let state = store.get("#name").expect("state");
        assert_eq!(state.value, "base");
        assert!(state.composition_text.is_none());
        assert!(!state.dirty_value);
    }

    #[test]
    fn pointer_target_does_not_steal_keyboard_focus() {
        let mut state = PageInteractionState::new();
        state.set_focus_owner(Some("#name".to_string()));
        state.set_pointer_target("#button".to_string());

        assert_eq!(state.focus_owner(), Some("#name"));
        assert_eq!(state.pointer_target(), "#button");
        assert_eq!(
            state.set_focus_owner(Some("#note".to_string())),
            Some("#name".to_string())
        );
    }
}
