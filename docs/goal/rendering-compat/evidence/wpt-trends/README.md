# wpt-trends 账目口径

本目录是 reftest-upstream 全量趋势的正式账目：`trend.csv` 一行一次全量跑批，
同名 `YYYY-MM-DD-<mode>.json` 为该行明细。由 `make reftest-trend`
（`scripts/record-wpt-trend.sh`）追加，不手改历史行。

## git_sha 字段口径

`git_sha` 记录**趋势记录命令执行时的 HEAD**（`git rev-parse --short HEAD`，
`record-wpt-trend.sh` 内联）。跑批本身在本机工作树执行——若当时工作树含
未提交改动，账目数字对应「HEAD + 工作树」状态，与本行 sha 所指树的产物
可能不同。

复核一条账目对应的确切代码状态时，以运行留档（日志、证据文件）为准，
不以本行 sha 单独推断。实例：2026-10-06 行 sha=`c8f1a90cd`（记录时 HEAD），
跑批工作树已含 R4946 修复（未提交）；合并后在 PR head `a307414ce` 复跑
全量，16060/14758P/1302F 与该行逐位一致。
