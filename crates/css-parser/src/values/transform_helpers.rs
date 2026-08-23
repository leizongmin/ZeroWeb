use super::{CalcExpr, eval_calc, parse_math_function};

pub(super) fn parse_finite_number_math(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if let Ok(number) = trimmed.parse::<f64>() {
        return number.is_finite().then_some(number);
    }
    if contains_dimension_token(trimmed) {
        return None;
    }
    let expr = parse_math_function(trimmed)?;
    if !calc_expr_is_number(&expr) {
        return None;
    }
    let value = eval_calc(&expr, None)?;
    value.is_finite().then_some(value)
}

fn calc_expr_is_number(expr: &CalcExpr) -> bool {
    match expr {
        CalcExpr::Number(_) => true,
        CalcExpr::Length(_) => false,
        CalcExpr::BinaryOp(left, _, right) => calc_expr_is_number(left) && calc_expr_is_number(right),
        CalcExpr::Min(args) | CalcExpr::Max(args) => args.iter().all(calc_expr_is_number),
        CalcExpr::Clamp { min, val, max } => {
            calc_expr_is_number(min) && calc_expr_is_number(val) && calc_expr_is_number(max)
        }
        CalcExpr::UnaryOp(_, inner) => calc_expr_is_number(inner),
        CalcExpr::BinaryMathOp(_, left, right) => calc_expr_is_number(left) && calc_expr_is_number(right),
    }
}

fn contains_dimension_token(input: &str) -> bool {
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let number_start = bytes[i].is_ascii_digit()
            || (bytes[i] == b'.' && bytes.get(i + 1).is_some_and(u8::is_ascii_digit))
            || ((bytes[i] == b'+' || bytes[i] == b'-')
                && bytes.get(i + 1).is_some_and(|b| b.is_ascii_digit() || *b == b'.'));
        if !number_start {
            i += 1;
            continue;
        }

        if bytes[i] == b'+' || bytes[i] == b'-' {
            i += 1;
        }
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if bytes.get(i) == Some(&b'.') {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
        }
        if bytes.get(i).is_some_and(|b| *b == b'e' || *b == b'E')
            && bytes
                .get(i + 1)
                .is_some_and(|b| b.is_ascii_digit() || *b == b'+' || *b == b'-')
        {
            i += 1;
            if bytes.get(i).is_some_and(|b| *b == b'+' || *b == b'-') {
                i += 1;
            }
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
        }

        if bytes.get(i).is_some_and(|b| *b == b'%' || b.is_ascii_alphabetic()) {
            return true;
        }
    }
    false
}

pub(super) fn split_top_level_comma_args(input: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;

    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            ',' if depth == 0 => {
                let part = input[start..idx].trim();
                if part.is_empty() {
                    return None;
                }
                args.push(part);
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }

    if depth != 0 {
        return None;
    }
    let last = input[start..].trim();
    if last.is_empty() {
        return None;
    }
    args.push(last);
    Some(args)
}

pub(super) fn split_transform_args(args: &str) -> Option<Vec<&str>> {
    let comma_args = split_top_level_comma_args(args)?;
    if comma_args.len() > 1 {
        let mut parts = Vec::new();
        for arg in comma_args {
            parts.extend(split_top_level_whitespace_args(arg)?);
        }
        return Some(parts);
    }
    split_top_level_whitespace_args(args)
}

fn split_top_level_whitespace_args(input: &str) -> Option<Vec<&str>> {
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = None;

    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => {
                depth += 1;
                start.get_or_insert(idx);
            }
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            }
            ch if ch.is_whitespace() && depth == 0 => {
                if let Some(begin) = start.take() {
                    let part = input[begin..idx].trim();
                    if !part.is_empty() {
                        args.push(part);
                    }
                }
            }
            _ => {
                start.get_or_insert(idx);
            }
        }
    }

    if depth != 0 {
        return None;
    }
    if let Some(begin) = start {
        let part = input[begin..].trim();
        if !part.is_empty() {
            args.push(part);
        }
    }
    if args.is_empty() { None } else { Some(args) }
}
