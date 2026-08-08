//! 事件类型 — 窗口事件和输入事件

/// 鼠标按钮
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// 左键
    Left,
    /// 右键
    Right,
    /// 中键
    Middle,
    /// 后退键
    Back,
    /// 前进键
    Forward,
    /// 其他按钮
    Other(u16),
}

/// 鼠标滚轮滚动增量
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MouseScrollDelta {
    /// 像素增量
    PixelDelta(f64, f64),
    /// 行增量
    LineDelta(f32, f32),
}

/// IME 输入事件
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImeEvent {
    /// IME 已启用
    Enabled,
    /// 预编辑文本（正在输入的文本，以及光标范围）
    Preedit {
        /// 预编辑文本
        text: String,
        /// 光标范围 (start, end)，None 表示无光标
        cursor: Option<(usize, usize)>,
    },
    /// 提交文本（用户确认输入）
    Commit(String),
    /// IME 已禁用
    Disabled,
}

/// 触摸事件阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    /// 手指按下
    Started,
    /// 手指移动
    Moved,
    /// 手指抬起
    Ended,
    /// 触摸取消
    Cancelled,
}

/// 触摸事件
#[derive(Debug, Clone)]
pub struct TouchEvent {
    /// 触摸点 ID
    pub id: u64,
    /// 触摸阶段
    pub phase: TouchPhase,
    /// 触摸位置（相对于窗口左上角，物理像素）
    pub x: f64,
    /// 触摸位置 Y
    pub y: f64,
}

/// 应用生命周期事件
#[derive(Debug)]
pub enum AppEvent {
    /// 宿主请求应用轮询后台状态，不触发窗口重绘。
    Poll,
    /// 窗口需要重绘
    RedrawRequested,
    /// 窗口大小变更
    Resized {
        /// 新宽度
        width: u32,
        /// 新高度
        height: u32,
    },
    /// 窗口缩放因子变更（例如移动到 Retina/HiDPI 屏幕）
    ScaleFactorChanged {
        /// 新缩放因子
        scale_factor: f64,
    },
    /// 窗口关闭请求
    CloseRequested,
    /// 窗口获得焦点
    Focused,
    /// 窗口失去焦点
    Unfocused,
    /// 键盘输入
    KeyboardInput {
        /// 按键名称（逻辑键名）
        key: String,
        /// 本次按键产生的文本（远程桌面/软键盘等场景下 logical_key 可能为 Unidentified）
        text: Option<String>,
        /// 是否按下（true = 按下, false = 释放）
        pressed: bool,
    },
    /// 鼠标移动
    MouseMoved {
        /// X 坐标（物理像素，相对于窗口左上角）
        x: f64,
        /// Y 坐标（物理像素，相对于窗口左上角）
        y: f64,
    },
    /// 鼠标按钮按下/释放
    MouseInput {
        /// 鼠标按钮
        button: MouseButton,
        /// 是否按下（true = 按下, false = 释放）
        pressed: bool,
        /// 最近一次指针位置 X（物理像素；触摸按下时常无前置 CursorMoved）
        x: f64,
        /// 最近一次指针位置 Y（物理像素）
        y: f64,
    },
    /// 鼠标滚轮滚动
    MouseWheel {
        /// 滚动增量
        delta: MouseScrollDelta,
        /// 指针位置 X（物理像素）
        x: f64,
        /// 指针位置 Y（物理像素）
        y: f64,
    },
    /// 触摸板/触摸屏平移手势（winit `PanGesture`）
    PanGesture {
        /// 水平平移增量（物理像素）
        delta_x: f32,
        /// 垂直平移增量（物理像素）
        delta_y: f32,
        /// 手势时指针位置 X（物理像素）
        x: f64,
        /// 手势时指针位置 Y（物理像素）
        y: f64,
    },
    /// 触摸事件
    Touch(TouchEvent),
    /// IME 输入事件（用于中文、日文、韩文等输入法）
    Ime(ImeEvent),
    /// 操作系统配色主题变更
    ThemeChanged {
        /// 是否为深色主题
        dark: bool,
    },
}

/// 将 winit 的 ElementState 转换为布尔值（pressed）
pub(crate) fn element_state_to_pressed(state: winit::event::ElementState) -> bool {
    match state {
        winit::event::ElementState::Pressed => true,
        winit::event::ElementState::Released => false,
    }
}

/// 将 winit 的 MouseButton 转换为自定义 MouseButton
pub(crate) fn convert_mouse_button(btn: winit::event::MouseButton) -> MouseButton {
    match btn {
        winit::event::MouseButton::Left => MouseButton::Left,
        winit::event::MouseButton::Right => MouseButton::Right,
        winit::event::MouseButton::Middle => MouseButton::Middle,
        winit::event::MouseButton::Back => MouseButton::Back,
        winit::event::MouseButton::Forward => MouseButton::Forward,
        winit::event::MouseButton::Other(n) => MouseButton::Other(n),
    }
}

/// 将 winit 的 MouseScrollDelta 转换为自定义 MouseScrollDelta
pub(crate) fn convert_scroll_delta(delta: winit::event::MouseScrollDelta) -> MouseScrollDelta {
    match delta {
        winit::event::MouseScrollDelta::PixelDelta(pos) => MouseScrollDelta::PixelDelta(pos.x, pos.y),
        winit::event::MouseScrollDelta::LineDelta(x, y) => MouseScrollDelta::LineDelta(x, y),
    }
}

/// 将 winit 的 Ime 转换为自定义 ImeEvent
pub(crate) fn convert_ime(ime: winit::event::Ime) -> ImeEvent {
    match ime {
        winit::event::Ime::Enabled => ImeEvent::Enabled,
        winit::event::Ime::Preedit(text, cursor) => ImeEvent::Preedit { text, cursor },
        winit::event::Ime::Commit(text) => ImeEvent::Commit(text),
        winit::event::Ime::Disabled => ImeEvent::Disabled,
    }
}

/// 将 winit 的 TouchPhase 转换为自定义 TouchPhase
pub(crate) fn convert_touch_phase(phase: winit::event::TouchPhase) -> TouchPhase {
    match phase {
        winit::event::TouchPhase::Started => TouchPhase::Started,
        winit::event::TouchPhase::Moved => TouchPhase::Moved,
        winit::event::TouchPhase::Ended => TouchPhase::Ended,
        winit::event::TouchPhase::Cancelled => TouchPhase::Cancelled,
    }
}

/// 从 winit KeyboardInput WindowEvent 中提取 AppEvent::KeyboardInput
pub(crate) fn convert_keyboard_input(
    device_id: winit::event::DeviceId,
    event: winit::event::KeyEvent,
    is_synthetic: bool,
) -> AppEvent {
    let _ = (device_id, is_synthetic);
    let key_text = match &event.logical_key {
        winit::keyboard::Key::Named(named) => format!("{:?}", named),
        winit::keyboard::Key::Character(ch) => ch.to_string(),
        winit::keyboard::Key::Unidentified(_) => String::from("Unidentified"),
        winit::keyboard::Key::Dead(dead) => {
            if let Some(ch) = dead {
                format!("Dead({})", ch)
            } else {
                String::from("Dead")
            }
        }
    };
    let pressed = element_state_to_pressed(event.state);
    let text = event.text.as_ref().filter(|t| !t.is_empty()).map(|t| t.to_string());
    AppEvent::KeyboardInput {
        key: key_text,
        text,
        pressed,
    }
}

#[cfg(test)]
#[path = "event_tests/mod.rs"]
mod tests;
