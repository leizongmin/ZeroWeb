//! Counter retained 运行时闭环集成测试（DC-14）。
//!
//! 证明：① SDK 可被外部复用（本 crate 不依赖浏览器 crate）；② retained 运行时
//! 事件→Action→AppState reducer→重建 WidgetSpec→re-layout/paint 闭环；
//! ③ Scene 文本随状态更新；④ 稳定 WidgetId 控件实例跨重建复用。

use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase, UiEvent};
use zero_ui_core::geometry::{Constraints, Point, Size};
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

/// 对指定 WidgetId 节点做一次完整点击（press + release），返回 release 发出的 actions。
fn click(host: &mut WidgetHost, id: &str) -> Vec<EmittedAction> {
    let rect = host
        .rect_of(&WidgetId::new(id))
        .unwrap_or_else(|| panic!("widget {id} must be laid out before click"));
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

#[test]
fn counter_retained_closed_loop() {
    let mut app = CounterApp::new();
    let mut host = WidgetHost::new();
    register_counter_factories(&mut host);
    let vp = Constraints::loose(Size::new(400.0, 300.0));

    // 初始：count=0，Label 文案 "Count: 0"。
    host.set_root(&app.build_spec());
    host.layout(vp);
    host.paint();
    assert_eq!(app.count(), 0);
    assert_eq!(label_text(host.scene()).as_deref(), Some("Count: 0"));

    // 记录 "+" 按钮的初始 epoch（断言跨重建复用）。
    let inc_epoch_before = host.creation_epoch(&WidgetId::new("inc")).unwrap();

    // 点 3 次 "+"：每次 emitted → reducer → 重建 → 重渲染。
    for _ in 0..3 {
        for action in click(&mut host, "inc") {
            app.reduce(&action);
        }
        host.set_root(&app.build_spec());
        host.layout(vp);
        host.paint();
    }
    assert_eq!(app.count(), 3, "three increments");
    assert_eq!(
        label_text(host.scene()).as_deref(),
        Some("Count: 3"),
        "scene text must reflect new state"
    );

    // "+" 按钮跨 4 次重建仍被复用（epoch 不变）。
    let inc_epoch_after = host.creation_epoch(&WidgetId::new("inc")).unwrap();
    assert_eq!(
        inc_epoch_before, inc_epoch_after,
        "stable WidgetId button must be reused across rebuilds"
    );

    // 点 1 次 "-"。
    for action in click(&mut host, "dec") {
        app.reduce(&action);
    }
    host.set_root(&app.build_spec());
    host.layout(vp);
    host.paint();
    assert_eq!(app.count(), 2);
    assert_eq!(label_text(host.scene()).as_deref(), Some("Count: 2"));
}

#[test]
fn counter_external_reusable_no_browser_deps() {
    // 结构性断言：counter 只用通用 SDK crate。
    // （依赖隔离的机械验证见 evidence/dep-isolation-*；这里断言运行时可用性。）
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
    // reducer 可独立于 host 驱动（无浏览器运行时；演示 SDK 外部可复用）。
    app.reduce(&EmittedAction {
        action: zero_ui_core::action::ActionId::new("counter.inc"),
        payload: None,
    });
    assert_eq!(app.count(), 1);
}
