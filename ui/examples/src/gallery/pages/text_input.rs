use crate::gallery::model::DemoPage;

pub const TEXT_INPUT_PAGE: DemoPage = DemoPage {
    id: "text_input",
    group: crate::gallery::model::GroupId::Widgets,
    title: "TextInput",
    title_zh: "文本输入",
    description: "文本输入框，支持光标、选区、受控模式。",
    description_zh: "Text input with cursor, selection, controlled mode.",
    source_dsl: r#"TextInput:
  id: search_field
  props:
    placeholder: "Search..."
    value: ""#,
    source_rust: r#"let state = TextInputState::empty();
// Controlled: app holds value, re-pushes via props
field.props.insert("text", Value::Text(self.name.clone()));"#,
};

pub const LIST_VIEW_PAGE: DemoPage = DemoPage {
    id: "list_view",
    group: crate::gallery::model::GroupId::Widgets,
    title: "ListView",
    title_zh: "列表视图",
    description: "可虚拟化的列表视图，支持选择集与滚动。",
    description_zh: "Virtualizable list view with selection and scrolling.",
    source_dsl: r#"ListView:
  id: file_list
  props:
    item_count: 100
    item_height_px: 40"#,
    source_rust: r#"let mut lv = ListView::new(100);
lv.select(5);
let window = lv.visible_window();"#,
};

pub const MENU_PAGE: DemoPage = DemoPage {
    id: "menu",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Menu",
    title_zh: "菜单",
    description: "上下文/下拉菜单，支持嵌套与快捷键标识。",
    description_zh: "Context/pull-down menu with nesting and shortcut hints.",
    source_dsl: r#"Menu:
  id: file_menu
  items:
    - label: "Open"
      action: "file.open"
    - label: "Save"
      action: "file.save""#,
    source_rust: r#"Menu::new(vec![
    MenuItem::new("file.open", "Open"),
    MenuItem::new("file.save", "Save"),
]);"#,
};

pub const TABS_PAGE: DemoPage = DemoPage {
    id: "tabs",
    group: crate::gallery::model::GroupId::Widgets,
    title: "Tabs",
    title_zh: "标签页",
    description: "多标签导航，支持选中切换。",
    description_zh: "Multi-tab navigation with selection.",
    source_dsl: r#"Tabs:
  id: pref_tabs
  props:
    tabs: ["General", "Privacy", "Security"]
    selected: 0"#,
    source_rust: r#"Tabs {
    tabs: vec!["General".into(), "Privacy".into()],
    selected: Some(0),
};"#,
};
