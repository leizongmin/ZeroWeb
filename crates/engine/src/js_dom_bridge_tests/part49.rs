// bing-t2（site-optimizer 2026-10-10，bing-20261010-r1 簇 C2）：strict classic 脚本
// 顶层声明的指令序言判定与压缩单行扫描钉测。
// 根因：bing 全站内联脚本 `//<![CDATA[` 注释前缀 + 指令在第二行，旧 is_strict
// 「首非空行恰为指令」漏检 → V8 已进 strict eval（顶层 var 困独立变量环境）而
// 导出机制不启动；且全部顶层声明在 32262 字符单行中部（`};;var _w=window,...`
// @col 26588），旧行首零缩进锚定扫描全漏。真实浏览器 Script Is A Global Code
// 恒建全局绑定（ECMA-262 §sec-scripts）。
// 复现证据：probe/t1f（离线 fixture `_w` undefined vs Chrome object）、t1j（严格性矩阵）。

/// bing #6 形态：CDATA 注释前缀 + 指令第二行 + 巨型单行中部的 var 链——
/// strict eval 下顶层 var 声明符全量经 accessor 全局可见（跨 `<script>`）。
#[test]
fn t2_cdata_strict_top_level_var_globals() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let first = crate::js_dom_bridge::script_run_classic_page(
        "//<![CDATA[\n\"use strict\";\nglobalThis._G={x:1};;var _w=globalThis,_d=globalThis,_ge=function(n){return 'ge:'+n};;var _qs=function(){return 7},_g2=globalThis\n//]]>",
        0,
        None,
    );
    sandbox.execute(&first).unwrap();
    let err = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err, "", "t2 CDATA 声明段无抛错（sentinel 干净）");
    let second = crate::js_dom_bridge::script_run_classic_page(
        "globalThis.__r = [typeof _w, typeof _d, typeof _ge, typeof _qs, typeof _g2, _ge('a'), String(_qs()), String(_w === globalThis), String(_d === globalThis)].join('|');",
        1,
        None,
    );
    sandbox.execute(&second).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "object|object|function|function|object|ge:a|7|true|true",
        "CDATA 前缀 strict 单行 var 链（行中声明）须全量全局可见（bing #6 形态）"
    );
}

/// 同形态下行中 `function` / `async function` / `class` / `const` / `let` 声明
/// （R147/WAB2-M1/R3254/R198 与 var 同一扫描面——行中压缩形态）。
#[test]
fn t2_cdata_strict_top_level_function_class_const_let() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let first = crate::js_dom_bridge::script_run_classic_page(
        "//<![CDATA[\n'use strict';\nvar _G={};;function sb_st(n){return n*2};;async function afn(){return 'a'};;function* gen1(){yield 1};;class Reg{hi(){return 'h'}};;const CC=41;;let LL=6\n//]]>",
        0,
        None,
    );
    sandbox.execute(&first).unwrap();
    let err = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err, "", "t2 CDATA function/class 段无抛错");
    let second = crate::js_dom_bridge::script_run_classic_page(
        "globalThis.__r = [typeof sb_st, String(sb_st(21)), typeof afn, typeof gen1, typeof Reg, String(new Reg().hi()), String(CC), String(LL)].join('|');",
        1,
        None,
    );
    sandbox.execute(&second).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "function|42|function|function|function|h|41|6",
        "CDATA 前缀 strict 行中 function/async/generator/class/const/let 全局可见"
    );
}

/// 指令序言（Directive Prologue）判定变体：空行 / 行注释 / 单行块注释 / 跨行块注释 /
/// 多指令序言前缀的 'use strict' 均须判 strict（V8 同判——前导注释不产生语句）。
#[test]
fn t2_strict_directive_prologue_variants() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let cases: &[(&str, &str)] = &[
        ("\"use strict\";\nvar zq_a = 5;", "zq_a"),
        ("\n\n\"use strict\";\nvar zq_b = 5;", "zq_b"),
        ("//<![CDATA[\n\"use strict\";\nvar zq_c = 5;", "zq_c"),
        ("/* header */\n\"use strict\";\nvar zq_d = 5;", "zq_d"),
        ("/* multi\nline */\n'use strict';\nvar zq_e = 5;", "zq_e"),
        ("\"use base\";\n'use strict';\nvar zq_f = 5;", "zq_f"),
    ];
    for (idx, (code, name)) in cases.iter().enumerate() {
        let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox
            .execute(&crate::js_dom_bridge::script_run_classic_page(code, 0, None))
            .unwrap();
        let probe = format!("globalThis.__v{idx} = String(typeof globalThis.{name});");
        sandbox
            .execute(&crate::js_dom_bridge::script_run_classic_page(&probe, 1, None))
            .unwrap();
        assert_eq!(
            sandbox.execute(&format!("globalThis.__v{idx}")).unwrap().value,
            "number",
            "指令序言变体 #{idx}（{name}）须判 strict 并导出 var"
        );
    }
}

/// var 链解析与负控：初始化器内逗号不分离声明符；字符串/模板字面量内的伪 `var`
/// 不导出；函数表达式名不导出；sloppy 脚本 var 保持数据属性（R201 非 strict
/// accessor 自递归回归钉）。
#[test]
fn t2_var_chain_initializer_and_negatives() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let first = crate::js_dom_bridge::script_run_classic_page(
        "//<![CDATA[\n\"use strict\";\nvar x1=Math.max(1,2),y1=3;var a2, b2;;var s1=\"a;var fake=1\",t1=2;;var tpl=`x${1}y`,u1=4;;var m1=(function(p,q){return p+q})(1,2),n1=9;;var h1=function(){return 'h'};;globalThis.k1=function kexpr(){return 2};;!function iife1(){}();\n//]]>",
        0,
        None,
    );
    sandbox.execute(&first).unwrap();
    let err = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err, "", "t2 var 链段无抛错（sentinel 干净）");
    let second = crate::js_dom_bridge::script_run_classic_page(
        "globalThis.__r = [typeof x1, String(x1), String(y1), String(typeof a2), String(typeof b2), s1, String(t1), String(typeof u1), String(m1), String(n1), h1(), String(typeof kexpr), String(typeof iife1), String(typeof fake), String(typeof nope)].join('|');",
        1,
        None,
    );
    sandbox.execute(&second).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "number|2|3|undefined|undefined|a;var fake=1|2|number|3|9|h|undefined|undefined|undefined|undefined",
        "var 链声明符全量导出（初始化器内逗号不分离）；字符串/模板内伪声明、函数表达式名不导出"
    );
}

/// R201 非 strict 钉（lit e2e 教训）：sloppy 间接 eval 的 var 本泄漏全局数据属性，
/// 不得改装 accessor（getter 自递归 Maximum call stack）。
#[test]
fn t2_sloppy_var_keeps_data_property_r201() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let first = crate::js_dom_bridge::script_run_classic_page(
        "// 无指令——sloppy\nvar log = [];\nlog.push('a');\nglobalThis.__kind = (function(d){return d ? (d.get ? 'accessor' : 'data') : 'none';})(Object.getOwnPropertyDescriptor(globalThis, 'log'));",
        0,
        None,
    );
    sandbox.execute(&first).unwrap();
    let err = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err, "", "t2 sloppy 段无抛错");
    assert_eq!(
        sandbox.execute("globalThis.__kind").unwrap().value,
        "data",
        "sloppy var 保持全局数据属性（R201 非 strict accessor 自递归回归钉）"
    );
    assert_eq!(
        sandbox.execute("String(log.length)").unwrap().value,
        "1",
        "sloppy var 泄漏语义不变"
    );
}

/// R147 关键形态钉（WPT prefixed-animation-event-tests.js 实形，bing-t2 状态机化后
/// 的一次回归收口）：`'use strict'` 无分号 + 多行注释块 + 换行 ASI 后的缩进
/// `function name(`——声明关键字不可能延续表达式，操作数结尾（此处字符串字面量）
/// 加换行即语句边界。改前该形态状态机漏导出（webkit-animation-*-event 4 文件
/// Timeout→Fail "runAnimationEventTests is not defined" 实证）。
#[test]
fn t2_no_semicolon_directive_newline_asi_function_r147() {
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    let first = crate::js_dom_bridge::script_run_classic_page(
        "'use strict'\n\n// Runs a set of tests for a given prefixed/unprefixed\n// animation event (e.g. animationstart/webkitAnimationStart).\n//\n// The eventDetails object must have the following form.\nfunction runAnimationEventTests(details) {\n  return 'ran:' + details;\n}\n\n/* block\n   comment */\nvar lastHelper = 1\nasync function asyncHelper() { return 'ah'; }\n",
        0,
        None,
    );
    sandbox.execute(&first).unwrap();
    let err = sandbox
        .execute(&crate::js_dom_bridge::page_script_error_check())
        .unwrap()
        .value;
    assert_eq!(err, "", "R147 ASI 形态声明段无抛错（sentinel 干净）");
    let second = crate::js_dom_bridge::script_run_classic_page(
        "globalThis.__r = [typeof runAnimationEventTests, runAnimationEventTests('e'), typeof lastHelper, String(typeof globalThis.lastHelper), typeof asyncHelper].join('|');",
        1,
        None,
    );
    sandbox.execute(&second).unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r").unwrap().value,
        "function|ran:e|number|number|function",
        "无分号指令 + 注释 + 换行 ASI 后的缩进 function/var/async function 须全局可见"
    );
}
