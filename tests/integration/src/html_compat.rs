//! HTML 行为兼容性的跨 crate 契约测试。

use zero_engine::{DomMutation, form_get_submission_url_with_values};
use zero_webview::{WebView, WebViewConfig};

#[test]
fn default_actions_work_without_javascript() {
    let html = r#"<html><body>
        <form id="f" action="https://zero.test/submitted">
          <input id="name" name="name">
          <input id="subscribe" name="subscribe" value="yes" type="checkbox">
          <input id="basic" name="plan" value="basic" type="radio" checked>
          <input id="pro" name="plan" value="pro" type="radio">
        </form>
        <output id="out">unchanged</output>
        <script>document.querySelector('#out').textContent = 'listener-ran';</script>
    </body></html>"#;
    let mut webview = WebView::new(WebViewConfig::default());
    webview.load_html(html, None);

    webview
        .apply_dom_mutations_and_render(&[
            DomMutation::SetFormValue {
                selector: "#name".to_string(),
                value: "before".to_string(),
            },
            DomMutation::SetAttr {
                selector: "#subscribe".to_string(),
                name: "checked".to_string(),
                value: String::new(),
            },
            DomMutation::RemoveAttr {
                selector: "#basic".to_string(),
                name: "checked".to_string(),
            },
            DomMutation::SetAttr {
                selector: "#pro".to_string(),
                name: "checked".to_string(),
                value: String::new(),
            },
        ])
        .expect("apply default actions");

    webview
        .apply_dom_mutations_and_render(&[
            DomMutation::SetFormValue {
                selector: "#name".to_string(),
                value: String::new(),
            },
            DomMutation::RemoveAttr {
                selector: "#subscribe".to_string(),
                name: "checked".to_string(),
            },
            DomMutation::SetAttr {
                selector: "#basic".to_string(),
                name: "checked".to_string(),
                value: String::new(),
            },
            DomMutation::RemoveAttr {
                selector: "#pro".to_string(),
                name: "checked".to_string(),
            },
        ])
        .expect("apply reset defaults");

    webview
        .apply_dom_mutations_and_render(&[
            DomMutation::SetFormValue {
                selector: "#name".to_string(),
                value: "after".to_string(),
            },
            DomMutation::SetAttr {
                selector: "#subscribe".to_string(),
                name: "checked".to_string(),
                value: String::new(),
            },
            DomMutation::RemoveAttr {
                selector: "#basic".to_string(),
                name: "checked".to_string(),
            },
            DomMutation::SetAttr {
                selector: "#pro".to_string(),
                name: "checked".to_string(),
                value: String::new(),
            },
        ])
        .expect("apply post-reset defaults");

    let live_values = webview.form_control_value_overrides();
    assert_eq!(
        form_get_submission_url_with_values(
            webview.html_content(),
            "#f",
            None,
            "https://zero.test/js-disabled",
            &live_values,
        ),
        Some("https://zero.test/submitted?name=after&subscribe=yes&plan=pro".to_string())
    );
    assert_eq!(
        zero_engine::query_text_from_html(webview.html_content(), "#out"),
        "unchanged"
    );
    assert_eq!(
        zero_engine::query_attr_from_html(webview.html_content(), "#name", "value"),
        ""
    );
}

#[test]
fn label_click_activates_associated_control_once() {
    let html = r#"<html><body>
        <label id="explicit" for="check">Explicit</label>
        <input id="check" type="checkbox">
        <label id="nested">Nested <input id="radio" type="radio"></label>
    </body></html>"#;

    assert_eq!(
        zero_engine::associated_label_control_selector(html, "#explicit").as_deref(),
        Some("#check")
    );
    assert_eq!(
        zero_engine::associated_label_control_selector(html, "#nested").as_deref(),
        Some("#radio")
    );
}
