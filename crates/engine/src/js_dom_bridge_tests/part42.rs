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

// t8g（site-compat bilibili-20261002-r1）回归钉：in-document 结构性二次方税根因修复。
// 根因链：`_unrecordHandleChild` R91 段引用未声明标识符 `ch` → `typeof ch` 沿全局链
// 落到 WindowProperties NPO has trap → 每次 removeChild 触发全局命名查找 →
// `_zwNPOIfrScan` 每次 cache miss（每 mutation bump ifrGen）getElementsByTagName
// 新建 live HTMLCollection 永久注册 `_zwLiveCollections` → 集合数随操作数线性涨 →
// `_zwHCLiveInvalidate` 每 mutation 遍历全部集合 × 每集合 matches 逐元素 tag 解析
// → O(ops²)。真站实证：400 对 append/remove 泄漏 409 集合、_realTag 43·i 次/对。

/// 泄漏钉：removeChild 循环不触发集合累积——600 对 append/remove（每对含一次
/// 全局命名查找，驱动 `_zwNPOIfrScan` 扫描路径真实执行）墙钟有界 + 机制断言：
/// `__zwLiveCollectionsStore.length === 0`（context 级稳定 global，页面可读，
/// 机器无关——扫描实现若回退为向登记表注册集合的形态，此断言先于墙钟报警；
/// 双审查红→绿实验实证回退态泄漏 436 集合/400 次命名查找）。修复前同形态约
/// 8s 量级（二次方 + 每对 1 个新集合的失效遍历）；修复后线性，进程内毫秒级。
/// 绝对阈值给足裕量（<2000ms），避免 CI 机器噪声误报。
#[test]
fn test_t8g_npo_iframe_scan_no_collection_leak() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    let t0 = std::time::Instant::now();
    sandbox
        .execute(
            "var c = document.createElement('div');\
             document.body.appendChild(c);\
             var d = document.createElement('div');\
             for (var i = 0; i < 600; i++) {\
               c.appendChild(d); c.removeChild(d);\
               globalThis.__t8gNpoProbe = typeof ch;\
             }\
             globalThis.__t8gStoreLen = globalThis.__zwLiveCollectionsStore.length;\
             globalThis.__t8gDone = true;",
        )
        .unwrap();
    let elapsed = t0.elapsed();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__t8gDone)")
            .unwrap()
            .value,
        "true",
        "600 对 append/remove 正常完成"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__t8gStoreLen)")
            .unwrap()
            .value,
        "0",
        "循环内命名查找不得向 _zwLiveCollections 泄漏注册任何集合（扫描应零注册）"
    );
    assert!(
        elapsed.as_millis() < 2000,
        "600 对 append/remove 应线性完成（实测 {:?}；二次方回归时为秒级到分钟级）",
        elapsed
    );
}

/// NPO 语义钉：修复不得破坏 Window 命名属性解析——`ch`（旧笔误标识符）不得被
/// 解析成任何值，命名 iframe 访问在 mutation 后仍走通（iframe 扫描为 qSA 静态
/// 枚举，每次现查反映当前文档）。
#[test]
fn test_t8g_window_named_properties_intact() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><iframe name=\"frameA\"></iframe></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    sandbox
        .execute(
            "var c = document.createElement('div');\
             document.body.appendChild(c);\
             var d = document.createElement('div');\
             for (var i = 0; i < 50; i++) { c.appendChild(d); c.removeChild(d); }\
             globalThis.__chType = typeof ch;\
             globalThis.__chIn = 'ch' in globalThis;\
             globalThis.__ifrLen = document.getElementsByTagName('iframe').length;\
             globalThis.__frameAType = typeof window.frameA;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__chType").unwrap().value,
        "undefined",
        "未声明标识符 `ch`（旧 R91 笔误名）不得解析为任何命名属性"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__chIn)").unwrap().value,
        "false",
        "`ch` 不得出现在 global 命名面"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__ifrLen)").unwrap().value,
        "1",
        "mutation 后 iframe 集合仍正确反映文档"
    );
    // 命名 iframe 访问：快照命名 iframe 经 NPO 主查找解析为其 contentWindow
    // （window 形态对象，非元素代理）。
    let v = sandbox.execute("globalThis.__frameAType").unwrap().value;
    assert_eq!(
        v, "object",
        "window.frameA 命名访问应解析为元素对象（不得抛异常或落空）"
    );
}

/// isConnected 语义钉：removeChild 后节点立即脱离文档可观测面，且同节点重挂载
/// 到新父后恢复连接（red→green 归因注记：双审查回退实验实证 isConnected 面由
/// `_mo_notify` 汇流点冗余保障、回退下同样绿——本钉为语义 green-guard，不判别
/// R91 清理行本身；R91 修复的检出力在泄漏钉的墙钟 + 机制断言）。
#[test]
fn test_t8g_r91_backlink_cleanup_after_remove() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new("<html><body></body></html>".to_string()));
    let page_url = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );
    sandbox
        .execute(
            "var a = document.createElement('div');\
             document.body.appendChild(a);\
             var b = document.createElement('div');\
             document.body.appendChild(b);\
             var d = document.createElement('div');\
             a.appendChild(d);\
             globalThis.__connInA = d.isConnected;\
             a.removeChild(d);\
             globalThis.__connAfterRemove = d.isConnected;\
             b.appendChild(d);\
             globalThis.__connAfterRemount = d.isConnected;",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("String(globalThis.__connInA)")
            .unwrap()
            .value,
        "true",
        "挂载期间 isConnected 为真"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__connAfterRemove)")
            .unwrap()
            .value,
        "false",
        "removeChild 后 isConnected 立即为假（反链清理语义面）"
    );
    assert_eq!(
        sandbox
            .execute("String(globalThis.__connAfterRemount)")
            .unwrap()
            .value,
        "true",
        "同节点重挂载到新父后 isConnected 恢复为真（反链重写语义不受修复影响）"
    );
}
