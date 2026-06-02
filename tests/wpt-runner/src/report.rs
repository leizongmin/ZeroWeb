//! 测试报告 — 收集和格式化测试结果。
//!
//! 支持多种输出格式（文本、JSON）和结果汇总。

use std::io::Write;

/// 单个测试的结果。
#[derive(Debug, Clone)]
pub struct TestResult {
    /// 测试标识符。
    pub id: String,
    /// 测试描述。
    pub description: String,
    /// 是否通过。
    pub passed: bool,
    /// 失败原因（通过时为空字符串）。
    pub message: String,
    /// 执行耗时（毫秒）。
    pub duration_ms: f64,
}

impl TestResult {
    /// 创建通过结果。
    pub fn pass(id: &str, description: &str, duration_ms: f64) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            passed: true,
            message: String::new(),
            duration_ms,
        }
    }

    /// 创建失败结果。
    pub fn fail(id: &str, description: &str, message: &str, duration_ms: f64) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            passed: false,
            message: message.to_string(),
            duration_ms,
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
    /// 失败数。
    pub failed: usize,
    /// 总耗时（毫秒）。
    pub total_duration_ms: f64,
    /// 失败测试列表。
    pub failures: Vec<TestResult>,
}

impl TestSummary {
    /// 从结果列表生成汇总。
    pub fn from_results(results: &[TestResult]) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration_ms = results.iter().map(|r| r.duration_ms).sum();
        let failures: Vec<TestResult> = results.iter().filter(|r| !r.passed).cloned().collect();

        Self {
            total,
            passed,
            failed,
            total_duration_ms,
            failures,
        }
    }

    /// 通过率（0.0 ~ 1.0）。
    pub fn pass_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.passed as f64 / self.total as f64
        }
    }
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
    out.push_str(&format!("Duration: {:.2}ms total\n\n", summary.total_duration_ms));

    for result in results {
        let status = if result.passed { "PASS" } else { "FAIL" };
        out.push_str(&format!(
            "  [{status}] {} — {} ({:.2}ms)\n",
            result.id, result.description, result.duration_ms
        ));
        if !result.passed && !result.message.is_empty() {
            out.push_str(&format!("         {}\n", result.message));
        }
    }

    if !summary.failures.is_empty() {
        out.push_str("\nFailed tests:\n");
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
        let status = if result.passed { "pass" } else { "fail" };
        let escaped_message = escape_json_string(&result.message);
        let escaped_desc = escape_json_string(&result.description);
        json_results.push_str(&format!(
            "{{\"id\":\"{}\",\"description\":\"{}\",\"status\":\"{}\",\"duration_ms\":{:.2},\"message\":\"{}\"}}",
            result.id, escaped_desc, status, result.duration_ms, escaped_message
        ));
    }

    json_results.push_str(&format!(
        "],\"summary\":{{\"total\":{},\"passed\":{},\"failed\":{},\"duration_ms\":{:.2}}}}}",
        summary.total, summary.passed, summary.failed, summary.total_duration_ms
    ));

    json_results
}

/// 将汇总写入标准输出（带颜色）。
pub fn print_summary(summary: &TestSummary) {
    let total_str = format!(
        "Total: {} | Passed: {} | Failed: {} | Rate: {:.1}%",
        summary.total,
        summary.passed,
        summary.failed,
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
        if result.passed {
            out.push_str(&format!("ok {n} - {}\n", result.id));
        } else {
            out.push_str(&format!("not ok {n} - {} # {}\n", result.id, result.message));
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
        r#"<testsuite name="wpt-runner" tests="{}" failures="{}" time="{:.3}">"#,
        summary.total,
        summary.failed,
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
        if !result.passed {
            let escaped_msg = escape_xml_string(&result.message);
            writeln!(
                writer,
                r#"    <failure message="{}">{}</failure>"#,
                escaped_desc, escaped_msg
            )?;
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

    fn make_result(id: &str, passed: bool, message: &str) -> TestResult {
        TestResult {
            id: id.to_string(),
            description: format!("Test {id}"),
            passed,
            message: message.to_string(),
            duration_ms: 1.0,
        }
    }

    #[test]
    fn test_result_pass() {
        let r = TestResult::pass("t1", "desc", 5.0);
        assert!(r.passed);
        assert_eq!(r.id, "t1");
        assert!(r.message.is_empty());
    }

    #[test]
    fn test_result_fail() {
        let r = TestResult::fail("t2", "desc", "broken", 3.0);
        assert!(!r.passed);
        assert_eq!(r.message, "broken");
    }

    #[test]
    fn test_summary_from_results() {
        let results = vec![
            make_result("a", true, ""),
            make_result("b", false, "fail reason"),
            make_result("c", true, ""),
        ];
        let summary = TestSummary::from_results(&results);
        assert_eq!(summary.total, 3);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.failures.len(), 1);
        assert_eq!(summary.failures[0].id, "b");
    }

    #[test]
    fn test_pass_rate() {
        let results = vec![make_result("a", true, ""), make_result("b", true, "")];
        let summary = TestSummary::from_results(&results);
        assert!((summary.pass_rate() - 1.0).abs() < f64::EPSILON);

        let empty_summary = TestSummary::from_results(&[]);
        assert!((empty_summary.pass_rate() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_format_results_text() {
        let results = vec![
            make_result("pass-test", true, ""),
            make_result("fail-test", false, "something broke"),
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
        let results = vec![make_result("t1", true, ""), make_result("t2", false, "error msg")];
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
        let results = vec![make_result("ok-test", true, ""), make_result("nok-test", false, "bad")];
        let tap = format_tap(&results);
        assert!(tap.contains("1..2"));
        assert!(tap.contains("ok 1"));
        assert!(tap.contains("not ok 2"));
    }

    #[test]
    fn test_write_junit_xml() {
        let results = vec![make_result("x", true, ""), make_result("y", false, "err")];
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
        let results = vec![make_result("a", true, "")];
        let summary = TestSummary::from_results(&results);
        // Just ensure it doesn't panic
        print_summary(&summary);
    }
}
