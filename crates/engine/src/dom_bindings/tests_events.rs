//! P1b 原生 DOM 绑定测试——行为方法 + 事件族（拆自 tests_dom_api.rs，rule 5 <2000 行；R3147+）。
//!
//! 覆盖：element.click()（R3147，dispatchEvent 重构无回归守卫）、element.focus() / blur() +
//! document.activeElement（R3148）、focusin/focusout 冒泡焦点事件（R3149，焦点事件模型闭合）。
//! 共享 [`run_script`]（tests.rs，pub(super)）。镜像 tests.rs：直接建 Isolate+Context + 安装绑定 +
//! 执行脚本（不经 shim 字符串桥）。

use super::tests::run_script;

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

/// R3149 focusin/focusout（冒泡焦点事件）：focus()/blur() 按 spec 焦点事件序列派发 focusout（旧，冒泡）→
/// focusin（new，冒泡）→ blur（旧）→ focus（new）。focusin/focusout 冒泡到祖先（焦点事件委托唯一手段，
/// jQuery/a11y 库惯用 `document.addEventListener('focusin', ...)`）。闭合焦点事件模型（polyfill 旧不派发）。
#[test]
fn native_element_focusin_focusout_r3149() {
    let html = r#"<div id="p"><span id="a"></span><span id="b"></span></div>"#;
    // focusin 冒泡到祖先：a.focus() → focusin(a) 上溯 p，p 的 focusin 监听器触发（e.target=a）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const p=__zw_native_element_for_id('p'); const a=__zw_native_element_for_id('a');\
             let log=''; p.addEventListener('focusin', e=>{ log+=e.target.tagName+'-fin;'; });\
             a.focus(); return log; })()"
        ),
        "SPAN-fin;"
    );
    // spec 焦点事件序列：a.focus()（无旧焦点）→ focusin(a)+focus(a)；b.focus()（旧=a）→
    // focusout(a)+focusin(b)+blur(a)+focus(b)。完整序：focusin@a;focus@a;focusout@a;focusin@b;blur@a;focus@b;。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const a=__zw_native_element_for_id('a'); const b=__zw_native_element_for_id('b');\
             let log='';\
             ['focusin','focusout','focus','blur'].forEach(t=>{\
               a.addEventListener(t, e=>{ log+=t+'@'+e.target.id+';'; });\
               b.addEventListener(t, e=>{ log+=t+'@'+e.target.id+';'; }); });\
             a.focus(); b.focus(); return log; })()"
        ),
        "focusin@a;focus@a;focusout@a;focusin@b;blur@a;focus@b;"
    );
    // blur() 派发 focusout（冒泡到祖先）：a.focus(); a.blur() → focusout(a) 上溯 p 触发 p.focusout 监听器。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const p=__zw_native_element_for_id('p'); const a=__zw_native_element_for_id('a');\
             let log=''; p.addEventListener('focusout', e=>{ log+=e.target.id+'-fout;'; });\
             a.focus(); a.blur(); return log; })()"
        ),
        "a-fout;"
    );
}

/// R3170 派发期间 `removeEventListener` 的监听器 skip（spec DOM「inner invoke」removed 标志）。
/// l1 先注册并移除 l2，l2 后注册；dispatch 快照=[l1,l2]，l1 运行后 l2 已删 → 须 skip（非调用）。
#[test]
fn native_dispatch_skips_listener_removed_during_dispatch_r3170() {
    let html = r#"<div id="x"></div>"#;
    // l1 移除 l2（在其被调用前）→ l2 须 skip。期望 log 'l1;'（非 'l1;l2;'）。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('x'); let log='';\
             const l2=()=>{ log+='l2;'; };\
             el.addEventListener('click', ()=>{ log+='l1;'; el.removeEventListener('click', l2); });\
             el.addEventListener('click', l2);\
             el.dispatchEvent({type:'click'}); return log; })()"
        ),
        "l1;"
    );
    // 对照：未被移除的监听器仍按注册序触发（l1; l2;）——确认 skip 仅针对 removed，非误伤。
    assert_eq!(
        run_script(
            html,
            "(()=>{ const el=__zw_native_element_for_id('x'); let log='';\
             el.addEventListener('click', ()=>{ log+='l1;'; });\
             el.addEventListener('click', ()=>{ log+='l2;'; });\
             el.dispatchEvent({type:'click'}); return log; })()"
        ),
        "l1;l2;"
    );
}
