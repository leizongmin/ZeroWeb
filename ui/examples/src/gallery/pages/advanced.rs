use crate::gallery::model::{DemoPage, GroupId};

pub const FORM_DEMO_PAGE: DemoPage = DemoPage {
    id: "form_demo",
    group: GroupId::Forms,
    title: "FormDemo",
    title_zh: "表单示例",
    description: "Composite form with validation, field grouping, and submit flow.",
    description_zh: "组合表单，含校验、字段分组和提交流程。",
    source_dsl: r#"Form:
  id: login_form
  fields:
    - name: email
      type: text
      validator: email
    - name: password
      type: password
      validator: min_length(8)
  submit: { action: "auth.login", label: "Sign in" }"#,
    source_rust: r#"let form = Form::new("login_form")
    .field(TextInput::new("email").validator(validators::email()))
    .field(TextInput::password("password")
        .validator(validators::min_length(8)))
    .on_submit(|v| host.emit("auth.login", v));"#,
};

pub const GESTURE_DEMO_PAGE: DemoPage = DemoPage {
    id: "gesture_demo",
    group: GroupId::Gestures,
    title: "Gestures",
    title_zh: "手势",
    description: "Tap/long-press/pan/swipe/pinch recognition and bubbling.",
    description_zh: "点按/长按/拖动/滑动/捏合 识别与冒泡。",
    source_dsl: r#"Container:
  id: gesture_pad
  gestures:
    tap: "pad.tap"
    long_press: { action: "pad.long_press", min_ms: 500 }
    pan: { action: "pad.pan", multi: true }
    swipe: { action: "pad.swipe", threshold_px: 40 }"#,
    source_rust: r#"host.gestures("gesture_pad", GestureMap::new()
    .on_tap(|p| host.emit("pad.tap", p))
    .on_long_press(Duration::from_millis(500), || ...)
    .on_pan(|d| host.emit("pad.pan", d))
    .on_swipe(40.0, |dir| host.emit("pad.swipe", dir)));"#,
};

pub const ANIMATION_DEMO_PAGE: DemoPage = DemoPage {
    id: "animation_demo",
    group: GroupId::Animation,
    title: "Animation",
    title_zh: "动画",
    description: "Tween/spring curves, keyframes, and target-driven animations.",
    description_zh: "缓动/弹簧曲线、关键帧和目标驱动动画。",
    source_dsl: r#"Animation:
  id: fade_in
  target: "dialog"
  keyframes:
    - { t: 0.0, opacity: 0.0 }
    - { t: 1.0, opacity: 1.0 }
  duration_ms: 200
  curve: ease_out"#,
    source_rust: r#"host.animate("dialog", Tween::opacity(0.0, 1.0)
    .duration(Duration::from_millis(200))
    .curve(Curve::EaseOut));"#,
};

pub const COLLECTION_DEMO_PAGE: DemoPage = DemoPage {
    id: "collection_demo",
    group: GroupId::Collections,
    title: "Collections",
    title_zh: "集合",
    description: "Virtualizable grid/list with selection and grouping.",
    description_zh: "可虚拟化的网格/列表，支持选择和分组。",
    source_dsl: r#"Collection:
  id: thumb_grid
  props:
    layout: grid
    columns: 4
    item_count: 200
    selection: single
    group_by: "category""#,
    source_rust: r#"let grid = Collection::grid("thumb_grid")
    .columns(4)
    .items(200)
    .selection(SelectionMode::Single)
    .group_by("category");"#,
};

pub const THEME_DEMO_PAGE: DemoPage = DemoPage {
    id: "theme_demo",
    group: GroupId::Theme,
    title: "ThemeDemo",
    title_zh: "主题示例",
    description: "Light/dark SemanticTokens swatches and live switching.",
    description_zh: "浅色/深色 SemanticTokens 色板和实时切换。",
    source_dsl: r#"ThemeDemo:
  id: theme_swatches
  props:
    show_tokens: [background, surface, primary, on_primary, error]
    live_switch: true"#,
    source_rust: r#"let view = ThemeSwatch::new(tokens)
    .show(TOKENS_TO_DEMO)
    .on_toggle(|t| host.set_theme(t));"#,
};

pub const I18N_DEMO_PAGE: DemoPage = DemoPage {
    id: "i18n_demo",
    group: GroupId::I18n,
    title: "I18nDemo",
    title_zh: "国际化示例",
    description: "Locale switching flows through widgets and bidi text.",
    description_zh: "locale 切换流转到各控件，含 bidi 文本。",
    source_dsl: r#"I18nDemo:
  id: locale_demo
  props:
    locales: [en, zh]
    bidi_sample: "مرحبا / שלום"
    locale_prop_path: "*""#,
    source_rust: r#"let view = I18nDemo::new()
    .locale(Locale::Zh)
    .bidi_sample("مرحبا / שלום");"#,
};

pub const DSL_DEMO_PAGE: DemoPage = DemoPage {
    id: "dsl_demo",
    group: GroupId::Dsl,
    title: "DslDemo",
    title_zh: "DSL 示例",
    description: "Side-by-side YAML DSL and Rust API building the same tree.",
    description_zh: "YAML DSL 与 Rust API 并排，构建同一棵树。",
    source_dsl: r#"# YAML DSL
Row:
  id: header
  children:
    - Text: { text: "Hi" }
    - Spacer: {}
    - IconButton: { icon: "close", action: "win.close" }"#,
    source_rust: r#"// Rust API
let row = WidgetSpec::new("Row")
    .child(TextSpec::new("Hi"))
    .child(WidgetSpec::new("Spacer"))
    .child(IconButton::new("close", "win.close"));"#,
};

pub const NAV_DEMO_PAGE: DemoPage = DemoPage {
    id: "nav_demo",
    group: GroupId::Navigation,
    title: "NavigationDemo",
    title_zh: "导航示例",
    description: "Stack/tab/split navigation patterns and transitions.",
    description_zh: "Stack/Tab/Split 导航模式与转场。",
    source_dsl: r#"NavigationStack:
  id: root_nav
  initial: "home"
  routes:
    home:    { push: HomeScreen }
    detail:  { push: DetailScreen, transition: slide }
    modal:   { present: ModalScreen, transition: fade }"#,
    source_rust: r#"let nav = NavigationStack::new("root_nav")
    .initial("home")
    .route("home", Screen::home())
    .route("detail", Screen::detail())
        .transition(Transition::Slide)
    .present("modal", Screen::modal())
        .transition(Transition::Fade);"#,
};
