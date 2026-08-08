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

#[test]
fn test_back_forward_hashchange_r3007() {
    // R3007：back/forward/go 跨 hash entry 须同时派 hashchange（spec：hash 变更的导航派 popstate + hashchange）。
    // 旧 _hist_dispatchPopState 仅派 popstate → hash router 的 back 按钮处理失效。本切片闭合。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/path".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 建 hash 序列：#foo → #bar（cursor 在 #bar）。装 hashchange + popstate listener。
    sandbox
        .execute(
            "location.hash = '#foo';\
             location.hash = '#bar';\
             globalThis.__hc = null; globalThis.__popFired = false;\
             addEventListener('hashchange', function(e){ globalThis.__hc = { newURL: e.newURL, oldURL: e.oldURL }; });\
             addEventListener('popstate', function(){ globalThis.__popFired = true; });",
        )
        .unwrap();

    // back()：cursor 回 #foo，须派 hashchange（newURL 含 #foo）+ popstate。
    sandbox.execute("history.back();").unwrap();
    sandbox
        .execute(
            "globalThis.__hcNew = globalThis.__hc ? globalThis.__hc.newURL : '(none)';\
             globalThis.__hcOld = globalThis.__hc ? globalThis.__hc.oldURL : '(none)';",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__hcNew").unwrap().value, "https://example.com/path#foo", "back() 跨 hash 派 hashchange.newURL='#foo'");
    assert_eq!(sandbox.execute("globalThis.__hcOld").unwrap().value, "https://example.com/path#bar", "hashchange.oldURL='#bar'（back 前 entry）");
    assert_eq!(sandbox.execute("String(globalThis.__popFired)").unwrap().value, "true", "back() 同时派 popstate");

    // forward()：cursor 前进回 #bar，须派 hashchange（newURL #bar）。
    sandbox.execute("globalThis.__hc = null; history.forward();").unwrap();
    sandbox.execute("globalThis.__hcFwd = globalThis.__hc ? globalThis.__hc.newURL : '(none)';").unwrap();
    assert_eq!(sandbox.execute("globalThis.__hcFwd").unwrap().value, "https://example.com/path#bar", "forward() 跨 hash 派 hashchange.newURL='#bar'");

    // 跨非 hash 变更的 back（两 entry 均无 hash）不应派 hashchange。
    sandbox
        .execute(
            "history.pushState({}, '', '/p1');\
             history.pushState({}, '', '/p2');\
             globalThis.__hc = null;\
             history.back();",
        )
        .unwrap();
    sandbox.execute("globalThis.__hcNoHash = globalThis.__hc;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__hcNoHash)").unwrap().value, "null", "跨非 hash 变更的 back 不派 hashchange（两 entry 均无 hash）");
}

#[test]
fn test_location_part_setters_r3008() {
    // R3008：location.pathname/search/href 旧无 setter（静默 no-op，仅 hash 有 setter per R3006）→ URL 变更路由
    // / 测试场景失效。补 setter：经 URL part setter 计算新 href，push history entry（navigation 语义，R3005 location 读之反映），
    // hash 变化时派 hashchange（与 hash setter / back-forward 对称）。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig { persistent_context: true, ..Default::default() };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://example.com/old?q=1".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // location.pathname = '/new'：pathname 替换，search/hash 保留。
    sandbox
        .execute(
            "globalThis.__p0 = location.pathname;\
             location.pathname = '/new';\
             globalThis.__p1 = location.pathname; globalThis.__href1 = location.href; globalThis.__s1 = location.search;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__p0").unwrap().value, "/old", "初始 pathname=/old");
    assert_eq!(sandbox.execute("globalThis.__p1").unwrap().value, "/new", "location.pathname='/new' 后 pathname=/new");
    assert_eq!(sandbox.execute("globalThis.__s1").unwrap().value, "?q=1", "pathname 替换保留 search=?q=1");
    assert_eq!(sandbox.execute("globalThis.__href1").unwrap().value, "https://example.com/new?q=1", "href 反映新 pathname");

    // location.search = '?q=2'：search 替换。
    sandbox.execute("location.search = '?q=2'; globalThis.__s2 = location.search; globalThis.__href2 = location.href;").unwrap();
    assert_eq!(sandbox.execute("globalThis.__s2").unwrap().value, "?q=2", "location.search='?q=2' 后 search=?q=2");
    assert_eq!(sandbox.execute("globalThis.__href2").unwrap().value, "https://example.com/new?q=2", "href 反映新 search");

    // location.href = 绝对 URL（含 hash）：整体替换 + hash 变化派 hashchange。
    sandbox
        .execute(
            "globalThis.__hc = null;\
             addEventListener('hashchange', function(e){ globalThis.__hc = e.newURL; });\
             location.href = 'https://example.com/full#h';",
        )
        .unwrap();
    sandbox
        .execute(
            "globalThis.__href3 = location.href; globalThis.__hash3 = location.hash;\
             globalThis.__hc3 = globalThis.__hc;",
        )
        .unwrap();
    assert_eq!(sandbox.execute("globalThis.__href3").unwrap().value, "https://example.com/full#h", "location.href 绝对 URL 整体替换");
    assert_eq!(sandbox.execute("globalThis.__hash3").unwrap().value, "#h", "hash='#h'");
    assert_eq!(sandbox.execute("globalThis.__hc3").unwrap().value, "https://example.com/full#h", "href 引入 hash 变化派 hashchange.newURL");

    // history.length 反映导航 entry 累积（初始 1 + pathname + search + href = 4）。
    sandbox.execute("globalThis.__len = history.length;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len)").unwrap().value, "4", "location setter 推 history entry：length=4");

    // 设同值 no-op（不增 entry）。
    sandbox.execute("location.href = 'https://example.com/full#h'; globalThis.__len2 = history.length;").unwrap();
    assert_eq!(sandbox.execute("String(globalThis.__len2)").unwrap().value, "4", "设同 href no-op（不增 entry）");
}


