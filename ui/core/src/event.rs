//! UI 事件模型 — 由 `ui-runtime` 把平台原始事件（winit）转成的浏览器无关 `UiEvent`。
//!
//! 事件路由优先级见 spec §8.4.3：capture → popup/modal → focus → hit-test → bubble → app shortcut。

use crate::geometry::{Point, Vec2};
use serde::{Deserialize, Serialize};

/// 修饰键位掩码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Modifiers(pub u8);

impl Modifiers {
    pub const NONE: Modifiers = Modifiers(0);
    pub const SHIFT: Modifiers = Modifiers(1 << 0);
    pub const CONTROL: Modifiers = Modifiers(1 << 1);
    pub const ALT: Modifiers = Modifiers(1 << 2);
    pub const SUPER: Modifiers = Modifiers(1 << 3);

    pub fn contains(self, other: Modifiers) -> bool {
        (self.0 & other.0) == other.0
    }
}

impl std::ops::BitOr for Modifiers {
    type Output = Modifiers;
    fn bitor(self, rhs: Modifiers) -> Modifiers {
        Modifiers(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for Modifiers {
    fn bitor_assign(&mut self, rhs: Modifiers) {
        self.0 |= rhs.0;
    }
}

/// 鼠标/笔/触摸按键。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    Other(u16),
}

/// 指针动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PointerPhase {
    Pressed,
    Released,
    Moved,
    Cancelled,
    /// 指针离开控件区域（host 合成，非原始输入；通知控件清除 hover/pressed 等交互态）。
    Exited,
}

/// 键盘动作。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyAction {
    Pressed,
    Released,
    Repeat,
}

/// 物理按键标识（简化版；M1 只覆盖浏览器 chrome 与基础控件所需）。
///
/// 用稳定字符串承载，避免在此处引入大型 keycode crate；具体命名见后续 ui/runtime 转换层。
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyCode(pub compact_str::CompactString);

impl KeyCode {
    pub fn new(name: &str) -> KeyCode {
        KeyCode(compact_str::CompactString::new(name))
    }
}

/// 滚动相位（对应 wheel / touchpad 两指）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScrollPhase {
    /// 离散滚轮（每次一格）。
    Discrete,
    /// 触控板连续滚动（带 start/update/end）。
    Start,
    Update,
    End,
}

/// 焦点事件。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusEvent {
    /// 获得焦点。
    Gained,
    /// 失去焦点。
    Lost,
}

/// 统一 UI 事件。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiEvent {
    /// 指针（鼠标/笔/触摸统一）。
    Pointer {
        phase: PointerPhase,
        button: Option<PointerButton>,
        position: Point,
        modifiers: Modifiers,
        /// 指针 id（DC-15 多指手势）：鼠标恒为 0；触摸为各手指的稳定 id（来自平台 touch id）。
        /// 手势 arena 据此区分多指（如双指 Pinch）。单指针路径（Tap/Pan/Fling）忽略非零值。
        pointer_id: u32,
    },
    /// 键盘。
    Key {
        code: KeyCode,
        action: KeyAction,
        modifiers: Modifiers,
        /// 可打印字符（受 shift/IME 影响）。
        text: Option<String>,
    },
    /// 滚轮/触控板滚动。
    Scroll {
        delta: Vec2,
        phase: ScrollPhase,
        position: Point,
        modifiers: Modifiers,
    },
    /// 焦点路由事件（Tab/Shift-Tab 或程序聚焦）。
    Focus(FocusEvent),
    /// 输入法合成事件；详细 IME 协议在 `ui/runtime::ime`。
    Ime(ImeEvent),
}

/// IME 事件（spec IF-001/IF-006；光标/选区/合成文本）。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImeEvent {
    /// 开始预处理（启用 IME，设置 composition 区域）。
    Enabled,
    /// 结束预处理。
    Disabled,
    /// 合成文本更新（未提交）。
    Preedit { text: String, cursor: Option<usize> },
    /// 提交文本（合成完成）。
    Commit(String),
}

impl UiEvent {
    /// 该事件是否携带位置（用于 hit-test）。
    pub fn position(&self) -> Option<Point> {
        match self {
            UiEvent::Pointer { position, .. } => Some(*position),
            UiEvent::Scroll { position, .. } => Some(*position),
            _ => None,
        }
    }

    /// 修饰键快照。
    pub fn modifiers(&self) -> Modifiers {
        match self {
            UiEvent::Pointer { modifiers, .. } => *modifiers,
            UiEvent::Key { modifiers, .. } => *modifiers,
            UiEvent::Scroll { modifiers, .. } => *modifiers,
            _ => Modifiers::NONE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modifiers_bitops() {
        let mut m = Modifiers::SHIFT | Modifiers::CONTROL;
        m |= Modifiers::ALT;
        assert!(m.contains(Modifiers::SHIFT));
        assert!(m.contains(Modifiers::CONTROL));
        assert!(m.contains(Modifiers::ALT));
        assert!(!m.contains(Modifiers::SUPER));
    }

    #[test]
    fn event_position_and_modifiers() {
        let pe = UiEvent::Pointer {
            phase: PointerPhase::Pressed,
            button: Some(PointerButton::Primary),
            position: Point::new(10.0, 20.0),
            modifiers: Modifiers::SHIFT,
            pointer_id: 0,
        };
        assert_eq!(pe.position(), Some(Point::new(10.0, 20.0)));
        assert!(pe.modifiers().contains(Modifiers::SHIFT));

        let ke = UiEvent::Key {
            code: KeyCode::new("Tab"),
            action: KeyAction::Pressed,
            modifiers: Modifiers::NONE,
            text: None,
        };
        assert_eq!(ke.position(), None);
        assert_eq!(ke.modifiers(), Modifiers::NONE);
    }
}
