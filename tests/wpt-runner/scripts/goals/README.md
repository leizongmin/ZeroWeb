# Goal 推进脚本（按序号执行）

每个 goal 一个带序号脚本：**fetch 上游 WPT 语料子集 + 本地盘点 + M1 下一步检查单**。
序号即建议推进顺序（快赢 → 主攻 → 深结构/门控收尾），留有间隙便于插入。

## 用法

```bash
bash tests/wpt-runner/scripts/goals/10-timing-animation-compat.sh
# FORCE=1 强制重列目录重拉（默认幂等：已拉过的目录自动跳过，防 GitHub 未认证 API 60 req/h 限流）
```

脚本只做 M1 的**资产预置**（语料落入 gitignored 的 `tests/wpt-runner/wpt-data/`）。
基线运行需要 runner 通道（`zero-wpt-runner` 子命令 + per-dir skip 规则 + Makefile
target）——那是各 goal M1 rally 轮的落地内容，检查单见脚本输出尾部，契约见各
goal 入口文档 `docs/goal/<goal>.md` DC-1。

## 推进顺序

| 序号 | Goal | WPT 测试集 | 门控/挂账 |
|------|------|-----------|----------|
| 10 | [timing-animation-compat](../../../../docs/goal/timing-animation-compat.md) | hr-time; performance-timeline; user-timing; web-animations | **已收口**（2026-10-04） |
| 15 | [webstorage-compat](../../../../docs/goal/webstorage-compat.md) | webstorage | 无（快赢；2026-10-08 立项） |
| 18 | [cookies-compat](../../../../docs/goal/cookies-compat.md) | cookies | 无（快赢；HTTP 头驱动用例可执行性 M1 甄别） |
| 20 | [net-api-compat](../../../../docs/goal/net-api-compat.md) | fetch; xhr; url; mimesniff; streams; eventsource | **已收口**（2026-10-04）；WebSocket 二期挂账 |
| 21 | [console-compat](../../../../docs/goal/console-compat.md) | console | 无（极小快赢；console 捕获通道 M1 甄别，触面先与 devtools 流协调） |
| 30 | [encoding-compat](../../../../docs/goal/encoding-compat.md) | encoding | **已收口**（2026-10-04） |
| 40 | [html-syntax-compat](../../../../docs/goal/html-syntax-compat.md) | html/syntax; html/dom | **已收口**（2026-10-04） |
| 50 | [uievents-compat](../../../../docs/goal/uievents-compat.md) | uievents; pointerevents | **已收口**（2026-10-07）；touch/pointerlock/IME 挂账 |
| 60 | [navigation-compat](../../../../docs/goal/navigation-compat.md) | history; navigation-api; html/browsers | M2 轻面推进中；**M3 iframe 切片用户门控** |
| 70 | [workers-compat](../../../../docs/goal/workers-compat.md) | workers; dedicated-workers | 推进中；zero-page-runtime 契约先行 |
| 80 | [svg-compat](../../../../docs/goal/svg-compat.md) | svg | 推进中；渲染面归 rendering-compat |
| 90 | [security-hardening](../../../../docs/goal/security-hardening.md) | content-security-policy; mixed-content; secure-contexts | **已收口**（2026-09-27） |
| 92 | [web-api-batch2](../../../../docs/goal/web-api-batch2.md) | clipboard-apis; fullscreen | **已收口**（2026-10-04） |
| 99 | [webgl-compat](../../../../docs/goal/webgl-compat.md) | webgl | **远期门控**（M2+ 全门控，M1 盘点为唯一自主切片） |

**预留编号（2026-10-08 规划，未立项）**：16 fileapi-compat、17 webmessaging-compat、
19 performance-api-compat（resource/navigation-timing/event-timing 扩面）、
22 touch-events-compat、23 selection-compat、24 compression-compat、
25 html-semantics-compat（主攻候选）。cssom-view/geometry-1 渲染 API 面与
rendering-compat 工作面重叠，立项前先按 run-rules §9 协调。

## 与 rally 启动脚本的关系

- `scripts/rally-NN-<goal>.sh`（仓库根 scripts/）：**启动无人值守推进循环**
  （`rally run docs/goal/<goal>.md`）——这是 goal 的执行入口。**编号与本目录
  槽位一一对应**（rally-10 ↔ 10-timing、rally-15 ↔ 15-webstorage……），
  按编号序执行即按优先级序执行；已收口 goal（上表标记者）的启动器仅作
  历史入口，无需重跑。2026-10-08 前旧编号为立项流水号，已全部重对号。
- 无数字的 rally 脚本（rally-zero-web / rally-rendering-compat / rally-cdp-protocol /
  rally-devtools / rally-desktop-browser / rally-android-browser / rally-cron）：
  非 WPT 语料目标与总控入口，不占本目录槽位。
- 本目录 `NN-<goal>.sh`：**M1 语料预置**（fetch + 盘点 + 检查单），rally 轮内或
  手动执行均可；两族脚本按 goal 名互指。

## 约定

- **同一 WPT_REV pin**（lib.sh，与既有 fetch-*-subset.sh 一致）——保证语料可比、可复现
- 每个 goal 基线落 `docs/goal/<goal>/evidence/` 后，按
  `docs/compat/trends/wpt-suites.csv` 头注释约定把 planned 行转数据行
  （官网测试集总表自动展示）
- 新增 goal：复制任一脚本改 `GOAL`/`DIRS`（或按序号表插空），并在本表登记
