//! WPT 测试运行器 — 加载和执行 Web Platform Tests。
//!
//! 提供三个子命令：
//! - `run` — 执行测试用例并报告结果
//! - `list` — 列出所有可用的测试用例
//! - `summary` — 执行测试并仅输出汇总信息

mod manifest;
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

Options:
  --json            Output results in JSON format
  --tap             Output results in TAP format
  --junit <path>    Write JUnit XML report to file
  --manifest <path> Load external WPT MANIFEST.json
  --width <px>      Viewport width (default: 800)
  --height <px>     Viewport height (default: 600)
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

    if summary.failed > 0 {
        std::process::exit(1);
    }
}

fn print_usage() {
    print!("{USAGE}");
}
