// s363（mixed eager/defer 同 URL 记录共享——eager 先取、defer 后取臂）：part01.js
// `__zw_load_module_deferred` 在 resolved 层命中 eager 缓存（`__moduleCache`，体已
// 求值）时，以其导出面构造「已求值三臂」记录入 defer 表，facade 属性访问直取导出
// 值——体不得二次求值（HTML spec module map 单键，两视图同一记录）。红态（修复前）：
// defer 表无记录 → 宿主重新编译+defer 求值，体跑两次。

#[test]
fn mixed_defer_hits_eager_record_s363() {
    use zero_script_sandbox::{Sandbox, SandboxConfig, V8Sandbox};

    // persistent context：shim/prelude 与调用体分多次 execute，global 须共享
    // （非 persistent 每次 execute 新建 global，先前的 shim 装载不存活）。
    let mut sandbox = V8Sandbox::with_config(SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    // eager 面 helper（s361 抽离的 prelude helper 段——与生产 module script 包同源）。
    sandbox
        .execute(&zero_script_sandbox::module_dynamic_import_helpers_prelude())
        .unwrap();
    // 宿主契约 stub：IIFE 同步求值计数（体副作用恰一次的账面）。
    sandbox.register_callback("__zw_compile_module", Box::new(|args| {
        let spec = args.first().cloned().unwrap_or_default();
        format!(
            "{spec}\u{1f}(function(){{ globalThis.__s363e = (globalThis.__s363e|0) + 1; \
             return {{ v: globalThis.__s363e }}; }})()"
        )
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
            "var spec = 'https://z.test/m.js';\n\
             var eager = __zw_load_module(spec, 'about:blank', '', '');\n\
             var afterEager = (globalThis.__s363e === 1);\n\
             var rec = __zw_load_module_deferred(spec, 'about:blank', '', '');\n\
             var shared = !!(rec && rec.exports === eager && rec.exports.v === 1);\n\
             var stillOne = (globalThis.__s363e === 1);\n\
             var facade = __zw_make_defer_facade(rec);\n\
             var facadeVal = (facade && facade.v === 1);\n\
             var facadeNoRerun = (globalThis.__s363e === 1);\n\
             __zw_report([afterEager, shared, stillOne, facadeVal, facadeNoRerun].join(','));\n",
        )
        .expect("defer 命中 eager 记录须零再求值");
    assert_eq!(
        settled.lock().unwrap().as_deref(),
        Some("true,true,true,true,true"),
        "eager 先取的 URL 被 import.defer 取用时共享同一导出面：体恰一次、facade 直取值不再触发"
    );
}
