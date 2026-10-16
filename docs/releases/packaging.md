# 打包与发布

## 本地打包

需要把 `zero-browser` 打成各平台安装包时，可以用仓库里的打包脚本（产物输出到 `target/packages/`）：

```bash
./scripts/package-linux.sh                                              # Linux：.AppImage / .deb（--appimage|--deb|--all）
./scripts/package-macos.sh                                              # macOS：ZeroBrowser.app + .zip（需在 macOS 上运行）
powershell -ExecutionPolicy Bypass -File scripts/package-windows.ps1    # Windows：.zip（-Installer 生成 NSIS 安装器）
```

## macOS 签名与公证

macOS 下载产物要免除 Gatekeeper 手工放行，必须使用 Apple Developer ID 签名并完成公证。release/weekly workflow 支持仓库 Secrets：`MACOS_CERTIFICATE`（base64 编码的 `.p12`）、`MACOS_CERTIFICATE_PASSWORD`、`MACOS_KEYCHAIN_PASSWORD`、`APPLE_ID`、`APPLE_TEAM_ID`、`APPLE_APP_PASSWORD`。未配置时仍会生成 ad-hoc 签名的 `.app` zip，但首次运行仍可能被 macOS 拦截。

## 自动发布

- 发布 GitHub Release（published）或手动 `workflow_dispatch` 触发时，[release.yml](../../.github/workflows/release.yml) 会在 Linux、macOS、Windows 上自动构建并附带产物。
- [weekly.yml](../../.github/workflows/weekly.yml) 每周有新提交时生成 `weekly-YYYY-MM-DD` 预发布。

项目仍在实验阶段，这些产物仅供本地测试与体验，不代表正式发布。
