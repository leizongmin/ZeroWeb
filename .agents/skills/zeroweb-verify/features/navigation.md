# F01/F02 网站导航、标签与会话

## 任务与入口

地址栏打开网站，后退/前进/刷新/主页；建立多标签并切换、拖拽、关闭，退出后恢复。
预期来自桌面 M1/M2 演示流及 shell session 契约。
源码：`apps/browser/src/app_input_keys.rs`、`addressbar_smoke.rs`、`tab_smoke.rs`、
`tab_manager.rs`；会话：`crates/browser-shell/src/session.rs`。

## 准备与操作

按 [主入口](../SKILL.md) Doctor 与 [既有门禁](../../../../docs/engineering/quality-gates.md)
准备当前源码的四个产品进程及 `target/test-guard`。
下面的现有 shell 脚本按 Linux X11/Wayland/Xvfb 驱动；macOS 不要因没有 DISPLAY 强行
套 Xvfb，改按原生 GUI/parity 入口准备。Windows 用已有平台说明，不宣称 shell 脚本适用。
使用新 OUT_DIR，先核对用户 profile 隔离。不要把 settings 恢复默认当恢复用户状态。

项目根、Linux、有预构建产物时：

```sh
RUN="$PWD/.agents/skills/zeroweb-verify/.verify-task/$(date +%Y%m%d-%H%M%S)-navigation"
OUT_DIR="$RUN/m2" SKIP_BUILD=1 ./target/test-guard \
  --per-proc-mem 4 --total-mem 8 --time-limit 900 -- bash scripts/desktop-browser-m2-smoke.sh
```

脚本自建 localhost HTTP fixture，执行 Ctrl+T、第二页导航、Ctrl+Tab、Alt+Left/Right、
F5、拖拽、Ctrl+W，再执行键入 URL、建议选择和慢页面加载指示。M1 的
`scripts/desktop-browser-m1-smoke.sh` 另外验证加载、滚动、缩放、刷新三条输出路径。
导航失败另用本次不可达本地 URL，检查错误提示，再导航成功恢复。

## 判据、恢复与清理

核对 TAB_SMOKE/ADDR_SMOKE 每步状态、URL/地址栏/活动页、epoch、标签顺序和内容帧；
不能只看启动成功。保存各步骤 PNG 和日志，零 compositor 断连/fallback。
会话恢复需另行在隔离 profile 保存多个 URL 后退出重开，验证无重复标签、无表单重放；
M2 不自动证明会话恢复。
失败后只重建本次实例并 Doctor，不清空用户 session。脚本 trap 清理本次 HTTP/PID，
仍须核对端口关闭、进程退出；记录本次 profile 归属后清理，保留证据并实际重读。

## 导航与状态

[M2 既有验收](../../../../docs/goal/desktop-browser/evidence/M2-tabs-navigation.md)、
[会话实现](../../../../crates/browser-shell/src/session.rs)、
[地图](index.md)。2026-10-14 初始化基准环境见 [记录](../initialization.md)；
F01/F02 全部 `NOT_RUN`，这里提供复用路径，不宣称本轮 GUI/会话通过。
