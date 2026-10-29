# M2 s158 · link rel 状态 ↔ 外链 CSS 级联活链路 + rel ASCII whitespace 分词

日期：2026-10-30 · 切片：html-semantics M2 · 前序：s157（尾随静态 link 触发收口）

## 入场定刀位

s157 下一步候选②③：link-rel-attribute 2F（green→rel=no-stylesheet→black→rel=
stylesheet→await load→green→removeAttribute→black 的 sheet 应用/removal 渲染面）+
link-rel-attribute-tokenization 1F（ASCII whitespace 变体）。候选①④先行勘根后降级
记账（见下）。

## 勘根（候选①④重定谳 + ②③根因链）

1. **候选①（stylesheet-media-change-no-fetch 1F）重定谳**：案面失败根因不是
   「media 变更不重取门」，而是**动态元素 computed style 根本性缺位**——
   `document.createElement` 产物是 handle-based proxy（`__zwSelector` 恒 null），
   shim getComputedStyle（part01.js 只读 `__zwSelector`）对动态元素恒 ''（display
   走 UA fallback 表）。探针钉死 q1/q3/q4 动态元素 sel=null、hostColor=''。刀口
   远大于本切片，降级记账（大面另立）。
2. **候选④（stylesheet-not-removed-until-next-stylesheet-loads 1F）重定谳**：失败
   形态 "Cannot read properties of null (reading 'includes')" = R5114 合成条目
   href=null（document.styleSheets 合成面无 href 通道）。s159 候选。
3. **②③根因链（四件叠加）**：fixture 缺失（lib.sh `fetch_dir_html` 2026-10-29 才
   加 .css 扩展，语料初拉在其前 + 幂等快路径按「目录已含 .html」跳过 → 缺口固化
   ——link-rel-attribute.css/neutral.css 本轮补拉）+ rel 变更无级联反应（host 级
   联外链槽只有 load 时一次装填，页面期无反应臂）+ removeAttribute('rel') 无触发
   臂（s155 只建了 setAttribute rel / IDL rel 臂）+ `\v` 分词偏差（helper rel 门
   `split(/\s+/)` 含 U+000B，spec ASCII whitespace 集不含——tokenization 案面
   `test\u000Bstylesheet` 单 token 误命中）。
4. **级联反应机制发现（P9-9）**：gCS 代际缓存键含 css/adopted 分量
   （computed_style_cache.rs `sheets_dirty` 判定 + `generation_cache_matches`）——
   外链 css 槽文本变化即自动换代重收集，只缺「shim 状态变化 → 槽文本」触发臂。
   级联源（`collect_stylesheets_with_parsed_external`）= meta color-scheme + 外链
   CSS 平面文本 + `<style>` + adoptedStyleSheets，**不走 `<link>` 元素**。
5. **键一致性**：`_elKey(sel,handle)`（handle 优先 '@' 前缀）；静态元素键=sel
   （unique selector，探针钉 `elSel === queryAll === "link"`）、动态='@'+handle
   （`__zw_handle_for_selector` 反查补 handle 形）。
6. **link-multiple 贴边案安全性**：case 5 preload 复用路径不产 overlay 条目 →
   覆盖门 fail → recompute 跳过 → 零新增 fetch。

## 施工（3 文件净 +131/-7）

1. **语料补拉**：`link-rel-attribute.css`（`.green { color: green; }`）、
   `neutral.css` 经 lib.sh `fetch_raw` 逐文件落位（不用 FORCE=1 全量重列——未认证
   API 60 req/h 限额）。
2. **engine `__zw_set_external_css` 回调**（callbacks.rs，`__zw_set_adopted_css`
   后）：shim 下发合并文本装 `ACTIVE_EXTERNAL_CSS` 同槽（页面期反应面；P9-9 快照
   臂装填点不变）。FIXME：生产 renderer 多进程路径的 SetDomSnapshot 恢复臂目前只
   覆盖 adopted 槽，外链槽反应面更新与快照恢复的组合语义未建模（runner/headless
   通道先落地）。
3. **part03 四件**：
   - `_zwExtSheetTexts` overlay 表（identity key → 表文本，''=取回失败空贡献）+
     `_zwLinkRelTokens`（spec ASCII whitespace `[\t\n\x0C\r ]+`，与重算遍历共用）+
     `globalThis._zwSsRecomputeExt`（`__zw_query_all('link')` 文档序遍历 + lw 读
     rel/href + handle 反查键 + **覆盖门**：live stylesheet link 全部有表项才替换，
     防瞬态部分状态与 base 丢条目；拼接分隔 `';\n'`）。
   - rel 门三分支：有 stylesheet token → 既有路径；无 token（含 rel 移除/空写）且
     身份在册（overlay/fired 有键）→ 清两表 + recompute；否则静默。幂等同值 rel
     写词集不变不进移除臂。
   - `_ssVisit` 加第 5 参 `_ssCollect` 收集数组（各层 text() push，内层递归透传）；
     done 存 overlay + recompute + 既有 fire 链。
   - **`noOverlay` 第 6 参（设计修订，见归因轮③）**：静态扫描臂传 true——done
     **存表但不下发**（解锁后续变更面覆盖门，税=0）；页面期变更臂（默认 undefined）
     存表 + 下发。
4. **part04 removeItem 陷阱 rel 移除臂**：镜像 s155 set 臂形态（`n === 'rel' &&
   targetTag === 'LINK'` → helper 3 参调用；rel 读缺失走移除臂清表 + 级联重算 +
   印记清空使再入 rel=stylesheet 时重新取回并重派 load）。

## 归因轮（两层 P2F/T Timeout → 三处修订）

1. **106 族 6 案 P2F（第一轮全量）**：execution-timing 106-*/150-import（@import
   内层规则 `#test{position:fixed}` 丢级联 → got static）。定谳过程：A/B stash
   （基线 fixed/本刀 static）→ noimport 对照组 P 隔离 @import 面 → 探针逐层钉态
   （hook 机制无效是**机制性的**：host 回调为只读绑定，sloppy 赋值静默丢弃——
   dump 探针证据全部作废；`globalThis._zwSsRecomputeExt` 是 shim 普通 global 可
   覆盖，包装探针钉 rcCalls=2 证明 done 链完整、recompute 正常、下发发生）→
   **根因 = 无分号 @import 语句残留吞规则**（webview resolve_external_css s89
   同款教训在 shim 层复现两次）：`join('\n')` 使顶层 `@import url(x)`（无 `;`，
   上游 import.css 形）preamble 延伸过换行吞掉内层表首规则。修法两层：recompute
   拼接分隔 `';\n'` + done 存表 join 分隔 `';\n'`（级联 matcher 对 `Rule::Import`
   本就跳过，`;` 终止后残留语句无级联影响）。
2. **details name-attribute 文件级 Timeout（第一/二轮全量，基线三轮无）**：该页
   含 testharness.css 等 6 个 link → 静态扫描臂 done 走 overlay 下发 → 下发文本
   与 base 逐字节不等（@import 原文残留 + 分隔符差异；base 有 absolutize_css_urls
   /@import 剥离）→ **每页面一次多余全文档级联换代**（css 键分量）→ 贴边长页
   （18 子测试）全量并发环境超预算。单文件跑 18/18 Pass（定向不贴边）佐证环境
   敏感。修法 = **静态扫描臂 `noOverlay` 门**（存表但不下发，见施工 3）；修订后
   终跑 name-attribute 18/18 Pass、全域 T 数回到基线 87。
3. **语义修订说明**：overlay 职责收窄为**页面期变更面**（rel/href 变更、动态插
   入、移除）——静态面由 base 通道（webview resolve）负责，二者以覆盖门衔接：
   扫描臂存表条目（不含下发）使多 link 页面的变更面 recompute 不被未入表静态
   link 卡住。

## 验证

- **语法**：规范序拼接 node --check SYNTAX_OK **110635 行**（s157 110462，净
  +173）。cargo fmt --check PASS；guarded-clippy workspace 全绿（CLIPPY_EXIT=0，
  callbacks.rs 定型后 rust 侧无再改动）。
- **靶面定向**：link-rel-attribute light DOM **Fail→Pass**、tokenization
  **Fail→Pass**（2/2 靶案 F→P）；shadow 案预期仍 F（shadow 级联面，动态元素
  gCS 缺位同根，记账）。
- **106 族回归修复**：10/10 全 Pass（execution-timing/106 定向三连跑确认）。
- **目录级**：the-link-element 全目录——link-multiple-load-events **6/6 保持**
  （贴边案在基线行为域内）；3 记账 F 保持（shadow/media-change/not-removed）。
- **全量案级 join（对 s157 终账 /tmp/zw-s157-full3.log，(key,state) 对集直接
  去重）**：**8082 对 = 8082 对**（7205P/793F/87T 对基线 7199P/796F/87T）——
  **F2P 2（两靶案）+ P2F 0 + LOST 0 + GAINED 0 全域零位移**。
- 探针语料验后已删（the-link-element/zz-s158-probe + execution-timing/
  zz-s158b~p 共 15 块，工作树零残留）。

## 挂账（本切片更新）

- **收口：link-rel-attribute light DOM 1F + link-rel-attribute-tokenization 1F**
  （rel 状态 ↔ 外链 CSS 级联活链路 + ASCII whitespace 分词，挂账销案）。
- **FIXME 新记**：
  - overlay 与 base 的文本语义差两面：@import 内层文本追加在顶层文本之后（原位
    展开未建模——同特异性下相对顶层后继规则的优先级偏差，语料无该形态）；url()
    相对引用未绝对化（base 有 absolutize_css_urls——外链 @font-face/背景图相对
    引用在 overlay 下发文本里按页面 URL 解析，与 base 的样式表 URL 基准有差；
    语料无该形态）。
  - link media 属性门未建模（级联无 media 求值，stylesheet-media-change-no-fetch
    与候选①同根记账 F）。
  - 生产 renderer SetDomSnapshot 恢复臂不覆盖外链槽（callbacks.rs FIXME）。
- **承袭不变**：候选①动态元素 computed style 根本缺位（`__zwSelector` 只属静态
  元素，handle 元素恒 ''——大面另立）；候选④ R5114 合成条目无 href（s159 候选）；
  link-multiple-load-events 全量贴边（案 1-4 竞速窗固有溢出，本轮全量 6/6 保持）；
  meta long × corpus 通道放宽失效；details 族贴边超时（本轮 noOverlay 修订消除
  name-attribute 面的换代税贡献，其余文件承袭）；promise_test 串行启动面；
  innerHTML 身份分裂表示面；sel 重解析同槽同 href 印记失配边角；待决三项须用户
  拍板。
- **语料面**：the-link-element 两个 .css fixture 已补；**全域 .css fixture 缺口
  仍在**（fetch_dir_html 2026-10-29 前拉的目录族，sweep 另立）；缺失 .html 用例
  不拉（分母稳定）；`302-no-Location-header-text-css.asis` 不拉（runner 无
  raw-response 通道，按 200 静态字节交付会使 stylesheet-non-OK-status 现绿翻红）。
- **host 侧**：`extract_stylesheet_hrefs` 用 Unicode `split_whitespace()`（含 \v）
  与 shim ASCII 集有差——host 侧 \v 分词偏差记账（本轮不动）。
- **探针方法论**：host 回调只读绑定不可 hook（sloppy 赋值静默丢弃）；shim 自设
  global（`globalThis._zwSsRecomputeExt` 型）可覆盖，是有效的调用链观察通道；
  `test()` 同步完成后再 step 是假 Pass（step-after-done），改 promise_test。

## 下一步（s159 候选）

①stylesheet-not-removed-until-next-stylesheet-loads 1F（R5114 合成条目 href 通道
——styleSheets 合成面）；②候选①动态元素 computed style 大面（`__zwSelector` 只
属静态元素——handle 元素 gCS 恒 ''，牵 display fallback 表与宿主查询通道，需先
定刀口）；③shadow DOM 级联面（link-rel-attribute shadow 案——shadow root 级联
收集缺失）；④全域 .css fixture 缺口 sweep（lib.sh 2026-10-29 前目录族）。
