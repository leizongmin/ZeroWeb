// part73 — html-semantics M2 切片 40（R5243，2026-10-23）：invoker custom command
//（`--` 前缀）CommandEvent 派发面——part01 `_zwInvokerCommandActivate` custom 分支
//（spec button activation behavior：command 事件派发后 return，无内建行为）+ part05
// CommandEvent 接口（Event 子类，command/source）。
// https://html.spec.whatwg.org/multipage/form-elements.html#the-button-element:activation-behavior

/// 断言反转面：custom 分支缺 → 事件恒 null；command 值被小写化 → R2 大小写断言红；
/// bubbles/composed/cancelable 误设 → R1 形状断言红；trusted 印记缺 → isTrusted 红；
/// dialog 目标误走内建分支 → R5 open=1 红。
#[test]
fn test_invoker_custom_command_event_r5243() {
    let mut sandbox = t59_sandbox_with_shim();
    let mutations: std::sync::Arc<std::sync::Mutex<Vec<DomMutation>>> = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let dom_html: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new(
        "<html><body></body></html>".to_string(),
    ));
    let page_url: std::sync::Arc<std::sync::Mutex<String>> = std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    crate::js_dom_bridge::register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);

    // R1：基面——button commandfor→div、command='--custom-command'，click 派
    // CommandEvent：type/command/source/target/bubbles/composed/cancelable/isTrusted
    // + 无 relatedTarget（读 undefined）+ instanceof CommandEvent。
    sandbox
        .execute(
            "var tgt = document.createElement('div'); tgt.id = 'tgtDiv'; document.body.appendChild(tgt); var b = document.createElement('button'); b.setAttribute('commandfor', 'tgtDiv'); b.setAttribute('command', '--custom-command'); document.body.appendChild(b); globalThis.__eT = tgt; globalThis.__eB = b; globalThis.__ev = null; tgt.addEventListener('command', function (e) { globalThis.__ev = e; }); b.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[String(globalThis.__ev instanceof globalThis.CommandEvent), globalThis.__ev.type, globalThis.__ev.command, String(globalThis.__ev.source === globalThis.__eB), String(globalThis.__ev.target === globalThis.__eT), String(globalThis.__ev.bubbles), String(globalThis.__ev.composed), String(globalThis.__ev.cancelable), String(globalThis.__ev.isTrusted), String(globalThis.__ev.relatedTarget === undefined)].join(',')")
            .unwrap()
            .value,
        "true,command,--custom-command,true,true,false,true,true,true,true",
        "R1：custom command 派 CommandEvent——全形状断言（含 isTrusted、无 relatedTarget）"
    );

    // R2：custom 值大小写保留 + 边界形态（'--' 恰两划、'---' 三划、尾划）。
    sandbox
        .execute(
            "globalThis.__r2 = []; var cmds = ['--cAsE-cArRiEs', '--', '---', '--foo-']; for (var i = 0; i < cmds.length; i++) { globalThis.__eB.setAttribute('command', cmds[i]); globalThis.__ev = null; globalThis.__eB.click(); globalThis.__r2.push(globalThis.__ev ? globalThis.__ev.command : 'null'); }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r2.join('|')").unwrap().value,
        "--cAsE-cArRiEs|--|---|--foo-",
        "R2：command 属性原值派发（大小写保留、'--'/'---' 边界均 custom）"
    );

    // R3：非 `--` 前缀不派事件（含内建关键字 show-modal 落 DIV 目标——validity
    // is-valid-command false → return，内建不漏 CommandEvent）。
    sandbox
        .execute(
            "globalThis.__r3 = []; var bads = ['foo-bar', '-foo', 'show-modal']; for (var j = 0; j < bads.length; j++) { globalThis.__eB.setAttribute('command', bads[j]); globalThis.__ev = null; globalThis.__eB.click(); globalThis.__r3.push(String(globalThis.__ev === null)); }",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r3.join(',')").unwrap().value,
        "true,true,true",
        "R3：非 custom 关键字不派 CommandEvent"
    );

    // R4：click preventDefault → 激活行为不跑、无 command 事件。
    sandbox
        .execute(
            "globalThis.__eB.setAttribute('command', '--custom-command'); globalThis.__ev = null; var pd = function (e) { e.preventDefault(); }; globalThis.__eB.addEventListener('click', pd, { once: true }); globalThis.__eB.click(); globalThis.__r4 = String(globalThis.__ev === null); globalThis.__eB.click(); globalThis.__r4b = String(globalThis.__ev !== null);",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("globalThis.__r4 + ',' + globalThis.__r4b").unwrap().value,
        "true,true",
        "R4：click preventDefault 抑制 command 事件；未取消 click 照常派"
    );

    // R5：dialog 目标 + custom command——只派事件不 showModal（custom 态 spec 激活
    // 行为派发后 return，内建 DIALOG 分支不适用）。
    sandbox
        .execute(
            "var dg = document.createElement('dialog'); dg.id = 'tgtDlg'; document.body.appendChild(dg); globalThis.__eD = dg; globalThis.__eB.setAttribute('commandfor', 'tgtDlg'); globalThis.__ev = null; dg.addEventListener('command', function (e) { globalThis.__ev = e; }); globalThis.__eB.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox
            .execute("[String(globalThis.__ev !== null), String(globalThis.__ev && globalThis.__ev.target === globalThis.__eD), String(Number(globalThis.__eD.hasAttribute('open')))].join(',')")
            .unwrap()
            .value,
        "true,true,0",
        "R5：dialog 目标 custom command 派事件且不 showModal"
    );

    // R6：command 事件 cancelable=true——handler preventDefault 后 defaultPrevented=true。
    sandbox
        .execute(
            "globalThis.__ev = null; globalThis.__eB.setAttribute('commandfor', 'tgtDiv'); var cdp = function (e) { e.preventDefault(); }; globalThis.__eT.addEventListener('command', cdp, { once: true }); globalThis.__eB.click();",
        )
        .unwrap();
    assert_eq!(
        sandbox.execute("String(globalThis.__ev && globalThis.__ev.defaultPrevented)").unwrap().value,
        "true",
        "R6：command 事件可取消（cancelable=true 面）"
    );
}
