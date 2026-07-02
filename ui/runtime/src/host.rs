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
use zero_ui_core::geometry::{Constraints, Point, Rect, Size, Vec2};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::semantics::{SemanticsFlags, SemanticsNode};
use zero_ui_core::widget::{
    ComponentType, EventCtx, LayoutCtx, MountCtx, PaintCtx, SemanticsCtx, Widget, WidgetId, WidgetSpec,
};
use zero_ui_gestures::{Gesture, GestureArena, PointerEvent};
use zero_ui_render::{Scene, SceneRecorder};

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

    /// 用新声明树 reconcile：按 `WidgetId` + `ComponentType` 复用既有 widget 实例（保留临时状态），
    /// props 变化时调 `Widget::update`；新增节点 mount。结构/props 变化触发 layout+paint 失效。
    pub fn set_root(&mut self, spec: &WidgetSpec) {
        self.epoch = self.epoch.wrapping_add(1);
        match &mut self.root {
            Some(root) => {
                reconcile_node(root, spec, &self.registry, self.epoch);
                // props/结构变化可能影响语义树（label/role/焦点项），标记 NEEDS_SEMANTICS（DC-8）。
                self.pending |= InvalidationFlags::NEEDS_PAINT | InvalidationFlags::NEEDS_SEMANTICS;
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
            paint_node(root, &mut self.scene, viewport, &self.tokens);
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

fn paint_node(
    node: &mut HostNode,
    scene: &mut Scene,
    parent_clip: Option<Rect>,
    tokens: &zero_ui_core::theme::SemanticTokens,
) {
    let own_clip = parent_clip.and_then(|pc| pc.intersect(node.cached_rect));
    if let Some(w) = node.widget.as_mut() {
        let mut rec = SceneRecorder::new(node.id.clone());
        rec.set_clip(own_clip);
        let mut ctx = PaintCtx {
            recorder: &mut rec,
            clip: own_clip,
            offset: Vec2::ZERO,
            tokens,
        };
        w.paint(&mut ctx);
        let local = rec.finish();
        let abs_offset = Vec2::new(node.cached_rect.origin.x, node.cached_rect.origin.y);
        for entry in local.translated(abs_offset).entries {
            scene.push(entry);
        }
    }
    for child in node.children.iter_mut() {
        paint_node(child, scene, own_clip, tokens);
    }
}

// ---------------- event dispatch ----------------

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
mod tests {
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
}
