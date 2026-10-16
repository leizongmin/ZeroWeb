//! `docs/compat/trends/wpt-suites.csv` 列数与类型防护校验（GB-20261014 ① 用户批复落地）。
//!
//! 该文件是 website/metrics.json 的唯一 suites 数据源，2026-10 多次在 Pages deploy 期才
//! 暴露列错位/类型错误。本模块把生成器 `scripts/generate-website-metrics.py`
//! `build_suites` 的校验契约镜像为常驻单测，把拦截点前移到 CI 测试面；生成器解析规则
//! 变化时须同步更新本模块（FIXME: 契约为手工镜像，无共享实现）。

use std::path::Path;

/// 列数，与生成器 `SUITES_COLUMNS` 一致。
const COLUMNS: usize = 15;
const STATUSES: [&str; 3] = ["active", "completed", "planned"];
const HARNESSES: [&str; 3] = ["reftest", "testharness", "mixed"];
const UNITS: [&str; 3] = ["tests", "subtests", "files"];

/// 校验全部数据行的列数与类型；错误信息带数据行号，口径与生成器一致。
fn check_suites_csv(content: &str, source: &str) -> Result<(), String> {
    let mut data_row = 0;
    for line in content.lines() {
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        data_row += 1;
        let label = format!("{source}: 数据行 {data_row}");
        let fields = split_fields(line).map_err(|message| format!("{label}: {message}"))?;
        if fields.len() != COLUMNS {
            return Err(format!(
                "{label}: 期望 {COLUMNS} 列，得 {}（字段内出现半角逗号会错位，见文件头注释第 4 条）",
                fields.len()
            ));
        }
        check_row(&fields, &label)?;
    }
    Ok(())
}

/// 引号感知字段切分，语义对齐 `csv.reader` 单行记录：字段以 `"` 开头时按引号字段解析
/// （`""` 为转义引号、内部逗号不切分），行尾未闭合引号报错（跨行记录不受支持）。
fn split_fields(line: &str) -> Result<Vec<String>, String> {
    let mut chars = line.chars().peekable();
    let mut fields = Vec::new();
    loop {
        let mut field = String::new();
        if chars.peek() == Some(&'"') {
            chars.next();
            loop {
                match chars.next() {
                    Some('"') if chars.peek() == Some(&'"') => {
                        chars.next();
                        field.push('"');
                    }
                    Some('"') => break,
                    Some(ch) => field.push(ch),
                    None => return Err("引号字段未闭合（不支持跨行记录，疑为 csv 模块整体回写污染）".into()),
                }
            }
        }
        // 引号闭合后到分隔符前的残余字符并入字段（对齐 csv 模块非严格行为）。
        while let Some(&ch) = chars.peek() {
            if ch == ',' {
                break;
            }
            field.push(ch);
            chars.next();
        }
        fields.push(field);
        if chars.peek().is_none() {
            return Ok(fields);
        }
        chars.next(); // 吃掉字段分隔符 `,`
    }
}

fn check_row(fields: &[String], label: &str) -> Result<(), String> {
    // 列序见文件头注释：date, suite_id, name_en, name_zh, wpt_dirs, harness, unit,
    // total, passed, rate, status, goal, evidence, note_en, note_zh。
    let date = fields[0].as_str();
    let harness = fields[5].as_str();
    let unit = fields[6].as_str();
    let rate = fields[9].as_str();
    let status = fields[10].as_str();
    if !date.is_empty() && !valid_date(date) {
        return Err(format!("{label}: date 期望 YYYY-MM-DD，得 {date:?}"));
    }
    if !STATUSES.contains(&status) {
        return Err(format!("{label}: status 非法 {status:?}"));
    }
    if !HARNESSES.contains(&harness) {
        return Err(format!("{label}: harness 非法 {harness:?}"));
    }
    if !UNITS.contains(&unit) {
        return Err(format!("{label}: unit 非法 {unit:?}"));
    }
    let total = parse_int_field(&fields[7], label, "total")?;
    let passed = parse_int_field(&fields[8], label, "passed")?;
    let _goal = parse_int_field(&fields[11], label, "goal")?;
    if status == "planned" {
        if total.is_some() || passed.is_some() {
            return Err(format!("{label}: planned 行 total/passed 必须留空"));
        }
        return Ok(());
    }
    let (Some(total), Some(passed)) = (total, passed) else {
        return Err(format!("{label}: {status} 行 total/passed 必填"));
    };
    if date.is_empty() {
        return Err(format!("{label}: {status} 行 date 必填"));
    }
    if passed > total {
        return Err(format!("{label}: passed {passed} > total {total}"));
    }
    check_rate(rate, total, passed, label)
}

/// date 空串合法（planned 行）；否则须为 `YYYY-MM-DD` 形态（生成器同样不做日历有效性校验）。
fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
}

/// 空串 → `None`；其余须为纯数字。比生成器 `int()` 更严（拒绝 `+`/空白/负号），
/// 现存数据均为普通十进制非负数。
fn parse_int_field(value: &str, label: &str, column: &str) -> Result<Option<u64>, String> {
    if value.is_empty() {
        return Ok(None);
    }
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(format!("{label}: {column} 期望整数，得 {value:?}"));
    }
    let parsed = value.parse().expect("纯数字必可解析为 u64");
    Ok(Some(parsed))
}

/// rate 可留空（生成器重算）；填写时须等于 round(passed/total×100, 1)，容差 0.05
/// （`round_ties_even` 对齐 python `round` 的银行家舍入，边界留 1e-6 余量）。
fn check_rate(rate: &str, total: u64, passed: u64, label: &str) -> Result<(), String> {
    if rate.is_empty() {
        return Ok(());
    }
    let value: f64 = rate
        .parse()
        .map_err(|_| format!("{label}: rate 期望数值，得 {rate:?}"))?;
    let expected = (passed as f64 / total as f64 * 100.0 * 10.0).round_ties_even() / 10.0;
    if (value - expected).abs() > 0.050_001 {
        return Err(format!("{label}: rate {rate} 与 passed/total 重算值 {expected} 不一致"));
    }
    Ok(())
}

const VALID_ROW: &str =
    "2026-10-15,demo-compat,Demo,演示,demo,testharness,subtests,10,8,,active,,docs/e.md,note en,note zh";

#[test]
fn checked_in_suites_csv_passes_generator_contract() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/compat/trends/wpt-suites.csv");
    let content = std::fs::read_to_string(&path).expect("suites csv 在仓库内");
    check_suites_csv(&content, "docs/compat/trends/wpt-suites.csv").expect("列数与类型须过生成器契约");
}

#[test]
fn quoted_note_fields_keep_column_integrity() {
    let row =
        "2026-10-15,demo-compat,Demo,演示,demo,testharness,subtests,10,8,,active,,docs/e.md,\"a, b\",\"注，备注\"";
    check_suites_csv(row, "<memory>").expect("引号字段（内含逗号）为现存合法形态");
}

#[test]
fn comma_drift_is_rejected() {
    let row = "2026-10-15,demo-compat,Demo,演,示,demo,testharness,subtests,10,8,,active,,docs/e.md,a,b";
    let error = check_suites_csv(row, "<memory>").unwrap_err();
    assert!(error.contains("期望 15 列，得 16"), "{error}");
}

#[test]
fn quoted_comment_line_is_rejected() {
    let content = format!("\"# 维护约定\"\n{VALID_ROW}");
    let error = check_suites_csv(&content, "<memory>").unwrap_err();
    assert!(error.contains("期望 15 列，得 1"), "{error}");
}

#[test]
fn unclosed_quote_is_rejected() {
    let row = "2026-10-15,demo-compat,Demo,演示,demo,testharness,subtests,10,8,,active,,docs/e.md,\"未闭合,备注";
    let error = check_suites_csv(row, "<memory>").unwrap_err();
    assert!(error.contains("未闭合"), "{error}");
}

#[test]
fn non_integer_goal_is_rejected() {
    let row = "2026-10-13,demo-compat,Demo,演示,demo,testharness,subtests,10,8,,active,2026-10-13,docs/e.md,,";
    let error = check_suites_csv(row, "<memory>").unwrap_err();
    assert!(error.contains("goal 期望整数"), "{error}");
}

#[test]
fn planned_row_must_leave_counts_empty() {
    let row = ",demo-compat,Demo,演示,demo,testharness,subtests,10,,,planned,,docs/e.md,,";
    let error = check_suites_csv(row, "<memory>").unwrap_err();
    assert!(error.contains("planned 行 total/passed 必须留空"), "{error}");
}

#[test]
fn comment_and_blank_lines_are_skipped() {
    let content = format!("# 注释行\n\n{VALID_ROW}\n");
    check_suites_csv(&content, "<memory>").expect("注释与空行不占数据行");
}
