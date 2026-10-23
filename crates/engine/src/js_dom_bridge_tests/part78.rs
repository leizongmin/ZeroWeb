// part78 — html-semantics M2 切片 46（R5249，2026-10-23）：历史 menu 面 IDL 归属
// 收口——track/option IDL `in` 可见性 tag 门 + R3069 reflected 写臂 MENU/MENUITEM
// 排除（contextmenu-historical 3F）。
//
// 根因（三半边，均「ID L 名无 tag 门」同模式）：①has trap（part05）track IDL 列
// `label`/`kind`/`srclang`/`default`（media-elements M1 切片 3 加）无条件命中——
// 历史 menu/menuitem（现代 spec 无任何 IDL 属性，menuitem 是 HTMLUnknownElement）
// `'label' in el` 假阳性；②set trap R3069 臂 `_reflectedStringAttr` 的
// _REFLECTED_STRING_FLAT 含 `type` 无 tag 排除——`menu.type =` 误写内容属性
//（getAttribute 反射漂移）；③同臂 `p === 'label'` 显式列（R3069-fix）无 tag 门
// ——`menu.label =` 同误写。
//
// 修复双刀：①has trap `kind`/`srclang` 收 TRACK 门、`label`/`default` 收
// TRACK+OPTION 门（HTMLOptionElement.label/.default 读侧 R3069-fix 对称）；②
// R3069 臂条件前置 MENU/MENUITEM 负门——赋值落 expando 默认路径，内容属性不变。
// https://html.spec.whatwg.org/multipage/media.html#htmltrackelement
// https://html.spec.whatwg.org/multipage/interaction.html#the-menuelement-object
// （historical：type/label IDL 随 HTMLMenuItemElement 删除，whatwg/html PR #2742）

/// 钉①：历史 menu 面 IDL 排除——(a) menuitem（HTMLUnknownElement）无 track IDL
/// 成员（label/default/kind `in` 均 false）；(b) menu `'label' in` false；(c)
/// menu.type= 赋值不写内容属性；(d) menu.label= 同；(e) menuitem.constructor 为
/// HTMLUnknownElement（既有面，防刀位误伤元素构造语义）。
/// 断言翻转：(a)(b) = has trap tag 门被撤（假阳性复活）；(c)(d) = R3069 负门被撤
///（内容属性反射漂移复活）。
#[test]
fn historical_menu_idl_exclusion_r5249() {
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
        "<html><body><menu type='context' label='label'><menuitem id='mi'>Text</menuitem></menu></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47e".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var menu = document.querySelector('menu');\
             var mi = document.querySelector('menuitem');\
             globalThis.__r5249a = String(!('label' in mi) && !('default' in mi) && !('kind' in mi));\
             globalThis.__r5249b = String(!('label' in menu));\
             menu.type = 'toolbar';\
             globalThis.__r5249c = String(menu.getAttribute('type') === 'context');\
             menu.label = 'new label';\
             globalThis.__r5249d = String(menu.getAttribute('label') === 'label');\
             globalThis.__r5249e = String(mi.constructor === globalThis.HTMLUnknownElement);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5249a").unwrap().value,
        "true",
        "(a) menuitem 无 track IDL 成员（label/default/kind in 均 false——has trap tag 门）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5249b").unwrap().value,
        "true",
        "(b) menu 无 label IDL 成员（has trap TRACK/OPTION 门不含 MENU）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5249c").unwrap().value,
        "true",
        "(c) menu.type= 赋值不写内容属性（R3069 负门，getAttribute 保持 context）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5249d").unwrap().value,
        "true",
        "(d) menu.label= 赋值不写内容属性（getAttribute 保持 label）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5249e").unwrap().value,
        "true",
        "(e) menuitem.constructor 为 HTMLUnknownElement（既有面，刀位不伤元素构造语义）"
    );
}

/// 钉②：门内既有面不回归——track IDL 四成员 `in` true（TRACK 门命中）、option
/// label/default `in` true（OPTION 门命中）、option.label= round-trip 写内容属性
///（R3069-fix 面保持）、input.type= round-trip（FLAT 表非 MENU 面不受 R3069 负门
/// 影响）。
/// 断言翻转：(a) = tag 门误杀 track/option（门过宽收窄回归）；(b)(c) = R3069 负门
/// 扩大到非 MENU 面（round-trip 断）。
#[test]
fn track_option_idl_gate_kept_r5249() {
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
        "<html><body><video><track kind='subtitles' label='L' srclang='en' default></video>\
         <select><option id='opt'>o</option></select><input id='inp' type='text'></body></html>"
            .to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("https://zero.test/s47f".to_string()));
    let canvas_registry: Arc<Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    sandbox
        .execute(
            "var track = document.querySelector('track');\
             var opt = document.querySelector('#opt');\
             var inp = document.querySelector('#inp');\
             globalThis.__r5249f = String(('kind' in track) && ('srclang' in track) \
                 && ('label' in track) && ('default' in track));\
             globalThis.__r5249g = String(('label' in opt) && ('default' in opt));\
             opt.label = 'x';\
             globalThis.__r5249h = String(opt.getAttribute('label') === 'x');\
             inp.type = 'checkbox';\
             globalThis.__r5249i = String(inp.getAttribute('type') === 'checkbox');",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r5249f").unwrap().value,
        "true",
        "(a) track 四 IDL 成员 in 可见性保持（TRACK 门命中，media 切片 3 面不回归）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5249g").unwrap().value,
        "true",
        "(b) option label/default in 可见性保持（OPTION 门命中）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5249h").unwrap().value,
        "true",
        "(c) option.label= round-trip 写内容属性保持（R3069-fix 面）"
    );
    assert_eq!(
        sandbox.execute("globalThis.__r5249i").unwrap().value,
        "true",
        "(d) input.type= round-trip 保持（FLAT 表面不受 MENU 排除影响）"
    );
}
