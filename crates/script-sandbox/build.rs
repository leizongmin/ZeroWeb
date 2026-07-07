// zero-script-sandbox 构建脚本。
//
// 仅做一件事：补 rusty_v8 0.32.1 上游缺失的 Windows 系统库链接指令。
// 背景：rusty_v8 0.32.1 的 build.rs::print_link_flags() 在 Windows 上只 emit
//   cargo:rustc-link-lib=dylib=winmm
//   cargo:rustc-link-lib=dylib=dbghelp
// 漏掉 advapi32。但 rusty_v8 release static lib 的 system-jit-win.obj 引用 ETW
// （EventRegister / EventSetInformation / EventUnregister / EventWriteTransfer），
// wintz.obj 引用注册表（RegOpenKeyExW / RegQueryInfoKeyW / RegCloseKey /
// RegEnumKeyExW / RegQueryValueExW）——这些符号全在 advapi32.lib。
// 结果：link.exe 报 LNK2019（9 unresolved externals）+ LNK1120，main CI Windows
// 自 2026-05-30（M1 后）持续红。
// 仅在 v8 feature 启用（rusty_v8 参与链接）且目标为 Windows 时 emit，Linux/macOS
// 构建零影响。CARGO_CFG_TARGET_OS 反映 target（非 host），cross-compile 安全。

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let v8_enabled = std::env::var("CARGO_FEATURE_V8").is_ok();
    if v8_enabled && target_os == "windows" {
        println!("cargo:rustc-link-lib=dylib=advapi32");
    }
    // rerun-if-changed 自身，避免每次都跑
    println!("cargo:rerun-if-changed=build.rs");
}
