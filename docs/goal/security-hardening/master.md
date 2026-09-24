# 安全加固 — 运行时控制面板（master.md）

**入口文档**: [../security-hardening.md](../security-hardening.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-24（R1：M1 勘察修正基线事实 + 三 corpus 导入 + 首跑基线）

---

## 当前状态

**专项定位**：zero-security 从「CSP 基础」推进到「CSP 主要指令完整 + report-only +
违规报告」+ Mixed Content 分级阻止 + HSTS + Permissions 语义层。WPT 三 corpus 为
验收标尺；CSP/Mixed Content 默认行为变更走 kill-switch + A/B 门禁（event-loop-spec ②
先例）。

**与兄弟 goal 的边界**：
- web-api-batch2 — 其 Clipboard 依赖本 goal DC-4 权限语义层（供数关系）；碰 engine
  shim 面 git log 核对
- cdp-protocol / devtools — CSP 违规 console 报告面共享管线，消费侧协调
- rendering-compat — crate 零重叠
- android-browser — 无共享面

## 基线事实（R1 勘察修正，2026-09-24 实测）

> 立项时（2026-09-12）「CSP 指令覆盖面/report-only/违规上报语义待实测盘点」的假设
> 已被实测推翻一半：**语义库层远超预期，管线接线层完全缺位**。目标实际剩余工作 =
> 接线（consume 侧）+ WPT 收敛，而非从零实现语义。

- **语义库层（crates/security，6-8 月已建成，单测齐全）**：
  - `csp.rs`（1,064 行）：主要指令全语义（default-src/script-src/style-src/img-src/
    connect-src/frame-src/font-src/media-src/object-src/base-uri/form-action/
    frame-ancestors/child-src/worker-src/manifest-src/navigate-to/sandbox +
    upgrade-insecure-requests/report-uri/report-to/strict-dynamic/eval/wasm-eval/
    nonce/hash）+ `ContentSecurityPolicyReportOnly`（report-only 模式）。历史修复：
    R3342/R3343/R3389 + 源列表惰性预计算 perf（9a2deff0e）。
  - `hsts.rs`（383 行）：解析/存储/`should_upgrade`/includeSubDomains/过期清理。
  - `mixed_content.rs`（380 行）：`check_mixed_content` 分级 + `upgrade_to_https`。
  - `permission.rs`（466 行）：PermissionManager query/grant/deny/revoke + 状态机。
  - `context.rs`：`SecurityContext` 统一面（HSTS store + page origin + CSP + mixed
    content + upgrade）→ `check_resource_url`。
- **接线层（真实缺口）**：
  - **CSP 头零解析**：`Content-Security-Policy` 字符串在 webview/engine/net 生产码
    零命中——导航响应头与 `<meta http-equiv>` 都不进 CSP 引擎，运行时无强制。
  - **`SecurityContext::check_subresource_url`（webview.rs:5426）零调用方**——子资源
    加载决策点未接入。
  - **HSTS `register_hsts` 零调用方**——net 响应头不进 HSTS store。
  - **`SecurityPolicyViolationEvent` 引擎零实现**——违规事件无派发面。
  - **Permissions 无 JS 面**——`PermissionManager` 仅 browser app UI 层消费
    （app_input.rs），navigator.permissions 不存在。
- **验收标尺**：三 corpus 已导入（见 M1 记录）；mixed-content 上游 corpus 的 window
  可执行面天然很小（gen/ 全为 window.js/iframe 包装形态）。

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | zero-security CSP 现状盘点 | ✅ R1（见上「基线事实」） |
| P2 | content-security-policy / mixed-content / secure-contexts 三 corpus 导入 + 基线 | ✅ R1（415+2+4 案执行；CSP subtests 16.3% 基线见 evidence/） |
| P3 | CSP 主要指令引擎完整化 + report-only + 违规报告 | ⏳ M2（= 接线：meta/头解析进引擎 + 子资源检查点 + violation 事件派发；kill-switch + A/B） |
| P4 | Mixed Content 分级阻止 + HSTS 接线 | ⏳ M3（mixed_content/checkpoint 接入子资源面；HSTS register 接 net 响应） |
| P5 | Permissions API headless 语义层 + 事件 | ⏳ M4（navigator.permissions JS 面挂 PermissionManager） |
| P6 | kill-switch + A/B + default-on 决策 | ⏳ M5 |

## 已完成切片

- **R1（2026-09-24）M1 资产切片**：
  - `tests/wpt-runner/scripts/fetch-security-csp-subset.sh`（新增）：三 corpus 首批
    导入，sparse blob:none 浅克隆策略（逐文件 raw + contents API 枚举在本机网络下
    10 分钟仅落 75 文件且触发 GitHub 匿名限额——脚本头注释记档）。
  - `testharness.rs`：`CSP_CORPUS_SUBDIRS`（22 子目录）/ `MIXED_CONTENT_SUBDIRS` /
    `SECURE_CONTEXTS_SUBDIRS` + `security_case_skipped`（ref/manual/iframe/worker
    内容筛减）+ `run_content_security_policy_cases` 等三 runner（run_corpus_subdirs
    同款形态）。
  - `main.rs`：`testharness-csp` / `testharness-mixed-content` /
    `testharness-secure-contexts` 三子命令；Makefile 三对目标。
  - 基线：evidence/2026-09-24-m1-security-wpt-baseline.md（CSP 415 案执行 / 30 全绿 /
    subtests 91/557 = 16.3%；mixed-content 2 案、secure-contexts 4 案 0 绿）。口径
    注记：无强制下「allowed」案天然绿，M2 接线后 blocked/allowed 双向受检——
    blocked 案全红（Timeout × 134 大半）才是真实缺口面。

## 下一步计划

1. **M2（接线主战场）**：CSP 消费侧接入——①导航/文档 CSP 源装配（响应头 +
   `<meta http-equiv>`）+ kill-switch（`ZW_CSP_ENFORCE` 类 env，default-off）；
   ②子资源检查点（engine/webview 资源加载决策处调 `check_subresource_url`）；
   ③`SecurityPolicyViolationEvent` 派发（blockedURI/sourceFile/lineNumber/column-
   Number/_effectiveDirective 语义对 securitypolicyviolation corpus）；
   ④report-only 面走 console 报告（与 cdp-protocol 消费侧协调）。
2. **M2 扩批**：CSP corpus 第二批目录（inheritance/navigation/sandbox/unsafe-eval/
   wasm-unsafe-eval/inside-worker 视 worker 面进度）。
3. **M3**：Mixed Content 子资源接入 + HSTS net 响应注册（`register_hsts` 挂头解析）。
4. **M4**：navigator.permissions JS 面（query/state/request headless 语义 + change
   事件），为 web-api-batch2 Clipboard 供数。
5. **M5**：A/B 零回归 + default-on 决策 + DC 逐项判定。

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 勘察 + WPT 导入与基线 | ✅ R1（2026-09-24，基线 16.3%） |
| M2 — CSP 指令引擎完整化（接线） | ⏳ |
| M3 — Mixed Content + HSTS | ⏳ |
| M4 — Permissions 语义层 | ⏳ |
| M5 — 收口 | ⏳ |

## 验证基线

- 测试基线：R1 时点 main 全绿（`make test` 19,402P 口径，c8ada9658 守成轮记录；
  经 test-guard）
- WPT 标尺：三 corpus 首批导入（WPT 315976933870b34d6ea30e3f6643403edae678ba，
  与 observers/clipboard-apis/fullscreen 同 pin）；通过率基线见 evidence/
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings`
  全过；行为变更（M2+ CSP 强制）走 kill-switch + 全量 A/B 零回归

**碰撞管理**：碰 engine shim 资源加载检查点前与 web-api-batch2 / cdp-protocol
`git log` 互核。runner 共享文件（testharness.rs/main.rs/Makefile）为各 goal 并行
编辑面——提交前 pull --rebase 核对。
