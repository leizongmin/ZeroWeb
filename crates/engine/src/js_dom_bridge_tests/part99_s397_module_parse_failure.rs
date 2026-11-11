// s397（module parse 失败缓存面）：tag 5 契约——module goal parse 预检失败的
// 同 URL 复 import 复抛**同一** SyntaxError 对象（module map 失败记账语义，与
// 态③ parse-eval 抛错同侧）；tag 3 链接失败对照臂保持逐次新对象（态④
// differentErrorObjects 面不回退）。红态（修复前）：parse gate 失败走 tag 3，
// 两次 import() 各得新 SyntaxError，WPT dynamic-imports-script-error
// 「the same error object for parse error」红。

#[test]
fn module_parse_failure_cached_same_object_s397() {
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};

    // persistent context：shim/prelude 与调用体分多次 execute，global 须共享
    //（同 part98 s363 形态）。
    let mut sandbox = V8Sandbox::with_config(SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // 生产同源 helper 段（__zw_load_module 消费 tag 5 的臂在本 prelude）。
    sandbox
        .execute(&zero_script_sandbox::module_dynamic_import_helpers_prelude())
        .unwrap();
    // 宿主契约 stub：parse-broken → tag 5（gate 臂形态）、link-broken → tag 3
    //（链接失败对照形态）；两键失败均不入 FetchKeys，复调用必达宿主。
    sandbox.register_callback("__zw_compile_module", Box::new(|args| {
        let spec = args.first().cloned().unwrap_or_default();
        if spec.contains("parse-broken") {
            format!("5\u{1f}{spec}\u{1f}SyntaxError: Unexpected token '%'")
        } else {
            format!("3\u{1f}{spec}\u{1f}SyntaxError: link failure")
        }
    }));
    let settled: std::sync::Arc<std::sync::Mutex<Option<String>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let settled_cb = settled.clone();
    sandbox.register_callback(
        "__zw_report",
        Box::new(move |args| {
            *settled_cb.lock().unwrap() = args.first().cloned();
            String::new()
        }),
    );
    sandbox
        .execute(
            "var e1 = null, e2 = null, l1 = null, l2 = null;\n\
             try { __zw_load_module('https://z.test/parse-broken.js', 'about:blank', '', ''); } catch (err) { e1 = err; }\n\
             try { __zw_load_module('https://z.test/parse-broken.js', 'about:blank', '', ''); } catch (err) { e2 = err; }\n\
             var sameParse = !!(e1 && e1 === e2 && e1 instanceof SyntaxError);\n\
             try { __zw_load_module('https://z.test/link-broken.js', 'about:blank', '', ''); } catch (err) { l1 = err; }\n\
             try { __zw_load_module('https://z.test/link-broken.js', 'about:blank', '', ''); } catch (err) { l2 = err; }\n\
             var freshLink = !!(l1 && l2 && l1 !== l2 && l1 instanceof SyntaxError);\n\
             __zw_report([sameParse, freshLink].join(','));\n",
        )
        .expect("tag 5 parse 失败须同对象复抛、tag 3 链接失败须逐次新对象");
    assert_eq!(
        settled.lock().unwrap().as_deref(),
        Some("true,true"),
        "parse 失败按 module map 失败记账（同对象复抛），链接失败逐次新对象（differentErrorObjects 面保持）"
    );
}
