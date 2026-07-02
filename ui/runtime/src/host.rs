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

use compact_str::CompactString;
use zero_ui_core::action::{ActionId, ActionPayload, EventResult};
use zero_ui_core::binding::{PropsMap, Value};
use zero_ui_core::event::UiEvent;
use zero_ui_core::geometry::{Constraints, Point, Rect, Size, Vec2};
use zero_ui_core::invalidation::InvalidationFlags;
use zero_ui_core::widget::{ComponentType, EventCtx, LayoutCtx, MountCtx, PaintCtx, Widget, WidgetId, WidgetSpec};
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
}

/// Retained widget host —— 三棵树运行态的驱动器。
pub struct WidgetHost {
    registry: WidgetRegistry,
    root: Option<HostNode>,
    scene: Scene,
    epoch: u32,
    pending: InvalidationFlags,
}

impl Default for WidgetHost {
    fn default() -> WidgetHost {
        WidgetHost {
            registry: WidgetRegistry::new(),
            root: None,
            scene: Scene::new(),
            epoch: 0,
            pending: InvalidationFlags::CLEAN,
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

    /// 用新声明树 reconcile：按 `WidgetId` + `ComponentType` 复用既有 widget 实例（保留临时状态），
    /// props 变化时调 `Widget::update`；新增节点 mount。结构/props 变化触发 layout+paint 失效。
    pub fn set_root(&mut self, spec: &WidgetSpec) {
        self.epoch = self.epoch.wrapping_add(1);
        match &mut self.root {
            Some(root) => {
                reconcile_node(root, spec, &self.registry, self.epoch);
                self.pending |= InvalidationFlags::NEEDS_PAINT;
            }
            None => {
                self.root = Some(build_node(spec, &self.registry, self.epoch));
                self.pending |= InvalidationFlags::NEEDS_LAYOUT | InvalidationFlags::NEEDS_PAINT;
            }
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
            paint_node(root, &mut self.scene, viewport);
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

    /// 派发带位置的输入事件：hit-test 到最深的、最上层命中节点，冒泡派发；
    /// 收集 widget 发出的 action。任何 action 发出都标记 `NEEDS_PAINT`（hover/pressed 变化）。
    ///
    /// 非位置事件（键盘/焦点/IME）不在本 host 路由——焦点路由见 DC-8 / `ui/runtime::ime`。
    pub fn dispatch_event(&mut self, event: &UiEvent) -> Vec<EmittedAction> {
        let mut emitted = Vec::new();
        let Some(root) = self.root.as_mut() else {
            return emitted;
        };
        let handled = dispatch_node(root, event, &mut emitted);
        if handled {
            self.pending |= InvalidationFlags::NEEDS_PAINT;
        }
        emitted
    }
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
    };
    if let Some(mut w) = registry.build(spec) {
        let mut flags = InvalidationFlags::CLEAN;
        w.mount(&mut MountCtx {
            id: &node.id,
            invalidation: &mut flags,
        });
        node.invalidation |= flags;
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

/// measure：自下而上算每节点尺寸，写入 `cached_size`，返回本节点尺寸。
fn measure(node: &mut HostNode, lctx: &mut LayoutCtx, constraints: Constraints) -> Size {
    let size = match node_container_kind(node) {
        Some(ContainerKind::Column) => {
            let gap = gap_from_props(&node.props);
            let mut cursor_y = 0.0_f32;
            let mut max_w = 0.0_f32;
            for child in node.children.iter_mut() {
                let remaining_h = (constraints.max_height - cursor_y).max(0.0);
                let child_c = Constraints {
                    min_width: 0.0,
                    max_width: constraints.max_width,
                    min_height: 0.0,
                    max_height: remaining_h,
                };
                let s = measure(child, lctx, child_c);
                cursor_y += s.height + gap;
                max_w = max_w.max(s.width);
            }
            let h = (cursor_y - gap).max(0.0);
            Size::new(max_w.min(constraints.max_width), h.min(constraints.max_height))
        }
        Some(ContainerKind::Row) => {
            let gap = gap_from_props(&node.props);
            let mut cursor_x = 0.0_f32;
            let mut max_h = 0.0_f32;
            for child in node.children.iter_mut() {
                let remaining_w = (constraints.max_width - cursor_x).max(0.0);
                let child_c = Constraints {
                    min_width: 0.0,
                    max_width: remaining_w,
                    min_height: 0.0,
                    max_height: constraints.max_height,
                };
                let s = measure(child, lctx, child_c);
                cursor_x += s.width + gap;
                max_h = max_h.max(s.height);
            }
            let w = (cursor_x - gap).max(0.0);
            Size::new(w.min(constraints.max_width), max_h.min(constraints.max_height))
        }
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
            let mut y = origin.y;
            for child in node.children.iter_mut() {
                arrange(child, Point::new(origin.x, y));
                y += child.cached_size.height + gap;
            }
        }
        Some(ContainerKind::Row) => {
            let gap = gap_from_props(&node.props);
            let mut x = origin.x;
            for child in node.children.iter_mut() {
                arrange(child, Point::new(x, origin.y));
                x += child.cached_size.width + gap;
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

fn paint_node(node: &mut HostNode, scene: &mut Scene, parent_clip: Option<Rect>) {
    let own_clip = parent_clip.and_then(|pc| pc.intersect(node.cached_rect));
    if let Some(w) = node.widget.as_mut() {
        let mut rec = SceneRecorder::new(node.id.clone());
        rec.set_clip(own_clip);
        let mut ctx = PaintCtx {
            recorder: &mut rec,
            clip: own_clip,
            offset: Vec2::ZERO,
        };
        w.paint(&mut ctx);
        let local = rec.finish();
        let abs_offset = Vec2::new(node.cached_rect.origin.x, node.cached_rect.origin.y);
        for entry in local.translated(abs_offset).entries {
            scene.push(entry);
        }
    }
    for child in node.children.iter_mut() {
        paint_node(child, scene, own_clip);
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
        } => UiEvent::Pointer {
            phase: *phase,
            button: *button,
            position: Point::new(position.x - origin.x, position.y - origin.y),
            modifiers: *modifiers,
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

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::binding::Value;
    use zero_ui_core::event::{Modifiers, PointerButton, PointerPhase};
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
}
