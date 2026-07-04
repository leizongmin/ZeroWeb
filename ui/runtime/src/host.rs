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
//! **本轮范围**：无窗口、单线程、确定性。焦点/键盘事件路由（DC-8）、winit 事件循环接入
//! （`ui/adapters/winit`，M2/M4）与多窗口在后续里程碑；本 host 只负责几何/绘制/指针命中闭环。

use crate::accessibility::AccessibilityBackend;
use compact_str::CompactString;
use zero_ui_core::action::{ActionId, ActionPayload, EventResult};
use zero_ui_core::binding::{PropsMap, Value};
use zero_ui_core::event::PointerPhase;
use zero_ui_core::event::UiEvent;
use zero_ui_core::focus::{FocusDirection, FocusScope};
use zero_ui_core::geometry::{Constraints, Point, Rect, Rounding, Size, Vec2};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::semantics::{SemanticsFlags, SemanticsNode};
use zero_ui_core::widget::{
    ComponentType, EventCtx, LayoutCtx, MountCtx, PaintCtx, SemanticsCtx, Widget, WidgetId, WidgetSpec,
};
use zero_ui_gestures::{Gesture, GestureArena, PointerEvent};
use zero_ui_render::{RenderPrimitive, Scene, SceneEntry, SceneRecorder};

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

    fn build(&self, spec: &WidgetSpec) -> Option<Box<dyn Widget>> {
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
}

impl ContainerKind {
    /// 由组件类型名识别内置容器；非容器返回 `None`（交给已注册 widget 或填充策略）。
    pub fn from_component(component: &ComponentType) -> Option<ContainerKind> {
        match component.0.as_str() {
            "Column" => Some(ContainerKind::Column),
            "Row" => Some(ContainerKind::Row),
            "Stack" => Some(ContainerKind::Stack),
            _ => None,
        }
    }
}

/// host 派发事件后收集到的、widget 发出的 action（单向数据流：交给应用 reducer）。
#[derive(Debug, Clone, PartialEq)]
pub struct EmittedAction {
    pub action: ActionId,
    pub payload: Option<ActionPayload>,
}

/// 单个 widget 实例的 retained 运行态节点。
///
/// 持有 `Box<dyn Widget>`（实例 + 临时状态如 hover/pressed）、上次应用的 props、
/// 缓存几何（measure 算 size / arrange 定 rect）与节点级失效标记。
#[derive(Default)]
struct HostNode {
    id: WidgetId,
    component: ComponentType,
    props: PropsMap,
    widget: Option<Box<dyn Widget>>,
    children: Vec<HostNode>,
    cached_size: Size,
    cached_rect: Rect,
    invalidation: InvalidationFlags,
    epoch: u32,
    focusable: bool,
}

/// Retained widget host —— 三棵树运行态的驱动器。
pub struct WidgetHost {
    registry: WidgetRegistry,
    root: Option<HostNode>,
    scene: Scene,
    epoch: u32,
    pending: InvalidationFlags,
    focused: Option<WidgetId>,
    /// 活跃焦点作用域（modal/popup trap，DC-8 phase-3）。
    ///
    /// 非空时 `focus_next` 在作用域可聚焦项内遍历（`trap=true` 折返不逃逸），
    /// 供弹层/模态对话框接管 Tab 焦点。`set_root` 后作用域可能失效（节点被移除），
    /// 调用方应于弹层关闭时 `exit_focus_scope`。
    active_scope: Option<FocusScope>,
    /// 当前主题 semantic token（DC-5：paint 时注入 `PaintCtx.tokens`，控件消费而非硬编码）。
    tokens: zero_ui_core::theme::SemanticTokens,
    /// 已挂载的平台无障碍后端（DC-8 平台桥接）：`flush_accessibility` 时推送语义树 + 焦点/通告。
    /// `None` 表示未启用 a11y（默认）；真实平台实现（Win UI Automation / macOS NSAccessibility /
    /// Linux AT-SPI / 移动 TalkBack）在 M4 runtime adapter 落地。
    a11y_backend: Option<Box<dyn AccessibilityBackend>>,
    /// 上次推送给 a11y 后端的焦点 id；与 `focused` 比较以检测焦点变化，
    /// 避免在本 host 的所有改焦点位点（focus_next/set_focus/click/作用域）分别埋标记。
    last_pushed_focus: Option<WidgetId>,
    /// DC-15 opt-in 手势 arena：`Some` 时 `dispatch_event` 把指针序列顺带喂入 arena 仲裁
    /// （Tap/Pan/Fling），识别出的手势缓冲到 `pending_gestures` 由 `take_gestures` 取回。
    /// `None`（默认）→ dispatch 行为与无 arena 逐位等价（向后兼容）。
    gesture_arena: Option<GestureArena>,
    /// arena 识别出但尚未被 `take_gestures` 取回的手势。
    pending_gestures: Vec<Gesture>,
    /// 单调时间戳（ms），喂给 arena 的每个指针事件递增（UiEvent::Pointer 不带时间戳）。
    gesture_clock: i64,
    /// 上次指针事件命中的最深节点 id（hover 追踪，DC-8/F1）。
    ///
    /// 在 Pressed/Moved 事件 hit-test 后更新；若新目标与上次不同，向前一节点派发
    /// 合成 `PointerPhase::Exited`（通知控件清除 hover/pressed 交互态）。
    /// Cancelled 时清空本字段并派发 Exited（如有上次悬停节点）。
    last_hovered: Option<WidgetId>,
    /// 可供 widgets 查询的实时字体度量 `(ascent, descent)`（DC-11 text path 统一）。
    ///
    /// 由应用层（apps/browser）从共享 `FontdueBackend` 查询后设置，`paint_node` 注入
    /// `PaintCtx.font_metrics`。`None` 时 `PaintCtx::line_metrics` 回落 heuristic。
    font_metrics: Option<(f32, f32)>,
}

impl Default for WidgetHost {
    fn default() -> WidgetHost {
        WidgetHost {
            registry: WidgetRegistry::new(),
            root: None,
            scene: Scene::new(),
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
    /// 主题变化时调用，并应同步标记 `needs_paint`（仅色变不布局）。
    pub fn set_tokens(&mut self, tokens: zero_ui_core::theme::SemanticTokens) {
        self.tokens = tokens;
    }

    /// 设置实时字体度量 `(ascent, descent)`，由应用层从共享 `FontdueBackend` 查询后注入。
    /// 设置后 `paint_node` 会将其注入 `PaintCtx.font_metrics`，使 widgets 经
    /// [`PaintCtx::line_metrics`] 得到与手绘 chrome 相同的基线计算（DC-11 text path 统一）。
    pub fn set_font_metrics(&mut self, ascent: f32, descent: f32) {
        self.font_metrics = Some((ascent, descent));
    }

    /// 用新声明树 reconcile：按 `WidgetId` + `ComponentType` 复用既有 widget 实例（保留临时状态），
    /// props 变化时调 `Widget::update`；新增节点 mount。结构/props 变化触发 layout+paint 失效。
    pub fn set_root(&mut self, spec: &WidgetSpec) {
        self.epoch = self.epoch.wrapping_add(1);
        match &mut self.root {
            Some(root) => {
                reconcile_node(root, spec, &self.registry, self.epoch);
                // props/结构变化可能改尺寸（reconcile_node 已在节点级标 NEEDS_LAYOUT）→ host 级
                // 也须标 NEEDS_LAYOUT，否则 needs_layout() 为假、driver/host 不 relayout → 几何停滞
                // （state 变化改 widget 尺寸时 scene 停在旧几何）。同时标 NEEDS_PAINT/SEMANTICS。
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
        // 活跃焦点作用域：reconcile 后子树可能变化，按 scope.id 重新收集可聚焦项；
        // 若作用域根已从树中移除（弹层关闭），则清除作用域（DC-8 phase-3）。
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

    /// 是否需要 re-layout。
    pub fn needs_layout(&self) -> bool {
        self.pending.requires_layout()
    }

    /// 是否需要 re-paint（含 layout 连带）。
    pub fn needs_paint(&self) -> bool {
        self.pending.requires_paint()
    }

    /// 是否需要重建并推送语义树到 a11y 后端（DC-8 平台桥接）。
    /// 结构变化（`set_root`）、locale 变化（外部 `mark(NEEDS_SEMANTICS)`）置位。
    pub fn needs_semantics(&self) -> bool {
        self.pending.contains(InvalidationFlags::NEEDS_SEMANTICS)
    }

    /// 挂载平台无障碍后端（DC-8 平台桥接，spec FR-011）。
    ///
    /// 挂载后立即标记需要推送语义树：下一次 `flush_accessibility` 推送全树 + 当前焦点。
    /// 真实平台实现（Win UI Automation / macOS NSAccessibility / Linux AT-SPI / 移动 TalkBack）
    /// 在 M4 runtime adapter 落地；SDK 侧只定义契约，测试用 `RecordingAccessibilityBackend`。
    pub fn set_accessibility_backend(&mut self, backend: Box<dyn AccessibilityBackend>) {
        self.a11y_backend = Some(backend);
        self.pending |= InvalidationFlags::NEEDS_SEMANTICS;
    }

    /// 把语义树 + 焦点变更推送到已挂载的 a11y 后端（DC-8 平台桥接）。
    ///
    /// 应在 `layout` 之后调用（语义节点绝对 rect 来自 layout 产物）；幂等——无变化时不推送。
    /// - `NEEDS_SEMANTICS`（结构/locale 变化）：重建语义树并 `update_tree`（全树）；
    /// - 焦点变化（与上次推送不同）：`focus_moved`（仅焦点变化不重建全树，廉价；与 Flutter
    ///   a11y 语义一致：平台以 focus_moved 事件追踪焦点，结构变化时重建的树才刷新 FOCUSED 标志）。
    ///
    /// 未挂载后端时清掉 `NEEDS_SEMANTICS`（避免标记无限累积，语义仅供 paint/a11y）。
    pub fn flush_accessibility(&mut self) {
        // 先在不可变借用 self 期间算出 owned 的语义树，再 &mut 借用后端，
        // 避免 `self.semantics()`（&self）与 `self.a11y_backend.as_mut()`（&mut self）冲突。
        let dirty = self.pending.contains(InvalidationFlags::NEEDS_SEMANTICS);
        let tree = if dirty { self.semantics() } else { None };

        let Some(backend) = self.a11y_backend.as_mut() else {
            // 无后端：清掉 NEEDS_SEMANTICS（语义仅供 paint/a11y，避免标记无限累积）。
            self.pending.remove(InvalidationFlags::NEEDS_SEMANTICS);
            return;
        };
        if dirty {
            backend.update_tree(tree.as_ref());
            self.pending.remove(InvalidationFlags::NEEDS_SEMANTICS);
        }
        // 焦点字段与 a11y_backend 为不相交字段借用，可与 backend(&mut a11y_backend) 并存。
        if self.focused != self.last_pushed_focus {
            backend.focus_moved(self.focused.clone());
            self.last_pushed_focus = self.focused.clone();
        }
    }

    /// 经 a11y 后端发出短暂通告（如「页面已加载」「已复制」），平台屏幕阅读器朗读（DC-8）。
    /// 未挂载后端时为 no-op。
    pub fn announce(&mut self, message: &str) {
        if let Some(backend) = self.a11y_backend.as_mut() {
            backend.announce(message);
        }
    }

    /// 挂载手势 arena（DC-15 移动端 / 触摸 skeleton，opt-in）。
    ///
    /// 挂载后，`dispatch_event` 在处理指针事件时顺带把序列喂入 arena 仲裁（Tap/Pan/Fling），
    /// 识别出的手势缓冲，由 [`take_gestures`](WidgetHost::take_gestures) 取回。调用方负责
    /// 预先在 arena 注册所需识别器（`arena.push(TapRecognizer/PanRecognizer/...)`）。
    ///
    /// **向后兼容**：未挂载时（默认）`dispatch_event` 行为逐位等价（无手势缓冲、无额外失效）。
    ///
    /// 限制：`UiEvent::Pointer` 为单指针抽象，本路径覆盖 Tap/Pan/Fling；多指 Pinch 需扩展
    /// UiEvent 携带指针 id（future work）。
    pub fn set_gesture_arena(&mut self, arena: GestureArena) {
        self.gesture_arena = Some(arena);
    }

    /// 是否挂载了手势 arena。
    pub fn has_gesture_arena(&self) -> bool {
        self.gesture_arena.is_some()
    }

    /// 取回 arena 自上次取回后识别出的手势（清空内部缓冲）。
    pub fn take_gestures(&mut self) -> Vec<Gesture> {
        std::mem::take(&mut self.pending_gestures)
    }

    /// 两遍布局：measure（自下而上算尺寸）+ arrange（自上而下定绝对 rect）。
    /// 返回根尺寸。调用后清除 `NEEDS_LAYOUT`（layout 隐含 re-paint 由 `NEEDS_PAINT` 表达）。
    pub fn layout(&mut self, constraints: Constraints) -> Size {
        let Some(root) = self.root.as_mut() else {
            return Size::ZERO;
        };
        let mut lctx = LayoutCtx { scale_factor: 1.0 };
        let size = measure(root, &mut lctx, constraints);
        arrange(root, Point::ZERO);
        self.pending.remove(InvalidationFlags::NEEDS_LAYOUT);
        size
    }

    /// 遍历 widget 实例树 paint 进全局 `Scene`：每节点 widget 以局部坐标 paint，
    /// 按 `cached_rect.origin` 平移到绝对坐标后并入；clip 取祖先 clip 与节点 rect 的交集。
    /// 调用后清空 pending（本轮几何/外观已落盘到 Scene）。
    pub fn paint(&mut self) -> &Scene {
        self.scene = Scene::new();
        if let Some(root) = self.root.as_mut() {
            let viewport = Some(root.cached_rect);
            paint_node(root, &mut self.scene, viewport, &self.tokens, self.font_metrics);
        }
        self.pending = InvalidationFlags::CLEAN;
        &self.scene
    }

    /// 最近一次 `paint` 产出的场景（只读）。
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    /// 查指定 `WidgetId` 节点的绝对 rect（layout 后有效；IME rect / 断言用）。
    pub fn rect_of(&self, id: &WidgetId) -> Option<Rect> {
        self.root.as_ref().and_then(|r| find_rect(r, id))
    }

    /// 节点创建/复用 epoch（用于断言与调试 retained 复用：reconcile 后同 `WidgetId` 节点
    /// epoch 不变表示被复用而非重建）。
    pub fn creation_epoch(&self, id: &WidgetId) -> Option<u32> {
        self.root.as_ref().and_then(|r| find_epoch(r, id))
    }

    /// 派发输入事件：
    /// - **位置事件**（指针/滚动）：hit-test 到最深的、最上层命中节点，冒泡派发。
    /// - **键盘事件**：`Tab`/`Shift-Tab` 推进焦点遍历；其它键路由到当前 focused widget（DC-8）。
    ///
    /// 收集 widget 发出的 action；任何 action 发出或焦点变化都标记 `NEEDS_PAINT`。
    pub fn dispatch_event(&mut self, event: &UiEvent) -> Vec<EmittedAction> {
        let mut emitted = Vec::new();
        // DC-15 opt-in 手势 arena：挂载时把指针序列顺带喂入 arena 仲裁（不影响下方 hit-test/冒泡）。
        // 用 event.pointer_id 区分多指（双指 Pinch）；内部 gesture_clock 提供时序；Cancelled → arena.reset。
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
                    PointerPhase::Exited => None, // 合成事件，不喂 arena
                }
            } else {
                None
            };
            if let Some(g) = recognized {
                self.pending_gestures.push(g);
                self.pending |= InvalidationFlags::NEEDS_PAINT;
            }
        }
        let Some(root) = self.root.as_mut() else {
            return emitted;
        };
        // F1 hover 追踪：在 Pressed/Moved 时检测悬停节点变化，合成为离开的 `Exited` 事件。
        // 根节点不可变借用（仅拿 id）——以下 match 中 `dispatch_node` 的 &mut 在独立作用域。
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
                        // 悬停变化是纯视觉（不 emit action），清空 emitted。
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
                    // Cancelled 也是纯视觉，清空 emitted。
                    emitted.clear();
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                }
                _ => {}
            }
        }
        match event {
            UiEvent::Key {
                code,
                action: key_action,
                modifiers,
                ..
            } => {
                // Tab / Shift-Tab → 焦点遍历（按声明顺序，wrap）。
                if code.0.as_str() == "Tab" && matches!(key_action, zero_ui_core::event::KeyAction::Pressed) {
                    let dir = if modifiers.contains(zero_ui_core::event::Modifiers::SHIFT) {
                        zero_ui_core::focus::FocusDirection::Backward
                    } else {
                        zero_ui_core::focus::FocusDirection::Forward
                    };
                    self.focus_next(dir);
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                    return emitted;
                }
                // 其它键 → 路由到 focused widget。
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
                phase: zero_ui_core::event::PointerPhase::Pressed,
                position,
                ..
            } => {
                // click-to-focus：按下时聚焦命中点最深 focusable widget（DC-8 phase-2）。
                let target = deepest_focusable_at(root, *position);
                let handled = dispatch_node(root, event, &mut emitted);
                if let Some(id) = target
                    && self.focused.as_ref() != Some(&id)
                {
                    self.focused = Some(id);
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                }
                if handled {
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                }
            }
            _ => {
                // 其它位置事件（移动/释放/滚动）。
                let handled = dispatch_node(root, event, &mut emitted);
                if handled {
                    self.pending |= InvalidationFlags::NEEDS_PAINT;
                }
            }
        }
        emitted
    }

    /// 当前 focused widget 的 IME 输入矩形（绝对坐标；DC-8 phase-2）。
    ///
    /// 由 focused widget 的 `Widget::ime_rect`（局部 caret 矩形）按节点 origin 平移得到，
    /// 供平台 IME / 软键盘定位。无焦点或 widget 无 caret 返回 `None`。
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

    /// 当前 focused widget id（DC-8）。
    pub fn focused_id(&self) -> Option<&WidgetId> {
        self.focused.as_ref()
    }

    /// 显式设置焦点到指定 widget（若存在）。
    pub fn set_focus(&mut self, id: WidgetId) {
        self.focused = Some(id);
        self.pending |= InvalidationFlags::NEEDS_PAINT;
    }

    /// 按方向推进焦点遍历（Tab 按声明顺序，wrap；DC-8 / spec FR-011）。
    ///
    /// 若存在活跃焦点作用域（`enter_focus_scope`，modal/popup trap）：在作用域可聚焦项内
    /// 遍历，`trap=true` 到边界折返（焦点不逃逸）；`trap=false` 到边界返回则退出作用域、
    /// 落到全局遍历。无作用域时按整树声明顺序 wrap。
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
                    // 非 trap 逃逸：退出作用域，落到全局遍历。
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

    /// 进入焦点作用域（DC-8 phase-3，modal/popup focus trap）。
    ///
    /// 在 `scope_root` 子树内收集可聚焦项作为作用域焦点候选；进入后焦点落到首个候选
    /// （按声明顺序）。`trap=true` 时 Tab 折返不逃逸（典型模态/弹层用例）；
    /// `trap=false` 时到边界逃逸到全局遍历。`scope_root` 不存在则忽略。
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

    /// 退出当前焦点作用域（弹层/模态关闭时调用，DC-8 phase-3）。焦点保持当前位置。
    pub fn exit_focus_scope(&mut self) {
        if self.active_scope.take().is_some() {
            self.pending |= InvalidationFlags::NEEDS_PAINT;
        }
    }

    /// 当前活跃焦点作用域（只读；用于断言/调试弹层焦点接管）。
    pub fn active_focus_scope(&self) -> Option<&FocusScope> {
        self.active_scope.as_ref()
    }

    /// 构建无障碍语义树（spec FR-011 / DC-8 phase-3）。
    ///
    /// 遍历 retained 树，对每个有 widget 的节点调 `Widget::semantics` 取自描述节点，
    /// 填入绝对 `rect`（layout 后有效）并 OR 进 host 级焦点标志（FOCUSABLE/FOCUSED），
    /// 按实际 widget 层级组装。纯容器节点（无 widget）做 semantics merge（子节点上浮），
    /// 避免无内容中间节点污染读屏树。根节点始终产出。返回 `None` 表示尚未 `set_root`。
    pub fn semantics(&self) -> Option<SemanticsNode> {
        let root = self.root.as_ref()?;
        let mut s = self_semantics(root, self.focused.as_ref());
        s.children.clear();
        for child in &root.children {
            build_semantics(child, self.focused.as_ref(), &mut s.children);
        }
        Some(s)
    }
}

/// 把 `EventResult` 中的 action 收集进 `emitted`。
fn collect_emit(res: EventResult, emitted: &mut Vec<EmittedAction>) {
    match res {
        EventResult::Ignored | EventResult::Consumed => {}
        EventResult::Emit(action) => emitted.push(EmittedAction { action, payload: None }),
        EventResult::EmitWithPayload(action, payload) => emitted.push(EmittedAction {
            action,
            payload: Some(payload),
        }),
    }
}

/// 按声明（前序）顺序收集所有 focusable 节点的 id。
fn collect_focusables(node: &HostNode, out: &mut Vec<WidgetId>) {
    if node.focusable {
        out.push(node.id.clone());
    }
    for c in &node.children {
        collect_focusables(c, out);
    }
}

/// 按 id 查可变节点。
fn find_node_mut<'a>(node: &'a mut HostNode, id: &WidgetId) -> Option<&'a mut HostNode> {
    if &node.id == id {
        return Some(node);
    }
    node.children.iter_mut().find_map(|c| find_node_mut(c, id))
}

/// 按 id 查不可变节点。
fn find_node<'a>(node: &'a HostNode, id: &WidgetId) -> Option<&'a HostNode> {
    if &node.id == id {
        return Some(node);
    }
    node.children.iter().find_map(|c| find_node(c, id))
}

/// 命中点下最深的 focusable 节点 id（click-to-focus 用，DC-8 phase-2）。
///
/// 优先返回子节点（最深、最上层优先，倒序），否则本节点（若 focusable）。
fn deepest_focusable_at(node: &HostNode, point: Point) -> Option<WidgetId> {
    if !node.cached_rect.contains(point) {
        return None;
    }
    for child in node.children.iter().rev() {
        if let Some(id) = deepest_focusable_at(child, point) {
            return Some(id);
        }
    }
    if node.focusable { Some(node.id.clone()) } else { None }
}

/// 命中点下最深节点 id（hover 追踪用，不论是否 focusable）。
///
/// 优先返回子节点（最深、最上层优先，倒序），否则本节点。
fn deepest_node_at(node: &HostNode, point: Point) -> Option<WidgetId> {
    if !node.cached_rect.contains(point) {
        return None;
    }
    for child in node.children.iter().rev() {
        if let Some(id) = deepest_node_at(child, point) {
            return Some(id);
        }
    }
    Some(node.id.clone())
}

// ---------------- 构建与 reconcile ----------------

fn build_node(spec: &WidgetSpec, registry: &WidgetRegistry, epoch: u32) -> HostNode {
    let id = spec.id.clone().unwrap_or_else(|| WidgetId::new("__anonymous__"));
    let mut node = HostNode {
        id: id.clone(),
        component: spec.component.clone(),
        props: spec.props.clone(),
        widget: None,
        children: Vec::with_capacity(spec.children.len()),
        cached_size: Size::ZERO,
        cached_rect: Rect::ZERO,
        invalidation: InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT,
        epoch,
        focusable: false,
    };
    if let Some(mut w) = registry.build(spec) {
        let mut flags = InvalidationFlags::CLEAN;
        w.mount(&mut MountCtx {
            id: &node.id,
            invalidation: &mut flags,
        });
        // 初始 props 同步：受控控件（如 TextField）需从 props 初始化内部状态，
        // 而非仅在 reconcile 时才看到 props。
        w.update(
            &mut zero_ui_core::widget::UpdateCtx {
                invalidation: &mut flags,
            },
            &node.props,
        );
        node.invalidation |= flags;
        node.focusable = w.focusable();
        node.widget = Some(w);
    }
    for child in &spec.children {
        node.children.push(build_node(child, registry, epoch));
    }
    node
}

/// 把既有节点与新 spec 对齐：props 变化 → update；children 递归 reconcile。
fn reconcile_node(node: &mut HostNode, spec: &WidgetSpec, registry: &WidgetRegistry, epoch: u32) {
    if node.props != spec.props {
        if let Some(w) = node.widget.as_mut() {
            let mut flags = InvalidationFlags::CLEAN;
            w.update(
                &mut zero_ui_core::widget::UpdateCtx {
                    invalidation: &mut flags,
                },
                &spec.props,
            );
            node.invalidation |= flags;
        }
        node.props = spec.props.clone();
        // props 变化可能改尺寸（如 label 变长）→ 标记 layout。
        node.invalidation |= InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
    }
    reconcile_children(&mut node.children, &spec.children, registry, epoch);
}

fn reconcile_children(existing: &mut Vec<HostNode>, new_specs: &[WidgetSpec], registry: &WidgetRegistry, epoch: u32) {
    let mut next: Vec<HostNode> = Vec::with_capacity(new_specs.len());
    for (i, spec) in new_specs.iter().enumerate() {
        let reuse = existing.get(i).is_some_and(|e| same_node(e, spec));
        if reuse {
            // 复用：把既有实例整块移出（保留 widget 实例 + 临时状态），再对齐其 props/children。
            let mut el = std::mem::take(&mut existing[i]);
            reconcile_node(&mut el, spec, registry, epoch);
            next.push(el);
        } else {
            next.push(build_node(spec, registry, epoch));
        }
    }
    *existing = next;
}

/// 复用条件：同 `WidgetId`（非匿名）+ 同 `ComponentType`（Flutter `canUpdate` 语义）。
fn same_node(node: &HostNode, spec: &WidgetSpec) -> bool {
    match &spec.id {
        Some(id) => &node.id == id && node.component == spec.component,
        None => false,
    }
}

// ---------------- layout ----------------

/// 从 props 读 `gap`（Column/Row 主轴间距）。接受 Int/Float，缺省 0。
fn gap_from_props(props: &PropsMap) -> f32 {
    match props.get("gap") {
        Some(Value::Float(f)) => *f as f32,
        Some(Value::Int(i)) => *i as f32,
        _ => 0.0,
    }
}

/// 节点的容器布局种类：优先读 `props.layout`（`"column"`/`"row"`/`"stack"`，大小写不敏感），
/// 否则按组件名识别内置容器（`Column`/`Row`/`Stack`）。
///
/// 让任意组件名（如 `browser.DesktopBrowserShell`、`browser.ToolbarRow`）经 props 声明为容器，
/// 无需 host 硬编码 chrome/业务组件名 —— 保持 host 浏览器无关。
fn node_container_kind(node: &HostNode) -> Option<ContainerKind> {
    if let Some(Value::Text(s)) = node.props.get("layout") {
        match s.as_str() {
            "column" | "Column" => return Some(ContainerKind::Column),
            "row" | "Row" => return Some(ContainerKind::Row),
            "stack" | "Stack" => return Some(ContainerKind::Stack),
            _ => {}
        }
    }
    ContainerKind::from_component(&node.component)
}

/// 从 props 读 `flex`（Row/Column 主轴弹性权重）。接受 Int/Float，缺省/负值 → 0。
///
/// `flex == 0`（默认）：子节点取其固有尺寸，按声明顺序贪心占用剩余主轴空间（历史行为）。
/// `flex > 0`：子节点参与弹性分配，按 `flex / Σflex` 比例瓜分非弹性子节点占用后的剩余空间
/// （`Expanded` 语义，tight 到份额）。这让 chrome toolbar 等多个「填宽」子节点能共存。
fn flex_from_props(props: &PropsMap) -> f32 {
    match props.get("flex") {
        Some(Value::Float(f)) => (*f as f32).max(0.0),
        Some(Value::Int(i)) => (*i as f32).max(0.0),
        _ => 0.0,
    }
}

/// 从 props 提取浮点值（`Float` 或 `Int`），缺省返回 `default`。
fn float_from_props(props: &PropsMap, key: &str, default: f32) -> f32 {
    match props.get(key) {
        Some(Value::Float(f)) => *f as f32,
        Some(Value::Int(i)) => *i as f32,
        _ => default,
    }
}

/// 从子节点 props 读取 min/max 约束（缺省：min = 0, max = f32::MAX，即不约束）。
fn child_constraints_from_props(props: &PropsMap) -> (f32, f32, f32, f32) {
    let min_w = float_from_props(props, "min_width", 0.0).max(0.0);
    let max_w = float_from_props(props, "max_width", f32::MAX).max(min_w);
    let min_h = float_from_props(props, "min_height", 0.0).max(0.0);
    let max_h = float_from_props(props, "max_height", f32::MAX).max(min_h);
    (min_w, max_w, min_h, max_h)
}

/// 把尺寸钳到 `(min_w, max_w, min_h, max_h)` 范围内。
fn clamp_size(s: Size, min_w: f32, max_w: f32, min_h: f32, max_h: f32) -> Size {
    Size::new(s.width.clamp(min_w, max_w), s.height.clamp(min_h, max_h))
}

/// 从 props 读交叉轴对齐（`cross_axis_align`：`"start"`/`"center"`/`"end"`，大小写不敏感；
/// Row 也接受 `"top"`/`"bottom"`、Column 也接受 `"left"`/`"right"`）。
///
/// 缺省 [`CrossAxisAlignment::Start`]（向后兼容历史顶/左对齐行为）。
fn cross_axis_alignment_from_props(props: &PropsMap) -> CrossAxisAlignment {
    if let Some(Value::Text(s)) = props.get("cross_axis_align") {
        match s.to_ascii_lowercase().as_str() {
            "center" => return CrossAxisAlignment::Center,
            "end" | "bottom" | "right" => return CrossAxisAlignment::End,
            "start" | "top" | "left" => return CrossAxisAlignment::Start,
            _ => {}
        }
    }
    CrossAxisAlignment::Start
}

/// 从 props 读主轴对齐（`main_axis_align`，大小写不敏感；`-`/`_`/无分隔均可）：
/// `"start"` / `"center"` / `"end"` / `"space_between"` / `"space_around"` / `"space_evenly"`
/// （Row 也接受 `"left"`/`"right"`、Column 也接受 `"top"`/`"bottom"`）。
///
/// 缺省 [`MainAxisAlignment::Start`]（向后兼容历史左/顶打包行为）。需容器主轴有剩余空间才生效
/// （fill-sizing 或父 tight/exact 约束）；弹性子节点消费剩余空间时主轴对齐无可见效果。
fn main_axis_alignment_from_props(props: &PropsMap) -> MainAxisAlignment {
    if let Some(Value::Text(s)) = props.get("main_axis_align") {
        // 归一化：去 `-`/`_` 再小写，使 "space-between"/"space_between"/"spacebetween" 统一。
        let norm: String = s
            .chars()
            .filter(|c| *c != '-' && *c != '_')
            .collect::<String>()
            .to_ascii_lowercase();
        match norm.as_str() {
            "center" => return MainAxisAlignment::Center,
            "end" | "right" | "bottom" => return MainAxisAlignment::End,
            "spacebetween" => return MainAxisAlignment::SpaceBetween,
            "spacearound" => return MainAxisAlignment::SpaceAround,
            "spaceevenly" => return MainAxisAlignment::SpaceEvenly,
            "start" | "left" | "top" => return MainAxisAlignment::Start,
            _ => {}
        }
    }
    MainAxisAlignment::Start
}

/// 线性容器（Row/Column）的主轴方向。
#[derive(Clone, Copy, PartialEq)]
enum MainAxis {
    /// Row：主轴 = X（width），交叉轴 = Y（height）。
    Horizontal,
    /// Column：主轴 = Y（height），交叉轴 = X（width）。
    Vertical,
}

/// 线性容器（Row/Column）的交叉轴对齐方式。
///
/// 控制子节点在容器交叉轴（Row=垂直 / Column=水平）上的放置。对齐基准是容器自身的交叉尺寸
/// （= 最高/最宽子节点，见 [`measure_linear`] 的 `total_cross`），故最高/最宽的子节点偏移恒为 0，
/// 较小的子节点按对齐方式定位——这正是 chrome 工具栏所需（高元素满高、短元素垂直居中）。
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum CrossAxisAlignment {
    /// 子节点紧贴交叉轴起点（Row=顶部 / Column=左侧）。默认，向后兼容历史行为。
    #[default]
    Start,
    /// 子节点在交叉轴居中。
    Center,
    /// 子节点紧贴交叉轴终点（Row=底部 / Column=右侧）。
    End,
}

/// 计算子节点在交叉轴上的偏移量（相对交叉轴起点）。
///
/// `container_cross` = 容器交叉尺寸；`child_cross` = 子节点交叉尺寸。
/// `free = container − child`（钳到非负）按对齐方式分配：Start→0、Center→free/2、End→free。
fn cross_offset(align: CrossAxisAlignment, container_cross: f32, child_cross: f32) -> f32 {
    let free = (container_cross - child_cross).max(0.0);
    match align {
        CrossAxisAlignment::Start => 0.0,
        CrossAxisAlignment::Center => free * 0.5,
        CrossAxisAlignment::End => free,
    }
}

/// 线性容器（Row/Column）的主轴对齐方式。
///
/// 控制整组子节点在容器主轴上的放置。**生效前提**：容器主轴有剩余空间（`free > 0`），即容器
/// 尺寸大于子节点打包长度。这发生在：父节点给容器 tight/exact 主轴约束（fill-sizing，
/// [`measure_linear`] 钳到 min），或容器主轴 max 大于内容。弹性子节点（`flex > 0`）会消费全部
/// 剩余空间 → `free = 0` → 主轴对齐无可见效果（与 CSS flexbox 一致：`flex-grow` 优先于
/// `justify-content`）。
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum MainAxisAlignment {
    /// 子节点紧贴主轴起点（Row=左侧 / Column=顶部），子节点间用 `gap` 间隔。默认，向后兼容。
    #[default]
    Start,
    /// 整组子节点（含 `gap`）在主轴居中。
    Center,
    /// 整组子节点（含 `gap`）紧贴主轴终点（Row=右侧 / Column=底部）。
    End,
    /// 首尾贴边，剩余空间均匀插入 `n−1` 个间隙（CSS `justify-content: space-between`）。
    SpaceBetween,
    /// 每个子节点两侧等量剩余空间（CSS `space-around`）。
    SpaceAround,
    /// 首尾与每对子节点之间均等剩余空间（CSS `space-evenly`）。
    SpaceEvenly,
}

/// 主轴分布：返回 `(起始偏移, 子节点间额外间距)`（CSS `justify-content` 语义）。
///
/// `extra` = 容器主轴尺寸 − 子节点主轴尺寸和 − `gap*(n−1)`（调用方钳到非负），即扣除最小 `gap`
/// 后可分配的剩余空间；`n` = 子节点数。最终子节点间间距 = `gap + between_extra`。
///
/// - Start / Center / End：`between_extra = 0`，剩余空间全作起始偏移（0 / extra/2 / extra）。
/// - SpaceBetween：起始偏移 0，剩余空间均分到 `n−1` 个间隙（`n ≤ 1` 退化为 Start）。
/// - SpaceAround：剩余空间均分到 `n` 个「环绕」槽，起始偏移 = extra/(2n)，间隙额外 = extra/n。
/// - SpaceEvenly：剩余空间均分到 `n+1` 个等分（含首尾），起始偏移 = 间隙额外 = extra/(n+1)。
///
/// `extra <= 0`（溢出）或 `n == 0` 时一律退化为 Start（offset=0，间距=gap）。
fn main_axis_layout(align: MainAxisAlignment, extra: f32, n: usize) -> (f32, f32) {
    if extra <= 0.0 || n == 0 {
        return (0.0, 0.0);
    }
    let nf = n as f32;
    match align {
        MainAxisAlignment::Start => (0.0, 0.0),
        MainAxisAlignment::Center => (extra * 0.5, 0.0),
        MainAxisAlignment::End => (extra, 0.0),
        MainAxisAlignment::SpaceBetween => {
            if n > 1 {
                (0.0, extra / (nf - 1.0))
            } else {
                (0.0, 0.0)
            }
        }
        MainAxisAlignment::SpaceAround => (extra / (2.0 * nf), extra / nf),
        MainAxisAlignment::SpaceEvenly => {
            let per = extra / (nf + 1.0);
            (per, per)
        }
    }
}

/// Row/Column 共用的弹性布局：两遍 measure。
///
/// 1. **非弹性子节点**（`flex == 0`）：按声明顺序贪心 measure（约束 = 剩余主轴空间），
///    与历史行为一致；累加其主轴尺寸得到 `used_nonflex`。
/// 2. **弹性子节点**（`flex > 0`）：剩余空间 `free = max_main − gaps − used_nonflex` 按
///    `flex` 权重比例分配，每子节点以 tight 主轴约束（min=max=份额）measure（fill 控件正好
///    填满份额）。无弹性子节点时退化为纯贪心，与旧实现逐位等价（向后兼容）。
///
/// 交叉轴：所有子节点以 `max_cross`（loose）measure，容器交叉尺寸 = 子节点交叉最大值。
///
/// **尺寸约束**：容器主/交叉尺寸钳到 `[min, max]`（遵守传入约束的 `min`，使容器可在父节点给定
/// tight/exact 约束时填满空间——为主轴对齐 [`MainAxisAlignment`] 提供可分配的剩余空间）。
fn measure_linear(node: &mut HostNode, lctx: &mut LayoutCtx, constraints: Constraints, axis: MainAxis) -> Size {
    let gap = gap_from_props(&node.props);
    let n = node.children.len();
    let (min_main, max_main, min_cross, max_cross) = match axis {
        MainAxis::Horizontal => (
            constraints.min_width,
            constraints.max_width,
            constraints.min_height,
            constraints.max_height,
        ),
        MainAxis::Vertical => (
            constraints.min_height,
            constraints.max_height,
            constraints.min_width,
            constraints.max_width,
        ),
    };
    // 主轴/交叉尺寸 per child（arrange 仍按 cached_size 顺序放置，无需调整）。
    let mut child_main = vec![0.0_f32; n];
    let mut child_cross = vec![0.0_f32; n];

    let flexes: Vec<f32> = node.children.iter().map(|c| flex_from_props(&c.props)).collect();
    let total_flex: f32 = flexes.iter().sum();

    // Pass 1：非弹性子节点（贪心，等同历史行为）。
    let mut cursor = 0.0_f32;
    for i in 0..n {
        if flexes[i] > 0.0 {
            continue;
        }
        let (min_w, max_w, min_h, max_h) = child_constraints_from_props(&node.children[i].props);
        let remaining = (max_main - cursor).max(0.0);
        let child_c = match axis {
            MainAxis::Horizontal => Constraints {
                min_width: 0.0,
                max_width: remaining,
                min_height: 0.0,
                max_height: max_cross,
            },
            MainAxis::Vertical => Constraints {
                min_width: 0.0,
                max_width: max_cross,
                min_height: 0.0,
                max_height: remaining,
            },
        };
        let s = clamp_size(
            measure(&mut node.children[i], lctx, child_c),
            min_w,
            max_w,
            min_h,
            max_h,
        );
        let (m, c) = match axis {
            MainAxis::Horizontal => (s.width, s.height),
            MainAxis::Vertical => (s.height, s.width),
        };
        child_main[i] = m;
        child_cross[i] = c;
        cursor += m + gap;
    }

    // Pass 2：弹性子节点（按权重瓜分剩余空间）。
    let gaps_total = gap * n.saturating_sub(1) as f32;
    let used_nonflex: f32 = child_main
        .iter()
        .enumerate()
        .filter(|(i, _)| flexes[*i] <= 0.0)
        .map(|(_, m)| *m)
        .sum();
    if total_flex > 0.0 {
        let free = (max_main - gaps_total - used_nonflex).max(0.0);
        for i in 0..n {
            if flexes[i] <= 0.0 {
                continue;
            }
            let share = free * (flexes[i] / total_flex);
            let (min_w, max_w, min_h, max_h) = child_constraints_from_props(&node.children[i].props);
            // Flex 子节点：tight share 作为默认分配量，但 min_w/max_w 从 props 覆盖
            // 使 min_w > share 时子节点获得更大的空间（CSS min-width 语义）。
            let child_c = match axis {
                MainAxis::Horizontal => Constraints {
                    min_width: share.max(min_w),
                    max_width: share.max(min_w).min(max_w),
                    min_height: 0.0,
                    max_height: max_cross,
                },
                MainAxis::Vertical => Constraints {
                    min_width: 0.0,
                    max_width: max_cross,
                    min_height: share.max(min_w),
                    max_height: share.max(min_w).min(max_h),
                },
            };
            let s = clamp_size(
                measure(&mut node.children[i], lctx, child_c),
                min_w,
                max_w,
                min_h,
                max_h,
            );
            let (m, c) = match axis {
                MainAxis::Horizontal => (s.width, s.height),
                MainAxis::Vertical => (s.height, s.width),
            };
            child_main[i] = m;
            child_cross[i] = c;
        }
    }

    // 容器尺寸钳到 [min, max]：遵守传入约束的 min（fill-sizing），使 tight/exact 约束下容器
    // 能填满空间，为主轴对齐提供剩余空间。默认 loose（min=0）时与历史 content-sized 行为一致。
    let content_main = (child_main.iter().sum::<f32>() + gaps_total).max(0.0);
    let total_main = content_main.max(min_main).min(max_main);
    let content_cross = child_cross.iter().copied().fold(0.0_f32, f32::max).max(0.0);
    let total_cross = content_cross.max(min_cross).min(max_cross);
    match axis {
        MainAxis::Horizontal => Size::new(total_main, total_cross),
        MainAxis::Vertical => Size::new(total_cross, total_main),
    }
}

/// measure：自下而上算每节点尺寸，写入 `cached_size`，返回本节点尺寸。
fn measure(node: &mut HostNode, lctx: &mut LayoutCtx, constraints: Constraints) -> Size {
    let size = match node_container_kind(node) {
        Some(ContainerKind::Column) => measure_linear(node, lctx, constraints, MainAxis::Vertical),
        Some(ContainerKind::Row) => measure_linear(node, lctx, constraints, MainAxis::Horizontal),
        Some(ContainerKind::Stack) => {
            let mut max_w = 0.0_f32;
            let mut max_h = 0.0_f32;
            for child in node.children.iter_mut() {
                let s = measure(
                    child,
                    lctx,
                    Constraints::loose(Size::new(constraints.max_width, constraints.max_height)),
                );
                max_w = max_w.max(s.width);
                max_h = max_h.max(s.height);
            }
            Size::new(max_w.min(constraints.max_width), max_h.min(constraints.max_height))
        }
        None => {
            if let Some(w) = node.widget.as_mut() {
                w.layout(lctx, constraints)
            } else {
                // 未注册组件 & 非内置容器：填充可用空间（占位）。
                Size::new(constraints.max_width, constraints.max_height)
            }
        }
    };
    node.cached_size = size;
    size
}

/// arrange：自上而下按 `cached_size` 定每节点绝对 `cached_rect`。
fn arrange(node: &mut HostNode, origin: Point) {
    node.cached_rect = Rect::from_origin_size(origin, node.cached_size);
    match node_container_kind(node) {
        Some(ContainerKind::Column) => {
            let gap = gap_from_props(&node.props);
            let cross = cross_axis_alignment_from_props(&node.props);
            let main = main_axis_alignment_from_props(&node.props);
            let container_cross = node.cached_size.width;
            let n = node.children.len();
            // 主轴剩余空间 = 容器主轴 − 子节点尺寸和 − 最小 gap（钳到非负），按 justify-content 分布。
            let content: f32 = node.children.iter().map(|c| c.cached_size.height).sum();
            let gaps_min = gap * n.saturating_sub(1) as f32;
            let extra = (node.cached_size.height - content - gaps_min).max(0.0);
            let (main_offset, between_extra) = main_axis_layout(main, extra, n);
            let spacing = gap + between_extra;
            let mut y = origin.y + main_offset;
            for child in node.children.iter_mut() {
                let cx = cross_offset(cross, container_cross, child.cached_size.width);
                arrange(child, Point::new(origin.x + cx, y));
                y += child.cached_size.height + spacing;
            }
        }
        Some(ContainerKind::Row) => {
            let gap = gap_from_props(&node.props);
            let cross = cross_axis_alignment_from_props(&node.props);
            let main = main_axis_alignment_from_props(&node.props);
            let container_cross = node.cached_size.height;
            let n = node.children.len();
            let content: f32 = node.children.iter().map(|c| c.cached_size.width).sum();
            let gaps_min = gap * n.saturating_sub(1) as f32;
            let extra = (node.cached_size.width - content - gaps_min).max(0.0);
            let (main_offset, between_extra) = main_axis_layout(main, extra, n);
            let spacing = gap + between_extra;
            let mut x = origin.x + main_offset;
            for child in node.children.iter_mut() {
                let cy = cross_offset(cross, container_cross, child.cached_size.height);
                arrange(child, Point::new(x, origin.y + cy));
                x += child.cached_size.width + spacing;
            }
        }
        Some(ContainerKind::Stack) => {
            for child in node.children.iter_mut() {
                arrange(child, origin);
            }
        }
        None => {
            for child in node.children.iter_mut() {
                arrange(child, origin);
            }
        }
    }
}

// ---------------- paint ----------------

/// 递归遍历 widget 实例树，paint 进全局 `Scene`。
///
/// # Clip 链契约
///
/// 视口外节点（`parent_clip.intersect(cached_rect) == None`）产出 `own_clip = None`。
/// `None` 裁剪语义由后端 adapter 定义：`ui/adapters/render-foundation` 把 `None`
/// 回落为视口（viewport fallback），使节点仍可能渲染在视口内。这是**有意设计**：
/// - 根节点 `parent_clip = Some(viewport)` → 可见节点产 `Some(intersection)` →
///   不可见节点产 `None`。
/// - 若改为 `.unwrap_or(Rect::ZERO)`（视口外→零面积 clip），虽然语义更纯，
///   但 bridge stateful-clip 路径对此未经充分验证（O1 follow-up）。
/// - `Rect::intersect` 边相接（共享一条边，零面积）返 `None`（ui/core 已回归测试覆盖），
///   此时 bridge viewport fallback 使边缘节点正确渲染。
fn paint_node(
    node: &mut HostNode,
    scene: &mut Scene,
    parent_clip: Option<Rect>,
    tokens: &zero_ui_core::theme::SemanticTokens,
    font_metrics: Option<(f32, f32)>,
) {
    let own_clip = parent_clip.and_then(|pc| pc.intersect(node.cached_rect));
    // 容器节点底色：无 widget 的容器（layout=column/row/stack）若声明 `bg` prop（**token 名**，
    // 如 "surface"/"background"），先铺底色再画子节点（子节点 paint 在上）。这闭合 SDK chrome
    // 容器（如 ToolbarRow）圆角/间隙透出帧白底的问题（DC-14 toolbar parity）——手绘 chrome 先铺
    // 整行 toolbar_bg 再画 pill/按钮；SDK 容器此前不画底色，address pill 圆角处透出帧白底。
    // 颜色经 semantic token 解析（DC-5 token 驱动；chrome 别名如 "toolbar_bg" 由 shell 经 leaf
    // 控件消费，容器 bg 用通用 token 名）。
    if node.widget.is_none()
        && let Some(Value::Text(token)) = node.props.get("bg")
        && let Some(color) = tokens.color_for(token)
    {
        scene.push(SceneEntry {
            source: node.id.clone(),
            clip: own_clip,
            primitive: RenderPrimitive::FillRect {
                rect: node.cached_rect,
                color,
                rounding: Rounding::ZERO,
            },
        });
    }
    if let Some(w) = node.widget.as_mut() {
        // widget 以**节点局部坐标** paint（原点 = 节点左上角）。own_clip 是绝对坐标
        //（parent_clip ∩ cached_rect），须平移到局部坐标后传给 recorder / PaintCtx；
        // 否则随后的 `local.translated(abs_offset)` 会把 clip 再平移一次（双重平移），
        // 使 clip 偏出节点 rect → 后端 `fill_rect ∩ current_clip` 为空、chrome fills 被丢弃。
        let local_clip = own_clip.map(|c| c.translate(-node.cached_rect.origin.x, -node.cached_rect.origin.y));
        let mut rec = SceneRecorder::new(node.id.clone());
        rec.set_clip(local_clip);
        let mut ctx = PaintCtx {
            recorder: &mut rec,
            clip: local_clip,
            offset: Vec2::ZERO,
            tokens,
            font_metrics,
        };
        w.paint(&mut ctx);
        let local = rec.finish();
        let abs_offset = Vec2::new(node.cached_rect.origin.x, node.cached_rect.origin.y);
        for entry in local.translated(abs_offset).entries {
            scene.push(entry);
        }
    }
    for child in node.children.iter_mut() {
        paint_node(child, scene, own_clip, tokens, font_metrics);
    }
}

// ---------------- event dispatch ----------------

/// 向指定 id 的节点（非 hit-test 定位）派发合成 `Exited` 事件（F1 hover 追踪）。
///
/// 与 [`dispatch_node`] 不同：本函数不遍历命中测试，而是按 id 精确查找目标节点
/// 并直接派发，用以通知旧悬停节点清除交互态（pressed/hover）。
/// 无 widget 或不命中 rect 时直接返回（不报错）。
fn dispatch_to_widget(node: &mut HostNode, target: &WidgetId, phase: PointerPhase, _emitted: &mut Vec<EmittedAction>) {
    if let Some(target_node) = find_node_mut(node, target)
        && let Some(w) = target_node.widget.as_mut()
    {
        let exited = UiEvent::Pointer {
            phase,
            button: None,
            position: Point::ZERO,
            modifiers: zero_ui_core::event::Modifiers::NONE,
            pointer_id: 0,
        };
        let mut flags = InvalidationFlags::CLEAN;
        let res = w.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &exited,
        );
        target_node.invalidation |= flags;
        // Exited 是纯视觉清除（不 emit action），忽略 EventResult。
        let _ = res;
    }
}

fn dispatch_node(node: &mut HostNode, event: &UiEvent, emitted: &mut Vec<EmittedAction>) -> bool {
    // 仅路由带位置的事件（指针/滚动）；焦点/键盘路由见 DC-8。
    let Some(abs_pos) = event.position() else {
        return false;
    };
    if !node.cached_rect.contains(abs_pos) {
        return false;
    }
    // 子节点优先，倒序（后绘制 = 更上层 先命中）。
    for child in node.children.iter_mut().rev() {
        if dispatch_node(child, event, emitted) {
            return true;
        }
    }
    // 交给本节点 widget（位置转为其局部坐标）。
    if let Some(w) = node.widget.as_mut() {
        let local_event = localize_position(event, node.cached_rect.origin);
        let mut flags = InvalidationFlags::CLEAN;
        let result = w.event(
            &mut EventCtx {
                invalidation: &mut flags,
            },
            &local_event,
        );
        node.invalidation |= flags;
        match result {
            EventResult::Ignored => false,
            EventResult::Consumed => true,
            EventResult::Emit(action) => {
                emitted.push(EmittedAction { action, payload: None });
                true
            }
            EventResult::EmitWithPayload(action, payload) => {
                emitted.push(EmittedAction {
                    action,
                    payload: Some(payload),
                });
                true
            }
        }
    } else {
        false
    }
}

/// 克隆事件并把位置平移为相对 `origin` 的局部坐标（widget 内部按自身坐标系处理）。
fn localize_position(event: &UiEvent, origin: Point) -> UiEvent {
    match event {
        UiEvent::Pointer {
            phase,
            button,
            position,
            modifiers,
            pointer_id,
        } => UiEvent::Pointer {
            phase: *phase,
            button: *button,
            position: Point::new(position.x - origin.x, position.y - origin.y),
            modifiers: *modifiers,
            pointer_id: *pointer_id,
        },
        UiEvent::Scroll {
            delta,
            phase,
            position,
            modifiers,
        } => UiEvent::Scroll {
            delta: *delta,
            phase: *phase,
            position: Point::new(position.x - origin.x, position.y - origin.y),
            modifiers: *modifiers,
        },
        other => other.clone(),
    }
}

// ---------------- 查询 ----------------

fn find_rect(node: &HostNode, id: &WidgetId) -> Option<Rect> {
    if &node.id == id {
        return Some(node.cached_rect);
    }
    node.children.iter().find_map(|c| find_rect(c, id))
}

fn find_epoch(node: &HostNode, id: &WidgetId) -> Option<u32> {
    if &node.id == id {
        return Some(node.epoch);
    }
    node.children.iter().find_map(|c| find_epoch(c, id))
}

// ---------------- semantics（a11y tree，DC-8 phase-3）----------------

/// 由一个 retained 节点产出其自身 `SemanticsNode`（不含 children）。
///
/// 向 widget 索要自描述节点（`Widget::semantics` 推送一个节点），再用 host 已知信息
/// 覆盖 `id`/`rect`，并 OR 进 host 级焦点标志（FOCUSABLE/FOCUSED）。widget 未推送时
/// 合成空标志节点（仍保留 id/rect 参与树形）。
fn self_semantics(node: &HostNode, focused: Option<&WidgetId>) -> SemanticsNode {
    let mut pushed: Vec<SemanticsNode> = Vec::new();
    if let Some(w) = node.widget.as_ref() {
        w.semantics(&mut SemanticsCtx { nodes: &mut pushed });
    }
    let mut s = pushed
        .pop()
        .unwrap_or_else(|| SemanticsNode::new(node.id.clone(), node.cached_rect, SemanticsFlags::NONE));
    s.id = node.id.clone();
    s.rect = node.cached_rect;
    if node.focusable {
        s.flags |= SemanticsFlags::FOCUSABLE;
    }
    if focused == Some(&node.id) {
        s.flags |= SemanticsFlags::FOCUSED;
    }
    s
}

/// 递归构建 a11y 树：有 widget 或可聚焦的节点产出独立语义节点；纯容器节点（无 widget）
/// 把子节点合并进父级（semantics merge），避免无内容中间节点污染读屏树。
fn build_semantics(node: &HostNode, focused: Option<&WidgetId>, out: &mut Vec<SemanticsNode>) {
    if node.widget.is_some() || node.focusable {
        let mut s = self_semantics(node, focused);
        for child in &node.children {
            build_semantics(child, focused, &mut s.children);
        }
        out.push(s);
    } else {
        for child in &node.children {
            build_semantics(child, focused, out);
        }
    }
}

#[cfg(test)]
#[path = "host_tests.rs"]
mod tests;
