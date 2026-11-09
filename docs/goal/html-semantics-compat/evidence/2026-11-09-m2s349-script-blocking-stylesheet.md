# M2 切片 349：script-blocking stylesheet 等待窗 + 内联 CSP hash 原文摘要 + 解析交付集预盖章

**日期**: 2026-11-09
**目标对**: `html/semantics/scripting-1/the-script-element/script-text-modifications.html` + `script-text-modifications-csp.html`（s348 基线 21F 中双 F）
**结果**: 双案 F→P；域 join det F→P=2 / P→F=∅ / T=8 同集零位移；语料口径 **21F→19F**

## 上游语义（勘根）

spec [prepare-a-script](https://html.spec.whatwg.org/multipage/scripting.html#prepare-a-script)：「a style sheet that is blocking scripts」（`slow.py?pipe=trickle(d1)`，1s）在飞行时推迟 parser-inserted script0 的**求值**（prepare 已完成）；页面首个脚本注册 500ms 定时器，在窗内把 `script0.textContent` 改写为 `t.unreached_func("This should not be evaluated")();`。样式表落定后 script0 按 **prepare 时刻快照（原文）** 求值，断言 DOM 的 `textContent` 已是改写文本 → `t.done()`。CSP 变体加 `script-src 'sha256-…'` hash 门：hash 源 = 元素 **child text content 原文**（[CSP source-list hash matching](https://w3c.github.io/webappsec-csp/#source-list-hash-matching-algorithm)，不 trim）。

## 旧红与三处根因

| 面 | 旧红形态 | 根因 |
|---|---|---|
| 等待窗缺失 | `assert_equals` textContent 失配（script0 执行时改写未发生） | runner 通道样式表不推迟脚本求值：s152 静态扫描窗把 trickle 延迟塌缩为 0；webview 无 script-blocking stylesheet 等待面 |
| CSP hash trim | `assert_unreached: CSP check was done against the original text…`（script0 被误阻 + violation） | extract 对脚本体 trim（`extract.rs` `strip_script_cdata(raw.trim()).trim()`），hash 按 trim 体摘要 ≠ spec 原文摘要 |
| LIVE 文本重 prepare（Face A/B 落地后暴露） | `assert_unreached: This should not be evaluated`（改写体被执行） | textContent= 的 childList 记录汇流 `_mo_notify`（part01 s109 臂）→ `_zwScriptMaybeStartDynamic` 读 **LIVE** `el.text` 执行改写体；upstream script0 在 prepare（parse）即 already-started，本架构 `_zwScriptStarted` 盖章在求值时刻（`__zw_set_current_script`），等待窗内尚未盖章 → 门失守 |

## 施工

1. **Face A 等待窗**（shim 计时 + webview 交错点轮询）：
   - part01 trickle 分支塌缩收窄：`_ssScanCollapseOnly` = 扫描窗 **且** 非 runner lane（`__zw_test_setTimeout` 不在场）——runner lane 保留 trickle 真延迟（1s），非 runner lane 维持 s152 塌缩基线；interactions 族 `_zwFetchSimDelay` 塌缩原样不动。
   - part03 `_ssDoVisit` media 匹配臂在飞计数 `_zwSsBlockPending`（s163 defer 臂不计数；同步 error 臂 done 内自归零无泄漏）+ 探针 `__zwScriptBlockingSheetsPending`。
   - webview `run_page_scripts_impl` 脚本循环内交错点：探针 true → 10ms 轮询环，每轮泵 runner timer wheel `__zw_fire_due_timers`（延迟由 shim setTimeout 实现，PyRoute 同步契约禁 host sleep），30s 封顶告警。
   - PyRoute 无 slow.py 臂不改：404 落定与真延迟对消费者时序等价（内容不敏感）；bare slow.py（无 pipe）无 shim 延迟，零波及。
2. **Face B hash 原文**：
   - extract `ScriptElementCsp.raw_text`：`doc.child_text_content(script_id)` 原文（首尾换行保留；src 形元素无文本子 → 恒 `Some("")`）；extract 单测扩 raw-html 断言 + 结构字面量 5 处。
   - webview 内联臂 hash 源改 `raw_text`（`filter` 非空 → `script_hash_sha256_base64`）；**执行体不变**（仍 extract trim 体，lineno 面零波及）。
   - csp_gate 新测 `csp_inline_hash_matches_raw_text_content_s349`（带首/尾换行原文摘要放行 + 零 violation；旧 trim 口径下红）。
3. **s109 触发臂交付集预盖章**（暴露面修复）：
   - part06 `__zw_stamp_parser_scripts(idxList)`：按文档序索引（与 currentScript 同号空间）取元素盖 `_zwParserDelivered[sel]`；空源 bail/type 阻断/nomodule/for-event 压制单元不在交付集（不盖）→ 145 形空脚本补文本触发保持。
   - part03 `_zwScriptMaybeStartDynamic` started 门并读 `_zwParserDelivered`。
   - webview `run_page_scripts_impl` 脚本阶段起点（首脚本前）投递交付索引表（best-effort，`typeof` 守卫）。
   - **与 `_zwScriptStarted` 分表**：s110 `script.async` getter 的「已 started → true」臂编码**执行事实**（引擎沙箱静态页 sc2 执行前读 false、122 reinsert 前读真由 started 臂保）——预盖章不得污染；新表只进 `_zwScriptMaybeStartDynamic`。

## Blast radius 普查（语料 grep）

- trickle + 静态样式表页：106 族 ~14 文件（`css/import.css?pipe=trickle(d2)`，全 Pass，每页 +~2s 墙钟）、150 族 ~4、107 族 2、link-load-event（document-metadata 域，域外 join；尾随 trickle(d3) 真延迟，load 由 s157 load-hold 机制持至落定）。
- bare `slow.py`（无 pipe → 无 shim 延迟）：load-error-events-2/3、promise-reject-and-remove-iframe、link-load-error-events——零波及。
- interactions tentative（media 不匹配 → s163 defer 臂不计数）。
- 063.html T 案无样式表（T 稳定）。

## 验证

- **solo 双案 P**：`make testharness-html-semantics "FILTER=script-text-modifications" TIME_LIMIT=300` → 2 Pass（evidence/logs/s349-solo-pair.log）。红态演进链：`assert_equals 失配`（仅 Face B）→ `unreached`（Face A 窗生效暴露 LIVE 重 prepare）→ 双 P（交付集预盖章后）。
- **域 join**（LC_ALL=C + `(key,state)` sort -u + 双态键检查——双日志双态键均 0）：A=s348-b-script-element.log **21F/1098P/8T** vs B=s349-b-script-element.log **19F/1100P/8T**；F→P=2 恰双目标逐键吻合；P→F=∅；F 集 removed-only/added-only 双 comm ∅；T 集 A−B/B−A 双向 ∅（同集）。
- **collateral 抽查**（vs s348 基线同键集）：execution-timing 144P/6F/1T + load-error-events 69P/0F/1T —— F/T 键集与基线逐键同（6F=041/056/063/068/143/probe12-defer-slow 既有；2T 既有）；emptyish-script-elements 7P/0F（145 形触发保持）；script-type-and-language 463P/0F（type 门/async getter 未染）。
- **单测**：webview `--lib` **844P/2F**（+1 新测；2F=image_decoder + csp_connect_gate_blocks_fetch_sh1_m2s5 既有 env 红）；script-sandbox **419P/0F**；runner `--bins` **298P/0F**；engine extract_script_csp_info **1P**（含 raw_text 原文断言）。
- **clippy** zero-webview/zero-engine/zero-script-sandbox `--all-targets -D warnings` 0 告警；`cargo fmt --all -- --check` 净。

## 挂账 / 下一步

- renderer 车道同窗触发面未接（`apps/renderer/runtime.rs` 页面脚本道不调预盖章——s349 域外，语料无案面；后续收口候选）。
- webview env 红 2 案承继（image_decoder；csp_gate fetch 归 M3 流）；仓内 stash@{0} 承继；QuickJS parse_check 接线承继（s348 FIXME）。
- 域残余 19F 逐案勘根（next 候选）；T=8 长尾在册。
- s152 塌缩语义收窄记录：非 runner lane 维持塌缩基线不动（站点扫描窗行为零变化）。

## 日志

- evidence/logs/s349-solo-pair.log（solo 双案）
- evidence/logs/s349-b-script-element.log（域跑 B 面）
- s348 基线 A 面：evidence/logs/s348-b-script-element.log（既有）
