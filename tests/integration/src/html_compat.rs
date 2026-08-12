//! HTML 行为兼容性的跨 crate 契约测试。

use zero_engine::{DomMutation, form_get_submission_url_with_values};
use zero_page_runtime::{HtmlActionRequest, HtmlUserAction, JsExecutor, PageEffect, PageNodeRef};
use zero_webview::{WebView, WebViewConfig};

const CONFORMANCE_SCRIPT: &str = r#"
globalThis.__events=[];
var name=document.querySelector('#name');
var check=document.querySelector('#check');
var form=document.querySelector('#form');
name.addEventListener('beforeinput',function(event){
  globalThis.__events.push('beforeinput:'+event.data);
  if(globalThis.__prevented)event.preventDefault();
});
name.addEventListener('input',function(event){
  globalThis.__events.push('input:'+event.data);
});
check.addEventListener('click',function(event){
  globalThis.__events.push('click:'+check.checked);
  if(globalThis.__prevented)event.preventDefault();
});
check.addEventListener('input',function(){
  globalThis.__events.push('input-check:'+check.checked);
});
check.addEventListener('change',function(){
  globalThis.__events.push('change:'+check.checked);
});
form.addEventListener('reset',function(event){
  globalThis.__events.push('reset');
  if(globalThis.__prevented)event.preventDefault();
});
form.addEventListener('submit',function(event){
  globalThis.__events.push('submit');
  if(globalThis.__prevented)event.preventDefault();
});
"#;

fn conformance_html(prevented: bool) -> String {
    format!(
        r#"<html><body>
        <form id="form" action="https://zero.test/submitted" method="get">
          <input id="name" name="name">
          <input id="next">
          <input id="check" name="subscribe" value="yes" type="checkbox">
          <button id="reset" type="reset">Reset</button>
          <button id="submit" type="submit" name="go" value="1">Submit</button>
        </form>
        <script>globalThis.__prevented={prevented};{CONFORMANCE_SCRIPT}</script>
        </body></html>"#
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ActionObservable {
    value: String,
    checked: bool,
    focus: Option<String>,
    events: Vec<String>,
    navigation: Option<(String, String, Option<String>)>,
}

fn action_request(target: PageNodeRef, action: HtmlUserAction) -> HtmlActionRequest {
    HtmlActionRequest {
        target,
        action,
        shift: false,
    }
}

fn dispatch(
    webview: &mut WebView,
    executor: Option<&dyn JsExecutor>,
    target: PageNodeRef,
    action: HtmlUserAction,
) -> zero_webview::WebViewUserActionResult {
    match executor {
        Some(executor) => webview.dispatch_external_user_action(executor, action_request(target, action)),
        None => webview.dispatch_user_action(action_request(target, action)),
    }
    .expect("dispatch user action")
}

fn run_conformance_host(executor: Option<&dyn JsExecutor>, prevented: bool) -> ActionObservable {
    let html = conformance_html(prevented);
    let url = "https://zero.test/form";
    let mut webview = WebView::new(WebViewConfig::default());
    webview.prepare_document_state(url);
    webview.load_html(&html, None);
    if let Some(executor) = executor {
        executor.set_dom_snapshot(&html, url);
        executor
            .execute_script_direct(&format!("globalThis.__prevented={prevented};{CONFORMANCE_SCRIPT}"))
            .expect("register external listeners");
    }
    let name = webview.page_node_ref_for_selector("#name").expect("name");
    let check = webview.page_node_ref_for_selector("#check").expect("check");
    let reset = webview.page_node_ref_for_selector("#reset").expect("reset");
    let submit = webview.page_node_ref_for_selector("#submit").expect("submit");

    let mut navigation = None;
    let actions = [
        (name, HtmlUserAction::InsertText { text: "A".to_string() }),
        (check, HtmlUserAction::Activate),
        (name, HtmlUserAction::MoveFocus { forward: true }),
        (reset, HtmlUserAction::Reset),
        (name, HtmlUserAction::InsertText { text: "B".to_string() }),
        (check, HtmlUserAction::Activate),
        (submit, HtmlUserAction::Submit),
    ];
    for (target, action) in actions {
        let result = dispatch(&mut webview, executor, target, action);
        for effect in result.effects {
            if let PageEffect::Navigate(intent) = effect {
                navigation = Some((intent.url, intent.method, intent.body));
            }
        }
    }
    let events = match executor {
        Some(executor) => executor.execute_script_direct("globalThis.__events.join('|')"),
        None => webview
            .execute_script("globalThis.__events.join('|')")
            .map_err(|error| error.to_string()),
    }
    .expect("read event log")
    .split('|')
    .filter(|event| !event.is_empty())
    .map(str::to_string)
    .collect();
    let focus = webview
        .user_action_focus_owner()
        .and_then(|node| webview.selector_for_page_node_handle(node.node().get()));
    ActionObservable {
        value: webview
            .form_control_value_overrides()
            .get("#name")
            .cloned()
            .unwrap_or_default(),
        checked: zero_engine::has_attribute(webview.html_content(), "#check", "checked"),
        focus,
        events,
        navigation,
    }
}

fn run_renderer_host(prevented: bool) -> ActionObservable {
    let mut worker = zero_renderer::js_worker::RendererJsWorker::spawn(91);
    let observable = run_conformance_host(Some(&worker), prevented);
    worker.shutdown();
    observable
}

fn run_tab_worker_host(prevented: bool) -> ActionObservable {
    let mut worker = zero_browser::tab_js_worker::TabJsWorkerHandle::spawn(zero_browser_shell::TabId(92));
    let observable = run_conformance_host(Some(&worker), prevented);
    worker.shutdown();
    observable
}

fn run_webview_host(prevented: bool) -> ActionObservable {
    run_conformance_host(None, prevented)
}

#[test]
fn default_action_conformance_across_hosts() {
    let renderer = run_renderer_host(false);
    let tab_worker = run_tab_worker_host(false);
    let webview = run_webview_host(false);
    assert_eq!(renderer, tab_worker);
    assert_eq!(renderer, webview);
    assert_eq!(renderer.value, "B");
    assert!(renderer.checked);
    assert_eq!(renderer.focus.as_deref(), Some("#next"));
    assert_eq!(
        renderer.navigation,
        Some((
            "https://zero.test/submitted?name=B&subscribe=yes&go=1".to_string(),
            "GET".to_string(),
            None,
        ))
    );
}

#[test]
fn prevented_action_conformance_across_hosts() {
    let renderer = run_renderer_host(true);
    let tab_worker = run_tab_worker_host(true);
    let webview = run_webview_host(true);
    assert_eq!(renderer, tab_worker);
    assert_eq!(renderer, webview);
    assert_eq!(renderer.value, "");
    assert!(!renderer.checked);
    assert_eq!(renderer.focus.as_deref(), Some("#next"));
    assert!(renderer.navigation.is_none());
}

#[test]
fn deterministic_short_action_replay_across_hosts() {
    fn replay(executor: Option<&dyn JsExecutor>) -> (String, bool) {
        let html = conformance_html(false);
        let mut webview = WebView::new(WebViewConfig::default());
        webview.prepare_document_state("https://zero.test/replay");
        webview.load_html(&html, None);
        if let Some(executor) = executor {
            executor.set_dom_snapshot(&html, "https://zero.test/replay");
            executor
                .execute_script_direct(&format!("globalThis.__prevented=false;{CONFORMANCE_SCRIPT}"))
                .expect("register replay listeners");
        }
        let name = webview.page_node_ref_for_selector("#name").expect("name");
        let check = webview.page_node_ref_for_selector("#check").expect("check");
        for _ in 0..20 {
            let _ = dispatch(
                &mut webview,
                executor,
                name,
                HtmlUserAction::InsertText { text: "x".to_string() },
            );
            let _ = dispatch(&mut webview, executor, name, HtmlUserAction::DeleteBackward);
            let _ = dispatch(&mut webview, executor, check, HtmlUserAction::Activate);
            let _ = dispatch(&mut webview, executor, check, HtmlUserAction::Activate);
        }
        (
            webview
                .form_control_value_overrides()
                .get("#name")
                .cloned()
                .unwrap_or_default(),
            zero_engine::has_attribute(webview.html_content(), "#check", "checked"),
        )
    }

    let mut renderer_worker = zero_renderer::js_worker::RendererJsWorker::spawn(93);
    let mut tab_worker = zero_browser::tab_js_worker::TabJsWorkerHandle::spawn(zero_browser_shell::TabId(94));
    let renderer = replay(Some(&renderer_worker));
    let tab = replay(Some(&tab_worker));
    let webview = replay(None);
    renderer_worker.shutdown();
    tab_worker.shutdown();
    assert_eq!(renderer, tab);
    assert_eq!(renderer, webview);
    assert_eq!(renderer, (String::new(), false));
}

#[test]
fn text_constraint_conformance_across_hosts() {
    const SCRIPT: &str = r#"
globalThis.__events=[];
['readonly','limited'].forEach(function(id){
  var input=document.getElementById(id);
  input.addEventListener('beforeinput',function(event){
    globalThis.__events.push(id+':beforeinput:'+event.data);
  });
  input.addEventListener('input',function(event){
    globalThis.__events.push(id+':input:'+event.data);
  });
});
document.getElementById('limited').setSelectionRange(1,1);
"#;
    let html = format!(
        r#"<html><body>
        <input id="readonly" value="fixed" readonly>
        <input id="limited" value="A" maxlength="3" minlength="4">
        <script>{SCRIPT}</script>
        </body></html>"#
    );

    fn run(
        html: &str,
        executor: Option<&dyn JsExecutor>,
    ) -> (String, String, String, Vec<Option<zero_page_runtime::ActionNoopReason>>) {
        let url = "https://zero.test/text-constraints";
        let mut webview = WebView::new(WebViewConfig::default());
        webview.prepare_document_state(url);
        webview.load_html(html, None);
        if let Some(executor) = executor {
            executor.set_dom_snapshot(html, url);
            executor
                .execute_script_direct(SCRIPT)
                .expect("register constraint listeners");
        }
        let readonly = webview.page_node_ref_for_selector("#readonly").expect("readonly");
        let limited = webview.page_node_ref_for_selector("#limited").expect("limited");
        let results = [
            dispatch(
                &mut webview,
                executor,
                readonly,
                HtmlUserAction::InsertText { text: "x".to_string() },
            ),
            dispatch(
                &mut webview,
                executor,
                limited,
                HtmlUserAction::InsertText {
                    text: "😀B".to_string(),
                },
            ),
            dispatch(
                &mut webview,
                executor,
                limited,
                HtmlUserAction::InsertText { text: "x".to_string() },
            ),
        ];
        let events = match executor {
            Some(executor) => executor.execute_script_direct("globalThis.__events.join('|')"),
            None => webview
                .execute_script("globalThis.__events.join('|')")
                .map_err(|error| error.to_string()),
        }
        .expect("constraint event log");
        let validity = match executor {
            Some(executor) => executor.execute_script_direct(
                "[String(document.getElementById('limited').validity.tooShort),\
                  String(document.getElementById('limited').checkValidity())].join(',')",
            ),
            None => webview
                .execute_script(
                    "[String(document.getElementById('limited').validity.tooShort),\
                      String(document.getElementById('limited').checkValidity())].join(',')",
                )
                .map_err(|error| error.to_string()),
        }
        .expect("constraint validity");
        (
            webview
                .form_control_value_overrides()
                .get("#limited")
                .cloned()
                .unwrap_or_default(),
            events,
            validity,
            results.into_iter().map(|result| result.noop_reason).collect(),
        )
    }

    let mut renderer_worker = zero_renderer::js_worker::RendererJsWorker::spawn(95);
    let mut tab_worker = zero_browser::tab_js_worker::TabJsWorkerHandle::spawn(zero_browser_shell::TabId(96));
    let renderer = run(&html, Some(&renderer_worker));
    let tab = run(&html, Some(&tab_worker));
    let webview = run(&html, None);
    renderer_worker.shutdown();
    tab_worker.shutdown();
    assert_eq!(renderer, tab);
    assert_eq!(renderer, webview);
    assert_eq!(renderer.0, "A😀");
    assert_eq!(renderer.1, "limited:beforeinput:😀|limited:input:😀");
    assert_eq!(renderer.2, "true,false");
    assert_eq!(
        renderer.3,
        [
            Some(zero_page_runtime::ActionNoopReason::ReadOnlyTarget),
            None,
            Some(zero_page_runtime::ActionNoopReason::MaxLengthReached),
        ]
    );
}

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
fn disabled_fieldset_blocks_interaction_and_submission() {
    // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#attr-fe-disabled
    let html = r#"<html><body>
        <form id="form" action="https://zero.test/submitted">
          <fieldset disabled>
            <legend><input id="legend" name="legend" value="allowed"></legend>
            <input id="blocked" name="blocked" value="fixed">
            <input id="check" name="check" value="yes" type="checkbox">
          </fieldset>
          <input id="after" name="after" value="outside">
        </form>
    </body></html>"#;
    let mut webview = WebView::new(WebViewConfig::default());
    webview.load_html(html, None);
    let legend = webview.page_node_ref_for_selector("#legend").expect("legend control");
    let blocked = webview.page_node_ref_for_selector("#blocked").expect("blocked control");
    let check = webview.page_node_ref_for_selector("#check").expect("blocked checkbox");

    let blocked_insert = dispatch(
        &mut webview,
        None,
        blocked,
        HtmlUserAction::InsertText { text: "x".to_string() },
    );
    let blocked_activate = dispatch(&mut webview, None, check, HtmlUserAction::Activate);
    let moved = dispatch(&mut webview, None, legend, HtmlUserAction::MoveFocus { forward: true });

    assert_eq!(
        blocked_insert.noop_reason,
        Some(zero_page_runtime::ActionNoopReason::DisabledTarget)
    );
    assert_eq!(
        blocked_activate.noop_reason,
        Some(zero_page_runtime::ActionNoopReason::DisabledTarget)
    );
    assert_eq!(
        webview
            .user_action_focus_owner()
            .and_then(|node| webview.selector_for_page_node_handle(node.node().get()))
            .as_deref(),
        Some("#after")
    );
    assert_eq!(moved.noop_reason, None);
    assert_eq!(
        form_get_submission_url_with_values(
            webview.html_content(),
            "#form",
            None,
            "https://zero.test/form",
            &webview.form_control_value_overrides(),
        ),
        Some("https://zero.test/submitted?legend=allowed&after=outside".to_string())
    );
}

#[test]
fn option_activation_conformance_across_hosts() {
    // https://html.spec.whatwg.org/multipage/form-elements.html#concept-option-selectedness
    const SCRIPT: &str = r#"
globalThis.__optionEvents=[];
globalThis.__preventOption=false;
['a','b','m2'].forEach(function(id){
  document.getElementById(id).addEventListener('click',function(event){
    globalThis.__optionEvents.push('click:'+id+':'+String(this.selected));
    if(id==='a'&&globalThis.__preventOption)event.preventDefault();
  });
});
['s','m'].forEach(function(id){
  var select=document.getElementById(id);
  select.addEventListener('input',function(){globalThis.__optionEvents.push('input:'+id+':'+select.value);});
  select.addEventListener('change',function(){globalThis.__optionEvents.push('change:'+id+':'+select.value);});
});
"#;
    let html = format!(
        r#"<html><body>
        <select id="s"><option id="a" selected>A</option><option id="b">B</option></select>
        <select id="m" multiple><option id="m1" selected>M1</option><option id="m2">M2</option></select>
        <script>{SCRIPT}</script>
        </body></html>"#
    );

    fn run(html: &str, executor: Option<&dyn JsExecutor>) -> (String, Vec<bool>) {
        let url = "https://zero.test/options";
        let mut webview = WebView::new(WebViewConfig::default());
        webview.prepare_document_state(url);
        webview.load_html(html, None);
        if let Some(executor) = executor {
            executor.set_dom_snapshot(html, url);
            executor
                .execute_script_direct(SCRIPT)
                .expect("register option listeners");
        }
        let b = webview.page_node_ref_for_selector("#b").expect("option b");
        let m2 = webview.page_node_ref_for_selector("#m2").expect("option m2");
        let a = webview.page_node_ref_for_selector("#a").expect("option a");
        let first = dispatch(&mut webview, executor, b, HtmlUserAction::Activate);
        assert!(!zero_engine::has_attribute(webview.html_content(), "#a", "selected"));
        assert!(zero_engine::has_attribute(webview.html_content(), "#b", "selected"));
        let second = dispatch(&mut webview, executor, m2, HtmlUserAction::Activate);
        assert_eq!(
            zero_engine::option_activation_snapshot(webview.html_content(), "#a")
                .and_then(|state| state.previous_selected_selector),
            Some("#b".to_string())
        );
        match executor {
            Some(executor) => executor
                .execute_script_direct("globalThis.__preventOption=true")
                .expect("enable cancellation"),
            None => webview
                .execute_script("globalThis.__preventOption=true")
                .expect("enable cancellation"),
        };
        let canceled = dispatch(&mut webview, executor, a, HtmlUserAction::Activate);
        assert!(!zero_engine::has_attribute(webview.html_content(), "#a", "selected"));
        assert!(zero_engine::has_attribute(webview.html_content(), "#b", "selected"));
        let script = "var s=document.getElementById('s'),m1=document.getElementById('m1'),m2=document.getElementById('m2');\
                      [s.value,String(m1.selected),String(m2.selected),globalThis.__optionEvents.join(',')].join('|')";
        let observed = match executor {
            Some(executor) => executor.execute_script_direct(script),
            None => webview.execute_script(script).map_err(|error| error.to_string()),
        }
        .expect("option observable");
        (observed, vec![first.canceled, second.canceled, canceled.canceled])
    }

    let mut renderer = zero_renderer::js_worker::RendererJsWorker::spawn(98);
    let mut tab = zero_browser::tab_js_worker::TabJsWorkerHandle::spawn(zero_browser_shell::TabId(99));
    let renderer_result = run(&html, Some(&renderer));
    let tab_result = run(&html, Some(&tab));
    let webview_result = run(&html, None);
    renderer.shutdown();
    tab.shutdown();

    assert_eq!(renderer_result, tab_result);
    assert_eq!(renderer_result, webview_result);
    assert_eq!(renderer_result.1, [false, false, true]);
    assert_eq!(
        renderer_result.0,
        "B|true|true|click:b:true,input:s:B,change:s:B,click:m2:true,input:m:M1,change:m:M1,click:a:true"
    );
}

#[test]
fn output_reset_and_form_owner_conformance_across_hosts() {
    // https://html.spec.whatwg.org/multipage/form-elements.html#the-output-element
    const SCRIPT: &str = r#"
document.getElementById('nested').value='dirty-nested';
document.getElementById('external').value='dirty-external';
document.getElementById('foreign').value='dirty-foreign';
"#;
    let html = format!(
        r#"<html><body>
        <form id="form">
          <output id="nested">nested-default</output>
          <output id="foreign" form="other">foreign-default</output>
          <button id="reset" type="reset">Reset</button>
        </form>
        <output id="external" form="form">external-default</output>
        <form id="other"></form>
        <script>{SCRIPT}</script>
        </body></html>"#
    );

    fn run(html: &str, executor: Option<&dyn JsExecutor>) -> String {
        let url = "https://zero.test/output-reset";
        let mut webview = WebView::new(WebViewConfig::default());
        webview.prepare_document_state(url);
        webview.load_html(html, None);
        if let Some(executor) = executor {
            executor.set_dom_snapshot(html, url);
            executor.execute_script_direct(SCRIPT).expect("set dirty output values");
        }
        let reset = webview.page_node_ref_for_selector("#reset").expect("reset button");
        let result = dispatch(&mut webview, executor, reset, HtmlUserAction::Reset);
        assert!(!result.canceled);
        let script = "var f=document.getElementById('form'),n=document.getElementById('nested'),\
                      e=document.getElementById('external'),x=document.getElementById('foreign');\
                      [n.value,n.defaultValue,e.value,e.defaultValue,x.value,n.form.id,e.form.id,x.form.id,\
                       Array.prototype.map.call(f.elements,function(c){return c.id;}).join(',')].join('|')";
        match executor {
            Some(executor) => executor.execute_script_direct(script),
            None => webview.execute_script(script).map_err(|error| error.to_string()),
        }
        .expect("output reset observable")
    }

    let mut renderer = zero_renderer::js_worker::RendererJsWorker::spawn(100);
    let mut tab = zero_browser::tab_js_worker::TabJsWorkerHandle::spawn(zero_browser_shell::TabId(101));
    let renderer_result = run(&html, Some(&renderer));
    let tab_result = run(&html, Some(&tab));
    let webview_result = run(&html, None);
    renderer.shutdown();
    tab.shutdown();

    assert_eq!(renderer_result, tab_result);
    assert_eq!(renderer_result, webview_result);
    assert_eq!(
        renderer_result,
        "nested-default|nested-default|external-default|external-default|dirty-foreign|form|form|other|nested,reset,external"
    );
}

#[test]
fn non_text_selection_api_matches_input_state() {
    // https://html.spec.whatwg.org/multipage/input.html#concept-input-apply
    let mut webview = WebView::new(WebViewConfig::default());
    webview.load_html(
        "<html><body><input id='number' type='number' value='42'>\
         <script>globalThis.__selectionReady=true;</script></body></html>",
        None,
    );
    webview.run_page_scripts().expect("initialize page DOM");

    let observed = webview
        .execute_script(
            "var number=document.getElementById('number');\
             var errorName='none';\
             try{number.setSelectionRange(0,1);}catch(error){errorName=error.name;}\
             [String(number.selectionStart),errorName,number.value].join('|');",
        )
        .expect("observe selection API");

    assert_eq!(observed, "null|InvalidStateError|42");
}

#[test]
fn text_control_hit_caret_and_ime_share_boundaries() {
    // https://html.spec.whatwg.org/multipage/form-control-infrastructure.html#textFieldSelection
    let html = r#"<html><body><input id="name" value="i中😀W"></body></html>"#;
    let mut pipeline = zero_engine::RenderPipeline::new(800.0, 600.0);
    let rendered = pipeline.render_html(html, "input { width: 240px; font-size: 20px; }");
    let boundaries = &rendered.primitives().text_control_boundaries;
    let expected = boundaries
        .iter()
        .find(|boundary| boundary.utf16_offset == 4)
        .expect("boundary after surrogate pair");
    let hit = zero_browser::page_selection::hit_test_text_control_boundary(
        boundaries,
        expected.node_handle,
        expected.x + 0.1,
        expected.y + expected.height / 2.0,
    )
    .expect("text-control boundary hit");
    assert_eq!(hit.utf16_offset, 4);
    assert_eq!((hit.x, hit.y, hit.height), (expected.x, expected.y, expected.height));

    let url = "https://zero.test/pointer-selection";
    let mut webview = WebView::new(WebViewConfig::default());
    webview.prepare_document_state(url);
    webview.load_html(html, None);
    let target = webview.page_node_ref_for_selector("#name").expect("input target");
    let mut worker = zero_renderer::js_worker::RendererJsWorker::spawn(97);
    worker.set_dom_snapshot(html, url);
    webview
        .set_external_text_control_selection(&worker, target, hit.utf16_offset as usize, hit.utf16_offset as usize)
        .expect("set pointer selection");
    assert!(
        dispatch(
            &mut webview,
            Some(&worker),
            target,
            HtmlUserAction::InsertText { text: "X".to_string() },
        )
        .changed
    );
    assert_eq!(
        worker
            .execute_script_direct("document.getElementById('name').value")
            .expect("input value"),
        "i中😀XW"
    );
    worker.shutdown();
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

#[test]
fn release_uses_stable_pressed_target_across_reflow() {
    let pressed = zero_page_runtime::PageTarget::new(
        zero_page_runtime::PageNodeRef::new(7, 3, zero_page_runtime::PageNodeHandle::new(42)),
        "#pressed".to_string(),
    );
    let current_hover = zero_page_runtime::PageTarget::new(
        zero_page_runtime::PageNodeRef::new(7, 3, zero_page_runtime::PageNodeHandle::new(99)),
        "#hover".to_string(),
    );

    assert!(pressed.node_ref().is_current(7, 3));
    assert_eq!(pressed.selector(), "#pressed");
    assert_ne!(pressed.node_ref(), current_hover.node_ref());
    assert!(!pressed.node_ref().is_current(7, 4));
}
