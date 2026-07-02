//! Form retained 运行时闭环集成测试（DC-14 + DC-8）。
//!
//! 证明：① SDK 多控件组合（Label + TextField + Button）；② Tab 焦点遍历（TextField↔Button）；
//! ③ 受控文本输入（键盘 → form.change → reducer → 重建回灌）；④ Enter 提交 + 校验。

use zero_ui_core::event::{KeyAction, KeyCode, Modifiers, UiEvent};
use zero_ui_core::geometry::{Constraints, Size};
use zero_ui_core::widget::WidgetId;
use zero_ui_examples::{FormApp, register_form_factories};
use zero_ui_render::Scene;
use zero_ui_render::render_node::RenderPrimitive;
use zero_ui_runtime::WidgetHost;

fn key(code: &str, text: Option<&str>) -> UiEvent {
    UiEvent::Key {
        code: KeyCode::new(code),
        action: KeyAction::Pressed,
        modifiers: Modifiers::NONE,
        text: text.map(String::from),
    }
}

fn all_text(scene: &Scene) -> String {
    scene
        .entries
        .iter()
        .filter_map(|e| match &e.primitive {
            RenderPrimitive::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

#[test]
fn form_typing_focus_submit_and_validation() {
    let mut app = FormApp::new();
    let mut host = WidgetHost::new();
    register_form_factories(&mut host);
    let vp = Constraints::loose(Size::new(400.0, 300.0));

    let render = |app: &mut FormApp, host: &mut WidgetHost| {
        host.set_root(&app.build_spec());
        host.layout(vp);
        host.paint();
    };
    render(&mut app, &mut host);

    // Tab → 聚焦 TextField "name"（首个 focusable）。
    host.dispatch_event(&key("Tab", None));
    assert_eq!(host.focused_id(), Some(&WidgetId::new("name")));

    // 类型 "Hi"：每次 key → form.change → reducer → 重建（受控回灌）。
    for ch in ["H", "i"] {
        for a in host.dispatch_event(&key("Key", Some(ch))) {
            app.reduce(&a);
        }
        render(&mut app, &mut host);
    }
    assert_eq!(app.name, "Hi");
    let text = all_text(host.scene());
    assert!(
        text.contains("Hi|"),
        "field should show typed value + caret, got: {text}"
    );

    // Tab → Submit button → Tab → name（wrap）。
    host.dispatch_event(&key("Tab", None));
    assert_eq!(host.focused_id(), Some(&WidgetId::new("submit")));
    host.dispatch_event(&key("Tab", None));
    assert_eq!(
        host.focused_id(),
        Some(&WidgetId::new("name")),
        "Tab wraps back to field"
    );

    // Enter（聚焦 name）→ form.submit → 校验通过 → "Hello, Hi!"。
    for a in host.dispatch_event(&key("Enter", None)) {
        app.reduce(&a);
    }
    render(&mut app, &mut host);
    assert!(
        all_text(host.scene()).contains("Hello, Hi!"),
        "submit should greet, got: {}",
        all_text(host.scene())
    );

    // Backspace×2 清空 → Enter → 校验失败 "Error"。
    for _ in 0..2 {
        for a in host.dispatch_event(&key("Backspace", None)) {
            app.reduce(&a);
        }
        render(&mut app, &mut host);
    }
    assert_eq!(app.name, "");
    for a in host.dispatch_event(&key("Enter", None)) {
        app.reduce(&a);
    }
    render(&mut app, &mut host);
    assert!(
        all_text(host.scene()).contains("Error"),
        "empty submit must fail validation, got: {}",
        all_text(host.scene())
    );
}

#[test]
fn submit_button_click_via_pointer() {
    // Submit 按钮也可点击触发提交（form.submit 无 payload；app 用已持有 name 校验）。
    use zero_ui_core::event::{PointerButton, PointerPhase};
    use zero_ui_core::geometry::Point;
    let mut app = FormApp::new();
    app.name = "Ada".into();
    let mut host = WidgetHost::new();
    register_form_factories(&mut host);
    let vp = Constraints::loose(Size::new(400.0, 300.0));
    host.set_root(&app.build_spec());
    host.layout(vp);
    host.paint();

    // 点击 Submit 按钮：press + release。
    let rect = host.rect_of(&WidgetId::new("submit")).expect("submit laid out");
    let c = Point::new(rect.origin.x + 4.0, rect.origin.y + 4.0);
    for phase in [PointerPhase::Pressed, PointerPhase::Released] {
        for a in host.dispatch_event(&zero_ui_core::event::UiEvent::Pointer {
            phase,
            button: Some(PointerButton::Primary),
            position: c,
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        }) {
            app.reduce(&a);
        }
    }
    assert!(
        app.message.contains("Hello, Ada!"),
        "click submit greets, got: {}",
        app.message
    );
}

#[test]
fn ime_rect_tracks_focused_text_field_caret() {
    // DC-8 phase-2：focused TextField 的 ime_rect = 节点绝对 origin + caret 局部位。
    let mut app = FormApp::new();
    app.name = "Hi".into(); // caret 在末尾：local caret_x = 6 + 2*8 = 22
    let mut host = WidgetHost::new();
    register_form_factories(&mut host);
    let vp = Constraints::loose(Size::new(400.0, 300.0));
    host.set_root(&app.build_spec());
    host.layout(vp);
    host.paint();

    // 无焦点 → 无 ime_rect。
    assert!(host.ime_rect().is_none(), "no focus → no ime rect");

    // 聚焦 name 字段 → ime_rect 在字段绝对 origin + caret 局部。
    host.set_focus(WidgetId::new("name"));
    let ime = host.ime_rect().expect("focused text field has ime_rect");
    let field = host.rect_of(&WidgetId::new("name")).expect("field laid out");
    assert!(
        (ime.origin.x - (field.origin.x + 22.0)).abs() < 0.5,
        "ime x = field origin + caret_x(22), got {} (field origin {})",
        ime.origin.x,
        field.origin.x
    );
    assert!(
        (ime.origin.y - (field.origin.y + 6.0)).abs() < 0.5,
        "ime y = field origin + 6, got {}",
        ime.origin.y
    );
    assert!(ime.size.height > 0.0 && ime.size.height <= field.size.height);
}
