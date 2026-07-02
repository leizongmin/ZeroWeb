//! Form 示例无窗口驱动（`cargo run -p zero-ui-examples --example form`）。
//!
//! 经 [`WinitDriver`](zero_ui_adapter_winit::WinitDriver)（DC-2 run-loop 核心）驱动 form：
//! Tab 聚焦 TextField → 键盘输入字符 → Enter 提交 → 校验 → 打印 message。driver 内部完成
//! dispatch → reducer → 重建 → invalidation → 帧重绘，无需手写 set_root/layout/paint。

use zero_ui_adapter_winit::WinitDriver;
use zero_ui_core::event::{KeyAction, KeyCode, Modifiers, UiEvent};
use zero_ui_core::layout::WindowMetrics;
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

fn scene_texts(scene: &Scene) -> Vec<String> {
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
    let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
    register_form_factories(driver.host_mut());
    driver.begin();
    println!("initial: {:?}", scene_texts(driver.host().scene()));

    // Tab 聚焦字段，输入 "Ada"：每次 key → driver 内部 form.change→reducer→重建→pump_frame 重绘。
    driver.pump_event(&key("Tab", None));
    for ch in ["A", "d", "a"] {
        driver.pump_event(&key("Key", Some(ch)));
        driver.pump_frame();
    }
    println!("after typing 'Ada': {:?}", scene_texts(driver.host().scene()));

    // Enter 提交 → 校验通过。
    driver.pump_event(&key("Enter", None));
    driver.pump_frame();
    println!("after submit: {:?}", scene_texts(driver.host().scene()));

    // Backspace×3 清空 → Enter → 校验失败。
    for _ in 0..3 {
        driver.pump_event(&key("Backspace", None));
        driver.pump_frame();
    }
    driver.pump_event(&key("Enter", None));
    driver.pump_frame();
    println!("after empty submit: {:?}", scene_texts(driver.host().scene()));
}
