//! P1b 原生 DOM 绑定测试——textContent / innerHTML / outerHTML setter spec 边界。
//!
//! 覆盖 R3184：三个 setter 均为 spec `[LegacyNullToEmptyString]`——赋 `null` 应视作空串（textContent/
//! innerHTML 清子、outerHTML 移除自身），而非通用 JS ToString 的 `"null"` 文本。`undefined` 不特判，
//! 仍 ToString → `"undefined"`。共享 [`run_script`]（tests.rs，pub(super)）。
//!
//! 镜像 tests.rs：直接建 Isolate + Context + 安装绑定 + 执行脚本（不经 shim 字符串桥）。

use super::tests::run_script;

// ── R3184 textContent setter：spec `LegacyNullToEmptyString`（dom-node-textcontent）──

/// `el.textContent = null` → spec 空串 → 清子（非写 "null" 文本）。验证 getter 回读 "" + 子数 0。
/// 旧实现 `local_value_to_string` 经 JS ToString(null)="null" → 建 "null" Text 节点（子数 1、回读 "null"）。
#[test]
fn native_text_content_setter_null_clears_r3184() {
    let html = r#"<div id="a"><b>x</b>hi</div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.textContent='hello';\
             e.textContent=null;\
             return e.textContent+'/'+e.childNodes.length; })()"
        ),
        "/0"
    );
}

/// `el.textContent = undefined` → spec 不特判（仅 null）→ ToString(undefined)="undefined" → 单 Text 节点。
/// 锁定 null/undefined 区别：null 清子、undefined 写字面 "undefined"。
#[test]
fn native_text_content_setter_undefined_is_string_r3184() {
    let html = r#"<div id="a"><b>x</b></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.textContent=undefined;\
             return e.textContent+'/'+e.childNodes.length; })()"
        ),
        "undefined/1"
    );
}

// ── R3184 innerHTML setter：spec `LegacyNullToEmptyString`（dom-element-innerhtml）──

/// `el.innerHTML = null` → spec 空串 → 清子（非写 "null" 文本）。回读 innerHTML="" + 子数 0。
#[test]
fn native_inner_html_setter_null_clears_r3184() {
    let html = r#"<div id="a"><b>x</b><i>y</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').innerHTML=null;\
             const e=__zw_native_element_for_id('a');\
             return e.innerHTML+'/'+e.childNodes.length; })()"
        ),
        "/0"
    );
}

/// `el.innerHTML = undefined` → spec 不特判 → ToString(undefined)="undefined" → 单 Text 节点。
#[test]
fn native_inner_html_setter_undefined_is_string_r3184() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.innerHTML=undefined;\
             return e.innerHTML+'/'+e.childNodes.length; })()"
        ),
        "undefined/1"
    );
}

// ── R3184 outerHTML setter：spec `LegacyNullToEmptyString`（dom-element-outerhtml）──

/// `el.outerHTML = null` → spec 空串 → 移除自身（非替换为 "null" 文本）。经父节点回读：原 id 已 detach，
/// 父 innerHTML 不再含该元素。
#[test]
fn native_outer_html_setter_null_removes_r3184() {
    let html = r#"<div id="p"><span id="a">x</span><i>y</i></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').outerHTML=null;\
             return __zw_native_element_for_id('p').innerHTML; })()"
        ),
        r#"<i>y</i>"#
    );
}

/// `el.outerHTML = undefined` → spec 不特判 → ToString(undefined)="undefined" → 替换为 "undefined" 文本节点。
#[test]
fn native_outer_html_setter_undefined_is_string_r3184() {
    let html = r#"<div id="p"><span id="a">x</span></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ __zw_native_element_for_id('a').outerHTML=undefined;\
             return __zw_native_element_for_id('p').innerHTML; })()"
        ),
        "undefined"
    );
}

// ── R3185 反射字符串属性 setter：spec `[LegacyNullToEmptyString]`（id/title/lang/accessKey）──
//
// spec HTML：id/title/lang/accessKey IDL 反射为 `[LegacyNullToEmptyString] attribute DOMString`——
// 赋 null 视作空串（写 content 属性为 ""），非通用 ToString 的 "null"。className/dir/aria*/role 为
// plain DOMString **非** LegacyNull（null→"null"）。用 `[`+val+`]` 包裹避免空串返值歧义。

/// `el.id = null` → 空串（LegacyNull），非 "null"。回读 `el.id` === ""。
#[test]
fn native_reflected_id_null_empty_r3185() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.id=null; return '['+e.id+']'; })()"
        ),
        "[]"
    );
}

/// `el.title = null` → 空串（LegacyNull）。
#[test]
fn native_reflected_title_null_empty_r3185() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.title=null; return '['+e.title+']'; })()"
        ),
        "[]"
    );
}

/// `el.lang = null` → 空串（LegacyNull）。
#[test]
fn native_reflected_lang_null_empty_r3185() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.lang=null; return '['+e.lang+']'; })()"
        ),
        "[]"
    );
}

/// `el.accessKey = null` → 空串（LegacyNull）。
#[test]
fn native_reflected_accesskey_null_empty_r3185() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.accessKey=null; return '['+e.accessKey+']'; })()"
        ),
        "[]"
    );
}

/// `el.className = null` → "null"（plain DOMString，**非** LegacyNull）。锁定 LegacyNull 与非的区别。
#[test]
fn native_reflected_classname_null_is_string_r3185() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.className=null; return '['+e.className+']'; })()"
        ),
        "[null]"
    );
}

/// `el.dir = null` → "null"（enumerated，**非** LegacyNull；setter stringifies，getter 简化直读）。
#[test]
fn native_reflected_dir_null_is_string_r3185() {
    let html = r#"<div id="a"></div>"#;
    assert_eq!(
        run_script(
            html,
            "(()=>{ const e=__zw_native_element_for_id('a'); e.dir=null; return '['+e.dir+']'; })()"
        ),
        "[null]"
    );
}
