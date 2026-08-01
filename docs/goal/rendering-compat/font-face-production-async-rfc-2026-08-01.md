# Spec + RFC：生产环境 @font-face 异步加载（live browser）

**版本**：v1.0
**日期**：2026-08-01
**作者**：Rally 执行 agent
**状态**：草稿（rally 自主推进模式，无人工确认环节；见 §6.3 决策 7）
**关联**：R2406 重大发现（`docs/goal/rendering-compat/master.md` 顶部裁决块）、`docs/goal/rendering-compat.md` DC-13 / DC-5（css-fonts）、run-rules §无人值守/本地 web server。

---

## 0. 执行摘要

- **一句话目标**：修复生产环境（live browser / renderer 进程 + tabworker 进程内路径）`@font-face` 字体字节被 fetch 后**丢弃**（仅 log）的严重 bug，使页面声明的自定义字体真正生效，而非回退到 fallback 字体。
- **本期范围**：① 提取器 pub 化 + 单测（slice 1）；② `AsyncPageLoad` family 跟踪 + drain 出口 + renderer/tabworker 接通 load/register/resolver/re-render + env kill-switch（slice 2）；③ live-browser 本地 web server @font-face 页验证 + A/B（slice 3）。整体跨 session 推进，本 session 仅 land RFC + slice 1。
- **明确排除**：font-weight 多 face 精细化匹配（R2405 已 gated，单列）；font-stack C-dep rebuild（用户护栏，等点名）；WOFF2 解码（fontdue 不支持，沿用现状静默失败跳过）；`local()` src 解析（生产无本地字体注册表，本期跳过，仅处理 `url()`）。
- **核心约束**：A/B 零回归（welcome/legacy smoke 不退化）；env kill-switch 默认开启但可一键关；不破坏 harness reftest 路径（`load_font_faces_into` 不动）；`make test` / `make reftest` / clippy 干净。
- **推荐方案**：**drain pattern**——`AsyncPageLoad` 在 tick 内收集已就绪 `(family, bytes)`，tick 结束后由宿主（持 `FontLoader` + `WebView`）drain → `load_font` + `register_family_alias` → `set_font_resolver(build_font_resolver())` → 触发重绘。**不**改 `AsyncFetchHost` trait（详见 §8.6 拒绝理由：renderer 内 `font_loader` 在 tick 期间被 `with_measure_ctx_opt` 不可变借用做文本度量，callback 在 tick 内 mutate 会别名冲突）。
- **首个落地步骤**：slice 1——在 `crates/engine/src/pipeline.rs` 新增 `pub fn extract_font_faces(css) -> Vec<(String, Vec<String>)>`（family-preserving，复用 css-parser `FontFaceRule`），补单测，zero 生产行为变更。

---

## 1. 背景与目标

### 1.1 背景

R2406 审计发现：live browser 的 `@font-face` **完全失效**。链路核查：

- **`AsyncPageLoad::poll_fonts`**（`crates/webview/src/async_load.rs:365-378`）：`fetch_bytes_meta` 取到 @font-face 字节后，仅 `tracing::info!("font fetched")` 即**丢弃**——无 `load_font`、无 `register_family_alias`。
- **`begin_font_fetch`**（`async_load.rs:349-363`）：用 `extract_font_face_urls(&self.css)`（`crates/engine/src/pipeline.rs:1107`，URL-only 扫描），**丢失 family**；且仅扫 `self.css`（外链样式表），**不扫 inline `<style>`**（对照 `begin_image_fetch` 在 `async_load.rs:414-416` 合并了 inline style），故 inline `<style>` 内的 @font-face 也不被抓取——**次要 bug**。
- **生产宿主**：默认多进程后端（`apps/browser/src/process_backend.rs:24` `use_multiprocess_backend` 默认 true）→ renderer 进程（`apps/renderer/src/main.rs`）。renderer `with_io`（main.rs:153-180）持 `font_loader` + `WebView`，已 `set_font_resolver`（main.rs:170）。renderer tick（main.rs:361-376）以 `IpcAsyncFetchHost` / `StubAsyncFetchHost` 驱动 `pending.load.tick(webview, &mut host, budget_ms)`。tabworker（`apps/browser/src/tab_worker.rs:131-288`）为进程内备选路径，同样持 `font_loader` + `WebView` + `AsyncPageLoad`。
- **关键借用事实**：renderer tick 在 `text_metrics::with_measure_ctx_opt(font_loader, ...)`（main.rs:362-363）内**不可变借 `&self.font_loader`**（设 thread-local `*const FontLoader` 供布局文本度量读取），webview 为 `&mut`。tick 期间 `font_loader` 不可 mut。→ 字体 mutate 必须在 **tick 之后**。
- **为何 product-smoke 未抓**：product-smoke / reftest 经 `zero-wpt-runner` 走 harness 路径 `load_font_faces_into`（`tests/wpt-runner/src/reftest/reftest_fonts.rs:146`，**会** load+register），与 live browser async 路径**分叉**——harness 加载 @font-face，production 不加载。故此 bug 是 product-smoke 的**盲区**，必须新增 live-browser 验证（slice 3）。
- **既有能力**：`FontLoader`（`crates/render-foundation/src/font/loader.rs`）已具备 `load_font(&[u8]) -> Result<u32>`、`register_family_alias(&str, u32)`、`build_font_resolver() -> HashMap<String,u32>`；`WebView::set_font_resolver(HashMap)`（`crates/webview/src/webview.rs:890`）已存在；harness 已证 load+register+resolver 链路可用。本期**纯接线**，不新增字体能力。

### 1.2 目标

- **业务目标**：live browser 中任何 `@font-face` 页面（morning-work JetBrainsMono、Google Fonts、内网自定义字体等）以**声明字体**渲染，而非 fallback。
- **用户目标**：浏览器对自定义字体的支持对齐主流浏览器（Chromium）基础行为。

### 1.3 范围边界

- **在范围内**：
  - family-preserving 提取器 pub 化（slice 1）。
  - `AsyncPageLoad` 字体 pending 改跟踪 `(family, url, rx)` + 已就绪 `(family, bytes)` 收集 + drain 出口（slice 2）。
  - `begin_font_fetch` 扫描合并 CSS（外链 + inline `<style>`），对齐 `begin_image_fetch`（slice 2，修复次要 bug）。
  - renderer 进程 + tabworker 进程内路径：drain → load/register/set_resolver/re-render + env kill-switch（slice 2）。
  - live-browser 本地 rust web server @font-face 验证 + A/B（slice 3）。
- **不在范围内**：
  - font-weight 多 face 精细化（R2405，单列）。
  - font-stack C-dep rebuild（用户护栏，等点名）。
  - WOFF2 解码、`local()` src 解析、`unicode-range` 子集化、`font-display` 调度策略。
  - harness reftest 路径改动（`load_font_faces_into` 保持原样；可选去重见 §7，非必须）。

---

## 2. 需求类型概览

| 类型 | 是否适用 | 来源 |
|------|---------|------|
| 业务需求 | 是 | R2406 发现：live browser @font-face 失效 |
| 用户需求 | 是 | 自定义字体生效，对齐 Chromium |
| 解决方案需求 | 是 | drain pattern 接线现有 FontLoader/WebView 能力 |
| 功能需求 | 是 | §3 |
| 非功能需求 | 是 | §4（零回归 / kill-switch / 不强依赖外网） |
| 接口需求 | 是 | §5（drain API + env 开关） |
| 过渡需求 | 是 | slice 1→3 多 session 渐进，每 slice 可独立 land + 回滚 |

---

## 3. 功能需求

### FR-001：family-preserving 提取器 pub 化（slice 1）
- **描述**：系统必须在 `crates/engine` 提供 `pub fn extract_font_faces(css: &str) -> Vec<(String, Vec<String>)>`，返回每个 @font-face 的 `(family, url_sources)`，family 已去引号、sources 已去 `url()` 包裹与引号、按出现顺序。行为对齐既有 harness 私有 `extract_font_faces`（`reftest_fonts.rs:86`）与 css-parser `FontFaceRule { family, sources }`（`crates/css-parser/src/ast.rs:85`）。
- **优先级**：必须
- **来源**：slice 1 计划 / R2406 fix scope ①

**验收场景**：

```
场景: 多 @font-face 提取 family + sources
  假设 CSS 含 `@font-face { font-family: "JetBrains Mono"; src: url(jb.woff2) format("woff2"), url(jb.ttf); }` 与另一 family "Title"
  当 调用 extract_font_faces(css)
  那么 返回 [(family="JetBrains Mono"(去引号), sources=["jb.woff2","jb.ttf"]), (family="Title", sources=[...])]
  验证: crates/engine 单测 test_extract_font_faces_family_and_sources

场景: 无 @font-face 返回空
  假设 CSS 仅含普通规则，无 @font-face
  当 调用 extract_font_faces(css)
  那么 返回空 Vec
  验证: test_extract_font_faces_empty

场景: data: URI 与 local() src 处理一致
  假设 CSS 含 `src: local(Foo), url(data:...), url(real.woff)`
  当 调用 extract_font_faces(css)
  那么 sources 仅含 css-parser 解析出的 url() 项（data:/local 行为与 FontFaceRule 一致，不额外过滤——过滤在抓取层 FR-002 处理）
  验证: test_extract_font_faces_src_passthrough（断言与 css-parser 直接解析结果一致）
```

### FR-002：AsyncPageLoad family 跟踪 + 合并 CSS 扫描（slice 2）
- **描述**：`begin_font_fetch` 必须用 `extract_font_faces`（family-preserving）替代 `extract_font_face_urls`，扫描**合并 CSS**（`self.css` 外链 + inline `<style>` 经 `extract_html_style_text`，对齐 `begin_image_fetch`）；`font_pending` 改跟踪 `(family, abs_url, rx)`；过滤 `data:`（已解码则不入 fetch，与图片路径一致）与 `local()`（本期不解析）。
- **优先级**：必须
- **来源**：R2406 fix scope ② / 次要 bug（inline @font-face 漏抓）

**验收场景**：

```
场景: 外链 CSS @font-face 抓取带 family
  假设 self.css 含 `@font-face { font-family: X; src: url(x.woff) }`
  当 begin_font_fetch 发起抓取
  那么 font_pending 含 ("X", <abs url>, rx)；fetch 以 ResourceFetchMeta::FONT 发起
  验证: crates/webview async_load 单测 test_begin_font_fetch_tracks_family

场景: inline <style> 内 @font-face 被抓取（修复次要 bug）
  假设 页面 HTML 含 `<style>@font-face{font-family:Y;src:url(y.woff)}</style>`，self.css 为空
  当 begin_font_fetch 扫描合并 CSS
  那么 font_pending 含 ("Y", <abs url>, rx)
  验证: test_begin_font_fetch_inline_style

场景: data: URI src 不抓取
  假设 @font-face src 为 `url(data:application/font-woff;base64,...)`
  当 begin_font_fetch 处理
  那么 不对该 data: 发起 fetch（与图片 data: 路径一致）
  验证: test_begin_font_fetch_skips_data_uri
```

### FR-003：drain 出口——已就绪字体字节回传宿主（slice 2）
- **描述**：`AsyncPageLoad::poll_fonts` 必须把 fetch 成功的 `(family, bytes)` 收集到内部字段（不再丢弃），并提供 `pub fn drain_loaded_fonts(&mut self) -> Vec<(String, Vec<u8>)>` 供宿主在 tick 之后取出并清空。fetch 失败仅 log，不影响其他 pending。
- **优先级**：必须
- **来源**：R2406 fix scope（drain pattern）

**验收场景**：

```
场景: 成功 fetch 字节经 drain 回传
  假设 MockFetchHost 对 font url 返回 Ok(<font bytes>)，font_pending 含 ("X", url, rx)
  当 tick 至 poll_fonts 收到字节，随后调用 drain_loaded_fonts()
  那么 返回 [("X", <font bytes>)]，再次 drain 返回空
  验证: test_drain_loaded_fonts_returns_bytes_then_clears

场景: fetch 失败不污染 drain
  假设 MockFetchHost 对 font url 返回 Err("timeout")
  当 tick 后 drain_loaded_fonts()
  那么 返回空 Vec（失败仅 log，*changed 仍置 true 以推进阶段）
  验证: test_drain_loaded_fonts_failure_empty

场景: 字节不再被丢弃（regression guard）
  假设 同成功场景
  当 调用 drain_loaded_fonts()
  那么 返回的 bytes.len() 与 fetch 返回一致（非空、非丢弃）
  验证: test_drain_loaded_fonts_not_discarded
```

### FR-004：宿主接通 load/register/resolver/re-render（slice 2）
- **描述**：renderer 进程 tick 循环与 tabworker 进程内 tick 循环，在 `load.tick(...)` 返回后，对 `drain_loaded_fonts()` 的每项执行 `font_loader.load_font(&bytes)` → 成功则 `register_family_alias(&family, id)`（Ahem 由 load_font 内部 ahem 检测处理，与 harness 一致）；若有任何加载成功，调用 `webview.set_font_resolver(font_loader.build_font_resolver())` 并置重绘标记。env `ZW_LIVE_FONTFACE=0` 时跳过整段（kill-switch，默认开启）。
- **优先级**：必须
- **来源**：R2406 fix scope ④⑤

**验收场景**：

```
场景: 声明字体加载并更新 resolver
  假设 drain 返回 [("JetBrains Mono", <valid ttf bytes>]，env ZW_LIVE_FONTFACE 未设（默认开）
  当 宿主 drain 处理
  那么 font_loader 含该字体 id 且 register_family_alias("JetBrains Mono", id) 已注册；webview.set_font_resolver 被以含 "JetBrains Mono" 的 map 调用；重绘标记置位
  验证: apps/renderer / apps/browser 单测（mock font_loader 或集成断言 resolver 含 family）

场景: kill-switch 关闭跳过
  假设 env ZW_LIVE_FONTFACE=0
  当 宿主 drain 处理
  那么 不调用 load_font / set_font_resolver（字节被忽略，行为退回 R2406 前的丢弃——但明确是 opt-out，非 bug）
  验证: test_killswitch_disables_live_fontface

场景: 无效字体字节不崩溃
  假设 drain 返回 [("X", <garbage bytes>]
  当 宿主 drain 处理
  那么 load_font 返回 Err 被 log 吞掉，不 panic，不更新 resolver
  验证: test_invalid_font_bytes_swallowed
```

### FR-005：live-browser 本地 @font-face 验证 + A/B（slice 3）
- **描述**：新增 rust 本地 web server fixture（run-rules 许，不强依赖外网），serve 一个声明 @font-face（指向本地字体文件）+ 含使用该 family 的可见文本的页面；启动 live browser（renderer 进程路径）加载该页面，截图断言：渲染像素与「使用声明字体」的参考一致，与「使用 fallback 字体」的参考**不一致**。A/B = 同 fixture 在 kill-switch 开/关下截图对比，验证差异符合预期。
- **优先级**：应该（验证项；非 CI 强制门，因 CI 难跑 live browser）
- **来源**：R2406 fix scope 验证 / run-rules 本地 web server

**验收场景**：

```
场景: 声明字体在 live browser 生效
  假设 本地 server serve @font-face 页（family=TestFont）+ 文本 "ABC"
  当 live browser 加载并渲染
  那么 截图 glyph 与 TestFont 参考一致，≠ fallback 字体参考
  验证: slice 3 验证脚本（人工/本地跑，结果记 evidence/）

场景: A/B 差异确认
  假设 同 fixture
  当 分别以 ZW_LIVE_FONTFACE=1 与 =0 渲染
  那么 两截图存在可测差异（开启=声明字体，关闭=fallback）
  验证: slice 3 A/B 脚本 → evidence/font-face-live-ab-<date>.md
```

---

## 4. 非功能需求

### NFR-001：零回归
- **描述**：welcome product-smoke（diff% 与 struct）与 legacy smoke（struct FAIL 数 / avg%）与 R2406 前持平或更优；全量 `make test` / `make reftest` 零新失败。
- **测量标准**：`make product-smoke` + `make product-smoke-legacy` + `make test` + `make reftest`（scoped 必要时）。
- **优先级**：必须

### NFR-002：kill-switch 可一键关
- **描述**：env `ZW_LIVE_FONTFACE=0` 完全禁用本特性，行为等价 R2406 前（除 fetch 仍发起但字节丢弃）。
- **测量标准**：单测断言 env=0 时 drain 处理为 no-op。
- **优先级**：必须

### NFR-003：不强依赖外网
- **描述**：所有验证用本地 rust web server + 本地字体文件，不依赖 Google Fonts 等外网（run-rules）。
- **测量标准**：slice 3 fixture 资源在仓内或可本地生成。
- **优先级**：必须

### NFR-004：文件大小
- **描述**：被改文件不超 2000 行（run-rules §5）；`async_load.rs` 现 725 行，预计 +60 行内；renderer main.rs 接近阈值需核查（slice 2 时确认）。
- **测量标准**：`wc -l` 核查。
- **优先级**：应该

---

## 5. 接口需求

### IF-001：`extract_font_faces`（engine 公开函数）
- **类型**：API（仓内库函数）
- **规格**：`pub fn extract_font_faces(css: &str) -> Vec<(String, Vec<String>)>`，位于 `crates/engine/src/pipeline.rs`（与 `extract_font_face_urls` 同位）。实现：经 `zero_css_parser::parser::Parser::parse_stylesheet` 收集 `Rule::FontFace(FontFaceRule)` → `(ff.family, ff.sources)`。无 panic（解析失败返回空）。
- **错误处理**：解析失败 → 空 Vec（与既有 harness 行为一致）。
- **默认动作**：不适用（纯函数）。
- **交叉引用**：实现来源见 §6.5A；消费方 FR-002。

### IF-002：`AsyncPageLoad::drain_loaded_fonts`（webview 公开方法）
- **类型**：API（结构体方法）
- **规格**：`pub fn drain_loaded_fonts(&mut self) -> Vec<(String, Vec<u8>)>`。取出并清空内部 `font_loaded` 缓冲。无数据时返回空 Vec。
- **错误处理**：无（字节解码失败由宿主 load_font 处理）。
- **默认动作**：不适用。
- **交叉引用**：FR-003；宿主消费见 RFC §8.4。

### IF-003：env kill-switch `ZW_LIVE_FONTFACE`
- **类型**：系统接口（环境变量）
- **规格**：`ZW_LIVE_FONTFACE=0` → 宿主跳过 drain 处理（不 load/register/set_resolver）；未设或非 `0`/`false` → 启用（默认）。读取方式对齐既有 env 模式（如 `use_multiprocess_backend`、`ZW_PERFONT_LINEHEIGHT`）。
- **错误处理**：非法值按「未设」处理（默认启用）。
- **默认动作**：未设 = 启用。
- **交叉引用**：FR-004、NFR-002。

---

## 6. 约束与假设

### 6.1 必须约束（Must）
- 字体 mutate（load_font/register）发生在 `AsyncPageLoad::tick` 返回**之后**（renderer 内 `font_loader` 在 tick 期间被 `with_measure_ctx_opt` 不可变借用）。
- 本特性默认启用，但可经 `ZW_LIVE_FONTFACE=0` 完全关闭并退回 R2406 前行为。
- harness reftest 路径（`load_font_faces_into`）**不得**因本改动而行为变化（reftest 通过率不退化）。
- 提交前走 `make test` / `make reftest`（test-guard 包裹）/ `cargo fmt` / `cargo clippy --workspace --all-targets -- -D warnings`。

### 6.2 禁止约束（Must Not）
- **不**改 `AsyncFetchHost` trait（drain pattern 替代 callback；理由 §8.6）。
- **不**新增第三方 crate（复用 fontdue + 现有 FontLoader）。
- **不**在 tick 内 mutate `font_loader`（借用冲突 / 潜在别名 UB）。
- **不**删除既有 `extract_font_face_urls` 的对外语义而不自清理——若 begin_font_fetch 切换后它变 dead code，按 code-guidelines §3 删除（自产遗留），并在 commit 注明。
- **不**把未验证的 live-browser 行为写成 CI 强制门（CI 难跑 live browser；slice 3 为本地验证 + evidence）。

### 6.3 已定决策
1. **drain pattern**（非 callback）——决定性理由：renderer tick 内 `&self.font_loader` 不可变借用做文本度量，callback 在 tick 内 mutate 会别名冲突（编译失败或 UB）。drain 把 mutate 推迟到 tick 后，借用安全。
2. **提取器放 engine pipeline.rs**（非 webview）——与 `extract_font_face_urls` 同位，webview 已从 engine 导入同类函数。
3. **合并 CSS 扫描**（外链 + inline `<style>`）——对齐 `begin_image_fetch`，修次要 bug。
4. **kill-switch env 名 `ZW_LIVE_FONTFACE`**——对齐既有 `ZW_*` 命名（`ZW_PERFONT_LINEHEIGHT`）。
5. **多进程 renderer 为主路径**，tabworker 进程内路径同步接通（双路径一致）。
6. **WOFF2 / local() 本期不做**——fontdue 不支持 WOFF2；local() 需本地字体注册表。
7. **rally 自主推进，无人工确认**——本 goal 处于「永不停 / 主做轻量修复」裁决下，且本任务为 R2406 明确授权的最高价值 live-browser 缺口；RFC 落地后直接进入 slice 实施，不走 spec-rfc 的「等用户确认」环节（rally 输出协议禁止向用户提问）。

### 6.4 技术约束
- Rust edition 2024，MSRV 1.85；`#![warn(missing_docs)]` 生效（新增 pub 项需 `///` doc）。
- 单文件 ≤ 2000 行（run-rules §5）。
- 资源抓取经 webview `net_pool`（InProcessFetchHost）或 renderer IPC（IpcAsyncFetchHost）；不强依赖外网。

### 6.5 假设
- `FontFaceRule { family, sources }` 的 `sources` 已由 css-parser 去除 `url()` 包裹与引号（已验证：`ast.rs:88` 注释 + `tests_10.rs:361` 断言 `sources == ["test.woff"]`）— 状态：已验证。
- renderer 默认路径为多进程（`use_multiprocess_backend` 默认 true）— 状态：已验证（`process_backend.rs:24`）。
- `FontLoader::load_font` 对 WOFF1 解码、ttf/otf 直接加载、WOFF2 静默失败 — 状态：已验证（`loader.rs:161` + 注释）。
- tabworker（进程内）在 `ZERO_BROWSER_MULTIPROCESS=0` 下为活跃路径 — 状态：已验证（`process_backend.rs:24`）。

### 6.5A 实现来源说明

| 能力/行为 | 来源类型 | 具体来源 | 备注 |
|----------|----------|----------|------|
| family-preserving 提取 | 复用现有模块 | `zero_css_parser` `FontFaceRule` + `Parser::parse_stylesheet`（`crates/css-parser/src/parser.rs:359`、`ast.rs:85`） | 行为对齐 harness `extract_font_faces`（`reftest_fonts.rs:86`） |
| 字体字节加载 | 复用现有模块 | `FontLoader::load_font`（`render-foundation/src/font/loader.rs:161`） | WOFF1 自动解码，WOFF2 失败跳过 |
| family 别名注册 | 复用现有模块 | `FontLoader::register_family_alias`（`loader.rs:196`） | — |
| resolver 重建 | 复用现有模块 | `FontLoader::build_font_resolver`（`loader.rs:219`） | — |
| resolver 注入 webview | 复用现有模块 | `WebView::set_font_resolver`（`webview/src/webview.rs:890`） | — |
| inline `<style>` 文本提取 | 复用现有模块 | `zero_engine::extract_html_style_text`（async_load 已导入） | 对齐 begin_image_fetch |
| env kill-switch 模式 | 复用现有模式 | 对齐 `use_multiprocess_backend` / `ZW_PERFONT_LINEHEIGHT` | 新读 `ZW_LIVE_FONTFACE` |

### 6.6 代码变更边界
- **允许修改**：
  - `crates/engine/src/pipeline.rs`（新增 `extract_font_faces` pub fn + 测试；可选删 dead `extract_font_face_urls`）。
  - `crates/webview/src/async_load.rs`（font_pending family 跟踪 + 合并 CSS + drain API + poll_fonts 收集 + 测试）。
  - `apps/renderer/src/main.rs`（tick 后 drain → load/register/set_resolver/re-render + env 开关）。
  - `apps/browser/src/tab_worker.rs`（同上，进程内路径）。
  - 可选：`apps/browser/src/app_platform.rs` 或新增 env 读取 helper（若需集中读取 env）。
- **禁止修改**：
  - `tests/wpt-runner/src/reftest/reftest_fonts.rs` 的 `load_font_faces_into` / `extract_font_faces` 行为（harness 路径，reftest 可信度依赖；可选去重在 §7 标注为可选非必须）。
  - `crates/render-foundation/src/font/loader.rs`（FontLoader API 本期不改）。
  - `crates/page-runtime/src/lib.rs` 的 `AsyncFetchHost` trait（drain 替代 callback）。

### 6.7 执行技能提示
| 范围 / 触发条件 | Skill | 模式 | 原因 |
|----------------|-------|------|------|
| 提交前 | `lei-pre-commit-guard` | required | 仓 CLAUDE.md 强制门禁 |
| 测试/reftest 运行 | （make test/make reftest via test-guard） | required | run-rules 强制包裹器 |
| 渲染变更回归 | `make product-smoke` / `make product-smoke-legacy` | preferred | run-rules 渲染变更建议 |

---

## 7. 优先级与里程碑建议

| ID | 需求 | 优先级 | 理由 | 里程碑 |
|----|------|--------|------|--------|
| FR-001 | 提取器 pub + 单测 | 必须 | slice 1，零行为变更，奠基 | M1 |
| FR-002 | family 跟踪 + 合并 CSS | 必须 | 修丢失 family + inline 漏抓 | M2 |
| FR-003 | drain 出口 | 必须 | 字节不再丢弃 | M2 |
| FR-004 | 宿主接通 + kill-switch | 必须 | 真正生效 | M2 |
| FR-005 | live-browser 验证 + A/B | 应该 | product-smoke 盲区，须新验证 | M3 |
| NFR-001~004 | 零回归 / kill-switch / 无外网 / 文件大小 | 必须/应该 | 门禁 | 各 M |

### 建议里程碑（多 session）
- **M1（slice 1，本 session）**：`extract_font_faces` pub + 单测；零生产变更；land。
- **M2（slice 2，后续 session）**：AsyncPageLoad family 跟踪 + 合并 CSS + drain + renderer/tabworker 接通 + env kill-switch；`make test`/`make reftest`/product-smoke A/B 零回归；land。
- **M3（slice 3，后续 session）**：本地 rust web server @font-face fixture + live-browser 截图验证 + A/B；evidence 落盘。

### 实施交接（Implementation Handoff）

#### 文件/模块清单

| 路径/模块 | 动作 | 目的 | 风险/注意事项 |
|----------|------|------|---------------|
| `crates/engine/src/pipeline.rs` | 新增 pub fn `extract_font_faces` + 测试 | slice 1 奠基 | 补 `///` doc 满足 missing_docs；可选删 dead `extract_font_face_urls` |
| `crates/webview/src/async_load.rs` | 改 font_pending 类型 + begin_font_fetch 合并 CSS + poll_fonts 收集 + 新增 drain_loaded_fonts + 测试 | slice 2 核心 | 文件 725 行，+60 内；MockFetchHost 已有 |
| `apps/renderer/src/main.rs` | tick 后 drain → load/register/set_resolver + 重绘 + env 开关 | slice 2 主路径 | main.rs 近阈值，slice 2 核查行数；借用：drain 在 `with_measure_ctx_opt` 闭包**外** |
| `apps/browser/src/tab_worker.rs` | 同 renderer 接通 | slice 2 进程内路径 | tick 闭包（line 287-288）外 drain |
| env 读取 helper（如需） | 可选新增或复用 | kill-switch | 对齐既有 env 读取模式 |

#### 职责映射

| 模块/文件 | 职责 | 依赖/被依赖 | 验证方式 |
|----------|------|------------|----------|
| engine `extract_font_faces` | CSS → (family, sources) | 依赖 css-parser；被 webview async_load + (可选) harness | engine 单测 |
| webview `AsyncPageLoad` | 抓取 + 收集 + drain | 依赖 engine 提取器 + page-runtime host；被 renderer/tabworker | async_load 单测（MockFetchHost） |
| renderer / tabworker | drain 消费 → FontLoader + WebView | 依赖 webview + render-foundation | 集成断言 resolver 含 family；slice 3 live 验证 |

#### 新能力来源对照

| 能力/需求 | 实现承载位置 | 来源类型 | 验证方式 |
|----------|--------------|----------|----------|
| family-preserving 提取 | engine pipeline.rs `extract_font_faces` | 复用 css-parser | 单测 |
| 字节→字体注册 | renderer/tabworker drain 处理 | 复用 FontLoader | 单测 + slice 3 live |
| kill-switch | env 读取 | 复用 env 模式 | 单测 env=0 no-op |

#### 推荐修改顺序
1. **slice 1**：engine `extract_font_faces` pub + 单测 → `make test -p zero-engine` + clippy → land。（本 session）
2. **slice 2-a**：webview async_load（font_pending family + 合并 CSS + poll_fonts 收集 + drain API + 单测）。
3. **slice 2-b**：renderer main.rs + tab_worker 接通 + env kill-switch → `make test` + `make reftest` + product-smoke A/B → land。
4. **slice 3**：本地 web server fixture + live 验证 + A/B → evidence。

#### 首批提交建议

| 提交/批次 | 范围 | 预期结果 | 验证 |
|----------|------|----------|------|
| Commit 1 (R2407) | 本 RFC 文档 | 设计落盘 | 文档 lint 自洽 |
| Commit 2 (R2408, slice 1) | engine `extract_font_faces` pub + 单测 | `make test -p zero-engine` 全绿；零生产变更 | engine 单测 + workspace clippy |
| Commit 3 (R2409+, slice 2) | async_load + renderer + tabworker | live @font-face 生效；A/B 零回归 | make test + make reftest + product-smoke |
| Commit 4 (slice 3) | live 验证 + evidence | 截图证声明字体生效 | evidence 落盘 |

---

## 8. 技术设计（RFC）

### 8.1 现状分析
- **当前架构**：`AsyncPageLoad` 分阶段抓取 document → stylesheets → fonts/images → lazy images。`poll_fonts` 取字节后丢弃。生产宿主（renderer 多进程 / tabworker 进程内）持 `FontLoader` + `WebView`（已 `set_font_resolver`）。
- **问题/痛点**：① @font-face 字节丢弃（R2406 主 bug）；② family 丢失（URL-only 提取）；③ inline `<style>` @font-face 漏抓（次要 bug）。
- **相关代码**：`async_load.rs:349-378`、`pipeline.rs:1107`、`reftest_fonts.rs:86,146`、`apps/renderer/src/main.rs:153-180,361-376`、`apps/browser/src/tab_worker.rs:131,287-288`。

### 8.2 目标状态
- `AsyncPageLoad` 用 family-preserving 提取器抓 @font-face（含 inline `<style>`），收集已就绪 `(family, bytes)`，经 `drain_loaded_fonts()` 回传宿主。
- 宿主 tick 后 drain → load/register/set_resolver/re-render，env-gated。
- 提议架构（drain pattern）数据流：

```
            tick(webview, host)                         [font_loader & 借用中，不可 mutate]
AsyncPageLoad: poll_fonts 收集 (family,bytes) → font_loaded buf
            ↓ tick 返回
宿主: drain_loaded_fonts() → [(family,bytes)]          [font_loader 可 &mut]
      → font_loader.load_font(bytes)? → register_family_alias(family,id)
      → webview.set_font_resolver(font_loader.build_font_resolver())
      → 置重绘标记 → 下一帧用声明字体
```

### 8.3 影响范围分析
| 影响项 | 影响程度 | 说明 |
|--------|----------|------|
| live browser @font-face 页 | 高（正向） | 声明字体生效 |
| renderer 进程 / tabworker | 中 | tick 循环 +drain 处理 |
| webview async_load | 中 | font_pending 类型 + drain API |
| engine pipeline | 低 | +1 pub fn |
| harness reftest | 无 | 路径不动 |
| product-smoke | 无（盲区） | 走 harness，不覆盖 live 路径 |

### 8.4 详细设计

**slice 1 — `extract_font_faces`（engine pub fn）**

```rust
/// 从 CSS 文本提取所有 `@font-face` 规则的 `(family, url_sources)` 列表。
///
/// family 已去引号；sources 为 css-parser 解析出的 url() 项（已去包裹/引号），按出现顺序。
/// 解析失败或无规则返回空。供生产 async 路径（保留 family）与抓取层使用。
pub fn extract_font_faces(css: &str) -> Vec<(String, Vec<String>)> {
    use zero_css_parser::parser::Parser as CssParser;
    use zero_css_parser::ast::Rule as CssRule;
    CssParser::parse_stylesheet(css).rules.iter().filter_map(|r| match r {
        CssRule::FontFace(ff) => Some((ff.family.clone(), ff.sources.clone())),
        _ => None,
    }).collect()
}
```
（实现来源：css-parser `FontFaceRule`；行为对齐 harness `extract_font_faces`。）

**slice 2 — `AsyncPageLoad` 改造**

```text
field: font_pending: Vec<(String, String, BytesFetchRx)>   // (family, abs_url, rx)
field: font_loaded: Vec<(String, Vec<u8>)>                  // 收集区（新增）

begin_font_fetch:
  css = self.css + "\n" + extract_html_style_text(html)     // 合并 inline <style>（对齐 begin_image_fetch）
  for (family, sources) in extract_font_faces(&css):
    for src in sources:
      if src.starts_with("data:") { continue }              // 不抓 data:（与图片一致）
      abs = base.join(src)
      font_pending.push((family, abs, host.fetch_bytes_meta(abs, FONT)))

poll_fonts:
  retain (family,url,rx): on Ok(bytes) => font_loaded.push((family,bytes)); *changed=true; false
                          on Err(e) => log; *changed=true; false

pub fn drain_loaded_fonts(&mut self) -> Vec<(String, Vec<u8>)> {
    std::mem::take(&mut self.font_loaded)
}
```

**slice 2 — 宿主接通（renderer 伪代码，tabworker 同构）**

```text
// main.rs tick 循环（with_measure_ctx_opt 闭包返回 changed 之后）：
let loaded = pending.load.drain_loaded_fonts();
if live_fontface_enabled() && !loaded.is_empty() {        // env ZW_LIVE_FONTFACE != "0"
    let mut updated = false;
    for (family, bytes) in loaded {
        if let Ok(id) = self.font_loader.load_font(&bytes) {
            self.font_loader.register_family_alias(&family, id);
            updated = true;
        } else { tracing::warn!(family, "live @font-face load failed"); }
    }
    if updated {
        self.webview.as_mut().expect("wv").set_font_resolver(self.font_loader.build_font_resolver());
        // 触发重绘：置 pending 重绘标记 / advance_render，沿用既有 changed → publish 路径
    }
}
```

> 借用安全：drain 在 `with_measure_ctx_opt` 闭包**外**调用，此时 `self.font_loader` 可 `&mut`，`self.webview` 可 `&mut`，二者不同字段可共存。

### 8.5 安全考虑
- **来源校验**：@font-face 字节来自页面声明的 URL，经既有 net 抓取（已含 CORS/同源/代理边界，由 net/security crate 负责）；本改动不引入新网络入口。
- **解码安全**：`FontLoader::load_font` 经 fontdue 解码，恶意字体最坏触发 fontdue 解码错误（返回 Err 被吞），不 panic；与 harness 路径风险等价。
- **kill-switch**：env 可一键关闭，回退行为明确。

### 8.6 替代方案

| 维度 | 方案 A：drain pattern ✅ | 方案 B：AsyncFetchHost callback（R2406 原描述） |
|------|--------|--------|
| 实现复杂度 | 🟢 低（+1 字段 +1 方法 + 宿主 drain 块） | 🔴 高（trait 改 → 4 impl + mock 全改） |
| 借用安全 | 🟢 安全（mutate 在 tick 后） | 🔴 renderer tick 内 `&font_loader` 不可变借用，callback mutate 别名冲突 |
| 性能 | 🟢 每 tick 最多一次 resolver 重建（批量） | 🟡 可能 per-font 重建 |
| 可靠性 | 🟢 复用既有 drain 语义 | 🟡 transient host 需注入 FontLoader 引用 |
| 可维护性 | 🟢 AsyncPageLoad 自持收集，宿主职责单一 | 🟡 host 持 FontLoader 耦合 |
| 成本 | 🟢 低 | 🔴 高 |

**最终选择**：方案 A（drain pattern）。
**理由**：1) renderer tick 内 `font_loader` 不可变借用是硬约束，callback 在 tick 内 mutate 不可行（编译失败/UB）；2) code-guidelines §2 简单至上——drain 用最少代码达成；3) 批量 resolver 重建更高效；4) 不污染 `AsyncFetchHost` trait 与其全部实现。

### 8.7 实施计划
1. slice 1（本 session）：engine `extract_font_faces` pub + 单测 → `make test -p zero-engine` + clippy → commit R2408。
2. slice 2-a（后续）：async_load family 跟踪 + 合并 CSS + drain + 单测。
3. slice 2-b（后续）：renderer + tabworker 接通 + env kill-switch → `make test` + `make reftest` + product-smoke/legacy A/B 零回归 → commit。
4. slice 3（后续）：本地 rust web server fixture + live 截图验证 + A/B → evidence。

### 8.8 测试策略
- **单元测试**：engine 提取器（FR-001 3 场景）；async_load drain + family 跟踪 + 合并 CSS + data: 跳过（FR-002/003，MockFetchHost 已有）；宿主 drain 处理 + kill-switch + 无效字节（FR-004）。
- **集成测试**：renderer `with_io` in-process（main.rs:152 已支持）断言 load + resolver 含 family。
- **live 验证（slice 3）**：本地 rust web server（`tiny_http`/`axum` 或既有 net crate 自带 server）serve @font-face 页 + 本地字体；live browser 截图 vs 声明字体参考 vs fallback 参考。非 CI 强制门。

### 8.9 回滚计划
- **slice 1**：纯新增 pub fn + 测试，回滚 = revert 单 commit。
- **slice 2**：env `ZW_LIVE_FONTFACE=0` 即时回退（无需 revert）；代码 revert 单 commit。
- **slice 3**：验证脚本/evidence，无运行时影响。

---

## 9. Spec Lint 报告

### 结构完整性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 执行摘要存在性 | ✅ Pass | §0 含目标/范围/排除/约束/方案/首步 |
| 场景存在性 | ✅ Pass | FR-001~005 各 ≥1 场景 |
| 异常路径覆盖 | ✅ Pass | FR-003 失败、FR-004 无效字节/kill-switch、FR-002 data: 跳过均覆盖 |
| 测试绑定 | ✅ Pass | 每场景绑 test 函数名/命令 |
| UI 对齐 | ⏭️ Skip | 非任务 |
| TBD 清零 | ✅ Pass | 无阻塞性 TBD（§10 仅 2 项可选） |
| 约束覆盖 | ✅ Pass | NFR-001~004 各被 ≥1 场景覆盖 |
| 实施交接完备 | ✅ Pass | §7 文件清单/职责映射/修改顺序/首批提交齐 |
| 首步可执行性 | ✅ Pass | §0 首步 + §7 顺序 1 明确 |

### 语言精确性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 模糊动词 | ✅ Pass | FR 用「必须返回/收集/drain/加载/注册」具体动词 |
| 无量化描述 | ✅ Pass | NFR 给 make 命令、行数阈值、env 值 |
| 非确定性措辞 | ✅ Pass | 用「必须/不得」，无「应该/可能」（优先级字段除外） |

### 一致性
| 规则 | 裁决 | 说明 |
|------|------|------|
| 范围冲突 | ✅ Pass | §1.3 在/不在范围无交集 |
| 约束冲突 | ✅ Pass | §6.1/6.2 无矛盾 |
| 方案漂移 | ✅ Pass | §8 drain pattern 与 §6.3 决策 1/§6.2 不改 trait 一致 |
| CLI 语义一致 | ⏭️ Skip | 非任务（env 非 CLI） |
| 默认动作闭合 | ✅ Pass | IF-003 env 未设=启用、非法=启用；§6.3 决策 4 |
| 章节引用正确 | ✅ Pass | IF-001→§6.5A、IF-002→RFC §8.4、IF-003→FR-004 均实指 |
| 外部事实保守化 | ✅ Pass | fontdue WOFF2/local() 行为标「已验证假设」§6.5；非断言为 FR |
| 未验证细节泄漏 | ✅ Pass | live-browser 截图预期仅描述差异方向，未硬编码像素 |
| 场景预期泄漏 | ✅ Pass | slice 3 场景验证「差异存在」非具体像素值 |
| 实现来源闭合 | ✅ Pass | §6.5A 每能力指明 css-parser/FontLoader/WebView/env 模式 |
| 来源-测试联动 | ✅ Pass | 提取器来源 css-parser + 单测；load/register 来源 + 单测 |
| 脆弱选择逻辑覆盖 | ✅ Pass | FR-002 data:/local 过滤、env 开关判断均有场景 |
| 类型分层清晰 | ✅ Pass | 需求/决策/假设分层，§6 分节 |
| 优先级完备 | ✅ Pass | FR/NFR 均标优先级 |
| 代码边界完备 | ✅ Pass | §6.6 允许/禁止修改声明 |
| 清单数量一致 | ✅ Pass | slice 1/2/3 与文件清单一致 |
| 依赖清单一致 | ✅ Pass | 无新依赖（§6.2），各处一致 |
| 重复失控 | ✅ Pass | env 规格 IF-003 主定义，他处引用 |

**汇总**：24 Pass / 0 Warning / 0 Fail / 3 Skip
**门禁判定**：Fail = 0 → 允许确认（rally 自主模式直接进入实施）

---

## 10. 待定列表

| ID | 项目 | 优先级 | 缺失信息 | 下一步 |
|----|------|--------|----------|--------|
| TBD-1 | harness `extract_font_faces` 是否迁移到 engine pub 版以去重 | 可选 | 减重复 vs 不动 harness 的权衡 | slice 2 时定（默认不动，§6.2） |
| TBD-2 | slice 3 本地 web server 选型（tiny_http/axum/net crate 自带） | 可选 | 仓内是否已有可复用 server | slice 3 时探查 |

---

## 11. 修订历史

| 版本 | 日期 | 变更内容 |
|------|------|----------|
| v1.0 | 2026-08-01 | 初始版本（R2407）：R2406 重大发现 → drain pattern RFC + slice 计划 |
