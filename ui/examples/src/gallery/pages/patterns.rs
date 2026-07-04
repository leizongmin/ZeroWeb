use crate::gallery::model::{DemoPage, GroupId};

pub const SEARCH_FIELD_PAGE: DemoPage = DemoPage {
    id: "search_field",
    group: GroupId::Patterns,
    title: "SearchField",
    title_zh: "搜索栏",
    description: "Composite of TextInput + clear button + result dropdown scaffold.",
    description_zh: "TextInput + 清除按钮 + 结果下拉 的组合模式。",
    source_dsl: r#"SearchField:
  id: omni_search
  props:
    placeholder: "Search components..."
    debounce_ms: 200
  on_action: "search.submit""#,
    source_rust: r#"let field = SearchField::new("omni_search")
    .placeholder("Search components...")
    .debounce(Duration::from_millis(200))
    .on_submit(|q| host.emit("search.submit", q));"#,
};

pub const DATA_LIST_PAGE: DemoPage = DemoPage {
    id: "data_list",
    group: GroupId::Patterns,
    title: "DataList",
    title_zh: "数据列表",
    description: "ListView bound to a query with loading/empty/error states.",
    description_zh: "绑定查询的 ListView，自带 loading/empty/error 状态。",
    source_dsl: r#"DataList:
  id: history_list
  props:
    query: "recent_history"
    item_height_px: 40
    states: [loading, empty, error]"#,
    source_rust: r#"let list = DataList::new("history_list")
    .item_height(40.0)
    .on_state(|s| match s {
        ListState::Loading => render_spinner(),
        ListState::Empty  => render_empty_hint(),
        ListState::Error(e) => render_error(e),
        ListState::Ready(items) => render_items(items),
    });"#,
};

pub const COMMAND_PALETTE_PAGE: DemoPage = DemoPage {
    id: "command_palette",
    group: GroupId::Patterns,
    title: "CommandPalette",
    title_zh: "命令面板",
    description: "Cmd+K style fuzzy command launcher with keyboard navigation.",
    description_zh: "Cmd+K 风格的模糊命令启动器，支持键盘导航。",
    source_dsl: r#"CommandPalette:
  id: cmdk
  props:
    trigger: "Cmd+K"
    commands: ["file.open", "file.save", "go.back"]
    fuzzy: true"#,
    source_rust: r#"let palette = CommandPalette::builder("cmdk")
    .trigger(HotKey::cmd_k())
    .command("file.open", "Open File")
    .command("file.save", "Save")
    .fuzzy_match(true);"#,
};

pub const STATUS_BUBBLE_PAGE: DemoPage = DemoPage {
    id: "status_bubble",
    group: GroupId::Patterns,
    title: "StatusBubble",
    title_zh: "状态气泡",
    description: "Transient toast/notification bubble with auto-dismiss.",
    description_zh: "瞬态 toast/通知气泡，自动消失。",
    source_dsl: r#"StatusBubble:
  id: save_toast
  props:
    severity: "success"
    message: "Saved"
    duration_ms: 3000"#,
    source_rust: r#"host.toast(Toast::success("Saved")
    .duration(Duration::from_secs(3)));"#,
};

pub const TAB_BAR_PAGE: DemoPage = DemoPage {
    id: "tab_bar",
    group: GroupId::Patterns,
    title: "TabBar",
    title_zh: "标签栏",
    description: "Multi-tab bar with close buttons and reordering.",
    description_zh: "多标签栏，支持关闭按钮和拖拽重排。",
    source_dsl: r#"TabBar:
  id: browser_tabs
  props:
    tabs:
      - { id: "t1", title: "Home", closable: true }
      - { id: "t2", title: "Docs", closable: true }
    selected: "t1"
    reorderable: true"#,
    source_rust: r#"let bar = TabBar::new("browser_tabs")
    .tab(Tab::new("t1", "Home").closable())
    .tab(Tab::new("t2", "Docs").closable())
    .selected("t1")
    .reorderable(true);"#,
};

pub const DIALOG_SCAFFOLD_PAGE: DemoPage = DemoPage {
    id: "dialog_scaffold",
    group: GroupId::Patterns,
    title: "DialogScaffold",
    title_zh: "对话框脚手架",
    description: "Modal/non-modal shell with header/body/footer slots.",
    description_zh: "模态/非模态外壳，含 header/body/footer 插槽。",
    source_dsl: r#"DialogScaffold:
  id: confirm_dialog
  props:
    modal: true
  header:
    Text: { text: "Confirm" }
  body:
    Text: { text: "Are you sure?" }
  footer:
    Row:
      children:
        - Button: { label: "OK", action: "dialog.ok" }
        - Button: { label: "Cancel", action: "dialog.cancel" }"#,
    source_rust: r#"let dialog = DialogScaffold::modal("confirm_dialog")
    .header(Text::new("Confirm"))
    .body(Text::new("Are you sure?"))
    .footer(Row::new()
        .push(Button::new("OK", "dialog.ok"))
        .push(Button::new("Cancel", "dialog.cancel")));"#,
};
