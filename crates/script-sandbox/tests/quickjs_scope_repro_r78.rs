//! R78 回归测试：QuickJS `execute` 间接 eval 全局声明语义（对齐 V8 `Script::run`）。
//!
//! 根因：旧 `String(eval(code))` 是直接 eval——QuickJS 下 `function`/`var` 声明落
//! eval 临时词法环境，跨 `execute`（跨页面 `<script>`）丢失；WPT dom/traversal
//! `assert_node is not defined` 整簇 47F 的根因。修复：`(0,eval)(code)` 间接 eval
//! （spec 全局环境语义），声明落 `globalThis`。
//! https://tc39.es/ecma262/#sec-indirect-eval
#![cfg(feature = "quickjs")]

#[test]
fn qjs_execute_indirect_eval_globals() {
    let mut sb = zero_script_sandbox::QuickJSSandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .expect("sandbox");

    // 裸声明：function/var 跨 execute 存活（全局代码语义，对齐 V8）。
    sb.execute("function bareFn(){return 1}; var bareVar = 2;")
        .expect("script1 ok");
    let v = sb
        .execute("typeof bareFn + ',' + typeof bareVar")
        .expect("script2 ok")
        .value;
    assert_eq!(v, "function,number", "间接 eval：function/var 声明落全局");

    // classic try/finally 包装（webview run_page_scripts_impl 生产形态同构；
    // __zw_begin_script 由 shim 预定义，此处全局预置模拟）。
    sb.execute("globalThis.__zw_begin_script = function(){return 1};")
        .expect("shim preset ok");
    sb.execute("try{__zw_begin_script&&__zw_begin_script();\nfunction wFn(){return 3}\nvar wVar = 4;\n}finally{}")
        .expect("wrapped script1 ok");
    let v = sb
        .execute("typeof wFn + ',' + typeof wVar + ',' + (typeof globalThis.wFn)")
        .expect("wrapped script2 ok")
        .value;
    assert_eq!(v, "function,number,function", "classic 包装后声明仍落全局");

    // 脚本体自带 "use strict"：strict 语义生效（undeclared 赋值抛错），声明仍全局。
    sb.execute("'use strict'; function sFn(){return 9} sFn();")
        .expect("strict script ok");
    let v = sb.execute("typeof sFn").expect("strict decl probe ok").value;
    assert_eq!(v, "function", "strict 脚本体声明仍全局可见");
    let err = sb
        .execute("'use strict'; undeclaredX = 1;")
        .expect_err("strict undeclared 应抛");
    assert!(err.to_string().contains("undeclaredX"), "strict 语义生效: {err:?}");

    // 返回值契约：execute 仍返回 String(result)。
    let v = sb.execute("1+1").expect("expr ok").value;
    assert_eq!(v, "2");
    let v = sb.execute("'str'").expect("string ok").value;
    assert_eq!(v, "str");
}
