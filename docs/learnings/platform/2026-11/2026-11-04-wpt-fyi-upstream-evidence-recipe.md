---
date: 2026-11-04
modules: docs/goal/html-semantics-compat
---

# wpt.fyi 上游取证通道（代理 + 端点口径 + per-test message 直键）

## 问题

wpt.fyi API 直连长期超时（html-semantics s274 一轮、s277 三轮网络超时，
multicol 上游主干补查挂账因此悬置两轮）。同机 `curl` 其他域正常，仅
wpt.fyi 不通，且本机存在常驻代理进程但未设环境变量。

## 根因与通道

- 直连超时是网络路径问题，非 API 故障：本机 monocloud 代理监听
  `127.0.0.1:10801`（`ss -tlnp` 可见），`https_proxy=http://127.0.0.1:10801`
  后全部端点立通。
- `/api/runs` 的 `from`/`to` 必须**成对**传 RFC3339（`2026-11-01T00:00:00Z`
  形、URL 编码冒号）；只给 `from` 被静默忽略返回最新 run，只给 `to` 报
  Go 时间解析错。
- `/api/search` 按 `run_ids=<id,id>&q=<测试路径>` 查单测状态；`q` 用裸
  路径（`/html/semantics/...`），带 `test:` 前缀反而返回空结果。返回
  `legacy_status[i] = {passes, total, status}` 与 run 数组对位。
- `/api/results?products=...&test=...` 会 302 到 GCS，但重定向键带双斜杠
  **必 404**（`<sha>//html/...`）。可靠做法是绕开该端点直拼键：
  `https://storage.googleapis.com/wptd/<full-revision>/<browser-ver-os-sha>/<test-path>`
  （test-path 无前导斜杠，单斜杠）。browser-ver-os-sha 取自 run 元数据
  `results_url` 的末段（如 `chrome-159.0.8140.0-linux-22.04-ca1359c7ca`）。
  该 per-test JSON 含 `subtests[].message`——上游失败签名取证的关键面，
  summary/search 只有 0/1 计数拿不到 message。

## 解决方案与应用

s278 用此通道完成两项挂账定谳：

1. multiple-times 销案复核：纵深 6 时点（18 个月）12/12 连续双红 +
   现势 message 与本地失败逐字一致 → 销案升级为签名级对齐。
2. multicol 上游补查：chrome 1/1 + firefox 1/1 双绿 → 挂账定性改写为
   我侧 runner 时序建模缺口。

方法论注意：上游红面会随 build 漂移（chrome 132 窗序断言面 → chrome 159
restart 断言面），「上游同红」类销案口径必须钉死到现势签名字面，不能只
记 0/1 计数；引擎翻绿重开先例见 s41→s53（dialog ESC）。
