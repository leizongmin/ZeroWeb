//! 测试报告 — 收集和格式化测试结果。
//!
//! 支持多种输出格式（文本、JSON）和结果汇总。

use std::io::Write;

/// 测试结果状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    /// 测试通过
    Pass,
    /// 测试失败（意外）
    Fail,
    /// 测试预期失败（已知问题，不阻断 CI）
    ExpectedFail,
    /// 测试意外通过（预期失败但实际通过）
    UnexpectedPass,
    /// 测试跳过
    Skip,
}

/// 单个测试的结果。
#[derive(Debug, Clone)]
pub struct TestResult {
    /// 测试标识符。
    pub id: String,
    /// 测试描述。
    pub description: String,
    /// 测试分类（如 html、css、layout）。
    pub category: String,
    /// 测试状态。
    pub status: TestStatus,
    /// 失败原因（通过时为空字符串）。
    pub message: String,
    /// 执行耗时（毫秒）。
    pub duration_ms: f64,
}

impl TestResult {
    /// 是否通过（含预期失败和跳过，不阻断 CI）。
    #[allow(dead_code)]
    pub fn passed(&self) -> bool {
        matches!(
            self.status,
            TestStatus::Pass | TestStatus::ExpectedFail | TestStatus::Skip
        )
    }

    /// 是否为意外失败（应阻断 CI）。
    pub fn is_unexpected_fail(&self) -> bool {
        self.status == TestStatus::Fail
    }

    /// 创建通过结果（向后兼容，无分类）。
    #[allow(dead_code)]
    pub fn pass(id: &str, description: &str, duration_ms: f64) -> Self {
        Self::pass_with_category(id, description, "", duration_ms)
    }

    /// 创建通过结果（含分类）。
    pub fn pass_with_category(id: &str, description: &str, category: &str, duration_ms: f64) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            status: TestStatus::Pass,
            message: String::new(),
            duration_ms,
        }
    }

    /// 创建失败结果（向后兼容，无分类）。
    #[allow(dead_code)]
    pub fn fail(id: &str, description: &str, message: &str, duration_ms: f64) -> Self {
        Self::fail_with_category(id, description, "", message, duration_ms)
    }

    /// 创建失败结果（含分类）。
    pub fn fail_with_category(id: &str, description: &str, category: &str, message: &str, duration_ms: f64) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            status: TestStatus::Fail,
            message: message.to_string(),
            duration_ms,
        }
    }

    /// 创建预期失败结果（向后兼容，无分类）。
    #[allow(dead_code)]
    pub fn expected_fail(id: &str, description: &str, message: &str, duration_ms: f64) -> Self {
        Self::expected_fail_with_category(id, description, "", message, duration_ms)
    }

    /// 创建预期失败结果（含分类）。
    pub fn expected_fail_with_category(
        id: &str,
        description: &str,
        category: &str,
        message: &str,
        duration_ms: f64,
    ) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            status: TestStatus::ExpectedFail,
            message: format!("[EXPECTED FAIL] {message}"),
            duration_ms,
        }
    }

    /// 创建意外通过结果（向后兼容，无分类）。
    #[allow(dead_code)]
    pub fn unexpected_pass(id: &str, description: &str, duration_ms: f64) -> Self {
        Self::unexpected_pass_with_category(id, description, "", duration_ms)
    }

    /// 创建意外通过结果（含分类）。
    pub fn unexpected_pass_with_category(id: &str, description: &str, category: &str, duration_ms: f64) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            status: TestStatus::UnexpectedPass,
            message: "[UNEXPECTED PASS] test expected to fail but passed".to_string(),
            duration_ms,
        }
    }

    /// 创建跳过结果。
    #[allow(dead_code)]
    pub fn skip(id: &str, description: &str) -> Self {
        Self::skip_with_category(id, description, "")
    }

    /// 创建跳过结果（含分类）。
    pub fn skip_with_category(id: &str, description: &str, category: &str) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            category: category.to_string(),
            status: TestStatus::Skip,
            message: "[SKIPPED]".to_string(),
            duration_ms: 0.0,
        }
    }
}

/// 测试结果汇总。
#[derive(Debug, Clone)]
pub struct TestSummary {
    /// 总测试数。
    pub total: usize,
    /// 通过数。
    pub passed: usize,
    /// 意外失败数。
    pub failed: usize,
    /// 预期失败数。
    pub expected_failures: usize,
    /// 跳过数。
    pub skipped: usize,
    /// 意外通过数。
    pub unexpected_passes: usize,
    /// 总耗时（毫秒）。
    pub total_duration_ms: f64,
    /// 意外失败测试列表。
    pub failures: Vec<TestResult>,
}

impl TestSummary {
    /// 从结果列表生成汇总。
    pub fn from_results(results: &[TestResult]) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.status == TestStatus::Pass).count();
        let failed = results.iter().filter(|r| r.status == TestStatus::Fail).count();
        let expected_failures = results.iter().filter(|r| r.status == TestStatus::ExpectedFail).count();
        let skipped = results.iter().filter(|r| r.status == TestStatus::Skip).count();
        let unexpected_passes = results
            .iter()
            .filter(|r| r.status == TestStatus::UnexpectedPass)
            .count();
        let total_duration_ms = results.iter().map(|r| r.duration_ms).sum();
        let failures: Vec<TestResult> = results.iter().filter(|r| r.is_unexpected_fail()).cloned().collect();

        Self {
            total,
            passed,
            failed,
            expected_failures,
            skipped,
            unexpected_passes,
            total_duration_ms,
            failures,
        }
    }

    /// 通过率（0.0 ~ 1.0）— 仅统计已执行的非跳过测试。
    pub fn pass_rate(&self) -> f64 {
        let executed = self.total - self.skipped;
        if executed == 0 {
            0.0
        } else {
            self.passed as f64 / executed as f64
        }
    }
}

/// 按分类汇总的测试结果。
#[derive(Debug, Clone)]
pub struct CategorySummary {
    /// 分类名称。
    pub category: String,
    /// 总测试数。
    pub total: usize,
    /// 通过数。
    pub passed: usize,
    /// 意外失败数。
    pub failed: usize,
    /// 预期失败数。
    pub expected_failures: usize,
    /// 跳过数。
    pub skipped: usize,
    /// 意外通过数。
    pub unexpected_passes: usize,
    /// 总耗时（毫秒）。
    pub total_duration_ms: f64,
}

impl CategorySummary {
    /// 从指定分类的测试结果生成汇总。
    pub fn from_results(category: &str, results: &[TestResult]) -> Self {
        let cat_results: Vec<&TestResult> = results.iter().filter(|r| r.category == category).collect();
        let total = cat_results.len();
        let passed = cat_results.iter().filter(|r| r.status == TestStatus::Pass).count();
        let failed = cat_results.iter().filter(|r| r.status == TestStatus::Fail).count();
        let expected_failures = cat_results
            .iter()
            .filter(|r| r.status == TestStatus::ExpectedFail)
            .count();
        let skipped = cat_results.iter().filter(|r| r.status == TestStatus::Skip).count();
        let unexpected_passes = cat_results
            .iter()
            .filter(|r| r.status == TestStatus::UnexpectedPass)
            .count();
        let total_duration_ms = cat_results.iter().map(|r| r.duration_ms).sum();

        Self {
            category: category.to_string(),
            total,
            passed,
            failed,
            expected_failures,
            skipped,
            unexpected_passes,
            total_duration_ms,
        }
    }

    /// 通过率（0.0 ~ 1.0）— 仅统计已执行的非跳过测试。
    pub fn pass_rate(&self) -> f64 {
        let executed = self.total - self.skipped;
        if executed == 0 {
            0.0
        } else {
            self.passed as f64 / executed as f64
        }
    }
}

/// 从测试结果中提取所有唯一分类。
pub fn extract_categories(results: &[TestResult]) -> Vec<String> {
    let mut categories: Vec<String> = results
        .iter()
        .map(|r| r.category.clone())
        .filter(|c| !c.is_empty())
        .collect();
    categories.sort();
    categories.dedup();
    categories
}

/// 生成按分类汇总的报告文本。
pub fn format_category_report(results: &[TestResult]) -> String {
    let categories = extract_categories(results);
    if categories.is_empty() {
        return "No categories found.\n".to_string();
    }

    let mut out = String::new();
    out.push_str("\n── Per-Category Pass Rate ──────────────────────────────────────\n");
    out.push_str(&format!(
        "{:<20} {:>6} {:>6} {:>6} {:>6} {:>8}  {}\n",
        "Category", "Total", "Pass", "Fail", "XFail", "Skip", "Rate"
    ));
    out.push_str(&"-".repeat(78));
    out.push('\n');

    let mut cat_summaries: Vec<CategorySummary> = Vec::new();
    for cat in &categories {
        cat_summaries.push(CategorySummary::from_results(cat, results));
    }

    // 按通过率排序（最低在前，便于发现薄弱点）
    cat_summaries.sort_by(|a, b| {
        a.pass_rate()
            .partial_cmp(&b.pass_rate())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for cs in &cat_summaries {
        let rate_pct = format!("{:.1}%", cs.pass_rate() * 100.0);
        let fail_indicator = if cs.failed > 0 { "⚠" } else { "✓" };
        out.push_str(&format!(
            "{:<20} {:>6} {:>6} {:>6} {:>6} {:>8}  {} {}\n",
            cs.category, cs.total, cs.passed, cs.failed, cs.expected_failures, cs.skipped, rate_pct, fail_indicator
        ));
    }

    out.push_str(&"-".repeat(78));
    out.push('\n');
    out.push_str(&format!("Total categories: {}\n", categories.len()));

    out
}

/// 生成按分类汇总的 JSON 报告。
pub fn format_category_report_json(results: &[TestResult]) -> String {
    let categories = extract_categories(results);
    let mut json = String::from("{\"categories\":[");
    for (i, cat) in categories.iter().enumerate() {
        if i > 0 {
            json.push(',');
        }
        let cs = CategorySummary::from_results(cat, results);
        json.push_str(&format!(
            "{{\"category\":\"{}\",\"total\":{},\"passed\":{},\"failed\":{},\"expected_failures\":{},\"skipped\":{},\"unexpected_passes\":{},\"pass_rate\":{:.4},\"duration_ms\":{:.2}}}",
            cat, cs.total, cs.passed, cs.failed, cs.expected_failures, cs.skipped, cs.unexpected_passes, cs.pass_rate(), cs.total_duration_ms
        ));
    }
    json.push_str("]}");
    json
}

/// 将测试结果格式化为可读文本。
pub fn format_results_text(results: &[TestResult], summary: &TestSummary) -> String {
    let mut out = String::new();

    out.push_str(&format!(
        "\nWPT Test Results: {}/{} passed ({:.1}%)\n",
        summary.passed,
        summary.total,
        summary.pass_rate() * 100.0
    ));
    if summary.expected_failures > 0 {
        out.push_str(&format!(
            "Expected failures: {} | Skipped: {} | Unexpected passes: {}\n",
            summary.expected_failures, summary.skipped, summary.unexpected_passes
        ));
    }
    out.push_str(&format!("Duration: {:.2}ms total\n\n", summary.total_duration_ms));

    for result in results {
        let status = match result.status {
            TestStatus::Pass => "PASS",
            TestStatus::Fail => "FAIL",
            TestStatus::ExpectedFail => "EXPECTED_FAIL",
            TestStatus::UnexpectedPass => "UNEXPECTED_PASS",
            TestStatus::Skip => "SKIP",
        };
        out.push_str(&format!(
            "  [{status}] {} — {} ({:.2}ms)\n",
            result.id, result.description, result.duration_ms
        ));
        if !result.message.is_empty() {
            out.push_str(&format!("         {}\n", result.message));
        }
    }

    if !summary.failures.is_empty() {
        out.push_str("\nUnexpected failures:\n");
        for f in &summary.failures {
            out.push_str(&format!("  - {} : {}\n", f.id, f.message));
        }
    }

    out
}

/// 将测试结果序列化为 JSON。
pub fn format_results_json(results: &[TestResult], summary: &TestSummary) -> String {
    let mut json_results = String::new();
    json_results.push_str("{\"results\":[");

    for (i, result) in results.iter().enumerate() {
        if i > 0 {
            json_results.push(',');
        }
        let status = match result.status {
            TestStatus::Pass => "pass",
            TestStatus::Fail => "fail",
            TestStatus::ExpectedFail => "expected_fail",
            TestStatus::UnexpectedPass => "unexpected_pass",
            TestStatus::Skip => "skip",
        };
        let escaped_message = escape_json_string(&result.message);
        let escaped_desc = escape_json_string(&result.description);
        json_results.push_str(&format!(
            "{{\"id\":\"{}\",\"description\":\"{}\",\"status\":\"{}\",\"duration_ms\":{:.2},\"message\":\"{}\"}}",
            result.id, escaped_desc, status, result.duration_ms, escaped_message
        ));
    }

    json_results.push_str(&format!(
        "],\"summary\":{{\"total\":{},\"passed\":{},\"failed\":{},\"expected_failures\":{},\"skipped\":{},\"unexpected_passes\":{},\"duration_ms\":{:.2}}}}}",
        summary.total, summary.passed, summary.failed, summary.expected_failures,
        summary.skipped, summary.unexpected_passes, summary.total_duration_ms
    ));

    json_results
}

/// 将汇总写入标准输出（带颜色）。
pub fn print_summary(summary: &TestSummary) {
    let total_str = format!(
        "Total: {} | Passed: {} | Failed: {} | Expected failures: {} | Skipped: {} | Rate: {:.1}%",
        summary.total,
        summary.passed,
        summary.failed,
        summary.expected_failures,
        summary.skipped,
        summary.pass_rate() * 100.0
    );
    println!("{total_str}");
    println!("Duration: {:.2}ms", summary.total_duration_ms);
}

/// 将结果写入 TAP（Test Anything Protocol）格式。
pub fn format_tap(results: &[TestResult]) -> String {
    let mut out = String::new();
    out.push_str(&format!("1..{}\n", results.len()));

    for (i, result) in results.iter().enumerate() {
        let n = i + 1;
        match result.status {
            TestStatus::Pass => out.push_str(&format!("ok {n} - {}\n", result.id)),
            TestStatus::Fail => out.push_str(&format!("not ok {n} - {} # {}\n", result.id, result.message)),
            TestStatus::ExpectedFail => out.push_str(&format!(
                "ok {n} - {} # TODO expected fail: {}\n",
                result.id, result.message
            )),
            TestStatus::UnexpectedPass => out.push_str(&format!("ok {n} - {} # UNEXPECTED PASS\n", result.id)),
            TestStatus::Skip => out.push_str(&format!("ok {n} - {} # SKIP\n", result.id)),
        }
    }

    out
}

/// 将 JUnit XML 格式的结果写入指定 writer。
pub fn write_junit_xml<W: Write>(results: &[TestResult], writer: &mut W) -> std::io::Result<()> {
    let summary = TestSummary::from_results(results);
    writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        writer,
        r#"<testsuite name="wpt-runner" tests="{}" failures="{}" skipped="{}" time="{:.3}">"#,
        summary.total,
        summary.failed,
        summary.skipped,
        summary.total_duration_ms / 1000.0
    )?;

    for result in results {
        let escaped_id = escape_xml_string(&result.id);
        let escaped_desc = escape_xml_string(&result.description);
        writeln!(
            writer,
            r#"  <testcase classname="wpt" name="{}" time="{:.3}">"#,
            escaped_id,
            result.duration_ms / 1000.0
        )?;
        match result.status {
            TestStatus::Pass | TestStatus::UnexpectedPass => {}
            TestStatus::Fail => {
                let escaped_msg = escape_xml_string(&result.message);
                writeln!(
                    writer,
                    r#"    <failure message="{}">{}</failure>"#,
                    escaped_desc, escaped_msg
                )?;
            }
            TestStatus::ExpectedFail => {
                writeln!(writer, r#"    <skipped message="expected failure"/>"#)?;
            }
            TestStatus::Skip => {
                writeln!(writer, r#"    <skipped/>"#)?;
            }
        }
        writeln!(writer, "  </testcase>")?;
    }

    writeln!(writer, "</testsuite>")?;
    Ok(())
}

// ── 辅助函数 ─────────────────────────────────────────────────────

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

fn escape_xml_string(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_result(id: &str, status: TestStatus, message: &str) -> TestResult {
        TestResult {
            id: id.to_string(),
            description: format!("Test {id}"),
            category: "test".to_string(),
            status,
            message: message.to_string(),
            duration_ms: 1.0,
        }
    }

    #[test]
    fn test_result_pass() {
        let r = TestResult::pass("t1", "desc", 5.0);
        assert!(r.passed());
        assert_eq!(r.status, TestStatus::Pass);
        assert_eq!(r.id, "t1");
        assert!(r.message.is_empty());
    }

    #[test]
    fn test_result_fail() {
        let r = TestResult::fail("t2", "desc", "broken", 3.0);
        assert!(!r.passed());
        assert!(r.is_unexpected_fail());
        assert_eq!(r.message, "broken");
    }

    #[test]
    fn test_result_expected_fail() {
        let r = TestResult::expected_fail("t3", "desc", "known issue", 2.0);
        assert!(r.passed());
        assert!(!r.is_unexpected_fail());
        assert_eq!(r.status, TestStatus::ExpectedFail);
    }

    #[test]
    fn test_result_skip() {
        let r = TestResult::skip("t4", "desc");
        assert!(r.passed());
        assert_eq!(r.status, TestStatus::Skip);
        assert_eq!(r.duration_ms, 0.0);
    }

    #[test]
    fn test_result_unexpected_pass() {
        let r = TestResult::unexpected_pass("t5", "desc", 1.0);
        assert_eq!(r.status, TestStatus::UnexpectedPass);
    }

    #[test]
    fn test_summary_from_results() {
        let results = vec![
            make_result("a", TestStatus::Pass, ""),
            make_result("b", TestStatus::Fail, "fail reason"),
            make_result("c", TestStatus::Pass, ""),
            make_result("d", TestStatus::ExpectedFail, "known"),
            make_result("e", TestStatus::Skip, ""),
        ];
        let summary = TestSummary::from_results(&results);
        assert_eq!(summary.total, 5);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.expected_failures, 1);
        assert_eq!(summary.skipped, 1);
        assert_eq!(summary.failures.len(), 1);
        assert_eq!(summary.failures[0].id, "b");
    }

    #[test]
    fn test_pass_rate() {
        let results = vec![
            make_result("a", TestStatus::Pass, ""),
            make_result("b", TestStatus::Pass, ""),
        ];
        let summary = TestSummary::from_results(&results);
        assert!((summary.pass_rate() - 1.0).abs() < f64::EPSILON);

        let empty_summary = TestSummary::from_results(&[]);
        assert!((empty_summary.pass_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pass_rate_excludes_skipped() {
        let results = vec![
            make_result("a", TestStatus::Pass, ""),
            make_result("b", TestStatus::Skip, ""),
        ];
        let summary = TestSummary::from_results(&results);
        // 1 passed, 1 skipped, 2 total → pass_rate = 1/(2-1) = 1.0
        assert!((summary.pass_rate() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_format_results_text() {
        let results = vec![
            make_result("pass-test", TestStatus::Pass, ""),
            make_result("fail-test", TestStatus::Fail, "something broke"),
        ];
        let summary = TestSummary::from_results(&results);
        let text = format_results_text(&results, &summary);
        assert!(text.contains("PASS"));
        assert!(text.contains("FAIL"));
        assert!(text.contains("pass-test"));
        assert!(text.contains("something broke"));
    }

    #[test]
    fn test_format_results_json() {
        let results = vec![
            make_result("t1", TestStatus::Pass, ""),
            make_result("t2", TestStatus::Fail, "error msg"),
        ];
        let summary = TestSummary::from_results(&results);
        let json = format_results_json(&results, &summary);
        assert!(json.contains("\"status\":\"pass\""));
        assert!(json.contains("\"status\":\"fail\""));
        assert!(json.contains("\"total\":2"));
        // JSON should be parseable
        let _: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    }

    #[test]
    fn test_format_tap() {
        let results = vec![
            make_result("ok-test", TestStatus::Pass, ""),
            make_result("nok-test", TestStatus::Fail, "bad"),
        ];
        let tap = format_tap(&results);
        assert!(tap.contains("1..2"));
        assert!(tap.contains("ok 1"));
        assert!(tap.contains("not ok 2"));
    }

    #[test]
    fn test_format_tap_skip() {
        let results = vec![make_result("s", TestStatus::Skip, "")];
        let tap = format_tap(&results);
        assert!(tap.contains("SKIP"));
    }

    #[test]
    fn test_write_junit_xml() {
        let results = vec![
            make_result("x", TestStatus::Pass, ""),
            make_result("y", TestStatus::Fail, "err"),
        ];
        let mut buf = Vec::new();
        write_junit_xml(&results, &mut buf).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert!(xml.contains("<?xml"));
        assert!(xml.contains("<testsuite"));
        assert!(xml.contains("<failure"));
    }

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("a\"b"), "a\\\"b");
        assert_eq!(escape_json_string("a\nb"), "a\\nb");
    }

    #[test]
    fn test_escape_xml_string() {
        assert_eq!(escape_xml_string("a<b"), "a&lt;b");
        assert_eq!(escape_xml_string("a&b"), "a&amp;b");
    }

    #[test]
    fn test_summary_duration() {
        let results = vec![TestResult::pass("a", "", 10.0), TestResult::pass("b", "", 20.0)];
        let summary = TestSummary::from_results(&results);
        assert!((summary.total_duration_ms - 30.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_print_summary_no_panic() {
        let results = vec![make_result("a", TestStatus::Pass, "")];
        let summary = TestSummary::from_results(&results);
        // Just ensure it doesn't panic
        print_summary(&summary);
    }

    // ── 分类报告测试 ─────────────────────────────────────

    fn make_cat_result(id: &str, category: &str, status: TestStatus, message: &str) -> TestResult {
        TestResult {
            id: id.to_string(),
            description: format!("Test {id}"),
            category: category.to_string(),
            status,
            message: message.to_string(),
            duration_ms: 1.0,
        }
    }

    #[test]
    fn test_extract_categories() {
        let results = vec![
            make_cat_result("a", "css", TestStatus::Pass, ""),
            make_cat_result("b", "html", TestStatus::Pass, ""),
            make_cat_result("c", "css", TestStatus::Fail, "err"),
        ];
        let cats = extract_categories(&results);
        assert_eq!(cats, vec!["css", "html"]);
    }

    #[test]
    fn test_extract_categories_empty() {
        let results: Vec<TestResult> = vec![];
        let cats = extract_categories(&results);
        assert!(cats.is_empty());
    }

    #[test]
    fn test_category_summary_from_results() {
        let results = vec![
            make_cat_result("a", "css", TestStatus::Pass, ""),
            make_cat_result("b", "css", TestStatus::Pass, ""),
            make_cat_result("c", "css", TestStatus::Fail, "err"),
            make_cat_result("d", "html", TestStatus::Pass, ""),
        ];
        let cs = CategorySummary::from_results("css", &results);
        assert_eq!(cs.total, 3);
        assert_eq!(cs.passed, 2);
        assert_eq!(cs.failed, 1);
        assert!((cs.pass_rate() - (2.0 / 3.0)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_category_summary_no_results() {
        let results: Vec<TestResult> = vec![];
        let cs = CategorySummary::from_results("css", &results);
        assert_eq!(cs.total, 0);
        assert!((cs.pass_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_format_category_report() {
        let results = vec![
            make_cat_result("a", "css", TestStatus::Pass, ""),
            make_cat_result("b", "css", TestStatus::Fail, "err"),
            make_cat_result("c", "html", TestStatus::Pass, ""),
        ];
        let report = format_category_report(&results);
        assert!(report.contains("css"));
        assert!(report.contains("html"));
        assert!(report.contains("Per-Category"));
    }

    #[test]
    fn test_format_category_report_json() {
        let results = vec![
            make_cat_result("a", "css", TestStatus::Pass, ""),
            make_cat_result("b", "html", TestStatus::Fail, "err"),
        ];
        let json = format_category_report_json(&results);
        assert!(json.contains("\"categories\""));
        assert!(json.contains("\"category\":\"css\""));
        let _: serde_json::Value = serde_json::from_str(&json).expect("Should be valid JSON");
    }

    #[test]
    fn test_pass_with_category() {
        let r = TestResult::pass_with_category("t1", "desc", "css", 5.0);
        assert_eq!(r.category, "css");
        assert!(r.passed());
    }

    #[test]
    fn test_fail_with_category() {
        let r = TestResult::fail_with_category("t2", "desc", "html", "broken", 3.0);
        assert_eq!(r.category, "html");
        assert!(r.is_unexpected_fail());
    }

    #[test]
    fn test_expected_fail_with_category() {
        let r = TestResult::expected_fail_with_category("t3", "desc", "layout", "known", 2.0);
        assert_eq!(r.category, "layout");
        assert!(r.passed());
    }

    #[test]
    fn test_skip_with_category() {
        let r = TestResult::skip_with_category("t4", "desc", "canvas");
        assert_eq!(r.category, "canvas");
        assert!(r.passed());
    }

    #[test]
    fn test_unexpected_pass_with_category() {
        let r = TestResult::unexpected_pass_with_category("t5", "desc", "security", 1.0);
        assert_eq!(r.category, "security");
        assert_eq!(r.status, TestStatus::UnexpectedPass);
    }
}
