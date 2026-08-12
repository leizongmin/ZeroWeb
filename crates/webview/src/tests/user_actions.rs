use zero_page_runtime::{ActionNoopReason, HtmlActionRequest, HtmlUserAction, PageEffect};

use crate::{WebView, WebViewConfig};

fn request(target: zero_page_runtime::PageNodeRef, action: HtmlUserAction) -> HtmlActionRequest {
    HtmlActionRequest {
        target,
        action,
        shift: false,
    }
}

#[test]
fn text_focus_and_stale_actions_use_scoped_identity() {
    let html = r#"<html><body>
        <input id="name"><input id="next">
        <script>
          globalThis.__init=(globalThis.__init||0)+1;
          globalThis.__events=[];
          var name=document.querySelector('#name');
          name.addEventListener('beforeinput',function(event){
            globalThis.__events.push('beforeinput:'+event.data);
            if(event.data==='X')event.preventDefault();
          });
          name.addEventListener('input',function(event){
            globalThis.__events.push('input:'+event.data);
          });
        </script>
    </body></html>"#;
    let mut webview = WebView::new(WebViewConfig::default());
    webview.prepare_document_state("https://zero.test/actions");
    webview.load_html(html, None);
    let name = webview.page_node_ref_for_selector("#name").expect("name ref");
    let next = webview.page_node_ref_for_selector("#next").expect("next ref");

    let inserted = webview
        .dispatch_user_action(request(name, HtmlUserAction::InsertText { text: "A".to_string() }))
        .expect("insert");
    assert!(inserted.changed);
    assert!(!inserted.canceled);

    let canceled = webview
        .dispatch_user_action(request(name, HtmlUserAction::InsertText { text: "X".to_string() }))
        .expect("canceled insert");
    assert!(canceled.canceled);
    assert_eq!(
        webview.form_control_value_overrides().get("#name").map(String::as_str),
        Some("A")
    );
    assert_eq!(
        webview
            .execute_script("[globalThis.__init,globalThis.__events.join(',')].join('|')")
            .expect("event log"),
        "1|beforeinput:A,input:A,beforeinput:X"
    );

    let focused = webview
        .dispatch_user_action(request(name, HtmlUserAction::MoveFocus { forward: true }))
        .expect("focus");
    assert_eq!(focused.effects, [PageEffect::Focus(Some(next))]);
    assert_eq!(webview.user_action_focus_owner(), Some(next));

    webview.load_html("<html><body><input id=\"name\"></body></html>", None);
    let stale = webview
        .dispatch_user_action(request(name, HtmlUserAction::DeleteBackward))
        .expect("stale action");
    assert_eq!(stale.noop_reason, Some(ActionNoopReason::StaleTarget));
}

#[test]
fn checked_reset_and_submit_actions_preserve_transactions() {
    let html = r#"<html><body>
        <form id="form" action="https://zero.test/submitted" method="get">
          <input id="name" name="name" value="base">
          <input id="check" type="checkbox">
          <button id="reset" type="reset">Reset</button>
          <button id="submit" type="submit" name="go" value="1">Submit</button>
        </form>
        <script>
          globalThis.__cancelClick=false;
          globalThis.__cancelReset=false;
          globalThis.__cancelSubmit=false;
          var check=document.querySelector('#check');
          check.addEventListener('click',function(event){
            globalThis.__seenChecked=check.checked;
            if(globalThis.__cancelClick)event.preventDefault();
          });
          var form=document.querySelector('#form');
          form.addEventListener('reset',function(event){
            if(globalThis.__cancelReset)event.preventDefault();
            queueMicrotask(function(){
              globalThis.__resetValue=document.querySelector('#name').value;
            });
          });
          form.addEventListener('submit',function(event){
            document.querySelector('#name').value='listener';
            if(globalThis.__cancelSubmit)event.preventDefault();
          });
        </script>
    </body></html>"#;
    let mut webview = WebView::new(WebViewConfig::default());
    webview.prepare_document_state("https://zero.test/form");
    webview.load_html(html, None);
    let check = webview.page_node_ref_for_selector("#check").expect("check ref");
    let name = webview.page_node_ref_for_selector("#name").expect("name ref");
    let reset = webview.page_node_ref_for_selector("#reset").expect("reset ref");
    let submit = webview.page_node_ref_for_selector("#submit").expect("submit ref");

    let checked = webview
        .dispatch_user_action(request(check, HtmlUserAction::Activate))
        .expect("check");
    assert!(!checked.canceled);
    assert_eq!(
        webview
            .execute_script("String(globalThis.__seenChecked)")
            .expect("pre-activation state"),
        "true"
    );
    webview
        .execute_script("globalThis.__cancelClick=true")
        .expect("cancel click");
    let rolled_back = webview
        .dispatch_user_action(request(check, HtmlUserAction::Activate))
        .expect("rollback");
    assert!(rolled_back.canceled);
    assert_eq!(
        webview
            .execute_script("String(document.querySelector('#check').checked)")
            .expect("checked state"),
        "true"
    );

    webview
        .dispatch_user_action(request(
            name,
            HtmlUserAction::InsertText {
                text: "dirty".to_string(),
            },
        ))
        .expect("dirty value");
    let reset_result = webview
        .dispatch_user_action(request(reset, HtmlUserAction::Reset))
        .expect("reset");
    assert!(!reset_result.canceled);
    assert_eq!(
        webview
            .execute_script("String(globalThis.__resetValue)")
            .expect("reset microtask"),
        "base"
    );

    webview
        .dispatch_user_action(request(name, HtmlUserAction::InsertText { text: "X".to_string() }))
        .expect("dirty value again");
    webview
        .execute_script("globalThis.__cancelReset=true")
        .expect("cancel reset");
    let canceled_reset = webview
        .dispatch_user_action(request(reset, HtmlUserAction::Reset))
        .expect("canceled reset");
    assert!(canceled_reset.canceled);
    assert_eq!(
        webview.form_control_value_overrides().get("#name").map(String::as_str),
        Some("baseX")
    );

    let submitted = webview
        .dispatch_user_action(request(submit, HtmlUserAction::Submit))
        .expect("submit");
    assert_eq!(submitted.effects.len(), 1);
    let PageEffect::Navigate(intent) = &submitted.effects[0] else {
        panic!("expected navigation effect");
    };
    assert_eq!(intent.url, "https://zero.test/submitted?name=listener&go=1");
    webview
        .execute_script("globalThis.__cancelSubmit=true")
        .expect("cancel submit");
    let canceled_submit = webview
        .dispatch_user_action(request(submit, HtmlUserAction::Submit))
        .expect("canceled submit");
    assert!(canceled_submit.canceled);
    assert!(canceled_submit.effects.is_empty());
}
