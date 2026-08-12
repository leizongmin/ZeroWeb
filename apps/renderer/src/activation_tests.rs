use super::*;

#[test]
fn label_click_activates_associated_control_once() {
    let html = r#"<html><body>
        <label id="label" for="check">Subscribe</label>
        <input id="check" type="checkbox">
        <script>
          globalThis.__labelClicks = 0;
          globalThis.__controlClicks = 0;
          globalThis.__changes = 0;
          document.querySelector('#label').addEventListener('click', function() { __labelClicks++; });
          document.querySelector('#check').addEventListener('click', function() { __controlClicks++; });
          document.querySelector('#check').addEventListener('change', function() { __changes++; });
        </script>
    </body></html>"#;
    let url = "https://zero.test/label-activation";
    let mut runtime = RendererRuntime::new(920);
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

    let label_click = runtime.dispatch_dom_at(Some("#label".to_string()), 0.0, 0.0, "click", None);
    assert!(label_click.default_allowed);
    assert!(runtime.activate_label_at("#label").unwrap());

    assert!(zero_engine::has_attribute(&runtime.cached_html, "#check", "checked"));
    assert_eq!(runtime.interaction.focus_owner(), Some("#check"));
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("[__labelClicks,__controlClicks,__changes].join(',')")
            .unwrap(),
        "1,1,1"
    );
}
