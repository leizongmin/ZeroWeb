//! Form 示例应用（DC-14 + DC-8）—— 验证 SDK 多控件组合 + 焦点遍历 + 键盘文本输入 + 校验。
//!
//! 演示 **受控文本输入**（controlled）：`FormApp` 持有字段值（单向数据流），`TextField`
//! 在聚焦时接收键盘 → 发出 `form.change`(新值) → reducer 更新 → 重建把新值经 props 回灌；
//! Enter / 提交按钮 → `form.submit` → 校验 → 结果 message。焦点经 Tab 在 TextField↔Button 间遍历。
//!
//! 本 crate **不依赖任何浏览器 crate**；`Label`/`Button` 复用 `counter` 示例的工厂。

use crate::counter::register_counter_factories;
use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::{KeyAction, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{
    EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, SemanticsCtx, UpdateCtx, Widget, WidgetId, WidgetSpec,
};
use zero_ui_runtime::{EmittedAction, UiApp, WidgetHost};

/// form 字段编辑 action（payload = 字段新值）。
pub const ACTION_CHANGE: &str = "form.change";
/// form 提交 action。
pub const ACTION_SUBMIT: &str = "form.submit";

// ---------------- 自定义控件：TextField（受控、focusable）----------------

/// 文本输入控件（演示 SDK 自定义 focusable 控件 + 键盘输入）。
///
/// **受控**：`self.text` 由 `props.text` 回灌（reducer 持有真值）。聚焦时键盘事件
/// 转 `form.change`(新值) / Enter 转 `form.submit`；本控件不自行持有编辑结果。
pub struct TextField {
    text: String,
    placeholder: String,
}

impl TextField {
    pub fn new(placeholder: &str) -> TextField {
        TextField {
            text: String::new(),
            placeholder: placeholder.to_string(),
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Widget for TextField {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        // 受控回灌：app 持有的字段值经 props.text 同步到本控件。
        let mut changed = false;
        if let Some(Value::Text(t)) = props.get("text")
            && *t != self.text
        {
            self.text = t.clone();
            changed = true;
        }
        if let Some(Value::Text(p)) = props.get("placeholder") {
            self.placeholder = p.clone();
        }
        if changed {
            *ctx.invalidation |= zero_ui_core::invalidation::InvalidationFlags::NEEDS_PAINT;
        }
    }

    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        // 仅处理 Key Pressed（host 已截获 Tab 用于焦点遍历）。
        let UiEvent::Key {
            code,
            action: KeyAction::Pressed,
            text,
            ..
        } = event
        else {
            return EventResult::Ignored;
        };
        match code.0.as_str() {
            "Enter" => EventResult::Emit(ActionId::new(ACTION_SUBMIT)),
            "Backspace" => {
                let mut t = self.text.clone();
                t.pop();
                EventResult::EmitWithPayload(ActionId::new(ACTION_CHANGE), ActionPayload::Text(t))
            }
            _ => match text {
                Some(ch) => {
                    let t = format!("{}{}", self.text, ch);
                    EventResult::EmitWithPayload(ActionId::new(ACTION_CHANGE), ActionPayload::Text(t))
                }
                None => EventResult::Ignored,
            },
        }
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 启发式宽度（与 widgets::Button 同口径）；最小 120。
        let char_w = 8.0_f32;
        let w = (self.text.chars().count().max(self.placeholder.chars().count()) as f32 * char_w).max(120.0);
        Size::new(
            w.clamp(constraints.min_width, constraints.max_width),
            32.0_f32.clamp(constraints.min_height, constraints.max_height),
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        let size = ctx.clip.map(|r| r.size).unwrap_or(Size::new(160.0, 32.0));
        // 背景框 + 文案（空值显示 placeholder）。
        ctx.recorder.fill_rect(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            Color::rgb(1.0, 1.0, 1.0),
        );
        ctx.recorder.stroke_rect(
            Rect::from_ltrb(0.0, 0.0, size.width, size.height),
            Color::rgb(0.6, 0.6, 0.6),
            1.0,
        );
        let display = if self.text.is_empty() {
            self.placeholder.clone()
        } else {
            // 光标 "_" 演示（真实 caret 位置由 DC-8 phase-2 IME rect 提供）。
            format!("{}|", self.text)
        };
        let color = if self.text.is_empty() {
            Color::rgb(0.6, 0.6, 0.6)
        } else {
            Color::rgb(0.1, 0.1, 0.1)
        };
        ctx.recorder.draw_text(&display, Point::new(6.0, 22.0), 14.0, color);
    }

    fn semantics(&self, _ctx: &mut SemanticsCtx) {}

    fn focusable(&self) -> bool {
        true
    }

    fn ime_rect(&self) -> Option<zero_ui_core::geometry::Rect> {
        // caret 位于已输入文本末尾（char_w=8，左 padding 6）；行高矩形供 IME 定位。
        let caret_x = 6.0 + self.text.chars().count() as f32 * 8.0;
        Some(zero_ui_core::geometry::Rect::from_ltrb(
            caret_x,
            6.0,
            caret_x + 2.0,
            30.0,
        ))
    }
}

// ---------------- 应用状态 + reducer + 声明树 ----------------

/// Form 应用状态（受控：持有字段值 + 校验结果 message）。
pub struct FormApp {
    pub name: String,
    pub message: String,
}

impl FormApp {
    pub fn new() -> FormApp {
        FormApp {
            name: String::new(),
            message: "Type your name, then press Enter".into(),
        }
    }

    pub fn reduce(&mut self, action: &EmittedAction) {
        match action.action.0.as_str() {
            ACTION_CHANGE => {
                if let Some(ActionPayload::Text(v)) = &action.payload {
                    self.name = v.clone();
                }
            }
            ACTION_SUBMIT => {
                let n = self.name.trim();
                self.message = if n.is_empty() {
                    "Error: name is required".into()
                } else {
                    format!("Hello, {}!", n)
                };
            }
            _ => {}
        }
    }

    /// 由当前状态产出声明树（受控：name 经 props.text 回灌 TextField；message 显示在 Label）。
    pub fn build_spec(&self) -> WidgetSpec {
        let mut col = WidgetSpec::new("Column");
        col.id = Some(WidgetId::new("root"));

        let mut message = WidgetSpec::new("Label");
        message.id = Some(WidgetId::new("message"));
        message.props.insert("text", Value::Text(self.message.clone()));
        col.children.push(message);

        let mut row = WidgetSpec::new("Row");
        row.id = Some(WidgetId::new("row"));
        row.props.insert("gap", Value::Float(8.0));

        let mut field = WidgetSpec::new("TextField");
        field.id = Some(WidgetId::new("name"));
        field.props.insert("text", Value::Text(self.name.clone()));
        field.props.insert("placeholder", Value::Text("Name".into()));
        row.children.push(field);

        let mut submit = WidgetSpec::new("Button");
        submit.id = Some(WidgetId::new("submit"));
        submit.props.insert("label", Value::Text("Submit".into()));
        submit.props.insert("action", Value::Text(ACTION_SUBMIT.into()));
        row.children.push(submit);

        col.children.push(row);
        col
    }
}

impl Default for FormApp {
    fn default() -> FormApp {
        FormApp::new()
    }
}

/// `UiApp` 适配（spec IF-006）：把 `FormApp` 接入通用运行时驱动器（如
/// `zero_ui_adapter_winit::WinitDriver`）——`root_spec` 产出声明树，`dispatch` 走 reducer。
/// 已知 action（`form.change`/`form.submit`）→ `Handled`（driver 据此重建 spec）；未知 →
/// `UnknownAction`（不重建）。保留既有 `reduce`/`build_spec` 公共 API（低层 host 路径仍可用）。
impl UiApp for FormApp {
    fn root_spec(&self) -> WidgetSpec {
        self.build_spec()
    }

    fn dispatch(&mut self, action: &ActionId, payload: Option<ActionPayload>) -> ActionResult {
        let known = matches!(action.0.as_str(), ACTION_CHANGE | ACTION_SUBMIT);
        let emitted = EmittedAction {
            action: action.clone(),
            payload,
        };
        self.reduce(&emitted);
        if known {
            ActionResult::Handled
        } else {
            ActionResult::UnknownAction(action.clone())
        }
    }
}

// ---------------- 工厂注册 ----------------

fn str_prop(spec: &WidgetSpec, key: &str) -> Option<String> {
    match spec.props.get(key) {
        Some(Value::Text(s)) => Some(s.clone()),
        _ => None,
    }
}

/// 注册 form 示例控件工厂：复用 counter 的 `Label`/`Button` + 本模块 `TextField`。
pub fn register_form_factories(host: &mut WidgetHost) {
    // Label（展示文案）+ Button（提交）—— 复用通用控件工厂。
    register_counter_factories(host);
    // TextField（受控文本输入，focusable）。
    host.register("TextField", |spec| {
        let placeholder = str_prop(spec, "placeholder").unwrap_or_else(|| "input".into());
        Box::new(TextField::new(&placeholder))
    });
}
