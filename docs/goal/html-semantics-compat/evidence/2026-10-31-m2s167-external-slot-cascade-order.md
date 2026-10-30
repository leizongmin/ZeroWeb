# M2 切片 167：外链槽位级联序——结构化条目 + collector 文档序交错装配（stylesheet-media-change 断言 2 翻绿）

- **日期**：2026-10-31
- **刀位**：s166 候选①（s161 FIXME 精确化后升格首选），靶面 `the-link-element/stylesheet-media-change-no-fetch.html` 断言 2
- **结果**：靶面整案翻 P（单 async_test 断言序执行，a2 在 Pass 键内）；the-link-element 目录 112P/2F → 113P/1F；s166 饿死解除三案金丝 3/3 保绿；全量 join 见尾段

## 入场勘根

s166 已把病根定位到 collector：`collect_stylesheets_with_parsed_external`（pipeline/mod.rs）把外链槽 blob 文本 push 在文档 `<style>` 循环**之前**——link 在 `<style>` 之后的页，同特异度 link 规则先到恒输（CSS Cascade 5 author origin 按引入元素文档序，https://drafts.csswg.org/css-cascade-5/#cascade-sort）。overlay 通道同病：`__zw_set_external_css` 单 blob 落同槽同位置。probe g/h（s166 账本）高特异度对照已证级联通道全通，缺口纯为序。

**修法定谳**：blob 是无位次的合并文本，序信息在下发侧已被合并抹掉——修序必须把「引入元素位次」从 shim 穿到 collector。方案 = 结构化条目槽 + 双写者所有权互斥 + collector 文档序交错装配；派生 blob 保 legacy 通道（CSSOM wire memo、webview 静态发布、缓存键 css 分量）字节同构零迁移。

## 施工（4 文件）

1. `crates/engine/src/js_dom_bridge/computed_style_cache.rs`——新槽 `ACTIVE_EXTERNAL_CSS_ENTRIES: RefCell<Arc<Vec<(String, String)>>>`（sel/text 按源序）。双写者所有权互斥：`set_active_external_css`（blob 写者：webview 静态发布 + 旧回调）清 entries；`set_active_external_css_entries` 写 entries 并派生 blob（texts 按序 `;\n` concat，与旧 `_rcOut` 逐位同构——CSSOM wire memo 与缓存键 css 分量消费 blob 不变）。派生 blob 字节巧合相等（静态 blob == shim concat）会漏换代 → entries 进全部相关缓存键分量：GENERATION_CACHE 8→9 元（类型 + `try_sync_with_drain_record`/`finish_generation_update` 签名 + dirty 判 `slot.8 != entries` + recollect 臂装槽 + 两个换代重填臂）；`generation_cache_matches` 与 `with_cached_document_styles` 的 `same_gen` 谓词逐字同步（在册纪律）；HANDLE_VIEW_CACHE 10→11 元（pattern `(h, g, v, vg, ap, c, a, e, ..)`）；callbacks cs_cache 6→7 元（need_reset 比较 + init tuple）。
2. `crates/engine/src/pipeline/mod.rs`——`collect_stylesheets_with_parsed_external` 分支化：entries 空 → blob 前置逐位不变（静态解析路径/未触发 recompute 的页/渲染线程 session.css 形态零回归）；非空 → `collect_interleaved_external`。三 helper：`style_sheet_if_eligible`（s160 三门提取共用，原 inline 块上移）、`collect_interleaved_external`（`get_elements_by_tag_names(&["style","link"])` 单次 DFS = 文档序；entries 经 `find_by_selector` 预锚 NodeId→idx；style 节点先消费锚定 @import 条目再推 base sheet（import 位次）；link 节点消费 + `link_entry_eligible` 走查期复核；未锚定尾接——selector 漂移保留「表在场」语义，与 rescue/slice33 同暴露面）、`link_entry_eligible`（rel ASCII whitespace 五字符集分词含 stylesheet + href 非空，s158 shim 同口径）。
3. `crates/engine/src/js_dom_bridge/callbacks.rs`——注册 `__zw_set_external_css_entries`（wire `\u{1}` 分记录 `\u{0}` 分 sel/text，空 sel/残记录弃）；cs_cache 键分量同步。
4. `crates/engine/src/js_dom_shim/part03.js`——`_zwSsRecomputeExt` 两循环逐条 push entries（link 用 `_rcSel`、style 用 `_rcSSel`，text 为 s166 wrap 后文本、NUL→U+FFFD 同 CSS preprocessing）；下发分支化：回调存在走 entries wire，缺失回落 `__zw_set_external_css(_rcOut)`（旧 shim 兼容）。

**跨线程零断链**：entries 槽与 blob 槽同为 js worker 线程 thread_local（host 回调单线程写）；渲染线程 `collect_stylesheets` 走显式 `session.css` 参数、其线程 entries 恒空 → 自然回落 blob 前置形态。gCS/handle 视图（js worker 侧）全路径经同一 collector，entries 模式全局一致（`compute_styles_for_doc` → `collect_stylesheets_with_parsed_external`，P9-9 共路）。

## 验证

- `cargo check -p zero-engine` 过（一处 cs_cache 比较解引用修复）；`cargo fmt -p zero-engine` 后 hunk 全落本次编辑函数，无越界扫动；`make guarded-clippy` workspace 全绿（一处 doc list 空行修复）。
- engine lib `js_dom_bridge` 单测 1359P/11F——失败名单与 s166 基线同集（fetch 网络族 9 + `ua_display_li` + `style_sheet_from_handle_style_element_r113` 环境面），零新增失败。
- shim 13 part 按 include_str! 装配序拼接 `node --check` SYNTAX_OK。
- 定向 `FILTER=stylesheet-media-change`：`stylesheet-media-change-no-fetch` **整案 Pass**（s166 后失败点为断言 2：`expected "rgb(0, 128, 0)" but got "rgb(255, 0, 0)"`——link 后到应胜实败；现断言 2 翻绿，a3 随整案 Pass 复验）。
- `FILTER=the-link-element` 目录：**113P/1F**（s166 基线 112P/2F）——F2P = 靶面；余 1F = `link-rel-attribute` shadow 案（s164 基线已红，候选②靶面未动），零 P2F。
- s166 门修复金丝三案（饿死解除 F2P 的保绿复验）：`conditionally-block-rendering-on-link-media-attr` ×2 + `style-element-media-match-block-script` 全 Pass——上游 c1f8e9b63 落地后重建的组合树二进制上保绿（本轮金丝首跑早退死，重建后三定向重跑）。
- 归因轮（P2F 13 环境漂移定谳）：`git stash` 四文件 → HEAD 二进制重建 → closedby 定向 19P/27F 与工作树**位集逐键一致** + auto-expand/name-attribute 单案同 Fail；`git stash pop` 恢复。工作树 stash 期间干净（无其他流未提交改动混入 bisect 二进制）。

## 全量 join（案级 (key,state) 对）

全量对 s166 基线 `/tmp/zw-s166-full1.log`（同为 raw runner 落盘）：(key,state) 对行首 `^(Pass|Fail|Timeout) html/semantics/` 过滤 + `sort -u`——基线 7952 对 → 本轮 7961 对。

- **F2P 1 = 靶案**：`stylesheet-media-change-no-fetch`（Fail → Pass，断言 2 级联序翻绿整案收口）。
- **P2F 13 全数归因环境漂移、零归因本刀（stash 二分铁证）**：closedby complex ×8 + simple ×3 + name-attribute ×1 + auto-expand ×1——`git stash` 本刀四文件后重建 release runner（= HEAD a897d536b 原样二进制）定向复验 **19P/27F 与 16P/24F、3P/3F 位集逐键一致** + auto-expand/name-attribute 单案复现——HEAD 同内容二进制同态，非负载噪声（定向轻载与全量重载同态）。该域为 s161 在册环境时序漂移域（当日 solo 4→11→16→16 漂移先例）；s166 full（三跑并发最重负载窗）16F 反低于本轮（两跑并发 24F）——负载非单调因素，机制面 = testdriver 坐标点击（`getBoundingClientRect` 中心 pointerEvent）+ fixed 覆盖层 hit-test 时序固有抖动（案面 closedby light dismiss 全依赖点击坐标落点）。
- **LOST 26**：select-event ×10（截断镜像——文件级 Timeout 键两轮均在、子测试深度差）+ focus-dynamic ×1 + name-attribute ×2（文件级 Timeout 键换子测键形态，子测 Fail 即上列 P2F 同源）+ 漂移域旧 Pass 键 13（P2F 对侧镜像）+ 靶案旧 Fail 键 1（F2P 对侧）。
- **GAINED 35**：focus-dynamic 完整上报 19 Pass（本轮文件跑全，Timeout 解除对侧）+ 漂移域新 Fail 态 15 + 靶案 Pass 键 1。
- raw 终账 **7090P/787F/87T**（s166 基线 7091P/775F/89T）。s166 饿死解除三案金丝（conditionally-block ×2 + style-element-media-match）全量键保持 Pass。

## FIXME 记账

- **解除**：外链槽位级联序（s161 记 → s166 精确化 → 本刀槽侧根落地：collector 交错装配 + shim 逐条下发）。
- **挂账（本刀暴露面）**：webview 静态 blob 路径未 entries 化——`webview.rs` 静态发布/`SetDomSnapshot` 恢复臂仍走 blob 前置（渲染线程 entries 槽不装填，回落形态即现状语义）；生产 renderer 多进程路径的反应面更新 + 快照恢复组合语义沿用 s158 在册 FIXME。runner/headless（gCS 面）已全覆盖。
- **保留**：link-rel-attribute shadow 案（候选②）；`color_scheme_decl` 继承序列化面（候选③勘根）。

## 下一步 s168 候选

1. shadow DOM 级联面（`link-rel-attribute` shadow 案，目录余留唯一 F）；
2. `color_scheme_decl` 继承序列化面勘根（s165 挂账）；
3. 全量 join 尾部新增 Fail 名单勘根（若有）。
