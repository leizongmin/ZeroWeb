// slice48（renderer 桥直接测试，slice45 测效 I4 留档收口）：
// 三桥生产链读码闭合（slice45 申报：engine callbacks.rs `__zw_contains` / renderer
// js_worker.rs `__zw_handle_for_selector` / `__zw_selector_for_handle`），本轮补**直接行为
// 测试**——真实 `RendererJsWorker`（js_worker_main 全量真实注册，无 GPU/window）上逐桥
// 验证请求→响应契约：正常路径 + 错误/关闭路径。
//   - `__zw_handle_for_selector`：selector→handle 反查（按值 O(n) 扫，js_worker.rs:1012 注册）
//   - `__zw_selector_for_handle`：handle→selector 正置 O(1) get（js_worker.rs:1034 注册）
//   - `__zw_contains`：容器包含判定（engine callbacks.rs:901，R51c 消零谓词核验面）
// handle_selector_map 经生产同款 merge 形态填充（page_scripts.rs:1087 `map.extend` 镜像——
// 生产该表由 apply_recorded_mutations 的 handle_selectors merge 维护）。

use crate::js_worker::RendererJsWorker;

/// 钉①：双向 identity 桥请求→响应契约。正常路径（正反两向命中）+ 错误路径（未知
/// selector/handle miss 返空串）。缺注册时 renderer 宿主派发对动态创建元素恒 miss
///（js_worker.rs 注册处注释申报面）——本钉把「注册存在且语义正确」固化为常驻断言。
#[test]
fn identity_bridges_request_response_contract_s48() {
    let mut worker = RendererJsWorker::spawn(481);
    // 生产同款 merge 形态（page_scripts.rs:1087 extend 镜像）：handle→selector 正置表。
    {
        let map_arc = worker.handle_selector_map();
        let mut map = map_arc.lock().unwrap_or_else(|e| e.into_inner());
        map.extend([
            ("__zwH_s48_a".to_string(), "#dyn48".to_string()),
            ("__zwH_s48_b".to_string(), "#other48".to_string()),
        ]);
    }
    // 正常路径：反查按值命中各自 handle（两表项互不串扰）。
    assert_eq!(
        worker
            .execute_script_direct("__zw_handle_for_selector('#dyn48')")
            .unwrap(),
        "__zwH_s48_a",
        "反查桥：selector→handle 按值命中"
    );
    assert_eq!(
        worker
            .execute_script_direct("__zw_handle_for_selector('#other48')")
            .unwrap(),
        "__zwH_s48_b",
        "反查桥：第二表项命中（非恒返首项）"
    );
    assert_eq!(
        worker
            .execute_script_direct("__zw_selector_for_handle('__zwH_s48_b')")
            .unwrap(),
        "#other48",
        "正置桥：handle→selector O(1) get"
    );
    // 错误路径：未知 selector/handle miss 返空串（注册处语义：unwrap_or_default）。
    assert_eq!(
        worker
            .execute_script_direct("__zw_handle_for_selector('#absent48')")
            .unwrap(),
        "",
        "反查桥：未知 selector miss 返空串"
    );
    assert_eq!(
        worker
            .execute_script_direct("__zw_selector_for_handle('__zwH_absent')")
            .unwrap(),
        "",
        "正置桥：未知 handle miss 返空串"
    );
    worker.shutdown();
}

/// 钉②：`__zw_contains` 请求→响应契约（engine callbacks.rs:901 读 dom_html 快照）。
/// 正常路径（真包含/逆序不包含）+ 错误路径（未知选择器 miss / 第二参缺省按空串判定）。
#[test]
fn contains_bridge_request_response_contract_s48() {
    let mut worker = RendererJsWorker::spawn(482);
    worker.set_dom_snapshot(
        "<html><body><div id='wrap'><p id='inner'>t</p></div></body></html>",
        "about:blank",
    );
    assert_eq!(
        worker
            .execute_script_direct("__zw_contains('body', 'p#inner')")
            .unwrap(),
        "1",
        "contains：祖先容器命中"
    );
    assert_eq!(
        worker
            .execute_script_direct("__zw_contains('div#wrap', 'p#inner')")
            .unwrap(),
        "1",
        "contains：中间层容器命中"
    );
    assert_eq!(
        worker
            .execute_script_direct("__zw_contains('p#inner', 'div#wrap')")
            .unwrap(),
        "0",
        "contains：逆序（子含父）不命中"
    );
    // 错误路径：未知选择器 miss 返 "0"。
    assert_eq!(
        worker
            .execute_script_direct("__zw_contains('#absent48', 'p#inner')")
            .unwrap(),
        "0",
        "contains：未知容器 miss"
    );
    // 错误路径：第二参缺省（args.len()<2 → 空串选择器）判定不命中。
    assert_eq!(
        worker.execute_script_direct("__zw_contains('body')").unwrap(),
        "0",
        "contains：缺省第二参按 miss 判定"
    );
    worker.shutdown();
}

/// 钉③：桥请求通道关闭路径——worker shutdown 后 execute 通道断（send 失败），三桥
/// 的请求面同源此通道（Execute 命令），关闭后请求不可达即 Err。
#[test]
fn bridge_channel_closed_after_shutdown_s48() {
    let mut worker = RendererJsWorker::spawn(483);
    worker.shutdown();
    assert!(
        worker
            .execute_script_direct("__zw_handle_for_selector('#dyn48')")
            .is_err(),
        "shutdown 后桥请求通道关闭（execute Err）"
    );
}
