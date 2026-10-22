//! FR-4/DC-3 验收探针：真实进程内安装生产 panic 钩子后注入 panic。
//!
//! 用法：`cargo run -p zero-browser-shell --example crash_probe -- <报告目录>`
//! 进程以 panic 退出（101）；验收方检查 `<报告目录>/crash-<unix秒>-browser.json`
//! 的白名单字段与隐私断言。报告目录取 argv[1]，缺省用进程默认配置目录。

fn main() {
    let dir = std::env::args()
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(zero_browser_shell::crash_reports_dir);
    eprintln!("crash_probe: writing panic report under {}", dir.display());
    zero_browser_shell::install_browser_panic_hook(dir, "26.10.22", "probe");
    panic!("intentional crash for FR-4 acceptance");
}
