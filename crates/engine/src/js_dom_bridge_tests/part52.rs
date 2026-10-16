#[test]
fn html_collection_array_from_and_for_of_accept_symbol_iterator() {
    use std::sync::{Arc, Mutex};
    use zero_script_sandbox::{Sandbox, V8Sandbox};

    let mut sandbox = V8Sandbox::with_config(zero_script_sandbox::SandboxConfig {
        persistent_context: true,
        ..Default::default()
    })
    .unwrap();
    sandbox.execute(generate_js_dom_shim()).unwrap();
    let mutations: Arc<Mutex<Vec<DomMutation>>> = Arc::new(Mutex::new(vec![]));
    let dom_html = Arc::new(Mutex::new(
        "<html><body><img id='first' alt='first'><img id='second' alt='second'>\
         <form id='form'><input id='control'><select id='choice'>\
         <option id='one'>1</option><option id='two'>2</option></select></form></body></html>"
            .to_string(),
    ));
    let page_url = Arc::new(Mutex::new("https://example.test/".to_string()));
    let canvas_registry = Arc::new(Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(
        &mut sandbox,
        &mutations,
        &dom_html,
        &page_url,
        &canvas_registry,
        None,
    );

    // https://dom.spec.whatwg.org/#interface-htmlcollection
    sandbox
        .execute(
            "globalThis.__imageIds = Array.from(document.images).map(image => image.id).join(',');\
             globalThis.__iteratedIds = [];\
             for (const image of document.images) globalThis.__iteratedIds.push(image.id);\
             globalThis.__iteratedIds = globalThis.__iteratedIds.join(',');\
             globalThis.__imageAttributeNames = Array.from(document.images[0].attributes)\
               .map(attribute => attribute.name).join(',');\
             globalThis.__optionIds = Array.from(document.getElementById('choice').options)\
               .map(option => option.id).join(',');\
             globalThis.__formControlIds = Array.from(document.getElementById('form').elements)\
               .map(control => control.id).join(',');",
        )
        .unwrap();

    assert_eq!(sandbox.execute("globalThis.__imageIds").unwrap().value, "first,second");
    assert_eq!(sandbox.execute("globalThis.__iteratedIds").unwrap().value, "first,second");
    assert_eq!(sandbox.execute("globalThis.__imageAttributeNames").unwrap().value, "id,alt");
    assert_eq!(sandbox.execute("globalThis.__optionIds").unwrap().value, "one,two");
    assert_eq!(sandbox.execute("globalThis.__formControlIds").unwrap().value, "control,choice");
}
