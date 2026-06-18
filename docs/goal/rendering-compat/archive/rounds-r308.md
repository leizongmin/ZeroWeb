# R308 — font-size 百分比解析修复（归档自 master.md）

> 归档说明：本文件为 master.md「最近轮次详细记录」中 R308 的逐轮详细记录，于 doc-maintenance 轮（2026-06-19）归档——并行 agent 提交 R328（单会话 lever 穷尽再确认 + DC-9/multicol 路径审计）后，master.md 最近轮次窗口达 21 轮（R308–R328），R308 作为第 21 轮迁出，窗口收窄为最近 20 轮（R309–R328）。R308 的核心结论（`computed.rs` font-size 调用点 Percentage 就地解析为父 font-size 百分比而非 px，修 anonymous-inline-inherit-001 chromium-Oracle 3.84→0.43%，strict 一处 revealed false-pass；POLLUTED 逐项 probe 仍可发现真实单点 bug）仍以浓缩形式保留在 master.md「综合裁决」与后续 R309/R311 等轮的「承接」引用中。本归档仅为可追溯性保留，archive 区不修改。

---

### R308 — font-size 百分比解析修复（code change，DC-14 真实一致性修复，loose 438/490 持平 / strict 296→295 一处 revealed false-pass）

**承接**：R307 关闭 near-pass clean-win 杠杆后，转攻 R307 evidence 里**未调查**的 POLLUTED 候选 `anonymous-inline-inherit-001`（self 0.00% / chromium 3.84%，CSS2 linebox，非 writing-mode/multicol/grid 聚类）。PIL 渲染对比 + LAYOUT_DUMP 实测定位到**真实单点 bug**。

**根因（computed.rs:51 + 186）**：`resolve_length` 的 `LengthValue::Percentage(v) => *v` 分支返回**原始百分比数值**（注释「由布局引擎按容器尺寸处理」——对 width/height 正确）。但 **font-size 属性的调用站点**（line 186 `resolve_length(&style.font_size, ...)`）复用了该泛型函数 → `font-size: 500%` 解析为 **500.0（当 px 用）** 而非 CSS §10.1 规定的「父元素 font-size 的百分比」。实测 anonymous-inline-inherit-001：inner `<span style="font-size:500%">` font-size=500px → line-height=600px → span h=600（LAYOUT_DUMP 实证），A glyph 500px 几乎不可见，整体内容下推至 y=588-599（chromium 在 y=27-79）。

**修复（computed.rs:186-191，surgical）**：font-size 调用站点就地处理 Percentage——
```rust
let font_size_px = match &style.font_size {
    LengthValue::Percentage(v) => v / 100.0 * font_size_context,  // 父 font-size 百分比
    other => resolve_length(other, font_size_context, vw, vh),    // em/rem/px 等不变
};
```
仅改 font-size 一个调用点；width/height/margin 等的百分比仍走 resolve_length 的容器相对解析（line 51 不动）。

**验证**：
- **chromium-Oracle**（真指标）：anonymous-inline-inherit-001 ZeroWeb-test vs chromium-oracle **3.84%→0.43%**（A 现以正确 80px 渲染，内容高度/位置对齐）。残余 0.43% = 独立的 `vertical-align: top` 未应用（content y=73 vs chr y=27，Phase A 墙③ 谱系，font-size 修复之外的独立子问题）。
- **loose self-source reftest**：**438/490 持平零回归**（font-size 修复对 test/ref 同步生效，自源计数不变）。
- **strict self-source**：296→**295**（-1，唯一翻转为 `font-features-across-space-1.html`）。该用例用 `font-size:150%`（旧 bug=150px，现正确=24px）+ 自定义 `@font-face ligsym` 字体 + `font-feature-settings:"liga"` 测连字。**150px（bug）掩盖了连字/回退字体差异**（<0.1% strict pass），**24px（正确）暴露 1.03% 差异**——这是 **revealed false-pass（DC-14 anti-false-pass 目标），非修复引入的 bug**。font-size 百分比修复是 CSS 规范正确行为，该 strict -1 是真实状态暴露。
- **新单测** `test_font_size_percentage_uses_parent`（3 断言：500%@16px→80 / 150%@20px→30 / 100%@root→16），回退守卫（旧实现返回 500/150/100 会 FAIL）。
- **make test**：**12254 passed / 0 failed**（10 ignored = real_website_compat）；clippy/fmt 干净。

**意义**：DC-14 真实 chromium 一致性提升（font-size 百分比是常见 CSS，`font-size:110%/90%/150%` 在真实页面普遍，旧 bug 把它们全当 px 渲染——影响任何用百分比 font-size 的页面/产品 smoke）。这是一次「**被 self-source 掩盖的真实缺口**」修复（anonymous-inline-inherit self 0% 不变故 self 计数不动，但 chromium 真实差距 3.84→0.43%）。strict -1 的 revealed false-pass 印证 DC-14 anti-false-pass 价值——R307 关闭的「near-pass clean win」是计数乐观，但**未调查的 POLLUTED 候选逐项 probe 仍能发现真实单点 bug**（区别于 near-pass frontier 的结构性聚类）。

**方法论复用**：R307 按聚类分类（near-pass=结构性拖尾）关闭了 frontier 杠杆；但**逐用例 probe 未调查的 POLLUTED 候选**（self 通过但 chr 不一致）仍是真实 bug 来源——anonymous-inline-inherit 非任一已知结构性聚类，PIL+LAYOUT_DUMP 定位到 font-size 百分比单点。下一轮可继续逐项 probe R298 POLLUTED 清单剩余未调查项（table-grid-item-dynamic-003 23.8% / collapsed-border-vertical-rtl-overflow 4.7% 联）。

**代码变更**：`crates/style-system/src/computed.rs`（font-size 调用点 Percentage 就地解析 + 1 新单测）。基线 loose 438/490 持平 / strict 295/490（一处 revealed false-pass）/ chromium-Oracle 真实一致率提升（anonymous-inline-inherit 3.84→0.43%）。
