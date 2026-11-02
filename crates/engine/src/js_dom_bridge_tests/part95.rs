// T42 P40：接口原型 Symbol.toStringTag 描述符 configurable 语义钉。sec_sdk/captcha
// 类脚本内嵌 core-js web.dom-iterables 对全局 DOM 接口原型 defineBuiltIn 重定义
// toStringTag（描述符带 configurable:true），守卫 classof(mi)!==symbol 恒真——
// own **不可配置** toStringTag 即抛 "Cannot redefine property:
// Symbol(Symbol.toStringTag)"，整个 bundle 顶层执行中止（juejin captcha 全 14 处
// 栈证）。WebIDL 接口原型对象 @@toStringTag = {writable:false, enumerable:false,
// configurable:true}（Chrome 实测对照全族 c=true；w/e 缺省 false 与 defineProperty
// 默认一致，故本组只钉 configurable 位点）。WPT 上游无单点用例（断言面拆散于
// 各接口 own-keys 测试），以等价本地沙箱钉常驻：Ke 臂 + 完整 dom-iterables 循环
// reduce + 同族描述符面。
// https://webidl.spec.whatwg.org/#interface-prototype-object

/// T42 钉共用沙箱装配（同 s37_sandbox!）。
macro_rules! t42_sandbox {
    ($html:expr) => {{
        use std::sync::{Arc, Mutex};
        use zero_script_sandbox::{Sandbox, V8Sandbox};

        let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
            persistent_context: true,
            ..Default::default()
        })
        .unwrap();
        sandbox.execute(generate_js_dom_shim()).unwrap();
        let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
        let dom_html = Arc::new(Mutex::new($html.to_string()));
        let page_url = Arc::new(Mutex::new("https://zero.test/t42".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①SVG 列表族：dom-iterables 循环表在册 5 接口（抛错首点 SVGLengthList）。描述符
// configurable:true + core-js Ke 臂（configurable:true 描述符重定义）零抛。
#[test]
fn svg_list_tostringtag_configurable_t42() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var names42 = ['SVGLengthList','SVGNumberList','SVGPointList','SVGStringList','SVGTransformList'];
             globalThis.__r_out = names42.map(function (n) {
               var C = globalThis[n];
               if (!C || !C.prototype) return n + ':absent';
               var d = Object.getOwnPropertyDescriptor(C.prototype, Symbol.toStringTag);
               if (!d) return n + ':none';
               try {
                 Object.defineProperty(C.prototype, Symbol.toStringTag, { value: n, configurable: true });
               } catch (e) { return n + ':THROW'; }
               return n + ':' + d.configurable + ':' + C.prototype[Symbol.toStringTag];
             }).join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r_out").unwrap().value;
    for name in ["SVGLengthList", "SVGNumberList", "SVGPointList", "SVGStringList", "SVGTransformList"] {
        let seg = out.split('|').find(|s| s.starts_with(name)).unwrap_or(&"");
        assert_eq!(
            seg,
            &format!("{name}:true:{name}"),
            "{name} 原型 toStringTag 须 configurable:true 且可被 core-js Ke 臂重定义（WebIDL 接口原型对象）"
        );
    }
}

// ②完整循环 reduce：captcha/index.js dom-iterables 逐名走表（37 名，恒真守卫同款
// ——classof(mi)!==symbol 恒真、无 own 跳过）。全部在册接口零抛即修复判据；
// 修复前此处于 SVGLengthList 止步（红钉活体 loop_done=7）。
#[test]
fn dom_iterables_full_loop_reduce_t42() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var table42 = {CSSRuleList:0,CSSStyleDeclaration:0,CSSValueList:0,ClientRectList:0,DOMRectList:0,DOMStringList:0,DOMTokenList:1,DataTransferItemList:0,FileList:0,HTMLAllCollection:0,HTMLCollection:0,HTMLFormElement:0,HTMLSelectElement:0,MediaList:0,MimeTypeArray:0,NamedNodeMap:0,NodeList:1,PaintRequestList:0,Plugin:0,PluginArray:0,SVGLengthList:0,SVGNumberList:0,SVGPathSegList:0,SVGPointList:0,SVGStringList:0,SVGTransformList:0,SourceBufferList:0,StyleSheetList:0,TextTrackCueList:0,TextTrackList:0,TouchList:0};
             var hit42 = 0, threw42 = '';
             for (var vi in table42) {
               var gi = globalThis[vi], mi = gi && gi.prototype;
               if (!mi) continue;
               try { Object.defineProperty(mi, Symbol.toStringTag, { value: vi, configurable: true }); hit42++; }
               catch (e) { threw42 = vi + ':' + (e && e.message ? e.message : String(e)); break; }
             }
             globalThis.__r_hit = hit42;
             globalThis.__r_threw = threw42;",
        )
        .unwrap();
    let hit: u32 = sandbox.execute("globalThis.__r_hit").unwrap().value.parse().unwrap();
    let threw = sandbox.execute("globalThis.__r_threw").unwrap().value;
    assert_eq!(
        threw, "",
        "core-js dom-iterables 循环全程零抛（Cannot redefine property 族断根）"
    );
    assert!(
        hit >= 15,
        "在册接口原型覆盖数 {hit} 过少——shim 暴露面或循环走表异常"
    );
}

// ③同族描述符面：part05 事件族（Event + 子类泛型位点 MessageEvent）+ canvas/位图族
// （ImageData/CanvasFilter/ImageBitmap/OffscreenCanvas）+ MessagePort + SVG 非列表族
// （SVGAnimatedString/SVGLength/SVGRect/SVGMatrix）——同病位点一并钉 configurable:true。
#[test]
fn sibling_families_tostringtag_configurable_t42() {
    let mut sandbox = t42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var fam42 = ['Event','MessageEvent','ProgressEvent','ImageData','CanvasFilter','ImageBitmap','OffscreenCanvas','MessagePort','SVGAnimatedString','SVGLength','SVGRect','SVGMatrix'];
             globalThis.__r_fam = fam42.map(function (n) {
               var C = globalThis[n];
               if (!C || !C.prototype) return n + ':absent';
               var d = Object.getOwnPropertyDescriptor(C.prototype, Symbol.toStringTag);
               if (!d) return n + ':none';
               try {
                 Object.defineProperty(C.prototype, Symbol.toStringTag, { value: n, configurable: true });
               } catch (e) { return n + ':THROW'; }
               return n + ':' + d.configurable;
             }).join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r_fam").unwrap().value;
    for seg in out.split('|') {
        assert!(
            seg.ends_with(":true"),
            "{seg}——接口原型 toStringTag 须 configurable:true（core-js Ke 臂可重定义；WebIDL 接口原型对象）"
        );
    }
}
