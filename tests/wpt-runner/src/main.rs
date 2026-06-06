//! WPT 测试运行器 — 加载和执行 Web Platform Tests。
//!
//! 提供子命令：
//! - `run [filter]` — 执行测试用例并报告结果
//! - `list` — 列出所有可用的测试用例
//! - `summary` — 执行测试并仅输出汇总信息
//! - `reftest` — 运行 WPT reftest（渲染对比测试）

mod manifest;
mod reftest;
mod reftest_data;
mod report;
mod runner;

use runner::{TestContext, builtin_tests, filter_tests_by_category, filter_tests_by_pattern, run_all};

/// CLI 用法说明。
const USAGE: &str = "\
ZeroWeb WPT Runner v0.1

Usage:
  zero-wpt-runner <command> [options]

Commands:
  run [filter]      Run tests (optional category/pattern filter)
  list              List all available test cases
  summary           Run tests and print summary only
  reftest           Run WPT reftest suite (rendering comparison tests)

Options:
  --json            Output results in JSON format
  --tap             Output results in TAP format
  --junit <path>    Write JUnit XML report to file
  --manifest <path> Load external WPT MANIFEST.json
  --width <px>      Viewport width (default: 800)
  --height <px>     Viewport height (default: 600)
  --category <cat>  Reftest category filter (layout|text|all)
  --output <path>   Reftest report output path
";

/// 解析命令行参数中的选项。
struct CliOptions {
    /// 输出格式。
    format: OutputFormat,
    /// JUnit XML 输出路径。
    junit_path: Option<String>,
    /// 外部 manifest 路径。
    manifest_path: Option<String>,
    /// 视口宽度。
    viewport_width: f32,
    /// 视口高度。
    viewport_height: f32,
}

/// 输出格式。
enum OutputFormat {
    /// 可读文本。
    Text,
    /// JSON 格式。
    Json,
    /// TAP 格式。
    Tap,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];
    let mut options = CliOptions {
        format: OutputFormat::Text,
        junit_path: None,
        manifest_path: None,
        viewport_width: 800.0,
        viewport_height: 600.0,
    };

    // 解析选项参数
    let mut filter = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => options.format = OutputFormat::Json,
            "--tap" => options.format = OutputFormat::Tap,
            "--junit" => {
                i += 1;
                if i < args.len() {
                    options.junit_path = Some(args[i].clone());
                }
            }
            "--manifest" => {
                i += 1;
                if i < args.len() {
                    options.manifest_path = Some(args[i].clone());
                }
            }
            "--width" => {
                i += 1;
                if i < args.len() {
                    options.viewport_width = args[i].parse().unwrap_or(800.0);
                }
            }
            "--height" => {
                i += 1;
                if i < args.len() {
                    options.viewport_height = args[i].parse().unwrap_or(600.0);
                }
            }
            _ => {
                if filter.is_none() {
                    filter = Some(args[i].clone());
                }
            }
        }
        i += 1;
    }

    match command.as_str() {
        "run" => cmd_run(&options, filter.as_deref()),
        "list" => cmd_list(&options),
        "summary" => cmd_summary(&options, filter.as_deref()),
        "reftest" => cmd_reftest(&options, filter.as_deref()),
        "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {command}");
            print_usage();
            std::process::exit(1);
        }
    }
}

/// `run` 子命令 — 执行测试并报告详细结果。
fn cmd_run(options: &CliOptions, filter: Option<&str>) {
    let mut tests = builtin_tests();

    // 加载外部 manifest（如果指定）
    if let Some(path) = &options.manifest_path {
        match manifest::parse_manifest_file(std::path::Path::new(path)) {
            Ok(_entries) => eprintln!("Loaded external manifest: {path}"),
            Err(e) => eprintln!("Warning: Failed to load manifest: {e}"),
        }
    }

    // 应用过滤器
    if let Some(f) = filter {
        let by_cat = filter_tests_by_category(&tests, f);
        if !by_cat.is_empty() {
            tests = by_cat;
        } else {
            tests = filter_tests_by_pattern(&tests, f);
        }
    }

    let ctx = TestContext {
        viewport_width: options.viewport_width,
        viewport_height: options.viewport_height,
    };

    eprintln!("Running {} tests...", tests.len());
    let results = run_all(&tests, &ctx);
    let summary = report::TestSummary::from_results(&results);

    // 输出结果
    match &options.format {
        OutputFormat::Json => {
            let json = report::format_results_json(&results, &summary);
            println!("{json}");
            let cat_json = report::format_category_report_json(&results);
            eprintln!("{cat_json}");
        }
        OutputFormat::Tap => {
            let tap = report::format_tap(&results);
            println!("{tap}");
        }
        OutputFormat::Text => {
            let text = report::format_results_text(&results, &summary);
            println!("{text}");
        }
    }

    // JUnit XML 输出
    if let Some(path) = &options.junit_path {
        match std::fs::File::create(path) {
            Ok(mut file) => {
                if let Err(e) = report::write_junit_xml(&results, &mut file) {
                    eprintln!("Error writing JUnit XML: {e}");
                } else {
                    eprintln!("JUnit XML written to: {path}");
                }
            }
            Err(e) => eprintln!("Error creating JUnit file: {e}"),
        }
    }

    // 非零退出码表示有测试失败
    if summary.failed > 0 {
        std::process::exit(1);
    }
}

/// `list` 子命令 — 列出所有可用测试。
fn cmd_list(_options: &CliOptions) {
    let tests = builtin_tests();

    println!("Available WPT tests ({} total):\n", tests.len());

    let mut categories: Vec<&str> = tests.iter().map(|t| t.category.as_str()).collect();
    categories.sort();
    categories.dedup();

    for cat in categories {
        let cat_tests: Vec<_> = tests.iter().filter(|t| t.category == cat).collect();
        println!("[{cat}] ({} tests)", cat_tests.len());
        for t in cat_tests {
            println!("  {} — {}", t.id, t.description);
        }
        println!();
    }
}

/// `summary` 子命令 — 执行测试但只输出汇总。
fn cmd_summary(options: &CliOptions, filter: Option<&str>) {
    let mut tests = builtin_tests();

    if let Some(f) = filter {
        let by_cat = filter_tests_by_category(&tests, f);
        if !by_cat.is_empty() {
            tests = by_cat;
        } else {
            tests = filter_tests_by_pattern(&tests, f);
        }
    }

    let ctx = TestContext {
        viewport_width: options.viewport_width,
        viewport_height: options.viewport_height,
    };

    eprintln!("Running {} tests...", tests.len());
    let results = run_all(&tests, &ctx);
    let summary = report::TestSummary::from_results(&results);

    report::print_summary(&summary);

    // 输出按分类汇总
    let cat_report = report::format_category_report(&results);
    eprintln!("{cat_report}");

    if summary.failed > 0 {
        std::process::exit(1);
    }
}

/// `reftest` 子命令 — 运行 WPT reftest 套件。
///
/// 从内联 CSS 2.1 核心 reftest 数据加载测试对，用 CPU 软件渲染器
/// 渲染测试和参考 HTML，比较像素输出，生成通过率报告。
fn cmd_reftest(options: &CliOptions, filter: Option<&str>) {
    use reftest::{ReftestCategory, ReftestResult, run_reftest};

    let cases = reftest_data::css21_reftest_cases();
    let configs = reftest_data::css21_reftest_configs();

    // 过滤
    let filtered: Vec<(usize, &reftest::ReftestCase)> = cases
        .iter()
        .enumerate()
        .filter(|(i, case)| {
            if let Some(f) = filter {
                case.id.contains(f)
                    || matches!(f, "layout" if configs[*i].category == ReftestCategory::Layout)
                    || matches!(f, "text" if configs[*i].category == ReftestCategory::Text)
            } else {
                true
            }
        })
        .collect();

    eprintln!("Running {} reftest cases...", filtered.len());

    let mut results: Vec<ReftestResult> = Vec::with_capacity(filtered.len());
    let mut pass_count = 0usize;
    let mut fail_count = 0usize;
    let start = std::time::Instant::now();

    for (idx, case) in &filtered {
        let mut config = configs[*idx].clone();
        config.viewport_width = options.viewport_width as u32;
        config.viewport_height = options.viewport_height as u32;

        let result = run_reftest(case, &config);
        let passed = result.passed;
        let status_char = if passed { '✓' } else { '✗' };
        eprintln!("  {} {} ({:.2}%)", status_char, case.id, result.diff_ratio * 100.0);

        if passed {
            pass_count += 1;
        } else {
            fail_count += 1;
            eprintln!("    {}", result.message);
        }

        results.push(result);
    }

    let duration = start.elapsed();
    let total = pass_count + fail_count;
    let pass_rate = if total > 0 {
        pass_count as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    // 输出报告
    let report_text = format_reftest_report(&results, pass_count, fail_count, pass_rate, duration);

    match &options.format {
        OutputFormat::Json => {
            let json = format_reftest_report_json(&results, pass_count, fail_count, pass_rate, duration);
            println!("{json}");
        }
        _ => {
            println!("{report_text}");
        }
    }

    // 保存报告到 evidence 目录（如果指定了 --output）
    if let Some(path) = &options.junit_path {
        if let Err(e) = std::fs::write(path, &report_text) {
            eprintln!("Error writing report: {e}");
        } else {
            eprintln!("Report saved to: {path}");
        }
    }

    if fail_count > 0 {
        std::process::exit(1);
    }
}

/// 格式化 reftest 报告（文本格式）。
fn format_reftest_report(
    results: &[reftest::ReftestResult],
    pass_count: usize,
    fail_count: usize,
    pass_rate: f64,
    duration: std::time::Duration,
) -> String {
    let total = pass_count + fail_count;
    let mut report = String::new();

    report.push_str("═══════════════════════════════════════════════\n");
    report.push_str("  WPT Reftest Report\n");
    report.push_str("═══════════════════════════════════════════════\n\n");
    report.push_str(&format!("  Total:   {}\n", total));
    report.push_str(&format!("  Passed:  {}\n", pass_count));
    report.push_str(&format!("  Failed:  {}\n", fail_count));
    report.push_str(&format!("  Pass Rate: {:.1}%\n", pass_rate));
    report.push_str(&format!("  Duration:  {:.2}s\n\n", duration.as_secs_f64()));

    if fail_count > 0 {
        report.push_str("── Failures ──────────────────────────────────\n\n");
        for r in results {
            if !r.passed {
                report.push_str(&format!("  ✗ {}\n", r.id));
                report.push_str(&format!("    {}\n\n", r.message));
            }
        }
    }

    // 按分类汇总
    let mut layout_pass = 0usize;
    let mut layout_total = 0usize;
    let mut text_pass = 0usize;
    let mut text_total = 0usize;

    for r in results {
        let category = reftest::ReftestCategory::from_path(&r.id);
        match category {
            reftest::ReftestCategory::Layout => {
                layout_total += 1;
                if r.passed {
                    layout_pass += 1;
                }
            }
            reftest::ReftestCategory::Text => {
                text_total += 1;
                if r.passed {
                    text_pass += 1;
                }
            }
            reftest::ReftestCategory::Unknown => {
                layout_total += 1;
                if r.passed {
                    layout_pass += 1;
                }
            }
        }
    }

    report.push_str("── By Category ───────────────────────────────\n\n");
    if layout_total > 0 {
        report.push_str(&format!(
            "  Layout: {}/{} ({:.1}%)\n",
            layout_pass,
            layout_total,
            layout_pass as f64 / layout_total as f64 * 100.0
        ));
    }
    if text_total > 0 {
        report.push_str(&format!(
            "  Text:   {}/{} ({:.1}%)\n",
            text_pass,
            text_total,
            text_pass as f64 / text_total as f64 * 100.0
        ));
    }

    report
}

/// 格式化 reftest 报告（JSON 格式）。
fn format_reftest_report_json(
    results: &[reftest::ReftestResult],
    pass_count: usize,
    fail_count: usize,
    pass_rate: f64,
    duration: std::time::Duration,
) -> String {
    let total = pass_count + fail_count;

    let mut json = String::from("{\n");
    json.push_str(&format!("  \"total\": {total},\n"));
    json.push_str(&format!("  \"passed\": {pass_count},\n"));
    json.push_str(&format!("  \"failed\": {fail_count},\n"));
    json.push_str(&format!("  \"pass_rate\": {pass_rate:.1},\n"));
    json.push_str(&format!("  \"duration_ms\": {},\n", duration.as_millis()));
    json.push_str("  \"results\": [\n");

    for (i, r) in results.iter().enumerate() {
        json.push_str("    {\n");
        json.push_str(&format!("      \"id\": \"{}\",\n", report::escape_json_string(&r.id)));
        json.push_str(&format!("      \"passed\": {},\n", r.passed));
        json.push_str(&format!("      \"diff_ratio\": {:.6},\n", r.diff_ratio));
        json.push_str(&format!("      \"diff_pixels\": {},\n", r.diff_pixels));
        json.push_str(&format!("      \"total_pixels\": {},\n", r.total_pixels));
        json.push_str(&format!("      \"max_channel_diff\": {}", r.max_channel_diff));
        if r.message.is_empty() {
            json.push_str("\n    }");
        } else {
            json.push_str(&format!(
                ",\n      \"message\": \"{}\"\n    }}",
                report::escape_json_string(&r.message)
            ));
        }
        if i < results.len() - 1 {
            json.push(',');
        }
        json.push('\n');
    }

    json.push_str("  ]\n}");
    json
}

fn print_usage() {
    print!("{USAGE}");
}
