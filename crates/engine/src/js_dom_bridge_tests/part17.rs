// js_dom_bridge 测试模块拆分 part 17，承接 part13 的 POST submission 测试。

#[test]
fn form_owner_attribute_does_not_fall_back_to_ancestor() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><form id='a'>\
          <input id='nested'><input id='empty' form=''>\
          <input id='missing' form='missing'><input id='nonform' form='owner'>\
         </form><div id='owner'></div><form id='b'></form><input id='external' form='b'>\
         </body></html>"
            .to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/form-owner".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry);

    assert_eq!(
        sandbox
            .execute(
                "['nested','empty','missing','nonform','external'].map(function(id){\
                   var owner=document.getElementById(id).form;return owner?owner.id:'-';\
                 }).join('|')"
            )
            .unwrap()
            .value,
        "a|-|-|-|b"
    );
}

#[test]
fn form_submission_uses_live_text_values_without_changing_defaults() {
    let html = "<html><body><form id='f' action='/s'>\
        <input id='name' name='name' value='default'>\
        <textarea id='note' name='note'>default note</textarea>\
        </form></body></html>";
    let live_values = std::collections::HashMap::from([
        ("#name".to_string(), "edited".to_string()),
        ("#note".to_string(), "live note".to_string()),
        ("#stale".to_string(), "ignored".to_string()),
    ]);

    assert_eq!(
        form_get_submission_url_with_values(
            html,
            "#f",
            None,
            "https://example.com/page",
            &live_values
        ),
        Some("https://example.com/s?name=edited&note=live+note".to_string())
    );
    assert_eq!(query_attr_from_html(html, "#name", "value"), "default");
    assert_eq!(query_text_from_html(html, "#note"), "default note");
}

#[test]
fn test_form_post_submission_r3055() {
    // R3055：form_post_submission 解析 <form method=post> 提交目标（action_url + urlencoded body）。
    // 对称 R3054 GET——POST 数据在 body，action_url 不含 query。控件收集规则与 GET 完全一致。
    let base = "https://example.com/page";

    // ① 基础 POST：text + password → action_url 无 query + body=urlencoded。
    let html = "<html><body>\
        <form id='f' method='post' action='https://example.com/login'>\
          <input name='user' value='alice'>\
          <input type='password' name='pw' value='s3cret'>\
        </form></body></html>";
    let got = form_post_submission(html, "#f", None, base);
    assert_eq!(
        got,
        Some(("https://example.com/login".to_string(), "user=alice&pw=s3cret".to_string())),
        "基础 POST：action_url 无 query，body=user=alice&pw=s3cret"
    );

    // ② method=POST 大小写不敏感 + action 缺省 → base_url。
    let html2 = "<html><body><form id='f' method='POST'><input name='x' value='1'></form></body></html>";
    assert_eq!(
        form_post_submission(html2, "#f", None, base),
        Some(("https://example.com/page".to_string(), "x=1".to_string())),
        "method=POST 大写 + action 缺省 → (base_url, x=1)"
    );

    // ③ checkbox/radio/select/textarea 收集规则同 GET（body 形式）。
    let html3 = "<html><body><form id='f' method='post' action='/s'>\
        <input type='checkbox' name='c' value='y' checked>\
        <input type='checkbox' name='c2'>\
        <select name='sz'><option value='m'>M</option><option value='l' selected>L</option></select>\
        <textarea name='t'>hi there</textarea>\
        </form></body></html>";
    let (_, body3) = form_post_submission(html3, "#f", None, base).expect("POST 表单");
    assert_eq!(
        body3, "c=y&sz=l&t=hi+there",
        "POST body 控件收集同 GET（checkbox checked / select selected / textarea 文本；空格→+）"
    );

    // ④ submitter：type=submit 含 name → name=value 入 body（NodeId 比较，id'd 按钮可靠）。
    let html4 = "<html><body><form id='f' method='post' action='/s'>\
        <input name='q' value='x'>\
        <button id='go' type='submit' name='go' value='send'>Go</button>\
        </form></body></html>";
    let (_, body4) = form_post_submission(html4, "#f", Some("#go"), base).expect("POST + submitter");
    assert_eq!(body4, "q=x&go=send", "POST submitter #go name=go=send 入 body");

    // ⑤ GET 表单 → form_post_submission 返 None（method 非 post）；POST 表单 → form_get_submission_url 返 None。
    let get_html = "<html><body><form id='f' action='/s'><input name='q' value='x'></form></body></html>";
    assert_eq!(
        form_post_submission(get_html, "#f", None, base),
        None,
        "GET 表单 → form_post_submission None"
    );
    let post_html = "<html><body><form id='f' method='post' action='/s'><input name='q' value='x'></form></body></html>";
    assert_eq!(
        form_get_submission_url(post_html, "#f", None, base),
        None,
        "POST 表单 → form_get_submission_url None（互补）"
    );

    // ⑥ method=dialog → None（关 dialog，headless 不导航）。
    assert_eq!(
        form_post_submission(
            "<html><body><form id='f' method='dialog' action='/s'><input name='q' value='x'></form></body></html>",
            "#f",
            None,
            base
        ),
        None,
        "method=dialog → form_post_submission None"
    );

    // ⑦ 特殊字符 urlencoded 进 body（form-urlencoded：& → %26，= → %3D，空格 → +）。
    let html7 = "<html><body><form id='f' method='post' action='/s'><input name='q' value='a b&c=d'></form></body></html>";
    let (_, body7) = form_post_submission(html7, "#f", None, base).unwrap();
    assert!(
        body7 == "q=a+b%26c%3Dd" || body7 == "q=a%20b%26c%3Dd",
        "POST body 特殊字符 urlencoded：{body7}"
    );

    // ⑧ action 相对 → 按 base 解析为绝对（action_url）。
    let html8 = "<html><body><form id='f' method='post' action='/submit'><input name='k' value='v'></form></body></html>";
    assert_eq!(
        form_post_submission(html8, "#f", None, base),
        Some(("https://example.com/submit".to_string(), "k=v".to_string())),
        "相对 action /submit → 绝对 action_url，body 不含 query"
    );
}
