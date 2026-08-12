use super::*;

fn runtime_with_scripts(html: &str, renderer_id: u64) -> RendererRuntime {
    let url = "https://zero.test/activation";
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.to_string();
    runtime.webview.as_mut().unwrap().load_html(html, None);
    {
        let mut ctx = PageScriptContext {
            html: &mut runtime.cached_html,
            url,
            js_worker: &runtime.js_worker,
            webview: runtime.webview.as_mut(),
        };
        page_scripts::run_page_scripts(&mut ctx, true, |_url| Err::<String, String>("no fetch".into()));
    }
    runtime
}

#[test]
fn label_click_activates_associated_control_once() {
    let html = r#"<html><body>
        <label id="label" for="check">Subscribe</label>
        <input id="check" type="checkbox">
        <script>
          globalThis.__labelClicks = 0;
          globalThis.__controlClicks = 0;
          globalThis.__changes = 0;
          globalThis.__seenChecked = false;
          document.querySelector('#label').addEventListener('click', function() { __labelClicks++; });
          document.querySelector('#check').addEventListener('click', function() {
            __controlClicks++;
            __seenChecked = this.checked;
          });
          document.querySelector('#check').addEventListener('change', function() { __changes++; });
        </script>
    </body></html>"#;
    let mut runtime = runtime_with_scripts(html, 920);

    let label_click = runtime.dispatch_dom_at(Some("#label".to_string()), 0.0, 0.0, "click", None);
    assert!(label_click.default_allowed);
    assert!(runtime.activate_label_at("#label").unwrap());

    assert!(zero_engine::has_attribute(&runtime.cached_html, "#check", "checked"));
    assert_eq!(runtime.interaction.focus_owner(), Some("#check"));
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("[__labelClicks,__controlClicks,__changes,__seenChecked].join(',')")
            .unwrap(),
        "1,1,1,true"
    );
}

#[test]
fn prevented_checkbox_click_rolls_back_checkedness() {
    let html = r#"<html><body>
        <input id="check" type="checkbox">
        <script>
          globalThis.__seen = false; globalThis.__inputs = 0; globalThis.__changes = 0;
          var check = document.querySelector('#check');
          check.addEventListener('click', function(event) {
            __seen = this.checked;
            event.preventDefault();
          });
          check.addEventListener('input', function() { __inputs++; });
          check.addEventListener('change', function() { __changes++; });
        </script>
    </body></html>"#;
    let mut runtime = runtime_with_scripts(html, 921);

    let (click, handled) = runtime.dispatch_checked_click("#check".to_string()).unwrap();
    assert!(handled);
    assert!(!click.default_allowed);
    assert!(!zero_engine::has_attribute(&runtime.cached_html, "#check", "checked"));
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("[__seen,__inputs,__changes].join(',')")
            .unwrap(),
        "true,0,0"
    );
}

#[test]
fn prevented_radio_click_restores_previous_group_member() {
    let html = r#"<html><body>
        <input id="a" type="radio" name="plan" checked>
        <input id="b" type="radio" name="plan">
        <script>
          globalThis.__seen = false; globalThis.__changes = 0;
          var b = document.querySelector('#b');
          b.addEventListener('click', function(event) {
            __seen = this.checked;
            event.preventDefault();
          });
          b.addEventListener('change', function() { __changes++; });
        </script>
    </body></html>"#;
    let mut runtime = runtime_with_scripts(html, 922);

    let (click, handled) = runtime.dispatch_checked_click("#b".to_string()).unwrap();
    assert!(handled);
    assert!(!click.default_allowed);
    assert!(zero_engine::has_attribute(&runtime.cached_html, "#a", "checked"));
    assert!(!zero_engine::has_attribute(&runtime.cached_html, "#b", "checked"));
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("[__seen,__changes].join(',')")
            .unwrap(),
        "true,0"
    );
}

#[test]
fn checked_radio_reactivation_does_not_dispatch_change() {
    let html = r#"<html><body>
        <input id="a" type="radio" name="plan" checked>
        <script>
          globalThis.__clicks = 0; globalThis.__changes = 0;
          var a = document.querySelector('#a');
          a.addEventListener('click', function() { __clicks++; });
          a.addEventListener('change', function() { __changes++; });
        </script>
    </body></html>"#;
    let mut runtime = runtime_with_scripts(html, 923);

    let (click, handled) = runtime.dispatch_checked_click("#a".to_string()).unwrap();
    assert!(handled);
    assert!(click.default_allowed);
    assert!(zero_engine::has_attribute(&runtime.cached_html, "#a", "checked"));
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("[__clicks,__changes].join(',')")
            .unwrap(),
        "1,0"
    );
}
