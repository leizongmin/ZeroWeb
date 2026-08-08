//! 构建脚本：Windows 下把应用图标嵌入 .exe 资源段。
//!
//! 仅 Windows 生效。使用已生成的 zero-browser.ico（由 tools/icon-gen 产出）。
//! 资源 ID = 32512 (IDI_APPLICATION 默认主图标)。
#[cfg(target_os = "windows")]
#[path = "../../build-support/product_version.rs"]
mod product_version;

fn main() {
    #[cfg(target_os = "windows")]
    {
        let version = product_version::resolve().unwrap_or_else(|error| panic!("invalid product version: {error}"));
        let icon = "assets/icons-gen/zero-browser.ico";
        if std::path::Path::new(icon).exists() {
            let mut res = winres::WindowsResource::new();
            res.set_icon_with_id(icon, "IDI_ICON1");
            res.set(
                "FileDescription",
                "ZeroBrowser — A cross-platform browser built with Rust",
            );
            res.set("ProductName", "ZeroBrowser");
            res.set("FileVersion", &version.text);
            res.set("ProductVersion", &version.text);
            res.set_version_info(winres::VersionInfo::FILEVERSION, version.windows_value);
            res.set_version_info(winres::VersionInfo::PRODUCTVERSION, version.windows_value);
            if let Err(e) = res.compile() {
                println!("cargo:warning=winres 编译失败（{e}）；.exe 将不含应用图标资源");
            }
        } else {
            println!("cargo:warning=未找到 {icon}；运行 cargo run -p zero-icon-gen 生成后再构建以嵌入图标");
        }
        println!("cargo:rerun-if-changed={icon}");
        println!("cargo:rerun-if-env-changed=ZERO_BUILD_VERSION");
        println!("cargo:rerun-if-env-changed=SOURCE_DATE_EPOCH");
        println!("cargo:rerun-if-changed=../../build-support/product_version.rs");
        println!("cargo:rerun-if-changed=.zero-build-version-always-rerun");
    }
}
