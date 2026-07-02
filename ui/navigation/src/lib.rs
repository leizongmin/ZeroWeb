//! # zero-ui-navigation
//!
//! 导航（spec §8.4.1 `zero-ui-navigation` / FR-016 / IF-010 `Navigator` / §8.4.1B route
//! 可恢复、phone bottom sheet route、权限/设置页 route）。
//!
//! 提供 [`RouteStack`]（page/modal/sheet 路由栈，push/pop/replace 返回稳定 [`RouteId`]）+
//! [`Navigator`] trait（IF-010）+ 路由恢复快照（§8.4.1B session restore）。
//!
//! 只管理 **app UI route**；不替代网页 navigation history（网页 history 由浏览器模型 + WebView
//! 负责，spec §8.4.10）。

use compact_str::CompactString;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 稳定路由标识（push 时分配的单调递增 id；用于跨重建跟踪同一 route、焦点作用域绑定、
/// 恢复点关联）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RouteId(pub u64);

/// 路由呈现方式（spec §8.4.1B：desktop popover / phone bottom sheet route / modal dialog）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RouteKind {
    /// 整页（默认）。
    Page,
    /// 模态对话框（捕获焦点、外层 barrier）。
    Modal,
    /// 底部 sheet（phone 权限/下载等；§8.4.1B）。
    Sheet,
}

impl RouteKind {
    pub fn is_overlay(&self) -> bool {
        matches!(self, RouteKind::Modal | RouteKind::Sheet)
    }
}

/// 单条路由（name + 参数 + 呈现方式）。即 spec IF-010 的 `RouteSpec`。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Route {
    pub name: CompactString,
    pub params: HashMap<CompactString, CompactString>,
    pub kind: RouteKind,
}

impl Route {
    pub fn new(name: &str) -> Route {
        Route::with_kind(name, RouteKind::Page)
    }

    /// 模态路由（dialog）。
    pub fn modal(name: &str) -> Route {
        Route::with_kind(name, RouteKind::Modal)
    }

    /// 底部 sheet 路由（phone）。
    pub fn sheet(name: &str) -> Route {
        Route::with_kind(name, RouteKind::Sheet)
    }

    pub fn with_kind(name: &str, kind: RouteKind) -> Route {
        Route {
            name: CompactString::new(name),
            params: HashMap::new(),
            kind,
        }
    }

    /// 附加参数（builder）。
    pub fn param(mut self, key: &str, value: &str) -> Route {
        self.params.insert(CompactString::new(key), CompactString::new(value));
        self
    }
}

/// 路由栈（root 不可弹出）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteStack {
    routes: Vec<Route>,
    ids: Vec<RouteId>,
    next_id: u64,
}

impl RouteStack {
    /// 空栈（无 root）；一般用 [`RouteStack::new`] 带 root。
    pub fn empty() -> RouteStack {
        RouteStack::default()
    }

    /// 以 root 路由构造（root id = 0）。
    pub fn new(root: Route) -> RouteStack {
        let mut s = RouteStack::empty();
        s.push(root);
        s
    }

    /// 压入路由，返回其稳定 [`RouteId`]。
    pub fn push(&mut self, route: Route) -> RouteId {
        let id = RouteId(self.next_id);
        self.next_id += 1;
        self.routes.push(route);
        self.ids.push(id);
        id
    }

    /// 弹出顶层；保留至少一条 root（仅剩一条时返回 None）。
    /// 返回被弹出路由的 id（路由数据不再可访问）。
    pub fn pop(&mut self) -> Option<RouteId> {
        if self.routes.len() <= 1 {
            return None;
        }
        self.routes.pop();
        self.ids.pop()
    }

    /// 替换顶层路由（分配新 id，返回之）。
    pub fn replace(&mut self, route: Route) -> RouteId {
        let id = RouteId(self.next_id);
        self.next_id += 1;
        if let Some(last) = self.routes.last_mut() {
            *last = route;
            *self.ids.last_mut().unwrap() = id;
        } else {
            self.routes.push(route);
            self.ids.push(id);
        }
        id
    }

    /// 顶层路由（只读）。
    pub fn top(&self) -> Option<&Route> {
        self.routes.last()
    }

    /// 顶层路由 id。
    pub fn top_id(&self) -> Option<RouteId> {
        self.ids.last().copied()
    }

    /// 按 id 查路由（跨重建跟踪同一 route 用）。
    pub fn route_of(&self, id: RouteId) -> Option<&Route> {
        self.ids
            .iter()
            .position(|i| *i == id)
            .and_then(|idx| self.routes.get(idx))
    }

    /// 栈深度（含 root）。
    pub fn depth(&self) -> usize {
        self.routes.len()
    }

    /// 是否为空（连 root 都没有）。
    pub fn is_empty(&self) -> bool {
        self.routes.is_empty()
    }

    /// 全部路由（按栈序，root 在前）。
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }

    /// 顶层 overlay 路由（最深的 Modal/Sheet）；用于事件屏障 / 焦点作用域绑定。
    pub fn top_overlay(&self) -> Option<&Route> {
        self.routes.iter().rev().find(|r| r.kind.is_overlay())
    }

    /// 恢复快照：路由名序列（root 在前），供 [`RestorationStore`](zero_ui_restoration::RestorationStore)
    /// 在 `route.stack` 恢复点保存（§8.4.1B session restore）。
    pub fn route_names(&self) -> Vec<&str> {
        self.routes.iter().map(|r| r.name.as_str()).collect()
    }

    /// 从已保存路由整列回填栈（反序列化路径，§8.4.1B 重启恢复）。
    /// 重新分配连续 id（root=0）；返回新栈。空切片返回空栈。
    pub fn from_routes(routes: Vec<Route>) -> RouteStack {
        let mut s = RouteStack::empty();
        for r in routes {
            s.push(r);
        }
        s
    }
}

/// IF-010 `Navigator` trait：把路由 push/pop/replace 抽象为宿主可持有的 trait 对象。
/// `RouteStack` 实现之；方法签名与 inherent 方法一致。
pub trait Navigator {
    fn push(&mut self, route: Route) -> RouteId;
    fn pop(&mut self) -> Option<RouteId>;
    fn replace(&mut self, route: Route) -> RouteId;
}

impl Navigator for RouteStack {
    fn push(&mut self, route: Route) -> RouteId {
        RouteStack::push(self, route)
    }
    fn pop(&mut self) -> Option<RouteId> {
        RouteStack::pop(self)
    }
    fn replace(&mut self, route: Route) -> RouteId {
        RouteStack::replace(self, route)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_replace_keep_root() {
        let mut s = RouteStack::new(Route::new("home"));
        let settings_id = s.push(Route::new("settings"));
        assert_eq!(s.depth(), 2);
        assert_eq!(s.top().unwrap().name.as_str(), "settings");
        assert_eq!(s.top_id(), Some(settings_id));
        assert_eq!(s.pop(), Some(settings_id));
        assert_eq!(s.top().unwrap().name.as_str(), "home");
        // 根路由 pop 返回 None。
        assert!(s.pop().is_none());
    }

    #[test]
    fn replace_top_assigns_new_id() {
        let mut s = RouteStack::new(Route::new("home"));
        let first = s.push(Route::new("a"));
        let replaced = s.replace(Route::new("dashboard"));
        assert_ne!(first, replaced, "replace gives a fresh RouteId");
        assert_eq!(s.top().unwrap().name.as_str(), "dashboard");
        assert_eq!(s.depth(), 2, "replace keeps depth");
        assert_eq!(s.top_id(), Some(replaced));
    }

    #[test]
    fn route_of_looks_up_by_stable_id() {
        let mut s = RouteStack::new(Route::new("home"));
        let a = s.push(Route::new("a").param("q", "1"));
        let _b = s.push(Route::new("b"));
        // 跨 pop 仍可查（只要未被弹出）。
        assert_eq!(s.route_of(a).unwrap().name.as_str(), "a");
        assert_eq!(s.route_of(a).unwrap().params.get("q").unwrap().as_str(), "1");
        s.pop(); // 弹出 b
        // a 仍在栈中。
        assert_eq!(s.route_of(a).unwrap().name.as_str(), "a");
        s.pop(); // 弹出 a
        assert!(s.route_of(a).is_none(), "popped route no longer reachable");
    }

    #[test]
    fn modal_sheet_routes_and_top_overlay() {
        let mut s = RouteStack::new(Route::new("home"));
        s.push(Route::new("settings"));
        s.push(Route::sheet("permission")); // phone bottom sheet
        // top_overlay = 最深的 overlay = permission sheet。
        let ov = s.top_overlay().unwrap();
        assert_eq!(ov.name.as_str(), "permission");
        assert_eq!(ov.kind, RouteKind::Sheet);

        // modal 更深 → top_overlay = modal。
        s.push(Route::modal("confirm"));
        assert_eq!(s.top_overlay().unwrap().name.as_str(), "confirm");
        assert_eq!(s.top_overlay().unwrap().kind, RouteKind::Modal);

        // 全 page 时无 overlay。
        let mut pages = RouteStack::new(Route::new("home"));
        pages.push(Route::new("settings"));
        assert!(pages.top_overlay().is_none());
    }

    #[test]
    fn navigator_trait_object() {
        // IF-010：宿主持 &mut dyn Navigator，仅暴露 push/pop/replace。
        let mut nav: Box<dyn Navigator> = Box::new(RouteStack::new(Route::new("home")));
        let id = nav.push(Route::new("settings"));
        assert_eq!(nav.pop(), Some(id));
        // trait replace 返回新 id（与已弹出的 id 不同；每次 replace 分配新 id）。
        let replaced = nav.replace(Route::new("dashboard"));
        assert_ne!(replaced, id);
        let replaced2 = nav.replace(Route::new("x"));
        assert_ne!(replaced, replaced2, "each replace yields a fresh RouteId");
    }

    #[test]
    fn route_names_snapshot_for_restoration() {
        let mut s = RouteStack::new(Route::new("home"));
        s.push(Route::new("settings"));
        s.push(Route::modal("confirm"));
        assert_eq!(s.route_names(), vec!["home", "settings", "confirm"]);
    }

    #[test]
    fn from_routes_roundtrip_for_restart_restore() {
        // §8.4.1B：重启后从保存的路由序列回填栈。
        let saved = vec![Route::new("home"), Route::new("settings"), Route::sheet("permission")];
        let restored = RouteStack::from_routes(saved);
        assert_eq!(restored.depth(), 3);
        assert_eq!(restored.route_names(), vec!["home", "settings", "permission"]);
        assert_eq!(restored.top().unwrap().kind, RouteKind::Sheet);
        // 重新分配的连续 id（root=0）。
        assert_eq!(restored.top_id(), Some(RouteId(2)));
        // 空 → 空栈。
        assert!(RouteStack::from_routes(Vec::new()).is_empty());
    }

    #[test]
    fn route_serde_roundtrip() {
        // Route 可序列化（供 host 用 RestorationStore 持久化整条路由含参数/kind）。
        let r = Route::sheet("permission").param("origin", "tab.0");
        let json = serde_json::to_string(&r).unwrap();
        let back: Route = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
        assert_eq!(back.kind, RouteKind::Sheet);
        assert_eq!(back.params.get("origin").unwrap().as_str(), "tab.0");
    }
}
