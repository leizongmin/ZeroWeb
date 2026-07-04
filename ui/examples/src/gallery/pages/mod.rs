use crate::gallery::model::DemoPage;

mod button;
mod text_input;
mod toggle;

use button::{BUTTON_PAGE, ICON_BUTTON_PAGE};
use text_input::{TEXT_INPUT_PAGE, LIST_VIEW_PAGE, MENU_PAGE, TABS_PAGE};
use toggle::{TOGGLE_PAGE, BADGE_PAGE, PROGRESS_PAGE};

pub static ALL_PAGES: &[DemoPage] = &[
    BUTTON_PAGE,
    ICON_BUTTON_PAGE,
    TOGGLE_PAGE,
    BADGE_PAGE,
    PROGRESS_PAGE,
    TEXT_INPUT_PAGE,
    LIST_VIEW_PAGE,
    MENU_PAGE,
    TABS_PAGE,
];
