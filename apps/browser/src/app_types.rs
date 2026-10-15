// BrowserApp 相关状态类型定义（ContentPointerDrag / ScrollbarDrag / TabFetchState /
// WindowChromeAction / TabDragState / AutocompleteState / ContextMenuState）。
// 从 app.rs 拆分以控制单文件体积，经 `include!` 文本包含进 app.rs 模块作用域，
// 与 app_render_geometry.rs 等同模式（pub/私有字段、impl 直接可达，无可见性变化）。

/// 页面内容区指针拖拽（鼠标左键；RDP/远程桌面触摸常模拟为此路径）
struct ContentPointerDrag {
    start_x: f64,
    start_y: f64,
    last_y: f64,
    scrolling: bool,
}

/// 滚动条滑块拖拽。
#[derive(Clone, Copy)]
struct ScrollbarDrag {
    tab_id: TabId,
    axis: page_scroll::ScrollbarAxis,
    grab_offset: f32,
}

/// 标签页 URL 加载状态（先绘制 loading，再发起 worker 加载）。
enum TabFetchState {
    None,
    WaitingPaint(TabId, String),
}

/// 自定义窗口控制按钮动作（Wayland 无系统装饰时使用）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowChromeAction {
    Minimize,
    ToggleMaximize,
    ToggleFullscreen,
    Close,
    StartDrag,
}

/// 标签拖拽状态。鼠标按下标签后，移动超过阈值即进入拖拽，
/// 释放时按位置重排序。
#[derive(Debug, Clone, Copy)]
pub struct TabDragState {
    pub tab_id: TabId,
    /// 鼠标按下时的物理 x 坐标。
    pub press_x: f32,
    /// 标签在按下时左边缘的物理 x 坐标。
    pub tab_origin_x: f32,
    /// 标签宽度（按下时记录）。
    pub tab_w: f32,
    /// 当前鼠标物理 x 坐标。
    pub current_x: f32,
    /// 是否已越过拖拽阈值（进入实际拖拽）。
    pub active: bool,
}

impl BrowserApp {
    /// 拖拽让位目标位移表（视觉层，不改变 move_tab 重排语义）。
    ///
    /// 目标 index 用与 `finish_tab_drag` 同一规则求出（鼠标 x 对原槽位中点，
    /// tab_layout 记录未含让位位移的稳定槽位），被跨过的兄弟向空出的槽位方向
    /// 让出一格——松手落位与拖拽中显示的排列一致。
    fn tab_drag_yield_targets(&self) -> HashMap<TabId, f32> {
        let mut targets = HashMap::new();
        let Some(drag) = self.tab_drag.as_ref().filter(|drag| drag.active) else {
            return targets;
        };
        let Some(drag_index) = self.tab_layout.iter().position(|(id, _, _)| *id == drag.tab_id) else {
            return targets;
        };
        let Some((_, _, drag_w)) = self.tab_layout.get(drag_index).copied() else {
            return targets;
        };
        let mut target_index = self.tab_layout.len().saturating_sub(1);
        for (i, &(_id, tx, tw)) in self.tab_layout.iter().enumerate() {
            if drag.current_x < tx + tw * 0.5 {
                target_index = i;
                break;
            }
        }
        if target_index < drag_index {
            // 左移：[target, drag) 的兄弟右移一格让位。
            for &(id, _, _) in &self.tab_layout[target_index..drag_index] {
                targets.insert(id, drag_w);
            }
        } else if target_index > drag_index {
            // 右移：(drag, target] 的兄弟左移一格让位。
            for &(id, _, _) in &self.tab_layout[drag_index + 1..=target_index] {
                targets.insert(id, -drag_w);
            }
        }
        targets
    }

    /// 朝目标位移推进让位动画（指数平滑，按帧时长换算系数）。
    /// 返回是否未收敛（未收敛时渲染帧置 needs_redraw 维持连续帧）。
    fn advance_tab_drag_yield(&mut self) -> bool {
        if !self.tab_drag.as_ref().is_some_and(|drag| drag.active) {
            self.tab_drag_paint_offsets.clear();
            self.tab_drag_anim_last = None;
            return false;
        }
        let targets = self.tab_drag_yield_targets();
        let now = Instant::now();
        let dt = self
            .tab_drag_anim_last
            .replace(now)
            .map_or(0.016, |last| now.duration_since(last).as_secs_f32());
        // 时间常数 ~45ms：肉眼平滑且停顿后续帧快速跟上（dt 大 → k→1）。
        let k = 1.0 - (-dt * 22.0).exp();
        const SETTLE_EPSILON: f32 = 0.5;
        let mut settled = true;
        let ids: Vec<TabId> = self
            .tab_drag_paint_offsets
            .keys()
            .copied()
            .chain(targets.keys().copied())
            .collect();
        for id in ids {
            let target = targets.get(&id).copied().unwrap_or(0.0);
            let current = self.tab_drag_paint_offsets.get(&id).copied().unwrap_or(0.0);
            let mut next = current + (target - current) * k;
            if (target - next).abs() <= SETTLE_EPSILON {
                next = target;
            } else {
                settled = false;
            }
            self.tab_drag_paint_offsets.insert(id, next);
        }
        settled
    }
}

/// 自动补全建议缓存
struct AutocompleteState {
    /// 当前显示的建议列表
    suggestions: Vec<zero_browser_shell::Suggestion>,
    /// 鼠标悬停的索引
    hovered_index: Option<usize>,
    /// 键盘选中的索引
    selected_index: Option<usize>,
}

impl AutocompleteState {
    fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            hovered_index: None,
            selected_index: None,
        }
    }

    fn clear(&mut self) {
        self.suggestions.clear();
        self.hovered_index = None;
        self.selected_index = None;
    }

    fn highlight_index(&self) -> Option<usize> {
        self.hovered_index.or(self.selected_index)
    }
}

/// 右键上下文菜单状态
pub struct ContextMenuState {
    /// 是否显示
    pub visible: bool,
    /// 菜单类型（预留用于区分不同场景的菜单行为）
    #[allow(dead_code)]
    pub context_type: ContextType,
    /// 菜单项（消费 browser-shell 的 MenuItem 模型，含 separator/disabled/icon/submenu）。
    pub items: Vec<zero_browser_shell::MenuItem>,
    /// 悬停索引
    pub hovered_index: Option<usize>,
    /// 分段行（MenuItemType::Segmented，如 Zoom 行）内悬停/按下的段索引；
    /// 非分段行或键盘导航时为 None。
    pub hovered_segment: Option<usize>,
    /// 当前展开的子菜单父项索引（None 表示无子菜单展开）。
    pub open_sub_menu: Option<usize>,
    /// 子菜单内悬停的子项索引。
    pub sub_menu_hovered: Option<usize>,
    /// 菜单左上角物理像素坐标
    pub x: f32,
    pub y: f32,
    /// 打开菜单时的源标签页。
    pub source_tab_id: Option<TabId>,
    /// 页面内容区文档坐标（审查元素用）。
    pub page_doc_x: f32,
    pub page_doc_y: f32,
    /// 书签栏右键菜单的目标书签 URL（仅书签上下文菜单使用）。
    pub bookmark_url: Option<String>,
    /// 书签栏右键菜单的目标书签标题（仅书签上下文菜单使用）。
    pub bookmark_title: Option<String>,
    /// 图片右键菜单的目标图片 URL（绝对化后的 src）。
    pub image_url: Option<String>,
    /// 链接右键菜单的目标链接 URL（绝对化后的 href）。
    pub link_url: Option<String>,
}

impl ContextMenuState {
    fn new() -> Self {
        Self {
            visible: false,
            context_type: ContextType::Page,
            items: Vec::new(),
            hovered_index: None,
            hovered_segment: None,
            open_sub_menu: None,
            sub_menu_hovered: None,
            x: 0.0,
            y: 0.0,
            source_tab_id: None,
            page_doc_x: 0.0,
            page_doc_y: 0.0,
            bookmark_url: None,
            bookmark_title: None,
            image_url: None,
            link_url: None,
        }
    }

    fn close(&mut self) {
        self.visible = false;
        self.items.clear();
        self.hovered_index = None;
        self.hovered_segment = None;
        self.open_sub_menu = None;
        self.sub_menu_hovered = None;
        self.source_tab_id = None;
        self.bookmark_url = None;
        self.bookmark_title = None;
        self.image_url = None;
        self.link_url = None;
    }
}
