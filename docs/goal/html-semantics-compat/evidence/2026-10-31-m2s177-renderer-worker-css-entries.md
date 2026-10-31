# M2-S177 多进程 renderer SetDomSnapshot 臂 entries 化——js worker gCS 文档序交错（s174 挂账收口）

## 入场定刀位

s174 挂账⑤「多进程 renderer js_worker `SetDomSnapshot` 臂 entries 化（低优先——
该臂已有 blob 文档化回落）」。勘根先核多进程链路三层事实：

1. **renderer navigate 路径自取主文档**（`navigate_with` → `AsyncPageLoad::start`
   → webview 侧 `prepare_page_subresources` → `load_html`）——s174 的
   stash→entries 发布在 renderer **pipeline 线程**已通（同一进程内 webview 路径）。
2. **缺口在 js worker 线程**（页面 JS gCS 真源）：runtime sync 咽喉
   （`sync_cached_html_from_webview`，runtime.rs:2498）只推 `wv.cached_css()`
   blob → `SetDomSnapshot` 臂 `set_active_external_css(blob)`——条目丢失，
   worker 侧 gCS 恒 blob 前置形态（link 在 `<style>` 之后的页同特异度恒输）。
3. **LoadHtml IPC 通道不在面内**：caller（headless/Android/embedder）传入的
   `css` 本就只有合并 blob，上游无位次信息可恢复——与嵌入者 custom-css 通道
   同为 s174 已文档化的 blob 回落语义。`JsWorkerCommand` 为进程内 mpsc 通道
   非 IPC——**zero-protocol 契约零改动**。

## 设计

发布形态镜像 + 钉版下发，三段：

1. **webview**（发布者记账）：`load_html` 实际发布形态镜像进
   `published_external_css_entries`（条目臂存条目表、blob 臂清空——每次加载
   无条件覆盖含清空，跨文档残留不可能存活）+ getter
   `external_css_entries()`（与 `cached_css()` 同性质的纯字段读）。
2. **js_worker**（通道扩展）：`SetDomSnapshot` 命令增
   `external_css_entries: Arc<Vec<(String, String)>>`（普通 + 优先两构造点同
   钉版）；worker 槽 `set_external_css_entries`/`pinned_external_css_entries`
   与 blob 槽同语义（不立即下发，快照钉版）；臂分派——**条目非空 →
   `set_active_external_css_entries`**（collector 文档序交错装配；派生 blob
   同步装 blob 槽，回落面不变，blob 字段被覆盖条目为权威源），空表 → blob
   通道（既有回落逐位保留）；`reset_document_state` 镜像清条目槽（P9-9
   DEF-1 同窗：复位后、下一次 sync 覆盖前旧页条目不得进新文档 gCS）。
3. **runtime**（同步咽喉）：`sync_cached_html_from_webview` 同点推入
   `wv.external_css_entries().to_vec()`——与 blob 通道同咽喉同时序。

## 施工（4 文件 +154/−4，rust 全量零 shim）

- `crates/webview/src/webview.rs`：镜像字段 + load_html 双臂记账 + getter。
- `apps/renderer/src/js_worker.rs`：命令字段 + worker 槽/setter/钉版 + 两
  构造点 + 臂分派 + reset 清槽。
- `apps/renderer/src/runtime.rs`：sync 咽喉推入 1 行。
- `crates/webview/src/tests/external_css_cascade_order.rs`：getter 双态钉测 2 案。

## 验证

- **webview lib 832P/0F**（基线 830 + 2 新钉测；`external_css_cascade` 组
  5 s174 案 + 2 s177 案全绿）。
- **renderer lib 354P/5F**：5F 与干净 main（stash A/B）**同键同态全既有**
  ——fetch 契约族 3 案（`renderer_js_worker_fetch_*`，`left: "undefined"`
  签名，与 engine fetch 族 9F 同族）+ page_scripts 2 案（form fixture
  `left: String("")` / `has_attribute checked`，测试体自 2026-08
  8cb3e919a 后未动，失败时点不明——挂账勘定，非 s177 面）。s177 worker
  钉测 `renderer_js_worker_external_css_entries_document_order_s177` Pass。
- **s177 worker 钉测判别断言**：blob 字段与条目文本刻意分歧（BlobFont vs
  GithubMono）——臂若退化回 blob 发布（整体前置）BlobFont 反超文档表断言
  即红；实测 `"Arial, serif"`（文档 `<style>` 后到后胜）= 条目权威 +
  blob 字段被忽略双钉。复位相：锚失配条目尾接（collector 回落）使残留
  显形为 GithubMono，清空显形为 `""`。
- **interactive-elements 483 键对 s176 终态日志逐键 bit-exact 零位移**
  （/tmp/s176/interactive_final2.log join common 483/483；对 s172 旧基线的
  4 位移恰为 s176 在册 4 项——F2P 2 = s173 双靶承接、P2F 2 = shadow-slots
  假 Pass 清除，非本轮）。runner 路径不经 js_worker；webview 改动仅记录
  镜像零行为面——零位移与预测一致。
- **engine lib 免跑**：engine 代码零 diff（entries 槽/collector 语义 s174
  已验，本轮仅新增调用方；webview lib 全量承载 crate 面回归）。
- fmt / guarded clippy `--all-targets`（zero-webview + zero-renderer）零告警。

## 归因与边界

- 生产增益面 = 多进程 ZeroBrowser 页面 JS 的 gCS 文档序交错（link-after-style
  页同特异度胜者由 blob 前置恒输纠正为文档序后到后胜）；单进程嵌入路径
  s174 已通，本轮补齐 worker 线程同语义。
- LoadHtml IPC 通道维持 blob 回落（caller 无位次信息，非本轮可恢复面）；
  若未来需要，须扩 zero-protocol `LoadHtmlParams` 契约并同步两端（架构
  边界规则），当前无语料需求不扩。

## FIXME 新记 / 挂账

- **renderer lib 首立全量基线 354P/5F**：page_scripts 2 案（
  `form_interaction_fixture_complete_sequence` /
  `javascript_disabled_skips_listeners_not_default_actions`）失败时点待勘
  ——测试体最后变更 8cb3e919a（2026-08-13 action transaction 系），归因
  候选该系后续演进；renderer 属本流面，入场顺序在基线翻拣候选后排定。
- 行数欠账在册（非本刀扩）：js_worker.rs 6.9k / runtime.rs 5.4k /
  webview.rs 9.2k 行均超 2000 行准则线——拆分属行数治理专项（selector_match
  先例），本轮新增 +30 行内不做顺带拆分。

## 下一步候选（本流）

基线翻拣下一单根面：forms 全树 402F 家族分析，或 interactive 域 44F 归因
复读，或 renderer page_scripts 2 案勘定（上条新记）。

## 中间产物

/tmp/s177/：webview_lib.log、renderer_lib_full.log、renderer_baseline_check.log
（stash 对照）、interactive_s177.log、new.tsv/s176.tsv（join 输入）。
