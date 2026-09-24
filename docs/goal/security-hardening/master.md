# 安全加固 — 运行时控制面板（master.md）

**入口文档**: [../security-hardening.md](../security-hardening.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-24（R3：M2-s2 检查点扩面 — 源位置定位 + 元素站 target + markup img 检查点；全绿 33→37 零丢失 / subtests 20.6%）

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
| P3 | CSP 主要指令引擎完整化 + report-only + 违规报告 | 🔄 M2-s1/s2 ✅（骨架 + 源位置 + 元素站 target + markup img 检查点；余 = style/attr/运行时 img/eval 检查点，见 evidence/2026-09-24-m2-s2-csp-extensions.md 缺口表） |
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
- **R3（2026-09-24）M2-s2 检查点扩面**（evidence/2026-09-24-m2-s2-csp-extensions.md）：
  - 源位置定位（`extract_script_source_positions` + runner 原始源预计算经
    `csp_script_positions` 覆盖 + prepared ordinal 重映射——blockeduri-inline 15:9
    精确命中全绿）；元素站 target 派发（`__zwSelector`/`__zwHandle` listener-store
    key + receiverProxy identity 保持）；markup img 检查点（`check_image` + fetch 侧
    装配前移/阻止不 fetch/页面脚本后派发 error——img-src 3 案翻绿）。
  - corpus：全绿 33→37（**+4 零丢失**）、subtests 109→115（19.6%→20.6%）。
  - 探针记档：runner harness 内联扭曲行号（5267→5→15:9 三段收敛）；error 派发须在
    页面脚本后 + `page_scripts_initialized` 置位后早退分支前（无变更页面同达）。
- **R2（2026-09-24）M2-s1 接线骨架**（evidence/2026-09-24-m2-s1-csp-wiring.md）：
  - SecurityPolicyViolationEvent 构造器（engine dom_bindings；constructor-required-
    fields 14/14 全绿）；meta CSP 装配 + script 检查点（kill-switch
    `WebViewConfig::csp_enforcement` default off）+ violation 事件 document 站派发
    （shim `__zw_dispatch_securitypolicyviolation`）。
  - zero-security：SecurityContext 文档级 CSP 装配 + `check_script`（nonce/hash/URL
    三路）+ `script_hash_sha256_base64`（sha2 workspace 依赖接入本 crate）。
  - runner：实验臂开关 + 注入脚本 nonce 戳记 + `data-zw-harness` 豁免。
  - corpus：全绿 30→33（**+3 零丢失**）、subtests 91→109（16.3%→19.6%）；中途
    「−4 假回退」两轮探针闭环归因（hash 缺位 + harness 被拦）修复后归零。

## 下一步计划

1. **M2-s3（检查点继续扩面）**：①style 检查点（inline `<style>`/style 属性/外链
   stylesheet——style-src 面最大红案簇）；②script-src-attr（onclick 等内联事件
   处理器——targeting block2/4 面）；③运行时 img src-set 钩子（createElement('img')
   族——securitypolicyviolation img 4 案，shim 属性写管道）；④eval 检查点
   （is_eval_allowed 钩 sandbox eval 面）。
2. **M2-s4**：report-only 政策面走 console 报告（与 cdp-protocol 消费侧协调）+
   CSP corpus 第二批目录扩批（inheritance/navigation/sandbox/unsafe-eval/
   wasm-unsafe-eval 视面进度）。
3. **M3**：Mixed Content 子资源接入 + HSTS net 响应注册（`register_hsts` 挂头解析）。
4. **M4**：navigator.permissions JS 面（query/state/request headless 语义 + change
   事件），为 web-api-batch2 Clipboard 供数。
5. **M5**：A/B 零回归 + default-on 决策（env 开关暴露 + 全量 A/B）+ DC 逐项判定。

**R2 实操记录（探针闭环，防复踩）**：
- document 级事件派发路径：shim doc 槽位独立于 EventTarget 链；native
  `__zw_native_get_document()` 返原始 id（无 dispatchEvent）；window 站经
  `__zw_dispatch_event('html',…)` 可达。violation 派发最终走专用 shim 全局
  `__zw_dispatch_securitypolicyviolation`（document 站 `_dispatchWithBubble`）。
- shim `init_string` 型 helper 对 absent/undefined 成员走 to_string 落 "undefined"
  字面——全字段可选缺省语义须显式 undefined 判定（SPV 构造器已按此写）。
- runner 注入/内联脚本在 no-nonce CSP 下会杀 harness——`data-zw-harness` 标记 +
  webview 检查点豁免（投递形态差异非页面内容）。

**待用户决策清单**：
- （暂无）

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — 勘察 + WPT 导入与基线 | ✅ R1（2026-09-24，基线 16.3%） |
| M2 — CSP 指令引擎完整化（接线） | 🔄 s1 ✅ s2 ✅（37/415 全绿零丢失，subtests 20.6%）；s3+ = style/attr 检查点 / 运行时 img / eval / connect 面 |
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
