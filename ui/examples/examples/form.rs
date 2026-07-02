//! Form 示例无窗口驱动（`cargo run -p zero-ui-examples --example form`）。
//!
//! 脚本化驱动 form：Tab 聚焦 TextField → 输入字符 → Enter 提交 → 校验 → 打印 message。

use zero_ui_core::event::{KeyAction, KeyCode, Modifiers, UiEvent};
use zero_ui_core::geometry::{Constraints, Size};
use zero_ui_examples::{FormApp, register_form_factories};
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

fn scene_texts(scene: &zero_ui_render::Scene) -> Vec<String> {
    scene
        .entries
        .iter()
        .filter_map(|e| match &e.primitive {
            RenderPrimitive::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

fn main() {
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
    println!("initial: {:?}", scene_texts(host.scene()));

    // Tab 聚焦字段，输入 "Ada"。
    host.dispatch_event(&key("Tab", None));
    for ch in ["A", "d", "a"] {
        for a in host.dispatch_event(&key("Key", Some(ch))) {
            app.reduce(&a);
        }
        render(&mut app, &mut host);
    }
    println!("after typing 'Ada': {:?}", scene_texts(host.scene()));

    // Enter 提交。
    for a in host.dispatch_event(&key("Enter", None)) {
        app.reduce(&a);
    }
    render(&mut app, &mut host);
    println!("after submit: {:?}", scene_texts(host.scene()));

    // 清空再提交 → 校验失败。
    for _ in 0..3 {
        for a in host.dispatch_event(&key("Backspace", None)) {
            app.reduce(&a);
        }
        render(&mut app, &mut host);
    }
    for a in host.dispatch_event(&key("Enter", None)) {
        app.reduce(&a);
    }
    render(&mut app, &mut host);
    println!("after empty submit: {:?}", scene_texts(host.scene()));
}
