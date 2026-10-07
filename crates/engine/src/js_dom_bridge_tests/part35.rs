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

// ②B-1 白名单边界：iframe 名不进**元素集合面**（spec name 面逐字仅 embed/form/img/
// object，_zwNAElemMatches 无 iframe 臂）；既有 img 同名 live 集合不因 appendChild
// iframe 并入成员。iframe 名的全局**值面** slice37 起 = NPO live 扫描（Chrome 实测 +
// WPT window-named-properties "Static name: bar"——bar 即 iframe 名，无 load 派发也
// 可见；值取首棵 contentWindow，R115/R139 物化面供给）——旧钉「fr 无 load 不见」是
// slice33 lazy 装载面残留，按新语义翻转。ret 面：window.fr 可见但不是 iframe 元素
// HTMLCollection。
#[test]
fn named_access_iframe_whitelist_boundary_s33() {
    let mut sandbox = s32_sandbox!(
        "<html><body><iframe name='fr'></iframe><iframe name='fr'></iframe>\
         <img name='zz'><img name='zz'></body></html>"
    );
    sandbox
        .execute(
            "globalThis.__r_fr_present = typeof window.fr !== 'undefined';
             globalThis.__r_fr_not_col = !(window.fr && typeof window.fr.item === 'function');
             globalThis.__r_zz_len0 = window.zz.length;
             var f3 = document.createElement('iframe');
             f3.setAttribute('name', 'zz');
             document.body.appendChild(f3);
             globalThis.__r_zz_len1 = window.zz.length;")
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r_fr_present").unwrap().value,
        "true",
        "iframe 名进 named property 值面（WPT Static name 面，NPO live 扫描）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_fr_not_col").unwrap().value,
        "true",
        "window.fr 不是 iframe 元素 HTMLCollection（B-1 白名单边界：集合面不含 iframe）"
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

// ③I-7 再生：2 命中集合 → 双双失格（1 命中 morph 元素；slice42 收口：0 命中时
// morph 产物 + backing 原集合双面回收，原「delete 后复活」的 stale 面已退役）→
// 回收态缺席 → 同名新元素重入 → 恢复（1 命中恢复元素）；live 名 window 级 delete
// 不清 named prop（slice36 FIXME ⑥ 收口：再读复活，Chrome 同款）——该面移到恢复
// 后的 live 名上验证。
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
             globalThis.__r_recycled = typeof window.zz === 'undefined';
             delete window.zz; // NPO 级 [[Delete]]=false 经链传播——no-op（ret 值不钉，偏差申报）
             globalThis.__r_absent0 = typeof window.zz === 'undefined';
             var z3 = document.createElement('img');
             z3.setAttribute('name', 'zz');
             document.body.appendChild(z3);
             globalThis.__r_restored = window.zz === z3;
             delete window.zz; // live 名 window 级 delete 不清 named prop（FIXME ⑥，NPO false）
             globalThis.__r_resurrect = window.zz === z3;")
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
        sandbox.execute("globalThis.__r_recycled").unwrap().value,
        "true",
        "0 命中双面回收（slice42 收口；修前 stale 元素/backing 残留复活）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_absent0").unwrap().value,
        "true",
        "回收态 window.zz 缺席（修后 __zwNADelete 已在 morph 尾叫完成）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_restored").unwrap().value,
        "true",
        "同名重入后全局恢复为唯一元素（I-7 再生面，修前恒 undefined）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r_resurrect").unwrap().value,
        "true",
        "live 名 window 级 delete 后再读复活（FIXME ⑥ 收口，Chrome 同款）"
    );
}
