# R5314 — XHTML 自闭合非 void 起始标签桥接（XML 1.0 §3.1 EmptyElemTag vs WHATWG HTML self-closing flag ignored）——`.xht` 树形对齐 chromium；kill-switch `ZW_XHTML_SELF_CLOSE`；corpus 834F→829F（FIXED=5 零新增红）

**日期**: 2026-10-28
**驱动案**: `css/CSS2/values/numbers-units-{010,012,013}.xht`（R5314 择轻扫描：834F 账本 3 级路径聚类 → values 域 3F，ref 页 `<div/><div/>` 渲单盒）
**corpus**: 同树 A/B（基线 = rebase 后 HEAD `4d0839dca`，stash 干净复跑）——基线 **834F**（fail-list 与 R5313 账本逐案全等，对端 engine 提交零 corpus 位移）vs 修复态 **829F**，逐案 diff = FIXED 5 案：`numbers-units-{010,012,013}` + `floats-clear/floats-{119,120}`（附带翻绿，同域因果——floats-119 **test 页** `<span/>` 把后继 `.float` div 嵌进 span 内，展开后兄弟归位）；**零新增红**。

## 定性（诊断链）

1. **现象**：numbers-units-010/012/013 三案 test 侧首盒与 ref 完全一致（96/96/100px 逐像素同位），差异簇全在 ref 页第 2/3 盒位（pixel-diff 9312/18528/20000px，恰为整盒域）——ref 页应渲 2-3 个兄弟盒，ZW 只渲 1 个。
2. **源级定位**：ref 页 `</p>` 后写 `&lt;div/&gt;&lt;div/&gt;`（XHTML 自闭合语法）。chromium 对 .xht（application/xhtml+xml）走 **XML 解析**：`/` 即空元素闭合（XML 1.0 §3.1 EmptyElemTag）→ 2 个兄弟盒。ZW 用 html5ever 统一按 **HTML 语义**解析：`/` 仅被 acknowledge 后 **ignored**（WHATWG HTML tree construction：self-closing flag 对 HTML 元素无效）→ 第二 div **嵌进**第一 div 内 → 块级 margin 折叠把嵌套链压成**一个**可见盒。
3. **既有标志核对**：`content_is_xml`（node.rs）已按 DOCTYPE public_id 含 "xhtml" 置位并供 style-system 属性选择器大小写分支消费（R3932 XML QName 拆分同门），但**解析树形**仍走 HTML 语义——标志在、语义缺。
4. **上限探测**：834F 全账本 ref 页非 void 自闭合扫描恰命中 3 案（即本族）；A/B 实际翻绿 5 案（floats-119/120 的自闭合在 **test 页**，探测只扫了 match-ref 侧，窄了）。

## 实现（dom parser 单侧预变换，一模块三函数，零 html5ever fork）

`crates/dom/src/parser/xhtml_self_close.rs`（新模块；R5000 `translate_cdata_sections` 同型预扫描）：

1. `source_is_xhtml_doctype`：源级预扫描 DOCTYPE 声明（引号感知）的 **PUBLIC 引用串**含 "xhtml"（ci）——与 `detect_and_set_content_is_xml` 的 parsed public_id 判据**同源**，SYSTEM-only 形态两处一致不判 XML（标志与变换不散页）；
2. `expand_xhtml_self_closing`：命中文档把非 void 自闭合起始标签 `<tag … />` → `<tag …></tag>`（解析产物 = XML 空元素语义）。非 XHTML 文档 **Cow::Borrowed 零拷贝直返**（纯 HTML 语料零开销）。字节面状态机：注释/CDATA/XML PI 整段原样拷贝（XML 字面语义内不出真实标签）；标签体引号感知（`<div title=">"/>`）；`/>` 前邻 `=`（未引号属性值尾 `/`，XML 本身非法）保守跳过；void 元素自闭合本就是 HTML 合法形态不双写；名字字节集含 `-`/`_`/`.`/`:`（`<my-tag/>`、`<svg:rect/>` 取全名）；
3. 接线点 `parse_html_with_builder`（runner/engine 全部文档载入的唯一 choke point，pipeline_budget.rs:305 `render_html` 链实测）；kill-switch `ZW_XHTML_SELF_CLOSE`（LazyLock 构造期读取，`=0` 回退 HTML 旧序，**在 doctype 嗅探之后**才读——纯 HTML 文档零 env 开销）；
4. `is_void_element` 复用 serializer.rs 既有判定（`pub(crate)` 化），不重建清单。FIXME(html5ever): xml5ever/XML 通道落地后移除本预扫描。

## 回归钉（6 件，mutant 红在案）

`crates/dom/src/tests/tests_7_parser.rs` 形状钉 5：

1. `xhtml_self_closing_expands_to_siblings_r5314`（主钉）：XHTML `<div/><div/>` → body 2 个兄弟 div 且各无子；
2. `html_doctype_self_closing_still_nests_r5314`：纯 HTML 保持嵌套旧序（桥接不越界）；
3. `xhtml_self_closing_quoted_attrs_and_voids_r5314`：`title="x/y"`、`data-a=">"` 引号感知 + br void 不双写；
4. `xhtml_system_id_only_stays_html_shape_r5314`：SYSTEM-only doctype 不判 XML（判据同源不散页）；
5. `xhtml_self_closing_comment_untouched_r5314`：注释内形似标签不参与变换。

`crates/dom/tests/r5314_xhtml_self_close_kill_switch.rs`（独立测试进程——LazyLock 构造期读取，set_var 须先于本进程首个 XHTML 解析，R5313 教训）：`ZW_XHTML_SELF_CLOSE=0` 断言嵌套旧序。**mutant 因果**：开关关闭态主钉红于兄弟断言（修复前路径复现，开/关双向可判别）。

## 门禁

- fmt 0 diff；clippy `zero-dom --all-targets -D warnings` 0 告警；**zero-dom lib 873P/0F**（873 含新增 5 钉）；**engine lib 3570P/0F**。
- corpus 同树 A/B（见题头）834F→829F 零新增红；5 案逐案核销；驱动族 FILTER 探测 17/17 全 ✓（010/012/013 ✗1.94/3.86/4.17%→✓0.00-0.04%）。
- product-smoke：struct ×6 全 PASS；welcome oracle **15.68%（75254/480000px）与 R5308-R5313 零位移同值**（welcome 为 HTML5 doctype，桥接路径不触及——预期一致）。
- bench-gate 定向 `ZERO_WEB_BENCH_CRATES=zero-dom`（静窗）：**GATE PASS**，19 指标全在预算内（NEW=9 为定向覆盖首次纳入、无基线可比，政策口径「perf-gate 只对报告实际包含条目做比较」）；可比面 parse_ms medium 5.93/预算 46.46（基线 5.61，+0.32ms——嗅探为 doctype 区短扫描，非 XHTML 文档零拷贝直返）。
- make test 残余腿（对端活跃窗跑）：renderer --lib **350P/2F**、browser --bin **703P/3F/1I**、browser --lib 10P、quickjs no-browser-renderer **186P/0F**、quickjs renderer --lib **347P/3F**——renderer/quickjs 5 红全为 s125 form fixture 族既有记账（名逐一吻合）；browser --bin 第三红 `bookmark_context_menu_open_in_new_tab_loads_url`（tab_count 断言）**kill-switch 关闭态同位复现 → 非本轮改动**（确定性失败，非负载脆弱；嫌疑面在对端近期提交，移交知悉）。

## 挂账

- make test workspace 全量腿未跑（残余腿已补，见门禁；image_batch_e2e + service_worker_owner 负载脆弱双族既有记账维持）。
- browser --bin `bookmark_context_menu_open_in_new_tab_loads_url` 既有红移交对端（kill-switch 关闭同位复现非本轮；tab_count 断言，非 form 族）。
- SYSTEM-only XHTML doctype 文档（public_id 缺席）仍按 HTML 解析——与既有 `content_is_xml` 判据一致地窄，corpus 无此形态案；若未来上游出现，需先扩判据再扩变换（两处同步）。
- `<foo/>` 自闭合**未知元素**（非 void、非已知 HTML 名）同样被展开——XML 语义正确方向，corpus 零翻红背书；理论面与 chromium XML 行为一致。
- R5313 挂账全维持（outline-028 lime 域 0.31%、multicol/Path B 让位门观察项、float walk y 双补偿、Path B border-box 带核对、form 族移交对端、负载脆弱双族、balance 9F、welcome 15.68% 对端流、corner-shape 族）。

## 下一步 R5315

①829F 账本续扫择轻（避开本轮 XHTML 自闭合域与 R5313 ib 域）②outline-028 lime 域定性 / balance 供给评估维持。

**域收口核验**：修复后账本全量复扫（test+ref 双侧、跨行标签）尚有 35 案含自闭合形态，逐案归因**全为 SVG/foreign content 元素**（`svg:rect`/`rect`/`circle`/`feFlood` 等，.html 文件居多）——foreign 元素 html5ever 本就按 spec 正确响应 self-closing flag（WHATWG foreign content 例外），与本轮 HTML 侧缺陷不同域；其红因属 SVG filter/transforms/border-shape 深供给域。**XHTML 自闭合域收口，无同域余量**。
