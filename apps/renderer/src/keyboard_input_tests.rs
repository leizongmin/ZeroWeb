use super::*;
use crate::page_scripts;

fn runtime_with_prevented_keydown(renderer_id: u64) -> RendererRuntime {
    let html = r#"<html><body>
        <input id="name" value="base">
        <input id="other">
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
    runtime.webview.as_mut().unwrap().prepare_document_state(url);
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
            text: None,
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
                selection_start: None,
                selection_end: None,
            },
        )
        .unwrap();
    assert_keydown_prevented(&dispatch);
}

#[test]
fn prevented_tab_keeps_focus_owner() {
    let mut runtime = runtime_with_prevented_keydown(912);
    runtime
        .handle_keyboard_event(KeyboardEventParams {
            key: "Tab".to_string(),
            code: "Tab".to_string(),
            text: None,
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            event_type: zero_protocol::message::KeyboardEventType::Down,
        })
        .unwrap();

    assert_eq!(runtime.interaction.focus_owner(), Some("#name"));
    assert!(
        runtime
            .webview
            .as_ref()
            .unwrap()
            .form_control_value_overrides()
            .is_empty()
    );
}

#[test]
fn host_focus_transition_dispatches_focus_event_order() {
    let html = r#"<html><body>
        <input id="a"><input id="b">
        <script>
          globalThis.__focusEvents = [];
          ['a','b'].forEach(function(id) {
            var el = document.querySelector('#' + id);
            ['focusout','blur','focus','focusin'].forEach(function(type) {
              el.addEventListener(type, function() { __focusEvents.push(id + ':' + type); });
            });
          });
        </script>
    </body></html>"#;
    let url = "https://zero.test/focus-order";
    let mut runtime = RendererRuntime::new(913);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.to_string();
    runtime.webview.as_mut().unwrap().prepare_document_state(url);
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

    runtime.focus_target("#a").unwrap();
    runtime
        .js_worker
        .execute_script_direct("globalThis.__focusEvents.length=0")
        .unwrap();
    runtime.blur_focused().unwrap();
    runtime.focus_target("#b").unwrap();

    // 序锚 uievents-compat M3 尾簇 6b（2026-10-04）：失焦相位 blur 先于 focusout
    //（WPT focus-events expected「blur@a → focusout@a → focus@b → focusin@b」——
    // 旧序 focusout→blur 为旧 shim 实现自定，尾簇 6b 已按上游修正）。
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("globalThis.__focusEvents.join(',')")
            .unwrap(),
        "a:blur,a:focusout,b:focus,b:focusin"
    );
}

#[test]
fn keyboard_defaults_enforce_readonly_and_maxlength() {
    let html = r#"<html><body>
        <input id="readonly" value="fixed" readonly>
        <input id="limited" value="A" maxlength="3">
        <script>
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
        </script>
    </body></html>"#;
    let url = "https://zero.test/text-constraints";
    let mut runtime = RendererRuntime::new(914);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.to_string();
    runtime.webview.as_mut().unwrap().prepare_document_state(url);
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

    runtime.focus_target("#readonly").unwrap();
    runtime
        .apply_keydown_default("#readonly", "x", Some("x"), false, false)
        .unwrap();
    runtime.blur_focused().unwrap();
    runtime.focus_target("#limited").unwrap();
    runtime
        .apply_keydown_default("#limited", "😀", Some("😀"), false, false)
        .unwrap();
    runtime
        .apply_keydown_default("#limited", "B", Some("B"), false, false)
        .unwrap();

    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct(
                "[document.getElementById('readonly').value,\
                  document.getElementById('limited').value,\
                  globalThis.__events.join('|')].join(',')"
            )
            .unwrap(),
        "fixed,A😀,limited:beforeinput:😀|limited:input:😀"
    );
}

#[test]
fn pointer_selection_uses_utf16_paint_boundary() {
    let html = r#"<html><body>
        <input id="name" value="i中😀W">
        <script>globalThis.__ready=true;</script>
    </body></html>"#;
    let url = "https://zero.test/pointer-selection";
    let mut runtime = RendererRuntime::new(915);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.to_string();
    runtime.webview.as_mut().unwrap().prepare_document_state(url);
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
        .handle_dispatch_dom_event(
            1,
            DispatchDomEventParams {
                selector: Some("#name".to_string()),
                x: 0.0,
                y: 0.0,
                event_type: "mousedown".to_string(),
                key: None,
                code: None,
                shift: false,
                selection_start: Some(2),
                selection_end: Some(2),
            },
        )
        .unwrap();
    runtime
        .apply_keydown_default("#name", "X", Some("X"), false, false)
        .unwrap();

    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("document.getElementById('name').value")
            .unwrap(),
        "i中X😀W"
    );
}

#[test]
fn ctrl_a_selects_all_text_without_inserting_character() {
    // cdp-protocol S16：Ctrl+A keydown 默认动作 = 全选文本控件（不注入 'a'）。
    let html = r#"<html><body>
        <input id="name" value="">
    </body></html>"#;
    let url = "https://zero.test/ctrl-a-select-all";
    let mut runtime = RendererRuntime::new(912);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.to_string();
    runtime.webview.as_mut().unwrap().prepare_document_state(url);
    runtime.webview.as_mut().unwrap().load_html(html, None);
    runtime.focus_target("#name").unwrap();

    // 输入 'abc'（含非 ASCII，验证 UTF-16 偏移口径）后 Ctrl+A 全选。
    runtime
        .apply_keydown_default("#name", "中", Some("中"), false, false)
        .unwrap();
    runtime
        .apply_keydown_default("#name", "a", Some("a"), false, false)
        .unwrap();
    runtime
        .apply_keydown_default("#name", "b", Some("b"), false, false)
        .unwrap();
    runtime
        .apply_keydown_default("#name", "c", Some("c"), false, false)
        .unwrap();
    // Ctrl+A 走 accel 全选分支——纯物理键语义，无需字符值（None）。
    runtime.apply_keydown_default("#name", "a", None, false, true).unwrap();

    let state = runtime.form_controls.get("#name").expect("form state");
    assert_eq!(state.value, "中abc");
    assert_eq!((state.selection_start, state.selection_end), (0, 4));
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct(
                "document.getElementById('name').selectionStart + ':' + document.getElementById('name').selectionEnd"
            )
            .unwrap(),
        "0:4"
    );
}

/// slice23 input events：全输入事件序记录页（input#name 空值聚焦；document 监听五类事件，
/// 记录 `type:事件时 value` 到 `__seq`）。
fn runtime_with_input_recorder(renderer_id: u64) -> RendererRuntime {
    let html = r#"<html><body>
        <input id="name">
        <script>
          globalThis.__seq = [];
          ['keydown','keypress','beforeinput','input','keyup'].forEach(function (t) {
            document.addEventListener(t, function (e) {
              var v = e.target && e.target.value !== undefined ? e.target.value : '';
              globalThis.__seq.push(t + ':' + v);
            });
          });
        </script>
    </body></html>"#;
    let url = "https://zero.test/keypress-synthesis";
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.to_string();
    runtime.webview.as_mut().unwrap().prepare_document_state(url);
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

#[test]
fn text_keydown_dispatches_keypress_before_input_events() {
    // slice23：产生字符值的 keydown（text 在场）→ keydown → keypress → beforeinput →
    // input（value 已落）→ keyup 全序（UI Events 键盘事件序；keypress 在插入默认动作前）。
    let mut runtime = runtime_with_input_recorder(916);
    runtime
        .handle_keyboard_event(KeyboardEventParams {
            key: "w".to_string(),
            code: "KeyW".to_string(),
            text: Some("w".to_string()),
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            event_type: zero_protocol::message::KeyboardEventType::Down,
        })
        .unwrap();
    runtime
        .handle_keyboard_event(KeyboardEventParams {
            key: "w".to_string(),
            code: "KeyW".to_string(),
            text: None,
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            event_type: zero_protocol::message::KeyboardEventType::Up,
        })
        .unwrap();

    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("globalThis.__seq.join(',')")
            .unwrap(),
        "keydown:,keypress:,beforeinput:,input:w,keyup:w",
        "keypress must fire between keydown and the insertion default action"
    );
    assert_eq!(
        runtime.form_controls.get("#name").map(|state| state.value.as_str()),
        Some("w")
    );
}

#[test]
fn textless_keydown_skips_keypress_and_insertion() {
    // slice23：纯物理键（text=None，rawKeyDown/修饰键宿主形态）不派 keypress、不插入——
    // keypress 仅属于产生字符值的键。
    let mut runtime = runtime_with_input_recorder(917);
    runtime
        .handle_keyboard_event(KeyboardEventParams {
            key: "w".to_string(),
            code: "KeyW".to_string(),
            text: None,
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            event_type: zero_protocol::message::KeyboardEventType::Down,
        })
        .unwrap();

    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("globalThis.__seq.join(',')")
            .unwrap(),
        "keydown:",
        "no keypress/insertion without a character value"
    );
    assert_eq!(
        runtime.form_controls.get("#name").map(|state| state.value.as_str()),
        Some("")
    );
}

// ===== t2-pd2：retained 表单值随快照换代回流 =====
// 根因：dirty value 只存宿主 side table（HTML §concept-fe-dirty），快照换代按裸 HTML
// 重解析 DOM 源——站点后台 JS 结构变异使 shim 值缓存键漂移后，用户键入静默丢失
// （DDG 首页 textarea[name=q] 键入 24 字符仅落 "ug" 的根因）。

fn runtime_with_nameless_textarea(renderer_id: u64) -> RendererRuntime {
    // 无 id/class 的 textarea（位置键形态，DDG 首页 textarea[name=q] 同构）+
    // 可移动兄弟（结构变异源）。
    let html = r#"<html><body>
        <div id="wrap">
          <textarea></textarea>
          <p id="p">x</p>
        </div>
    </body></html>"#;
    let url = "https://zero.test/t2pd2-typing";
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.to_string();
    runtime.webview.as_mut().unwrap().prepare_document_state(url);
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
    runtime.focus_target("textarea").unwrap();
    runtime
}

fn type_text_t2pd2(runtime: &mut RendererRuntime, chars: &str) {
    for ch in chars.chars() {
        runtime
            .handle_keyboard_event(KeyboardEventParams {
                key: ch.to_string(),
                code: "KeyT".to_string(),
                text: Some(ch.to_string()),
                ctrl: false,
                shift: false,
                alt: false,
                meta: false,
                event_type: zero_protocol::message::KeyboardEventType::Down,
            })
            .unwrap();
    }
}

#[test]
fn typed_value_survives_background_mutation_snapshot_regeneration_t2pd2() {
    let mut runtime = runtime_with_nameless_textarea(930);
    type_text_t2pd2(&mut runtime, "abc");
    // 键入已落值（宿主 retained 表 + shim 缓存双面在位）
    let overrides = runtime.webview.as_ref().unwrap().form_control_value_overrides();
    assert_eq!(
        overrides.values().cloned().collect::<Vec<_>>(),
        vec!["abc".to_string()],
        "前置：键入产生宿主 retained value"
    );

    // 站点后台 JS 结构变异：兄弟移动 → textarea 位置键漂移 → 宿主结构性重解析
    runtime
        .js_worker
        .execute_script_direct("document.querySelector('#wrap').appendChild(document.querySelector('#p')); 'ok'")
        .unwrap();
    {
        let mut ctx = PageScriptContext {
            html: &mut runtime.cached_html,
            url: "https://zero.test/t2pd2-typing",
            js_worker: &runtime.js_worker,
            webview: runtime.webview.as_mut(),
        };
        let html_snap = ctx.html.clone();
        assert!(
            page_scripts::apply_recorded_mutations(&mut ctx, &html_snap).is_some(),
            "前置：结构变异产出新快照"
        );
        // 下一帧 tick：快照重装（修复前按无值 HTML 重解析 → 值缓存 orphan → 读回 ""）
        page_scripts::tick_observers(&mut ctx);
    }
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("document.querySelector('textarea').value")
            .unwrap(),
        "abc",
        "快照换代重解析后 dirty value 必须存活（HTML §concept-fe-dirty）"
    );
    // 邻近边界：回流只写 dirty 当前值缓存，不触碰内容属性（defaultValue 语义，R2996）
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("String(document.querySelector('textarea').getAttribute('value'))")
            .unwrap(),
        "null",
        "retained 回流不得污染 value 内容属性"
    );
}

#[test]
fn navigation_snapshot_still_resets_form_state_t2pd2() {
    // 回流不得破坏导航清空语义：URL 变化的快照安装（空 retained 表）后值缓存清空。
    let mut runtime = runtime_with_nameless_textarea(931);
    type_text_t2pd2(&mut runtime, "abc");
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("document.querySelector('textarea').value")
            .unwrap(),
        "abc",
        "前置：键入落值"
    );
    // 导航：新 URL + 新文档 + 宿主 retained 表已随导航换代清空
    runtime
        .webview
        .as_mut()
        .unwrap()
        .load_html(r#"<html><body><textarea></textarea></body></html>"#, None);
    runtime.cached_html = r#"<html><body><textarea></textarea></body></html>"#.to_string();
    {
        let mut ctx = PageScriptContext {
            html: &mut runtime.cached_html,
            url: "https://zero.test/t2pd2-next",
            js_worker: &runtime.js_worker,
            webview: runtime.webview.as_mut(),
        };
        page_scripts::tick_observers(&mut ctx);
    }
    assert_eq!(
        runtime
            .js_worker
            .execute_script_direct("document.querySelector('textarea').value")
            .unwrap(),
        "",
        "导航换页必须清空上一文档的值缓存（防跨页 stale value）"
    );
}

/// t7 提交导航共享夹具：form + 指定控件焦点。
fn runtime_with_submit_form(renderer_id: u64, body_html: &str) -> RendererRuntime {
    let html = format!(r#"<html><body>{body_html}</body></html>"#);
    let url = "https://zero.test/dir/page";
    let mut runtime = RendererRuntime::new(renderer_id);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some(url.to_string());
    runtime.cached_html = html.clone();
    runtime.webview.as_mut().unwrap().prepare_document_state(url);
    runtime.webview.as_mut().unwrap().load_html(&html, None);
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

fn press_enter(runtime: &mut RendererRuntime) {
    runtime
        .handle_keyboard_event(KeyboardEventParams {
            key: "Enter".to_string(),
            code: "Enter".to_string(),
            text: None,
            ctrl: false,
            shift: false,
            alt: false,
            meta: false,
            event_type: zero_protocol::message::KeyboardEventType::Down,
        })
        .unwrap();
}

#[test]
fn enter_on_form_input_navigates_to_action_query() {
    let mut runtime = runtime_with_submit_form(
        920,
        r#"<form id="f" action="/search"><input id="q" name="q" value="rust"><input type="hidden" name="src" value="web"></form>"#,
    );
    runtime.focus_target("#q").unwrap();
    press_enter(&mut runtime);
    assert_eq!(
        runtime.current_url.as_deref(),
        Some("https://zero.test/search?q=rust&src=web"),
        "Enter 隐式提交必须导航到 action?entry-list（webview 层 SubmitForm→Navigate 转换，Rust 端构造）"
    );
}

#[test]
fn prevented_submit_keeps_url() {
    let mut runtime = runtime_with_submit_form(
        921,
        r#"<form id="f" action="/search"><input id="q" name="q" value="x">
<script>document.querySelector('#f').addEventListener('submit', function (e) { e.preventDefault(); });</script></form>"#,
    );
    runtime.focus_target("#q").unwrap();
    press_enter(&mut runtime);
    assert_eq!(
        runtime.current_url.as_deref(),
        Some("https://zero.test/dir/page"),
        "submit 事件被 preventDefault 后不得导航（HTML §4.10.22 步骤 5 取消面）"
    );
}

#[test]
fn enter_on_textarea_submits_site_compat() {
    // 已知偏离（#154 决策沿用）：规范默认 textarea Enter 换行，此处提交——DDG 等
    // textarea 搜索框站点的主路径，行为钉防回退。
    let mut runtime = runtime_with_submit_form(
        922,
        r#"<form id="f" action="/search"><textarea id="q" name="q">rust programming</textarea></form>"#,
    );
    runtime.focus_target("#q").unwrap();
    press_enter(&mut runtime);
    assert_eq!(
        runtime.current_url.as_deref(),
        Some("https://zero.test/search?q=rust+programming"),
        "textarea Enter 沿 #154 站点兼容决策走隐式提交"
    );
}

#[test]
fn relative_action_resolves_and_replaces_query() {
    let mut runtime = runtime_with_submit_form(
        923,
        r#"<form id="f" action="search?src=old"><input id="q" name="q" value="x"></form>"#,
    );
    runtime.focus_target("#q").unwrap();
    press_enter(&mut runtime);
    assert_eq!(
        runtime.current_url.as_deref(),
        Some("https://zero.test/dir/search?q=x"),
        "相对 action 按当前 URL 解析（上游 Rust form_get_submission_url_with_values），query 按规范置换而非追加"
    );
}

/// t8/F1 回归钉：worker 忙臂占住时，输入路径的有界调用必须按 USER_ACTION_SCRIPT_TIMEOUT
/// 超时降级（text_control_state_from_worker → None）并在超时上限附近返回，而不是无界
/// 阻塞至忙臂结束——键盘/交互路径的无界直呼正是 DDG 楔死（键入后求值 35s 无响应）的
/// 根因形态。回退为无界直呼时本钉红（elapsed ≥ 忙臂时长且返回 Some）。
#[test]
fn busy_worker_bounded_input_call_degrades_instead_of_blocking_t8() {
    let mut runtime = runtime_with_submit_form(
        924,
        r#"<form id="f" action="/search"><input id="q" name="q" value="x"></form>"#,
    );
    runtime.focus_target("#q").unwrap();
    // ① 3s 忙臂经 50ms 有界等待放行：worker 占用至 t0+3000（超过 2s 有界上限）。
    let _ = runtime.js_worker.execute_script_direct_bounded(
        "const __t_arm = Date.now(); while (Date.now() - __t_arm < 3000) {}",
        std::time::Duration::from_millis(50),
    );
    // ② 有界调用：2s 上限到期必返回（降级 None），不得等忙臂走完（≥3s）。
    let t0 = std::time::Instant::now();
    let snap = runtime.text_control_state_from_worker("#q");
    let elapsed = t0.elapsed();
    assert!(
        elapsed < std::time::Duration::from_millis(2900),
        "忙臂下有界调用须在超时上限附近返回（实际 {elapsed:?}；≥3s 即回退为无界直呼）"
    );
    assert!(snap.is_none(), "超时按「结果未知」降级为 None（fix#13 同语义）");
}

/// T68-D1 返修钉（缺陷首轮）：DOM-mutating click 派发（dispatch_dom_at）的
/// 渲染帧曾无度量 ctx → 整页 glyph 回退 estimate（menu/手风琴/Tab 点击均
/// 触发）。锚定帧行判定 + 可重入 ctx 修复后：onclick 改文本的 click 帧
/// 前缀 glyph x 必须与初始布局逐位一致（est 15.4px 步进 vs 真实 7.781px
/// 不规则 advance，判别力见 activation_tests 同款钉）。
#[test]
fn dom_mutating_click_dispatch_keeps_h1_glyph_positions_aligned() {
    let html = "<html><head><style>\
            h1 { font-size: 28px; color: #1a73e8; }\
            </style></head><body style=\"font-family: Segoe UI, sans-serif; margin: 0; padding: 40px;\">\
            <h1 id=\"h\">Import Bookmarks — ZeroBrowser</h1>\
            <div id=\"b\" onclick=\"document.getElementById('h').textContent='Import Bookmarks — ZeroBrowser XX';\">BTN</div>\
            </body></html>";
    let mut runtime = RendererRuntime::new(3469);
    runtime.compositor_publish = None;
    runtime.outbound = PipeTransport::new(std::io::empty(), Box::new(std::io::sink()));
    runtime.current_url = Some("zero://t68-d1-probe".to_string());
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
    // onclick 改 h1 文本（追加 XX）——click 帧覆盖前缀 + 新增尾部。
    runtime.dispatch_dom_at(Some("b".to_string()), 0.0, 0.0, "click", None);
    let after = extract(
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
    assert_eq!(
        initial, after,
        "click 派发帧 h1 前缀 glyph x 与初始布局分叉（estimate 回退）"
    );
}
