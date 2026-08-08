# macOS 子进程使用嵌套 Helper app

- 日期：2026-08-08
- 相关模块：`scripts/package-macos.sh`、`apps/browser/src/process_backend.rs`

## 问题描述

把 `zero-renderer` 作为普通文件放在 `ZeroBrowser.app/Contents/MacOS/` 时，Activity Monitor 只能显示二进制文件名。改成嵌套 Helper app 后名称正确，但直接 spawn 的 renderer 仍没有图标。

## 根因分析

macOS 的应用显示名和图标由 `.app` bundle 的 `Info.plist` 与 `Resources/*.icns` 提供，而不是从普通 Mach-O 可执行文件读取。嵌套 Helper bundle 只提供元数据和资源；通过 `Command::spawn` 直接执行 bundle 内 Mach-O 时，进程不会自动注册为 LaunchServices/AppKit running application。此时 `lsappinfo` 的 bundle id、display name 和 ASN 均为空，Activity Monitor 不显示图标。winit 的窗口图标接口在 macOS 上也不能替代应用 bundle 图标。

## 解决方案

将 renderer 包装为 `ZeroBrowser.app/Contents/Frameworks/ZeroBrowser Helper (Renderer).app`，在 Helper 的 `Info.plist` 中设置独立的 `CFBundleName`、`CFBundleDisplayName`、`CFBundleIdentifier` 和 `CFBundleIconFile`，并复用主应用的 `.icns`。browser 优先查找 Helper 内的可执行文件，同时保留同目录 `zero-renderer` 作为本地开发和旧布局兜底。

Helper 启动后必须在主线程运行 AppKit event loop，并使用 `NSApplicationActivationPolicyAccessory` 与 `LSUIElement=true` 注册为 UIElement application。原 renderer runtime 移到工作线程，stdin/stdout pipe IPC 保持不变。注册完成后，`lsappinfo` 能返回 Helper 的 bundle id、display name 和 ASN，Activity Monitor 显示图标，同时 Dock 不出现额外图标。

renderer 的 stdout 是二进制 IPC 专用通道。诊断命令或子进程必须将 stdout 重定向到 `/dev/null` 或其他独立通道；任何文本输出都会被 browser 当作协议帧解析，随后触发 IPC 断开并关闭 renderer。

签名必须由内向外执行：Helper 可执行文件、Helper bundle、主应用 bundle。否则修改嵌套内容会使外层代码签名失效。

裸 Mach-O 没有受支持的应用图标载体。需要完整的 ZeroBrowser 名称和图标时必须运行 `.app`，不应使用私有 API 或仅修改进程参数伪装。

参考：

- [Apple Bundle Programming Guide](https://developer.apple.com/library/archive/documentation/CoreFoundation/Conceptual/CFBundles/BundleTypes/BundleTypes.html)
- [winit: Window Icon can neither be set nor changed](https://github.com/rust-windowing/winit/issues/3398)
