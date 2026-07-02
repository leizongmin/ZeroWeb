//! Counter retained 运行时闭环集成测试（DC-14 / DC-2）。
//!
//! 证明：① SDK 可被外部复用（本 crate 不依赖浏览器 crate）；② 经 [`WinitDriver`]
//! （DC-2 run-loop 核心）驱动 retained 闭环：事件 → Action → AppState reducer →
//! 重建 WidgetSpec → re-layout/paint；③ Scene 文本随状态更新；④ 稳定 WidgetId 控件
//! 实例跨重建复用。第二个测试保留低层 host 直接驱动，证明 SDK 在多个抽象层级可用。

use zero_ui_adapter_winit::WinitDriver;
use zero_ui_core::action::ActionId;
use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Size};
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::widget::WidgetId;
use zero_ui_examples::{CounterApp, register_counter_factories};
use zero_ui_render::Scene;
use zero_ui_render::render_node::RenderPrimitive;
use zero_ui_runtime::{EmittedAction, WidgetHost};

/// 取 Scene 里第一条文本图元的文案（Label 输出）。
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

/// 经 driver 对指定按钮做一次完整点击（press + release + 帧推进）。
/// driver.pump_event 内部 dispatch → reducer → Handled 即重建；pump_frame 重绘。
fn click(driver: &mut WinitDriver<'_>, id: &WidgetId) {
    let center = {
        let rect = driver
            .host()
            .rect_of(id)
            .unwrap_or_else(|| panic!("widget {} must be laid out before click", id.0));
        Point::new(
            rect.origin.x + rect.size.width / 2.0,
            rect.origin.y + rect.size.height / 2.0,
        )
    };
    driver.pump_event(&pointer(PointerPhase::Pressed, center));
    driver.pump_event(&pointer(PointerPhase::Released, center));
    driver.pump_frame();
}

#[test]
fn counter_retained_closed_loop_via_driver() {
    // 经 WinitDriver 驱动：证明 DC-2 run-loop 核心端到端打通 counter 的 retained 闭环。
    let mut app = CounterApp::new();
    let inc = WidgetId::new("inc");
    let dec = WidgetId::new("dec");
    // driver 持 &mut app → app.count() 须在 driver 作用域外读（下方最终断言）；
    // 中间状态经 Label 文案（scene）断言，等价于 count。
    {
        let mut driver = WinitDriver::new(&mut app, WindowMetrics::desktop());
        register_counter_factories(driver.host_mut());
        driver.begin();

        // 初始：Label 文案 "Count: 0"。
        assert_eq!(label_text(driver.host().scene()).as_deref(), Some("Count: 0"));

        // 记录 "+" 按钮初始 epoch（断言跨重建复用）。
        let inc_epoch_before = driver.host().creation_epoch(&inc).unwrap();

        // 点 3 次 "+"：每次 pump_event 内部 emit→reducer→重建→pump_frame 重绘。
        for _ in 0..3 {
            click(&mut driver, &inc);
        }
        assert_eq!(
            label_text(driver.host().scene()).as_deref(),
            Some("Count: 3"),
            "scene text must reflect new state"
        );

        // "+" 按钮跨多次重建仍被复用（epoch 不变）。
        let inc_epoch_after = driver.host().creation_epoch(&inc).unwrap();
        assert_eq!(
            inc_epoch_before, inc_epoch_after,
            "stable WidgetId button must be reused across rebuilds"
        );

        // 点 1 次 "-"。
        click(&mut driver, &dec);
        assert_eq!(label_text(driver.host().scene()).as_deref(), Some("Count: 2"));
    }
    // driver 离开作用域 → 释放对 app 的可变借用。
    assert_eq!(app.count(), 2, "reducer 最终状态正确");
}

#[test]
fn counter_external_reusable_no_browser_deps() {
    // 结构性断言：counter 只用通用 SDK crate（依赖隔离的机械验证见 evidence/dep-isolation-*）。
    // 保留低层 host 直接驱动，证明 SDK 在 host 与 driver 两个抽象层级都可复用。
    let mut app = CounterApp::new();
    let mut host = WidgetHost::new();
    register_counter_factories(&mut host);
    host.set_root(&app.build_spec());
    let size = host.layout(Constraints::loose(Size::new(200.0, 200.0)));
    // Column：Label(24h) + Row(32h) = 56；宽 = max(Label 宽, Row 宽)。
    assert!(size.height >= 56.0, "column height should fit label + buttons");
    let scene = host.paint();
    // Label 文本 + 2 个 Button 背景 = 至少 3 个图元。
    assert!(
        scene.entries.len() >= 3,
        "scene should contain label text + button backgrounds"
    );
    // reducer 可独立于 host/driver 驱动（无浏览器运行时；演示 SDK 外部可复用）。
    app.reduce(&EmittedAction {
        action: ActionId::new("counter.inc"),
        payload: None,
    });
    assert_eq!(app.count(), 1);
}
