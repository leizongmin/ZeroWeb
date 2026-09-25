# 安全加固 — 运行时控制面板（master.md）

**入口文档**: [../security-hardening.md](../security-hardening.md)
**创建日期**: 2026-09-12（goal 立项）
**最后更新**: 2026-09-25（R11：M3/M4 补齐落库 — MC 子资源面接线 + HSTS 响应注册 + navigator.permissions JS 面/change 事件 + 单测；make test 全绿 + reftest 202/202 + clippy/fmt 干净；**余项 = CSP corpus 账册零丢失核验（TIME_LIMIT≥5400）+ MC/SC off 臂全量 evidence + DC-2 翻 ✅ 终判**）

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
| P3 | CSP 主要指令引擎完整化 + report-only + 违规报告 | 🔄 M2-s1→s8 ✅ 检查点面收齐（script/style/img 元素+运行时+attr/connect/eval 全门禁 + hash 三算法 + 元素面口径 + GET 模板面 + corpus 第二批；全绿 74/445、subtests 174；余项全部记账/挂账，见 evidence/2026-09-25-m2-s8-csp-extensions.md） |
| P4 | Mixed Content 分级阻止 + HSTS 接线 | ✅ R11（kill-switch `mixed_content_enforcement` default-on + env `ZW_MIXED_CONTENT_ENFORCEMENT=0` 回退；stylesheet/img/script 三面接 `check_resource_url`；`note_hsts_response` 挂 fetch_url——见 evidence/2026-09-25-m3-m4-wiring.md） |
| P5 | Permissions API headless 语义层 + 事件 | ✅ R11（navigator.permissions 单例 status + state 活值 + change 派发 + request headless 语义 + desc 校验；WAB2 既有测试按意图适配——见 evidence/2026-09-25-m3-m4-wiring.md） |
| P6 | kill-switch + A/B + default-on 决策 | ⏳ M5 |

## 已完成切片

- **R11（2026-09-25）M3/M4 补齐**（evidence/2026-09-25-m3-m4-wiring.md）：
  - **M3a Mixed Content 子资源面**：`WebViewConfig::mixed_content_enforcement`
    （default-on；env `ZW_MIXED_CONTENT_ENFORCEMENT=0` 回退，照 `ZW_MO_HOST_TRIGGER`
    先例）+ 三检查点接 `check_resource_url`——外链 stylesheet（Blockable 阻止；
    Upgraded 按升级 URL 抓取与 url() 解析）、img（OptionallyBlockable 升级 HTTPS
    抓取；**缓存键留 markup 原始 URL** 防 painter 失配）、外链 script/module
    （Blockable 不执行；Upgraded 走升级 fetch）。`complete_fetched_page` 补
    `set_page_origin` 与 fetch_url 对齐。零影响面：判定以 https 页面源为前提，
    file:// 工作区结构性短路。
  - **M3b HSTS 注册**：`note_hsts_response` 挂 `fetch_url` HTTPS 响应分支（按最终
    响应 URL host 解析 `Strict-Transport-Security` → `register_hsts`）。
  - **M4 navigator.permissions**（shim part02.js）：单例 status（spec
    identical-descriptor）+ state 活值 getter + change 派发（`__zwSetPermission`/
    request 状态实际变化时 listener+onchange，同值不派发）+ request headless 语义
    （prompt 自动授予/denied 维持）+ desc 必选成员校验。JS 面挂 shim 权限注册表
    （WAB2 clipboard/fullscreen 同一 store）；PermissionManager 保留 app UI 侧。
    WAB2-M2-s3 既有测试按意图拆 execute 适配活值语义（期望串不变）。
  - **单测**：webview `mixed_content_gate` 5 test + engine part07 1 test；engine
    全量 2735 绿、`make test` 全绿、`make reftest` 202/202、clippy/fmt 干净。
  - **运维记档**：CSP corpus 全量实际墙钟 >3600s（账册红案 ×30s 内部超时累加），
    复跑须 `TIME_LIMIT>=5400`（Makefile 已透传；testharness-dom 先例）。本 clone 与
    同 goal 后继会话并行碰撞 ×2（互杀 corpus run）——按 §9 让行不硬解，corpus
    零丢失核验移交后继会话。
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
- **R9（2026-09-25）M2-s8 运行时 img src-set 检查点**（evidence/2026-09-25-m2-s8-csp-extensions.md）：
  - shim 双路径门禁（part05 R56h IDL setter fetch 链前判定 + part04 setAttribute
    路径）+ `__zwCspImgCheck` 回调（violation document 站 + targetSelector 元素站
    能力就位）+ **注册点迁移无条件区**（首版误落 fetch_handler 条件块——探针修复）。
  - corpus：全绿 72→74（meta-img-src/meta-modified 翻绿）、subtests 172→174。
  - img corpus 剩余红点 = `{{location[scheme]}}` location 模板族（记账）+ 位置断言。
- **R8（2026-09-25）M2-s7 script-src-attr 检查点**（evidence/2026-09-25-m2-s7-csp-extensions.md）：
  - `check_script_attr`（处理器原文 hash，不 trim）+ `effective_script_attr_directive`
    （元素面口径 script-src-attr）；shim 双求值点接门禁（_ensureInlineHandler 编译
    缓存 + R155 activation 直 eval）+ `__zwAttrClear` 标记通道（预检在未包装原文、
    代理只信任标记——双查包装壳 hash 必失配，探针修复）。
  - corpus：全绿 71→72（**M2-s6 −1 回退案闭合**，零丢失）、subtests 169→172。
- **R7（2026-09-25）M2-s6 corpus 第二批 + eval 门禁**（evidence/2026-09-25-m2-s6-csp-extensions.md）：
  - 扩批 +5 目录（inheritance/navigation/sandbox/unsafe-eval/wasm-unsafe-eval，
    分母 415→445）；eval 门禁 shim 层（v8 无 codegen 写入面）——`eval_violation`
    多政策并集 + eval/Function Proxy 包装 + `__zwCspEvalBlocked` 回调入队。
  - **per-script 作用域**（关键设计）：持久包装折断 harness/shim 装配链（首跑 −34）
    ——机制侧 `(0,globalThis.__zwRealEval||eval)` + 非 harness 脚本前后装/卸两段修复；
    Function Proxy 后置 ensure_js_shim（shim init 经 new Function）。
  - corpus：全绿 65→71（+7 净，−1 event-handler 案归 attr 面 M2-s7）、subtests
    165→169；unsafe-eval 簇 5/10。
- **R6（2026-09-25）M2-s5 connect-src 阻止族**（evidence/2026-09-25-m2-s5-csp-extensions.md）：
  - `check_connect` + `has_directive` pub 面；`__zw_fetch` 统一桥检查点（导航面豁免；
    被阻止 → `__zw_fetch_error:` + Arc 共享队列 run 尾 document 站派发）。
  - corpus：全绿 62→65（**+3 零丢失**）、subtests 159→165（28.9%）。
  - 跨域归因：外链 stylesheet ID 选择器应用缺口（渲染流域域，实证 default 配置同黑）
    + fetch blocked shim resolve(ok:false) 契约 divergence——均记账不硬改。
- **R5（2026-09-25）M2-s4 元素面口径 + GET 模板面**（evidence/2026-09-25-m2-s4-csp-extensions.md）：
  - 元素面指令口径（style-src/script-src 显式时上报 X-elem——Chromium 对齐，corpus
    实参交换形断言元素面；无案断言裸指令名零回退）。
  - **`{{GET[name]}}` 模板替换**（runner script fetcher）+ corpus 级 support/ 目录
    补拉——logTest/alertAssert/checkReport 全族 support 脚本解锁，**+25 全绿主杠杆**。
  - corpus：全绿 37→62（**+25 零丢失**）、subtests 118→159（21.4%→27.9%）。
- **R4（2026-09-25）M2-s3 style 检查点**（evidence/2026-09-25-m2-s3-csp-extensions.md）：
  - inline `<style>` 检查点（`extract_style_elements_csp` 原文扫描 + 等长空白原地
    清空——元素保留 target 可达）+ 外链 stylesheet 检查点（skip fetch + link error）
    + run 尾元素站 violation 派发（targetTag/targetOrdinal 泛化）。
  - **hash 匹配器补全**（探针闭环两轮）：三算法并立（sha384/sha512——
    style-src-hash-allowed 一政策各算法一枚对应不同元素）+ 算法前缀 ASCII 大小写
    不敏感（'SHA256-'/'sHa256-' 面）。首跑 −3 假回退全数恢复。
  - corpus：全绿 37 持平零丢失、subtests 115→118（21.4%）。
  - 工程记档：修 csp.rs 同型代码块须逐一核对宿主函数（script/style 两侧匹配器
    锚点错位曾致第一轮修错面）。
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

1. **M5 终判收口（R12，唯一余项）**：①CSP corpus 全量复跑（`make testharness-csp
   TIME_LIMIT=5400`）→ 账册对比 m2-s8-final.json（74 全绿/445、subtests 174）零丢失；
   ②`ZW_MIXED_CONTENT_ENFORCEMENT=0 make test` off 臂全量零 delta evidence（kill-switch
   实效面，DC-3 letter）；③DC-2 Mixed Content/HSTS/Permissions 翻 ✅ → goal DONE 判定。
2. **碰撞注意**：同 goal 后继会话已在跑 off 臂/reftest/smoke/corpus 序列
   （systemd scope zw-r11-*）——接手前先 `git pull` + 查 `/tmp/zw-r11-*.log` 避免重复跑。

**跨域记账（协调不硬改）**：
- 外链 stylesheet ID 选择器应用缺口——pipeline/style 面预存（渲染流域 crates 域）。
- fetch blocked 契约：shim host 错误 wire → resolve(ok:false)（上游期望 reject）。
- inheritance/sandbox 簇全红（跨文档导航/iframe 管道重入条件）——深多进程面挂账。
- eval 违例调用点定位（V8 栈面）——v8 crate 无 codegen 回调写入面，shim 层位置捕获
  需 per-eval 包装传参（M2-s7 评估）。

**跨域记账（协调不硬改）**：
- 外链 stylesheet ID 选择器应用缺口（stylenonce-blocked allowed.css black）——
  pipeline/style 面预存（default 无 CSP 同样黑），渲染流域 crates 域。
- fetch blocked 契约：shim host 错误 wire → resolve(ok:false)（上游期望 reject）——
  shim 面语义 divergence。
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
| M2 — CSP 指令引擎完整化（接线） | ✅ s1-s8（74/445 零丢失，subtests 174=28.8%；检查点面收齐） |
| M3 — Mixed Content + HSTS | ✅ R11（语义+单测 ✅；导航面 ✅；子资源面三检查点 + HSTS 响应注册 ✅——mixed-content corpus 2 案维持 infra 挂账：popups/模板/ResourceTiming） |
| M4 — Permissions 语义层 | ✅ R11（PermissionManager 语义+单测 ✅；navigator.permissions query/request + 单例 + change 事件 + 活值 state ✅） |
| M5 — 收口 | ◐ A/B 零回归门禁 ✅ + default-on 落定 ✅ + reftest ✅ + DC 逐项判定 ✅ + 挂账定稿 ✅（evidence/2026-09-25-m5-ab-default-on.md）；**余 = corpus 零丢失核验 + off 臂 evidence → DONE 终判** |

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
