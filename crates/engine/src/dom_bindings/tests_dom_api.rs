//! P1b 原生 DOM 绑定测试——扩展 API 表面 + 生命周期（拆自 tests.rs，rule 5 <2000 行；R3136+）。
//!
//! 覆盖：文档级工厂（documentElement/body/head、getElementsByTagName/ClassName、title、createEvent）、
//! 现代插入/替换族（remove/prepend/append/before/after/replaceWith + fragment flatten）、节点导航、
//! attributes/NamedNodeMap/Attr、innerHTML/outerHTML 序列化、cloneNode、contains、事件监听器生命周期
//! 与 GC 终结器（R3133）、NNM/Attr/DOMTokenList 身份缓存 weak 回收（R3134/R3145）。共享
//! [`run_script`]（tests.rs，pub(super)）。
//!
//! 镜像 tests.rs：直接建 Isolate+Context + 安装绑定 + 执行脚本（不经 shim 字符串桥）。

use std::cell::RefCell;
use std::rc::Rc;

use zero_dom::parse_html;

use super::gc::test_helpers::{attr_cache_alive, dtl_cache_alive, listener_keys_for, nnm_cache_alive, reset_for_test};
use super::tests::{run_script, run_script_with_referrer, run_script_with_url};
use super::{encode_node_id, install_dom_bindings};

// ── R3136 文档级只读属性工厂（documentElement / body / head）──

/// `__zw_native_get_document_element()`：返文档根元素 <html>（nodeType=1、tagName=HTML）。
/// `__zw_native_get_body()` / `__zw_native_get_head()`：返 <body>/<head> native 元素。
#[test]
fn native_document_properties_r3136() {
    let html = r#"<html><head><title>t</title></head><body><div id="a">x</div></body></html>"#;
    assert_eq!(
        run_script(html, "(__zw_native_get_document_element().tagName)"),
        "HTML",
        "documentElement 为 <html> 根元素"
    );
    assert_eq!(
        run_script(html, "(__zw_native_get_document_element().nodeType)"),
        "1",
        "documentElement nodeType=1（Element）"
    );
    assert_eq!(
        run_script(html, "(__zw_native_get_body().tagName)"),
        "BODY",
        "body 为 <body> 元素"
    );
    assert_eq!(
        run_script(html, "(__zw_native_get_head().tagName)"),
        "HEAD",
        "head 为 <head> 元素"
    );
}

/// 文档属性与既有 querySelector / getElementById 一致：body 内 `#a` 经 body.querySelector 可达。
/// 验证 documentElement/body/head 返的对象与 element 工厂共享同一 NodeId↔对象映射（身份）。
#[test]
fn native_document_properties_identity_and_navigation_r3136() {
    let html = r#"<html><head></head><body><div id="a">x</div></body></html>"#;
    // body.querySelector('#a') === getElementById('a')（同一 NodeId → 同对象）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_body().querySelector('#a') === __zw_native_element_for_id('a'))"
        ),
        "true",
        "body.querySelector('#a') 与 getElementById 返同对象（NodeId↔对象映射共享）"
    );
    // documentElement 包含 body（documentElement.contains(body)）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document_element().contains(__zw_native_get_body()))"
        ),
        "true",
        "documentElement.contains(body)===true（<html> 含 <body>）"
    );
}

/// 运行时移除 body 后 get_body() 返 null（spec：无对应元素时 null）——html5ever 总把片段归一化
/// 成完整 <html><head><body>，故 null 路径经 removeChild detach 触发（get_elements_by_tag_name DFS
/// 从 root 不再可达 detached 节点）。验证工厂 None 分支（返 null）正确。
#[test]
fn native_document_properties_absent_null_r3136() {
    let html = r#"<html><head></head><body><div id="a"></div></body></html>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const de=__zw_native_get_document_element();\
             de.removeChild(__zw_native_get_body());\
             return (__zw_native_get_body()===null); })()"
        ),
        "true",
        "removeChild(body) 后 get_body()===null（DFS 不达 detached 节点）"
    );
}

// ── R3137 document.getElementsByTagName(name) 工厂 ──

/// `__zw_native_get_elements_by_tag_name(name)`：按标签名（大小写不敏感）收集文档序 V8 Array。
/// 多个 span 文档序 + 大小写不敏感（'SPAN' 匹配 span）+ 无匹配空数组。
#[test]
fn native_get_elements_by_tag_name_r3137() {
    let html = r#"<div id="root"><span class="a">1</span><span class="b">2</span><p>x</p></div>"#;
    // 多个 span（文档序）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_tag_name('span').length)"),
        "2",
        "getElementsByTagName('span').length===2（文档序全部 span）"
    );
    // 文档序读属性（区分两 span）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_elements_by_tag_name('span')[0].className+'/'+\
             __zw_native_get_elements_by_tag_name('span')[1].className)"
        ),
        "a/b",
        "文档序 span[0]/[1] className 区分"
    );
    // 大小写不敏感（'SPAN' 匹配 span，HTML 元素）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_tag_name('SPAN').length)"),
        "2",
        "getElementsByTagName 大小写不敏感（'SPAN' 匹配 span）"
    );
    // 无匹配 → 空 Array（length 0）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_tag_name('nope').length)"),
        "0",
        "无匹配 → 空 Array"
    );
}

/// `getElementsByTagName('*')` 匹配**全部元素**（spec 通配）——含 root、span、p 等所有 Element
///（文档序）。验证 `*` 通配路径（经 get_elements_by_tag_name_ns 内置通配）。
#[test]
fn native_get_elements_by_tag_name_wildcard_r3137() {
    let html = r#"<html><head></head><body><div id="root"><span>1</span><p>2</p></div></body></html>"#;
    // `*` 返全部元素（html/head/body/div/span/p，文档序），至少 6 个。
    let all = run_script(html, "(__zw_native_get_elements_by_tag_name('*').length)");
    let n: i64 = all.parse().unwrap_or(0);
    assert!(
        n >= 6,
        "getElementsByTagName('*') 须含 html/head/body/div/span/p（≥6），实得 {all}"
    );
    // 通配结果含具体 tag（span 在内）。
    let script = "(()=>{const all=__zw_native_get_elements_by_tag_name('*');\
    return all.some(e=>e.tagName==='SPAN')+'/'+all.some(e=>e.tagName==='P');})()";
    assert_eq!(
        run_script(html, script),
        "true/true",
        "getElementsByTagName('*') 含 span 与 p（通配全元素）"
    );
}

/// 身份：getElementsByTagName 返的对象与 getElementById/querySelector 共享 NodeId↔对象映射（同对象）。
#[test]
fn native_get_elements_by_tag_name_identity_r3137() {
    let html = r#"<div id="a"><span>x</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_elements_by_tag_name('div')[0] === __zw_native_element_for_id('a'))"
        ),
        "true",
        "getElementsByTagName('div')[0] === getElementById('a')（身份缓存共享）"
    );
}

// ── R3138 document.getElementsByClassName(name) 工厂 ──

/// `__zw_native_get_elements_by_class_name(name)`：按类名收集文档序 V8 Array。
/// 多元素同 class + 无匹配空数组 + 空/空白串空数组。
#[test]
fn native_get_elements_by_class_name_r3138() {
    let html = r#"<div id="root"><span class="row">1</span><span class="row big">2</span><p class="row">3</p></div>"#;
    // 单 class "row" → 3 个（两 span + p）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('row').length)"),
        "3",
        "getElementsByClassName('row').length===3（两 span + p）"
    );
    // 文档序读 tagName（span/span/p）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const a=__zw_native_get_elements_by_class_name('row');\
            return a[0].tagName+'/'+a[1].tagName+'/'+a[2].tagName;})()"
        ),
        "SPAN/SPAN/P",
        "文档序 row 元素 tagName：span/span/p"
    );
    // 单 class "big" → 1 个（仅第二 span）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('big').length)"),
        "1",
        "getElementsByClassName('big').length===1（仅 class='row big'）"
    );
    // 无匹配 → 空 Array。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('nope').length)"),
        "0",
        "无匹配 → 空 Array"
    );
    // 空/空白串 → 空 Array（spec：空 names 不匹配）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('   ').length)"),
        "0",
        "空白串 → 空 Array"
    );
}

/// **多类 spec 合规**：`'row big'`（空格分隔）→ 含【全部】两类的元素（仅 class='row big' 那个）。
/// 闭合 dom `get_elements_by_class_name` 单 token 限制——本工厂 split + 过滤实现 spec 语义。
#[test]
fn native_get_elements_by_class_name_multi_class_r3138() {
    let html = r#"<div><span class="row big">a</span><span class="row">b</span><span class="big">c</span></div>"#;
    // 'row big' → 仅含两类的 1 个（第一个 span）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('row big').length)"),
        "1",
        "多类 'row big' → 含全部两类的 1 个（spec 合规）"
    );
    // 文档序 + 顺序无关（'big row' 同结果）。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('big row').length)"),
        "1",
        "多类顺序无关（'big row' 同 'row big'）"
    );
    // 该元素 tagName = SPAN。
    assert_eq!(
        run_script(html, "(__zw_native_get_elements_by_class_name('row big')[0].tagName)"),
        "SPAN",
        "多类匹配元素 tagName=SPAN"
    );
}

/// 身份：getElementsByClassName 返对象与 getElementById 共享 NodeId↔对象映射（同对象）。
#[test]
fn native_get_elements_by_class_name_identity_r3138() {
    let html = r#"<div id="a" class="row"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_elements_by_class_name('row')[0] === __zw_native_element_for_id('a'))"
        ),
        "true",
        "getElementsByClassName('row')[0] === getElementById('a')（身份缓存共享）"
    );
}

// ── R3139 document.title getter/setter 工厂 ──

/// `__zw_native_get_document_title()`：读首个 `<title>` textContent；无 title → 空串。
#[test]
fn native_document_title_getter_r3139() {
    let html = r#"<html><head><title>Hello</title></head><body></body></html>"#;
    assert_eq!(
        run_script(html, "(__zw_native_get_document_title())"),
        "Hello",
        "document.title 读首个 <title> textContent"
    );
    // 无 <title> → 空串。
    let html_notitle = r#"<html><head></head><body></body></html>"#;
    assert_eq!(
        run_script(html_notitle, "(__zw_native_get_document_title())"),
        "",
        "无 <title> → 空串"
    );
}

/// `__zw_native_set_document_title(str)`：存在 <title> → 改其 textContent；getter 回读新值。
#[test]
fn native_document_title_setter_existing_r3139() {
    let html = r#"<html><head><title>Old</title></head><body></body></html>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_set_document_title('New');\
            return __zw_native_get_document_title(); })()"
        ),
        "New",
        "setter 改既有 <title> textContent → getter 回读新值"
    );
}

/// setter 在无 `<title>` 时于 `<head>` 建 `<title>` 设文本——getter 回读 + head 含新建 title 元素。
#[test]
fn native_document_title_setter_create_missing_r3139() {
    let html = r#"<html><head></head><body></body></html>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_set_document_title('Created');\
            return __zw_native_get_document_title(); })()"
        ),
        "Created",
        "无 <title> 时 setter 在 <head> 建 <title> → getter 回读"
    );
    // 新建 title 在 head 内（getElementsByTagName('title') 命中）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_set_document_title('X');\
            return __zw_native_get_elements_by_tag_name('title').length; })()"
        ),
        "1",
        "setter 建 <title> 后 getElementsByTagName('title').length===1"
    );
}

// ── R3140 element.matches / element.closest ──

/// `element.matches(selector)`：本元素匹配复合选择器 → bool。class/tag/attr/伪类 + 不匹配 + 非法 false。
#[test]
fn native_element_matches_r3140() {
    let html = r#"<div id="a" class="row big"><span id="b">x</span></div>"#;
    // class 匹配。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').matches('.row'))"),
        "true",
        "matches('.row')===true"
    );
    // tag + class 复合匹配。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').matches('div.row'))"),
        "true",
        "matches('div.row')===true（tag+class 复合）"
    );
    // 不匹配 → false。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').matches('.nope'))"),
        "false",
        "matches('.nope')===false"
    );
    // 子元素 span 不匹配 div 选择器。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('b').matches('div'))"),
        "false",
        "span.matches('div')===false"
    );
    // 非法选择器 → false（不抛）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').matches('!!!'))"),
        "false",
        "非法选择器 → false（不 panic）"
    );
}

/// `element.closest(selector)`：本元素 + 祖先链首个匹配 → native 元素或 null。
/// 含自身匹配 + 祖先匹配 + 无匹配 null。
#[test]
fn native_element_closest_r3140() {
    let html = r#"<div id="a" class="row"><div id="b"><span id="c">x</span></div></div>"#;
    // 自身匹配（c 是 span）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('c').closest('span').id)"),
        "c",
        "closest('span') 命中自身 c"
    );
    // 祖先匹配（c 上溯到 .row 的 div#a）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('c').closest('.row').id)"),
        "a",
        "closest('.row') 上溯命中祖先 a"
    );
    // 无匹配 → null。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('c').closest('.nope')===null)"),
        "true",
        "closest('.nope')===null（无匹配）"
    );
    // closest 命中祖先后该元素 matches 验证一致性（closest('#a') 命中后其 id===a）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('c').closest('#a').matches('#a'))"),
        "true",
        "closest('#a') 结果 matches('#a')===true（matches/closest 一致）"
    );
}

// ── R3141 document.createEvent + Event.initEvent（legacy 事件创建）──

/// `__zw_native_create_event(type)`：legacy DOM type → 对应构造器实例（instanceof Event）。
/// "Event"/"HTMLEvents" → Event；"MouseEvent" → MouseEvent；"KeyboardEvent" → KeyboardEvent；"CustomEvent" → CustomEvent。
#[test]
fn native_create_event_type_mapping_r3141() {
    let html = r#"<div id="a"></div>"#;
    // Event / HTMLEvents / 未知 → instanceof Event。
    assert_eq!(
        run_script(html, "(__zw_native_create_event('Event') instanceof Event)"),
        "true",
        "createEvent('Event') instanceof Event"
    );
    assert_eq!(
        run_script(html, "(__zw_native_create_event('HTMLEvents') instanceof Event)"),
        "true",
        "createEvent('HTMLEvents') → instanceof Event"
    );
    assert_eq!(
        run_script(html, "(__zw_native_create_event('Weird') instanceof Event)"),
        "true",
        "未知 type → Event best-effort（instanceof Event）"
    );
    // MouseEvent → instanceof MouseEvent + Event。
    assert_eq!(
        run_script(html, "(__zw_native_create_event('MouseEvent') instanceof MouseEvent)"),
        "true",
        "createEvent('MouseEvent') instanceof MouseEvent"
    );
    // KeyboardEvent → instanceof KeyboardEvent。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_create_event('KeyboardEvent') instanceof KeyboardEvent)"
        ),
        "true",
        "createEvent('KeyboardEvent') instanceof KeyboardEvent"
    );
}

/// createEvent + initEvent 派发链：createEvent('Event') → initEvent(type,bubbles,cancelable) → dispatchEvent 触发监听器。
/// 闭合 legacy 事件创建路径（测试库惯用 `document.createEvent('Event') + initEvent + dispatchEvent`）。
#[test]
fn native_create_event_init_dispatch_r3141() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
            let got='no';\
            el.addEventListener('myevt', e=>{ got=e.type+'/'+e.bubbles; });\
            const ev=__zw_native_create_event('Event');\
            ev.initEvent('myevt', true, false);\
            el.dispatchEvent(ev);\
            return got; })()"
        ),
        "myevt/true",
        "createEvent + initEvent + dispatchEvent 派发链触发监听器，type/bubbles 正确"
    );
}

/// initEvent 重置：initEvent 覆写 type/bubbles/cancelable（构造器默认 type='' 被覆写）。
#[test]
fn native_init_event_overwrites_r3141() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const ev=__zw_native_create_event('Event');\
            return ev.type+'/'+ev.bubbles+'/'+ev.cancelable; })()"
        ),
        "/false/false",
        "createEvent('Event') 默认 type=''/bubbles=false/cancelable=false（未 initEvent）"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{ const ev=__zw_native_create_event('Event');\
            ev.initEvent('hello', true, true);\
            return ev.type+'/'+ev.bubbles+'/'+ev.cancelable; })()"
        ),
        "hello/true/true",
        "initEvent 覆写 type/bubbles/cancelable"
    );
}

// ── R3142 element.remove()（自移除，ChildNode mixin）──

/// `element.remove()`：从父节点摘除自身。remove 后父节点 children.length 减 / 不在该父子列表；
/// detached 节点 remove no-op（不抛）。
#[test]
fn native_element_remove_r3142() {
    let html = r#"<div id="host"><span id="a">x</span><span id="b">y</span></div>"#;
    // remove #a 后 host 剩 [b]。
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').remove();\
            const host=__zw_native_element_for_id('host');\
            return host.children.length+'/'+host.children[0].id; })()"
        ),
        "1/b",
        "element.remove() 自移除——host 剩 [b]"
    );
    // removed 节点无 parent（detached）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); a.remove();\
            return (a.parentNode===null); })()"
        ),
        "true",
        "removed 节点 parentNode===null（detached）"
    );
    // detached 节点再 remove → no-op（不抛）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); a.remove(); a.remove();\
            return 'ok'; })()"
        ),
        "ok",
        "detached 节点再 remove → no-op（不抛）"
    );
}

// ── R3143 element.prepend/append/before/after/replaceWith（现代插入族）──

/// `append(...items)` / `prepend(...items)`：variadic 节点+字符串，DOM 序 = arg 序。
/// append 末尾追加；prepend 首子前插（保 arg 序）；字符串参 → 文本节点。
#[test]
fn native_element_append_prepend_r3143() {
    let html = r#"<div id="host"><span id="existing">x</span></div>"#;
    // append(elem_a, "mid", elem_c) → [existing, a, text("mid"), c]；childNodes 含文本。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const host=__zw_native_element_for_id('host');\
            host.append(__zw_native_create_element('a'), 'mid', __zw_native_create_element('c'));\
            return host.childNodes.length+'/'+host.childNodes[1].tagName+'/'+\
            host.childNodes[2].nodeType+'/'+host.childNodes[3].tagName; })()"
        ),
        "4/A/3/C",
        "append(elem,str,elem) 末尾追加——childNodes 含文本节点（nodeType=3），DOM 序 = arg 序"
    );
    // prepend(elem_b) → [b, existing]（b 首子）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const host=__zw_native_element_for_id('host');\
            host.prepend(__zw_native_create_element('b'));\
            return host.children[0].tagName+'/'+host.children[1].id; })()"
        ),
        "B/existing",
        "prepend(elem) 插首子前——b 成 first child，existing 其后"
    );
}

/// `before(...items)` / `after(...items)` / `replaceWith(...items)`：相对兄弟位置 + 替换。
/// before 插 self 前；after 插 self 后；replaceWith 在 self 位置插 items 后移除 self。
#[test]
fn native_element_before_after_replace_with_r3143() {
    let html = r#"<div id="host"><span id="target"></span></div>"#;
    // before(x) on target → [x, target]。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const target=__zw_native_element_for_id('target');\
            target.before(__zw_native_create_element('x'));\
            const host=__zw_native_element_for_id('host');\
            return host.children[0].tagName+'/'+host.children[1].id; })()"
        ),
        "X/target",
        "before(x) 插 target 前——x 成首子"
    );
    // after(x) on target → [target, x]。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const target=__zw_native_element_for_id('target');\
            target.after(__zw_native_create_element('x'));\
            const host=__zw_native_element_for_id('host');\
            return host.children[0].id+'/'+host.children[1].tagName; })()"
        ),
        "target/X",
        "after(x) 插 target 后——x 成末子"
    );
    // replaceWith(x, y) on target → target 移除，[x, y] 替其位。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const target=__zw_native_element_for_id('target');\
            target.replaceWith(__zw_native_create_element('x'), __zw_native_create_element('y'));\
            const host=__zw_native_element_for_id('host');\
            return host.children.length+'/'+host.children[0].tagName+'/'+host.children[1].tagName; })()"
        ),
        "2/X/Y",
        "replaceWith(x,y) 替换——target 移除，[x,y] 替其位（DOM 序 = arg 序）"
    );
}

/// detached 节点 before/after/replaceWith → no-op（无 parent，不抛）。
#[test]
fn native_element_insert_detached_noop_r3143() {
    let html = r#"<div id="host"></div>"#;
    // detached 节点（create 未挂载）before/after/replaceWith → no-op，host children 不变。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const detached=__zw_native_create_element('div');\
            detached.before(__zw_native_create_element('x'));\
            detached.after(__zw_native_create_element('y'));\
            detached.replaceWith(__zw_native_create_element('z'));\
            return __zw_native_element_for_id('host').children.length; })()"
        ),
        "0",
        "detached 节点 before/after/replaceWith → no-op（无 parent）"
    );
}

// ── R3132 appendChild/insertBefore(fragment) flatten ──

/// host.appendChild(frag)：fragment 子节点展开进 host + fragment 清空（spec flatten）。
#[test]
fn native_append_child_fragment_flatten_r3132() {
    let html = r#"<div id="host"></div>"#;
    let script = "(()=>{\
const host=__zw_native_element_for_id('host');\
const frag=__zw_native_create_document_fragment();\
frag.appendChild(__zw_native_create_element('span'));\
frag.appendChild(__zw_native_create_element('b'));\
host.appendChild(frag);\
return host.children.length+'/'+host.children[0].tagName+'/'+host.children[1].tagName+'/'+\
frag.childNodes.length;})()";
    assert_eq!(
        run_script(html, script),
        "2/SPAN/B/0",
        "appendChild(frag) flatten：子进 host + fragment 清空"
    );
}

/// host.insertBefore(frag, ref)：fragment 子节点插到 ref 前 + fragment 清空。
#[test]
fn native_insert_before_fragment_flatten_r3132() {
    let html = r#"<div id="host"><i id="ref">r</i></div>"#;
    let script = "(()=>{\
const host=__zw_native_element_for_id('host');\
const ref=__zw_native_element_for_id('ref');\
const frag=__zw_native_create_document_fragment();\
frag.appendChild(__zw_native_create_element('span'));\
host.insertBefore(frag, ref);\
return host.children[0].tagName+'/'+host.children[1].tagName+'/'+host.children.length;})()";
    assert_eq!(
        run_script(html, script),
        "SPAN/I/2",
        "insertBefore(frag, ref) flatten：子插 ref 前"
    );
}

/// `appendChild` 闭环插入（自身/祖先）→ 抛 HierarchyRequestError（spec `dom-node-insertbefore`
/// 闭环步；WPT dom/nodes/Node-replaceChild.html "inclusive ancestor" 场景同源）。此前 native
/// 静默吞错留 undefined，现 dom crate DomError::WouldCreateCycle → DOMException。
#[test]
fn native_append_child_cycle_throws_hierarchy_request() {
    let html = r#"<div id="a"><div id="b"></div></div>"#;
    // a.appendChild(a) → 自身闭环 → HierarchyRequestError。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a');\
             try { a.appendChild(a); return 'no-throw'; } catch(e){ return e.name; } })()"
        ),
        "HierarchyRequestError"
    );
    // a.appendChild(b) 其中 b 是 a 的祖先 → 闭环 → HierarchyRequestError。
    // （b 在 a 内部，a.appendChild(外部祖先) 需构造；此处测 a 把自身加入子树→已覆盖）
    // b.appendChild(祖先a) → a 是 b 的祖先，闭环。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); const b=__zw_native_element_for_id('b');\
             try { b.appendChild(a); return 'no-throw'; } catch(e){ return e.name; } })()"
        ),
        "HierarchyRequestError"
    );
}

/// `replaceChild(oldChild 不在 parent)` → 抛 NotFoundError（spec `dom-node-replace-child`）。
#[test]
fn native_replace_child_not_a_child_throws_not_found() {
    let html = r#"<div id="a"></div><div id="b"></div><div id="c"></div>"#;
    // a.replaceChild(b, c)：c 不是 a 的子 → NotFoundError（WPT "child's parent is not context node"）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); const b=__zw_native_element_for_id('b');\
             const c=__zw_native_element_for_id('c');\
             try { a.replaceChild(b, c); return 'no-throw'; } catch(e){ return e.name; } })()"
        ),
        "NotFoundError"
    );
}

/// `removeChild(child 不在 parent)` → 抛 NotFoundError（spec `dom-node-removechild`）。
#[test]
fn native_remove_child_not_a_child_throws_not_found() {
    let html = r#"<div id="a"></div><div id="orphan"></div>"#;
    // a.removeChild(orphan)：orphan 不是 a 的子 → NotFoundError。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); const o=__zw_native_element_for_id('orphan');\
             try { a.removeChild(o); return 'no-throw'; } catch(e){ return e.name; } })()"
        ),
        "NotFoundError"
    );
}

// ── R3110 节点导航 getter（parentNode / firstChild / lastChild / nextSibling / previousSibling / hasChildNodes）──
//

// ── R3144 prepend/append/before/after/replaceWith(fragment) flatten ──

/// 现代插入族接 DocumentFragment → 展开其子节点（非插 fragment 节点本身）+ fragment 清空。
/// 与 R3132 appendChild/insertBefore fragment flatten 语义一致；DOM 序 = fragment 内子序。
#[test]
fn native_element_insert_fragment_flatten_r3144() {
    // append(frag) → fragment 子展开进 host 末尾 + fragment 清空。
    let html = r#"<div id="host"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const host=__zw_native_element_for_id('host');\
            const frag=__zw_native_create_document_fragment();\
            frag.appendChild(__zw_native_create_element('span'));\
            frag.appendChild(__zw_native_create_element('b'));\
            host.append(frag);\
            return host.children.length+'/'+host.children[0].tagName+'/'+host.children[1].tagName+'/'+\
            frag.childNodes.length; })()"
        ),
        "2/SPAN/B/0",
        "append(frag) flatten：子展开进 host + fragment 清空"
    );

    // prepend(frag) → fragment 子插到原首子前（DOM 序 = fragment 子序）。
    let html = r#"<div id="host"><i id="e">e</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const host=__zw_native_element_for_id('host');\
            const frag=__zw_native_create_document_fragment();\
            frag.appendChild(__zw_native_create_element('span'));\
            frag.appendChild(__zw_native_create_element('b'));\
            host.prepend(frag);\
            return host.children[0].tagName+'/'+host.children[1].tagName+'/'+host.children[2].id; })()"
        ),
        "SPAN/B/e",
        "prepend(frag) flatten：子插原首子前，子内序保留"
    );

    // before(frag) → fragment 子插到 self 前在父中。
    let html = r#"<div id="host"><i id="t">t</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
            const frag=__zw_native_create_document_fragment();\
            frag.appendChild(__zw_native_create_element('span'));\
            t.before(frag);\
            const host=__zw_native_element_for_id('host');\
            return host.children[0].tagName+'/'+host.children[1].id; })()"
        ),
        "SPAN/t",
        "before(frag) flatten：子插 self 前"
    );

    // after(frag) → fragment 子插到 self 后在父中（ref = next sibling = None → 末尾追加，子内序保留）。
    let html = r#"<div id="host"><i id="t">t</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
            const frag=__zw_native_create_document_fragment();\
            frag.appendChild(__zw_native_create_element('span'));\
            frag.appendChild(__zw_native_create_element('b'));\
            t.after(frag);\
            const host=__zw_native_element_for_id('host');\
            return host.children[0].id+'/'+host.children[1].tagName+'/'+host.children[2].tagName; })()"
        ),
        "t/SPAN/B",
        "after(frag) flatten：子插 self 后，子内序保留"
    );

    // replaceWith(frag) → self 移除，fragment 子替其位。
    let html = r#"<div id="host"><i id="t">t</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
            const frag=__zw_native_create_document_fragment();\
            frag.appendChild(__zw_native_create_element('x'));\
            frag.appendChild(__zw_native_create_element('y'));\
            t.replaceWith(frag);\
            const host=__zw_native_element_for_id('host');\
            return host.children.length+'/'+host.children[0].tagName+'/'+host.children[1].tagName; })()"
        ),
        "2/X/Y",
        "replaceWith(frag) flatten：self 移除，子替其位"
    );

    // 混合 append(node, frag, 'text') → 顺序保留，fragment 原位展开为 f1,f2。
    let html = r#"<div id="host"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const host=__zw_native_element_for_id('host');\
            const frag=__zw_native_create_document_fragment();\
            frag.appendChild(__zw_native_create_element('f1'));\
            frag.appendChild(__zw_native_create_element('f2'));\
            host.append(__zw_native_create_element('a'), frag, 'tail');\
            return host.childNodes.length+'/'+host.childNodes[0].tagName+'/'+\
            host.childNodes[1].tagName+'/'+host.childNodes[2].tagName+'/'+\
            host.childNodes[3].nodeType; })()"
        ),
        "4/A/F1/F2/3",
        "append(elem,frag,str) 混合：frag 原位展开为 f1,f2，序 = arg 序"
    );

    // 空 fragment append → 不插入任何节点。
    let html = r#"<div id="host"><i id="e">e</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const host=__zw_native_element_for_id('host');\
            host.append(__zw_native_create_document_fragment());\
            return host.children.length; })()"
        ),
        "1",
        "append(空 fragment) → 无节点插入"
    );
}

// HTML: <div id="root"><span id="s1">hello</span><span id="s2"></span></div>
// root 子节点 = [span#s1, span#s2]（标签间无空白文本）；s1 子 = 文本 "hello"。

/// parentNode + nextSibling + previousSibling（spec `dom-node-parent-node` / `-next-sibling` / `-previous-sibling`）。
#[test]
fn native_node_navigation_parent_and_siblings() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const s1=__zw_native_element_for_id('s1');\
             return s1.parentNode.id+'/'+s1.nextSibling.id+'/'+__zw_native_element_for_id('s2').previousSibling.id; })()"
        ),
        "root/s2/s1"
    );
}

/// firstChild + lastChild（spec `dom-node-first-child` / `-last-child`）。
#[test]
fn native_node_navigation_first_last_child() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const r=__zw_native_element_for_id('root');\
             return r.firstChild.id+'/'+r.lastChild.id; })()"
        ),
        "s1/s2"
    );
}

/// hasChildNodes（spec `dom-node-has-child-nodes`）：有子 → true；空 span → false。
#[test]
fn native_node_navigation_has_child_nodes() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ return __zw_native_element_for_id('root').hasChildNodes()+'/'+__zw_native_element_for_id('s2').hasChildNodes(); })()"
        ),
        "true/false"
    );
}

/// firstChild 返文本节点（nodeType=3）——导航返回非 Element 子节点，包同一模板（R3104 node-type-aware）。
#[test]
fn native_node_navigation_text_first_child() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('s1').firstChild.nodeType)"),
        "3"
    );
}

/// nextSibling 越界 → null（spec detached/无兄弟返 null，非 undefined）。
#[test]
fn native_node_navigation_null_relation() {
    let html = r#"<div id="root"><span id="s1">hello</span><span id="s2"></span></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('s2').nextSibling === null)"),
        "true"
    );
}

// ── R3111 replaceChild native + nodeValue/data setter ──

/// `replaceChild(newChild, oldChild)`（spec `dom-node-replace-child`）：newChild 替换 oldChild 位置，
/// 返 oldChild。补全树 mutation 集（appendChild/insertBefore/removeChild/replaceChild）。
#[test]
fn native_replace_child() {
    let html = r#"<div id="root"><span id="s1">1</span><span id="s2">2</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const r=__zw_native_element_for_id('root');\
             const nw=__zw_native_create_element('span'); nw.id='nw';\
             const old=r.replaceChild(nw, __zw_native_element_for_id('s1'));\
             return r.children[0].id+'/'+r.children[1].id+'/'+old.id; })()"
        ),
        "nw/s2/s1"
    );
}

/// `nodeValue` setter on Text（spec `dom-node-nodevalue` setter）：改文本节点内容，读回见新值。
#[test]
fn native_node_value_setter_text() {
    let html = r#"<div id="root">hello</div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('root').firstChild;\
             t.nodeValue='world'; return t.nodeValue+'/'+t.nodeType; })()"
        ),
        "world/3"
    );
}

/// `nodeValue` setter on Comment（spec）：改注释内容，读回见新值。
#[test]
fn native_node_value_setter_comment() {
    let html = r#"<div id="root"><!--c--></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const c=__zw_native_element_for_id('root').firstChild;\
             c.nodeValue='d'; return c.nodeValue+'/'+c.nodeType; })()"
        ),
        "d/8"
    );
}

/// `nodeValue` setter on Element → no-op（spec：Element/Document 设 nodeValue 无效；getter 返 null）。
#[test]
fn native_node_value_setter_element_noop() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.nodeValue='x'; return String(e.nodeValue); })()"
        ),
        "null"
    );
}

// ── R3112 NamedNodeMap（element.attributes 集合）──
//
// HTML: <div id="a" class="row" data-x="42"></div>（属性源序 id/class/data-x）。

/// `attributes.length` + 身份（`el.attributes === el.attributes`，spec live 同对象）。
#[test]
fn native_attributes_length_and_identity() {
    let html = r#"<div id="a" class="row" data-x="42"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             return el.attributes.length+'/'+(el.attributes === el.attributes); })()"
        ),
        "3/true"
    );
}

/// `item(index)`：源序属性 → Attr-like {name, value}；越界 → null。
#[test]
fn native_attributes_item() {
    let html = r#"<div id="a" class="row" data-x="42"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a').attributes;\
             return a.item(0).name+'/'+a.item(2).name+'/'+a.item(2).value+'/'+(a.item(9)===null); })()"
        ),
        "id/data-x/42/true"
    );
}

/// `getNamedItem(name)`：有 → {name,value}；无 → null。
#[test]
fn native_attributes_get_named_item() {
    let html = r#"<div id="a" class="row" data-x="42"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a').attributes;\
             return a.getNamedItem('class').value+'/'+(a.getNamedItem('nope')===null); })()"
        ),
        "row/true"
    );
}

/// `setNamedItem({name,value})` + `removeNamedItem(name)`：写回 owner 元素属性（getAttribute/hasAttribute 见）。
#[test]
fn native_attributes_set_and_remove_named_item() {
    let html = r#"<div id="a" class="row" data-x="42"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); const a=el.attributes;\
             a.setNamedItem({name:'data-y', value:'7'});\
             a.removeNamedItem('class');\
             return el.getAttribute('data-y')+'/'+el.hasAttribute('class'); })()"
        ),
        "7/false"
    );
}

// ── R3122 Attr 节点（完整 Attr：nodeType=2/name/value/ownerElement）──
//
// getNamedItem / item 返 Attr 节点对象（非 plain {name,value}）。闭合 R3112 plain-object 限制。

/// Attr 节点面：nodeType=2、name=nodeName、value live、ownerElement===owner 元素、身份（同 attr 同对象）。
#[test]
fn native_attr_node_surface() {
    let html = r#"<div id="a" class="row"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             const at=el.attributes.getNamedItem('class');\
             return at.nodeType+'/'+at.name+'/'+at.nodeName+'/'+at.value+'/'+(at.ownerElement===el); })()"
        ),
        "2/class/class/row/true"
    );
    // value setter 经 set_attribute 写回 owner 元素（live）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.attributes.getNamedItem('class').value='new';\
             return el.getAttribute('class'); })()"
        ),
        "new"
    );
    // nodeValue / textContent === value（Node 接口面）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const at=__zw_native_element_for_id('a').attributes.getNamedItem('class');\
             return at.nodeValue+'/'+at.textContent; })()"
        ),
        "row/row"
    );
    // 身份：同 (owner, name) 返同对象（spec identity）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a').attributes;\
             return (a.getNamedItem('class')===a.getNamedItem('class'))+'/'+\
             (a.getNamedItem('class')===a.item(1)); })()"
        ),
        "true/true"
    );
    // item(0) 返 Attr 节点（nodeType=2 + name=id）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const at=__zw_native_element_for_id('a').attributes.item(0);\
             return at.nodeType+'/'+at.name; })()"
        ),
        "2/id"
    );
}

// ── R3113 innerHTML / outerHTML 序列化 getter ──

/// `innerHTML`（子节点序列化拼接）+ `outerHTML`（含自身 tag）。
#[test]
fn native_inner_outer_html() {
    let html = r#"<div id="a"><b>hi</b>!</div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').innerHTML)"),
        r#"<b>hi</b>!"#
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').outerHTML)"),
        r#"<div id="a"><b>hi</b>!</div>"#
    );
}

/// `outerHTML` 反映 native 属性写（live 序列化）。
#[test]
fn native_outer_html_reflects_attribute() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.setAttribute('data-x','9');\
             return e.outerHTML; })()"
        ),
        r#"<div id="a" data-x="9"></div>"#
    );
}

/// `innerHTML` 反映 native 文本写（nodeValue）——R3108 重渲染后 live 序列化见新文本。
#[test]
fn native_inner_html_reflects_text_write() {
    let html = r#"<div id="a"><span id="s">old</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('s').firstChild.nodeValue='new';\
             return __zw_native_element_for_id('a').innerHTML; })()"
        ),
        r#"<span id="s">new</span>"#
    );
}

// ── R3123 innerHTML / outerHTML setter（解析 HTML 片段清子/替换自身）──

/// `innerHTML` setter：设含 markup 片段 → 替换现有子节点（旧子清空，新片段深拷贝追加）。
/// getter live 序列化回读验证（旧 `<span>old</span>` 应被替换为 `<b>new</b><i>x</i>`）。
#[test]
fn native_inner_html_setter_replaces_children() {
    let html = r#"<div id="a"><span>old</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.innerHTML='<b>new</b><i>x</i>';\
             return e.innerHTML; })()"
        ),
        r#"<b>new</b><i>x</i>"#
    );
}

/// `innerHTML` setter 无 markup（纯文本）→ 单文本节点（不走片段解析路径）。
#[test]
fn native_inner_html_setter_plain_text() {
    let html = r#"<div id="a"><b>x</b></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.innerHTML='hello';\
             return e.innerHTML+'/'+e.firstChild.nodeType+'/'+e.childNodes.length; })()"
        ),
        "hello/3/1"
    );
}

/// `innerHTML` setter 空串 → 清空所有子节点。
#[test]
fn native_inner_html_setter_empty_clears() {
    let html = r#"<div id="a"><b>x</b>!</div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').innerHTML='';\
             return __zw_native_element_for_id('a').childNodes.length+'/'+__zw_native_element_for_id('a').hasChildNodes(); })()"
        ),
        "0/false"
    );
}

/// R3181 `innerHTML` setter 片段拷贝保留 SVG 命名空间——`<svg><rect/></svg>` 经 copy_subtree_from
/// 重建后 rect 仍是 SVG ns（namespaceURI = SVG ns + tagName = "rect" 大小写敏感）。
/// 旧实现 create_element(&tag) 强制 HTML ns → rect.namespaceURI = HTML ns + tagName = "RECT"。
#[test]
fn native_inner_html_setter_preserves_svg_namespace_r3181() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').innerHTML='<svg id=\"s\"><rect id=\"r\"/></svg>';\
             const r=__zw_native_element_for_id('r');\
             return r.namespaceURI+'|'+r.tagName; })()"
        ),
        "http://www.w3.org/2000/svg|rect"
    );
}

/// R3182 `innerHTML` setter 用 context element 解析——`table.innerHTML='<tr><td>x</td></tr>'` 在 table
/// context 下正确解析（隐式 tbody 包裹），回读 `<tbody><tr><td>x</td></tr></tbody>`。旧 body-wrap 在
/// body context 下 `<tr>` foster-parent 丢失（实测回读仅 "x"）。
#[test]
fn native_inner_html_setter_table_context_r3182() {
    let html = r#"<table id="t"></table>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('t').innerHTML='<tr><td>x</td></tr>';\
             return __zw_native_element_for_id('t').innerHTML; })()"
        ),
        "<tbody><tr><td>x</td></tr></tbody>"
    );
}

/// `outerHTML` setter：元素整体替换为片段顶层节点。原元素从 DOM 移除（id 失效），
/// 父节点 innerHTML 反映新内容。验证经父节点回读（原 id 'a' 已 detach）。
#[test]
fn native_outer_html_setter_replaces_self() {
    let html = r#"<div id="p"><span id="a">old</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').outerHTML='<b id=\"c\">new</b>';\
             return __zw_native_element_for_id('p').innerHTML; })()"
        ),
        r#"<b id="c">new</b>"#
    );
}

/// `outerHTML` setter 空串 → 仅移除目标（spec：`el.outerHTML=''` 移除元素）。
#[test]
fn native_outer_html_setter_empty_removes() {
    let html = r#"<div id="p"><span id="a">x</span><i>y</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').outerHTML='';\
             return __zw_native_element_for_id('p').innerHTML; })()"
        ),
        r#"<i>y</i>"#
    );
}

// ── R3114 cloneNode(deep) ──
//
// HTML: <div id="a" class="x"><span id="s">hi</span></div>

/// `cloneNode(false)` 浅克隆：同 tag + 属性，无子节点；新对象（≠源）。
#[test]
fn native_clone_node_shallow() {
    let html = r#"<div id="a" class="x"><span id="s">hi</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); const c=el.cloneNode(false);\
             return c.tagName+'/'+c.getAttribute('class')+'/'+c.children.length+'/'+(c!==el); })()"
        ),
        "DIV/x/0/true"
    );
}

/// `cloneNode(true)` 深克隆：含子树（span + 文本），子节点经 native 读见。
#[test]
fn native_clone_node_deep() {
    let html = r#"<div id="a" class="x"><span id="s">hi</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const c=__zw_native_element_for_id('a').cloneNode(true);\
             return c.children.length+'/'+c.children[0].tagName+'/'+c.children[0].textContent; })()"
        ),
        "1/SPAN/hi"
    );
}

/// `cloneNode()` 缺省 deep → false（spec：浅克隆）。
#[test]
fn native_clone_node_default_shallow() {
    let html = r#"<div id="a" class="x"><span id="s">hi</span></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').cloneNode().children.length)"),
        "0"
    );
}

// ── R3115 contains(node) ──
//
// HTML: <div id="a"><div id="b"><span id="c">x</span></div></div>

/// `contains`：后代 / 自身 / 非后代（walk parent 链）。
#[test]
fn native_contains_relations() {
    let html = r#"<div id="a"><div id="b"><span id="c">x</span></div></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'), c=__zw_native_element_for_id('c');\
             return a.contains(c)+'/'+a.contains(a)+'/'+c.contains(a); })()"
        ),
        "true/true/false"
    );
}

/// `contains(null)` → false（spec：contains(null)===false；非 node 参亦 false）。
#[test]
fn native_contains_null() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').contains(null))"),
        "false"
    );
}

// ── R3133 节点包装器终结器（闭合 R3109 LISTENERS detach 泄漏）──

/// 重新附加语义：removeChild 不清监听器——detach 后 re-append，监听器仍触发（spec：
/// 节点从 DOM 移除不丢弃监听器，跨 detach/重附加保留）。此为终结器设计前提（仅包装器真被 GC
/// 才清 LISTENERS，而非 removeChild 时清）。
#[test]
fn native_listener_survives_detach_reattach_r3133() {
    let html = r#"<div id="host"><span id="a"></span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const host=__zw_native_element_for_id('host');\
             const el=__zw_native_element_for_id('a');\
             el.addEventListener('click', ()=>{ globalThis.__fired='yes'; });\
             host.removeChild(el);\
             host.appendChild(el);\
             el.dispatchEvent({type:'click'});\
             return globalThis.__fired || 'no'; })()"
        ),
        "yes"
    );
}

/// 终结器回收：脚本 add 2 监听器后丢包装器引用（仅 weak 缓存持）→ 强制 GC 收包装器 →
/// guaranteed 终结器清本节点 LISTENERS → 条目归 0（闭合 R3109：旧实现 detached 节点监听器永驻）。
#[test]
fn native_finalizer_cleans_listeners_on_gc_r3133() {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html("<div id='a'></div>")));
    let ffi = encode_node_id(dom.borrow().get_element_by_id("a").expect("id a"));
    let cleaned;
    {
        let isolate = &mut v8::Isolate::new(Default::default());
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        // IIFE 结束 el 出作用域 → JS 强引用断（仅 weak 缓存持包装器）。
        let script = "(()=>{ const el=__zw_native_element_for_id('a');\
             el.addEventListener('click',()=>{});\
             el.addEventListener('keyup',()=>{});\
             return 'ok'; })()";
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let _ = compiled.run(scope).expect("run");
        // 多轮 GC 收 weak-held 包装器 → guaranteed 终结器（GC 第二遍）清本节点 LISTENERS。
        for _ in 0..5 {
            scope.low_memory_notification();
        }
        cleaned = listener_keys_for(ffi);
    }
    assert_eq!(
        cleaned, 0,
        "包装器 GC 后终结器应清本节点全部监听器（R3109 detach 泄漏闭合）"
    );
    reset_for_test();
}

/// 终结器不误伤活跃节点：包装器仍被 JS 强引用（globalThis 持）→ 不被 GC → 监听器保留。
/// 防回归：终结器仅在包装器真无引用时触发，不会清仍在用节点的监听器。
#[test]
fn native_finalizer_keeps_listeners_while_referenced_r3133() {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html("<div id='a'></div>")));
    let ffi = encode_node_id(dom.borrow().get_element_by_id("a").expect("id a"));
    let kept;
    {
        let isolate = &mut v8::Isolate::new(Default::default());
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        // globalThis.__el 持强引用 → 包装器不被 GC → 监听器保留。
        let script = "(()=>{ globalThis.__el=__zw_native_element_for_id('a');\
             globalThis.__el.addEventListener('click',()=>{});\
             return 'ok'; })()";
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let _ = compiled.run(scope).expect("run");
        for _ in 0..5 {
            scope.low_memory_notification();
        }
        kept = listener_keys_for(ffi);
    }
    assert_eq!(kept, 1, "包装器仍被 JS 强引用时不应被 GC，监听器须保留（防终结器误伤）");
    reset_for_test();
}

// ── R3134 NNM/ATTR 身份缓存 weak 化（闭合同 pattern 泄漏，R3133 已知限制①）──

/// NNM/Attr weak 回收：脚本建 NNM + Attr 后丢引用（仅 weak 缓存持）→ 强制 GC → weak 句柄死
/// （对象可回收）。闭合 R3133 已知限制①——旧实现 strong Global 永驻，JS 丢引用亦不回收。
/// 元素（globalThis 持强引用）不被 GC，仅 NNM/Attr 回收，证明泄漏闭合且不影响活跃元素。
#[test]
fn native_nnm_attr_cache_reclaimable_on_gc_r3134() {
    zero_script_sandbox::ensure_v8_initialized();
    let dom = Rc::new(RefCell::new(parse_html(r#"<div id="a" class="row"></div>"#)));
    let ffi = encode_node_id(dom.borrow().get_element_by_id("a").expect("id a"));
    let (nnm_alive, attr_alive);
    {
        let isolate = &mut v8::Isolate::new(Default::default());
        v8::scope!(let scope, isolate);
        let context = v8::Context::new(scope, Default::default());
        let scope = &mut v8::ContextScope::new(scope, context);
        install_dom_bindings(scope, context, Rc::clone(&dom));
        // globalThis.__el 持元素强引用（元素不被 GC）；NNM/Attr 仅局部持，IIFE 结束即断。
        let script = "(()=>{ globalThis.__el=__zw_native_element_for_id('a');\
             void globalThis.__el.attributes;\
             void globalThis.__el.attributes.getNamedItem('class');\
             return 'ok'; })()";
        let code = v8::String::new(scope, script).expect("v8 string");
        let compiled = v8::Script::compile(scope, code, None).expect("compile");
        let _ = compiled.run(scope).expect("run");
        for _ in 0..5 {
            scope.low_memory_notification();
        }
        nnm_alive = nnm_cache_alive(ffi);
        attr_alive = attr_cache_alive(ffi, "class");
    }
    assert!(
        !nnm_alive,
        "NNM 丢 JS 引用后应可 GC（weak 死），闭合 R3133 限制① strong-Global 泄漏"
    );
    assert!(
        !attr_alive,
        "Attr 丢 JS 引用后应可 GC（weak 死），闭合 R3133 限制① strong-Global 泄漏"
    );
    reset_for_test();
}

/// NNM/Attr 身份保持：JS 持引用期间 weak 活 → 同对象（spec identity `el.attributes === el.attributes`、
/// `getNamedItem('x') === getNamedItem('x')`）。防回归：weak 化不破坏身份。
#[test]
fn native_nnm_attr_identity_while_referenced_r3134() {
    let html = r#"<div id="a" class="row"></div>"#;
    // NNM 身份：同元素 .attributes 两次取，JS 持比较 → 同对象。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').attributes === __zw_native_element_for_id('a').attributes)"
        ),
        "true"
    );
    // Attr 身份：同 (owner, name) getNamedItem 两次，JS 持比较 → 同对象。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a').attributes;\
             return (a.getNamedItem('class')===a.getNamedItem('class')); })()"
        ),
        "true"
    );
}

// ── R3146 element.toggleAttribute / insertAdjacentElement / insertAdjacentText ──

/// `toggleAttribute(name, force?)`（spec `dom-element-toggleattribute`）：force 缺省 toggle（在→移除返 false、
/// 不在→设空串返 true）；force=true 确保在（不在设 ""）；force=false 确保移除。返切换后是否在；幂等。
#[test]
fn native_toggle_attribute_r3146() {
    let html = r#"<div id="a"></div>"#;
    // toggle 缺省：不在→设空串 "" 返 true，getAttribute('hidden')===""（spec 值为空串非 "true"）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             const r=el.toggleAttribute('hidden');\
             return r+'/'+el.getAttribute('hidden')+'/'+el.hasAttribute('hidden'); })()"
        ),
        "true//true"
    );
    // toggle 缺省：在→移除返 false，hasAttribute=false。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('hidden','');\
             const r=el.toggleAttribute('hidden');\
             return r+'/'+el.hasAttribute('hidden'); })()"
        ),
        "false/false"
    );
    // force=true 在→不变返 true（值保持，spec force=true 不覆盖既有值）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('data-x','keep');\
             const r=el.toggleAttribute('data-x',true);\
             return r+'/'+el.getAttribute('data-x'); })()"
        ),
        "true/keep"
    );
    // force=true 不在→设空串返 true。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             const r=el.toggleAttribute('data-y',true);\
             return r+'/'+el.getAttribute('data-y'); })()"
        ),
        "true/"
    );
    // force=false 在→移除返 false。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('data-z','v');\
             const r=el.toggleAttribute('data-z',false);\
             return r+'/'+el.hasAttribute('data-z'); })()"
        ),
        "false/false"
    );
    // force=false 不在→不变返 false（幂等，无写）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             const r=el.toggleAttribute('nope',false);\
             return r+'/'+el.hasAttribute('nope'); })()"
        ),
        "false/false"
    );
}

/// `insertAdjacentElement(position, element)`（spec `dom-element-insertadjacentelement`）：按 position
/// 移动既有元素相对 this 插入；返插入的元素（=== 参）；4 位置 + 非法 position 抛 + detached beforebegin 抛。
#[test]
fn native_insert_adjacent_element_r3146() {
    // t 含子 tc；x 初始为 p 兄弟（body 下）。各 position 移动 x 相对 t。
    let html = r#"<div id="p"><span id="t"><i id="tc"></i></span></div><span id="x">X</span>"#;
    // beforeend：x 进 t 末（tc 后）→ t.children.length=2、[1].id=x。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             t.insertAdjacentElement('beforeend', __zw_native_element_for_id('x'));\
             return t.children.length+'/'+t.children[1].id; })()"
        ),
        "2/x"
    );
    // afterbegin：x 进 t 首（tc 前）→ t.children[0].id=x。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             t.insertAdjacentElement('afterbegin', __zw_native_element_for_id('x'));\
             return t.children[0].id; })()"
        ),
        "x"
    );
    // beforebegin：x 进 p 中 t 前 → p.children[0].id=x。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             t.insertAdjacentElement('beforebegin', __zw_native_element_for_id('x'));\
             return __zw_native_element_for_id('p').children[0].id; })()"
        ),
        "x"
    );
    // afterend：x 进 p 中 t 后 → p.children[1].id=x。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             t.insertAdjacentElement('afterend', __zw_native_element_for_id('x'));\
             return __zw_native_element_for_id('p').children[1].id; })()"
        ),
        "x"
    );
    // 返插入的元素（=== 参，spec）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t'); const x=__zw_native_element_for_id('x');\
             return (t.insertAdjacentElement('beforeend', x) === x); })()"
        ),
        "true"
    );
    // 非法 position → 抛 TypeError。
    assert_eq!(
        run_script(
            html,
            "(()=>{ try { __zw_native_element_for_id('t').insertAdjacentElement('nope',\
             __zw_native_element_for_id('x')); return 'no-throw'; } catch(e){ return 'threw'; } })()"
        ),
        "threw"
    );
    // beforebegin 无父（detached）→ 抛（spec NotFoundError，headless 取 TypeError）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             __zw_native_element_for_id('p').removeChild(t);\
             try { t.insertAdjacentElement('beforebegin', __zw_native_element_for_id('x')); return 'no-throw'; }\
             catch(e){ return 'threw'; } })()"
        ),
        "threw"
    );
}

/// `insertAdjacentText(position, string)`（spec `dom-element-insertadjacenttext`）：字符串作**字面 Text 节点**
///（不解析 HTML，区别于 insertAdjacentHTML）按 position 插入。4 位置 + 字面性 + 非法 position 抛。
#[test]
fn native_insert_adjacent_text_r3146() {
    let html = r#"<div id="p"><span id="t"></span></div>"#;
    // beforeend：文本作 Text 节点（nodeType 3）进 t；`<b>` **不解析**（nodeValue 字面含 `<b>`）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             t.insertAdjacentText('beforeend', '<b>x</b>');\
             return t.childNodes.length+'/'+t.childNodes[0].nodeType+'/'+t.childNodes[0].nodeValue; })()"
        ),
        "1/3/<b>x</b>"
    );
    // beforebegin：文本进 p 中 t 前 → p.childNodes[0]=text(3)、[1]=t 元素(1)。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             t.insertAdjacentText('beforebegin', 'hi');\
             const p=__zw_native_element_for_id('p');\
             return p.childNodes[0].nodeType+'/'+p.childNodes[1].nodeType; })()"
        ),
        "3/1"
    );
    // afterend：文本进 p 中 t 后 → p.childNodes[0]=t(1)、[1]=text(3)。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             t.insertAdjacentText('afterend', 'hi');\
             const p=__zw_native_element_for_id('p');\
             return p.childNodes[0].nodeType+'/'+p.childNodes[1].nodeType; })()"
        ),
        "1/3"
    );
    // afterbegin：文本进 t 首（t 空，等价首位）→ t.childNodes[0].nodeValue。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const t=__zw_native_element_for_id('t');\
             t.insertAdjacentText('afterbegin', 'ok');\
             return t.childNodes[0].nodeValue; })()"
        ),
        "ok"
    );
    // 非法 position → 抛 TypeError。
    assert_eq!(
        run_script(
            html,
            "(()=>{ try { __zw_native_element_for_id('t').insertAdjacentText('nope','x'); return 'no-throw'; }\
             catch(e){ return 'threw'; } })()"
        ),
        "threw"
    );
}

/// R3150 element.hasAttributes() / getAttributeNames()（spec `dom-element-hasattributes` /
/// `-getattributenames`）：attribute 族收尾。hasAttributes 返是否有任意属性；getAttributeNames 返全部
/// 属性名（文档序 Array，空属性集 → 空 Array）。
#[test]
fn native_has_attributes_attribute_names_r3150() {
    let html = r#"<div id="a" class="row" data-x="1"></div>"#;
    // hasAttributes：有属性（a）→ true。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').hasAttributes())"),
        "true"
    );
    // 无属性元素（createElement 新建，无 id）→ hasAttributes false + getAttributeNames 空。
    assert_eq!(
        run_script(html, "(__zw_native_create_element('div').hasAttributes())"),
        "false"
    );
    assert_eq!(
        run_script(html, "(__zw_native_create_element('div').getAttributeNames().length)"),
        "0"
    );
    // getAttributeNames：文档序全部属性名（含 id/class/data-x）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').getAttributeNames().join('|'))"),
        "id|class|data-x"
    );
    // setAttribute 后 hasAttributes / getAttributeNames 反映（live）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const d=__zw_native_create_element('div'); d.setAttribute('new','v');\
             return d.hasAttributes()+'/'+d.getAttributeNames().join('|'); })()"
        ),
        "true/new"
    );
}

/// R3151 element.style（CSSStyleDeclaration，spec `dom-cssstyledeclaration`）：named-property-handler 拦
/// camelCase 动态属性（`el.style.color`/`backgroundColor`）+ cssText(+setter)/length/item +
/// getPropertyValue/setProperty/removeProperty。live——经 owner `style` 属性 parse/serialize。
#[test]
fn native_element_style_r3151() {
    let html = r#"<div id="a"></div>"#;
    // 动态属性 set（camelCase→kebab）+ get + getAttribute live 写回。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.style.color='red';\
             return el.style.color+'/'+el.getAttribute('style'); })()"
        ),
        "red/color: red"
    );
    // camelCase → kebab：backgroundColor → background-color。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.style.backgroundColor='blue';\
             return el.getAttribute('style'); })()"
        ),
        "background-color: blue"
    );
    // 未设属性读 → 空串（spec：CSSStyleDeclaration 对未设属性返 ""）。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').style.color)"), "");
    // getPropertyValue（kebab）+ 多属性 + 未设空串。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('style','color: red; font-size: 12px');\
             return el.style.getPropertyValue('color')+'/'+el.style.getPropertyValue('font-size')+'/'+el.style.getPropertyValue('nope'); })()"
        ),
        "red/12px/"
    );
    // setProperty（kebab）+ getPropertyValue 回读 + 写回 style 属性。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.style.setProperty('margin','5px');\
             return el.style.getPropertyValue('margin')+'/'+el.getAttribute('style'); })()"
        ),
        "5px/margin: 5px"
    );
    // removeProperty 返旧值 + 移除后读空串。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.style.color='red';\
             return el.style.removeProperty('color')+'/'+el.style.color; })()"
        ),
        "red/"
    );
    // cssText get（规范化序列化——多余空格 trim）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('style','color:red;  background:blue');\
             return el.style.cssText; })()"
        ),
        "color: red; background: blue"
    );
    // cssText set（整体替换）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.style.cssText='display: none';\
             return el.style.display+'/'+el.getAttribute('style'); })()"
        ),
        "none/display: none"
    );
    // length + item（kebab 属性名）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('style','color: red; margin: 5px');\
             return el.style.length+'/'+el.style.item(0)+'/'+el.style.item(1); })()"
        ),
        "2/color/margin"
    );
    // 身份：同元素 .style 两次 → 同对象（spec `el.style === el.style`）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').style === __zw_native_element_for_id('a').style)"
        ),
        "true"
    );
    // live 反射：外部 setAttribute('style') → style 读反映。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('style','display: block');\
             return el.style.display; })()"
        ),
        "block"
    );
    // 协议名 fallthrough：el.style.constructor 不被动态拦截器吞（typeof==='function'；若被吞返空串则 'string'）。
    assert_eq!(
        run_script(html, "(typeof __zw_native_element_for_id('a').style.constructor)"),
        "function"
    );
    // R3211：IDL setter 空值移除声明（`el.style.color=''`；spec setProperty/IDL 空值语义）——
    // 非 dangling `color: ` 空值。`el.style.display=''` 是 reset inline 样式事实标准高频用法。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.setAttribute('style','color: red; margin: 5px'); el.style.color='';\
             return el.style.length+'/'+el.style.color+'/'+el.getAttribute('style'); })()"
        ),
        "1//margin: 5px"
    );
    // R3211：setProperty 空值移除（spec 空值语义，与 named setter / polyfill 三处对称）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.setAttribute('style','color: red'); el.style.setProperty('color','');\
             return el.style.length+'/'+el.getAttribute('style'); })()"
        ),
        "0/"
    );
    // R3212：parse_style 丢弃空值声明（spec「parse a list of declarations」：无值声明 invalid）。
    // `width:`（无值）/ `width:  `（空白值）直接经 setAttribute 注入 → 读 cssText 应不含空值段、length 不计。
    // 与 R3211 setter 空值移除对称（set 与 parse 两路均丢空值）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.setAttribute('style','color: red; width: ; margin:  ');\
             return el.style.length+'/'+el.style.cssText; })()"
        ),
        "1/color: red"
    );
    // R3213：duplicate prop 末值胜（spec「set a CSS declaration」in-place replace，保首次位置、末次值）。
    // `color:red;margin:5px;color:blue` → cssText=`color: blue; margin: 5px`（color 保位 0 值蓝）、
    // getPropertyValue('color')=blue、length=2。旧「去重保首」返 red（首值，错）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.setAttribute('style','color: red; margin: 5px; color: blue');\
             return el.style.getPropertyValue('color')+'/'+el.style.length+'/'+el.style.cssText; })()"
        ),
        "blue/2/color: blue; margin: 5px"
    );
}

/// R3152 element.dataset（DOMStringMap，spec HTML `dom-dataset`）：named-property-handler 拦 camelCase
/// 键 ↔ `data-*` 属性。get/set/delete + Object.keys 枚举 + 缺失→undefined（对象语义）。
#[test]
fn native_element_dataset_r3152() {
    let html = r#"<div id="a" data-foo-bar="x"></div>"#;
    // dataset.fooBar ↔ data-foo-bar get（camelCase→data-kebab 反射）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').dataset.fooBar)"),
        "x"
    );
    // set（camelCase→data-kebab）+ getAttribute 回读。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.dataset.bazQux='y';\
             return el.getAttribute('data-baz-qux'); })()"
        ),
        "y"
    );
    // 缺失键 → undefined（对象语义，区别于 style 的 ""）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('a').dataset.nope)"),
        "undefined"
    );
    // delete 移除 data-* 属性。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); delete el.dataset.fooBar;\
             return el.hasAttribute('data-foo-bar'); })()"
        ),
        "false"
    );
    // Object.keys 枚举 data-* → camelCase 键（经 named enumerator）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.dataset.aB='1'; el.dataset.cD='2';\
             return Object.keys(el.dataset).sort().join('|'); })()"
        ),
        "aB|cD|fooBar"
    );
    // 身份：同元素 .dataset 两次 → 同对象。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_element_for_id('a').dataset === __zw_native_element_for_id('a').dataset)"
        ),
        "true"
    );
    // 协议名 fallthrough（constructor typeof==='function'，非 undefined）。
    assert_eq!(
        run_script(html, "(typeof __zw_native_element_for_id('a').dataset.constructor)"),
        "function"
    );
}

/// R3153 element.aria* / role 反射属性（spec WAI-ARIA IDL reflection）：`el.ariaLabel`↔`aria-label`、
/// `el.ariaDescribedBy`↔`aria-describedby`（aria 前缀后整体小写单 hyphen，非 kebab）、`el.role`↔`role`。
#[test]
fn native_element_aria_role_r3153() {
    let html = r#"<div id="a" role="button" aria-label="Save"></div>"#;
    // role get（反射读 content 属性）+ set（写回）。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').role)"), "button");
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.role='link'; return el.getAttribute('role'); })()"
        ),
        "link"
    );
    // ariaLabel ↔ aria-label（camelCase→aria-hyphen）。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').ariaLabel)"), "Save");
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.ariaLabel='Close'; return el.getAttribute('aria-label'); })()"
        ),
        "Close"
    );
    // ariaDescribedBy ↔ aria-describedby（多词整体小写单 hyphen，非 aria-described-by）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.ariaDescribedBy='d1'; return el.getAttribute('aria-describedby'); })()"
        ),
        "d1"
    );
    // 缺失 → ""（reflected string 属性缺省空串）。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').ariaExpanded)"), "");
    // set 后 get 回读（live reflection）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.ariaExpanded='true'; return el.ariaExpanded; })()"
        ),
        "true"
    );
    // setAttribute 反向反射到 IDL（content→IDL 同步）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('aria-hidden','true'); return el.ariaHidden; })()"
        ),
        "true"
    );
}

/// R3154 扩展 ARIA 集：value 族 / 集合 size-pos / 标签族扩展等经同一反射机制（idl_to_attr 机械转）。
#[test]
fn native_element_aria_extended_r3154() {
    let html = r#"<div id="a" aria-valuenow="50"></div>"#;
    // ariaValueNow ↔ aria-valuenow（value 族，机械转 aria + 小写余）。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').ariaValueNow)"), "50");
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.ariaValueNow='75'; return el.getAttribute('aria-valuenow'); })()"
        ),
        "75"
    );
    // ariaLabelledBy ↔ aria-labelledby（多词单 hyphen）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.ariaLabelledBy='h1 h2'; return el.getAttribute('aria-labelledby'); })()"
        ),
        "h1 h2"
    );
    // ariaPosInSet ↔ aria-posinset（驼峰内大写转小写：PosInSet→posinset）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.ariaPosInSet='3'; return el.getAttribute('aria-posinset'); })()"
        ),
        "3"
    );
    // 多属性齐设（invalid / readonly / placeholder）经反射回读。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.ariaInvalid='true'; el.ariaReadOnly='true'; el.ariaPlaceholder='hint';\
             return el.ariaInvalid+'/'+el.ariaReadOnly+'/'+el.ariaPlaceholder; })()"
        ),
        "true/true/hint"
    );
}

/// R3155 element.tabIndex（spec HTML `tabIndex`，`tabindex` content 反射 `long`）：getter 解析 content
/// 属性为 i32（缺省/非法 → -1），setter 经 ToInt32 强转写回。补充 R3148 焦点工作（a11y 焦点序核心属性）。
#[test]
fn native_element_tab_index_r3155() {
    // 缺省 → -1（headless 简化：无原生可聚焦性判定，统一默认 -1；匹配 `document.createElement('div').tabIndex`）。
    let html = r#"<div id="a"></div>"#;
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').tabIndex)"), "-1");
    // content 属性存在且可解析 → 解析值。
    assert_eq!(
        run_script(
            r#"<div id="a" tabindex="3"></div>"#,
            "(__zw_native_element_for_id('a').tabIndex)"
        ),
        "3"
    );
    // setter：el.tabIndex = 5 → getAttribute('tabindex') === '5'。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.tabIndex=5; return el.getAttribute('tabindex'); })()"
        ),
        "5"
    );
    // setter round-trip：el.tabIndex = -1 → el.tabIndex === -1（-1 = 可聚焦但不在 Tab 序，a11y 高频）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.tabIndex=-1; return el.tabIndex; })()"
        ),
        "-1"
    );
    // 非法 content 属性 → -1（HTML 整数解析失败回退默认）。
    assert_eq!(
        run_script(
            r#"<div id="a" tabindex="abc"></div>"#,
            "(__zw_native_element_for_id('a').tabIndex)"
        ),
        "-1"
    );
    // ToInt32 强转：el.tabIndex = 3.7 → getAttribute('tabindex') === '3'（spec long setter 经 ToInt32）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.tabIndex=3.7; return el.getAttribute('tabindex'); })()"
        ),
        "3"
    );
    // setAttribute 反向反射到 IDL（content→IDL live）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('tabindex','7'); return el.tabIndex; })()"
        ),
        "7"
    );
}

/// R3156 element.hidden（spec HTML `hidden`，content 反射 `boolean`）：getter 属性在且非
/// `"until-found"` → true，setter 经 ToBoolean 强转 set `""` / remove。条件显隐组件高频。
#[test]
fn native_element_hidden_r3156() {
    // 缺省 → false。
    let html = r#"<div id="a"></div>"#;
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').hidden)"), "false");
    // content 属性存在（boolean 空串）→ true。
    assert_eq!(
        run_script(
            r#"<div id="a" hidden></div>"#,
            "(__zw_native_element_for_id('a').hidden)"
        ),
        "true"
    );
    // setter true → getAttribute('hidden') === ''（boolean content 属性空串 = 存在）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.hidden=true; return el.getAttribute('hidden'); })()"
        ),
        ""
    );
    // setter false → 移除属性（hasAttribute false + hidden false）。
    assert_eq!(
        run_script(
            r#"<div id="a" hidden></div>"#,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.hidden=false; return el.hasAttribute('hidden')+'/'+el.hidden; })()"
        ),
        "false/false"
    );
    // ToBoolean 强转：空串/0 → false（移除），非空串/"1" → true（设）。
    assert_eq!(
        run_script(
            r#"<div id="a" hidden></div>"#,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.hidden=''; return el.hidden; })()"
        ),
        "false"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.hidden=1; return el.hidden; })()"
        ),
        "true"
    );
    // `hidden="until-found"` → getter 返 false（「hidden until found」独立状态，IDL boolean false）。
    assert_eq!(
        run_script(
            r#"<div id="a" hidden="until-found"></div>"#,
            "(__zw_native_element_for_id('a').hidden)"
        ),
        "false"
    );
}

/// R3157 批量反射属性（spec HTML）：title/lang/dir/accessKey（字符串反射，IDL 名经 to_ascii_lowercase
/// 映射 content 名）+ inert（boolean 反射）。共用 name-dispatched string 反射 + 专用 inert boolean。
#[test]
fn native_element_batch_reflected_r3157() {
    // title 字符串反射：缺省 "" + get/set round-trip + content 属性同步。
    let html = r#"<div id="a" title="hint"></div>"#;
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').title)"), "hint");
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.title='tip'; return el.getAttribute('title')+'/'+el.title; })()"
        ),
        "tip/tip"
    );
    // accessKey 字符串反射：camelCase IDL → 小写 content 名（accessKey↔accesskey）。
    assert_eq!(
        run_script(
            r#"<div id="a" accesskey="h"></div>"#,
            "(__zw_native_element_for_id('a').accessKey)"
        ),
        "h"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.accessKey='k'; return el.getAttribute('accesskey'); })()"
        ),
        "k"
    );
    // lang / dir 字符串反射（i18n，dir 影响 layout 方向）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.lang='en'; el.dir='rtl'; return el.lang+'/'+el.dir+'/'+el.getAttribute('lang')+'/'+el.getAttribute('dir'); })()"
        ),
        "en/rtl/en/rtl"
    );
    // 缺省字符串属性 → ""。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').lang)"), "");
    // inert boolean 反射：缺省 false + content 属性在 true + setter ToBoolean set/remove。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').inert)"), "false");
    assert_eq!(
        run_script(r#"<div id="a" inert></div>"#, "(__zw_native_element_for_id('a').inert)"),
        "true"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.inert=true; return el.hasAttribute('inert')+'/'+el.getAttribute('inert'); })()"
        ),
        "true/"
    );
    assert_eq!(
        run_script(
            r#"<div id="a" inert></div>"#,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.inert=0; return el.inert+'/'+el.hasAttribute('inert'); })()"
        ),
        "false/false"
    );
}

/// R3158 element.draggable（spec HTML `draggable`，enumerated content 反射 boolean）：第 4 种反射子类型
///——content 属性取 `"true"`/`"false"` 字面值。getter 值 == `"true"` → true 余 false；setter ToBoolean 写
/// `"true"`/`"false"` 字面串（区别 pure-boolean 写空串）。
#[test]
fn native_element_draggable_r3158() {
    // 缺省 → false（headless 简化：统一默认 false，匹配通用 div 等多数元素）。
    let html = r#"<div id="a"></div>"#;
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').draggable)"), "false");
    // content 属性值 == "true" → true。
    assert_eq!(
        run_script(
            r#"<div id="a" draggable="true"></div>"#,
            "(__zw_native_element_for_id('a').draggable)"
        ),
        "true"
    );
    // content 属性值 == "false" → false。
    assert_eq!(
        run_script(
            r#"<div id="a" draggable="false"></div>"#,
            "(__zw_native_element_for_id('a').draggable)"
        ),
        "false"
    );
    // setter true → getAttribute('draggable') === 'true'（字面串，非空串）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.draggable=true; return el.getAttribute('draggable'); })()"
        ),
        "true"
    );
    // setter false → getAttribute('draggable') === 'false'（仍写字面串，非移除）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.draggable=false; return el.getAttribute('draggable')+'/'+el.hasAttribute('draggable'); })()"
        ),
        "false/true"
    );
    // ToBoolean 强转：1 → 'true'，0 → 'false'。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.draggable=1; return el.getAttribute('draggable'); })()"
        ),
        "true"
    );
    assert_eq!(
        run_script(
            r#"<div id="a" draggable="true"></div>"#,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.draggable=0; return el.getAttribute('draggable'); })()"
        ),
        "false"
    );
    // setAttribute 反向反射到 IDL（content→IDL live）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); el.setAttribute('draggable','true'); return el.draggable; })()"
        ),
        "true"
    );
}

/// R3159 native `document` 对象（escape-hatch 铺路）：`__zw_native_get_document()` 返命名空间对象，
/// 方法复用 factories（getElementById/querySelector/createElement 等）+ getter 读 live Document
///（documentElement/body/head/activeElement）。spec 身份（`=== ` 同对象，gc.rs 单例 weak 缓存）。
#[test]
fn native_document_object_r3159() {
    let html = r#"<html><head><title>t</title></head><body><div id="a"><span class="x">hi</span></div></body></html>"#;
    // documentElement getter → <html>（tagName HTML）。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().documentElement.tagName)"),
        "HTML"
    );
    // body / head getter → <body>/<head>。
    assert_eq!(run_script(html, "(__zw_native_get_document().body.tagName)"), "BODY");
    assert_eq!(run_script(html, "(__zw_native_get_document().head.tagName)"), "HEAD");
    // activeElement getter → 无焦点 null。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().activeElement === null)"),
        "true"
    );
    // getElementById 方法（复用 factories）→ <div id=a>。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().getElementById('a').tagName)"),
        "DIV"
    );
    // querySelector 方法 → <span class=x>。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().querySelector('span.x').tagName)"),
        "SPAN"
    );
    // querySelectorAll 方法 → 长度。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().querySelectorAll('span').length)"),
        "1"
    );
    // getElementsByTagName 方法 → body 数 1。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().getElementsByTagName('body').length)"),
        "1"
    );
    // createElement 方法 → 未挂载新 <p>（nodeType 1，tagName P）。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().createElement('p').tagName)"),
        "P"
    );
    // createTextNode 方法 → nodeType 3。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().createTextNode('ok').nodeType)"),
        "3"
    );
    // 身份：__zw_native_get_document() === __zw_native_get_document()（单例缓存）。
    assert_eq!(
        run_script(html, "(__zw_native_get_document() === __zw_native_get_document())"),
        "true"
    );
    // getElementById 结果与既有 element_for_id 工厂共享 NodeId↔对象映射（同对象）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().getElementById('a') === __zw_native_element_for_id('a'))"
        ),
        "true"
    );
}

/// R3160 `document.title` get/set（spec `dom-document-title`）：经共享 factories helper（与
/// `__zw_native_*_document_title` 工厂共用，DRY）。补全 native document 对象常用 API。
#[test]
fn native_document_title_r3160() {
    // 读既有 <title> textContent。
    let html = r#"<html><head><title>Old</title></head><body></body></html>"#;
    assert_eq!(run_script(html, "(__zw_native_get_document().title)"), "Old");
    // setter → 改 <title> textContent（经工厂 read 回读验证 live）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const d=__zw_native_get_document(); d.title='New'; return d.title; })()"
        ),
        "New"
    );
    // setter 与 __zw_native_get_document_title 工厂共享底层（工厂读同值）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_get_document().title='Shared'; return __zw_native_get_document_title(); })()"
        ),
        "Shared"
    );
    // 无 <title> → 空串（getter 缺省）。
    let no_title_html = r#"<html><head></head><body></body></html>"#;
    assert_eq!(run_script(no_title_html, "(__zw_native_get_document().title)"), "");
    // setter 无 <title> 时在 <head> 建 <title>（html5ever 归一化有 head）→ 读回新值。
    assert_eq!(
        run_script(
            no_title_html,
            "(()=>{ __zw_native_get_document().title='Created'; return __zw_native_get_document().title; })()"
        ),
        "Created"
    );
    // setAttribute('title') **不** 反射到 document.title（element title 属性 ≠ document title）——
    // document.title 读 <title> 元素（此 html 的 <title>Old</title>），非 title 属性。
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_get_document().body.setAttribute('title','attr-tip'); return __zw_native_get_document().title; })()"
        ),
        "Old"
    );
}

/// R3161 `document.createEvent(type)`（spec `dom-document-createevent`，legacy 事件创建）：复用 event
/// 子模块工厂（R3141，type→构造器映射 + `new Ctor("")`）。补全 document 创建 API 三件套。
#[test]
fn native_document_create_event_r3161() {
    let html = r#"<html><head></head><body></body></html>"#;
    // createEvent('Event') → 未初始化 event（type=""，待 initEvent 覆写）。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().createEvent('Event').type)"),
        ""
    );
    // initEvent 覆写 type/bubbles（经原型链）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_get_document().createEvent('Event'); e.initEvent('click', true, false); return e.type+'/'+e.bubbles; })()"
        ),
        "click/true"
    );
    // createEvent('CustomEvent') → CustomEvent 对象（type="" 未初始化）。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().createEvent('CustomEvent').type)"),
        ""
    );
    // createEvent 经 Event 构造器产 instanceof Event 对象。
    assert_eq!(
        run_script(
            html,
            "((__zw_native_get_document().createEvent('Event') instanceof Event))"
        ),
        "true"
    );
    // createEvent 产对象可 dispatchEvent（端到端：createEvent + initEvent + 监听器触发）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const d=__zw_native_get_document(); const b=d.body; var got='none';\
             b.addEventListener('x', function(e){ got=e.type; });\
             const ev=d.createEvent('Event'); ev.initEvent('x', false, false);\
             b.dispatchEvent(ev); return got; })()"
        ),
        "x"
    );
}

/// R3162 `document.importNode(node, deep)` / `adoptNode(node)`（spec `dom-document-importnode` /
/// `-adoptnode`）：importNode 克隆节点（复用 clone_node，模板实例化高频）；adoptNode headless 单文档
/// = identity（同对象）。
#[test]
fn native_document_import_adopt_node_r3162() {
    let html = r#"<html><head></head><body><div id="src"><span>hi</span></div></body></html>"#;
    // importNode 浅克隆 → 新 div（tagName DIV，≠ 源身份）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const d=__zw_native_get_document(); const s=d.getElementById('src');\
             const c=d.importNode(s, false); return c.tagName+'/'+(c===s); })()"
        ),
        "DIV/false"
    );
    // importNode 深克隆 → 子 span 在（deep=true 递归）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const d=__zw_native_get_document(); const s=d.getElementById('src');\
             const c=d.importNode(s, true); return c.children.length+'/'+c.children[0].tagName; })()"
        ),
        "1/SPAN"
    );
    // importNode 浅克隆 → 子 span 不在（deep=false）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const d=__zw_native_get_document(); const s=d.getElementById('src');\
             const c=d.importNode(s, false); return c.children.length; })()"
        ),
        "0"
    );
    // adoptNode headless 单文档 → identity（同对象 ===）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const d=__zw_native_get_document(); const s=d.getElementById('src');\
             return (d.adoptNode(s) === s); })()"
        ),
        "true"
    );
    // importNode 非节点参（null）→ null（best-effort）。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().importNode(null, true) === null)"),
        "true"
    );
}

/// R3163 `document.createElementNS(ns, qualifiedName)`（spec `dom-document-createelementns`）：带命名空间
/// 创建（SVG/MathML 编程创建高频）。dom `create_element_ns` 解析 prefix:local + 建 QualName。
#[test]
fn native_document_create_element_ns_r3163() {
    let html = r#"<html><head></head><body></body></html>"#;
    // createElementNS(svg, "svg") → SVG 元素（local "svg"；SVG 非 HTML 命名空间，tagName 原样小写 → "svg"，
    // spec `dom-element-tagname` 大小写敏感——R3166 修正，此前误返 "SVG"）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/2000/svg','svg').tagName)"
        ),
        "svg"
    );
    // createElementNS(svg, "rect") → rect（SVG 子元素，原样小写）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/2000/svg','rect').tagName)"
        ),
        "rect"
    );
    // 带前缀 qualifiedName "svg:rect" → 解析 prefix=svg / local=rect（tagName 取 local，原样小写）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/2000/svg','svg:rect').tagName)"
        ),
        "rect"
    );
    // createElementNS(html, "div") → HTML div（XHTML 命名空间，tagName 大写 → "DIV"）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/1999/xhtml','div').tagName)"
        ),
        "DIV"
    );
    // createElementNS 产新对象（未挂载，nodeType 1）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/2000/svg','circle').nodeType)"
        ),
        "1"
    );
}

/// `document.createElement(tag)` spec validate（`dom-document-createelement`）：非法标签名抛
/// InvalidCharacterError DOMException。对齐 WPT `dom/nodes/Document-createElement.html` invalid 列表。
#[test]
fn native_document_create_element_invalid_name_throws() {
    let html = r#"<html><head></head><body></body></html>"#;
    // invalid 列表（WPT）：空 / 数字首 / `-`/`.`首 / 含空白 / `<`/`>` / `}` → InvalidCharacterError。
    let invalid = [
        "", "1foo", "1:foo", "fo o", "}foo", "<foo", "foo>", "<foo>", "-foo", ".foo",
    ];
    for tag in invalid {
        let escaped = tag.replace('\\', "\\\\").replace('\'', "\\'");
        // 用 run_script 跑会抛 → 返异常串；改用 try/catch 包装返 name（run_script 不捕异常）。
        // 注：run_script 遇未捕获异常返 V8 异常串（非 panic），断言含 InvalidCharacterError。
        let script = format!(
            "(()=>{{ try {{ __zw_native_get_document().createElement('{escaped}'); return 'no-throw'; }}\
             catch(e){{ return e.name; }} }})()"
        );
        let got = run_script(html, &script);
        assert_eq!(
            got, "InvalidCharacterError",
            "createElement({tag:?}) 应抛 InvalidCharacterError，实际：{got}"
        );
    }
}

/// `document.createElement` valid 标签不抛（含 undefined/null ToString、含 `:`、Unicode 首字符）。
/// 对齐 WPT valid 列表（防 is_valid_qualified_name 误伤）。
#[test]
fn native_document_create_element_valid_name_passes() {
    let html = r#"<html><head></head><body></body></html>"#;
    // 合法名 → 不抛，返元素 tagName（HTML 大写）。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().createElement('foo').tagName)"),
        "FOO"
    );
    // 含 `:`（Name production 允许，非 QName 限制）→ 合法。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().createElement('f:oo').tagName)"),
        "F:OO"
    );
    // createElement(undefined) → JS ToString 成 "undefined"（首字符字母）合法通过（WPT valid 列表）。
    assert_eq!(
        run_script(html, "(__zw_native_get_document().createElement(undefined).tagName)"),
        "UNDEFINED"
    );
}

/// R3166 `element.tagName` / `nodeName` 命名空间大小写敏感（spec `dom-element-tagname`：
/// HTML-uppercased local name）。闭合 R3163 限制①——HTML 命名空间元素大写，SVG/MathML 等原样小写。
#[test]
fn native_element_tag_name_namespace_case_r3166() {
    let html = r#"<html><head></head><body><svg id="s"><rect id="r"/></svg></body></html>"#;
    // HTML 命名空间（createElement 默认 XHTML）→ ASCII 大写。
    assert_eq!(run_script(html, "(__zw_native_create_element('div').tagName)"), "DIV");
    // createElementNS(svg, "rect") → SVG 命名空间 → 原样小写。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/2000/svg','rect').tagName)"
        ),
        "rect"
    );
    // createElementNS(mathml, "mi") → MathML 命名空间 → 原样小写。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/1998/Math/MathML','mi').tagName)"
        ),
        "mi"
    );
    // 解析得到的 SVG 元素（html5ever 赋 SVG 命名空间）→ 原样小写。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('s').tagName)"), "svg");
    assert_eq!(run_script(html, "(__zw_native_element_for_id('r').tagName)"), "rect");
    // nodeName == tagName（Element 上，spec `dom-node-nodename`），SVG 元素亦原样小写。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('s').nodeName)"), "svg");
    // HTML 元素 nodeName == tagName == 大写。
    let html2 = r#"<div id="d"></div>"#;
    assert_eq!(run_script(html2, "(__zw_native_element_for_id('d').nodeName)"), "DIV");
}

/// R3169 `document.URL` / `document.documentURI`（spec `dom-document-url` / `dom-document-documenturi`）：
/// 经 live Document `url()` 读导航层注入的页面地址（分析/框架高频）。两别名同值。
#[test]
fn native_document_url_r3169() {
    let std_html = r#"<!DOCTYPE html><html><head></head><body></body></html>"#;
    // 注入页面 URL → document.URL 读回 + documentURI 别名同值。
    assert_eq!(
        run_script_with_url(
            std_html,
            "https://example.com/page?q=1",
            "(__zw_native_get_document().URL)"
        ),
        "https://example.com/page?q=1"
    );
    assert_eq!(
        run_script_with_url(
            std_html,
            "https://example.com/page?q=1",
            "(__zw_native_get_document().documentURI)"
        ),
        "https://example.com/page?q=1"
    );
    // run_script 模型不注入 URL → document.URL 空串（headless 简化，真实浏览器路径经导航注入）。
    assert_eq!(run_script(std_html, "(__zw_native_get_document().URL)"), "");
}

/// R3176 `document.referrer`（spec `dom-document-referrer`）：经 live Document `referrer()` 读
/// 导航层注入的来源页 URL（分析/框架高频，GA/Sentry 等必读）。未注入 → 空串。
#[test]
fn native_document_referrer_r3176() {
    let std_html = r#"<!DOCTYPE html><html><head></head><body></body></html>"#;
    // 注入来源页 URL → document.referrer 读回。
    assert_eq!(
        run_script_with_referrer(
            std_html,
            "https://ref.example.com/prev-page",
            "(__zw_native_get_document().referrer)"
        ),
        "https://ref.example.com/prev-page"
    );
    // run_script 模型不注入 referrer → document.referrer 空串（headless 简化，真实浏览器路径
    // 经导航注入 = 导航前的页面 URL；直接打开页面无来源亦为空串）。
    assert_eq!(run_script(std_html, "(__zw_native_get_document().referrer)"), "");
}

/// R3168 `document.compatMode` / `characterSet` / `contentType` / `readyState`（spec
/// `dom-document-compatmode` 等）：文档元数据只读字符串（分析/框架高频读取）。
#[test]
fn native_document_metadata_r3168() {
    // 标准 doctype → no-quirks → compatMode "CSS1Compat"。
    let std_html = r#"<!DOCTYPE html><html><head></head><body></body></html>"#;
    assert_eq!(
        run_script(std_html, "(__zw_native_get_document().compatMode)"),
        "CSS1Compat"
    );
    // 无 doctype → quirks → compatMode "BackCompat"。
    let quirks_html = r#"<html><head></head><body></body></html>"#;
    assert_eq!(
        run_script(quirks_html, "(__zw_native_get_document().compatMode)"),
        "BackCompat"
    );
    // characterSet 固定 UTF-8（html5ever HTML 解析默认）。
    assert_eq!(
        run_script(std_html, "(__zw_native_get_document().characterSet)"),
        "UTF-8"
    );
    // contentType 固定 text/html（HTML 文档）。
    assert_eq!(
        run_script(std_html, "(__zw_native_get_document().contentType)"),
        "text/html"
    );
    // readyState 固定 complete（headless 全解析后）。
    assert_eq!(
        run_script(std_html, "(__zw_native_get_document().readyState)"),
        "complete"
    );
}

/// R3167 `element.contentEditable`（枚举反射 + setter 非法值抛 SyntaxError）+ `isContentEditable`
///（继承走查）+ `spellcheck`（带继承 boolean）。spec HTML `dom-contenteditable` / `dom-iscontenteditable`
/// / `dom-spellcheck`。
#[test]
fn native_element_content_editable_r3167() {
    let html = r#"<html><head></head><body>
      <div id="e"></div>
      <div id="p" contenteditable="true"><span id="c1">x</span></div>
      <div id="pf" contenteditable="false"><span id="c2">y</span></div>
      <input id="sp" spellcheck="true"/>
    </body></html>"#;
    // contentEditable 默认（无属性）→ "inherit"。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('e').contentEditable)"),
        "inherit"
    );
    // setter "true" → contentEditable "true" + isContentEditable true + contenteditable 属性 "true"。
    assert_eq!(
        run_script(
            html,
            "(()=>{const e=__zw_native_element_for_id('e'); e.contentEditable='true';\
             return e.contentEditable+'/'+e.isContentEditable+'/'+e.getAttribute('contenteditable');})()"
        ),
        "true/true/true"
    );
    // setter "false" → "false" + isContentEditable false。
    assert_eq!(
        run_script(
            html,
            "(()=>{const e=__zw_native_element_for_id('e'); e.contentEditable='false';\
             return e.contentEditable+'/'+e.isContentEditable;})()"
        ),
        "false/false"
    );
    // setter "inherit" → 移除属性 + contentEditable "inherit"。
    assert_eq!(
        run_script(
            html,
            "(()=>{const e=__zw_native_element_for_id('e'); e.contentEditable='inherit';\
             return e.contentEditable+'/'+e.hasAttribute('contenteditable');})()"
        ),
        "inherit/false"
    );
    // setAttribute("contenteditable","garbage")（非法 keyword）→ 状态 inherit → contentEditable "inherit"。
    assert_eq!(
        run_script(
            html,
            "(()=>{const e=__zw_native_element_for_id('e'); e.setAttribute('contenteditable','garbage');\
             return e.contentEditable;})()"
        ),
        "inherit"
    );
    // isContentEditable 继承：父 contenteditable=true，子 span 继承 → isContentEditable true。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('c1').isContentEditable)"),
        "true"
    );
    // 父 contenteditable=false，子继承 → isContentEditable false。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('c2').isContentEditable)"),
        "false"
    );
    // setter 非法值 → 抛 SyntaxError（e.name === "SyntaxError"）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const e=__zw_native_element_for_id('e');\
             try{ e.contentEditable='garbage'; return 'no-throw'; }catch(ex){ return ex.name; }})()"
        ),
        "SyntaxError"
    );
    // spellcheck：显式 spellcheck=true → getter true。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('sp').spellcheck)"),
        "true"
    );
    // spellcheck setter ToBoolean 强转：set true → 属性 "true" + getter true；set 0 → "false"。
    assert_eq!(
        run_script(
            html,
            "(()=>{const e=__zw_native_element_for_id('e'); e.spellcheck=1;\
             return e.spellcheck+'/'+e.getAttribute('spellcheck');})()"
        ),
        "true/true"
    );
    assert_eq!(
        run_script(
            html,
            "(()=>{const e=__zw_native_element_for_id('e'); e.spellcheck=0;\
             return e.spellcheck+'/'+e.getAttribute('spellcheck');})()"
        ),
        "false/false"
    );
    // spellcheck 默认（无属性，无可编辑祖先）→ false。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('e').spellcheck)"),
        "false"
    );
}

/// R3187 `contentEditable` / `isContentEditable` / `spellcheck` **空串 keyword** spec 合规。
///
/// spec HTML：`contenteditable`/`spellcheck` 为枚举属性，关键字为「空串、true、false」——**空串与 `true`
/// 同映射 true 状态**。故 `contenteditable=""`（经典 `<div contenteditable>` 可编辑元素写法）须返
/// `contentEditable="true"` + `isContentEditable=true`。旧实现把空串当 inherit（`contentEditable="inherit"`、
/// `isContentEditable=false`）。spec `dom-contenteditable` / `dom-iscontenteditable` / `dom-spellcheck`。
#[test]
fn native_content_editable_empty_keyword_true_state_r3187() {
    let html = r#"<html><head></head><body>
      <div id="e"></div>
      <div id="ce" contenteditable=""></div>
      <div id="ctrue" contenteditable="TRUE"></div>
      <input id="spe" spellcheck=""/>
    </body></html>"#;
    // 解析期 contenteditable=""（空串 keyword）→ true 状态 → contentEditable "true"。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('ce').contentEditable)"),
        "true"
    );
    // 空串 → true 状态 → isContentEditable true（旧实现 false，把空串当 inherit）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('ce').isContentEditable)"),
        "true"
    );
    // case-insensitive "TRUE" → true 状态 → contentEditable "true"（规范化小写）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('ctrue').contentEditable)"),
        "true"
    );
    // setAttribute 空串（spec 空串 keyword = true 状态；空串仅 content-attribute keyword，非 IDL setter
    // keyword——IDL setter `e.contentEditable=''` 会抛 SyntaxError，故用 setAttribute 路径）→ 回读 "true" +
    // isContentEditable true + 属性 ""。
    assert_eq!(
        run_script(
            html,
            "(()=>{const e=__zw_native_element_for_id('e'); e.setAttribute('contenteditable','');\
             return e.contentEditable+'/'+e.isContentEditable+'/'+e.getAttribute('contenteditable');})()"
        ),
        "true/true/"
    );
    // spellcheck="" （空串 keyword）→ true 状态 → getter true（旧实现 false，把空串当 inherit）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('spe').spellcheck)"),
        "true"
    );
}

/// R3188 `draggable` enumerated getter：case-insensitive keyword + auto-state default-draggable。
///
/// spec HTML `draggable`（https://html.spec.whatwg.org/multipage/dnd.html#the-draggable-attribute）：
/// 枚举属性，关键字 true/false（ASCII case-insensitive），缺省/非法 → auto 状态。IDL getter（boolean）：
/// true 状态 → true；auto 状态且元素 default-draggable（img/audio/video/a[href]）→ true；余 → false。
/// 旧实现 case-sensitive（`draggable="TRUE"`→false）+ auto 状态统一 false（`<img>` 误判不可拖拽）。
#[test]
fn native_draggable_enumerated_auto_state_r3188() {
    let html = r#"<html><head></head><body>
      <div id="dtrue" draggable="true"></div>
      <div id="dupper" draggable="TRUE"></div>
      <div id="dfalse" draggable="false"></div>
      <div id="dgarbage" draggable="foo"></div>
      <div id="div"></div>
      <img id="img"/>
      <a id="ahref" href="x.html"></a>
      <a id="anohref"></a>
      <audio id="aud"></audio>
    </body></html>"#;
    // 显式 true（小写/大写 case-insensitive）→ true。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('dtrue').draggable)"),
        "true"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('dupper').draggable)"),
        "true"
    );
    // 显式 false → false。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('dfalse').draggable)"),
        "false"
    );
    // auto 状态（invalid "foo" / 缺省）→ default-draggable：div → false。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('dgarbage').draggable)"),
        "false"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('div').draggable)"),
        "false"
    );
    // auto 状态 default-draggable：img / audio → true。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('img').draggable)"),
        "true"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('aud').draggable)"),
        "true"
    );
    // a 带 href → true；a 无 href → false。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('ahref').draggable)"),
        "true"
    );
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('anohref').draggable)"),
        "false"
    );
    // 显式 true 覆盖 default：img draggable=false → false（非 auto）。
    assert_eq!(
        run_script(
            html,
            "(()=>{const i=__zw_native_element_for_id('img'); i.setAttribute('draggable','false');\
             return i.draggable;})()"
        ),
        "false"
    );
}

/// R3172 HTML 序列化 tag 小写：createElement('DIV').outerHTML → '<div></div>'（dom serializer 对 HTML
/// 命名空间元素 ASCII 小写，SVG/MathML 保留）。修正编程创建大写 tag 的序列化 + void 元素识别。
#[test]
fn native_create_element_uppercase_serializes_lowercase_r3172() {
    let html = r#"<html><head></head><body></body></html>"#;
    // createElement('DIV')（大写）→ outerHTML '<div></div>'（HTML 序列化小写；旧 '<DIV></DIV>'）。
    assert_eq!(
        run_script(html, "(__zw_native_create_element('DIV').outerHTML)"),
        "<div></div>"
    );
    // void 元素 createElement('BR')（大写）→ outerHTML '<br>'（is_void_element 识别小写 'br'；旧 '<BR></BR>'）。
    assert_eq!(run_script(html, "(__zw_native_create_element('BR').outerHTML)"), "<br>");
    // 对照：createElement('div')（小写）→ '<div></div>'（无回归）。
    assert_eq!(
        run_script(html, "(__zw_native_create_element('div').outerHTML)"),
        "<div></div>"
    );
    // SVG createElementNS 保留 tag 名大小写（非 HTML 命名空间不强制小写——HTML↔SVG 的核心区别）。
    // 真实浏览器：createElementNS(svg,'RECT').outerHTML → '<RECT></RECT>'（SVG 大小写敏感保留）。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/2000/svg','RECT').outerHTML)"
        ),
        "<RECT></RECT>"
    );
}

/// R3165 `element.namespaceURI` getter（spec `dom-node-namespaceuri`）：元素命名空间 URI 字符串，
/// 空 namespace → null。闭合 R3163 限制②（namespace 经 native 可读）。
#[test]
fn native_element_namespace_uri_r3165() {
    let html = r#"<html><head></head><body><svg id="s"></svg></body></html>"#;
    // createElementNS(svg, ...) → SVG 命名空间 URI。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/2000/svg','rect').namespaceURI)"
        ),
        "http://www.w3.org/2000/svg"
    );
    // createElementNS(mathml, ...) → MathML 命名空间 URI。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/1998/Math/MathML','mi').namespaceURI)"
        ),
        "http://www.w3.org/1998/Math/MathML"
    );
    // createElementNS(xhtml, ...) → XHTML 命名空间 URI。
    assert_eq!(
        run_script(
            html,
            "(__zw_native_get_document().createElementNS('http://www.w3.org/1999/xhtml','div').namespaceURI)"
        ),
        "http://www.w3.org/1999/xhtml"
    );
    // 解析得到的 SVG 元素（html5ever 赋 SVG 命名空间）。
    assert_eq!(
        run_script(html, "(__zw_native_element_for_id('s').namespaceURI)"),
        "http://www.w3.org/2000/svg"
    );
}

/// `document.createProcessingInstruction(target, data)`（spec `dom-document-createprocessinginstruction`）。
/// R7：valid 返 PI 对象（target/data/nodeName=target/nodeType=7）；invalid target/data 抛 InvalidCharacterError。
#[test]
fn native_document_create_processing_instruction_r7() {
    let html = r#"<html><head></head><body></body></html>"#;
    // valid：返 PI，target/data/nodeName/nodeType 正确。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const pi=__zw_native_get_document().createProcessingInstruction('xml-stylesheet','href=\"x.css\"');\
             return pi.target+'/'+pi.data+'/'+pi.nodeName+'/'+pi.nodeType; })()"
        ),
        "xml-stylesheet/href=\"x.css\"/xml-stylesheet/7"
    );
    // invalid target（数字首 "0"）→ InvalidCharacterError。
    assert_eq!(
        run_script(
            html,
            "(()=>{ try { __zw_native_get_document().createProcessingInstruction('0','x'); return 'no-throw'; }\
             catch(e){ return e.name; } })()"
        ),
        "InvalidCharacterError"
    );
    // invalid data（含 `?>`）→ InvalidCharacterError。
    assert_eq!(
        run_script(
            html,
            "(()=>{ try { __zw_native_get_document().createProcessingInstruction('ok','a?>b'); return 'no-throw'; }\
             catch(e){ return e.name; } })()"
        ),
        "InvalidCharacterError"
    );
}
