// js_dom_bridge_tests part11（R2962 tests 拆分后续片）——自 part10 溢出迁移的新增测试。
// 与 part01..10 同处 js_dom_bridge::tests 模块（经 include!），共享 super::* 导入。

#[test]
fn test_document_evaluate_xpath_r2981() {
    // R2981：document.evaluate / XPathResult——此前全缺（document.evaluate 与 XPathResult 零定义）→ 任何
    // XPath 查询抛 ReferenceError 中断脚本。补实用 XPath 1.0 子集：路径（//、/、相对 child）、节点测试
    //（tag/*/text()）、谓词（[n]/[last()]/[@a]/[@a='v']/[contains()]/[text()='v']/[position() op n]）、
    // 属性轴结果（@attr → 伪 Attr 节点）、多上下文 dedup、snapshot/iterateNext/singleNodeValue 表面。
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};
    let config = zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    };
    let mut sandbox = V8Sandbox::with_config(config).unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    // ul#list > li.item/active(A/B/C/D)；div#box > 2 p；3 a（含 class=ext 的 /c）。
    let dom_html: Arc<Mutex<String>> = Arc::new(Mutex::new(
        "<html><body>\
         <ul id='list'>\
         <li class='item'>A</li>\
         <li class='item active'>B</li>\
         <li class='item'>C</li>\
         <li class='active'>D</li>\
         </ul>\
         <div id='box'><p>hello</p><p>world</p></div>\
         <a href='/a'>link-a</a>\
         <a href='/b'>link-b</a>\
         <a href='/c' class='ext'>link-c</a>\
         </body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url);

    // 表面存在性 + XPathResult 常量。
    assert_eq!(
        sandbox.execute("typeof document.evaluate").unwrap().value,
        "function",
        "document.evaluate 是 function"
    );
    assert_eq!(
        sandbox.execute("typeof XPathResult").unwrap().value,
        "object",
        "XPathResult 是 object"
    );
    assert_eq!(
        sandbox
            .execute("String(XPathResult.ORDERED_NODE_SNAPSHOT_TYPE)")
            .unwrap()
            .value,
        "7",
        "XPathResult.ORDERED_NODE_SNAPSHOT_TYPE = 7"
    );

    // 辅助：求值 → 快照结果。
    sandbox
        .execute(
            "globalThis.__xp = function(expr, ctx, type) {\
             return document.evaluate(expr, ctx || document, null,\
             (type == null ? XPathResult.ORDERED_NODE_SNAPSHOT_TYPE : type), null);\
             };",
        )
        .unwrap();

    // //li：descendant 全部 li = 4，resultType=7（ORDERED_NODE_SNAPSHOT）。
    assert_eq!(
        sandbox
            .execute("String(__xp('//li').resultType)")
            .unwrap()
            .value,
        "7",
        "//li resultType = ORDERED_NODE_SNAPSHOT_TYPE(7)"
    );
    assert_eq!(
        sandbox.execute("__xp('//li').snapshotLength").unwrap().value,
        "4",
        "//li snapshotLength = 4"
    );

    // //li[1]：descendant 候选集首位 = A。
    assert_eq!(
        sandbox
            .execute("__xp('//li[1]').snapshotItem(0).textContent")
            .unwrap()
            .value,
        "A",
        "//li[1] = A（descendant 候选集首位）"
    );

    // //li[last()]：末位 = D。
    assert_eq!(
        sandbox
            .execute("__xp('//li[last()]').snapshotItem(0).textContent")
            .unwrap()
            .value,
        "D",
        "//li[last()] = D"
    );

    // //li[@class='item']：精确 class=item → A、C（B 为 'item active'，D 为 'active'）= 2。
    assert_eq!(
        sandbox
            .execute("__xp(\"//li[@class='item']\").snapshotLength")
            .unwrap()
            .value,
        "2",
        "//li[@class='item'] = 2（A/C，精确匹配）"
    );

    // //li[contains(@class,'active')] → B、D = 2。
    assert_eq!(
        sandbox
            .execute("__xp(\"//li[contains(@class,'active')]\").snapshotLength")
            .unwrap()
            .value,
        "2",
        "//li[contains(@class,'active')] = 2（B/D）"
    );

    // 谓词链：//li[@class='item'][2] → class=item（A/C）中第 2 = C。
    assert_eq!(
        sandbox
            .execute("__xp(\"//li[@class='item'][2]\").snapshotItem(0).textContent")
            .unwrap()
            .value,
        "C",
        "//li[@class='item'][2] = C（谓词链顺序应用）"
    );

    // position() op：//li[position()>=3] → C、D = 2。
    assert_eq!(
        sandbox
            .execute("__xp('//li[position()>=3]').snapshotLength")
            .unwrap()
            .value,
        "2",
        "//li[position()>=3] = 2（C/D）"
    );

    // child 轴 per-parent 位置：//ul/li[2] → ul 的第 2 个 li = B。
    assert_eq!(
        sandbox
            .execute("__xp('//ul/li[2]').singleNodeValue.textContent")
            .unwrap()
            .value,
        "B",
        "//ul/li[2] = B（child 轴 per-parent 位置）"
    );

    // FIRST_ORDERED_NODE_TYPE(9) singleNodeValue。
    assert_eq!(
        sandbox
            .execute(
                "String(__xp('//ul/li[2]', null, XPathResult.FIRST_ORDERED_NODE_TYPE).resultType)"
            )
            .unwrap()
            .value,
        "9",
        "FIRST_ORDERED_NODE_TYPE resultType = 9"
    );

    // last()-n：//ul/li[last()-1] → 第 3 个 li = C。
    assert_eq!(
        sandbox
            .execute("__xp('//ul/li[last()-1]').singleNodeValue.textContent")
            .unwrap()
            .value,
        "C",
        "//ul/li[last()-1] = C"
    );

    // text() 谓词：//a[text()='link-b'] → link-b = 1。
    assert_eq!(
        sandbox
            .execute("__xp(\"//a[text()='link-b']\").snapshotLength")
            .unwrap()
            .value,
        "1",
        "//a[text()='link-b'] = 1"
    );
    // contains(@attr,'sub')：//a[contains(@href,'b')] → /b = 1。
    assert_eq!(
        sandbox
            .execute("__xp(\"//a[contains(@href,'b')]\").snapshotLength")
            .unwrap()
            .value,
        "1",
        "//a[contains(@href,'b')] = 1"
    );

    // 属性轴结果：//a/@href → 3 个伪 Attr 节点，[0].value='/a'。
    assert_eq!(
        sandbox
            .execute("__xp('//a/@href').snapshotLength")
            .unwrap()
            .value,
        "3",
        "//a/@href = 3（属性轴伪 Attr 节点）"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//a/@href').snapshotItem(0).value")
            .unwrap()
            .value,
        "/a",
        "//a/@href[0].value = '/a'"
    );

    // 相对路径（child 轴 from context）：#box 子 p = 2。
    assert_eq!(
        sandbox
            .execute(
                "globalThis.__box = document.querySelector('#box');\
                 __xp('p', __box).snapshotLength"
            )
            .unwrap()
            .value,
        "2",
        "相对 'p' from #box = 2（child 轴）"
    );

    // iterateNext() 游标：//a 迭代 3 次后返 null。
    assert_eq!(
        sandbox
            .execute(
                "globalThis.__it = __xp('//a', null, XPathResult.ORDERED_NODE_ITERATOR_TYPE);\
                 globalThis.__cnt = 0;\
                 while (__it.iterateNext()) globalThis.__cnt++;\
                 String(__it.iterateNext());"
            )
            .unwrap()
            .value,
        "null",
        "iterateNext() 迭代完返 null"
    );
    assert_eq!(
        sandbox.execute("String(globalThis.__cnt)").unwrap().value,
        "3",
        "//a iterateNext 计数 = 3"
    );

    // 无匹配：singleNodeValue=null，snapshotLength=0。
    assert_eq!(
        sandbox
            .execute("String(__xp('//nonexistent').singleNodeValue === null)")
            .unwrap()
            .value,
        "true",
        "//nonexistent singleNodeValue = null"
    );
    assert_eq!(
        sandbox
            .execute("__xp('//nonexistent').snapshotLength")
            .unwrap()
            .value,
        "0",
        "//nonexistent snapshotLength = 0"
    );

    // 无效表达式抛 Error（spec INVALID_EXPRESSION_ERR 语义）。
    sandbox
        .execute(
            "globalThis.__threw = 'no';\
             try { document.evaluate('', document, null, 6, null); }\
             catch (e) { globalThis.__threw = (e instanceof TypeError || e instanceof Error) ? 'yes' : 'other'; }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__threw)").unwrap().value,
        "yes",
        "空 XPath 表达式抛 Error（honest failure）"
    );
}
