//! media march 提交门（bg02-media 基线 v2 边沿拍）真值表锁定。
//!
//! 锁定机制面：`media_march_submittable` 的边沿补拍语义——any_playing true→false
//! 切换拍（registry 全部流末停播）必须补交一拍 march（shim 观察 isEnded → ended
//! 派发/loop 回卷），停播稳态不提交（零开销面保持）。shim 侧 march 行为（ended
//! 派发/loop 分叉）由 engine js_dom_bridge_tests 覆盖；产品腿验收见
//! docs/goal/archive/media-runtime-isolation/evidence/2026-10-25-bg02-media-v3/。

use super::media_march_submittable;

#[test]
fn media_march_submits_while_playing() {
    assert!(media_march_submittable(true, true, false), "播放中每拍提交");
    assert!(
        media_march_submittable(true, true, true),
        "持续播放提交（前一拍也在播）"
    );
}

#[test]
fn media_march_edge_tick_after_eof_stop() {
    // true→false 切换拍：补交一拍——ended 观察窗口（本修复的回归面）。
    assert!(
        media_march_submittable(true, false, true),
        "停播切换拍必须补交 march（ended 派发/loop 回卷依赖此拍）"
    );
}

#[test]
fn media_march_idle_steady_state_zero_cost() {
    assert!(
        !media_march_submittable(true, false, false),
        "停播稳态零提交（泵空转面保持）"
    );
}

#[test]
fn media_march_never_submits_with_js_disabled() {
    assert!(
        !media_march_submittable(false, true, true),
        "javascript_enabled 关恒不提交"
    );
}
