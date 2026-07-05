//! winit 运行时驱动器（spec IF-006 `PlatformRuntime` 的**可测试核心**）。
//!
//! [`WinitDriver`] 把「事件 → host dispatch → 收 emit → 应用 reducer → 重建声明树 →
//! 失效 → 帧重布局/重绘」这条编排从**阻塞的** winit `EventLoop::run` 中解耦出来，使其
//! 可在无窗口环境注入事件、断言 scene / 失效行为。真实 `EventLoop::run` 只需：
//!
//! 1. 构造驱动器，经 [`WinitDriver::host_mut`] 注册控件工厂、（可选）挂载手势 arena /
//!    a11y 后端，调 [`WinitDriver::begin`] 产出首帧；
//! 2. 对每个 winit 事件，用 [`event_map`](crate::event_map) 映射成 `UiEvent`，调
//!    [`WinitDriver::pump_event`]；
//! 3. 每帧（vsync / redraw 请求）调 [`WinitDriver::pump_frame`]，并经
//!    [`WinitDriver::host`]`.scene()` 把 Scene 喂给渲染桥（如
//!    `zero-ui-adapter-render-foundation`）。
//!
//! 这样事件循环的「可测试骨架」与「平台阻塞壳」分离——后者需 GUI 才能验证首帧，
//! 前者可完全 headless 验证（见本模块 tests）。

use zero_ui_core::action::ActionResult;
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::Constraints;
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::layout::WindowMetrics;
use zero_ui_core::theme::SemanticTokens;
use zero_ui_runtime::{UiApp, WidgetHost};

/// 一帧的绘制结果（[`WinitDriver::pump_frame`] 返回）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameOutcome {
    /// 本帧无需重绘（invalidation 为空）。
    Idle,
    /// 已重布局/重绘（Scene 已更新，可取回喂给渲染桥）。
    Repainted,
}

/// 一个事件的处理结果（[`WinitDriver::pump_event`] 返回）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventOutcome {
    /// 派发给应用 reducer 的 action 数（host 命中并 emit 的）。
    pub emitted_actions: usize,
    /// 应用 `Handled` 了至少一个 action 并据此重建了声明树（单向数据流：状态变 → rebuild）。
    pub spec_rebuilt: bool,
    /// 是否需要重绘（layout 或 paint 失效）。宿主据此决定是否请求下一帧。
    pub needs_redraw: bool,
}

/// winit 运行时驱动器（DC-2 终端项「真实 winit EventLoop 驱动」的可测试核心）。
///
/// 持有宿主应用（`&mut dyn UiApp`，提供根声明 + action reducer）与 [`WidgetHost`]（retained
/// 运行态：reconcile/layout/paint/事件命中）+ 当前 [`WindowMetrics`]。把示例中重复的
/// 编排（`set_root → layout → paint → dispatch_event → reducer → 重建 → invalidation → frame`）
/// 收敛为一处，作为真实 `EventLoop::run` 的可组合骨架。
pub struct WinitDriver<'app> {
    app: &'app mut dyn UiApp,
    host: WidgetHost,
    metrics: WindowMetrics,
}

impl<'app> WinitDriver<'app> {
    /// 构造驱动器（尚未 reconcile root）：先经 [`host_mut`](Self::host_mut) 注册控件工厂，
    /// 再调 [`begin`](Self::begin) 产出首帧。
    pub fn new(app: &'app mut dyn UiApp, metrics: WindowMetrics) -> WinitDriver<'app> {
        WinitDriver {
            app,
            host: WidgetHost::new(),
            metrics,
        }
    }

    /// 当前窗口度量。
    pub fn metrics(&self) -> &WindowMetrics {
        &self.metrics
    }

    /// 窗口 resize / 度量变化（含 `text_scale`/`density`/`orientation`）：标记需要重布局。
    /// 下一帧 [`pump_frame`](Self::pump_frame) 会按新度量重布局 + 重绘。
    pub fn set_metrics(&mut self, metrics: WindowMetrics) {
        self.metrics = metrics;
        self.host
            .mark(InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT);
    }

    /// 设置主题 semantic token（DC-5：仅色变 → `needs_paint`，不触发布局）。
    /// 字号/间距变化应经度量（`text_scale`/`density`）走 [`set_metrics`](Self::set_metrics)。
    pub fn set_tokens(&mut self, tokens: SemanticTokens) {
        self.host.set_tokens(tokens);
        self.host.mark(InvalidationFlags::NEEDS_PAINT);
    }

    /// 只读访问 host（读 `scene()` / 几何 `rect_of` / a11y 等）。
    pub fn host(&self) -> &WidgetHost {
        &self.host
    }

    /// 可变访问 host（注册控件工厂、挂载手势 arena / a11y 后端等；应在 [`begin`](Self::begin)
    /// 前完成配置）。
    pub fn host_mut(&mut self) -> &mut WidgetHost {
        &mut self.host
    }

    /// 首帧：reconcile 根声明树（`app.root_spec()`）+ 紧约束布局（窗口尺寸）+ 首次 paint。
    /// 控件工厂必须已注册。调用后 invalidation 清空（`pump_frame` 随后返回 [`Idle`](FrameOutcome::Idle)）。
    pub fn begin(&mut self) {
        // P1-6 主题单源：app 通过 theme_tokens() 注入 host 级 tokens，控件 paint 直接读 ctx.tokens。
        if let Some(tokens) = self.app.theme_tokens() {
            self.host.set_tokens(tokens);
        }
        let spec = self.app.root_spec();
        self.host.set_root(&spec);
        self.host.layout(Constraints::tight(self.metrics.logical_size));
        self.host.paint();
    }

    /// 处理一个**已映射好**的 `UiEvent`（来自 winit 原始事件经 [`event_map`](crate::event_map)）。
    ///
    /// 流程：host `dispatch_event`（命中/冒泡/手势 arena/click-to-focus）→ 收集 emit 的
    /// action → 逐个派发给 `app.dispatch` reducer → 任一 `Handled` 即重建声明树并 reconcile
    /// （稳定 `WidgetId` 复用控件实例状态）。返回是否需要重绘。
    pub fn pump_event(&mut self, event: &UiEvent) -> EventOutcome {
        // host 与 app 为不相交字段，顺序借用不冲突；emitted 为 owned Vec，不别名 self。
        let emitted = self.host.dispatch_event(event);
        let count = emitted.len();
        let mut handled = false;
        for a in &emitted {
            if matches!(self.app.dispatch(&a.action, a.payload.clone()), ActionResult::Handled) {
                handled = true;
            }
        }
        if handled {
            // P1-6：状态可能含主题切换 → 重新注入 tokens（即便未变也无害，set_tokens 不失效）。
            if let Some(tokens) = self.app.theme_tokens() {
                self.host.set_tokens(tokens);
            }
            // 状态可能变化 → 重建声明树；reconcile 按稳定 WidgetId 复用，廉价。
            let spec = self.app.root_spec();
            self.host.set_root(&spec);
        }
        EventOutcome {
            emitted_actions: count,
            spec_rebuilt: handled,
            needs_redraw: self.host.needs_layout() || self.host.needs_paint(),
        }
    }

    /// 推进一帧：按 invalidation 标记重布局（needs_layout）+ 重绘（needs_paint）。
    /// 真实 run loop 每帧（vsync / `request_redraw`）调用；无失效时为 [`Idle`](FrameOutcome::Idle)
    /// （不重绘，省 CPU）。重绘后 [`host`](Self::host)`.scene()` 可取回新 Scene 喂给渲染桥。
    pub fn pump_frame(&mut self) -> FrameOutcome {
        // 先捕获是否需要 paint（requires_paint 含 NEEDS_LAYOUT）：layout() 会清除 NEEDS_LAYOUT，
        // 若之后再读 needs_paint()，单独的 NEEDS_LAYOUT（无 NEEDS_PAINT 位，经 host.mark 可达）
        // 会被误判为无需 paint → 几何已变但 scene 不刷新。故先捕获决策。
        let should_paint = self.host.needs_paint();
        if self.host.needs_layout() {
            self.host.layout(Constraints::tight(self.metrics.logical_size));
        }
        if should_paint {
            self.host.paint();
            FrameOutcome::Repainted
        } else {
            FrameOutcome::Idle
        }
    }

    /// 消费驱动器，取出内部的 [`WidgetHost`]（retained 运行态：scene/几何/焦点/a11y）。
    ///
    /// 供「经 driver 建树 + 首帧后检视 host」的场景（如 demo helper：driver 跑 begin 后把 host
    /// 交给测试/打印检视，不再驱动事件）。取出后 host 与 app 的可变借用关系解除。
    pub fn into_host(self) -> WidgetHost {
        self.host
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::action::{ActionId, ActionPayload, EventResult};
    use zero_ui_core::binding::Value;
    use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase};
    use zero_ui_core::geometry::{Point, Rect, Size};
    use zero_ui_core::theme::Color;
    use zero_ui_core::widget::{
        EventCtx, LayoutCtx, MountCtx, PaintCtx, Props, UpdateCtx, Widget, WidgetId, WidgetSpec,
    };

    // ---- 最小可交互控件：ClickBox（按下/抬起在其矩形内 → emit 一个 action） ----

    /// 测试用控件：占位矩形，指针 Released 时 emit 给定 action。
    struct ClickBox {
        action: ActionId,
    }

    impl Widget for ClickBox {
        fn mount(&mut self, _ctx: &mut MountCtx) {}
        fn update(&mut self, _ctx: &mut UpdateCtx, _props: &Props) {}
        fn event(&mut self, _ctx: &mut EventCtx, event: &UiEvent) -> EventResult {
            match event {
                UiEvent::Pointer {
                    phase: PointerPhase::Released,
                    ..
                } => EventResult::Emit(self.action.clone()),
                _ => EventResult::Ignored,
            }
        }
        fn layout(&mut self, _ctx: &mut LayoutCtx, c: Constraints) -> Size {
            // 自然 100×40，但遵守 tight 约束（填满窗口）。
            Size::new(
                100.0_f32.clamp(c.min_width, c.max_width),
                40.0_f32.clamp(c.min_height, c.max_height),
            )
        }
        fn paint(&mut self, ctx: &mut PaintCtx) {
            ctx.recorder.fill_rect(
                Rect::from_origin_size(Point::ZERO, Size::new(100.0, 40.0)),
                Color::rgb(0.2, 0.4, 0.6),
            );
        }
        fn focusable(&self) -> bool {
            true
        }
    }

    // ---- 测试用应用：计数器，Handled "tap.inc" → count+=1 → 重建 spec ----

    struct CounterApp {
        count: i32,
    }

    impl UiApp for CounterApp {
        fn root_spec(&self) -> WidgetSpec {
            let mut spec = WidgetSpec::new("ClickBox");
            spec.id = Some(WidgetId::new("box"));
            spec.props.insert("count", Value::Int(self.count as i64));
            spec
        }
        fn dispatch(&mut self, action: &ActionId, _payload: Option<ActionPayload>) -> ActionResult {
            if action.0.as_str() == "tap.inc" {
                self.count += 1;
                ActionResult::Handled
            } else {
                ActionResult::UnknownAction(action.clone())
            }
        }
    }

    fn tap_at(x: f32, y: f32, phase: PointerPhase) -> UiEvent {
        UiEvent::Pointer {
            phase,
            button: Some(PointerButton::Primary),
            position: Point::new(x, y),
            modifiers: Modifiers::NONE,
            pointer_id: 0,
        }
    }

    fn new_driver(app: &mut CounterApp) -> WinitDriver<'_> {
        let mut d = WinitDriver::new(app, WindowMetrics::tablet());
        d.host_mut().register("ClickBox", |spec| {
            let action = match spec.props.get("action") {
                Some(Value::Text(s)) => ActionId::new(s),
                _ => ActionId::new("tap.inc"),
            };
            Box::new(ClickBox { action })
        });
        d
    }

    #[test]
    fn begin_renders_and_clears_invalidation() {
        let mut app = CounterApp { count: 0 };
        let mut d = new_driver(&mut app);
        d.begin();
        // 首帧后 scene 非空、invalidation 清空 → 紧接一帧应 Idle。
        assert!(!d.host().scene().entries.is_empty(), "begin 产出非空 scene");
        assert_eq!(d.pump_frame(), FrameOutcome::Idle, "begin 后无失效");
    }

    #[test]
    fn pump_event_click_emits_dispatches_and_rebuilds() {
        let mut app = CounterApp { count: 0 };
        // 驱动器持有 &mut app：需在其作用域内完成所有 host/pump 操作，
        // 释放可变借用后再读 app.count（下方断言）。
        {
            let mut d = new_driver(&mut app);
            d.begin();
            // 完整点击：Pressed（无 emit）→ Released（emit "tap.inc"）。
            let out_press = d.pump_event(&tap_at(10.0, 10.0, PointerPhase::Pressed));
            assert_eq!(out_press.emitted_actions, 0, "Pressed 不 emit");
            let out_release = d.pump_event(&tap_at(10.0, 10.0, PointerPhase::Released));
            assert_eq!(out_release.emitted_actions, 1, "Released emit 1 action");
            assert!(out_release.spec_rebuilt, "Handled → 重建声明树");
            assert!(out_release.needs_redraw, "重建 → 需要重绘");
            // pump_frame 落盘：Repainted，随后 Idle（失效已清）。
            assert_eq!(d.pump_frame(), FrameOutcome::Repainted);
            assert_eq!(d.pump_frame(), FrameOutcome::Idle);
        }
        assert_eq!(app.count, 1, "reducer 消费 action，count+1");
    }

    #[test]
    fn unknown_action_does_not_rebuild() {
        // 换一个 emit 未知 action 的 ClickBox（action="tap.unknown"）。
        let mut app = CounterApp { count: 0 };
        {
            let mut d = WinitDriver::new(&mut app, WindowMetrics::tablet());
            d.host_mut().register("ClickBox", |_spec| {
                Box::new(ClickBox {
                    action: ActionId::new("tap.unknown"),
                })
            });
            d.begin();
            let out = d.pump_event(&tap_at(10.0, 10.0, PointerPhase::Released));
            assert_eq!(out.emitted_actions, 1);
            assert!(!out.spec_rebuilt, "UnknownAction → 不重建");
            // dispatch_event 仍可能标 NEEDS_PAINT（hover/pressed 状态），但未重建 spec。
            // 关键断言：spec_rebuilt=false（reducer 未 Handled）。
        }
        assert_eq!(app.count, 0, "未知 action 不改状态");
    }

    #[test]
    fn set_metrics_marks_layout() {
        let mut app = CounterApp { count: 0 };
        let mut d = new_driver(&mut app);
        d.begin();
        assert_eq!(d.pump_frame(), FrameOutcome::Idle);

        // resize：标记 needs_layout（非仅 needs_paint）。
        let mut m = WindowMetrics::tablet();
        m.logical_size = Size::new(500.0, 700.0);
        d.set_metrics(m);
        assert!(d.host().needs_layout(), "resize → needs_layout");
        assert_eq!(d.pump_frame(), FrameOutcome::Repainted, "resize 后重布局+重绘");
        assert_eq!(d.pump_frame(), FrameOutcome::Idle);
    }

    #[test]
    fn set_tokens_marks_paint_not_layout() {
        let mut app = CounterApp { count: 0 };
        let mut d = new_driver(&mut app);
        d.begin();
        assert_eq!(d.pump_frame(), FrameOutcome::Idle);

        // 主题色变：仅 needs_paint，不触发布局（DC-5/DC-9 不变式）。
        d.set_tokens(SemanticTokens::dark());
        assert!(!d.host().needs_layout(), "色变不触发布局");
        assert!(d.host().needs_paint(), "色变 → needs_paint");
        assert_eq!(d.pump_frame(), FrameOutcome::Repainted);
        assert_eq!(d.pump_frame(), FrameOutcome::Idle);
    }

    #[test]
    fn multiple_clicks_accumulate_state() {
        // 多次点击 → 多次 rebuild；稳定 WidgetId 让 ClickBox 实例复用（不重新 mount）。
        let mut app = CounterApp { count: 0 };
        let box_id = WidgetId::new("box");
        {
            let mut d = new_driver(&mut app);
            d.begin();
            let epoch0 = d.host().creation_epoch(&box_id).expect("box mounted");

            for _ in 0..3 {
                d.pump_event(&tap_at(10.0, 10.0, PointerPhase::Pressed));
                d.pump_event(&tap_at(10.0, 10.0, PointerPhase::Released));
                d.pump_frame();
            }
            let epoch_after = d.host().creation_epoch(&box_id).expect("box still mounted");
            assert_eq!(
                epoch0, epoch_after,
                "稳定 WidgetId → 同一实例跨 rebuild 复用（epoch 不变）"
            );
        }
        assert_eq!(app.count, 3, "三次点击 count=3");
    }

    #[test]
    fn pump_frame_paints_when_only_needs_layout_marked() {
        // DC-2/DC-9 robustness 回归守卫：单独 NEEDS_LAYOUT（经 host.mark 可达，无 NEEDS_PAINT 位）
        // 也必须触发 paint——layout 改了几何 → scene 需刷新。此前 pump_frame 在 layout() 清除
        // NEEDS_LAYOUT 后重读 needs_paint()（requires_paint 此时已不含 NEEDS_LAYOUT）会漏 paint
        // （返回 Idle，scene 停留在旧几何）。修复：先捕获 should_paint 决策。
        let mut app = CounterApp { count: 0 };
        let mut d = new_driver(&mut app);
        d.begin();
        assert_eq!(d.pump_frame(), FrameOutcome::Idle);
        // 模拟「仅 layout 失效」（如某 widget 经 ctx.invalidation 标 NEEDS_LAYOUT 但未标 NEEDS_PAINT）。
        d.host_mut().mark(InvalidationFlags::NEEDS_LAYOUT);
        assert_eq!(
            d.pump_frame(),
            FrameOutcome::Repainted,
            "layout-only 失效必须重绘（几何变了）"
        );
        assert_eq!(d.pump_frame(), FrameOutcome::Idle);
    }
}
