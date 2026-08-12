use super::*;

fn runtime_with_prevented_keydown(renderer_id: u64) -> RendererRuntime {
    let html = r#"<html><body>
        <input id="name" value="base">
        <script>
          globalThis.__keys = [];
          document.querySelector('#name').addEventListener('keydown', function(event) {
            globalThis.__keys.push(event.key);
            event.preventDefault();
          });
        </script>
    </body></html>"#;
    let url = "https://zero.test/keyboard-entry";
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
    runtime.focus_target("#name").unwrap();
    runtime
}

fn assert_keydown_prevented(runtime: &RendererRuntime) {
    assert_eq!(
        runtime.form_controls.get("#name").map(|state| state.value.as_str()),
        Some("base")
    );
    assert!(
        runtime
            .webview
            .as_ref()
            .unwrap()
            .form_control_value_overrides()
            .is_empty()
    );
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("globalThis.__keys.join(',')")
            .unwrap(),
        "A"
    );
}

#[test]
fn keyboard_entry_points_share_prevented_default_action() {
    let mut keyboard = runtime_with_prevented_keydown(910);
    keyboard
        .handle_keyboard_event(KeyboardEventParams {
            key: "A".to_string(),
            code: "KeyA".to_string(),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            event_type: zero_protocol::message::KeyboardEventType::Down,
        })
        .unwrap();
    assert_keydown_prevented(&keyboard);

    let mut dispatch = runtime_with_prevented_keydown(911);
    dispatch
        .handle_dispatch_dom_event(
            1,
            DispatchDomEventParams {
                selector: Some("#name".to_string()),
                x: 0.0,
                y: 0.0,
                event_type: "keydown".to_string(),
                key: Some("A".to_string()),
                code: Some("KeyA".to_string()),
                shift: false,
            },
        )
        .unwrap();
    assert_keydown_prevented(&dispatch);
}
