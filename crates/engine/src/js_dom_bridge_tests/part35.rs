// slice33（RP-3 收口）：Window named access 面补钉。①S-2 失格路径——removeAttribute
// ('id'/'name')（slice32 钉仅覆盖 setAttribute 面，remove 面同经 _zwNAAttrChanged
// 重核，本钉补资产）；②I-1 name 面白名单补 iframe（spec named objects 含
// HTMLIFrameElement）——安装器（Rust 采集器 + shim _namedAccessMatches）与 live
// matcher（_zwNAElemMatches）同口径；③I-7 再生边界——0 命中回收后成员重入全局
// 恢复（2→0→1 修前恒 undefined）。
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

// ②I-1 iframe 名面：双 iframe name=fr 安装集合（安装器面——Rust 采集器
// collect_ids_multi + shim _namedAccessMatches 同口径）；appendChild 第三 iframe
// 集合即时 +1（live matcher 面——_zwNAElemMatches iframe 臂）。
#[test]
fn named_access_iframe_name_face_s33() {
    let mut sandbox = s32_sandbox!(
        "<html><body><iframe name='fr'></iframe><iframe name='fr'></iframe></body></html>"
    );
    sandbox
        .execute(
            "globalThis.__r_is_col = window.fr && typeof window.fr.item === 'function';
             globalThis.__r_len0 = window.fr ? window.fr.length : -1;
             var f3 = document.createElement('iframe');
             f3.setAttribute('name', 'fr');
             document.body.appendChild(f3);
             globalThis.__r_len1 = window.fr.length;
             globalThis.__r_member = window.fr[2] === f3;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_is_col").unwrap().value,
        "true",
        "双 iframe name=fr 安装集合（I-1 安装器面，修前 iframe 不入面 → undefined）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len0").unwrap().value,
        "2",
        "集合长度 = 命中数"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_len1").unwrap().value,
        "3",
        "appendChild iframe 后集合即时 +1（I-1 live matcher 面，修前恒 2/不入）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_member").unwrap().value,
        "true",
        "新 iframe 成员并入集合"
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
