// T72（site-optimizer perf-mem T72/P42）：iframe win.navigator 平面对象语义常驻钉。
// 缺陷面（T66 DF 首轮 N1 入池 P42）：part05.js _zwMakeIframeWin 工厂的 iframe
// win.navigator 原为纯字面量 `navigator: { serviceWorker: serviceWorker }`——无
// Navigator 接口/原型链/品牌，[[Prototype]] 直落 Object.prototype。主窗面 T66 已修
//（part02.js，钉在 part95.rs navigator_platform_object_t66），iframe 面漏网：原型
// 定向写（T65 定谳的「从根沿原型走 s 步再存储」canary opcode，运行目录 evidence/
// t65-diagnosis.md）经 iframe navigator 落 Object.prototype 共享可枚举面——同缺陷
// 第二产点。修复：[[Prototype]] 挂 globalThis.Navigator.prototype（同 realm 单例），
// own 键面（serviceWorker 单键）恒等，仅原型换位；serviceWorker wrapper 实体与
// 创建期 controller-on-load 时序（iframeServiceWorkerCreationControllerId）不变。
// 钉五断言面：原型链 / 品牌 / Illegal constructor 双臂（①） / serviceWorker 槽恒一
//（②） / 走链吸收（③）。WPT 上游无单点用例（原型/品牌面拆散于各接口 own-keys
// 测试），以等价本地沙箱钉常驻（part95 t42_sandbox 同款形制）。
// https://html.spec.whatwg.org/multipage/system-state.html#the-navigator-object
// https://webidl.spec.whatwg.org/#interface-object
// https://webidl.spec.whatwg.org/#interface-prototype-object

/// T72 钉共用沙箱装配（同 part95 t42_sandbox!）。
macro_rules! t72_sandbox {
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
        let page_url = Arc::new(Mutex::new("https://zero.test/t72".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①核心语义钉：iframe navigator 原型链恒等 + '[object Navigator]' 品牌 + instanceof
// + Illegal constructor 双臂（调用臂与 new 臂均 TypeError「Illegal constructor」）+
// own 键面不扩（serviceWorker 单键恒一）+ 主窗 navigator 面零污染（跨面守卫——
// 主窗 T66 面读数与本钉同沙箱并测，防修复面串扰）。
#[test]
fn iframe_navigator_plane_object_t72() {
    let mut sandbox = t72_sandbox!("<html><body><iframe name='cw72'></iframe></body></html>");
    sandbox
        .execute(
            "var f72 = document.getElementsByTagName('iframe')[0];
             var cw72 = f72.contentWindow;
             var cw72b = f72.contentWindow;
             var ctor72 = '';
             try { globalThis.Navigator(); } catch (e) { ctor72 = e.constructor.name + ':' + e.message; }
             var ctorNew72 = '';
             try { new globalThis.Navigator(); } catch (e) { ctorNew72 = e.constructor.name + ':' + e.message; }
             var d72 = Object.getOwnPropertyDescriptor(cw72.navigator, 'serviceWorker');
             globalThis.__r72 = [
               String(cw72 === cw72b),
               String(Object.getPrototypeOf(cw72.navigator) === globalThis.Navigator.prototype),
               String(Object.getPrototypeOf(globalThis.Navigator.prototype) === Object.prototype),
               Object.prototype.toString.call(cw72.navigator),
               String(cw72.navigator instanceof globalThis.Navigator),
               ctor72,
               ctorNew72,
               Object.getOwnPropertyNames(cw72.navigator).join(','),
               d72.writable + ':' + d72.enumerable + ':' + d72.configurable,
               String(Object.getPrototypeOf(globalThis.navigator) === globalThis.Navigator.prototype),
               Object.getOwnPropertyNames(globalThis.navigator).slice(0, 3).join(','),
               Object.prototype.toString.call(globalThis.navigator),
             ].join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r72").unwrap().value;
    let parts: Vec<&str> = out.split('|').collect();
    assert_eq!(parts[0], "true", "contentWindow 同 frame 缓存同 win（_iframeDocCache 记账——槽恒一前提）");
    assert_eq!(parts[1], "true", "getPrototypeOf(iframeWin.navigator) === Navigator.prototype（平面对象缺陷修复面）");
    assert_eq!(parts[2], "true", "原型链 iframe navigator → Navigator.prototype → Object.prototype");
    assert_eq!(parts[3], "[object Navigator]", "toString 品牌（原型 @@toStringTag，主窗 T66 同语义）");
    assert_eq!(parts[4], "true", "iframe navigator instanceof Navigator（品牌可判别）");
    assert_eq!(parts[5], "TypeError:Illegal constructor", "接口调用臂 TypeError（WebIDL 无 [Constructor]，同主窗 T66）");
    assert_eq!(parts[6], "TypeError:Illegal constructor", "接口 new 臂 TypeError（Illegal constructor 双臂）");
    assert_eq!(parts[7], "serviceWorker", "iframe navigator own 键面不扩（userAgent/platform 缺位是既有面，本切片不硬扩）");
    assert_eq!(parts[8], "true:true:true", "serviceWorker 槽 own 数据属性 {{w,e,c}}=true（原字面量描述符恒等）");
    assert_eq!(parts[9], "true", "主窗 navigator 原型链零变化（跨面守卫）");
    assert_eq!(parts[10], "userAgent,appName,appVersion", "主窗 navigator own 键面零变化（T66 part95 同款读数）");
    assert_eq!(parts[11], "[object Navigator]", "主窗 navigator 品牌零变化（跨面守卫）");
}

// ②serviceWorker 槽恒一钉（无回退面）：iframe navigator.serviceWorker 仍为
// ServiceWorkerContainer.prototype wrapper（parentServiceWorker 派生；创建期
// controller-on-load 基线 iframeServiceWorkerCreationControllerId 时序不动——该面由
// 既有 skip-waiting WPT 簇常驻覆盖），win 缓存下 wrapper identity 恒一，register/
// oncontrollerchange 操作面在案，且与主窗 serviceWorker 实体分离不串线。
#[test]
fn iframe_navigator_service_worker_slot_t72() {
    let mut sandbox = t72_sandbox!("<html><body><iframe name='cw72sw'></iframe></body></html>");
    sandbox
        .execute(
            "var f72 = document.getElementsByTagName('iframe')[0];
             var cw72 = f72.contentWindow;
             var cw72b = f72.contentWindow;
             var sw72 = cw72.navigator.serviceWorker;
             globalThis.__r72sw = [
               typeof sw72,
               String(Object.getPrototypeOf(sw72) === globalThis.ServiceWorkerContainer.prototype),
               String(cw72.navigator === cw72b.navigator),
               String(cw72b.navigator.serviceWorker === sw72),
               typeof sw72.register,
               String('oncontrollerchange' in sw72),
               String(sw72 !== globalThis.navigator.serviceWorker),
             ].join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r72sw").unwrap().value;
    let parts: Vec<&str> = out.split('|').collect();
    assert_eq!(parts[0], "object", "serviceWorker 槽实体在案（主窗 navigator.serviceWorker 派生 wrapper，非 undefined）");
    assert_eq!(parts[1], "true", "wrapper 原型恒为 ServiceWorkerContainer.prototype（实体不换实现）");
    assert_eq!(parts[2], "true", "win 缓存下 navigator 对象 identity 恒一");
    assert_eq!(parts[3], "true", "serviceWorker wrapper identity 恒一（同 win 槽同实体）");
    assert_eq!(parts[4], "function", "register 操作面在案（part05 原实现，无回退）");
    assert_eq!(parts[5], "true", "oncontrollerchange 槽在案（controllerchange 时序面载体）");
    assert_eq!(parts[6], "true", "iframe wrapper 与主窗 serviceWorker 实体分离（per-iframe 面不串线）");
}

// ③走链吸收钉（T65 canary 落点语义，iframe 面）：原型定向写经 iframe navigator 的
// [[Prototype]]（SDK「从根沿原型走 s 步」的落点）进 Navigator.prototype own——
// 页面对象 for-in 不可见（吸收）；不直透 Object.prototype 共享可枚举面（T65 污染
// 形态不复发）。receiver 普通 [[Set]] 仍落 iframe navigator own（正常 JS 语义，
// Chrome 一致）。红钉：修复前 p72===Object.prototype，canary 直落共享面 →
// for-in 染键 + Navigator.prototype 无落点，①③两臂红（EXIT=101 先红实证）。
#[test]
fn iframe_navigator_proto_absorb_t72() {
    let mut sandbox = t72_sandbox!("<html><body><iframe name='cw72abs'></iframe></body></html>");
    sandbox
        .execute(
            "var nav72 = document.getElementsByTagName('iframe')[0].contentWindow.navigator;
             var p72 = Object.getPrototypeOf(nav72);
             p72.__zwCanary72 = {probe:1};
             var onNavProto72 = !!(globalThis.Navigator.prototype &&
               Object.getOwnPropertyDescriptor(globalThis.Navigator.prototype, '__zwCanary72'));
             var onObjProto72 = !!Object.getOwnPropertyDescriptor(Object.prototype, '__zwCanary72');
             var ks72 = [];
             for (var k72 in {}) ks72.push(k72);
             nav72.__zwOwn72 = 1;
             var ownOnNav72 = Object.prototype.hasOwnProperty.call(nav72, '__zwOwn72');
             var ownOnNavProto72 = !!Object.getOwnPropertyDescriptor(globalThis.Navigator.prototype, '__zwOwn72');
             var ownOnObjProto72 = !!Object.getOwnPropertyDescriptor(Object.prototype, '__zwOwn72');
             try { delete globalThis.Navigator.prototype.__zwCanary72; } catch (_e72) {}
             try { delete Object.prototype.__zwCanary72; } catch (_e72) {}
             globalThis.__r72abs = [
               String(onNavProto72),
               String(onObjProto72),
               String(ks72.length),
               String(ownOnNav72),
               String(ownOnNavProto72),
               String(ownOnObjProto72),
             ].join('|');",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r72abs").unwrap().value;
    let parts: Vec<&str> = out.split('|').collect();
    assert_eq!(parts[0], "true", "原型定向写落 Navigator.prototype own（吸收面——SDK canary 落点）");
    assert_eq!(parts[1], "false", "不直透 Object.prototype（共享可枚举面零污染）");
    assert_eq!(parts[2], "0", "页面对象 for-in 零继承键（T65 污染形态不复发——{{}} 视图吸收不可见）");
    assert_eq!(parts[3], "true", "receiver 普通 [[Set]] 落 iframe navigator own（正常 JS 语义）");
    assert_eq!(parts[4], "false", "[[Set]] 不污染 Navigator.prototype");
    assert_eq!(parts[5], "false", "[[Set]] 不污染 Object.prototype");
}
