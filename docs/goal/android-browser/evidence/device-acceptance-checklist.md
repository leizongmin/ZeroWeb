# Android 真机/模拟器验收清单（M3 收口 + M4/M5 设备面）

**日期**: 2026-09-12（清单建立；执行记录待设备/模拟器解锁后追加）
**前置**: 构建/安装前提见 [emulator-feasibility.md](emulator-feasibility.md)；
构建入口 `make android-renderer-apk`（页面功能）或 `make android-apk`（仅 chrome 冒烟）。
**用法**: 每项执行后在本文件追加记录（设备/日期/结果/证据截取），失败项连同 adb logcat
片段记入，不充数。

## 0. 安装冒烟（自动化）

```bash
make android-install-smoke          # chrome 级：四进程拓扑/UID 隔离/decoder+compositor probe
# 或 renderer 版（KVM 模拟器需 x86_64 renderer .so；真机用 arm64 产物）：
adb install -r apps/android-browser/app/build/outputs/apk/arm64/release/app-arm64-release.apk
./scripts/android/install-smoke.sh <apk> --require-renderer
```

期望：`install-smoke PASS`——browser/renderer×1/compositor/image-decoder 四进程在列，
renderer/decoder 走 isolated UID ≠ browser UID，compositor 同 UID，probe 日志齐。

## 1. FR-001 启动与会话恢复

| 步骤 | 期望 |
|------|------|
| 冷启动 app | 三服务就绪提示；chrome（标签/书签/历史）为上次退出状态 |
| 杀进程（`adb shell am force-stop com.leizm.zeroweb`）后重启 | 活动标签 URL 恢复，无重复标签 |

## 2. FR-002 导航与页面交互

| 步骤 | 期望 |
|------|------|
| 地址栏输入 https URL 并「前往」 | 页面帧渲染；标题进标签行 |
| 预览区上下滑动 | 页面滚动（滚动条/内容位移） |
| 点「后退/前进」 | 历史导航生效 |

## 3. FR-003 多标签与缩略图

| 步骤 | 期望 |
|------|------|
| 新建标签并各自导航 | 切换标签各自渲染正确页面 |
| ≥2 个后台标签 | 标签行显示 56×36dp 缩略图（最后渲染态） |
| 建 >8 个标签 | 最久未用标签被逐出（切回自动重导航，无崩溃） |

## 4. FR-004/005 书签与历史

收藏/取消收藏、书签页打开/删除、历史页打开/清空；重启后仍持久。

## 5. FR-006 下载全链路

| 步骤 | 期望 |
|------|------|
| 导航到 attachment 链接（如某 PDF 直链） | 停留原页不渲染二进制；下载页出现 Completed 条目 |
| 下载完成 | 系统通知「下载完成」+ 文件名（API 33+ 需先授予通知权限） |
| 下载页「导出」 | 系统文件选择器；选择位置后写入；取消选择仅无导出，不报假成功 |
| 下载页「打开」 | 外部查看器打开（如 PDF 查看器）；只读授予 |
| 杀进程重启 | 下载记录仍在（downloads.json），**不自动重放下载** |

## 6. FR-007 生命周期与返回

| 步骤 | 期望 |
|------|------|
| 返回键 | 先页面后退；无可后退时退出 |
| 旋转屏幕 | Activity 重建后活动标签/URL/滚动保持，页面重新 attach |
| 退后台等待系统回收后返回 | 会话恢复，无表单/下载重放 |

## 7. FR-008 故障恢复

| 步骤 | 期望 |
|------|------|
| `adb shell ps -A \| grep RendererService` 取 pid 后 `kill <pid>` | 该标签导航报恢复提示并自动重绑；重试后恢复渲染 |
| `kill <compositor pid>` | 帧暂时停更，服务自动重启后帧恢复 |

## 8. FR-009 外部 Intent

```bash
adb shell am start -a android.intent.action.VIEW -d "https://example.com/" com.leizm.zeroweb
adb shell am start -a android.intent.action.VIEW -d "file:///sdcard/x" com.leizm.zeroweb
```

期望：HTTPS 新标签打开；file/未知 scheme 拒绝并显示错误。

## 9. 记录

| # | 场景 | 设备 | 日期 | 结果 | 证据/备注 |
|---|------|------|------|------|-----------|
| — | （待设备解锁后追加） | | | | |
