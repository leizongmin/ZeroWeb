//! Counter 示例应用（DC-14）—— 验证通用 UI SDK 可被**外部程序**复用，跑通
//! retained 运行时闭环：事件 → Action → AppState reducer → 重建 WidgetSpec → re-layout/paint。
//!
//! 本 crate **不依赖任何浏览器 crate**（`zero-browser-shell` / `zero-webview` / `zero-engine` / `zero-net`），
//! 依赖仅为 `ui/core` + `ui/render` + `ui/runtime` + `ui/widgets`。
//!
//! 应用结构：`Column[ Label(count), Row[ Button("-", dec), Button("+", inc) ] ]`。
//! `Label` 是本示例自带的展示控件（演示如何用 SDK 自定义控件并 paint 文本）。

use zero_ui_core::action::{ActionId, ActionPayload, ActionResult, EventResult};
use zero_ui_core::binding::Value;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Size};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::theme::Color;
use zero_ui_core::widget::{EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, UpdateCtx, Widget, WidgetId, WidgetSpec};
use zero_ui_runtime::{EmittedAction, UiApp, WidgetHost};
use zero_ui_widgets::button::{Button, ButtonSpec};

// ---------------- 自定义展示控件：Label ----------------

/// 简单文本展示控件（演示用 SDK 自定义控件；paint 文本走 `PaintRecorder::draw_text`）。
pub struct Label {
    text: String,
}

impl Label {
    pub fn new(text: String) -> Label {
        Label { text }
    }

    pub fn text(&self) -> &str {
        &self.text
    }
}

impl Widget for Label {
    fn mount(&mut self, _ctx: &mut MountCtx) {}

    fn update(&mut self, ctx: &mut UpdateCtx, props: &Props) {
        if let Some(Value::Text(t)) = props.get("text")
            && t != &self.text
        {
            self.text = t.clone();
            // 仅文案变化、字体/字号不变 → 只需 paint，不触发布局。
            *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
        }
    }

    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }

    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 启发式估算（与 widgets::Button 同口径；精确度量留 foundation/text 接入）。
        let char_w = 8.0_f32;
        let line_h = 24.0_f32;
        let w = self.text.chars().count() as f32 * char_w;
        Size::new(
            w.clamp(constraints.min_width, constraints.max_width),
            line_h.clamp(constraints.min_height, constraints.max_height),
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx) {
        // 文本基线约 18px（line-height 24）。
        ctx.recorder
            .draw_text(&self.text, Point::new(0.0, 18.0), 16.0, Color::rgb(0.1, 0.1, 0.1));
    }
}

// ---------------- 应用状态 + reducer + 声明树 ----------------

/// Counter 应用状态（单向数据流：状态由应用持有，控件只发 action）。
pub struct CounterApp {
    count: i32,
}

impl CounterApp {
    pub fn new() -> CounterApp {
        CounterApp { count: 0 }
    }

    pub fn count(&self) -> i32 {
        self.count
    }

    /// reducer：消费 widget 发出的 action，更新业务状态。返回是否产生状态变化。
    pub fn reduce(&mut self, action: &EmittedAction) -> bool {
        match action.action.0.as_str() {
            "counter.inc" => {
                self.count += 1;
                true
            }
            "counter.dec" => {
                self.count -= 1;
                true
            }
            _ => false,
        }
    }

    /// 由当前状态产出声明树（可频繁重建；reconcile 按稳定 WidgetId 复用控件实例状态）。
    pub fn build_spec(&self) -> WidgetSpec {
        let mut col = WidgetSpec::new("Column");
        col.id = Some(WidgetId::new("root"));

        let mut label = WidgetSpec::new("Label");
        label.id = Some(WidgetId::new("count"));
        label
            .props
            .insert("text", Value::Text(format!("Count: {}", self.count)));
        col.children.push(label);

        let mut row = WidgetSpec::new("Row");
        row.id = Some(WidgetId::new("buttons"));
        row.props.insert("gap", Value::Float(8.0));

        let mut dec = WidgetSpec::new("Button");
        dec.id = Some(WidgetId::new("dec"));
        dec.props.insert("label", Value::Text("-".into()));
        dec.props.insert("action", Value::Text("counter.dec".into()));

        let mut inc = WidgetSpec::new("Button");
        inc.id = Some(WidgetId::new("inc"));
        inc.props.insert("label", Value::Text("+".into()));
        inc.props.insert("action", Value::Text("counter.inc".into()));

        row.children.push(dec);
        row.children.push(inc);
        col.children.push(row);
        col
    }
}

impl Default for CounterApp {
    fn default() -> CounterApp {
        CounterApp::new()
    }
}

/// `UiApp` 适配（spec IF-006）：把 `CounterApp` 接入通用运行时驱动器（如
/// `zero_ui_adapter_winit::WinitDriver`）——`root_spec` 产出声明树，`dispatch` 走 reducer。
/// 这样 counter 既可被 `WinitDriver` 驱动（事件→dispatch→重建→帧），也可被低层 host
/// 直接驱动（保留 `reduce`/`build_spec` 公共 API）。
impl UiApp for CounterApp {
    fn root_spec(&self) -> WidgetSpec {
        self.build_spec()
    }

    fn dispatch(&mut self, action: &ActionId, payload: Option<ActionPayload>) -> ActionResult {
        // 复用既有 reducer（单一状态变更源）；Handled/UnknownAction 由是否产生状态变化决定。
        let emitted = EmittedAction {
            action: action.clone(),
            payload,
        };
        if self.reduce(&emitted) {
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

/// 把 counter 示例用到的控件工厂注册到 host。
///
/// `Label` 用本 crate 自定义控件；`Button` 复用 `ui/widgets::Button`——
/// 演示 SDK 通用控件可被外部直接组装。
pub fn register_counter_factories(host: &mut WidgetHost) {
    host.register("Label", |spec| {
        let text = str_prop(spec, "text").unwrap_or_default();
        Box::new(Label::new(text))
    });
    host.register("Button", |spec| {
        let label = str_prop(spec, "label").unwrap_or_default();
        let action = ActionId::new(&str_prop(spec, "action").unwrap_or_else(|| "noop".into()));
        let enabled = !matches!(spec.props.get("enabled"), Some(Value::Bool(false)));
        Box::new(Button::new(ButtonSpec {
            label,
            action,
            enabled,
            hover_action: None,
            variant: zero_ui_widgets::ButtonVariant::Primary,
        }))
    });
}
