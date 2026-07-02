//! Counter 示例的无窗口驱动（`cargo run -p zero-ui-examples --example counter`）。
//!
//! 无真实窗口后端（winit adapter 在 M2/M4）；此处用脚本化点击驱动 retained 运行时，
//! 打印每次状态变化后的计数与场景图元数，证明事件→Action→状态→重渲染闭环可运行。

use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Size};
use zero_ui_core::widget::WidgetId;
use zero_ui_examples::{CounterApp, register_counter_factories};
use zero_ui_runtime::{EmittedAction, WidgetHost};

/// 对指定按钮做一次点击，返回 release 发出的 actions（真实 retained loop：交给应用 reducer）。
fn click(host: &mut WidgetHost, id: &str) -> Vec<EmittedAction> {
    let rect = host.rect_of(&WidgetId::new(id)).expect("button must be laid out");
    let center = Point::new(
        rect.origin.x + rect.size.width / 2.0,
        rect.origin.y + rect.size.height / 2.0,
    );
    let _ = host.dispatch_event(&UiEvent::Pointer {
        phase: PointerPhase::Pressed,
        button: Some(PointerButton::Primary),
        position: center,
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    });
    host.dispatch_event(&UiEvent::Pointer {
        phase: PointerPhase::Released,
        button: Some(PointerButton::Primary),
        position: center,
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    })
}

fn main() {
    let mut app = CounterApp::new();
    let mut host = WidgetHost::new();
    register_counter_factories(&mut host);
    let vp = Constraints::loose(Size::new(400.0, 300.0));

    // 初始渲染。
    host.set_root(&app.build_spec());
    host.layout(vp);
    host.paint();
    println!(
        "initial count: {} (scene primitives: {})",
        app.count(),
        host.scene().entries.len()
    );

    // 点 3 次 "+"，每次 emitted → reducer → 重建 → 重渲染。
    for i in 1..=3 {
        for action in click(&mut host, "inc") {
            println!("click #{}: action {}", i, action.action.0);
            app.reduce(&action);
        }
        host.set_root(&app.build_spec());
        host.layout(vp);
        host.paint();
        println!(
            "  count: {} (scene primitives: {})",
            app.count(),
            host.scene().entries.len()
        );
    }

    // 点 1 次 "-"。
    for action in click(&mut host, "dec") {
        app.reduce(&action);
    }
    host.set_root(&app.build_spec());
    host.layout(vp);
    host.paint();
    println!(
        "  final count: {} (scene primitives: {})",
        app.count(),
        host.scene().entries.len()
    );
}
