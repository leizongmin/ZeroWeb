//! 局部失效刷新（spec FR-012 / DC-9）。
//!
//! 区分四类失效，避免不必要的全量 layout/paint：
//! - `needs_layout`：尺寸/位置变化（最贵）。
//! - `needs_paint`：仅外观变化（主题色、hover、pressed），几何不变。
//! - `needs_semantics`：a11y 树变化。
//! - `needs_composite`：仅合成层变化（transform/opacity 动画）。
//!
//! 关键不变量：paint-only 变化（如主题色切换且字体/间距不变）只标记 `needs_paint`，
//! 不标记 `needs_layout`（spec FR-004 验收场景 / DC-9）。

use serde::{Deserialize, Serialize};

/// 失效标志位。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct InvalidationFlags(pub u8);

impl InvalidationFlags {
    pub const CLEAN: InvalidationFlags = InvalidationFlags(0);
    pub const NEEDS_LAYOUT: InvalidationFlags = InvalidationFlags(1 << 0);
    pub const NEEDS_PAINT: InvalidationFlags = InvalidationFlags(1 << 1);
    pub const NEEDS_SEMANTICS: InvalidationFlags = InvalidationFlags(1 << 2);
    pub const NEEDS_COMPOSITE: InvalidationFlags = InvalidationFlags(1 << 3);

    pub fn contains(self, flag: InvalidationFlags) -> bool {
        (self.0 & flag.0) == flag.0
    }

    /// 是否完全干净。
    pub fn is_clean(self) -> bool {
        self.0 == 0
    }

    /// 是否需要 layout（layout 隐含需要 re-paint）。
    pub fn requires_layout(self) -> bool {
        self.contains(Self::NEEDS_LAYOUT)
    }

    /// 是否需要 paint（含 layout 触发的连带 paint）。
    pub fn requires_paint(self) -> bool {
        self.contains(Self::NEEDS_LAYOUT) || self.contains(Self::NEEDS_PAINT)
    }

    /// 清除指定标记位（如 layout 完成后清除 `NEEDS_LAYOUT`，保留 `NEEDS_PAINT` 直到 paint 完成）。
    pub fn remove(&mut self, flag: InvalidationFlags) {
        self.0 &= !flag.0;
    }
}

impl std::ops::BitOr for InvalidationFlags {
    type Output = InvalidationFlags;
    fn bitor(self, rhs: InvalidationFlags) -> InvalidationFlags {
        InvalidationFlags(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for InvalidationFlags {
    fn bitor_assign(&mut self, rhs: InvalidationFlags) {
        self.0 |= rhs.0;
    }
}

impl std::ops::BitAnd for InvalidationFlags {
    type Output = InvalidationFlags;
    fn bitand(self, rhs: InvalidationFlags) -> InvalidationFlags {
        InvalidationFlags(self.0 & rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paint_only_change_does_not_request_layout() {
        // 主题色切换：字体/间距不变 → 仅 needs_paint。
        let flags = InvalidationFlags::NEEDS_PAINT;
        assert!(flags.requires_paint());
        assert!(!flags.requires_layout(), "paint-only change must not request layout");
    }

    #[test]
    fn layout_implies_paint() {
        // 文本变长导致测量变化 → needs_layout 连带 needs_paint。
        let flags = InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
        assert!(flags.requires_layout());
        assert!(flags.requires_paint());
    }

    #[test]
    fn composite_only_is_cheapest() {
        let flags = InvalidationFlags::NEEDS_COMPOSITE;
        assert!(flags.contains(InvalidationFlags::NEEDS_COMPOSITE));
        assert!(!flags.requires_layout());
        assert!(!flags.requires_paint());
        assert!(!flags.contains(InvalidationFlags::NEEDS_SEMANTICS));
    }

    #[test]
    fn clean_state() {
        assert!(InvalidationFlags::CLEAN.is_clean());
        assert!(!InvalidationFlags::NEEDS_PAINT.is_clean());
    }
}
