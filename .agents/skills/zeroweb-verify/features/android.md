# F12 Android 移动浏览

## 任务与契约

移动用户导航、多标签/缩略图、触摸/IME、旋转/返回/后台恢复、收藏历史、下载通知、
SAF 导出/取消、外部 Intent，服务断连后恢复。
完整 FR-001~009 原操作及预期复用
[设备验收清单](../../../../docs/goal/android-browser/evidence/device-acceptance-checklist.md)，
不复制已有清单，不把安装/四进程启动算作全部功能通过。
源码：`apps/android-browser/app/src/main/java/com/leizm/zeroweb/MainActivity.kt`、
`apps/android-browser/rust`；准备：
[Rust 桥构建说明](../../../../apps/android-browser/rust/README.md)。

## Prepare / Doctor / Drive

本轮没有连接/使用设备。后续先按 [主入口](../SKILL.md) 确认范围、设备归属与权限，
使用受既有内存/墙钟守卫的 Android preflight/构建命令；
`adb devices -l` 只读核对，指定本次设备，确认匹配 ABI 的 renderer 版 APK、
存储/通知权限与本地 HTTP fixture。用户既有应用数据不能 reset/clear。
设备不足或只能装 chrome 版时页面路径 BLOCKED。

按原清单在本次测试应用/隔离设备上：
地址栏前往、滑动、后退/前进，新建并切换多标签、输入法键入；
旋转/退出重开验证恢复，再收藏/历史操作。下载 attachment 后检查真实文件/通知，
SAF 选择本次目标导出；取消另测无文件且不报成功。外部 HTTPS Intent 打开新标签，
file/未知 scheme 拒绝。
仅在授权的隔离设备执行既有
[chaos 脚本](../../../../scripts/android/chaos-smoke.sh) 的服务中断与重绑检查。

## 判据、清理与证据

设备截图与活状态/日志一致；后台标签显示各自页面；恢复无重复标签或下载/表单重放。
导出逐字节核对源文件；取消没有副作用；断连后服务/PID/UID/页面帧恢复按清单判定。
保存设备/OS/API/ABI、APK 身份、logcat、截图、文件验证和清理记录到本次 `.verify-task/`。
失败重新检查设备/服务/输入与权限，保留错误，不自动清用户应用数据。
只清本次文件/隔离应用与本次服务，关闭本次授权应用；保留证据并实际重读。
2026-10-14 F12 全部 `NOT_RUN`，本轮 macOS API 证据不能证明 Android 功能。
[版本环境](../initialization.md)、[地图](index.md)。
