pub type PageId = &'static str;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupId {
    Widgets,
    Patterns,
    Forms,
    Gestures,
    Animation,
    Collections,
    Theme,
    I18n,
    Dsl,
    Navigation,
}

impl GroupId {
    pub fn name_en(&self) -> &'static str {
        match self {
            GroupId::Widgets => "Widgets",
            GroupId::Patterns => "Patterns",
            GroupId::Forms => "Forms",
            GroupId::Gestures => "Gestures",
            GroupId::Animation => "Animation",
            GroupId::Collections => "Collections",
            GroupId::Theme => "Theme",
            GroupId::I18n => "i18n",
            GroupId::Dsl => "DSL",
            GroupId::Navigation => "Navigation",
        }
    }

    pub fn name_zh(&self) -> &'static str {
        match self {
            GroupId::Widgets => "控件",
            GroupId::Patterns => "组合模式",
            GroupId::Forms => "表单",
            GroupId::Gestures => "手势",
            GroupId::Animation => "动画",
            GroupId::Collections => "集合",
            GroupId::Theme => "主题",
            GroupId::I18n => "国际化",
            GroupId::Dsl => "DSL",
            GroupId::Navigation => "导航",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    Zh,
}

impl Locale {
    pub fn toggle(self) -> Locale {
        match self {
            Locale::En => Locale::Zh,
            Locale::Zh => Locale::En,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Locale::En => "EN",
            Locale::Zh => "中文",
        }
    }
}

pub struct DemoPage {
    pub id: PageId,
    pub group: GroupId,
    pub title: &'static str,
    pub title_zh: &'static str,
    pub description: &'static str,
    pub description_zh: &'static str,
    pub source_dsl: &'static str,
    pub source_rust: &'static str,
}

impl DemoPage {
    pub fn title_for(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::En => self.title,
            Locale::Zh => self.title_zh,
        }
    }

    pub fn description_for(&self, locale: Locale) -> &'static str {
        match locale {
            Locale::En => self.description,
            Locale::Zh => self.description_zh,
        }
    }
}
