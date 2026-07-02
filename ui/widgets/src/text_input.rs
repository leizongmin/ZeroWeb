//! TextInput — 文本输入控件（spec FR-009 / DC-8 IME）。
//!
//! 控件内部保存临时 UI 状态（光标/选区）；业务文本（如 URL）由应用状态持有（spec FR-003）。
//! M1 提供 retained 编辑状态模型 + IME 光标 rect；真实 shaping/测量在 M2 接 foundation/text。

use zero_ui_core::geometry::Rect;

/// TextInput 的 retained 编辑状态。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextInputState {
    pub text: String,
    /// 光标字节偏移（= 选区锚点）。
    pub cursor: usize,
    /// 选区（起止字节偏移）；None = 无选区（collapsed caret）。
    pub selection: Option<(usize, usize)>,
}

impl TextInputState {
    pub fn empty() -> TextInputState {
        TextInputState {
            text: String::new(),
            cursor: 0,
            selection: None,
        }
    }

    fn clamp_cursor(&self, c: usize) -> usize {
        c.min(self.text.len())
    }

    /// 在光标处插入文本（替换当前选区）。
    pub fn insert(&mut self, s: &str) {
        if let Some((start, end)) = self.selection {
            let (lo, hi) = (start.min(end), start.max(end));
            self.text.replace_range(lo..hi, s);
            self.cursor = lo + s.len();
        } else {
            self.text.insert_str(self.cursor, s);
            self.cursor += s.len();
        }
        self.selection = None;
    }

    /// 向前删除（backspace）。
    pub fn backspace(&mut self) {
        if let Some((start, end)) = self.selection {
            let (lo, hi) = (start.min(end), start.max(end));
            self.text.replace_range(lo..hi, "");
            self.cursor = lo;
            self.selection = None;
            return;
        }
        if self.cursor == 0 {
            return;
        }
        // 退到前一个字符边界。
        let prev = self.text[..self.cursor]
            .char_indices()
            .last()
            .map(|(i, _)| i)
            .unwrap_or(0);
        self.text.replace_range(prev..self.cursor, "");
        self.cursor = prev;
    }

    /// 移动光标（dir = -1/1）。
    pub fn move_cursor(&mut self, dir: i32) {
        if dir < 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .last()
                .map(|(i, _)| i)
                .unwrap_or(0);
        } else if let Some((_, ch)) = self.text[self.cursor..].char_indices().next() {
            self.cursor += ch.len_utf8();
        }
        self.selection = None;
    }

    /// 由 caret 字节偏移推算的 IME 光标屏幕 rect（M1：等宽启发式；M2 用真实 shaped metrics）。
    pub fn ime_caret_rect(&self, origin: Rect) -> Rect {
        let char_w = 8.0_f32;
        let char_count = self.text[..self.clamp_cursor(self.cursor)].chars().count();
        Rect::from_ltrb(
            origin.left() + char_count as f32 * char_w,
            origin.top(),
            origin.left() + char_count as f32 * char_w + 2.0,
            origin.bottom(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_move_backspace() {
        let mut st = TextInputState::empty();
        st.insert("abc");
        assert_eq!(st.text, "abc");
        assert_eq!(st.cursor, 3);
        st.move_cursor(-1);
        st.insert("X");
        assert_eq!(st.text, "abXc");
        st.move_cursor(1);
        st.backspace();
        assert_eq!(st.text, "abX");
    }

    #[test]
    fn replace_selection() {
        let mut st = TextInputState::empty();
        st.insert("hello");
        st.selection = Some((1, 4)); // "ell"
        st.insert("EL");
        assert_eq!(st.text, "hELo");
        assert_eq!(st.cursor, 3);
        assert_eq!(st.selection, None);
    }

    #[test]
    fn ime_caret_rect_advances_with_cursor() {
        let mut st = TextInputState::empty();
        st.insert("abcd");
        st.cursor = 2;
        let r = st.ime_caret_rect(Rect::from_ltrb(0.0, 0.0, 100.0, 20.0));
        // 2 个字符 → caret x = 16。
        assert_eq!(r.left(), 16.0);
    }
}
