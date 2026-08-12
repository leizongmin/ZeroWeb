//! Deterministic retained form-input performance report.

use std::time::Instant;

use serde_json::json;
use zero_engine::{DomMutation, RenderPipeline};
use zero_page_runtime::{FrameInvalidation, FrameTransaction};

const WARMUP_ITERATIONS: usize = 20;
const MEASURED_ITERATIONS: usize = 200;
const JANK_BUDGET_MS: f64 = 20.0;

fn percentile(sorted: &[f64], percentile: f64) -> f64 {
    let index = (percentile * (sorted.len() - 1) as f64).floor() as usize;
    sorted[index]
}

fn main() {
    let mut pipeline = RenderPipeline::new(800.0, 600.0);
    let _ = pipeline.render_html(
        r#"<html><body><input id="name" style="width:240px" value=""><textarea id="note"></textarea></body></html>"#,
        "",
    );

    let mut samples_ms = Vec::with_capacity(MEASURED_ITERATIONS);
    let mut max_parse_count = 0;
    let mut max_style_count = 0;
    let mut max_layout_count = 0;
    let mut max_paint_count = 0;
    let mut max_publish_count = 0;

    for iteration in 0..(WARMUP_ITERATIONS + MEASURED_ITERATIONS) {
        let mutation = DomMutation::SetFormValue {
            selector: "#name".to_string(),
            value: format!("ZeroWeb retained input {iteration} 中文"),
        };
        let started = Instant::now();
        let (result, snapshot, _) = pipeline
            .render_with_dom_mutations(std::slice::from_ref(&mutation), "")
            .expect("form value mutation must apply");
        assert!(snapshot.is_none(), "IDL value edits must not serialize the document");

        let mut transaction = FrameTransaction::default();
        transaction.begin();
        transaction.invalidate(FrameInvalidation::NEEDS_PAINT);
        let publish_count = usize::from(transaction.finish().is_some());
        let elapsed_ms = started.elapsed().as_secs_f64() * 1_000.0;

        if iteration >= WARMUP_ITERATIONS {
            samples_ms.push(elapsed_ms);
            max_parse_count = max_parse_count.max(result.timings.parse_count);
            max_style_count = max_style_count.max(result.timings.style_count);
            max_layout_count = max_layout_count.max(result.timings.layout_count);
            max_paint_count = max_paint_count.max(result.timings.paint_count);
            max_publish_count = max_publish_count.max(publish_count);
        }
    }

    samples_ms.sort_by(f64::total_cmp);
    let jank_samples = samples_ms.iter().filter(|sample| **sample > JANK_BUDGET_MS).count();
    let report = json!({
        "schema_version": 1,
        "scenario": "retained_fixed_size_input_value",
        "platform_class": format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH),
        "profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "iterations": samples_ms.len(),
        "input_to_publish_ms": {
            "p50": percentile(&samples_ms, 0.50),
            "p95": percentile(&samples_ms, 0.95),
            "max": samples_ms[samples_ms.len() - 1]
        },
        "jank_20ms_ratio": jank_samples as f64 / samples_ms.len() as f64,
        "max_counts_per_input": {
            "parse": max_parse_count,
            "style": max_style_count,
            "full_layout": max_layout_count,
            "paint": max_paint_count,
            "publish": max_publish_count
        }
    });
    println!("{}", serde_json::to_string_pretty(&report).expect("serialize report"));
}
