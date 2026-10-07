// js_dom_bridge 测试模块拆分 part 40（t8c：`__zw_get_tag_handle` 增量 handle→tag 备忘）。
// 根因：get trap 每属性读经 `_realTag` 打宿主 tag 查询，底源 `query_tag_from_mutations`
// 全队列逆序线性扫——队列随结构操作增长，append/remove 循环 O(n²)（沙箱实测 200 append
// 触发 44 万次查询、831ms/1098ms）。钉测覆盖：备忘化后 tag 解析正确性（行为）+ 备忘单元
// 语义（增量水位 / latest-wins / drain 重置）+ handle 路径 childNodes 融合视图回归守卫。

#[test]
fn t8c_tag_resolution_correct_through_bulk_appends() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8c-tag".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 大批量 append 队列持续增长后，handle 元素 tag 解析（`_realTag` → 备忘）仍正确：
    // tagName 大写语义、firstChild/lastChild 定位、AREA gate 判定源同链。
    sandbox
        .execute(
            "var p = document.createElement('div');\n\
             for (var i = 0; i < 300; i++) p.appendChild(document.createElement('span'));\n\
             var area = document.createElement('area');\n\
             p.appendChild(area);\n\
             globalThis.__firstTag = p.firstChild.tagName;\n\
             globalThis.__lastTag = p.lastChild.tagName;\n\
             globalThis.__areaTag = area.tagName;\n\
             globalThis.__areaLower = p.lastChild.tagName.toLowerCase();\n\
             var em = document.createElementNS('http://www.w3.org/1999/xhtml', 'em');\n\
             p.appendChild(em);\n\
             globalThis.__nsTag = em.tagName;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__firstTag").unwrap().value,
        "SPAN",
        "t8c：300 append 后首子 tag 解析正确（备忘不串 tag）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__lastTag").unwrap().value,
        "AREA",
        "t8c：尾子 tag 解析正确（AREA gate 判定源）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__areaLower").unwrap().value,
        "area",
        "t8c：tag 小写化读正确"
    );
    assert_eq!(
        sandbox.execute("globalThis.__nsTag").unwrap().value,
        "EM",
        "t8c：createElementNS 产物 tag 解析正确（CreateElementNS 记录入备忘）"
    );
}

#[test]
fn t8c_handle_childnodes_fusion_guard() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations = Arc::new(Mutex::new(Vec::<DomMutation>::new()));
    let dom_html = Arc::new(Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://zero.test/t8c-fusion".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // 本切片触碰 tag 解析链（child 身份/tag 读），handle 路径 childNodes 融合视图
    //（registry ∪ pending added − pending removed）语义回归守卫：重复读长度恒定
    //（pending identity 剔除不双计）、removeChild 后子消失不回魂、二轮 append 可见。
    sandbox
        .execute(
            "var p = document.createElement('div');\n\
             var c1 = document.createElement('span');\n\
             var c2 = document.createElement('em');\n\
             p.appendChild(c1);\n\
             p.appendChild(c2);\n\
             var l1 = p.childNodes.length;\n\
             var l2 = p.childNodes.length;\n\
             globalThis.__lens = l1 + ',' + l2;\n\
             p.removeChild(c1);\n\
             var vAfter = p.childNodes;\n\
             globalThis.__lenAfterRemove = vAfter.length;\n\
             globalThis.__c2First = (vAfter[0] === c2);\n\
             globalThis.__c1Gone = Array.prototype.indexOf.call(vAfter, c1) === -1;\n\
             var c3 = document.createElement('b');\n\
             p.appendChild(c3);\n\
             globalThis.__lenThird = p.childNodes.length;\n\
             globalThis.__c3In = Array.prototype.indexOf.call(p.childNodes, c3) !== -1;",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__lens").unwrap().value,
        "2,2",
        "t8c：handle 路径重复读 childNodes 长度恒定（不双计）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__lenAfterRemove").unwrap().value,
        "1",
        "t8c：removeChild 后融合视图长度 1"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c2First").unwrap().value,
        "true",
        "t8c：removeChild 后 c2 位次正确"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c1Gone").unwrap().value,
        "true",
        "t8c：removed 子从融合视图消失"
    );
    assert_eq!(
        sandbox.execute("globalThis.__lenThird").unwrap().value,
        "2",
        "t8c：二轮 append 立即可见"
    );
    assert_eq!(
        sandbox.execute("globalThis.__c3In").unwrap().value,
        "true",
        "t8c：二轮 append 的 c3 在融合视图中"
    );
}
