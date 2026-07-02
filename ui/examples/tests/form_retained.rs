//! Form retained 运行时闭环集成测试（DC-14 + DC-8 + DC-2）。
//!
//! 经 [`WinitDriver`]（DC-2 run-loop 核心）驱动，证明：① SDK 多控件组合（Label +
//! TextField + Button）；② Tab 焦点遍历（TextField↔Button）；③ 受控文本输入（键盘 →
//! form.change → reducer → 重建回灌）；④ Enter 提交 + 校验；⑤ Submit 按钮也可点击触发；
//! ⑥ focused TextField 的 ime_rect（DC-8 phase-2）。driver 内部 dispatch→reducer→重建→
//! invalidation→帧，无需手写 set_root/layout/paint。

use zero_ui_adapter_winit::WinitDriver;
use zero_ui_core::event::{KeyAction, KeyCode, Modifiers, PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::Point;
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::widget::WidgetId;
use zero_ui_examples::{FormApp, register_form_factories};
use zero_ui_render::Scene;
use zero_ui_render::render_node::RenderPrimitive;

fn key(code: &str, text: Option<&str>) -> UiEvent {
    UiEvent::Key {
        code: KeyCode::new(code),
        action: KeyAction::Pressed,
        modifiers: Modifiers::NONE,
        text: text.map(String::from),
    }
}

fn pointer(phase: PointerPhase, position: Point) -> UiEvent {
    UiEvent::Pointer {
        phase,
        button: Some(PointerButton::Primary),
        position,
        modifiers: Modifiers::NONE,
        pointer_id: 0,
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
    // 经 WinitDriver 驱动：键盘输入 + Tab 焦点 + Enter 提交 + 校验全闭环。
    let mut app = FormApp::new();
    // driver 持 &mut app → app.name/message 须在 driver 作用域外读（下方最终断言）；
    // 中间状态经 scene 文本断言。
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_form_factories(driver.host_mut());
        driver.begin();

        // Tab → 聚焦 TextField "name"（首个 focusable）。
        driver.pump_event(&key("Tab", None));
        assert_eq!(driver.host().focused_id(), Some(&WidgetId::new("name")));

        // 类型 "Hi"：每次 key → driver 内部 form.change→reducer→重建→pump_frame 重绘。
        for ch in ["H", "i"] {
            driver.pump_event(&key("Key", Some(ch)));
            driver.pump_frame();
        }
        let text = all_text(driver.host().scene());
        assert!(
            text.contains("Hi|"),
            "field should show typed value + caret, got: {text}"
        );

        // Tab → Submit button → Tab → name（wrap）。
        driver.pump_event(&key("Tab", None));
        assert_eq!(driver.host().focused_id(), Some(&WidgetId::new("submit")));
        driver.pump_event(&key("Tab", None));
        assert_eq!(
            driver.host().focused_id(),
            Some(&WidgetId::new("name")),
            "Tab wraps back to field"
        );

        // Enter（聚焦 name）→ form.submit → 校验通过 → "Hello, Hi!"。
        driver.pump_event(&key("Enter", None));
        driver.pump_frame();
        assert!(
            all_text(driver.host().scene()).contains("Hello, Hi!"),
            "submit should greet"
        );

        // Backspace×2 清空 → Enter → 校验失败 "Error"。
        for _ in 0..2 {
            driver.pump_event(&key("Backspace", None));
            driver.pump_frame();
        }
        driver.pump_event(&key("Enter", None));
        driver.pump_frame();
        assert!(
            all_text(driver.host().scene()).contains("Error"),
            "empty submit must fail validation"
        );
    }
    // driver 离开作用域 → 读最终 app 状态。
    assert_eq!(app.name, "", "name cleared by backspace");
    assert!(app.message.contains("Error"), "final message is validation error");
}

#[test]
fn submit_button_click_via_pointer() {
    // Submit 按钮经 driver 点击触发提交（form.submit 无 payload；app 用已持有 name 校验）。
    let mut app = FormApp::new();
    app.name = "Ada".into();
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_form_factories(driver.host_mut());
        driver.begin();
        // 点击 Submit 按钮：press + release（Button 在 release emit form.submit）。
        let center = {
            let rect = driver
                .host()
                .rect_of(&WidgetId::new("submit"))
                .expect("submit laid out");
            Point::new(rect.origin.x + 4.0, rect.origin.y + 4.0)
        };
        driver.pump_event(&pointer(PointerPhase::Pressed, center));
        driver.pump_event(&pointer(PointerPhase::Released, center));
        driver.pump_frame();
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
    // driver.begin 完成 set_root+layout+paint；后续经 host 访问器查询。
    let mut app = FormApp::new();
    app.name = "Hi".into(); // caret 在末尾：local caret_x = 6 + 2*8 = 22
    let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
    register_form_factories(driver.host_mut());
    driver.begin();

    // 无焦点 → 无 ime_rect。
    assert!(driver.host().ime_rect().is_none(), "no focus → no ime rect");

    // 聚焦 name 字段 → ime_rect 在字段绝对 origin + caret 局部。
    driver.host_mut().set_focus(WidgetId::new("name"));
    let ime = driver.host().ime_rect().expect("focused text field has ime_rect");
    let field = driver.host().rect_of(&WidgetId::new("name")).expect("field laid out");
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
