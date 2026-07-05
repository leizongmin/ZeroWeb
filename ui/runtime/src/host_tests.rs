use super::*;
use zero_ui_core::binding::Value;
use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase};
use zero_ui_core::semantics::SemanticsLabel;
use zero_ui_core::theme::Color;
use zero_ui_render::render_node::RenderPrimitive;

/// 测试用「色块」叶子控件：按 props.color 填充自身 layout 尺寸；点击 emit `tap`。
struct Patch {
    color: Color,
    action: ActionId,
}
impl Widget for Patch {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut zero_ui_core::widget::UpdateCtx, props: &PropsMap) {
        if let Some(Value::Text(_)) = props.get("color") {
            *ctx.invalidation |= InvalidationFlags::NEEDS_PAINT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        if let UiEvent::Pointer {
            phase: PointerPhase::Released,
            button: Some(PointerButton::Primary),
            ..
        } = event
        {
            EventResult::Emit(self.action.clone())
        } else if let UiEvent::Key {
            code,
            action: key_action,
            ..
        } = event
        {
            // Enter（聚焦时）→ emit action（演示键盘路由）。
            if code.0.as_str() == "Enter" && matches!(key_action, zero_ui_core::event::KeyAction::Pressed) {
                EventResult::Emit(self.action.clone())
            } else {
                EventResult::Ignored
            }
        } else {
            EventResult::Ignored
        }
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        // 期望 50x50，受约束裁剪。
        Size::new(50.0, 50.0).clamp(constraints)
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, 50.0, 50.0), self.color);
    }
    fn semantics(&self, _ctx: &mut zero_ui_core::widget::SemanticsCtx) {}
    fn focusable(&self) -> bool {
        true
    }
}

/// 用 Patch 工厂构造 host（component "Patch"）。
fn patch_host() -> WidgetHost {
    let mut host = WidgetHost::new();
    host.register("Patch", |spec| {
        let color = match spec.props.get("color") {
            Some(Value::Text(s)) => match s.as_str() {
                "red" => Color::rgb(1.0, 0.0, 0.0),
                "blue" => Color::rgb(0.0, 0.0, 1.0),
                _ => Color::rgb(0.5, 0.5, 0.5),
            },
            _ => Color::rgb(0.5, 0.5, 0.5),
        };
        let action = spec
            .props
            .get("action")
            .and_then(|v| match v {
                Value::Text(s) => Some(ActionId::new(s)),
                _ => None,
            })
            .unwrap_or_else(|| ActionId::new("tap"));
        Box::new(Patch { color, action })
    });
    host
}

fn patch(color: &str, id: &str, action: &str) -> WidgetSpec {
    let mut s = WidgetSpec::new("Patch");
    s.id = Some(WidgetId::new(id));
    s.props.insert("color", Value::Text(color.into()));
    s.props.insert("action", Value::Text(action.into()));
    s
}

/// 拿到 Scene 里所有 FillRect 的 (rect, color)。
fn fills(scene: &Scene) -> Vec<(Rect, Color)> {
    scene
        .entries
        .iter()
        .filter_map(|e| match &e.primitive {
            RenderPrimitive::FillRect { rect, color, .. } => Some((*rect, *color)),
            _ => None,
        })
        .collect()
}

/// 测试用叶子控件：layout 宽度由 props.w 决定（演示「props 变化改尺寸 → 需 relayout」）。
/// 受控：update 同步 props.w 到 self.w（factory 仅在 mount 时跑，reconcile 靠 update）。
struct Sizer {
    w: f32,
}
impl Widget for Sizer {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, ctx: &mut zero_ui_core::widget::UpdateCtx, props: &PropsMap) {
        let new_w = match props.get("w") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => self.w,
        };
        if (new_w - self.w).abs() > f32::EPSILON {
            self.w = new_w;
            *ctx.invalidation |= InvalidationFlags::NEEDS_LAYOUT;
        }
    }
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
        Size::new(
            self.w.clamp(c.min_width, c.max_width),
            10.0_f32.clamp(c.min_height, c.max_height),
        )
    }
    fn paint(&mut self, _ctx: &mut PaintCtx) {}
    fn semantics(&self, _ctx: &mut zero_ui_core::widget::SemanticsCtx) {}
}

fn sizer_host() -> WidgetHost {
    let mut host = WidgetHost::new();
    host.register("Sizer", |spec| {
        let w = match spec.props.get("w") {
            Some(Value::Float(f)) => *f as f32,
            Some(Value::Int(i)) => *i as f32,
            _ => 10.0,
        };
        Box::new(Sizer { w })
    });
    host
}

fn sizer_spec(w: f32) -> WidgetSpec {
    let mut s = WidgetSpec::new("Sizer");
    s.id = Some(WidgetId::new("sizer"));
    s.props.insert("w", Value::Float(w as f64));
    s
}

#[test]
fn reconcile_props_change_marks_host_needs_layout() {
    // 回归守卫：set_root reconcile（props 变化）必须在 host 级标 NEEDS_LAYOUT。
    // 此前 set_root reconcile 只标 NEEDS_PAINT|SEMANTICS → needs_layout() 为假 →
    // driver/host 不 relayout → state 变化改 widget 尺寸时 scene 停在旧几何。
    // reconcile_node 已在节点级标 NEEDS_LAYOUT（host.rs:705），但未传播到 host.pending。
    let mut host = sizer_host();
    host.set_root(&sizer_spec(100.0));
    host.layout(Constraints::loose(Size::new(800.0, 600.0)));
    let w0 = host.rect_of(&WidgetId::new("sizer")).expect("laid out").size.width;
    assert_eq!(w0, 100.0);
    assert!(!host.needs_layout(), "首帧 layout 后 NEEDS_LAYOUT 应清空");

    // props 变化（w 100→250）→ reconcile 应标 host NEEDS_LAYOUT（可能改尺寸）。
    host.set_root(&sizer_spec(250.0));
    assert!(
        host.needs_layout(),
        "props 变化 → host 须标 NEEDS_LAYOUT（否则不 relayout，几何停滞）"
    );
    // relayout 后几何反映新 w（旧 bug 下 geometry 停在 100）。
    host.layout(Constraints::loose(Size::new(800.0, 600.0)));
    let w1 = host.rect_of(&WidgetId::new("sizer")).expect("laid out").size.width;
    assert_eq!(w1, 250.0, "relayout 后宽度更新为新 w");
}

#[test]
fn column_layout_and_paint_produces_stacked_absolute_rects() {
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.props.insert("gap", Value::Float(10.0));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);

    // 视口足够大：Column 高 = 50 + gap10 + 50 = 110，宽 = 50。
    let size = host.layout(Constraints::loose(Size::new(400.0, 400.0)));
    assert_eq!(size, Size::new(50.0, 110.0));
    // 子节点绝对 rect：a 在 (0,0,50,50)；b 在 (0,60,50,110)。
    assert_eq!(
        host.rect_of(&WidgetId::new("a")),
        Some(Rect::from_ltrb(0.0, 0.0, 50.0, 50.0))
    );
    assert_eq!(
        host.rect_of(&WidgetId::new("b")),
        Some(Rect::from_ltrb(0.0, 60.0, 50.0, 110.0))
    );

    let scene = host.paint().clone();
    let f = fills(&scene);
    assert_eq!(f.len(), 2, "two patches → two fill_rects");
    // a（红）在 (0,0)；b（蓝）平移到 (0,60)。
    assert!(f.contains(&(Rect::from_ltrb(0.0, 0.0, 50.0, 50.0), Color::rgb(1.0, 0.0, 0.0))));
    assert!(f.contains(&(Rect::from_ltrb(0.0, 60.0, 50.0, 110.0), Color::rgb(0.0, 0.0, 1.0))));
}

#[test]
fn click_hit_tests_to_correct_patch_and_emits_action() {
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    // 点击下半区（b 的位置 y≈60..110）→ emit app.b，不是 app.a。
    let click_on_b = UiEvent::Pointer {
        phase: PointerPhase::Released,
        button: Some(PointerButton::Primary),
        position: Point::new(10.0, 80.0),
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    };
    let emitted = host.dispatch_event(&click_on_b);
    assert_eq!(emitted.len(), 1);
    assert_eq!(emitted[0].action, ActionId::new("app.b"));
    assert!(host.needs_paint(), "emit should request re-paint");

    // 点击上半区（a）→ emit app.a。
    let click_on_a = UiEvent::Pointer {
        phase: PointerPhase::Released,
        button: Some(PointerButton::Primary),
        position: Point::new(10.0, 10.0),
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    };
    let emitted = host.dispatch_event(&click_on_a);
    assert_eq!(emitted[0].action, ActionId::new("app.a"));
}

#[test]
fn rebuild_with_stable_widget_id_reuses_instance() {
    // 通过 epoch 间接验证复用：reconcile 后同 id 节点的 epoch 不变（未被重建）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);

    // 首次建树 epoch=1。
    assert_eq!(host.creation_epoch(&WidgetId::new("a")), Some(1));

    // 重建（同 id "a"，新增兄弟 "b"）→ a 应被复用，epoch 仍为 1。
    let mut root2 = WidgetSpec::new("Column");
    root2.id = Some(WidgetId::new("root"));
    root2.children.push(patch("red", "a", "app.a"));
    root2.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root2);

    assert_eq!(
        host.creation_epoch(&WidgetId::new("a")),
        Some(1),
        "stable WidgetId node must be reused, not recreated"
    );
    assert_eq!(
        host.creation_epoch(&WidgetId::new("b")),
        Some(2),
        "new node gets current epoch"
    );
}

#[test]
fn rebuild_changing_component_type_recreates_node() {
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    let mut a = patch("red", "a", "app.a");
    root.children.push(a.clone());
    host.set_root(&root);
    assert_eq!(host.creation_epoch(&WidgetId::new("a")), Some(1));

    // 同 id "a" 但 component 换成 "Row"（非 Patch）→ 不应复用，重建为 epoch=2。
    a.component = ComponentType::new("Row");
    let mut root2 = WidgetSpec::new("Column");
    root2.id = Some(WidgetId::new("root"));
    root2.children.push(a);
    host.set_root(&root2);
    assert_eq!(
        host.creation_epoch(&WidgetId::new("a")),
        Some(2),
        "changed ComponentType must recreate the node"
    );
}

#[test]
fn row_and_stack_geometry() {
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.id = Some(WidgetId::new("root"));
    root.props.insert("gap", Value::Float(5.0));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    let size = host.layout(Constraints::loose(Size::new(400.0, 400.0)));
    // Row：宽 = 50 + gap5 + 50 = 105，高 = 50。
    assert_eq!(size, Size::new(105.0, 50.0));
    assert_eq!(
        host.rect_of(&WidgetId::new("b")),
        Some(Rect::from_ltrb(55.0, 0.0, 105.0, 50.0))
    );

    // Stack：两子叠在同一起点。
    let mut host2 = patch_host();
    let mut stack = WidgetSpec::new("Stack");
    stack.id = Some(WidgetId::new("root"));
    stack.children.push(patch("red", "a", "app.a"));
    stack.children.push(patch("blue", "b", "app.b"));
    host2.set_root(&stack);
    host2.layout(Constraints::loose(Size::new(400.0, 400.0)));
    assert_eq!(
        host2.rect_of(&WidgetId::new("a")),
        host2.rect_of(&WidgetId::new("b")),
        "Stack children share origin"
    );
}

#[test]
fn custom_container_via_layout_prop() {
    // 任意组件名经 props.layout="row" 声明为行容器（host 不硬编码业务组件名）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("browser.ToolbarRow");
    root.id = Some(WidgetId::new("root"));
    root.props.insert("layout", Value::Text("row".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    let size = host.layout(Constraints::loose(Size::new(400.0, 400.0)));
    // Row：两 Patch(50) 水平排列 → 宽 100，高 50。
    assert_eq!(size, Size::new(100.0, 50.0));
    assert_eq!(
        host.rect_of(&WidgetId::new("b")),
        Some(Rect::from_ltrb(50.0, 0.0, 100.0, 50.0))
    );
}

#[test]
fn unknown_component_fills_available_space() {
    let mut host = WidgetHost::new(); // 无任何注册
    let mut root = WidgetSpec::new("Mystery");
    root.id = Some(WidgetId::new("root"));
    host.set_root(&root);
    let size = host.layout(Constraints::loose(Size::new(300.0, 200.0)));
    assert_eq!(size, Size::new(300.0, 200.0));
    // paint 不 panic、产出空 Scene（无 widget）。
    let scene = host.paint();
    assert!(scene.entries.is_empty());
}

// ── flex 弹性布局（DC-2 host 布局增强）──────────────────────────────

/// 测试用「填满」叶子控件：layout 返回约束的 max（填满被分配空间），paint 填该尺寸。
/// 用于验证 flex 子节点按份额填满主轴。携带上次 measure 的尺寸供 paint 复用。
struct Fill {
    color: Color,
    size: Size,
}
impl Widget for Fill {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut zero_ui_core::widget::UpdateCtx, _props: &PropsMap) {}
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        let s = Size::new(constraints.max_width, constraints.max_height);
        self.size = s;
        s
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.recorder
            .fill_rect(Rect::from_origin_size(Point::ZERO, self.size), self.color);
    }
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

fn fill_host() -> WidgetHost {
    let mut host = patch_host();
    host.register("Fill", |spec| {
        let color = match spec.props.get("color") {
            Some(Value::Text(s)) => match s.as_str() {
                "red" => Color::rgb(1.0, 0.0, 0.0),
                "blue" => Color::rgb(0.0, 0.0, 1.0),
                "green" => Color::rgb(0.0, 0.5, 0.0),
                _ => Color::rgb(0.5, 0.5, 0.5),
            },
            _ => Color::rgb(0.5, 0.5, 0.5),
        };
        Box::new(Fill {
            color,
            size: Size::ZERO,
        })
    });
    host
}

fn fill(color: &str, id: &str, flex: i64) -> WidgetSpec {
    let mut s = WidgetSpec::new("Fill");
    s.id = Some(WidgetId::new(id));
    s.props.insert("color", Value::Text(color.into()));
    s.props.insert("flex", Value::Int(flex));
    s
}

#[test]
fn row_flex_distributes_space_evenly_between_two_flex_children() {
    // 两个 flex=1 的 Fill 子节点均分 Row 主轴（width=400）→ 各 200。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.id = Some(WidgetId::new("root"));
    root.children.push(fill("red", "a", 1));
    root.children.push(fill("blue", "b", 1));
    host.set_root(&root);
    let size = host.layout(Constraints::loose(Size::new(400.0, 50.0)));
    assert_eq!(size, Size::new(400.0, 50.0), "flex children fill the row");
    assert_eq!(
        host.rect_of(&WidgetId::new("a")),
        Some(Rect::from_ltrb(0.0, 0.0, 200.0, 50.0)),
        "first flex child gets first half"
    );
    assert_eq!(
        host.rect_of(&WidgetId::new("b")),
        Some(Rect::from_ltrb(200.0, 0.0, 400.0, 50.0)),
        "second flex child gets second half"
    );
}

#[test]
fn row_flex_with_one_fixed_and_one_flex_child() {
    // 固定 Patch(50x50, flex=0) + Fill(flex=1)：固定取固有 50，Fill 填满剩余 350。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "fixed", "app.fixed"));
    root.children.push(fill("blue", "flex", 1));
    host.set_root(&root);
    let size = host.layout(Constraints::loose(Size::new(400.0, 50.0)));
    assert_eq!(size, Size::new(400.0, 50.0));
    assert_eq!(
        host.rect_of(&WidgetId::new("fixed")),
        Some(Rect::from_ltrb(0.0, 0.0, 50.0, 50.0)),
        "non-flex child keeps intrinsic size"
    );
    assert_eq!(
        host.rect_of(&WidgetId::new("flex")),
        Some(Rect::from_ltrb(50.0, 0.0, 400.0, 50.0)),
        "flex child fills remaining space after fixed child"
    );
}

#[test]
fn row_flex_respects_weight_ratios_and_gap() {
    // flex 权重 1:2:1（总 4）+ gap=10，主轴 410：gaps=20，free=390 → 份额 97.5/195/97.5。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.id = Some(WidgetId::new("root"));
    root.props.insert("gap", Value::Float(10.0));
    root.children.push(fill("red", "a", 1));
    root.children.push(fill("blue", "b", 2));
    root.children.push(fill("green", "c", 1));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(410.0, 30.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    let c = host.rect_of(&WidgetId::new("c")).unwrap();
    assert!(
        (a.size.width - 97.5).abs() < 1e-5,
        "a gets free/4 = 97.5, got {}",
        a.size.width
    );
    assert!(
        (b.size.width - 195.0).abs() < 1e-5,
        "b gets free*2/4 = 195, got {}",
        b.size.width
    );
    assert!(
        (c.size.width - 97.5).abs() < 1e-5,
        "c gets free/4 = 97.5, got {}",
        c.size.width
    );
    // gap 投影到相邻起点：b 起点 = a 终点 + gap。
    assert!((b.origin.x - (a.origin.x + a.size.width + 10.0)).abs() < 1e-5);
    assert!((c.origin.x - (b.origin.x + b.size.width + 10.0)).abs() < 1e-5);
}

#[test]
fn column_flex_distributes_vertical_space_proportionally() {
    // Column 主轴 = Y：三个 flex=1 Fill 均分 height=300 → 各 100，垂直堆叠。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(fill("red", "a", 1));
    root.children.push(fill("blue", "b", 1));
    root.children.push(fill("green", "c", 1));
    host.set_root(&root);
    let size = host.layout(Constraints::loose(Size::new(100.0, 300.0)));
    assert_eq!(size, Size::new(100.0, 300.0));
    assert_eq!(
        host.rect_of(&WidgetId::new("a")),
        Some(Rect::from_ltrb(0.0, 0.0, 100.0, 100.0))
    );
    assert_eq!(
        host.rect_of(&WidgetId::new("b")),
        Some(Rect::from_ltrb(0.0, 100.0, 100.0, 200.0))
    );
    assert_eq!(
        host.rect_of(&WidgetId::new("c")),
        Some(Rect::from_ltrb(0.0, 200.0, 100.0, 300.0))
    );
}

#[test]
fn flex_default_zero_keeps_backward_compatible_greedy_behavior() {
    // 无 flex（全 0）时：第一个 Fill（填满）吃光全部主轴，第二个 Fill 得 0——
    // 与历史贪心行为一致（向后兼容），证明 flex 是 opt-in。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.id = Some(WidgetId::new("root"));
    let mut first = WidgetSpec::new("Fill");
    first.id = Some(WidgetId::new("a"));
    first.props.insert("color", Value::Text("red".into()));
    // 不设 flex → 默认 0（贪心）。
    let mut second = WidgetSpec::new("Fill");
    second.id = Some(WidgetId::new("b"));
    second.props.insert("color", Value::Text("blue".into()));
    root.children.push(first);
    root.children.push(second);
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 50.0)));
    assert_eq!(
        host.rect_of(&WidgetId::new("a")),
        Some(Rect::from_ltrb(0.0, 0.0, 400.0, 50.0)),
        "greedy non-flex fill child consumes all space (legacy behavior)"
    );
    assert_eq!(
        host.rect_of(&WidgetId::new("b")),
        Some(Rect::from_ltrb(400.0, 0.0, 400.0, 50.0)),
        "second non-flex child gets zero remaining (legacy behavior)"
    );
}

#[test]
fn flex_paints_into_unified_scene_with_correct_rects() {
    // flex 分配后的几何应反映到统一 Scene：两个 flex Fill 各画一个 200x50 矩形。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.id = Some(WidgetId::new("root"));
    root.children.push(fill("red", "a", 1));
    root.children.push(fill("blue", "b", 1));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 50.0)));
    let scene = host.paint().clone();
    let f = fills(&scene);
    assert_eq!(f.len(), 2, "two flex fills → two fill_rects");
    assert!(f.contains(&(Rect::from_ltrb(0.0, 0.0, 200.0, 50.0), Color::rgb(1.0, 0.0, 0.0))));
    assert!(f.contains(&(Rect::from_ltrb(200.0, 0.0, 400.0, 50.0), Color::rgb(0.0, 0.0, 1.0))));
}

// ── 交叉轴对齐（cross_axis_align）测试 ────────────────
//
// 容器交叉尺寸 = 最高/最宽子节点（measure_linear 的 total_cross）。Fill 满高/宽，
// Patch 固有 50x50 较小 → 对齐把 Patch 在交叉轴上居中/贴底/贴顶。这是 chrome 工具栏
// 「高元素满高、短元素（如文本/图标）垂直居中」所需的真实排版能力。

#[test]
fn row_cross_axis_center_aligns_shorter_child_vertically() {
    // Row 高 100：Fill(flex=1) 满高 100 + Patch(50x50)。容器交叉高 = max(100,50) = 100。
    // center → Patch 垂直居中：y 偏移 (100-50)/2 = 25 → rect top=25, bottom=75。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("cross_axis_align", Value::Text("center".into()));
    root.children.push(fill("blue", "fill", 1));
    root.children.push(patch("red", "small", "app.small"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 100.0)));
    let small = host.rect_of(&WidgetId::new("small")).unwrap();
    assert_eq!(small.top(), 25.0, "centered Patch top = (100-50)/2");
    assert_eq!(small.bottom(), 75.0);
}

#[test]
fn row_cross_axis_end_aligns_shorter_child_to_bottom() {
    // end → Patch 贴底：y 偏移 100-50 = 50 → rect top=50, bottom=100。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("cross_axis_align", Value::Text("end".into()));
    root.children.push(fill("blue", "fill", 1));
    root.children.push(patch("red", "small", "app.small"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 100.0)));
    let small = host.rect_of(&WidgetId::new("small")).unwrap();
    assert_eq!(small.top(), 50.0);
    assert_eq!(small.bottom(), 100.0);
}

#[test]
fn row_cross_axis_default_start_is_backward_compatible() {
    // 无 cross_axis_align → 缺省 Start → Patch 顶部：top=0, bottom=50（历史行为不变）。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.children.push(fill("blue", "fill", 1));
    root.children.push(patch("red", "small", "app.small"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 100.0)));
    let small = host.rect_of(&WidgetId::new("small")).unwrap();
    assert_eq!(small.top(), 0.0, "default Start keeps legacy top alignment");
    assert_eq!(small.bottom(), 50.0);
}

#[test]
fn row_cross_axis_accepts_top_bottom_aliases() {
    // Row 接受 top/bottom 别名（= start/end）。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("cross_axis_align", Value::Text("bottom".into()));
    root.children.push(fill("blue", "fill", 1));
    root.children.push(patch("red", "small", "app.small"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 100.0)));
    let small = host.rect_of(&WidgetId::new("small")).unwrap();
    assert_eq!(small.top(), 50.0, "bottom alias = end");
}

#[test]
fn column_cross_axis_center_aligns_shorter_child_horizontally() {
    // Column 宽 100：Fill(flex=1) 满宽 100 + Patch(50x50)。容器交叉宽 = 100。
    // center → Patch 水平居中：x 偏移 (100-50)/2 = 25 → rect left=25, right=75。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Column");
    root.props.insert("cross_axis_align", Value::Text("center".into()));
    root.children.push(fill("blue", "fill", 1));
    root.children.push(patch("red", "small", "app.small"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(100.0, 400.0)));
    let small = host.rect_of(&WidgetId::new("small")).unwrap();
    assert_eq!(small.left(), 25.0, "centered Patch left = (100-50)/2");
    assert_eq!(small.right(), 75.0);
}

#[test]
fn cross_axis_unknown_value_falls_back_to_start() {
    // 未知对齐值 → 回落 Start（不 panic）。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("cross_axis_align", Value::Text("diagonal".into()));
    root.children.push(fill("blue", "fill", 1));
    root.children.push(patch("red", "small", "app.small"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 100.0)));
    let small = host.rect_of(&WidgetId::new("small")).unwrap();
    assert_eq!(small.top(), 0.0, "unknown value → Start fallback");
}

// ── fill-sizing（遵守 min 约束）+ 主轴对齐（main_axis_align）测试 ──────────
//
// 容器现遵守传入约束的 min：tight/exact 约束下容器填满主轴（fill-sizing），从而为主轴对齐
// 提供剩余空间。默认 loose（min=0）仍为 content-sized（向后兼容）。

#[test]
fn row_fills_to_tight_main_constraint() {
    // tight(400,100) 根 Row + 两 Patch(50)：内容打包 100，min=400 → 容器填满到 400x100
    // （此前 content-sized 会返回 100x50）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    let size = host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    assert_eq!(
        size,
        Size::new(400.0, 100.0),
        "tight constraint → container fills main+cross"
    );
}

#[test]
fn row_loose_constraint_still_content_sized_backward_compatible() {
    // loose(400,100)（min=0）→ 容器仍 content-sized 100x50（fill-sizing 不激活）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    let size = host.layout(Constraints::loose(Size::new(400.0, 100.0)));
    assert_eq!(size, Size::new(100.0, 50.0), "loose → content-sized (backward compat)");
}

#[test]
fn row_main_axis_center_with_tight_constraint() {
    // tight(400,100) + center：容器填满 400，打包 100，free=300，偏移 150 → 第一 Patch 150..200。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("main_axis_align", Value::Text("center".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    assert_eq!(a.left(), 150.0, "center: first child at free/2 = 150");
    assert_eq!(a.right(), 200.0);
    assert_eq!(b.left(), 200.0);
    assert_eq!(b.right(), 250.0);
}

#[test]
fn row_main_axis_end_with_tight_constraint() {
    // tight(400,100) + end：偏移 300 → 第二 Patch 350..400（贴右）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("main_axis_align", Value::Text("end".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    assert_eq!(b.right(), 400.0, "end: last child flush to right edge");
    assert_eq!(b.left(), 350.0);
}

#[test]
fn row_main_axis_center_respects_gap_in_packed_length() {
    // tight(400,100) + center + gap=10：打包 110，free=290，偏移 145 → 第一 145..195，第二 205..255。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("main_axis_align", Value::Text("center".into()));
    root.props.insert("gap", Value::Float(10.0));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    assert_eq!(a.left(), 145.0, "center with gap: (400-110)/2 = 145");
    assert_eq!(b.left(), 205.0, "second = 145 + 50 + gap(10)");
}

#[test]
fn row_main_axis_space_between_flush_first_and_last() {
    // tight(400,100) + space_between + 2×Patch(50)：content=100，extra=300，n=2 →
    // between_extra=300，spacing=300。首 Patch 0..50，末 Patch 350..400（贴两端）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props
        .insert("main_axis_align", Value::Text("space-between".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    assert_eq!(a.left(), 0.0, "space-between: first flush left");
    assert_eq!(a.right(), 50.0);
    assert_eq!(b.left(), 350.0);
    assert_eq!(b.right(), 400.0, "space-between: last flush right");
}

#[test]
fn row_main_axis_space_around_equal_margin_each() {
    // tight(400,100) + space_around + 2×Patch(50)：extra=300 → offset=75，between_extra=150。
    // a 75..125，b 275..325（每子节点两侧 75 等量空间，中段 150）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("main_axis_align", Value::Text("space_around".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    assert_eq!(a.left(), 75.0, "space-around: left margin = extra/(2n) = 75");
    assert_eq!(a.right(), 125.0);
    assert_eq!(b.left(), 275.0);
    assert_eq!(b.right(), 325.0);
    assert_eq!(b.right() + 75.0, 400.0, "right margin = 75 (symmetric)");
}

#[test]
fn row_main_axis_space_evenly_equal_divisions() {
    // tight(400,100) + space_evenly + 2×Patch(50)：extra=300 → n+1=3 等分，每分 100。
    // a 100..150，b 250..300（首尾与中段均 100）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("main_axis_align", Value::Text("spaceevenly".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    assert_eq!(a.left(), 100.0, "space-evenly: end space = extra/(n+1) = 100");
    assert_eq!(a.right(), 150.0);
    assert_eq!(b.left(), 250.0, "between space = 100");
    assert_eq!(b.right(), 300.0);
}

#[test]
fn row_main_axis_space_between_three_children() {
    // tight(500,100) + space_between + 3×Patch(50)：content=150，extra=350，n=3 →
    // between_extra=350/2=175，spacing=175。a 0..50，b 225..275，c 450..500。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props
        .insert("main_axis_align", Value::Text("space_between".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("green", "b", "app.b"));
    root.children.push(patch("blue", "c", "app.c"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(500.0, 100.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    let c = host.rect_of(&WidgetId::new("c")).unwrap();
    assert_eq!(a.left(), 0.0);
    assert_eq!(b.left(), 225.0, "middle: 50 (a) + 175 (spacing) = 225");
    assert_eq!(c.left(), 450.0, "last: 225 + 50 + 175 = 450");
    assert_eq!(c.right(), 500.0);
}

#[test]
fn row_main_axis_space_between_respects_gap() {
    // tight(400,100) + space_between + gap=20 + 2×Patch(50)：content=100，gaps_min=20，
    // extra=400-100-20=280，spacing=20+280=300。a 0..50，b 350..400。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props
        .insert("main_axis_align", Value::Text("space-between".into()));
    root.props.insert("gap", Value::Float(20.0));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    assert_eq!(a.left(), 0.0);
    assert_eq!(b.left(), 350.0, "50 (a) + gap(20) + extra(280) = 350");
    assert_eq!(b.right(), 400.0);
}

#[test]
fn row_main_axis_space_between_single_child_degenerates_to_start() {
    // n=1：space_between 无间隙可分 → 退化为 Start（offset=0，spacing=gap）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Row");
    root.props
        .insert("main_axis_align", Value::Text("space-between".into()));
    root.children.push(patch("red", "only", "app.only"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let only = host.rect_of(&WidgetId::new("only")).unwrap();
    assert_eq!(only.left(), 0.0, "single child space-between → Start (flush left)");
}

#[test]
fn column_main_axis_space_between_vertical() {
    // tight(100,400) Column + space_between + 2×Patch(50)：extra=300，between_extra=300。
    // a top 0..50，b top 350..400。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.props
        .insert("main_axis_align", Value::Text("space_between".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(100.0, 400.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    let b = host.rect_of(&WidgetId::new("b")).unwrap();
    assert_eq!(a.top(), 0.0, "column space-between: first flush top");
    assert_eq!(b.top(), 350.0);
    assert_eq!(b.bottom(), 400.0, "last flush bottom");
}

#[test]
fn row_main_axis_center_no_effect_when_flex_consumes_space() {
    // tight(400,100) + center + Patch(50,flex=0) + Fill(flex=1)：Fill 消费剩余 350 → free=0
    // → 主轴对齐无可见效果（flex-grow 优先于 justify-content）。Patch 仍在 0..50。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.props.insert("main_axis_align", Value::Text("center".into()));
    root.children.push(patch("red", "fixed", "app.fixed"));
    root.children.push(fill("blue", "flex", 1));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(400.0, 100.0)));
    let fixed = host.rect_of(&WidgetId::new("fixed")).unwrap();
    assert_eq!(
        fixed.left(),
        0.0,
        "flex child consumes free space → justify has no effect"
    );
}

#[test]
fn column_main_axis_center_with_tight_constraint() {
    // tight(100,400) Column + center：容器填满高 400，打包 100，free=300 → 第一 Patch top=150。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.props.insert("main_axis_align", Value::Text("center".into()));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(100.0, 400.0)));
    let a = host.rect_of(&WidgetId::new("a")).unwrap();
    assert_eq!(a.top(), 150.0, "column center: first child top = free/2 = 150");
}

#[test]
fn clip_intersects_parent_and_node_rect() {
    // 父 Column 宽 40，但 Patch 期望 50；measure 给子 max_width=40 → 子填充 40。
    // 验证 clip = viewport ∩ node rect，且不为 None。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(40.0, 40.0)));
    let scene = host.paint().clone();
    let entry = &scene.entries[0];
    assert_eq!(entry.clip, Some(Rect::from_ltrb(0.0, 0.0, 40.0, 40.0)));
}

#[test]
fn theme_changed_paint_only_marks_paint_not_layout() {
    // DC-5 端到端：ThemeProvider 系统主题变化 → ThemeChanged（paint-only）→
    // host.mark(invalidation) → needs_paint 真 / needs_layout 假（字体/间距不变不布局）。
    use crate::ThemeProvider;
    use zero_ui_core::theme::{ColorPalette, ColorSchemePreference, ResolvedColorScheme, SystemThemeSnapshot};

    let sys_light = SystemThemeSnapshot {
        system_scheme: ResolvedColorScheme::Light,
        high_contrast: false,
    };
    let mut provider = ThemeProvider::new(ColorSchemePreference::System, ColorPalette::default(), sys_light);
    let mut host = WidgetHost::new();
    host.set_root(&patch("red", "r", "app.x"));
    // 先完成首次 layout + paint，把初始 NEEDS_LAYOUT/NEEDS_PAINT 清空。
    host.layout(Constraints::loose(Size::new(100.0, 100.0)));
    let _ = host.paint();
    assert!(!host.needs_paint() && !host.needs_layout(), "baseline clean");

    // 系统 Light → Dark：仅颜色变化 → ThemeChanged.invalidation = NEEDS_PAINT。
    let changed = provider
        .on_system_change(SystemThemeSnapshot {
            system_scheme: ResolvedColorScheme::Dark,
            high_contrast: false,
        })
        .expect("scheme changed → ThemeChanged");
    assert!(changed.invalidation.requires_paint());
    assert!(!changed.invalidation.requires_layout());

    host.mark(changed.invalidation);
    assert!(host.needs_paint(), "color-only theme change must request re-paint");
    assert!(
        !host.needs_layout(),
        "color-only theme change must not request re-layout"
    );
}

#[test]
fn tab_cycles_focus_in_declaration_order() {
    // DC-8：Tab 按声明顺序推进焦点（wrap）；Shift-Tab 反向。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    root.children.push(patch("red", "c", "app.c"));
    host.set_root(&root);

    assert!(host.focused_id().is_none(), "no focus initially");
    // Tab → a。
    host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("a")));
    // Tab → b → c → a（wrap）。
    host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("b")));
    host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("c")));
    host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("a")), "forward wraps");
    // Shift-Tab（Backward）从 a → c。
    host.focus_next(zero_ui_core::focus::FocusDirection::Backward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("c")));
}

#[test]
fn tab_key_event_advances_focus() {
    // DC-8：dispatch 收到 Tab 键 → 推进焦点。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);

    let tab = UiEvent::Key {
        code: zero_ui_core::event::KeyCode::new("Tab"),
        action: zero_ui_core::event::KeyAction::Pressed,
        modifiers: zero_ui_core::event::Modifiers::NONE,
        text: None,
    };
    let emitted = host.dispatch_event(&tab);
    assert!(emitted.is_empty(), "Tab does not emit action");
    assert_eq!(host.focused_id(), Some(&WidgetId::new("a")));
    host.dispatch_event(&tab);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("b")));
}

#[test]
fn key_routed_to_focused_widget() {
    // DC-8：非 Tab 键路由到 focused widget；Patch 在聚焦时按 Enter → emit action。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    // 聚焦到 b。
    host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
    host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("b")));

    let enter = UiEvent::Key {
        code: zero_ui_core::event::KeyCode::new("Enter"),
        action: zero_ui_core::event::KeyAction::Pressed,
        modifiers: zero_ui_core::event::Modifiers::NONE,
        text: None,
    };
    let emitted = host.dispatch_event(&enter);
    assert_eq!(emitted.len(), 1);
    assert_eq!(
        emitted[0].action,
        ActionId::new("app.b"),
        "Enter routed to focused widget b"
    );
}

#[test]
fn key_without_focus_is_ignored() {
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);
    // 无焦点时按 Enter → 无 emit。
    let enter = UiEvent::Key {
        code: zero_ui_core::event::KeyCode::new("Enter"),
        action: zero_ui_core::event::KeyAction::Pressed,
        modifiers: zero_ui_core::event::Modifiers::NONE,
        text: None,
    };
    let emitted = host.dispatch_event(&enter);
    assert!(emitted.is_empty(), "key without focus must not emit");
}

#[test]
fn click_focuses_deepest_focusable() {
    // DC-8 phase-2：按下时聚焦命中最深 focusable widget。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));
    host.paint();
    // Column 堆叠：a 在 y≈0..50，b 在 y≈50..100（Patch 50x50）。
    let press = |host: &mut WidgetHost, y: f32| {
        host.dispatch_event(&UiEvent::Pointer {
            phase: PointerPhase::Pressed,
            button: Some(PointerButton::Primary),
            position: Point::new(10.0, y),
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        });
    };
    assert!(host.focused_id().is_none(), "no focus initially");
    press(&mut host, 75.0); // 命中 b
    assert_eq!(host.focused_id(), Some(&WidgetId::new("b")));
    press(&mut host, 25.0); // 命中 a
    assert_eq!(host.focused_id(), Some(&WidgetId::new("a")));
}

#[test]
fn click_empty_space_keeps_focus() {
    // DC-8 phase-2：按下点无 focusable（命中点外）→ 焦点不变。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));
    host.paint();
    host.focus_next(zero_ui_core::focus::FocusDirection::Forward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("a")));
    // 点击根 rect 之外（Column root 仅 ~50x50）。
    host.dispatch_event(&UiEvent::Pointer {
        phase: PointerPhase::Pressed,
        button: Some(PointerButton::Primary),
        position: Point::new(300.0, 300.0),
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    });
    assert_eq!(
        host.focused_id(),
        Some(&WidgetId::new("a")),
        "click in empty space must not steal focus"
    );
}

// 让 `Size::clamp` 在测试里可用：Constraints 已有 is_satisfied，这里给 Size 一个临时裁剪。
trait ClampSize {
    fn clamp(self, c: Constraints) -> Size;
}
impl ClampSize for Size {
    fn clamp(self, c: Constraints) -> Size {
        Size::new(
            self.width.clamp(c.min_width, c.max_width),
            self.height.clamp(c.min_height, c.max_height),
        )
    }
}

// ── DC-8 phase-3：焦点作用域 trap（modal/popup）─────────────────────

#[test]
fn focus_scope_traps_tab_within_subtree() {
    // 树：root Column [a, modal(Column [m1, m2]), c]
    // 进入 modal scope(trap) → Tab 在 m1/m2 间折返，绝不跳到 a/c。
    let mut host = patch_host();
    let mut modal = WidgetSpec::new("Column");
    modal.id = Some(WidgetId::new("modal"));
    modal.children.push(patch("red", "m1", "app.m1"));
    modal.children.push(patch("blue", "m2", "app.m2"));
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(modal);
    root.children.push(patch("red", "c", "app.c"));
    host.set_root(&root);

    host.enter_focus_scope(WidgetId::new("modal"), true);
    assert_eq!(
        host.focused_id(),
        Some(&WidgetId::new("m1")),
        "scope entry focuses first"
    );
    host.focus_next(FocusDirection::Forward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("m2")));
    host.focus_next(FocusDirection::Forward);
    assert_eq!(
        host.focused_id(),
        Some(&WidgetId::new("m1")),
        "trap wraps within scope, never escapes to a/c"
    );
    // Shift-Tab 也折返（m1 → m2）。
    host.focus_next(FocusDirection::Backward);
    assert_eq!(host.focused_id(), Some(&WidgetId::new("m2")));

    // 退出作用域后恢复全局遍历：focused=m2 → Forward 落到 c。
    host.exit_focus_scope();
    host.focus_next(FocusDirection::Forward);
    assert_eq!(
        host.focused_id(),
        Some(&WidgetId::new("c")),
        "global traversal resumes after scope exit"
    );
}

#[test]
fn focus_scope_cleared_when_subtree_removed() {
    // set_root 移除 modal 子树 → 活跃作用域自动清除（不指向已删除节点）。
    let mut host = patch_host();
    let mut modal = WidgetSpec::new("Column");
    modal.id = Some(WidgetId::new("modal"));
    modal.children.push(patch("red", "m1", "app.m1"));
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(modal);
    host.set_root(&root);
    host.enter_focus_scope(WidgetId::new("modal"), true);
    assert!(host.active_focus_scope().is_some());

    // 重建：移除 modal，仅留 a。
    let mut root2 = WidgetSpec::new("Column");
    root2.id = Some(WidgetId::new("root"));
    root2.children.push(patch("red", "a", "app.a"));
    host.set_root(&root2);
    assert!(
        host.active_focus_scope().is_none(),
        "scope rooted at removed subtree must be cleared on reconcile"
    );
}

// ── DC-8 phase-3：SemanticsNode a11y 树 ─────────────────────────────

/// 测试用「链接标签」控件：不可聚焦，推送 LINK 角色 + Literal label 的语义节点。
struct Tag {
    label: CompactString,
}
impl Widget for Tag {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut zero_ui_core::widget::UpdateCtx, _props: &PropsMap) {}
    fn event(&mut self, _ctx: &mut EventCtx, _event: &UiEvent) -> EventResult {
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, _constraints: Constraints) -> Size {
        Size::new(40.0, 20.0)
    }
    fn paint(&mut self, ctx: &mut PaintCtx) {
        ctx.recorder
            .fill_rect(Rect::from_ltrb(0.0, 0.0, 40.0, 20.0), Color::rgb(0.0, 0.0, 0.0));
    }
    fn semantics(&self, ctx: &mut SemanticsCtx) {
        ctx.nodes.push(SemanticsNode {
            id: WidgetId::new(""),
            rect: Rect::ZERO,
            flags: SemanticsFlags::LINK,
            label: Some(SemanticsLabel::Literal(self.label.clone())),
            value: None,
            children: Vec::new(),
        });
    }
}

fn tag_host() -> WidgetHost {
    let mut host = patch_host();
    host.register("Tag", |spec| {
        let label = match spec.props.get("label") {
            Some(Value::Text(s)) => CompactString::from(s.as_str()),
            _ => CompactString::new(""),
        };
        Box::new(Tag { label })
    });
    host
}

fn tag(label: &str, id: &str) -> WidgetSpec {
    let mut s = WidgetSpec::new("Tag");
    s.id = Some(WidgetId::new(id));
    s.props.insert("label", Value::Text(label.into()));
    s
}

/// 在语义树里按 id 串查节点。
fn find_sem<'a>(node: &'a SemanticsNode, id: &str) -> Option<&'a SemanticsNode> {
    if node.id.0.as_str() == id {
        return Some(node);
    }
    for c in &node.children {
        if let Some(n) = find_sem(c, id) {
            return Some(n);
        }
    }
    None
}

#[test]
fn semantics_tree_mirrors_widgets_with_focus_flags() {
    // 树：root Column [a(Patch focusable), b(Patch focusable)]
    // → root 节点（容器合并）children = [a, b]；focus a → a 带 FOCUSABLE|FOCUSED。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    host.focus_next(FocusDirection::Forward); // → a
    assert_eq!(host.focused_id(), Some(&WidgetId::new("a")));

    let sem = host.semantics().unwrap();
    assert_eq!(sem.id, WidgetId::new("root"));
    assert_eq!(sem.children.len(), 2, "root merges its widget children");
    let a = find_sem(&sem, "a").expect("a present");
    assert!(a.flags.contains(SemanticsFlags::FOCUSABLE));
    assert!(a.flags.contains(SemanticsFlags::FOCUSED), "a is focused");
    assert_eq!(
        a.rect,
        Rect::from_ltrb(0.0, 0.0, 50.0, 50.0),
        "absolute rect from layout"
    );
    let b = find_sem(&sem, "b").expect("b present");
    assert!(b.flags.contains(SemanticsFlags::FOCUSABLE));
    assert!(!b.flags.contains(SemanticsFlags::FOCUSED));
}

#[test]
fn semantics_carries_widget_label_and_role() {
    // Tag 推送 LINK + Literal label；经 host semantics 透出，rect 被 host 覆盖为绝对。
    let mut host = tag_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(tag("Learn more", "lnk"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    let sem = host.semantics().unwrap();
    let lnk = find_sem(&sem, "lnk").expect("lnk present");
    assert!(
        lnk.flags.contains(SemanticsFlags::LINK),
        "widget-provided role flows through"
    );
    assert_eq!(
        lnk.label,
        Some(SemanticsLabel::Literal(CompactString::new("Learn more")))
    );
    // host 用绝对 rect 覆盖 widget 推送的 Rect::ZERO。
    assert_eq!(lnk.rect, Rect::from_ltrb(0.0, 0.0, 40.0, 20.0));
}

#[test]
fn semantics_is_none_before_set_root() {
    let host = WidgetHost::new();
    assert!(host.semantics().is_none());
}

// ── DC-2 child min/max constraint props ─────────────────────────────────

#[test]
fn child_min_width_prop_enforces_lower_bound() {
    // min_width=80 on a 40-wide widget → clamped to 80.
    struct Tiny(Size);
    impl Widget for Tiny {
        fn mount(&mut self, _: &mut MountCtx) {}
        fn update(&mut self, _: &mut zero_ui_core::widget::UpdateCtx, _: &PropsMap) {}
        fn event(&mut self, _: &mut EventCtx, _: &UiEvent) -> EventResult {
            EventResult::Ignored
        }
        fn layout(&mut self, _: &mut LayoutCtx, constraints: Constraints) -> Size {
            let s = Size::new(
                self.0.width.min(constraints.max_width),
                self.0.height.min(constraints.max_height),
            );
            self.0 = s;
            s
        }
        fn paint(&mut self, _: &mut PaintCtx) {}
        fn semantics(&self, _: &mut SemanticsCtx) {}
    }

    let mut host = WidgetHost::new();
    host.register("Tiny", |_| Box::new(Tiny(Size::new(40.0, 20.0))));
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    let mut child = WidgetSpec::new("Tiny");
    child.props.insert("min_width", Value::Int(80));
    root.children.push(child);
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));
    let r = host.rect_of(&WidgetId::new("root")).unwrap();
    assert!(
        (r.size.width - 80.0).abs() < 0.1,
        "min_width 80 > measured 40, got {}",
        r.size.width
    );
}

#[test]
fn child_max_width_prop_enforces_upper_bound() {
    // max_width=60 → Fill in loose=400 clamped to 60.
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    let mut child = WidgetSpec::new("Fill");
    child.props.insert("max_width", Value::Int(60));
    root.children.push(child);
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 100.0)));
    let r = host.rect_of(&WidgetId::new("root")).unwrap();
    assert!((r.size.width - 60.0).abs() < 0.1, "max_width=60, got {}", r.size.width);
}

#[test]
fn child_constraints_default_to_noop() {
    // 无 min/max props → 缺省不约束（向后兼容）。
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(WidgetSpec::new("Fill"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(200.0, 100.0)));
    let r = host.rect_of(&WidgetId::new("root")).unwrap();
    assert_eq!(r.size.width, 200.0, "no constraints → Fill fills parent");
}

#[test]
fn flex_child_max_width_clamps_share() {
    // flex=1 in loose(400) with max_width=80 → child clamped to 80.
    let mut host = fill_host();
    let mut root = WidgetSpec::new("Row");
    root.id = Some(WidgetId::new("root"));
    let mut child = WidgetSpec::new("Fill");
    child.props.insert("flex", Value::Int(1));
    child.props.insert("max_width", Value::Int(80));
    root.children.push(child);
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 100.0)));
    let r = host.rect_of(&WidgetId::new("root")).unwrap();
    assert!((r.size.width - 80.0).abs() < 0.1, "flex+max=80, got {}", r.size.width);
}

// ── DC-8 WidgetHost → AccessibilityBackend 自动推送 ────────────────────────
//
// 用共享态后端（Rc<RefCell<记录>>）让测试在 host 内部驱动 Box<dyn AccessibilityBackend>
// 后仍能读取推送记录，无需在 host 暴露后端访问器。

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
enum A11yRec {
    Tree(Option<usize>),
    Focus(Option<String>),
    Announce(String),
}

/// 共享态 a11y 后端：把 update_tree/focus_moved/announce 记录到 Rc<RefCell>，
/// 测试可在 host 拥有 Box<dyn> 的同时读取记录。
struct SharedA11yRec {
    records: Rc<RefCell<Vec<A11yRec>>>,
}

impl AccessibilityBackend for SharedA11yRec {
    fn update_tree(&mut self, root: Option<&SemanticsNode>) {
        let n = root.map(|r| crate::accessibility::node_count(Some(r)));
        self.records.borrow_mut().push(A11yRec::Tree(n));
    }
    fn focus_moved(&mut self, focused: Option<WidgetId>) {
        self.records
            .borrow_mut()
            .push(A11yRec::Focus(focused.map(|f| f.0.to_string())));
    }
    fn announce(&mut self, message: &str) {
        self.records.borrow_mut().push(A11yRec::Announce(message.to_string()));
    }
}

fn shared_backend() -> (SharedA11yRec, Rc<RefCell<Vec<A11yRec>>>) {
    let records = Rc::new(RefCell::new(Vec::new()));
    let backend = SharedA11yRec {
        records: Rc::clone(&records),
    };
    (backend, records)
}

/// root Column [a(focusable), b(focusable)]，已 set_root + layout + 挂载后端（未 flush）。
fn a11y_host_with_backend() -> (WidgetHost, Rc<RefCell<Vec<A11yRec>>>) {
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    root.children.push(patch("blue", "b", "app.b"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));
    let (backend, records) = shared_backend();
    host.set_accessibility_backend(Box::new(backend));
    (host, records)
}

#[test]
fn flush_pushes_full_tree_on_first_flush() {
    // 首次 flush：set_root 标 NEEDS_SEMANTICS → 推送全树（3 节点：root+a+b）。
    // 焦点 None == 初始 last_pushed_focus(None) → 不推 focus_moved（树内 FOCUSED 标志
    // 已表达「无焦点」，平台据此推断；与 Flutter a11y 语义一致）。
    let (mut host, records) = a11y_host_with_backend();
    assert!(host.needs_semantics(), "set_root + attach marked NEEDS_SEMANTICS");
    host.flush_accessibility();
    assert!(!host.needs_semantics(), "flush cleared NEEDS_SEMANTICS");
    assert_eq!(
        *records.borrow(),
        vec![A11yRec::Tree(Some(3))],
        "first flush pushes full tree; no focus_moved when focus unchanged from initial None"
    );
}

#[test]
fn flush_pushes_initial_focus_when_set_before_attach() {
    // 挂载前已聚焦 a → 首次 flush 焦点 Some(a) 与初始 last_pushed_focus(None) 不同
    // → 同时推送全树 + focus_moved(Some(a))。
    let (mut host, records) = a11y_host_with_backend();
    host.focus_next(FocusDirection::Forward); // → a（挂载后、flush 前）
    host.flush_accessibility();
    assert_eq!(
        *records.borrow(),
        vec![A11yRec::Tree(Some(3)), A11yRec::Focus(Some("a".to_string()))],
        "non-None initial focus is pushed on first flush"
    );
}

#[test]
fn flush_pushes_focus_change_without_full_tree_rebuild() {
    // 焦点变化不应重建/重推整棵语义树（廉价，与 Flutter a11y 一致）：
    // 第二次 flush 仅追加 focus_moved(Some(a))，无 Tree 记录。
    let (mut host, records) = a11y_host_with_backend();
    host.flush_accessibility(); // 全树 + Focus(None)
    records.borrow_mut().clear();

    host.focus_next(FocusDirection::Forward); // → a
    assert_eq!(host.focused_id(), Some(&WidgetId::new("a")));
    host.flush_accessibility();
    assert_eq!(
        *records.borrow(),
        vec![A11yRec::Focus(Some("a".to_string()))],
        "focus-only change pushes focus_moved, no tree rebuild"
    );
}

#[test]
fn flush_is_idempotent_when_nothing_changed() {
    // 无结构/焦点变化时再 flush 不产生任何推送。
    let (mut host, records) = a11y_host_with_backend();
    host.flush_accessibility();
    records.borrow_mut().clear();
    host.flush_accessibility();
    host.flush_accessibility();
    assert!(records.borrow().is_empty(), "no-op flush pushes nothing");
}

#[test]
fn set_root_remarks_semantics_for_subsequent_flush() {
    // 第二次 set_root（结构/props 变化）重新标 NEEDS_SEMANTICS → flush 再推全树。
    let (mut host, records) = a11y_host_with_backend();
    host.flush_accessibility();
    records.borrow_mut().clear();

    // 重建同一棵树（结构等价，但 set_root 保守地标 NEEDS_SEMANTICS）。
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);
    assert!(host.needs_semantics());
    host.flush_accessibility();
    // b 被移除 → 树缩为 root+a = 2 节点；焦点 a 仍存在 → 不变，不推 focus。
    assert_eq!(*records.borrow(), vec![A11yRec::Tree(Some(2))]);
}

#[test]
fn flush_without_backend_clears_dirty_without_panic() {
    // 未挂载后端：set_root 标 NEEDS_SEMANTICS；flush 清掉标记、不 panic、不推送。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);
    assert!(host.needs_semantics());
    host.flush_accessibility(); // 无后端
    assert!(!host.needs_semantics(), "dirty flag cleared even without backend");
}

#[test]
fn announce_routes_to_backend_and_is_noop_without_one() {
    // 挂载后端：announce 经 host 路由到后端。
    let (mut host, records) = a11y_host_with_backend();
    host.announce("Page loaded");
    host.announce("Copied");
    assert_eq!(
        records
            .borrow()
            .iter()
            .filter(|r| matches!(r, A11yRec::Announce(_)))
            .count(),
        2,
        "both announcements reached backend"
    );
    // 未挂载后端：announce 为 no-op，不 panic。
    let mut host2 = patch_host();
    host2.set_root(&WidgetSpec::new("Column"));
    host2.announce("nothing happens");
}

#[test]
fn focus_moved_reflects_clearing_focus() {
    // 聚焦 a 后退出作用域/清焦点：flush 推送 focus_moved 反映新焦点。
    let (mut host, records) = a11y_host_with_backend();
    host.flush_accessibility();
    host.focus_next(FocusDirection::Forward); // → a
    host.flush_accessibility();
    records.borrow_mut().clear();

    host.set_focus(WidgetId::new("nonexistent")); // 焦点设到不存在的 id
    host.flush_accessibility();
    assert_eq!(
        *records.borrow(),
        vec![A11yRec::Focus(Some("nonexistent".to_string()))],
        "set_focus change is detected and pushed"
    );
}

// ── DC-15 opt-in GestureArena 接入 dispatch_event ──────────────────────────────
use zero_ui_gestures::{PanRecognizer, TapRecognizer};

fn pointer(phase: PointerPhase, position: Point) -> UiEvent {
    UiEvent::Pointer {
        phase,
        button: Some(PointerButton::Primary),
        position,
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    }
}

/// 同 [`pointer`] 但指定 `pointer_id`（多指手势测试用，DC-15 Pinch）。
fn pointer_id(phase: PointerPhase, position: Point, id: u32) -> UiEvent {
    UiEvent::Pointer {
        phase,
        button: Some(PointerButton::Primary),
        position,
        modifiers: Modifiers::NONE,
        pointer_id: id,
    }
}

#[test]
fn gesture_arena_off_by_default_no_breakage() {
    // 未挂载 arena：dispatch 行为不变，take_gestures 永远空。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));
    assert!(!host.has_gesture_arena());

    // 指针 Released 命中 a → 仍 emit app.a（既有 click 路径不变）。
    let click = pointer(PointerPhase::Released, Point::new(10.0, 10.0));
    let emitted = host.dispatch_event(&click);
    assert_eq!(emitted.len(), 1, "click 路径不受 arena 影响");
    assert!(host.take_gestures().is_empty(), "无 arena 不缓冲手势");
}

#[test]
fn gesture_arena_recognizes_tap_from_pointer_sequence() {
    // 挂载 arena + TapRecognizer：Pressed → Released（同点，无 move）→ Tap 缓冲。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    let mut arena = GestureArena::new();
    arena.push(TapRecognizer::new(8.0));
    host.set_gesture_arena(arena);
    assert!(host.has_gesture_arena());

    let p = Point::new(10.0, 10.0);
    // Pressed 不立即裁决 Tap。
    host.dispatch_event(&pointer(PointerPhase::Pressed, p));
    assert!(host.take_gestures().is_empty(), "down 不立即产出 Tap");
    // Released → Tap。
    host.dispatch_event(&pointer(PointerPhase::Released, p));
    let gestures = host.take_gestures();
    assert_eq!(gestures.len(), 1, "Released 产出 1 个手势");
    assert!(matches!(gestures[0], Gesture::Tap(_)), "识别为 Tap");
    // 再次取空（已 drain）。
    assert!(host.take_gestures().is_empty());
}

#[test]
fn gesture_arena_recognizes_pan_from_move_sequence() {
    // 挂载 arena + PanRecognizer：Pressed → Moved（位移超阈值）→ Pan 缓冲。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    let mut arena = GestureArena::new();
    arena.push(PanRecognizer::with_thresholds(8.0, 1.5));
    host.set_gesture_arena(arena);

    host.dispatch_event(&pointer(PointerPhase::Pressed, Point::new(100.0, 100.0)));
    assert!(host.take_gestures().is_empty(), "down 不产出");
    // 位移 30px 超阈值（8）→ Pan。
    host.dispatch_event(&pointer(PointerPhase::Moved, Point::new(100.0, 130.0)));
    let gestures = host.take_gestures();
    assert_eq!(gestures.len(), 1);
    assert!(matches!(gestures[0], Gesture::Pan { .. }), "识别为 Pan");
}

#[test]
fn gesture_arena_cancel_resets_without_buffering() {
    // Cancelled → arena.reset，不缓冲手势。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    let mut arena = GestureArena::new();
    arena.push(TapRecognizer::new(8.0));
    host.set_gesture_arena(arena);

    host.dispatch_event(&pointer(PointerPhase::Pressed, Point::new(10.0, 10.0)));
    host.dispatch_event(&pointer(PointerPhase::Cancelled, Point::new(10.0, 10.0)));
    assert!(host.take_gestures().is_empty(), "Cancelled reset，不产出 Tap");
}

#[test]
fn gesture_arena_coexists_with_click_dispatch() {
    // arena 挂载时，既有 click-to-focus + emit 仍工作（arena 是 additive，不替代 hit-test）。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(patch("red", "a", "app.a"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    let mut arena = GestureArena::new();
    arena.push(TapRecognizer::new(8.0));
    host.set_gesture_arena(arena);

    let p = Point::new(10.0, 10.0);
    host.dispatch_event(&pointer(PointerPhase::Pressed, p));
    let emitted = host.dispatch_event(&pointer(PointerPhase::Released, p));
    // click 仍 emit app.a（hit-test 不受 arena 影响）。
    assert_eq!(emitted.len(), 1, "click 路径仍 emit");
    assert_eq!(emitted[0].action, ActionId::new("app.a"));
    // 同时 arena 识别出 Tap。
    let gestures = host.take_gestures();
    assert!(matches!(gestures.first(), Some(Gesture::Tap(_))));
}

#[test]
fn gesture_arena_recognizes_pinch_from_two_pointer_sequence() {
    // DC-15 多指：UiEvent::Pointer.pointer_id 区分双指 → PinchRecognizer 仲裁。
    use zero_ui_gestures::PinchRecognizer;
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    let mut arena = GestureArena::new();
    arena.push(PinchRecognizer::new());
    host.set_gesture_arena(arena);

    let p0 = Point::new(100.0, 100.0);
    let p1 = Point::new(200.0, 100.0); // 双指初始距离 100。
    // 第一指 down（id=0）。
    host.dispatch_event(&pointer_id(PointerPhase::Pressed, p0, 0));
    assert!(host.take_gestures().is_empty());
    // 第二指 down（id=1）→ 双指就位，arena 记录初始距离。
    host.dispatch_event(&pointer_id(PointerPhase::Pressed, p1, 1));
    assert!(host.take_gestures().is_empty(), "两指 down 不立即裁决 Pinch");
    // 移动第二指（id=1）到 250 → 距离 150 → scale=1.5 → Won(Pinch)。
    host.dispatch_event(&pointer_id(PointerPhase::Moved, Point::new(250.0, 100.0), 1));
    let gestures = host.take_gestures();
    assert_eq!(gestures.len(), 1, "双指移动产出 1 个 Pinch");
    match gestures[0] {
        Gesture::Pinch { scale, .. } => {
            assert!((scale - 1.5).abs() < 0.01, "pinch scale = 1.5，got {scale}");
        }
        ref g => panic!("识别为 Pinch，got {g:?}"),
    }
}

// ── F1 hover 追踪：PointerPhase::Exited 合成 ─────────────────────────────

/// 测试用「事件记录」叶子控件：把收到的 PointerPhase 写入共享记录。
struct Recorder {
    log: Rc<RefCell<Vec<PointerPhase>>>,
}
impl Widget for Recorder {
    fn mount(&mut self, _ctx: &mut MountCtx) {}
    fn update(&mut self, _ctx: &mut zero_ui_core::widget::UpdateCtx, _props: &PropsMap) {}
    fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
        if let UiEvent::Pointer { phase, .. } = event {
            self.log.borrow_mut().push(*phase);
        }
        EventResult::Ignored
    }
    fn layout(&mut self, _ctx: &mut LayoutCtx, constraints: Constraints) -> Size {
        Size::new(50.0, 50.0).clamp(constraints)
    }
    fn paint(&mut self, _ctx: &mut PaintCtx) {}
    fn semantics(&self, _ctx: &mut SemanticsCtx) {}
}

/// 构造 recorder host + 取出 log 引用。
fn recorder_log(_id: &str) -> (WidgetHost, Rc<RefCell<Vec<PointerPhase>>>) {
    let log = Rc::new(RefCell::new(Vec::new()));
    let log_clone = Rc::clone(&log);
    let mut host = patch_host();
    host.register("Recorder", move |_| {
        Box::new(Recorder {
            log: Rc::clone(&log_clone),
        })
    });
    (host, log)
}

fn recorder_spec(id: &str) -> WidgetSpec {
    let mut s = WidgetSpec::new("Recorder");
    s.id = Some(WidgetId::new(id));
    s
}

#[test]
fn hover_move_between_widgets_dispatches_exited() {
    // F1：指针从 A 移到 B → A 收到 Exited（hover 状态清除），B 收到 Moved。
    let (mut host, log_a) = recorder_log("a");
    let record_b = Rc::new(RefCell::new(Vec::new()));
    let host = &mut host;
    host.register("RecorderB", {
        let log = Rc::clone(&record_b);
        move |_| Box::new(Recorder { log: Rc::clone(&log) })
    });

    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(recorder_spec("a"));
    let mut b_spec = WidgetSpec::new("RecorderB");
    b_spec.id = Some(WidgetId::new("b"));
    root.children.push(b_spec);
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    // Moved 到 a（首次悬停）→ a 收到 Moved，无 Exited（无上一悬停节点）。
    host.dispatch_event(&UiEvent::Pointer {
        phase: PointerPhase::Moved,
        button: None,
        position: Point::new(10.0, 10.0),
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    });
    assert_eq!(*log_a.borrow(), vec![PointerPhase::Moved], "首次 Moved → a 收到 Moved");

    // Moved 到 b（Column 堆叠 Recorder 50x50 → b 在 y≈50..100）→ a 收到 Exited。
    host.dispatch_event(&UiEvent::Pointer {
        phase: PointerPhase::Moved,
        button: None,
        position: Point::new(10.0, 60.0),
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    });
    assert_eq!(
        *log_a.borrow(),
        vec![PointerPhase::Moved, PointerPhase::Exited],
        "指针离开 a → a 收到 Exited"
    );
    assert_eq!(
        *record_b.borrow(),
        vec![PointerPhase::Moved],
        "指针进入 b → b 收到 Moved"
    );
}

#[test]
fn cancelled_dispatches_exited_and_clears_last_hovered() {
    // F1：Cancelled → 悬停节点收到 Exited，last_hovered 清空。
    let (mut host, log) = recorder_log("a");
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("root"));
    root.children.push(recorder_spec("a"));
    host.set_root(&root);
    host.layout(Constraints::loose(Size::new(400.0, 400.0)));

    // 先悬停到 a。
    host.dispatch_event(&UiEvent::Pointer {
        phase: PointerPhase::Moved,
        button: None,
        position: Point::new(10.0, 10.0),
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    });
    log.borrow_mut().clear();

    // Cancelled → a 收到合成 Exited（hover 追踪）+ 原 Cancelled（正常派发）。
    host.dispatch_event(&UiEvent::Pointer {
        phase: PointerPhase::Cancelled,
        button: None,
        position: Point::new(10.0, 10.0),
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    });
    assert_eq!(
        *log.borrow(),
        vec![PointerPhase::Exited, PointerPhase::Cancelled],
        "Cancelled → 悬停节点收到 Exited（hover 追踪）+ Cancelled（正常派发）"
    );

    // 再次 Moved 到 a → 无 Exited（last_hovered 已被 Cancelled 清空）。
    log.borrow_mut().clear();
    host.dispatch_event(&UiEvent::Pointer {
        phase: PointerPhase::Moved,
        button: None,
        position: Point::new(10.0, 10.0),
        modifiers: Modifiers::NONE,
        pointer_id: 0,
    });
    assert_eq!(
        *log.borrow(),
        vec![PointerPhase::Moved],
        "Cancelled 后首次 Moved 不产 Exited（last_hovered=None）"
    );
}

#[test]
fn scroll_vertical_offset_shifts_children_and_clamps() {
    // DC-16：声明 scroll=vertical 的 Column 在收到向下 Wheel 后，
    // 子节点 y 应当上移（scroll_offset>0），且 offset 被 clamp 到 [0, content-viewport]。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("scroller"));
    root.props.insert("scroll", Value::Text("vertical".into()));
    for i in 0..4 {
        let mut child = WidgetSpec::new("Patch");
        child.id = Some(WidgetId::new(&format!("p{i}")));
        child.props.insert("color", Value::Text("red".into()));
        root.children.push(child);
    }
    host.set_root(&root);
    // 视口高度 100，content=4*50=200 → 最大可滚 100。
    host.layout(Constraints::tight(Size::new(200.0, 100.0)));

    // 初始：p0.y=0, p1.y=50, p2.y=100, p3.y=150（后两个超出视口）。
    assert_eq!(host.rect_of(&WidgetId::new("p0")).unwrap().origin.y, 0.0);
    assert_eq!(host.rect_of(&WidgetId::new("p1")).unwrap().origin.y, 50.0);

    // 向下滚 60（delta.y=+60）→ offset=60，所有子节点 y 上移 60。
    host.dispatch_event(&UiEvent::Scroll {
        delta: Vec2::new(0.0, 60.0),
        phase: zero_ui_core::event::ScrollPhase::Discrete,
        position: Point::new(10.0, 10.0),
        modifiers: Modifiers::NONE,
    });
    host.layout(Constraints::tight(Size::new(200.0, 100.0)));
    assert_eq!(
        host.rect_of(&WidgetId::new("p0")).unwrap().origin.y,
        -60.0,
        "向下滚 60 后 p0 应上移到 y=-60（超出视口顶部，将被 clip）"
    );
    assert_eq!(
        host.rect_of(&WidgetId::new("p1")).unwrap().origin.y,
        -10.0,
        "p1 应上移到 y=-10"
    );
    assert_eq!(
        host.rect_of(&WidgetId::new("p2")).unwrap().origin.y,
        40.0,
        "p2 应上移到 y=40（进入视口）"
    );

    // 越界滚：delta.y=+200，期望 clamp 到 max=100，不超界。
    host.dispatch_event(&UiEvent::Scroll {
        delta: Vec2::new(0.0, 200.0),
        phase: zero_ui_core::event::ScrollPhase::Discrete,
        position: Point::new(10.0, 10.0),
        modifiers: Modifiers::NONE,
    });
    host.layout(Constraints::tight(Size::new(200.0, 100.0)));
    assert_eq!(
        host.rect_of(&WidgetId::new("p3")).unwrap().origin.y,
        50.0,
        "越界滚后 offset clamp 到 100，p3 应位于 y=50（content 末尾对齐视口底）"
    );
}

#[test]
fn paint_node_skips_subtree_outside_viewport() {
    // P3-1：滚动到中间时，离屏子节点的整棵子树应被 paint_node early-out 跳过。
    // 验证：滚到 offset=60（p0 在 y=-60..-10，完全在视口 [0,100] 之外）后，
    // scene 中 p0 的 fill_rect 应不存在（否则它会画到视口外、被 SceneRecorder clip 丢弃，
    // 但仍然走了 widget.paint() 全路径）。early-out 让它根本不进 paint 调用。
    let mut host = patch_host();
    let mut root = WidgetSpec::new("Column");
    root.id = Some(WidgetId::new("scroller"));
    root.props.insert("scroll", Value::Text("vertical".into()));
    for i in 0..4 {
        let mut child = WidgetSpec::new("Patch");
        child.id = Some(WidgetId::new(&format!("p{i}")));
        child.props.insert("color", Value::Text("red".into()));
        root.children.push(child);
    }
    host.set_root(&root);
    host.layout(Constraints::tight(Size::new(200.0, 100.0)));

    // 向下滚 60：p0 上移到 y=-60..-10（完全在视口 [0,100] 之外）。
    host.dispatch_event(&UiEvent::Scroll {
        delta: Vec2::new(0.0, 60.0),
        phase: zero_ui_core::event::ScrollPhase::Discrete,
        position: Point::new(10.0, 10.0),
        modifiers: Modifiers::NONE,
    });
    host.layout(Constraints::tight(Size::new(200.0, 100.0)));

    let scene = host.paint().clone();
    let f = fills(&scene);
    // p1（y=-10..40）部分在视口内；p2/p3 完全在视口内；p0 完全离屏。
    // 视口内的 3 个应当画，p0 不画 → scene 里有 3 个 fill，没有 rect 在 (-60,-10) 的那一个。
    assert_eq!(
        f.len(),
        3,
        "p0 完全在视口外，应被 early-out 跳过；剩下 p1/p2/p3 共 3 个 fill"
    );
    assert!(
        !f.iter().any(|(r, _)| (r.origin.y - (-60.0)).abs() < 0.01),
        "scene 中不应包含 p0 的 fill_rect（y=-60）"
    );
}
