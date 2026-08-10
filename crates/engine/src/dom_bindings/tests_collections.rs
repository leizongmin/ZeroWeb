//! P1b 原生 DOM 绑定测试——named-property-handler 集合面（拆自 tests_dom_api.rs，rule 5 <2000 行）。
//!
//! 覆盖：DOMTokenList（element.classList R3145）——首个 named-handler 风格集合（虽 classList 用模板方法
//! 非 named-handler，但同属 live 集合面）。后续 style/dataset 等真 named-handler 集合测试按需迁入。
//! 共享 [`run_script`]（tests.rs，pub(super)）；classList GC 回收测用低层 Isolate+Context 直装绑定。
//! 镜像 tests.rs：直接建 Isolate+Context + 安装绑定 + 执行脚本（不经 shim 字符串桥）。

use std::cell::RefCell;
use std::rc::Rc;

use v8;

use zero_dom::parse_html;

use super::gc::test_helpers::{dtl_cache_alive, reset_for_test};
use super::tests::run_script;
use super::{encode_node_id, install_dom_bindings};

// ── R3145 DOMTokenList（element.classList）── spec `dom-element-classlist` / `dom-domtokenlist-*` ──

/// `classList` 身份（同元素返同对象，spec `el.classList === el.classList`）+ 读 API
/// （length / value / item(i) / contains）。polyfill 旧每调新建，native 修正为 spec 合规。
#[test]
fn native_class_list_identity_and_read_r3145() {
    let html = r#"<div id="a" class="row  cell"></div>"#;
    // 身份：同元素两次取 → 同对象。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').classList === __zw_native_element_for_id('a').classList)"
        ),
        "true"
    );
    // length：split_whitespace 去重前计数（含多空格 + leading/trailing）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').classList.length)"),
        "2"
    );
    // value：原样 `class` 属性串（live，含多余空格未规范化——spec value = serializer 输出）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').classList.value)"),
        "row  cell"
    );
    // item(i)：文档序 token；越界 → null（字符串 "null"）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').classList.item(0))"),
        "row"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').classList.item(1))"),
        "cell"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').classList.item(2))"),
        "null"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').classList.item(-1))"),
        "null"
    );
    // contains：含/不含。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').classList.contains('cell'))"),
        "true"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').classList.contains('nope'))"),
        "false"
    );
    // 无 class 属性元素：空 DTL（length 0 / item null / contains false）。
    let html2 = r#"<div id="b"></div>"#;
    assert_eq!(
        run_script(html2, "(__zw_native_element_for_id('b').classList.length)"),
        "0"
    );
    assert_eq!(
        run_script(html2, "(__zw_native_element_for_id('b').classList.item(0))"),
        "null"
    );
}

/// 写 API（add / remove / toggle / replace）+ value setter + toString：经 `set_attribute("class", joined)`
/// 写回 owner 元素（dom crate node.class_list 自动同步），getAttribute 回读验证真实 DOM 落地。
#[test]
fn native_class_list_mutation_r3145() {
    let html = r#"<div id="a" class="a"></div>"#;
    // add（variadic + 去重）：加 b、c（c 重复加一次不重复）→ "a b c"。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const cl=__zw_native_element_for_id('a').classList;\
             cl.add('b','c','c'); return __zw_native_element_for_id('a').getAttribute('class'); })()"
        ),
        "a b c"
    );
    // remove：移 b → "a c"。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.className='a b c';\
             el.classList.remove('b'); return el.getAttribute('class'); })()"
        ),
        "a c"
    );
    // toggle（切换模式）：不在→加返 true；在→移返 false。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.className='a c';\
             const r1=el.classList.toggle('d'); const after1=el.getAttribute('class');\
             const r2=el.classList.toggle('d'); const after2=el.getAttribute('class');\
             return r1+'/'+after1+'/'+r2+'/'+after2; })()"
        ),
        "true/a c d/false/a c"
    );
    // toggle（force 模式）：force=true 在→不变返 true；force=false 在→移返 false；force=true 不在→加返 true。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.className='a';\
             const r1=el.classList.toggle('a',true); const a1=el.getAttribute('class');\
             const r2=el.classList.toggle('a',false); const a2=el.getAttribute('class');\
             const r3=el.classList.toggle('z',true); const a3=el.getAttribute('class');\
             return r1+'/'+a1+'/'+r2+'/'+a2+'/'+r3+'/'+a3; })()"
        ),
        "true/a/false//true/z"
    );
    // replace：oldT→newT 原位替换返 true；oldT 不在 → false（不写）；oldT==newT → 返是否含。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.className='a c';\
             const r1=el.classList.replace('a','x'); const a1=el.getAttribute('class');\
             const r2=el.classList.replace('nope','y'); const a2=el.getAttribute('class');\
             return r1+'/'+a1+'/'+r2+'/'+a2; })()"
        ),
        "true/x c/false/x c"
    );
    // value setter：整体替换（无 token 校验，任意串）→ set_attribute 写回。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.classList.value='p q r';\
             return el.getAttribute('class'); })()"
        ),
        "p q r"
    );
    // toString：= 当前 `class` 属性串（= value getter）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.className='m n';\
             return el.classList.toString(); })()"
        ),
        "m n"
    );
    // 移除全部 token → 属性为空串（非删除属性，spec）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.className='x y';\
             el.classList.remove('x','y'); return el.getAttribute('class')+'|'+el.hasAttribute('class'); })()"
        ),
        "|true"
    );
}

/// token 校验（spec `dom-domtokenlist-validation`）：空串 / 含空白 token → 抛 TypeError，
/// 且 add 多 token 时任一非法即抛、已校验通过的 token 不写入（spec 原子性：校验全部先于 mutation）。
/// 用 try/catch 捕获 → 返是否抛 + 抛后 class 属性是否被部分写入。
#[test]
fn native_class_list_token_validation_r3145() {
    let html = r#"<div id="a" class="a"></div>"#;
    // add("") → 抛 + 不写入（class 仍 "a"）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             try { el.classList.add(''); return 'no-throw'; }\
             catch(e) { return 'threw|'+el.getAttribute('class'); } })()"
        ),
        "threw|a"
    );
    // add("foo bar")（含空白）→ 抛。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             try { el.classList.add('foo bar'); return 'no-throw'; }\
             catch(e) { return 'threw'; } })()"
        ),
        "threw"
    );
    // add 原子性：第二 token 非法 → 抛，第一（合法）token 不写入。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             try { el.classList.add('b',''); return 'no-throw'; }\
             catch(e) { return 'threw|'+el.getAttribute('class'); } })()"
        ),
        "threw|a"
    );
    // toggle / contains / replace 同样校验非法 token → 抛。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             try { el.classList.toggle(''); return 'no-throw'; }\
             catch(e) { return 'toggle-threw'; } })()"
        ),
        "toggle-threw"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             try { el.classList.contains('a b'); return 'no-throw'; }\
             catch(e) { return 'contains-threw'; } })()"
        ),
        "contains-threw"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             try { el.classList.replace('a','x y'); return 'no-throw'; }\
             catch(e) { return 'replace-threw'; } })()"
        ),
        "replace-threw"
    );
}

/// liveness：外部 setAttribute('class') 改变反映到 classList 读（每次读经 owner 当前 class 属性
/// split_whitespace）；classList mutation 反映到 getAttribute。双向 live（spec DOMTokenList 是 live view）。
#[test]
fn native_class_list_live_reflection_r3145() {
    let html = r#"<div id="a" class="old"></div>"#;
    // 外部 setAttribute → classList 读反映（length/contains/value/item）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.setAttribute('class','x y z');\
             return el.classList.length+'/'+el.classList.contains('y')+'/'+el.classList.value+'/'+el.classList.item(2); })()"
        ),
        "3/true/x y z/z"
    );
    // classList.add → getAttribute 反映（live 写）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.className='one';\
             el.classList.add('two'); return el.getAttribute('class'); })()"
        ),
        "one two"
    );
}

/// DTL weak 身份缓存可回收（mirror R3134 NNM/ATTR）：JS 丢 classList 引用 → 多次 low_memory_notification
/// → weak 死（dtl_cache_alive false）；元素强引用仍在（globalThis.__el）。防回归：weak 化不泄漏，
/// 闭合 R3133 限制① strong-Global 泄漏在 DTL 集合面（同 NNM/ATTR R3134 pattern）。
#[test]
fn native_class_list_cache_reclaimable_on_gc_r3145() {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html(r#"<div id="a" class="row"></div>"#)));
    let ffi = encode_node_id(dom.borrow().get_element_by_id("a").expect("id a"));
    let dtl_alive;
    {
        let isolate = &mut v8::Isolate::new(Default::default());
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        // globalThis.__el 持元素强引用（元素不被 GC）；classList 仅局部持，IIFE 结束即断。
        let script = "(()=>{ globalThis.__el=__zw_native_element_for_id('a');\
             void globalThis.__el.classList;\
             return 'ok'; })()";
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let _ = compiled.run(scope).expect("run");
        for _ in 0..5 {
            scope.low_memory_notification();
        }
        dtl_alive = dtl_cache_alive(ffi);
    }
    assert!(
        !dtl_alive,
        "classList 丢 JS 引用后应可 GC（weak 死），闭合 R3133 限制① strong-Global 泄漏（DTL 面）"
    );
    reset_for_test();
}

/// R3171 CSSOM `!important`/priority：getPropertyValue 剥离 `!important`、getPropertyPriority 单独读、
/// setProperty 第三参 priority、serialize 重新附加、named setter 重置 priority、removeProperty 旧值不含。
#[test]
fn native_style_important_priority_r3171() {
    // 初始 style 属性含 !important → 解析剥离：getPropertyValue 不含，getPropertyPriority="important"。
    let html = r#"<div id="a" style="color: red !important"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').style.getPropertyValue('color'))"
        ),
        "red"
    );
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').style.getPropertyPriority('color'))"
        ),
        "important"
    );
    // setProperty priority 参 → value 不含 !important、priority="important"、style 属性附 " !important"。
    assert_eq!(
        run_script(
            r#"<div id="a"></div>"#,
            "(()=>{ const el=__zw_native_element_for_id('a'); const s=el.style;\
             s.setProperty('color','blue','important');\
             return s.getPropertyValue('color')+'/'+s.getPropertyPriority('color')+'/'+el.getAttribute('style'); })()"
        ),
        "blue/important/color: blue !important"
    );
    // named setter（el.style.color=X）重置 priority（setProperty(prop,value) 无 priority 语义）。
    assert_eq!(
        run_script(
            r#"<div id="a" style="color: red !important"></div>"#,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.style.color='green';\
             return el.style.getPropertyValue('color')+'/'+el.style.getPropertyPriority('color'); })()"
        ),
        "green/"
    );
    // removeProperty 返旧值（不含 !important）+ 删除后 priority=""。
    assert_eq!(
        run_script(
            r#"<div id="a" style="color: red !important"></div>"#,
            "(()=>{ const s=__zw_native_element_for_id('a').style;\
             const old=s.removeProperty('color');\
             return old+'/'+s.getPropertyPriority('color'); })()"
        ),
        "red/"
    );
}
