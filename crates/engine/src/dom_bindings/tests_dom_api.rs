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
use super::tests::run_script;
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

// ── R3147 element.click()（spec dom-element-click）+ dispatchEvent 重构无回归 ──

/// `element.click()`：派发合成 click MouseEvent（bubbles + cancelable）到 this。触发本元素 click 监听器
///（event.type==='click'、event.target===this）+ 冒泡到祖先 + 返 `!(cancelable && defaultPrevented)`
///（preventDefault 时 false）。复用 dispatch_event_impl 三阶段派发核心（R3147 抽出）。
#[test]
fn native_element_click_r3147() {
    let html = r#"<div id="p"><span id="c"></span></div><span id="a"></span>"#;
    // 触发 click 监听器 + event.type / event.target 正确（target===被点元素）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a'); let got='';\
             el.addEventListener('click', e=>{ got=e.type+'/'+(e.target===el); });\
             el.click(); return got; })()"
        ),
        "click/true"
    );
    // 冒泡到祖先：child.click() 触发 parent click 监听器（click 事件 bubbles=true）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const p=__zw_native_element_for_id('p'); const c=__zw_native_element_for_id('c');\
             let bubbled='no'; p.addEventListener('click', ()=>{ bubbled='yes'; });\
             c.click(); return bubbled; })()"
        ),
        "yes"
    );
    // 返值 true（未 preventDefault）。
    assert_eq!(run_script(html, "(__zw_native_element_for_id('a').click())"), "true");
    // 返值 false（监听器 preventDefault——cancelable 事件被 preventDefault 则返 false，spec）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('a');\
             el.addEventListener('click', e=>{ e.preventDefault(); });\
             return el.click(); })()"
        ),
        "false"
    );
}

/// dispatchEvent 重构（R3147 抽 dispatch_event_impl）无回归守卫：既有 dispatchEvent 行为
///（触发监听器 + 冒泡 + stopPropagation）经重构后仍正确。
#[test]
fn native_dispatch_event_refactor_no_regress_r3147() {
    let html = r#"<div id="p"><span id="c"></span></div>"#;
    // dispatchEvent 仍触发监听器 + event.type 读自对象。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('c'); let got='';\
             el.addEventListener('click', e=>{ got=e.type; });\
             el.dispatchEvent({type:'click'}); return got; })()"
        ),
        "click"
    );
    // dispatchEvent 冒泡仍正确（bubbles:true 上溯祖先）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const p=__zw_native_element_for_id('p'); const c=__zw_native_element_for_id('c');\
             let n=0; p.addEventListener('click', ()=>{ n++; });\
             c.dispatchEvent({type:'click', bubbles:true}); return n; })()"
        ),
        "1"
    );
    // dispatchEvent 字符串参仍标准化为 {type:str}。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('c'); let got='';\
             el.addEventListener('x', e=>{ got=e.type; });\
             el.dispatchEvent('x'); return got; })()"
        ),
        "x"
    );
}

// ── R3148 element.focus() / element.blur() + document.activeElement ──

/// `element.focus()` / `element.blur()`（spec `dom-element-focus` / `-blur`）：焦点更新/失焦步骤——
/// 派发非冒泡 focus/blur 事件 + 追踪 document.activeElement（gc.rs ACTIVE_ELEMENT）。闭合 polyfill 限制②
///（旧 focus/blur 不派发事件）。focus 切换 blur old→focus new 顺序 + 幂等 + blur no-op + 非冒泡。
#[test]
fn native_element_focus_blur_r3148() {
    let html = r#"<div id="p"><span id="a"></span><span id="b"></span></div>"#;
    // focus() 派发 focus 事件 + 追踪 activeElement。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); let log='';\
             a.addEventListener('focus', ()=>{ log+='a-focus;'; });\
             a.focus(); return log+'/'+(__zw_native_get_active_element()===a); })()"
        ),
        "a-focus;/true"
    );
    // focus 切换：blur old（a）→ focus new（b），顺序 a-blur 先于 b-focus；activeElement=b。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); const b=__zw_native_element_for_id('b');\
             let log='';\
             a.addEventListener('focus', ()=>{ log+='af;'; });\
             a.addEventListener('blur', ()=>{ log+='ab;'; });\
             b.addEventListener('focus', ()=>{ log+='bf;'; });\
             b.addEventListener('blur', ()=>{ log+='bb;'; });\
             a.focus(); b.focus();\
             return log+'/'+(__zw_native_get_active_element()===b); })()"
        ),
        "af;ab;bf;/true"
    );
    // focus() 幂等：已聚焦时再 focus 不重复派发（spec no-op）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); let n=0;\
             a.addEventListener('focus', ()=>{ n++; });\
             a.focus(); a.focus(); return String(n); })()"
        ),
        "1"
    );
    // blur() 派发 blur 事件 + 清 activeElement（null）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); let log='';\
             a.addEventListener('blur', ()=>{ log+='a-blur;'; });\
             a.focus(); a.blur();\
             return log+'/'+(__zw_native_get_active_element()===null); })()"
        ),
        "a-blur;/true"
    );
    // blur() 非当前焦点 → no-op（不派发 blur，activeElement 不变）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); const b=__zw_native_element_for_id('b');\
             let log=''; a.addEventListener('blur', ()=>{ log+='a-blur;'; });\
             b.focus(); a.blur();\
             return log+'/'+(__zw_native_get_active_element()===b); })()"
        ),
        "/true"
    );
    // focus/blur 非冒泡：child.focus() 不触发 parent focus 监听器（spec：focus/blur 不冒泡）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const p=__zw_native_element_for_id('p'); const a=__zw_native_element_for_id('a');\
             let fired='no'; p.addEventListener('focus', ()=>{ fired='yes'; });\
             a.focus(); return fired; })()"
        ),
        "no"
    );
}
