# R11 — M3/M4 门禁收口 + goal 终判（DC 全 ✅）

**日期**: 2026-09-25
**前置**: M3/M4 补齐 commit `0035219dd`（Mixed Content 子资源面 + HSTS 注册 +
navigator.permissions JS 面）+ 本轮 1 例测试健壮性修复（见「本轮修复」）。
**本轮修复文件**: `crates/webview/src/tests/mixed_content_gate.rs`（仅测试构造，产品码零改动）

## A/B 双臂（DC-3，`mixed_content_enforcement` default-on）

| 臂 | 门禁 | 结果 |
|---|---|---|
| on（default：`csp_enforcement` + `mixed_content_enforcement` 均 on） | `make test` 全量（test-guard，systemd 24G scope） | **19,451 passed / 0 failed** |
| off（`ZW_MIXED_CONTENT_ENFORCEMENT=0` 全量环境翻转） | 同上 | **19,451 passed / 0 failed，与 on 臂完全同值——零 delta** |

- 零 delta 机理 + CSP 侧 A/B（R2→R9 default-off 逐轮 + R10 翻转臂）见
  [2026-09-25-m5-ab-default-on.md](2026-09-25-m5-ab-default-on.md)；本轮为 MC
  kill-switch 的 off 臂全量补录（R10 时开关未出生）。
- **本轮修复（off 臂首跑抓到）**：M3 五个 gate 用例中四个经
  `WebViewConfig::default()` 构造——Default 读 `ZW_MIXED_CONTENT_ENFORCEMENT` env，
  off 臂环境下翻为 off 致 4 例假红（产品码无涉）。修法照 `csp_gate.rs` 既有口径：
  强制面用例显式 `mixed_content_enforcement: true`。修后定向双环境复核 5/5 绿
  （env=0 / env unset），双臂全量各复跑一轮（数字见上表）。

## DC-4 质量门禁（最终树，修复后）

| 门禁 | 结果 |
|---|---|
| `cargo fmt --all -- --check` | 干净 |
| `cargo clippy --workspace --all-targets -- -D warnings` | 干净（exit 0） |
| `make reftest` | **691/691 = 100%**（Layout 489 + Text 202），零回归 |
| `make product-smoke` | PASS（welcome + article @375/@320 struct-check 0 issues） |
| `make test` on 臂 | 19,451/0（+6 vs R10 锚 19,440P = M3 5 test + M4 1 test） |

## 三 corpus 复核（WPT 315976933870b34d6ea30e3f6643403edae678ba，不重拉）

| corpus | M2-s8/M1 记录 | R11 实测 | 判定 |
|---|---|---|---|
| content-security-policy | 74/445 全绿，subtests 174/604 = 28.8% | **74/445，174/604 = 28.8%（逐 case 同值）** | 零丢失零扰动 |
| mixed-content | 2 案 0 绿（blob Timeout + imageset 缺 support 文件） | 同两案同失败签名 | 挂账不变（infra） |
| secure-contexts | 4 案 0 绿（popup/worker infra） | 同四案同失败签名 | 挂账不变（infra） |

原始数据：[2026-09-25-r11-csp-final.json](2026-09-25-r11-csp-final.json)（runner
`--json` 全量 445 案逐 subtest）。

**CSP corpus 时长注记**：本机全量 ~65 分钟，`make testharness-csp` 默认 test-guard
`--time-limit 3600` 不够（本轮首跑被杀，exit 124）——目标已支持 `TIME_LIMIT` 覆盖，
R11 实跑经 test-guard 同款包裹 `--time-limit 7200`（仅墙钟放宽，内存阈值
per-proc 4G / total 8G 未动，非测量配置变更）。

## DC 逐项终判（对照入口文档）

| DC | 判定 | 证据 |
|---|---|---|
| DC-1 WPT 导入与基线 | **✅** | R1（三 corpus + 基线 16.3%）→ 收敛轨迹 R2→R9 至 28.8% |
| DC-2 四面语义收敛 | **✅** | CSP：检查点面收齐 + 74/445 可追踪提升；Mixed Content：分级阻止 + 子资源三面接线（style/img/script，`0035219dd`）；HSTS：解析/存储/includeSubDomains + net 响应注册 + 单测；Permissions：navigator.permissions 单例/活值/change/request + TypeError 校验 + 单测 |
| DC-3 行为变更门禁 | **✅** | CSP：R2→R9 default-off 逐轮零 delta + R10 翻转臂全绿；MC：本轮 off 臂全量 19,451/0 零 delta + on 臂同值（双臂记录见上） |
| DC-4 测试与质量 | **✅** | 本文件上表（fmt/clippy/make test/make reftest 全绿） |

**goal 判定：DONE 允许条件满足**（DC-1~4 全 ✅；验收基于上游真实 WPT corpus；
build/make test/clippy 全过；master.md 自洽；evidence 持久化）。

## 遗留挂账（域外/infra，不阻塞判定——移交后续 goal 或等点名）

承接 [2026-09-25-m5-ab-default-on.md](2026-09-25-m5-ab-default-on.md) 挂账 1-6
（fetch blocked 契约 divergence、外链 stylesheet ID 选择器、report-uri infra、
`{{location[scheme]}}` 模板族、inheritance/sandbox 深多进程、eval 调用点定位），
另增 2 项：

7. CSP corpus 全量跑墙钟 ~65 min 超过 `make testharness-csp` 默认 guard 3600s——
   后续轮跑用 `make testharness-csp TIME_LIMIT=7200`（本轮已实测可行）；是否调
   Makefile 默认值留给守护轮观察。
8. `WebViewConfig::default()` 读 env 的开关族（csp_enforcement /
   mixed_content_enforcement）对「显式构造才不随环境漂移」的测试写法约定——本轮已照
   `csp_gate.rs` 口径收口 MC 侧；后续新增开关 test 套件须沿用显式布尔写法。
