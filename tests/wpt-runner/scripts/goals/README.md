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
| 10 | [timing-animation-compat](../../../../docs/goal/timing-animation-compat.md) | hr-time; performance-timeline; user-timing; web-animations | 无（轻量热身） |
| 20 | [net-api-compat](../../../../docs/goal/net-api-compat.md) | fetch; xhr; url; mimesniff; streams; eventsource | WebSocket 二期挂账 |
| 30 | [encoding-compat](../../../../docs/goal/encoding-compat.md) | encoding | 无（小快赢） |
| 40 | [html-syntax-compat](../../../../docs/goal/html-syntax-compat.md) | html/syntax; html/dom | 无 |
| 50 | [uievents-compat](../../../../docs/goal/uievents-compat.md) | uievents; pointerevents | touch/pointerlock/IME 挂账 |
| 60 | [navigation-compat](../../../../docs/goal/navigation-compat.md) | history; navigation-api; html/browsers | **M3 iframe 切片用户门控** |
| 70 | [workers-compat](../../../../docs/goal/workers-compat.md) | workers; dedicated-workers | zero-page-runtime 契约先行 |
| 80 | [svg-compat](../../../../docs/goal/svg-compat.md) | svg | 渲染面归 rendering-compat |
| 90 | [security-hardening](../../../../docs/goal/security-hardening.md) | content-security-policy; mixed-content; secure-contexts | 已立项（2026-09-12） |
| 92 | [web-api-batch2](../../../../docs/goal/web-api-batch2.md) | clipboard-apis; fullscreen | 已立项（2026-09-12） |
| 99 | [webgl-compat](../../../../docs/goal/webgl-compat.md) | webgl | **远期门控**（M2+ 全门控，M1 盘点为唯一自主切片） |

## 约定

- **同一 WPT_REV pin**（lib.sh，与既有 fetch-*-subset.sh 一致）——保证语料可比、可复现
- 每个 goal 基线落 `docs/goal/<goal>/evidence/` 后，按
  `docs/compat/trends/wpt-suites.csv` 头注释约定把 planned 行转数据行
  （官网测试集总表自动展示）
- 新增 goal：复制任一脚本改 `GOAL`/`DIRS`（或按序号表插空），并在本表登记
