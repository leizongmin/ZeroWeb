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
/// name → value 映射。同名合并按级联语义：`!important` 压过 normal，
/// 同重要性时后面的声明覆盖前面的（css-variables-1 §2 —— 自定义属性走
/// 普通 cascade，`!important` 是声明级标记、不属于值）。
pub fn collect_custom_properties(css: &str) -> Vec<(String, String)> {
    // (name, value, important)——important 参与同名合并裁决，返回时剥掉。
    let mut map: Vec<(String, String, bool)> = Vec::new();
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
                        let raw = css[val_start..k].trim();
                        let (value, important) = split_important(raw);
                        if !value.is_empty() {
                            // 同名合并：important 压过 normal；同级后者覆盖前者
                            let name_s = name.to_string();
                            if let Some(entry) = map.iter_mut().find(|(n, _, _)| *n == name_s) {
                                if important || !entry.2 {
                                    entry.1 = value.to_string();
                                    entry.2 = important;
                                }
                            } else {
                                map.push((name_s, value.to_string(), important));
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
    map.into_iter().map(|(n, v, _)| (n, v)).collect()
}

/// 剥离声明值尾部的 `!important` 级联标记，返回 `(纯值, 是否 important)`。
/// 按 CSS Syntax 声明解析：`!` 与 `important` 间空白任意，匹配大小写不敏感；
/// 仅在值末尾识别，出现在值中间的 `!important` 字样按普通值保留。
fn split_important(value: &str) -> (&str, bool) {
    let t = value.trim();
    if t.len() >= 10
        && t[t.len() - 9..].eq_ignore_ascii_case("important")
        && let Some(head) = t[..t.len() - 9].trim_end().strip_suffix('!')
    {
        return (head.trim_end(), true);
    }
    (t, false)
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
/// 单层 var() 替换。用 str::find + &str 切片保证 UTF-8 安全。
fn substitute_vars_once(css: &str, props: &[(String, String)]) -> String {
    let mut out = String::with_capacity(css.len());
    let mut search_from = 0;

    while search_from < css.len() {
        match css[search_from..].find("var(") {
            Some(rel) => {
                let var_start = search_from + rel;
                out.push_str(&css[search_from..var_start]);

                let open = var_start + 3;
                let bytes = css.as_bytes();
                let mut depth = 0usize;
                let mut j = open;
                let mut found = false;
                while j < bytes.len() {
                    match bytes[j] {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                found = true;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                if !found {
                    out.push_str(&css[var_start..]);
                    break;
                }

                let inner = &css[open + 1..j];
                if let Some((name, fallback)) = parse_var_ref(inner) {
                    if let Some((_, val)) = props.iter().find(|(n, _)| *n == name) {
                        out.push_str(val);
                    } else if let Some(fb) = fallback {
                        out.push_str(fb.trim());
                    } else {
                        out.push_str(&css[var_start..=j]);
                    }
                } else {
                    out.push_str(&css[var_start..=j]);
                }
                search_from = j + 1;
            }
            None => {
                out.push_str(&css[search_from..]);
                break;
            }
        }
    }
    out
}
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
    fn chinese_text_not_mojibaked() {
        // bing-t5 审查 F1 实证：bytes[i] as char 逐字节 push 导致中文变 mojibake。
        // 修正后 var( 之间的中文文本（导航标签、字体名）须原样保留。
        let css = "body { font-family: var(--lfd); color: var(--cda); }\n/* 导航栏 */\n.nav { content: \"图片\"; font-family: 微软雅黑, sans-serif; }";
        let props = collect_custom_properties(css);
        let out = substitute_vars(css, &props);
        assert!(out.contains("微软雅黑"), "中文字体名须原样保留: {out}");
        assert!(out.contains("导航栏"), "中文注释须原样保留: {out}");
        assert!(out.contains("图片"), "中文字符串须原样保留: {out}");
        assert!(!out.contains("\u{c3}"), "不得出现 UTF-8 mojibake 特征码点");
    }

    #[test]
    fn url_with_double_slash_not_eaten() {
        // bing-t5 审查 F2：url(https://...) 中 // 不应被当注释吃掉后续声明
        let css = ".a { background: url(\"https://example.com/img.png\"); --bg: blue; }\n.b { color: var(--bg); }";
        let props = collect_custom_properties(css);
        assert!(
            props.iter().any(|(n, _)| n == "--bg"),
            "url 后的 --bg 声明须可见: {props:?}"
        );
    }

    #[test]
    fn bing_end_to_end_var_resolution() {
        // bing-t5（C3+C4）：模拟 bing 真实 CSS 变量使用模式——
        // :root 定义变量 → 其他规则引用 var(--name) → background-image 引用
        let css = concat!(
            "html{--mpm:0 0 0 1px rgba(0,0,0,.05),0 2px 4px 1px rgba(0,0,0,.14);",
            "--lfd:\"Segoe UI\",arial,sans-serif;",
            "--cda:rgba(0,0,0,.7);",
            "--bby:normal 400 18px/28px var(--lfd);}",
            "body{background:rgba(34,34,34,.902);color:var(--cda);font:var(--bby);",
            "background-image:url(data:image/svg+xml,%3Csvg%3E);}",
            ".nav{font:var(--dxf,var(--bby));box-shadow:var(--mpm)}"
        );
        let props = collect_custom_properties(css);
        // 变量数（不含嵌套 var(--lfd) 内层引用重复计数）
        assert!(
            props.len() >= 4,
            "应收集至少 5 个自定义属性: {:?}",
            props.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()
        );
        let out = substitute_vars(css, &props);
        // --cda 替换
        assert!(out.contains("color:rgba(0,0,0,.7)"), "color var 替换: {out}");
        // --mpm 替换
        assert!(out.contains("box-shadow:0 0 0 1px rgba(0,0,0,.05)"), "mpm 替换: {out}");
        // background-image 保留（data-URI 不受 var 影响）
        assert!(
            out.contains("background-image:url(data:image/svg+xml"),
            "data-URI bg 保留: {out}"
        );
    }

    #[test]
    fn unresolvable_var_kept() {
        let css = ".a { color: var(--unknown); }";
        let props = vec![];
        let out = substitute_vars(css, &props);
        assert!(out.contains("var(--unknown)"), "unresolvable var 应保留");
    }

    #[test]
    fn important_beats_later_normal_declaration() {
        // CI-2026-10-13 smoke 回归根因：driving = WPT
        // css/css-variables/css-vars-custom-property-inheritance.html——
        // `--color-1: green !important` 在前、`--color-1: red` 在后，级联应取
        // important 的 green（同名后覆盖须让位于重要性比较）；值剥离 !important
        // 尾标记（css-variables-1 §2，标记属声明不属值）。
        let css = concat!(
            ":root { --color-1: green !important; }\n",
            ":root { --color-1: red; }\n",
            "div.color1 { background-color: var(--color-1); }\n",
            ":root { --color-2: green; }\n",
            "div.color2 { background-color: red; background-color: var(--color-2); }\n"
        );
        let props = collect_custom_properties(css);
        assert_eq!(
            props.iter().find(|(n, _)| n == "--color-1").map(|(_, v)| v.as_str()),
            Some("green"),
            "important green 须压过后面的 normal red，且值不含 !important: {props:?}"
        );
        let out = substitute_vars(css, &props);
        assert!(
            out.contains("background-color: green"),
            "color1 引用解析为 green: {out}"
        );
        assert!(!out.contains("var(--color-1)"), "color1 引用已替换: {out}");
    }

    #[test]
    fn later_important_still_wins_over_earlier_normal() {
        let css = ":root { --x: red; }\n:root { --x: blue !important; }\n.a { color: var(--x); }";
        let props = collect_custom_properties(css);
        assert_eq!(
            props.iter().find(|(n, _)| n == "--x").map(|(_, v)| v.as_str()),
            Some("blue"),
            "后置 important 压过前置 normal: {props:?}"
        );
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
