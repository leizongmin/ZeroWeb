// MATHFN 刀端到端钉测：style 属性文本/IDL 写面 → cascade → gCS 读全链
//（WPT numeric-testcommon _test_math 同款通道）——signs-abs-computed 语料
// INVALID 桶的宿主面验收：Rust 门修复后写面受理 + computed 读值正确串化。
#[test]
fn mathfn_idl_write_and_gcs_read_pins() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let html = "<html><body><div id=\"t\"></div></body></html>";
    // (CSS 属性名, gCS JS 侧 camelCase 属性名, 测试值, 期望 computed 读值)
    let cases: [(&str, &str, &str, &str); 8] = [
        // scale <number> 槽（number → scale）
        ("scale", "scale", "calc(10 - abs(10 - abs(-30)))", "-10"),
        ("scale", "scale", "sign(1)", "1"),
        // z-index <integer> 槽：font 相对叶短路面 + time 原子擦除形
        ("z-index", "zIndex", "sign(1em)", "1"),
        ("z-index", "zIndex", "sign(1s)", "1"),
        ("z-index", "zIndex", "sign(-3)", "-1"),
        // rotate <angle> 槽：sign 型别擦除 + 默认字体基准（42 - 2*16 = 10 → 1）
        (
            "rotate",
            "rotate",
            "calc(90deg + 30deg * sign(42px - 2em))",
            "120deg",
        ),
        // transition-delay <time> 槽：Time 叶保族负号
        ("transition-delay", "transitionDelay", "abs(-1s)", "1s"),
        // IEEE 负零钳形：sign(-0px) = -0 → 1/-0 = -∞ → clamp = -1
        (
            "z-index",
            "zIndex",
            "clamp(-1, calc( 1 / sign(sign(-0px))), 1)",
            "-1",
        ),
    ];
    for (prop, js_prop, v, want) in cases {
        let config = zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        };
        let mut sandbox = V8Sandbox::with_config(config).unwrap();
        sandbox.execute(generate_js_dom_shim()).unwrap();
        let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
        let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(html.to_string()));
        let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
        let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
            Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(
            &mut sandbox,
            &mutations,
            &dom_html,
            &page_url,
            &canvas_registry,
            None,
        );
        // IDL setter 写面（_test_math 通道）→ gCS 读。
        let g = format!(
            "var el = document.querySelector('#t');\
             el.style.{js_prop} = '{v}';\
             globalThis.__r = getComputedStyle(el).{js_prop};"
        );
        sandbox.execute(&g).unwrap();
        assert_eq!(
            sandbox.execute("globalThis.__r").unwrap().value,
            want,
            "gCS({prop}) for {v:?}"
        );
    }
}
