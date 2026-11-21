# DC-3 · 周跑接入预检（通道矩阵核对 + syntax-dom 预算修正）

2026-11-22。DC-3 对账收账前的接入核对：批九 html-infra 通道、批二 rendering
通道、批十一 template 腿、批十注入刀与 DC-1 实体矩阵在周跑
（weekly.yml `wptfyi-coverage` 管线）的落位、fetch 与预算逐环节核实；发现并
修正一处预算缺陷（syntax-dom 3600s 截断）。

## 通道落位（ci_discover_shards.py 本地复跑实证）

runner 套件 146 = 手调分片 43 覆盖 128 + 零头 18 → catch-all 分片 9：

| 本 goal 通道 | 分片 | fetch 目标 | 判定 |
|---|---|---|---|
| testharness-html-infra（批九，+623 基数） | catch-all-3 | `fetch-wpt-html-infra` | 落位 ✓ |
| testharness-html-rendering（批二通道） | catch-all-3 | `fetch-wpt-html-rendering` | 待首跑入账 |
| testharness-html-syntax template 腿（批十一） | syntax-dom（手调） | `fetch-wpt-html-syntax` | 随套件全量入账 ✓ |
| 批十注入刀（+29） | 随 html-infra 套件 | 同上 | 零 CI 编辑 ✓ |
| testharness-xhtml-syntax 实体矩阵（DC-1，25,606） | catch-all-9 | `fetch-wpt-xhtml-syntax` | 落位 ✓ |

- catch-all 编号随零头集合漂移：DC-1 时 xhtml-syntax 落 catch-all-8，批九/
  批二套件入池后漂至 catch-all-9——编号不锁，以套件名 + fetch 目标为准。
- workflow fetch 步容错契约（fetch 失败只废本套件面、不连带兄弟套件）与
  GITHUB_TOKEN 认证均已就位；结果文件名按套件互异，aggregate
  merge-multiple 平铺无碰撞。

## 预算修正：syntax-dom 3600→13200

- 本地实测：批十 preflight syntax 腿 **93 min**（1,049 键，其中 148 页级
  TO × 30s ≈ 74 min 为计时器型墙钟，不随 CI CPU 缩放）。
- CI 截断实证：coverage-2026-11-20 快照 html/syntax 仅 **40/439 文件、
  2,715/9,013 子测试**入账——3600s 预算烧尽即止，残面记 0。
- 修正：13200s ≈ 本地 ×2.4，与手调分片既有校准档一致（html-sem
  10,800s/本地 4,732s）；即使 syntax 与 dom 双套件病态同烧满上限，
  GH job 硬顶 360 min 兜底、聚合护栏按环比回归拦截坏快照回写
  （11-08 html-sem 事故实证护栏生效）。

## 预算核对（无缺陷面）

- catch-all-3/9 time_limit 5,400s：html-infra 本地 15 min（批九，8 jobs）→
  CI ≈2× 充裕；rendering 套件本地 ~1 min；实体矩阵「整轮秒级完成」
  （DC-1 实证，全过零 TO）。
- syntax-dom 期望 CI ≈110–190 min（TO 面计时器型 + 计算面 ×2），加
  build/fetch 远小于 360 min 硬顶。

## 门禁

`ci_discover_shards.py` 复跑 diff：52 分片仅 syntax-dom time_limit 一字段
变化（3600→13200），其余矩阵字节级同前。YAML+文档变更零 Rust 代码，无
构建/测试/clippy 门。

[批十证据](2026-11-22-dc2-batch10-query-encoding-getsub.md) ·
[批十一证据](2026-11-22-dc2-batch11-syntax-template-leg.md) ·
[DC-1 证据](2026-11-20-dc1-baseline-and-xhtml-entity-trial.md)
