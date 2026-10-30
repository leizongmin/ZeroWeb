# M2-S164 静态 style 元素 @import 取回通道（级联 + CSSImportRule 子表 + 扫描臂）

日期：2026-10-31。基线：s163 终案对账（`/tmp/zw-s162-full1.log` 7009P/783F/91T 子测级 + 21 NotRun，命名承 s163 刀落盘习惯）。

## 切片目标

s163 候选①：style-element-media-match-block-script 1F——`<style>@import url('stylesheet.py?delay=1')</style>` 后随同步 script 断言 `styleExists("h1 { color: purple; }")`（经 `CSSImportRule.styleSheet` 递归逐字节比对）+ gCS `rgb(128,0,128)`。

## 勘根定谳

引擎 base 路径对 style 文本 @-rule defer（零取回零级联）；shim 取回通道只覆盖 link（s150 动态/s152 静态）；CSSOM 面引擎 wire 对 @-rule defer，style 表 cssRules 恒空——`styleExists` 递归无 import 规则可下钻，恒 false。三件套缺口：级联文本、CSSImportRule 子表、扫描触发臂。爆炸半径盘点：html/semantics 树内静态 `<style>` 含 @import 仅 3 文件（靶面 match/not-match + style_events）；106/150-import 与 link-load-event-alt 族是 link 通道/动态创建不受影响；style-load-after-mutate、style-error-01、script-created-style-element、dynamic-render-blocking-style-element 均动态创建，不在扫描快照。

## 施工面（2 文件均 shim，rust 零改动）

1. **part03 取回引擎平移**：`_zwStylesheetMaybeStart` 内本地闭包 `_ssVisit` → 模块级 `globalThis._zwSsVisitCss`（无调用点特定捕获，逐字节平移零语义差；link 通道 `_ssDoVisit` 改调全局）。style 臂与 link 臂共用 mixed 门/judge/递归/pending 计数全链。
2. **part03 style @import 通道**：状态表三件（`_zwStyleImportTexts` 落表文本/`_zwStyleImportUrls` 解析 URL/`_zwStyleImportFired` 幂等印记，键 = 扫描快照 sel 原文——静态 parse 元素无 handle 形）；`_zwStyleScanVisit` 扫描臂——type eligible 门（非 eligible 不取回不置印）→ `__zw_get_text_lw` 文本读 → `_zwStylesheetImports` 提取（零命中零成本 bail）→ media 门（s163 link 同款 `__zw_match_media` 1280×800，no-match → setTimeout(0) 推迟——扫描窗外执行，delay 折叠失效，落表时点在断言后）→ 多 import 聚合取回（任一 error → error；文本 join `;\n`）→ 落表 + `_zwSsRecomputeExt()` 级联下发（overlay 是级联唯一载体）；error → 空贡献 + 向元素 fire error（连接门 fire 点重查，s162 派发形态）。`_zwStyleImportEntries(sel)` 查询口供 part06（落表文本经 s163 `__zw_style_rules_text` 桥转 wire）。
3. **part03 静态扫描扩展**：`_zwStylesheetStaticScanUpto` 建表加收 style 元素（`_zwScanStyles`/`_zwScanStylePrefix`，同 script 交错口径）；扫描窗内 prefix 游标触发（与 link 同窗 `__zwScanCollapseTrickle` 折叠——delay=1 同步结算，script-blocking 时序先于脚本体）。动态创建 style 不在快照，「script-created 不阻塞」案语义天然保持。
4. **part03 `_zwSsRecomputeExt` 合并面**：style 落表文本追加并入 overlay 合并文本（逐 style live media 属性 wrap `@media`——no-match 文本级联零应用；error 空贡献跳过）。
5. **part06 CSSOM 注入**：`_makeSheetBuild.getRules()` 尾部——`_zwStyleImportEntries(sel)` 有条目时逆序 unshift CSSImportRule 形对象（`Object.create(CSSImportRule.prototype)` 满足 `instanceof`；type 3；cssText `@import url("<abs>");` real UA 序列化形；`styleSheet` = `_makeStyleSheet(null, wireText, href)` 子表；`__zwImport: true` 标记）；`flushToOwner` 过滤 `__zwImport` 条目（import face 不回写 `<style>` 文本——引擎 @-rule defer，回写零级联收益纯污染）。

## 验证

- shim 14-part 拼接 `node --check` ✓（111943 行，净 +159）；rust 零改动 fmt/clippy 零波及（s155/s162 shim-only 先例）。
- 定向 interactions-of-styling-and-scripting 目录 **9/9 全绿**（前态 8P/1F）：靶面 style-element-media-match-block-script **F→P**；not-match/conditionally-block 双案/script-created/dynamic-render-blocking 全保持 P。
- document-metadata/the-style-element 目录 join 对 s163 full1：**style_events.html 文件级 Timeout → 2P**（tError 面搭车兑现——404 import 链 error 事件接通；该文件级 T 键自 s161 起在册）；style-error-01（动态创建，Timeout 慢性在册）、tentative 族（基线同态 Timeout/NotRun）零位移；style_load_event / style-load-after-mutate 等 s162 面全保持。
- scripting-1/execution-timing（106/150 link @import 内层通道）全 Pass 保持。
- 全量案级 join（full1 对 s163 full1，(key,state) 对 sort -u，LC_ALL=C）：**P2BAD 0**（零 P→F/P→T）；唯一 OTHER 位移 = 靶面 Fail→Pass。LOST 5 = style_events 文件级 T（rider 兑现，镜像 GAINED 1 error 子测 P）+ 4 在册交互时序域文件级 T（selection/focus-dynamic-type-change/dialog-closedby/select-home-end）。GAINED 157（143P+14F）= select-event/dialog/focus 族完整行集首报（s163 8-job 并发截断镜像，s163 已 solo 定谳 flake 属性）。raw 终账 **7156P/795F/87T**（+21 NotRun 两侧同）。
- LOST 4 flake 域 solo/A/B 定谳：dialog-closedby **stash 基线 solo 同 27P/8F**（本树两跑 27P/8F→27P/0F 抖动同形）；selection.html 文件级 T 自 s161 full4 慢性在册；focus-dynamic solo 104P 全恢复；select-home-end 族 s161 即有在册 F 行。零归因本刀。
- reftest/product-smoke 未跑——级联变更只在静态 `<style>@import` 页显形，`tests/fixtures/` 全量 grep 零 @import，product/reftest 语料不可达（s163 无渲染路径变更先例同款记账）。

## 范围与门禁

shim part03.js（取回引擎平移 + 通道 + 扫描 + 合并面）+ part06.js（cssRules 注入 + flush 过滤）。rust 零改动。

## 记账面（本切片不解决）

1. **link 通道 recompute media wrap 死代码**：`_zwSsRecomputeExt` link 条目 `_rcText` 装填未消费（s161 引入，wrap 未落 `_rcOut`）——本刀 style 条目 wrap 正确应用，link 条目不动仅记账；现网被加载期门（webview resolve_external_css media 门）遮蔽，动态 link media 变更面潜在暴露。
2. **@import 原位语义未建模**：取回文本整体追加在 style 规则文本后（CSSImportRule.styleSheet 子表 face 读取序正确；同特异性级联序偏差，语料无形态）。
3. **动态 style @import 臂未挂**：script-created style 的 @import 不取回（style-error-01 的 error 断言面、style-load-after-mutate 的 await onload 面在既有语义下各自保持）；type/title 属性变更触发面同 s162 债。
4. **CSSOM 规则接口族**整体建模未做（承 s163——本轮 CSSImportRule 仅空壳 + Object.create 实例）。
5. **media 求值视口 1280×800 FIXME** 承袭（s160/s161/s163 同债）。
