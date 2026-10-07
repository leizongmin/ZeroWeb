// slice42（named-access 内聚性三件套收口）：
// ①morph 产物元素全局 0 命中回收（slice33 VERDICT 残余①收口——`_zwNAGlobalMorph`
//   morph 跟随面补 0 命中回收 + wired 腿批删 0 命中 `delete globalThis[name]`
//   no-op 改 `__zwNADelete` 双面清除）。
// ②`_proxyCache` 死条目边界钉（误解析面已被 R100/R315 守卫，现状正确钉固化）。
// ③live NA 集合树序（slice32 末位近似收口——childList 中插 + attr 并入两面
//   按树序落位，https://dom.spec.whatwg.org/#concept-collection）。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object

/// slice42 钉共用沙箱装配（s32_sandbox 同款：shim + 快照 + DOM 回调）。
macro_rules! s42_sandbox {
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
        let page_url = Arc::new(Mutex::new("https://zero.test/s42".to_string()));
        let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
        register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
        sandbox
    }};
}

// ①morph 产物 0 命中回收：双 name=zz 集合 → 逐一失格（2→1 morph 元素，1→0）→
// window.zz 应回收为 undefined（spec：named objects 空集则无 named property）。
// 修前：morph 产物（own expando 元素）stale 至快照换代（slice32 申报钉，part34
// `__r_stale` 边界钉本轮翻转为正确行为）。
#[test]
fn named_access_morph_product_zero_hit_recycle_s42() {
    let mut sandbox = s42_sandbox!(
        "<html><body><img name='zz' id='z1'><img name='zz' id='z2'></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.zz;
             globalThis.__r_len0 = col0.length;
             document.getElementById('z1').setAttribute('name', 'other');
             globalThis.__r_morph = window.zz === document.getElementById('z2');
             document.getElementById('z2').setAttribute('name', '');
             globalThis.__r_gone = typeof window.zz === 'undefined';
             globalThis.__r_col_len = col0.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "双 name=zz 安装集合（slice30 基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_morph").unwrap().value,
        "true",
        "跌破单命中 morph 元素（spec 取值算法，slice32 基线不回退）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_gone").unwrap().value,
        "true",
        "morph 产物 0 命中回收（spec：0 named objects 则属性缺席；修前 stale 元素）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_col_len").unwrap().value,
        "0",
        "captured 集合持续反映 0 命中（live 维护不回退）"
    );
}

// ①扩展（判定中发现同路径缺陷）：批删 0 命中不经过 morph 中间态（g 仍 === installed）
// 时回收走 `delete globalThis[name]`——wired 腿 NPO 化后对 backing 安装值 no-op
//（part05 `__zwNADelete` 接线处申报同款：NPO deleteProperty 恒 false 经链传播），
// 容器批删双成员单批 → stale 空集合残留。成员经 createElement 构造（handle 树，
// remFlat 子树展开可达）；集合由 slice36 动态面安装（同 `_zwNAInstalled` 账本）。
// 修前：window.bb = 0 成员空集合（backing 残留）；修后：undefined。
#[test]
fn named_access_batch_zero_hit_recycle_backing_s42() {
    let mut sandbox = s42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var b1 = document.createElement('img'); b1.setAttribute('name', 'bb');
             var b2 = document.createElement('img'); b2.setAttribute('name', 'bb');
             var box = document.createElement('div');
             box.appendChild(b1); box.appendChild(b2);
             document.body.appendChild(box);
             var col0 = window.bb;
             globalThis.__r_is_col = col0 === window.bb && window.bb.length === 2;
             document.body.removeChild(box);
             globalThis.__r_gone = typeof window.bb === 'undefined';
             globalThis.__r_col_len = col0.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_is_col").unwrap().value,
        "true",
        "脚本建双 name=bb 动态面安装集合（slice36 基线）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_gone").unwrap().value,
        "true",
        "批删 0 命中回收（wired 腿 backing 双面清除；修前 stale 空集合）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_col_len").unwrap().value,
        "0",
        "captured 集合 0 命中（live 维护不回退）"
    );
}

// ③live NA 集合树序——childList 中插面：insertBefore 中插命中成员应落树序位
//（https://dom.spec.whatwg.org/#concept-collection，HTMLCollection 成员树序）。
// 修前：末位近似（slice32 申报边界）。
#[test]
fn named_access_live_collection_tree_order_childlist_s42() {
    let mut sandbox = s42_sandbox!(
        "<html><body><div id='x'>a</div><div id='mid'></div><div id='x'>b</div></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.x;
             globalThis.__r_len0 = col0.length;
             var first = col0[0], second = col0[1];
             var nu = document.createElement('div');
             nu.setAttribute('id', 'x');
             document.body.insertBefore(nu, document.getElementById('mid'));
             globalThis.__r_order = window.x[0] === first
               && window.x[1] === nu
               && window.x[2] === second;
             globalThis.__r_len1 = window.x.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "安装期双 id=x 集合（树序 identity）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_order").unwrap().value,
        "true",
        "insertBefore 中插成员落树序位（修前末位近似：[a,b,nu]）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "3",
        "中插后集合长度 3（live 并入不回退）"
    );
}

// ③live NA 集合树序——attr 并入面：已存集合并入新命中成员应按树序落位（修前
// `_zwNAAttrChanged` 并入恒末位 push）。
#[test]
fn named_access_live_collection_tree_order_attr_join_s42() {
    let mut sandbox = s42_sandbox!(
        "<html><body><div id='p'></div><div id='x'>a</div><div id='x'>b</div></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.x;
             globalThis.__r_len0 = col0.length;
             var first = col0[0], second = col0[1];
             var pEl = document.getElementById('p');
             pEl.setAttribute('id', 'x');
             globalThis.__r_order = window.x[0] === pEl
               && window.x[1] === first
               && window.x[2] === second;
             globalThis.__r_len1 = window.x.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "安装期双 id=x 集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_order").unwrap().value,
        "true",
        "attr 并入成员落树序位（p 在树首；修前末位 push：[a,b,p]）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "3",
        "attr 并入后集合长度 3（live 重核不回退）"
    );
}

// ②`_proxyCache` 死条目边界钉（slice33 VERDICT 残余②现状固化）：死 handle 的
// shim 侧 `_proxyCache` 条目不清属**纯卫生**——误解析面已被既有守卫覆盖：
// (a) 新查询不再经死 handle 解析（R100 `__zw_handle_for_selector` 反查对已移除
//     元素失映射，回落 sel proxy 面——slice32/33 修后语义）；
// (b) JS 已持有的旧 proxy 读回落 R3029 语义保持（detached 读 + 重挂 identity 翻转，
//     R315 `_recordHandleChild` 缓存对账）——账本不清正是重挂 identity 稳定的
//     前提（R52 消零清除曾致 proxy 分裂，R315 翻转修复）。
// 清除机制（如 remFlat 逐出）会重引 R52/R315 一类 identity 分裂缺陷，边界保持。
#[test]
fn named_access_dead_handle_proxy_entry_boundary_s42() {
    let mut sandbox = s42_sandbox!("<html><body></body></html>");
    sandbox
        .execute(
            "var img = document.createElement('img');
             document.body.appendChild(img);
             var captured = document.querySelector('img');
             globalThis.__r_r100 = captured === img;
             // (a) 移除后新查询不得经死 handle 解析（返回新元素 identity）
             document.body.removeChild(img);
             globalThis.__r_detached_read = captured.nodeType === 1;
             var img2 = document.createElement('img');
             document.body.appendChild(img2);
             var q2 = document.querySelector('img');
             globalThis.__r_fresh = q2 === img2 && q2 !== captured;
             // (b) 旧 proxy 重挂 identity 翻转保持（页面持有身份继续有效）
             document.body.removeChild(img2);
             document.body.appendChild(captured);
             globalThis.__r_regraft = document.querySelector('img') === captured;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_r100").unwrap().value,
        "true",
        "查询返回点 identity 反查（R100 基线：createElement proxy 复用）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_detached_read").unwrap().value,
        "true",
        "死 handle 旧 proxy detached 读回落（R3029 语义保持）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_fresh").unwrap().value,
        "true",
        "移除后新查询不再经死 handle 解析（新元素 identity，R100 修后语义钉）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_regraft").unwrap().value,
        "true",
        "旧 proxy 重挂 identity 翻转保持（R315：账本不清是 identity 稳定前提）"
    );
}
