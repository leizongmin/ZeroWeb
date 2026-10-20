# 规划资产检查

**日期**：2026-10-20。范围仅为 Ladybird 后续 goal 文档、目录与 Shell 启动入口。

| 检查 | 结果 | 覆盖 |
|---|---|---|
| JSON 与资产存在性 | PASS | 7 个 catalog 条目、契约、master、入口；源码/复用路径；14 个 evidence/archive 占位文件 |
| 本地 Markdown 引用 | PASS | 初次扫描 91 条本地引用；包含目标、控制面、脚本与现有规则/调研 |
| Shell 语法 | PASS | 7 个脚本分别用 `sh -n` 与 `bash -n`，共 14 次 |
| 参数与路径路由 | PASS | 临时 fake Rally 接收 argv；7 个入口各覆盖透传参数/异目录、非零退出码、带空格仓库路径，共 21 项 |
| 文档状态一致性 | PASS | M1–M4 均 NOT_STARTED，DC-1–5 均 NOT_RUN，碰撞占用 NOT_CHECKED |
| Python 入口替换 | PASS | 移除本轮草拟的 `scripts/ladybird-goal.py`；正式入口只用 sh |

模拟检查没有调用真实 Rally；临时 fake 程序与目录随检查清理。每条子命令限时 5 秒；未构建浏览器或测试工具，未运行产品测试、采样、安装、故障注入或开发轮。

最终本地链接/空白检查在交付前覆盖本检查文档和总索引。此报告只能证明规划资产可用，不能勾选任何产品 Done Criteria；运行数据仍须后续真实采集。
