# M2 切片 106：SVG script 族静态面——child text content 源 + end-tag 序 + href 空串

日期：2026-10-27。范围：`crates/dom/src/document/mod.rs`（新 `child_text_content`）+ `crates/engine/src/pipeline/extract.rs`（源文本取法 + end-tag 重排 + href 空串面 + 4 回归钉）。上轮：[M2 切片 105](2026-10-27-m2s105-import-attributes-static-validation.md)。

## 1. 入场定谳

- s106 探针三候选（承 s105 下一步）：execution-timing 50F 按 fail 形状重新分组发现 **SVG script 族 8 页**（137/138/139/140/142/143/144/145）——其中 138/139/143 为 harness 级失败（`page script threw: SyntaxError: Unexpected token '.'`，143 为 `token ':'` 正是语料故意埋的 `assert_unreached():`），137/140/142/144/145 为用例级。族内共同根因优先于 scheduler 异质 50F，定刀。
- 根因（探针实证）：①html5ever 对 foreign content **按 spec markup 解析**——SVG script 内嵌套 `<script>` 是真实子元素（探针 dump 树形确认）；②提取器 `extract_page_scripts_indexed` 取源用递归 `text_content`（textContent 语义含后代元素文本）——外层源吞入嵌套脚本文本（`var a = {` 后接 `t.step(` → `Unexpected token '.'`）。spec 语义：脚本源文本 = **child text content**（直接子文本节点拼接，prepare-the-script-element「source text」）；HTML script 内容为 raw text 仅文本子节点，两取法恒等（其余 585 行零影响）。
- 判据（语料原文核对）：138 SVG script nested inlines 期望 `["inline script #1", "inline script #2", "end inline script #2"]`——嵌套子脚本先于外层执行（**end-tag 序**：parser-inserted 脚本于自身 end tag 完成时执行，子先闭合先执行；提取序为 pre-order 父先子后，序错）；139 nested external in inline 期望 `[external, #1, end #1]`（外链子经 xlink:href 取回，子 end tag 先闭合）；137 empty xlink:href 期望 `assert_unreached` 不可达（外链形态成立空 URL 不落内联臂）。
- 族内余页属运行时面，本轮不触：142（document.write 注入外链 SVG script）、143（write 双 `</script>` 提前闭合外层——语料故意双写，静态提取修不到）、144/145（DOM mutation 改 textContent/type 后重触发 prepare）。

## 2. 接线

- zero-dom 新 `child_text_content(id)`：直接子 Text 节点拼接（spec infrastructure#child-text-content），与 `text_content`（递归）并列，文档注释注明 SVG script 嵌套场景差异。
- extract 三刀：①源文本 `text_content` → `child_text_content`；②执行序重排——按「pre-order 先行者是后行者祖先 → 后行者先行」`sort_by`（区间序，传递一致；无嵌套恒等），`this_idx` 序号保持全体 script pre-order 序与 shim `getElementsByTagName('script')` 对齐（执行序与序号解耦）；③SVG 命名空间 script href/xlink:href 空串 → 跳过不落内联臂（HTML script `src=""` 维持旧回落，不扩大）。
- 诊断路径：全扫日志 fail 形状分组 → harness 级（页路径为名）三页语料核对 → 「Unexpected token '.'」坐标疑云以探针测试 dump 双取法源文本实证（探针已删，结论留档）。

## 3. 验证

- 单测：zero-dom lib **868/868**；zero-engine lib **3540/3540**（+4 回归钉：嵌套源+序+序号 / 外链嵌套序 / 空 href 双面 / 无嵌套序不变）；zero-webview lib **809/809**（提取面变更过管线回归）。fmt clean；clippy 三 crate（dom/engine/webview）`-D warnings` clean。
- 定向：**FILTER=execution-timing 与 s105 基线 diff 恰 4 行（137/138/139/140 全 F→P）**，其余 46 页零变化。
- 全子树终扫（scripting-1 口径 593 行）：**356P/185F/37T + 15 NotRun**（s105：352P/189F/37T+15N）；行级 diff **恰 4 行**（4 目标页 F→P），**零 P→非P**（comm 校验）、T/N 持平零旁损。
- 渲染面核对：本轮提取/排序纯静态面 + DOM 访问器，无布局/绘制依赖，reftest/product-smoke 未跑。

## 4. 挂账

- SVG script 族运行时面 4 页（142/143/144/145）：document.write 注入/闭合 SVG script、DOM mutation 重触发 prepare——需 script 元素生命周期面（prepare 与 DOM 变更联动），另立项。
- execution-timing 余 ~42F（scheduler 排序语义长尾 + 063 runner 脚本记账）；数据模块全量支持（fetcher MIME 门 + extractor 类型捕获 + registry 合成源 + rewriter）；.py 执行器前置。
- 承袭挂账不变：charset 族（Timeout 形）、svg-script-fragment-clone 3F、FIXME 组、接口对象基建面、testharness.rs /tmp/zw-hb*.txt 插桩来源待查。execution-timing 137/138/139/140 本轮销账。
