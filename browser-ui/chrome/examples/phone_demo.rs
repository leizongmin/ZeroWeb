//! phone_demo（DC-15）—— 移动端 phone shell + 多指手势 + 平台 back 可运行制品。
//!
//! `cargo run -p zero-browser-chrome --example phone_demo`
//!
//! 用 `WindowMetrics::phone()` preset 构造手机视口，经 adaptive 选 PhoneBrowserShell，
//! 跑 retained 闭环；脚本化驱动触摸 Tap/Pinch（GestureArena，多指用 pointer_id）+ 平台 back
//! （BackNavigationService→Navigator.pop），打印汇总。证明 DC-15 移动 SDK 在 headless 可运行。

use zero_browser_chrome::phone_demo::{PhoneDemoSummary, run_phone_demo};

fn main() {
    let s: PhoneDemoSummary = run_phone_demo();

    println!("\n=== phone_demo (DC-15) — WindowMetrics::phone() ===");
    println!("adaptive shell: {:?}", s.shell_kind);
    println!(
        "Scene: {} entries（chrome + viewport 填充，safe_area 避让）",
        s.scene_entries
    );
    println!(
        "SDK chrome 布局：内容区 viewport 顶部 y={:.1}px（避让顶部 safe_area），底部导航栏高 {:.1}px",
        s.viewport_top_y, s.bottom_chrome_height
    );
    println!("识别手势: {:?}", s.gesture_kinds);
    println!(
        "平台 back #1（注册 overlay handler）→ {:?}（消耗，不退栈）",
        s.back_with_handler
    );
    println!(
        "平台 back #2（无 handler）→ {:?} → Navigator.pop，剩余 nav depth={}",
        s.back_default, s.nav_depth_after_back
    );
    println!("\n（DC-15 移动 SDK：phone shell 可用 + Tap/Pinch 经 host arena + back 仲裁；真实移动后端首帧待设备）");
}
