// js-dom M3 R100：**真实 Vue 3**端到端验收（首切片）——DC-2「SPA 框架（React /
// Vue / Svelte 之一）代表性页面可真实加载、渲染、交互」的推进资产。
//
// Vue 来源：上游官方发布产物（jsdelivr CDN 快照，MIT，license 头保留在文件
// 内）——`vue@3.5.13/dist/vue.global.js`（runtime-dom + runtime-core + compiler
// 全量 global build，~550KB）。页面侧经 `globalThis.Vue` 消费（createApp / ref /
// reactive / compile 等），与真实页面 `<script src="vue.global.js">` +
// `const { createApp } = Vue` 等价（模块解析层是 global 形态差异，组件代码面
// 不变）。
//
// 本切片目标（探针驱动，缺口逐个修）：
// - Vue global build 在 shim 环境完整求值（无语法/求值错误）
// - createApp(组件).mount('#app')：模板编译 + 渲染 + patch 到真实 DOM
// - 响应式：ref/reactive state 变更 → 组件重渲染
// - 事件：@click → handler → state 更新 → patch

#[cfg(test)]
mod vue_e2e {
    use zero_webview::{WebView, WebViewConfig};

    /// Vue global build（构建产物，含上游 license 头）。嵌入测试二进制，无网络依赖。
    const VUE_BUNDLE: &str = include_str!("../fixtures/vue/vue.global.js");

    /// 宿主 HTML：`<div id="app">` 挂载点（Vue mount 目标）。含一个 no-op 内联脚本
    /// ——run_page_scripts 对无脚本页面提前返回（不初始化沙箱），后续 execute 会
    /// "no js sandbox"；no-op 脚本保证沙箱 + shim 就绪。
    const VUE_HOST_HTML: &str = r#"<html><head><title>Vue E2E</title></head><body><div id="app"></div><script>
0;
</script></body></html>"#;

    fn run_vue_page(page_script: &str) -> String {
        // R100 装载方式注记：bundle **不** inline 进 HTML——HTML tokenizer 的 script
        // data double-escaped 状态（spec 行为，Chrome 同款）会把 bundle 字符串字面量
        // 中的 `<!--` + `<script` 组合解析成双转义模式，吞掉真正的 `</script>` 闭合
        // 标签（实证：extract 把 bundle + 后续 script 全并进一个 563KB 脚本）。真实
        // 页面从不 inline 大 bundle（Vue 官方部署一律 <script src>）。装载序 =
        // 外链 bundle 的真实执行序：① onerror 捕获脚本（inline）→ ② bundle →
        // ③ 页面脚本。①经 run_page_scripts（顺带完成 shim 注入 + document 建立），
        // ②③经 execute_script_with_dom。
        let html = r#"<html><head><title>Vue E2E</title></head><body><div id="app"></div><script>
globalThis.__errLog = [];
window.onerror = function (m) { globalThis.__errLog.push(String(m)); return true; };
</script></body></html>"#;
        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        wv.load_html(html, None);
        let _ = wv.run_page_scripts();
        // ② bundle（求值错误捕获到 stderr 诊断）。
        if let Err(ref e) = wv.execute_script_with_dom(VUE_BUNDLE) {
            eprintln!("R100 bundle eval error: {e}");
        }
        // ③ 页面脚本。
        if let Err(ref e) = wv.execute_script_with_dom(page_script) {
            eprintln!("R100 page script error: {e}");
        }
        wv.execute_script_with_dom(
            r#"(function(){
var out = [];
out.push('Vue:' + typeof globalThis.Vue);
out.push('report:' + String(globalThis.__vueReport));
out.push('errs:' + String((globalThis.__errLog || []).join(';;')));
var host = document.getElementById('app');
out.push('host-html:' + (host ? String(host.innerHTML).slice(0, 60) : 'no-host'));
out.push('p-late:' + String(host && host.querySelector('p') !== null));
return out.join('|');
})()"#,
        )
        .unwrap_or_else(|_| "EXEC-ERR".to_string())
    }

    /// R100 断言组 A：**Vue 3 真实 mount 落地**——bundle 求值 + createApp +
    /// 模板编译（`{{ msg }}` 插值）+ mount 到真实 DOM + post-flush 可查询。
    ///
    /// R100 修复的引擎缺口（探针实证）：
    /// 1. `generate_dom_api_polyfill` 每次 execute 覆写 `globalThis.document`
    ///    （空 stub 桥）——execute 路径上 getElementById/body 视图全空。改幂等
    ///    安装（shim document 带 `__zwShimInstalled` 标记时不覆盖）。
    /// 2. `execute_script_with_dom` 不收集/不应用 DOM 变更（`__zw_*` 回调未注册，
    ///    JS 侧写静默丢失——mount 后 host 侧查询全 miss）。镜像
    ///    run_page_scripts 机制：注册回调 → 执行 → 应用变更 + 快照同步。
    /// 3. SVG 元素接口构造器缺失（Vue runtime patchSVG 读 `SVGElement`——
    ///    ReferenceError 使 mount 中止）。补 ~36 个 SVG*Element stub（链
    ///    SVGElement → Element.prototype）。
    ///
    /// 装载方式注记（见 run_vue_page）：bundle 不 inline（HTML tokenizer 的
    /// script data double-escaped 状态会吞 `</script>`——bundle 字符串字面量
    /// 含 `<!--` + `<script` 组合，spec 行为 Chrome 同款，真实部署一律外链）。
    #[test]
    fn vue_mount_lands() {
        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        wv.load_html(VUE_HOST_HTML, None);
        let _ = wv.run_page_scripts();
        let bundle = wv.execute_script_with_dom(VUE_BUNDLE);
        assert!(bundle.is_ok(), "Vue global build 须完整求值, got: {:?}", bundle.err());
        let page = wv.execute_script_with_dom(
            r#"(function(){
var app = globalThis.Vue.createApp({
  data: function () { return { msg: 'Hello Vue!' }; },
  template: '<p class="msg">{{ msg }}</p>'
});
var vm = app.mount('#app');
return 'mount:' + typeof vm;
})()"#,
        );
        assert_eq!(
            page.unwrap_or_default().trim(),
            "mount:object",
            "Vue createApp().mount 须成功"
        );
        // post-flush 读数（本 execute 开头注册的回调 Arc 含最新 cached_html）。
        let post = wv
            .execute_script_with_dom(
                r#"(function(){
var host = document.getElementById('app');
var p = host ? host.querySelector('p') : null;
return 'html:' + (host ? String(host.innerHTML) : 'no-host')
  + '|p:' + (p ? p.tagName : 'null')
  + '|class:' + (p ? String(p.className) : 'null')
  + '|text:' + (p ? String(p.textContent) : 'null');
})()"#,
            )
            .unwrap_or_else(|_| "EXEC-ERR".to_string());
        assert_eq!(
            post, "html:<p class=\"msg\">Hello Vue!</p>|p:P|class:msg|text:Hello Vue!",
            "Vue mount 须把编译后的模板渲染进真实 DOM（{{ msg }} 插值求值）, got: {post}"
        );
    }

    /// R100 断言组 B：**Vue 响应式 + 事件**——@click handler 派发 + ref state 更新
    /// 触发重渲染。
    ///
    /// 读数分段（R97 教训：host 侧 mutation 在 execute 结束才应用，同 execute 内
    /// querySelector 拿不到新挂载子树；且跨 execute 的元素 identity 经 R100
    /// selector→handle 反查统一——querySelector 返回原 handle proxy，@click
    /// invoker（注册在 handle proxy 的 listener key 下）可达）：
    /// - execute ②：mount（同步返回 vm）
    /// - execute ③：post-drain 查 button + 读 t0 + 派发 click
    /// - execute ④：post-drain 读 t1（handler 已跑 + 响应式重渲染 commit）
    #[test]
    fn vue_reactive_and_event_lands() {
        let mut wv = WebView::new(WebViewConfig {
            width: 800,
            height: 600,
            ..Default::default()
        });
        wv.load_html(VUE_HOST_HTML, None);
        let _ = wv.run_page_scripts();
        assert!(wv.execute_script_with_dom(VUE_BUNDLE).is_ok(), "bundle 求值");
        let page = wv.execute_script_with_dom(
            r#"(function(){
var Vue = globalThis.Vue;
var app = Vue.createApp({
  data: function () { return { count: 0 }; },
  template: '<button @click="inc">{{ count }}</button>',
  methods: { inc: function () { this.count = this.count + 1; } }
});
var vm = app.mount('#app');
return 'mount:' + typeof vm;
})()"#,
        );
        assert_eq!(
            page.unwrap_or_default().trim(),
            "mount:object",
            "Vue createApp().mount 须成功"
        );
        // post-drain：button 落地 + t0 + 点击派发（Vue 的 @click 经 addEventListener
        // 注册在 button 上——R100 反查后此 proxy 与 invoker 注册的 proxy 同一）。
        let mid = wv
            .execute_script_with_dom(
                r#"(function(){
var host = document.getElementById('app');
var btn = host ? host.querySelector('button') : null;
var t0 = btn ? String(btn.textContent) : 'null';
if (btn) {
  var ev = new Event('click', { bubbles: true, cancelable: true });
  btn.dispatchEvent(ev);
}
return 'btn:' + (btn ? btn.tagName : 'null') + '|t0:' + t0 + '|dispatch:' + String(!!btn);
})()"#,
            )
            .unwrap_or_else(|_| "EXEC-ERR".to_string());
        assert_eq!(
            mid, "btn:BUTTON|t0:0|dispatch:true",
            "Vue mount 后 button 落地（post-drain）, got: {mid}"
        );
        // post-drain：handler 已跑 + 响应式重渲染 commit（count 0→1）。
        let post = wv
            .execute_script_with_dom(
                r#"(function(){
var host = document.getElementById('app');
var btn = host ? host.querySelector('button') : null;
return 'text:' + (btn ? String(btn.textContent) : 'null');
})()"#,
            )
            .unwrap_or_else(|_| "EXEC-ERR".to_string());
        assert_eq!(
            post, "text:1",
            "Vue @click handler + 响应式重渲染（count 0→1）, got: {post}"
        );
    }
}
