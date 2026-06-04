//! 内置 WPT 测试用例定义。
//!
//! 测试用例按逻辑分组拆分为多个子模块：
//!
//! - `test_cases_core`: 原始 20 + CSS 基础测试
//! - `test_cases_html_layout`: HTML/DOM 结构 + 布局 + 错误恢复测试
//! - `test_cases_css_extended`: CSS 扩展 + HTML/Form + 布局扩展测试
//! - `test_cases_wpt`: WPT 扩展测试
//! - `test_cases_standard`: 标准合规性扩展测试
//! - `test_cases_dom_api`: DOM Level 2+ API 标准合规性测试
//! - `test_cases_css_compliance`: CSS 选择器和属性标准合规性测试
//! - `test_cases_js_dom`: JavaScript/DOM 交互标准合规性测试
//! - `test_cases_navigation`: 导航、安全与存储标准合规性测试
//! - `test_cases_es_modules`: ES Module 和 Web Worker 标准合规性测试
//! - `test_cases_canvas`: Canvas 2D API 标准合规性测试
//! - `test_cases_storage`: Storage 和 Web Worker 标准合规性测试
//! - `test_cases_geometry`: 精确布局几何测试
//! - `test_cases_web_api`: Web API 标准合规性测试（Fetch/WebSocket/Performance/Observers）
//! - `test_cases_security`: 安全策略标准合规性测试（CSP/CORS/Sandbox/SOP）
//! - `test_cases_a11y_i18n`: 可访问性和国际化测试（ARIA/CJK/RTL/Unicode）

mod test_cases_a11y_i18n;
mod test_cases_canvas;
mod test_cases_core;
mod test_cases_css_compliance;
mod test_cases_css_extended;
mod test_cases_css_layout;
mod test_cases_dom_api;
mod test_cases_es_modules;
mod test_cases_geometry;
mod test_cases_html_layout;
mod test_cases_js_dom;
mod test_cases_navigation;
mod test_cases_security;
mod test_cases_standard;
mod test_cases_storage;
mod test_cases_web_api;
mod test_cases_web_platform;
mod test_cases_wpt;

use super::TestCase;

/// 返回所有内置测试用例。
pub fn builtin_tests() -> Vec<TestCase> {
    let mut tests = Vec::new();
    tests.extend(test_cases_core::core_tests());
    tests.extend(test_cases_html_layout::html_layout_tests());
    tests.extend(test_cases_css_extended::css_extended_tests());
    tests.extend(test_cases_wpt::wpt_expansion_tests());
    tests.extend(test_cases_standard::standard_compliance_tests());
    tests.extend(test_cases_dom_api::dom_api_tests());
    tests.extend(test_cases_css_compliance::css_compliance_tests());
    tests.extend(test_cases_css_layout::css_layout_compliance_tests());
    tests.extend(test_cases_js_dom::js_dom_tests());
    tests.extend(test_cases_navigation::navigation_security_tests());
    tests.extend(test_cases_es_modules::es_module_and_worker_tests());
    tests.extend(test_cases_canvas::canvas_compliance_tests());
    tests.extend(test_cases_storage::storage_compliance_tests());
    tests.extend(test_cases_web_platform::web_platform_tests());
    tests.extend(test_cases_geometry::geometry_tests());
    tests.extend(test_cases_web_api::web_api_tests());
    tests.extend(test_cases_security::security_tests());
    tests.extend(test_cases_a11y_i18n::a11y_i18n_tests());
    tests
}
