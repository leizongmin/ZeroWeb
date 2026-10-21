# M2-S4P — C 类余项第三批：session history 上限 + 速率限制 + 动态脚本执行 + traverse 语义面（2026-10-09/10）

**通道**: `make testharness-navigation`（test-guard 包裹，TIME_LIMIT=2700）
**全量运行日志**: [2026-10-09-m2-s4p-full-corpus.txt](2026-10-09-m2-s4p-full-corpus.txt)
**前序**: [2026-10-09-m2-s4o-c-class-ii.md](2026-10-09-m2-s4o-c-class-ii.md)

## 切片内容（三独立根因）

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **joint session history 50 条上限**：`_histTrimOldest` 幂等 helper（classic 列表 shift + cursor 前移 + Navigation API 记录出列 `_navList`/`_navDetached` 摘除 + dispose 派发），`_pushHistNav`/`_histApplyNav` push 支路接线——超限最旧 entry 出列即 dispose | `part02.js` | spec 无上限（浏览器共识 50）；WPT dispose-for-full-session-history.tentative「entry removed due to too many entries → dispose」 |
| **pushState/replaceState 速率限制**：10s 滑动窗 100 次封顶（`_histRateStamps` 共享窗 + `_histRateAllow` 门，pushState/replaceState 入口前置；超限**静默 no-op** 不派 navigate 无 entry；窗随 `__zw_reset_history` 跨文档重置） | `part02.js` | spec session history push/replace rate limit；WPT history_pushstate/replacestate_too_many_calls.optional「href 不变即通过」 |
| **动态 classic 脚本执行**（append 通道）：① native 绑定侧——`run_prepared_scripts`（连接态判定 + inline 文本 eval + `_zwScriptRan` run-once 标记 + src 型跳过），接线 appendChild/insertBefore/replaceChild/insert_variadic 四站点（fragment 展开候选预收集；innerHTML/insertAdjacentHTML 路径不调——spec 解析注入脚本不执行）；② shim 视图侧——`_appendVariadic`（append/replaceChildren 共享通道）尾部 sel-backed 父的 script 子执行（run-once 同 `_zwRanScripts` R387 表，双通道不重跑） | `dom_bindings/node.rs` + `part05.js` | spec prepare the script element；WPT create-script-set-location「appended script 内 location.href 触发 push navigate」 |
| **traverse 语义面**（批二，2026-10-10）：① navigate() classic 槽 null 化（未带 state 时 `history.state === null` 非 undefined；navState 分槽不动）；② 入队时可达性快照——被 push 截断剪除的 traverse → AbortError 双 reject（navigate 事件未派故 navigateerror 不发），入队即越界维持 InvalidStateError；③ `back({info})`/`forward({info})` 线程 traverse 事件 `e.info`；④ host page_url 同步 setter `__zw_set_page_url`（`_zwSyncDocUrl` 接线，**仅 hash-only 变更**——跨文档内存近似提交被 stop 中止后 filename 须停留载入文档）+ 初始 entry url 补章 `_histStampInitialUrl`（`_navPub` url getter 的 page_url 回退随同步漂移面）+ hash-setter 成功步骤（navigatesuccess 微任务——hash-only 锚点击 `location.hash=` 通道） | `part02.js` + `js_dom_bridge/callbacks.rs` | spec navigate/notify-committed/notify-about-aborting；WPT navigate-history-state ×2 + navigate-intercept-history-state + forward-to-pruned-entry + navigate-navigation-back-same-document + navigation-back-same-document-preventDefault + navigatesuccess-same-document + intercept-handler-throws/-reject/-multiple-times-reject（连带收口） |

## 数字

| corpus 域 | S4O 后 | 本轮（终态） | Δ |
|---|---|---|---|
| navigation-api（NotRun 14 不计分母） | 218/255 = 85.5% | **233/255 = 91.4%** | +15 |
| the-history-interface | 45/49 = 91.8% | **48/49 = 98.0%** | +3 |
| the-location-interface | 41/43 = 95.3% | **42/43 = 97.7%** | +1 |
| 全量 | 358 P / 476 = 75.2% | **368 P / 476 = 77.3%** | +10 |

**18 翻全 Fail/Timeout → Pass，零 Pass 回归**（全量 per-subtest 精确 diff，S4O vs S4P 终态）：
history_pushstate/replacestate_too_many_calls.optional ×2、create-script-set-location、
dispose-for-full-session-history.tentative、navigate-history-state ×2、
navigate-intercept-history-state、forward-to-pruned-entry、
navigate-navigation-back-same-document、navigation-back-same-document-preventDefault、
navigatesuccess-same-document、intercept-handler-throws、intercept-reject、
intercept-multiple-times-reject（后三连带收口——AbortError stack/page_url 同步闭合其
filename 断言）。状态变化非 Pass 面：
replaceState-inside-back-handler-infinite.optional Timeout↔Fail 摆动（速率限制使无限
replaceState 循环封顶后进入断言面——两态均非 Pass）。

**host 交接回归（批二 make test 当场捕获 + 修复）**：`__zw_set_page_url` 同步初版挂在
`_pushHistNav`（锚/href/assign 共享通道）——shim 抢写 page_url 使 host 对 AREA 激活
（`fragment_anchor_updates_history_without_new_document`）视 URL 已达而 no-op 导航
（hash 空/length 1）。收口：`_zwSyncDocUrl(u, syncPageUrl)` 拆门——page_url 同步仅
navigate() API 路径（`_histApplyNav`）开门；锚/href/assign host 交接路径保持 doc 槽
（`_zwFragmentUrl` + live doc url）同步（target-pseudo-after-reinsertion `:target` 面）。

## 挂账（本轮诊断记录）

- **the-history-interface 007/008**：外部脚本基建缺口（007 依赖 `/xhr/resources/delay.py`
  服务端 CGI——静态 runner 不可达；008 依赖跨域宿主脚本注入）——维持 helper infra 挂账。
- **动态 src 型脚本**（native 插入路径）未接 fetch 执行面（shim R387b 通道已盖）——记账。
- replace-before-load 38F / bfcache / 跨文档链维持 M4 定性（B 类，runner 形态/M3 依赖）。

## 质量门禁

- `make test`：全绿（S4P 批一 20,340 P / 0 F；批二终态 20,343 P + 1 已知 load-flake
  `finish_page_load_fires_body_onload_r2946`——S4L/S4M 同案先例，单测与页脚本批复跑即绿）。
- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning。
- `cargo fmt --all -- --check` + `git diff --check`：零 diff。
