// js-dom M3（R90）：Web Components 端到端验收资产——customElements define/升级 +
// lifecycle 四件套 + 属性反射 + Shadow DOM 基础 + 自定义事件，经 WebView 真实页面脚本
// 管线（load_html + run_page_scripts）跑通。断言收集进 `__wcReport` 字符串逐组核验，
// 是 DC-2「WC 端到端跑通」的首个常驻资产（make test 内运行）。
//
// 页面脚本形态刻意贴近真实 WC 库用法：class extends HTMLElement、customElements.define、
// observedAttributes、connectedCallback 建 shadow 子树、attributeChangedCallback 反射。

#[cfg(test)]
mod wc_e2e {
    use zero_webview::{WebView, WebViewConfig};

    fn run_wc_page(page_script: &str) -> String {
        let html = format!(
            r#"<html><head><title>WC E2E</title></head><body><div id="host"></div><script>
{page_script}
</script></body></html>"#
        );
        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        wv.load_html(&html, None);
        // 页面脚本在 load 后同步执行（run_page_scripts 走完整 shim + 回调管线）。
        let _ = wv.run_page_scripts_strict();
        // 收集阶段也在页面侧完成（报告字符串拼接），此处只读回。
        wv.execute_script_with_dom("(typeof globalThis.__wcReport === 'string') ? globalThis.__wcReport : 'NO-REPORT'")
            .unwrap_or_else(|_| "EXEC-ERR".to_string())
    }

    /// 断言组 1：define + createElement 升级 + lifecycle connected/disconnected。
    /// 已知限制（R90）：class ctor 体无法在既有 proxy 上重执行（JS 语义限制），
    /// 升级 = setPrototypeOf + connectedCallback 承载初始化（imperative WC 模式）。
    #[test]
    fn wc_define_upgrade_and_lifecycle() {
        let report = run_wc_page(
            r#"
var log = [];
class MyGreeting extends HTMLElement {
  connectedCallback() { log.push('connected'); }
  disconnectedCallback() { log.push('disconnected'); }
}
customElements.define('my-greeting', MyGreeting);
var el = document.createElement('my-greeting');
log.push('instanceof:' + (el instanceof MyGreeting));
log.push('tagName:' + el.tagName);
var host = document.getElementById('host');
host.appendChild(el);
log.push('after-append:' + el.isConnected);
host.removeChild(el);
log.push('after-remove:' + el.isConnected);
globalThis.__wcReport = log.join('|');
"#,
        );
        assert_eq!(
            report, "instanceof:true|tagName:MY-GREETING|connected|after-append:true|disconnected|after-remove:false",
            "WC define/升级/lifecycle 报告须逐项匹配，got: {report}"
        );
    }

    /// 断言组 2：observedAttributes + attributeChangedCallback 三参与 remove 路径。
    #[test]
    fn wc_observed_attributes_and_reflection() {
        let report = run_wc_page(
            r#"
var log = [];
class MyBadge extends HTMLElement {
  static get observedAttributes() { return ['level']; }
  attributeChangedCallback(name, oldV, newV) {
    log.push('attr:' + name + ':' + oldV + '->' + newV);
  }
  connectedCallback() { this.textContent = 'badge-' + (this.getAttribute('level') || 'none'); }
}
customElements.define('my-badge', MyBadge);
var el = document.createElement('my-badge');
el.setAttribute('level', 'gold');
log.push('set-attr');
el.removeAttribute('level');
var host = document.getElementById('host');
host.appendChild(el);
log.push('text:' + el.textContent);
globalThis.__wcReport = log.join('|');
"#,
        );
        // 首次 setAttribute：old null -> gold；remove：gold -> null（缺失属性 no-op 不派发）。
        assert!(
            report.contains("attr:level:null->gold"),
            "首射须派 attributeChanged (null->gold)，got: {report}"
        );
        assert!(
            report.contains("attr:level:gold->null"),
            "移除须派 attributeChanged (gold->null)，got: {report}"
        );
        assert!(
            report.contains("text:badge-none"),
            "connectedCallback 须按当前属性渲染文本，got: {report}"
        );
    }

    /// 断言组 3：Shadow DOM attachShadow + shadowRoot open + 子树文本。
    /// （attachShadow 在 connectedCallback 内执行——ctor 体不重跑的 R90 已知限制下
    /// 的 imperative 模式；shadowRoot 在 append 后可读。）
    #[test]
    fn wc_shadow_dom_basic() {
        let report = run_wc_page(
            r#"
var log = [];
class MyCard extends HTMLElement {
  connectedCallback() {
    var root = this.attachShadow({ mode: 'open' });
    var inner = document.createElement('span');
    inner.textContent = 'card-inner';
    root.appendChild(inner);
    log.push('shadow-attached');
  }
}
customElements.define('my-card', MyCard);
var el = document.createElement('my-card');
var host = document.getElementById('host');
host.appendChild(el);
log.push('has-shadow:' + (el.shadowRoot !== null && el.shadowRoot !== undefined));
if (el.shadowRoot) {
  var span = el.shadowRoot.querySelector('span');
  log.push('shadow-text:' + (span ? span.textContent : 'no-span'));
}
globalThis.__wcReport = log.join('|');
"#,
        );
        assert!(
            report.contains("shadow-attached"),
            "connectedCallback 内 attachShadow 须可执行，got: {report}"
        );
        assert!(
            report.contains("has-shadow:true"),
            "open shadowRoot 须可读，got: {report}"
        );
        assert!(
            report.contains("shadow-text:card-inner"),
            "shadow 子树查询须命中（querySelector + textContent），got: {report}"
        );
    }

    /// 断言组 4：customElements.get/getName 反查 + whenDefined resolve（R91 闭环：
    /// define 触发 pending resolve 后经 run_page_scripts 的微任务 checkpoint flush，
    /// 第二次 execute 读回 __futureResolved）。
    #[test]
    fn wc_registry_lookup_and_when_defined() {
        let page_script = r#"
var log = [];
class MyWidget extends HTMLElement {}
customElements.define('my-widget', MyWidget);
log.push('get:' + (customElements.get('my-widget') === MyWidget));
log.push('get-miss:' + (customElements.get('no-such') === undefined || customElements.get('no-such') === null));
log.push('getName:' + (customElements.getName(MyWidget) === 'my-widget'));
customElements.whenDefined('future-el').then(function () { globalThis.__futureResolved = 'yes'; });
customElements.define('future-el', class extends HTMLElement {});
log.push('whenDefined-pending');
globalThis.__wcReport = log.join('|');
"#;
        let html = format!(
            r#"<html><head><title>WC E2E</title></head><body><div id="host"></div><script>
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
        let report = wv
            .execute_script_with_dom(
                "(typeof globalThis.__wcReport === 'string') ? globalThis.__wcReport : 'NO-REPORT'",
            )
            .unwrap_or_else(|_| "EXEC-ERR".to_string());
        assert!(
            report.contains("get:true") && (report.contains("getName:my-widget") || report.contains("getName:true")),
            "registry get/getName 反查，got: {report}"
        );
        assert!(
            report.contains("get-miss:true"),
            "未定义 tag 的 get 须返 null/undefined，got: {report}"
        );
        // R91：whenDefined 的 promise resolve 经第二次脚本执行（微任务 checkpoint 已跑）。
        let resolved = wv
            .execute_script_with_dom(
                "(typeof globalThis.__futureResolved === 'string') ? globalThis.__futureResolved : 'no'",
            )
            .unwrap_or_else(|_| "EXEC-ERR".to_string());
        assert_eq!(
            resolved, "yes",
            "whenDefined pending 须在 define 时 resolve（flush 后读到），got: {resolved}"
        );
    }

    /// 断言组 5：自定义事件 dispatchEvent + shadow 内 listener（事件面端到端）。
    #[test]
    fn wc_custom_event_dispatch() {
        let report = run_wc_page(
            r#"
var log = [];
class MyEmitter extends HTMLElement {
  connectedCallback() {
    var self = this;
    this.addEventListener('ping', function (e) {
      log.push('heard:' + e.detail.val);
    });
    var ev = new CustomEvent('ping', { detail: { val: 42 }, bubbles: true });
    this.dispatchEvent(ev);
  }
}
customElements.define('my-emitter', MyEmitter);
var el = document.createElement('my-emitter');
document.getElementById('host').appendChild(el);
globalThis.__wcReport = log.join('|');
"#,
        );
        assert_eq!(
            report, "heard:42",
            "connectedCallback 内 dispatch CustomEvent 须被自身 listener 收到 detail，got: {report}"
        );
    }

    /// 断言组 6（R92）：lit 风格模板渲染——innerHTML 模板（含插值结果）写入 shadow
    /// root，子树查询（tag/class/id）+ 二次渲染覆盖 + 嵌套层可达。
    #[test]
    fn wc_lit_style_template_render() {
        let report = run_wc_page(
            r#"
var log = [];
function render(root, title, val) {
  root.innerHTML = '<style>:host { display: block; }</style>'
    + '<div class="card"><h2 id="t">' + title + '</h2><p class="v">' + val + '</p>'
    + '<span class="nested"><em>deep</em></span></div>';
}
class MyView extends HTMLElement {
  connectedCallback() {
    render(this.attachShadow({ mode: 'open' }), 'Title-42', 'val-x');
  }
}
customElements.define('my-view', MyView);
var el = document.createElement('my-view');
document.getElementById('host').appendChild(el);
var root = el.shadowRoot;
log.push('has-root:' + !!root);
var h2 = root.querySelector('h2');
log.push('h2:' + (h2 ? h2.textContent : 'null'));
log.push('byId:' + (root.querySelector('#t') ? root.querySelector('#t').textContent : 'null'));
log.push('cls:' + (root.querySelector('.v') ? root.querySelector('.v').textContent : 'null'));
// 嵌套层（depth-2/3）+ 组合器：
var em = root.querySelector('.nested em');
log.push('deep:' + (em ? em.textContent : 'null'));
var cardKids = root.querySelector('.card').childNodes.length;
log.push('cardKids:' + cardKids);
var style = root.querySelector('style');
log.push('style:' + (style ? 'found' : 'null'));
// 二次渲染（re-render 覆盖）：
render(root, 'Title-99', 'val-y');
log.push('re-h2:' + (root.querySelector('h2') ? root.querySelector('h2').textContent : 'null'));
globalThis.__wcReport = log.join('|');
"#,
        );
        let expected =
            "has-root:true|h2:Title-42|byId:Title-42|cls:val-x|deep:deep|cardKids:3|style:found|re-h2:Title-99";
        assert_eq!(
            report, expected,
            "lit 风格模板渲染 + shadow 子树查询 + 二次渲染，got: {report}"
        );
    }

    /// 断言组 7（R93）：CE 原型方法可达——custom element 的用户 prototype 方法
    /// （`MyEl.prototype.bump`，lit/stencil 组件形态）经 proxy 派发可调用且 this 绑定
    /// 正确。R90 的 getPrototypeOf trap CE registry 分支已把链顶换成 ctor.prototype，
    /// 但 get trap 对方法名恒返 undefined 使方法不可达（`el.bump is not a function`）；
    /// R93 把 get trap 的原型链回落从仅 SCREAMING_SNAKE 常量放宽到全部未命中属性
    /// （沿链只取 own 命中，限 8 层）。
    #[test]
    fn wc_prototype_method_dispatch() {
        let report = run_wc_page(
            r#"
var log = [];
class MyCounter extends HTMLElement {
  bump(step) { this._count += (step || 1); return this._count; }
  get doubled() { return this._count * 2; }
}
customElements.define('my-counter', MyCounter);
var el = document.createElement('my-counter');
el._count = 0;
log.push('method:' + (typeof el.bump));
log.push('call:' + el.bump());
log.push('call-arg:' + el.bump(5));
// 原型方法内 this 为元素自身（写 expando 经 setter 持久化、getter 读回）：
log.push('this-ok:' + (el._count === 6));
// 原型 getter（accessor）沿链命中：
log.push('getter:' + el.doubled);
// instance 方法重写（per-instance expando 优先于原型）：
el.bump = function () { return 'overridden'; };
log.push('override:' + el.bump());
// 未定义成员仍是 undefined（不误报）：
log.push('miss:' + (el.noSuchMethod === undefined));
globalThis.__wcReport = log.join('|');
"#,
        );
        let expected = "method:function|call:1|call-arg:6|this-ok:true|getter:12|override:overridden|miss:true";
        assert_eq!(
            report, expected,
            "CE 原型方法/ getter 须经 proxy 派发可达且 this 绑定正确，got: {report}"
        );
    }
}
