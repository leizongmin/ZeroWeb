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

#[test]
fn typed_key_render_keeps_h1_glyph_positions_aligned_with_initial_layout() {
    // T68 复现钉（diag t63 第三发现，t68-discrim e2e 判别）：点击聚焦后帧干净，
    // **首个键入字符的渲染帧**整页字距错乱——键入走 `execute_shared_action` →
    // `dispatch_external_user_action_with_javascript` → `apply_dom_script_result` →
    // `render_with_dom_mutations`（纯表单值臂 `paint_form_value_mutations` 只重绘不
    // 重排）。该分发链是 runtime 中唯一未包 `with_measure_ctx_opt` 的产帧路径：
    // paint 阶段 `shape_text_for_paint` 在空 measure ctx 下返回 None → 整页 glyph
    // 位置回退 estimate 启发式（与初始布局的 hmtx/shaping 分叉 → "I npor t"）。
    // h1 文本两帧未变 → 同码点 glyph x 序列必须逐位相同。
    // 页面复刻 zero://bookmarks/import（pages.rs generate_bookmarks_import_html）。
    let html = "<html><head><style>\
        h1 { font-size: 28px; color: #1a73e8; }\
        </style></head><body style=\"font-family: Segoe UI, sans-serif; margin: 0; padding: 40px;\">\
        <h1>Import Bookmarks — ZeroBrowser</h1>\
        <ol style=\"font-size:14px;color:#3c4043;line-height:1.9;padding-left:20px;\">\
        <li>Export bookmarks as an HTML file (Netscape format) from your previous browser.</li>\
        <li>Open the file in a text editor and copy its entire contents.</li>\
        <li>Come back here and press the button below to import from the clipboard.</li></ol>\
        <p><a href=\"zero://bookmarks/import/apply\" style=\"display:inline-block;background:#1a73e8;color:#fff;font-size:14px;padding:9px 22px;border-radius:4px;text-decoration:none;\">Import from Clipboard</a>\
        <a href=\"zero://bookmarks\" style=\"margin-left:12px;color:#1a73e8;text-decoration:none;font-size:13px;\">View Bookmarks</a></p>\
        <hr style=\"border:none;border-top:1px solid #dadce0;margin:18px 0;\">\
        <p style=\"font-size:14px;color:#3c4043;\">Or import directly from a file:</p>\
        <form action=\"zero://bookmarks/import/file\" method=\"get\" style=\"margin:0;\">\
        <input type=\"text\" name=\"path\" value=\"\" placeholder=\"Path to bookmarks HTML file\" aria-label=\"Path to bookmarks HTML file\" style=\"font-size:14px;padding:6px 10px;border:1px solid #dadce0;border-radius:4px;width:420px;\">\
        <button type=\"submit\" style=\"margin-left:8px;font-size:14px;padding:7px 16px;background:#1a73e8;color:#fff;border:none;border-radius:4px;\">Import from File</button>\
        </form></body></html>";
    let mut runtime = RendererRuntime::new(3468);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some("zero://t68-probe".to_string());
    {
        let font_loader = &runtime.font_loader;
        let font_id = runtime.font_id;
        let wv = runtime.webview.as_mut().unwrap();
        text_metrics::with_measure_ctx_opt(font_loader, font_id, || {
            wv.load_html(html, None);
        });
    }
    let h1: Vec<char> = "Import Bookmarks — ZeroBrowser".chars().collect();
    let extract = |glyphs: &[zero_render_foundation::primitive::GlyphPrimitive], text: &[char]| -> Vec<(char, i64)> {
        // 按码点子序列匹配 h1（跨元素噪声过滤），返回 (码点, x 的 1/1000 整数)。
        // 空格不产 glyph 图元（词间隔由 advance 承载），跳过匹配。
        let mut out = Vec::new();
        let mut it = glyphs.iter();
        'outer: for &want in text {
            if want == ' ' {
                continue;
            }
            for g in it.by_ref() {
                if g.glyph_id == want as u32 {
                    out.push((want, (g.x * 1000.0).round() as i64));
                    continue 'outer;
                }
            }
            panic!("glyph for {want:?} not found");
        }
        out
    };
    let initial = extract(
        runtime
            .webview
            .as_ref()
            .unwrap()
            .last_render()
            .unwrap()
            .primitives
            .glyphs
            .as_slice(),
        &h1,
    );
    // 生产键入入口（runtime.rs apply_text_input_at → execute_shared_action）。
    runtime
        .apply_text_input_at("input[name=\"path\"]", "a")
        .expect("typed key dispatch");
    let typed = extract(
        runtime
            .webview
            .as_ref()
            .unwrap()
            .last_render()
            .unwrap()
            .primitives
            .glyphs
            .as_slice(),
        &h1,
    );
    assert_eq!(initial, typed, "键入渲染帧 h1 glyph x 与初始布局分叉（字距错乱）");
}
