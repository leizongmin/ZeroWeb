//! 单行文本输入状态（光标、选区、编辑操作）。

/// 单行文本编辑器状态。
#[derive(Debug, Clone)]
pub struct TextInput {
    text: String,
    cursor: usize,
    anchor: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            anchor: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn has_selection(&self) -> bool {
        self.cursor != self.anchor
    }

    pub fn selection_char_range(&self) -> (usize, usize) {
        if self.cursor <= self.anchor {
            (self.cursor, self.anchor)
        } else {
            (self.anchor, self.cursor)
        }
    }

    pub fn selected_text(&self) -> &str {
        if !self.has_selection() {
            return "";
        }
        let (start, end) = self.selection_char_range();
        let byte_start = char_index_to_byte(&self.text, start);
        let byte_end = char_index_to_byte(&self.text, end);
        &self.text[byte_start..byte_end]
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
        let len = self.char_len();
        self.cursor = len;
        self.anchor = len;
    }

    pub fn clear(&mut self) {
        self.set_text(String::new());
    }

    pub fn select_all(&mut self) {
        self.anchor = 0;
        self.cursor = self.char_len();
    }

    pub fn set_cursor(&mut self, index: usize, extend_selection: bool) {
        let index = index.min(self.char_len());
        if extend_selection {
            self.cursor = index;
        } else {
            self.cursor = index;
            self.anchor = index;
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        if s.is_empty() {
            return;
        }
        self.delete_selection();
        let byte = char_index_to_byte(&self.text, self.cursor);
        self.text.insert_str(byte, s);
        self.cursor += s.chars().count();
        self.anchor = self.cursor;
    }

    pub fn delete_backward(&mut self) {
        if self.has_selection() {
            self.delete_selection();
            return;
        }
        if self.cursor == 0 {
            return;
        }
        let prev = self.cursor - 1;
        let start = char_index_to_byte(&self.text, prev);
        let end = char_index_to_byte(&self.text, self.cursor);
        self.text.replace_range(start..end, "");
        self.cursor = prev;
        self.anchor = self.cursor;
    }

    pub fn delete_forward(&mut self) {
        if self.has_selection() {
            self.delete_selection();
            return;
        }
        if self.cursor >= self.char_len() {
            return;
        }
        let start = char_index_to_byte(&self.text, self.cursor);
        let end = char_index_to_byte(&self.text, self.cursor + 1);
        self.text.replace_range(start..end, "");
        self.anchor = self.cursor;
    }

    pub fn delete_selection(&mut self) {
        if !self.has_selection() {
            return;
        }
        let (start, end) = self.selection_char_range();
        let byte_start = char_index_to_byte(&self.text, start);
        let byte_end = char_index_to_byte(&self.text, end);
        self.text.replace_range(byte_start..byte_end, "");
        self.cursor = start;
        self.anchor = start;
    }

    pub fn move_left(&mut self, extend: bool) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        if !extend {
            self.anchor = self.cursor;
        }
    }

    pub fn move_right(&mut self, extend: bool) {
        if self.cursor < self.char_len() {
            self.cursor += 1;
        }
        if !extend {
            self.anchor = self.cursor;
        }
    }

    pub fn move_home(&mut self, extend: bool) {
        self.cursor = 0;
        if !extend {
            self.anchor = 0;
        }
    }

    pub fn move_end(&mut self, extend: bool) {
        self.cursor = self.char_len();
        if !extend {
            self.anchor = self.cursor;
        }
    }

    pub fn select_word_at(&mut self, char_index: usize) {
        let len = self.char_len();
        if len == 0 {
            return;
        }
        let idx = char_index.min(len - 1);
        let mut start = idx;
        let mut end = idx + 1;
        let chars: Vec<char> = self.text.chars().collect();
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }
        while end < len && is_word_char(chars[end]) {
            end += 1;
        }
        self.anchor = start;
        self.cursor = end;
    }

    /// 将相对文本起点的 x 坐标映射为字符索引。
    pub fn x_to_cursor(&self, rel_x: f32, measure: impl Fn(&str) -> f32) -> usize {
        if rel_x <= 0.0 || self.text.is_empty() {
            return 0;
        }
        let mut prefix = String::new();
        for (i, ch) in self.text.chars().enumerate() {
            let next = format!("{prefix}{ch}");
            if measure(&next) > rel_x {
                let mid = measure(&prefix) + measure(&ch.to_string()) / 2.0;
                return if rel_x > mid { i + 1 } else { i };
            }
            prefix.push(ch);
        }
        self.char_len()
    }

    pub fn copy_selection(&self) -> bool {
        let selected = self.selected_text();
        if selected.is_empty() {
            return false;
        }
        crate::clipboard::write_text(selected)
    }

    pub fn cut_selection(&mut self) -> bool {
        if !self.copy_selection() {
            return false;
        }
        self.delete_selection();
        true
    }

    pub fn paste_from_clipboard(&mut self) -> bool {
        let Some(text) = crate::clipboard::read_text() else {
            return false;
        };
        let sanitized: String = text.chars().filter(|c| *c != '\n' && *c != '\r').collect();
        if sanitized.is_empty() {
            return false;
        }
        self.insert_str(&sanitized);
        true
    }

    fn char_len(&self) -> usize {
        self.text.chars().count()
    }
}

fn char_index_to_byte(s: &str, char_index: usize) -> usize {
    if char_index == 0 {
        return 0;
    }
    s.char_indices().nth(char_index).map(|(i, _)| i).unwrap_or(s.len())
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_select_all() {
        let mut input = TextInput::new();
        input.insert_str("hello");
        assert_eq!(input.text(), "hello");
        input.select_all();
        assert_eq!(input.selected_text(), "hello");
    }

    #[test]
    fn delete_selection() {
        let mut input = TextInput::new();
        input.insert_str("abcdef");
        input.set_cursor(2, false);
        input.set_cursor(4, true);
        input.delete_selection();
        assert_eq!(input.text(), "abef");
    }
}
