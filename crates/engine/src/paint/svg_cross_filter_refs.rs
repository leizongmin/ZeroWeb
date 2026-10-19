//! R5200（filter-effects-1 §11.2 #element-attrdef-filter-primitiveunits +
//! SVG2 §struct-defs 文档级 id 语义）：跨 svg 元素的 `<filter>` 引用注入。
//!
//! inline `<svg>` 子树绘制走「序列化 DOM 子树 → resvg::render」路径
//!（paint_svg_element / R3938），源内 `url(#id)` 只能在序列化 mini-doc 内
//! 解析；`<filter>` 定义在**另一 svg 元素**时解析失败 → resvg 按 SVG 无效
//! 引用规则整元素不绘（svg-filter-primitive-units-user-space ②⑤ 实证：
//! 连未滤红 rect 都缺失）。SVG 里 id 全文档唯一、引用全文档解析——本模块
//! 序列化后静态走查源内 `url(#…)` fragment 引用，对子树内未定义、全文档
//! 可解析为 `<filter>` 的 id，把该 `<filter>` 子树序列化注入根 svg 闭标签
//! 前的 `<defs>`，使 mini-doc 自洽。
//!
//! 范围保守：只注入 `<filter>`（渐变/clipPath/mask 的跨 svg 引用同族但
//! 独立域，挂账）；id 在子树内已定义或全文档不存在（真无效引用）不注入
//! ——保持 resvg 无效引用不绘语义。调用点置于 R4291 xmlns 补丁与 R4270
//! taint / R5123 位移补偿等源级 pass 之前：注入内容同获命名空间声明、
//! 同受后续规则约束。
//!
//! kill-switch：env `ZW_SVG_CROSS_FILTER_REFS=0`（default-on）。

use zero_dom::{Document, NodeKind};

/// 对序列化 SVG 源注入跨 svg 引用的 `<filter>` defs（自洽化）。
pub(crate) fn inject_cross_svg_filter_defs(source: &str, doc: &Document) -> String {
    if std::env::var("ZW_SVG_CROSS_FILTER_REFS").as_deref() == Ok("0") {
        return source.to_string();
    }
    let mut injected = String::new();
    for id in collect_url_fragment_refs(source) {
        // 子树内已定义（双引号/单引号 attr 形态都查）→ 非跨 svg 引用。
        if source.contains(&format!("id=\"{id}\"")) || source.contains(&format!("id='{id}'")) {
            continue;
        }
        let Some(node_id) = doc.get_element_by_id(&id) else {
            continue;
        };
        let Some(node) = doc.get(node_id) else {
            continue;
        };
        let NodeKind::Element(elem) = &node.kind else {
            continue;
        };
        if elem.local_name() != "filter" {
            continue;
        }
        injected.push_str(&doc.outer_html(node_id));
    }
    if injected.is_empty() {
        return source.to_string();
    }
    // 根 svg 闭标签收尾于序列化末尾（嵌套 svg 先闭合）→ rfind 定位。
    let Some(close_rel) = source.rfind("</svg>") else {
        return source.to_string();
    };
    let mut out = String::with_capacity(source.len() + injected.len() + 14);
    out.push_str(&source[..close_rel]);
    out.push_str("<defs>");
    out.push_str(&injected);
    out.push_str("</defs></svg>");
    out
}

/// 收集源内 `url(#frag)` / `url('#frag')` / `url("#frag")` 引用的 fragment id
/// （去重保序）。非 `#` 开头（完整 URL/裸串）不是文档内引用，跳过。
fn collect_url_fragment_refs(source: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0usize;
    while let Some(pos) = source[i..].find("url(") {
        let start = i + pos + 4;
        let mut j = start;
        let quote = match bytes.get(j) {
            Some(b'\'') | Some(b'"') => {
                let q = bytes[j];
                j += 1;
                Some(q)
            }
            _ => None,
        };
        if bytes.get(j) == Some(&b'#') {
            j += 1;
            let id_start = j;
            while j < bytes.len() {
                let c = bytes[j];
                let terminated = match quote {
                    Some(q) => c == q,
                    None => c == b')' || c.is_ascii_whitespace(),
                };
                if terminated {
                    break;
                }
                j += 1;
            }
            let id = &source[id_start..j];
            if !id.is_empty() && !ids.iter().any(|v| v == id) {
                ids.push(id.to_string());
            }
        }
        i = start;
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::inject_cross_svg_filter_defs;
    use zero_dom::parse_html;

    #[test]
    fn r5200_cross_svg_filter_def_injected() {
        let doc = parse_html(
            r#"<html><body>
            <svg width="0" height="0"><defs><filter id="f1" primitiveUnits="userSpaceOnUse"><feFlood flood-color="green" width="100%" height="100%"/></filter></defs></svg>
            <svg id="s2" width="150" height="150"><rect x="10" y="10" width="100" height="100" fill="red" filter="url(#f1)"/></svg>
            </body></html>"#,
        );
        let s2 = doc.get_element_by_id("s2").expect("svg#s2");
        let src = doc.outer_html(s2);
        let out = inject_cross_svg_filter_defs(&src, &doc);
        assert!(out.contains("<defs><filter"), "应注入 defs 包裹：{out}");
        assert!(out.contains("id=\"f1\""), "应含注入的 filter：{out}");
        assert!(out.contains("feFlood"), "filter 子树应完整注入：{out}");
        assert!(out.ends_with("</defs></svg>"), "注入点在根闭标签前：{out}");
    }

    #[test]
    fn r5200_in_subtree_filter_ref_untouched() {
        let doc = parse_html(
            r#"<html><body>
            <svg id="s3" width="200" height="200"><defs><filter id="f2"><feFlood flood-color="green"/></filter></defs><rect filter="url(#f2)"/></svg>
            </body></html>"#,
        );
        let s3 = doc.get_element_by_id("s3").expect("svg#s3");
        let src = doc.outer_html(s3);
        assert_eq!(inject_cross_svg_filter_defs(&src, &doc), src);
    }

    #[test]
    fn r5200_missing_and_non_filter_refs_untouched() {
        let doc = parse_html(
            r#"<html><body>
            <rect id="notfilter" width="1" height="1"/>
            <svg id="s4" width="150" height="150"><rect filter="url(#nope)"/><rect filter="url('#notfilter')"/></svg>
            </body></html>"#,
        );
        let s4 = doc.get_element_by_id("s4").expect("svg#s4");
        let src = doc.outer_html(s4);
        assert_eq!(inject_cross_svg_filter_defs(&src, &doc), src);
    }

    #[test]
    fn r5200_quoted_url_form_injected() {
        let doc = parse_html(
            r#"<html><body>
            <svg width="0" height="0"><filter id="f9"><feFlood flood-color="green"/></filter></svg>
            <svg id="s5" width="150" height="150"><rect filter="url('#f9')"/></svg>
            </body></html>"#,
        );
        let s5 = doc.get_element_by_id("s5").expect("svg#s5");
        let src = doc.outer_html(s5);
        let out = inject_cross_svg_filter_defs(&src, &doc);
        assert!(out.contains("id=\"f9\""), "带引号形式同样注入：{out}");
    }
}
