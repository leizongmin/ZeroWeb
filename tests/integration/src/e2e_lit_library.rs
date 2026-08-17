// js-dom M3 R95：**真实 lit 库**端到端验收（首切片）——DC-2「Web Components
// （customElements + lit 之一）代表性页面可真实运行」的推进资产。
//
// lit 来源：上游官方发布产物（jsdelivr CDN 快照，BSD-3-Clause，license 头保留在
// bundle 内）——`@lit/reactive-element 2.1.1` + `lit-html 3.3.2` + `lit-element 4.2.1`
// 五模块，经构建脚本打成单一 classic script（ESM import/export 剥离 + 每 IIFE 作用域
// 显式交接绑定），产物 `fixtures/lit/lit.bundle.js`（~17.8KB）。页面侧经
// `globalThis.lit` 消费（LitElement/html/svg/render/...），与真实页面 `import {
// LitElement, html } from 'lit'` 等价（模块解析层是 bundle 形态差异，组件代码面不变）。
//
// 已验收面（R95 首切片 → R97 首渲染落地）：
// - 真实 lit bundle 在 shim 环境完整求值（无语法/求值错误，LitElement/html 可用）
// - LitElement 子类 define + createElement 升级 + constructor 体以元素为 this 执行
//   （R94 Proxy-ctor 桥 + R95 `constructor` 短路——lit 实例方法读
//   `this.constructor.elementProperties`）
// - LitElement/HTMLElement instanceof + lit 内部状态（_$ES Promise / renderRoot）
// - connectedCallback → createRenderRoot → attachShadow（shadow root 建立）
// - `<template>`.content（R95 新增面，lit-html Template 管线前置原语）
// - **首渲染落地（R97）**：performUpdate → render → lit-html render() → Template
//   构建 → importNode → TreeWalker parts 提取 → insertBefore commit 全链贯通，
//   插值内容真实出现在 shadow root（组 C）；lit-html render() 直测容器路径（组 D）。
//   R97 修复的四个缺口见组 C doc comment（walker 跨树重定位 / insertBefore
//   registry / 无 handle fragment 插入 / _zwMEl hasAttributes）。
//
// 已知剩余（后续切片）：
// - lit 响应式更新链：property set → requestUpdate → 二次 render 的 diff commit
//   （首渲染后的增量更新面）；M3 SPA 面（React/Vue 之一）评估。

#[cfg(test)]
mod lit_e2e {
    use zero_webview::{WebView, WebViewConfig};

    /// lit bundle（构建产物，含上游 license 头）。嵌入测试二进制，无网络依赖。
    const LIT_BUNDLE: &str = include_str!("../fixtures/lit/lit.bundle.js");

    fn run_lit_page(page_script: &str) -> String {
        let html = format!(
            r#"<html><head><title>Lit E2E</title></head><body><div id="host"></div><script>
{LIT_BUNDLE}
</script><script>
{page_script}
</script></body></html>"#
        );
        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        wv.load_html(&html, None);
        let _ = wv.run_page_scripts_strict();
        wv.execute_script_with_dom(
            "(typeof globalThis.__litReport === 'string') ? globalThis.__litReport : 'NO-REPORT'",
        )
        .unwrap_or_else(|_| "EXEC-ERR".to_string())
    }

    /// 断言组 A：真实 LitElement 子类全链路——bundle 求值 + define + createElement
    /// 升级 + constructor 体以元素为 this 执行（R94 桥）+ `this.constructor` 可达
    /// （R95 短路）+ instanceof 双层 + lit 内部状态 + shadow root 建立。
    #[test]
    fn lit_component_chain() {
        let report = run_lit_page(
            r#"
var log = [];
// bundle 求值面：真实 lit 五模块在 shim 环境完整求值（任何顶层错误都会中断后续）。
log.push('bundle-lit:' + [typeof globalThis.lit, typeof globalThis.lit.LitElement, typeof globalThis.lit.html].join(','));
const { LitElement, html } = globalThis.lit;
class GreetingEl extends LitElement {
  constructor() {
    super();
    this._built = 'ctor';
  }
  static get properties() {
    return { name: { type: String } };
  }
  render() {
    return html`<p class="greet">Hello, ${this.name}!</p>`;
  }
}
customElements.define('greeting-el', GreetingEl);
var el = document.createElement('greeting-el');
el.name = 'ZeroWeb';
document.getElementById('host').appendChild(el);
// constructor 体以元素为 this 执行（R94 桥）；this.constructor 可达（R95 短路）。
log.push('ctor-init:' + (el._built === 'ctor'));
log.push('el-constructor:' + (el.constructor === GreetingEl));
// instanceof 双层：用户类 + HTMLElement。
log.push('instanceof-lit:' + (el instanceof LitElement));
log.push('instanceof-htmlel:' + (el instanceof HTMLElement));
// lit 内部状态：_$ES（ctor 建 enableUpdating promise）+ renderRoot（connected
// Callback → createRenderRoot → attachShadow 建立）。
log.push('lit-internals:' + [typeof el._$ES, typeof el.renderRoot].join(','));
// shadow root 建立（attachShadow）。
log.push('has-shadow:' + (el.shadowRoot !== null && el.shadowRoot !== undefined));
globalThis.__litReport = log.join('|');
"#,
        );
        let expected = concat!(
            "bundle-lit:object,function,function|",
            "ctor-init:true|el-constructor:true|",
            "instanceof-lit:true|instanceof-htmlel:true|",
            "lit-internals:object,object|",
            "has-shadow:true",
        );
        assert_eq!(
            report, expected,
            "真实 lit 组件链（bundle 求值 + ctor 桥 + constructor 可达 + shadow root）须通过，got: {report}"
        );
    }

    /// 断言组 B：`<template>`.content（R95 新增面）——lit-html Template 管线的
    /// 前置原语：template 元素 innerHTML= 解析后 content 返回 DocumentFragment
    /// 形态视图（nodeType 11 / childNodes / firstChild）。
    #[test]
    fn template_content_fragment_view() {
        let report = run_lit_page(
            r#"
var log = [];
var t = document.createElement('template');
t.innerHTML = '<span id="a">x</span><b>y</b>';
var c = t.content;
log.push('frag:' + [c.nodeType, c.nodeName].join(','));
log.push('kids:' + c.childNodes.length);
log.push('first:' + (c.firstChild ? c.firstChild.tagName : 'null'));
log.push('query-a:' + (c.childNodes[0] && c.childNodes[0].id));
log.push('empty:' + document.createElement('template').content.childNodes.length);
globalThis.__litReport = log.join('|');
"#,
        );
        let expected = "frag:11,#document-fragment|kids:2|first:SPAN|query-a:a|empty:0";
        assert_eq!(
            report, expected,
            "template.content 须为 fragment 视图（lit-html Template 前置原语），got: {report}"
        );
    }

    /// R97 断言组 C：**lit 首渲染落地**（R95 诊断的异步 update 链阻塞全链打通）——
    /// 真实 LitElement 组件（properties + html`` 插值 + shadow root）经
    /// `performUpdate → render → lit-html render() → Template 构建 → importNode →
    /// TreeWalker parts 提取 → insertBefore commit` 全链，首渲染 DOM 落地到
    /// renderRoot。读数在第二次 execute（post-drain——第一次 execute 末
    /// microtask checkpoint 排水后，`await this._$ES` 已 resume）。
    ///
    /// R97 修复的四个缺口（探针实证定位）：
    /// 1. TreeWalker `currentNode` 跨树重定位（lit-html 单全局 walker 经
    ///    `P.currentNode = fragment` 遍历 template parts——旧 order 快照不含 root
    ///    外节点，重定位后 nextNode 仍从 document 头走）
    /// 2. `insertBefore(node, null)` handle 父不记 registry（marker 插入后容器
    ///    childNodes 视图漏子）
    /// 3. 无 handle fragment 视图（template.content 派生）插入 no-op
    ///    （imported fragment 无 `__zwHandle` 落到静默分支）
    /// 4. `_zwMEl` 解析子缺 `hasAttributes()`/`getAttributeNames()`
    ///    （lit Template 属性 parts 提取直接 TypeError）
    #[test]
    fn lit_first_render_lands() {
        let html_doc = format!(
            r#"<html><head><title>Lit E2E</title></head><body><div id="host"></div><script>
{}
</script><script>
const {{ LitElement, html }} = globalThis.lit;
class GreetingEl extends LitElement {{
  constructor() {{ super(); this._built = 'ctor'; }}
  static get properties() {{ return {{ name: {{ type: String }} }}; }}
  render() {{ return html`<p class="greet">Hello, ${{this.name}}!</p>`; }}
}}
customElements.define('greeting-el', GreetingEl);
var el = document.createElement('greeting-el');
el.name = 'ZeroWeb';
document.getElementById('host').appendChild(el);
globalThis.__elForLater = el;
</script></body></html>"#,
            include_str!("../fixtures/lit/lit.bundle.js")
        );
        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        wv.load_html(&html_doc, None);
        let _ = wv.run_page_scripts_strict();
        let post = wv
            .execute_script_with_dom(
                r#"(function(){
var el = globalThis.__elForLater;
var rr = el.renderRoot;
var out = [];
out.push('pending:' + String(el.isUpdatePending));
out.push('hasUpdated:' + String(el.hasUpdated));
out.push('rr-kids:' + (rr && rr.childNodes ? rr.childNodes.length : 'no-rr'));
var p = rr ? rr.querySelector('p') : null;
out.push('p-tag:' + (p ? p.tagName : 'null'));
out.push('p-class:' + (p ? String(p.className) : 'null'));
out.push('p-text:' + (p ? String(p.textContent) : 'null'));
return out.join('|');
})()"#,
            )
            .unwrap_or_else(|_| "EXEC-ERR".to_string());
        let expected = "pending:false|hasUpdated:true|rr-kids:2|p-tag:P|p-class:greet|p-text:Hello, ZeroWeb!";
        assert_eq!(
            post, expected,
            "lit 首渲染须落地到 renderRoot（R95 异步 update 链阻塞全链打通后的验收）, got: {post}"
        );
    }

    /// R97 断言组 D：lit-html `render()` 直测——不经 LitElement，直接
    /// `render(html`…`, container)`，验证 Template 构建 + importNode +
    /// TreeWalker parts 提取 + insertBefore commit 全链在普通容器上的行为
    /// （与组 C 的 shadow root 路径互补）。
    #[test]
    fn lit_html_render_direct() {
        let report = run_lit_page(
            r#"
var log = [];
const { html, render } = globalThis.lit;
var container = document.createElement('div');
try {
  var result = html`<p class="greet">Hello, World!</p>`;
  render(result, container);
  log.push('kids:' + container.childNodes.length);
  var p = container.querySelector('p');
  log.push('p:' + (p ? p.tagName : 'null'));
  log.push('text:' + (p ? String(p.textContent) : 'null'));
} catch (e) {
  log.push('ERR:' + e);
}
// marker/insertBefore 基础面（lit ChildPart commit 原语）。
var m = document.createComment('m');
var c2 = document.createElement('div');
var ins = c2.insertBefore(m, null);
log.push('ins:' + [c2.childNodes.length, String(ins === m), String(m.parentNode === c2)].join(','));
globalThis.__litReport = log.join('|');
"#,
        );
        let expected = "kids:2|p:P|text:Hello, World!|ins:1,true,true";
        assert_eq!(
            report, expected,
            "lit-html render() 直测（Template/TreeWalker/insertBefore commit 全链）, got: {report}"
        );
    }

    /// R98 断言组 E：**lit 响应式更新链**——首渲染后 `el.name = 'Updated!'` 触发
    /// requestUpdate → 二次 render → ChildPart 文本 commit（真实响应式闭环）。
    /// 三段式：① 首渲染 + 诊断 accessor 安装；② property set（本段末 checkpoint
    /// 排水 _$EP 微任务）；③ post-drain 读二次 render 结果。
    ///
    /// R98 修复的三个缺口：
    /// 1. `customElements.define` 不读 `ctor.observedAttributes`（spec define step 5
    ///    Get）——lit 的静态 getter 内调 `finalize()` → `createProperty` 装 prototype
    ///    accessor；不读则 accessor 从未安装，property set 不触发 requestUpdate
    /// 2. set trap 不派发原型链 accessor setter——lit setter 内调
    ///    `this.requestUpdate`；旧恒落 expando 存储（uc-changed:false）
    /// 3. symbol-keyed 写丢失（lit accessor fallback `this[s] = v` 以 Symbol 存值）
    ///    与首层原型 accessor getter 优先级（旧被 shim 反射属性分支先吞——首渲染
    ///    插值空串）
    #[test]
    fn lit_reactive_update_lands() {
        let html_doc = format!(
            r#"<html><head><title>Lit E2E</title></head><body><div id="host"></div><script>
{}
</script><script>
const {{ LitElement, html }} = globalThis.lit;
class GreetingEl extends LitElement {{
  static get properties() {{ return {{ name: {{ type: String }} }}; }}
  render() {{ return html`<p class="greet">Hello, ${{this.name}}!</p>`; }}
}}
customElements.define('greeting-el', GreetingEl);
var el = document.createElement('greeting-el');
el.name = 'ZeroWeb';
document.getElementById('host').appendChild(el);
globalThis.__elForLater = el;
</script></body></html>"#,
            include_str!("../fixtures/lit/lit.bundle.js")
        );
        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        wv.load_html(&html_doc, None);
        let _ = wv.run_page_scripts_strict();
        // 第二段：触发响应式更新。本段末 checkpoint 排水 update 微任务。
        let mid = wv
            .execute_script_with_dom(
                r#"(function(){
var el = globalThis.__elForLater;
var out = [];
var p1 = el.renderRoot ? el.renderRoot.querySelector('p') : null;
out.push('t1:' + (p1 ? String(p1.textContent) : 'null'));
var uc = el.updateComplete;
el.name = 'Updated!';
var uc2 = el.updateComplete;
out.push('uc-changed:' + String(uc !== uc2));
out.push('pending-at-set:' + String(el.isUpdatePending));
return out.join('|');
})()"#,
            )
            .unwrap_or_else(|_| "EXEC-ERR".to_string());
        assert_eq!(
            mid, "t1:Hello, ZeroWeb!|uc-changed:true|pending-at-set:true",
            "lit 首渲染正确 + property set 触发 requestUpdate（accessor setter 派发）, got: {mid}"
        );
        // 第三段：post-drain 读二次 render 结果。
        let post = wv
            .execute_script_with_dom(
                r#"(function(){
var el = globalThis.__elForLater;
var out = [];
var p = el.renderRoot ? el.renderRoot.querySelector('p') : null;
out.push('t2:' + (p ? String(p.textContent) : 'null'));
out.push('pending:' + String(el.isUpdatePending));
return out.join('|');
})()"#,
            )
            .unwrap_or_else(|_| "EXEC-ERR".to_string());
        assert_eq!(
            post, "t2:Hello, Updated!!|pending:false",
            "lit 响应式二次 render 落地（ChildPart 文本 commit）, got: {post}"
        );
    }
}
