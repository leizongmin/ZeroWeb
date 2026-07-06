//! Retained widget host — 三棵树运行态（spec FR-003/FR-004 / DC-2 / DC-9 / DC-14 运行时基础）。
//!
//! 把声明树 `WidgetSpec` reconcile 为 retained **widget 实例树**（`Box<dyn Widget>` + 缓存几何），
//! 驱动 layout（measure 自下而上算尺寸 / arrange 自上而下定 rect）与 paint（遍历树把每节点
//! widget 以局部坐标 paint 进 `SceneRecorder`，按绝对 origin 平移并入全局 `Scene`），
//! 并对带位置的输入事件做 hit-test + 冒泡派发，收集 widget 发出的 `Action`。
//!
//! **依赖方向**：host 不依赖 `ui/widgets`——具体控件由宿主（example / app）通过
//! [`WidgetRegistry`] 注册（`ComponentType` → 工厂闭包）。内置容器布局（Column/Row/Stack）
//! 由 host 自带的 [`ContainerKind`] 处理，是 host 级布局策略，而非叶子控件职责。
//!
//! **模块结构（P0-2 拆分）**：本文件保留公开 API 与 [`WidgetHost`] 主体；reconcile /
//! layout / paint / event / semantics 分别拆到 `host/` 子模块（职责分离，便于阅读与测试）。
//!
//! **本轮范围**：无窗口、单线程、确定性。焦点/键盘事件路由（DC-8）、winit 事件循环接入
//! （`ui/adapters/winit`，M2/M4）与多窗口在后续里程碑；本 host 只负责几何/绘制/指针命中闭环。

mod event;
mod layout;
mod paint;
mod reconcile;
mod semantics;

use crate::accessibility::AccessibilityBackend;
use compact_str::CompactString;
use zero_ui_core::action::ActionPayload;
use zero_ui_core::binding::PropsMap;
use zero_ui_core::event::PointerPhase;
use zero_ui_core::event::UiEvent;
use zero_ui_core::focus::{FocusDirection, FocusScope};
use zero_ui_core::geometry::{Constraints, Point, Rect, Size};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::semantics::SemanticsNode;
use zero_ui_core::widget::{ComponentType, EventCtx, LayoutCtx, Widget, WidgetId, WidgetSpec};
use zero_ui_gestures::{Gesture, GestureArena, PointerEvent};

use event::{
    collect_emit, collect_focusables, deepest_node_at, deepest_scroll_vertical_at, dispatch_focus_event, dispatch_node,
    dispatch_pressed_with_focus, dispatch_to_widget, find_epoch, find_node, find_node_mut, find_rect,
};
use paint::paint_node;
use reconcile::{build_node, reconcile_node};
use semantics::{build_semantics, self_semantics};

// layout 子模块的 measure / arrange 仅由 WidgetHost::layout 直接调用。
use layout::{arrange, measure};

/// Widget 工厂闭包：从 `WidgetSpec` 构造一个具体控件实例。
pub type WidgetFactory = Box<dyn Fn(&WidgetSpec) -> Box<dyn Widget>>;

/// `ComponentType` → 工厂的注册表。
///
/// 宿主（example / app）注册具体控件工厂；host 本身保持浏览器无关、不依赖 `ui/widgets`。
#[derive(Default)]
pub struct WidgetRegistry {
    factories: hashbrown::HashMap<CompactString, WidgetFactory>,
}

impl WidgetRegistry {
    pub fn new() -> WidgetRegistry {
        WidgetRegistry::default()
    }

    /// 注册一个组件工厂。
    pub fn register<F>(&mut self, component: &str, factory: F) -> &mut WidgetRegistry
    where
        F: Fn(&WidgetSpec) -> Box<dyn Widget> + 'static,
    {
        self.factories.insert(CompactString::new(component), Box::new(factory));
        self
    }

    pub(super) fn build(&self, spec: &WidgetSpec) -> Option<Box<dyn Widget>> {
        self.factories.get(spec.component.0.as_str()).map(|f| f(spec))
    }
}

/// 内置容器布局种类（host 级布局策略）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    /// 垂直堆叠（主轴 = Y）。
    Column,
    /// 水平堆叠（主轴 = X）。
    Row,
    /// 叠放（全部置于同一起点，后绘制在上层）。
    Stack,
    /// 垂直滚动容器（DC-16）。
    ///
    /// 与 `Column` 的差异：
    /// - measure：子节点 max_main 放开为 `f32::MAX`，让 content 自然超出 viewport
    /// - arrange：按 `HostNode::scroll_offset` 上移子节点 y，超视口部分由 clip 链裁掉
    /// - 事件：host 收到 Wheel 时累加 `scroll_offset`，clamp 到 `[0, content-viewport]`
    ///
    /// 滚动状态（offset / content_height）目前挂在 `HostNode` 上作为 host 级 layout state。
    /// 未来若需 widget 自管（如编程式 scroll_to），可迁移到独立 `ScrollView` widget。
    ScrollVertical,
}

impl ContainerKind {
    /// 由组件类型名识别内置容器；非容器返回 `None`（交给已注册 widget 或填充策略）。
    pub fn from_component(component: &ComponentType) -> Option<ContainerKind> {
        match component.0.as_str() {
            "Column" => Some(ContainerKind::Column),
            "Row" => Some(ContainerKind::Row),
            "Stack" => Some(ContainerKind::Stack),
            "ScrollVertical" | "ScrollView" => Some(ContainerKind::ScrollVertical),
            _ => None,
        }
    }
}

/// host 派发事件后收集到的、widget 发出的 action（单向数据流：交给应用 reducer）。
#[derive(Debug, Clone, PartialEq)]
pub struct EmittedAction {
    pub action: zero_ui_core::action::ActionId,
    pub payload: Option<ActionPayload>,
}

/// 单个 widget 实例的 retained 运行态节点。
///
/// 持有 `Box<dyn Widget>`（实例 + 临时状态如 hover/pressed）、上次应用的 props、
/// 缓存几何（measure 算 size / arrange 定 rect）与节点级失效标记。
///
/// **可见性**（P0-2）：字段 `pub(super)` 仅对 `host::` 子模块暴露（reconcile / layout /
/// paint / event / semantics 需直接读写），外部 crate 经 [`WidgetHost`] 公开 API 访问。
#[derive(Default)]
pub(super) struct HostNode {
    pub(super) id: WidgetId,
    pub(super) component: ComponentType,
    pub(super) props: PropsMap,
    pub(super) widget: Option<Box<dyn Widget>>,
    pub(super) children: Vec<HostNode>,
    pub(super) cached_size: Size,
    pub(super) cached_rect: Rect,
    pub(super) invalidation: InvalidationFlags,
    pub(super) epoch: u32,
    pub(super) focusable: bool,
    /// 垂直滚动偏移（DC-16 gallery scroll）。仅当 `props["scroll"] == "vertical"` 时生效；
    /// arrange 阶段按此值上移子节点 y，超出视口的子节点由 paint 阶段的 clip 链自然裁掉。
    /// content_height 在 measure 后写入（children 主轴尺寸之和），用来钳 scroll_offset 上界。
    pub(super) scroll_offset: f32,
    pub(super) content_height: f32,
}

/// Retained widget host —— 三棵树运行态的驱动器。
pub struct WidgetHost {
    registry: WidgetRegistry,
    root: Option<HostNode>,
    scene: zero_ui_render::Scene,
    epoch: u32,
    pending: InvalidationFlags,
    focused: Option<WidgetId>,
    /// 活跃焦点作用域（modal/popup trap，DC-8 phase-3）。
    active_scope: Option<FocusScope>,
    /// 当前主题 semantic token（DC-5：paint 时注入 `PaintCtx.tokens`，控件消费而非硬编码）。
    tokens: zero_ui_core::theme::SemanticTokens,
    /// 已挂载的平台无障碍后端（DC-8 平台桥接）。
    a11y_backend: Option<Box<dyn AccessibilityBackend>>,
    /// 上次推送给 a11y 后端的焦点 id。
    last_pushed_focus: Option<WidgetId>,
    /// DC-15 opt-in 手势 arena。
    gesture_arena: Option<GestureArena>,
    pending_gestures: Vec<Gesture>,
    gesture_clock: i64,
    /// 上次指针事件命中的最深节点 id（hover 追踪，DC-8/F1）。
    last_hovered: Option<WidgetId>,
    /// 可供 widgets 查询的实时字体度量 `(ascent, descent)`（DC-11 text path 统一）。
    font_metrics: Option<(f32, f32)>,
    /// 文本度量后端（P1-5）：由 driver 从 FontdueBackend 注入，layout 时放进 LayoutCtx。
    /// 让 widget 在 layout 阶段调 [`LayoutCtx::measure_text`](zero_ui_core::widget::LayoutCtx::measure_text)
    /// 拿到真实文本宽度，替代 `chars * 9` 估算。
    text_measure: Option<Box<dyn zero_ui_core::widget::TextMeasure>>,
    /// P3-4-3：浮层管理（modal/popover/tooltip/sheet + dismiss 策略）。
    /// 应用层通过 [`Self::show_overlay`] / [`Self::dismiss_overlay`] 维护。
    overlay: zero_ui_overlay::OverlayHost,
    /// P3-4-3：浮层视觉子树（独立于主 root；paint 时主树完成后再 paint 此树并 append 到 scene，
    /// 确保浮层视觉在主树之上）。每个 OverlayEntry 对应一个 widget 子树，按 id 索引。
    /// 简化方案：当前只支持单个 overlay 子树（最上层 entry）；多 entry 嵌套后续扩展。
    overlay_root: Option<HostNode>,
    /// P3-4-3：overlay 子树是否需要 reconcile（应用层 set_overlay_spec 后置 true）。
    overlay_dirty: bool,
    /// P3-4-5：动画时钟（毫秒，自 host 启动）。driver 在 pump_frame 时推进。
    animation_now_ms: i64,
    /// P3-4-5：上次 paint 期间 widget 累计的 request_frame 调用数。
    /// 非 0 表示有未完成动画，driver 应继续 pump_frame。
    last_frame_requests: u64,
}

impl Default for WidgetHost {
    fn default() -> WidgetHost {
        WidgetHost {
            registry: WidgetRegistry::new(),
            root: None,
            scene: zero_ui_render::Scene::new(),
            epoch: 0,
            pending: InvalidationFlags::CLEAN,
            focused: None,
            active_scope: None,
            tokens: zero_ui_core::theme::SemanticTokens::light(),
            a11y_backend: None,
            last_pushed_focus: None,
            gesture_arena: None,
            pending_gestures: Vec::new(),
            gesture_clock: 0,
            last_hovered: None,
            font_metrics: None,
            text_measure: None,
            overlay: zero_ui_overlay::OverlayHost::new(),
            overlay_root: None,
            overlay_dirty: false,
            animation_now_ms: 0,
            last_frame_requests: 0,
        }
    }
}

impl WidgetHost {
    pub fn new() -> WidgetHost {
        WidgetHost::default()
    }

    /// 注册一个组件工厂（builder 便捷方法）。
    pub fn register<F>(&mut self, component: &str, factory: F) -> &mut WidgetHost
    where
        F: Fn(&WidgetSpec) -> Box<dyn Widget> + 'static,
    {
        self.registry.register(component, factory);
        self
    }

    /// 设置当前主题 semantic token（DC-5）：控件 paint 时经 `PaintCtx.tokens` 消费。
    pub fn set_tokens(&mut self, tokens: zero_ui_core::theme::SemanticTokens) {
        self.tokens = tokens;
    }

    /// 设置实时字体度量 `(ascent, descent)`，由应用层从共享 `FontdueBackend` 查询后注入。
    pub fn set_font_metrics(&mut self, ascent: f32, descent: f32) {
        self.font_metrics = Some((ascent, descent));
    }

    /// 注入文本度量后端（P1-5）：layout 时放进 `LayoutCtx.text_measure`，让 widget
    /// 在 layout 阶段调 [`LayoutCtx::measure_text`](zero_ui_core::widget::LayoutCtx::measure_text)
    /// 拿到真实文本宽度。
    pub fn set_text_measure(&mut self, tm: Box<dyn zero_ui_core::widget::TextMeasure>) {
        self.text_measure = Some(tm);
    }

    /// 用新声明树 reconcile：按 `WidgetId` + `ComponentType` 复用既有 widget 实例（保留临时状态），
    /// props 变化时调 `Widget::update`；新增节点 mount。结构/props 变化触发 layout+paint 失效。
    pub fn set_root(&mut self, spec: &WidgetSpec) {
        self.epoch = self.epoch.wrapping_add(1);
        match &mut self.root {
            Some(root) => {
                reconcile_node(root, spec, &self.registry, self.epoch);
                self.pending |= InvalidationFlags::NEEDS_LAYOUT
                    | InvalidationFlags::NEEDS_PAINT
                    | InvalidationFlags::NEEDS_SEMANTICS;
            }
            None => {
                self.root = Some(build_node(spec, &self.registry, self.epoch));
                self.pending |= InvalidationFlags::NEEDS_LAYOUT
                    | InvalidationFlags::NEEDS_PAINT
                    | InvalidationFlags::NEEDS_SEMANTICS;
            }
        }
        // 活跃焦点作用域：reconcile 后子树可能变化，按 scope.id 重新收集可聚焦项。
        if let Some(scope) = self.active_scope.take() {
            let refreshed = self.root.as_ref().and_then(|root| {
                let node = find_node(root, &scope.id)?;
                let mut focusables = Vec::new();
                collect_focusables(node, &mut focusables);
                Some(FocusScope {
                    id: scope.id,
                    focusables,
                    trap: scope.trap,
                })
            });
            self.active_scope = refreshed;
        }
    }

    /// 标记外部失效（主题/locale 变化等）。
    pub fn mark(&mut self, flags: InvalidationFlags) {
        self.pending |= flags;
    }

    pub fn needs_layout(&self) -> bool {
        self.pending.requires_layout()
    }

    pub fn needs_paint(&self) -> bool {
        self.pending.requires_paint()
    }

    pub fn needs_semantics(&self) -> bool {
        self.pending.contains(InvalidationFlags::NEEDS_SEMANTICS)
    }

    /// 挂载平台无障碍后端（DC-8 平台桥接，spec FR-011）。
    pub fn set_accessibility_backend(&mut self, backend: Box<dyn AccessibilityBackend>) {
        self.a11y_backend = Some(backend);
        self.pending |= InvalidationFlags::NEEDS_SEMANTICS;
    }

    pub fn flush_accessibility(&mut self) {
        let dirty = self.pending.contains(InvalidationFlags::NEEDS_SEMANTICS);
        let tree = if dirty { self.semantics() } else { None };

        let Some(backend) = self.a11y_backend.as_mut() else {
            self.pending.remove(InvalidationFlags::NEEDS_SEMANTICS);
            return;
        };
        if dirty {
            backend.update_tree(tree.as_ref());
            self.pending.remove(InvalidationFlags::NEEDS_SEMANTICS);
        }
        if self.focused != self.last_pushed_focus {
            backend.focus_moved(self.focused.clone());
            self.last_pushed_focus = self.focused.clone();
        }
    }

    pub fn announce(&mut self, message: &str) {
        if let Some(backend) = self.a11y_backend.as_mut() {
            backend.announce(message);
        }
    }

    /// 挂载手势 arena（DC-15 移动端 / 触摸 skeleton，opt-in）。
    pub fn set_gesture_arena(&mut self, arena: GestureArena) {
        self.gesture_arena = Some(arena);
    }

    pub fn has_gesture_arena(&self) -> bool {
        self.gesture_arena.is_some()
    }

    pub fn take_gestures(&mut self) -> Vec<Gesture> {
        std::mem::take(&mut self.pending_gestures)
    }

    /// 两遍布局：measure + arrange。返回根尺寸。
    pub fn layout(&mut self, constraints: Constraints) -> Size {
        // P1-5：先 take text_measure，避免与 &mut self.root 冲突（layout 完放回）。
        let tm = self.text_measure.take();
        let tm_ref: Option<&dyn zero_ui_core::widget::TextMeasure> = tm.as_ref().map(|b| b.as_ref());
        let size = {
            let Some(root) = self.root.as_mut() else {
                self.text_measure = tm;
                return Size::ZERO;
            };
            let mut lctx = LayoutCtx {
                scale_factor: 1.0,
                text_measure: tm_ref,
                font_metrics: self.font_metrics,
            };
            let size = measure(root, &mut lctx, constraints);
            arrange(root, Point::ZERO);
            size
        };
        // P3-4-3：overlay 子树也 layout（用同一 viewport 约束；锚定由 widget 内部处理）。
        if let Some(overlay_root) = self.overlay_root.as_mut() {
            let mut lctx = LayoutCtx {
                scale_factor: 1.0,
                text_measure: tm_ref,
                font_metrics: self.font_metrics,
            };
            measure(overlay_root, &mut lctx, constraints);
            arrange(overlay_root, Point::ZERO);
        }
        self.text_measure = tm;
        self.pending.remove(InvalidationFlags::NEEDS_LAYOUT);
        size
    }

    /// 遍历 widget 实例树 paint 进全局 `Scene`。
    pub fn paint(&mut self) -> &zero_ui_render::Scene {
        self.scene = zero_ui_render::Scene::new();
        // P3-4-5：每帧重置 frame_requests 计数；widget paint 时若调 ctx.request_frame()
        // 会把此 Cell 递增。paint 完读值决定是否需要下一帧。
        let frame_requests_cell = std::cell::Cell::new(0u64);
        if let Some(root) = self.root.as_mut() {
            let viewport = Some(root.cached_rect);
            paint_node(
                root,
                &mut self.scene,
                viewport,
                &self.tokens,
                self.font_metrics,
                Some(self.animation_now_ms),
                &frame_requests_cell,
            );
        }
        // P3-4-3：overlay 子树 paint 在主树之后（append 到 scene = 视觉在上层）。
        if let Some(overlay_root) = self.overlay_root.as_mut() {
            let viewport = overlay_root.cached_rect;
            paint_node(
                overlay_root,
                &mut self.scene,
                Some(viewport),
                &self.tokens,
                self.font_metrics,
                Some(self.animation_now_ms),
                &frame_requests_cell,
            );
        }
        self.last_frame_requests = frame_requests_cell.get();
        self.pending = InvalidationFlags::CLEAN;
        &self.scene
    }

    pub fn scene(&self) -> &zero_ui_render::Scene {
        &self.scene
    }

    pub fn rect_of(&self, id: &WidgetId) -> Option<Rect> {
        self.root.as_ref().and_then(|r| find_rect(r, id))
    }

    pub fn creation_epoch(&self, id: &WidgetId) -> Option<u32> {
        self.root.as_ref().and_then(|r| find_epoch(r, id))
    }

    /// 派发输入事件：位置事件（指针/滚动）走 hit-test + 冒泡；键盘事件走焦点路由（DC-8）。
    pub fn dispatch_event(&mut self, event: &UiEvent) -> Vec<EmittedAction> {
        let mut emitted = Vec::new();
        // P3-4-4：overlay 优先吃事件。
        // - outside-click（Pressed 落在所有 popover 锚定矩形之外）→ dismiss 最上层候选
        // - Escape 键 → dismiss 最上层 escape-able entry
        // - modal barrier 存在时 → 主树事件路由完全屏蔽（点哪都不命中下层）
        if let UiEvent::Pointer {
            phase: PointerPhase::Pressed,
            position,
            ..
        } = event
        {
            let dismissed = self.overlay.dismiss_on_outside_click(*position);
            if !dismissed.is_empty() {
                // 清掉被 dismiss 的 overlay 视觉子树。
                self.overlay_root = None;
                self.pending |=
                    InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
                // outside-click 触发的 dismiss 视为"消费"该点击，不再冒泡到下层。
                return emitted;
            }
        }
        if let UiEvent::Key { code, action, .. } = event
            && code.0.as_str() == "Escape"
            && matches!(action, zero_ui_core::event::KeyAction::Pressed)
        {
            let dismissed = self.overlay.dismiss_on_escape();
            if !dismissed.is_empty() {
                self.overlay_root = None;
                self.pending |=
                    InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
                return emitted;
            }
        }
        // modal barrier：主树完全不接收任何事件。
        let overlay_blocked_main = self.has_modal();
        // DC-15 opt-in 手势 arena。
        if self.gesture_arena.is_some() {
            let recognized = if let UiEvent::Pointer {
                phase,
                position,
                pointer_id,
                ..
            } = event
            {
                let arena = self.gesture_arena.as_mut().expect("checked Some above");
                self.gesture_clock = self.gesture_clock.saturating_add(16);
                let ts = self.gesture_clock;
                match phase {
                    PointerPhase::Pressed => arena.route(&PointerEvent::down(*pointer_id, *position, ts)),
                    PointerPhase::Moved => arena.route(&PointerEvent::move_(*pointer_id, *position, ts)),
                    PointerPhase::Released => arena.route(&PointerEvent::up(*pointer_id, *position, ts)),
                    PointerPhase::Cancelled => {
                        arena.reset();
                        None
                    }
                    PointerPhase::Exited => None,
                }
            } else {
                None
            };
            if let Some(g) = recognized {
                self.pending_gestures.push(g);
                self.pending |= InvalidationFlags::NEEDS_PAINT;
            }
        }
        // P3-7/U1：Tab 键焦点切换提前到 root 借用之前做（focus_next 需 &mut self），
        // 派发 Focus event 延迟到 root 借用之后（需要 root 走树）。
        let mut pending_focus_change: Option<(Option<WidgetId>, Option<WidgetId>)> = None;
        if let UiEvent::Key { code, action: key_action, modifiers, .. } = event
            && code.0.as_str() == "Tab"
            && matches!(key_action, zero_ui_core::event::KeyAction::Pressed)
            && !overlay_blocked_main
        {
            let dir = if modifiers.contains(zero_ui_core::event::Modifiers::SHIFT) {
                FocusDirection::Backward
            } else {
                FocusDirection::Forward
            };
            let old = self.focused.clone();
            self.focus_next(dir);
            self.pending |= InvalidationFlags::NEEDS_PAINT;
            if self.focused != old {
                pending_focus_change = Some((old, self.focused.clone()));
            }
            // Tab 键消费事件，提前 return（但先派发 focus event）
        }

        let Some(root) = self.root.as_mut() else {
            return emitted;
        };
        // P3-7/U1：派发延迟的 Focus event（Tab 切换产生）。
        if let Some((old, new)) = pending_focus_change.take() {
            if let Some(old_id) = old {
                dispatch_focus_event(root, &old_id, UiEvent::Focus(zero_ui_core::event::FocusEvent::Lost));
            }
            if let Some(new_id) = new {
                dispatch_focus_event(root, &new_id, UiEvent::Focus(zero_ui_core::event::FocusEvent::Gained));
            }
            return emitted;  // Tab 键消费，不继续派发
        }
        // P3-4-4：modal barrier / outside-click dismiss 已消费事件 → 主树不路由。
        if overlay_blocked_main {
            return emitted;
        }
        // F1 hover 追踪：在 Pressed/Moved 时检测悬停节点变化，合成为离开的 `Exited` 事件。
        if let UiEvent::Pointer { phase, position, .. } = event {
            match phase {
                PointerPhase::Pressed | PointerPhase::Moved => {
                    let new_target = deepest_node_at(root, *position);
                    if self.last_hovered != new_target {
                        if self.last_hovered.is_some() {
                            dispatch_to_widget(
                                root,
                                self.last_hovered.as_ref().unwrap(),
                                PointerPhase::Exited,
                                &mut emitted,
                            );
                        }
                        self.last_hovered = new_target;
                        emitted.clear();
                        if self.last_hovered.is_some() {
                            self.pending |= InvalidationFlags::NEEDS_PAINT;
                        }
                    }
                }
                PointerPhase::Cancelled => {
                    if let Some(ref old_id) = self.last_hovered {
                        dispatch_to_widget(root, old_id, PointerPhase::Exited, &mut emitted);
                    }
                    self.last_hovered = None;
                    emitted.clear();
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                }
                _ => {}
            }
        }
        match event {
            UiEvent::Key { .. } => {
                if let Some(focused) = self.focused.clone()
                    && let Some(node) = find_node_mut(root, &focused)
                    && let Some(w) = node.widget.as_mut()
                {
                    let mut flags = InvalidationFlags::CLEAN;
                    let res = w.event(
                        &mut EventCtx {
                            invalidation: &mut flags,
                        },
                        event,
                    );
                    node.invalidation |= flags;
                    collect_emit(res, &mut emitted);
                }
                if !emitted.is_empty() {
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                }
            }
            UiEvent::Pointer {
                phase: PointerPhase::Pressed,
                position: _,
                ..
            } => {
                // P2-7：合并 hit-test + focus 收集为单次遍历（dispatch_pressed_with_focus）。
                let (handled, target) = dispatch_pressed_with_focus(root, event, &mut emitted);
                if let Some(id) = target
                    && self.focused.as_ref() != Some(&id)
                {
                    let old = self.focused.clone();
                    self.focused = Some(id.clone());
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                    // P3-7/U1：派发 Focus event，让 TextInput 显示光标。
                    if let Some(old_id) = old {
                        dispatch_focus_event(root, &old_id, UiEvent::Focus(zero_ui_core::event::FocusEvent::Lost));
                    }
                    dispatch_focus_event(root, &id, UiEvent::Focus(zero_ui_core::event::FocusEvent::Gained));
                }
                if handled {
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                }
            }
            UiEvent::Scroll { delta, position, .. } => {
                if delta.y != 0.0
                    && let Some(target_id) = deepest_scroll_vertical_at(root, *position)
                    && let Some(node) = find_node_mut(root, &target_id)
                {
                    let viewport = node.cached_rect.size.height;
                    let max_scroll = (node.content_height - viewport).max(0.0);
                    let new_offset = (node.scroll_offset + delta.y).clamp(0.0, max_scroll);
                    if new_offset != node.scroll_offset {
                        node.scroll_offset = new_offset;
                        self.pending |= InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
                    }
                    return emitted;
                }
                let _ = dispatch_node(root, event, &mut emitted);
            }
            _ => {
                let handled = dispatch_node(root, event, &mut emitted);
                if handled {
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                }
            }
        }
        emitted
    }

    pub fn ime_rect(&self) -> Option<Rect> {
        let root = self.root.as_ref()?;
        let focused = self.focused.as_ref()?;
        let node = find_node(root, focused)?;
        let w = node.widget.as_ref()?;
        let local = w.ime_rect()?;
        Some(Rect::from_origin_size(
            Point::new(
                node.cached_rect.origin.x + local.origin.x,
                node.cached_rect.origin.y + local.origin.y,
            ),
            local.size,
        ))
    }

    pub fn focused_id(&self) -> Option<&WidgetId> {
        self.focused.as_ref()
    }

    pub fn set_focus(&mut self, id: WidgetId) {
        self.focused = Some(id);
        self.pending |= InvalidationFlags::NEEDS_PAINT;
    }

    pub fn focus_next(&mut self, dir: FocusDirection) {
        if let Some(scope) = self.active_scope.clone() {
            if scope.focusables.is_empty() {
                self.focused = None;
                return;
            }
            match scope.next(self.focused.as_ref(), dir) {
                Some(id) => {
                    let new_focus = id.clone();
                    let changed = self.focused.as_ref() != Some(&new_focus);
                    self.focused = Some(new_focus);
                    if changed {
                        self.pending |= InvalidationFlags::NEEDS_PAINT;
                    }
                    return;
                }
                None => {
                    self.active_scope = None;
                }
            }
        }
        let Some(root) = self.root.as_ref() else {
            return;
        };
        let mut focusables: Vec<WidgetId> = Vec::new();
        collect_focusables(root, &mut focusables);
        if focusables.is_empty() {
            self.focused = None;
            return;
        }
        let idx = self
            .focused
            .as_ref()
            .and_then(|f| focusables.iter().position(|x| x == f));
        let len = focusables.len();
        let next_idx = match (dir, idx) {
            (FocusDirection::Forward, Some(i)) => (i + 1) % len,
            (FocusDirection::Forward, None) => 0,
            (FocusDirection::Backward, Some(i)) => (i + len - 1) % len,
            (FocusDirection::Backward, None) => len - 1,
        };
        let new_focus = focusables[next_idx].clone();
        if self.focused.as_ref() != Some(&new_focus) {
            self.focused = Some(new_focus);
            self.pending |= InvalidationFlags::NEEDS_PAINT;
        }
    }

    pub fn enter_focus_scope(&mut self, scope_root: WidgetId, trap: bool) {
        let scope = self.root.as_ref().and_then(|root| {
            let node = find_node(root, &scope_root)?;
            let mut focusables = Vec::new();
            collect_focusables(node, &mut focusables);
            Some(FocusScope {
                id: scope_root.clone(),
                focusables,
                trap,
            })
        });
        let Some(scope) = scope else {
            return;
        };
        let first = scope.focusables.first().cloned();
        self.active_scope = Some(scope);
        if let Some(first) = first {
            self.focused = Some(first);
            self.pending |= InvalidationFlags::NEEDS_PAINT;
        }
    }

    pub fn exit_focus_scope(&mut self) {
        if self.active_scope.take().is_some() {
            self.pending |= InvalidationFlags::NEEDS_PAINT;
        }
    }

    pub fn active_focus_scope(&self) -> Option<&FocusScope> {
        self.active_scope.as_ref()
    }

    /// 构建无障碍语义树（spec FR-011 / DC-8 phase-3）。
    pub fn semantics(&self) -> Option<SemanticsNode> {
        let root = self.root.as_ref()?;
        let mut s = self_semantics(root, self.focused.as_ref());
        s.children.clear();
        for child in &root.children {
            build_semantics(child, self.focused.as_ref(), &mut s.children);
        }
        Some(s)
    }

    // ── P3-4-3/4 浮层 API ──────────────────────────────────────────────────

    /// 浮层管理器不可变借用（应用层查询 has_modal / top_id 等）。
    pub fn overlay(&self) -> &zero_ui_overlay::OverlayHost {
        &self.overlay
    }

    /// 显示一个浮层：注册 entry 到 OverlayHost，并（可选）设置该浮层的视觉子树 spec。
    ///
    /// `spec` = 浮层内容（dialog body / popover card / tooltip bubble 等）。
    /// 传 `None` 表示只注册 entry 不更新视觉子树（例如外部已通过主 root 管理视觉）。
    pub fn show_overlay(
        &mut self,
        entry: zero_ui_overlay::OverlayEntry,
        spec: Option<WidgetSpec>,
    ) -> WidgetId {
        let id = entry.id.clone();
        self.overlay.show(entry);
        if let Some(spec) = spec {
            self.epoch = self.epoch.wrapping_add(1);
            match &mut self.overlay_root {
                Some(root) => reconcile_node(root, &spec, &self.registry, self.epoch),
                None => self.overlay_root = Some(build_node(&spec, &self.registry, self.epoch)),
            }
            self.overlay_dirty = false;
            self.pending |= InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
        }
        id
    }

    /// 移除一个浮层（按 id）。若当前 overlay_root 对应被移除的 entry，清空 overlay_root。
    pub fn dismiss_overlay(&mut self, id: &WidgetId) -> bool {
        let removed = self.overlay.dismiss(id);
        if removed {
            // 简化方案：单 overlay_root，任何 dismiss 都清空视觉子树（应用层若有多 entry 应重新 show 剩余）。
            self.overlay_root = None;
            self.pending |= InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
        }
        removed
    }

    /// overlay 子树是否参与 layout/paint。
    pub fn has_overlay_visual(&self) -> bool {
        self.overlay_root.is_some()
    }

    /// 当前是否存在任意 modal barrier 浮层（事件路由屏蔽下层）。
    pub fn has_modal(&self) -> bool {
        self.overlay.has_modal()
    }

    /// 浮层根节点 rect（layout 后可用；用于 a11y / hit-test 上界）。
    pub fn overlay_rect(&self) -> Option<Rect> {
        self.overlay_root.as_ref().map(|n| n.cached_rect)
    }

    // ── P3-4-5 动画时钟 API ────────────────────────────────────────────────

    /// 推进动画时钟（毫秒）。driver 在每 frame 调用，传入距上帧的 delta。
    pub fn advance_clock(&mut self, delta_ms: i64) {
        self.animation_now_ms = self.animation_now_ms.saturating_add(delta_ms);
    }

    /// 当前动画时间（毫秒）。widget 在 layout/paint 中可读此值采样 Tween。
    pub fn animation_now_ms(&self) -> i64 {
        self.animation_now_ms
    }

    /// 是否有 widget 在上次 paint 中请求了下一帧（即动画未完成）。
    /// driver 据此决定是否继续 pump_frame。
    pub fn has_pending_animation(&self) -> bool {
        self.last_frame_requests > 0
    }
}

#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;
