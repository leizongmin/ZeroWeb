//! # zero-ui-navigation
//!
//! 导航（spec §8.4.1 `zero-ui-navigation` / FR-016）。
//!
//! M1 提供 route stack（push/pop/replace）骨架；route 可被 restoration 恢复（spec §8.4.1B）。

use compact_str::CompactString;
use std::collections::HashMap;

/// 单条路由（name + 参数）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Route {
    pub name: CompactString,
    pub params: HashMap<CompactString, CompactString>,
}

impl Route {
    pub fn new(name: &str) -> Route {
        Route {
            name: CompactString::new(name),
            params: HashMap::new(),
        }
    }
}

/// 路由栈。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RouteStack {
    pub stack: Vec<Route>,
}

impl RouteStack {
    pub fn new(root: Route) -> RouteStack {
        RouteStack { stack: vec![root] }
    }

    pub fn top(&self) -> Option<&Route> {
        self.stack.last()
    }

    pub fn push(&mut self, route: Route) {
        self.stack.push(route);
    }

    /// 弹出顶层；保留至少一条根路由。
    pub fn pop(&mut self) -> Option<Route> {
        if self.stack.len() <= 1 { None } else { self.stack.pop() }
    }

    pub fn replace(&mut self, route: Route) {
        if let Some(top) = self.stack.last_mut() {
            *top = route;
        } else {
            self.stack.push(route);
        }
    }

    pub fn depth(&self) -> usize {
        self.stack.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_replace_keep_root() {
        let mut s = RouteStack::new(Route::new("home"));
        s.push(Route::new("settings"));
        assert_eq!(s.depth(), 2);
        assert_eq!(s.top().unwrap().name.as_str(), "settings");
        assert!(s.pop().is_some());
        assert_eq!(s.top().unwrap().name.as_str(), "home");
        // 根路由 pop 返回 None。
        assert!(s.pop().is_none());
    }

    #[test]
    fn replace_top() {
        let mut s = RouteStack::new(Route::new("home"));
        s.replace(Route::new("dashboard"));
        assert_eq!(s.top().unwrap().name.as_str(), "dashboard");
        assert_eq!(s.depth(), 1);
    }
}
