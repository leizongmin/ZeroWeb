//! Counter 示例的无窗口驱动（`cargo run -p zero-ui-examples --example counter`）。
//!
//! 经 [`WinitDriver`](zero_ui_adapter_winit::WinitDriver)（DC-2 winit 事件循环的可测试
//! run-loop 核心）驱动 retained 闭环：事件 → host dispatch → `app.dispatch` reducer →
//! 重建声明树 → invalidation → 帧重绘。真实 winit `EventLoop::run`（开窗/surface/首帧）
//! 需 GUI；此处用脚本化点击喂 `pump_event`，打印每次状态变化后的 Label 文案与场景图元数，
//! 证明**同一 driver 路径**在 headless 可运行（driver 内部已 dispatch + reducer + 重建，
//! 无需手写 `set_root`/`layout`/`paint`）。

use zero_ui_adapter_winit::WinitDriver;
use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::Point;
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::widget::WidgetId;
use zero_ui_examples::{CounterApp, register_counter_factories};
use zero_ui_render::Scene;
use zero_ui_render::render_node::RenderPrimitive;

/// 取 Scene 里第一条文本图元的文案（counter 的 Label 输出 "Count: N"）。
fn label_text(scene: &Scene) -> Option<String> {
    scene.entries.iter().find_map(|e| match &e.primitive {
        RenderPrimitive::Text { text, .. } => Some(text.clone()),
        _ => None,
    })
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

/// 对指定按钮做一次完整点击（press + release + 帧推进）。
/// `pump_event` 内部完成 dispatch → 收 emit → app.dispatch reducer → Handled 即重建；
/// `pump_frame` 按 invalidation 重绘。无需手写 set_root/layout/paint。
fn click(driver: &mut WinitDriver<'_>, id: &str) {
    let center = {
        let rect = driver
            .host()
            .rect_of(&WidgetId::new(id))
            .unwrap_or_else(|| panic!("widget {id} must be laid out before click"));
        Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        )
    };
    driver.pump_event(&pointer(PointerPhase::Pressed, center));
    driver.pump_event(&pointer(PointerPhase::Released, center));
    driver.pump_frame();
}

fn main() {
    let mut app = CounterApp::new();
    let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
    register_counter_factories(driver.host_mut());
    driver.begin();
    println!(
        "initial: {} (scene primitives: {})",
        label_text(driver.host().scene()).unwrap_or_default(),
        driver.host().scene().entries.len()
    );

    // 点 3 次 "+"：driver 内部 emit → app.dispatch(reducer) → 重建 → 重绘。
    for i in 1..=3 {
        click(&mut driver, "inc");
        println!(
            "click #{i}: {} (scene primitives: {})",
            label_text(driver.host().scene()).unwrap_or_default(),
            driver.host().scene().entries.len()
        );
    }

    // 点 1 次 "-"。
    click(&mut driver, "dec");
    println!(
        "final: {} (scene primitives: {})",
        label_text(driver.host().scene()).unwrap_or_default(),
        driver.host().scene().entries.len()
    );
}
