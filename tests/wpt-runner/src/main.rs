//! WPT 测试运行器 — 加载和执行 Web Platform Tests。
//!
//! 提供子命令：
//! - `run [filter]` — 执行测试用例并报告结果
//! - `list` — 列出所有可用的测试用例
//! - `summary` — 执行测试并仅输出汇总信息
//! - `reftest` — 运行 WPT reftest（渲染对比测试）
//! - `reftest-upstream` — 运行上游 WPT reftest（wpt-data/）
//! - `product-smoke <html>` — 渲染产品静态 fixture 到 CPU PNG，可与 chromium Oracle PNG 像素对比（DC-13）

mod manifest;
mod product_smoke;
mod reftest;
mod reftest_data;
mod report;
mod runner;
mod runner_text_metrics;
mod testharness;
mod wpt_file_loader;

use rayon::prelude::*;
use runner::{TestContext, builtin_tests, filter_tests_by_category, filter_tests_by_pattern};

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
  reftest-upstream  Run upstream WPT reftest files from wpt-data/
  testharness-html  Run selected media/forms/focus/input-event testharness cases
  layout-dump [filter]  B1: dump layout tree for upstream test pages (golden compare,
                       see scripts/run-layout-golden.sh)
  reftest-oracle [filter]  DC-14: render upstream test pages vs chromium oracle-shots (true pass-rate)
  struct-sweep [filter]   DC-13: sibling-overlap struct-check sweep over upstream test pages
  product-smoke <html>  Render a product static fixture to CPU PNG (DC-13)
                       (--base-dir, --oracle <png>, --out <png>, --max-diff <pct>,
                        --channel-diff <0..255>, --geometry-oracle <json>,
                        --max-geometry-diff <px>, --region <id>:<max-pct>)
  perf                  Page-level perf benchmark (perf-gate page scenarios)
                       (--scenario <id>:<path> [repeatable], --base-dir,
                        --width <px>, --height <px>, --iterations <n>)

Options:
  --json            Output results in JSON format
  --tap             Output results in TAP format
  --junit <path>    Write JUnit XML report to file
  --manifest <path> Load external WPT MANIFEST.json
  --wpt-data <dir>  Path to wpt-data directory (for reftest-upstream)
  --width <px>      Viewport width (default: 800)
  --height <px>     Viewport height (default: 600)
  --jobs <n>        Number of parallel test jobs (default: min(CPU-1, 8))
  --gpu             Route reftest through the GPU path (NOTE: currently a CPU-fallback stub —
                    does NOT use GpuRenderer, gives no speedup; kept as a hook for future work)
  --media <type>    Rendering media type: print|screen (default: screen; applies @media print/screen cascade)
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
    /// 使用 GPU 渲染模式。
    use_gpu: bool,
    /// 并行执行的测试 worker 数。
    jobs: Option<usize>,
    /// 上游 WPT 数据目录。
    wpt_data: Option<String>,
    /// 渲染媒体类型（DC-12 @media print/screen；R1991）。默认 `Screen`。
    media_type: zero_css_parser::media_query::MediaType,
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

    if std::env::var("ZW_SHAPED_ADVANCE_TRACE").as_deref() == Ok("1") {
        tracing_subscriber::fmt().with_target(false).without_time().init();
    }

    // R1765：注册 fontdue 真实 advance 测量回调（镜像 browser app.rs:204）。
    // 此前 runner 未注册 → paint 回退 estimate_char_width（0.55×fs）→ reftest/product-smoke
    // 测量用 estimate paint（'m'=0.584×fs）vs chromium 0.797×fs，font-wall 部分是测量 artifact。
    // 注册后经 with_measure_ctx（reftest.rs render 包裹）注入 fontdue measure_advance。
    zero_engine::set_char_measure_fn(runner_text_metrics::measure_char);
    zero_engine::set_text_shape_fn(runner_text_metrics::shape_text);

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    let command = &args[1];

    // product-smoke / perf 有独立参数（--scenario/--base-dir/--oracle/--out），提前分支避免污染通用选项解析。
    if command == "product-smoke" {
        cmd_product_smoke(&args[2..]);
        return;
    }
    if command == "perf" {
        cmd_perf(&args[2..]);
        return;
    }

    let mut options = CliOptions {
        format: OutputFormat::Text,
        junit_path: None,
        manifest_path: None,
        viewport_width: 800.0,
        viewport_height: 600.0,
        use_gpu: false,
        jobs: None,
        wpt_data: None,
        media_type: zero_css_parser::media_query::MediaType::Screen,
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
            "--gpu" => options.use_gpu = true,
            "--jobs" => {
                i += 1;
                if i < args.len() {
                    options.jobs = args[i].parse::<usize>().ok().filter(|jobs| *jobs > 0);
                }
            }
            "--wpt-data" => {
                i += 1;
                if i < args.len() {
                    options.wpt_data = Some(args[i].clone());
                }
            }
            // R1991：渲染媒体类型（DC-12 @media print/screen 级联过滤）。
            // `--media print` 量 @media print 真实 WPT yield；默认 screen = 零变更。
            "--media" => {
                i += 1;
                if i < args.len() {
                    match args[i].to_ascii_lowercase().as_str() {
                        "print" => options.media_type = zero_css_parser::media_query::MediaType::Print,
                        "screen" => options.media_type = zero_css_parser::media_query::MediaType::Screen,
                        other => eprintln!("Unknown --media value '{other}' (expected print|screen), ignoring"),
                    }
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
        "reftest-upstream" => cmd_reftest_upstream(&options, filter.as_deref()),
        "testharness-html" => cmd_testharness_html(&options, filter.as_deref()),
        "layout-dump" => cmd_layout_dump(&options, filter.as_deref()),
        "reftest-oracle" => cmd_reftest_oracle(&options, filter.as_deref()),
        "struct-sweep" => cmd_struct_sweep(&options, filter.as_deref()),
        "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {command}");
            print_usage();
            std::process::exit(1);
        }
    }
}

fn cmd_testharness_html(options: &CliOptions, filter: Option<&str>) {
    let wpt_root = options
        .wpt_data
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("tests/wpt-runner/wpt-data"));
    let cases = testharness::run_html_interaction_cases(&wpt_root, filter);
    let failed = cases.iter().any(|(_, results)| {
        results
            .iter()
            .any(|result| result.status != testharness::HarnessStatus::Pass)
    });

    match options.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&cases).unwrap_or_else(|_| "[]".into())
            );
        }
        OutputFormat::Text | OutputFormat::Tap => {
            for (case, results) in &cases {
                for result in results {
                    println!("{:?} {case} :: {}", result.status, result.name);
                    if let Some(message) = &result.message {
                        println!("  {message}");
                    }
                }
            }
        }
    }
    if failed || cases.is_empty() {
        std::process::exit(1);
    }
}

/// `product-smoke` 子命令 — 渲染产品静态 fixture 到 CPU PNG（DC-13）。
///
/// 用途：把 `apps/browser/assets/` 下的产品 fixture（welcome/morning-work/wintertc 等）
/// 经 ZeroWeb CPU 软件渲染（800×600，base_dir 加载外链 CSS/图片）输出为 PNG，
/// 并可选与 chromium Oracle PNG 做像素对比，量化产品可见渲染差距。
///
/// 用法：
///   zero-wpt-runner product-smoke <html-path> [--base-dir <dir>] [--oracle <png>] [--out <png>] [--width N] [--height N]
fn cmd_product_smoke(args: &[String]) {
    let mut html_path: Option<String> = None;
    let mut base_dir: Option<String> = None;
    let mut oracle: Option<String> = None;
    let mut out: Option<String> = None;
    let mut width: u32 = 800;
    let mut height: u32 = 600;
    // DC-13 回归门禁阈值（%）：diff 超过则非零退出。用于每轮 product-smoke 检查
    // 捕获产品可见回归（如 R428 min-size:auto 致 welcome +7.65pp，此前藏了 14 轮）。
    let mut max_diff: Option<f64> = None;
    let mut channel_diff: u8 = 0;
    let mut pixel_radius: usize = 0;
    let mut geometry_oracle: Option<String> = None;
    let mut max_geometry_diff: f64 = 2.0;
    let mut region_gates = Vec::new();
    // DC-13 line 321：经 zero-webview 嵌入边界渲染（对照 engine-direct 默认路径），
    // 验证产品层与 WebView 层不互相掩盖问题。仅自包含 fixture（无外链资源）适用。
    let mut via_webview = false;
    // DC-13 line 322-326：结构自动检查（同父兄弟盒 border-box 重叠 = 产品可见排版退化）。
    // 仅 engine-direct 路径（render_to_framebuffer_with_layout_with_base 暴露 layout 树）。
    let mut struct_check = false;
    // DC-13 line 322：「四个 feature card」/「四个 nav button」等元素计数断言（可重复）。
    // 格式 `<class>:<min-count>`（如 `card:4`）；按 collect_dom_labels 的 tag.class 标签匹配。
    let mut expect_classes: Vec<(String, usize)> = Vec::new();
    // DC-13 line 323/324：行数断言（可重复）。「标题不拆行」/「tagline 保持 2 行」等；
    // 格式 `<class>:<line-count>`，经 content_height / line_height 估算（无度量则跳过）。
    let mut expect_lines: Vec<(String, usize)> = Vec::new();
    // DC-13 line 327：行数下限断言（可重复）。「正文按宽度换行」= 行数 ≥ N（如 text-justify:2
    // 证明段落换行而非压成一行）。格式 `<class>:<min-lines>`。
    let mut expect_lines_min: Vec<(String, usize)> = Vec::new();
    // DC-13 line 327「参与方 Logo 网格 SVG/PNG 可见不退化」：opt-in 替换元素（img/logo）塌缩
    // 检查。仅对「所有图片都应可见」的 fixture 启用——morning 含故意缺失图（cc_unavailable），
    // 通用 gate 会误报。
    let mut check_img_visibility = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--base-dir" => {
                i += 1;
                if i < args.len() {
                    base_dir = Some(args[i].clone());
                }
            }
            "--oracle" => {
                i += 1;
                if i < args.len() {
                    oracle = Some(args[i].clone());
                }
            }
            "--out" => {
                i += 1;
                if i < args.len() {
                    out = Some(args[i].clone());
                }
            }
            "--width" => {
                i += 1;
                if i < args.len() {
                    width = args[i].parse().unwrap_or(800);
                }
            }
            "--height" => {
                i += 1;
                if i < args.len() {
                    height = args[i].parse().unwrap_or(600);
                }
            }
            "--max-diff" => {
                i += 1;
                if i < args.len() {
                    max_diff = args[i].parse().ok();
                }
            }
            "--channel-diff" => {
                i += 1;
                if i < args.len() {
                    channel_diff = args[i].parse().unwrap_or(0);
                }
            }
            "--pixel-radius" => {
                i += 1;
                if i < args.len() {
                    pixel_radius = args[i].parse().unwrap_or(0);
                }
            }
            "--geometry-oracle" => {
                i += 1;
                if i < args.len() {
                    geometry_oracle = Some(args[i].clone());
                }
            }
            "--max-geometry-diff" => {
                i += 1;
                if i < args.len() {
                    max_geometry_diff = args[i].parse().unwrap_or(2.0);
                }
            }
            "--region" => {
                i += 1;
                if i < args.len() {
                    match product_smoke::parse_region_gate(&args[i]) {
                        Some(gate) => region_gates.push(gate),
                        None => eprintln!("Warning: --region expects <id>:<max-pct>, got {}", args[i]),
                    }
                }
            }
            "--via-webview" => {
                via_webview = true;
            }
            "--struct-check" => {
                struct_check = true;
            }
            "--expect-class" => {
                // --expect-class <class>:<min-count>（可重复），结构计数断言。
                i += 1;
                if i < args.len() {
                    if let Some((class, n)) = args[i].rsplit_once(':') {
                        if let Ok(min) = n.parse::<usize>() {
                            expect_classes.push((class.to_string(), min));
                        } else {
                            eprintln!("Warning: --expect-class count not a number: {}", args[i]);
                        }
                    } else {
                        eprintln!("Warning: --expect-class expects <class>:<count>, got {}", args[i]);
                    }
                }
            }
            "--expect-lines" => {
                // --expect-lines <class>:<line-count>（可重复），行数断言。
                i += 1;
                if i < args.len() {
                    if let Some((class, n)) = args[i].rsplit_once(':') {
                        if let Ok(lines) = n.parse::<usize>() {
                            expect_lines.push((class.to_string(), lines));
                        } else {
                            eprintln!("Warning: --expect-lines count not a number: {}", args[i]);
                        }
                    } else {
                        eprintln!("Warning: --expect-lines expects <class>:<count>, got {}", args[i]);
                    }
                }
            }
            "--expect-lines-min" => {
                // --expect-lines-min <class>:<min-lines>（可重复），行数下限断言。
                i += 1;
                if i < args.len() {
                    if let Some((class, n)) = args[i].rsplit_once(':')
                        && let Ok(min) = n.parse::<usize>()
                    {
                        expect_lines_min.push((class.to_string(), min));
                    } else {
                        eprintln!("Warning: --expect-lines-min expects <class>:<count>, got {}", args[i]);
                    }
                }
            }
            "--check-img-visibility" => {
                check_img_visibility = true;
            }
            s if !s.starts_with('-') && html_path.is_none() => {
                html_path = Some(s.to_string());
            }
            _ => {}
        }
        i += 1;
    }

    let Some(html_path) = html_path else {
        eprintln!("Error: product-smoke requires an <html-path> argument");
        eprintln!("Usage: product-smoke <html-path> [--base-dir <dir>] [--oracle <png>] [--out <png>]");
        std::process::exit(1);
    };

    let html = match std::fs::read_to_string(&html_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading {html_path}: {e}");
            std::process::exit(1);
        }
    };

    let config = reftest::ReftestConfig {
        viewport_width: width,
        viewport_height: height,
        ..Default::default()
    };
    let base = base_dir.as_deref().map(std::path::Path::new);

    eprintln!(
        "product-smoke: rendering {html_path} ({}x{}, base_dir={:?}, via_webview={})",
        width, height, base, via_webview
    );
    // DC-13 结构检查（--struct-check / --expect-class）需 layout 树，仅 engine-direct 路径暴露；
    // via_webview 路径无 layout 访问，结构检查静默跳过。
    let need_layout = struct_check
        || !expect_classes.is_empty()
        || !expect_lines.is_empty()
        || !expect_lines_min.is_empty()
        || check_img_visibility
        || geometry_oracle.is_some()
        || !region_gates.is_empty();
    // layout_html = render_html 实际解析的 HTML（经 script DOM 变更后的 mutated_html）。
    // 结构检查须用它建 labels（node_id 与 layout 树一致），否则真元素误标 "(anon)"。
    // R2198：paint_skip = orphan inline 元素集（R2197 Phase A slice 3），供 struct-check
    // `check_sibling_overlaps` 排除 orphan 假阳性（orphan 是 hit-test proxy，paint-skip）。
    // via_webview / 无 layout 路径无 paint_skip（空集）。
    let (fb, layout_root, layout_html, paint_skip) = if via_webview {
        (
            reftest::render_via_webview_to_framebuffer_with_base(&html, "", &config, base),
            None,
            html.clone(),
            std::collections::HashSet::new(),
        )
    } else if need_layout {
        let (fb, root, ps, rendered_html) =
            reftest::render_to_framebuffer_with_layout_and_paint_skip_with_base(&html, "", &config, base);
        (fb, Some(root), rendered_html, ps)
    } else {
        (
            reftest::render_to_framebuffer_with_base(&html, "", &config, base),
            None,
            html.clone(),
            std::collections::HashSet::new(),
        )
    };

    let out_path = out.as_deref().unwrap_or("product-smoke-cpu.png");
    reftest::save_fb_as_png(&fb, std::path::Path::new(out_path));
    eprintln!("wrote ZeroWeb CPU PNG: {out_path} ({}x{})", fb.width, fb.height);

    let mut visual_failed = false;
    if let Some(oracle_path) = oracle {
        match load_png_to_framebuffer(&oracle_path) {
            Ok(oracle_fb) => {
                if oracle_fb.width != fb.width || oracle_fb.height != fb.height {
                    eprintln!(
                        "Warning: size mismatch ZeroWeb={}x{} vs oracle={}x{}; clamping comparison to min",
                        fb.width, fb.height, oracle_fb.width, oracle_fb.height
                    );
                }
                let (_, max_channel_diff) = reftest::compare_pixels(&fb, &oracle_fb, channel_diff);
                let diff_px = product_smoke::full_diff_pixels(&fb, &oracle_fb, channel_diff, pixel_radius);
                let w = fb.width.min(oracle_fb.width) as usize;
                let h = fb.height.min(oracle_fb.height) as usize;
                let total = w * h;
                let pct = if total > 0 {
                    100.0 * diff_px as f64 / total as f64
                } else {
                    0.0
                };
                println!(
                    "product-smoke diff vs chromium {}: {diff_px}/{total} px ({:.2}%, channel tolerance={}, pixel radius={}, max channel diff={})",
                    oracle_path, pct, channel_diff, pixel_radius, max_channel_diff
                );
                if let Some(threshold) = max_diff
                    && !product_smoke::full_diff_passes(pct, threshold)
                {
                    eprintln!(
                        "REGRESSION: product-smoke diff {:.2}% meets or exceeds strict threshold {:.2}%",
                        pct, threshold
                    );
                    visual_failed = true;
                }

                if let Some(path) = geometry_oracle.as_deref() {
                    match product_smoke::load_geometry_oracle(path) {
                        Ok(geometry) => {
                            let actual = layout_root
                                .as_ref()
                                .map(|root| product_smoke::collect_layout_rects(root, &layout_html))
                                .unwrap_or_default();
                            for gate in &region_gates {
                                match product_smoke::geometry_diff(&gate.id, &geometry, &actual) {
                                    Ok(diff) => {
                                        println!(
                                            "product-smoke geometry #{}: max delta {:.2}px (threshold {:.2}px)",
                                            gate.id, diff, max_geometry_diff
                                        );
                                        if diff > max_geometry_diff {
                                            visual_failed = true;
                                        }
                                    }
                                    Err(error) => {
                                        eprintln!("REGRESSION: {error}");
                                        visual_failed = true;
                                    }
                                }
                                match (actual.get(&gate.id), geometry.rects.get(&gate.id)) {
                                    (Some(actual_rect), Some(expected_rect)) => {
                                        let pct = product_smoke::region_diff_pct(
                                            &fb,
                                            &oracle_fb,
                                            *actual_rect,
                                            *expected_rect,
                                            channel_diff,
                                            pixel_radius,
                                        );
                                        println!(
                                            "product-smoke region #{}: {:.2}% (threshold {:.2}%)",
                                            gate.id, pct, gate.max_diff_pct
                                        );
                                        if pct > gate.max_diff_pct {
                                            visual_failed = true;
                                        }
                                    }
                                    _ => {
                                        eprintln!("REGRESSION: geometry missing #{}", gate.id);
                                        visual_failed = true;
                                    }
                                }
                            }
                        }
                        Err(error) => {
                            eprintln!("Error loading geometry oracle: {error}");
                            visual_failed = true;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error loading oracle {oracle_path}: {e}");
                std::process::exit(1);
            }
        }
    }

    // DC-13 line 322-326：结构自动检查（兄弟盒重叠 + 元素计数），仅 engine-direct 路径
    //（--struct-check / --expect-class 且非 --via-webview）执行。与像素 diff 门禁互补——
    // 像素 diff 量化整体差距，本检查定位结构性退化（兄弟盒重叠 / 卡片按钮塌缩 = 用户可见
    // 排版 breakage，即使像素差小）。退出码 3（区别于像素 diff 门禁的 2 与参数错误的 1）。
    if let Some(root) = layout_root {
        if std::env::var("LAYOUT_DUMP").is_ok() {
            reftest::dump_layout_tree(&root, &layout_html);
        }
        let labels = reftest::collect_dom_labels(&layout_html);
        let mut issues: Vec<String> = Vec::new();
        if struct_check {
            issues.extend(reftest::check_sibling_overlaps(&root, &labels, &paint_skip));
            // R1579：check_collapsed_containers 入 product-smoke gate。R1576（inline>inline-block
            // 递归）+ R1578（inline>inline-IMG 固有尺寸）修了 wintertc footer 塌缩后，产品 fixture
            // 不再含已知塌缩，可入 gate 守未来 collapse 回归（exit 3）。诊断仍经 struct-sweep。
            issues.extend(reftest::check_collapsed_containers(&root, &labels));
            // DC-13 line 325「不同 sibling card/link/shortcut 的文本不串联」：检测容器把 block
            // 子元素文本吸收进自身 IFC（R109 inline-ownership 退化）。信号 = 容器 text_node 映射
            // 含子元素子树的非空白文本节点（store_font_sizes_from_ifc 主路径存储）。welcome/
            // wintertc/morning 当前不触发（grid/flex 容器 text_node 映射为空）；守未来串联回归。
            let (has_direct_text, non_ws_text_nodes) = reftest::collect_concat_dom_info(&layout_html);
            issues.extend(reftest::check_text_concatenation(
                &root,
                &labels,
                &has_direct_text,
                &non_ws_text_nodes,
            ));
        }
        // DC-13 line 327 opt-in：img/logo 塌缩检查（仅对「所有图片都应可见」的 fixture）。
        if check_img_visibility {
            issues.extend(reftest::check_replaced_collapse(&root, &labels));
        }
        // 元素计数断言（DC-13「四个 feature card」/「四个 nav button」等）。
        for (class, min) in &expect_classes {
            let count = reftest::count_boxes_by_class(&root, &labels, class);
            if count < *min {
                issues.push(format!("class .{class}: {count} boxes (expected >= {min})"));
            }
        }
        // 行数断言（DC-13「标题不拆行」/「tagline 保持 2 行」等）。
        for (class, want) in &expect_lines {
            match reftest::count_lines_for_class(&root, &labels, class) {
                Some(got) if got == *want => {}
                Some(got) => issues.push(format!("class .{class}: {got} lines (expected {want})")),
                None => eprintln!("Warning: --expect-lines .{class}: no line metric (skip)"),
            }
        }
        // 行数下限断言（DC-13「正文按宽度换行」= 行数 ≥ N）。
        for (class, min) in &expect_lines_min {
            match reftest::count_lines_for_class(&root, &labels, class) {
                Some(got) if got >= *min => {}
                Some(got) => issues.push(format!("class .{class}: {got} lines (expected >= {min})")),
                None => eprintln!("Warning: --expect-lines-min .{class}: no line metric (skip)"),
            }
        }
        if issues.is_empty() {
            let overlap_note = if struct_check { " + sibling-overlap" } else { "" };
            let img_note = if check_img_visibility { " + img-visible" } else { "" };
            let count_note = if expect_classes.is_empty() {
                String::new()
            } else {
                format!(
                    " + count[{}]",
                    expect_classes
                        .iter()
                        .map(|(c, n)| format!(".{c}={n}"))
                        .collect::<Vec<_>>()
                        .join(",")
                )
            };
            let lines_note = if expect_lines.is_empty() && expect_lines_min.is_empty() {
                String::new()
            } else {
                let exact = expect_lines
                    .iter()
                    .map(|(c, n)| format!(".{c}={n}"))
                    .collect::<Vec<_>>();
                let min = expect_lines_min
                    .iter()
                    .map(|(c, n)| format!(".{c}>={n}"))
                    .collect::<Vec<_>>();
                let mut all = exact;
                all.extend(min);
                format!(" + lines[{}]", all.join(","))
            };
            println!("product-smoke struct-check: PASS (0 issues{overlap_note}{img_note}{count_note}{lines_note})");
        } else {
            println!("product-smoke struct-check: FAIL ({} issue(s))", issues.len());
            for iss in &issues {
                println!("  - {iss}");
            }
            std::process::exit(3);
        }
    }
    if visual_failed {
        std::process::exit(2);
    }
}

/// `perf` 子命令 — 页面级性能基准（性能门禁体系的页面场景测量）。
///
/// 对每个 fixture 页面做 `--iterations` 次首屏渲染（第 1 次为 warmup，付字体加载/
/// 图片缓存等进程级一次性成本，不计入样本），输出各阶段耗时（parse/style/layout/
/// paint/total，来自 `zero_engine::PipelineTimings`）与首屏墙钟耗时、进程峰值 RSS
/// （Linux VmHWM / macOS getrusage / 其他平台 null）。输出 JSON 供
/// `scripts/bench-report.sh` 合并进 bench 报告，`scripts/perf-gate.sh` 对比基线
/// （见 docs/specs/performance-and-resource-budget.md）。
///
/// 用法：
///   zero-wpt-runner perf --scenario <id>:<path> [--scenario ...] [--base-dir <dir>]
///                        [--width N] [--height N] [--iterations N]
fn cmd_perf(args: &[String]) {
    let t_start = std::time::Instant::now();
    let mut scenarios: Vec<(String, String)> = Vec::new();
    let mut base_dir: Option<String> = None;
    let mut width: u32 = 800;
    let mut height: u32 = 600;
    let mut iterations: usize = 15;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--scenario" => {
                i += 1;
                if i < args.len() {
                    if let Some((id, path)) = args[i].split_once(':') {
                        scenarios.push((id.to_string(), path.to_string()));
                    } else {
                        eprintln!("perf: --scenario 格式应为 <id>:<path>，忽略 {}", args[i]);
                    }
                }
            }
            "--base-dir" => {
                i += 1;
                if i < args.len() {
                    base_dir = Some(args[i].clone());
                }
            }
            "--width" => {
                i += 1;
                if i < args.len() {
                    width = args[i].parse().unwrap_or(width);
                }
            }
            "--height" => {
                i += 1;
                if i < args.len() {
                    height = args[i].parse().unwrap_or(height);
                }
            }
            "--iterations" => {
                i += 1;
                if i < args.len() {
                    // 至少 1 次 warmup + 1 次计时的样本
                    iterations = args[i].parse().unwrap_or(iterations).max(2);
                }
            }
            other => eprintln!("perf: 未知参数 {other}，忽略"),
        }
        i += 1;
    }

    if scenarios.is_empty() {
        eprintln!("perf: 至少需要一个 --scenario <id>:<path>");
        std::process::exit(2);
    }

    let config = reftest::ReftestConfig {
        viewport_width: width,
        viewport_height: height,
        ..Default::default()
    };

    let mut out_scenarios = Vec::new();
    // startup_ms：进程入口（cmd_perf 起点）→ 首个场景 warmup 渲染完成，即「冷启动到首帧」。
    let mut startup_ms: Option<f64> = None;
    for (id, path) in &scenarios {
        let html = match std::fs::read_to_string(path) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("perf: 无法读取 fixture {path}: {e}");
                std::process::exit(2);
            }
        };
        // base_dir 缺省取 fixture 所在目录（与 product-smoke 显式传 base-dir 等价，
        // 自包含 fixture 传 None 路径行为一致）。
        let base = base_dir
            .as_deref()
            .map(std::path::Path::new)
            .or_else(|| std::path::Path::new(path).parent());
        // warmup：付字体加载/图片缓存等进程级一次性成本，不计入样本
        let (_fb, _timings) = reftest::render_to_framebuffer_with_timings(&html, "", &config, base);
        if startup_ms.is_none() {
            startup_ms = Some(t_start.elapsed().as_secs_f64() * 1000.0);
        }
        let mut samples = Vec::new();
        for _ in 0..(iterations - 1) {
            let t0 = std::time::Instant::now();
            let (_fb, timings) = reftest::render_to_framebuffer_with_timings(&html, "", &config, base);
            samples.push(serde_json::json!({
                "parse_ms": timings.parse_ms,
                "style_ms": timings.style_ms,
                "layout_ms": timings.layout_ms,
                "paint_ms": timings.paint_ms,
                "total_ms": timings.total_ms,
                "wall_ms": t0.elapsed().as_secs_f64() * 1000.0,
            }));
        }
        out_scenarios.push(serde_json::json!({
            "id": id,
            "fixture": path,
            "viewport": [width, height],
            "iterations": iterations - 1,
            "samples": samples,
        }));
    }

    // VmHWM 为高水位标记，全程结束后读取即进程峰值
    let (peak_rss_mb, rss_method) = perf_peak_rss_mb();
    let report = serde_json::json!({
        "schema_version": 1,
        "kind": "perf-pages",
        "os": std::env::consts::OS,
        "cpu_model": perf_cpu_model_name(),
        "cpu_cores": std::thread::available_parallelism().map(|n| n.get()).unwrap_or(0),
        "startup_ms": startup_ms,
        "resource": { "peak_rss_mb": peak_rss_mb, "method": rss_method },
        "scenarios": out_scenarios,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string())
    );
}

/// 进程峰值 RSS（MB）。Linux：`/proc/self/status` VmHWM（高水位，单位 kB）；
/// macOS：`getrusage` `ru_maxrss`（单位字节）；其他平台无测量 → `(None, "none")`。
fn perf_peak_rss_mb() -> (Option<f64>, &'static str) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if let Some(kb) = line
                    .strip_prefix("VmHWM:")
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|v| v.parse::<f64>().ok())
                {
                    return (Some(kb / 1024.0), "vmhwm");
                }
            }
        }
        (None, "vmhwm")
    }
    #[cfg(target_os = "macos")]
    {
        use libc::{RUSAGE_SELF, getrusage, rusage};
        let mut usage: rusage = unsafe { std::mem::zeroed() };
        if unsafe { getrusage(RUSAGE_SELF, &mut usage) } == 0 {
            // macOS ru_maxrss 单位是字节（Linux 是 kB）
            (Some(usage.ru_maxrss as f64 / (1024.0 * 1024.0)), "rusage")
        } else {
            (None, "rusage")
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        (None, "none")
    }
}

/// CPU 型号名（报告元数据，用于基线硬件固定判定）。Linux：/proc/cpuinfo 首个
/// `model name`；其他平台 `unknown`（os 已由 `std::env::consts::OS` 覆盖）。
fn perf_cpu_model_name() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(cpuinfo) = std::fs::read_to_string("/proc/cpuinfo") {
            for line in cpuinfo.lines() {
                if let Some(name) = line.strip_prefix("model name").and_then(|rest| rest.split(':').nth(1)) {
                    return name.trim().to_string();
                }
            }
        }
        "unknown".to_string()
    }
    #[cfg(not(target_os = "linux"))]
    {
        "unknown".to_string()
    }
}

/// 把 PNG 文件解码为 FrameBuffer（RGBA8），供与 ZeroWeb 渲染结果像素对比。
fn load_png_to_framebuffer(path: &str) -> Result<zero_render_foundation::surface::FrameBuffer, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open {path}: {e}"))?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("read_info: {e}"))?;
    let (w, h) = (reader.info().width, reader.info().height);
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let frame = reader.next_frame(&mut buf).map_err(|e| format!("next_frame: {e}"))?;
    let rgba = reftest::convert_png_buffer_to_rgba(&buf[..frame.buffer_size()], frame.color_type, frame.bit_depth);
    let mut fb = zero_render_foundation::surface::FrameBuffer::new(w, h);
    fb.data = rgba;
    Ok(fb)
}

/// DC-14 非平凡性检查：判定一帧是否「退化/接近纯色」（>99.9% 像素为同一颜色）。
///
/// 这类帧通常是 parsing/animation/print/crashtest/JS-only 用例在 headless 下渲染为
/// 空白/纯色，若 ZeroWeb 也渲染为空白则 z_vs_chr≈0% → 假 PASS（历史 R135/R149 PNG
/// 加载 bug 同类）。采样每 16 个像素估算主色占比，O(n/16) 避免 13793 case × 全帧扫描。
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
    let jobs = effective_jobs(options);
    eprintln!("Using {jobs} test job(s).");
    let results = run_wpt_cases(&tests, &ctx, jobs);
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
    let jobs = effective_jobs(options);
    eprintln!("Using {jobs} test job(s).");
    let results = run_wpt_cases(&tests, &ctx, jobs);
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
    use reftest::{ReftestCategory, ReftestResult, run_reftest, run_reftest_gpu};

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
    let jobs = effective_jobs(options);
    eprintln!("Using {jobs} test job(s).");

    let start = std::time::Instant::now();

    let results: Vec<ReftestResult> = parallel_map(&filtered, jobs, |(idx, case)| {
        let _timer = CaseTimer::new(&case.id);
        let mut config = configs[*idx].clone();
        config.viewport_width = options.viewport_width as u32;
        config.viewport_height = options.viewport_height as u32;
        config.media_type = options.media_type;

        if options.use_gpu {
            run_reftest_gpu(case, &config)
        } else {
            run_reftest(case, &config)
        }
    });

    let mut pass_count = 0usize;
    let mut fail_count = 0usize;

    for result in &results {
        let passed = result.passed;
        let status_char = if passed { '✓' } else { '✗' };
        eprintln!("  {} {} ({:.2}%)", status_char, result.id, result.diff_ratio * 100.0);

        if passed {
            pass_count += 1;
        } else {
            fail_count += 1;
            eprintln!("    {}", result.message);
        }
    }

    let duration = start.elapsed();
    let total = pass_count + fail_count;
    let pass_rate = if total > 0 {
        pass_count as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    // DC-14 self-source 三态分类（严格容差真通过率 = 唯一可信达标指标）。
    // results 与 filtered 同序，category 取自 configs[filtered[i].0]。
    let categories: Vec<ReftestCategory> = filtered.iter().map(|(i, _)| configs[*i].category).collect();
    print_dc14_three_state(&results, &categories);

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

/// DC-14 self-source 三态分类报告 + 非平凡性（近纯色退化）审计。
///
/// 弥补 self-source 路径此前的 loose 二元（通过/失败）报告——在保持与实际 pass/fail
/// 一致（`strict_credible + strict_suspicious + near_pass == pass_count`、
/// `mismatch == fail_count`）的前提下，把通过项进一步按 DC-14 锁定严格容差
///（布局 ≤0.1% & channel≤2；文字 ≤0.5% & channel≤5）拆成两态，并对 strict-pass 施加
/// 非平凡性检查（`test_near_solid`，DC-14 防退化假绿）：
/// - **真通过（可信）**：`passed && ≤strict && !test_near_solid`（唯一可信达标指标）
/// - **真通过（可疑）**：`passed && ≤strict && test_near_solid`——test 帧近纯色，须审计
///   （test==ref 退化假绿，如 headless 空白页；历史 R135/R149 harness PNG 加载 bug）
/// - **近似通过**：`passed` 但不满足严格容差（loose 通过但非严格，含同源假通过与字体噪声）
/// - **不一致**：`!passed`（loose 失败）
///
/// near/mismatch 边界用 `result.passed`（编码实际有效 loose 阈值，含 ZERO_REFTEST_STRICT
/// 与 per-test fuzzy override），因此计数与上方 pass/fail 报告自洽；strict 边界用 DC-14
/// 锁定阈值，与 oracle 路径（`cmd_reftest_oracle`，R851 三态 + R852 非平凡性）口径一致。
fn print_dc14_three_state(results: &[reftest::ReftestResult], categories: &[reftest::ReftestCategory]) {
    let mut strict_credible = 0usize;
    let mut strict_suspicious = 0usize;
    let mut near_pass = 0usize;
    let mut mismatch = 0usize;
    let mut suspicious_ids: Vec<&str> = Vec::new();
    for (r, cat) in results.iter().zip(categories.iter()) {
        let strict_ratio = cat.strict_max_diff_ratio();
        let strict_chan = cat.strict_max_channel_diff();
        if r.passed && r.diff_ratio <= strict_ratio && r.max_channel_diff <= strict_chan {
            if r.test_near_solid {
                strict_suspicious += 1;
                suspicious_ids.push(r.id.as_str());
            } else {
                strict_credible += 1;
            }
        } else if r.passed {
            near_pass += 1;
        } else {
            mismatch += 1;
        }
    }
    let total = results.len();
    let pct = |n: usize| {
        if total > 0 {
            100.0 * n as f64 / total as f64
        } else {
            0.0
        }
    };
    eprintln!();
    eprintln!("  ── DC-14 self-source 三态分类 + 非平凡性（严格容差 = 唯一可信达标指标）──");
    eprintln!(
        "  真通过-可信 (passed 且 ≤strict 且非近纯色): {} ({:.1}%)",
        strict_credible,
        pct(strict_credible)
    );
    eprintln!(
        "  真通过-可疑 (≤strict 但 test 近纯色，须审计): {} ({:.1}%)",
        strict_suspicious,
        pct(strict_suspicious)
    );
    eprintln!("  近似通过 (passed 但 >strict): {} ({:.1}%)", near_pass, pct(near_pass));
    eprintln!("  不一致 (failed):            {} ({:.1}%)", mismatch, pct(mismatch));
    // 列出可疑 case 供人工审计（DC-14：退化假绿不得计入 credible pass）。
    if !suspicious_ids.is_empty() {
        eprintln!("  可疑（近纯色）case 审计列表（前 20）：");
        for id in suspicious_ids.iter().take(20) {
            eprintln!("    - {id}");
        }
    }
}

/// `reftest-upstream` 子命令 — 运行从 wpt-data/ 加载的真实上游 WPT reftest。
fn cmd_reftest_upstream(options: &CliOptions, filter: Option<&str>) {
    use reftest::{ReftestResult, run_reftest_gpu_with_base, run_reftest_with_base};

    let wpt_data_dir = match &options.wpt_data {
        Some(dir) => std::path::PathBuf::from(dir),
        None => std::path::PathBuf::from("tests/wpt-runner/wpt-data"),
    };

    if !wpt_data_dir.is_dir() {
        eprintln!("Error: wpt-data directory not found: {}", wpt_data_dir.display());
        eprintln!("Run `make fetch-wpt-data` first.");
        std::process::exit(1);
    }

    eprintln!("Loading upstream reftests from: {}", wpt_data_dir.display());
    let file_cases = wpt_file_loader::load_file_reftests(&wpt_data_dir);

    if file_cases.is_empty() {
        eprintln!("No upstream reftest cases found in {}", wpt_data_dir.display());
        std::process::exit(1);
    }

    // 过滤
    let filtered: Vec<&wpt_file_loader::FileReftestCase> = file_cases
        .iter()
        .filter(|case| {
            if let Some(f) = filter {
                case.id.contains(f)
                    || f.eq_ignore_ascii_case("layout") && case.category == reftest::ReftestCategory::Layout
                    || f.eq_ignore_ascii_case("text") && case.category == reftest::ReftestCategory::Text
            } else {
                true
            }
        })
        .collect();

    eprintln!("Running {} upstream reftest cases...", filtered.len());
    let jobs = effective_jobs(options);
    eprintln!("Using {jobs} test job(s).");

    let skip_count = 0usize;
    let start = std::time::Instant::now();

    let results: Vec<ReftestResult> = parallel_map(&filtered, jobs, |case| {
        let reftest_case = case.to_reftest_case();
        let _timer = CaseTimer::new(&reftest_case.id);
        let mut config = case.to_config(options.viewport_width as u32, options.viewport_height as u32);
        config.media_type = options.media_type;
        config.wpt_root = Some(wpt_data_dir.clone());
        let base_dir = case.base_dir.as_deref();

        if options.use_gpu {
            run_reftest_gpu_with_base(&reftest_case, &config, base_dir)
        } else {
            run_reftest_with_base(&reftest_case, &config, base_dir)
        }
    });

    let mut pass_count = 0usize;
    let mut fail_count = 0usize;

    for result in &results {
        let status_char = if result.passed { '✓' } else { '✗' };
        eprintln!("  {} {} ({:.2}%)", status_char, result.id, result.diff_ratio * 100.0);

        if result.passed {
            pass_count += 1;
        } else {
            fail_count += 1;
            eprintln!("    {}", result.message);
        }
    }

    let duration = start.elapsed();
    let total = pass_count + fail_count + skip_count;
    let pass_rate = if total > 0 {
        pass_count as f64 / total as f64 * 100.0
    } else {
        0.0
    };

    // 按目录分类统计
    let mut dir_stats: std::collections::HashMap<String, (usize, usize)> = std::collections::HashMap::new();
    for (i, case) in filtered.iter().enumerate() {
        let dir = case.id.split('/').nth(1).unwrap_or("unknown").to_string();
        let (passed, total) = dir_stats.entry(dir).or_insert((0, 0));
        *total += 1;
        if results[i].passed {
            *passed += 1;
        }
    }

    // 输出报告
    eprintln!("\n═══════════════════════════════════════════════");
    eprintln!("  Upstream WPT Reftest Report");
    eprintln!("═══════════════════════════════════════════════");
    eprintln!("  Source:  {}", wpt_data_dir.display());
    eprintln!("  Total:   {}", total);
    eprintln!("  Passed:  {}", pass_count);
    eprintln!("  Failed:  {}", fail_count);
    eprintln!("  Skipped: {}", skip_count);
    eprintln!("  Pass Rate: {:.1}%", pass_rate);
    eprintln!("  Duration:  {:.2}s", duration.as_secs_f64());
    eprintln!();

    // 按目录输出
    let mut dirs: Vec<_> = dir_stats.iter().collect();
    dirs.sort_by_key(|(k, _)| k.as_str());
    for (dir, (pass, total_count)) in &dirs {
        let rate = if *total_count > 0 {
            *pass as f64 / *total_count as f64 * 100.0
        } else {
            0.0
        };
        eprintln!("  {:30} {}/{} ({:.1}%)", format!("{}/", dir), pass, total_count, rate);
    }

    // DC-14 self-source 三态分类（严格容差真通过率 = 唯一可信达标指标）。
    // results 与 filtered 同序，category 取自 filtered（FileReftestCase.category）。
    let categories: Vec<reftest::ReftestCategory> = filtered.iter().map(|c| c.category).collect();
    print_dc14_three_state(&results, &categories);

    // JSON 输出
    if matches!(options.format, OutputFormat::Json) {
        let json = format_reftest_report_json(&results, pass_count, fail_count, pass_rate, duration);
        println!("{json}");
    }

    if fail_count > 0 {
        std::process::exit(1);
    }
}

/// `layout-dump` 子命令（B1/P3 布局树 dump golden）— 渲染上游 WPT reftest 的 test 页，
/// 输出布局树 dump（stderr，格式与 product-smoke --struct-check 的 LAYOUT_DUMP 一致：
/// 固定 1 位小数，便于 golden 对比）。
///
/// 配合 scripts/run-layout-golden.sh 使用：
///   bash scripts/run-layout-golden.sh --update [filter]   # 生成/更新 golden
///   bash scripts/run-layout-golden.sh [filter]            # 对比（不一致退出 1）
/// golden 存 tests/wpt-runner/layout-golden/（提交进 git，测试资产化）。
fn cmd_layout_dump(options: &CliOptions, filter: Option<&str>) {
    use reftest::{dump_layout_tree, render_to_framebuffer_with_layout_with_base};

    let wpt_data_dir = match &options.wpt_data {
        Some(dir) => std::path::PathBuf::from(dir),
        None => std::path::PathBuf::from("tests/wpt-runner/wpt-data"),
    };

    if !wpt_data_dir.is_dir() {
        eprintln!("Error: wpt-data directory not found: {}", wpt_data_dir.display());
        eprintln!("Run `make fetch-wpt-data` first.");
        std::process::exit(1);
    }

    let file_cases = wpt_file_loader::load_file_reftests(&wpt_data_dir);

    // 过滤（与 reftest-upstream 同语义：子串匹配 case.id）
    let filtered: Vec<&wpt_file_loader::FileReftestCase> = file_cases
        .iter()
        .filter(|case| {
            if let Some(f) = filter {
                case.id.contains(f)
            } else {
                true
            }
        })
        .collect();

    eprintln!("Dumping layout tree for {} case(s)...", filtered.len());
    eprintln!("(输出为 LAYOUT_DUMP 格式，供 scripts/run-layout-golden.sh 做 golden 对比)");

    for case in &filtered {
        let mut config = case.to_config(options.viewport_width as u32, options.viewport_height as u32);
        config.media_type = options.media_type;
        config.wpt_root = Some(wpt_data_dir.clone());
        let base_dir = case.base_dir.as_deref();

        // 只渲染 test 页；ref 页布局不在 dump 范围
        let (_, root, rendered_html) =
            render_to_framebuffer_with_layout_with_base(&case.test_html, "", &config, base_dir);

        eprintln!("##### {} #####", case.id);
        dump_layout_tree(&root, &rendered_html);
    }
}

/// `reftest-oracle` 子命令（DC-14 独立 Oracle）— 渲染上游 WPT reftest 的 test 页，
/// 与 chromium Oracle 截图（`oracle-shots/{safe_id}.png`）对比，报告 chromium-Oracle
/// 一致率（DC-14 真通过指标，替代 ZeroWeb self-ref 的 ~46.5% 假通过）。
///
/// 用法：
///   zero-wpt-runner reftest-oracle [filter]
///   env: ORACLE_DIR=<dir>（默认 tests/wpt-runner/oracle-shots）
///        ORACLE_PASS_RATIO=<f>（默认 0.01=1%，z_vs_chr < 此值判 oracle-pass）
///        --wpt-data <dir>、--viewport、--jobs 等通用选项
fn cmd_reftest_oracle(options: &CliOptions, filter: Option<&str>) {
    use reftest::{ReftestConfig, compare_pixels, render_to_framebuffer_with_base};

    let wpt_data_dir = match &options.wpt_data {
        Some(p) => std::path::PathBuf::from(p),
        None => std::path::PathBuf::from("tests/wpt-runner/wpt-data"),
    };
    if !wpt_data_dir.is_dir() {
        eprintln!("Error: wpt-data directory not found: {}", wpt_data_dir.display());
        eprintln!("Run `make fetch-wpt-data` first.");
        std::process::exit(1);
    }
    let oracle_dir_str = std::env::var("ORACLE_DIR").unwrap_or_else(|_| "tests/wpt-runner/oracle-shots".to_string());
    let oracle_dir = std::path::Path::new(&oracle_dir_str);
    if !oracle_dir.is_dir() {
        eprintln!(
            "Error: oracle directory not found: {} (capture via capture-oracle-per-dir.mjs)",
            oracle_dir.display()
        );
        std::process::exit(1);
    }
    let pass_ratio: f64 = std::env::var("ORACLE_PASS_RATIO")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.01);

    let file_cases = wpt_file_loader::load_file_reftests(&wpt_data_dir);
    if file_cases.is_empty() {
        eprintln!("No upstream reftest cases found in {}", wpt_data_dir.display());
        std::process::exit(1);
    }
    let filtered: Vec<&wpt_file_loader::FileReftestCase> = file_cases
        .iter()
        .filter(|case| {
            if let Some(f) = filter {
                case.id.contains(f)
                    || f.eq_ignore_ascii_case("layout") && case.category == reftest::ReftestCategory::Layout
                    || f.eq_ignore_ascii_case("text") && case.category == reftest::ReftestCategory::Text
            } else {
                true
            }
        })
        .collect();

    eprintln!(
        "reftest-oracle: {} cases, oracle_dir={}, pass_ratio={:.2}%",
        filtered.len(),
        oracle_dir.display(),
        pass_ratio * 100.0
    );
    let jobs = effective_jobs(options);
    eprintln!("Using {jobs} job(s).");

    // (safe_id, has_oracle, z_vs_chr_pct, strict_thresh_pct, oracle_near_solid)
    // strict_thresh = DC-14 锁定严格容差（布局 0.1% / 文字 0.5%，ReftestCategory::strict_max_diff_ratio），
    // 用于三态分类（真通过 < strict / 近似通过 strict..loose / 不一致 >= loose）。
    // oracle_near_solid = DC-14 非平凡性检查（oracle 帧退化/纯色 → 假绿可疑，排除出 credible pass）。
    let results: Vec<(String, bool, Option<f64>, f64, bool)> = parallel_map(&filtered, jobs, |case| {
        let _timer = CaseTimer::new(&case.id);
        log_mem_if_enabled(&case.id);
        let safe_id = case.id.replace(['/', '\\', '.'], "_");
        let oracle_path = oracle_dir.join(format!("{safe_id}.png"));
        let strict_thresh_pct = case.category.strict_max_diff_ratio() * 100.0;
        if !oracle_path.exists() {
            return (case.id.clone(), false, None, strict_thresh_pct, false);
        }
        let config = ReftestConfig {
            viewport_width: options.viewport_width as u32,
            viewport_height: options.viewport_height as u32,
            media_type: options.media_type,
            ..Default::default()
        };
        let test_fb = render_to_framebuffer_with_base(&case.test_html, "", &config, case.base_dir.as_deref());
        let oracle_fb = match load_png_to_framebuffer(&oracle_path.to_string_lossy()) {
            Ok(fb) => fb,
            Err(_) => return (case.id.clone(), false, None, strict_thresh_pct, false),
        };
        let near_solid = reftest::frame_is_near_solid(&oracle_fb);
        let (diff_px, _max_diff) = compare_pixels(&test_fb, &oracle_fb, 0);
        let w = test_fb.width.min(oracle_fb.width) as usize;
        let h = test_fb.height.min(oracle_fb.height) as usize;
        let total = (w * h).max(1) as f64;
        (
            case.id.clone(),
            true,
            Some(100.0 * diff_px as f64 / total),
            strict_thresh_pct,
            near_solid,
        )
    });

    let with_oracle: Vec<&(String, bool, Option<f64>, f64, bool)> = results.iter().filter(|r| r.1).collect();
    let no_oracle = results.len() - with_oracle.len();
    let loose_pct = pass_ratio * 100.0;
    let oracle_pass = with_oracle
        .iter()
        .filter(|r| r.2.is_some_and(|p| p < loose_pct))
        .count();
    // DC-14 三态分类：真通过（严格容差，唯一可信达标指标）/ 近似通过（strict..loose）/ 不一致（>=loose）。
    let mut strict_pass = 0usize;
    let mut near_pass = 0usize;
    for r in &with_oracle {
        if let Some(p) = r.2 {
            if p < r.3 {
                strict_pass += 1;
            } else if p < loose_pct {
                near_pass += 1;
            }
        }
    }
    // DC-14 非平凡性检查：oracle-pass 中 oracle 帧退化/纯色的（假绿可疑），排除出 credible pass。
    let degenerate_pass = with_oracle
        .iter()
        .filter(|r| r.4 && r.2.is_some_and(|p| p < loose_pct))
        .count();
    let credible_pass = oracle_pass.saturating_sub(degenerate_pass);
    let total = with_oracle.len();
    let rate = if total > 0 {
        100.0 * oracle_pass as f64 / total as f64
    } else {
        0.0
    };
    let credible_rate = if total > 0 {
        100.0 * credible_pass as f64 / total as f64
    } else {
        0.0
    };
    let strict_rate = if total > 0 {
        100.0 * strict_pass as f64 / total as f64
    } else {
        0.0
    };

    eprintln!("\n═══════════════════════════════════════════════");
    eprintln!("  DC-14 chromium-Oracle 真一致率（独立参考）");
    eprintln!("═══════════════════════════════════════════════");
    eprintln!("  cases scanned:      {}", results.len());
    eprintln!("  with chromium oracle: {}", total);
    eprintln!("  no oracle (skip):   {}", no_oracle);
    eprintln!(
        "  oracle-pass (z_vs_chr < {:.1}%): {} ({:.1}%)",
        loose_pct, oracle_pass, rate
    );
    eprintln!("  ── DC-14 非平凡性检查（排除退化/纯色假绿）──");
    eprintln!("  退化可疑 pass (oracle 帧近纯色): {} → 排除", degenerate_pass);
    eprintln!("  credible pass (排除退化): {} ({:.1}%)", credible_pass, credible_rate);
    eprintln!("  ── DC-14 三态分类（严格容差真通过率 = 唯一可信达标指标）──");
    eprintln!(
        "  真通过 (z_vs_chr < 布局0.1%/文字0.5%): {} ({:.1}%)",
        strict_pass, strict_rate
    );
    eprintln!(
        "  近似通过 (strict..{:.1}%): {} ({:.1}%)",
        loose_pct,
        near_pass,
        100.0 * near_pass as f64 / total.max(1) as f64
    );
    eprintln!("  不一致 (>= {:.1}%): {}", loose_pct, total - strict_pass - near_pass);
    eprintln!("  (cf. self-source ~56.5% / DC-14 46.5% false-pass)");

    // 列出 z_vs_chr 最大的 15 个（最不一致，候选修复目标）
    let mut sorted: Vec<&(String, bool, Option<f64>, f64, bool)> = with_oracle.clone();
    sorted.sort_by(|a, b| {
        b.2.unwrap_or(0.0)
            .partial_cmp(&a.2.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    eprintln!("\n  Top 15 worst z_vs_chr（修复候选）:");
    for (id, _, pct, _, _) in sorted.iter().take(15) {
        eprintln!("    {:.2}%  {}", pct.unwrap_or(0.0), id);
    }
    if std::env::var("ORACLE_DUMP_ALL").is_ok() {
        eprintln!("\n  ALL cases (sorted desc):");
        for (id, _, pct, _, _) in sorted.iter() {
            eprintln!("    ALL {:.2}%  {}", pct.unwrap_or(0.0), id);
        }
    }
    // DC-14 非平凡性：列出退化为纯色但被判 pass 的可疑 case（供单独审计）。
    let degenerate: Vec<&(String, bool, Option<f64>, f64, bool)> = with_oracle
        .iter()
        .filter(|r| r.4 && r.2.is_some_and(|p| p < loose_pct))
        .copied()
        .collect();
    if !degenerate.is_empty() {
        eprintln!(
            "\n  退化可疑 pass（oracle 帧近纯色，z_vs_chr<{:.1}%）— 供审计:",
            loose_pct
        );
        for (id, _, pct, _, _) in degenerate.iter().take(50) {
            eprintln!("    {:.2}%  {}", pct.unwrap_or(0.0), id);
        }
        if degenerate.len() > 50 {
            eprintln!("    ... 共 {} 个（仅显示前 50）", degenerate.len());
        }
    }

    // 按目录聚合
    use std::collections::BTreeMap;
    let mut by_dir: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    for (id, _has, pct, _, _) in &with_oracle {
        let dir = id
            .rsplit_once('/')
            .map(|(d, _)| d.to_string())
            .unwrap_or_else(|| id.clone());
        let entry = by_dir.entry(dir).or_insert((0, 0));
        entry.1 += 1;
        if pct.is_some_and(|p| p < pass_ratio * 100.0) {
            entry.0 += 1;
        }
    }
    eprintln!("\n  per-directory oracle pass-rate:");
    for (dir, (p, t)) in &by_dir {
        let r = if *t > 0 { 100.0 * *p as f64 / *t as f64 } else { 0.0 };
        eprintln!("    {}/{} ({:.0}%)  {}", p, t, r, dir);
    }
    let _ = no_oracle; // 已在报告中输出
}

/// `struct-sweep [filter]` 子命令（R1504）— DC-13 结构检查 corpus sweep。
///
/// 对上游 WPT reftest 的 test 页（经 script DOM 变更后的 mutated_html）渲染并跑
/// `check_sibling_overlaps`，报告有兄弟盒重叠的 case（按总重叠面积降序 top-N）。
/// 用途：把 product-smoke struct-check（R1489-R1503 在 welcome/wintertc/morning 找到
/// R1492/R1498 等真 bug）系统化扩展到 corpus，hunt 更多 R1492-class 真 layout bug。
///
/// 用法：zero-wpt-runner struct-sweep [filter]   （filter = case.id 子串，如 "normal-flow"）
fn cmd_struct_sweep(options: &CliOptions, filter: Option<&str>) {
    use reftest::ReftestConfig;
    let wpt_data_dir = match &options.wpt_data {
        Some(p) => std::path::PathBuf::from(p),
        None => std::path::PathBuf::from("tests/wpt-runner/wpt-data"),
    };
    if !wpt_data_dir.is_dir() {
        eprintln!("Error: wpt-data directory not found: {}", wpt_data_dir.display());
        std::process::exit(1);
    }
    let file_cases = wpt_file_loader::load_file_reftests(&wpt_data_dir);
    let filtered: Vec<&wpt_file_loader::FileReftestCase> = file_cases
        .iter()
        .filter(|c| filter.is_none_or(|f| c.id.contains(f)))
        .collect();
    eprintln!("struct-sweep: {} cases (filter={:?})", filtered.len(), filter);
    let jobs = effective_jobs(options);
    // (id, total_overlap_px², top_issue_string)
    let results: Vec<(String, f32, String)> = parallel_map(&filtered, jobs, |case| {
        let config = ReftestConfig {
            viewport_width: options.viewport_width as u32,
            viewport_height: options.viewport_height as u32,
            ..Default::default()
        };
        let (_fb, root, paint_skip, rendered_html) =
            reftest::render_to_framebuffer_with_layout_and_paint_skip_with_base(
                &case.test_html,
                "",
                &config,
                case.base_dir.as_deref(),
            );
        let labels = reftest::collect_dom_labels(&rendered_html);
        let issues = reftest::check_sibling_overlaps(&root, &labels, &paint_skip);
        if issues.is_empty() {
            (case.id.clone(), 0.0, String::new())
        } else {
            // 提取每个 issue 的面积（"overlap {N}px²"）求和，取最大 issue 作样本。
            let mut total = 0.0f32;
            let mut top = String::new();
            let mut top_a = 0.0f32;
            for s in &issues {
                let a = s
                    .split("overlap")
                    .nth(1)
                    .and_then(|t| t.split("px²").next())
                    .and_then(|t| t.trim().parse::<f32>().ok())
                    .unwrap_or(0.0);
                total += a;
                if a > top_a {
                    top_a = a;
                    top = s.clone();
                }
            }
            (case.id.clone(), total, top)
        }
    });
    let mut flagged: Vec<&(String, f32, String)> = results.iter().filter(|r| r.1 > 0.0).collect();
    flagged.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    let total_cases = results.len();
    let clean = total_cases - flagged.len();
    eprintln!(
        "\n═══════════════════════════════════════════════\n  struct-sweep: {} cases, {} clean, {} with sibling-overlap (top {} shown)\n═══════════════════════════════════════════════",
        total_cases,
        clean,
        flagged.len(),
        flagged.len().min(30)
    );
    for (id, total, top) in flagged.iter().take(30) {
        eprintln!("  {:.0}px²  {}  | {}", total, id, top);
    }

    // R1575：collapsed-container 诊断（非 gating）—— 真实元素盒有显著高度子内容但自身
    // 高度近 0（layout grow 失败，如 inline>inline-block 的 `<p>` 塌缩）。独立于
    // sibling-overlap 报告，定位 IFC/inline-box-model 类 layout gap。
    let collapsed: Vec<(String, Vec<String>)> = filtered
        .iter()
        .filter_map(|case| {
            let config = ReftestConfig {
                viewport_width: options.viewport_width as u32,
                viewport_height: options.viewport_height as u32,
                ..Default::default()
            };
            let (_fb, root, rendered_html) = reftest::render_to_framebuffer_with_layout_with_base(
                &case.test_html,
                "",
                &config,
                case.base_dir.as_deref(),
            );
            let labels = reftest::collect_dom_labels(&rendered_html);
            let issues = reftest::check_collapsed_containers(&root, &labels);
            if issues.is_empty() {
                None
            } else {
                Some((case.id.clone(), issues))
            }
        })
        .collect();
    eprintln!(
        "\n  collapsed-container: {} cases with塌缩 containers (top 30 shown)",
        collapsed.len()
    );
    for (id, issues) in collapsed.iter().take(30) {
        eprintln!("  {}  | {}", id, issues.first().unwrap_or(&String::new()));
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
                report.push_str(&format!("    {}\n", r.message));
                // D2 亚像素统计：通道差恰好为 1 的像素占比（诊断维度，不参与判定）
                report.push_str(&format!(
                    "    (subpixel={}/{} — 亚像素级差异占比 {:.1}%，见 f32-layout-precision-audit)\n\n",
                    r.subpixel_diff_pixels,
                    r.diff_pixels,
                    if r.diff_pixels > 0 {
                        r.subpixel_diff_pixels as f64 / r.diff_pixels as f64 * 100.0
                    } else {
                        0.0
                    }
                ));
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
        json.push_str(&format!("      \"max_channel_diff\": {},\n", r.max_channel_diff));
        json.push_str(&format!("      \"subpixel_diff_pixels\": {}", r.subpixel_diff_pixels));
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

fn run_wpt_cases(tests: &[runner::TestCase], ctx: &TestContext, jobs: usize) -> Vec<report::TestResult> {
    if jobs <= 1 {
        return runner::run_all(tests, ctx);
    }

    let expectations = runner::TestExpectations::new();
    parallel_map(tests, jobs, |case| {
        runner::run_single_with_expectations(case, ctx, &expectations)
    })
}

/// 诊断（env `REFTEST_MEM_LOG=1` 启用）：打印当前进程 VmRSS / VmHWM，用于定位
/// 跨 reftest 用例的内存累积（区分真 leak vs glibc allocator RSS retention）。
/// 零开销：env 关时仅一次 `var` 读 + 即时返回。仅 Linux（读 /proc/self/status）。
fn log_mem_if_enabled(label: &str) {
    if std::env::var("REFTEST_MEM_LOG").ok().as_deref() != Some("1") {
        return;
    }
    let mut rss = String::from('?');
    let mut hwm = String::from('?');
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if let Some(rest) = line.strip_prefix("VmRSS:") {
                rss = rest.trim().to_string();
            } else if let Some(rest) = line.strip_prefix("VmHWM:") {
                hwm = rest.trim().to_string();
            }
        }
    }
    eprintln!("[mem] {label} | VmRSS={rss} | VmHWM={hwm}");
}

/// 诊断（env `REFTEST_TIME_LOG=1` 启用）：RAII 守卫，per-case closure 退出时
/// （任意 return 路径）打印该 case 耗时，用于定位 CPU-straggler 慢案。零开销：
/// env 关时仅一次 `var` 读 + 一次 `Instant::now`。
struct CaseTimer {
    id: String,
    start: std::time::Instant,
    enabled: bool,
}

impl CaseTimer {
    fn new(id: &str) -> Self {
        let enabled = std::env::var("REFTEST_TIME_LOG").ok().as_deref() == Some("1");
        Self {
            id: id.to_string(),
            start: std::time::Instant::now(),
            enabled,
        }
    }
}

impl Drop for CaseTimer {
    fn drop(&mut self) {
        if self.enabled {
            eprintln!("[time] {} | {:.3}s", self.id, self.start.elapsed().as_secs_f64());
        }
    }
}

fn parallel_map<T, R, F>(items: &[T], jobs: usize, f: F) -> Vec<R>
where
    T: Sync,
    R: Send,
    F: Fn(&T) -> R + Sync + Send,
{
    if jobs <= 1 || items.len() <= 1 {
        return items.iter().map(f).collect();
    }

    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(jobs)
        .build()
        .expect("failed to build WPT runner thread pool");

    pool.install(|| items.par_iter().map(f).collect())
}

fn effective_jobs(options: &CliOptions) -> usize {
    // 不再对 --gpu 强制 jobs=1：reftest 的 GPU 路径目前是 CPU 回退 stub（见
    // reftest.rs `render_to_framebuffer_gpu_with_base`），并不真正使用 GpuRenderer，
    // 故走与 CPU 相同的默认并行度。待真正的 GPU reftest（接入 GpuRenderer + 全图元
    // + 图片加载 + device 复用）落地后，再按 GPU_CREATE_MUTEX 的约束重新评估默认并行度。
    options.jobs.unwrap_or_else(default_parallel_jobs)
}

/// reftest 默认并行度硬上限。实测（16 核机）并行扩展在 ~8 线程即饱和：
/// jobs=1→1.0×、jobs=8→5.5×、jobs=15→5.8×——瓶颈是 CPU 软光栅的内存带宽，
/// 再加线程几乎不提速（15→8 仅慢 5%），却多耗近一倍内存，逼近 test-guard
/// 全树 16GB 上限（每个 job 持有独立 RenderPipeline + 字体/图像缓存 + 帧缓冲）。
/// 故默认封顶 8：速度持平、内存压力减半。用户可 --jobs 显式覆盖。
const DEFAULT_MAX_JOBS: usize = 8;

fn default_parallel_jobs() -> usize {
    std::thread::available_parallelism()
        .map(|jobs| jobs.get())
        .unwrap_or(1)
        .saturating_sub(1)
        .clamp(1, DEFAULT_MAX_JOBS)
}

fn print_usage() {
    print!("{USAGE}");
}
