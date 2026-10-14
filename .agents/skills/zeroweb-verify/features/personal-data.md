# F03 收藏、历史与设置

## 用户任务与预期

收藏/取消收藏、管理文件夹和书签栏，查询/清空历史，修改搜索引擎、隐私与主页，
退出重开后恢复。来源为 shell 数据模型和桌面 M4，不把只更新内存当持久化通过。
源码：`crates/browser-shell/src/{bookmarks,history,settings,profile}.rs`，
`apps/browser/src/pages.rs`、`data_smoke.rs`。

## 入口、准备与动作

GUI：工具栏、书签栏、`zero://history`、`zero://settings` 和 Alt+Home。
先 [Doctor](../SKILL.md)，当前源码四产品进程、守卫、Linux display、本地 fixture 已就绪。
M4 设置 `XDG_CONFIG_HOME`；执行前核对当前平台的 dirs 行为确实落在本次 profile。
不能读取用户真实收藏/历史后做 clear，也不能在不隔离的平台照搬此设置。
项目根执行现有 Linux 演示流：

```sh
RUN="$PWD/.agents/skills/zeroweb-verify/.verify-task/$(date +%Y%m%d-%H%M%S)-data"
OUT_DIR="$RUN/m4" SKIP_BUILD=1 ./target/test-guard \
  --per-proc-mem 4 --total-mem 8 --time-limit 600 -- bash scripts/desktop-browser-m4-smoke.sh
```

流程开书签栏、添加、点击打开、右键删除，展示历史再清空；切搜索引擎/隐私，
设置本地主页并 Alt+Home。持久化还应另跑隔离 profile 重启，读 JSON/页面一致性。
低成本书签底座路径为 `crates/browser-shell/src/tests/bookmarks.rs::test_bookmarks_save_load_roundtrip`：
调用真实 save/load；它不证明窗口按钮、历史清空或设置接线。

## 判据与清理

DATA_SMOKE 每步成功、书签目标增删可见、历史清空、主页 URL 落地、设置与磁盘一致。
文件夹删除只删除本次目标；损坏 profile 的 fail-safe 与写失败错误传播另测，
不能降低预期或把 silent fallback 当恢复成功。
留存 JSON 副本、状态日志、PNG；脚本 HTTP trap 后核对进程/端口，删除本次 profile，
保留证据重读。异常仅重建本次隔离实例并 Doctor。

## 来源与证据

[M4 数据面](../../../../docs/goal/desktop-browser/evidence/M4-data-plane.md)、
[profile 实现](../../../../crates/browser-shell/src/profile.rs)、
[地图](index.md)。版本环境见 [初始化记录](../initialization.md)；
2026-10-14 本轮 F03 所有入口 `NOT_RUN`。
