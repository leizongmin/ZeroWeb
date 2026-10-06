// slice33（RP-3 收口 + 缺陷轮 B-1 收敛）：Window named access 面补钉。①S-2 失格
// 路径——removeAttribute('id'/'name')（slice32 钉仅覆盖 setAttribute 面，remove 面
// 同经 _zwNAAttrChanged 重核，本钉补资产）；②B-1 白名单边界——iframe 名**不进**
// 元素集合面（spec name 面供名元素逐字仅 embed/form/img/object；iframe 名属
// document-tree child navigable target name 通道，named object 值 = active
// WindowProxy（contentWindow）且取值算法 navigable 优先于元素，引擎侧由 R139
// `__zwRegisterNamedIframes` 委托承载；缺陷轮 I-1 曾误收 iframe 入面，B-1 撤出）；
// ③I-7 再生边界——0 命中回收后成员重入全局恢复（2→0→1 修前恒 undefined）。
// https://html.spec.whatwg.org/multipage/window-object.html#named-access-on-the-window-object
// https://dom.spec.whatwg.org/#concept-collection-live

// ①S-2 id 失格：双 id=mm 集合安装后 removeAttribute('id') → 成员剔除 + 跌破 2
// morph 元素（修前无钉资产；_zwNAAttrChanged remove 面行为保持）。
#[test]
fn named_access_remove_attribute_id_disqualifies_s33() {
    let mut sandbox = s32_sandbox!("<html><body><div id='mm'></div><div id='mm' id2='x'></div></body></html>");
    sandbox
        .execute(
            "var col0 = window.mm;
             globalThis.__r_len0 = col0.length;
             var second = document.querySelectorAll('[id=mm]')[1];
             second.removeAttribute('id');
             globalThis.__r_len1 = col0.length;
             globalThis.__r_morph = window.mm === document.querySelector('[id=mm]');
             globalThis.__r_is_col = window.mm === col0;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "双 id=mm 安装集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "1",
        "removeAttribute('id') 后集合剔除失格成员（S-2 remove 面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_morph").unwrap().value,
        "true",
        "跌破单命中后 window.mm morph 为唯一元素（spec 取值算法）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_is_col").unwrap().value,
        "false",
        "全局不再是集合对象（morph 生效）"
    );
}

// ①S-2 name 失格：embed+form 同名集合安装后 removeAttribute('name') → 剔除 + morph。
#[test]
fn named_access_remove_attribute_name_disqualifies_s33() {
    let mut sandbox = s32_sandbox!(
        "<html><body><embed name='cc' id='e1'></embed><form name='cc' id='f1'></form></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.cc;
             globalThis.__r_len0 = col0.length;
             document.getElementById('f1').removeAttribute('name');
             globalThis.__r_len1 = col0.length;
             globalThis.__r_morph = window.cc === document.getElementById('e1');")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "双 name=cc 安装集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "1",
        "removeAttribute('name') 后集合剔除失格成员（S-2 remove 面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_morph").unwrap().value,
        "true",
        "跌破单命中后 window.cc morph 为唯一元素（spec 取值算法）"
    );
}

// ②B-1 白名单边界：iframe 名不进元素集合面。双 iframe name=fr 在安装器面不装
// 集合（spec name 面逐字仅 embed/form/img/object）；既有 img 同名 live 集合不因
// appendChild iframe 并入成员（live matcher 面——_zwNAElemMatches 无 iframe 臂）。
// iframe 名的全局值面 = R139 `__zwRegisterNamedIframes` 委托（contentWindow 即
// WindowProxy，load 派发物化 + 首读 lazy 注册）——本钉环境无 load 物化，R139 面
// 以浏览器探针归档（diag/evidence/slice33/rework/ iframe-face before/after PNG）。
#[test]
fn named_access_iframe_whitelist_boundary_s33() {
    let mut sandbox = s32_sandbox!(
        "<html><body><iframe name='fr'></iframe><iframe name='fr'></iframe>\
         <img name='zz'><img name='zz'></body></html>"
    );
    sandbox
        .execute(
            "globalThis.__r_fr_absent = typeof window.fr === 'undefined';
             globalThis.__r_fr_not_col = !(window.fr && typeof window.fr.item === 'function');
             globalThis.__r_zz_len0 = window.zz.length;
             var f3 = document.createElement('iframe');
             f3.setAttribute('name', 'zz');
             document.body.appendChild(f3);
             globalThis.__r_zz_len1 = window.zz.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_fr_absent").unwrap().value,
        "true",
        "iframe 名不进安装器面（spec name 面仅 embed/form/img/object，R139 委托面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_fr_not_col").unwrap().value,
        "true",
        "window.fr 不是 iframe 元素 HTMLCollection（B-1 白名单边界）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_zz_len0").unwrap().value,
        "2",
        "img 同名多命中照常安装集合（白名单内元素不受影响）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_zz_len1").unwrap().value,
        "2",
        "appendChild iframe 后 live 集合不并入 iframe（_zwNAElemMatches 无 iframe 臂）"
    );
}

// ③I-7 再生：2 命中集合 → 双双失格（1 命中 morph 元素——slice32 边界钉保持；
// 0 命中回收保留账本）→ 全局清空后同名新元素重入 → 恢复（1 命中恢复元素）。
// 修前账本随回收删除、重入后恒 undefined（再生缺口，slice32 缺陷轮 I-7 边界）。
#[test]
fn named_access_recycled_collection_regenerates_s33() {
    let mut sandbox = s32_sandbox!(
        "<html><body><img name='zz' id='z1'><img name='zz' id='z2'></body></html>"
    );
    sandbox
        .execute(
            "var col0 = window.zz;
             globalThis.__r_len0 = col0.length;
             document.getElementById('z1').removeAttribute('name');
             document.getElementById('z2').removeAttribute('name');
             globalThis.__r_len1 = col0.length;
             delete window.zz; // renderer 快照换代登记·回收链路对 stale 名的同款清理
             globalThis.__r_absent0 = typeof window.zz === 'undefined';
             var z3 = document.createElement('img');
             z3.setAttribute('name', 'zz');
             document.body.appendChild(z3);
             globalThis.__r_restored = window.zz === z3;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "双 name=zz 安装集合"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "0",
        "双失格后集合成员清空（0 命中回收态）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_absent0").unwrap().value,
        "true",
        "全局清空后 window.zz 缺席"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_restored").unwrap().value,
        "true",
        "同名重入后全局恢复为唯一元素（I-7 再生面，修前恒 undefined）"
    );
}
