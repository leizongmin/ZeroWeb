// slice32（RP-3）：Window named access 多命中集合 live 语义钉。slice30 安装的静态
// 快照集合升格 live：①childList 变异（appendChild/removeChild）经 _zwLiveCollections
// 设施（liveSpec.matches）同步维护 len/order；②id/name 属性变异经 part04 钩子
// _zwNAAttrChanged 重核成员（失格剔除/新中末位并入）；③成员跌破 2 全局形态跟随
//（_zwNAGlobalMorph：1→元素、0→回收）；④同名重装置旧集合 dead（captured 引用冻结
// = 旧文档语义）；⑤动态名边界钉——slice36 已收口为动态解析正钉（脚本后建名访问时
// 解析，见 part37.rs `_zwNADynamicSync` 面）。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
// https://dom.spec.whatwg.org/#concept-collection-live

/// slice32 钉共用沙箱装配：shim + 快照 + DOM 回调（install 于回调注册时执行）。
macro_rules! s32_sandbox {
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
        let page_url = Arc::new(Mutex::new("https://zero.test/s32".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①childList live：安装集合后 appendChild/removeChild 同名元素 → len/order 即时更新
//（修前静态快照：len 恒安装时点值 → RED）。
#[test]
fn named_access_live_childlist_append_remove_s32() {
    let mut sandbox = s32_sandbox!("<html><body><div id='mm'></div><div id='mm'></div></body></html>");
    sandbox
        .execute(
            "var col0 = window.mm;
             globalThis.__r_len0 = col0.length;
             var add = document.createElement('div');
             add.setAttribute('id', 'mm');
             document.body.appendChild(add);
             globalThis.__r_len1 = window.mm.length;
             globalThis.__r_order = window.mm[0] === col0[0] && window.mm[1] === col0[1] && window.mm[2] === add;
             globalThis.__r_same = window.mm === col0;
             document.body.removeChild(add);
             globalThis.__r_len2 = window.mm.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "安装时点集合长度 = 多命中数（slice30 静态基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "3",
        "appendChild 同名元素后集合 length 即时 +1（live，修前恒 2）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_order").unwrap().value,
        "true",
        "并入成员保持既有序 + 末位追加（append 形态树序近似）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_same").unwrap().value,
        "true",
        "集合对象身份不变（live 维护非重装）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len2").unwrap().value,
        "2",
        "removeChild 后集合 length 即时 -1（live）"
    );
}

// ②属性变异 live：成员 name 属性改空 → 集合剔除失格成员；跌破 2 → 全局 morph 元素
//（WPT named-objects.html window['a'] / window['b'] 断言形态）。
#[test]
fn named_access_attr_shrink_and_morph_s32() {
    let mut sandbox = s32_sandbox!(
        "<html><body><embed name='cc' id='e1'></embed><form name='cc' id='f1'></form></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.cc;
             globalThis.__r_len0 = col0.length;
             document.getElementById('f1').setAttribute('name', '');
             globalThis.__r_len1 = col0.length;
             globalThis.__r_morph = window.cc === document.getElementById('e1');
             globalThis.__r_is_col = window.cc === col0;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "双 name 命中安装集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "1",
        "name 属性改空后集合剔除失格成员（live，修前恒 2）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_morph").unwrap().value,
        "true",
        "跌破单命中后 window.cc morph 为唯一元素（spec 取值算法）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_is_col").unwrap().value,
        "false",
        "全局不再是集合对象（morph 生效）"
    );
}

// ②扩展：morph 后 captured 集合仍 live（spec：脚本先捕获的集合对象持续反映文档）；
// 全局 morph 成元素后 0 命中不回收（元素全局不跟随——slice27 元素全局语义保持，
// stale 到下次快照换代由 renderer 登记·回收链路清理——live 边界如实申报钉）。
#[test]
fn named_access_attr_shrink_morph_then_captured_live_s32() {
    let mut sandbox = s32_sandbox!(
        "<html><body><img name='zz' id='z1'><img name='zz' id='z2'></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.zz;
             globalThis.__r_len0 = col0.length;
             document.getElementById('z1').setAttribute('name', 'other');
             globalThis.__r_len1 = col0.length;
             globalThis.__r_morph = window.zz === document.getElementById('z2')
               && !(window.zz instanceof window.HTMLCollection);
             document.getElementById('z2').setAttribute('name', '');
             globalThis.__r_len2 = col0.length;
             globalThis.__r_stale = window.zz === document.getElementById('z2');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "安装时点双命中集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "1",
        "name 改他值即失格（成员剔除，captured 集合持续反映）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_morph").unwrap().value,
        "true",
        "跌破单命中后全局 morph 元素（spec 取值算法）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len2").unwrap().value,
        "0",
        "captured 集合 0 命中（全失格剔除，集合对象仍存活）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_stale").unwrap().value,
        "true",
        "元素全局 0 命中不回收（边界：元素全局 stale 到换代清理，如实申报）"
    );
}

// slice30 静态行为回归钉：多命中集合形态/树序 identity、单命中元素形态、脚本自有
// 全局不被遮蔽（live 化不改安装面既有语义）。
#[test]
fn named_access_static_regression_s32() {
    let mut sandbox = s32_sandbox!(
        "<html><body>\
         <img name='rg' data-mark='a'><img name='rg' data-mark='b'>\
         <div id='solo'></div>\
         </body></html>"
    );
    sandbox
        .execute(
            "globalThis.__r_col = window.rg instanceof window.HTMLCollection && window.rg.length === 2;
             globalThis.__r_order = window.rg[0].getAttribute('data-mark') === 'a'
               && window.rg[1].getAttribute('data-mark') === 'b';
             globalThis.__r_solo = window.solo && window.solo.nodeType === 1
               && !(window.solo instanceof window.HTMLCollection);
             globalThis.__own = 'kept';
             globalThis.__r_own = window.__own === 'kept';")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_col").unwrap().value,
        "true",
        "多命中集合形态保持（slice30 基线不回退）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_order").unwrap().value,
        "true",
        "树序 identity 保持"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_solo").unwrap().value,
        "true",
        "单命中元素形态保持（slice27 面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_own").unwrap().value,
        "true",
        "脚本自有全局不被遮蔽"
    );
}

// ④同名重装 dead 语义：全局清除后重装（renderer 换代登记·重装链路的沙箱缩影）→
// 新集合对象 + 新内容；旧 captured 引用冻结（不再随变异更新——旧文档集合不跨换代存活）。
// slice37 NPO 化后清理面换 `__zwNADelete`（window 级 delete 对 named property 是
// no-op——Chrome 同款，见 recycled_collection_regenerates_s33 复活面），沙箱缩影随链路。
#[test]
fn named_access_reinstall_dead_marks_old_s32() {
    let mut sandbox = s32_sandbox!("<html><body><div id='rr'></div><div id='rr'></div></body></html>");
    sandbox
        .execute(
            "var old = window.rr;
             globalThis.__r_old_alive0 = (function () { document.body.appendChild(document.createElement('div')).setAttribute('id', 'rr'); return old.length; })();
             __zwNADelete('rr');
             __zwInstallNamedAccess();
             globalThis.__r_new_obj = window.rr !== old;
             globalThis.__r_new_len0 = window.rr.length;
             document.body.removeChild(document.body.lastChild);
             globalThis.__r_old_frozen = old.length;
             globalThis.__r_new_live = window.rr.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_old_alive0").unwrap().value,
        "3",
        "重装前旧集合 live（append 即 +1，对照冻结面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_new_obj").unwrap().value,
        "true",
        "重装产生新集合对象（换代新身份）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_new_len0").unwrap().value,
        "3",
        "重装集合构建期并入 pending-added 同名成员（R50 合并，live 安装语义）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_old_frozen").unwrap().value,
        "3",
        "旧 captured 集合 dead 冻结（removed 剔除不再入账）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_new_live").unwrap().value,
        "2",
        "新集合接棒 live 维护"
    );
}

// ⑤动态名面（slice36 收口，原 s32 live 边界负控钉按其断言信息翻转）：脚本后建
// 元素（createElement + id + appendChild）在访问时点解析——spec named property
// visibility 按次访问计算，不限定安装时点快照。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
#[test]
fn named_access_dynamic_name_boundary_s32() {
    let mut sandbox = s32_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var d = document.createElement('div');
             d.setAttribute('id', 'dyn32');
             document.body.appendChild(d);
             globalThis.__r_undef = window.dyn32 === undefined;
             globalThis.__r_ident = window.dyn32 === d;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_undef").unwrap().value,
        "false",
        "slice36 已收口动态名面：脚本后建名访问时解析（本钉为机制设计的强制申报翻转，非回归）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_ident").unwrap().value,
        "true",
        "解析目标即脚本创建的同一元素（身份一致）"
    );
}
