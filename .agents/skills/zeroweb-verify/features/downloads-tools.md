# F04/F05 下载、查找、缩放与上下文操作

## 任务、预期与入口

点击 attachment 链接，下载到本次目录并打开所在文件夹；失败不记完成。
Ctrl+F 查找、Enter 下一项、Esc 关闭；滚动/缩放改变页面显示；右键查看源代码、复制选择。
预期来自桌面 M3 演示流和对应生产输入代码，不以 DownloadManager 状态单测冒充网络落盘。
源码：`apps/browser/src/fetch_proxy.rs`、`download_smoke.rs`、`menu_zoom_smoke.rs`、
`find_smoke.rs`、`page_find.rs`、`app_input_context_menus.rs`。

## 准备与操作

按 [主入口](../SKILL.md) Doctor；当前产品进程、守卫、Linux 窗口系统、Python HTTP
及 `examples/m3-downloads` fixture 已就绪。本次下载目录和 profile 必须隔离；
“打开文件夹”会启动系统应用，先确认允许该副作用。未获剪贴板写入许可时只记录缺口。
M3 会连续跑下载、菜单缩放、查找，选择此命令即包含这三个既有流程，不默认全图：

```sh
RUN="$PWD/.agents/skills/zeroweb-verify/.verify-task/$(date +%Y%m%d-%H%M%S)-tools"
OUT_DIR="$RUN/m3" SKIP_BUILD=1 ./target/test-guard \
  --per-proc-mem 4 --total-mem 8 --time-limit 1200 -- bash scripts/desktop-browser-m3-smoke.sh
```

下载路径：页面点击、attachment 响应、Pending/Downloading/Completed、下载页、Show in folder。
查找路径：输入关键词、计数高亮、Enter 推进并滚动、Esc 清除。另测零匹配和反向查找。
菜单路径：右键、刷新/inspect、选择复制、zoom-in；另测缩放重置和查看源代码内容。
Android 下载通知/SAF 路径见 [移动端](android.md)，不与桌面文件夹打开合并。

共享工作区新增 T8 UI 后，还需单独核对多任务面板、暂停/取消、加载状态提示和
菜单渐入；旧 M3 运行不自动覆盖这些入口。当前 `apps/browser/src/tests.rs` 的
`download_panel_lists_all_active_downloads`、`download_panel_collapses_overflow_rows`、
`download_panel_pause_and_cancel_buttons_act`、`status_bar_shows_loading_domain_while_loading`
及 `context_menu_fades_in_on_open` 提供状态和场景图元判据。
生产窗口应观察前三行与溢出计数、按钮命中后的状态/行数变化、链接提示优先级、
加载完成后提示消失和菜单打开的时间序列。上述测试及窗口动作本轮均未执行。
暂停/取消当前只改 DownloadManager 元数据；若验收传输暂停或取消，必须另用受控
流式响应观察网络与文件字节，不能用按钮状态单测代替该预期。

## 判据、错误与清理

字节必须与 `examples/m3-downloads/file.zip` 经 cmp 一致；DL/MZ/FIND 日志终点、
每步状态和 PNG 齐全；下载完成日志不能代替文件存在/字节。
查找 count/当前项正确，关闭清高亮；缩放值与布局/帧联动；复制读回实际选择文本。
失败 fixture 应返回错误、无 Completed/成功导出、旧页面仍可用；失败恢复后重查 Doctor。
只停本次 server/PID，清本次文件/profile；系统打开的窗口仅在归属明确时关闭，
不要退出用户文件管理器。保留 cmp 结果、日志/截图及动作前后的副作用，清理后重读。

## 来源与证据

[M3 下载](../../../../docs/goal/archive/desktop-browser/evidence/M3-download-manager.md)、
[M3 查找](../../../../docs/goal/archive/desktop-browser/evidence/M3-page-find.md)、
[M3 菜单缩放](../../../../docs/goal/archive/desktop-browser/evidence/M3-context-menu-zoom.md)。
2026-10-14 本轮 F04/F05 全部 `NOT_RUN`，版本环境见 [初始化记录](../initialization.md)。
