// R380（js-dom M4，pending-apply RFC pa3 前置）：part25——pa1 §3.3/§4 探针实证的
// registry 缺口修复（part24 超 2900 行后的新切片段；CLAUDE.md §5 文件大小控制）：
// ① sel 域融合 innerHTML——pending 桶非空时从 `_childNodeList` 融合视图序列化，
//   替代 host 快照旧树（R377 Fail 实际形态：innerHTML 读 apply 滞后旧树）；
// ② innerHTML setter 克隆路径纯文本内容补 text registry 子（R151 只填 markup 形态，
//   纯文本源码落 else 清空分支 → script 克隆的 `_handleChildren[scriptH]` 恒空，
//   R377 插入期脚本钩子源码收集失败 no-op）。
// 注：fragment 展开的「registry 文本子随迁」实验（把 fragment 顶层子数组搬给首个
// 克隆子）已**证伪移除**——fragment registry 只存顶层子，子元素后代经 `_zwMEl
// appendChild` 自记账；顶层数组搬给首子造出自环（`a.children=[a,b]`）→
// `_ceApplyConn`/`_zwHCCollectSubtree` DFS 死循环（part02
// test_fragment_flatten_all_insertion_paths_e2e 复现，100% CPU 挂起）。本测试同时
// 锁定该回归不复发（replaceWith(fragment) 后 connected 传播终止）。

#[test]
fn r387_dynamic_script_append_executes() {
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
        "<html><body><div id='host'></div></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // R387：动态 classic 脚本插入期执行——createElement('script') + textContent= + appendChild
    // 入文档即同步跑（spec prepare-the-script-element；SPA 加载器/分析 SDK 标准路径）。
    // run-once：重复 append 不重跑（`_zwRanScripts` 标记，R377 同源语义）。
    sandbox.execute(
        "try {\
         var s = globalThis.document.createElement('script');\
         s.textContent = \"globalThis.__r387ran = 'yes';\";\
         globalThis.document.getElementById('host').appendChild(s);\
         var first = String(globalThis.__r387ran);\
         globalThis.__r387ran = 'second';\
         globalThis.document.getElementById('host').appendChild(s);\
         globalThis.__r387a = first + ':' + String(globalThis.__r387ran);\
         } catch (err) { globalThis.__r387a = 'ERR:' + err.message; }",
    ).unwrap();
    let out = sandbox.execute("globalThis.__r387a").unwrap().value;
    assert_eq!(
        out, "yes:second",
        "R387：动态 script appendChild 同步执行 + run-once 不重跑"
    );
}

#[test]
fn r380_fused_innerhtml_and_text_registry_children() {
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
        "<html><body><div id='container'><div id='target'></div><b></b></div><template><span>New </span><script>document.querySelector('b').remove();</script><span>content</span></template></body></html>".to_string(),
    ));
    let page_url: Arc<Mutex<String>> = Arc::new(Mutex::new("about:blank".to_string()));
    let canvas_registry: std::sync::Arc<std::sync::Mutex<crate::js_dom_bridge::CanvasRegistry>> =
        std::sync::Arc::new(std::sync::Mutex::new(crate::js_dom_bridge::CanvasRegistry::new()));
    register_dom_callbacks(&mut sandbox, &mutations, &dom_html, &page_url, &canvas_registry, None);
    // ① 克隆路径：innerHTML setter 纯文本分支补 text 子（R380 ②）→ 克隆 script 有
    //    registry 源码（R377 钩子/序列化的读取面）。
    // ② replaceWith fragment 展开后 target parentNode 同步 null（R379 M6 标记语义）。
    // ③ sel 域融合 innerHTML：同 turn 内 script.remove()（WPT 用例序——remove 先于
    //    host apply）→ container.innerHTML 从融合 childNodes 序列化（含两 span、
    //    不含旧 target / script / b）。与 WPT 用例断言同构。
    //    （JS 串内不用 `//` 行注释——Rust `\<newline>` 行继续使整串成单行，注释会吞代码。）
    sandbox
        .execute(
            "try {\
             var log = [];\
             var target = globalThis.document.getElementById('target');\
             var tpl = globalThis.document.querySelector('template');\
             var frag = tpl.content.cloneNode(true);\
             var sc = frag.querySelector('script');\
             log.push('clone:' + (sc && sc.textContent && sc.textContent.indexOf('querySelector') >= 0 ? 'src' : 'empty'));\
             target.replaceWith(frag);\
             log.push('pw:' + (target.parentNode === null ? 'null' : 'non-null'));\
             var container = globalThis.document.getElementById('container');\
             container.querySelector('script').remove();\
             var ih = container.innerHTML;\
             log.push('fused:' + (ih === '<span>New </span><span>content</span>' ? 'exact' : ('no:' + ih.slice(0, 80))));\
             globalThis.__r380a = log.join('|');\
             } catch (err) { globalThis.__r380a = 'ERR:' + err.message; }",
        )
        .unwrap();
    let out = sandbox.execute("globalThis.__r380a").unwrap().value;
    assert_eq!(
        out, "clone:src|pw:null|fused:exact",
        "R380：克隆 script 经纯文本 registry 分支有源码 + replaceWith 同步标记 + sel 域 innerHTML 融合序列化与 WPT 期望串全等"
    );
}
