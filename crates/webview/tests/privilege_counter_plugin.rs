//! renderer 信任边界 ②C'：counter-plugin 库路径装配回归（`ensure_js_shim` 装配点）。
//!
//! 集成测试二进制不携带 zero-webview 的 cfg(test)——[`WebView`] 的 counter-plugin
//! 分支真实生效（lib 单测经 cfg(test) 跳过，钩子直调用例不受影响）。单测试函数内
//! 串行两阶段，避免 env 进程级信号（`ZW_RENDERER_AUTOMATION`）与并行测试竞态：
//! 阶段 1 非测试模式（删除面 + request() 闭包化非回归）；阶段 2 置位测试模式后新建
//! WebView（`shim_test_mode_enabled` 每次装配时读 env）验钩子保留（runner 契约）。

use zero_webview::{WebView, WebViewConfig};

#[test]
fn counter_plugin_library_wiring_non_test_then_test_mode() {
    const MUTATORS: &str = "['__zwUserActivate','__zwClipboardStoreWrite',\
       '__zwDeleteAllCookies','__zwSetPermission']\
       .map(function(n){ return typeof globalThis[n]; }).join(',')";

    // 阶段 1：无测试模式信号 → mutator 页面不可见。外部环境已置信号时删除面断言
    // 不成立（装配契约即测试模式跳过），跳过本阶段。
    if std::env::var("ZW_RENDERER_AUTOMATION").is_ok() {
        eprintln!("ZW_RENDERER_AUTOMATION 已置位，跳过非测试模式删除面断言");
    } else {
        let mut wv = WebView::new(WebViewConfig::default());
        let deleted = wv.execute_script(MUTATORS).unwrap();
        assert_eq!(
            deleted, "undefined,undefined,undefined,undefined",
            "库路径非测试模式：4 枚特权 mutator 删除面生效"
        );
        // ②C' 闭包化非回归（真实装配序）：request() headless 自动授予经闭包别名工作。
        wv.execute_script(
            "globalThis.__req = 'pending';\
             navigator.permissions.request({ name: 'fullscreen' }).then(function(s){\
               globalThis.__req = s.state;\
             });",
        )
        .unwrap();
        let granted = wv.execute_script("globalThis.__req").unwrap();
        assert_eq!(granted, "granted", "request() 自动授予删除全局后不回归");
    }

    // 阶段 2：置测试模式 → 新建 WebView 装配（ensure 每次读 env）→ 钩子保留。
    // SAFETY: 测试进程内本函数串行执行；env 仅影响本测试二进制后续装配。
    unsafe {
        std::env::set_var("ZW_RENDERER_AUTOMATION", "1");
    }
    let mut wv2 = WebView::new(WebViewConfig::default());
    let kept = wv2
        .execute_script(
            "['__zwUserActivate','__zwClipboardStoreWrite',\
       '__zwDeleteAllCookies','__zwSetPermission']\
       .map(function(n){ return typeof globalThis[n] === 'function'; }).every(Boolean)",
        )
        .unwrap();
    assert_eq!(kept, "true", "测试模式（env 置位）钩子原样保留（runner 契约）");
}
