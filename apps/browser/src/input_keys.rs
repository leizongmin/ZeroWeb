//! 键盘逻辑键名匹配（winit Named 键 Debug 为 ArrowLeft 等）。

/// 判断 `key` 是否对应给定的命名键（兼容 Arrow* 与短名）。
pub fn key_matches(key: &str, name: &str) -> bool {
    if key == name {
        return true;
    }
    matches!(
        (key, name),
        ("ArrowLeft", "Left") | ("ArrowRight", "Right") | ("ArrowUp", "Up") | ("ArrowDown", "Down")
    )
}

#[cfg(test)]
mod tests {
    use super::key_matches;

    #[test]
    fn arrow_aliases_match_short_names() {
        assert!(key_matches("ArrowLeft", "Left"));
        assert!(key_matches("ArrowRight", "Right"));
        assert!(key_matches("ArrowUp", "Up"));
        assert!(key_matches("ArrowDown", "Down"));
        assert!(!key_matches("ArrowLeft", "Right"));
    }

    #[test]
    fn exact_names_match() {
        assert!(key_matches("Home", "Home"));
        assert!(key_matches("Enter", "Enter"));
    }
}
