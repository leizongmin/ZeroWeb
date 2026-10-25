// part90 — html-semantics M2 切片 78（s78，2026-10-25）：F4 image-map 命中测试
// （the-area-element 语料 area-coords 32F / area-shape 17F / area-processing 14F）。
//
// 根因：runner headless 无渲染管线——host `__zw_elementFromPoint`（R2924
// HitTestCache）与 `__zw_getBoundingClientRect` 均未注册，`document.
// elementFromPoint(x, y)` 在 usemap img 命中面恒 null（63F 全部「expected
// area/img but got null」）。s78 三刀：① 解析核（part03）——「rules for parsing
// a list of floating-point numbers」（`__zwImgMapFloatList5278`：分隔集 ASCII
// 空白+逗号+分号、垃圾跳读、浮点前缀截断、error→0）、shape 枚举
// （`__zwImgMapShape5278`：ASCII case-insensitive、missing/invalid → rect）、
// 点入形状（`__zwImgMapShapeAt5278`：rect min/max 归一半开框、circle d²≤r²、
// poly 奇数丢尾 + even-odd 半开边射线）、hash-name reference
// （`__zwImgMapHashRef5278`）；② 命中面（part06）——`_zwImgMapHitSel` 挂
// elementFromPoint 的 host-bridge 缺席臂（SVG 命中后置；img 框 (0,0) 起点 +
// IDL width/height 近似，FIXME: 真实几何归 layout 集成面）；③ 无形状覆盖 →
// img 自身（spec pointing device 臂）。
// https://html.spec.whatwg.org/multipage/image-maps.html#image-map-processing-model

/// 钉①：解析核真值点。(a) float list：基础逗号、连续分隔一体跳（'2,,10,10' →
/// [2,10,10]）、分号/空白分隔、\v 非空白（token 连体 → 单数）、前导垃圾跳读
///（'="2' → 2）、浮点前缀截断（'2in' → 2）、小数（'.4' → 0.4）、指数（'1e1' →
/// 10）、error→0（'--5'）、全垃圾/空/null → 空表；(b) shape 枚举：大小写、
/// circ/polygon 别名、'CİRCLE'（非 ASCII 折叠）→ rect、missing/invalid → rect；
/// (c) shape-at：rect 最小计数/交换归一/负坐标、circle r≤0 空/计数 3、poly
/// 奇数丢尾/even-odd 边界行（(119,101)/(199,101) in、(150,125)/(118,118) out）、
/// default 恒真；(d) hash-name ref：无 #/#尾 → null、首 # 截断。
/// 断言翻转：解析核撤（(a)–(d) 全红）。
#[test]
fn image_map_parse_r5278() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> =
        Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s78p".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    let js = concat!(
            "var L = function (s) { return JSON.stringify(globalThis.__zwImgMapFloatList5278(s)); };\
             var S = function (s) { return String(globalThis.__zwImgMapShape5278(s)); };\
             var A = function (sh, co, x, y) { return String(globalThis.__zwImgMapShapeAt5278(globalThis.__zwImgMapShape5278(sh), globalThis.__zwImgMapFloatList5278(co), x, y)); };\
             var H = function (s) { return JSON.stringify(globalThis.__zwImgMapHashRef5278(s)); };\
             globalThis.__r = [\
               // (a) float list
               L('2,2,10,10'),\
               L('2,,10,10'),\
               L('2;;10,10'),\
               L(' 2,2,10,10'),\
               L('2\\v2'),\
               L(\"='2,2,10,10\"),\
               L('2in,2in,10cm,10cm'),\
               L('1.4,1.4,10,10'),\
               L('.4'),\
               L('1e1'),\
               L('2%'),\
               L('--5'),\
               L('abc'),\
               L(''),\
               L(null),\
               // (b) shape 枚举
               S('rect'), S('RECT'), S('rectangle'), S('circle'), S('CIRC'),\
               S('C\\u0130RCLE'), S('c\\u0131rcle'), S('DEFAULT'), S('poly'), S('polygon'),\
               S('foobar invalid'), S(''), S(null),\
               // (c) shape-at：rect
               A('rect', '2,2,10', 3, 3),\
               A('rect', '10,2,2,10', 3, 3),\
               A('rect', '10,2,2,10', 9, 9),\
               A('rect', '2,2,10,10', 1, 3),\
               A('rect', '-10,-10,10,10', 1, 1),\
               A('rect', '-10,-10,10,10', 299, 299),\
               // (c) shape-at：circle
               A('circle', '20,40,10', 11, 40),\
               A('circle', '20,40,10', 12, 32),\
               A('circle', '20,40,0', 20, 40),\
               A('circle', '20,40,-10', 20, 40),\
               A('circle', '20,40', 20, 40),\
               // (c) shape-at：poly
               A('poly', '100,100,120,100,100,120', 101, 101),\
               A('poly', '100,100,120,100,100,120', 119, 101),\
               A('poly', '100,100,120,100,100,120', 101, 119),\
               A('poly', '100,100,120,100,100,120', 118, 118),\
               A('poly', '100,100,120,100,100,120,300', 101, 101),\
               A('polygon', '100,100,200,100,100,200,150,50,200,200', 101, 101),\
               A('polygon', '100,100,200,100,100,200,150,50,200,200', 199, 101),\
               A('polygon', '100,100,200,100,100,200,150,50,200,200', 150, 51),\
               A('polygon', '100,100,200,100,100,200,150,50,200,200', 150, 125),\
               A('poly', '100,100,120,100', 101, 101),\
               // (c) shape-at：default
               A('default', '', 101, 101),\
               // (d) hash-name ref
               H('#x'), H('x'), H('#'), H('a#b#c'), H('#MAP1')\
             ];");
    sandbox
        .execute(js)
        .unwrap();
    let get = |sandbox: &mut V8Sandbox, i: usize| -> String {
        sandbox
            .execute(&format!("globalThis.__r[{}]", i))
            .unwrap()
            .value
    };
    // (a) float list
    assert_eq!(get(&mut sandbox, 0), "[2,2,10,10]", "(a1) 逗号基础面");
    assert_eq!(get(&mut sandbox, 1), "[2,10,10]", "(a2) 连续分隔一体跳（不产 0 项）");
    assert_eq!(get(&mut sandbox, 2), "[2,10,10]", "(a3) 分号分隔");
    assert_eq!(get(&mut sandbox, 3), "[2,2,10,10]", "(a4) 前导空白分隔");
    assert_eq!(get(&mut sandbox, 4), "[2]", "(a5) \\v 非空白 → token 连体浮点前缀截断");
    assert_eq!(get(&mut sandbox, 5), "[2,2,10,10]", "(a6) 垃圾跳读到数值起点");
    assert_eq!(get(&mut sandbox, 6), "[2,2,10,10]", "(a7) 单位尾垃圾忽略");
    assert_eq!(get(&mut sandbox, 7), "[1.4,1.4,10,10]", "(a8) 小数保真");
    assert_eq!(get(&mut sandbox, 8), "[0.4]", "(a9) 纯小数 '.4'");
    assert_eq!(get(&mut sandbox, 9), "[10]", "(a10) 指数 '1e1' → 10");
    assert_eq!(get(&mut sandbox, 10), "[2]", "(a11) 百分号尾垃圾忽略");
    assert_eq!(get(&mut sandbox, 11), "[0]", "(a12) '--5' error → 0");
    assert_eq!(get(&mut sandbox, 12), "[]", "(a13) 全垃圾 → 空表");
    assert_eq!(get(&mut sandbox, 13), "[]", "(a14) 空串 → 空表");
    assert_eq!(get(&mut sandbox, 14), "[]", "(a15) null → 空表");
    // (b) shape 枚举
    assert_eq!(get(&mut sandbox, 15), "rect", "(b1) rect");
    assert_eq!(get(&mut sandbox, 16), "rect", "(b2) ASCII 大写");
    assert_eq!(get(&mut sandbox, 17), "rect", "(b3) rectangle 别名");
    assert_eq!(get(&mut sandbox, 18), "circle", "(b4) circle");
    assert_eq!(get(&mut sandbox, 19), "circle", "(b5) circ 别名");
    assert_eq!(get(&mut sandbox, 20), "rect", "(b6) 'CİRCLE' 非 ASCII 折叠 → invalid → rect");
    assert_eq!(get(&mut sandbox, 21), "rect", "(b7) 'cırcle' → invalid → rect");
    assert_eq!(get(&mut sandbox, 22), "default", "(b8) DEFAULT ASCII 大写");
    assert_eq!(get(&mut sandbox, 23), "poly", "(b9) poly");
    assert_eq!(get(&mut sandbox, 24), "poly", "(b10) polygon 别名");
    assert_eq!(get(&mut sandbox, 25), "rect", "(b11) invalid value default → rect");
    assert_eq!(get(&mut sandbox, 26), "rect", "(b12) 空 → rect");
    assert_eq!(get(&mut sandbox, 27), "rect", "(b13) missing value default → rect");
    // (c) shape-at：rect
    assert_eq!(get(&mut sandbox, 28), "false", "(c1) rect 3 数 → 空形");
    assert_eq!(get(&mut sandbox, 29), "true", "(c2) rect 1↔3 交换归一（10,2,2,10 → [2,10)）");
    assert_eq!(get(&mut sandbox, 30), "true", "(c3) 交换归一右上点");
    assert_eq!(get(&mut sandbox, 31), "false", "(c4) 半开框左边界外（x=1 < 2）");
    assert_eq!(get(&mut sandbox, 32), "true", "(c5) 负坐标矩形含点");
    assert_eq!(get(&mut sandbox, 33), "false", "(c6) 负坐标矩形外点");
    // (c) shape-at：circle
    assert_eq!(get(&mut sandbox, 34), "true", "(c7) 圆内点（d=9 ≤ r=10）");
    assert_eq!(get(&mut sandbox, 35), "false", "(c8) 圆外点（d≈11.3）");
    assert_eq!(get(&mut sandbox, 36), "false", "(c9) r=0 → 空形（圆心亦不含）");
    assert_eq!(get(&mut sandbox, 37), "false", "(c10) 负半径 → 空形");
    assert_eq!(get(&mut sandbox, 38), "false", "(c11) circle 2 数 → 空形");
    // (c) shape-at：poly
    assert_eq!(get(&mut sandbox, 39), "true", "(c12) 三角形内点");
    assert_eq!(get(&mut sandbox, 40), "true", "(c13) 斜边边界行（x ≤ 交点半开边）");
    assert_eq!(get(&mut sandbox, 41), "true", "(c14) 垂直边边界行");
    assert_eq!(get(&mut sandbox, 42), "false", "(c15) 斜边外点");
    assert_eq!(get(&mut sandbox, 43), "true", "(c16) 7 数奇数丢尾（'300' 弃）");
    assert_eq!(get(&mut sandbox, 44), "true", "(c17) star 咬合区（3 边计数奇）");
    assert_eq!(get(&mut sandbox, 45), "true", "(c18) star 边界行 (199,101)");
    assert_eq!(get(&mut sandbox, 46), "true", "(c19) star 顶尖 (150,51)");
    assert_eq!(get(&mut sandbox, 47), "false", "(c20) star 咬合外 (150,125)（2 边偶）");
    assert_eq!(get(&mut sandbox, 48), "false", "(c21) poly 4 数（丢尾后 <6）→ 空形");
    // (c) shape-at：default
    assert_eq!(get(&mut sandbox, 49), "true", "(c22) default → 整图恒真");
    // (d) hash-name ref
    assert_eq!(get(&mut sandbox, 50), "\"x\"", "(d1) '#x' → 'x'");
    assert_eq!(get(&mut sandbox, 51), "null", "(d2) 无 '#' → null");
    assert_eq!(get(&mut sandbox, 52), "null", "(d3) '#' 末字符 → null");
    assert_eq!(get(&mut sandbox, 53), "\"b#c\"", "(d4) 首 '#' 截断（尾 '#' 保留）");
    assert_eq!(get(&mut sandbox, 54), "\"MAP1\"", "(d5) 大写 map 名");
}

/// 钉②：elementFromPoint 缺席臂 image-map 命中面。(a) 静态 DOM（img 300×300
/// usemap=#x + map name=x + area rect 2,2,10,10）——形状内 → area、形状外图内
/// → img、图外 → null；(b) IDL 反射面——area.shape/coords IDL setter 改 circle
/// 后命中随变（circle 内 → area、rect 残点 → img）；(c) map id 匹配面——map
/// 无 name 仅 id 时 usemap 仍解析；(d) usemap 无 '#' → 图内点回落 img（无
/// map 解析）；(e) 无形状覆盖（default 缺席 + 空 coords）→ img 自身。
/// 断言翻转：`_zwImgMapHitSel` 撤（(a)–(e) 全 null 红）。
#[test]
fn image_map_element_from_point_r5278() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <img id='img' width='300' height='300' usemap='#x'>\
         <map name='x'><area id='area' shape='rect' coords='2,2,10,10'></map>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s78h".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var T = function (x, y) {\
               var el = document.elementFromPoint(x, y);\
               return el ? String(el.tagName) : 'null';\
             };\
             globalThis.__r = [\
               T(3, 3),\
               T(1, 3),\
               T(9, 9),\
               T(301, 301),\
               T(150, 150)\
             ];\
             // (b) IDL 反射改 shape → 命中面随变（corpus area-shape 同路径）
             var area = document.getElementById('area');\
             area.shape = 'circle';\
             area.coords = '20,40,10';\
             globalThis.__r.push(T(11, 40), T(3, 3));\
             // (c) map id 匹配（name 撤除）
             var map = document.querySelector('map');\
             map.removeAttribute('name');\
             map.id = 'x';\
             globalThis.__r.push(T(11, 40));\
             // (d) usemap 无 '#'（hash-name ref null → 无 map 解析 → img）
             var img = document.getElementById('img');\
             img.setAttribute('usemap', 'x');\
             globalThis.__r.push(T(11, 40));\
             // (e) 无形状覆盖（area 无 shape/coords → rect 0 数空形 → img 自身）
             globalThis.__err = '';\
             function st(f) { try { f(); } catch (e) { globalThis.__err += 'E:' + String((e && e.message) || e) + ';'; } }\
             st(function () { img.setAttribute('usemap', '#x'); });\
             st(function () { map.setAttribute('name', 'x'); });\
             st(function () { area.removeAttribute('shape'); });\
             st(function () { area.removeAttribute('coords'); });\
             st(function () { globalThis.__r.push(T(150, 150)); });\
             globalThis.__r.push(globalThis.__err);",
        )
        .unwrap();
    let get = |sandbox: &mut V8Sandbox, i: usize| -> String {
        sandbox
            .execute(&format!("globalThis.__r[{}]", i))
            .unwrap()
            .value
    };
    // (a)
    assert_eq!(get(&mut sandbox, 0), "AREA", "(a1) rect 形状内 → area");
    assert_eq!(get(&mut sandbox, 1), "IMG", "(a2) 形状外图内 → img 自身");
    assert_eq!(get(&mut sandbox, 2), "AREA", "(a3) rect 右上角内点");
    assert_eq!(get(&mut sandbox, 3), "null", "(a4) 图外 → null");
    assert_eq!(get(&mut sandbox, 4), "IMG", "(a5) 图心（形状外）→ img 自身");
    // (b)
    assert_eq!(get(&mut sandbox, 5), "AREA", "(b1) IDL 改 circle 后圆内 → area");
    assert_eq!(get(&mut sandbox, 6), "IMG", "(b2) 旧 rect 残点 → img（形状随 attrs 变）");
    // (c)
    assert_eq!(get(&mut sandbox, 7), "AREA", "(c1) map name 撤除后 id='x' 仍解析");
    // (d)
    assert_eq!(get(&mut sandbox, 8), "IMG", "(d1) usemap 无 '#' → ref null → img 自身");
    // (e)
    assert_eq!(get(&mut sandbox, 9), "IMG", "(e1) 无形状覆盖 → img 自身");
    assert_eq!(get(&mut sandbox, 10), "", "(e2) (e) 块无异常");
}
// R5279（s79）：IDL `.id=` 改名的同 execute 可见性——set trap 与 setAttribute('id')
// 同记 R125 sel-based id 覆盖表（缺登记 → getElementById/querySelector('[id=…]')
// miss 新 id；latest-wins 缺写 → 「改名回原 id」被残留旧条目毒化——WPT
// hash-name-reference-dynamic 的 `map.id = …` 动态改名族根因）。
// https://dom.spec.whatwg.org/#dom-nonelementparentnode-getelementbyid
#[test]
fn idl_id_rename_getelementbyid_r5279() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='dv'></div><map id='testmap'></map></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s79".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__r = [];\
             function st(f) { try { f(); } catch (e) { globalThis.__r.push('E:' + String((e && e.message) || e)); } }\
             var dv = document.getElementById('dv');\
             var map = document.querySelector('map');\
             // (a) IDL 改名 → gEBI 新 id 命中、旧 id 落空、qs('[id=…]') 命中
             st(function () {\
               dv.id = 'dx';\
               globalThis.__r.push(document.getElementById('dx') === dv);\
               globalThis.__r.push(document.getElementById('dv') === null);\
             });\
             // (b) 改名回原 id → latest-wins 覆盖残留条目（毒化面）
             st(function () {\
               dv.id = 'dv';\
               globalThis.__r.push(document.getElementById('dv') === dv);\
               globalThis.__r.push(document.getElementById('dx') === null);\
             });\
             // (c) setAttribute 路径对称仍在（R125 原面）
             st(function () {\
               dv.setAttribute('id', 'da');\
               globalThis.__r.push(document.getElementById('da') === dv);\
               dv.setAttribute('id', 'dv');\
             });\
             // (d) map IDL 改名（hash-name-reference-dynamic 形态）
             st(function () {\
               map.id = 'mx';\
               globalThis.__r.push(document.getElementById('mx') === map);\
               map.removeAttribute('name');\
               map.id = 'testmap';\
               globalThis.__r.push(document.getElementById('testmap') === map);\
             });",
        )
        .unwrap();
    let get = |sandbox: &mut V8Sandbox, i: usize| -> String {
        sandbox
            .execute(&format!("globalThis.__r[{}]", i))
            .unwrap()
            .value
    };
    // (a)
    assert_eq!(get(&mut sandbox, 0), "true", "(a1) IDL 改名后 gEBI 新 id 命中");
    assert_eq!(get(&mut sandbox, 1), "true", "(a2) 旧 id 落空");
    // 注：querySelector/matches/closest 的同 turn 改名可见性由 s81 独立切片承接
    //（见 query_id_rename_same_turn_visibility_s81）。
    // (b)
    assert_eq!(get(&mut sandbox, 2), "true", "(b1) 改名回原 id → gEBI 命中（毒化面）");
    assert_eq!(get(&mut sandbox, 3), "true", "(b2) 中间 id 落空");
    // (c)
    assert_eq!(get(&mut sandbox, 4), "true", "(c1) setAttribute 路径对称仍在");
    // (d)
    assert_eq!(get(&mut sandbox, 5), "true", "(d1) map IDL 改名 gEBI 命中");
    assert_eq!(get(&mut sandbox, 6), "true", "(d2) map 改名回原 id gEBI 命中");
    // st() 仅在异常时追加条目——全块无异常则索引 7 越界读 undefined（s78 (e2) 同法）
    assert_eq!(get(&mut sandbox, 7), "undefined", "(e) 全块无异常");
}

// R5281（s81）：querySelector/querySelectorAll/matches/closest 的 id 改名同 turn
// 可见性——R125 读侧查询半边（gEBI 双臂之外的共享查询面）。host 查询视图
// `with_query_view_doc` 只应用结构级 mutation（R3190「查询反映 pending 前」属性
// 语义锚定），同 turn `setAttribute('id',…)`/`.id=` 改名不入快照——双臂补偿：
// ① 命中复验（lw 现 id 不满足最终复合段 id 约束 → stale 剔除，全形态）；
// ② 单 compound 拉真（R125 表候选 + `_mutationInDoc` 门 + full compound 复验；
//    complex 祖先段约束不接——见 `_zwSelIdConstraint.single` 门注记）。
// IDL 反射读同步 lw 化（part03 get trap id 臂）是 `_hIdOf` 匹配读同源前提。
// FIXME(s81 后续面)：同 turn insert 元素无 `__zwSelector`（批末 apply 才有身份），
// 从其上行的 closest/query 全盲——结构性身份缺口，与 id 改名无关。
// https://dom.spec.whatwg.org/#dom-parentnode-queryselector
#[test]
fn query_id_rename_same_turn_visibility_s81() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body><div id='pa'><div id='old1'></div><div id='old2'></div>\
         <div id='old3'></div></div></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s81".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "globalThis.__r = [];\
             var pa = document.getElementById('pa');\
             var e1 = document.getElementById('old1');\
             // (a) setAttribute 改名——文档查询面双向
             e1.setAttribute('id', 'n1');\
             globalThis.__r.push(document.querySelector('#n1') === e1);\
             globalThis.__r.push(document.querySelector('#old1') === null);\
             globalThis.__r.push(document.querySelector('[id=\"n1\"]') === e1);\
             globalThis.__r.push(document.querySelectorAll('#n1').length);\
             globalThis.__r.push(document.querySelectorAll('#old1').length);\
             // (b) IDL 改名对称（s79 写侧 + s81 查询读侧）
             var e2 = document.getElementById('old2');\
             e2.id = 'n2';\
             globalThis.__r.push(document.querySelector('#n2') === e2);\
             globalThis.__r.push(document.querySelector('#old2') === null);\
             // (c) 元素子树查询 + tag 复合段
             var e3 = document.getElementById('old3');\
             e3.setAttribute('id', 'n3');\
             globalThis.__r.push(pa.querySelector('#n3') === e3);\
             globalThis.__r.push(pa.querySelector('#old3') === null);\
             globalThis.__r.push(pa.querySelector('div#n3') === e3);\
             // (d) matches 双向（含 tag 复合段）
             globalThis.__r.push(e1.matches('#n1'));\
             globalThis.__r.push(e1.matches('#old1'));\
             globalThis.__r.push(e1.matches('div#n1'));\
             // (e) closest 改名面（self；同 turn insert 元素上行盲区见 FIXME）
             globalThis.__r.push(e1.closest('#old1') === null);\
             globalThis.__r.push(e1.closest('#n1') !== null);\
             // (f) 改名回原 id——latest-wins 不残留（毒化面）
             e1.setAttribute('id', 'old1');\
             globalThis.__r.push(document.querySelector('#old1') === e1);\
             globalThis.__r.push(document.querySelector('#n1') === null);",
        )
        .unwrap();
    let get = |sandbox: &mut V8Sandbox, i: usize| -> String {
        sandbox
            .execute(&format!("globalThis.__r[{}]", i))
            .unwrap()
            .value
    };
    // (a)
    assert_eq!(get(&mut sandbox, 0), "true", "(a1) qS 新 id 拉真命中");
    assert_eq!(get(&mut sandbox, 1), "true", "(a2) qS 旧 id stale 剔除");
    assert_eq!(get(&mut sandbox, 2), "true", "(a3) qS [id=…] 属性形态命中");
    assert_eq!(get(&mut sandbox, 3), "1", "(a4) qSA 新 id 计 1");
    assert_eq!(get(&mut sandbox, 4), "0", "(a5) qSA 旧 id 计 0");
    // (b)
    assert_eq!(get(&mut sandbox, 5), "true", "(b1) IDL 改名 qS 新 id 命中");
    assert_eq!(get(&mut sandbox, 6), "true", "(b2) IDL 改名 qS 旧 id 剔除");
    // (c)
    assert_eq!(get(&mut sandbox, 7), "true", "(c1) 子树 qS 新 id 命中");
    assert_eq!(get(&mut sandbox, 8), "true", "(c2) 子树 qS 旧 id 剔除");
    assert_eq!(get(&mut sandbox, 9), "true", "(c3) 子树 div#id 复合段命中");
    // (d)
    assert_eq!(get(&mut sandbox, 10), "true", "(d1) matches 新 id true");
    assert_eq!(get(&mut sandbox, 11), "false", "(d2) matches 旧 id false");
    assert_eq!(get(&mut sandbox, 12), "true", "(d3) matches div#id true");
    // (e)
    assert_eq!(get(&mut sandbox, 13), "true", "(e1) closest 旧 id null");
    assert_eq!(get(&mut sandbox, 14), "true", "(e2) closest 新 id 非空");
    // (f)
    assert_eq!(get(&mut sandbox, 15), "true", "(f1) 改名回原 id qS 命中");
    assert_eq!(get(&mut sandbox, 16), "true", "(f2) 中间 id 落空");
}
