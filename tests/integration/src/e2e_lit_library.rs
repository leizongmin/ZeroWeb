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
// 已验收面（本切片）：
// - 真实 lit bundle 在 shim 环境完整求值（无语法/求值错误，LitElement/html 可用）
// - LitElement 子类 define + createElement 升级 + constructor 体以元素为 this 执行
//   （R94 Proxy-ctor 桥 + R95 `constructor` 短路——lit 实例方法读
//   `this.constructor.elementProperties`）
// - LitElement/HTMLElement instanceof + lit 内部状态（_$ES Promise / renderRoot）
// - connectedCallback → createRenderRoot → attachShadow（shadow root 建立）
// - `<template>`.content（R95 新增面，lit-html Template 管线前置原语）
//
// 已知剩余（下一切片，R95 诊断记录在 master.md）：
// - 异步 update 链：lit 的 `await this._$ES` 恢复依赖 `this.enableUpdating` 实例
//   属性（Promise executor 内赋值）能正确读回——e2e 实证该 expando 读被 get trap
//   中间分支遮蔽（读到原型 noop）→ 首渲染不落地（shadow root 恒空）。expando
//   优先化（own-shadow 语义）与 define 期 observedAttributes→finalize 触发是
//   下一步（后者会暴露 `hasOwnProperty` 在 get trap 长链上的可达性缺口——
//   bisect 已定位控制流在 part03→part04 中途被吞，具体分支待查）。

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
}
