//! R5314 kill-switch 回退锚（独立测试进程——`ZW_XHTML_SELF_CLOSE=0` 恢复 HTML
//! 旧序：自闭合仅 acknowledge 后忽略、后继兄弟嵌进其内）。开关经 LazyLock 构造
//! 期单次读取，set_var 必须先于本进程首个 XHTML 文档解析（同进程双解析第二腿被
//! 缓存吞掉的教训，R5313 同款）。

use zero_dom::parse_html;

#[test]
fn kill_switch_restores_html_nested_shape() {
    unsafe { std::env::set_var("ZW_XHTML_SELF_CLOSE", "0") };
    let doc = parse_html(
        "<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.0 Strict//EN\" \"http://www.w3.org/TR/xhtml1/DTD/xhtml1-strict.dtd\"><html><body><div/><div/></body></html>",
    );
    let body = doc.query_selector(doc.root(), "body").expect("body");
    let divs: Vec<_> = doc
        .child_nodes(body)
        .into_iter()
        .filter(|&c| {
            matches!(doc.get(c).map(|n| &n.kind), Some(zero_dom::NodeKind::Element(e)) if e.name.local.as_ref() == "div")
        })
        .collect();
    assert_eq!(divs.len(), 1, "开关关闭应回退 HTML 旧序（body 仅 1 个顶层 div）");
    assert_eq!(
        doc.child_nodes(divs[0]).len(),
        1,
        "第二个 div 应嵌进第一个（HTML 旧序不变形）"
    );
}
