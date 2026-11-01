//! `__zw*` 特权 mutator counter-plugin（renderer 信任边界 goal ②C'，2026-11-01）。
//!
//! shim 的 4 枚特权 mutator 全局（`__zwUserActivate` / `__zwClipboardStoreWrite` /
//! `__zwDeleteAllCookies` / `__zwSetPermission`）任何页面无条件可见，构成标准 API 门控
//! 旁路面：瞬态激活注入（fullscreen / pointer lock 激活门）、剪贴板存储直写、in-JS
//! cookie jar 清空、权限注册表直写（fullscreen 权限路自授）。本模块提供删除面与测试
//! 模式判定，shim 三文件零行为变更（定义保留，内部消费闭包化在 part02/part06 单点）。
//!
//! - [`counter_plugin_script`]：宿主在各页面脚本上下文装载 [`generate_js_dom_shim`]
//!   之后、页面脚本执行之前执行（幂等，每次 shim 装载后跟随执行——context 重建会重新
//!   定义全局，须再次删除）。
//! - [`shim_test_mode_enabled`]：测试模式判定（②A-env：`ZW_RENDERER_AUTOMATION=1`）。
//!   置位时宿主跳过 counter-plugin，runner testdriver stub 依赖的钩子原样保留。
//!
//! 装配契约（4 个 shim 装载点）：`apps/renderer` js_worker（bootstrap + context 重建）、
//! `apps/browser` tab_js_worker（单进程）、`zero-webview` webview `ensure_js_shim`（库
//! 嵌入；该 crate 单测直调钩子，cfg(test) 跳过）。信号经进程 env 派生：runner 启动自置、
//! browser 子进程经 `Command` 默认继承传递，页面脚本无从翻转（wire 不变）。
//! `__zwModalGateBypass` 为接受残余（shim 自管旗标，删除会被 shim 复写；仅自影响本页
//! 焦点语义，goal 决策包 ② 记档）。

/// counter-plugin 脚本：删除页面可见特权 mutator 全局（见模块文档与脚本头注释）。
pub fn counter_plugin_script() -> &'static str {
    include_str!("js_dom_counter_plugin.js")
}

/// 测试模式判定：`ZW_RENDERER_AUTOMATION=1`（runner/自动化宿主启动时注入；`Command`
/// 默认继承父 env，browser 派生 renderer 自动携带）。
pub fn shim_test_mode_enabled() -> bool {
    zero_runtime_config::enabled_when_true("ZW_RENDERER_AUTOMATION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_plugin_targets_exactly_the_four_mutators() {
        let script = counter_plugin_script();
        // 删除面锚定 `_zwMutators` 数组的带引号字面量恰为 4 枚——reader 钩子
        // （__zwPermissionState/__zwNotifyPermissionChange）与接受残余
        // __zwModalGateBypass 不进数组（头注释提及不受 contains 干扰）。
        for name in [
            "__zwUserActivate",
            "__zwClipboardStoreWrite",
            "__zwDeleteAllCookies",
            "__zwSetPermission",
        ] {
            assert!(script.contains(&format!("'{name}'")), "counter-plugin 须覆盖 {name}");
        }
        assert_eq!(
            script.matches("'__zw").count(),
            4,
            "删除面恰为 4 枚特权 mutator（无扩面）"
        );
    }
}
