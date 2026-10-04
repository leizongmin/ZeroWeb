use super::*;
use crate::page_scripts;

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
fn focused_checkbox_keeps_undimmed_accent_after_activation() {
    // Chrome 真值（R3877，2026-08-31 GUI Chrome 151 探针）：焦点态填充保持原 accent 色。
    let checked_html = r#"<html><head><style>
        #check { accent-color: rgb(0, 92, 200); }
    </style></head><body><input id="check" type="checkbox" checked></body></html>"#;
    let mut focused_runtime = runtime_with_scripts(checked_html, 926);

    focused_runtime.focus_target("#check").unwrap();
    focused_runtime
        .webview
        .as_mut()
        .unwrap()
        .reload_html_after_script(&focused_runtime.cached_html);
    assert!(
        focused_runtime
            .webview
            .as_ref()
            .unwrap()
            .last_render()
            .unwrap()
            .primitives
            .fills
            .iter()
            .any(|fill| fill.color == zero_render_foundation::color::Color::rgba(0, 92, 200, 255)),
        "focused checkbox should keep the undimmed accent before activation"
    );

    let unchecked_html = r#"<html><head><style>
        #check { accent-color: rgb(0, 92, 200); }
    </style></head><body>
        <input id="check" type="checkbox">
        <output id="state"></output>
        <script>
        document.querySelector('#check').addEventListener('change', function() {
          document.querySelector('#state').textContent = this.checked ? 'checked' : 'unchecked';
        });
        </script>
    </body></html>"#;
    let mut runtime = runtime_with_scripts(unchecked_html, 927);
    runtime.focus_target("#check").unwrap();
    let (click, handled) = runtime.dispatch_checked_click("#check".to_string()).unwrap();
    assert!(click.default_allowed);
    assert!(handled);
    assert!(zero_engine::has_attribute(&runtime.cached_html, "#check", "checked"));

    runtime.webview.as_mut().unwrap().resize(200, 100);
    runtime.webview.as_mut().unwrap().render();
    let render = runtime.webview.as_ref().unwrap().last_render().unwrap();
    assert!(
        render
            .primitives
            .fills
            .iter()
            .any(|fill| fill.color == zero_render_foundation::color::Color::rgba(0, 92, 200, 255)),
        "focused checkbox should retain the undimmed accent through activation paint; fills={:?}",
        render
            .primitives
            .fills
            .iter()
            .map(|fill| fill.color)
            .collect::<Vec<_>>()
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

#[test]
fn shared_form_actions_preserve_reset_and_submit_semantics() {
    let html = r#"<html><body>
        <form id="form" action="https://zero.test/submitted">
          <input id="name" name="name" value="base">
          <button id="reset" type="reset">Reset</button>
          <button id="submit" type="submit" name="go" value="1">Submit</button>
        </form>
        <script>
          globalThis.__cancelSubmit = true;
          document.querySelector('#form').addEventListener('reset', function() {
            queueMicrotask(function() {
              globalThis.__resetValue = document.querySelector('#name').value;
            });
          });
          document.querySelector('#form').addEventListener('submit', function(event) {
            document.querySelector('#name').value = 'listener';
            if (globalThis.__cancelSubmit) event.preventDefault();
          });
        </script>
    </body></html>"#;
    let mut runtime = runtime_with_scripts(html, 924);
    runtime.stub_network = true;
    runtime.focus_target("#name").unwrap();

    runtime.apply_text_input_at("#name", "dirty").unwrap();
    runtime.reset_form_on_click_at("#reset").unwrap();
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("[document.querySelector('#name').value,globalThis.__resetValue].join(',')")
            .unwrap(),
        "base,base"
    );

    runtime.submit_form_on_click_at("#submit").unwrap();
    assert_eq!(runtime.current_url.as_deref(), Some("https://zero.test/activation"));

    runtime
        .js_worker
        .execute_script_direct("globalThis.__cancelSubmit=false")
        .unwrap();
    runtime.submit_form_on_click_at("#submit").unwrap();
    assert_eq!(
        runtime.history.last().map(String::as_str),
        Some("https://zero.test/submitted?name=listener&go=1")
    );
}

#[test]
fn blank_target_anchor_click_navigates_s29() {
    // slice29 缺陷面钉（baidu SERP 旅程探索 P1 阻断卡）：target=_blank 锚点 click 在
    // renderer 管线曾恒 no-op（anchor_click_target 把 _blank/_top/_parent 过滤为
    // None）——SERP 结果链接全为 target=_blank，点击后无导航，旅程点击段死。
    // 修复契约：激活保导航——「新建辅助 browsing context」面嵌入态降级为当前
    // traversable 导航（与 R2979 window.open popup-blocked 降级同族）；FIXME(tab-ipc)：
    // renderer→browser 新建 tab IPC 落地后新 tab 意图交宿主。
    // https://html.spec.whatwg.org/multipage/nav-history-apis.html#chosen-navigable
    let html = r#"<html><body>
        <a id="ext" href="/dest" target="_blank">result</a>
    </body></html>"#;
    let mut runtime = runtime_with_scripts(html, 930);
    runtime.stub_network = true;

    let click = runtime.dispatch_dom_at(Some("#ext".to_string()), 0.0, 0.0, "click", None);
    assert!(click.default_allowed);

    let url = zero_engine::anchor_click_target(
        &runtime.cached_html,
        "#ext",
        runtime.current_url.as_deref().unwrap_or("about:blank"),
    )
    .unwrap_or_else(|| panic!("判据失败：target=_blank 锚点点击必须解析出导航 URL"));
    runtime
        .handle_navigate(zero_protocol::message::NavigateParams {
            url,
            referrer: runtime.current_url.clone(),
            navigation_epoch: runtime.navigation_epoch.wrapping_add(1),
        })
        .unwrap();
    assert_eq!(
        runtime.history.last().map(String::as_str),
        Some("https://zero.test/dest"),
        "_blank 锚点点击 → 当前 traversable 导航落地 history"
    );
}

#[test]
fn blank_target_anchor_hit_descendant_navigates_s29() {
    // slice29 P1b 缺陷面钉（活体 baidu SERP 判别探针：dp=false、事件 target=SPAN、
    // 无导航）：renderer hit_test 取最深元素，真实站锚点点击常命中内层后代
    // （`<a><span>标题</span></a>`）。事件路径激活目标 = 路径上首个具激活行为的元素
    // （锚点祖先）——anchor_click_target 需沿祖先行走，否则默认动作链全 miss。
    // https://dom.spec.whatwg.org/#concept-event-dispatch
    let html = r#"<html><body>
        <a href="/dest" target="_blank"><em id="tt">result title</em></a>
    </body></html>"#;
    let mut runtime = runtime_with_scripts(html, 931);
    runtime.stub_network = true;

    let click = runtime.dispatch_dom_at(Some("#tt".to_string()), 0.0, 0.0, "click", None);
    assert!(click.default_allowed, "span 命中点击默认动作不被取消");

    let url = zero_engine::anchor_click_target(
        &runtime.cached_html,
        "#tt",
        runtime.current_url.as_deref().unwrap_or("about:blank"),
    )
    .unwrap_or_else(|| panic!("判据失败：锚点内层后代命中必须沿祖先行走解析出导航 URL"));
    runtime
        .handle_navigate(zero_protocol::message::NavigateParams {
            url,
            referrer: runtime.current_url.clone(),
            navigation_epoch: runtime.navigation_epoch.wrapping_add(1),
        })
        .unwrap();
    assert_eq!(
        runtime.history.last().map(String::as_str),
        Some("https://zero.test/dest"),
        "span-in-anchor 点击 → 激活 a 祖先 → 导航落地 history"
    );
}
