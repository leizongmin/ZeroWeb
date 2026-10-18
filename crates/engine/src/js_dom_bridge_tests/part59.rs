// js_dom_bridge 测试切片 59。本文件经 `js_dom_bridge_tests.rs` 的 `include!` 并入同一模块，
// 与前序切片共享模块作用域（generate_js_dom_shim / register_dom_callbacks / DomMutation 等）。
// t6（siteopt qq-20261016-r1）：qq.com 首页水合断点簇四根因的常驻回归钉——
// ① handle-only 元素 class/id 选择器 matches（Swiper mount `Kj(el,'.swiper-wrapper')[0]`
//    全 miss → slidesEl undefined → 'children' TypeError；probe-swiper-mount 探针实证）；
// ② children 集合 R333 桶并入的直接子判定（append 携带子树时孙代渗入 `el.children`）；
// ③ CSSOM View `Element.prototype.scrollTop/scrollLeft` 访问器 gOPD 可见（scroll-behavior-
//    polyfill 装载即 `gOPD(Element.prototype,'scrollTop').set` 解引用 TypeError）；
// ④ insertBefore 不可见容器（漂移 sel）lenient 臂——融合子列表空 + **有父**外部 ref 不抛
//    NotFound（React D-back 水合 commit 臂死亡级联根因），非空容器照抛、ref 无父链
//    （detached 哨兵）即便空视图容器也照抛（返修 F1；WPT 守卫）。

fn t59_sandbox_with_shim() -> zero_script_sandbox::V8Sandbox {
    let mut sandbox = zero_script_sandbox::V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    sandbox
}

/// ① handle-only 元素 class/id 匹配：detached 与 attached 两形态（邻近变体：tag 不匹配
/// 照 false、`tag.cls` 复合、`#id`、`div.a.b` 链）。
#[test]
fn t59_dynamic_element_matches_class_and_id_selectors() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body><div id='host'></div></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var w = document.createElement('div');\
             w.className = 'swiper-wrapper';\
             globalThis.__r1 = w.matches('.swiper-wrapper');\
             globalThis.__r2 = w.matches('div.swiper-wrapper');\
             globalThis.__r3 = w.matches('span');\
             w.id = 't59w';\
             globalThis.__r4 = w.matches('#t59w');\
             w.className = 'a b';\
             globalThis.__r5 = w.matches('.a.b');\
             globalThis.__r6 = w.matches('.a.c');\
             document.getElementById('host').appendChild(w);\
             globalThis.__r7 = w.matches('.a.b');",
        )
        .unwrap();
    let mut q = |expr: &str| {
        sandbox
            .execute(expr).unwrap().value
    };
    assert_eq!(q("String(globalThis.__r1)"), "true", "detached 元素 class 选择器须命中");
    assert_eq!(q("String(globalThis.__r2)"), "true", "tag.cls 复合须命中");
    assert_eq!(q("String(globalThis.__r3)"), "false", "tag 不匹配须 false");
    assert_eq!(q("String(globalThis.__r4)"), "true", "#id 选择器须命中");
    assert_eq!(q("String(globalThis.__r5)"), "true", "多 class 链须命中");
    assert_eq!(q("String(globalThis.__r6)"), "false", "缺 class 须 false");
    assert_eq!(q("String(globalThis.__r7)"), "true", "挂载后 class 匹配保持");
}

/// ② children 集合只含直接子（append 携带子树时孙代不渗入）。
#[test]
fn t59_children_collection_excludes_grandchildren() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var c = document.createElement('div');\
             var w = document.createElement('div');\
             var s = document.createElement('div');\
             w.appendChild(s);\
             c.appendChild(w);\
             document.body.appendChild(c);\
             globalThis.__kids = c.children.length;\
             globalThis.__kid0 = c.children[0] === w;\
             globalThis.__childNodes = c.childNodes.length;",
        )
        .unwrap();
    let mut q = |expr: &str| {
        sandbox
            .execute(expr).unwrap().value
    };
    assert_eq!(q("String(globalThis.__kids)"), "1", "children 须只含直接子（旧渗入孙代返 2）");
    assert_eq!(q("String(globalThis.__kid0)"), "true", "唯一子须为 w 本体");
    assert_eq!(q("String(globalThis.__childNodes)"), "1", "childNodes 同形（对照面）");
}

/// ③ CSSOM View：`Element.prototype.scrollTop/scrollLeft` gOPD 可见 + 实例读写经 trap。
#[test]
fn t59_scroll_top_left_prototype_accessors_visible() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> =
        std::sync::Arc::new(std::sync::Mutex::new("<html><body><div id='d'></div></body></html>".to_string()));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 旧形态：gOPD 返 undefined → polyfill `.set` 解引用 TypeError（qq.com 每轮 1 条实证）。
    sandbox
        .execute(
            "var dTop = Object.getOwnPropertyDescriptor(globalThis.Element.prototype, 'scrollTop');\
             var dLeft = Object.getOwnPropertyDescriptor(globalThis.Element.prototype, 'scrollLeft');\
             globalThis.__hasSetTop = !!(dTop && typeof dTop.set === 'function' && typeof dTop.get === 'function');\
             globalThis.__hasSetLeft = !!(dLeft && typeof dLeft.set === 'function');\
             var el = document.getElementById('d');\
             el.scrollTop = 33;\
             globalThis.__roundtrip = el.scrollTop;",
        )
        .unwrap();
    let mut q = |expr: &str| {
        sandbox
            .execute(expr).unwrap().value
    };
    assert_eq!(q("String(globalThis.__hasSetTop)"), "true", "gOPD(scrollTop) 须返 {{get,set}}（polyfill 解引用面）");
    assert_eq!(q("String(globalThis.__hasSetLeft)"), "true", "gOPD(scrollLeft) 同形");
    assert_eq!(q("String(globalThis.__roundtrip)"), "33", "实例 scrollTop 读写经 trap 往返");
}

/// ④ insertBefore 不可见容器 lenient 臂：空融合视图容器 + **有父**外部 ref 放行
/// （React 漂移案 ref.parentNode 落在真实路径上）；对照面两臂照抛 NotFound——
/// 有子容器 + 外部 ref（WPT pre-insertion 守卫不放宽）、无父 ref（detached 哨兵，
/// WPT Node-insertBefore pre-insert 步骤 3，返修 F1 收窄臂）。
#[test]
fn t59_insert_before_invisible_container_lenient_arm() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body><div id='visible'><a id='kid'></a></div><a id='outsider'></a></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var invisible = document.createElement('div');\
             var fresh = document.createElement('div');\
             var outsider = document.getElementById('outsider');\
             globalThis.__lenientErr = '';\
             try { invisible.appendChild(fresh); } catch (e) { /* detached 容器可能走其他分支 */ }\
             try { invisible.insertBefore(document.createElement('p'), outsider); } catch (e) { globalThis.__lenientErr = String(e.name || e); }\
             var parentless = document.createElement('div');\
             globalThis.__parentlessErr = '';\
             try { invisible.insertBefore(document.createElement('p'), parentless); } catch (e) { globalThis.__parentlessErr = String(e.name || e); }\
             var visible = document.getElementById('visible');\
             var kid = document.getElementById('kid');\
             globalThis.__visibleErr = '';\
             try { visible.insertBefore(document.createElement('p'), outsider); } catch (e) { globalThis.__visibleErr = String(e.name || e); }",
        )
        .unwrap();
    let mut q = |expr: &str| {
        sandbox
            .execute(expr).unwrap().value
    };
    assert_eq!(
        q("globalThis.__lenientErr"),
        "",
        "不可见容器（空融合视图）+ 有父外部 ref 须放行（React 漂移 sel 水合形态）"
    );
    assert_eq!(
        q("globalThis.__parentlessErr"),
        "NotFoundError",
        "无父 ref（detached 哨兵）即便空视图容器也照抛 NotFound（WPT pre-insert 步骤 3，返修 F1）"
    );
    assert_eq!(
        q("globalThis.__visibleErr"),
        "NotFoundError",
        "有子容器 + 外部 ref 照抛 NotFound（WPT 守卫面不放宽）"
    );
}
