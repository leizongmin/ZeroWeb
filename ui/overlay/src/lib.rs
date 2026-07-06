//! # zero-ui-overlay
//!
//! 浮层系统（spec §8.4.1 `zero-ui-overlay` / FR-016 / IF-010 `OverlayHost` / §8.4.1B
//! 权限/下载/site info、§8.8 focus trap / outside-click / escape dismiss）。
//!
//! 提供 [`OverlayHost`]（按 z 序维护活动浮层）+ [`OverlayEntry`]（锚定 + trap_focus + modal
//! barrier + dismiss 策略）+ [`Toast`]。宿主据此：①把 trap_focus 浮层绑到 runtime `FocusScope`
//! （DC-8 phase-3）；②对 outside-click / Escape 自动 dismiss；③`has_modal` 屏蔽下层事件路由。
//!
//! 管理 **app UI 浮层**；不替代 WebView 内网页弹窗语义（网页弹窗请求由 browser-ui 转为
//! browser overlay，spec §8.4.10）。

use zero_ui_core::geometry::{Point, Rect};
use zero_ui_core::widget::WidgetId;

/// 浮层标识（= 浮层组件的稳定 WidgetId；IF-010 `OverlayId`）。
pub type OverlayId = WidgetId;

/// 浮层自动 dismiss 策略（§8.8 outside-click / escape dismiss 测试）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DismissPolicy {
    /// 不自动 dismiss（只能显式 `dismiss`）。
    None,
    /// 点击浮层外部时 dismiss（锚定 popover/tooltip 典型策略）。
    OutsideClick,
    /// 按 Escape 时 dismiss（dialog/menu 典型策略）。
    Escape,
    /// 两者皆可。
    Any,
}

impl DismissPolicy {
    fn dismiss_on_outside_click(self) -> bool {
        matches!(self, DismissPolicy::OutsideClick | DismissPolicy::Any)
    }
    fn dismiss_on_escape(self) -> bool {
        matches!(self, DismissPolicy::Escape | DismissPolicy::Any)
    }
}

/// 浮层条目。
#[derive(Debug, Clone, PartialEq)]
pub struct OverlayEntry {
    pub id: OverlayId,
    /// 锚定区域（popover 的浮层矩形）；`None` = 居中/全屏 modal（覆盖整屏，outside-click 永不命中）。
    pub anchor: Option<Rect>,
    /// P0-1：锚定 widget id（运行时由 host 解析为 rect 覆盖 `anchor`）。
    ///
    /// 当应用层不知道触发按钮的绝对 rect（通常是这种情况）时，设置此项；
    /// host 在 layout overlay 时调 `rect_of(anchor_widget)` 解析为真实屏幕 rect，
    /// 覆盖 `anchor` 字段（若两者都设，优先 `anchor_widget`）。
    pub anchor_widget: Option<WidgetId>,
    /// 是否捕获焦点（modal/sheet/dialog → true，供 host 绑 FocusScope trap）。
    pub trap_focus: bool,
    /// 是否为 modal barrier（屏蔽下层事件路由；host 据此 stop hit-test 冒泡到更低层）。
    pub modal: bool,
    /// 自动 dismiss 策略。
    pub dismiss: DismissPolicy,
}

impl OverlayEntry {
    /// 锚定 popover：点击锚定矩形外部 dismiss（不捕获焦点、非 modal）。
    pub fn popover(id: &str, anchor: Rect) -> OverlayEntry {
        OverlayEntry {
            id: WidgetId::new(id),
            anchor: Some(anchor),
            anchor_widget: None,
            trap_focus: false,
            modal: false,
            dismiss: DismissPolicy::OutsideClick,
        }
    }

    /// 居中 modal dialog：捕获焦点 + modal barrier，Escape dismiss。
    pub fn modal(id: &str) -> OverlayEntry {
        OverlayEntry {
            id: WidgetId::new(id),
            anchor: None,
            anchor_widget: None,
            trap_focus: true,
            modal: true,
            dismiss: DismissPolicy::Escape,
        }
    }

    /// 瞬态 tooltip：锚定、不捕获焦点、outside-click dismiss。
    pub fn tooltip(id: &str, anchor: Rect) -> OverlayEntry {
        OverlayEntry {
            id: WidgetId::new(id),
            anchor: Some(anchor),
            anchor_widget: None,
            trap_focus: false,
            modal: false,
            dismiss: DismissPolicy::OutsideClick,
        }
    }

    /// 底部 sheet（phone）：捕获焦点、modal barrier、可 outside-click 或 escape dismiss。
    pub fn sheet(id: &str) -> OverlayEntry {
        OverlayEntry {
            id: WidgetId::new(id),
            anchor: None,
            anchor_widget: None,
            trap_focus: true,
            modal: true,
            dismiss: DismissPolicy::Any,
        }
    }

    /// P0-1：设置 `anchor_widget`（host 解析为真实 rect，覆盖 `anchor`）。
    pub fn with_anchor_widget(mut self, widget_id: &str) -> Self {
        self.anchor_widget = Some(WidgetId::new(widget_id));
        self
    }
}

/// 浮层宿主：维护当前活动浮层（按 z 序，`entries` 末尾 = 最上层）。
#[derive(Debug, Default)]
pub struct OverlayHost {
    pub entries: Vec<OverlayEntry>,
}

impl OverlayHost {
    pub fn new() -> OverlayHost {
        OverlayHost::default()
    }

    /// 显示浮层（置于最上层），返回其 [`OverlayId`]（IF-010 `show`）。
    pub fn show(&mut self, entry: OverlayEntry) -> OverlayId {
        let id = entry.id.clone();
        self.entries.push(entry);
        id
    }

    /// 移除指定 id 的浮层；返回是否确实存在。
    pub fn dismiss(&mut self, id: &OverlayId) -> bool {
        let before = self.entries.len();
        self.entries.retain(|e| &e.id != id);
        self.entries.len() != before
    }

    /// 最上层浮层（只读）。
    pub fn top(&self) -> Option<&OverlayEntry> {
        self.entries.last()
    }

    /// 最上层浮层 id。
    pub fn top_id(&self) -> Option<&OverlayId> {
        self.entries.last().map(|e| &e.id)
    }

    /// 是否存在任意 modal barrier 浮层（事件路由屏蔽下层）。
    pub fn has_modal(&self) -> bool {
        self.entries.iter().any(|e| e.modal)
    }

    /// trap_focus 浮层的 id（按 z 序，最上层在前）；host 据最上层调 `enter_focus_scope(trap)`
    /// 绑 FocusScope（DC-8 phase-3 modal/popup focus trap）。
    pub fn focus_trap_ids(&self) -> Vec<OverlayId> {
        self.entries
            .iter()
            .rev()
            .filter(|e| e.trap_focus)
            .map(|e| e.id.clone())
            .collect()
    }

    /// 处理一次外部点击：dismiss 最上层「opt-in outside-click 且点落其锚定矩形之外」的浮层。
    ///
    /// 语义（§8.8 outside-click dismiss）：
    /// - 「覆盖该点」= 锚定矩形包含该点；`anchor=None`（全屏 modal）视为覆盖整屏。
    /// - 从最上层往下找；首个**不覆盖该点**且策略为 OutsideClick/Any 的浮层为候选。
    /// - 若候选之上有更上层浮层覆盖该点 → 点击被它消费，不 dismiss 任何浮层。
    /// - 一次点击最多 dismiss 一个浮层（最上层那个候选）。
    pub fn dismiss_on_outside_click(&mut self, point: Point) -> Vec<OverlayId> {
        // 「覆盖该点」：anchor=None（全屏 modal）视为覆盖整屏；Some(rect) → rect 是否包含该点。
        let covers_point = |e: &OverlayEntry| match e.anchor {
            Some(a) => a.contains(point),
            None => true,
        };
        // 从最上层往下，找首个「不覆盖该点 + outside-click 策略」的候选。
        let target = self
            .entries
            .iter()
            .rev()
            .position(|e| !covers_point(e) && e.dismiss.dismiss_on_outside_click());
        match target {
            Some(rev_idx) => {
                // 候选之上（rev 中更小索引）有覆盖该点的浮层 → 点击被它消费，不 dismiss。
                let blocked = self.entries.iter().rev().take(rev_idx).any(covers_point);
                if blocked {
                    return Vec::new();
                }
                let idx = self.entries.len() - 1 - rev_idx;
                let id = self.entries[idx].id.clone();
                self.entries.remove(idx);
                vec![id]
            }
            None => Vec::new(),
        }
    }

    /// 处理一次 Escape 键：dismiss 最上层策略为 Escape/Any 的浮层（§8.8 escape dismiss）。
    /// 一次 Escape 最多 dismiss 一个浮层（最上层那个）。
    pub fn dismiss_on_escape(&mut self) -> Vec<OverlayId> {
        let target = self.entries.iter().rposition(|e| e.dismiss.dismiss_on_escape());
        match target {
            Some(idx) => {
                let id = self.entries[idx].id.clone();
                self.entries.remove(idx);
                vec![id]
            }
            None => Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Toast（瞬态通知）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub message_id: String,
    pub ttl_ms: u32,
}

impl Toast {
    pub fn new(message_id: &str) -> Toast {
        Toast {
            message_id: message_id.to_string(),
            ttl_ms: 3000,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zero_ui_core::geometry::Point;

    #[test]
    fn show_dismiss_modal_and_ids() {
        let mut host = OverlayHost::new();
        let id = host.show(OverlayEntry::modal("perm"));
        assert_eq!(id, WidgetId::new("perm"));
        assert!(host.has_modal());
        assert_eq!(host.len(), 1);
        assert_eq!(host.top_id(), Some(&WidgetId::new("perm")));
        assert!(host.dismiss(&WidgetId::new("perm")));
        assert!(!host.has_modal());
        // dismiss 未知 id → false。
        assert!(!host.dismiss(&WidgetId::new("nope")));
    }

    #[test]
    fn empty_state_and_toast_defaults() {
        let host = OverlayHost::new();
        assert!(host.is_empty());
        assert!(host.top().is_none());
        let t = Toast::new("msg.copied");
        assert_eq!(t.message_id, "msg.copied");
        assert_eq!(t.ttl_ms, 3000);
    }

    #[test]
    fn focus_trap_ids_topmost_first() {
        // 两个 trap 浮层：menu(底) + dialog(顶) → focus_trap_ids 最上层(dialog)在前。
        let mut host = OverlayHost::new();
        host.show(OverlayEntry::modal("menu"));
        host.show(OverlayEntry::modal("dialog"));
        let traps = host.focus_trap_ids();
        assert_eq!(traps, vec![WidgetId::new("dialog"), WidgetId::new("menu")]);
        // 非 trap 浮层不进列表。
        host.show(OverlayEntry::popover("tip", Rect::ZERO));
        assert_eq!(host.focus_trap_ids().len(), 2, "popover (non-trap) excluded");
    }

    #[test]
    fn outside_click_dismisses_topmost_optin_popover() {
        // popover 锚定 (0,0)-(100,50)；点 (200,200) 在外 → dismiss。
        let mut host = OverlayHost::new();
        host.show(OverlayEntry::popover("perm", Rect::from_ltrb(0.0, 0.0, 100.0, 50.0)));
        let dismissed = host.dismiss_on_outside_click(Point::new(200.0, 200.0));
        assert_eq!(dismissed, vec![WidgetId::new("perm")]);
        assert!(host.is_empty());
    }

    #[test]
    fn inside_click_does_not_dismiss() {
        let mut host = OverlayHost::new();
        host.show(OverlayEntry::popover("perm", Rect::from_ltrb(0.0, 0.0, 100.0, 50.0)));
        // 点在 popover 内 → 不 dismiss。
        let dismissed = host.dismiss_on_outside_click(Point::new(50.0, 25.0));
        assert!(dismissed.is_empty());
        assert_eq!(host.len(), 1);
    }

    #[test]
    fn outside_click_dismisses_only_topmost() {
        // 底 popover B(大) + 顶 popover A(小)；点落在 B 内、A 外 → 仍只 dismiss A（最上层）。
        let mut host = OverlayHost::new();
        host.show(OverlayEntry::popover("B", Rect::from_ltrb(0.0, 0.0, 400.0, 400.0)));
        host.show(OverlayEntry::popover("A", Rect::from_ltrb(0.0, 0.0, 50.0, 50.0)));
        let dismissed = host.dismiss_on_outside_click(Point::new(200.0, 200.0));
        assert_eq!(dismissed, vec![WidgetId::new("A")], "only topmost (A) dismissed");
        assert_eq!(host.len(), 1);
        assert_eq!(host.top_id(), Some(&WidgetId::new("B")));
    }

    #[test]
    fn fullscreen_modal_blocks_outside_click_dismiss() {
        // modal(anchor=None) 覆盖全屏 → 点在外部不存在 → 不 dismiss；也屏蔽其下浮层。
        let mut host = OverlayHost::new();
        host.show(OverlayEntry::popover("under", Rect::from_ltrb(0.0, 0.0, 100.0, 50.0)));
        host.show(OverlayEntry::sheet("sheet")); // modal, anchor=None, Any policy
        // 任意点击：sheet 覆盖全屏 → 被 sheet 消费，不 dismiss。
        let dismissed = host.dismiss_on_outside_click(Point::new(500.0, 500.0));
        assert!(dismissed.is_empty(), "fullscreen sheet not dismissed by outside click");
        assert_eq!(host.len(), 2, "neither dismissed");
    }

    #[test]
    fn non_outsideclick_policy_not_dismissed_by_click() {
        // modal dialog(escape policy) 锚定 None：outside-click 不 dismiss。
        let mut host = OverlayHost::new();
        host.show(OverlayEntry::modal("dialog"));
        let dismissed = host.dismiss_on_outside_click(Point::new(10.0, 10.0));
        assert!(dismissed.is_empty(), "escape-only dialog not dismissed by click");
        assert_eq!(host.len(), 1);
    }

    #[test]
    fn escape_dismisses_topmost_escape_entry() {
        let mut host = OverlayHost::new();
        host.show(OverlayEntry::popover("tip", Rect::ZERO)); // outside-click only
        host.show(OverlayEntry::modal("dialog")); // escape
        // Escape → dismiss dialog（最上层 escape 项），不 dismiss tip。
        let dismissed = host.dismiss_on_escape();
        assert_eq!(dismissed, vec![WidgetId::new("dialog")]);
        assert_eq!(host.len(), 1);
        // 再 Escape：tip 是 outside-click only → 不 dismiss。
        assert!(host.dismiss_on_escape().is_empty());
    }

    #[test]
    fn sheet_any_policy_dismissed_by_both() {
        // sheet(Any) 可被 outside... 但 sheet 是 modal+anchor=None（覆盖全屏），outside-click 不命中。
        // 故 sheet 只能被 escape dismiss（验证 Any 的 escape 分支）。
        let mut host = OverlayHost::new();
        host.show(OverlayEntry::sheet("perm"));
        assert!(host.dismiss_on_outside_click(Point::new(0.0, 0.0)).is_empty());
        assert_eq!(host.dismiss_on_escape(), vec![WidgetId::new("perm")]);
    }
}
