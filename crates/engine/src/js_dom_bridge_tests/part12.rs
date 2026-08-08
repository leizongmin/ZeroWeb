// R3006+ 新测试（part01-11 满载后扩展，控制单文件 <2000 行）。经 js_dom_bridge_tests.rs include。

#[test]
fn test_location_hash_setter_hashchange_r3006() {
    // R3006：`location.hash = v` 须更新 hash + 新 history entry + 派发 hashchange（SPA hash 路由核心）。
    // 旧 _makeLocation 仅 getter（无 setter）→ location.hash = '#foo' 静默 no-op，hash router 全失效。
    // HashChangeEvent（R2812）已有 oldURL/newURL 字段，本切片补 setter + 派发。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/path".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 安装 hashchange listener 捕获 newURL/oldURL。
    sandbox
        .execute(
            "globalThis.__hc = null;\
             addEventListener('hashchange', function(e){ globalThis.__hc = { newURL: e.newURL, oldURL: e.oldURL }; });",
        )
        .unwrap();

    // location.hash = '#foo'：hash 更新 + history entry + hashchange 派发（_defer 异步）。
    sandbox.execute("location.hash = '#foo';").unwrap();
    sandbox
        .execute(
            "globalThis.__h1 = location.hash;\
             globalThis.__href1 = location.href;\
             globalThis.__hcNew1 = globalThis.__hc ? globalThis.__hc.newURL : '(none)';\
             globalThis.__hcOld1 = globalThis.__hc ? globalThis.__hc.oldURL : '(none)';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__h1").unwrap().value, "#foo", "location.hash='#foo' 后 hash='#foo'");
    assert_eq!(sandbox.execute("globalThis.__href1").unwrap().value, "https://example.com/path#foo", "location.href 含 #foo");
    assert_eq!(sandbox.execute("globalThis.__hcNew1").unwrap().value, "https://example.com/path#foo", "hashchange.newURL 含 #foo");
    assert_eq!(sandbox.execute("globalThis.__hcOld1").unwrap().value, "https://example.com/path", "hashchange.oldURL 无 hash");

    // 无 '#' 前缀：spec 自动补 '#'。
    sandbox
        .execute(
            "globalThis.__hc = null;\
             location.hash = 'bar';\
             globalThis.__h2 = location.hash;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__h2").unwrap().value, "#bar", "location.hash='bar'（无#）spec 自动补 → '#bar'");
    sandbox.execute("globalThis.__hcNew2 = globalThis.__hc ? globalThis.__hc.newURL : '(none)';").unwrap();
    assert_eq!(sandbox.execute("globalThis.__hcNew2").unwrap().value, "https://example.com/path#bar", "hashchange.newURL='#bar'");

    // 设同 hash：no-op（不派 hashchange，spec）。
    sandbox
        .execute(
            "globalThis.__hc = null;\
             location.hash = '#bar';\
             globalThis.__hcSame = globalThis.__hc;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__hcSame)").unwrap().value, "null", "设同 hash no-op（不派 hashchange）");

    // history.length 反映 hash entry 累积（初始 1 + #foo + #bar = 3）。
    sandbox.execute("globalThis.__len = history.length;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "3", "hash setter 推 history entry：length=3");

    // back() 回 #foo：location.hash 反映（R3005 location 读 history entry url）。
    sandbox.execute("history.back(); globalThis.__hBack = location.hash;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__hBack").unwrap().value, "#foo", "back() 后 location.hash='#foo'（history entry 反映）");
}
