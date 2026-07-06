//! 标准 prop key 常量（P1-3）。
//!
//! `PropsMap` 是 `HashMap<String, Value>`，key 字符串散落在各 widget update / layout 中
//! 容易拼错（`"lable"` vs `"label"`）且重构时难追踪。本模块集中**标准** prop key，
//! 各处用 `props.get(prop_keys::LABEL)` 替代字面量，编译期检查拼写。
//!
//! 通用 key（chrome / 容器 / SDK 自带 widget）放这里；业务 crate 自定义 key 由业务层
//! 自己定义（避免本核心 crate 知道浏览器业务概念）。

/// 文本内容（HeaderTitle / DemoTitle / SourceLabel / counter / form）。
pub const TEXT: &str = "text";
/// 按钮标签（HeaderButton / NavItem / Button）。
pub const LABEL: &str = "label";
/// 描述文字（DemoTitle.desc）。
pub const DESC: &str = "desc";
/// 主题（ThemeKind 文本形式：`"light"` / `"dark"`）。
pub const THEME: &str = "theme";
/// 语言（Locale 文本形式：`"en"` / `"zh"`）。
pub const LOCALE: &str = "locale";
/// 触发 action 的标识（HeaderButton / Button）。
pub const ACTION: &str = "action";
/// 当前页 id（NavItem / DemoPreview）。
pub const PAGE_ID: &str = "page_id";
/// 分组 id（GroupHeader，序列化进 action payload）。
pub const GROUP: &str = "group";
/// 查询文本（NavSearch）。
pub const QUERY: &str = "query";
/// 源码内容（SourceCode）。
pub const SOURCE: &str = "source";
/// 源码语言（SourceCode：`"rust"` / `"yaml"` / ...）。
pub const LANG: &str = "lang";
/// 弹性主轴方向（Spacer：`"horizontal"` / `"vertical"`）。
pub const AXIS: &str = "axis";

// ---- 状态布尔 ----
/// 选中态（NavItem / Tab 等）。
pub const SELECTED: &str = "selected";
/// 折叠态（GroupHeader / Disclosure 等）。
pub const COLLAPSED: &str = "collapsed";

// ---- 容器布局 ----
/// 容器种类（`"column"` / `"row"` / `"stack"` / `"scroll_vertical"`）。
pub const LAYOUT: &str = "layout";
/// 主轴间距（Column/Row）。
pub const GAP: &str = "gap";
/// 弹性权重（Row/Column 主轴）。
pub const FLEX: &str = "flex";
/// 主轴对齐（`"start"` / `"center"` / ...）。
pub const MAIN_AXIS_ALIGN: &str = "main_axis_align";
/// 交叉轴对齐。
pub const CROSS_AXIS_ALIGN: &str = "cross_axis_align";
/// 滚动方向（向后兼容 gallery 旧写法 `scroll=vertical`，新代码用 layout=scroll_vertical）。
pub const SCROLL: &str = "scroll";
/// 是否显示 scrollbar（ScrollVertical 容器；默认 true，设 false 关闭）。
pub const SHOW_SCROLLBAR: &str = "show_scrollbar";

// ---- 尺寸约束（子节点 props 覆盖父级 constraint）----
pub const MIN_WIDTH: &str = "min_width";
pub const MAX_WIDTH: &str = "max_width";
pub const MIN_HEIGHT: &str = "min_height";
pub const MAX_HEIGHT: &str = "max_height";

// ---- 外观 ----
/// 背景色 token 名（容器节点底色：`"surface"` / `"background"` 等，由 SemanticTokens 解析）。
pub const BG: &str = "bg";

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prop_keys_stable_strings() {
        // 常量字符串值应稳定（外部 YAML/DSL 也用这些字面量，改名要同步）。
        assert_eq!(TEXT, "text");
        assert_eq!(LABEL, "label");
        assert_eq!(THEME, "theme");
        assert_eq!(LAYOUT, "layout");
        assert_eq!(GAP, "gap");
        assert_eq!(BG, "bg");
    }
}
