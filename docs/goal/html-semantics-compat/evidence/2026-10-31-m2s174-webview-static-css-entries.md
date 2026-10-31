# M2-S174 webview 静态外链 CSS entries 化——级联文档序（s167 挂账收口）

**日期**: 2026-10-31
**刀位**: s167 挂账「webview 静态 blob 路径 entries 化」。静态发布（`prepare_page_subresources` → `load_html`）此前把外链表取回合并为**单 blob 整体前置**（`set_active_external_css`），link 在文档 `<style>` 之后的页面同特异度恒输——blob 无位次，违反 CSS Cascade 5 author origin 按引入元素文档序（css-cascade-5 §cascade-sort）。engine collector 早在 s167 就有 `collect_interleaved_external` 文档序交错装配通道，但静态发布侧从未喂过 entries。

## 生产影响面核实

browser tabs（tab_manager.rs:1392）与 renderer 运行时均创建 `zero_webview::WebView` 直调 `load_html`——**嵌入式静态路径就是生产路径**，非测试专用面。WPT runner 的 complete_fetched_page 同走此路径，entries 化对 runner 页生效。

## 施工（rust 3：engine 1 + webview 2）

1. **crates/engine/src/pipeline/extract.rs**：`extract_stylesheet_link_anchors`——返回 `(位次结构路径, href 原始值, media 原文)` 文档序。rel/href 门与 `extract_stylesheet_links` 逐字一致；路径由 `positional_structural_path` 生成（`tag:nth-child(pos)` 段 `>` 连接，pos 按 1-based **元素序**兄弟计数——文本/注释不计，根元素 `:nth-child(1)`），形态与 engine `structural_path_selector` 逐字同约定 → `find_by_selector` t8b fastpath 对该形态 O(1) 下降锚定。同 parser（`zero_dom::parse_html`）→ 新解析树与 engine 树同形，位置路径跨树成立。单测 `link_anchors_positional_paths_s174`（嵌套路径/media 门与多 link/无 link 空集）。
2. **crates/webview/src/webview.rs**：
   - `resolve_external_css` 改返回 `(entries, combined)`：queue 携带**根 link 序号** `(usize, String)`；CSP 门 `target_ordinal` 用根 link 序号（原 `unwrap_or(0)`）；**@import 子表携带根 link 序号传播，展开文本归入根 link 桶**（替换旧平铺队尾尾接——子表与根表同位次）；空文本桶不入 entries；`combined = buckets.concat()` 保持 byte 形态兜底。
   - `prepare_page_subresources` stash `pending_external_css_entries: Option<(u64, Vec<(String,String)>)>`（**generation 配对**——取 `self.document_generation` 现值）；`load_html` 先 `document_generation += 1` 再消费：仅 `stashed_gen.wrapping_add(1) == document_generation` 时走 `set_active_external_css_entries`，**不匹配回落旧 blob 发布**。该配对防跨文档错配（连续 load_html/导航代际跳跃），并保住嵌入器 custom-css 通道（`css` 参数路径不经 stash → 必然回落 blob 槽 → `set_active_external_css` 清 entries 槽的双写者互斥保持）。pub API 零变化（zero-webview 稳定嵌入边界）。
3. **crates/webview/src/tests/external_css_cascade_order.rs**（新，5 钉测）：load_and_probe harness（fetcher 注入 css + 页面脚本内 gCS 探针——gCS 面与布局级联同线程同槽）：
   - `link_after_style_wins_by_document_order`：link 在文档 `<style>` 后 → `fixed`（红态 `static`）；
   - `link_before_style_loses_by_document_order`：防过度矫正守卫 → `static`；
   - `interleaved_link_style_link_document_order`：linkA→`<style>`→linkB body 内交错 → `absolute`（红态 `static`）；
   - `media_gated_link_stays_out_of_entries`：media 不匹配不取回不入条目 → `static`；
   - `import_expansion_stays_with_root_link_bucket`：import→base 两级，link 在文档 `<style>` 后 → 子表规则随根 link 位次后到后胜 → `fixed`（红态 `static`）。

## 验证

- **红→绿 stash 双态对照**（`git stash push` 实现 2 文件、保留测试注册）：红态 3 预期红案全 Fail 且失败值逐字吻合推演（`static`/`static`/`absolute` 案中 `static`×3），2 守卫案红态亦 Pass（无过度矫正）；绿态 5/5。
- **webview lib 830P/0F**（基线 825 + 5 新增）。
- **engine lib 3616P/10F**（+1 = 新单测；10F 与 s173 基线同键：fetch/request/response 契约族 9 + ua_display_li s24）。
- **WPT 域门**（runner 同路径）：
  - interactive-elements **463 键 (key,state) join 对 s173 终态 437P/26F 逐键 bit-exact**：P2F=0/F2P=0/LOST=0/GAINED=0——shadow 面零扰动；
  - the-link-element（114 键）/ the-style-element（43 键）红绿双态 join **零漂移**——两域断言未踩「link 在文档 style 之后」形态，修复价值由钉测红案承载；
  - the-form-element 家族 96P/12F 与 s173 持平（12F 全既有已归因）。
- fmt 全绿 + guarded clippy `--all-targets` 0 警告。

## 归因与边界

- 多进程 renderer js_worker `SetDomSnapshot` 臂（js_worker.rs:1853）维持 blob 通道不变（既有文档化回落）——其 entries 化另行挂账。
- 嵌入器 custom-css（`css` 参数）不经 stash → 必然回落 blob 发布，通道语义不变。
- WPT the-link-element/the-style-element 域无「文档序交错」断言可翻——本刀 F2P=0 属预期（WPT 该域测元素语义；级联序真实页面形态由钉测钉住）。

## FIXME 新记 / 挂账

- 多进程 renderer `SetDomSnapshot` 臂 entries 化（新挂账，低优先——该臂已有 blob 文档化回落）。
- s167 FIXME 就本刀范围销账：runner + embedded 静态路径已 entries 化。

## 并行合并复证（2026-10-31 push 时）

commit 47370ed94 push 前 rebase 并入并行流 0b7c00f83（site-opt T27 click-latency：apps/renderer + page-runtime/lib.rs 1 行）。page-runtime 在 webview 依赖链上——合并树 webview lib 复跑 **830P/0F** 复证通过，零新面孔。

## 下一步候选

①声明式根内容枚举（s173 LOST 20 承接）；③ua_display_li（style-system 跨流）；④selection-pointer forms 域 22 案归因勘定（s172 记）。
