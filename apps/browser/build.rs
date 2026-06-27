//! 构建脚本：Windows 下把应用图标嵌入 .exe 资源段。
//!
//! 仅 Windows 生效。使用已生成的 zero-browser.ico（由 tools/icon-gen 产出）。
//! 资源 ID = 32512 (IDI_APPLICATION 默认主图标)。
fn main() {
    #[cfg(target_os = "windows")]
    {
        let icon = "assets/icons-gen/zero-browser.ico";
        if std::path::Path::new(icon).exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon_with_id(icon, "IDI_ICON1");
            res.set(
                "FileDescription",
                "ZeroBrowser — A cross-platform browser built with Rust",
            );
            res.set("ProductName", "ZeroBrowser");
            if let Err(e) = res.compile() {
                println!("cargo:warning=winres 编译失败（{e}）；.exe 将不含应用图标资源");
            }
        } else {
            println!("cargo:warning=未找到 {icon}；运行 cargo run -p zero-icon-gen 生成后再构建以嵌入图标");
        }
        println!("cargo:rerun-if-changed={icon}");
    }
}
