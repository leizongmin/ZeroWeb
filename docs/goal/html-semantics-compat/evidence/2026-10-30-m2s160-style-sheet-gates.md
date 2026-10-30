# M2 s160 · style 元素 sheet 三门（media/type/disabled）+ 漂移钉死——11 F→P 零 P→F

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s159（合成条目 href 通道）

## 入场：环境漂移面钉死（s160 候选①，上轮遗留）

三项复跑（/tmp/zw-s160-drift-*.log，21:03-21:05）：

- **selection-pointer：0P/21F/0T——持久漂移定谳**。第三次复跑（s159 轮内
  空闲态 + 本轮），跨 ~14h、跨 engine 提交边界（当日 13:44-17:46 并发流
  engine 提交 R5341-R5344/CE join/trap perf 六笔均晚于 07:01→07:17 翻转
  时刻——纯时间变量 A/B 维持：s158-full3 07:01 同树 21P → s159-full1
  07:17 同树同刀移除 21F）。温控/负载暂态假说排除，持久环境漂移维持
  记账（时序敏感交互案，A/B 已证与代码无关）。
- **select-event：270P/9F/0T**——9F 全部在 s158/s159 基线 Fail 集
  （customizable-select :open 选择器等在册面），零新 Fail；s159 全量
  91 键截断确认为环境性。
- **focus-dynamic：80P/0F/0T**——全恢复。

## 候选④定谳销案（不施工）

全域 .css fixture 扫描（html/semantics 语料，link rel=stylesheet +
@import 双 pattern + 嵌套 .css→.css）：169 引用 / 18 缺失，与 793F
失败案交集 **0**；嵌套 @import 缺失 **0**。缺失 fixture 全部被 Pass 案
引用（error-expect 或无关形态）——拉取反而引入 P→F 回归风险
（error→load 翻转）。零收益 + 负收益风险，销案记账。

## 候选②勘根记账（动态元素 computed style 大面收窄）

失败案 getComputedStyle 依赖扫描（修正 `html/semantics/` 前缀剥离后）：
48 失败文件 / 96F 触及 gCS。最大内聚桶 meta color-scheme（24F）三重
依赖栈定谳：①动态元素 gCS（createElement div 无 `__zwSelector`，shim
hasHost=false）+ ②used color-scheme 传播（meta→root→后代）+ ③系统色
解析（CanvasText 现为静态黑映射 named_colors.rs:164，不随 color-scheme
翻转）。三重栈非单刀面，收窄记账；单刀入口仍是宿主 '@handle' 求值通
道（`find_by_selector` 快照 doc 不含动态元素）。

## 入场定刀位

style 元素 sheet 应用门（s158 FIXME「link media 门未建模」style 侧延
伸）：document-metadata the-style-element 族 media 门 7F / type 门 5F /
disabled 门 1F。

## 勘根（三轮收敛）

1. **断言机制**：style_media_change/style_media/style_disabled 全部静态
   元素 + `getComputedStyle` 宿主通道（静态有 `__zwSelector`）——缺口
   纯在宿主级联的 sheet 收集。
2. **反应管线现成**：`media`/`type` 已入 `_REFLECTED_STRING_FLAT`
   （part01.js:1716）、`disabled` 走显式布尔分支（part05.js:495，任意
   tag 反射内容属性）——IDL 写全部产 SetAttr mutation → gCS style_
   version 换代 → `apply_inline_style_overrides` 四类 mutation 全 apply
   （computed_style.rs:126）→ `is_style_or_meta` 判脏 → `finish_
   generation_update` 重收集 sheets（computed_style_cache.rs:517）。唯
   一缺口 = `collect_stylesheets_with_parsed_external` 零门。
3. **media 不入注册面**（spec 钉）：style_non_matching_media（当前 P）
   断言 `styleSheets.length===1` with `media="unknown"`——sheet 创建注
   册与 media 匹配无关（cssom），media 只门级联应用。disabled 同理
   （CSSOM disabled flag 只禁应用）。type 才是「未创建」门
   （style-eligible）。
4. **type 精确匹配**（style_type_html 钉）："" 应用、TEXT/CsS 应用、
   " text/css " / "text/css; charset=utf-8" 不应用——spec exact-match
   无 trimming 无 parameters；s156 part04 sheet 门的 essence 语义
   （split ';' + trim）与 spec 有偏差（params 形态误创建），一并统一。

## 施工（3 文件净 +61/-10，shim 2 处 rust 1 处）

1. **pipeline/mod.rs `collect_stylesheets_with_parsed_external`**（style
   循环头插三门，净 +45）：
   - type 门：type 存在且非空且非 ASCII 大小写不敏感 `text/css` →
     continue（sheet 未创建，规则不入级联）。
   - media 门：media 存在且非空 → `parse_media_query` + `evaluate_media_
     query`（MediaContext::new(1280,800) screen 默认，与 compute_styles_
     for_doc 同源；FIXME 记会话真 viewport 未传入）→ 全不匹配 →
     continue（已创建不应用）。
   - disabled 门：disabled 内容属性存在 → continue（CSSOM disabled flag
     近似——shim 布尔反射内容属性存储；FIXME 记 setAttribute('disabled')
     语义偏差，语料无负案）。
2. **part06.js styleSheets getter**（+15/-2）：type 门过滤入列元素
   （lw 读 `__zw_get_attr_lw`——同批 IDL 写同步断言立即可见，
   style_type_change t2 面）；media/disabled **不过滤**（勘根 3）。
3. **part04.js sheet property type 门**（+5/-3）：essence 语义收紧为
   精确匹配（`!== '' && toLowerCase() !== 'text/css'` 返 null），与宿
   主级联门 + styleSheets 列表门同规则（LinkStyle 5/5 案面两种规则等
   价——TeXt/CsS 创建、text/html、text/cſs 不创建全保持）。

## 验证

- **语法/门禁**：shim 拼接（include_str! 运行时序）node --check
  SYNTAX_OK + cargo fmt -p zero-engine PASS + `make guarded-clippy`
  workspace 全绿。
- **靶面定向**：the-style-element **28P/2F/0T**——9 靶 F 全翻绿
  （style_media_change 2 + style_media 1 + style_type_change 3 +
  style_type_html 2 + style_disabled 1）；2 记账 F 保持
  （non-parser-created-doc、style_load_event）。
- **搭车 2 案（合法收益）**：tentative/style-element-csp-nonce-style-
  only + style-element-csp-script-src-blocked——`<style type="module"
  specifier="foo">` CSS Modules 形态，type 门使 sheet 未创建 → 断言
  「不应用」翻 Pass。同门合法收益非巧合。
- **回归面**：LinkStyle **5/5 保持**（s156 面）；the-link-element
  **112P/2F/0T = s159 基线逐位一致**（shadow + media-change-no-fetch
  两记账 F 保持）；interactions-of-styling-and-scripting **6P/3F**（3
  预期 F 保持：conditionally-block 2 + media-match-block-script 1——
  link media 平文本槽/@import 级联通道未建模，记账）；style_non_
  matching_media Pass 保持（media 不入列过滤正确性钉）。
- **全量案级 join（对 s159-full1 raw，(key,state) 对集）**：
  **F2P 11（9 靶案 + 2 搭车）/ P2F 0 / 其他位移全部为交互时序域全量
  Timeout 环境截断**（LOST 21：select-event tel 键 ~18 + dialog-closedby
  6 + popover-closedby-complex 文件级——定向全恢复：dialog-closedby
  74P/3F（3F 异文件在册）、popover-complex 24P/16F（16F 同域在册形态）、
  select-event tel 45P）。raw 计数 7080P/809F/90T → 7082P/803F/92T。
- **渲染门禁**：`make reftest` **724/725 与 stash 基线逐位一致**（唯一
  失败 css-float/float-shrink-inline-children-zw-001 3.68% A/B 钉死既
  有环境面，非本刀引入；css-transforms 9×`media="screen"` +
  layer-media-toggle print→all 终态不变全保持）；`make product-smoke`
  exit 0 全 struct PASS。
- **波及面预检**：reftest/corpus 全量 `<style media=...>` 扫描 15 文件
  ——9 screen（匹配不受扰）+ 2 print（not-match-does-not-block 黑色断
  言维持、layer-media-toggle 脚本翻转后终态同形）+ 1 all，零翻转。

## 挂账（本切片更新）

- **收口**：style_media_change 2F / style_media 1F / style_type_change
  3F / style_type_html 2F / style_disabled 1F（style 元素 sheet 三门，
  销案）；搭车 CSP tentative type=module ×2（同门收益）。
- **FIXME 新记**：media 门求值视口 1280×800 默认（宽度条件媒体查询按
  默认视口）；disabled 内容属性存储近似（setAttribute('disabled') 亦
  触发禁用，语料无负案）；link media 门仍未建模（外链 ACTIVE_EXTERNAL_
  CSS 平文本槽无 per-sheet 结构——conditionally-block 2F + link media
  渲染面依赖）；@import→级联通道缺口（style-element-media-match-block-
  script 1F）。
- **承袭不变**：候选②动态元素 computed style 大面（收窄为三重栈勘根
  记账，宿主 '@handle' 求值通道为单刀入口）；meta color-scheme 24F 三
  重栈；selection-pointer 21F 持久环境漂移（同树同时段 A/B 铁证）；
  shadow DOM 级联面（link-rel-attribute shadow 案）；style_load_event
  事件面（1F）；non-parser-created-doc 1F；link-multiple 全量贴边；
  meta long × corpus；details 族贴边；promise_test 串行启动面；
  innerHTML 身份分裂；sel 重解析印记失配；host 侧 \v 分词；overlay/
  base 文本语义差两面；生产 renderer SetDomSnapshot 外链槽恢复臂；
  待决三项须用户拍板。

## 下一步（s161 候选）

①link media 门 + @import→级联通道（conditionally-block 2F + style-
element-media-match-block-script 1F——外链 per-sheet 结构化通道设计
面）；②meta color-scheme 三重栈拆解（24F——先做 used-scheme 传播 +
系统色动态解析两层，动态 gCS 另立）；③style_load_event 事件面（1F，
style 元素 textContent 变更 re-process + load 派发）；④shadow DOM
级联面（link-rel-attribute shadow 案）。
