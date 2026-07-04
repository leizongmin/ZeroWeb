use crate::gallery::model::DemoPage;

pub const TOGGLE_PAGE: DemoPage = DemoPage {
    id: "toggle",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Toggle",
    title_zh: "开关",
    description: "Binary toggle (on/off). Fires action on state flip.",
    description_zh: "双态开关（on/off）。点击翻转状态并发出 action。",
    source_dsl: r#"Toggle:
  id: dark_mode
  props:
    checked: false
    action: "theme.toggle"
    label: "Dark mode"#,
    source_rust: r#"let toggle = Toggle::new(false, "theme.toggle")
    .with_label("settings.dark_mode");"#,
};

pub const BADGE_PAGE: DemoPage = DemoPage {
    id: "badge",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Badge",
    title_zh: "角标",
    description: "Numeric or dot badge attached to other widgets.",
    description_zh: "数字/圆点角标，附着于其他控件之上。",
    source_dsl: r#"Badge:
  id: notify_badge
  props:
    count: 3
    max: 99"#,
    source_rust: r#"Badge::new(3).with_max(99);"#,
};

pub const PROGRESS_PAGE: DemoPage = DemoPage {
    id: "progress",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Progress",
    title_zh: "进度条",
    description: "Indeterminate or determinate progress indicator.",
    description_zh: "不确定/定值进度条。",
    source_dsl: r#"Progress:
  id: loading
  props:
    value: 0.6
    indeterminate: false"#,
    source_rust: r#"Progress { value: Some(0.6), indeterminate: false };"#,
};
