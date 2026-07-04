use crate::gallery::model::DemoPage;

pub const BUTTON_PAGE: DemoPage = DemoPage {
    id: "button",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Button",
    title_zh: "按钮",
    description: "Clickable button with hover/pressed/disabled states. Emits Action on click.",
    description_zh: "可点击按钮，支持 hover/pressed/disabled 状态。点击发出 Action，由应用层处理。",
    source_dsl: r#"Button:
  id: my_button
  props:
    label: "Click me"
    action: "button.clicked"
    enabled: true"#,
    source_rust: r#"let btn = WidgetSpec::new("Button")
    .with_prop("label", Value::Text("Click me"))
    .with_prop("action", Value::Text("button.clicked"));
host.register("Button", |spec| {
    Button::new(ButtonSpec {
        label: str_prop(spec, "label"),
        action: ActionId::new(&str_prop(spec, "action")),
        enabled: true,
    })
});"#,
};

pub const ICON_BUTTON_PAGE: DemoPage = DemoPage {
    id: "icon_button",
    group: crate::gallery::model::GroupId::Widgets,
    title: "IconButton",
    title_zh: "图标按钮",
    description: "Button with icon identifier for navigation, menu triggers, etc.",
    description_zh: "带图标标识的按钮，导航按钮、菜单触发等用。",
    source_dsl: r#"IconButton:
  id: nav_back
  props:
    icon: "nav-back"
    action: "browser.go_back"
    enabled: true"#,
    source_rust: r#"let btn = IconButton::new("nav-back", "browser.go_back")
    .with_tooltip("Go back")
    .disabled();"#,
};
