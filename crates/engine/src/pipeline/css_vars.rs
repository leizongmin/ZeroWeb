//! CSS Custom Properties (CSS Variables Level 1) 最小文本级实现。
//!
//! bing-t5（site-optimizer bing-20261010-r1 簇 C3+C4 根因，2026-10-11）：
//! 引擎此前不 resolve `var(--name)` → bing 整个主题系统（壁纸/图标色/字体族）
//! 走 guaranteed-invalid → 暗色回退 + 豆腐块。本模块在 CSS 解析前做文本级
//! 自定义属性声明收集与 `var()` 替换，覆盖静态 CSS（`<style>` 块 + 外链 CSS
//! inline 化后合并文本）中最常见模式。
//!
//! 已知限制（文本级方法固有）：不支持 per-element 继承、不支持运行时动态变更、
//! 不支持 `@property` 注册。对静态站（bing.com 等）足够恢复正确渲染。

/// 收集 CSS 文本中所有自定义属性声明（`--name: value;`），返回有序去重的
/// name → value 映射。后面出现的同名声明覆盖先前的（cascade 简化模型）。
pub fn collect_custom_properties(css: &str) -> Vec<(String, String)> {
    let mut map: Vec<(String, String)> = Vec::new();
    let bytes = css.as_bytes();
    let len = bytes.len();
    let mut i = 0;
    let mut depth: i32 = 0; // {} 嵌套深度

    while i < len {
        let c = bytes[i];
        match c {
            b'/' if i + 1 < len && bytes[i + 1] == b'*' => {
                // 跳过块注释
                i += 2;
                while i + 1 < len && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                    i += 1;
                }
                i += 2;
            }
            b'/' if i + 1 < len && bytes[i + 1] == b'/' => {
                while i < len && bytes[i] != b'\n' {
                    i += 1;
                }
            }
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
            }
            b'-' if i + 1 < len && bytes[i + 1] == b'-' && depth >= 1 => {
                // 可能是自定义属性声明 `--name: value`
                let start = i;
                let mut j = i + 2;
                // 读自定义属性名（直到 ':'）
                while j < len && bytes[j] != b':' && bytes[j] != b'{' && bytes[j] != b'}' && bytes[j] != b';' {
                    j += 1;
                }
                if j < len && bytes[j] == b':' {
                    let name = css[start..j].trim();
                    if is_valid_custom_prop_name(name) {
                        // 读值（直到 ';' 或匹配的 '}'）
                        let val_start = j + 1;
                        let mut k = val_start;
                        let mut local_depth = 0;
                        while k < len {
                            match bytes[k] {
                                b'(' | b'[' => local_depth += 1,
                                b')' | b']' => local_depth -= 1,
                                b';' if local_depth <= 0 => break,
                                b'}' if local_depth <= 0 => break,
                                _ => {}
                            }
                            k += 1;
                        }
                        let value = css[val_start..k].trim().to_string();
                        if !value.is_empty() {
                            // 覆盖同名（cascade 简化）
                            let name_s = name.to_string();
                            if let Some(entry) = map.iter_mut().find(|(n, _)| *n == name_s) {
                                entry.1 = value;
                            } else {
                                map.push((name_s, value));
                            }
                        }
                        i = if k < len && bytes[k] == b';' { k + 1 } else { k };
                        continue;
                    }
                }
                i = j;
            }
            _ => {
                i += 1;
            }
        }
    }
    map
}

/// 替换 CSS 文本中所有 `var(--name)` 和 `var(--name, fallback)` 引用。
/// 迭代解析嵌套 `var(var(--a), var(--b))`（最多 8 层，防止循环）。
pub fn substitute_vars(css: &str, props: &[(String, String)]) -> String {
    let mut result = css.to_string();
    for _ in 0..8 {
        let substituted = substitute_vars_once(&result, props);
        if substituted == result {
            break;
        }
        result = substituted;
    }
    result
}

/// 单层 var() 替换。
fn substitute_vars_once(css: &str, props: &[(String, String)]) -> String {
    let bytes = css.as_bytes();
    let len = bytes.len();
    let mut out = String::with_capacity(len);
    let mut i = 0;

    while i < len {
        // 查找 "var(" 子串
        if i + 4 <= len && &bytes[i..i + 4] == b"var(" {
            // 找匹配的 ')'（考虑嵌套括号）
            let open = i + 3; // '(' 位置
            let mut depth = 0;
            let mut j = open;
            let mut found_close = false;
            while j < len {
                match bytes[j] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            found_close = true;
                            break;
                        }
                    }
                    _ => {}
                }
                j += 1;
            }
            if found_close {
                let inner = &css[open + 1..j];
                if let Some((name, fallback)) = parse_var_ref(inner) {
                    if let Some((_, value)) = props.iter().find(|(n, _)| *n == name) {
                        out.push_str(value);
                        i = j + 1;
                        continue;
                    }
                    if let Some(fb) = fallback {
                        out.push_str(fb.trim());
                        i = j + 1;
                        continue;
                    }
                }
                // 无法解析的 var() —— 输出原样（保留让下游报 invalid）
                out.push_str(&css[i..j + 1]);
                i = j + 1;
                continue;
            }
        }
        // 跳过字符串字面量
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            let q = bytes[i];
            out.push(bytes[i] as char);
            i += 1;
            while i < len {
                if bytes[i] == b'\\' && i + 1 < len {
                    out.push(bytes[i] as char);
                    out.push(bytes[i + 1] as char);
                    i += 2;
                    continue;
                }
                if bytes[i] == q {
                    out.push(bytes[i] as char);
                    i += 1;
                    break;
                }
                out.push(bytes[i] as char);
                i += 1;
            }
            continue;
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// 解析 var() 内部参数为 (name, Option<fallback>)。
fn parse_var_ref(inner: &str) -> Option<(String, Option<String>)> {
    let inner = inner.trim();
    if !inner.starts_with("--") {
        return None;
    }
    match inner.find(',') {
        Some(pos) => {
            let name = inner[..pos].trim().to_string();
            let fallback = inner[pos + 1..].to_string();
            Some((name, Some(fallback)))
        }
        None => Some((inner.to_string(), None)),
    }
}

fn is_valid_custom_prop_name(name: &str) -> bool {
    name.starts_with("--") && name.len() > 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_and_substitute_basic() {
        let css = ":root { --mpm: 0 0 0 1px rgba(0,0,0,.05); --clv: 0 0 0 1px rgba(0,0,0,.08); }\n.box { margin: var(--mpm); border: var(--clv); }";
        let props = collect_custom_properties(css);
        assert!(props.iter().any(|(n, v)| n == "--mpm" && v.contains("rgba(0,0,0,.05)")));
        let out = substitute_vars(css, &props);
        assert!(out.contains("margin: 0 0 0 1px rgba(0,0,0,.05)"));
        assert!(!out.contains("var(--mpm)"));
    }

    #[test]
    fn substitute_fallback_when_missing() {
        let css = ".a { color: var(--nonexistent, red); }";
        let props = collect_custom_properties(css);
        let out = substitute_vars(css, &props);
        assert!(out.contains("color: red"), "got: {out}");
    }

    #[test]
    fn substitute_nested_var() {
        let css = ":root { --a: blue; --b: var(--a); }\n.x { color: var(--b); }";
        let props = collect_custom_properties(css);
        let out = substitute_vars(css, &props);
        assert!(out.contains("color: blue"), "got: {out}");
    }

    #[test]
    fn font_family_var_bing_pattern() {
        let css =
            "html { --lfd: \"Segoe UI\", -apple-system, BlinkMacSystemFont, sans-serif; }\nbody { font: var(--lfd); }";
        let props = collect_custom_properties(css);
        let out = substitute_vars(css, &props);
        assert!(out.contains("font: \"Segoe UI\""), "got: {out}");
    }

    #[test]
    fn bing_cda_pattern() {
        let css = r#":root { --cda: rgba(0,0,0,.7); } body { background: rgba(34,34,34,0.902); color: var(--cda); }"#;
        let props = collect_custom_properties(css);
        let out = substitute_vars(css, &props);
        assert!(out.contains("color: rgba(0,0,0,.7)"), "got: {out}");
    }

    #[test]
    fn unresolvable_var_kept() {
        let css = ".a { color: var(--unknown); }";
        let props = vec![];
        let out = substitute_vars(css, &props);
        assert!(out.contains("var(--unknown)"), "unresolvable var 应保留");
    }

    #[test]
    fn bing_real_world_var_heavy() {
        let css = concat!(
            "html{--mpm:0 0 0 1px rgba(0,0,0,.05),0 2px 4px 1px rgba(0,0,0,.14);",
            "--clv:0 0 0 1px rgba(0,0,0,.08),0 2px 4px 1px rgba(0,0,0,.16);",
            "--cch:0 0 0 1px rgba(0,0,0,.05),0 2px 4px 1px rgba(0,0,0,.18);",
            "--lfd:arial,'Segoe UI',sans-serif}",
            "html{--lqb:16px}",
            "body{font:var(--lfd);box-shadow:var(--mpm);border:var(--clv);font-size:var(--lqb,14px)}"
        );
        let props = collect_custom_properties(css);
        let out = substitute_vars(css, &props);
        assert!(out.contains("font:arial"), "font var 替换");
        assert!(out.contains("box-shadow:0 0 0 1px"), "mpm 替换");
        assert!(out.contains("font-size:16px"), "lqb fallback 替换");
        assert!(!out.contains("var(--"), "无残余 var()");
    }
}
