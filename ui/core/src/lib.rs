//! # zero-ui-core
//!
//! 通用 UI SDK 的核心类型与协议层（浏览器无关）。
//!
//! 覆盖 spec §8.4.1 `zero-ui-core` 全部模块：
//! - [`geometry`]：点/尺寸/矩形/向量/内边距/约束/圆角。
//! - [`event`]：统一 `UiEvent`（pointer/key/scroll/focus/ime）。
//! - [`widget`]：声明树 `WidgetSpec`、`Widget` trait、上下文边界。
//! - [`element`]：retained 实例状态 `Element` tree、按 `WidgetId` 复用。
//! - [`action`]：`ActionId`/`EventResult`/`ActionRegistry`（单向数据流）。
//! - [`binding`]：`Value`/`PropsMap`/`Binding`/`StatePath`/`BindingSchema`。
//! - [`theme`]：semantic token、`Theme`、`ThemeResolver`、paint-only 失效判定。
//! - [`focus`]：焦点遍历与作用域。
//! - [`semantics`]：a11y `SemanticsNode`。
//! - [`invalidation`]：layout/paint/semantics/composite 失效标志。
//! - [`layout`]：`WindowMetrics`/`ViewportClass`/adaptive 分支。
//!
//! 不依赖任何浏览器业务 crate（DC-1）。

pub mod action;
pub mod binding;
pub mod element;
pub mod event;
pub mod focus;
pub mod geometry;
pub mod image;
pub mod invalidation;
pub mod layout;
pub mod prop_keys;
pub mod scroll;
pub mod semantics;
pub mod theme;
pub mod widget;
