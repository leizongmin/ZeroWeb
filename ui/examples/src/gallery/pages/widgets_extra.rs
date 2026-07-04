use crate::gallery::model::DemoPage;

pub const TOOLTIP_PAGE: DemoPage = DemoPage {
    id: "tooltip",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Tooltip",
    title_zh: "工具提示",
    description: "Hover-triggered hint label anchored to a target widget.",
    description_zh: "悬停触发的提示标签，依附于目标控件。",
    source_dsl: r#"Tooltip:
  id: back_tip
  props:
    text: "Go back"
    anchor: "nav_back""#,
    source_rust: r#"Tooltip::new("nav_back", "Go back")
    .with_delay(Duration::from_millis(300));"#,
};

pub const TOOLBAR_PAGE: DemoPage = DemoPage {
    id: "toolbar",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Toolbar",
    title_zh: "工具栏",
    description: "Horizontal action bar of IconButtons with optional overflow menu.",
    description_zh: "由 IconButton 组成的水平操作栏，可带溢出菜单。",
    source_dsl: r#"Toolbar:
  id: main_toolbar
  children:
    - IconButton: { icon: "nav-back", action: "go_back" }
    - IconButton: { icon: "nav-forward", action: "go_forward" }
    - IconButton: { icon: "reload", action: "reload" }"#,
    source_rust: r#"let mut tb = Toolbar::new("main_toolbar");
tb.push(IconButton::new("nav-back", "go_back"));
tb.push(IconButton::new("nav-forward", "go_forward"));
tb.push(IconButton::new("reload", "reload"));"#,
};

pub const POPOVER_PAGE: DemoPage = DemoPage {
    id: "popover",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Popover",
    title_zh: "弹出面板",
    description: "Non-modal anchored panel dismissed by outside click.",
    description_zh: "非模态锚定面板，点击外部关闭。",
    source_dsl: r#"Popover:
  id: share_popover
  props:
    anchor: "share_btn"
    open: false
  child:
    Column:
      children:
        - Button: { label: "Copy link", action: "share.copy" }"#,
    source_rust: r#"Popover::new("share_btn", |host| {
    host.push(Button::new("Copy link", "share.copy"));
})
.dismiss_on_outside_click(true);"#,
};

pub const POPUP_PAGE: DemoPage = DemoPage {
    id: "popup",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Popup",
    title_zh: "模态弹窗",
    description: "Modal dialog that blocks underlying UI until dismissed.",
    description_zh: "模态对话框，屏蔽底层 UI 直到关闭。",
    source_dsl: r#"Popup:
  id: confirm_popup
  props:
    modal: true
    title: "Delete file?"
  child:
    Column:
      children:
        - Button: { label: "OK", action: "popup.confirm" }
        - Button: { label: "Cancel", action: "popup.cancel" }"#,
    source_rust: r#"let popup = Popup::modal("confirm_popup")
    .title("Delete file?")
    .child(Button::new("OK", "popup.confirm"));
host.open_popup(popup);"#,
};
