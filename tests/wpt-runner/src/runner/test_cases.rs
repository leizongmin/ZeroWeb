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

mod test_cases_core;
mod test_cases_css_compliance;
mod test_cases_css_extended;
mod test_cases_dom_api;
mod test_cases_html_layout;
mod test_cases_standard;
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
    tests
}
