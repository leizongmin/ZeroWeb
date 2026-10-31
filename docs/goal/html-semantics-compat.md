# HTML Semantics 兼容 — html/semantics 语义面（主攻大域）

**版本**: v2.0
**日期**: 2026-10-26（v1.0 立项 2026-10-08）
**状态**: Active——M2 切片 162 施工收口（style 元素 load 事件面三触发臂——勘根（候选③ style_load_event 1F 入场，定谳该面整体缺位三形态：解析静态首派（loadCount 0≠1）、动态插入/子变更（style-load-after-mutate await style.onload 永挂文件级 Timeout）、textContent 变更（style_load_event 变更段 await listener 永挂）；spec update a style block 派发半——sheet 就绪即 fire load、更新 queue a task 使同 turn 后挂监听可收、type eligible 门 + 连接门）；施工 2 文件 +100 rust 零改动（part03 共享 helper `_zwStyleBlockQueueUpdate`——首派/变更双印记去重（变更臂同 tick 去重=spec 单 task 更新）+ setTimeout(0) task 尾派发 + fire 点现读 type/连接门；解析静态臂 = `_zwStylesheetStaticScanUpto` 越界形快照 style 全量首派；part04 appendChild trap STYLE 父/子双臂 + textContent setter 写后臂）；验证：node --check SYNTAX_OK + 定向 style_load_event Fail→Pass、style-load-after-mutate Timeout→Pass + document-metadata 目录 join 对 s161 full4 F2P 1/P2F 0 + 2 文件级 T→P（181P/31F/12T→184P/30F/10T）+ 全量 join 对 full4 (key,state) 对 F2P 1（靶案）/P2F 0/P2T 0/T↔P 0，LOST 77 全归因环境截断（交互时序域 74 + zz-s161-probe 探针已删）+ GAINED 6（3 靶族 Pass 首报 + 1 环境恢复 + 在册 flaky 域 1F+1T 本轮报出）；FIXME 新记 type/title 属性变更臂与非 textContent 子变更臂未挂（语料无形态）；并发注记：切片号 162 与并发会话 per-sheet styleSheets 枚举 WIP 撞号（其注释亦标 s162 未落账）——本刀先落账保号，其收口时顺延重编号 s163；账本见 [evidence/2026-10-30-m2s162-style-load-event.md](html-semantics-compat/evidence/2026-10-30-m2s162-style-load-event.md)）；承前：M2 切片 161 施工收口（动态元素 gCS 通道 + 外链 media 门三层——靶面断言级翻绿 + 回归修复 style-system UA display 命名空间门 + corpus 零实质位移：勘根（s158 定谳动态元素 computed style 根缺位施工——handle 视图 = `parse_html` + `apply_dom_mutations_full` 重放 `[applied..count)` + `compute_styles_for_doc` 槽化（HANDLE_VIEW_CACHE 10-tuple 代际键），宿主回调 `__zw_get_computed_style_by_handle`，shim gCS `__zwHandle` 回落臂 display/position/float 全通；外链 media 门三层——Rust 静态路径 `extract_stylesheet_links`（href+media 原文）+ webview `resolve_external_css`/`async_load begin_stylesheet_fetch` 加载期门（非空非 all → s160 同款 1280×800 evaluate，no-match 跳过不取回不入级联）+ shim overlay `_zwSsRecomputeExt` 逐表 `@media` wrap + setAttribute/removeAttribute/IDL reflected 三触发臂（`_realTag` 动态 link 回落 DIV——handle-only TAG 容忍放行，重算幂等误触发无害）；探针定谳 Rust 静态槽 media 无知必须同门，否则加载期一次写入永远赢过 overlay 翻转）；施工 11 文件（rust 7 + shim 4）；验证：check/fmt/guarded-clippy workspace + shim 14-part `node --check` 111228 行全绿；探针（静态 link media=none→all）门半+翻转半双 Pass 验后删；靶面 stylesheet-media-change-no-fetch 断言 1 翻 P（失败点移断言 2 = 动态 link query_all 不可见记账）；the-link-element 113P/2F 双 F 既有 + the-style-element bad-set ⊆ s159（s160 11F→P 保持）+ style_non_matching_media 保持；全量案级 join（full3+full4 对 s159 (key,state) 对 sort -u）F2P 恰 s160 的 11、P→F 唯一实质位移 **SVG noscript 已修**——style-system `ua_default_display` 注入 `is_html_ns` 门（HTML 渲染规范 UA 表 real UA 经 `@namespace xhtml` 限定，Firefox html.css bug 1470150 先例；此前 localname 盲匹配误隐藏异命名空间同名元素，动态 gCS 暴露真实计算值翻红；修复静态渲染与 gCS 双通道同源生效）修复后 non-html-noscript 复 P 终账 full4 7093P/803F/91T；dialog-popover-closedby-complex 5 子测位移 A/B 定谳环境漂移非回归（带/不带刀口 style-system 改动 solo 跑同 16 bad、s159 自身同文件 12 bad、当日 full1→full4 呈 4→11→16→16 时序漂移、文件无 SVG）；FIXME 新记四项记账：动态 link 对 `__zw_query_all` 不可见（R51c 查询视图不应用 handle 链——overlay 枚举缺口，per-sheet 结构化通道 = s161 候选①设计面）、外链 css 槽位级联序先于页面 style 表（同特异性后者胜，文档序翻转需全语料 A/B 独立切片）、跨批 handle 边界（computed_style_by_handle FIXME 已注）、layout collect_items 兜底 `ua_default_display` 无命名空间门（无失败证据暂不扩面）；media 求值视口 1280×800 FIXME 承袭；账本见 [evidence/2026-10-30-m2s161-handle-gcs-media-gates.md](html-semantics-compat/evidence/2026-10-30-m2s161-handle-gcs-media-gates.md)）；承前：M2 切片 160 施工收口（style 元素 sheet 三门——media/type/disabled 级联应用门 + styleSheets type 门统一：勘根（断言机制全静态元素宿主 gCS 通道——缺口纯在 collect_stylesheets 零门；反应管线现成——media/type 入 _REFLECTED_STRING_FLAT、disabled 显式布尔分支，IDL 写产 SetAttr mutation → style_version 换代 → apply_inline_style_overrides 四类全 apply → is_style_or_meta 判脏重收集，唯一缺口即收集期三门；media 不入注册面 spec 钉——style_non_matching_media「media=unknown 长度 1」当前 P；type 精确匹配 spec 钉——style_type_html ""/TEXT/CsS 应用、" text/css "/charset 参数不应用）；施工 3 文件净 +61/-10 （pipeline collect_stylesheets_with_parsed_external 三门——type continue 未创建 / media parse+evaluate 1280×800 screen 默认全不匹配 continue / disabled 属性存在 continue + part06 styleSheets getter type 门 lw 读过滤 + part04 sheet property essence→exact 统一）；入场候选①漂移钉死（selection-pointer 21F 持久定谳——第三次复跑跨 14h 跨 engine 提交边界，同树同时段 A/B 铁证维持；select-event 270P/9F 全在册、focus-dynamic 80P 全恢复=环境截断确认）；候选④销案（18 缺失 .css 与 793F 交集 0、嵌套 0——拉取反有 error-expect 案 P→F 回归风险）；候选②勘根收窄（meta color-scheme 24F 三重栈——动态 gCS + used-scheme 传播 + 系统色 CanvasText 静态黑）；验证：node --check SYNTAX_OK + fmt + guarded-clippy workspace 全绿；靶面 the-style-element 28P/2F/0T（9 靶 F 全翻绿）+ 搭车 2 案合法收益（CSP tentative type=module CSS Modules 形态）；回归 LinkStyle 5/5 + the-link-element 112P/2F/0T = s159 基线逐位一致 + interactions 6P/3F 三预期 F 保持 + style_non_matching_media 保持；全量 join 对 s159-full1 **F2P 11 / P2F 0**（其他位移全为交互时序域全量 Timeout 环境截断——dialog-closedby 6/6、popover-complex 24P/16F、select-event tel 45P 定向全恢复）；渲染门禁 make reftest 724/725 与 stash 基线逐位一致（唯一失败 float-shrink 既有 A/B 钉死）+ product-smoke exit 0；账本见 evidence/2026-10-30-m2s160-style-sheet-gates.md）；承前：（合成条目 href 通道——stylesheet-not-removed-until-next-stylesheet-loads F→P：勘根（案面双时序断言——test 同步段：初始 fetch 在飞读旧生效 href、href 变更新 fetch 排队旧值不动；R5114 合成条目 href=null 硬编码 null.includes 崩；spec sheet 生命周期近似——processing 创建即有 href 不等 load、load 完成替换、error 替换未发生保持旧值）；施工 2 文件净 +160 rust 零改动（part03 五件——`_zwSsSheetHrefMap` 生效 href 状态表（键口径同 `_zwExtSheetTexts`）+ `_zwSsSheetHrefFor` 查询口（live 文档序首个 rel=stylesheet link 首命中否则 null——合成 blob 单条目 href 映射取级联首序）+ fetch 发起创建写（fired 幂等门后挂解析 URL 不等 load——断言 2 时点可读）+ preload 复用臂创建写（转换面零网络生效）+ done load 覆写 error 不动（最小状态机）+ rel 移除臂删键；part06 五处——`_makeStyleSheet`/`_makeSheetBuild` 第 3 参 externalHref + `href: externalHref || null` + R5114 limitation 收窄（per-link 文本拆分仍记账）+ styleSheets getter 每访问调查询口 live 传值）；验证：node --check 110795 行（净 +160）+ 靶面定向 F→P（双断言时序逐点过）+ 目录级 the-link-element 112P/2F/0T（两记账 F 保持、link-multiple 6/6 保持、0 Timeout）+ 全量 join 对 s158 F2P 1（靶案）+ **域外环境漂移面 A/B 铁证与刀无关记账**（selection-pointer 21 P2F——A/B stash 基线同态 Fail、s157/s158 两轮终账同域 Pass、语料/runner 资产零变化、07:10 并发 css 流 checkout 活动时间线吻合、空闲态复跑仍 Fail = 持久漂移；select-event 91 + focus-dynamic 12 全量 Timeout 截断——定向 270P/80P 全恢复；dialog 杂项 2+2；全部在交互时序域与刀口零交集，机器冷却复跑钉死留下轮入场）；FIXME 解除——R5114 合成条目 href（per-link href 通道落地）；新记 error link sheetHref 不删回归面说明（error 后查询口返失败 link 解析 URL=虚构生效 href，全量未暴露不收紧）+ 新记交互时序域环境漂移；账本见 [evidence/2026-10-30-m2s159-link-sheet-href-channel.md](html-semantics-compat/evidence/2026-10-30-m2s159-link-sheet-href-channel.md)）；承前：M2 切片 158 施工收口（link rel 状态 ↔ 外链 CSS 级联活链路 + rel ASCII whitespace 分词——link-rel-attribute light DOM + tokenization 2F→P 全量 join 零位移：勘根（候选①重定谳——stylesheet-media-change-no-fetch 失败根因是**动态元素 computed style 根本缺位**（createElement 产物 handle-based proxy 无 `__zwSelector`，shim getComputedStyle 恒 ''），刀口另立；候选④重定谳——R5114 合成条目 href=null，s159 候选；②③根因链四件叠加——语料 .css fixture 缺失（fetch_dir_html 2026-10-29 才加扩展）+ rel 变更无级联反应（外链槽 load 期一次装填无页面期臂）+ removeAttribute rel 无触发臂 + \v 分词偏差（`\s+` 含 U+000B）；机制发现——gCS 代际缓存键含 css 分量（P9-9），槽文本变化自动换代重收集，只缺 shim→槽触发臂）；施工 3 文件净 +131/-7（engine `__zw_set_external_css` 回调装 ACTIVE_EXTERNAL_CSS 同槽 + part03 四件——`_zwExtSheetTexts` overlay 表 / `_zwLinkRelTokens` spec ASCII ws 分词 / `_zwSsRecomputeExt` 覆盖门重算（query_all 文档序 + handle 反查键 + 全有才替换）/ `_ssVisit` 第 5 参文本收集 + rel 门三分支（stylesheet 路径 / 无 token 在册清两表 + recompute / 静默）+ 静态扫描臂 `noOverlay` 存表不下发 + part04 removeItem rel 移除臂；语料补拉 link-rel-attribute.css/neutral.css）；归因轮两层——①106 族 6 案 P2F 定谳**无分号 @import 残留吞规则**（webview s89 教训 shim 层双现：join('\n') 使 @import preamble 过换行吞内层首规则——host 回调只读绑定不可 hook（sloppy 赋值静默丢弃）证据作废，改 shim global 包装探针钉 rcCalls=2 证明下发发生；修法 recompute 与 join 双层 `';'` 分隔）；②details name-attribute 文件级 Timeout（基线无）——testharness.css 等 6 link 的静态 done 走 overlay 下发、文本与 base 逐字节不等 → 每页一次多余全文档级联换代 → 贴边页超时；修法 noOverlay 门（扫描臂存表解锁覆盖门但不下发，税=0）；验证：node --check 110635 行 + fmt + guarded-clippy 全绿 + 靶面 2F→P（shadow 案预期 F 记账）+ 106 族 10/10 + link-multiple 6/6 + 全量案级 join 对 s157（8082 对 = 8082 对，7205P/793F/87T）F2P 2/P2F 0/LOST 0/GAINED 0 全域零位移（name-attribute 18/18 恢复）；FIXME 新记 overlay 与 base 文本语义差两面（@import 追加非原位展开、url() 未绝对化）+ link media 门未建模 + 生产 renderer SetDomSnapshot 不恢复外链槽；账本见 [evidence/2026-10-30-m2s158-link-rel-ext-css-cascade.md](html-semantics-compat/evidence/2026-10-30-m2s158-link-rel-ext-css-cascade.md)）；承前：M2 切片 157 施工收口（尾随静态 link 触发收口——link-load-event.html F→P 全量 join 零位移：勘根（根因升级定谳——失败根因非「stylesheet 未 delay load」而是**尾随静态 link 零触发**——s152 静态扫描按 script ordinal 触发「script 之前的 link」，最后一个 script 之后的尾随 link 从未被任何臂触发，link.onload 恒不 fire；trickle 滞留假设被证据否定——runner 对 stylesheet 忽略 pipe=trickle MIME 面 + 尾扫窗折叠 trickle）；施工 3 文件净 +61 两件套 + 归因轮门控修订（webview.rs 脚本流末尾补扫 _zwStylesheetStaticScanUpto(2147483647)——序号越界 shim 侧收敛为全部 link 游标幂等；part03 _ssVisit 在飞计数 _zwSsLoadPending——每层恰一次 settle 包装归零，@import 内层递归逐层自计，fetch 同步抛按取回拒绝归零派 error，preload 不计数；testharness.rs loadStep5275 第四谓词 __zwStylesheetFetchPending；归因轮——全量 link-multiple Timeout 复现窗定位（定向/目录级不复现、document-metadata 级复现）→ 语料探针定谳案 1-4 竞速窗溢出 ~440ms/案（ZZ 探针 9757ms）→ stash 基线 A/B 基线自身 9334ms 溢出 ~1.33s 距预算仅 0.67s——贴边固有，本刀边际 +423ms → 谓词按尾扫触发印记 _zwSsTailFired 门控（无尾随 link 页面谓词恒 false 与基线全同）→ 修订后全量 link-multiple 6/6 全 Pass）；验证：node --check 110462 行 + fmt PASS + 靶面定向 Fail→Pass MAKE_EXIT=0 + 全量案级 join 对 s156（8082 对 = 8082 对，7199P/796F/87T 保持）F2P 1（靶案）/P2F 0/LOST 1（即靶案旧键）/GAINED 0 全域零位移；FIXME 新记 link-multiple 全量贴边（案 1-4 竞速窗 ×4 名义 8s 固有溢出 ~1.3s，余量 0.7s 噪声敏感——s155/s156 全量通过属侥幸）；账本见 [evidence/2026-10-30-m2s157-trailing-static-link-load-hold.md](html-semantics-compat/evidence/2026-10-30-m2s157-trailing-static-link-load-hold.md)）；承前：M2 切片 156 施工收口（LinkStyle sheet 创建门收口——styling/LinkStyle.html null 侧案 F→P 5/5 全绿：勘根（探针一轮钉态——style2（alternate+空 title）/style3（alternate 无 title）/style4（style type=text/html）sheet 应 null 现 OBJ，disabled 全 boolean false 无需动面；getter 定位 R3036 sheet 臂 LINK 仅 rel 正则门，spec 创建条件未建模；位移面收敛——语料 .sheet 读取点 the-style-element 族无 type gate 跳过，type 变体在 reftest 渲染面）；施工 part04 sheet getter 净 +28 两门（type essence 门 STYLE/LINK 共用——type 属性存在且 essence 非 text/css 返 null，TeXt/CsS 小写归一创建、text/cſs 不创建，缺省不设门；LINK alternate 空 title 门——rel 含 alternate 且 title 空串含缺失返 null，style6 非空 title 照常创建）；验证：靶面 5/5 全 Pass + 全量案级 join 对 s155（8082 对 = 8082 对，7198P/797F/87T → 7199P/796F/87T）F2P 1（靶案）/P2F 0/LOST 0/GAINED 0 全域零位移；node --check 110429 行 + Rust 面零改动；FIXME 在册 sheet 创建未建模 fetch 完成状态（语料 window load 后断言豁免）；账本见 [evidence/2026-10-30-m2s156-linkstyle-sheet-creation-gates.md](html-semantics-compat/evidence/2026-10-30-m2s156-linkstyle-sheet-creation-gates.md)）；承前：M2 切片 155 施工收口（rel=preload fetch 面 + rel→stylesheet 转换 preload cache 复用收口——link-multiple-load-events 文件级 Timeout 键消 6/6 全 Pass：勘根（Timeout 形态先解——诊断行「2 test(s) unreported pending:2」定谳 testharness 全局默认超时收文件，案 1-4 幂等语义正确烧满 2s×4=8s 吃掉全局预算、案 5/6 到点前强收（s153「页级 10s 时序必然」注记机制来源），修复须毫秒级完成非纯计数面；触发面缺口——rel 变更无任何臂 + helper rel 门只认 stylesheet；as IDL 反射已有；位移面收敛——html/semantics rel=preload 唯一 = 靶面案 5）；施工 3 文件净 +60/−3（part03 rel 门双旗标分流 + preload 取回臂——as 非空门 + 印记去重 + 页面 fetch 完成播种 `_zwPreloadCache` 派 load/error + sheet 分支印记门前复用臂——cache 命中零网络 setTimeout 派 load；复用臂先于印记门防自伤——印记命中点静默返语义保住，sheet 取回不播种，双 load 断言面零波及；part04 setAttribute rel 臂 + part05 IDL rel 臂——TAG 门甄别 helper(null)）；验证：靶面 6/6 全 Pass MAKE_EXIT=0 + 定向 113 键 join LOST 1（Timeout 键）GAINED 2 零位移 + 全量 join（s154 8084 行/8081 对 7196P/797F/88T → s155 8085 行/8082 对 7198P/797F/87T）F2P 0/P2F 0/other 位移 0 全域零位移；node --check 110401 行 + Rust 面零改动；FIXME 新记 preload 资源级共享缓存/as destination 分型/rel 双类型分流不建模（语料无形态）；账本见 [evidence/2026-10-30-m2s155-preload-fetch-rel-conversion.md](html-semantics-compat/evidence/2026-10-30-m2s155-preload-fetch-rel-conversion.md)）；承前：M2 切片 154 施工收口（adopt/innerHTML 身份双表示收口——document-without-browsing-context 2F→P：勘根三轮收敛（探针 wrap 法——包 _zwStylesheetMaybeStart/fetch 实证 createElement 控制组正常、adopt 形 4 参臂已带章不 fetch、innerHTML 形两臂触发均不 fetch；breadcrumbs 定案：案 1 = R4004 属性回填依赖 getAttributeNames——DOMParser 解析树 M 节点无该方法（探针 THROW 实证）→ 宿主属性表恒空 → handle 读 ''（非 null）→ `_ssReadAttr` `== null` 回退不触发 → rel='' bail；案 2 = 身份双表示时序缺口——innerHTML 物化身份分裂（汇流臂记账时 addedNodes 还是解析树 plain M 节点永不盖章 → TAG 门 `_realTag(null,null)` 恒 DIV bail；物化产物另一 JS 对象盖章走 attr 臂但 `_zwNodeParent` 反链注册滞后于属性写 → 连接门 bail））；施工 part03 helper 净 +59/−4 三件（`_ssReadAttr` 回退判据 `== null` → falsy（宿主 absent '' 与解析树 null 同判 miss）；身份键取回印记表 `_zwSsFiredMap`——同 (sel,handle) 同解析 URL 单次发起，保住先赋 href 后插形防双请求；单发身份键重查 `_ssScheduleRetry`——TAG/连接门 bail 时 setTimeout(0) 原身份重跑 + `_zwSsRetryDone` 单发印记（v1 elObj 现读身份重查因身份分裂自弃失败——改身份键设计））；验证：定向 the-link-element 113 键 join 对 s153 基线 F2P 2/LOST 0/其他位移 0 + 全量案级 join（8080=8080 对，7193P/799F/88T → 7195P/797F/88T）F2P 2/P2F 0/other-diff 0/LOST 0/GAINED 0 全域零位移（fetch API/XHR 面、link-multiple、s152 静态扫描面全保持）；node --check 110325 行 + Rust 面零改动（clippy/fmt 零波及）；FIXME 新记 sel 重解析同槽同 href 印记失配边角（语料无形态）；账本见 [evidence/2026-10-30-m2s154-adopt-innerhtml-stylesheet-trigger.md](html-semantics-compat/evidence/2026-10-30-m2s154-adopt-innerhtml-stylesheet-trigger.md)）；承前：M2 切片 153 施工收口（跨源 stylesheet 通道 CORS 豁免——上游 no-cors 语义归位：影响面盘点刀法取舍（runner 静态臂普发 ACAO 否决——上游 wptserve 行为篡改 + fetch 负语义面风险；shim helper 通道豁免采纳——上游 `<link rel=stylesheet>` 无 crossorigin 是 no-cors 请求不做 CORS check 跨源无 ACAO 照常 load，shim fetch(cors) 仿真引入上游不存在的 check；波及面收敛 helper 取回链、css-module module 通道与页面 fetch()/XHR 零波及）→ 三轮定向勘根转折：一轮揭内层 mixed 缺口（data: 外层的 @import http: 内层此前碰巧由 fetch CORS check 顶住双错凑巧 Pass，豁免后 2 案 P→F → 逐层 mixed 门补齐）+ 二轮揭 `_zwStylesheetImports` 正则截断单根（url() 臂捕获类 `[^'")]` 排除 `)` 把 `pipe=status(404)` 截断 → 404 退化 200 → 4 案误 load → 带引号臂回引边界修复）+ 三轮揭 @import 一层递归边界（双层案第二层不可见 3 案误 load，s151「语料无多层形态」勘根有误 → `_ssVisit` 递归化深度 3 上限兜底环）→ 施工 shim 2 文件净 +42/−13（part01 `_zwStylesheetChannelDepth` IIFE 局部旗标页面不可见 + `_zwFetchApplyFilteredResponse` 第 6 参 check/过滤双门 + Impl 同步段快照进闭包含 trickle 异步臂；part03 取回段递归化 + 逐层 mixed 门 + 正则回引）；定向 the-link-element **7 案 F→P LOST 0 零 P→F**（link-load-error-events 全文件 17/17，基线 11P/6F）+ 全量案级 join（对 s152）**真修复 7 案零回归**（7186P/806F/88T → 7194P/799F/88T，P→F 0、Timeout 位移 0；GAINED 1 = link-multiple media 在册少报键复现）；node --check 110270 行 + Rust 面零改动（clippy/fmt 零波及）；FIXME 在册：crossorigin 属性形态未分型（语料无）、@import 环检测未建模（深度上限兜底）；账本见 [evidence/2026-10-30-m2s153-crossorigin-stylesheet-cors-exemption.md](html-semantics-compat/evidence/2026-10-30-m2s153-crossorigin-stylesheet-cors-exemption.md)）；承前：M2 切片 152 施工收口（静态 link rel=stylesheet 文档序交错触发——Face C script-blocking 时序面收口：v1「全部脚本前一次触发全部 link」语义错误勘根转折（spec 只阻塞 link 之后 pending script——前置 `var t` 声明 script 被推迟 → onerror 内 t.step ReferenceError）→ v2 交错触发三件套净 +103/−1：shim part03 `_zwStylesheetStaticScanUpto(scriptOrdinal)`（`_zwDocAllElements` 文档序交错表单次建 + scriptPrefix 预计算 + fired 前缀游标单调推进 O(1)/次，逐 link 复用 `_zwStylesheetMaybeStart` 全门）+ shim part01 fetch wrapper trickle 仿真扫描窗折叠（`__zwScanCollapseTrickle` 旗标同步段置位即清——探针链 8 文件五轮二分定谳单根：`pipe=trickle(d3)` 的 setTimeout(3000) 把 load 推迟到后随 script 断言之后，上游 parser 阻塞 3s 等待语义在 runner 无 parser 阻塞形态下不可承载；fetch API 面 timeout 竞速用例零变化）+ webview.rs 脚本循环内每脚本体前独立 execute scanUpto(script_index)（序号与 shim getElementsByTagName('script') 文档序在册对齐——extract_page_scripts_indexed 过滤前递增口径；独立求值保证 `__zw_fetch` 同步契约微任务清空先于脚本体）；定向 fired-before-scripting **2F→2P** + 全量案级 join（对 s151）**真修复 2 案零回归**（F 新增 0、P→F 0；P 消失 1 案 link-multiple media 定向复验稳定 P——s151 在册负载 flake 同形）；clippy 零告警 + fmt 过 + node --check 110217 行 + zero-webview lib 809/9F 与 zero-engine lib 3600/9F stash 基线逐字一致（9F 既有环境依赖）；conditionally-block 整文件维持挂账（delay=2 同步契约不可仿真 + styleSheets blob 单条 R5114 分条面）；账本见 [evidence/2026-10-30-m2s152-static-stylesheet-docorder-interleave.md](html-semantics-compat/evidence/2026-10-30-m2s152-static-stylesheet-docorder-interleave.md)）；承前：M2 切片 151 施工收口（stylesheet MIME/nosniff 判定矩阵——Face B 收口：勘根证伪 s150「48P/12F」口径（实为 42P/36F：`<link>` 形 12F 全部期望 load 案假阳反向、`@import` 形 24F 全部期望 error 案内层缺失；quirks 宽容分支因 shim compatMode 常量遮蔽从未激活）→ 五件套施工净 +155/−8：宿主桥 `__zw_get_compat_mode`（live Document quirks_mode 求值，`document_compat_mode_string()` 语义封装导出）+ shim part06 compatMode 常量改 getter（桥读白名单回落）+ shim part03 `_zwStylesheetJudge`（nosniff 严检/wrong-CT standards-strict quirks-宽容/broken-CT 无头放行——css-syntax §stylesheet-content-type-error 上游判式逐腿同构）+ `_zwStylesheetImports`（三书写形一层递归展开）+ fetch 回调改造（ok→判定→text() 展开 @import 内层逐个 fetch 同判定链 pending 聚合，任一 error → 外层 error）+ runner css.py 端点（content_type/nosniff/cors 参数 + ACAO/ACEH `*` 通道妥协——shim fetch 默认 cors 模式跨源 reject + cors 过滤剥非 safelisted 头两面）+ 页锚谓词 `link_style_error_case_uses_http_page`（三文件 http 页锚，referrer 先例同构——上游 wptserve 无 mixed 阻断、页 https 时 CO 案被 s150 mixed 门拦死）；定向三文件 **74P/0F/0T**（36F 全转绿）+ link-multiple 复验 4P+文件 T（s150 同形）；全量案级 join（对 s150 合并树）**真修复 41 案 F→P**（link-style-error 36 + link-load-error-events 5——non-CSS 内层 404 传播/http Import 跨源 error 传播）+ **假阳消除 2 案 P→F**（https Import 两案：s150 内层未取外层 data: 恒 load 凑巧 Pass，本刀内层真取 `https://wpt.test:8444` 跨端口撞 CORS check 静态臂无 ACAO → reject 挂账）+ flake 形态（select-event 26P+文件 T 键消、link-multiple media 1P——在册负载面非本刀位移）+ 文件级键 −3F−1T；clippy 零告警 + runner bin **265/0F** + fmt 过 + shim 拼接 node --check 110138 行；账本见 [evidence/2026-10-29-m2s151-stylesheet-mime-nosniff-matrix.md](html-semantics-compat/evidence/2026-10-29-m2s151-stylesheet-mime-nosniff-matrix.md)）；承前：M2 切片 150 施工收口（动态 link rel=stylesheet 事件面——runner headless 路径动态 link 零 fetch 零事件（探针 p1–p4 neither event 实测；`resolve_external_css` 仅静态收集、浏览器 AsyncPageLoad 对照面完整）→ shim `_zwStylesheetMaybeStart` helper（part03，s124 modulepreload 结构模板）：TAG 门（plain handle 形 `_realTag` 回落 DIV 放行）+ 连接门（`_zwMutationInDoc` 父链爬升）+ attr 读通道（sel/handle 宿主桥 + plain handle 形 elObj getAttribute 回落）+ rel 空白分词含 stylesheet + href 非空 + **幂等门**（prevHref 解析后绝对 URL 比较，null=插入总取）+ mixed content 门（http 子资源 https 页 → setTimeout error，webappsec should-block-fetch）→ fetch `ok ? load : error`，`_dispatchWithBubble` 元素站派发；part01 s111 两臂 LINK 甄别修正（仅 LINK，SCRIPT 不动）+ 插入触发、part04/part05 setAttribute href 写前捕获旧值 + 写后触发（写前捕获是幂等基准正确关键）；runner stylesheet.py 端点 per-case 计数表（take-and-clear 同形：`count` 读不回填、否则 take+1 回填返 `body {color: red;}`）+ 上游字节 good.css/style.css；定向 link-multiple 案 1–4 **4F→4P**（幂等精确 count=1/count=0 全过）+ alt-001 **T→P** + link-load-error-events 文件 Timeout 消（8 案 Pass）+ link-style-error 三文件事件面活化 **48P/12F**（MIME/nosniff 判定缺失为预期边界 Face B）+ bad-mime 2P + non-OK-status 1P；全量 **7148P/847F/88T** join（对 s149 7993 对）lost 18 全审计（4F→P 幂等案 + 6F→P urlencoded2 归档观察——零机制关联、三跑稳定、s149 全量两次撞 test-guard 4GB 限内存压力伪红假说，encoding 流对账核销 + 8 文件级 Timeout 键消）**P 键 0 = 零真 P→F**；**s149 单测回归顺带修复**——runner bin 263/1F 定谳 s149 引入（父提交 6e9f0a5ac 绿；`__zw_harness_complete` 前移丢 mock completion-先发形态的结果行，真实 testharness 无该活路径），修 mock 为真实顺序 + 新增早收形守卫测试，runner bin **265/0F**；clippy workspace 零告警 + fmt 过（既有 text_encoding.rs 漂移归口 encoding 流 a828dfac4 提及不修）；账本见 [evidence/2026-10-29-m2s150-dynamic-link-stylesheet-event-face.md](html-semantics-compat/evidence/2026-10-29-m2s150-dynamic-link-stylesheet-event-face.md)）；承前：M2 切片 149 施工收口（runner 终态判定早收形收口——harness 页级超时早收形 = 文件级 Timeout 族 91/96 单根：勘根推翻「per-test timeout 缺失」假设——pinned testharness **无默认 per-test 超时**（`settings.test_timeout=null`），真实兜底是 Tests 构造期 arm 的**页级 10s 超时**（meta long 60s）：Tests.timeout 把全部测试强制标 COMPLETE 后 notify_complete 同步发 completion 回调，未报测试 done() 此后 no-op → partial rows 即终量、num_pending 恒 >0；runner `terminal_harness_state` 的 phase4+pending0+tests==results 配平永不满足 → 探针空转至 30s 案超时（s148 全量 96 文件级 Timeout 键 **91 例 state.phase==4** 实测 = harness 已自收而 runner 不认）；施工 testharness.rs 3 处净 +26/−3——probe JS `__zw_harness_complete` 检查前移 timer grace（completion 后 done/step 全 no-op 不可能有新行）、终态判定新增 `complete && phase==4` 臂、早收形保留文件级 Timeout 键（upstream wptrunner 对 harness status=TIMEOUT 记 TIMEOUT+partial rows 同形，join 零位移）+ 消息准确化；定向 link-multiple-load-events 行集逐字节同形、**墙钟 30s→10.1s**（run=9995ms = 页级超时即收）+ input-type-change-submit 20/20 零回归；全量 join 对 s148 7993 对**精确守恒**——唯一位移 modulepreload-referrerpolicy 2F→P（在册全量负载 flake 自然复绿）零真 P→F 零键增减，早收 91 = 26（有行+补标记）+65（零行既有路径）+5 phase==2 原路径（meta long × corpus 30s 通道放宽失效独立挂账）；clippy workspace 零告警 + runner bin 264/0F（s148 基线 263/1F，trend_csv_check 已绿并行流归口）+ fmt 过 + 零 shim/.js 改动（注：该 264/0F 声明经 s150 复验证伪——HEAD 8f4228c5d 实为 263/1F，waits_for_active_step_timeout 单测回归已由 s150 顺带修复至 265/0F，见下）；承前注记：上一 session 429 中断前已落 4 个 s149 勘根探针（timer/ladder Pass、trace/ht 文件 Timeout——与本轮勘根同向），干净全量前已删）；承前：M2 切片 148 施工收口（input-type-change-submit 挂死根收口——`_validityState` type 读 sel 分支快照盲区（R2995 IDL 写只落 lw 层），s136 激活门同根第 2 处：click handler 内 `input.type='submit'` 后默认动作约束校验仍按旧 email/url state 判 typeMismatch → invalid 中止提交 → loadPromise 挂死 → 文件级 Timeout（state pending:17 = 案 5-20 从未启动——promise_test 串行启动观察记录）；上游无 .ini 全绿 + spec §4.10.5.2.6 typeMismatch 仅 email/url state、校验时 type 已 Submit Button（§4.10.5.4 + §4.10.21 barred）；探针四案三分辨（p1 detached handle 分支 valid 翻真 / p2 sel 分支 invalid 仍派发 / p3 不导航 / p4 非 submit 激活 no-op 旁证）→ 单点归因视图分裂；施工 part03.js 1 处净 +9——sel 分支换 `__zw_get_attr_lw` 融合读（s136 同形，handle 分支不动）；定向 input-type-change-submit **20/20 全 Pass**（修复前 3 Pass + 文件 Timeout）；全量 join（对 s147 7977 对）：s148 **7085P/812F/96T** = 7993 对，lost 5/gained 21 全归因**零真 P→F**（knife Timeout→17P + form-double-submit-input-type-change **2F→2P 同根连带** + modulepreload-referrerpolicy 2P→F 全量负载 flake 定向 7/7 复绿——s124 族在册先例）；门禁 clippy 零告警 + node --check 109736 行（HEAD 109727+9）+ engine lib **3593/10F 名单逐字节同 s147** + runner bin 263/1F（trend_csv_check 归口不变）+ 零 .rs 改动）；承前：M2 切片 147 施工收口（刀 B「runner 同键多行」根收口——testharness 注册 API 子文档 realm 隔离，同路径自导航递归注册泄漏单点收口：勘根定谳「多跑」非 runner 循环重跑——`run_html_semantics_cases` 每文件只执行一次，同键多行 = 同名多次注册（s141「runner 执行 3 遍」旧解读修正）；泄漏通道 = 同路径自导航递归——form-submit-action「missing form action」提交 URL = 文档自身地址 → target iframe 装载父页副本 → 嵌套副本脚本体裸标识符 test()/async_test() 落 globalThis（宿主 harness）→ tests 数组重复注册 ×N → 嵌套 populateForm/再提交递归（s146 全量 50 行 = 1 清洁 Pass + 4 级联 Fail 实测）；首遍挂/键集漂移同根（嵌套副本 loadPromise 挂死 → 永不 done → 文件级 Timeout + 注册子集上报）；施工 part01.js 1 处（+36/−2）——R206 `_zwRunIframeScripts` 页面段求值前置影子声明：激活门 = 嵌套文档路径（去 query/fragment）== 顶层 case 路径，18 个 harness 注册 API 名 Function 作用域影子声明、声明式委派 window[name]（调用时取值防 pre-harness undefined）、子窗无该名 ReferenceError 中止（per-part try/catch 接住，副本页面段首个注册调用即止、populateForm/再提交级联源头掐断）；异路径 helper iframe 承重契约不动、exportSuffix206 不含影子名无自委派环、页面段同名声明照常覆写；定向 form-submit-action **10/10 全 Pass 单清洁执行**（s146 前态 40F+10P 五注册）+ src-then-lazy-load 8/8 Pass（文件 Timeout 未复现）+ link-multiple media 键 Fail 确定性复现（s146 挂起吞没键解除）；全量 join（对 s146 7986 对）：s147 **7071P/812F/97T**（7980 行，Δ = −40 form-submit-action 级联 Fail 行 +1 link-multiple media 行），键对 7986→7977 位移 13 条全归因**零 P→F**（lost 11 = form 10 Fail 对真翻红 + src-then-lazy 文件 Timeout 对 T→P；gained 2 = src 第 8 键 Pass + link-multiple media Fail）；门禁 clippy workspace 零告警 + shim 13-part node --check 109701 行 + engine lib 3592/10F（9 名单逐字节不变 + details 在册间歇红）+ runner bin 258/1F + 零 .rs 改动 + grep -a 核验 _zwRealmShadow 进 release 二进制）；承前：M2 切片 146 施工收口（iframe.src pending 导航门——algorithm t10 文件级 Timeout 收口 + t11 连带落定：入场甄别候选① accept-charset 18 案**归口 encoding 流 M1**（form-url-encoding-compat 立项控制面明文「25 号流不重复施工」，s145 挂账承袭时未对照、本轮修正移出候选池），取候选②；勘根探针五轮二分定谳——挂点非装载链（dummy.xhtml 静态可达 + 直调 reload 成功 + loadPromise 通道 t1-t9 旁证），是 set trap 导航门 `_zwIframeNavigationConnected` sel 臂依赖 `__zw_contains`（R102 live_aware live_ok 硬编码 true——live/cached 双路径均不重放 pending structural）对同任务 insertAdjacentHTML 产物恒判 detached → `iframe.src=` 导航钩子永不触发；施工 part01.js 1 处（sel 臂前置 `_zwPendingParsedForSel` 签名命中即 connected——pending 已插入形态对 spec「insertion into a connected tree」成立，常态零开销，handle 臂不在病灶不动）；定向 algorithm **11/11 全绿**（t10 Pass + t11 被挂死阻塞案连带落定）；定向目录 **279P/63F/0T**（vs s145 277P/63F/1T，join 位移恰 3 条零坏位移）；全量 **7070P/851F/98T**（结果行守恒 8019、键对 7986 守恒）位移 6 条全归因——主刀 3 条（algorithm 文件 Timeout→Pass + t10/t11）+ 在册 flake 报法 2 条（src-then-lazy-load 负载时序刀 B 邻域 + link-multiple media 键挂起下键集漂移）零真 P→F；engine lib 3593/9F（名单与 s145 逐字节相同，details flake 本轮采样回绿）+ runner bin 258/1F（trend_csv_check 归口不变）+ clippy 零告警 + shim 拼接 node --check 过 + 二进制 grep 核验；新记挂账 R102 live_aware 回调簇 pending 失明理论缺口（低优先，踩坑语料再补不做预防性扩面））；承前：M2 切片 145 施工收口（刀 D 祖先锚定结构路径——selector 键跨代碰撞根除：勘根下钻三层定谳**元素身份碰撞**——`_proxyCache`（part01.js:63，P1b S2）按 `_elKey`（part03.js:10995）裸 selector 串缓存 proxy，同串两代元素共享同一 proxy 对象（listener/expando/`===` 全串，s144 所判双通道实为三重：`_inputValues`/`_inputValuesSet` + 宿主 lw pending 属性队列 + listener store 累积闭包）+ 碰撞面收窄 form 内歧义控件结构路径（s142 已锚 form[target]，afterbegin 连环插入使每代新 form 恰落 nth-child(2)、各代 issue 时刻路径串字节相同，绝对位置路径只在单一时刻唯一）+ 方案三选一定**祖先锚定**（锚 id/class/name/target 随节点移动 + 锚内 nth-child 只在锚子树定位，零新增基建；epoch 视图戳过度失效断 `.value=` read-back、R100 持久 NodeId 需 slotmap removal 失效架构远期项，均不取；裸 tag 锚排除——对位移零稳定增益）；施工 js_dom_bridge.rs 3 处（`unique_attr_anchor` 提取 + `unique_selector_for_node` 纯重构 + `structural_path_selector` 祖先锚定/`ancestor_anchor_selector` 新增，无锚链回落旧 root 形态、fastpath 不认领锚定首段安全回落通用引擎）+ part39.rs 2 处（t8b 主钉断言域收窄 + 刀 D 回归钉两代 populateForm 逐代互异）；定向 form-submission-0 **277P/63F/1T**——requestsubmit **20F→0F（36P 全 Pass）**零新 Fail；全量 **7069P/852F/98T**（8019 行）join **状态位移恰 20 键全 Fail→Pass 零 P→F** 全落 form-double-submit-requestsubmit.html（runTest 4 + runTest2 16，刀 D 靶面精确闭合）、key-diff 40 = select-event flake 负偏差恢复 39 + algorithm 报法漂移 1、行数守恒 7981+39−1=8019；details 族 flake 归因实验定谳非本修引入（祖先锚定开/关各 10 次采样失败率同 4F/6P，翻转率≈40% 沙箱异步调度非确定性归口刀 B）；engine lib 3592/10F（9 名单不变 + details 在册间歇）+ runner bin 258/1F（trend_csv_check 并行流归口不变）+ clippy 零告警 + fmt 本轮区域零波及 + shim 未动；无锚链碰撞理论面（多层匿名 div 包裹）挂 s146 候选④）；承前：M2 切片 144 施工收口（刀 C re-entrant submission hang 收口：spec 4.10.22.3 原文钉死步 5 整段门控「submitted from submit() method = false」——form.submit() 跳过 firing flag/交互校验/submit 事件直达 entry list 构造 + 步 9 cannot-navigate 复查 + 导航；施工 part04.js 3 处——submit() 去 m3a `_zwSubmitBusy` 短路（宿主 Enter 通道本就不设，去守卫 = 消除 JS/宿主双通道不一致）+ `_zwFormSubmitNavigate` 函数顶与 formdata 派发后 cannot-navigate 中止（spec 步 1/5.9/9 原位，connectedness 近似 + 完整定义 FIXME）；提交前 rebase 并入并行流 6 提交（EventTarget null listener shim 面 + layout R5324 + encoding 流 goal 脚本）后门禁全数合并树重出：全量 join 零 P→F 零键丢失、状态位移恰 3 键全改善（preventdefault Fail→Pass + Timeout→Pass + runTest2 首案 Timeout→Pass）、runTest2 从 1T+24 未上报全落定 16P/20F（解 hang 新可见 23 键 7 新 Pass + 16 新 Fail）、行数守恒 7994−3+24=8018（7038P/857F/99T → 7048P/872F/98T）；探针链定谳 16+4 键 Fail 单根 = **selector 键元素状态跨代泄漏**（T1 .value 写入泄漏/T5 setAttribute overlay 泄漏/T6 干净页复刻无 v4——v4 全部来自 runTest 前序键写入）→ 刀 D 立项；algorithm t10 残留 1T 归 iframe.src 异步装载面独立挂账；engine lib 3592/9F（9 名单 = 8 已知 + R3045 归口 net-api，details 族 flake 本轮回绿，总数 3601）+ runner bin 257/1F（trend_csv_check 行 196 列错位 = 并入 f62bf89aa encoding 流 CSV 编辑引入归口该流）+ clippy 三 crate 零告警 + fmt 两文件均并行流区域归口 + shim 拼接 node --check；过程注记：合并树首轮全量遭 test-guard 瞬态误杀（横幅后零测试行即报 4.31GB，重跑原样通过））；承前：M2 切片 143 施工收口（查询视图已应用前缀重放双计修复——implicit-submission 提交按钮键翻红：勘根推翻 s142b「片段真实再物化」归因（探针 v25-v27 三级证伪：基座 cached_html 恒单份/双 wrapper 均位置选择器 nth-child(2)/(4)/`__zw_query_all` 返并集），钉死 with_query_view_doc 重放区间 `[0..count)` + 共享队列 append-only（R100）+ R55/R348 注册站换新 Arc 已含已应用插入 + id 去重只覆盖带 id 片段（targetted-form 的 iframe+form 无 id）四件套——已应用 InsertAdjacentHtml 视图内再插一次，form.submitButton 2 元素集合 vs event.submitter 单元素翻红（s142 属性锚使 submitter 语义正确化，基线双向一致错误被揭穿）；施工 4 文件 +129/−11：桥侧 MUT_APPLIED_CURSOR 游标镜像（重放/live_ok 扫描区间收窄 [applied..count)、链步不变用 prev_count、重建起点 applied、min(count) 钳制）+ webview/user_actions 7 发布点（3 共享队列注册站/executor 清陈值 0/私有队列 0/R348 重绑 applied/导航复位 0）+ part05 flatten 鬼影随根消退 + 刀 A′ script 边界幂等 bump 拼头（后两刀保留、注释改记不承担该键）；全量 join 对 s142 fixed 7994 行 (key,state) 对级**恰 1 键 Fail→Pass = 目标键、零 P→F、零键增减、行数守恒 7994=7994（7038P/857F/99T，ΔP=+1/ΔF=−1 精确闭合）**；提交前 rebase 并入并行流 5 提交（net-api M2-S6/S8/S9/S10 + layout ×2）后门禁全数合并树重出：engine lib 3591/9F（8 名单与 s140/s142 基线逐字节相同；第 9 位轮换 details flake 通过/新面孔 test_request_signal_passthrough_r3045 = net-api M2-S8 破 R3045 契约测试归口该流）+ runner bin 258/0F（trend_csv_check 被并行流收口）+ clippy 三 crate 零告警 + fmt 本切片 4 文件零 diff（js_dom_bridge.rs 7 处 + testharness.rs 3 处均并行流区域不重排）+ shim 拼接 node --check；全量 rebase 前后两跑逐字节相同（7038P/857F/99T），并行流未动 html/semantics 面；探针验后删语料零改动）；承前：M2 切片 142 施工收口（name/target 属性锚 + pending 签名 +target——targetted-form 族 21 键翻红 + 17 键新上报，1 键揭片段双重物化旧疾：施工两处——host 侧 unique_selector_for_node 结构路径前插属性锚臂（name/target 值域限 [A-Za-z0-9_-]、qSA 验唯一才采用；iframe[name=form-test-target-N]/form[target=…] 获稳定身份，位置漂移免疫）+ shim _zwPendingParsedForSel 签名 tag|id|name|class 扩 +target（populateForm 的 form 无 id/class/name 全同签 FORM|||，多 form pending 歧义 hits>1 不归一→落 pending 失明 live 臂→form.previousSibling 崩 null 主根因消解；target=frameName 每次唯一）；全量 join（真基线 8001 行=7000P/904F/97T 与 s141 在册吻合；fixed 7994 行=7037P/858F/99T）**key 级 state-set 分类**：21 键 Fail→Pass（double-submit-default-action 4/requestsubmit runTest 7/preventdefault-click 2/input-type-change-submit 1/request-submit-activation 1/form-submission-algorithm 5——全集中 form-submission-0 targetted-form 族）+ 17 键新上报 Pass（base 早期崩溃未上报）vs 2 键 Fail→Timeout + 26 键 base-only（26F+1T 先前已在失败，fixed 文件级 hang 不再上报——crash→hang 递进记账非回归）+ 1 键 Pass→Fail（implicit-submission optional——s142b 探针 4 版 5 跑+stash 对照定谳：提交导航触发 (iframe+form) pending 解析片段在 body **真实再物化两份逐字副本**为既有引擎旧疾（基线同现），基线过测系双向一致错误（结构路径钉单位置→命名作用域单元素=submitter 单元素相等），本修 event.submitter 语义正确化 + 属性锚在逐字副本复制树失唯一（任何属性方案不可分辨、位置方案则漂移别名）→form.submitButton 2 元素集合 vs 单元素 submitter 不等翻红；修法=消灭双重物化非选择器面，刀 A 升格刀 A′）；engine lib 3588/9F（rebase 后合并树，失败名单与 stash 基线逐字节相同全环境面；s141 在册 8F→9F 轮间环境漂移两侧同现）+ runner bin 256/1F（trend_csv_check 归口不变）+ clippy 零告警（rebase 撞并行流 4f2230a6d url-setter 落地区域 4 处 -D warnings 阻断顺手机械修复并归口）+ fmt 本切片区域零 diff（上游区域 9 处 fmt diff 归口该流未顺手重排）+ shim 拼接 node --check；探针验后删语料零改动）；承前：M2 切片 141 施工收口（targetted-form 族勘根轮：探针 14 轮钉死三层根因——①兄弟导航 apply 代际缺口（insertAdjacentHTML 注入 iframe+form 静态读 previousSibling 走 R3254-K3 pending 臂正常，提交触发 `__zw_apply_generation_bump` 定点清 pending 解析节点后 JS proxy 持有的 stale sel 落 live 臂 `__zw_sibling_nodes`/`_childNodeList` 恒 miss→null，语料 `loadPromise(form.previousSibling)` 崩 Cannot set properties of null——form-submission-algorithm 8F + requestsubmit onload 形态 ~20 键波及面；文档未导航/iframe named 查询/innerHTML 序列化/children 枚举/form proxy 身份五通道旁证在，缺口精确 sel 换代失效非树损）；②apply 滞后一拍 + runner 同文件多跑前遍挂（form-submit-action 单文件 runner 执行 3 遍 30 行=10 案×3，遍 1/2 全挂 gEBI click not function、遍 3 才 8P/2F；语料本体减半单案冷启动行为完全正确——formdata 全序 dump：entry 构造在 submit 事件后、A 的 IDL set 落 attr（value mode=default）、B 的 preventDefault 拦住第二次提交，双提交语义定谳无 bug，原版失败全在案间/遍间状态；统一假说 = 注入节点 host 物化跨 script 边界 pending→apply→稳定两拍，提交管线 host 通道对 pending 失明）；③空串 formaction 回落误判（④ action 解析 `=== ''` 并入回落分支错回落 form action 属性；spec #concept-fs-action：submitter 有 formaction 属性含空串即以其为 action，空串 resolve 即 base=文档地址）；施工 part04 1 处——回落判定改只按 `== null` 属性缺席、空串交 `new URL('', base)` 天然落文档地址 + FIXME 清单行同步——定向 form-submission-0 **210P/114F/1T**（对 s140b (key,state) 对级 293→295 对恰 +2 F→P 零 losses：button/input 两案 no formaction expect document URL）；全量 join **零 losses 恰 +2 gains**（两键新增 Pass 态；Fail 态并存 = 多跑前遍挂 click 形态未达 formaction 断言属刀 B 面；行数 7981→8001 +20 = form-submit-action 多跑 +2 遍，6998P/886F/97T → 7000P/904F/97T，ΔP=+2、ΔF=+18 全为多跑行）；engine lib 3582/8F（HEAD 基线在册环境面）+ runner bin 256/1F（trend_csv_check 归口不变）+ clippy 零告警 + shim 拼接 node --check + fmt runner 过；探针页验后删语料零改动）；承前：M2 切片 140 施工收口（File 条目层 + multipart filename 不归一定谳轮：spec 核实定谳三点——construct entry list file 状态步对每个 selected file 贡献 entry（name, File）无选中零条目、normalize-newlines 步只作用 entry name 与非 File value（File.name 不触）、filename 仅 escape a string 逐字节转义（LF→%0A/CR→%0D/`"`→%22，「must not perform any other escapes」——先归一再转义为错，语料 `b\nc`→`b%0Ac` lone LF 保持实证）；施工 part04 两处——file input entry 分支（`_c.files` item() 枚举逐 File `_zwFdEntry` 贡献）+ ⑤a 序列化层字节构建器（`_entryStr` File→filename 三通道共用 + multipart `_mpBytes`/`_mpPushStr` 逐字节构建：part 头 name 经 `_mpEsc` 归一后转义、File 分支 filename 裸转义 + Content-Type file.type||octet-stream + `_zw_blobBytes` 原始字节、`_zwEncodeBytesPrefix` `__zw_bytes:` wire 传 host）——定向 form-submission-0 **208P/96F/1T**（对 s139 148P/156F/1T，60 F→P 干净闭合：text-plain +20/urlencoded2 +20/multipart +14，multipart filename 换行 6 案由不归一定谳修复）；全量 join **恰 60 F→P 全部为三文件案（各 20）、键零消失零新增零其他位移**（行数守恒 6938P/946F/97T → 6998P/886F/97T，ΔP=+60/ΔF=−60）；engine lib 3581-3582/8-9F（stash 对照定谳 HEAD 基线同样 8F——js_dom_bridge fetch/service worker/request 桥接系列环境性失败面非本切片引入，新挂账勘根）+ runner bin 256/1F（trend_csv_check 归口不变）+ clippy 零告警 + shim 拼接 node --check + fmt runner 过；语料零改动无探针页）；承前：M2 切片 139 施工收口（text/plain + multipart 编码器轮：spec 核实定谳三点——convert 步 File→filename + lone CR/LF→CRLF 归一为 urlencoded/text/plain 共用、text/plain `name=value\r\n` 逐对零转义（NUL 字面、无百分号编码）、multipart part 头 name/filename escape a string（仅 LF→%0A/CR→%0D/`"`→%22）+ 非 File value 不转义 + 语料 expected.name 为转义后形态无冲突；施工 part04 enctype 读提升 body 构建前 + `_normNL` 三序列化共用 + POST body 按 enctype 分型（text/plain 零转义逐对 / multipart boundary 47 字符撞串重掷 + escape a string + `--B--\r\n` 终结；`_urlBody` 另存 GET query 恒 urlencoded）+ runner echo 端点 CR/LF 改字面交付（s138 转义形态与 helper `unescape()` 首步换行归一不兼容——归一先于 `\xNN` 解码、转义产物脱离归一）+ 单测重写为 helper unescape 的 JS 等价实现双向校验 + lone LF 案实证——定向 form-submission-0 **三文件各 36P/26F 完全镜像**（text-plain/multipart 各 0P→36P）；全量 join **(key,state) 对级去重后恰 72 F→P 全部为两文件字符串案、键零消失零新增零其他位移**（行数守恒 6866P/1018F/97T → 6938P/946F/97T，ΔP=+72/ΔF=−72 精确闭合）；engine lib 3590/0 + runner bin 255/2F（+1F = 端点单测自身 helper 笔误 `take(2)` 从 `x` 起收吃掉首 hex 位，修复后过；另一 F = trend_csv_check encoding 行归口不变）+ clippy 零告警 + shim 拼接 node --check + fmt runner 过（engine 在册 url_idna.rs 违例归口并行流不变）；语料零改动无探针页）；承前：M2 切片 138 施工收口（form-submission-0 提交主干轮：构成面勘根定谳 186F 单一主干四件套——POST 零投递（`if (_isPost) return;` documented 边界）+ form target 无解析 + iframe POST 装载通道缺（`_zwLoadIframeEntry` 硬编码 GET 空 body）+ echo-content-escaped.py 端点缺，`but got ""` 全形态一字不差（about:blank 初始 onload 先触发）；施工 runner fetch handler 端点路由（POST body 逐字节 `\xNN` 转义 + `\`→`\x5C` 防 unescape 正则误吞 + round-trip 直测）+ part01 flags.method/body/headersWire 透传 `__zw_fetch` + part04 ⑤b target 分支（submitter formtarget > form target → named iframe 经 `__zw_reload_iframe` 复用装载管线，GET query/POST body+enctype CT，未命中落顶层行为）+ `_encPair` 补编 `! ' ( ) ~`（encodeURIComponent vs spec urlencoded percent-encode set 差异，原注释「除 `*` 一致」有误）——urlencoded2 **62F→26F（36 F→P）**；全量 join **7914→7981 行配平完全闭合、36 F→P 真翻红、零坏位移**（6814P/996F/102T → 6864P/1018F/97T；+72 新见键 14P/58F = targetted-form 族 5 文件级 Timeout 解除后首次出案、−5 T 行解除）；engine lib 3606/0 + runner bin 256/1F（唯一 F = 既有 trend_csv_check encoding 行归口不变）+ clippy 零告警 + shim 拼接 node --check + fmt runner 过（engine 报并行流 url_idna.rs 既有归口该流）；语料零改动无探针页）；承前：M2 切片 137 施工收口（FORM named properties 语义补全轮：勘根定谳 form-nameditem 10F 六组——gOPD 不见 named ×3 / 多匹配返首元素非 RadioNodeList ×2 / expando 优先序缺失 / proxy invariant 违例 / past names map 缺失 / 「override builtins」与 fix#19c 冲突挂待决；施工 part04 1 处 + part05 4 处——get trap FORM gate 内重写五臂序（invariant→own expando→named→past listed 复核→s119 索引）+ part05 顶层 `_zwFormNamedLookup` helper（多匹配 `_zwMakeCollection(false)` + per-form per-name 实例缓存 `_zwFormNamedCollCache` 成员身份比对复用）+ gOPD trap invariant/named 两臂 + has trap named 臂 + set trap named 拒写 + defineProperty/deleteProperty trap 新增——form-nameditem **16P/1F 收口 9**（8 F→P，唯一 F = override builtins 挂待决）；全量 join **恰 8 Fail→Pass 真翻红 0**（6805P/1004F/102T → 6814P/996F/102T，7914 行守恒，3 NEW 全 dup-key 副本假阳性）；engine lib 3584 连续两轮 0F + runner bin 255/1F（唯一 F = 并行流 5a5da454b 对 wpt-suites.csv 写回损坏——注释行 quote 包裹已机械恢复、encoding 行字段错位归口 encoding 流挂账，非本轮引入）+ clippy 零告警 + shim 拼接 node --check；探针页验后删语料零改动）；承前：M2 切片 136 施工收口（激活门 type 读通道换 lw 融合读轮：勘根定谳单根——type IDL setter 推 SetAttr mutation 但 R2995 lw 层立即可见，click() API 门与 R106 dispatchEvent 臂的 sel 分支读纯快照 `__zw_get_attr` 恒空 → BUTTON 空 type 恒判 submit（t2 reset click 误 submit 永不 done = 文件级 Timeout 根因 + t3 type=button 误 submit unreached；t1/t4 碰巧对因期望恰为 submit）；handle 分支 `__zw_get_attr_handle` 本身 consult 当前批 mutations 无坑；施工 part04.js 2 处——两臂 sel 分支换 R2995 lw 融合读（consult 变更列表 latest-wins → applied view 回落，与 getAttribute 同语义）——button-events **9/9 全绿收口**（t3 F→P + t2 新键 Pass + **文件级 Timeout 解除挂起清零**）；全量 join 恰 **1 Fail→Pass 真翻红 0**（6805P/1005F/104T → 6808P/1004F/102T，src-then-lazy-load Timeout 波动自然回落对冲净零）；engine lib 3584 用例（并行流并入 +13 新基线）默认并行间歇 1F = details template deep entry 与 s135 定谳同款（2F/8P + 4 线程 0F/2P + 失败名抓取确认，挂账延续）+ runner 256/0 + clippy 零告警 + shim 拼接 node --check；探针页验后删语料零改动）；承前：M2 切片 135 施工收口（select reset 默认层捕获轮：select.value= 经 SelectOption 推送时 apply 侧（js_dom_bridge.rs:1405）改写快照 selected 属性破坏默认层——form reset() 清 shim dirty/clean 键后读链穿透宿主 stale 编程选中（`__zw_option_selected`）遮蔽属性层，`_zwSelectednessSetting` 走查同链穿透 fallback 不触发；施工 shim 3 文件 5 处——part01 `_optSelAttrDefault` 默认层捕获 store + part03 `_zwOptCaptureSelDefaults` fill-once helper（唯一钩点 = select.value setter，SelectOption 唯一推送源；selectedIndex=/option.selected= 只写 JS 层无需钩）+ part04 value setter 钩捕获 + reset SELECT 分支删 dirty/clean 后显式落 `_optSelectedClean[k] = capture ?? live-attr` 遮挡宿主——reset-form.html select ×3 **3F 收口**全文件 14/14（reset() 方法 + click 两通道同链）；全量 join 恰 **3 Fail→Pass 真翻红 0**（唯一非 Pass 变动 = the-img-element src-then-lazy-load.html 文件级 Timeout 边界波动，lazy-loading 网络等待面与改动零关联，前轮同形）；engine lib 3571/0（默认并行间歇 details template deep entry flake 定谳负载敏感型非本轮引入，挂账独立勘根）+ runner 256/0 + clippy 零告警 + shim 拼接 node --check；无探针页语料零改动）；承前：M2 切片 134 施工收口（reset 按钮 click 激活接 form.reset() 轮：`_zwSubmitBtnDefaultAction` 加 asReset 参数单点接线（.click() API 门 R57 与 dispatchEvent R106 两臂共用，form 反链零重复）——reset 型（INPUT/BUTTON）激活行为=spec §4.10.5.4 form.reset()，经 FORM proxy 进 R3048 闭包自动继承 s129 trusted reset 事件 + s133 双身份枚举/清脏复位；勘根定谳双链分离——proxy 链（runner 到达路径）reset 型原完全无处理，part03:17793 raw 链为 UA 指针激活平行近似链不动；reset-form.html input/textarea click **4F 收口** + the-input-element/reset.html **4F 收口**（form owner/disabled/form= 属性 ×2）+ button-click-resets-with-commandfor **3F 收口** + button-events t5/t7 挂起案 **2 新键 Pass**（文件级 Timeout 挂起 3→1 收窄仍在）；全量 join 恰 **11 Fail→Pass + 2 新键 真翻红 0**；engine lib 3571/0 + runner 256/0 + clippy 零告警 + shim 拼接 node --check；无探针页语料零改动）；承前：M2 切片 133 施工收口（form reset handle 身份枚举 + 清脏复位轮：reset-form-2 **1F 收口**全文件 1/1（脚本构造 form reset 整体 no-op 双根因——`_formControls(sel)` 仅 sel 身份枚举盲区（part05 双身份化 walk `_makeProxy(null, handle)`，part04 调用方 9 处传 handle，submit entry list 既有限记同单解除）+ 赋值式复位脏旗驻留（赋值恢复→清脏键两步式，R2998 捕获模型 checkbox/radio 必须先赋值后清键）；form-nameditem「innermost form」1 案**搭车翻绿**；全量 join 恰 **2 Fail→Pass 真翻红 0**；engine lib 3570/0 + runner 256/0 + clippy 零告警 + shim 拼接 node --check；无探针页语料零改动）；承前：M2 切片 132 施工收口（radio 组排他现象 4（type 变更信号）接线轮：radio-morphed **1F 收口**全文件 1/1（spec §4.10.5.4 type change steps 步骤 6「Signal a type change」+ §4.10.5.1.16 现象 4——part01 `_zwApplyInputTypeChange` 尾部单点接线，IDL setter 与 setAttribute 双通道既有汇点零拓扑成本；BUTTON 经 setAttribute 设 type=radio 由 sweep checked 门自兜；现象② name 设/改/删无独立 corpus 面——radio.html:88 案由现象①覆盖）；全量 join 恰 **1 Fail→Pass 真翻红 0**；engine lib 3570/0 + runner 256/0 + clippy 零告警 + shim 拼接 node --check；无探针页语料零改动）；承前：M2 切片 131 施工收口（radio 组排他现象 3（form owner 变更）四通道接线轮：radio-disconnected-group-owner 残余 2F 收口全文件 **8/8**（Removed elements 后半 + Appending … disconnect form 全案——appendChild trap / insertBefore trap / R334 sel 臂 / innerHTML registry 物化后四通道接线 + part05 sweep 收编 `_zwRadioUncheckGroupOthers` 共享出口）；全量 join 恰 **2 Fail→Pass 真翻红 0**；engine lib 3570/0 + runner 256/0 + clippy 零告警 + shim 拼接 node --check；探针页验后删）；承前：M2 切片 130 施工收口（radio 组根域枚举轮：radio.html 组属主 3F 收口（orphan tree / different form owners / different name——组枚举根域 detached 化：parent 链顶 nodeType 9 → document qSA 绿面不变，链顶非 document → 链顶子树 qSA + part03 valueMissing radio 臂 form owner 键比对）+ radio-disconnected-group-owner 5F 搭车翻绿；全量 join 恰 **8 Fail→Pass 真翻红 0**；engine lib 3570/0 + runner 256/0 + clippy 零告警 + fmt + shim 拼接 node --check；探针页验后删）；承前：M2 切片 129 施工收口（input/change 事件属性 + UA trusted 面轮：input-change-event-properties **11F→0F**（composed 全族 part01×16/part03/part06 + 宿主 change-on-blur 断链接线 + runner SELECT 键接线）+ 同域 rider：UA 语义事件 trusted（reset/submit/script error/激活 input/change——R312 `__zwMakeTrustedEvent` 双印通道；合成 click() 保持 untrusted 负案不回翻）+ checkbox indeterminate pre-click/canceled 清除 4F→0 + radio.html t1 1F→0；全量 **6758P/1051F/103T → 6780P/1032F/103T** join 恰 **19 Fail→Pass 真翻红 0**；勘根转折：typing 面报错 `undefined reading 'bubbles'` 系 s128 首断言（input composed）修复揭开的 **change 前存断链**——宿主 `_zwFocusSel → __zw_host_focus/__zw_host_blur` 管道未接 s128 基线/change 两臂（send_keys 首键自聚焦即宿主通道），探针四轮定谳非回归后接线（`__zw_host_focus` 记基线 + `__zw_host_blur` 派生 change，与 part04 页面 JS 臂幂等互斥）；engine lib 3570/0 + runner 256/0 + clippy 零告警 + fmt + shim 拼接 node --check；探针页验后删）；承前：M2 切片 128 施工收口（textarea 域残余同域收割轮：change-on-blur 共享面（`_userEdited` 脏旗 + 净值基线比较——script value= 不算用户编辑，restore-then-set 负案不误触）+ textLength/type 单点 getter + maxLength/minLength WebIDL NaN→0 折叠——定向 9F→0F（textarea 5F + input 域 change ×2 与 maxlength/minlength 搭车 4F），全量 join 恰 9 Fail→Pass 真翻红 0；渲染流面让位：wrapping hard-wrap ×2 + placeholder-lineheight；engine lib 3570/0 + runner 256/0 + clippy 零告警 + fmt + shim 拼接 node --check；下一步 s129 候选：挂账序复核 / 全量基线翻拣下一单根面——已由 s129 收口，见上）；承前：M2 切片 127（TEXTAREA value/defaultValue getter 面轮：value-defaultValue-textContent **10F→0F** + wrapping 换行归一搭车 1F→0F，全量 join 恰 11 Fail→Pass 真翻红 0；勘根定谳：value getter 非脏路径 handle 形落 stable-key 惰性初始化读 value 属性缓存 '' 钉死 + API 值缺 \r\n|\r→\n 归一 + child text content 误用 textContent 融合视图 + value setter 缺 [LegacyNullToEmptyString] null 折叠 + defaultValue getter R3049 冻结捕获 stale；施工：part03 live 读双形态 + 读侧统一归一 + part04 setter null 折叠缓存写保留 + `_textareaDefault` 全链删（part01/03/04/05）+ Rust 测试断言不变仅注释更新；engine lib 3570/0 + runner 256/0 + clippy 零告警 + fmt + shim 拼接 node --check）；承前：M2 切片 126（img relevant mutations 三根因轮：relevant-mutations 家族 **114P/7F → 121/121 全 Pass，真翻红 0**（全量 6690P/1118F/103T → 6737P/1071F/103T 配平，47 翻绿 = 40 语料资源 green-2x2.png 补齐 + 7 本修复）；勘根定谳：`__zwAttrState5277` 把 referrerpolicy missing/invalid/empty 归并 `'no-referrer-when-downgrade'`（现行 spec 缺省态是空串，n-r-w-d 独立）×5 + removeChild 后调致 SOURCE 移除臂 `after` 旗标死码 ×1 + lazy img width/height 属性臂缺失（whatwg/html#8492）×1；施工：part03 状态表空串态 + 树触发 helper 增 former-next-sibling 位置锚（part04 R188 `_r188Next` 同源传递）+ IMG 属性分支 lazy width/height 臂；engine lib 3570/0 + runner 256/0 + clippy 零告警 + fmt + shim 拼接 node --check）；承前：M2 切片 125（input 克隆状态传播轮：clone.html 19F→19/19 + radio.html 4F→0，全量真翻红 0；part04 deep clone HTML-ns 物化 + form 状态 walk + part05 checked setter radio 组排他）；承前：M2 切片 124（module 网络面收口轮：modulepreload 家族基线 6 行（4F+2T）→ 15/15 全 Pass 真翻红 0；shim `_zwModulePreloadMaybeStart` 同步内联取回 + `_mo_notify` LINK 触发臂 + classic 动态 import() module map 记忆化（恰好一次求值 + 失败不入缓存）+ runner 端点仿真 ×2 + pin ×2；module-import-referrer TLA 根因精确化：入口 TLA 可用、依赖模块 TLA 不可用——build_dep_iife 同步 IIFE，深转换架构面；dynamic import() with 透传 = css-module 复合保留）；承前：M2 切片 123（FormData 条目构造收口轮：newline-normalization **12F→0F**，全量 A/B 恰 14 Fail→Pass 真翻红 0；勘根证伪换行归一假设——真根因 document 级逗号列表 live 树查询恒空 + file 状态无枚举臂；双刀：枚举双源并集 + file 臂；新挂账 ×2：FileList 缺索引访问 + live 树 document 级逗号列表 qSA 根缺口）；承前：M2 切片 122（classic script 编码链三面 4F→0F）；**用户决策 2026-10-23**：parser form element pointer 4F **缓行**——与 declarative shadow DOM 打捆 parser 批次、M2 收口后尾段单独立项（[决策记录](html-semantics-compat/evidence/2026-10-23-user-decision-parser-form-pointer-defer.md)），流按其余挂账序推进

> **▶ 语料资产轮（2026-10-31，非施工切片——css/text 数据模块资源补拉，待决①销账，零源码改动）**：
> 用户授权「补语料属基准变更」解除 s108 待决①。缺口机制 = M1 首拉（10-15）无 .css
> 白名单 + .txt opt-in + 幂等快路径固化（s158 同坑）。25 号脚本 ③b 段 8 文件显式
> fetch_raw（R4300 同构；atImported.css 为 bad-import @import 传递引用扫描新增；
> not_found.* 负向保持缺席；utf-8.css/utf-8.txt 仅经 serve-with-content-type.py
> 引用——runner 端点未实现 FIXME，端点落地再补）。**A/B（同二进制 stash 法，
> test-guard + RAF 环境齐全）**：css-module 28/29 稳定 + text-module 15/16 稳定，
> 唯二位移 = **2 案 P→F 同族**（script src 非 JS 错误事件面——基线缺资源 404 error
> 凑巧 Pass，补后真缺口揭穿：classic script 装载器对非 .js src 静默丢弃，FIXME
> 新记）。预期面（integrity matches/content-type matches/charset）未翻绿——blocked
> 在 css/text module 执行面（s107 记账延续），本轮资产为该面切片前置。
> **基准变更注记**：下轮全量 join 对应位移按语料变更归因非回归。
> **施工候选承 s171 不变**。另记：s171 solo 冷却回绿销账佐证漂移域为负载敏感暂态，
> 并发 rally 期漂移域取证须 solo 窗口（run-rules 候选①对齐）。
> **账本**：[evidence/2026-10-31-corpus-css-text-module-resources.md](html-semantics-compat/evidence/2026-10-31-corpus-css-text-module-resources.md)
>
> **▶ 勘定轮（2026-10-31，非施工切片——候选④ selection-pointer forms 域翻绿归因勘定，零源码改动）**：
> 三态 A/B 矩阵定谳**环境漂移非修复收益**：①main s176 全量 21P；②s172 gEBI
> 修复逆补丁（code-only，part06.js+coverage.rs）solo 仍 21P——修复非本域门；
> ③s167 时代树 d5ec12d94（full1 记红基点）今日 solo 仍 21P——红不可由时代代码
> 复现。红时代失败签名（`anchorNode expected Text node "foo" but got Text node
> "foo"` 打印同串身份分裂）= ghost/双视图类症状的机器级暂态显形，与 s160「与
> 刀无关环境漂移」/s171「负载敏感暂态」在册结论同形互证。候选④销案；回红再查
> 方法升级：先历史树对照再 s172 逆补丁对照，两步定环境/代码归因后进勘根。
> **下一步候选（本流）**：⑤多进程 renderer SetDomSnapshot entries 化（s174 挂账，低优先）。
> **账本**：[evidence/2026-10-31-selection-pointer-domain-attribution.md](html-semantics-compat/evidence/2026-10-31-selection-pointer-domain-attribution.md)
>
> **▶ 当前状态（2026-10-31 M2 切片 176 收口，shadow 域定位选择器寻址全链路——compound `zwsh<len>:<host>|<local>` + slot 分配接线 + 同树 closest 根修，shadow-slots 段 LOST 20 → 恢复注册 2P/18F，rust 6 + shim 3 + 新模块 + 钉测 2）**
> **入场定刀位**：s175 FIXME 承接——引擎侧 shadow 寻址（枚举通道重开前置）。s173/s175 定谳 detached 镜像对引擎不可见（shadow-slots 段 0P/20F）。
> **根因链四层**（逐层修复后 replica 3P/17F 实证）：①shim `_zwLdFlatParent` 把 shadow 节点摊平进宿主 slot——engine parentNode 跳过 SR 非元素父使 shadow 子 parentNode 锚宿主，摊平 = 元素塌进自身 slot 64-hop 环 → doc-order sort 腐败 → light-dismiss 误选；②`parent_selector_for_doc` 经封装门恒空串 → flat 链首槽 hop 截断；③dom `resolve_slots` **全库零调用点**——分配恒空 → layout slot 替换全走回退内容 → 被分配 light 子树无布局盒 → gBCR [0,0,0,0] → 点击坐标塌 body@(0,0)；④runner 页内扫描（封装门）+ hit-test 铸造点对 shadow 内容无地址。
> **回归勘根（本刀内闭环）**：form-submit-dialog-shadow P2F——s173/s175 凑巧 Pass 系镜像惰性（寻址不着 → click no-op）；实地址化后 click 到达真激活臂，`closest('dialog')` 匹配集从 doc root 求 + parent 链穿 SR→host **跨界命中 light dialog**（spec：closest 祖先不穿 shadow 边界，whatwg/html#7971 即本语料）。修 = closest 匹配域换 `shadow_scope_root`（matches 先例）+ compound 回落；light 调用者逐字节不变。
> **施工**（11 文件）：新模块 `shadow_address.rs`（compound 铸造 + fastpath 三层返回识别即权威 + local 阶梯/位置链元素序直查 + `shadow_scope_root` + `shadow_content_entry_selector` 嵌套守卫 + `ds_scoped_entry`）；bridge fastpath/node_entry_json 回落；selector_match 两修；shadow.rs 逐宿主 resolve_slots；hit_test 两铸造点回落；webview `element_from_point`；runner compound 命中采信门（仅 zwsh 前缀）；part01 flat-parent 守卫 + light-dismiss shadow 扩展（flat chain/doc-order/compound 同串比对）；part04/05 children ensure-content + `.open` IDL + hydrate/mirrors（枚举通道实地址重开）；钉测 2。
> **验证**：engine lib **3619P/10F**（+2 钉测，10F 基线同键）；dom lib 880P/0F；webview lib 830P/0F；interactive-elements **439P/44F/483** 对 s172 基线 join 恰 4 位移——F2P=2（s173 双靶承接）+ P2F=2（shadow-slots 段 2 行假 Pass 清除，s173/s175 在册归因非回归）；**对 s175 shipped 终态 common 463 键逐键同态零真 P2F，GAINED=20（2P/18F）**；forms 全树 4298P/402F 对 s175 子集 108 键零位移（closest 修后复跑同态）；fmt/guarded-clippy 零告警；shim 装配序 node --check SYNTAX_OK 113576 行；探针验后删。
> **挂账**：渲染流——nested-shadow 布局几何三面（dialogB 零尺寸盒 (8,100,0,0)/嵌套 shadow fixed 定位/popoverB slot 流位 vs top:250），shadow-slots 段 18F 主体归因面；FIXME——命令式 attachShadow 与宿主 childlist 变异不触发 resolve_slots 重解析。
> **下一步候选**：④selection-pointer forms 域翻绿归因勘定（s172 记）；⑤多进程 renderer SetDomSnapshot entries 化（s174 挂账）；ua_display_li（渲染流，不越界）；nested-shadow 布局几何（渲染流挂账，上）。
> **账本**：[evidence/2026-10-31-m2s176-shadow-addressing.md](html-semantics-compat/evidence/2026-10-31-m2s176-shadow-addressing.md)
>
> **▶ 上轮状态（2026-10-31 M2 切片 175 收口，detached 节点选择器身份盗用根修——unique_selector 可达门 + 枚举镜像撤下归因，rust 2 + shim 注 1）**
> **入场定刀位**：候选①声明式根内容枚举（s173 LOST 20 承接）——shadowRoot childNodes/children/firstChild 族惰性镜像钩。首刀 red→green 4/4 后 WPT 域门 **P2F=16**（dialog-popover-closedby-complex normal 段），两刀勘根定谳（枚举钩 + 管线等值轻量臂均试过并撤下，详见账本）。
> **定谳**：detached 镜像节点的 `unique_selector_for_node` **盗用**在树同 `tag.class` 真元素的唯一选择器（uniqueness 门在 doc 上查询、detached 节点不在 doc，真元素顶替唯一匹配——探针实证镜像 `__n13` → `dialog.dialogA`）→ apply handle_selectors 进 sel↔handle 表顶替真元素句柄绑定 → normal 段 showModal/close 全打在镜像上（close 打镜像 → 真 dialogB 永不关，失败值逐字吻合）。**s173 首刀 18 键 P2F「代际偏斜」归因不完整，真机制同此**；等值轻量臂实证换代理论与 P2F 无关（light arm 触发而 P2F 不动）。
> **施工**（3 文件 +66 行）：①engine `js_dom_bridge.rs` `unique_selector_for_node` 顶部可达门——parent 链不达 `doc.root()` 返 `None`（detached 不可寻址；物化 shadow 树链达 root 不受扰，shadow 内容 None 系封装查询门先证非本门）；②engine 钉案 `test_detached_node_unique_selector_is_none_s175`（镜像句柄不得进 handle→selector 表 + 真元素寻址不受扰）；③shim part05 枚举钩撤下留 s176 deferral 注（重开前置：引擎侧 shadow 寻址，非 detached 镜像）。
> **验证**：engine lib **3617P/10F**（+1 钉测，10F 基线同键）；webview lib **830P/0F**（= s174 基线）；interactive-elements **437P/26F/463** 对 s172 基线 join **P2F=0**、F2P=2（s173 双靶既有）、LOST=20（s173 账面既有归因）——与 s173 终态逐键同态（根修硬化零域扰动）；forms 家族 96P/12F 持平；shim 装配序 node --check SYNTAX_OK。
> **枚举通道结论**：hooks-on 实证恢复注册零 Pass 增益（shadow 段 20 行 0P/20F，基线 4 假 Pass 未幸存——detached 镜像对引擎 light-dismiss/hit-test 不可见，R2926 渲染/交互语义前无翻绿价值）。
> **FIXME 新记**：声明式根内容枚举通道重开 = s176 引擎侧 shadow 寻址设计（物化 shadow 根 handle 化）。
> **下一步候选**：①s176 引擎侧 shadow 寻址（枚举通道重开前置，本刀 FIXME 承接）；③ua_display_li（style-system 跨流）；④selection-pointer forms 域翻绿归因勘定（s172 记）；⑤多进程 renderer SetDomSnapshot entries 化（s174 挂账）。
> **账本**：[evidence/2026-10-31-m2s175-detached-selector-identity.md](html-semantics-compat/evidence/2026-10-31-m2s175-detached-selector-identity.md)
>
> **▶ M2 切片 174（2026-10-31 收口，webview 静态外链 CSS entries 化——级联文档序，s167 挂账收口，rust 3 零 shim）**
> **入场定刀位**：候选② s167 挂账——静态发布（`prepare_page_subresources` → `load_html`）把外链表合并**单 blob 整体前置**（`set_active_external_css`），link 在文档 `<style>` 之后的页面同特异度恒输（blob 无位次，违 CSS Cascade 5 author origin 引入元素文档序）；engine collector s167 已有 `collect_interleaved_external` 文档序交错通道但静态侧从未喂 entries。生产影响面核实：browser tabs（tab_manager.rs:1392）与 renderer 运行时均创建 WebView 直调 `load_html`——嵌入式静态路径即生产路径；WPT runner complete_fetched_page 同路径。
> **施工**（rust 3，零 shim）：①engine `extract_stylesheet_link_anchors`——`(位次结构路径, href, media)` 文档序，rel/href 门与 `extract_stylesheet_links` 逐字一致；`positional_structural_path` 生成 `tag:nth-child(pos)` 段（1-based **元素序**兄弟计数——文本/注释不计，根 `:nth-child(1)`），形态与 engine `structural_path_selector` 逐字同约定 → `find_by_selector` t8b fastpath O(1) 锚定；同 parser（`zero_dom::parse_html`）同树形，位置路径跨树成立；单测 3 案。②webview `resolve_external_css` 返回 `(entries, combined)`——queue 携带**根 link 序号**，@import 子表传播根序号、展开文本归根 link 桶（替换旧平铺队尾尾接）；CSP 门 target 用根序号（原 `unwrap_or(0)`）；空文本桶不入 entries。③**generation 配对 stash**——`prepare_page_subresources` 存 `(document_generation, entries)`，`load_html` 代际 +1 后仅 `stashed_gen+1 == generation` 消费走 `set_active_external_css_entries`，不匹配回落旧 blob 发布（嵌入器 custom-css 通道 + `set_active_external_css` 清 entries 槽的双写者互斥保持；多进程 renderer SetDomSnapshot 臂不动另行挂账）；pub API 零变化（zero-webview 稳定嵌入边界）。④钉测 5 案（link after/before/交错/media 门/import 桶，页内脚本 gCS 探针——gCS 面与布局级联同线程同槽）。
> **验证**：红→绿 stash 双态对照（保留测试注册只 stash 实现 2 文件）——红态 3 预期红案全 Fail 且失败值逐字吻合推演（`static`×3），2 守卫案红态 Pass（无过度矫正）；webview lib **830P/0F**（基线 825 + 5）；engine lib **3616P/10F**（+1 单测，10F 基线同键 fetch 族 9 + ua_display_li）；WPT 域门（runner 同路径）：interactive-elements **463 键 (key,state) join 对 s173 终态 437P/26F bit-exact**——P2F=0/F2P=0/LOST=0/GAINED=0 零扰动；the-link-element（114 键）/the-style-element（43 键）红绿双态 join 零漂移（两域断言未踩交错形态，F2P=0 属预期，修复价值由钉测红案承载）；forms 家族 96P/12F 持平（12F 全既有已归因）；fmt/guarded-clippy `--all-targets` 全绿。
> **FIXME 销账/新记**：s167 挂账**本刀范围销账**——runner + embedded 静态路径 entries 化落地；新挂账：多进程 renderer `SetDomSnapshot` 臂 entries 化（低优先，js_worker.rs:1853 既有 blob 文档化回落不变）。
> **下一步候选**：①声明式根内容枚举（s173 LOST 20 承接）；③ua_display_li（style-system 跨流）；④selection-pointer forms 域翻绿归因勘定（s172 记）。
> **账本**：[evidence/2026-10-31-m2s174-webview-static-css-entries.md](html-semantics-compat/evidence/2026-10-31-m2s174-webview-static-css-entries.md)
>
> **▶ M2 切片 173（2026-10-31 收口，html5ever 0.29.1 声明式 shadow 解析缺陷绕道 + register/ensure 惰性镜像——双靶 F→P、首刀 18 键 P2F 根因拆分清零，rust 5 + shim 2 + 新预变换模块）**
> **入场定刀位**：s168 施工后遗留双案红（form-indexed-element-shadow / form-submit-dialog-shadow）+「shadowRoot 恒 null」FIXME。勘根升级定谳**解析层出生即坏**：html5ever 0.29.1 `should_attach_declarative_shadow` 值精确匹配 `"open"`/`"close"`（spec 枚举 open/closed，html5ever 自身缺陷）——命中即 template 不入树、内容摊平进宿主，s168 materialize 读侧对 `open` 形态从未见过模板；`closed` 反而正常。
> **施工**（7 文件）：①新模块 `declarative_shadow.rs`（323 行）——parse 前预变换 `<template>` 内 shadowrootmode 值首字母大写（spec 枚举 ASCII 大小写不敏感语义不变；tokenizer 镜像扫描：名字边界 + 引号感知 + 属性名前界 + RAWTEXT 区跳过；6 单测；FIXME(html5ever) 0.38 升级移除）；②parser.rs 接线（无 shadowrootmode 零拷贝 fast path）；③④bridge deep JSON 生产函数 + `__zw_declarative_shadow_deep`/`__zw_declare_fragment`（给定句柄 fragment 绑定）wire；⑤钉测 s173；⑥part04 shadowRoot getter 接 hydrate；⑦part05 **register/ensure 分离**——getter 只零 mutation 登记（`__ds{n}` shim 本地句柄防 HANDLE_COUNTER 碰撞 + `_zwDsRegMiss` 负缓存），`_zwDeclarativeShadowEnsureMirrors` 惰性镜像只挂 R2928 `_handleQueryFirst`/`_handleQueryAll` 顶部（真内容查询才付结构批代价）。
> **回归勘根（本刀内闭环）**：首刀 hydrate 即建镜像 → structural batch → `render_with_dom_mutations_persistent` 全快照重建（slotmap 换代）落 initial-script 时点 → dialog-popover-closedby-complex light-dismiss 代际偏斜 **18 键 P2F**（423P/60F）；探针 A/E bit-exact 定谳后拆分清零。教训：getter 类 API 绝不能触发 structural batch。
> **验证**：dom lib 880P/0F（6 单测）；engine lib **3615P/10F**（+1 钉测，10F 基线同键）；webview lib 825P/0F；定向双靶 F→P（s168 账面各 1F）；interactive-elements 终态 **437P/26F/463** 对 s172 基线 439P/44F/483 (key,state) 严格 join——**P2F=0、F2P=2**（靶案 + dialog-focus-previous-outside shadowroot-outside 案）、**LOST=20（4P+16F）**= dialog-popover-closedby-complex shadow-slots 全段（内容**枚举** deferred 段自中断；4P 为摊平 DOM 凑巧 Pass 的 modeless 组合，double-error 抵消假 Pass 清除归因非回归）、GAINED=0；forms 家族 96P/12F 全既有（form-elements-filter stash 双态对照 HEAD 同键；其余零 shadow 关联 inert）；fmt/guarded-clippy 全绿；shim 装配序拼接 node --check SYNTAX_OK；探针验后删全清。
> **FIXME 新记**：声明式根内容**枚举** deferred（LOST 20 归因面，下轮候选①）；shadow 内容不渲染 R2926 承接 defer；innerHTML/fragment-parse 不跑 materialize；嵌套级联 + 镜像写穿 render 树 defer。
> **下一步候选**：①声明式根内容枚举（本刀 LOST 20 直接承接）；②webview 静态 blob 路径 entries 化（s167 挂账）；③ua_display_li（style-system 跨流）；④selection-pointer forms 域翻绿归因勘定（s172 记）。
> **账本**：[evidence/2026-10-31-m2s173-html5ever-declarative-shadow-prepass.md](html-semantics-compat/evidence/2026-10-31-m2s173-html5ever-declarative-shadow-prepass.md)
>
> **▶ 上轮状态（2026-10-31 M2 切片 172 收口，getElementById 幽灵命中根修——name-attribute 五连 F 销账，pending 索引出树候选剔除 + overrides 门链，shim 单文件 + 钉案）**
> **入场定刀位**：s171 挂账归因+根修。runner 语料探针五层剥开：①互斥扫描无罪（单次 Scan 零 Exclusivity、全树无 ghost、e2 open 属性在树上）——「跨群误关」语义面反转；②读侧分裂（gEBI proxy 读 open=0/attr=null vs 树实节点 open=1，sameObj=0）；③身份铁证（gEBI 结果 handle `__n0` = 前案 createElement 版 e2 幽灵 proxy）；④路径（旗标插桩）= pending-ID 索引 poolN 臂，`_r4006TreeCmp` 出树候选不可比返 0 插入序保旧；⑤根因双层 = innerHTML= 替换不走 removeChild 流（无 pending-removed 登记）+ in-doc 门走宿主快照 stale 且 handle 反链悬指旧挂载点；伴生 `_zwIdOverridesEntries` 臂裸返零门。
> **施工**（part06.js 单文件 + 钉案）：新增 `_r4006LiveRooted` 正向位次验证（每跳先验子在实际父 childNodes、达 document 即 rooted、链断无 pending 槽即出树）补 pending 索引三臂（fast/pool/R334 resel）；overrides 臂补 R125 门链逐字同口径；探针形态矩阵转常驻钉案 `test_get_element_by_id_stale_replacement_s171`。spec 同向收紧：gEBI 只搜文档树。
> **验证**：name-attribute 整案 **18 案全绿**（五连 F 销）；details 目录 42P/1F（布局既有）；interactive-elements **439P/44F/0T** F 集合逐键 diff 零位移唯靶案翻绿；webview lib **824P/1F**（+1 钉案绿，image_decoder 既有）；engine lib **3612P/11F** stash 二分同名 SAME 零回归；全域 **7250P/753F/84T** join s167 full1——Fail→Pass 36（靶案 + closedby simple ×3 + selection-pointer ~22 意外收益域）、Pass→Fail 2（css-module/json-src stash 对照 HEAD 同态，零归因本刀）、新报出 126、Timeout 键互换 3。fmt/guarded-clippy 全绿、shim 装配 SYNTAX_OK。
> **FIXME 销账**：s171「name-attribute 跨群互斥误关」resolved——实为 gEBI 幽灵 proxy（pending 索引 + overrides 臂），根修已落。
> **下一步候选**：①嵌套 shadow 级联（s168 commit1 FIXME 承接，`_shadowRoots` 顶层遍历嵌套面不入）；②selection-pointer forms 域 22 案翻绿归因勘定（修复收益 vs 负载差，回红再查）；③webview 静态 blob 路径 entries 化（s167 挂账，生产 renderer 多进程组合语义）；~~④style_sheet_from_handle_r113 期望同步小刀~~（**同日 resolved** eabdd5efa，引擎基线 11F→10F）。
> **并发复证**：双会话同树推进同一切片（rollover 双触发）——先commit 方落 8d29b5d6a 码+账；复证方（本行）对已推代码独立复跑全绿（定向 18P、域 439P/44F 严格 join 唯靶键 F→P、webview lib 825P/0F solo、clippy/fmt 复证），增量记账并入同账本「并发会话复证」节。
> **账本**：[evidence/2026-10-31-m2s172-gebi-ghost-nameattr-rootfix.md](html-semantics-compat/evidence/2026-10-31-m2s172-gebi-ghost-nameattr-rootfix.md)
> **▶ 上轮状态（2026-10-31 M2 切片 171 收口，closedby 漂移域 solo 冷却钉态复跑 + name-attribute 跨群分叉勘根——s167 挂账销账 + 新挂账，纯复跑零代码改动）**
> **入场定刀位**：s167 挂账「closedby/details 漂移域 solo 冷却复跑钉态」——s170 时 load 92 不成立推迟，本轮 load 5.58 冷却态成立。FILTER=interactive-elements solo 复跑 438P/45F/0T（Timeout 0 无截断）。
> **钉态定谳**：closedby complex+simple 46 键与 s166 full1 基线**逐键一致**（30P/16F）——s167 重载态 P2F 11 键（complex 8 + simple 3）全部 Fail→Pass 回绿，漂移定谳环境时序抖动（hit-test 落点），域钉态回归 s166 基线；auto-expand IDENTICAL。s167 挂账 **resolved**。
> **新挂账（未定谳）**：P2F 13 余下 name-attribute 1 键 solo 态**保持 Fail 不回绿**——`handling of name attribute changes` expected `[1,0,1]` got `[1,0,0]`，语义 = innerHTML 注入**跨 name 群独立性**（`name="a"` 群 + `name="b"` 群，b 群 e2 被 a 群互斥误伤 closed）；同目录其余 name 群案全绿。时间线：s161/s162/s164/s166 四基线 P → s167 stash 二分（a897d536b）起五连 F 稳定（solo 全域 + 定向×2 同二进制）——非单跑抖动，s166 full1 二进制与 s167 重建二进制间持续分叉。c1f8e9b63（03:00 并发流撞车仲裁）已排除：diff 全在 CE caret 世界同步路径，门 try/finally 无泄漏面，与 details 零交集。嫌疑面：s166 树本身已 F（需 bisect s164↔s166）或 release 重建时序分叉（同二进制稳定 F 反驳随机抖动形态）；解析面 `__zwDetailsScan4100` given-prong 分群逻辑读毕正确，误关路径未定位。
> **下一步候选**：①name-attribute 跨群互斥误关归因+根修（worktree 检出 a897d536b 重建 runner 二分定谳，rusty_v8 档案 + wpt-data 语料手拷前置；或解析注入属性时序面直接勘根）；②嵌套 shadow 级联（s168 commit1 FIXME 承接，顺延）。
> **账本**：[evidence/2026-10-31-m2s171-closedby-solo-repin-nameattr-divergence.md](html-semantics-compat/evidence/2026-10-31-m2s171-closedby-solo-repin-nameattr-divergence.md)
> **▶ 上轮状态（2026-10-31 M2 切片 170 收口，scrollingElement compatMode 分派归位——s168c 账本新记承接，shim + webview 测试 2 文件）**
> **入场定刀位**：s168c 后候选③。s151 已真值化 compatMode getter（宿主 quirks_mode 桥），但 shim scrollingElement 恒返 documentElement 无视 quirks——HTML §3.1.1 要求 BackCompat → body element、CSS1Compat → documentElement。伴生测试假钉：r3259 案 fixture 缺 `<!DOCTYPE html>`，实际跑 quirks 模式断言 standards 结果，对分派不敏感双向掩盖。
> **施工**（2 文件）：`js_dom_shim/part06.js` scrollingElement getter 改 compatMode 分派（BackCompat → `document.body || null`，否则 documentElement；body getter 同处已有 frameset 分派，body 缺位返 null 同 spec）；`webview coverage.rs` r3259 fixture 补 doctype 钉 standards 面 + 新增 quirks 案（BackCompat + BODY 标签 + 与 document.body 同一性 + 非 documentElement）。
> **验证**：webview lib **823P/1F**——双 scrolling 案翻绿，唯一 F = image_decoder 环境案既有；engine lib **3612P/11F** 基线同集零回归；shim 拼接 SYNTAX_OK。使用方抽查 keyboard-scroll / scrollable-overflow-fixed-positioned 均 0P/0F/0T——两文件在 wpt-data css/css-overflow|css-contain，不在 testharness-html-semantics 语料域（html/semantics），属域覆盖事实非回归；standards 分派不变（doctype 前提）由 webview lib 双案 + engine lib 基线承载。fmt/guarded-clippy 全绿；reftest/product-smoke 未跑（shim 属性 getter 查询面，无渲染管线变更）。
> **下一步候选**：①closedby/details 漂移域 solo 冷却复跑钉态（s167 挂账；s170 入场时 load average 92、并发会话 testharness-editing-other 占 104% CPU，solo 冷却态不成立，复跑无信息量，需负载空闲窗）；②嵌套 shadow 级联（s168 commit1 FIXME 承接）。
> **账本**：[evidence/2026-10-31-m2s170-scrolling-element-quirks.md](html-semantics-compat/evidence/2026-10-31-m2s170-scrolling-element-quirks.md)
> **▶ 上轮状态（2026-10-31 M2 切片 169 收口，color_scheme_decl 序列化面勘根——s165 挂账销账 + 归一化真修点，rust 3 文件 + 4 单测）**
> **入场定刀位**：s165 挂账「无声明后代经 parent-copy 获 decl（序列化现父值）」勘根。形态矩阵 4 案（新增单测）实证两分：①**parent-copy 非偏差，销账**——css-color-adjust-1 §color-scheme-prop Inherited: yes，无声明后代 computed = 父 computed，gCS 输出父值即继承语义；「decl None→normal」回落仅根/链首未声明（meta 只入 used fallback）。computed 面（decl 串）与 used 面（dark 位，normal 回落文档 scheme）分离且各自正确——父 normal+子无声明：子 decl=Some("normal") 同时 dark 位随 meta，实证挂账疑虑混淆两面。②**真修点 = 显式声明值未归一化**——`color-scheme: DARK   Light` decl 原样存串，gCS 原样输出；CSSOM computed 序列化要求 keyword 小写 + token 单空格（cssom-1 §serializing-css-values）；used 面 `eq_ignore_ascii_case` 不受影响，纯序列化面偏差；语料 14 文件无 colorScheme gCS 断言（全走系统色渲染面）、css-color-adjust 域不在本地——即 s165「语料未覆盖」所指。
> **施工**（2 rust 文件）：`inheritance.rs` Concrete 臂 decl 归一化（split_whitespace + 小写 + 单空格 join，带 cssom-1 锚）；`engine computed_style.rs` 序列化臂注释对齐定谳（decl=computed 值串、继承/回落分界）；`color_scheme_synthesis.rs` +4 形态矩阵案（继承父值/全链 None/继承显式 normal/归一化规范形）。
> **验证**：style-system lib **2467P/0F**（归一化案修前红修后绿）；engine lib **3612P/11F** 基线同集零新增；color-scheme 语料 **24P/0F** 零回归（语料输入全小写）；fmt/guarded-clippy 全绿；reftest/product-smoke 未跑（gCS 序列化通道 readback-only，沿 s165 先例记账）。
> **FIXME 销账**：s165 记账面 1 resolved——parent-copy=继承语义（实证），归一化已修。
> **下一步候选**：①closedby/details 漂移域 solo 冷却复跑钉态（s167 挂账）；②嵌套 shadow 级联（s168 commit1 FIXME 承接）；③webview scrollingElement compatMode 断言归位（s168c 账本新记，可选）。
> **账本**：[evidence/2026-10-31-m2s169-color-scheme-decl-serialization.md](html-semantics-compat/evidence/2026-10-31-m2s169-color-scheme-decl-serialization.md)
> **▶ 上轮状态（2026-10-31 M2 切片 168c 收口，桥基座失配收口——stylesheet-media-change-no-fetch 翻绿，the-link-element 目录 0F，WebView 2 文件 engine 零改动）**
> **入场定刀位**：s168b 挂账既有 bug 候选①。探针矩阵（zz-s168c-probe 五案，验后删）钉边界：动态 sheet link append 后**同步** gCS 全 Pass、**+1 tick** gCS(p)=`''`（p4 唯一 F）——drain 后窗口专属，与 link 语义无关（任何「drain 消费全部 mutation 后读动态元素」形态皆触发）。
> **勘根**（插桩四步钉死）：①engine 视图重建臂 drain 后 `applied=25 mlen=25 start=25 nodes=0`——重放区间空，视图=parse(基座)；②基座内容=**探针自身源码**（初始页面，html 无 `<p`/`<link`）——forward 位次 selector `p:nth-child(9)` 锚定 miss → 返空串；③webview apply 臂 `ok tail=25 snap_some=true handles=8`——apply 成功、`cached_html` 已含全部动态元素；④失配定位：gCS 回调读**闭包捕获的 dom_html Arc**，内容停留武装时点——apply 只更新 `cached_html` 不换 Arc，基座刷新只发生在「重注册换新 Arc」站点，而 runner 探测环 `execute_script` 武装一次化（form-url-encoding CI 回归修）后 apply→下一脚本之间**无任何重注册**——s143 契约「查询基座=最新 cached_html」前提（游标推进必经重注册，MUT_APPLIED_CURSOR 文档）被打破。selector 基 gCS 不受影响（元素在初始基座 + latest-wins overlay），handle 元素必须靠基座物化。
> **施工**（WebView 2 文件）：`webview.rs` 新字段 `dom_html_bridge`（最近注册站 Arc，6 站登记）+ `sync_dom_html_bridge()`（内容差分就地写 + `bump_dom_view_gen`，R358 模式 apply 侧补全）；同步点四处各配不双计论证——apply 成功臂（游标发布配对）/apply_mutations_subset/sync_render_after_native_dom（native 写不入队列）/refresh_if_html_changed（门控：队列无 pending 才同步，否则 shim 真相已含 pending 效果重放双计）。perf：bump 只落 apply 边界，turn 内链式增量不受影响。
> **验证**：fmt/guarded-clippy 全绿；product-smoke 6/6；靶面 **F→P**；the-link-element **115P/0F**（基线 114P/1F 净 +1P−1F）；document-metadata **215P/1F/9T**（F/T 集合 diff 仅目标案）；implicit-submission 3P/0F + image-map 9P/0F（s143/尾簇13 敏感面零回归）；engine lib **3612P/11F** 基线同集净 +1P；webview lib 821P/2F 双 F 既有（scrollingElement stash 复证 HEAD 同败——无 doctype 断言 CSS1Compat 与 spec 相悖，新记 FIXME）。
> **下一步候选**：①s167 遗留 `color_scheme_decl` 继承序列化面勘根（s165 挂账）；②closedby/details 漂移域 solo 冷却复跑钉态；③webview scrollingElement compatMode 断言归位（账本新记，可选）。
> **账本**：[evidence/2026-10-31-m2s168c-bridge-base-resync.md](html-semantics-compat/evidence/2026-10-31-m2s168c-bridge-base-resync.md)
> **▶ 上轮状态（2026-10-31 M2 切片 168 commit 2 收口，Shadow CSS 级联下发——靶面 shadow 案 a1–a4 全绿，勘根四症结 + 既有 bug 一枚，非探针 +1P/−1F 零 P2F）**
> **入场定刀位**：s168 commit 1 落地后续接（上轮 429 中断遗留 WIP：ACTIVE_SHADOW_CSS_ENTRIES 槽 + `compute_styles_with_shadow` 填表 + shim shadow 枚举已成形，靶面 a1 即卡）。勘根四症结（全部插桩/探针钉死）：①light 覆盖门吞 shadow 下发——s168 物化产物进 `__zw_query_all` 全域枚举而表键是 wrapper handle 形，覆盖门双 miss 整体 return 拦掉尾部全部下发（a4 清空死 + p2 初始死）；修复 = shadow 收集块前移 light 枚举前 + `_rcShadowSels` 身份集合 light 循环跳过；②query 枚举键形漂移——`__zwSelector` 物化产物恒 null，实际发 id 派生 `#sl1`（id 有）/裸 tag `link`（id 无），身份集合补 `#<id>` + tag 小写两形；③part04 双 rel 臂 TAG 门严格 `=== 'LINK'` 阻挡 shadow 物化 link（`_realTag` 宿主表 miss 恒 DIV），修复 = 同 media 臂 handle-only 容忍式；④s168 memo 臂先行于 s154 印记门无 fired 守卫——同值 attr touch 二次派 load，link-multiple-load-events 四案 P→F，修复 = `_zwSsFiredMap[key] !== abs` 守卫（rel 移除臂清印记后再入仍命中，语义不变）。另修 tree_dirty 分支 shadow_scopes 占位 `Vec::new()`（`slot.10.is_empty()` 谓词失真 → shadow 页误入增量通道退化文档表级联）→ 现场 `collect_shadow_scoped_sheets` 重建。
> **施工**：7 文件（shim 3：part03 前移/集合/守卫、part04 双 rel 臂容忍、part05 inDoc shadow 爬升臂；rust 4：callbacks `__zw_set_shadow_css_entries` + cs_cache 键扩元 11 元组（两谓词逐字同步）+ `finish_generation_update` shadow 分支 + computed_style 汇点 + pipeline `collect_shadow_scoped_sheets`）。
> **验证**：check/fmt/guarded-clippy workspace 全绿；engine lib **3611P/11F** = 基线同集零新增；靶面 `link-rel-attribute` **3P/0F**（a1–a4 全绿）；诊断探针 zz-s168b-shadow-cascade-probe（p1–p6 链）F→P；目录 the-link-element **114P/1F**（剔探针 113P/1F，对 HEAD 基线 112P/2F 净 +1P/−1F）；document-metadata **214P/2F**（对 HEAD 213P/3F 净 +1P/−1F）；P2F 0。
> **挂账更新**：既有 bug 定谳——`stylesheet-media-change-no-fetch` 断言 1 gCS 全空（`''`）= 动态 id-less handle-only link 取回完成后文档 `<style>` 计算样式整体丢失，worktree 88ca4f5db 复证 **commit1 已然非本刀回归**（commit-1 账本目录口径与今日语料不同代，未覆盖此案），根因未勘挂下一步候选①；FIXME 新记——engine `__zw_query_all` 枚举排除 shadow 子树（spec：document.querySelectorAll 不见 shadow 树；shim 键形补全是兜底）。
> **下一步候选**：①`stylesheet-media-change-no-fetch` gCS 全空勘根（engine 动态 link 原生通道 × overlay 下发交互，worktree 复证链已在账本）；②s167 遗留 `color_scheme_decl` 继承序列化面勘根；③嵌套 shadow 级联（commit1 FIXME 承接）。
> **账本**：[evidence/2026-10-31-m2s168b-shadow-css-cascade.md](html-semantics-compat/evidence/2026-10-31-m2s168b-shadow-css-cascade.md)
> **▶ 上轮状态（2026-10-31 M2 切片 168 施工收口 commit 1/2，Shadow DOM 物化与查询封装门——attach_shadow mutation 落树 + light 查询封装化 + 声明式序列化回环 + shadow 作用域走查空表语义，探针 p9/p10 翻绿零回归；commit 2 级联下发待续）**
> **入场定刀位**：s167 候选②（`link-rel-attribute` shadow 案，目录余留唯一 F）。勘根（探针三态验后删）：p9 cloneNode 变体 inline-style 反射 `''`、p10 importNode 变体黑、p11 `.green` 级联黑——统一机制 = shim shadow 容器是 fragment handle，engine 侧 flatten 落 **detached fragment**（styles map 无条目 → serialize 黑；replay Err 截断 → nodes miss → `''`），级联走查永不访问 shadow 子树；layout 侧 shadow+slotting 基建休眠只等树里有内容。
> **施工 commit 1（10 文件 +411/−16）**：engine 4（js_dom_bridge.rs `AttachShadow`/`AttachShadowOnHandle` 双变体翻译/执行臂——root_handle 注册 ShadowRoot NodeId 后既有 flatten/append 零改动自然落树；callbacks.rs `__zw_attach_shadow(_handle)` 注册；part05.js `_attachShadow` 尾部 wire 上报；computed_style_cache.rs gCS by handle miss 臂 `query_selector_including_shadow` 身份通道回落）+ dom 4（document/mod.rs `is_shadow_root_node` 九遍历下降点 light 查询封装门 + get_element_by_id enclosure 校验空表快门；document/shadow.rs `shadow_root_ids`/`query_selector_including_shadow`/`materialize_declarative_shadow_roots`；serializer.rs ShadowRoot arm 输出 `<template shadowrootmode>` 声明式形态——drain 重序列化回环不丢 + inner_html shadow 跳过；parser.rs 解析后 materialize）+ style-system 2（lib.rs `ShadowScopeStylesheets`/`ShadowScopeView`/`compute_styles_with_shadow` 17 参——子代循环 ShadowRoot 边界换传 per-scope pair、预扫并集 + cache_safe AND、增量 shadow 包含退化门；font_feature_values.rs IntoIterator 化）。commit 1 级联语义 = shadow 子树走查**空表**（UA + host 继承）。
> **验证**：check/fmt/guarded-clippy workspace 全绿；engine lib `js_dom_bridge` 3611P/11F = s167 基线同集零新增；shim 13-part node --check 过。探针：**p9/p10 `''`/黑 → Pass**（物化 + 身份通道双臂）、p11 保持红（级联 commit 2 面，blob `.green` 不漏入）——三态与勘根逐位对应，验后删。目录 the-link-element 剔探针 **113P/1F 与 s167 账面逐位一致**零位移。`shadowrootmode` 语料命中面 5 案定向（materialize 实际作用域）：form 两案 1F = M1 同态、focus-previous-outside 3P/3F = s167 stash 二分位集、closedby-complex 24P/16F 落 s167 钉定漂移带——零 P2F。过程注记：runner 直跑 `FILTER=x` 带前缀字面量进路径子串匹配恒假 → 空集静默 exit 1（make 目标剥前缀，直跑须裸值），插桩二轮定谳非回归已还原。
> **挂账更新**：FIXME 新记 ×3——嵌套 shadow 级联（查询/走查/声明式重建均不入嵌套面）、`document.outerHTML` 含声明式模板（Chrome 不含——回环需要、上游对照语义差）、`getHTML()` serializableShadowRoots 分型未建模。
> **下一步 commit 2（s168b 级联下发）**：ACTIVE_SHADOW_CSS_ENTRIES 槽（host_sel/sel/text）+ `__zw_set_shadow_css_entries` + 缓存键扩元（两谓词逐字同步纪律）+ collector `collect_shadow_scoped_sheets` + `compute_styles_with_shadow` 填表 + shim `_zwSsRecomputeExt` shadow 循环 + a2–a4 反应链 → 靶案 a1–a4 → 目录 113P/1F 收口 0F。s168 后候选：②`color_scheme_decl` 继承序列化面勘根（s165 挂账）；③closedby/details 漂移域 solo 冷却复跑钉态。
> **账本**：[evidence/2026-10-31-m2s168-shadow-materialize.md](html-semantics-compat/evidence/2026-10-31-m2s168-shadow-materialize.md)
> **▶ 上轮状态（2026-10-31 M2 切片 167 施工收口，外链槽位级联序——结构化条目槽 + collector 文档序交错装配，stylesheet-media-change 断言 2 翻绿整案 Pass + stash 二分零归因）**
> **入场定刀位**：s166 候选①（s161 FIXME 精确化升格首选），靶面 `stylesheet-media-change-no-fetch` 断言 2。勘根承 s166 证据链：collector 外链 blob 整体前置（push 先于文档 `<style>` 循环）→ link 在 `<style>` 之后的页同特异度恒输（CSS Cascade 5 author origin 按引入元素文档序，https://drafts.csswg.org/css-cascade-5/#cascade-sort）；overlay 通道同病（`__zw_set_external_css` 单 blob 落同槽同位——合并文本无位次信息，修序必须把引入元素位次从 shim 穿到 collector）。
> **施工**：4 文件——①结构化条目槽 `ACTIVE_EXTERNAL_CSS_ENTRIES`（sel/text 按源序，computed_style_cache.rs）+ 双写者所有权互斥（blob 写者 `set_active_external_css` 清 entries——webview 静态发布/导航复位经此自然回落；entries 写者派生 blob 按序 `;\n` concat 与旧 `_rcOut` 字节同构——CSSOM wire memo 与缓存键 css 分量零迁移）；②collector 分支化（pipeline/mod.rs：entries 空 → blob 前置形态逐位不变；非空 → `collect_interleaved_external`——`get_elements_by_tag_names(&["style","link"])` 单次 DFS 文档序 + `find_by_selector` 预锚 NodeId→idx + style 节点先消费锚定 @import 条目再推 base sheet（import 位次）+ link 走查期 rel/href 复核（s158 五字符集口径）+ 未锚定尾接（selector 漂移保留「表在场」语义，rescue/slice33 同暴露面）；s160 三门提取共用 `style_sheet_if_eligible`）；③shim `_zwSsRecomputeExt` 两循环逐条 push（text 为 s166 wrap 后文本，NUL→U+FFFD 同 CSS preprocessing）+ 下发分支化（`__zw_set_external_css_entries` wire `\u{1}` 分记录 `\u{0}` 分 sel/text；回调缺失回落 `__zw_set_external_css(_rcOut)` 旧 shim 兼容）；④缓存键 entries 分量全覆盖（GENERATION_CACHE 8→9 元 + `try_sync_with_drain_record`/`finish_generation_update` 签名与 dirty/recollect + `generation_cache_matches`/`with_cached_document_styles` same_gen 谓词逐字同步 + HANDLE_VIEW_CACHE 10→11 元 + callbacks cs_cache 6→7 元——**派生 blob 字节巧合相等时仍换代**）。跨线程零断链：entries 槽与 blob 槽同为 js worker 线程 thread_local，渲染线程 collect 走显式 `session.css` 参数、其线程 entries 恒空 → 自然回落 blob 形态；gCS/handle 视图全路径经同一 collector（`compute_styles_for_doc` 共路）entries 模式全局一致。
> **验证**：check/fmt（hunk 全落本次编辑函数）/guarded-clippy workspace 全绿；engine lib `js_dom_bridge` 1359P/11F = s166 基线同集；shim 13-part 装配序 `node --check` 过。定向：**靶面整案 Pass**（断言 2 `expected rgb(0,128,0) got rgb(255,0,0)` 翻绿 + a3 随整案收口）；the-link-element 目录 112P/2F → **113P/1F**（余 1F shadow 案=候选②）；s166 饿死解除三案金丝 3/3 保绿。全量 join（对 s166 raw 7952 对，行首过滤）：**F2P 1 = 靶案 / P2F 0 归因本刀**——13 P2F（closedby complex ×8 + simple ×3 + name-attribute ×1 + auto-expand ×1）经 **stash 二分**（四文件 stash → HEAD a897d536b 二进制重建 → 定向复验）**位集逐键一致**定谳环境时序漂移域延续（s161 在册 closedby solo 当日漂移先例；机制面 = testdriver 坐标点击 + fixed 覆盖层 hit-test 抖动，轻载/重载同态非负载因素）；LOST 26 全镜像（select-event ×10 截断深度差 + focus-dynamic/name-attribute Timeout 键互换 + P2F 对侧 13 + 靶案旧键 1）；GAINED 35（focus-dynamic 完整上报 19 + 漂移域新态 15 + 靶案）；raw **7090P/787F/87T**。
> **挂账更新**：FIXME 解除 ×1——外链槽位级联序（s161 记 → s166 精确化 → 本刀槽侧根落地）；挂账 +1——webview 静态 blob 路径未 entries 化（渲染线程回落形态即现状语义，生产 renderer 多进程组合语义沿用 s158 在册 FIXME）。
> **下一步 s168 候选**：①shadow DOM 级联面（`link-rel-attribute` shadow 案，目录余留唯一 F）；②`color_scheme_decl` 继承序列化面勘根（s165 挂账）；③closedby/details 漂移域 solo 冷却复跑钉态（可选，非刀口）。
> **账本**：[evidence/2026-10-31-m2s167-external-slot-cascade-order.md](html-semantics-compat/evidence/2026-10-31-m2s167-external-slot-cascade-order.md)
> **▶ 上轮状态（2026-10-31 M2 切片 166 施工收口，跨批 handle 正置表锚定 + media 翻转链三修——stylesheet-media-change 断言 1 翻绿（键保 Fail）+ 全量 join F2P 3 覆盖门饿死解除 / P2F 0）**
> **入场定刀位**：s165 候选①（R51c 查询视图扩展）+ 候选③前半（recompute media wrap 死代码），靶面 `the-link-element/stylesheet-media-change-no-fetch.html`。账本（s161）记载失败点断言 2，实测前移断言 1（got `""`）。
> **勘根定谳**：三层——①断言 1 根因 = **跨批 handle 边界**（gCS-by-handle 视图 `nodes` 只覆盖重放后缀，前批 CreateElement 物化进基座后 `nodes.get(handle)` miss → 恒 `''`，s161 FIXME 在册；修法复用 R100 正置表——`FORWARD_HANDLE_MAP` 已被 `__zw_get_tag_handle` 同法解跨批身份，gCS 通道未接入是唯一缺口，零新增 plumbing）；②断言 2 探针 8 轮（验后删）链路面三缺——media sel-form lw 读盲（`sel_attr_override` 仅 selector 键，SetAttrOnHandle 不可见）、覆盖门单键形（存表 `@handle` 形/查找 sel 形失配整体 return）、`_rcText` wrap 装填未消费（s161 死代码）；③probe g/h 判层（直接注入同特异度不翻绿、高特异度翻绿）→ 通道全通，残余 = **外链槽位级联序**（collector 外链槽 push 先于文档 `<style>`，s161 FIXME 精确化——跨切片债本刀不扩面）。
> **施工**：3 文件——`computed_style_by_handle` 加 `forward` 参数 + miss 路径经正置表锚 selector 视图 doc `find_by_selector` 回定位（与 rescue/slice33 同暴露面）（computed_style_cache.rs）；回调传 `sel_handle_map_snapshot()`（callbacks.rs）；`_zwSsRecomputeExt` 三处 media handle-first 读 + 覆盖门双键形 + `_rcOut` 消费 wrap 文本（part03.js）。
> **验证**：check/fmt/guarded-clippy workspace 全绿（fmt 对上游区域扫动已回退归口不动）；engine lib `js_dom_bridge` 1359P/11F = HEAD A/B 基线同集；定向断言 1 翻 P（失败点移断言 2）；the-link-element 目录 112P/2F 双 F 归因（a2 债 + shadow 案基线红）。全量 join（对 `/tmp/s165-full-run.log` 8036 对，行首过滤——基线为 make+tee 包装混入噪声，未过滤首跑 308 LOST 假位移）：**F2P 3**（conditionally-block ×2 + style-element-media-match ×1——基线断言级 Fail 逐字对上本轮 Pass；归因 = 覆盖门双键形饿死解除，旧门单键形 miss 整体 return 饿死该页全部 overlay 下发含 s164 style 文本通道；单样本触发页形未二分，三案保绿转 s167 金丝复验）/**P2F 0**/LOST 90 全截断镜像（select-event 75 + focus-dynamic 8 深度差 Timeout 键两轮均在、name-attribute 镜像、dialog/style_events 文件键互换）；raw **7091P/775F/89T**（基线 7172P/776F/88T，ΔP −81 截断净差）。
> **挂账更新**：FIXME 解除 ×2——跨批 handle 引用边界（正置表锚定回落落地）、link 通道 recompute media wrap 死代码（消费落地）；精确化 ×1——外链槽位级联序（勘根证据链齐：collector push 序 + probe g/h 高特异度对照 + 双通道翻绿）升格 s167 首选；新记 ×1——pending 键形盲区（SetAttrOnHandle 对 sel-form lw 读不可见，同类读点未普查）。
> **下一步 s167 候选**：①外链槽位级联序（collector/webview/shim 三层文档序交错，本刀证据链齐，需全语料 A/B 独立切片）；②shadow DOM 级联面（link-rel-attribute shadow 案）；③三案金丝复验 + 覆盖门饿死触发页形二分；④color_scheme_decl 继承序列化面勘根（s165 挂账）。
> **账本**：[evidence/2026-10-31-m2s166-cross-batch-handle-anchor.md](html-semantics-compat/evidence/2026-10-31-m2s166-cross-batch-handle-anchor.md)
> **▶ 上轮状态（2026-10-31 M2 切片 165 施工收口，meta color-scheme 文档 scheme 三层栈 + gCS 结构性重放通道——color-scheme 语料 24 靶断言全绿，全量 join 零回归）**
> **入场定刀位**：s164 候选① meta color-scheme 三重栈续拆（used-scheme 传播 + 系统色动态解析——24F 无主池）。切片号注记：施工期原标 s164，并发会话已按 s163 候选清单①占号施工（其 /tmp 工件族与控制面块可证），落账改号 s165、注释同步，与 s164 块「下一步 s165 候选①」投影吻合。
> **勘根定谳**：四层——①used-scheme 系统色静态化（CanvasText 族按 prefers 一次性解析，meta dark 后仍黑）；②文档级 meta color-scheme 整体缺位（HTML §4.2.5.3：树序首个有效、无效 content 跳过、shadow tree 排除、显式 normal 回落文档 scheme 非 parent）；③既有 R2285/R2286 注入冲突（`html{color-scheme:content}` 使 used-scheme 生效但根 computed 序列化变 "dark" 违背 'normal' 断言 + shadow 泄漏——两需求相反不可两全）；④gCS replay 通道结构性失明（attr/style 四类之外变异静默丢弃 + 跨段 handle 身份丢失——同 turn gCS 与变异交错切段后 fresh nodes map 不跨留存，InsertBefore ref 失配）。
> **施工**：12 文件——系统色 dark 面板（css-parser color.rs）；`scan_document_color_scheme`（style-system lib.rs，shadow 排除 + first-valid-wins）；Concrete 臂 normal 回落 + `color_scheme_decl` 字段（inheritance/property/computed_style/default_impl）；主循环臂收口预解析（apply.rs）；注入块移除（pipeline/mod.rs）；gCS 树级后缀全量重建 + `is_tree_mutation` 判定 + E14 gen 归零防错切（computed_style_cache.rs）；gCS 序列化臂 decl None→"normal" + 兜底换权威 applier（js_dom_bridge/computed_style.rs）；InsertBefore/ByHandle ref handle 表回落（js_dom_bridge.rs）；shim insertBefore sel 父 + handle-only ref 分支（part04.js）；+4 单测（color_scheme_synthesis.rs）。
> **验证**：shim 拼接 node --check SYNTAX_OK；style-system 2461+5 双 target 全过；engine lib 3612 过/10 基线既有败（stash 对干净基点 b51d8981a 复验 clone 既有：fetch wire 9 + ua_display 1）；定向 corpus 25P/0F/0T（探针删后复跑零新增回归）。全量案级 join（full1 对 s162 full1，(key,state) 对 sort -u）：**F2P 24 全靶面** / P2T/T2P/T2F 0；P2F 2 = conditionally-block 双断言归因**基线树态含 s163**（基点先于 s163 提交 00:01，solo 复跑确定性 F 确证非本刀回归，rebase 后由 s163 提交带回）；LOST 4/GAINED 154 全落交互时序族文件级截断镜像（base 截断→now 完整，改善方向）。raw 终账 **7172P/776F/88T**（rebase 后投影 ≈7174P/774F/88T）。reftest/product-smoke 未跑——无渲染管线变更（系统色在 style 解析面、gCS readback-only、shim 限 insertBefore wire 形态），24 断言常驻 corpus 承载回归，沿 s161-s163 先例记账。
> **挂账更新**：FIXME 新记——color_scheme_decl 继承序列化偏差（无声明后代 parent-copy vs spec used/normal 表达，语料无形态）；跨批 handle 引用边界（增量通道仍限八类，树级全量重建为规避非结构化修复）。承袭：CSSOM 规则接口族、视口 1280×800、R51c、@import 原位语义、动态 style @import 臂、link 通道 recompute media wrap 死代码。
> **下一步 s166 候选**：①R51c 查询视图扩展（动态 link 对 query_all 可见——stylesheet-media-change 断言 2 结构化修法）；②shadow DOM 级联面（link-rel-attribute shadow 案）；③link 通道 recompute media wrap 死代码修复 + 外链槽位级联序；④color_scheme_decl 继承序列化面勘根。
> **账本**：[evidence/2026-10-31-m2s165-meta-color-scheme.md](html-semantics-compat/evidence/2026-10-31-m2s165-meta-color-scheme.md)
> **并发撞号注记（2026-10-31 第二会话）**：本机另一 rally 会话同候选独立实现 s165 静态面
>（合成注入 + 深色表 + page_scheme 线程，20P/4F，全量 join P2BAD 0，css-color light-dark/
> system-color 池 stash A/B 逐位一致），rebase 时发现本刀已先行推送且覆盖更全（文档级
> scan 替代合成注入 + gCS 树级结构重放，24 靶全绿）——重复实现弃用（本地备份分支
> `wip/s165-duplicate-superseded`，commit 09589c837，reflog 可溯），其独立产出价值仅在
> 交叉验证：上游 HEAD color-scheme 池 24P/0F/0T 本机复现确认。
> **▶ 上轮状态（2026-10-31 M2 切片 164 施工收口，静态 style 元素 @import 取回通道——靶面 F→P + style_events 文件级 Timeout 搭车兑现 + 全量 join 零坏位移）**：
> **入场定刀位**：s164 候选① style-element-media-match-block-script 1F（s163 记账面——
> `<style>@import url('stylesheet.py?delay=1')</style>` 后随同步 script 断言 styleExists
> 经 `CSSImportRule.styleSheet` 递归逐字节比对 + gCS purple）。
> **勘根定谳**：三件套缺口——引擎 base 路径 style 文本 @-rule defer（零取回零级联）+
> shim 取回通道只覆盖 link + CSSOM 面引擎 wire @-rule defer（style 表 cssRules 恒空，
> styleExists 递归无 import 规则可下钻恒 false）。爆炸半径：静态 `<style>` 含 @import
> 仅 3 文件（match/not-match/style_events）；106/150 是 link 通道、其余 @import 族全
> 动态创建不在扫描快照。
> **施工（2 文件均 shim，rust 零改动）**：①`_ssVisit` 取回引擎平移模块级
> `_zwSsVisitCss`（无调用点特定捕获，逐字节平移零语义差）；②`_zwStyleScanVisit` 扫描臂
> ——type eligible 门 → `__zw_get_text_lw` 文本读 → `_zwStylesheetImports` 提取（零命中
> bail）→ media 门（s163 link 同款 no-match setTimeout(0) 推迟，窗外 delay 真延迟）→
> 聚合取回（任一 error → error）→ 落表 + `_zwSsRecomputeExt()` 级联下发（overlay 唯一
> 载体）；error → fire error（style_events tError 面接通）；③`_zwStylesheetStaticScanUpto`
> 建表加收 style + 扫描窗内 prefix 游标触发（同窗 trickle 折叠——delay=1 同步结算，
> script-blocking 先于脚本体；动态创建不入快照，「不阻塞」案语义天然保持）；④recompute
> 合并面 style 文本 media wrap 并入；⑤part06 `getRules()` 注入 CSSImportRule 形条目
> （`Object.create(CSSImportRule.prototype)` + `styleSheet` = 落表文本子表 + `__zwImport`
> 标记 flushToOwner 过滤不回写）。
> **验证**：shim 14-part node --check ✓（111943 行净 +159）；rust 零改动 fmt/clippy 零
> 波及。定向 interactions 目录 **9/9 全绿**（8P/1F→9P）：靶面 **F→P** + not-match/
> conditionally-block/script-created/dynamic 全保持。the-style-element join：**style_events
> 文件级 Timeout → 2P**（404 import error 事件搭车兑现，s161 起在册 T 键）；style-error-01
> /tentative 族基线同态零位移；s162 面全保持。execution-timing（link @import 内层通道）
> 全 Pass。全量案级 join（full1 对 s163 full1，LC_ALL=C）：**P2BAD 0**，唯一 OTHER =
> 靶面 F→P；LOST 5 = style_events 文件 T（rider）+ 4 在册交互时序域文件 T（A/B 定谳：
> dialog-closedby stash 基线 solo 同 27P/8F、selection 文件 T 自 s161 慢性、focus solo
> 104P 恢复、select-home-end 族 s161 在册 F）；GAINED 157（143P+14F）= select-event/
> dialog 族完整行集首报（s163 并发截断镜像）。raw 终账 **7156P/795F/87T**。reftest/
> product-smoke 未跑——级联变更只在静态 `<style>@import` 页显形，`tests/fixtures/` grep
> 零 @import 不可达（s163 先例同款记账）。
> **范围**：shim part03.js + part06.js。
> **挂账更新**：FIXME 新记——link 通道 recompute media wrap 死代码（`_rcText` 装填未
> 消费，s161 引入，现网被 webview 加载期门遮蔽）、@import 原位语义未建模（子表 face
> 读取序正确）、动态 style @import 臂未挂（style-error-01/style-load-after-mutate 各自
> 保持既有语义）。承袭：CSSOM 规则接口族、视口 1280×800 FIXME、R51c、meta color-scheme。
> **下一步 s165 候选**：①meta color-scheme 三重栈续拆（used-scheme 传播 + 系统色动态
> 解析——24F 无主池）；②R51c 查询视图扩展（动态 link 对 query_all 可见——stylesheet-
> media-change 断言 2 结构化修法）；③shadow DOM 级联面（link-rel-attribute shadow 案）；
> ④link 通道 recompute media wrap 死代码修复 + 外链槽位级联序（文档序翻转需全语料
> A/B 独立切片）。
> **账本**：[evidence/2026-10-31-m2s164-style-import-channel.md](html-semantics-compat/evidence/2026-10-31-m2s164-style-import-channel.md)
> **▶ 上轮状态（2026-10-31 M2 切片 163 施工收口，per-sheet styleSheets 通道 + 外链 media 推迟取回 + CSSOM 尾随分号——conditionally-block 2F→2P + 全量 join 零坏位移）**：
> **入场定刀位**：s161 候选①设计面（并发注记顺延重编号：原 WIP 标 s162 与已落账 s162
> style-load 撞号，本刀落账按指示改号 s163 并同步注释 10 处）。靶面
> conditionally-block-rendering-on-link-media-attr 2F（同步断言段 gCS purple got black +
> 3000ms 注册段）。
> **勘根三轮定谳**：①runner 端点 `interactions-of-styling-and-scripting/stylesheet.py`
> 缺失 + 引擎 base 槽外链 CSS 走 `wpt_data_image_fetcher` 文件映射**不过 fetch 桥**（.py
> 源码当 CSS 读，purple 恒缺级联）；②`styleExists` 逐字节比对依赖 CSSOM 序列化尾随分
> 号形（`h1 { color: purple; }`），旧 wire 无分号恒 false；③host sleep 在同步契约下同
> execute 内结算（取回时点对页面不后移，delay 对页面不可见——net-api M4-S20 delay.py
> 同根，既有 `_zwFetchSimDelay` 发题侧延迟通道复用）；④CSSImportRule 全局缺位使上游
> styleExistsInSheet 首规则 `instanceof` 即 ReferenceError（首轮被 assert_equals 先行失
> 败遮蔽）；⑤per-sheet 表存 fetch 原文非 wire（`_makeStyleSheet` 消费形错位）。
> **施工（6 文件：rust 3 + shim 3）**：css_wire 声明串尾随 `;`（serialize a declaration
> block 规范形，`_parseDeclarations` 尾空段天然跳过零波及）；runner 端点 + `serve_
> interactions_stylesheet` 双站点共用（fetch 桥 + image fetcher）；**delay 不 sleep**——
> part01 `_zwFetchSimDelay` 识别本端点 delay 参数发题侧延迟（30000ms cap）+ 扫描窗折叠
> （s152 trickle 同款）；part03 `_ssVisit` media 推迟取回（`__zw_match_media` 1280×800
> 不匹配 → setTimeout(0)，mixed 门同款异步形；匹配 link 同步先于脚本体，s152 语义保持）
> + `_zwSsExtSheetEntries` per-sheet 枚举口（R100 身份键派生与静态扫描同形无错位；R51c
> 动态 link 尾追近似）；callbacks 新桥 `__zw_style_rules_text`（原文→wire）；part06
> styleSheets per-sheet 模式（逐 link `_makeStyleSheet(null, text, href)`，零条目 blob
> fallback 保 R5114）+ CSSImportRule 空壳全局（FIXME 接口族整体建模记账）。
> **验证**：check/fmt/guarded-clippy workspace + shim 14-part node --check 全绿。定向
> 靶面 **2F→2P**（8P/1F，剩 style-element-media-match = @import 通道记账）。document-
> metadata 目录 join NEW-BAD 零、净 −5（含前段 s162 style-load 面复现 3）。全量案级
> join（full1 对 s161 full4，(key,state) 对 sort -u）：**P2F 0 / P2T 0 / T↔P/T2F 0**，
> 实质 F2P 3（靶面 2 + 前段 s162 复现 1）；终账 **7009P/783F/91T**；18 条 F2P 伪影 =
> select-event/dialog 族 5 文件 8-job 并发截断（solo 复跑全对齐 s161 基线逐位，dialog 族
> 已在册 flake），LOST 108 同源 + s161 探针文件已删。css-module 面（尾随分号潜在波及）
> 全量 join 仲裁零位移。reftest/product-smoke 未跑（无渲染路径变更——cssText readback-
> only、级联不经 wire；shim 时序限 testharness 面，沿 s161 先例记账）。
> **范围**：rust css_wire.rs + callbacks.rs + testharness.rs；shim part01/part03/part06。
> **挂账更新**：FIXME 新记——style-element-media-match 1F（style @import 通道 =
> engine 级联 + CSSImportRule.styleSheet 子表 + style 元素扫描，下一候选勘根面）、
> CSSOM 规则接口族未建模、link type essence 门枚举/fetch 通道同缺（语料无形态）、
> R51c 动态 link 尾追序近似（stylesheet-media-change 断言 2 仍 F）。承袭：meta
> color-scheme 三重栈、外链槽位级联序、跨批 handle 边界、视口 1280×800 FIXME。
> **下一步 s164 候选**：①style-element-media-match-block-script 勘根（style @import
> 通道三件套）；②meta color-scheme 三重栈续拆（used-scheme 传播 + 系统色动态解析）；
> ③R51c 查询视图扩展（动态 link 可见性——断言 2 结构化修法）；④shadow DOM 级联面
> （link-rel-attribute shadow 案）。
> **账本**：[evidence/2026-10-31-m2s163-per-sheet-stylesheets-media-defer.md](html-semantics-compat/evidence/2026-10-31-m2s163-per-sheet-stylesheets-media-defer.md)
> **▶ 上轮状态（2026-10-30 M2 切片 162 施工收口，style 元素 load 事件面三触发臂——靶面 F→P + 两文件级 Timeout 恢复 + 全量 join 零实质位移）**：
> **入场定刀位**：s162 候选③ `style_load_event.html`（1F）。勘根扩面定谳非单案——update a
> style block 的派发半整体缺位三形态：解析静态首派（初始 loadCount 0≠1）、动态插入/子
> 变更（style-load-after-mutate `await style.onload` 永挂 → 文件级 Timeout）、textContent
> 变更（style_load_event 变更段 await listener 永挂）；style_events / style_load_async 同
> 族文件级 Timeout。
> **勘根**：spec sheet 就绪即向元素 fire load，inline 文本同步就绪但更新走「queue a
> task」——task 异步使同 turn 后挂的 onload/addEventListener 可收；type eligible 门
> （s156/s160 精确匹配口径）与连接门（shadow-including connected）都在 fire 路径。
> **施工（2 文件净 +100，rust 零改动）**：part03 共享 helper `_zwStyleBlockQueueUpdate`
> ——identity key 双印记去重（首派 `_zwStyleLoadFired` 永久 / 变更 `_zwStyleUpdatePending`
> 同 tick=spec 单 task 更新）+ `setTimeout(0)` task 尾派发 + fire 点现读 type/连接门 +
> `_dispatchWithBubble` 元素站派发；解析静态臂 = `_zwStylesheetStaticScanUpto` 越界（尾扫）
> 形对快照 style 全量首派（零 script 页面首次调用即末尾）；part04 appendChild trap STYLE
> 父/子双臂（handle-only 经 `_zwHandleCreateTag` 盖章链无漏）+ textContent setter 写后臂。
> **验证**：node --check SYNTAX_OK。定向 style_load_event **Fail→Pass**、
> style-load-after-mutate **Timeout→Pass**。document-metadata 目录 join 对 s161 full4
> **F2P 1 / P2F 0** + 3 新现子测全 Pass + 2 文件级 T→P（181P/31F/12T→184P/30F/10T）。
> 全量 join 对 full4 (key,state) 对 **F2P 1（靶案）/P2F 0/P2T 0/T↔P 0**；LOST 77 全归因
> 本轮环境截断（forms 71 + interactive-elements 2 + scripting-1 1 + zz-s161-probe 探针已
> 删 1）；GAINED 6 = 3 靶族 Pass 首报 + 1 环境恢复 + 在册 flaky 域 1F+1T（customizable-select
> optional / details name-attribute）本轮报出。门禁：shim-only 轮惯例（s155/s156/s159 先例）
> rust 零改动 fmt/clippy 零波及；提交树态与 full1 run 编译树态同构——7934 键零 P2F 即端到端
> 门禁（共享树 make test 的 T58 fmt 前置在并发方 testharness.rs WIP 失败，非本刀文件不代修）。
> **范围**：shim part03.js + part04.js（索引级 hunk 分割提交，并发方 part03 WIP 留工作树零触碰）。
> **挂账更新**：FIXME 新记两项——type/title 属性变更触发面、非 textContent 子变更
> （insertBefore/append 入 style、CharacterData 写）未挂臂（语料无形态暂不扩面）。并发
> 注记：切片号 162 与并发会话 per-sheet styleSheets 枚举 WIP 撞号（其 part03 枚举口 +
> media 推迟取回 hunk + css_wire/testharness WIP 注释亦标 s162，未落账）——本刀先落账
> 保号，其收口时应顺延重编号 s163 并同步注释。
> **下一步 s163 候选**：①并发会话施工面收口核对（per-sheet styleSheets 枚举 + media
> 推迟取回——落账时重编号 s163，含 stylesheet-media-change 断言 2 + media-match-block-script
> 设计面）；②meta color-scheme 三重栈续拆（used-scheme 传播 + 系统色动态解析）；③style
> 元素 load 面 FIXME 收尾（type/title 属性变更臂——语料无形态，低优先）；④shadow DOM
> 级联面（link-rel-attribute shadow 案）。
> **账本**：[evidence/2026-10-30-m2s162-style-load-event.md](html-semantics-compat/evidence/2026-10-30-m2s162-style-load-event.md)
> **▶ 上轮状态（2026-10-30 M2 切片 161 施工收口，动态元素 gCS 通道 + 外链 media 门三层——断言级翻绿 + style-system UA display 命名空间门回归修复 + corpus 零实质位移）**：
> **入场定刀位**：s161 候选①拆两半施工——动态元素 gCS 读值通道（s158 定谳 handle-based proxy 无
> `__zwSelector`，gCS 恒 ''）+ 外链 link media 门（s160 FIXME「link media 门仍未建模」）。
> **勘根**：handle 链权威 applier 现成（`apply_dom_mutations_full` 带 persistent_nodes 回填），缺的只是
> 「parse+重放+求值」的代际槽化与 shim 回落臂；外链 media 门探针定谳必须三层同门——Rust 静态槽加载期一次
> 写入 media 无知，仅 shim overlay wrap 时它永远赢过翻转（探针 v4 green 违规应用铁证）。
> **施工（11 文件：rust 7 + shim 4）**：①handle 视图 gCS 通道——computed_style_cache 新 HANDLE_VIEW_CACHE
> 10-tuple 代际槽 + `computed_style_by_handle`（parse_html + apply_dom_mutations_full 重放 `[applied..count)`
> + compute_styles_for_doc + apply_intrinsic_size_overrides）+ callbacks 注册 `__zw_get_computed_style_by_handle`
> + part01 gCS `__zwHandle` 回落臂（catch 返 ''）；②外链 media 门三层——extract.rs `extract_stylesheet_links`
> （href+media）+ webview.rs resolve_external_css / async_load.rs begin_stylesheet_fetch 加载期门 + part03
> `_zwSsRecomputeExt` 逐表 `@media` wrap + part04/part05 三触发臂（handle-only TAG 容忍——`_realTag` 动态
> link 恒 DIV）。
> **验证**：check/fmt/guarded-clippy workspace 全绿 + shim 14-part node --check 111228 行。探针（静态 link
> media=none→all 翻转）门半+翻转半双 Pass 验后删。靶面 stylesheet-media-change-no-fetch **断言 1 翻 P**
> （动态 p gCS 读 red 通路实证；失败点移断言 2 = 记账面）。the-link-element 113P/2F 双 F 既有 +
> the-style-element bad-set ⊆ s159（s160 11F→P 保持）+ style_non_matching_media 保持。全量案级 join
> （full3+full4 对 s159 (key,state) 对 sort -u）F2P 恰 s160 的 11；P→F 唯一实质位移 SVG noscript **当场修复**：
> style-system `ua_default_display` 注入 `is_html_ns` 门（HTML 渲染规范 UA 表 real UA `@namespace xhtml`
> 限定，Firefox html.css bug 1470150 先例——此前 localname 盲匹配误隐藏异命名空间同名元素，动态 gCS 暴露
> 真实计算值翻红；修复后 non-html-noscript 复 P，静态渲染与 gCS 双通道同源生效），终账 full4
> **7093P/803F/91T**。dialog-popover-closedby-complex 5 子测位移 A/B 定谳环境漂移非回归（带/不带刀口
> style-system 改动 solo 同 16 bad、s159 自身 12 bad、当日 4→11→16→16 时序漂移、文件无 SVG）。
> **范围**：engine computed_style_cache/computed_style/callbacks/extract + webview webview/async_load +
> style-system lib + shim part01/03/04/05（rust 7 + shim 4）。
> **挂账更新**：FIXME 新记四项——①动态 link 对 `__zw_query_all` 不可见（R51c 查询视图不应用 handle 链，
> overlay 枚举缺口，事件通道有级联 blob 通道无；stylesheet-media-change 断言 2 当前根因）；②外链 css 槽位
> 级联序先于页面 style 表（同特异性后者胜，真实浏览器按文档序 link 在后应胜；翻转需全语料 A/B 独立切片）；
> ③跨批 handle 边界（computed_style_by_handle 仅重放 `[applied..count)`，FIXME 已注）；④layout collect_items
> 兜底 `ua_default_display` 无命名空间门（本刀只修 style-system 注入点，无失败证据暂不扩面）。承袭不变：
> meta color-scheme 24F 三重栈（动态 gCS 已通，剩 used-scheme 传播 + 系统色动态解析）、selection-pointer
> 持久环境漂移、shadow 级联面、style_load_event 1F、non-parser-created-doc 1F、link-multiple 全量贴边、
> meta long × corpus、details 族贴边、promise_test 串行启动、innerHTML 身份分裂、sel 印记失配、host \v 分词、
> overlay/base 语义差、生产 renderer 外链槽恢复臂、待决三项须用户拍板。
> **下一步 s162 候选**：①外链 per-sheet 结构化通道设计（动态 link 枚举 + 槽位级联序一并——stylesheet-media-change
> 断言 2 + media-match-block-script 1F 设计面）；②meta color-scheme 三重栈续拆（used-scheme 传播 + 系统色
> 动态解析）；③style_load_event 事件面（1F）；④shadow DOM 级联面（link-rel-attribute shadow 案）。
> **账本**：[evidence/2026-10-30-m2s161-handle-gcs-media-gates.md](html-semantics-compat/evidence/2026-10-30-m2s161-handle-gcs-media-gates.md)
> **▶ 上轮状态（2026-10-30 M2 切片 159 施工收口，合成条目 href 通道——stylesheet-not-removed-until-next-stylesheet-loads F→P + 域外环境漂移面 A/B 铁证记账）**：
> **入场定刀位**：s158 下一步候选①（stylesheet-not-removed 1F——R5114
> 合成条目 href 通道）。
> **勘根**：失败形态 "Cannot read properties of null (reading
> 'includes')" = R5114 合成条目 href=null。案面双时序断言（test 同步
> 段）：初始 fetch 在飞读 `styleSheets[0].href` 含 style.css；href 改
> neutral 新 fetch 排队后仍读 style.css（旧表在位）。spec sheet 对象
> 生命周期近似（#link-processing-model）：processing 创建即有 href 不
> 等 load；href 变更排队新 fetch 旧生效值不动；load 完成替换；error
> 替换未发生保持旧值。R5114 limitation 收窄：per-link href 有查询通
> 道可近似，per-link 文本拆分仍记账保留。
> **施工（2 文件净 +160，rust 零改动）**：part03 五件——
> `_zwSsSheetHrefMap` 生效 href 状态表（identity key → 生效 URL，键
> 口径同 `_zwExtSheetTexts`）；`globalThis._zwSsSheetHrefFor` 查询口
>（query_all('link') 文档序 + lw 读 + ASCII ws 分词 + href 非空 +
> handle 反查键，首命中返回全 miss null——合成 blob 单条目 href 映射
> 取级联首序）；fetch 发起创建写（fired 幂等门后挂解析 URL 不等
> load——断言 2 时点可读）；preload 复用臂创建写（转换面零网络生
> 效）；done 包装 load 覆写（error 不动最小状态机）；rel 移除臂删键。
> part06 五处——`_makeStyleSheet`/`_makeSheetBuild` 第 3 参
> externalHref 透传 + `href: externalHref || null`（元素表与
> constructable CSSStyleSheet 不动）+ R5114 注释更新 + styleSheets
> getter 合成条目每访问调查询口 live 传值。
> **验证**：node --check 110795 行（净 +160）+ rust 零改动。靶面定
> 向 **F→P**（双断言时序逐点过）。目录级 the-link-element
> **112P/2F/0T**——两记账 F 保持（shadow、media-change），link-multiple
> **6/6 保持**，0 Timeout。全量 join 对 s158 终账 F2P **1**（靶案）+
> **域外环境漂移面（A/B 铁证与刀无关，记账）**：selection-pointer 21
> P2F——A/B stash 基线同态 Fail、s157/s158 两轮终账同域 Pass（05:15/
> 07:01）、语料 mtime 10-15 未变、07:10 并发 css 流 checkout 活动时间
> 线吻合（三 runner 文件 mtime 更新内容与 HEAD 零差异）、空闲态复跑
> 仍 Fail = 持久漂移；select-event 91 + focus-dynamic 12 全量 Timeout
> 截断（定向 270P/80P 全恢复）；dialog 杂项 2+2；全部在 forms/selection
> + dialog 交互时序域与刀口零交集。机器冷却复跑钉死留下轮入场。
> **范围**：js_dom_shim part03.js/part06.js。
> **挂账更新**：收口——stylesheet-not-removed（销案）。FIXME 解除
> ——R5114 合成条目 href（per-link href 通道落地）。新记：error link
> sheetHref 不删回归面说明（error 后查询口返失败 link 解析 URL=虚构
> 生效 href，全量未暴露不收紧）；交互时序域环境漂移（下轮入场先钉
> 暂态 vs 持久）。承袭不变：候选②动态元素 computed style 大面（先定
> 刀口）、③shadow 级联面、④全域 .css fixture sweep、R5114 per-link
> 文本拆分、link-multiple 全量贴边（6/6 保持）、meta long × corpus、
> details 族贴边、promise_test 串行启动面、innerHTML 身份分裂、sel
> 重解析印记失配边角、host 侧 \v 分词、overlay/base 文本语义差两面、
> link media 门未建模、生产 renderer SetDomSnapshot 外链槽恢复臂、
> 待决三项须用户拍板。
> **下一步 s160 候选**：①入场先钉环境漂移面（selection-pointer 复
> 跑——回 Pass 暂态销案，仍 Fail 持久记账；顺带 select-event/
> focus-dynamic 定向核对）；②候选②动态元素 computed style 大面（先
> 定刀口）；③shadow DOM 级联面（link-rel-attribute shadow 案）；④全
> 域 .css fixture 缺口 sweep。
> **账本**：[evidence/2026-10-30-m2s159-link-sheet-href-channel.md](html-semantics-compat/evidence/2026-10-30-m2s159-link-sheet-href-channel.md)
>

> **▶ 上轮状态（2026-10-30 M2 切片 158 施工收口，link rel 状态 ↔ 外链 CSS 级联活链路 + rel ASCII whitespace 分词——link-rel-attribute light DOM + tokenization 2F→P + 全量 join 零位移）**：
> **入场定刀位**：s157 下一步候选②③（link-rel-attribute 2F + tokenization
> 1F——sheet 应用/removal 渲染面 + ASCII whitespace 变体）。
> **勘根（候选①④重定谳 + ②③根因链）**：①stylesheet-media-change-no-fetch
> 失败根因是**动态元素 computed style 根本缺位**——createElement 产物 handle
> proxy 无 `__zwSelector`，shim getComputedStyle 恒 ''（探针钉 sel=null/
> hostColor=''）——刀口另立降级记账。④stylesheet-not-removed 失败形态
> "Cannot read properties of null (reading 'includes')" = R5114 合成条目
> href=null——s159 候选。②③根因链四件叠加：语料 .css fixture 缺失
>（fetch_dir_html 2026-10-29 才加 .css 扩展 + 幂等快路径固化缺口——补拉
> link-rel-attribute.css/neutral.css）；rel 变更无级联反应（级联外链槽 load 期
> 一次装填，级联源走平面文本不走 `<link>`）；removeAttribute('rel') 无触发臂
>（s155 只建 set/IDL 臂）；`\v` 分词偏差（helper `\s+` 含 U+000B，spec ASCII
> whitespace 不含——tokenization 案单 token 误命中）。**机制发现**：gCS 代际
> 缓存键含 css 分量（computed_style_cache sheets_dirty/generation_cache_matches
>）——槽文本变化自动换代重收集，只缺 shim→槽触发臂。键一致性：静态元素
> `__zwSelector`（探针钉 elSel === queryAll === "link"）、动态 '@'+handle。
> **施工（3 文件净 +131/-7）**：①engine `__zw_set_external_css` 回调
>（callbacks.rs）——shim 下发合并文本装 ACTIVE_EXTERNAL_CSS 同槽（页面期反应
> 面；生产 renderer SetDomSnapshot 恢复臂 FIXME）。②part03 四件：
> `_zwExtSheetTexts` overlay 表 + `_zwLinkRelTokens`（spec ASCII ws
> `[\t\n\x0C\r ]+`）+ `_zwSsRecomputeExt`（query_all('link') 文档序 + lw 读 +
> handle 反查键 + 覆盖门全有才替换 + `';\n'` 拼接）；rel 门三分支（stylesheet
> token → 既有路径；无 token 且身份在册 → 清两表 + recompute；否则静默）；
> `_ssVisit` 第 5 参收集数组（各层 text() push，内层透传）；**静态扫描臂
> noOverlay 第 6 参（设计修订）**——存表但不下发。③part04 removeItem rel 移除
> 臂（镜像 s155 set 臂）。
> **归因轮（两层）**：①106 族 6 案 P2F（@import 内层规则丢级联 got static）
> ——A/B stash + noimport 对照隔离 + 探针链定谳：**无分号 @import 残留吞规则**
>（webview resolve s89 教训 shim 层双现——join('\n') 使 @import preamble 过
> 换行吞内层首规则）。探针方法论修正：host 回调只读绑定不可 hook（sloppy 赋值
> 静默丢弃，dump 证据作废）；shim 自设 global 可覆盖（包装探针钉 rcCalls=2
> 证明 done 链完整、下发发生）。修法双层 `';'` 分隔（recompute 拼接 + 存表
> join；级联 matcher 对 Import 规则本就跳过）。②details name-attribute 文件级
> Timeout（基线三轮无）——页面 6 link 静态 done 走 overlay 下发、文本与 base
> 逐字节不等 → 每页一次多余全文档级联换代 → 贴边长页超时；单文件 18/18 Pass
> 佐证环境敏感。修法 noOverlay 门（扫描臂存表解锁后续变更面覆盖门但不下发，
> 税=0；overlay 职责收窄为页面期变更面）。语义修订说明：overlay 与 base 以
> 覆盖门衔接，静态面 base 通道负责。
> **验证**：node --check 110635 行（s157 110462，净 +173）+ fmt PASS +
> guarded-clippy workspace 全绿。靶面定向：link-rel-attribute light DOM
> **Fail→Pass**、tokenization **Fail→Pass**（shadow 案预期仍 F 记账——shadow
> 级联面）。106 族回归修复 10/10 全 Pass。目录级 link-multiple **6/6 保持**
>（贴边案在基线行为域内）。全量案级 join（对 s157 终账 /tmp/zw-s157-full3.log
>，(key,state) 对集直接去重）**8082 对 = 8082 对**（7205P/793F/87T 对 7199P/
> 796F/87T）——F2P 2（两靶案）+ P2F 0 + LOST 0 + GAINED 0 **全域零位移**
>（name-attribute 18/18 恢复、details Timeout 消失即 noOverlay 归零税实证）。
> 探针语料验后已删（15 块零残留）。
> **范围**：engine js_dom_bridge/callbacks.rs + js_dom_shim part03.js/part04.js
>；wpt-data 本地补拉 2 个 .css fixture（gitignore 不随提交）。
> **挂账更新**：收口——link-rel-attribute light DOM + tokenization（销案）。
> FIXME 新记：overlay 与 base 文本语义差两面（@import 内层追加非原位展开、
> url() 未绝对化——语料无该形态）；link media 属性门未建模；生产 renderer
> SetDomSnapshot 恢复臂不覆盖外链槽。承袭不变：候选①动态元素 computed style
> 大面（__zwSelector 只属静态元素——另立）；候选④ R5114 合成条目 href（s159
> 候选）；link-multiple 全量贴边（本轮 6/6 保持）；meta long × corpus 失效；
> details 族贴边（name-attribute 面本轮消除）；promise_test 串行启动面；
> innerHTML 身份分裂表示面；sel 重解析同槽同 href 印记失配边角；host 侧
> extract_stylesheet_hrefs Unicode \v 分词偏差；待决三项须用户拍板。
> **下一步 s159 候选**：①stylesheet-not-removed-until-next-stylesheet-loads
> 1F（R5114 合成条目 href 通道）；②候选①动态元素 computed style 大面（先定
> 刀口）；③shadow DOM 级联面（link-rel-attribute shadow 案）；④全域 .css
> fixture 缺口 sweep（lib.sh 2026-10-29 前目录族）。
> **账本**：[evidence/2026-10-30-m2s158-link-rel-ext-css-cascade.md](html-semantics-compat/evidence/2026-10-30-m2s158-link-rel-ext-css-cascade.md)
>
> **▶ 上轮状态（2026-10-30 M2 切片 157 施工收口，尾随静态 link 触发收口——link-load-event.html F→P + 全量 join 零位移）**：
> **入场定刀位**：s156 下一步候选②（link-load-event document load 阻塞面
> 1F——saw_link_onload 恒 false）。
> **勘根（根因升级定谳）**：①根因非「stylesheet 未 delay load」而是**尾随
> 静态 link 零触发**——案面 3 scripts + 尾随 link(rel=stylesheet onload) +
> window load 断言；s152 静态扫描按 script ordinal 触发「script 之前的
> link」，最后一个 script 之后的尾随 link 从未被任何臂触发 → link.onload
> 恒不 fire。②spec 时序不变式：script-blocking stylesheet 只阻塞其后
> pending script，尾随 stylesheet 不阻塞任何已执行 script、只 delay document
> load event。③trickle 滞留假设被证据否定（runner 对 stylesheet 忽略
> pipe=trickle MIME 面 testharness.rs:5251 + 扫描窗折叠 __zwScanCollapseTrickle）。
> **施工（3 文件净 +61，两件套 + 归因轮门控修订）**：①webview.rs 脚本流
> 末尾补扫 _zwStylesheetStaticScanUpto(2147483647)——序号越界 shim 侧收敛
> 为全部 link，游标幂等已触发段零成本。②part03 _ssVisit 在飞计数
> _zwSsLoadPending——每层恰一次 settle（同步门/then 链全路径）包装归零，
> @import 内层递归逐层自计，fetch 同步抛按取回拒绝归零派 error（旧路径
> 外层 catch 静默无事件——计数平衡优先），preload 不计数（resource hint
> 不 delay load）。③testharness.rs loadStep5275 第四谓词
> __zwStylesheetFetchPending 轮询在飞取回。④归因轮门控修订：谓词按尾扫
> 触发印记 _zwSsTailFired 门控——仅越界补扫真实推进游标的页面置位，常规
> 交错扫描不置位，无尾随 link 页面谓词恒 false 与基线全同。
> **归因轮（全量 link-multiple Timeout → A/B 定谳）**：①复现窗定位——
> 定向 6/6 Pass、目录级不复现、document-metadata 级复现（Timeout 键
> pending:2/tests:6 unreported）。②语料探针定谳（wpt-data gitignore 本地
> 数据 stash 不收，探针还原须手动）：案 1-4 幂等属性写各烧 2s 竞速窗 =
> 名义 8s，探针 A 实测 9757ms（溢出 ~440ms/案），案 5/6 仅 38ms。
> ③基线 A/B（stash 源码、同探针语料）：基线 9334ms——基线自身溢出
> ~1.33s、距预算仅 0.67s 余量；本刀边际 +423ms（n=1，同源噪声带 ≥200ms）
> ——贴边固有，非尾随面按页边际须归零。④门控修订后全量 link-multiple
> **6/6 全 Pass**。
> **验证**：node --check 110462 行（s156 110429，净 +61）+ cargo fmt PASS +
> guarded-clippy workspace 全绿（顺带修 78754deb3 引入面——quickjs_dom_bindings
> s151 compatmode 孪生无 cfg、唯一调用臂 cfg 在 all(quickjs, not(v8))，默认
> v8 构建 dead-code 断 clippy；修法 cfg 镜像调用臂，与本刀无机制关联）。
> 靶面定向 link-load-event **Fail→Pass**（2/2 Pass）、MAKE_EXIT=0。全量案级
> join（对 s156，(key,state) 对集直接去重）**8082 对 = 8082 对**（7199P/
> 796F/87T 保持）——F2P 1（靶案）+ P2F 0 + LOST 1（即靶案旧 Fail 键）+
> GAINED 0 全域零位移（the-style-element 族、LinkStyle 面、s150-156 修复面
> 全保持）。探针语料验后已删零残留。
> **范围**：engine js_dom_shim part03.js + webview.rs + wpt-runner
> testharness.rs（bootstrap 谓词字符串）+ quickjs_dom_bindings.rs（clippy
> 门禁顺带修复）；wpt-data 零改动。
> **挂账更新**：收口——link-load-event document load 阻塞面（销案）。FIXME
> 新记：link-multiple 全量贴边——案 1-4 竞速窗 ×4 名义 8s 固有溢出 ~1.3s
> （基线标度实测 9.3s/~10s 预算）余量 0.7s 噪声敏感，s155/s156 全量通过属
> 侥幸；上游语料无 meta timeout=long + 语料 gitignore 修改不持久，runner
> 侧根治面（meta long 通道放宽/计时器溢出收敛）另立。承袭不变：link-rel-
> attribute 2F（sheet 应用/removal 渲染面——跨域边界待归因）；link-rel-
> attribute-tokenization 1F；stylesheet-media-change-no-fetch 1F；stylesheet-
> not-removed-until-next-stylesheet-loads 1F；meta long × corpus 通道放宽
> 失效；details 族 lib flake；promise_test 串行启动面；innerHTML 身份分裂
> 表示面；sel 重解析同槽同 href 印记失配边角；待决三项须用户拍板。
> **下一步 s158 候选**：①stylesheet-media-change-no-fetch 1F（media 变更
> 不重取门——helper 同型扩展面）；②link-rel-attribute 2F 归因轮
>（getComputedStyle 绿色断言挂样式应用管线——渲染流域边界候选，先归因再
> 定刀法）；③link-rel-attribute-tokenization 1F；④stylesheet-not-removed-
> until-next-stylesheet-loads 1F。
> **账本**：[evidence/2026-10-30-m2s157-trailing-static-link-load-hold.md](html-semantics-compat/evidence/2026-10-30-m2s157-trailing-static-link-load-hold.md)
>
> **▶ 上轮状态（2026-10-30 M2 切片 156 施工收口，LinkStyle sheet 创建门——styling/LinkStyle.html 5/5 全绿 + 全量 join 零位移）**：
> **入场定刀位**：s155 下一步候选①（LinkStyle sheet 属性面 1F——
> styling/LinkStyle.html null 侧案 expected null but got object）。
> **勘根（探针一轮 + getter 定位）**：①失败消息精确——style2.sheet（`rel=
> "alternate stylesheet" title=""`）expected null but got object；同文件其余
> 4 案 Pass（含 must-return-StyleSheet 案——style5/6/7 sheet 对象面已达）。
> ②getter 定位：R3036 sheet 臂（part04 set trap）LINK 仅 rel 正则门——spec
> CSSOM 创建条件未建模（type essence 门 + alternate 空 title 门缺）。③探针
> 一轮钉态（zz-s156-probe 验后删）：p2/p3/p4（alternate 空 title/无 title/
> style type=text/html）应 null 现 OBJ；disabled 全 boolean false（通用反射
> 达标，无需 disabled 面）。④位移面收敛：语料 .sheet 读取点全查——the-style-
> element 族无 type gate 跳过、type 变体在 reftest 渲染面不读 .sheet。
> **施工（part04 sheet getter 净 +28）**：①type essence 门（STYLE/LINK 共用
> ——type 属性存在且 essence ≠ text/css 返 null；TeXt/CsS 小写归一创建、
> text/cſs SHARP S 不归一不创建、缺省不设门）。②LINK alternate 空 title 门
>（rel 分词含 alternate 且 title 空串含缺失返 null；style6 非空 title 照常
> 创建）。两门前置于既有 LINK rel 门。
> **验证**：靶面 5/5 全 Pass、MAKE_EXIT=0。全量案级 join（对 s155，(key,
> state) 对集直接去重）**8082 对 = 8082 对**（7198P/797F/87T → 7199P/796F/
> 87T）——F2P 1（靶案）+ P2F 0 + LOST 0 + GAINED 0 全域零位移（the-style-
> element 族、CSS-in-JS 面、s150-155 修复面全保持）。node --check 110429 行
>（净 +28）；Rust 面零改动；探针验后已删。
> **范围**：engine js_dom_shim part04.js；wpt-data 零改动。
> **挂账更新**：收口——LinkStyle sheet 属性面（销案）。FIXME 在册：sheet
> 创建未建模 fetch 完成/加载中状态（getter 纯 attr 门，语料以 window load
> 后断言豁免）。承袭不变：link-rel-attribute 2F（sheet 应用/removal 渲染面
> ——跨域边界待归因）；link-rel-attribute-tokenization 1F；link-load-event
> document load 阻塞面 1F；stylesheet-media-change-no-fetch 1F；stylesheet-
> not-removed-until-next-stylesheet-loads 1F；meta long × corpus 通道放宽
> 失效；details 族 lib flake；promise_test 串行启动面；innerHTML 身份分裂
> 表示面；sel 重解析同槽同 href 印记失配边角；待决三项须用户拍板。
> **下一步 s157 候选**：①stylesheet-media-change-no-fetch 1F（media 变更
> 不重取门——helper 同型扩展面）；②link-load-event document load 阻塞面
> 1F；③link-rel-attribute 2F 归因轮（getComputedStyle 绿色断言挂样式应用
> 管线——渲染流域边界候选，先归因再定刀法；shadow DOM 案 getComputedStyle
> 返 '' 另根）。
> **账本**：[evidence/2026-10-30-m2s156-linkstyle-sheet-creation-gates.md](html-semantics-compat/evidence/2026-10-30-m2s156-linkstyle-sheet-creation-gates.md)
>
> **▶ 上轮状态（2026-10-30 M2 切片 155 施工收口，rel=preload fetch + 转换复用——link-multiple 文件级 Timeout 键消 6/6 全 Pass + 全量 join 零位移）**：
> **入场定刀位**：s154 下一步候选①（rel=preload fetch 面——link-multiple-
> load-events.html 基线案 1-4 Pass + 文件级 Timeout 键，案 5/6 未打印）。
> **勘根（Timeout 形态先解再定刀）**：①定向诊断行「testharness page timeout
> completed the file with 2 test(s) unreported (pending:2,phase:4,tests:6)」
> 定谳——语料无 meta timeout → testharness 全局默认超时收文件（s149 早收形
> 同路径）；案 1-4 幂等语义的正确形态是第二个 race 等**满 2s**（load 不再
> fire = 正确）→ 4×2s=8s 吃掉全局预算，案 5/6 到点前强收——s153「案 6 页级
> 10s 时序必然」注记机制来源。修复必须让案 5/6 毫秒级完成（8s+ε<10s 全报）。
> ②案 5 两段语义：rel=preload as=style 设 href → 网络取回 + onload（端点
> count "1"）；rel 改 stylesheet → 转换后按 stylesheet 处理模型从 **preload
> cache** 采用（零网络）但 **load 事件再 fire**（上游明示注释）。③触发面
> 缺口：href 变更有 part04/part05 两臂（s150），**rel 变更无任何臂**；helper
> rel 门只认 stylesheet。④`as` IDL 反射已有（part05 reflected 列表）——
> `link.as='style'` 落属性可读。⑤位移面收敛：html/semantics rel=preload
> 唯一 = 靶面案 5（modulepreload 是 s124 已收面）。
> **施工（3 文件净 +60/−3）**：①part03 rel 门双旗标分流（stylesheet/preload
> 不互斥扫描）+ preload 取回臂（as 非空门——missing/invalid as 上游不发起
> FIXME 分型不建模；同身份印记去重复用 `_zwSsFiredMap`；页面 fetch 无 s153
> 通道旗标——preload 是 cors 模式，同源语料过 check；404 fetch resolve →
> load，error 仅网络失败；完成播种 `_zwPreloadCache[key]=abs` + 派 load /
> 拒绝派 error）+ sheet 分支印记门前**复用臂**（cache 命中 → setTimeout(0)
> 派 load 零网络返）。②part04 setAttribute rel 臂 + ③part05 IDL rel 臂——
> TAG 门甄别 helper(sel,handle,null)，幂等 rel 写由 sheet 分支印记静默拦截。
> **复用臂位置取舍**：置于印记命中点后自伤（s154 印记命中本是静默返——append
> 直取后重查再入，改派 load 即双 load，case 1-4「second load unexpectedly
> fired」全翻红）；先于印记门且只认 `_zwPreloadCache`（sheet 取回不播种）——
> case 1 重查 cache miss → 印记静默返不变；案 5 转换 cache 命中 → 派 load。
> **验证**：靶面 6/6 全 Pass、MAKE_EXIT=0、文件级 Timeout 键消。定向
> the-link-element join（对 s154 基线）LOST 1（Timeout 键）+ GAINED 2（案
> 5/6）零位移。全量案级 join（(key,state) 对集直接去重）s154 8084 行/8081 对
>（7196P/797F/88T）→ s155 8085 行/8082 对（7198P/797F/87T）——F2P 0/P2F 0/
> other 位移 0 全域零位移（fetch API/XHR 面、s152/s153/s154 修复面全保持）。
> 口径注记：s154 账本「8080 对/7195P」系上轮 join 脚本差一（直计 8081 对/
> 7196P，同态重复行计入差异），本轮起统一直接去重计数。node --check 110401
> 行（净 +76）；Rust 面零改动；探针零语料（勘根全走 runner 诊断行 + 日志直读）。
> **范围**：engine js_dom_shim part03.js/part04.js/part05.js；wpt-data 零改动。
> **挂账更新**：收口——rel=preload fetch 面 + rel→stylesheet 转换复用面
>（Timeout 键销案，「案 5/6 页级 10s 时序必然」注记销案）。FIXME 新记：preload
> 资源级共享缓存/as destination 分型/rel 双类型分流不建模（语料无形态）。
> 承袭不变：LinkStyle sheet 属性面 1F；stylesheet-media-change-no-fetch 1F；
> meta long × corpus 通道放宽失效；details 族 lib flake；promise_test 串行
> 启动面；innerHTML 身份分裂表示面；sel 重解析同槽同 href 印记失配边角；
> 待决三项须用户拍板。
> **下一步 s156 候选**：①LinkStyle sheet 属性面 1F；②link-load-event
> document load 阻塞面（Check if the stylesheet's load event blocks the
> document load event 1F）；③stylesheet-media-change-no-fetch 1F（media 变更
> 门——helper rel 门同型扩展面）。
> **账本**：[evidence/2026-10-30-m2s155-preload-fetch-rel-conversion.md](html-semantics-compat/evidence/2026-10-30-m2s155-preload-fetch-rel-conversion.md)
>
> **▶ 上轮状态（2026-10-30 M2 切片 154 施工收口，adopt/innerHTML 身份双表示——document-without-browsing-context 2F→P + 全量 join 全域零位移）**：
> **入场定刀位**：s153 下一步候选①（adopt/innerHTML 身份双表示深水面——
> document-without-browsing-context 两案 expected "1" got "0"，stylesheet
> 取回从未发起）。
> **勘根（探针五轮 + shim breadcrumbs 两轮收敛）**：runner stylesheet.py 端点
> per-id take-and-clear 计数——got "0" = 资源请求从未到达端点。①探针 wrap 法
> （包 globalThis._zwStylesheetMaybeStart/fetch）实证：createElement 控制组
> 正常；adopt 形 4 参臂已带章（h=__n1）但不 fetch；innerHTML 形两臂触发均不
> fetch（attr 臂 3 参 h=__n2 + 汇流臂 4 参 h=null plain 形）。②breadcrumbs
> 定案案 1（adopt）：R368 盖章路径 R4004 属性回填依赖 `child.getAttributeNames`
> ——DOMParser 解析树 M 节点无该方法（探针 THROW 实证）→ 宿主属性表恒空 →
> handle 读 ''（非 null）→ `_ssReadAttr` `== null` 回退不触发 → rel='' 分词
> miss → bail。③案 2（innerHTML）= 身份双表示时序缺口：innerHTML 物化身份
> 分裂——汇流臂记账时 addedNodes 还是解析树 plain M 节点（永不盖章）→ TAG 门
> `_realTag(null,null)` 恒 DIV bail；物化产物是另一 JS 对象（盖章 __n2 走
> attr 臂）但 `_zwNodeParent` 反链注册滞后于属性写 → 连接门 bail；两臂当时
> 态都判 not connected。④修复迭代教训：重查首版从 elObj 现读身份——身份分裂
> 使 innerHTML 案 elObj 恒无章 → 重查自弃（dbg 无 enter 实证）→ 改**身份键**
> （`_elKey(sel,handle)`）设计：重查按触发臂给定身份原样重跑，印记同键表驱动
> （对 3 参/4 参臂一致）。
> **施工（part03 helper 净 +59/−4）**：①`_ssReadAttr` 回退判据 `== null` →
> falsy（宿主 absent '' 与解析树 null 同判 miss；rel/href 非空串语义空值本
> 就 miss）。②身份键取回印记表 `_zwSsFiredMap`——同 (sel,handle) 同解析 URL
> 只发起一次（发起前置位；先赋 href detached bail → 重查排程 + 插入臂直取的
> 既有防双请求形保持；href 变更解析 URL 不同失配照常重取；无身份键照旧直
> 取）。③单发身份键重查 `_ssScheduleRetry`——TAG/连接门 bail 且身份键存在时
> setTimeout(0) 按原身份重跑一次（身份缺失回落 elObj 现读——R368 盖章形
> append 臂 elObj 后到章）+ `_zwSsRetryDone` 单发印记防同身份多臂重复排程与
> detached 恒态循环。
> **过程事故**：debug breadcrumbs 残留行引用已删 `var _ssDbg` → ReferenceError
> → helper 外层 catch 静默吞 → 修复版首跑假阴（定向仍 2F），二轮复跑定位。
> 教训：门链插桩移除与声明同批全清 + grep 收尾核验（本轮 grep `_ssDbg|__zwSsDbg`
> 零残留）。make 后台跑 cwd 不保持——`sh -c 'cd <repo-root> && make ...'`
> 显式锚定（两度「没有规则可制作目标」）。
> **验证**：定向 the-link-element 全目录对 s153 全量基线 join——靶面 2 案 F→P、
> LOST 0、其他位移 0（113=113 精确配平）。全量案级 join（对 s153
> /tmp/zw-s153-full.log，(key,state) 对 defaultdict(set) 对比）8080 对 =
> 8080 对（7193P/799F/88T → 7195P/797F/88T）——F2P 2（靶面）+ P2F 0 +
> other-diff 0 + LOST 0 + GAINED 0 全域零位移；重查/印记对 fetch API/XHR 面、
> link-multiple（先赋后插形）、s152 静态扫描面零波及实证。node --check
> 110325 行（净 +55）；Rust 面零改动（clippy/fmt 零波及）。探针语料验后
> 已删（工作树零残留）。
> **范围**：engine js_dom_shim part03.js；wpt-data 零改动。
> **挂账更新**：收口——adopt/innerHTML 身份双表示触发通道面（2F 全销）。
> FIXME 新记：sel 重解析同槽同 href 印记失配边角（`_zwSsFiredMap` 身份键跨
> 代际沿用——语料无该形态不建模）；innerHTML 身份分裂表示面本身（查询/树视
> 图统一）仍是深水面，触发通道收口 ≠ 表示面收口。承袭不变：LinkStyle sheet
> 属性面 1F；rel=preload fetch 面；stylesheet-media-change-no-fetch 1F；
> meta long × corpus 通道放宽失效；details 族 lib flake；promise_test 串行
> 启动面；待决三项须用户拍板。
> **下一步 s155 候选**：①rel=preload fetch 面（link-multiple 案 5 + 缓存
> 复用——s124 modulepreload 臂同型扩展）；②LinkStyle sheet 属性面 1F；③
> link-load-event document load 阻塞面（Check if the stylesheet's load event
> blocks the document load event 1F）。
> **账本**：[evidence/2026-10-30-m2s154-adopt-innerhtml-stylesheet-trigger.md](html-semantics-compat/evidence/2026-10-30-m2s154-adopt-innerhtml-stylesheet-trigger.md)
>
> **▶ 上轮状态（2026-10-30 M2 切片 153 施工收口，跨源 stylesheet 通道 CORS 豁免——link-load-error-events 7F→P + 全量 join 零回归）**：
> **入场定刀位**：s152 下一步候选①（跨源静态资源 ACAO 面影响面盘点——
> s151 新记 2+1 案修复面）。
> **影响面盘点（刀法取舍）**：修复面 = link-load-error-events.html https 3 案
> （`httpsSheet()` = `https://wpt.test:8444` 跨端口 vs 页锚 `https://wpt.test`；
> .https.html 变体不在 corpus 清单——执行清单 17 文件零行实证）。刀法 A'
> （静态臂普发 ACAO/ACEH）否决：上游 wptserve 静态 .css 确实无 ACAO，普发
> = 行为篡改 + fetch 负语义面风险（css-module without-cors 负案在 resources/
> 子目录不在度量面——后验修正）。刀法 C（shim stylesheet 通道豁免）采纳：
> 上游 stylesheet 取回是 no-cors 请求不做 CORS check（HTML fetching
> stylesheets），shim fetch(cors) 仿真引入上游不存在的 check——豁免 = 语义
> 归位，波及面收敛 helper 取回链。
> **勘根（三轮定向转折）**：①旗标信任边界——IIFE 模块局部 var 页面不可见
> 不可伪造；`_zwFetchMainImpl` 同步段快照进闭包，trickle/delay setTimeout
> 异步臂同样生效；豁免 = check + cors 头过滤双跳（全头保留——judge 须读
> nosniff）。②一轮定向揭内层 mixed 缺口（P→F 形）——data: 外层的 @import
> http: 内层此前碰巧由 fetch CORS check 顶住（静态臂无 ACAO → reject →
> error ✓ 双错凑巧 Pass），豁免后 load 误 fire 2 案 → 逐层 mixed 门补齐
> （s151 假阳消除同型教训重现）。③二轮定向揭 `_zwStylesheetImports` 正则
> 截断单根——url() 臂捕获类 `[^'")]` 排除 `)` 把 `pipe=status(404)` 截断
> （URL 丢 `)` → `wpt_pipe_status` strip_suffix 失败 → 404 退化 200）→
> nonexistent Import ×2 + slow/fast ×2 四案「load fired when error expected」
> 同根（外层直 href 案不经提取、link-style-error 案 URL 无括号——与基线
> Pass/Fail 分布完全吻合）→ 带引号臂回引边界修复。④三轮定向揭 @import
> 一层递归边界——双层案（import of import of nonexistent/non-CSS/http）
> 第二层不可见 3 案误 load；s151 账本「语料无多层形态」勘根有误（案 5/6/
> 9/17 即双层）→ 取回段递归化。案 9 基线 Pass 机制考古未完全（正则截断 +
> 一层边界的某凑巧组合），现状期望 error 明确、递归修复语义正确——归档。
> **施工（shim 2 文件净 +42/−13）**：①part01 `_zwStylesheetChannelDepth`
> 旗标 + `_zwFetchApplyFilteredResponse` 第 6 参（check 臂 + cors 过滤臂双
> 门）+ Impl 同步段快照 + finishFetch/finishHop/settleFetch 三调用点透传。
> ②part03 `_zwStylesheetMaybeStart` 取回段 `_ssVisit` 递归化（逐层 mixed 门
> 同步回调标 failed 先于微任务 load 子归零——聚合 error 优先；fetch 调用窗
> 旗标；深度 3 上限兜底环 FIXME）+ `_zwStylesheetImports` 正则带引号臂回引
> 边界。
> **验证**：定向 the-link-element 全目录对 s152 基线 join——**7 案 F→P、
> LOST 0、零 P→F**（https Load/Import×2 + nonexistent Import×2 + slow/fast
> ×2；link-load-error-events 全文件 17/17 基线 11P/6F；GAINED 1 =
> link-multiple media 在册少报键复现非位移）；s152/s151 修复面全保持。
> 全量案级 join（对 s152，(key,state) 对 sort -u 去重）8080 对 vs 8081 对
>（7186P/806F/88T → 7194P/799F/88T）——真修复 7 案 + P→F 0 + Timeout 位移
> 0 + GAINED 1；全语料 fetch API/XHR 面零位移 = 页面 CORS check 语义保持
> 实证。门禁 node --check 110270 行；Rust 面零改动（clippy/fmt 零波及）。
> **范围**：engine js_dom_shim part01.js/part03.js；wpt-data 零改动。
> **挂账更新**：收口——跨源静态资源 ACAO 面（s151 新记全销）+ @import
> 内层链残余（正则截断 + 一层递归边界，「多层递归链」挂账销账）。FIXME
> 在册：crossorigin 形态未分型、@import 环检测。承袭不变：adopt/innerHTML
> 身份双表示深水面 2F；LinkStyle sheet 属性面 1F；rel=preload fetch 面；
> stylesheet-media-change-no-fetch 1F；meta long × corpus 通道放宽失效；
> details 族 lib flake；promise_test 串行启动面；待决三项须用户拍板。
> **下一步 s154 候选**：①adopt/innerHTML 身份双表示深水面
>（document-without-browsing-context 2F）；②rel=preload fetch 面
>（link-multiple 案 5 + 缓存复用）；③LinkStyle sheet 属性面 1F；④
> link-load-event document load 阻塞面 1F。
> **账本**：[evidence/2026-10-30-m2s153-crossorigin-stylesheet-cors-exemption.md](html-semantics-compat/evidence/2026-10-30-m2s153-crossorigin-stylesheet-cors-exemption.md)
>
> **▶ 上轮状态（2026-10-30 M2 切片 152 施工收口，静态 link rel=stylesheet 文档序交错触发——fired-before-scripting 2F→2P + 全量 join 零回归）——其下一步 s153 已由 s153 收口（候选①跨源静态资源 ACAO 面：shim stylesheet 通道 CORS 豁免语义归位——内层 mixed 缺口 + @import 正则截断 + 一层递归边界三根连带收口，7F→P 零回归，见上**：
> **入场定刀位**：s151 下一步候选①（Face C render-blocking/script-blocking
> 时序面——link-{error,load}-fired-before-scripting-unblocked 2F；
> conditionally-block 同面并入勘根，结论不可修维持挂账）。
> **勘根 6 条**：①语料形态——前置 script 声明 `var t` → link（onerror/onload
> **inline content attribute**，区别于 s151 IDL 属性形；派发面
> `_ensureInlineHandler` 在位）→ 后随 script `t.step(assert_true(saw))`。
> ②spec 语义——script-blocking stylesheet 只阻塞**文档序位于其后的** pending
> parsing-blocking script（prepare-a-script + the-link-element）；宿主「全解析
> 后统一执行脚本」管线缺静态 link 触发点（s150 helper 仅动态臂）。③v1 刀法
> 失败定谳——脚本阶段起点一次触发全部 link 把 link **之前**的 script 也推迟，
> `var t` 未声明时 onerror 内 `t.step` 撞 ReferenceError → saw 恒 false（探针
> 三截面：事件链通 p1/p2，真根因在推迟不在事件）。④v2 序号对齐——runner
> `extract_page_scripts_indexed` script 序号 = 全部 script 元素文档序（过滤前
> 递增，与 shim `getElementsByTagName('script')`→`_zwDocAllElements` 在册对齐，
> currentScript 面 part06:401 同口径；template 子树两侧均不枚举）。⑤trickle
> 仿真单根（load 案拦路虎，探针链五轮二分）——v2 就位后 error 案即 Pass、
> load 案仍 Fail：probe-log 包装 fetch 记 `f0|f1:function` 无 f2（promise 永不
> 结算）；probe-wire 直调 `__zw_fetch` 三案同步 wire（排除宿主 handler）；
> probe-iso 直接 fetch+text 全 Pass（排除 query/文件/MIME）→ 唯一变量 = 扫描
> 窗内取回体；根 = shim fetch wrapper wptserve trickle 仿真
> `[?&]pipe=trickle\((?:[a-z]+)?(\d+)\)` → `setTimeout(_issueFetch, N×1000)`，
> `trickle(d3)` 把 load 推迟 3s 到断言之后（上游 parser 阻塞 3s 等待语义在
> runner 无 parser 阻塞形态下不可承载；probe-iso 过测系 promise_test 等 3s
> 定时器非同窗结算）。⑥conditionally-block 案级定谳不可修——案 1 `?delay=2`
> 延迟投递同步契约不可仿真（即时仿真 brown sheet 立即注册 assert_false 恒
> Fail）；案 2 per-sheet `styleExists` 踩 document.styleSheets 单条合成 blob
> （R5114）不可满足。
> **施工 3 件净 +103/−1**：①shim part03 `_zwStylesheetStaticScanUpto(n)`——
> 首次调用单次建交错表（`_zwDocAllElements()` 文档序 SCRIPT/LINK 分型，LINK
> 不预过 rel——MaybeStart 门内 rel 分词自甄别）+ `scriptPrefix[]` 预计算 +
> `_zwScanFiredCount` 前缀游标单调推进（O(1)/次）；触发窗置位
> `__zwScanCollapseTrickle` 同步段结束即清（.then 链内 @import 内层与页面
> fetch 零波及）；FIXME 快照语义注记（动态插入序移未追踪，幂等门兜底）。
> ②shim part01 fetch wrapper trickle 臂加 `__zwScanCollapseTrickle !== true`
> 门——扫描窗内取回体折叠延迟即时发出，fetch API timeout 竞速面零变化。
> ③webview.rs `run_page_scripts_impl`——v1 全前置块删除；脚本循环内每脚本体
> 求值前独立 execute `scanUpto(script_index)`（独立求值保证微任务清空先于
> 脚本体，`__zw_begin_script` 与体同求值不可用；CSP/混合内容拦截 script 序号
> 仍占位，游标随下一执行脚本补齐无漏触发；E0499 教训：用循环体既有 sandbox
> 重借不另开借用）。
> **验证**：探针链 8 文件（验后已删；踩坑在册——探针名含 `-manual` 尾缀撞
> runner `-manual.html` skip 规则零输出 exit 1，改名后正常回报）；定向
> fired-before-scripting **2F→2P**；全量案级 join（对 s151，(key,state) 对
> sort -u 去重）7186P/806F/88T vs 7185P/808F/88T——真修复 2 案 + F 新增 0 +
> P→F 0 + P 消失 1（link-multiple media 案定向复验稳定 P，s151 在册负载 flake
> 同形非本刀位移）；门禁 clippy -D warnings 零告警 + fmt 过 + node --check
> 110217 行 + zero-webview lib 809/9F 与 zero-engine lib 3600/9F stash 基线
> 逐字一致（9F 既有环境依赖）。
> **范围**：engine js_dom_shim part03.js/part01.js + webview webview.rs；
> wpt-data 零改动（探针 8 文件验后已删）。
> **挂账更新**：Face C 时序面收窄——fired-before-scripting 2 案收口；
> conditionally-block 整文件维持挂账（delay=2 不可仿真 + styleSheets 分条面，
> 修法前置 styleSheets 按 link 分条建模）。新记方法论：全量 join 的 P 消失键
> 先定向复验再归因（s151/s152 连续两轮同形）。承袭不变：跨源静态资源 ACAO 面
> （s151 新记 2+1 案）；adopt/innerHTML 身份双表示深水面 2F；LinkStyle sheet
> 属性面 1F；rel=preload fetch 面；stylesheet-media-change-no-fetch 1F；
> @import 多层递归链；meta long × corpus 通道放宽失效；details 族 lib flake；
> promise_test 串行启动面；待决三项须用户拍板。
> **下一步 s153 候选**：①跨源静态资源 ACAO 面影响面盘点（通用静态臂加 ACAO
> 全语料波及评估——2+1 案修复面，收益最大）；②adopt/innerHTML 身份双表示
> 深水面（document-without-browsing-context 2F）；③rel=preload fetch 面
> （link-multiple 案 5 + 缓存复用）；④LinkStyle sheet 属性面 1F。
> **账本**：[evidence/2026-10-30-m2s152-static-stylesheet-docorder-interleave.md](html-semantics-compat/evidence/2026-10-30-m2s152-static-stylesheet-docorder-interleave.md)
>
> **▶ 上轮状态（2026-10-29 M2 切片 151 施工收口，stylesheet MIME/nosniff 判定矩阵——link-style-error 三文件 36F 全转绿 + 全量案级 join 真修复 41F→P、假阳消除 2 案挂账）——其下一步 s152 已由 s152 收口（候选①Face C 时序面：静态 link 文档序交错触发 + trickle 扫描窗折叠——v1 全前置扫描语义错误勘根转折 + fetch wrapper setTimeout 延迟单根定谳，fired-before-scripting 2F→2P 零回归，见上**：
> **入场定刀位**：s150 下一步候选①（Face B MIME/nosniff 判定矩阵——勘根证实
> s150 后三文件真值 42P/36F 而非在册「48P/12F」口径）。
> **勘根 6 条**：①上游 css.py 真值（pin master 逐字）——`content_type` 参数
> 原样作 Content-Type 头（**缺席 → 无 CT 头**）、`nosniff` 旗标 →
> `x-content-type-options: nosniff`、`cors` 旗标 → ACAO:*（语料 URL 未携带）、
> 体恒 `body { background:red }`、status 200；runner fetch handler 响应 headers
> 通道已有（s138/s150 端点先例，`wpt_add_fetch_metadata` 不加 CT——no-CT 案
> 可保真）。②期望公式定谳（css-syntax §stylesheet-content-type-error）：no-CT+
> 非nosniff→load；wrong-CT(text/plain)+no-nosniff+quirks+同源→load（legacy
> 宽容）其余→error；broken-CT（oops 无 '/' 不可解析）+非nosniff→load；
> **limited-quirks 文件 compatMode 也是 CSS1Compat**（期望与 01 同形），唯
> quirks 文件（BackCompat）激活宽容分支。③runner 通道 compatMode 缺位实证
> （探针 zz-s151-probe-quirks）——quirks doctype 页 `document.compatMode`
> 实得 CSS1Compat（shim part06 常量遮蔽），quirks 宽容分支从未激活、三文件
> 判式全按 01 语义跑；宿主侧真值链已备（load_html → dom_bindings DOM_SOURCE
> → zero_dom parse_html quirks 判定 set_quirks_mode 在位）。④shim Response
> headers 通道现成——`__zw_fetch` wire 含 headersWire、`_parseHeadersWire`/
> `_takeHeaderValue`（part01）+ Content-Type 大小写不敏感，判定无需桥扩展。
> ⑤页锚 scheme 撕裂——上游语料假设 http 页（wptserve 无 mixed-content 阻断、
> HTTP_REMOTE_ORIGIN `http://www1.wpt.test:8000` 相对判定），runner 页锚
> https 时 CO 案被 s150 helper mixed 门拦死 error、期望 load 的案到不了 MIME
> 判定面（net-api/webcrypto/referrer `*_case_uses_http_page` 谓词先例同构）。
> ⑥fetch shim CORS 两截面（part01:3955–4014）——默认 cors 模式跨源无 ACAO →
> reject「Failed to fetch」；ACAO 通配放行后 cors 过滤响应剥非 safelisted 头
> （`x-content-type-options` 读不到）→ 端点须 ACAO + ACEH 双 `*` 头（`*`
> 通配在非 include 分支生效，3988–4001 实码确认）。
> **施工 6 件净 +155/−8**（shim 2 文件 + 宿主桥 2 文件 + runner testharness.rs）：
> ①callbacks.rs `__zw_get_compat_mode` 桥——经 live Document `quirks_mode()`
> 求值（Quirks→"BackCompat"、其余→"CSS1Compat"），映射语义封装
> `dom_bindings::document_compat_mode_string()`（document.rs 新增，mod.rs
> `pub(crate) use` 导出——gc/document 子模块私有，桥不可直入）；②shim part06
> compatMode 常量改 getter（桥读、值白名单 BackCompat/CSS1Compat、桥缺席回落
> CSS1Compat；scrollingElement 不读该属性，本刀不动滚动面）；③shim part03
> `_zwStylesheetJudge`（nosniff 严检 / wrong-CT standards-strict quirks-宽容 /
> broken-CT 无头放行——上游判式逐腿同构，origin 判定 `new URL().origin` 比较
> 页锚）+ `_zwStylesheetImports`（`url('..')`/`".."`/`'..'` 三书写形，去重
> 保序，绝对化失败项跳过）；④shim part03 fetch 回调改造（s150 helper）——
> ok → 判定 → 通过后 `response.text()` 展开 @import 内层 URL 逐个 fetch +
> 同判定链（一层递归——内层响应体不再展开语料形态；pending 计数聚合，任一
> error → 外层 error、全部 load → load）；s150「@import 内层解析不在本刀面」
> 注记同步更新；⑤runner css.py 端点（stylesheet.py 臂后）——content_type /
> nosniff / cors 参数 + 体恒式；无条件补 ACAO/ACEH `*`（上游 wptserve
> stylesheet 加载不走 CORS 检查，两头是通道妥协——语料不读 ACAO/ACEH，期望面
> 不变）；⑥页锚谓词 `link_style_error_case_uses_http_page`（三文件名尾缀
> 匹配，referrer 先例同构）接入 page_scheme 判定。
> **验证**：探针 quirks compatMode=BackCompat Pass（真值化链路通）；定向
> link-style-error 三文件 **74P/0F/0T**（行集守恒 26+24+24；基线 42P/36F 全
> 转绿——`<link>` 形期望 load 案（no-CT/broken no-nosniff SO/CO）判定放行 +
> 期望 error 案从 404/mixed 假阳变真判定，`@import` 形期望 error 案内层判
> error 传播、期望 load 案保持；quirks 宽容分支激活后 wrong-CT SO 案 load ✓
> 三文件判式不再同判）；定向 document-metadata 全目录 + link-multiple 复验
> 4P + 文件级 Timeout（案 6 preload 页级 10s 必然面——s150 在册挂账不变；
> media 触碰案 1–4 稳定 P 与 s150 同形）；全量 join（**案级全键**，对 s150
> 合并树基线 /tmp/zw-s150-rebase-full.log；文件粒度键 join 会折叠案名对案级
> P→F 不敏感，本轮方法论修正）真修复 **41 案 F→P**（link-style-error 36 +
> link-load-error-events 5——non-CSS 三案内层 404 error 传播 + http Import
> 两案跨源 CORS reject error 传播——`success: !isHttps` 页 https 时期望
> error，s150 外层恒 load 恒 F）+ **假阳消除 2 案 P→F**（https Import 两案
> ——s150 假阳 Pass：内层未取、外层 data: 恒 load 凑巧命中期望；本刀内层
> 真取 `https://wpt.test:8444` 跨端口=跨源撞 fetch CORS check（通用静态臂无
> ACAO）→ reject ≠ 期望 load，挂账见下）+ flake 形态非本刀位移（select-event
> 26P 键增 + 文件级 Timeout 键消——在册全量负载 flake 复绿形，s148「负偏差
> 恢复 39」/s150 合并树复验 270/270 先例同族；link-multiple media 案 1P——
> s150rb 全量早收形态少报，定向复验案 1–4 稳定 P）+ 文件级键 −3F（三文件）
> −1T（select-event）。
> **门禁**：clippy workspace --all-targets -D warnings 零告警；runner bin
> **265/0F**；fmt 过（本轮 .rs 干净）；shim 规范序拼接 node --check SYNTAX_OK
> 110138 行。
> **范围**：engine js_dom_shim part03.js/part06.js + js_dom_bridge callbacks.rs
> + dom_bindings document.rs/mod.rs + runner testharness.rs；wpt-data 零改动
> （探针 zz-s151-probe-quirks 验后已删）。
> **挂账更新**：新记**跨源静态资源 ACAO 面**——httpsSheet 内层链
> （`https://wpt.test:8444` 跨端口）被 fetch CORS check 拒（通用静态臂无
> ACAO），2 案假阳消除后挂账、上游期望 load；修法候选：通用静态臂加
> ACAO/ACEH（影响全语料静态资源语义，需影响面盘点）或 helper 通道对
> stylesheet 加载改 no-cors 语义仿真（opaque 头剥更严，不可行）。承袭不变：
> Face C script/render-blocking 时序面（conditionally-block 2F、
> link-error-fired-before-scripting-unblocked 等）；adopt/innerHTML 身份双
> 表示深水面（document-without-browsing-context 2F）；LinkStyle sheet 属性面
> 1F；link-load-error-events 余 https Load 案 1F（同跨源 ACAO 面，非 import
> 形一直 Fail）；rel=preload fetch 面（link-multiple 案 5 + 案 6 页级 10s
> 时序必然）；stylesheet-media-change-no-fetch 1F；@import 多层递归链（本刀
> 一层，语料无多层形态）；meta long × corpus 通道放宽失效；details 族 lib
> flake；promise_test 串行启动面；待决三项（css/text 语料资源、green-2x2.png、
> parser form pointer 缓行）须用户拍板。
> **下一步 s152 候选**：①Face C render-blocking 时序面勘根（conditionally-block
> 2F / link-error-fired-before-scripting-unblocked——link load 时机 vs script
> 阻断语义）；②跨源静态资源 ACAO 面影响面盘点（通用静态臂加 ACAO 全语料
> 波及评估——2 案 + https Load 案 3F 修复面）；③adopt/innerHTML 身份双表示
> 深水面（document-without-browsing-context 2F）；④rel=preload fetch 面
> （link-multiple 案 5 + 缓存复用）。
> **账本**：[evidence/2026-10-29-m2s151-stylesheet-mime-nosniff-matrix.md](html-semantics-compat/evidence/2026-10-29-m2s151-stylesheet-mime-nosniff-matrix.md)
> **▶ 上轮状态（2026-10-29 M2 切片 150 施工收口，动态 link rel=stylesheet 事件面——link-multiple 4F→4P + 8 文件级 Timeout 键消 + 零真 P→F + s149 单测回归顺带修复）——其下一步 s151 已由 s151 收口（Face B MIME/nosniff 判定矩阵：css.py 端点仿真 + shim @import 内层链 + compatMode 真值化，link-style-error 36F 全转绿 + 真修复 41 案、假阳消除 2 案挂账；其「48P/12F」定向口径经勘根证伪实为 42P/36F，见上**：
> **入场定刀位**：s149 下一步候选①（早收形 91 文件 partial rows 单根翻拣）
> 首靶 document-metadata/the-link-element/——勘根发现 stylesheet.py 端点仿真
> 补不齐：runner headless 路径**动态 link 完全无 fetch、无 load/error 事件
> 记账**，升级为 shim 事件面主刀（Face A），端点仿真为配套。
> **勘根（探针 p1–p4 + 浏览器路径对照）**：①runner headless `resolve_external_css`
> 仅静态 CSS 收集，动态 `<link rel=stylesheet>` 插入后**零 fetch 零事件**
> （动态插入/setAttribute href/innerHTML 注入/load+error 监听四探针 1s 窗口
> 均「neither event」）；②浏览器 AsyncPageLoad 路径完整（静态 link 有 fetch
> 与样式应用）——事件面缺失是 headless shim 通道结构洞非 runner 特有；
> ③幂等语义（HTML spec link processing + CSSOM view）——href 解析后绝对
> URL 不变不重取、rel 空白分词含 stylesheet 即算、detached 不触发（连接门）、
> https 页 http 子资源 mixed-content 阻断 → error（webappsec
> should-block-fetch）；④shim 身份双表示约束——innerHTML flatten 产物
> plain handle（`'__n1'` 形）宿主 tag/attr 表 miss（`_realTag` 回落恒 DIV），
> s111 既有 LINK/SCRIPT 甄别同样 miss；s124 modulepreload helper 为结构模板。
> **施工**：shim 4 文件 + runner 1 文件净 +232——①part03
> `_zwStylesheetMaybeStart` helper：TAG 门（plain handle 形放行，后继
> rel/href 门兜底误触发面）→ 连接门（`_zwMutationInDoc` 父链爬升）→
> attr 读通道（sel/handle 宿主桥 lw 读 + plain handle 形 elObj 第 4 参
> getAttribute 回落）→ rel 分词 → href 非空 → 幂等门（prevHref 第 3 参
> 绝对 URL 比较，null = 插入总取）→ mixed 门（setTimeout error）→ fetch
> `ok ? load : error`（`_dispatchWithBubble` + `_makeEvent` bubbles:false
> 元素站）；②part01 s111 两臂（顶层 `_an109n` + 下降 `_walk111`）LINK 甄别
> 修正（plain handle 形按节点 tagName 判，仅 LINK——SCRIPT 行为不动）+
> 插入触发；③part04 setAttribute href 写前捕获旧值（`_zwSsPrev` lw 读）
> + 写后触发（写前捕获是幂等基准正确关键——写后再读即新值恒真恒不触发）；
> ④part05 R3069 臂同形钩子（`_zwSsPrevI`）；⑤runner stylesheet.py 端点
> （per-case `Arc<Mutex<HashMap>>` 计数表，take-and-clear 同形——`count`
> 参数读计数不回填、否则 take+1 回填返 `text/css` `body {color: red;}`）
> + 上游字节 good.css/style.css 落 wpt-data。
> **验证**：link-multiple-load-events 案 1–4 **4F→4P**（幂等门精确
> count=1/count=0 断言全过：插入取 1 次、href/rel/type/media 触碰不重取）；
> link-load-event-alt-001 **T→P**；link-load-error-events 文件级 Timeout 消
> （10s 即收出具体行，8 案 Pass：Basic load/404/Import 链 success/https
> control）；link-style-error 三文件事件面活化 **48P/12F**（nosniff/MIME
> 判定缺失为预期边界 Face B）；stylesheet-bad-mime-type 2P +
> stylesheet-non-OK-status 1P（ok:false → error 实证：wpt_pipe_status 404
> 仿真 + MIME 面上游同形）。**全量 join**（对 s149 /tmp/zw-s149-full.log，
> (key,state) 对 sort -u）：s150 **7148P/847F/88T = 8080 对**（s149 7993
> 对），lost 18 全审计——4F→P link-multiple 幂等案 + 6F→P urlencoded2
> 归档观察 + 8 文件级 Timeout 键消——**lost 中 P 键 = 0 = 零真 P→F**。
> urlencoded2 6F→P 归档观察（不冒认修复）：与 shim/runner 改动零机制关联、
> 三跑稳定（全量 + 定向 ×2 均 62P/0F）、同族 text-plain/multipart 12F
> 逐字节不变；假说 s149 全量轮两次撞 test-guard 单进程 4GB 限的内存压力
> 窗口伪红（encoding 解码内存敏感路径），encoding 流对账时核销。
> **s149 单测回归顺带修复**：runner bin 263/1F——
> `waits_for_active_step_timeout_before_accepting_completion` 0.1s 确定性
> 失败，干净树（HEAD=s149）同红、父提交 6e9f0a5ac 绿 → **s149 引入**
> （其提交信息「runner bin 264/0F」声明与事实不符，验证盲区）。机理：s149
> probe JS `__zw_harness_complete` 检查前移 timer grace——completion 已发
> 即立即终态，老 mock 在 async_test 工厂内同步发 completion、4ms timer 后
> 才发结果行（真实 pinned testharness 无此活路径：页级超时强制 COMPLETE 后
> done()/step_func_done 全 no-op）→ runner 立即收口丢 mock 结果行。裁决：
> 生产语义正确（s149 主目标由 `harness_probe_is_terminal` phase==4 臂达成
> 不依赖前移；前移防恒近 pending timer 页面空转——s148 91 例实证）——修
> 测试不回滚生产：mock 改为结果行后发 completion（真实顺序同形，原断言
> 契约保留）+ 新增早收形守卫测试
> `early_completion_with_no_results_yields_file_level_timeout_s149`
> （completion 先发无结果行 → 立即收口 + 文件级兜底 Fail 键不空转）。
> runner bin **265/0F**（264 旧 + 1 新增零失败）。门禁：clippy workspace
> --all-targets -D warnings 零告警；fmt 过（本轮 .rs 干净；既有
> text_encoding.rs fmt 漂移归口 encoding 流提交 a828dfac4 遗留——
> `make test` fmt 前置门会先挡它，按工作面不重叠纪律不动仅提及）。验证集
> 对齐 s142–s149 同面先例（无 reftest 义务——行为面非渲染）。**合并树
> 复验**（提交后 pull --rebase 并入 encoding 流 d31398b12/548019ec7 两提
> 交，s143/s144 先例）：全量重跑 7122P/847F/89T——位移 26 P 键 + 1 文件
> T 键全部集中在 textfieldselection/select-event.html（该文件全量跑文件
> 级超时早收形，在册负载 flake s148「负偏差恢复 39」先例同族）；定向复
> 跑 **270/270 键逐字节复绿**（零真回归，encoding 流并入与 link 事件面
> 零交叉）；runner bin 合并树复出 **265/0F**。账本见
> [evidence/2026-10-29-m2s150-dynamic-link-stylesheet-event-face.md](html-semantics-compat/evidence/2026-10-29-m2s150-dynamic-link-stylesheet-event-face.md)。
> **范围**：engine src/js_dom_shim/part01.js/part03.js/part04.js/part05.js
> + tests/wpt-runner/src/testharness.rs（stylesheet.py 端点 + 单测修复）。
> **挂账更新**：**Face B MIME/nosniff 判定矩阵**（link-style-error 12F
> wrong/no Content-Type sniff 语义 + data: URL @import 内层解析 10F 主体 +
> 跨源 ACAO——shim fetch 通道无 CORS 判定；css.py 端点可配套）；**Face C
> script/render-blocking 时序面**（link-load-event/script-unblocked 两案/
> interactions 组 delay/conditionally-block/stylesheet-change/
> remove-href 系）；**新记 stylesheet-media-change-no-fetch 1F**（media
> 变更不重取语义，幂等门只挂 href，Face B 邻接面）；**link-multiple 案
> 5/6**（案 5 需 rel=preload fetch 面 + 缓存复用；案 6 页级 10s 时序必然
> ——案 1–4 各吃 ~2.1s race 窗口计 8.4s → 案 5/6 撞早收，上游靠 preload
> 快速 settle）；承袭不变——document-without-browsing-context adopt/
> innerHTML 身份双表示面（R3031/t8r，2F 持平）、meta long × corpus 通道
> 放宽失效（s149 记）、details 族 lib flake、promise_test 串行启动面、
> 待决三项（css/text 语料资源、green-2x2.png、parser form pointer 缓行）；
> **观察归档 urlencoded2 6F→P**（s149 内存压力伪红假说，encoding 流对账
> 核销）。
> **下一步**：s151 入场定刀位（候选：①Face B MIME/nosniff 判定矩阵
> （link-style-error 12F 真兼容面 + css.py 端点仿真，s138/s150 端点先例）；
> ②Face C render-blocking 时序面勘根（link-load-event/script-unblocked）；
> ③adopt/innerHTML 身份双表示深水面（document-without-browsing-context
> 2F）；④rel=preload fetch 面（link-multiple 案 5 + 缓存复用））。
> **▶ 上轮状态（2026-10-29 M2 切片 149 施工收口，runner 终态判定早收形收口——文件级 Timeout 族 91/96 单根，join 精确守恒零 P→F）——其下一步 s150 已由 s150 收口（候选①首靶 link-element 勘根升级 shim 事件面主刀：动态 link rel=stylesheet 插入/href 变更 fetch + load/error 派发，link-multiple 4F→4P + 8 文件级 Timeout 键消；其「runner bin 264/0F」门禁声明经复验证伪（单测回归已顺带修复），见上**：
> **入场定刀位**：s148 下一步候选①（文件级 Timeout 族单根翻拣——
> link-multiple-load-events/csp-blocked，s148 观察①同构「per-test timeout
> 缺失面勘根」）。承前注记：上一 session 429 中断前（16:31–16:40）已在
> form-submission-0/ 落 4 个勘根探针（zz-s149-probe-{timer,ladder,trace,ht}，
> gitignored 临时语料）——timer/ladder Pass（setTimeout 300ms 真实 fire +
> clearTimeout 在位）、trace/ht 文件 Timeout，与本轮勘根同向；干净全量前
> 已按惯例删除。
> **勘根（上游 harness 源码 + 定向复现定谳）**：①定向复现 link-multiple-
> load-events——4 案 2s step_timeout 兜底正常翻 Fail（timer 通道无恙）+
> 文件级 Timeout，completion state `pending:2, phase:4, tests:6` = **harness
> 已自收（completion 回调已发）而 runner 报「completion callback was not
> called」**。②上游结构定谳——pinned testharness.js **无默认 per-test 超时**
> （`settings.test_timeout=null`，Test 级定时器仅显式 `setup({timeout})`
> 才挂；控制面原文「上游 8s 默认 per-test 超时」假设不成立），真实兜底 =
> Tests 构造期 arm 的**页级 10s 超时**（`<meta name=timeout content=long>`
> 60s）：`Tests.prototype.timeout` 全部测试强制标 COMPLETE → `complete()`
> → `notify_complete()` 同步发 completion 回调；早收后未报测试 done()
> no-op → **结果行不再变化、num_pending 恒 >0**。③runner 侧单点归因——
> `terminal_harness_state` 要求 phase4+pending0+tests==results 配平，早收形
> 永不满足 → 探针空转 30s 案超时；s148 全量 96 文件级 Timeout 键 **91 例
> phase==4**（5 例 phase==2 独立子面见挂账）。④次因同修——probe JS
> complete 计算把 1s timer grace 放在 `__zw_harness_complete` 之前，页面有
> 持续定时器时 completion 已发也到不了终态。
> **施工**：tests/wpt-runner/src/testharness.rs 3 处（净 +26/−3）——①probe
> JS `__zw_harness_complete` 检查前移 timer grace 循环；②
> `harness_probe_is_terminal` 新增 `complete && state.phase==4` 臂；
> ③终态收集块早收形保留**文件级 Timeout 键**（upstream wptrunner 对
> harness status=TIMEOUT 文件记 TIMEOUT + partial rows，键/态与案超时旧
> 路径同形 = join 零位移）+ 消息准确化（`page timeout completed the file
> with N test(s) unreported`）。
> **验证**：定向 link-multiple-load-events 行集逐字节同形（4 Fail + 1 文件
> 级 Timeout），**墙钟 30s→10.1s**（`ZW_TESTHARNESS_TIME_LOG=1` run=9995ms
> = 页级超时即收）；定向 input-type-change-submit **20/20 全 Pass** 零回归；
> **全量 join**（对 s148 /tmp/zw-s148-full.log，(key,state) 对 sort -u）：
> s149 **7087P/810F/96T = 7993 对**（s148 7085P/812F/96T = 7993 对）——
> **键对数与 96T 键集逐字节守恒**，唯一位移 = modulepreload-referrerpolicy
> 2F→P（s148 在册全量负载 flake 本轮自然复绿，s124 族先例），**零真 P→F
> 零键增减**；早收 91 = 26（partial rows + 文件级 Timeout 标记）+ 65（零行
> 既有「completed without reporting」路径）+ 5 phase==2 原路径。门禁：
> clippy workspace --all-targets -D warnings 零告警；runner bin **264/0F**
> （s148 基线 263/1F——trend_csv_check 本轮已绿，并行流归口）；fmt 过；
> 零 shim/.js 改动（纯 runner 终态面，产品路径零触碰）。验证集对齐
> s142/s145/s146/s147/s148 同面先例（无 reftest 义务——行为面非渲染）。
> 账本见
> [evidence/2026-10-29-m2s149-runner-terminal-page-timeout-early-complete.md](html-semantics-compat/evidence/2026-10-29-m2s149-runner-terminal-page-timeout-early-complete.md)。
> **范围**：tests/wpt-runner/src/testharness.rs（3 处）。
> **挂账更新**：**候选①「per-test timeout 缺失面」定谳收口**（缺失的不是
> harness 超时——页级 10s 兜底在位且真实 fire；是 runner 终态判定对早收形
> 的识别；键集报法漂移结构根随完成点确定性收窄）；**新记挂账：meta
> timeout=long × corpus_case_timeout 通道**——5 例 phase==2
> （object-events/beforeinput.tentative/rel-attribute-clearing/
> target_blank_implicit_noopener ×2）全带 meta long：`run_testharness_html`
> long 放宽判定 `timeout == CASE_TIMEOUT`（10s）对 corpus 通道（30s）失效
> → 60s 页级超时无窗口 fire → 30s 案超时收场（T 分类与上游一致，仅墙钟
> 差；放宽语义是否对 corpus 通道恢复待后续单独定夺）；承袭不变——details
> 族 lib flake、无锚链碰撞理论面盘点、with(window) 影子旁路面、影子名单
> 非注册类 API 不全、engine lib 环境面、R102 live_aware pending 失明、
> promise_test 串行启动面（s148 观察①——本轮后仅影响文件内部尾部冻结，
> 文件级完成已不受牵连）、待决三项（css/text 语料资源、green-2x2.png、
> parser form pointer 缓行 2026-10-23）。
> **下一步**：s150 入场定刀位（候选：①早收形 91 文件 partial rows 单根
> 翻拣——真兼容面 2F→P 类，如 link-multiple 4F 的 stylesheet.py 端点仿真
> 面勘根（s138 echo 端点先例）；②63F 目录基线面再翻拣（s148 候选④承
> 袭）；③无锚链碰撞理论面影响面盘点（s148 候选②承袭）；④encoding 流
> M1 进度对账（s148 候选③承袭））。
> **▶ 上轮状态（2026-10-29 M2 切片 148 施工收口，input-type-change-submit 挂死根收口——url/email 案翻 Pass 全文件 20/20 + 零真 P→F）——其下一步 s149 已由 s149 收口（候选①文件级 Timeout 族单根定谳：非 per-test timeout 缺失，系 runner 终态判定不认 harness 页级超时早收形，91/96 单根收口 + join 精确守恒），见上**：
> **入场定刀位**：s147 候选④ form-submission-0 余下基线 Fail 面单根翻拣——入场甄别：
> urlencoded2/text-plain/multipart 各 6F = 18 案归口 encoding 流 M1（不重复施工）；
> submission-checks 3F = `document.createElement("frame")` 需 ~30 shim gate 点 +
> host 侧代价过高否决；取 input-type-change-submit（C2 face）——定向复现
> text/search/tel 3 Pass 后文件级 Timeout（state `pending:17` = 第 4 案 url
> `loadPromise` 永不 resolve；s147 全量该文件仅 3 行 + 1 文件 Timeout 对）。
> **勘根（上游定谳 + 探针四案三分辨）**：①上游基线——上游同路径文件存在且无
> .ini 期望文件（gh api 核对）→ 上游全绿，语料 20 案全断言导航取参 n1=v2，
> 真浏览器不拦 url/email 案。②spec 面——click handler 派发窗口内改
> `input.type="submit"`，默认动作（input activation behavior §4.10.5.4）运行时
> type 已 Submit Button（§4.10.21 barred from constraint validation）→
> typeMismatch（§4.10.5.2.6 仅 email/url state 适用）不触发，提交放行——
> shim 拦截 = 实现伪象。③探针四案（临时语料 zz-s148-probe-itcs，跑后已删）
> ——p1 detached createElement input（handle 分支）type 改 submit 后
> validity.valid 翻真；p2 sel 路径 form 内 input（insertAdjacentHTML 产物）
> click 后 invalid 事件仍派发；p3 同形态 iframe 不导航（提交中止实锤）；p4
> 无 type 变更 click 无 invalid 无导航（非 submit 激活 no-op，激活门语义旁
> 证）——p1/p2 矛盾定谳视图分裂，唯一分歧点 = type 属性读取通道。④单点归
> 因——`_validityState`（part03.js:11597 前态）type 读 sel 分支走**纯快照**
> `__zw_get_attr`，R2995 latest-wins 下 type IDL 写只落 lw 变更列表不落快照
> → 校验时 ty 恒旧值（url/email）→ typeMismatch → invalid 中止提交 →
> loadPromise 挂死 → 文件级 Timeout；**s136 同根第 2 处**（s136 修激活门
> part04 两处 type 读换 lw，本轮补 validity 计算位）。⑤观察记录——挂死时
> pending:17 = 案 5-20 从未启动：shim promise_test 体**串行启动**（前案未决
> 不启后案；上游注册即启动 20 案并行），单案挂死冻结尾部结构面与 Timeout
> 族直接相关（新挂账①）。
> **施工**：part03.js 1 处（净 +9）——`_validityState` type 读 sel 分支换
> `__zw_get_attr_lw` 融合读（consult 变更列表 → applied view 回落，s136 同形
> 三目 + typeof 守卫；handle 分支 `__zw_get_attr_handle` 本身 consult 当前批
> mutations 不动）；3805 SVG script 加载 / 13612 族 checkbox/radio pre-click
> 快照 type 读非本语料流 exercised 不触碰（精准修改）。
> **验证**：定向 input-type-change-submit **20/20 全 Pass**（url/email 案从挂
> 死翻 Pass）；**全量 join**（对 s147 /tmp/zw-s147-full.log，(key,state) 对
> sort -u）：s148 **7085P/812F/96T = 7993 对**（s147 7068P/812F/97T = 7977
> 对），lost 5/gained 21 全归因**零真 P→F**——input-type-change-submit 文件
> Timeout 对 → 17 子测试 Pass 对（主刀）；form-double-submit-input-type-change
> **2F→2P 同根连带**（click handler 内 email/url→submit 同形态 stale
> typeMismatch 同根误拦——候选④ 面意外再收 2 案）；modulepreload-
> referrerpolicy 2P→F **全量负载 flake**（定向复跑 7/7 全 Pass 复绿；s124 轮
> 「两 flake 页过滤重跑复绿」在册先例同族；validity 读面与 module 加载/
> referrer 通道零机制关联）。门禁：clippy workspace --all-targets -D warnings
> 零告警；shim 13-part 规范序拼接 node --check 过（**109736 行** = HEAD
> 109727 + 9 净增；s147 账本 109701 为注释措辞整理前口径）；engine lib
> **3593/10F**（失败名单与 s147 逐字节相同——8 已知环境面 + R3045 归口
> net-api + details 在册间歇红；+1 pass = s147 rebase 并入 net-api 流新增测
> 试）；runner bin **263/1F**（trend_csv_check 归口 encoding 流不变；+5 pass
> 同 rebase 源）；零 .rs 改动（fmt 无面）。验证集对齐 s142/s145/s146/s147
> 同面先例（无 reftest 义务——行为面非渲染）。账本见
> [evidence/2026-10-29-m2s148-input-type-change-validity-stale-type.md](html-semantics-compat/evidence/2026-10-29-m2s148-input-type-change-validity-stale-type.md)。
> **范围**：engine src/js_dom_shim/part03.js（1 处）。
> **挂账更新**：**C2 input-type-change 面收口**（面域 = 全文件 20 案——挂账
> 原文「url/email 2F」系旧报法口径，近轮全量实为 3 行 + 文件 Timeout，本轮
> 翻 20/20）；**form-double-submit-input-type-change 2F 连带收口**（同根连
> 带）；**候选④ 面关闭**（18 案归口 encoding + submission-checks FRAME 否决
> + input-type-change 收口，三项全定谳）；**新观察挂账 2 条**：①shim
> promise_test 串行启动面（单案挂死冻结全文件尾部，上游注册即启动语义——
> Timeout 族与 per-test timeout 缺失面的共同结构根）；②modulepreload-
> referrerpolicy 全量负载 flake 2 键（strict-origin/unsafe-url，定向复绿，
> s124 族观察先例）；承袭不变——details 族 lib flake ≈40% 翻转率独立勘根、
> link-multiple/csp-blocked 文件级 Timeout 族（候选①）、无锚链碰撞理论面
> 盘点（候选②）、with(window) 包合分支影子旁路面、影子名单非注册类 API
> 不全、engine lib 9F 环境面、R102 live_aware pending 失明；**待决（须用户
> 拍板）**：css/text 语料资源、green-2x2.png、parser form pointer 缓行
>（2026-10-23）。
> **下一步**：s149 入场定刀位（候选：①文件级 Timeout 族单根翻拣——
> link-multiple-load-events/csp-blocked（刀 B 拆账余面 + 本轮观察①同构：
> per-test timeout 缺失面勘根——上游 testharness 8s 默认 per-test 超时，挂
> 死案翻 Fail 行、文件完成，键集报法漂移解除面）；②无锚链碰撞理论面影响
> 面盘点（刀 D 残余，盘点性质）；③encoding 流 M1 进度对账（不施工，仅边
> 界对账）；④全量基线目录面再翻拣（63F 目录基线面其他文件族单根勘根））。
> **▶ 上轮状态（2026-10-29 M2 切片 147 施工收口，刀 B「runner 同键多行」根收口——testharness 注册 API 子文档 realm 隔离，form-submit-action 10 键真翻红 + 零 P→F）——其下一步 s148 已由 s148 收口（候选④ input-type-change 挂死单根——_validityState type 读快照盲区 s136 同根第 2 处，全文件 20/20 全 Pass），见上**：

> **过程注记**：本切片勘根与施工主体在上一 session 完成（429 限流
> 中断于全量验证后），本轮从工作树现场接续：复核 diff、重建 join、
> 补齐定向复现与全部门禁，证据口径以本轮可复现命令为准。
> **入场定刀位**：s146 下一步候选①（刀 B runner 多跑机制勘根——多
> 跑/首遍挂/键集报法漂移三面，s144/s145/s146 三轮在册漂移观察）。
> **勘根（探针七轮二分 + runner 架构核对定谳）**：①「多跑」不是
> runner 循环重跑——`run_html_semantics_cases`
> （testharness_html_semantics.rs:144）每文件只执行一次
> `run_testharness_html`，`Vec<HarnessSubtestResult>` 按注册逐条打
> 印，同键多行 = **同名多次注册**（s141「runner 执行 3 遍」旧解读
> 修正）。②泄漏通道 = 同路径自导航递归——form-submit-action
> 「missing form action」案（formAction/submitterFormAction/
> formOwnerAction 全 null）提交 URL = 文档自身地址 → target iframe
> 装载**父页副本** → 嵌套副本脚本体（R206 `_zwRunIframeScripts`
> 合并 Function）裸标识符 test()/async_test() 落 globalThis（宿主
> harness 所在）→ tests 数组重复注册 ×N → 嵌套副本继续
> populateForm/提交递归（s146 实测 4 层 Fail 注册 + 1 层清洁
> Pass）；上游语义 iframe 文档脚本 test() 走子窗自身全局（独立
> realm）——无 harness 即 ReferenceError 永不可达父窗 harness。③
> 首遍挂/键集漂移同根：嵌套副本 loadPromise/导航链挂死 → harness
> 永不 done → 文件级 Timeout；挂起时序下注册子集上报 → 键集随负
> 载漂移。探针链 probe1-4（修复前/部分变体）恒 4×Fail 注册，
> probe5-6（影子落位）1×Pass 收敛。
> **施工**：part01.js 1 处（+36/−2）——R206 页面段求值前置影子声
> 明：激活门 = 嵌套文档路径（去 query/fragment）== 顶层 case 路径
> （仅同路径自导航形态；异路径 helper iframe runTestsFromIframe/
> dynamic-imports-credentials 族的嵌套 test() 落宿主 harness 承重
> 契约不动）；18 个 harness 注册 API 名（test/async_test/
> promise_test/generate_tests/setup/promise_setup/done/on_event/
> step_timeout/add_*_callback ×3/promise_rejects_* ×4）各生成
> Function 作用域影子声明，**声明式委派** window[name]（调用时取
> 值——harness 段先于页面段求值，prologue 直捕拿 pre-harness
> undefined）；子窗无该名 → ReferenceError 中止（per-part try/catch
> 接住，副本页面段首个注册调用即止、populateForm/再提交级联源头
> 掐断）。安全边界：影子仅本 Function 作用域；exportSuffix206 扫
> 描 parts 原文不含影子名——无 win 回写无自委派环；页面段同名
> function/var 声明照常覆写影子；门未命中拼入空串 corpus 面逐字
> 节不变。
> **验证**：定向 form-submit-action **10/10 全 Pass 单清洁执行**
> （修复前 s146 全量 50 行 = 40F+10P 五注册；s141 遍 1/2 全挂
> gEBI click not function 旧版失败证据在 /tmp/zw-s146-full.log 50
> 行块）；定向 src-then-lazy-load **8/8 全 Pass**（s146 全量 7P +
> 文件 Timeout；本轮全量该文件 8 键 Pass 文件 Timeout 未复现）；
> 定向 link-multiple-load-events media 键 **Fail 确定性复现**
> （s146 被文件挂起吞没——键集报法漂移解除面非代码位移），文件
> 级 Timeout 保持（独立挂账面，该文件无表单不自导航与级联根无
> 关）；**全量 join**（对 s146 /tmp/zw-s146-full.log，(key,state)
> 对 sort -u 去重）：s147 **7071P/812F/97T** = 7980 结果行（Δ =
> −40 form-submit-action 级联 Fail 行 +1 link-multiple media 行），
> 键对 **7986 → 7977** 位移 13 条全归因**零 P→F**——lost 11 =
> form 10 Fail 对（键级真翻红：同键 Pass 对两轮均在，Fail 态为级
> 联注册假象态）+ src-then-lazy 文件 Timeout 对（T→P）；gained 2 =
> src 第 8 键 `srcset = /images/not-found 2 time(s)` Pass +
> link-multiple media 键 Fail。门禁：clippy workspace --all-targets
> -D warnings 零告警；shim 13-part 规范序拼接 node --check 过
> （109701 行 = s146 109667 + 34 净增）；engine lib **3592/10F**
> （总数 3602 守恒；9 名单与 s146 逐字节相同 + details 在册间歇本
> 轮采样红——s145 ≈40% 翻转率观察同型）；runner bin **258/1F**
> （trend_csv_check 行 196 归口不变）；零 .rs 改动（fmt 无新面）；
> release runner 二进制（14:35 构建，早于 14:44 全量日志）
> `grep -a` 核验 `_zwRealmShadow` 在执行体内（构建后注释措辞整理
> 不改行为，eval 源内注释惰性）。验证集对齐 s142/s145/s146 同面先
> 例（无 reftest 义务——行为面非渲染）。
> 账本见
> [evidence/2026-10-29-m2s147-realm-shadow-self-navigation-cascade.md](html-semantics-compat/evidence/2026-10-29-m2s147-realm-shadow-self-navigation-cascade.md)。
> **范围**：engine src/js_dom_shim/part01.js（1 处）。
> **挂账更新**：刀 B「同键多行」面收口（本轮主刀——多跑/首遍挂/
> 键集漂移三面同根单点收口）；**残余理论面 2 条新记**（低优先，
> 踩坑语料再补）：①with(window) 包合分支（页脚本含
> on(message|popstate) 命中 R206 包合）下影子在 with 对象环境内
> 层，若子窗 win 暴露同名 harness 属性则 with 查找先于 Function
> 作用域影子——既有全量绿面未踩未验证；②影子名单为注册 API 子
> 集（非注册类 harness 全局不在列），上游 realm 全隔离语义更宽按
> 语料踩坑扩面；**刀 B 余面拆账**：details 族 lib flake ≈40% 翻转
> 率（s145 归因实验定谳非确定性调度非级联根）保持独立勘根挂账；
> link-multiple-load-events / csp-blocked 文件级 Timeout 族保持独
> 立挂账（非自导航形态本轮 join 证实与级联根无关）；承袭不变——
> C2 input-type-change url/email 2F、engine lib 9F 环境面、runner
> trend_csv_check 行 196（归口 encoding 流）、「真实再物化」观察
> 项、无锚链碰撞理论面盘点（刀 D 残余）、R102 live_aware 回调簇
> pending 失明理论缺口；**待决（须用户拍板）**：css/text 语料资
> 源、green-2x2.png、parser form pointer 缓行（2026-10-23）。
> **下一步**：s148 入场定刀位（候选：①全量基线翻拣下一单根面
> （63F 目录基线面之外的文件级 Timeout 族——link-multiple/
> csp-blocked 报法漂移同面，刀 B 余面拆账入列）；②无锚链碰撞理
> 论面影响面盘点（刀 D 残余，盘点性质）；③encoding 流 M1 进度对
> 账 form 面 46 文件能力缺口（不施工，仅对账边界）；④
> form-submission-0 余下基线 Fail 面单根翻拣（urlencoded2 6F/
> text-plain 6F/multipart 6F/submission-checks 3F/
> input-type-change 2F））。
> **▶ 上轮状态（2026-10-29 M2 切片 146 施工收口，iframe.src pending 导航门——algorithm t10 文件级 Timeout 收口 + t11 连带落定——其下一步 s147 已由 s147 收口（①刀 B runner 多跑机制勘根，realm 隔离单点收口），见上）**：
> **入场定刀位（候选①归口甄别）**：s145 候选①（accept-charset 18
> 案）**跳过**——form-url-encoding-compat 立项控制面明文「s138 挂
> 账 ④ 归口本 goal M1，25 号流不重复施工」，s145 挂账承袭时未对照
> 该归口声明，本轮修正移出候选池（run-rules §9）；取候选②
> algorithm t10 iframe.src 异步装载面（s144 定谳独立挂账）。失败形
> 态：t10 文件级 Timeout（`pending:2`，挂死阻塞 t11）。
> **勘根（探针五轮二分定谳）**：①挂点非装载链本体——runner 静态
> 兜底 `fs::read` 存在且 `wpt-data/common/dummy.xhtml` 在位、直调
> `__zw_reload_iframe` 装载成功（P0f）、src 属性写入正常（P0g）、
> loadPromise/onload 通道 t1-t9 同链 Pass 旁证（其装载源 = form 提
> 交导航 reload 无门）。②挂点 = set trap 导航门——`iframe.src = …`
> 走 part05 R3069 反射分支尾钩，门条件 `_zwIframeNavigationConnected`
> 对 populateForm 产物恒 false → `__zw_reload_iframe` 永不触发
> （P0a/P0b：drain 后 location 恒 about:blank、load listener 零触
> 发）。③门失明机制——sel 臂依赖 `__zw_contains('html', sel)`，
> 该回调走 `with_query_doc_live_aware(&snap, true, …)`（R102，
> live_ok 硬编码 true）：live doc 与 cached_html 快照**均不重放
> pending structural mutations**（重放是 `with_query_view_doc` 独有
> 语义），同任务 insertAdjacentHTML 产物对 contains 恒不可见（P0e/
> P0i 锚定 sel 返 '0'）→ 已插入 connected 树的 iframe 被误判
> detached。④为什么现在才冒——src setter 门（navigation-compat
> M3 引入）既有消费形态是静态/已物化 iframe；t10 是首个
> 「insertAdjacentHTML 产物 + 同任务连环设 src + loadPromise」三要
> 素齐备语料；t10 语义本体（formdata remove → cannot-navigate 中
> 止）s144 已收口，本轮残面纯为装载触发时序。
> **施工**：part01.js 1 处（+14 行）——`_zwIframeNavigationConnected`
> sel 臂前置 pending 解析节点判定：`_zwPendingParsedForSel(sel)` 签
> 名命中（挂 `_zwSelPendingParent` 槽 = 插入点已定且目标在
> connected 树内）即 true——spec「navigation starts once the element
> has a browsing context through insertion into a connected tree」
> 对 pending 已插入形态成立；常态零开销（helper 无 pending 即早
> 退），签名歧义保守落原 contains 判定，handle 臂（createElement
> 链本地 registry）不在病灶不动。
> **验证**：探针 zz-probe-s146（验后删）修复前五轮二分（装载未触
> 发→门挂→contains 两拍失明）、修复后 P0k/P0l（完整 t10 形态）全
> Pass；定向 form-submission-algorithm **11/11 全绿**（t10 + t11
> 「Submission URL should always have a non-null query part」被挂
> 死阻塞案连带落定）；定向 form-submission-0 全目录 **279P/63F/0T**
> （vs s145 277P/63F/1T），(key,state) 对级 join 位移恰 3 条（文件
> Timeout 行消失 + t10/t11 Pass）零键丢失零坏位移；**全量 join**
> （对 s145 7986 对）：s146 **7070P/851F/98T**（结果行守恒 8019、
> 键对守恒 7986）位移 6 条全归因——主刀 3 条 + 在册 flake 报法 2
> 条（src-then-lazy-load 键 Pass→文件 Timeout，s144/s135/s136 在册
> 负载时序刀 B 邻域；link-multiple media 键本轮未及上报，两轮同为
> 文件级 Timeout 挂起时序键集漂移）零真 P→F。门禁：clippy
> zero-engine --all-targets -D warnings 零告警；shim 13-part 规范
> 序拼接 node --check 过（109667 行）；engine lib **3593/9F**（总
> 数 +1 并行流新测试；名单与 s145 9F 逐字节相同——8 已知 + R3045
> 归口 net-api；details 在册间歇本轮采样回绿）；runner bin
> **258/1F**（trend_csv_check 行 196 归口不变）；零 .rs 改动（fmt
> 无新面）；`grep -a` 核验新 shim 串进 runner 二进制（执行体证据）；
> 验证集对齐 s142/s145 同面先例（无 reftest 义务——行为面非渲染）。
> 账本见
> [evidence/2026-10-29-m2s146-iframe-src-pending-navigation-gate.md](html-semantics-compat/evidence/2026-10-29-m2s146-iframe-src-pending-navigation-gate.md)。
> **范围**：engine src/js_dom_shim/part01.js（1 处）。
> **挂账更新**：t10/t11 收口（本轮主刀）；**候选① accept-charset
> 18 案改记归口 form-url-encoding-compat M1**（该流控制面明文，
> 25 号流移出候选池）；**R102 live_aware(true) 回调簇 pending 失明
> 理论缺口新记**（低优先——同款 live_ok 硬编码回调
> sibling/child_nodes/template_contents/element_exists 等对「同任
> 务 insertAdjacentHTML 后立即查询」有同构失明窗口，既有全量绿面
> 说明语料未踩；踩坑语料再按同型补 pending 臂，不做预防性扩面，
> R3190「查询反映 pending 前」断言语义先例）；刀 B runner 多跑 +
> 首遍挂（本轮再添两例同型报法漂移观察：src-then-lazy-load、
> link-multiple media 键）；承袭不变——C2 input-type-change 2F、
> engine lib 9F 环境面、runner trend_csv_check 行 196（归口
> encoding 流）、「真实再物化」观察项、无锚链碰撞理论面盘点（刀 D
> 残余）；其余承袭清单不变；**待决（须用户拍板）**：css/text 语料
> 资源、green-2x2.png、parser form pointer 缓行（2026-10-23）。
> **下一步**：s147 入场定刀位（候选：①刀 B runner 多跑机制勘根
> ——多跑/首遍挂/键集报法漂移三面收口，含 details flake ≈40% 翻
> 转率强化观察；②无锚链碰撞理论面影响面盘点（刀 D 残余，盘点性
> 质）；③全量基线翻拣下一单根面（63F 目录基线面之外的文件级
> Timeout 族——csp-blocked 报法漂移同面）；④encoding 流 M1 进度
> 对账 form 面 46 文件能力缺口（不施工，仅对账边界））。
> **▶ 上轮状态（2026-10-29 M2 切片 145 施工收口，刀 D 祖先锚定结构路径——selector 键跨代碰撞根除，requestsubmit 20 键全翻 + 零 P→F）**：
> **入场定刀位**：s144 下一步候选①（刀 D selector 键元素状态跨代
> 泄漏——runTest2 16 键 + runTest 4 键 Fail，T1/T5/T6 探针已单根
> 定谳：parse 产物 sel-identity 按碰撞性 selector 字符串键存状态，
> 跨 populateForm 代际泄漏）。
> **勘根（下钻三层）**：①碰撞不止脏值 store，是元素身份本身——
> `_elKey`（part03.js:10995）对无 handle 元素落裸 selector 串，
> `_proxyCache`（part01.js:63，P1b S2）按 `_elKey` 缓存 proxy →
> 同串两代元素**共享同一 proxy 对象**（listener store/expando/
> `===` 全部跨代串；s144 所判「双通道」实为三重：`_inputValues`/
> `_inputValuesSet`、宿主 lw pending 属性队列（`__zw_get_attr_lw`
> 扫 selector-keyed 共享 mutation 队列）、listener store 累积闭
> 包）。②碰撞形态收窄——s142 已锚 form[target]（frameName 计数
> 器逐代唯一）→ form/iframe 不碰撞，碰撞面 = form 内歧义控件的
> 结构路径 `html > body > form:nth-child(2) > input:nth-child(1)`：
> afterbegin 连环插入使每代新 form 恰落 nth-child(2)，各代 issue
> 时刻算出的路径串**字节相同**，绝对位置路径只在单一时刻唯一。
> ③方案三选一取**祖先锚定**——锚（id/class/name/target）随节点
> 移动，锚内 `:nth-child` 段只在锚子树内定位，零新增基建单函数
> 收口；epoch 视图戳过度失效（read-after-unrelated-insert 断
> `.value=` read-back——`SetFormValue` 在 applied view no-op，宿
> 主树不存值）、R100 持久 NodeId 见证（需新宿主绑定 + slotmap
> 复用 removal 失效，apply 核心手术架构远期项）均不取；裸 tag 锚
> 排除（body 唯一但携带节点身份为零，对 afterbegin 位移与
> `html > body` 同层暴露，排除后形态变化面收敛到真锚子树）。
> **施工**：js_dom_bridge.rs 3 处——`unique_attr_anchor` 提取
> （s142 name/target 锚块成函数，self/祖先两用）+
> `unique_selector_for_node` 重排纯重构（stable → 属性锚 → 结构
> 路径，语义不变）+ `structural_path_selector` 祖先锚定
> （`ancestor_anchor_selector` 新增：`#id`/`tag.class` 唯一匹配门
> 裸 tag 排除 + name/target 锚；自 node 上行最近身份锚祖先落根
> `anchor > tag:nth-child(pos) > …`，无锚链回落旧 root 形态；
> fastpath 不认领锚定首段 `split_once` 失败安全回落通用引擎——
> 锚定路径天然短解析税可忽略，扩展暂缓 OPTIMIZATION 注释在位）；
> part39.rs 2 处——t8b 主钉断言域收窄（引擎命中恒断言，fastpath
> 仅对其认领形态断言 + 认领覆盖 ≥6 下限，无锚链 root 形态保持）
> + 刀 D 回归钉
> `structural_path_anchored_distinct_across_afterbegin_generations`
> （两代 populateForm 形态：逐代互异 + 锚定落根 + 各自恒解析回本
> 代节点，InsertAdjacentHtml 单条保序）。
> **验证**：宿主 structural-path 单测 6/6（含新钉，t8b 其余 3 钉
> 不动全绿）；定向 form-submission-0 全目录 **277P/63F/1T**（vs
> s144 目录 256P/83F/1T——requestsubmit **20F→0F（36P 全 Pass）**，
> preventdefault 5P 保持，其余文件零位移（form-submit-action 40F/
> urlencoded2 6F/text-plain 6F/multipart 6F/submission-checks 3F/
> input-type-change 2F = 已知基线面），唯一 Timeout = 已知 t10）；
> **全量 join**（对 s144 最终树 /tmp/zw-s144-full4.log 7981 行 =
> 7010P/872F/99T）：s145 **7069P/852F/98T**（8019 行），(key,state)
> 状态集 join **状态位移恰 20 键全 Fail→Pass、零 P→F、全落
> form-double-submit-requestsubmit.html**（runTest 4 + runTest2 16
> ——刀 D 靶面精确闭合）；key-diff 40 = select-event 39（s144 基
> 线文件级 Timeout flake 负偏差恢复，−38P/+39P + Timeout −1 归因
> 闭合）+ algorithm 1（文件级 Timeout 下键集报法漂移，t10 同面）；
> 行数守恒 7981+39−1=8019 精确闭合。
> **details 族 flake 强化观察（归因实验定谳非本修引入）**：
> test_details_name_exclusivity_template_deep_entry 单测隔离跑翻
> 转（`2:1,0` 期望得 `2:0,0`）——祖先锚定**短路（全关）与开启各
> 10 次采样，失败率相同（4F/6P 两臂）**，与本轮改动零相关（沙箱
> 内异步调度非确定性，R5254 resolver 时序邻域，刀 B 在册）；本轮
> 观测翻转率 ≈40% 负载敏感面，复现归口刀 B 勘根时一并收口。
> 门禁：clippy zero-engine --all-targets -D warnings 零告警；fmt
> 本轮编辑区（js_dom_bridge.rs 618-720/part39）零波及，7 处 diff
> stash 对照 HEAD 同现（4248-4559 net-api 区预存漂移归口）；
> engine lib **3592/10F**（9 名单 = 8 已知 + R3045 归口 net-api 不
> 变；第 10 = details 在册间歇）；runner bin **258/1F**（trend_csv
> _check 行 196，并行流 f62bf89aa 归口不变）；shim 未动无 node
> --check 项；验证集对齐 s142 同面先例（定向 + 全量 join + lib +
> runner bin，无 reftest 义务）。
> 账本见
> [evidence/2026-10-29-m2s145-ancestor-anchored-structural-path.md](html-semantics-compat/evidence/2026-10-29-m2s145-ancestor-anchored-structural-path.md)。
> **范围**：engine src/js_dom_bridge.rs（3 处）+ src/js_dom_bridge
> _tests/part39.rs（2 处）。
> **挂账更新**：刀 D 收口（本轮主刀，20 键全翻）；残余理论面 =
> **无身份锚祖先链**（多层匿名 div 包裹 + afterbegin 连环插入）
> 仍有跨代碰撞可能——本目录语料已无此形态，影响面盘点挂 s146 候
> 选④；真节点身份注册制（持久 handle 化，R100 persistent_nodes +
> slotmap removal 失效）保留架构远期项；承袭不变——accept-charset
> w1252 + `&#NNNN;` 实体回退 + lone surrogate USV 18 案（候选②）、
> algorithm t10 iframe.src 异步装载（候选③）、刀 B runner 多跑 +
> 首遍挂（含本轮 details flake 强化观察）、C2 input-type-change
> url/email 2F、engine lib 9F 环境面、runner trend_csv_check 行
> 196（归口 encoding 流）、「真实再物化」观察项；其余承袭清单不
> 变；**待决（须用户拍板）**：css/text 语料资源、green-2x2.png、
> parser form pointer 缓行（2026-10-23）。
> **下一步**：s146 入场定刀位（候选：①accept-charset w1252 +
> `&#NNNN;` 实体回退 + lone surrogate USV 18 案（面单在案，
> urlencoded2 6F 同根）；②algorithm t10 iframe.src 异步装载面；
> ③刀 B runner 多跑机制勘根（含 details flake 强化观察）；④无锚
> 链碰撞理论面影响面盘点（刀 D 残余））。
> **▶ 上轮状态（2026-10-29 M2 切片 144 施工收口，刀 C re-entrant submission hang 收口——form.submit() 放行 + cannot-navigate 中止）**：
> **入场定刀位**：s143 下一步候选①（刀 C re-entrant submission
> hang——runTest2 首案 loadPromise 挂死 → 文件级 Timeout + 24 键未
> 上报；preventdefault 键 1 Fail + 键 2 Timeout）。
> **勘根**：spec 4.10.22.3 原文（web reader 取得）钉死步 5 整段门控
> 「submitted from submit() method = false」——submit() 跳过 firing
> flag 检查/置位、交互校验、submit 事件、事后复查，直达 entry list
> 构造 + 步 9 cannot-navigate 复查（原文「Cannot navigate is run
> again as dispatching the formdata event …」）+ 导航；语料锚点
> runTest2 期望值矩阵（handler 内 submit() 必须产生导航）+ prevent
> default 键 2 + algorithm t10 名字钉死检查位次。m3a `_zwSubmitBusy`
> 短路为 spec 逆语义，且宿主 Enter 通道本就不设（m3b）——去守卫 =
> 消除 JS/宿主双通道不一致。C3「第二次提交 PD 未生效」假说被探针
> T2/T3 证伪（click 默认动作 PD 生效、内层 requestSubmit 被挡），
> 4 键实归 selector 键泄漏根。
> **施工**（part04.js 3 处）：①form.submit() 去 `_zwSubmitBusy` 短
> 路 + 注释重写（m3b 宿主通道 FIXME 保留）；②`_zwFormSubmitNavigate`
> 函数顶 cannot-navigate（步 1/5.9；完整定义 FIXME，connectedness
> 近似）；③formdata finally 后 dialog 分支前 cannot-navigate 复查
>（步 9 原位）。
> **验证**：目录定向 runTest2 从 1T+24 未上报 → 16P/20F 全落定、
> preventdefault 双键 Pass、algorithm 残留 1T（t10 iframe.src 异步
> 装载面独立挂账）；提交前 rebase 并入并行流 6 提交（EventTarget
> null listener shim 面 part03/04/06 + layout R5324 + encoding 流
> goal 脚本，含真实代码面）后门禁全数合并树重出（s142/s143 先例）：
> 全量 join 对 s143 基线（7994 行 = 7038P/857F/99T）**(key,state)
> 状态集 7534/7534 零键丢失、位移恰 3 键全改善、零 P→F**，解 hang
> 新可见 23 键（7 新 Pass + 16 新 Fail），行数守恒 7994−3+24=8018
>（**7048P/872F/98T**；csp-blocked 文件级键 NotRun→Timeout 报法漂移
> 1 = 刀 B 时序方差非位移；rebase 前 pre-merge 树跑 runTest2 24 键
> 逐键相同）。门禁：clippy 三 crate 零告警 + engine lib 3592/9F（9
> 名单 = 8 已知 + R3045 归口 net-api，details 族 flake 本轮回绿，总
> 数 3601）+ runner bin 257/1F（trend_csv_check 行 196 列错位 = 并入
> f62bf89aa encoding 流 CSV 编辑引入，归口该流）+ fmt 两文件均并行
> 流区域（js_dom_bridge 7 net-api 区 + testharness 3）+ shim 拼接
> node --check；探针验后删语料零改动。过程注记：合并树首轮全量遭
> test-guard 瞬态误杀（runner 横幅后零测试行即报 4.31GB>4GB，重跑
> 原样通过——与 s143 陈旧二进制误杀不同因，复现再勘）。二次 rebase
> 复验（推送时远端再前移：net-api M2-S11 CORS 收口触碰 part01
> fetch 面 + rustls bump）：再变基后 join 排除 flake 文件位移仍恰
> 3 键全改善零 P→F、runTest2 16F+8P 与 preventdefault 双 Pass 逐键
> 复现；本轮全量 select-event.html 文件级 Timeout 1 次（−38P）定向
> 复跑 270/270 全绿 = 负载时序 flake（刀 B 邻域）非回归。
> **新泄漏面定谳（刀 D 立项）**：16+4 键 Fail 单根 = selector 键
> 元素状态跨代泄漏——探针链实证 `.value` 写入（T1）与 setAttribute
> overlay（T5）跨 populateForm 代际泄漏、T6 干净页复刻无 v4（真实
> 文件 v4 全部来自 runTest 前序键写入）；机制 = insertAdjacentHTML
> 产物 sel-identity（无 handle）落 selector 字符串键 + body
> afterbegin 连环插入前后代同 selector 字符串（s142 属性锚只覆盖
> iframe[name]/form[target]）；修复归 stable_selector 注册制架构
> 面，独立立项勘根。
> 账本见
> [evidence/2026-10-29-m2s144-form-submit-unblock-and-not-connected-abort.md](html-semantics-compat/evidence/2026-10-29-m2s144-form-submit-unblock-and-not-connected-abort.md)。
> **范围**：engine js_dom_shim/part04.js（3 处）。
> **挂账更新**：刀 D 新立（selector 键状态泄漏，探针证据在案）；刀
> C 改记 hang 已收口（残余 runTest2 16F + runTest 4F 移交刀 D；
> algorithm t10 残留 1T = iframe.src 异步装载面独立挂账）；C2
> input-type-change url/email 2F（不变，基线既有独立面）；刀 B
> runner 多跑（不变）；accept-charset w1252 + `&#NNNN;` 实体回退 +
> lone surrogate USV 18 案（不变）；「真实再物化」观察项（不变）；
> engine lib 9F 环境面（合并树 9 名单 = 8 已知 + R3045 归口
> net-api 流；details 族 flake 在册间歇 pre-merge 现合并树绿）；
> runner trend_csv_check 行 196 列错位新面（并行流 f62bf89aa 引入，
> 归口该流）；其余承袭清单不变；**待决（须用户拍板）**：
> css/text 语料资源、green-2x2.png、parser form pointer 缓行
>（2026-10-23）。
> **下一步**：s145 入场定刀位（候选：①刀 D selector 键状态泄漏面
> 勘根——16+4 键，影响面可能超本目录；②accept-charset w1252 + 实
> 体回退 + lone surrogate USV 18 案；③algorithm t10 iframe.src 异
> 步装载面；④刀 B runner 多跑机制勘根）。
> **▶ 上轮状态（2026-10-29 M2 切片 143 施工收口，查询视图已应用前缀重放双计修复——implicit-submission 提交按钮键翻红）**：
> **入场定刀位**：s142 挂账刀 A′ 候选①（implicit-submission
> optional「Submit event with a submit button」1 键 Pass→Fail 勘根
> 收口）。勘根推翻 s142b「片段真实再物化」归因，钉死查询视图重放
> 双计并收口。
> **勘根**（探针页 zz-probe-s143 v25-v27 + host 双侧日志，验后删）：
> 初判 flatten 鬼影（pending 表永久滞留）与次判刀 A′（overlay 滞留
> 窗口）均实证为真但修复后该键不翻，因果解耦（两修复保留）；v25-v27
> 钉死真根因——提交 handler 执行中查询干净/导航后查询出双份的精确
> 时间窗 + 基座 cached_html 恒单份（REFRESH dump）+ 双 wrapper 均位
> 置选择器（nth-child(2)/(4)，后者基座不存在）+ `__zw_query_all` 返
> 并集 = 视图内插入应用两次；源码四件套闭环：with_query_view_doc 重
> 放区间 `[0..count)` + 共享队列 append-only（R100）+ R55/R348 注册
> 站换新 Arc 已含已应用插入 + id 去重只覆盖带 id 片段（targetted-form
> 的 iframe+form 无 id）——已应用 InsertAdjacentHtml 视图内再插一次
> → form.submitButton 2 元素集合 vs event.submitter 单元素翻红
>（s142 属性锚使 submitter 语义正确化，基线双向一致错误被揭穿）。
> 触发面拓扑：execute_dom_script 私有队列 → R4400 回灌 + 游标推进 →
> R348 重绑共享队列 + 新 Arc → 重绑后异步续延查询触雷；executor 优
> 先通道每批清空天然正确；dispatch_dom_event_inner 每次新 Arc 同理。
> **施工**（4 文件 +129/−11）：①callbacks.rs `MUT_APPLIED_CURSOR`
> thread_local + `publish_mut_applied_cursor()`（applied_mutations 桥
> 侧镜像）——重放区间/live_ok 扫描区间收窄 `[applied..count)`、链步
> 不变用 prev_count（同 Arc 同游标纪元）、重建起点 applied、
> `min(count)` 钳制；②webview.rs 4 发布点（三共享队列注册站 + 导航
> 复位 0）；③user_actions.rs 3 发布点（executor 清陈值 0/私有队列
> 0/R348 重绑 applied）；④part05.js flatten 鬼影随根消退
>（`_zwMoFlatRoot` 根印 + bump 随根作废）+ 刀 A′ script 边界幂等
> bump 拼头——后两刀保留，注释改记不承担该键。语义后果两条：
> live_ok 收窄后 apply 后查询更多直读 live（R102 设计对齐）；subset
> apply Err 丢弃批不再被视图复活（pump 错误路径设计对齐）。
> **验证**：定向 implicit-submission.optional 3/3 Pass；目录 A/B
> +3P 零坏位移；全量 join 对 s142 fixed（7994 行 = 7037P/858F/99T）
> `(key,state)` 对级恰 1 键 Fail→Pass = 目标键、零 P→F、零键增减、
> 行数守恒 7994 = 7994（**7038P/857F/99T**，ΔP=+1/ΔF=−1 精确闭合；
> form-submit-action 10 键 {Fail,Pass} 双侧同形多跑 flake +
> execution-timing/149 子测试名嵌 TAB 为分类假象非位移；s142 记载
> 26 base-only + 2 Fail→Timeout 本轮零重现；**rebase 前后两跑逐字节
> 相同，并行流未动 html/semantics 面**）。提交前 rebase 并入并行流
> 5 提交（net-api M2-S6/S8/S9/S10 + layout ×2，含真实代码面）后门禁
> 全数合并树重出：engine lib 3591/9F（8 名单与 s140/s142 基线逐字节
> 相同；第 9 位轮换 details flake 通过/新面孔
> test_request_signal_passthrough_r3045 = net-api M2-S8 破 R3045 契
> 约测试，delta=并行流归口该流）+ runner bin 258/0F（+2 并行流新测
> 试，trend_csv_check 被并行流收口）+ clippy 三 crate 零告警 + fmt
> 本切片 4 文件零 diff（js_dom_bridge.rs 7 处 + testharness.rs 3 处
> 均并行流区域不重排）+ shim 拼接 node --check；探针验后删语料零改
> 动。过程注记：①首轮全量误用删插桩前陈旧二进制被 test-guard 杀
>（ZWDBG 56 万行超 4GB），重建后重跑——无人值守跑先验二进制新鲜度；
> ②rebase 带真实代码变更时全部门禁在合并树重出（s142 先例）。
> 账本见
> [evidence/2026-10-29-m2s143-query-view-applied-prefix-replay.md](html-semantics-compat/evidence/2026-10-29-m2s143-query-view-applied-prefix-replay.md)。
> **范围**：engine js_dom_bridge/callbacks.rs + js_dom_shim/part05.js
> + webview webview.rs + webview/user_actions.rs。
> **归因修正**：s142b「片段真实再物化」重新定性——submitButton 键
> 无须真实再物化即可完整解释（视图双计，基座恒单份）；v3
> body.innerHTML 双副本是否含独立真实再物化成分待鬼影修复后复核
>（观察项）。
> **挂账更新**：刀 A′ 改记（submitButton 键已收口非该刀；已施工部
> 分为 shim overlay 滞留面/记账正确性收口；「真实再物化」降观察项，
> crash→hang 递进面归刀 C 邻域）；刀 C re-entrant submission hang
>（不变）；刀 B runner 多跑+首遍挂（不变，本轮再实证）；accept-
> charset w1252 + `&#NNNN;` 实体回退 + lone surrogate USV 18 案
>（不变）；engine lib 9F 环境面（8 名单逐字节重验不变；新面孔
> R3045 契约测试归口 net-api 流勘根）；其余承袭清单
> 不变；**待决（须用户拍板）**：css/text 语料资源、green-2x2.png、
> parser form pointer 缓行（2026-10-23）。
> **下一步**：s144 入场定刀位（候选：①刀 C re-entrant submission
> hang 语义收口——firing submission events 重入防护 spec 步骤；
> ②刀 B runner 多跑机制勘根；③accept-charset w1252 + 实体回退 +
> lone surrogate USV 18 案；④「真实再物化」观察项复核——鬼影修复
> 后 v3 形态）。
> **▶ 上轮状态（2026-10-29 M2 切片 142 施工收口，name/target 属性锚 + pending 签名 +target——targetted-form 族 21 键翻红 + 17 键新上报，1 键揭片段双重物化旧疾——其勘根「片段真实再物化」已由 s143 修正为查询视图重放双计，该键已收口，见上）**：
> **入场定刀位**：s141 挂账刀 A（apply 滞后一拍/sel 换代失效）
> 候选①。施工两处：host 侧选择器总漏斗加属性锚 + shim pending
> 签名扩 target——把「无 id/class/name 的 form / 有 name 的
> iframe」从位置锚（`:nth-child` 绝对位置，afterbegin 连环插入
> 即漂移）换成属性身份锚（位置漂移免疫）。
> **施工**（2 文件）：①js_dom_bridge.rs `unique_selector_for_node`
> 结构路径前插属性锚臂——name/target 属性值域限 `[A-Za-z0-9_-]`
>（无转义面），qSA 验唯一才采用；iframe[name=form-test-target-N]/
> form[target=…] 获稳定身份；无 name/target 节点行为同旧零额外
> 宿主查询。②part03 `_zwPendingParsedForSel` 签名
> `tag|id|name|class` → `+target`（节点侧 `attrs.target`、sel 侧
> `__zw_get_attr_lw(sel,'target')`）——populateForm 的 form 无
> id/class/name 全同签 `FORM|||`，多 form pending 歧义 hits>1
> 不归一→落 pending 失明 live 臂→`form.previousSibling` 崩 null
>（s141 主根因之一）；target=frameName 每次唯一，签名歧义消解。
> **验证**：定向 A/B（stash 对照）gains 36P+3F / losses 全为
> crash→hang 递进+多跑波动；全量 join（真基线 8001 行
> =7000P/904F/97T 与 s141 在册吻合；fixed 7994 行=7037P/858F/99T）
> **key 级 state-set 分类**：21 键 Fail→Pass（double-submit-
> default-action 4/requestsubmit runTest 7/preventdefault-click 2/
> input-type-change-submit 1/request-submit-activation 1/form-
> submission-algorithm 5——全集中 form-submission-0 targetted-form
> 族）+ 17 键新上报 Pass（base 早期崩溃未上报）vs 2 键
> Fail→Timeout + 26 键 base-only（26F+1T 先前已在失败，fixed
> 文件级 hang 不再上报——crash→hang 递进记账非回归）+ 1 键
> Pass→Fail（implicit-submission optional，见勘根）。engine lib
>（rebase 后合并树）3588/9F 失败名单与 stash 基线逐字节相同（全
> 环境面；s141 在册 8F→9F = 轮间环境漂移两侧同现非本流引入）+
> runner bin 256/1F（trend_csv_check 归口不变）+ clippy 零告警
>（rebase 撞并行流 4f2230a6d url-setter 落地区域 4 处 -D warnings
> 阻断，顺手机械修复 unwrap_or_default/collapsible_if×2/
> is_some_and 并归口）+ fmt 本切片区域零 diff（上游区域 9 处
> fmt diff 归口该流，未顺手重排）+ shim 拼接 node --check；探针
> 验后删语料零改动。账本见
> [evidence/2026-10-29-m2s142-attr-anchor-and-pending-target-signature.md](html-semantics-compat/evidence/2026-10-29-m2s142-attr-anchor-and-pending-target-signature.md)。
> **范围**：engine js_dom_bridge.rs + js_dom_shim/part03.js。
> **勘根**（s142b 探针 4 版 5 跑 + stash 对照，验后删）：
> implicit-submission 1 键 Pass→Fail 定谳——提交导航触发
> (iframe+form) pending 解析片段在 body **真实再物化两份逐字副本**
>（同名同 target，基线同现=既有旧疾非本修引入）；基线过测系双向
> 一致错误（命名作用域按结构路径钉单位置→单元素=submitter 单元素
> 相等），本修 event.submitter 语义正确化 + 属性锚在复制树失唯一
>（逐字副本共享全部属性，任何属性方案不可分辨；位置方案则漂移
> 别名）→ form.submitButton 2 元素集合 vs 单元素 submitter 不等
> 翻红。正确修法=消灭双重物化（frame 导航/apply 基建），选择器
> 面只是症状。v4 时线旁证：populateForm 后 qSA('form') 瞬时双计
>（pending 窗口 JS 视图双视，apply bump 后归一）与导航后真实再
> 物化为两个独立现象。
> **挂账更新**：**刀 A′（原刀 A 升格）片段双重物化**——提交导航
> 触发 pending 片段 body 再物化（勘根入口：`_zwFormSubmitNavigate`
> 目标解析→frame 导航物化路径→`__zw_apply_generation_bump` 清除
> 时序；预期 implicit-submission 1 键回归 Pass + crash→hang 递进
> 面 sel 消费失配同步收敛）；**刀 C（从刀 A 分立）re-entrant
> submission hang**——剩余 hang 全为 submit handler 内 re-entrant
> requestSubmit/submit 形态（runTest2 矩阵 + form-submission-
> algorithm 第 9 子测），sel 面修复后仍需独立语义收口（firing
> submission events 重入防护）；承袭：刀 B runner 多跑+首遍挂；
> accept-charset w1252 + 实体回退 + lone surrogate USV（候选②，
> 18 案）；engine lib 9F 环境面（本轮逐字节重验）；其余承袭清单
> 不变；**待决（须用户拍板）**：css/text 语料资源、green-2x2.png、
> parser form pointer 缓行（2026-10-23）。
> **下一步**：s143 入场定刀位（候选：①刀 A′ 片段双重物化勘根
> 收口——提交导航触发 pending 片段再物化（v3 探针实证基线同现，
> 勘根入口 `_zwFormSubmitNavigate`→frame 导航物化→apply bump
> 时序；预期 implicit-submission 回归 + crash→hang 递进面收敛）；
> ②刀 C re-entrant submission hang 语义收口（firing submission
> events 重入防护 spec 步骤）；③刀 B runner 多跑机制勘根；
> ④accept-charset w1252 + 实体回退 + lone surrogate USV 18 案）。
> **▶ 上轮状态（2026-10-29 M2 切片 141 施工收口，targetted-form 族勘根轮——三根因定谳 + 空串 formaction 小修 2 键翻红，主干根因挂账三刀——其下一步 s142 已由 s142 收口（属性锚 + pending 签名 target），见上）**：
> **入场定刀位**：控制面 s141 候选①：targetted-form 族 58F
> 深层面勘根（double-submit 防护族）。本轮勘根轮：探针 14 跑
> 钉死三层根因，施工仅收确定性小修（空串 formaction），主干
> 根因挂账独立刀。
> **勘根**（探针页 zz-probe-prevsib/zz-probe-ds 验后删，FILTER
> 定向 14 轮）：①**兄弟导航 apply 代际缺口**——insertAdjacentHTML
> 注入 iframe+form 静态读 previousSibling 走 R3254-K3 pending 臂
> 正常；提交触发 `__zw_apply_generation_bump`（part05）定点清
> pending 解析节点后 JS proxy 持有的 stale sel 落 live 臂
> `__zw_sibling_nodes`/`_childNodeList` 恒 miss→null——语料
> `loadPromise(form.previousSibling)` 崩 Cannot set properties
> of null（波及 form-submission-algorithm 8F + requestsubmit
> onload 形态 ~20 键）；五通道旁证（文档未导航/iframe named
> 查询/innerHTML 序列化/children 枚举/form proxy 身份）都在，
> 缺口精确在 sel 换代失效非树损。②**apply 滞后一拍 + runner
> 同文件多跑**——form-submit-action 单文件 runner 执行 3 遍
>（30 行=10 案×3），遍 1/2 全挂 gEBI click not function、遍 3
> 才 8P/2F；语料本体减半单案冷启动行为完全正确（formdata 全序
> dump：entry 构造在 submit 事件后、A 的 IDL set 落 attr、B 的
> preventDefault 拦第二次提交——双提交语义定谳无 bug），原版
> 失败全在案间/遍间状态；统一假说 = 注入节点 host 物化跨
> script 边界 pending→apply→稳定两拍，提交管线 host 通道对
> pending 失明。③**空串 formaction 回落误判**——④ action 解析
> `=== ''` 并入回落分支错回落 form action；spec #concept-fs-
> action：submitter 有 formaction 属性（含空串）即以其为
> action，空串 resolve 即 base=文档地址。
> **施工**（shim 1 文件 1 处）：part04 ④ action 解析回落判定改
> 只按 `== null`（属性缺席），空串保留交 `new URL('', base)`
> 天然落文档地址；FIXME 清单行同步标记已修。
> **验证**：定向 form-submission-0 **210P/114F/1T**（对 s140b
> (key,state) 对级 293→295 对、恰 +2 F→P 零 losses：button/
> input 两案 `no formaction, expect to navigate to document
> URL`；行数 305→325 差 20 = form-submit-action 多跑遍数波动）；
> 全量 join（对 s140 full）**零 losses 恰 +2 gains**——两键新增
> Pass 态（Fail 态并存：多跑前遍挂 click 形态未达 formaction
> 断言，刀 B 面）；行数 7981→8001（+20 = form-submit-action
> 多跑 +2 遍），6998P/886F/97T → 7000P/904F/97T（ΔP=+2、
> ΔF=+18 全为多跑行）。engine lib **3582/8F**（HEAD 基线在册
> 环境面）+ runner bin **256/1F**（trend_csv_check 归口不变）+
> clippy 零告警 + shim 拼接 node --check + fmt runner 过；探针
> 页验后删语料零改动。账本见
> [evidence/2026-10-29-m2s141-formaction-empty-and-apply-lag-probe.md](html-semantics-compat/evidence/2026-10-29-m2s141-formaction-empty-and-apply-lag-probe.md)。
> **范围**：engine shim part04.js。
> **挂账更新**（targetted-form 族三立刀）：**刀 A** apply 滞后
> 一拍/sel 换代失效（js-dom 节点视图基建：apply 清 pending 时
> 「host 已进快照」前提对 sel 消费方不成立；候选修法 = 旧 sel
> →新位置签名映射或 live 臂 stale sel 回退重定位——无 name
> form 签名歧义须树位置/反链方案；预计解锁 form-submission-
> algorithm 8F + requestsubmit onload ~20 键 + form-submit-
> action click 形态 20 键）；**刀 B** runner 同文件多跑 + 首遍
> 挂执行模型（多跑机制本身 + 首遍冷态行为与遍 3 不一致；对账
> 纪律 = 多跑下行数波动以 (key,state) 对级为准）；**刀 C** 双
> 提交语义残余（减半探针定谳单案正确；遍 3 全过后残余案再细
> 勘）；accept-charset w1252 + 实体回退 + lone surrogate USV
>（候选②，18 案）；engine lib js_dom_bridge 8F 环境面（s140
> 挂）；其余承袭不变（override builtins 待决、encoding CSV
> 归口、fmt url_idna 归口、echo 回显体观察、RadioNodeList
> live、past names、expando 双轨、part03 raw 链、engine lib
> details flake、reset.html pointer 缓行、execution-timing、
> lazy-loading 26F、meta color-scheme 21F、写流盖章、part02
> setAttribute、041 nested write、deep clone、css-module 门
>（待决）、www2、module TLA、dynamic import() with、classic
> SRI、143 write、defer 序、接口对象、FIXME 组、/tmp/zw-hb*.txt、
> FileList、live 树 qSA、渲染流让位、s131 插入步、radio 现象⑤、
> s135 限制 3 条）；**待决（须用户拍板）**：css/text 语料资源、
> green-2x2.png、parser form pointer 缓行（2026-10-23）。
> **下一步**：s142 入场定刀位（候选：①刀 A apply 滞后一拍基建
> 修复——script 执行边界/host 查询前强制 flush apply-pending 或
> stale sel 重定位（targetted-form 族大面积翻红主刀）；②刀 B
> runner 多跑机制勘根；③accept-charset w1252 + 实体回退 + lone
> surrogate USV——三文件 18 案收口面；④engine lib js_dom_bridge
> 桥接系列 8F 环境性失败面勘根）。
> **▶ 上轮状态（2026-10-29 M2 切片 140 施工收口，File 条目层 + multipart filename 不归一定谳轮——form-submission-0 三文件 +60 F→P，全量零坏位移——其下一步 s141 已由 s141 勘根收口（①三根因定谳挂账 + 空串 formaction 小修 2 键），见上）**：
> **入场定刀位**：控制面 s140 候选①：File 条目层（file input
> entry 贡献 + input.files 面 + FormData append File 通道 +
> multipart File part）。s139 后三文件各 36P/26F 残余即本刀。
> **spec 核实**（WebFetch 对 spec 域不可达，web-reader 抓取全
> 文）：construct entry list file 状态步对每个 selected file
> 贡献 entry（name, File）、无选中零条目；normalize-newlines
> 步只作用 entry name 与非 File value（File.name 不触）；
> filename 仅 escape a string 逐字节转义（must not perform any
> other escapes——先归一再转义为错）。语料三向核对一致：name
> 归一（`a\rb`→`a%0D%0Ab`）、value 归一不转义（`b\nc`→
> `b\r\nc`）、filename 不归一（`b\nc`→`b%0Ac`、`b\n\rc`→
> `b%0A%0Dc` lone LF 保持）。
> **施工**（shim 1 文件 2 处）：part04 ①file input entry 分支
>（`_c.files` item() 枚举逐 File 经 `_zwFdEntry` 贡献条目，
> File 身份 + filename 第三元与 FormData append 同型）；
> ②⑤a 序列化层字节构建器（替换 s139 三分支）——`_entryStr`
> File→filename 字符串（urlencoded/text-plain/GET query 共用，
> spec convert 步）+ multipart `_mpBytes`/`_mpPushStr` 逐字节
> 构建（part 头 name 经 `_mpEsc` 归一后转义；File 分支 CD
> filename **裸转义不归一** + `Content-Type: file.type||
> 'application/octet-stream'` + `_zw_blobBytes` 原始字节；
> `_zwEncodeBytesPrefix` `__zw_bytes:` csv-decimal wire 传
> host，s138/R3020 通道复用）。
> **验证**：定向 form-submission-0 **208P/96F/1T**（对 s139
> 148P/156F/1T，60 F→P——(key,state) 对级 comm 键集合 diff
> IDENTICAL、losses 全 Fail/gains 全 Pass；两段施工 54+6：⑤a
> 后 text-plain +20/urlencoded2 +20/multipart +14，filename 裸
> 转义 fix 后 multipart filename 换行 6 案补齐）；全量 join（对
> s139 full）恰 **60 F→P 全部为三文件案（multipart/text-plain/
> urlencoded2 各 20）、键零消失零新增零其他位移**；行数守恒
> 精确闭合（6938P/946F/97T → 6998P/886F/97T，ΔP=+60/ΔF=−60/
> ΔT=0，7981 行不变）。engine lib **3581-3582/8-9F**（stash
> 对照实验定谳 HEAD 基线同样 8F——js_dom_bridge fetch/service
> worker/request 桥接系列环境性失败面，非本切片引入，新挂账
> 勘根）+ runner bin **256/1F**（trend_csv_check encoding 行
> 归口不变）+ clippy（zero-engine + zero-wpt-runner）零告警 +
> fmt runner 过 + shim 拼接 node --check 过；语料零改动；无探
> 针页。账本见
> [evidence/2026-10-29-m2s140-file-entry-serialization.md](html-semantics-compat/evidence/2026-10-29-m2s140-file-entry-serialization.md)。
> **范围**：engine shim part04.js。
> **挂账更新**：**form-submission-0 残余**（三文件 88F/1T）：
> targetted-form 族 58 新见 F 深层面勘根（候选②，double-submit
> 防护族——form-double-submit-requestsubmit 35F/form-submit-
> action 22F 为主）；accept-charset w1252 + `&#NNNN;` 实体回退
> + lone surrogate USV（候选③，6 案/文件含 multipart
> `character not in encoding in filename`）；**新挂账**：
> engine lib js_dom_bridge fetch/service worker/request 桥接
> 系列环境性失败面（HEAD 基线 stash 对照 8F 稳定复现
> 2026-10-29——fetch_sync_wire/cross_origin_origin/
> forbidden_headers/passes_credentials/iframe_service_worker_
> messages/headers_guard_r3223/body_readers_r2982/
> response_request_constructors + details flake 间歇第 9；与
> s139 记录 3590/0 不符，失败形态 JS 求值 undefined vs 期望
> wire 串）；其余承袭不变（override builtins 待决、encoding
> CSV 归口、fmt url_idna 归口、echo 回显体观察、RadioNodeList
> live、past names、expando 双轨、part03 raw 链、engine lib
> details flake、reset.html pointer 缓行、execution-timing、
> lazy-loading 26F、meta color-scheme 21F、写流盖章、part02
> setAttribute、041 nested write、deep clone、css-module 门
>（待决）、www2、module TLA、dynamic import() with、classic
> SRI、143 write、defer 序、接口对象、FIXME 组、/tmp/zw-hb*.txt、
> FileList、live 树 qSA、渲染流让位、s131 插入步、radio 现象⑤、
> s135 限制 3 条）；**待决（须用户拍板）**：css/text 语料资源、
> green-2x2.png、parser form pointer 缓行（2026-10-23）。
> **下一步**：s141 入场定刀位（候选：①targetted-form 族 58F
> 深层面勘根——double-submit 防护族（form-double-submit-
> requestsubmit 35F/form-submit-action 22F 为主，文件级
> Timeout 1 个 form-double-submit-input-type-change）；②accept-
> charset w1252 + `&#NNNN;` 实体回退 + lone surrogate USV——
> 三文件 6 案/文件共 18 案收口面；③engine lib js_dom_bridge
> 桥接系列 8F 环境性失败面勘根——恢复质量门禁基线）——①已由
> s141 勘根收口（三根因定谳挂账三刀 + 空串 formaction 小修
> 2 键），见上。
> **▶ 上轮状态（2026-10-29 M2 切片 139 施工收口，text/plain + multipart 编码器轮——form-submission-0 三文件镜像 36P/26F（72 F→P），全量零坏位移——其下一步 s140 已由 s140 收口（①），见上）**：
> **入场定刀位**：控制面 s139 候选①：text/plain +
> multipart/formdata 编码器（s138 主干后两文件仍 0P/62F 全 F）。
> **spec 核实**（WebFetch 对 spec 域不可达，换 web-reader 抓取
> 全文）：convert 步（File 值取 filename + lone CR/LF→CRLF 归一）
> 为 urlencoded/text/plain 共用；text/plain `name=value\r\n` 逐对
> 零转义；multipart escape a string 仅 LF/CR/引号、非 File value
> 不转义、boundary 不得出现在任何 name/value——语料
> `expected.name` 为转义后形态（`a\nb`→`a%0D%0Ab`），开场担心的
> 转义/字面冲突不存在。
> **施工**（shim 1 + runner 1）：part04 enctype 读提升 body 构建
> 前 + `_normNL`（lone CR/LF→CRLF，三序列化共用）+ POST body 分型
>（text/plain 零转义逐对；multipart boundary 固定前缀+随机 24 字
> 符=47 字符<70 上限、撞串重掷 8 次兜底 + part 头 escape a
> string + value 不转义 + `--B--\r\n`）+ `_urlBody` 另存（GET
> query 恒 urlencoded 与 enctype 无关）+ ⑤b headersWire
> multipart 值带 `; boundary=`；runner echo-content-escaped 端点
> CR/LF 改字面交付（0x0D/0x0A 不转义——s138 `\x0D\x0A` 转义与
> helper `unescape()` 首步 `/\r\n?|\n/g→"\r\n"` 归一不兼容：归一
> 只作用于字面 CR/LF 且先于 `\xNN` 解码）+ 单测重写（helper
> unescape 的 JS 等价实现——首步归一 + `\xNN` 还原——双向校验
> + lone LF 案 `a\nb=c\n`→`a\r\nb=c\r\n` 实证）。
> **验证**：定向 form-submission-0 全目录**三文件各 36P/26F 完全
> 镜像**（text-plain/multipart 各 0P→36P、urlencoded2 36P 不变；
> 残余 26F/文件 = File/filename 层（s140）+ w1252/surrogate
> （候选④）；唯一 Timeout = form-double-submit-input-type-change
> 在册 targetted-form 族）；全量 join（对 s138 full，**(key,state)
> 对级 sort -u 去重后 comm/join**）恰 **72 F→P 全部为两文件字符
> 串案（各 36）、键零消失零新增零其他位移**——同文件在全量日志
> 内可跑多次（form-submit-action 同键同跑 2F+1P），键级 join 对
> 重复键叉乘出 16 假 P→F（对级 comm 证明两日志状态集逐键一致，
> 88−16=72 恰为预期翻红）；行数守恒精确闭合（6866P/1018F/97T →
> 6938P/946F/97T，ΔP=+72/ΔF=−72/ΔT=0）。engine lib **3590/0**；
> runner bin 255/2F（+1F = 端点单测 helper 笔误 `take(2)` 从
> `x` 起收，修复后过；另一 F = trend_csv_check encoding 行归口
> 不变）；clippy（zero-engine + zero-wpt-runner）零告警；fmt
> runner 过（engine 报 url_idna.rs 既有违例归口并行流
> 14dbfb011）；shim 拼接 node --check 过；语料零改动；无探针页。
> 账本见
> [evidence/2026-10-29-m2s139-text-plain-multipart-encoders.md](html-semantics-compat/evidence/2026-10-29-m2s139-text-plain-multipart-encoders.md)。
> **范围**：engine shim part04.js；
> tests/wpt-runner/src/testharness.rs（端点 CR/LF + 单测重写）。
> **挂账更新**：**form-submission-0 残余**（每文件 26F 同构）：
> File 条目层（s140 候选①：file input entry 贡献 +
> input.files/DataTransfer 面 + FormData append File 通道——
> formdata 通道 File 案 got `a=%5Bobject+File%5D`；multipart
> File part 另需 filename/Content-Type/File bytes 面）；
> accept-charset w1252 + 实体回退 + lone surrogate USV（候选②，
> 每文件 4-6 案）；targetted-form 58 新见 F 深层面勘根（候选③）；
> **新挂账**：echo 回显体 HTML 解析面观察（零语料暴露不施工——
> iframe 子文档按 HTML 解析回显体、`&`/`<` 字面若入提交体会被
> 解析面吞，当前三语料仅 w1252 案期望侧含字面 `&`、提交体永不
> 携带，语料扩展时复核）；其余承袭不变（override builtins 待决、
> encoding CSV 归口、fmt url_idna 归口、RadioNodeList live、
> past names、expando 双轨、part03 raw 链、engine lib details
> flake、reset.html pointer 缓行、execution-timing、lazy-loading
> 26F、meta color-scheme 21F、写流盖章、part02 setAttribute、
> 041 nested write、deep clone、css-module 门（待决）、www2、
> module TLA、dynamic import() with、classic SRI、143 write、
> defer 序、接口对象、FIXME 组、/tmp/zw-hb*.txt、FileList、
> live 树 qSA、渲染流让位、s131 插入步、radio 现象⑤、s135
> 限制 3 条）；**待决（须用户拍板）**：css/text 语料资源、
> green-2x2.png、parser form pointer 缓行（2026-10-23）。
> **下一步**：s140 入场定刀位（候选：①File 条目层——file input
> entry 贡献 + input.files setter/DataTransfer 面 + FormData
> append File 通道 + multipart File part（filename/Content-Type/
> File bytes），formdata 通道 File 案可先行（append File→条目
> File 值，三文件 formdata 通道 File 案即翻红面）；②targetted-form
> 58F 深层面勘根——double-submit 防护族；③accept-charset w1252
> + 实体回退 + lone surrogate USV）。
> **▶ 上轮状态（2026-10-29 M2 切片 138 施工收口，form-submission-0 提交主干轮——urlencoded2 62F→26F（36 F→P）+ targetted-form 族文件级 Timeout 解锁，全量零坏位移——其下一步 s139 已由 s139 收口（①），见上）**：
> **入场定刀位**：控制面 s138 候选②：form-submission-0 186F
>（urlencoded2 / text-plain / multipart-formdata 三 .window.js
> 各 62F）构成面勘根。
> **勘根定谳**：**186 案全部 `but got ""` 单一主干**——①POST
> 零投递（`_zwFormSubmitNavigate` 未拦截分支 `if (_isPost)
> return;`，host 导航契约无 method/body 面 M2-S4J documented
> 边界）；②form target 无解析（提交通道只走顶层路径）；
> ③iframe POST 装载通道缺（`_zwLoadIframeEntry` 对 `__zw_fetch`
> 硬编码 'GET' 空 body）；④echo 端点缺（`/FileAPI/file/
> resources/echo-content-escaped.py` wpt-data 无此文件、runner
> fetch handler 无路由）。`""` 来源：helper 在 form.submit() 后
> 挂 iframe.onload，about:blank 初始装载先触发 → 空串 assert
> 失败非 Timeout。**附加层构成**：enctype 编码器（urlencoded
> 已有；text/plain、multipart 缺）；File 条目层（file input
> entry `continue` FIXME 零贡献 + FormData.append String(File)
> = "[object File]" 实证）；accept-charset 层（w1252 + `&#NNNN;`
> 实体回退缺、lone surrogate 缺 USV 转换实测序列化抛错提交
> 中止）。
> **施工**（runner 1 + shim 2）：testharness.rs fetch handler 新
> 路由 echo-content-escaped.py（POST body 逐字节 `\xNN` 转义、
> `\`→`\x5C` 防 unescape 正则误吞、printable ASCII 原样 +
> round-trip 直测）；part01 `_zwLoadIframeEntry` flags.method/
> body/headersWire 透传 `__zw_fetch` 位置 1/3/4（缺省 GET 零
> 变化）；part04 `_zwFormSubmitNavigate` ⑤b target 分支
>（submitter formtarget > form target；命中 named iframe →
> GET query/POST body+enctype CT 经 `__zw_reload_iframe` 复用
> 装载管线、return 不入顶层 navigate；未命中/`_self` 等关键词
> 落既有顶层行为）+ `_encPair` 补编 `! ' ( ) ~`（原注释
> 「除 `*` 与 spec 编码集一致」有误——encodeURIComponent
> unescaped 集含 `!'()~`，spec urlencoded percent-encode set 仅
> 豁免 `*-.0-9A-Z_a-z`）。
> **验证**：定向 urlencoded2 **36P/26F**（26 残余 = 22 File +
> 4 w1252 全挂账层）；全量 join **7914→7981 行配平完全闭合、
> 36 F→P 真翻红、零坏位移**（无 P→F/T→F/P→T）：P
> 6814+36+14=6864 ✓；F 996−36+58=1018 ✓（58 = targetted-form.js
> 族 5 文件级 Timeout 解除后**新见键** 72 个中 58F——`<form
> target="named-iframe">` 提交面与主干同构，先前藏于文件级
> Timeout 非回归）；T 102−5=97 ✓（requestsubmit/default-action/
> preventdefault ×2/request-submit-activation 五文件解除）。
> engine lib **3606/0**；runner bin **256/1F**（+1 = 新端点直
> 测；唯一 F = 既有 trend_csv_check encoding 行归口不变）；
> clippy（zero-engine + zero-wpt-runner）零告警；fmt runner 过
>（engine 报 url_idna.rs 既有违例 = 并行流 14dbfb011 非本轮
> 面，归口该流）；shim 拼接（13 part 按 js_dom_bridge.rs 序）
> node --check 过；语料零改动；无探针页。账本见
> [evidence/2026-10-29-m2s138-form-submission-post-target-iframe.md](html-semantics-compat/evidence/2026-10-29-m2s138-form-submission-post-target-iframe.md)。
> **范围**：engine shim part01.js + part04.js；
> tests/wpt-runner/src/testharness.rs（端点路由 + 直测）。
> **挂账更新**：**form-submission-0 残余构成**（s139 候选刀位
> 按量序）：①text/plain + multipart 编码器（124 案非 File 部
> 分，multipart 含 boundary/binary 面）；②File 条目层（file
> input entry 贡献 + input.files/DataTransfer 面 + FormData
> append File 通道）；③targetted-form 解锁 58 新见 F 深层面
> 勘根（double-submit 防护族为主）；④accept-charset w1252 +
> 实体回退 + lone surrogate USV（4 案）；**新挂账**：fmt
> engine url_idna.rs 既有违例归口并行流；form-submission-0
> 186F 挂账精确化为分层构成；其余承袭不变（override builtins
> 待决、encoding CSV 归口、RadioNodeList live、past names、
> expando 双轨、part03 raw 链、engine lib details flake、
> reset.html pointer 缓行、execution-timing、lazy-loading 26F、
> meta color-scheme 21F、写流盖章、part02 setAttribute、041
> nested write、deep clone、css-module 门（待决）、www2、
> module TLA、dynamic import() with、classic SRI、143 write、
> defer 序、接口对象、FIXME 组、/tmp/zw-hb*.txt、FileList、
> live 树 qSA、渲染流让位、s131 插入步、radio 现象⑤、s135
> 限制 3 条）；**待决（须用户拍板）**：css/text 语料资源、
> green-2x2.png、parser form pointer 缓行（2026-10-23）。
> **下一步**：s139 入场定刀位（候选：①text/plain + multipart
> 编码器——text/plain 近似零成本先行，multipart 需 boundary
> 面；②File 条目层（file input entry + input.files/
> DataTransfer + FormData File 通道）；③targetted-form 58F
> 深层面勘根——double-submit 防护族）——①已由 s139 收口，见上。
> **▶ 上轮状态（2026-10-29 M2 切片 137 施工收口，FORM named properties 语义补全轮——form-nameditem 10F→1F 收口 9 + 全量 8 F→P 真翻红 0——其下一步 s138 已由 s138 收口（②），见上）**：
> **入场定刀位**：控制面 s136 候选②：form-nameditem 残余 10F
> 勘根收口。该文件 17 子测试（10F/7P），覆盖 FORM named
> properties 全语义面（named getter 序 / gOPD 描述符 / expando
> 遮蔽序 / non-configurable invariant / past names map /
> RadioNodeList 形态）。
> **勘根定谳**（10F 六组）：①gOPD 不见 named 属性 ×3（single/
> multiple/expando already-existing）——gOPD trap 只报 expando；
> ②多匹配返首元素非 RadioNodeList ×2（item/indexed）——旧
> R3254-K3 臂 `_formControls` 首命中直返；③expando 优先序缺失
>（added later 案）——R3042 expando 读在 named access 之后；
> ④proxy invariant 违例（non-configurable 案）——get/gOPD trap
> 不报 target non-configurable own 即 V8 TypeError；⑤past names
> map 缺失；⑥「Named elements should override builtins」与
> fix#19c 站点兼容偏离正面冲突（reset 等不被遮蔽有实测回归
> 证据）——**挂待决（须用户拍板）**不施工保持 Fail。
> **spec 序与语义定谳**：named getter 序 = non-configurable
> target own（invariant）→ own expando → named → past；描述符
> {value, writable:false, enumerable:false, configurable:true}
>（[LegacyUnenumerableNamedProperties]）；**past names 条目仅在
> 元素仍是 listed 控件时有效**（保改名身份非 removal——语料
> remove 后读 undefined 双面断言，施工中按语料改正初版理解）；
> named 集合实例稳定化（`gOPD(form,name).value === form[name]`
> 须同一对象——real browser 每 form 每 name 缓存）。
> **施工**（part04 1 处 + part05 4 处，无 Rust 变更）：part04
> get trap FORM gate 内重写五臂序（①invariant 臂 `_t[prop]`
> 直读（accessor 形态安全）→②own expando 臂→③named 臂+
> past 记账→④past 回查臂 **listed 复核**（`_formControls`
> 逐项 `===`）→⑤s119 索引臂保留）；part05 顶层
> `_zwFormNamedLookup`（恰 1→元素；>1→`_zwMakeCollection(false)`
> + per-form per-name 实例缓存 `_zwFormNamedCollCache`（WeakMap
> 键=元素 proxy、成员逐项身份比对不变复用/变更重建、换代清空同
> `__zw_reset_pending_state`）+ `_zwFormPastNames` store——
> **IIFE 顶层声明**（初版误放 get trap 体内每属性访问重建且
> part05 trap 不可见，同 `_zwSubmitBusy` 判例施工中搬家））；
> gOPD trap invariant/named 两臂；has trap named 臂（namedItem
> 案 `"namedItem" in form` 须 false）；set trap else-if 链末端
>（反射 IDL 先行 fix#19c 同侧）named 命中 `return false`；
> **defineProperty/deleteProperty trap 新增**（handler 原无此二
> 键）——defineProperty named 命中 false / 否则落 target
>（non-configurable expando 语义）；deleteProperty 四序（target
> non-configurable false→expando store 真删→named false→target
> ordinary 兜底）。
> **验证**：定向 form-nameditem **16P/1F**（10F 收口 9，唯一
> F = override builtins 挂待决）；九域键级 join——**恰 8 F→P、
> 零位移、零 GONE**（textarea 1 NEW 为 dup-key 副本假阳性两侧
> 各 2 份同 Pass）；全量 join **7914/7914 行守恒、恰 8 Fail→Pass
> 真翻红 0、new non-Pass 0**（6805P/1004F/102T → 6814P/996F/102T；
> 3 NEW——textarea / srcset `\n` 变体 / execution-timing 149——
> 逐一核验均 dup-key 副本假阳性两侧各 2 份同 Pass 净零）。engine
> lib **3584 连续两轮 0F**（间歇 details flake 未复现挂账口径
> 不变）；runner bin **255/1F**（76i）——唯一 F =
> trend_csv_check 契约违约，**定谳并行流 5a5da454b（encoding
> legacy-mb DC-3）对 wpt-suites.csv 写回损坏非本轮引入**（本轮
> 零 Rust 变更）：①头部注释行 2/4 被 csv 模块 quote 包裹（违反
> 文件头第 6 条自明规则）**本轮已机械恢复两行原样**（第一炸点
> 消除）；②encoding-legacy-mb 2026-10-28 行（文件行 209）字段
> 语义错位——rate 位落 `completed`、status 位落 `active`、
> evidence 列被 note 顶替（漏 evidence 路径）——跨流数据行
> 账务内容不越权代写，**归口 encoding 流**（挂账）；clippy 零
> 告警；零 Rust 变更 fmt 不适用；shim 拼接（13 part）node
> --check 过。探针页（identity/corpus-clone 两页）验后删；语料
> 零改动。账本见
> [evidence/2026-10-29-m2s137-form-nameditem-named-properties.md](html-semantics-compat/evidence/2026-10-29-m2s137-form-nameditem-named-properties.md)。
> **范围**：engine shim part04.js + part05.js；顺带修复
> docs/compat/trends/wpt-suites.csv 注释行损坏（独立提交，见上
> runner bin 项①）。
> **挂账更新**：**「Named elements should override builtins」1F
> 挂待决（须用户拍板）**——spec 全覆盖 vs fix#19c 偏离，六
> trap 排除名单同款落地，拍板翻转时改名单即可；**新挂账**：
> encoding-legacy-mb CSV 行错位归口 encoding 流（runner bin 唯一
> F）；RadioNodeList live 语义近似（快照+成员变更重建）；past
> names 条目不显式清理（listed 复核天然过滤，内存面可控）；
> expando store/target 双轨（defineProperty configurable:true
> 值不可见，语料无面 pre-existing 类同）；form-nameditem 10F
> 挂账终结；其余承袭不变（reset.html form element pointer 面
> 待决缓行、part03 raw 链 type 读通道未勘 + reset 臂无还原臂、
> engine lib details flake 独立勘根、empty.html/empty-with-base.html
> Timeout、form-submission-0 186F、execution-timing 构成面、
> lazy-loading 26F、meta color-scheme 21F、写流盖章断链、
> part02 setAttribute 绕 notify、041 nested write、deep clone
> 非 HTML ns、css-module corpus 门（待决）、www2 可达性、
> module-import-referrer TLA、dynamic import() with 透传、
> classic 外链 SRI、143 write、defer 多写完成序、接口对象
> 基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、
> live 树逗号列表 qSA 根缺口、渲染流面让位累积、s131 插入步
> 缺口细化、radio 现象⑤/checked 属性增设/defaultChecked 无
> 独立 corpus 面、s135 已知限制 3 条）；**待决（须用户拍板）**：
> css/text 数据模块语料资源缺失、green-2x2.png 语料资源分发、
> parser form element pointer 4F 缓行（2026-10-23）。
> **下一步**：s138 入场定刀位（候选：①全量基线翻拣下一单根面
> ——域翻拣刀法同款；②form-submission-0 186F 构成面勘根；
> ③part03 raw 链激活 type 读通道同构清查——corpus 无面低
> 优先）——②已由 s138 收口，见上。
> **▶ 上轮状态（2026-10-29 M2 切片 136 施工收口，激活门 type 读通道换 lw 融合读轮——button-events 9/9 全绿收口 + 文件级 Timeout 解除——其下一步 s137 已由 s137 收口，见上）**：
> **入场定刀位**：控制面 s136 候选①：button-events 挂起残案 +
>「type=button shouldn't trigger submit or reset events」submit
> 误派发异根勘根。s134 定谳该 Fail 为「submit 误派发异根非本轮
> reset 臂引入」，同文件级 Timeout 挂起 1（t2 无行）。
> **勘根定谳**：①断言面盘点——t3 Fail = assert_unreached
>（submit 误派发非 reset）；t2「The reset event must be fired」
> **无输出行 = pending**，文件级 Timeout 根因。②事件序列探针
>（断言 events 数组）——p2（type=reset click）收到 `p1:submit`
>（onsubmit 为 p1 残留）即 p2 自己的 click 误走 submit 出口；
> p3（type=button）同根。③回调包装 TRACE 定谳单根——IDL setter
>（btn.type=X）**推 SetAttr mutation 但 R2995 lw 层立即可见**
>（`__zw_get_attr_lw` 读到新值），click 门 sel 分支读纯快照
> `__zw_get_attr` **恒空** → `_subIsBtn` 命中 `BUTTON && ty==''`
>（空 type 默认 submit）→ 恒走 submit 出口；t1/t4 碰巧对（期望
> 恰为 submit）。④handle 分支无坑——`__zw_get_attr_handle`
> 本身 consult 当前批 mutations（latest-wins），修复面仅 sel
> 分支。⑤探针 p5 反常已解释——corpus 同构三连击修复前 Pass 为
> 假象：seq=submit×3 第三次 push 触发 t.done()，step_timeout
> 终断言在 test 完成后执行被 harness 无效化。
> **施工**（part04.js 2 处，无 Rust 变更）：click() API 门（R57）
> 与 R106 dispatchEvent 臂的 type 读取 sel 分支换 R2995 lw 融合
> 读（consult 变更列表 latest-wins → applied view 回落，与
> getAttribute 同语义；`typeof __zw_get_attr_lw === 'function'`
> 守卫照 part01:294 惯例，失败回落纯快照）；handle 分支不动。
> type 读取正确后既有出口判定全继承（`_subIsBtn`/`_subIsReset`
> s134 两臂自动收敛）。
> **验证**：定向 button-events.html **9/9 全绿**（t3 F→P + t2
> 新键 Pass + **文件级 Timeout 行解除**）；域回归键级 join——
> resetting-a-form 14 / the-form-element 108 /
> form-control-infrastructure 121 / constraints 920 /
> the-input-element 1644 / the-select-element 314 /
> the-option-element 108 / the-textarea-element 41 **逐键零
> 位移** + the-button-element 恰 t2 新键 + t3 F→P + 文件
> Timeout gone；全量 A/B × /tmp/s135-full.log 键级 join
> **恰 1 Fail→Pass 真翻红 0、new non-Pass 0**（6805P/1005F/104T
> → 6808P/1004F/102T，行数 7914 守恒；另一对 NEW/GONE =
> the-img-element src-then-lazy-load.html 文件级 Timeout 波动
> **自然回落**——s135 记录的边界波动往返对冲净零非本轮改动面；
> 3 条 NEW 为 dup-key 副本假阳性两份同 Pass）。engine lib
> **3584 用例**（并行流并入 +13 后新基线）默认并行间歇 1F——
> 失败测试 = test_details_name_exclusivity_template_deep_entry
> **与 s135 定谳完全同款**（默认并行 2F/8P + 4 线程 0F/2P +
> 失败名抓取确认），延续挂账非本轮引入；runner bin **256/0**
>（76i）；clippy 零告警；零 Rust 变更 fmt 不适用；shim 拼接
>（13 part）node --check 过。探针页验后删；语料零改动。账本见
> [evidence/2026-10-29-m2s136-button-type-lw-read.md](html-semantics-compat/evidence/2026-10-29-m2s136-button-type-lw-read.md)。
> **范围**：engine shim part04.js。
> **挂账更新**：button-events 文件级 Timeout **解除（挂起清零）
> + t3 收口 + t2 新键**——s134「收窄待续」终结；**新挂账**：
> part03 raw 链（UA 指针激活）type 读通道未勘（proxy 链两臂已
> 修，raw 链同构嫌疑在册，corpus 无断言面不扩面）；engine lib
> 默认并行 details flake 第 2 数据点（挂账独立勘根不变）；
> 其余承袭不变（reset.html form element pointer 面待决缓行、
> part03 raw 链 reset 臂无还原臂、form-nameditem 残余 10F、
> empty.html/empty-with-base.html Timeout、form-submission-0
> 186F、execution-timing 构成面、lazy-loading 26F、meta
> color-scheme 21F、写流盖章断链、part02 setAttribute 绕
> notify、041 nested write、deep clone 非 HTML ns、css-module
> corpus 门（待决）、www2 可达性、module-import-referrer TLA、
> dynamic import() with 透传、classic 外链 SRI、143 write、
> defer 多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、
> FileList 缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面
> 让位累积、s131 插入步缺口细化、radio 现象⑤/checked 属性增设/
> defaultChecked 无独立 corpus 面、s135 已知限制 3 条）；**待决
>（须用户拍板）**：css/text 数据模块语料资源缺失、green-2x2.png
> 语料资源分发、parser form element pointer 4F 缓行
>（2026-10-23）。
> **▶ 上轮状态（2026-10-29 M2 切片 135 施工收口，select reset 默认层捕获轮——reset-form select ×3 收口——其下一步 s136 已由 s136 收口，见上）**：
> **入场定刀位**：s134 挂账「select-by-reset R3909 3 面」入场：
> reset-form.html select ×3（reset() 方法 + button click + input
> click，激活臂均经 s134 接通的 `form.reset()`）——form.reset()
> 时 select 的 selectedness 未回落默认层。
> **勘根定谳**：①selectedness 三层读链（option.selected getter
> 序）——`_optSelectedDirty`（dirty，条目存在即权威）→
> `_optSelectedClean`（显式初始/写侧 spill 混合层）→ 宿主
> `__zw_option_selected`（callbacks.rs:1824，感知 renderer
> SelectOption 编程选中，回落快照 attr）→ selected 属性 lw 链。
> ②单根 = apply 侧改写快照属性破坏默认层——`select.value=` setter
> 经 `__zw_select_option`（callbacks.rs:1847）推
> `DomMutation::SelectOption`，apply 臂（js_dom_bridge.rs:1405）
> 改写快照 selected 属性（mark 目标 + deselect 兄弟），此后 live
> attr 承载编程选中态不再是默认层。③reset 链穿透——form.reset()
> 清 shim dirty/clean 键（s133）后读链穿透到宿主层，宿主返回
> stale 编程选中（o2=true 而默认层 o2 无属性）遮蔽属性层；
> `_zwSelectednessSetting` 走查同链读 true → last=o2 → fallback
> 不触发。探针页 shim 内埋点 pre/post 快照实证穿透点，排除脏态
> 存活、双 select 干扰两误判（后者靠载荷转储纠正：「branch ran
> twice」实为表单两个 select 各走一条）。
> **施工**（shim 3 文件 5 处，无 Rust 变更）：①part01 声明
> `_optSelAttrDefault = {}`（默认层捕获 store）+
> `__zw_reset_form_state` 清链同步清空；②part03
> `_zwOptCaptureSelDefaults(sel, handle)` helper——
> `_zwListOfOptions` 走查 `!hasOwnProperty` 时按 key 只填缺
> （fill-once）捕获 live attr 布尔，**唯一钩点 = select.value
> setter**（SelectOption 唯一推送源；selectedIndex= /
> option.selected= 只写 JS 层不改快照 attr 无需钩）；③part04
> value setter 调捕获 + reset SELECT 分支重写——删 dirty/clean
> 后显式落 `_optSelectedClean[k] = capture ?? live-attr`（遮挡
> 宿主 stale），再跑 `_zwSelectednessSetting`。
> **验证**：定向 resetting-a-form **14/14 全绿**（select ×3 F→P，
> reset() 方法与 click 两通道同链收口）；域回归八域键级 join
> 零位移（the-input-element 1644 / constraints 920 /
> form-control-infrastructure 121 / the-form-element 108 /
> the-button-element / the-select-element / the-option-element /
> the-textarea-element）；全量 A/B × /tmp/s134-full.log 键级 join
> **恰 3 Fail→Pass，真翻红 0、new non-Pass 0**（6803P/1008F/103T
> → 6805P/1005F/104T，行数 7914 自洽；唯一非 Pass 变动 =
> the-img-element/update-the-image-data/src-then-lazy-load.html
> 子测试 Pass → 文件级 Timeout，lazy-loading 网络等待面与改动零
> 关联，文件级超时边界波动前轮同形；join 另报 3 条 NEW 为
> dup-key 副本假阳性两份同 Pass）。engine lib **3571/0**（4 线程
> + 默认并行第 3 次；默认并行前 2 次
> test_details_name_exclusivity_template_deep_entry 间歇失败定谳
> 负载敏感型既有 flake——改动不触达 details/template 路径 + 单测
> 独跑/组跑/限并行全绿，挂账独立勘根）；runner bin **256/0**
>（76i）；clippy 零告警；零 Rust 变更 fmt 不适用；shim 拼接
>（13 part）node --check 过。探针页验后删；语料零改动。账本见
> [evidence/2026-10-29-m2s135-select-reset-default-layer-capture.md](html-semantics-compat/evidence/2026-10-29-m2s135-select-reset-default-layer-capture.md)。
> **范围**：engine shim part01.js + part03.js + part04.js。
> **已知限制（挂账）**：constructed-option（`new Option(...,
> true)` 第 4 参）在带 value= 的 select 中被捕获为 false（捕获只
> 读 live attr，无 corpus 面）；reset 后宿主 SelectOption 不同步
>（渲染面 stale 窗口，JS 读已正确，corpus 断言走 JS 读）；捕获
> 时机（首次 value=）之后用户改 selected 属性的 stale 窗口
>（fill-once 不更新，无 corpus 面）。
> **挂账更新**：select-by-reset 3 面**收口**（reset-form select
> ×3）；reset.html form element pointer 面仍归 parser form
> pointer 待决项缓行；**新挂账**：engine lib 默认并行偶发
> test_details_name_exclusivity_template_deep_entry flake（负载
> 敏感型待独立勘根）+ 已知限制 3 条；button-events 文件级
> Timeout 收窄待续（挂起 1）+「type=button shouldn't trigger
> submit」submit 误派发异根、part03 raw 链 reset 臂无还原臂
>（UA 指针激活平行近似 corpus 无面）承袭；其余承袭不变
>（form-nameditem 残余 10F、empty.html/empty-with-base.html
> Timeout、form-submission-0 186F、execution-timing 构成面、
> lazy-loading 26F、meta color-scheme 21F、写流盖章断链、part02
> setAttribute 绕 notify、041 nested write、deep clone 非 HTML
> ns、css-module corpus 门（待决）、www2 可达性、
> module-import-referrer TLA、dynamic import() with 透传、classic
> 外链 SRI、143 write、defer 多写完成序、接口对象基建面、FIXME
> 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号列表 qSA
> 根缺口、渲染流面让位累积、s131 插入步缺口细化、radio 现象⑤/
> checked 属性增设/defaultChecked 无独立 corpus 面）；**待决（须
> 用户拍板）**：css/text 数据模块语料资源缺失、green-2x2.png
> 语料资源分发、parser form element pointer 4F 缓行
>（2026-10-23）。

> **▶ 上轮状态（2026-10-29 M2 切片 134 施工收口，reset 按钮 click 激活接 form.reset() 轮——reset-form 4F + reset.html 4F + commandfor 3F 收口 + button-events 2 新键——其下一步 s135 已由 s135 收口，见上）**：
> **入场定刀位**：s133 挂账「click 复位 6 面」入场：spec §4.10.5.4
> reset button 激活行为 = form.reset()（trusted reset 事件 + 控件
> 复位），目标翻 reset-form.html input/textarea click 4 面。
> **勘根定谳**：①激活通道拓扑（双链分离）——proxy `.click()`
>（part04:5566 click() API 门，WPT runner 到达路径）默认动作臂只认
> submit/image，reset 型**完全无处理（连事件都不派发）**；
> part03:17793 raw node 链（UA 指针激活）派发无信任 reset 事件但无
> 还原臂。s133 账本原记「part03:17793 click 通道」实为 raw 链——
> runner 断言面走 proxy 链 no-op；两链与既有 submit 拓扑同构
>（raw 链裸派发、proxy 链走完整机制），raw 链本轮不动。②共享出口
> 单点——`_zwSubmitBtnDefaultAction`（net-api M5-S8）已是 .click()
> API 门（R57）与 dispatchEvent 派发 click 默认动作（R106 扩面）
> 两臂共用的 form 反链 + 激活入口，加 asReset 参数即单点接线。
> ③爆炸半径先圈后施工——corpus reset 按钮 click 断言面 4 文件
> 17 Fail + button-events 文件级 Timeout（t5/t7 挂起无行）；
> form-nameditem/constructing-form-data-set/form-elements-filter/
> form-requestsubmit 仅 markup/属性断言不卷入。④复位语义全继承
> ——激活臂经 FORM proxy `frm.reset()` 进 R3048 闭包（s129 trusted
> 事件 + s133 双身份枚举/清脏复位/R2998 两步式零新逻辑）。
> **施工**（part04.js 5 处，无 Rust 变更）：①
> `_zwSubmitBtnDefaultAction(sel, handle, asReset)` 尾部分支——
> asReset 走 `frm.reset()`（R3048），否则原 `_zwRunFormSubmit`；
> ②R106 dispatchEvent 臂 `_r106is` 扩 type=reset + 透传；
> ③.click() API 臂 `_subIsReset` 同门检测，disabled 守卫与 submit
> 同清。
> **验证**：定向 4 文件——resetting-a-form **恰 4 F→P**（input/
> textarea × button/input click；select ×3 R3909 既红同根、output
> ×2 保持 Pass）+ the-input-element/reset **恰 4 F→P**（form owner/
> disabled/form= 属性 ×2；form element pointer 面仍红=parser form
> pointer 待决项）+ button-click-resets-with-commandfor **3/3 F→P**
> + button-events **2 新键 Pass**（t5/t7 原挂起无行）；域回归键级
> join——the-input-element 1644 行恰 4 F→P 零他移、the-button-element
> 3 F→P + 2 新键零他移、resetting-a-form 14 行恰 4 F→P、constraints
> 920 行 / form-control-infrastructure 121 行 / the-form-element
> 108 行**逐键相同零位移**；全量 A/B × /tmp/s133-full.log 键级 join
> **恰 11 Fail→Pass + 2 新键 Pass**，**真翻红 0、new non-Pass 0、
> gone 0**（6790P/1019F/103T → 6803P/1008F/103T，行数 7912→7914 自
> 洽）。engine lib **3571/0**（s133 提交新增 1 案后基线）；runner
> bin **256/0**（76i）；clippy 零告警；零 Rust 变更 fmt 不适用；
> shim 拼接（13 part）node --check 过。无探针页；语料零改动。渲染
> 面零依赖。账本见
> [evidence/2026-10-29-m2s134-reset-button-click-activation.md](html-semantics-compat/evidence/2026-10-29-m2s134-reset-button-click-activation.md)。
> **范围**：engine shim part04.js。
> **挂账更新**：click 复位 6 面**收口移出候选**（input/textarea 4
> 面翻绿；select 2 面归 select-by-reset R3909 根）；**reset 域既有
> 异根更新**：①select-by-reset 3 面（reset-form select ×3）+
> reset.html form element pointer 面前置=双根既红；②button-events
> 文件级 Timeout 收窄待续（挂起 3→1，文件行 Timeout 仍在；
> 「type=button shouldn't trigger」Fail 承袭 submit 误派发异根非本
> 轮引入）；③part03 raw 链 reset 臂（UA 指针激活）仍无还原臂——
> 与 raw 链 submit 裸派发同构既有近似，corpus 无对应用例不扩面；
> form-nameditem 残余 10F 承袭；其余承袭不变（empty.html/
> empty-with-base.html Timeout、form-submission-0 186F、
> execution-timing 构成面、lazy-loading 26F、meta color-scheme 21F、
> 写流盖章断链、part02 setAttribute 绕 notify、041 nested write、
> deep clone 非 HTML ns、css-module corpus 门（待决）、www2 可达性、
> module-import-referrer TLA、dynamic import() with 透传、classic
> 外链 SRI、143 write、defer 多写完成序、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号列表 qSA 根缺口、
> 渲染流面让位累积、s131 插入步缺口细化、radio 现象⑤/checked 属性
> 增设/defaultChecked 无独立 corpus 面）；**待决（须用户拍板）**：
> css/text 数据模块语料资源缺失、green-2x2.png 语料资源分发、parser
> form element pointer 4F 缓行（2026-10-23）。

> **▶ 上轮状态（2026-10-28 M2 切片 133 施工收口，form reset handle 身份枚举 + 清脏复位轮——reset-form-2 1F 收口 + form-nameditem 1 案搭车——其下一步 s134 已由 s134 收口，见上）**：
> **入场定刀位**：s132 收口后按控制面候选②入场：接线缺口 corpus
> 面勘根——defaultChecked 全语义树仅 reset-form-2.html 用例（1F，
> 首断言 text.value reset 后应「text default」仍「text new value」）。
> 刀位定性：脚本构造（detached）form 的 reset 算法面，非
> defaultChecked/setAttribute 面本体。
> **勘根定谳**：①根因 1（枚举盲区）——`_formControls(sel)` 仅 sel
> 身份（`if (!sel) return []` 前置早退），纯 createElement form
>（handle 身份）控件枚举恒空 → reset 还原循环整体 no-op；
> `_zwFormSubmitNavigate` 注释已有同根限记（空 entry list 导航），
> 共享出口单点修复同单解除。②根因 2（脏旗驻留）——旧复位赋值式经
> IDL setter 重置 dirty 旗，复位后 default 层变更不联动（L54/L55
> 断言面）；spec reset 置 dirty 旗 false。③R2998 捕获模型约束——
> checkbox/radio default 在 `_boolDefault` 捕获不在 attr（前置
> .checked=false 已移除 attr），复位必须「经 defaultChecked 读捕获
> 赋值 → 再清 _checkedDirtyState」两步式，裸清键翻红。④value getter
> 脏判谓词 `_inputValues[key] != null && (set || stableKey)`——delete
> 双键即清脏；TEXTAREA 清键落 child text 活文本读，INPUT 落内容属
> 性 lw 读。⑤**基线误读排误（虚惊）**——resetting-a-form 定向 7 Fail
> 曾疑真翻红，系先前基线 grep 截断管道误读；完整基线逐键比对证实
> 7 Fail 与 s131/s132 完全同额，真翻红以全量键级 join 为准（=0）。
> **施工**（part04/05.js，无 Rust 变更）：①part05
> `_formControls(sel, handle)` 双身份：walk sel→`_wrapSelector` /
> handle→`_makeProxy(null, handle)`，host 视图加 sel 门；②part04
> 调用方 9 处传 handle（FORM trap 区 7 + _zwFormSubmitNavigate 2）；
> ③reset 体 `sel || handle` 守卫 + TEXTAREA/INPUT 赋值后清脏键
>（checkbox/radio 保留赋值 + 清 _checkedDirtyState）；④注释同步
>（10968 限记标解除）。
> **验证**：定向 reset-form-2 **1/1 Pass**（1F→0）+ resetting-a-form
> **7 Fail 逐键同额**（虚惊排除）+ form-control-infrastructure
> **117P/4F 同额** + the-form-element **87P/21F**（+1 F→P）+
> the-input-element **1512P/132F/3T 同额** + constraints **917P/3F
> 同额**；全量 A/B × /tmp/s132-full.log 键级 join **恰 2 Fail→Pass**
>（reset-form-2 主面 + form-nameditem「innermost form」案——共享出
> 口外溢），**真翻红 0、new non-Pass 0、gone 0**；唯一键 6785P/1021F
> → 6787P/1019F（Timeout 103 恒）。engine lib **3570/0**；runner bin
> **256/0**（76i）；clippy 零告警；零 Rust 变更 fmt 不适用；shim 拼
> 接（13 part）node --check 过。无探针页；语料零改动。渲染面零依
> 赖。账本见
> [evidence/2026-10-28-m2s133-form-reset-handle-identity.md](html-semantics-compat/evidence/2026-10-28-m2s133-form-reset-handle-identity.md)。
> **范围**：engine shim part04/05.js。
> **挂账更新**：reset-form-2 1F **收口移出候选**；handle 身份 form
> 枚举盲区（elements/length/named access/submit entry list）**共享
> 出口收口**；**reset 域既有异根显性入账**：click 复位 6 面（reset
> 按钮激活仅派发 reset 事件无控件还原臂——part03:17793；spec 激活
> 行为=form.reset()；input/textarea 两对面接共享还原即可翻，select
> 对面需先收 select-by-reset 根）+ select-by-reset 1 面（R3909 面既
> 红）+ form-nameditem 残余 10F（命名 getter 机制面异根）；承袭不变
>（button-events「type=button shouldn't trigger」+ 文件级 Timeout、
> empty.html/empty-with-base.html Timeout、form-submission-0 186F、
> execution-timing 构成面、lazy-loading 26F、meta color-scheme 21F、
> 写流盖章断链、part02 setAttribute 绕 notify、041 nested write、
> deep clone 非 HTML ns、css-module corpus 门（待决）、www2 可达性、
> module-import-referrer TLA、dynamic import() with 透传、classic
> 外链 SRI、143 write、defer 多写完成序、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号列表 qSA 根缺口、
> 渲染流面让位累积、s131 插入步缺口细化、radio 现象⑤/checked 属性
> 增设/defaultChecked 无独立 corpus 面）；**待决（须用户拍板）**：
> css/text 数据模块语料资源缺失、green-2x2.png 语料资源分发、parser
> form element pointer 4F 缓行（2026-10-23）。

> **▶ 上轮状态（2026-10-28 M2 切片 132 施工收口，radio 组排他现象 4（type 变更信号）接线轮——其下一步 s133 已由 s133 收口，见上）**：
> **入场定刀位**：s131 收口后按控制面下一步入场。候选①「form=
> 属性变更 radio 组重排」勘根先翻 corpus 断言面——**无 radio 组重
> 排面**（全语义树 setAttribute('form') 共 5 文件：form_attribute
> Pass；form-elements-filter Fail=form.elements 成员重排异根；
> form-nameditem Fail=命名 getter 机制面异根；radio.html /
> radio-keyboard-navigation-order 非 sweep 面）。radio 域残余单根面
> 显性：**radio-morphed.html 1F**——「Setting type attribute must
> unset checkedness of other elements」= spec 现象表现象 4（s131
> FIXME 已入码缺口），刀位收窄为现象④单根接线。
> **勘根定谳**：①spec 定谳（先拉原文）——§4.10.5.4 input type
> change steps 步骤 6「Signal a type change for the element. (The
> Radio Button state uses this, in particular.)」= 现象④触发信号；
> type 变更算法无 checkedness 重置步骤——morphed text（带 checked
> 属性）非脏回落属性读即 true，occurrence 后置真门自然成立。②现象
> ②（name 设/改/删）无独立 corpus 面：radio.html:88 案由现象①
>（checkedness 置 true，s125 已接）在新组员资格下 sweep 覆盖，不
> 单独立项。③**接线点单点覆盖双通道**——input.type IDL setter
>（part05 type 分支）与 setAttribute（part04 R3901 臂）均汇入
> `_zwApplyInputTypeChange`（R3901 既有共享出口），尾部单点接线即
> 双通道覆盖，无需 s131 式四通道拓扑（type 变更不走插入管道）；
> s131 已证拼接单闭包声明提升跨 part 可调用。④BUTTON 经
> setAttribute 设 type=radio 边界——sweep checked 门自兜（BUTTON 无
> checked 属性读 undefined 即返），不新增 tag 门（最小范围）。
> **施工**（part01/05.js，无 Rust 变更）：①`_zwApplyInputTypeChange`
> 尾部 `oldTy !== newTy && newTy === 'radio'` 门 + try/catch 调
> `_zwRadioUncheckGroupOthers(_makeProxy(sel, handle))`（同态再设
> 不触发——spec「changes state」语义）；②part05 s125 FIXME 更新
>（type 变更移出未接线清单）。
> **验证**：定向 radio-morphed **1/1 Pass**（1F→0）+
> the-input-element **1512P/132F/3T**（s131 1511P，+1 全为
> radio-morphed）+ constraints **917P/3F 与 s129/s130/s131 同额**；
> 全量 A/B × /tmp/s131-full.log 键级 join **恰 1 Fail→Pass**
>（radio-morphed 单案），**真翻红 0、new non-Pass 0、gone 0**；唯一
> 键 6784P/1022F/103T → 6785P/1021F/103T。engine lib **3570/0**；
> runner bin **256/0**（76i）；clippy（zero-engine -D warnings）零
> 告警；零 Rust 变更 fmt 不适用；shim 拼接（13 part）node --check
> 过。无探针页；语料零改动。全量 runner 以 corpus 已知 Fail 非零
> 退出（make 错误 1，s131 同形非异常）。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s132-radio-type-change-signal.md](html-semantics-compat/evidence/2026-10-28-m2s132-radio-type-change-signal.md)。
> **范围**：engine shim part01/05.js。
> **挂账更新**：radio-morphed 1F **收口移出候选**；the-input-element
> radio 族残余 Fail 全为 focus-navigation/keyboard 家族（交互面异根
> 另账）；**现象接线缺口收窄**（FIXME 已同步）：余 checked 属性增设
> /defaultChecked/becomes connected（现象⑤）未接，现象②由现象①
> 覆盖不单独立项；承袭不变（button-events「type=button shouldn't
> trigger」+ 文件级 Timeout、empty.html/empty-with-base.html
> Timeout、form-submission-0 186F、execution-timing 构成面、
> lazy-loading 26F、meta color-scheme 21F、写流盖章断链、part02
> setAttribute 绕 notify、041 nested write、deep clone 非 HTML ns、
> css-module corpus 门（待决）、www2 可达性、module-import-referrer
> TLA、dynamic import() with 透传、classic 外链 SRI、143 write、
> defer 多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、
> FileList 缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面让位累
> 积、s131 插入步缺口细化）；**待决（须用户拍板）**：css/text 数据
> 模块语料资源缺失、green-2x2.png 语料资源分发、parser form element
> pointer 4F 缓行（2026-10-23）。
> **下一步**：s133 入场定刀位（候选：全量基线翻拣下一单根面——域
> 翻拣刀法同款，radio 域已无单根 sweep 面让位；或 s131/s132 接线
> 缺口中翻拣有 corpus 面者——checked 属性增设/defaultChecked 面先
> 勘根再定）——已由 s133 收口，见上。

> **▶ 上轮状态（2026-10-28 M2 切片 131 施工收口，radio 组排他现象 3（form owner 变更）四通道接线轮——其下一步 s132 已由 s132 收口，见上）**：
> **入场定刀位**：s130 收口后按控制面下一步入场：插入步 radio
> checkedness 同步族 2F（radio-disconnected-group-owner 残余——
> 「Removed elements」后半 radio2 带 checked 属性 appendChild 进
> detached form 后 radio3 应 uncheck；「Appending … disconnect form」
> 全案 radio1 应 uncheck + innerHTML 重建后 children[0] 应 uncheck）。
> **勘根定谳**：①spec 现象表定谳（先拉原文）——HTML
> §4.10.5.1.16 现象表 5 项，DIV/FORM 对照案的判别维度是现象③
> 「form owner 变更」而非 root/tree 变更（detached DIV owner 恒 null
> 不触发=子测 7 负案在账不得回翻；⑤becomes connected 两案不沾）；
> s130「checked 属性同步不跑排他」归因据此修正。②**JS 插入四通道
> 拓扑**（勘根主转折）——appendChild 与 insertBefore 是 proxy
> handler 平行独立实现（6575/7952），原型方法只是轻节点兜底，首轮
> 接 `Node.prototype.insertBefore` 定向 2F 原样；加 R334 sel 子臂
> （提前 return）与 innerHTML setter 实为四通道。③innerHTML 面时序
> （探针 B 三轮）——桥调用点枚举取旧 registry（钩子内 qSA=__n1/__n2
> 旧子，同 turn 测试面 qSA=__n3/__n4 物化子），钩子必须在 t8r
> `_handleChildren[handle] = _ihMat` 重建之后；「桥读触发惰性刷新」
> 假设被探针否证。④R334 臂 `.form` wire 后活解析命中新 owner
>（探针 A1 formSame=true）。
> **施工**（part03/04/05.js，无 Rust 变更）：①part05 顶层共享出口
> `_zwRadioUncheckGroupOthers`（s125 sweep 体收编 + 现象门
> type/name/checkedness + s130 根域枚举 + form 键比对），set-trap 改
> 3 行调用；②appendChild trap head before 快照 + tail after 比对
> sweep；③insertBefore trap 同款（moveBefore 委托顺带覆盖）；④R334
> 臂出口前同款比对；⑤innerHTML setter registry 重建后逆序 sweep
>（`.form` 门；逆序后案 checked 胜出）；⑥part03 原型 insertBefore
> 三出口汇单出口 + 同款钩子（轻节点路径，与 trap 幂等双 sweep 无害）。
> **验证**：定向 radio-disconnected-group-owner **8/8 全 Pass**
>（2F→0）+ constraints **917P/3F 与 s129/s130 同额**（异根不动）+
> the-input-element **1511P/132F/3T**（join 翻动恰 2 行全 F→P、
> new/gone 0）；全量 A/B × /tmp/s130-full.log 键级 join **恰 2
> Fail→Pass**（radio-disconnected-group-owner 两案），**真翻红 0、
> new non-Pass 0**（other 5 = s130 探针页 Fail 键出清，页已删）；
> 唯一键 6781P/1029F/103T → 6783P/1022F/103T。engine lib **3570/0**；
> runner bin **256/0**（76i）；clippy（zero-engine -D warnings）零
> 告警；零 Rust 变更 fmt 不适用；shim 拼接 node --check 过。首轮全量
> guard 单进程内存杀一次（4194576KB>4194304KB 超 272KB，重跑通过，
> 阈值边缘抖动非本修复引入——账本如实记）。探针页 zw-probe-s131
> 验后删；语料零改动。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s131-radio-form-owner-change.md](html-semantics-compat/evidence/2026-10-28-m2s131-radio-form-owner-change.md)。
> **范围**：engine shim part03/04/05.js。
> **挂账更新**：插入步 radio checkedness 同步族 2F **收口移出候选**
>；**现象接线缺口细化入账**（FIXME 已入码）：form= 属性变更重排
>（input.form IDL setter 面）、移除方向（form → detached）、子树
> 深层 radio、_zwMEl registry 树 own insertBefore、sel 面 innerHTML
>（R304/R309 pending overlay 域）、outerHTML/insertAdjacentHTML；
> becomes connected（现象⑤）未接（无 corpus 面让位）；承袭不变
>（button-events「type=button shouldn't trigger」+ 文件级 Timeout、
> empty.html/empty-with-base.html Timeout、form-submission-0 186F、
> execution-timing 构成面、lazy-loading 26F、meta color-scheme 21F、
> 写流盖章断链、part02 setAttribute 绕 notify、041 nested write、
> deep clone 非 HTML ns、css-module corpus 门（待决）、www2 可达性、
> module-import-referrer TLA、dynamic import() with 透传、classic
> 外链 SRI、143 write、defer 多写完成序、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号列表 qSA 根缺口、
> 渲染流面让位累积）；**待决（须用户拍板）**：css/text 数据模块语料
> 资源缺失、green-2x2.png 语料资源分发、parser form element pointer
> 4F 缓行（2026-10-23）。
> **下一步**：s132 入场定刀位（候选：接线缺口中有 corpus 面者优先
>——form= 属性变更 radio 组重排（input.form IDL setter / form 内容
> 属性变更面，勘根先翻 corpus 有无断言面）；或全量基线翻拣下一单根
> 面——域翻拣刀法同款）——已由 s132 收口，见上。

> **▶ 上轮状态（2026-10-28 M2 切片 130 施工收口，radio 组根域枚举轮——其下一步 s131 已由 s131 收口，见上）**：
> **入场定刀位**：s129 收口勘误（join +3 归因修正）后按挂账入场
> radio.html 组属主 3F 单根勘根（orphan tree / different form
> owners / different name——s125 组排他轮新入账）。
> **勘根定谳**：①三案同根——detached 容器（createElement div +
> innerHTML 不入文档）内构组，s125 组排他 document qSA 枚举看不见
> detached 树组员，排他静默失效；探针两轮：detached 子代 handle 代
> 理、parentNode 链顶可达容器根、容器根 qSA identity 真、`.form`
> detached 活解析（FORM/null/异 form 异键）。②「not in the same
> tree」案 Pass 系 form-owner 键侥幸（detached null 键 ≠ 文档 form
> 键）非树域判定成立。③valueMissing radio 臂同根（document qSA）
> 且缺 form owner 比对——orphan 案 detached form 内 checked 组员误
> 并入无 form 组。④残余 2F 异根 = 插入步 radio checkedness 同步族
>（s125 FIXME「其余置真现象」同族显性化；本修复推进「Removed
> elements」案 2 断言）。
> **施工**（part03/05.js，无 Rust 变更）：枚举根域——parent 链顶
>（≤64 跳）nodeType 9 → document qSA（文档面绿面不变；探针实测
> documentElement 级元素 qSA 异常返 0，文档面不经元素级），链顶非
> document（detached）→ 链顶子树 qSA；part03 另加 `_vlFoKey` form
> owner 键比对（同 form 或均无才计组员）。
> **验证**：定向 radio.html **12/12 全 Pass**（3F→0）+ constraints
> **917P/3F 与 s129 同额**（异根不动）+ the-input-element
> **1509P/132F/3T**（s129 1501P/140F/3T，join 翻动恰 8 行全 F→P、
> new/gone 0）；全量 A/B × /tmp/s129-full.log 原始行口径
> **6780P/1032F/103T → 6785P/1029F/103T**（本轮探针页参跑 +5 探针
> Fail 键、−3 s129 探针 Pass 键），净语料键级 join **恰 8
> Fail→Pass**（radio 3 + radio-disconnected-group-owner 5：tree
> root/disconnect tree/document fragments/shadow roots/non-HTML
> containers），**真翻红 0、new non-Pass 0、gone 0**。engine lib
> **3570/0**；runner **256/0**（76i）；clippy（zero-engine -D
> warnings）零告警；fmt 净；shim 拼接 node --check 过。探针页
>（zw-probe-s130/s130b）验后删；语料零改动。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s130-radio-group-root-scope.md](html-semantics-compat/evidence/2026-10-28-m2s130-radio-group-root-scope.md)。
> **范围**：engine shim part03/05.js。
> **挂账更新**：radio 组属主 3F / radio-disconnected-group-owner 5F
> **收口移出候选**；**新增插入步 radio checkedness 同步族 2F**
>（radio-disconnected-group-owner 残余：appendChild 进 form/tree 后
> checked 属性同步或脏 checkedness 均不触发新组组员 uncheck——
> s131 候选）；承袭不变（button-events「type=button shouldn't
> trigger」+ 文件级 Timeout、empty.html/empty-with-base.html
> Timeout、form-submission-0 186F、execution-timing 构成面、
> lazy-loading 26F、meta color-scheme 21F、写流盖章断链、part02
> setAttribute 绕 notify、041 nested write、deep clone 非 HTML ns、
> css-module corpus 门（待决）、www2 可达性、module-import-referrer
> TLA、dynamic import() with 透传、classic 外链 SRI、143 write、
> defer 多写完成序、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt、
> FileList 缺索引访问、live 树逗号列表 qSA 根缺口、渲染流面让位累
> 积）；**待决（须用户拍板）**：css/text 数据模块语料资源缺失、
> green-2x2.png 语料资源分发、parser form element pointer 4F 缓行
>（2026-10-23）。
> **下一步**：s131 入场定刀位（候选：插入步 radio checkedness 同步
> 族单根勘根（part04 插入步 + checked 属性变更步接线）；或全量基线
> 翻拣下一单根面——域翻拣刀法同款）——已由 s131 收口，见上。

> **▶ 上轮状态（2026-10-28 M2 切片 129 施工收口，input/change 事件属性 + UA trusted 面轮——其下一步 s130 已由 s130 收口，见上）**：
> **入场定刀位**：s128 后全量基线翻拣——forms/input-change-
> event-properties.html 整页 11F 单根（input 事件 composed 全族缺，
> UI Events 事件定义表 composed:true）；同域 rider 五面：UA 派发语义
> 事件 trusted 面（reset/submit/script error/激活 input/change）、
> checkbox indeterminate（pre-click/canceled activation 清除）、
> runner SELECT 键导航接线（`script_select_key_action` 零调用点）、
> radio.html t1（trusted 同根）。
> **勘根定谳**：①composed 缺失面——part01 全文件 16 处 InputEvent
> 构造 + part03 `_zwFireInputChange` + part06 `__zw_dispatch_event`
> 两分支与 5 处导航 `new Event('input')` + execCommand 4 处；change
> composed:false 已对不动，无 corpus 负案。②R312 trusted 是**双印
> 通道**——`__zwTrusted`（构造）+ `_zwUaDispatch`（dispatch 免翻面，
> page-facing dispatchEvent 对 `isTrusted && !_zwUaDispatch` 强制翻
> false）；`_dispatchWithBubble` 内部通道不翻面可单印；新派发统一走
> `__zwMakeTrustedEvent`（dispatchEvent 面）。③checkbox pre-click/
> canceled activation steps 缺 indeterminate 清除。④**勘根转折（非
> 回归）**：定向跑后 typing 8 子测 `undefined reading 'bubbles'`
> 疑似施工回归——探针页四轮取证（含 el.blur() 对照）定谳：undefined
> 的是 changeEvent，s128 该文件 typing 在首断言（input composed）即
> 失败、change 断言从未执行，change 缺失是**被揭开的下一层前存断
> 链**——宿主焦点迁移管道 `_zwFocusSel → __zw_host_focus/__zw_host_
> blur`（slice22 钩子）只派事件不接 s128 两臂（focus 不记基线 →
> `__zw_text_change_due` 基线 undefined 恒 false；blur 无 change 派
> 生）；send_keys 首键自聚焦即宿主通道，s128 账本「change-on-blur
> 未接路线：宿主 `__zw_host_focus`」注记即此。⑤探针页首跑
> `test_driver is not defined` 系漏引 /resources/testdriver*.js 两
> 行（runner 适配依赖页面 script 声明面）。
> **施工**（part01/03/04/06.js + runner testharness.rs）：①part01
> 16 处 InputEvent init 加 composed:true（s129 注释块一处总注）；
> ②part03 `_zwFireInputChange` input composed+trusted / change
> trusted（合成 click() 本身保持 untrusted——checkbox.html:37/
> radio.html:119 负案不回翻）+ S110 排水与 R96 fetch reject 两 error
> 臂 → `__zwMakeTrustedEvent('error')` + checkbox pre-click/rollback
> 两点清 indeterminate；③part04 reset/submit 派发加 __zwTrusted；
> ④part06 `__zw_dispatch_event` beforeinput/input/plain input 分支 +
> 5 处导航 input + execCommand 4 处加 composed；⑤part06
> `__zw_host_focus` 记 text control change 基线（镜像 part04 focus
> 臂）+ `__zw_host_blur` change-on-blur 派生（`__zw_text_change_due`
> 门 → `_dispatchWithBubble` change bubbles+trusted，序 blur →
> focusout → change；与 part04 页面 JS 臂经 `_activeElKey` 幂等守卫
> 互斥不双派）；⑥runner scroll_key 分支 SELECT/radio 焦点键消费门
> （`script_select_key_action` 返 true 即跳过滚动默认动作，非 SELECT
> 照旧滚动——R3254-K5 生成器首次接线）。
> **验证**：定向 input-change-event-properties **11/11 全 Pass** +
> checkbox.html **6/6**（4F→0）+ radio.html t1 Pass（残余 3F = 组属
> 主语义异根同额）+ reset-event/button-events submit/fetch-src/
> failure 全 Pass；全量 A/B × /tmp/s128-full.log 原始行口径
> **6758P/1051F/103T → 6780P/1032F/103T**，键级 join **恰 19
> Fail→Pass**（11+4+1+1+1+1，全 `1F→1P`），**真翻红 0、new non-Pass
> key 0**（+3 唯一键为 zw-probe-s129.html 探针页三 Pass 键——参跑
> 时在语料、验后已删；初记 149.html tab 归一伪影系误归因，勘误：
> 149.html 12 行结果行两轮逐字节一致）。
> engine lib **3570/0**；runner **256/0**（76 ignored）；clippy
> （zero-wpt-runner -D warnings）零告警；fmt 净（fmt 修 testharness.rs
> 一处换行）；shim 拼接 node --check 过。探针页（gitignored）验后删
> ；语料零改动。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s129-uievents-composed-trusted-hostblur-change.md](html-semantics-compat/evidence/2026-10-28-m2s129-uievents-composed-trusted-hostblur-change.md)。
> **范围**：engine shim part01/03/04/06.js + runner testharness.rs。
> **挂账更新**：input-change-event-properties/checkbox.html t1·t3·
> t4·t6/radio.html t1/reset-event/button-events submit/fetch-src/
> failure **收口移出候选**；**新增 radio 组属主语义 3F**（orphan
> tree / different form owners / different name——组属主判定缺失，
> 异根让位）；s128 挂账两项收口（宿主 `__zw_host_focus` 断链接线 +
> checkbox 即时 change）；承袭不变（button-events「type=button
> shouldn't trigger」+ 该文件级 Timeout、empty.html/
> empty-with-base.html Timeout、form-submission-0 186F、execution-
> timing 构成面、lazy-loading 26F、meta color-scheme 21F、写流盖章
> 断链、part02 setAttribute 绕 notify、041 nested write、radio 组排
> 他其余置真、deep clone 非 HTML ns、css-module corpus 门（待决）、
> www2 可达性、module-import-referrer TLA、dynamic import() with
> 透传、classic 外链 SRI、143 write、defer 多写完成序、接口对象基
> 建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号
> 列表 qSA 根缺口、渲染流面让位累积）；**待决（须用户拍板）**：css/
> text 数据模块语料资源缺失、green-2x2.png 语料资源分发、parser
> form element pointer 4F 缓行（2026-10-23）。
> **下一步**：s130 入场定刀位（候选：radio 组属主语义 3F 单根勘根
> （tree/form/name 分组判定——同文件残余整簇）；或全量基线翻拣下一
> 单根面——域翻拣刀法同款）。

> **▶ 上轮状态（2026-10-28 M2 切片 128 施工收口，textarea 域残余同域收割轮——其下一步 s129 已由 s129 收口，见上）**：
> **入场定刀位**：s127 后全量基线翻拣 the-textarea-element 域残余 8F
> 逐点勘根：type/textLength = IDL getter 缺失单点；maxlength/minlength
> = IDL setter NaN 折叠缺失；change-to-empty = blur→change 提交面
> （勘根发现 the-input-element 域 change-set-value/change-to-empty-
> value 同根——共享面扩刀）；wrapping ×2 + placeholder-lineheight =
> 渲染流面让位。
> **勘根定谳**：①change-on-blur 共享面（3 断言）——语料定模型：用户
> 编辑置脏（`_userEdited`，`__zw_text_input`/`__zw_text_delete`/计划
> 管道 mark_user_edited 置；script `value=` 不算）且 blur 时现值 ≠
> focus 基线（净值比较——restore-then-set 与 script-set 两负案双双
> 依赖）；基建已有（`_userEdited` validity 消费中 + `_zwSelNormValue`
> R3906 双形态 live 读）。②textLength/type getter 缺失。③maxLength/
> minLength setter `Number()` NaN → String(NaN)="NaN" 写属性 → getter
> 解析失败返 -1；WebIDL long NaN→0。④渲染流面：hard wrap 依布局软
> 换行位置、placeholder-lineheight 19.5 分数行高——让位。
> **施工**（part01/03/04/05.js，无 Rust 变更）：①part01 `_s128ChgBase`
> + reset 链 + `__zw_text_change_baseline`/`__zw_text_change_due` 两
> 钩子；②part04 focus 臂获焦记基线 + 焦点迁移旧元素同语义提交；
> ③part04 blur 臂 blur→focusout 后提交 change（序 blur → focusout →
> change，互操作序；detached 臂不接）；④part03 TEXTAREA textLength
> getter（同 value 双路径）；⑤part04 type getter TEXTAREA →
> 'textarea'；⑥part05 R5009 setter maxLength/minLength NaN→0 折叠
> （负值 throw 既有路径不动）。
> **验证**：定向 `FILTER=forms/the-textarea-element` **38P/3F**
>（8F→3F，5F 收口）+ `FILTER=forms/the-input-element` **1496P/145F/3T**
>（join 翻动恰 4 行、无真翻红）；全量 A/B × /tmp/s127-full.log 原始
> 行口径 **6749P/1060F/103T → 6758P/1051F/103T**，键级 join 恰 **9
> Fail→Pass**（4 input + 5 textarea，全 `1F→1P`），**真翻红 0、漂移
> 0、gone/new 0**。本轮无 guard 内存杀。engine lib **3570/0**；runner
> **256/0**；clippy 零告警；fmt 净；shim 拼接 node --check 过。语料
> 零改动、无探针页。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s128-change-on-blur-harvest.md](html-semantics-compat/evidence/2026-10-28-m2s128-change-on-blur-harvest.md)。
> **范围**：engine shim part01/03/04/05.js。
> **挂账更新**：textarea 域残余收割**收口移出候选**；渲染流面让位
> 承袭累积（wrapping hard-wrap ×2、placeholder-lineheight、
> naturalWidth SVG viewBox 22F、current-pixel-density 11F）；新增
> change-on-blur 未接路线注记（宿主 `__zw_host_focus` Tab 驱动失焦、
> checkbox 即时 change——checkbox.html 'click → input → change' 3F
> 在账，语料静默未施工）。承袭其余不变（form-submission-0 186F、写
> 流盖章断链、part02 setAttribute 绕 notify、execution-timing 构成
> 面、041 nested write、radio 组排他其余置真、deep clone 非 HTML ns、
> lazy-loading 26F、meta color-scheme 21F、css-module corpus 门（待
> 决）、www2 可达性、module-import-referrer TLA、dynamic import()
> with 透传、classic 外链 SRI、143 write、defer 多写完成序、接口对
> 象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live 树
> 逗号列表 qSA 根缺口）；**待决（须用户拍板）**：css/text 数据模块
> 语料资源缺失、green-2x2.png 语料资源分发、parser form element
> pointer 4F 缓行（2026-10-23）。
> **下一步**：s129 入场定刀位（候选：挂账序复核——form-submission-0
> 186F POST 传输契约勘根 / execution-timing 构成面单根；或全量基线
> 翻拣下一单根面——域翻拣刀法同款，整页整簇 Fail 勘根优先）。

> **▶ 上轮状态（2026-10-28 M2 切片 127 施工收口，TEXTAREA value/defaultValue getter 面轮：value-defaultValue-textContent 10F→0F + wrapping 换行归一搭车 1F→0F，全量 join 恰 11 Fail→Pass 真翻红 0——其下一步 s128 已由 s128 收口，见上）**：
> **入场定刀位**：全量基线翻拣 forms/the-textarea-element 域
> value-defaultValue-textContent.html 整页 12 子测试 10F，语料全为
> detached createElement textarea（handle 形）——value/defaultValue
> 读面单根，入场。
> **勘根定谳**：①value getter 非脏路径 handle 形落 stable-key 惰性
> 初始化读 value **属性**——textarea 无该属性 → 缓存 `''` 钉死，
> live 树 textContent/nodeValue/data/子节点增删全部不可见（6+3 断言
> 主根因）。②API 值缺 `\r\n|\r → \n` 归一（CRLF 2 断言 + 搭车
> wrapping 换行归一 1F）。③child text content 误并入元素
> textContent（"reflect child text content, not textContent"——仅
> 直接 Text 子数据拼接，R81 融合视图语义不受影响）。④value setter
> 缺 [LegacyNullToEmptyString] null→'' 折叠。⑤defaultValue getter
> R3049 冻结捕获 stale，应 live child text content。
> **施工**（part01/03/04/05.js + part01.rs 注释，无 Rust 行为变更）：
> ①`_zwTaLiveText` 删元素文本并入臂；②value getter 补 TEXTAREA 臂：
> 脏读 dirty 缓存、非脏 sel 形 `__zw_get_text` / handle 形
> `_zwTaLiveText`，读侧统一归一；③defaultValue getter 改 live 双
> 形态；④setter `_s127TaSet` null 折叠（`_inputValues`/
> `_inputValuesSet` 缓存写保留——2026-10-15 learning 写入点契约）；
> ⑤`_textareaDefault` 全链删除（声明/reset 链/三处捕获写点）；
> ⑥Rust 测试断言不变仅注释更新（①defaultValue=初值 ②value= 不改
> defaultValue ③顺序无关 ④reset 还原 全兼容）。
> **验证**：定向 `FILTER=forms/the-textarea-element` **33P/8F**
> （value-defaultValue-textContent 10F→0F，12/12 Pass）；全量 A/B ×
> /tmp/s126-full.log 原始行口径 **6738P/1073F/103T →
> 6749P/1060F/103T**，键级 join 翻动 = 11 Fail→Pass（恰 10 本修复 +
> 1 wrapping 搭车）+ 2 GONE（zw-probe-s126.html s126 探针已删 corpus
> 态差），**真翻红 0、漂移 0、新增 0**（既有 ×2 重跑键两轮一致；
> s126 账本 6737P/1071F/103T = 同文件原始行扣探针 2F − ×2P 记 1 行
> 口径，本轮起记原始行口径以 join 为门禁）。首跑 guard 内存超限杀
> 一次（4.31GB，ZeroWeb-3 并行测试流在场）复跑过——并行摆动（s126
> 同款注记）。engine lib **3570/0**；runner **256/0**；clippy 零
> 告警；fmt 净；shim 拼接 node --check 过。语料零改动、无探针页。
> 渲染面零依赖。账本见
> [evidence/2026-10-28-m2s127-textarea-value-defaultValue.md](html-semantics-compat/evidence/2026-10-28-m2s127-textarea-value-defaultValue.md)。
> **范围**：engine shim part01/03/04/05.js + part01.rs 测试注释。
> **挂账更新**：textarea value/defaultValue 面**收口移出候选**；
> the-textarea-element 域残余 8F 各单点（change-to-empty-value /
> maxlength / minlength / placeholder-lineheight / textLength / type
> / wrapping LF+col-count ×2）未勘根，入同域收割候选。承袭其余不变
> （form-submission-0 186F、写流盖章断链、part02 setAttribute 绕
> notify、execution-timing 构成面、041 nested write、radio 组排他
> 其余置真、deep clone 非 HTML ns、lazy-loading 26F、meta
> color-scheme 21F、naturalWidth SVG viewBox 22F、current-pixel-
> density 11F、css-module corpus 门（待决）、www2 可达性、module-
> import-referrer TLA、dynamic import() with 透传、classic 外链
> SRI、143 write、defer 多写完成序、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号列表 qSA 根缺
> 口）；**待决（须用户拍板）**：css/text 数据模块语料资源缺失、
> green-2x2.png 语料资源分发、parser form element pointer 4F 缓行
> （2026-10-23）。
> **下一步**：s128 入场定刀位（候选：挂账序复核——form-submission-0
> 186F POST 传输契约勘根 / execution-timing 构成面单根；或全量基线
> 翻拣下一单根面；或 textarea 域残余 8F 同域收割逐点勘根）。
> （已由 s128 择「textarea 域残余 8F 同域收割」收口，见上）

> **▶ 上轮状态（2026-10-28 M2 切片 126 施工收口，img relevant mutations 三根因轮：relevant-mutations 家族 114P/7F→121/121 全 Pass，全量 47 翻绿配平真翻红 0——其下一步 s127 已由 s127 收口，见上）**：
> **入场定刀位**：全量基线翻拣 the-img-element 域 relevant-mutations.html
> 整页 46F。前置动作（非代码）：corpus 依赖 `wpt-data/images/
> green-2x2.png` 为 gitignore 本地语料缺失——生成本地 2×2 绿 PNG 后
> 46F→6F（+40 资源假红翻绿，真翻红 0；资源分发策略挂账待决）。余 7F
> 三根因代码修复收口。
> **勘根定谳**：①referrerpolicy ×5——`__zwAttrState5277` 把
> missing/invalid/empty 归并 `'no-referrer-when-downgrade'`（旧 spec
> invalid default），现行 spec 缺省态为空串、n-r-w-d 独立——状态比对
> 判「未变」不触发；Pass/Fail 分界线 100% 吻合（凡 `'no-referrer'` 新旧
> 态 7 行真变迁经 avail 重派本就 Pass）。②"parent is picture, previous
> source removed"——removeChild trap 在节点摘除**后**调 helper，`after`
> 旗标依赖 `kids[i] === node` 永不置位，SOURCE 移除臂死码。③lazy
> "width attribute changes"（whatwg/html#8492）——part05 维度 setter 经
> moAttr 汇流，IMG 分支无 width/height 臂。
> **施工**（part03.js + part04.js，无 Rust 变更）：①状态表 referrerpolicy
> 缺省 `'no-referrer-when-downgrade'` → `''`（crossorigin 分支不动）；
> ②`__zwImgTreeMutated5277` 增第 5 参 `afterAnchor`（former next
> sibling 位置锚：从锚含向后重验、null=原为末子不重验、缺省保守全量；
> part04 sel 路径尾调传 R188 `_r188Next`，plain 子路径不变）；③IMG 属性
> 分支补 width/height lazy 臂（loading 归一 `'lazy'` 才重验）。
> **验证**：定向 relevant-mutations **121/121**（113+8；修复前 114P/7F；
> src-absent referrerpolicy 族保持 timeout-Pass（null selection 静默）、
> state-not-changed 族保持 Pass）；the-img-element 全域 A/B 资源同态
> **827P/110F/17T → 834P/103F/17T** 翻动恰 7 行、零漂移；全量 A/B
> **6690P/1118F/103T → 6737P/1071F/103T**（47 翻绿配平 = 40 资源 +
> 7 修复，真翻红 0、漂移 0、gone/new 0；首跑 test-guard 单进程内存
> 超限杀一次复跑过——并行摆动）。engine lib **3570/0**；runner
> **256/0**；clippy 零告警；fmt 净；shim 拼接 node --check 过。语料零
> 改动（探针 ×1 验后删；green-2x2.png 不入提交）。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s126-img-relevant-mutations.md](html-semantics-compat/evidence/2026-10-28-m2s126-img-relevant-mutations.md)。
> **范围**：engine shim part03.js + part04.js。
> **挂账更新**：img relevant mutations **收口移出候选**；新增
> green-2x2.png 语料资源分发策略待决（gitignore 本地数据他机假红面，
> 须用户拍板）；naturalWidth SVG viewBox 22F / current-pixel-density
> 11F = 渲染流面让位不施工。承袭其余不变（form-submission-0 186F、
> 写流盖章断链、part02 setAttribute 绕 notify、execution-timing 构成
> 面、041 nested write、radio 组排他其余置真、deep clone 非 HTML ns、
> lazy-loading 26F、meta color-scheme 21F、css-module corpus 门（待
> 决）、www2 可达性、module-import-referrer TLA、dynamic import()
> with 透传、classic 外链 SRI、143 write、defer 多写完成序、接口对象
> 基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号
> 列表 qSA 根缺口）；**待决（须用户拍板）**：css/text 数据模块语料资
> 源缺失、parser form element pointer 4F 缓行（2026-10-23）。
> **下一步**：s127 入场定刀位（候选：挂账序复核——form-submission-0
> 186F POST 传输契约勘根 / execution-timing 构成面单根；或全量基线
> 翻拣下一单根面——本轮域翻拣刀法同款，整页整簇 Fail 勘根优先）。
> （已由 s127 择「全量基线翻拣下一单根面」收口，见上）

> **▶ 上轮状态（2026-10-28 M2 切片 125 施工收口，input 克隆状态传播轮：deep clone form 状态传播——clone.html 19F→19/19 全 Pass，radio.html 搭车 4F→0，真翻红 0——其下一步 s126 已由 s126 收口，见上）**：
> **入场定刀位**：s124 候选均为大基建/跨流面，全量基线翻拣 forms 域
> 单根面：`forms/the-input-element/clone.html` 整页 19F（4 checked +
> 15 value）——克隆状态传播单根，入场。
> **勘根定谳（探针页 gitignored corpus 验后删）**：①deep clone 子树
> innerHTML 序列化重解析产物是 **`_zwMEl` plain 视图**（无 handle、无
> IDL 面）——克隆 input 的 value/checked IDL get 无脏缓存无键，form
> 状态传播断链（浅克隆臂 R3901/R3902b 正常，隔离探针对照）；②修复
> 18/19 后余 1 定谳——corpus setup 跳过 `inputs[3].checked = false`，
> 依赖 spec §4.10.5.1.16 radio 组排他（checkedness 置 true 时同组其余
> 置 false），引擎 checked setter 无此逻辑；③序列化面只含已 apply 面
> ——pending RemoveAttr/SetAttr 不入克隆属性复制，克隆侧需显式对齐。
> **本轮施工**（净变更 part04.js + part05.js，无 Rust）：①deep clone
> HTML-ns 物化（`_wcRebuildAsHandle` t8r 同款，`_zwNodeParent` 反链；
> **含 TEMPLATE 后代的顶层子豁免**——WC-M1 85d3987cc content 惰性视图
> 契约 + 模板 content 子 inert 无传播需求，首版全量物化翻红
> wc_m1 inShadow 形态哨兵已修）；②form 状态配对 walk
> （`_inputValues`/`_inputValuesSet` 复制 + `_zwInputValueMode` 分派
> default 族补写 value 属性、value 族只写脏缓存 + `_checkedDirtyState`/
> `_indeterminate` 复制 + checked 属性 latest-wins 对齐含
> `_zwAttrInstanceRemoveKey`）；③part05 checked setter radio 组排他
> （qSA 同名 + form owner 键比对；脏件写 `_checkedDirtyState=false` 保
> dirty 旗标、净件移属性 + `_mo_notify`；FIXME 仅 IDL setter 面）。
> **验证**：定向 clone **19/19**；**全量 A/B ×2（修正前后各一轮）真翻
> 红 0、Fail→Timeout 漂移 0**，翻绿 23 = clone 19 + radio 4（组排他
> 搭车：only one control/non-ASCII name/changing the name/moving
> into a form）；6668P/1141F/103T → 6691P/1118F/103T 干净配平。
> engine lib **3570/0**（连续两轮）；runner 全量 **256/0**；guard 内
> clippy 零告警；fmt 净；shim 拼接 node --check 过。语料零改动（探针
> ×1 验后删）。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s125-input-clone-state.md](html-semantics-compat/evidence/2026-10-28-m2s125-input-clone-state.md)。
> **范围**：engine shim part04.js + part05.js。
> **挂账更新**：input 克隆状态传播**收口移出候选**；radio 组排他其余
> 置真现象（属性增设/defaultChecked/激活）FIXME 保留（无语料需求）；
> deep clone 非 HTML ns 保持 plain 视图路径（R190 面向）。承袭其余
> 不变（form-submission-0 219F、写流盖章断链、part02 setAttribute 绕
> notify、execution-timing 构成面、041 nested write、lazy-loading 26F、
> meta color-scheme 21F、css-module corpus 门（待决）、www2 可达性、
> module-import-referrer TLA、dynamic import() with 透传、classic 外
> 链 SRI、script-crossorigin-network cookies、redirect-relative-urls、
> export-from with FIXME、143 write、defer 多写完成序、接口对象基建面、
> FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号列表 qSA
> 根缺口）；**待决（须用户拍板）**：css/text 数据模块语料资源缺失、
> parser form element pointer 4F 缓行（2026-10-23）。
> **下一步**：s126 入场定刀位（候选：挂账序复核——form-submission-0
> 186F POST 传输契约勘根 / execution-timing 构成面单根；或全量基线
> 翻拣下一 forms 域单根面——本轮 clone.html 同款刀法，域挂账外单页
> 整簇 Fail 勘根优先）。

> **▶ 上轮状态（2026-10-28 M2 切片 124 施工收口，module 网络面收口轮：link rel=modulepreload 取回面——modulepreload 家族基线 6 行（4F+2T）→ 15/15 全 Pass，真翻红 0——其下一步 s125 已由 s125 收口，见上）**：
> **入场定刀位**：s123 下一步首选 module 网络面挂账项勘根（四项逐一辨
> shim/runner/host 契约面）。**勘根定谳**：①modulepreload referrerpolicy
> 族三面缺口（echo-referrer.py 端点仿真缺 / shim 无 link modulepreload
> 取回面 / 元素级 referrerpolicy 无通道）+ cross-origin 页堵点**非
> https 传输**（www1:8444 别名已剥）实缺 script.py 端点；②
> module-import-referrer TLA 根因精确化——入口 TLA 可用、**依赖模块
> TLA 不可用**（build_dep_iife 同步 IIFE SyntaxError，深转换架构面）；
> ③dynamic import() with 透传——语料页全为 css-module 复合面，保留；
> ④classic 外链 SRI——无语料需求，保留 FIXME。
> **本轮施工**（shim part01/part03 + script-sandbox v8_runtime.rs +
> runner testharness.rs）：①shim `_zwModulePreloadMaybeStart` 取回面
> （script prepare 同连接门 + rel 白名单 + `__zw_lp` 内部查询参数携
> 元素级 referrerpolicy + `__zw_fetch_script` 取回 + 绝对 URL 键播种
> `__moduleCache` + 异步 load/error）；②**同步内联取回**（时序勘根：
> setTimeout 延后臂在断言前从未轮转——ZW 文档脚本与 testharness 起跑
> 间不泵 timer 队列，body 的 import()×4 先行重取回覆写 referrers——
> mp5 忠实复刻探针实证）；③`_mo_notify` childList 汇流点 LINK 触发臂
> （顶层 + _walk111 下降，SCRIPT 臂同型；innerHTML 重建产物 body-context
> 平铺解析代理，连接门父链爬升复核）；④classic 动态 import() **module
> map 记忆化**（原生回调此前每调用重取回重求值——成功条目读
> `__moduleCache[resolved]` 命中即 resolve 恰好一次求值 + 成功写表 +
> 失败三态不入缓存 s87 口径保持，best-effort 双向）；⑤runner 端点仿真
> ×2（echo-referrer.py 策略链 `__zw_lp`→pipe→文档→默认逐臂过滤 +
> script.py 非空 JS 回体）；⑥pin ×2（runner 端点四面 +
> sandbox 记忆化四臂——persistent context 必要性首版钉假阳性已修正）。
> **验证**：定向 `FILTER=modulepreload` **15/15**；**全量 A/B 逐行
> status join：+15P/−4F/−2T 全落 modulepreload 家族，真翻红 = 0**
> （6653P/1145F/105T → 6668P/1141F/103T，行数 +9 = 基线挂起页未完成
> 注册的 9 子测试现完成打印，总量口径变化已记账）。script-sandbox lib
> **377/0**；engine lib **3570/0**；wpt-runner 全量 **256/0**；guard 内
> clippy（sandbox+runner+engine -D warnings）零告警；fmt 净；shim 拼接
> node --check 过。语料零改动（探针页 ×7 + dep ×2 验后删，调试追踪桩
> 全移除）。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s124-modulepreload-face.md](html-semantics-compat/evidence/2026-10-28-m2s124-modulepreload-face.md)。
> **范围**：engine shim part01/part03 + script-sandbox v8_runtime.rs +
> runner testharness.rs。
> **挂账更新**：modulepreload referrerpolicy 族 + inline 重建路径**收口
> 移出候选**；module-import-referrer TLA 根因精确化保留（依赖图 async
> 语义 = module transform 管线重构级）；dynamic import() with 透传保留
> （css-module 复合）；classic 外链 SRI 保留 FIXME。承袭其余不变
> （form-submission-0 219F、写流盖章断链、part02 setAttribute 绕 notify、
> execution-timing 构成面、041 nested write、lazy-loading 26F、meta
> color-scheme 21F、css-module corpus 门（待决）、www2 可达性、
> script-crossorigin-network cookies、redirect-relative-urls、
> export-from with FIXME、143 write、defer 多写完成序、接口对象基建面、
> FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号列表 qSA
> 根缺口）；**待决（须用户拍板）**：css/text 数据模块语料资源缺失、
> parser form element pointer 4F 缓行（2026-10-23）。
> **下一步**：s125 入场定刀位（候选：承袭挂账序复核——form-submission-0
> 186F POST 传输契约勘根 / execution-timing 构成面单根；或 meta
> color-scheme 21F 跨流面评估（style-system 协同））。

> **▶ 上轮状态（2026-10-28 M2 切片 123 施工收口，FormData 条目构造收口轮：newline-normalization 12F→0F，全量 A/B 恰 14 Fail→Pass 真翻红 0——其下一步 module 网络面已由 s124 收口，见上）**：
> **入场定刀位**：取 s122 挂账首选 newline-normalization 12F 单根候选
> （原假设「FormData 构造换行归一过度」）——**勘根证伪原假设**：条目枚举
> 整体为零，换行归一面反而正确（idlVal 原文保真）。
> **根因定谳（探针五轮递进，全 gitignored corpus 验后删）**：语料 12 测
> = value/name/filename × \n \r \r\n \n\r 原文保真断言。①get null/spread
> 空实锤**条目枚举整体为零**（非归一）；②attrName="a" 正常排除 stale；
> ③doc qSA('input')=2 疑查询通（**误导一轮**——单选择器有 R161 pending
> 回落臂）；④`'input,input'` **逗号列表**=0 实锤——live 树 face 列表查
> 询恒空（R57 :invalid 臂既有先例「分开查询 concat」）；⑤static 标记页
> 同查询=2 定界 host 快照支持列表、**缺口仅 live 树 face**。第二根因：
> file 状态在构造器枚举臂无专属处理（落通用臂零条目，spec file 臂要求
> 每选中文件一条）。
> **本轮施工**（净变更 part02.js + testharness.rs pin ×1，无其他 Rust）：
> ①**枚举双源并集**——document 级查询（静态页原路径 + form= 属性站外
> 关联 best-effort）在前 + `form.querySelectorAll` 子树 live 查询随后，
> `_fdSeen` 按元素键去重（`_elKeyOf` 优先 handle/selector 回落）——静态
> 页序与原行为逐项一致，动态页子树序即树序。②**file 状态条目臂**——
> `type === 'file'` 每选中文件一条 `_zwFdEntry(String(name), _fdFile)`
> （File 本体保真，File.name 不做换行归一）；无选中零条目（spec 同）；
> 兜底 else-if 去 `type !== 'file'` 守卫（联动防双计）。③pin
> `formdata_entry_list_s123` 两面：hidden value='b\nc' 动态建表 live 枚
> 举 + 原文换行零归一；DataTransfer→files setter→`fd.get('f').name` 原
> 文 filename。
> **验证**：定向 `FILTER=newline-normalization` **12/12**（12F→0F）；
> **全量 A/B 逐行 status join：恰 14 对 Fail→Pass（12 newline +
> 2 wrapping-transformation 搭车——FormData(form) 同路径），真翻红 =
> 0**（6639P/1159F/105T → 6653P/1145F/105T，总数 7903 恒定；首跑探针
> 未删时取数 +7 Fail 噪声总量不配平，被 pre-commit 门禁事实核查项
> BLOCK 抓出，删净补跑定谳）。bin pin 新代码**绿**；part02.js
> stash 往返**两面红**（`live form 枚举 + 原文换行零归一: null` /
> `file 条目 File.name 原文: null`——行为失败非编译失败）。engine lib
> **3570/0**；guard 内 clippy（runner --all-targets）零告警；fmt 净；
> shim 拼接 node --check 过。渲染面零依赖。账本见
> [evidence/2026-10-28-m2s123-formdata-entry-list.md](html-semantics-compat/evidence/2026-10-28-m2s123-formdata-entry-list.md)。
> **范围**：engine shim part02.js + runner testharness.rs（commit
> 7232a83d8）；语料零改动（探针页 ×5 验后删除）。
> **挂账更新**：newline-normalization 12F **收口移出候选**；新挂账 ×2：
> **FileList 缺索引访问**（files[0] undefined，仅 item()/length/iterator
> ——spec 缺口真实但 corpus 不需要，精准修改未顺带扩面）；**live 树
> document 级逗号列表 qSA 根缺口**（单选择器 R161 回落臂列表没有——本
> 切片以双源并集绕开，根缺口在 host 查询 face，同症 face 按 R57 先例分
> 开查询 concat 或补列表回落臂）。
> 承袭：form-submission-0 219F 精确化挂账（186F POST 传输契约 + 33F
> iframe 文档管道，均大基建）、写流执行身份盖章断链 FIXME、part02
> setAttribute 绕 notify 臂（gap 2）、execution-timing 构成面 056/063+
> 051/052 单根（增量解析大基建）、041 nested write（写流队列重构）、
> lazy-loading 族 26F（viewport 基建大项）、meta color-scheme 21F（跨流
> style-system）、css-module 族 corpus 门（待决须用户拍板）、www2 可达
> 性、modulepreload referrerpolicy 族、module-import-referrer TLA、
> script-crossorigin-network cookies、redirect-relative-urls、
> export-from with FIXME、143 write、defer 多写完成序、dynamic import()
> with 透传、classic 外链 SRI、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；
> **待决（须用户拍板）**：css/text 数据模块语料资源缺失、parser form
> element pointer 4F 缓行（2026-10-23，并入 declarative shadow DOM
> parser 批次）。
> **切片间隙**：本轮 FormData 条目构造 shim 单根面（枚举双源 + file 臂）；
> 下一候选 module 网络面挂账项勘根轮（已由 s124 收口，见上）。

> **▶ 上轮状态（2026-10-28 M2 切片 120 施工收口，fragment 解析 script 克隆 already-started 继承轮——其下一步 s121 已由 s121 收口，见上）**：
> **入场定谳**：s118 FIXME(spec-gap)「cloneNode 重解析产物 fragment 章」由语料
> `scripting-1/the-script-element/svg-script-fragment-parsed-not-executed-on-clone.html`
> （3F）坐实需求（innerHTML 产物 script 浅克隆/含 script 子树深克隆再插入均执
> 行）。同轮勘察：lazy-loading 族 26F（viewport 基建，挂大基建）、dialog 43F
> 多根、meta color-scheme 21F 刀位在 style-system（渲染流让位）——本页唯一单
> 根常规刀位。探针页四臂定谳两断链：①t8r 物化把 s118 打章 `_zwMEl` 重建为
> **新建 wrapper** 存 registry，章丢（查询产物 flag undefined）；②R387 直插臂
> /s97 insertBefore 镜像臂同步执行**先于** notify s111 臂章门（带章脚本插入照
> 跑 p2=true）。
> **本轮一刀**（净变更 part03/part04 + testharness.rs pin ×1）：①
> `_wcRebuildAsHandle` 章随物化传递（R5197 序列化印章同款，children 递归子层
> 继承）②R387 臂 + insertBefore 镜像臂章门（`child.__zwFragNoExec === true`）
> ③s95 克隆块源章检查继承 run-once 表 + deep 分支重解析产物按源章 gate 整体
> 打章 ④`_zwDeepCloneEl` 章传播（plain 克隆臂）。FIXME(spec-gap) 收窄：克隆传
> 播面已补，余 iah 产物待语料需求。
> **首刀教训**：s95 块初版读 `n.__zwFragNoExec`——`n` 是 proxy 内部 target 不
> 持章（章经 proxy set/get 存取），改 `_makeProxy(sel, handle)` 重读后全通；
> detached 树 deep 克隆走 `_cclone` 递归臂（host innerHTML 空，重解析分支不触
> 发），s95 一处修正覆盖浅/深两面；`tbl=true` 歧义（执行器跑完也盖章）须
> append 前采样定谳章继承。
> **验证**：探针四臂语义全对（源章在场/章门消费/tblBefore=true 章继承）；语料
> 页 **3F→P**（4/4 含 sanity）；bin pin 旧 shim stash 往返红（`clone runs` 行
> 为失败）→ 绿；定向 `FILTER=execution-timing/0` 78 行零变化；**全量 A/B**
> 6612P/1186F/105T→6602P/1184F/107T 原始计数，目标面 3F→P 精确三行对，另三
> 处漂移逐一归因非翻转（两 flake 页过滤重跑复绿 80P/8P；modulepreload 挂账
> 族页挂起→跑完行形漂移）→ **真翻红 0**。engine lib **3570/0**；guard 内
> clippy（runner/engine --all-targets）零告警；fmt、shim 拼接语法过。渲染面
> 零依赖——reftest/product-smoke 未跑；make test 未跑（Rust 增量仅 test 模块）。
> 账本见 [evidence/2026-10-28-m2s120-fragment-clone-already-started.md](html-semantics-compat/evidence/2026-10-28-m2s120-fragment-clone-already-started.md)。
> **范围**：engine shim part03/part04 + runner testharness.rs；语料零改动（探
> 针页新建验后删除）。
> **挂账新增**：无（FIXME(spec-gap) 收窄为仅余 iah 产物）。承袭：写流执行身
> 份盖章断链 FIXME、part02 setAttribute 绕 notify 臂（gap 2）、execution-timing
> 构成面 056/063+051/052 单根（增量解析大基建）、041 nested write（写流队列重
> 构）、lazy-loading 族 26F（viewport 基建大项）、dialog 43F 多根、meta
> color-scheme 21F（跨流 style-system）、css-module 族 corpus 门（待决须用户拍
> 板）、www2 可达性、modulepreload referrerpolicy 族、module-import-referrer
> TLA、script-crossorigin-network cookies、redirect-relative-urls、export-from
> with FIXME、script-charset-02/03、143 write、defer 多写完成序、dynamic
> import() with 透传、classic 外链 SRI、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt；**待决（须用户拍板）**：css/text 数据模块语料资源缺失。
> **下一步**：s121 入场定刀位（候选：the-dialog-element 43F 簇勘根 / meta-
> element 21F 簇跨流让位确认 / execution-timing 余 F / 承袭挂账序；lazy-loading
> 26F 与增量解析同档挂大基建不作常规刀位）。
> **切片间隙**：本轮脚本克隆/插入触发链面 + runner pin 面；下一候选同流同面，
> 间隙开放。
>
> **▶ 上轮状态（2026-10-28 M2 切片 119 施工收口，HTMLFormElement 索引命名属性臂轮——其下一步 s120 已由 s120 收口，见上）**：
> **入场定谳**：s118 候选一定刀 031「focus and blur events」1F——corpus 探针两
> 轮（插桩断言面带运行时值，备份 /tmp 验后还原）：①eventOrder 全零 + 'end
> script #1' 缺席 → script #1 中途抛异常，嫌疑非 focus 派发链；②逐句 try 定谳
> `forms[0][0]` undefined → TypeError。静态对照：焦点面成熟（R114/R148/s29/
> R4003b），R3254-K3 FORM named access 只匹配 id/name——canonical 数字索引臂
> 缺失（spec：无 id/name listed 控件按 form.elements 序号可寻址）。
> **本轮一刀**（净变更 part04 + testharness.rs pin ×1）：FORM named access 块
> id/name 循环后追加 canonical 数字索引臂（parseInt + String 归一判据同
> elements 集合 get 臂；命中返 `_formControls(sel)[idx]` 与 elements[N] 同源
> 恒等；越界 undefined 落通用路径；命名先行索引兜底）。焦点/点击派发面零改动。
> **验证**：定向 `FILTER=execution-timing/0` **031 F→P** 全行 diff 唯一变化
> （72P+5F+1T→73P+4F+1T）；**全量 A/B** 6610P/1188F/105T→**6612P/1186F/105T**
> diff 两行对 = 031 + 同面专测 **form-indexed-element.html** 意外翻绿 → **真翻
> 红 0**。bin pin 旧 shim stash 往返红（form[0] undefined → addEventListener
> TypeError 行为失败）→ 绿；engine lib **3570/0**（与全量套件并发曾撞
> artifact，串行重跑过——双 cargo 并发注意排队）；guard 内 clippy 零告警；
> fmt、shim 拼接语法过。渲染面零依赖——reftest/product-smoke 未跑；make test
> 未跑（Rust 增量仅 test 模块）。账本见
> [evidence/2026-10-28-m2s119-form-indexed-access.md](html-semantics-compat/evidence/2026-10-28-m2s119-form-indexed-access.md)。
> **范围**：engine shim part04 + runner testharness.rs；语料零改动（探针验后
> 还原）。
> **挂账新增**：无。承袭：写流执行身份盖章断链 FIXME、part02 setAttribute 绕
> notify 臂（gap 2）、execution-timing 构成面 056/063+051/052 单根（增量解析
> 大基建）、041 nested write（写流队列重构）、FIXME(spec-gap) iah/clone
> fragment 章、css-module 族 corpus 门（待决须用户拍板）、www2 可达性、
> modulepreload referrerpolicy 族、module-import-referrer TLA、
> script-crossorigin-network cookies、redirect-relative-urls、export-from with
> FIXME、script-charset-02/03、143 write、defer 多写完成序、dynamic import()
> with 透传、classic 外链 SRI、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；
> **待决（须用户拍板）**：css/text 数据模块语料资源缺失。
>
> **▶ 上轮状态（2026-10-28 M2 切片 118 施工收口，innerHTML 产物惰性章 + location javascript: 求值面轮——其下一步 s119 已由 s119 收口，见上）**：
> **入场定谳**：execution-timing 余 F 复盘三面并审——033「innerHTML scripts moved
> in DOM」1F（`div.innerHTML` 产物 appendChild 移入 body 后全执行，spec fragment
> parsing already-started 面缺失）；028/079「location javascript: URL」2F（三入口
> 缺 S4U 同款拦截，体从未求值）；排除 041（写流队列重构挂账）/031（焦点事件面
> 单列）。
> **本轮两刀**（净变更 part02/part03/part04 + testharness.rs pin ×3）：①part03
> helper 顶部 fragment 章门（`el.__zwFragNoExec` 对象属性键）+ part04 innerHTML
> setter `_ihAdded` 深走打章 + **R368 盖 handle 章点章转移 run-once 表**
> （`_zwRanScripts[handle]`）②part02 `_navLocationJavascript` 共用 helper +
> `_locationReplace`/`_locationAssign`/`_setLocationPart` href 臂三入口门。
> **首刀教训**：marker Set 对象键在 R368 身份盖章点断链（`_zwMEl` append 时无
> 身份 → 盖 `__zwHandle` → notify 链换 handle 代理），pin 首跑红定谳；章键必须
> 落在移动换代后的可寻址身份（handle）上——与 s117「章键落执行时身份」结论同构。
> **验证**：定向 `FILTER=execution-timing/0` **028/033/079 F→P**、全行归一 diff
> 唯一变化此三页（69P+8F+1T→72P+5F+1T）；**全量 A/B** 6607P/1191F/105T→
> 6610P/1188F/105T，diff 仅三页翻绿 → **真翻红 0**（forms jsurl 相邻页无翻转，
> location 门与 form submit 路径无交集）。bin pin ×3 旧 shim stash 往返 **3/3 红**
> （行为失败：体缺席/产物执行）→ 新 shim 3/3 绿；engine lib **3570/0**；
> guarded-clippy（runner --all-targets + engine）零告警；fmt、shim 拼接语法过。
> 渲染面零依赖——reftest/product-smoke 未跑；make test 未跑（Rust 增量仅 test
> 模块，shim 面由 engine lib + 全量 corpus 覆盖）。账本见
> [evidence/2026-10-28-m2s118-script-move-inert-and-location-jsurl.md](html-semantics-compat/evidence/2026-10-28-m2s118-script-move-inert-and-location-jsurl.md)。
> **范围**：engine shim part02/part03/part04 + runner testharness.rs；语料零改动
> （本轮无探针插桩——pin 面即判据）。
> **挂账新增**：031 focus/blur（焦点基建面单列）；FIXME(spec-gap) iah/clone
> fragment 章（同 spec 面、033 需求只覆盖 innerHTML setter，待语料需求补）；
> html/browsers javascript-url 族页域外无 A/B 覆盖（S4U 先例背书，browsers 域
> 跑时复核）。承袭：写流执行身份盖章断链 FIXME、part02 setAttribute 绕 notify
> 臂（gap 2）、execution-timing 构成面 056/063+051/052 单根（增量解析大基建）、
> 041 nested write（写流队列重构）、css-module 族 corpus 门（待决须用户拍板）、
> www2 可达性、modulepreload referrerpolicy 族、module-import-referrer TLA、
> script-crossorigin-network cookies、redirect-relative-urls、export-from with
> FIXME、script-charset-02/03、143 write、defer 多写完成序、dynamic import()
> with 透传、classic 外链 SRI、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；
> **待决（须用户拍板）**：css/text 数据模块语料资源缺失。
> **下一步**：s119 入场定刀位（候选：031 焦点事件面 / image-loading-lazy
> deferred 1F / 承袭挂账序；构成面与写流队列挂大基建，不作常规刀位）。
> **切片间隙**：本轮 shim 触发链/导航入口面 + runner pin 面；下一候选同流同面，
> 间隙开放。
>
> **▶ 上轮状态（2026-10-28 M2 切片 117 施工收口，连接态 sel 形 SCRIPT src 后设 prepare 触发面轮——其下一步 s118 已由 s118 收口，见上）**：
> **入场定谳**：026「doc write added script, .src set later」1F——corpus 探针 7 轮
> （断言面带出运行时值，验后手动还原）+ 静态读定谳三缺口：①connected 假阴性
> （`_zwScriptPrepConnected` 直信 getter false；sel 形 isConnected 走宿主快照查询
> `__zw_contains`，执行期 doc.write 插入只在 mut-tree，探针 `conn:false`/`pnode:1`
> 实证）②part05 s96 IDL setter `handle &&` 门注解理由对 doc.write 空脚本不成立
> （解析期空源 bail 未置 already-started）③part02 mut-tree setAttribute 绕过
> `_mo_notify` script 臂（gap 2，setAttribute 形不触发，挂账）。
> **027 对偶定谳**（首刀直扩 s96 sel 形臂 → 027 真翻红回滚重定刀）：`scripts[4]`
> 即 doc.write 带内容脚本；写流面执行对象是**回放重建 plain wrapper**（body 子元素
> 探针全 `nosel`——`__zwSelector`/`__zwHandle` 空），already-started 章落不了；
> 换代重注册后 src-set 读到新 proxy（sel `nth-child(4)`）与执行时身份断链 → sel
> 盖章路线不可行（补章后 027 仍红实证、回滚）。附带定谳：IDL set 实际触发点 =
> 属性写→`__zw_set_attr` 记录→`_mo_notify` 属性臂（s96 直呼冗余；baseline 027
> 不翻红正是旧 connected 假阴性同挡属性臂）。
> **本轮三刀**（净变更 part01/part03/part05 + testharness.rs pin）：①part03
> connected——true 捷径保留，false 落 parentNode 爬升复核（可达宿主连接祖先即
> 连接），爬升不可得再落身份反链（undefined 路径原序不变；detached 仍 false）
> ②part01 s109 属性臂——sel 形**非空体跳过门**（非空体必已随投递/写流执行过，
> src/href 后写按 already-started 语义；handle 形与 childList 臂不动——145 形
> textContent= 依赖非空体照常执行；FIXME(architecture) 增量解析落地回归表判）
> ③part05 s96 注解修正（触发代码回原形）。
> **验证**：定向 `FILTER=execution-timing/0` 026 F→P、027/048/051/052 维持 P、
> 归一 diff 唯一变化 = 026；**全量 A/B** 6606P/1192F/105T→6607P/1191F/105T，
> 全行归一 diff 仅 026 一行 → **真翻红 0**（名字级交集会伪报同页混合状态子测试
> 大批翻红——全行 diff 权威，工具坑记录）。bin pin ×2（empty 红转绿 stash 往返 +
> content 对偶）＋既有 319 过；engine lib **3570/0**；workspace guarded-clippy、
> fmt、shim 拼接语法（真实装配序）过。渲染面零依赖——reftest/product-smoke 未
> 跑；make test 未跑（Rust 增量仅 test 模块，shim 面由 engine lib + 全量 corpus
> 覆盖）。账本见
> [evidence/2026-10-28-m2s117-script-src-prepare-sel-form.md](html-semantics-compat/evidence/2026-10-28-m2s117-script-src-prepare-sel-form.md)。
> **范围**：engine shim part01/part03/part05 + runner testharness.rs；语料不可改
> 维持（026/027 探针验后还原）。
> **挂账新增**：写流执行身份盖章断链 FIXME（体空性门近似，增量解析后回归表判）；
> part02 mut-tree setAttribute 绕 notify 臂（gap 2 明确化）；**execution-timing
> 构成面 056/063 F+T 与 051/052 P 单根**（eager-parse 全树可见性 vs report 元素
> 移除构成差，两页组无法同修、须增量解析——大基建项挂账）；css-module 族全脸
> corpus 门（待决须用户拍板，承袭 s116 确认）。承袭：www2 可达性、modulepreload
> referrerpolicy 族、module-import-referrer TLA、script-crossorigin-network
> cookies、redirect-relative-urls、export-from with FIXME、script-charset-02/03、
> 143 write、defer 多写完成序、063、dynamic import() with 透传、classic 外链 SRI、
> 接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；**待决（须用户拍板）**：css/text
> 数据模块语料资源缺失。
> **下一步**：s118 入场定刀位（候选：execution-timing 余 F 复盘定刀 /
> image-loading-lazy deferred 1F / 承袭挂账序；构成面挂增量解析大基建，不作
> 常规刀位）。
> **切片间隙**：本轮 shim 触发链面 + runner pin 面；下一候选同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-28 M2 切片 116 施工收口，referrer 族 JS checker 轮——其下一步 s117 已由 s117 收口，见上）**：
> **入场定谳**：referrer 族判据链完整（上游 handler GitHub Contents API 实读 →
> Fetch determine-request's-referrer → Referrer-Policy 交付链），credentials 12F
> （cookies 基建 + www2）与 modulepreload 族（echo-referrer.py server-state + link
> preload 面）入场即排除挂账。**本轮刀面**：①`referrer-checker.py` 端点仿真
> （八策略 Referer 计算表 + 降级 + 同源三元组；引用源 = fetcher 首参逐层穿
> parent；host 别名窄收 www1、**www2 保持不可达**——fetch-error 判据页依赖其必败）
> ②window.origin（part01，configurable accessor 可遮蔽）③split_statements
> `from` 续行守卫（多行 import 跨行劈裂 → 提取/变换双死）④页 HTML wptserve
> 变量替换主文档通道首接 ⑤**端口替换值对齐上游 get-host-info 真值**（http
> [8000,8081] / https[8444,8443]——[0] 非默认端口下字面量与 new URL 序列化两
> 判据形归一；SW fixture 专用替换 + origin 校验同步防脑裂）⑥policy 链逐臂过滤
> （invalid pipe 顺延 doc policy）⑦export-from 再导出补进图收集提取（纯 re-export
> wrapper 链接期查空 → tests:0 根因；三 import+局部 const wrapper 因导出表直查
> 幸免——形态差异即对照；s107 pin 契约同步修订，with 属性解析 FIXME）⑧
> SCRIPT.referrerPolicy 反射门（R5009 片 e tag 门）。
> **验证**：定向 referrer **0P+15F+12T→41P+5F+6T**（6 referrer 页 + descendants
> 9/9 + strict-policies 全绿；idl 1F→P）；余 5F6T 全挂账面。**全量 A/B**：
> 6565P/1202F/111T→6606P/1192F/105T，真翻红（基线 Pass × 当轮 F/T 交集）= **0**；
> +41P 全落 referrer/idl。两红形状漂移定谳为本轮修复良性后果：
> script-crossorigin-network F→T（基线 `{{ports[http][1]}}` 裸 token 落 JS 代码位 =
> 语法错误页级 Fail；页替换后首次真跑、credentials 断言等待——页内二分实证与
> 端口值无关）、css-module/redirect-relative-urls T→F（新断言面暴露 1F）。
> sandbox lib **376/0**（+6 钉/修 1 契约）、runner bin **243/0**；shim 拼接语法
>（真实装配序 part01+01b+02..07）、fmt、guarded-clippy 过；`make test` 三跑：
> 首两跑 image_batch_e2e 1F（**zero-browser bin 被 target-disk-guard 清理**——
> 根因：全量测试构建把 target/ 堆到 76GB 超 50GB 阈值 → tier-3 全量清空 →
> make test 自身 build 行不含 zero-browser → 必败；环境副作用非行为失败）；
> 第三跑（target/ 重置 33G 后守卫放行）**90/91 suite 绿**——唯一失败
> zero-renderer lib `page_scripts::tests::mutation_arm_does_not_apply_across_
> document_reset_window` 满载时序 flake（2026-10-25 aa66d1530 引入的滞留窗
> 观测断言，满载 340.93s vs 单独 19.76s；单独复跑 1.63s 绿 + 守卫包全
> renderer lib **352/0** 绿——非本切片面，预存挂账）。
> 渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-28-m2s116-referrer-checker-referer.md](html-semantics-compat/evidence/2026-10-28-m2s116-referrer-checker-referer.md)。
> **范围**：runner testharness.rs + script-sandbox es_module.rs + engine shim
> part01/part04；webview 图注册点仅 DBG 探针已撤（净零变更）；语料不可改维持。
> **挂账新增**：www2 host 可达性（CORS+cookies 基建前置，放行须同批复核
> dynamic-imports-fetch-error 翻转）；modulepreload referrerpolicy 族 3F+2T
>（echo-referrer.py server-state + link rel=modulepreload 取回面）；
> module-import-referrer T（TLA 面未建模 + 动态 import referrer 语义）；
> text/css/json-module referrer-policies 3T（数据模块 checker 体——**并入待决
> 须用户拍板**）；script-crossorigin-network T（cookies/credentials 基建）；
> css-module/redirect-relative-urls 1F（动态 CSS module 重定向相对 URL 解析基）；
> export-from with 属性解析 FIXME。承袭：script-charset-02/03、143 write
> `</script>`、defer 多写完成序、063 Timeout、non-external-no-import、
> parse-error 族、http-accept-header、dynamic import() with 透传、classic 外链
> SRI、collect 账本打包重构、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt；
> **待决（须用户拍板）**：css/text 数据模块语料资源缺失。
> **下一步**：s117 入场定刀位（候选：execution-timing 余 F 定刀 /
> css-module redirect-relative-urls 1F / image-loading-lazy deferred 1F /
> 承袭挂账序）。
> **切片间隙**：本轮 runner fetcher/替换面 + sandbox 图提取面 + shim 反射门；
> 下一候选同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-28 M2 切片 115 施工收口，script charset 响应解码轮——其下一步 s116 referrer 族已由 s116 收口，见上）**：
> **入场定谳**：charset 族五组失败三根因——①`serve-with-content-type.py` 动态
> 端点不在语料 pull 面：classic 面文件读落空→整段透传外链→宿主取回 Err 不执行
>（script-charset-01 全 6F）；模块面 s107 仿真被 JSON 意图门收窄，非 JSON ct →
> Err 网络错误（module/charset-01/02/03 全 11F）。②文档编码 windows-1250 未建模
>（script-charset-02 3F）。③运行期 charset 属性变脸无元素上下文（03 1F）。
> ②③挂账；css/text-module charset 5T 属待决（须用户拍板）。
> **spec 分流**：classic 解码序 BOM > ct charset 参数 > charset 属性 > 文档编码
>（runner 恒 UTF-8）——parser classic 走 inline_local_scripts（元素上下文在位）；
> module 恒 UTF-8（charset 参数/属性/BOM 三忽略——charset-01 判据即
> `assert_not_equals` mojibake；charset-02 UTF-8 BOM 体内 U+FEFF 为 JS 空白照常
> 执行、UTF-16 乱码语法错）——模块走 wpt_data_script_fetcher。
> **本轮三刀**：①共享查询解析 helper（fn/ct 百分号解码）②fetcher scripty 意图
> 分支（非 JSON 而 scripty → 交付 fn 文件恒 UTF-8 lossy，BOM 不嗅探；非 scripty
> 非 JSON → Err s107 口径）③inline classic 仿真（非 module 型 + 文件名匹配；
> decode_classic_script_bytes 四级解码序）。**首版乌龙**：inline 未辨 module——
> module 外链被 charset 感知内联（charset-01 余 4F 反向翻红、charset-02
> 1F→2F：BOM 嗅探令 UTF-16 模块照常执行）；修正 module 整段透传（ASCII 剥白 +
> ASCII-ci 本地判定，不扩 engine 公共面）。
> **验证**：runner bin **240/0**（+2 钉）。charset 族定向：四页 **17F 全翻绿**
>（script-01 6P / module-01 6P / module-02 3P / module-03 4P）；挂账三组形态
> 不动。**全量套件 A/B**（vs s114 基线）：6545P/1220F/112T→6565P/1202F/111T，
> 归一 diff = charset 17F→P + focus flake 复原（1T+77P→80P）+ modulepreload
> 假行消退（3F+1T→2F+1T 常态，与 s113 基线/定向重跑一致）——**零 P→非P 成立**。
> fmt + clippy（runner --all-targets -D warnings）过。渲染面零依赖——
> reftest/product-smoke 未跑。账本见
> [evidence/2026-10-28-m2s115-script-charset-decode.md](html-semantics-compat/evidence/2026-10-28-m2s115-script-charset-decode.md)。
> **范围**：runner testharness.rs + Cargo.toml（encoding_rs workspace 既有）；
> engine/webview/语料零变更；语料不可改维持。
> **挂账**：script-charset-02（文档编码 windows-1250——corpus loader 编码面
> 基建）；script-charset-03（运行期 charset 属性——shim fetch 桥带提示，跨界
> 契约）；css/text-module charset 5T（待决须用户拍板）；classic 运行时取回无
> charset 上下文恒 UTF-8 近似（语料无此形）。承袭：143 write 注入 `</script>`、
> defer 多写入完成序近似、063 Timeout、non-external-no-import、parse-error 族、
> http-accept-header、referrer-policies、dynamic import() with 透传、classic
> 外链 SRI、collect 账本打包重构、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt
> 插桩来源待查；**待决（须用户拍板）**：css/text 数据模块语料资源缺失。
> **下一步**：s116 入场定刀位（候选：execution-timing 余 F 定刀 /
> dynamic-import credentials 12F / referrer-policy 族 9F / 承袭挂账序）。
> **切片间隙**：本轮 runner 取回/内联面；下一候选同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 114 施工收口，write 面 defer 改道 + xlink:href 回退轮——其下一步 s115 charset 族已由 s115 收口，见上）**：
> **入场定谳**：三候选各 1F——096 write defer 外链走「取回即执行」阻塞链早执行
>（got `[#1, external#1, #2]`）；142 write 只读 `src`，SVG xlink:href 写入形外链
> 不执行（缺 `external script #1`）；143 write 注入多余 `</script>` 须关外层 SVG
> script 提前执行——write 片段解析与主解析器 open-element 栈耦合，挂账不动。
> write+defer 语料全树仅 096（102 命中 grep 但 defer 只在标题文字）；
> xlink:href 写入形仅 142，爆炸半径封闭。
> **本轮两刀**（shim part06.js `__zwDocWriteRunFace`）：①defer 改道臂——立即
> fetch（挂 s110 在飞计数挂 load 等待）、代码入 FIFO 队、排水投
> `__zwSchedRegister('defer',0,fn)`（s112 布防实时查 `__zwSchedHasDeferred()`，
> 运行期注册可见；排水同任务派 DCL = spec the-end 序；慢源迟到 `DeferDrained`
> 补执行；nomodule/s113 classify 门保持；throwaway 槽位仍留主链；非 runner
> 环境维持阻塞链）②src 读空回退 `xlink:href` + 副本补 `setAttribute('src')`
> ——既有 fetch 链零改动。③open() 三记账随 s97 同步重置。
> **验证**：定向 096/142 Pass。execution-timing 全子树 A/B（150 页）：唯一变化
> 096/142 F→P、行数恒等、零 P→非P。**全量套件 A/B**：6546P/1221F/111T→
> 6545P/1220F/112T 行数对账 7878→7877 无蒸发页；另两处异常定性负载 flake
> （focus 3 P→T 定向重跑 80P/0T 复原；modulepreload 基线即 2F+1T 满量漂 3F
> 重跑复原）——**全树零 P→非P 成立**。Rust/依赖零变更，无新增 fmt/clippy 面。
> 渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s114-write-defer-and-xlink-href.md](html-semantics-compat/evidence/2026-10-27-m2s114-write-defer-and-xlink-href.md)。
> **范围**：shim part06.js 单文件（+51 行）；engine Rust/webview/语料/依赖零变更；
> 语料不可改维持。
> **挂账**：143 深层面（write 注入 `</script>` 关外层 SVG script + SVG script
> 文本拼接求值）；defer 改道近似——多 defer 写入按取回完成序近似文档序、
> interactive 后注册无二次布防（语料均无此形）；part06.js 18436 行既有超限
>（拆分需独立切片）。承袭：063 Timeout、non-external-no-import（module async）、
> parse-error 族、http-accept-header、referrer-policies、dynamic import() with
> 透传、classic 外链 SRI、collect 账本打包重构、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt 插桩来源待查；**待决（须用户拍板）**：css/text 数据模块语料
> 资源缺失。
> **下一步**：s115 入场定刀位（候选：143 深层面评估 / execution-timing 余 F
> 定刀 / 承袭挂账序）。
> **切片间隙**：本轮 engine shim 面；下一候选同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 113 施工收口，type/language essence 16 集合轮——其下一步 s114 write 面族已由 s114 收口，见上）**：
> **入场定谳**：s113 承 s112 解锁页 script-type-and-language-js（456 子测
> 229P/227F）。227 失败全为「shouldn't run」判据：type 臂 53 + language 臂 173 +
> 解析插入面 1（type="javascript" 被旧 `ends_with("javascript")` 误跑）。四动态
> 执行臂门况盘点：maybe-start 旧 `indexOf('javascript')` 子串 lax 门；R387 三臂/
> s95 children-changed/`_zwPrepareSrcScriptGate` **全无 type 门**。
> **spec 模型**（live spec 提取）：type string 推导——type 字面空串/无 type 且
> language 字面空串/双缺 → "text/javascript"；type 在场取原值；否则
> "text/"+language **原值不剥白**。分类：type string essence match（HTML
> §scripting languages 16 项 legacy 全表 ASCII-ci **等值、无参数语法**——
> with-params 语料 text/javascript;charset 不跑）→ classic；type 属性在场加
> **第二替代**：属性值剥边缘 ASCII ws 后 match（language 推导无此替代——
> " javascript" 不中，空白形不跑）；module；余 data block。
> **本轮两刀**：①extract.rs `script_type_is_javascript` 收窄 16 集合等值表
> （字面空串/剥后空串分轨；module 剥白口径不动，s93 钉保持）；②shim
> `_zwScriptTypeClassify`（推导+essence 双轨）+ blocked 门接入四臂（blocked 不执行
> 不置 run-once，摘属性重走门；module 沿用各臂既有让位）。
> **中间态一乌龙**：首版对 language 推导串也做边缘剥白——"text/javascript " 剥后
> 中集合误跑（family 余 60F 全空白形）；对照 spec 双轨文本修正后 family 463P/0F。
> essence 语义与 mimesniff MIME 解析不同：是字符串比对，参数串整体不中。
> **验证**：engine lib **3564/0**（含 s113 essence 钉）；webview **814/0**；runner
> bin **238/0**。**全量套件 A/B**（stash 前/后二进制同协议）：7900 子测行数恒等，
> 归一 per-page diff **唯一变化**即目标页 227F+229P→456P，**全树零 P→非P**
>（/tmp/s113-full-base.log、/tmp/s113-full-suite.log 存档）。fmt + clippy
>（engine/wpt-runner -D warnings）过。渲染面零依赖——reftest/product-smoke 未跑。
> 账本见
> [evidence/2026-10-27-m2s113-type-language-essence.md](html-semantics-compat/evidence/2026-10-27-m2s113-type-language-essence.md)。
> **范围**：engine extract.rs + shim part03/part04；webview/语料/依赖零变更；
> 语料不可改维持。
> **挂账**：解析面 language 属性未建模（无语料需求，遇语料再补）；shim module
> 判定剥白口径与 spec 精确等值差（无语料判据不动）；js-svg/-xhtml 变体不在
> runner 枚举面（SVG language 豁免未验证）；096/142/143（write 面族）、063
> Timeout、non-external-no-import（module async）；承袭：parse-error 族、
> http-accept-header、referrer-policies、dynamic import() with 透传、classic 外链
> SRI、collect 账本打包重构、接口对象基建面、FIXME 组、/tmp/zw-hb*.txt 插桩来源
> 待查；**待决（须用户拍板）**：css/text 数据模块语料资源缺失。
> **下一步**：s114 入场定刀位（候选：142 write 臂 xlink:href 单点 / 143
> eventOrder 核验 / 096 write(defer)）。
> **切片间隙**：本轮 engine shim+提取面；下一候选同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 112 施工收口，解析期外链时序调度轮——其下一步 s113 type/language 缺口已由 s113 收口，见上）**：
> **入场定谳**：s112 承挂账「trickle 建模 ~11 行」——探针实证 trickle 族实为十页
> 而非六页，且分三根因：①调度缺失族八页（050/085/086/088/103/110/111/112）——
> runner `inline_local_scripts` 解析期外链一律原位内联直执，async/defer/trickle
> 时序语义塌缩为「立即同步」；②投递截断族——099（include-11.js srcdoc 串含字面
> `</script`，投递面提前封口 SyntaxError 零日志）+ script-type-and-language-js.js
> 43-45 行**注释内** `</script>` 同根因（`testParserInsertedDidNotRun` 定义于注释
> 后未注册，页面首内联 ReferenceError 短路，456 子测不可见）；③write 面族三页
> 不在本轮（096/142/143）。
> **本轮两刀**（单文件：tests/wpt-runner/src/testharness.rs）：①分类器
> `script_sched_unit`（预处理时刻冻结——spec prepare 定格，运行期 removeAttribute
> 不重分类；nomodule/nonce/module 维持原路径）+ 页面侧调度器
> `__zwSchedRegister`：async=trickle 后 setTimeout；defer 入 deferred 队列，
> `__zwSchedArmDrain` 于 interactive 后 DCL 前统一 drain（文档序）；block=trickle
> 后执行（解析阻塞近似，判据页只断言 log 序可观察面等价）。接线：drain 臂装在
> chain task1 **内部**（setTimeout FIFO 保 interactive 面先跑，103 断言依赖此序），
> DCL 派发幂等化改 drain 回调，load 门加 `__zwSchedPending()`；②投递体内
> `</script` → `<\/script` 转义（字符串/正则/注释 `\`+`/` 等值，
> any_js_window_wrapper 同口径）。
> **中间态回归（一乌龙）**：首轮终扫 load-error-events-1 四子测 Pass→Timeout——
> 调度单元 load/error 投递尾读 `document.currentScript`，相位执行时已清（getter
> 返 null，part06 R3258）事件永不派发。修正：注册语句同步原位执行时捕获
> `_zwEl` 传入闭包，投递尾 `script_event_dispatch_tail_el`。体内 currentScript
> 读取仍 null（挂账）。
> **验证**：runner bin **238/0**（236 基线 + s112 分类/转义两钉）。execution-timing
> 定向 22F→13F 九页转绿零 P→非P。全子树终扫 **638P/367F/37T+15N**（+240 行）：
> 十一页 Fail→Pass（上述九页 + script-defer——defer 不再抢跑 t.step 前断言 +
> delay-load-event-1——trickle(d3) 不再原位直执）；script-type-and-language-js
> 解锁 456 子测全注册（229P/227F，页保持 Fail——227 为先前不可见的真实
> type/language 分类缺口，候选独立切片）。归一 diff（/tmp/s112-final-scripting-1.log
> 存档）除上述外零变化。fmt + clippy（-p zero-wpt-runner --all-targets -D warnings）
> 过——as_deref 误加被 needless_option_as_deref 抓回（src 本就是 `Option<&str>`
> Copy）。渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s112-parser-sched-and-body-escape.md](html-semantics-compat/evidence/2026-10-27-m2s112-parser-sched-and-body-escape.md)。
> **范围**：仅 runner 投递/生命周期面；engine/webview/语料零变更；语料不可改维持。
> **挂账**：调度单元体内 currentScript 读取（需 shim idx 通道，遇实证再立项）；
> block 类不真暂停解析（050 等价仅判据面）；script-type-and-language-js 227 真实
> 缺口；096/142/143（write 面族原挂账）；063 Timeout；non-external-no-import
> （module async）；承袭：parse-error 族、http-accept-header、referrer-policies、
> dynamic import() with 透传、classic 外链 SRI、collect 账本打包重构、接口对象
> 基建面、FIXME 组、/tmp/zw-hb*.txt 插桩来源待查；**待决（须用户拍板）**：css/text
> 数据模块语料资源缺失。
> **下一步**：s113 入场定刀位（候选：script-type-and-language-js type/language
> 分类缺口 / 142 write 臂 xlink:href 单点 / 143 eventOrder 核验 / 096 write(defer)）。
> **切片间隙**：本轮 runner 投递面；下一候选同流，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 111 施工收口，原子插入语义轮——其下一步 s112 trickle 建模已由 s112 收口，见上）**：
> **入场定谳**：s111 承 147/148——fragment 内 3 工厂 inline script，#1 执行时
> `getElementById` 同批后继兄弟（147 移动/148 删除），修前报 `Cannot read
> properties of null (reading 'parentNode')`。四轮探针定谳双根因：①R387 直插臂无
> connected 门（s96 注释「连接态判定归调用方」，调用方从未补）——`frag.appendChild`
> 期 detached eval + 置 run-once，容器插入时被记账挡掉（npReg trace 实锤 gEBI 夹在
> fragment 内部追加与 s2 追加之间）；②s109 addedNodes 臂挂 notify 头部先于
> `_zwNodeParent` 链接记账——后继兄弟反链未指向真实插入目标，`_zwMutationInDoc`
> 上行失败 getElementById miss。spec 模型（spec-scripting.html 提取）：post-connection
> steps 只在插入元素仍连接时运行（HTML spec 自带示例与 148 同构）；「atomically
> inserting nodes」先完成全部 generic insert 再按文档序逐 script prepare；insert
> 遍历 inclusive descendants。
> **本轮三刀**：①part01 臂移位——addedNodes script 臂移至链接记账之后（批内前置
> 脚本执行时后继反链已登记）+ 非 SCRIPT 元素子树下降（文档序递归，129 形 div 内
> script）；②part03 `_zwScriptPrepConnected` 统一连接判定（proxy isConnected 布尔
> 直用；plain/盖章 plain 子——R368 只贴 `__zwHandle` 无 get trap——属性 undefined
> 回落身份反链 + parentNode 链爬升）；③part04 R387/R387b 三臂前置 connected 门，
> 未连接不置 run-once（重插入重走门）。
> **调试定谳（本轮两乌龙）**：①链式合并改分支语义——R387 原为「独立 if + if/else
> 对」三平铺语句，接成单链后 R377 回落臂命中吞掉外链 else 臂；平铺语句合并
> else-if 链前必须核对条件互斥性。②臂移位隐能化——旧位臂先于链接记账，maybe-start
> connected 门对刚插入节点恒 bail，外链 prepare 实际全靠 R387b；移位后 notify 臂
> 首次触达插入态外链，helper 只认 `__zwHostOwnsDynamicScripts` 不认 R387b 的
> `__zwHostDynamicScripts` → engine `r387b2` 钉抓到双执行（fetchCalls "2"≠"1"），
> 外链分支补同款让位（不置记账）后转绿。移位使原本「意外失活」的路径生效时，
> 排查该路径上所有让位契约。
> **验证**：engine lib **3548/3548**（r387b2 回归钉）；webview lib **814/814**；
> runner bin **236/0**（s109/s110 pin 保持）。execution-timing 定向扫页集合 vs
> s110 基线零 P→非P（117-119/129 首轮回归经子树下降+helper 修复回绿）。全子树
> 终扫 **398P/152F/37T**（+3 行/3 页），归一 diff（/tmp/s111-scripting-1b.log 存档）
> 恰为 049/147/148 三页 Fail→Pass。本轮零 .rs 变更（shim 全 JS + 文档），
> rustfmt/clippy 按实际影响跳过。渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s111-atomic-insert-and-connected-gate.md](html-semantics-compat/evidence/2026-10-27-m2s111-atomic-insert-and-connected-gate.md)。
> **范围**：engine shim part01/part03/part04；webview/语料/依赖零变更；语料不可改
> 维持（zz111/zz111b 探针页已删）。
> **挂账**：050/085/086/088/111/112（slow-loading 外链时序 pipe=trickle 建模
> ~11 行）；142 write 臂 xlink:href；143 eventOrder 完整核验；063 Timeout；
> non-external-no-import（module async）；**待决（须用户拍板）**：css/text 数据
> 模块语料资源缺失；承袭：parse-error 族、http-accept-header、referrer-policies、
> dynamic import() with 透传、classic 外链 SRI、collect 账本打包重构、接口对象
> 基建面、FIXME 组、/tmp/zw-hb*.txt 插桩来源待查。
> execution-timing 049/147/148 本轮销账（s110 销账面不变）。
> **下一步**：s112 入场定刀位（候选：050/085/086/088/111/112 trickle 建模 /
> 142 write 臂 xlink:href 单点 / 143 eventOrder 核验）。
> **切片间隙**：本轮 engine shim 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 108 施工收口，strict 首错聚合 + 短路收窄 + id 保真轮——其下一步 s109 入场探针定刀位已由 s109 收口，见上）**：
> **入场定谳**：s108 探针承 s107 三候选——候选 A（css/text 数据模块合成器）**证伪**：
> wpt-data（独立 repo，pinned tag）缺 css-module/resources/basic.css 等五个 CSS 资源
> 与 text-module/file.txt，fetch 恒 404，切片不可达；补语料属基准变更（guidelines
> #31 语料红线）记待决清单，s107「~30 行」估计据此修正。execution-timing 余 46F+1T
> 按根因分组定刀：①122-126 族 getElementById null（runner 装配丢 id）；④143 语料
> 故意 SyntaxError 被 runner 升级页级 Fail；②mutation 重触发机制面（②七行留 s109）、
> ③142 write 臂 xlink:href 不本轮。
> **本轮三刀**：①runner 投递 id 保真（`inline_local_scripts` 投递标签保留原标签 id，
> 与 nonce/nomodule/type/onload/onerror 同族「内联投递丢属性改语义」保真——122-126 族
> 以 getElementById("test") 取回 type=text/plain 外链单元做 mutation 重触发）；②runner
> 短路收窄（脚本抛错臂读 declared：==0 维持短路 Fail 防假绿、>0 放行轮询收集——WPT
> 语义 uncaught exception 报 window.onerror 不降级 test status；143 抛错时刻 pending=1
> 属正常中间态，故判据用 declared>0 非 pending==0）；③webview strict abort→首错聚合
> （两处 return Err 改 get_or_insert_with + continue——真浏览器语义 script 元素相互
> 独立；循环末聚合返 Err，Err 仅承载信息供 runner 防假绿判定）。**配套重构**：收尾段
> 单一出口——旧 `!pending` 分支在聚合判定前 `return Ok` 早退，无 DOM 变更的纯断言页
> （execution-timing 主体形态）首错被静默吞掉（新钉首轮红灯根因）；改 if/else 汇合后
> 统一 drain + 聚合 + 返回。
> **验证**：zero-webview lib **814/814**（+1 钉：三 script 页 is_err 且前后脚本照跑）；
> zero-wpt-runner bin **233/0**（+3 钉：投递 id 保留 / 抛错+declared>0 照收 Pass /
> 抛错+declared==0 维持短路——stateful MINI_HARNESS 变体带 tests 注册表与
> test_environment stub）；fmt clean + clippy webview/wpt-runner `-D warnings` clean。
> 定向 FILTER=execution-timing：非 P 页 **45 → 41**（**4 页销账**：067 语法错
> has-run——刀2+刀2b，语料故意埋错 + allow_uncaught_exception；104/114/115 defer
> mutation 族——刀1，三页均 getElementById("defer-script")）；122 null 崩溃消除进第二
> 层（页级 Fail→2 子测试断言级 Fail：async IDL 属性 + mutation 重触发，挂 s109）；
> 143 崩溃形→断言级 Fail（log 顺序断言真正执行，缺 inline script #3——write 嵌套
> 内联，挂 s109）。全子树终扫（FILTER=scripting-1 口径）**377P/173F/37T+15N**
> （s107：372P/175F/35T+15N，**+5P**），行级归一 diff（/tmp/s108-final-scripting-1.log
> 存档，基线 /tmp/s107-final-scan.log）**零 P→非P**（无 Pass 行消失）：4 Fail→Pass +
> 1 新增子测试 Pass（evaluation-order-1 module 抛错臂——刀2 直接战果）；Fail −2 净额
> 逐项核账；Timeout +2（evaluation-order-1/delay-load-event-1 页级落 deadline——均原
> Fail 页，declared>0 放行后未完成挂 deadline 的预测形）；microtasks/moving-between-
> documents 三处形态重排全为原 Fail 页短路→子测试级分解。渲染面零依赖——
> reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s108-strict-continue-and-id-fidelity.md](html-semantics-compat/evidence/2026-10-27-m2s108-strict-continue-and-id-fidelity.md)。
> **范围**：webview webview.rs ×1（首错聚合 + 单一出口 + 1 钉）+ runner testharness
> ×1（id 保真 + declared 收窄 + 3 钉）；解析器/语料/依赖零变更；语料不可改维持。
> **挂账**：s109 主候选 mutation 重触发机制面七行（122 第二层 async IDL+时序、
> 123/124/125/126、144/145——嵌套执行 + 重扫 diff，共享主循环高风险面，独立切片细
> 设计）；142 write 臂 xlink:href；143 eventOrder 完整核验；063 Timeout（runner 脚本
> 记账）；slow-loading 外链时序 ~11 行（pipe=trickle 建模）；**待决（须用户拍板）**：
> css/text 数据模块语料资源缺失（五 CSS + file.txt 不在 pinned tag）——补语料属基准
> 变更；承袭：parse-error 族、http-accept-header、referrer-policies、dynamic import()
> with 透传、classic 外链 SRI、collect 账本打包重构、接口对象基建面、FIXME 组、
> /tmp/zw-hb*.txt 插桩来源待查。execution-timing 067/104/114/115 本轮销账。
> **下一步**：s109 入场定刀位（候选：mutation 重触发机制面七行体量评估——共享主循环
> 高风险面 / 142 write 臂 xlink:href 单点 / 143 eventOrder 核验搭车）。
> **切片间隙**：本轮 webview 执行面 + runner 收集/投递面 + 单测；下一挂账同流同面，
> 间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 107 施工收口，JSON module 执行面 + 顶层模块 SRI 轮——其下一步 s108 入场探针定刀位已由 s108 收口，见上）**：
> **入场定谳**：三候选——module 域 92 行最大、数据模块域合计 47 行（css 21 + json
> 17 + text 9）、execution-timing 余 46 行 scheduler 长尾异质。JSON module 定最小
> 可收口切片。MIME 合法集按**语料判据**定谳（不凭现行 spec 印象）：essence
> application/json | text/json | +json 后缀、`;参数` 剥离——valid-content-type 8 臂
> 全应成功（现行 spec 已收窄至 application/json + +json 后缀，text/json 是 corpus
> 时代合法集）；`serve-with-content-type.py` 端点本体不在 runner 语料——fetcher 按
> handler 语义模拟 `?fn=&ct=`，与 imported_script.py 模拟同模式。风险面定谳两处：
> ①integrity.html mismatch 行平凡 Pass（fetch 恒败→体不执行）——JSON 放行后必翻
> Fail，**须真实现 SRI** 才能保零 P→非P；②script-element-json-src 负例 src 是
> file.txt 非 .json，解除。
> **本轮四刀**：①fetcher JSON 意图分支（.json 静态文件默认 application/json +
> 端点 fn+ct 模拟，mime_is_json 门：合法交付原文/非法 Err 网络错误语义；ct 百分号
> 解码、裸 `+` 保留——`text/html+json` 裸 + 在 query 不表示空格）；②script-sandbox
> 新 `extract_static_module_imports_with_attributes`（复用 s105
> suffix_after_specifier_literal/parse_with_clause 提 pub(crate)，仅 import 语句，
> export-from/动态 import() 不提取）；③webview collect 链增 type_attr 参数（9 形参
> 函数级 allow + FIXME 挂账），type=json → serde_json 校验（非法→DepFetch/TopFetch
> 网络错误语义）+ 规范化内嵌合成 `export default <字面量>;`（default 导入经既有
> export transform 与导出扫描自动成立）；④顶层模块 SRI：`PageScript::ExternalModule
> { src, integrity }` 结构化（六消费点模式同步）+ webview
> integrity_metadata_matches（空格分隔多 hash 任一命中、algo 大小写不敏感、
> sha256/384/512，base64_encode 启用 + sha2 workspace dep）+ mismatch→元素 fire
> error + 体不执行。附刀：**外链模块成功 load 面**——模块成功后按 src 匹配 fire
> load（此前仅失败面有事件派发），**内联成功不派**（load-error-events-inline 四臂
> assert_unreached 判据；首版内联也派致 2 行 P→F，定向轮即定位改外链限定）；
> runner inline_local_scripts 对带 integrity 属性单元整段透传不内联（内联投递丢
> src/integrity 双锚——SRI 校验与事件 src 匹配都以 src 为锚）。
> **验证**：script-sandbox **373/373**（+带属性提取钉）；engine lib **3541/3541**
> （+integrity 提取钉）；webview lib **813/813**（+5 钉：JSON default 导入执行+
> 内联成功无 load / JSON 语法失败元素 error / SRI 命中执行+load / SRI 失配
> error+体不执行 / 承袭 s105）；wpt-runner fetcher 钉（.json 默认 MIME/端点
> fn+ct/解码/非 JSON 拒/sidecar 覆盖拒）；定向 **FILTER=json-module 26→30 行，
> 9F+2T→13P 零 P→非P**；load-error-events 滤页对照零回归（module/ 族行不变）；
> 全域交叉 integrity 四页（module 1P+2F→3P、json-module 2P、css/text mismatch 行
> 维持 Pass）；全子树终扫（FILTER=scripting-1 口径）**372P/175F/35T+15N**
> （s106：356P/185F/37T+15N，**+16P**），行级归一 diff（/tmp/s107-final-scan.log
> 存档）10 Fail + 2 Timeout → Pass 全正向、**零 P→非P**；fmt clean + clippy 六
> crate clean（renderer/browser/wpt-runner 模式改形波及）。渲染面零依赖——
> reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s107-json-module-sri.md](html-semantics-compat/evidence/2026-10-27-m2s107-json-module-sri.md)。
> **范围**：script-sandbox es_module/import_attributes ×2 + engine extract ×1 +
> webview webview.rs/Cargo.toml ×2（sha2 dep）+ runner testharness ×1 + 六消费点
> 模式改形（browser/renderer/reftest 无行为变化）；解析器/语料零变更；语料不可改
> 维持。
> **挂账**：css/text 数据模块执行面（同一 collect 分派骨架已就位，type=css/text
> 合成器另切片，~30 行）；load-error-events.py 端点 module 路径 3 行；parse-error
> 族 Timeout（parse 失败元素事件模型与 s102 报告面差异）；http-accept-header
> （Accept 头建模 + 动态 import with 子句）；script-element-json-src（顶模块 MIME
> 拒绝→元素 error 事件模型与 s102 冲突待梳理）；referrer-policies；dynamic
> import() with 子句透传（现被丢弃）；export-from with 属性；classic 外链 SRI；
> collect_module_deps_recursive 账本打包重构（FIXME allow 承接）；hash 列表
> options；SVG script 族运行时面 4 页、execution-timing 余 ~42F、charset 族残留、
> svg-script-fragment-clone 3F、FIXME 组、接口对象基建面、/tmp/zw-hb*.txt 插桩
> 来源待查（承袭）。json-module 13 行 + module/integrity 2 行本轮销账。
> **下一步**：s108 入场探针定刀位（候选：css/text 数据模块合成器——本轮分派骨架
> 就位，css-module/text-module ~30 行 / execution-timing 余 ~42F scheduler 语义
> 定刀 / SVG script 运行时面 4 页体量评估）。
> **切片间隙**：本轮 script 管线面（fetch/编译/事件派发）+ 单测；下一挂账同流
> 同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 106 施工收口，SVG script 族静态面轮——其下一步 s107 入场探针定刀位已由 s107 收口，见上）**：
> **入场定谳**：三候选按 fail 形状重新分组——execution-timing 50F 中藏 **SVG script
> 族 8 页**（137/138/139/140/142/143/144/145），138/139/143 为 harness 级失败
> （`page script threw: SyntaxError: Unexpected token '.'`，143 的 `':'` 正是语料
> 故意埋的 `assert_unreached():`），共同根因优先于 scheduler 异质长尾，定刀。
> **根因（探针实证）**：①html5ever 对 foreign content 按 spec **markup 解析**——
> SVG script 内嵌套 `<script>` 为真实子元素（探针 dump 树形）；②提取器取源误用递归
> `text_content`（textContent 语义含后代元素文本），外层源吞入嵌套脚本文本。spec：
> 脚本源文本 = **child text content**（直接子文本节点拼接）；HTML script raw text
> 仅文本子节点，两取法恒等（其余 585 行零影响）。判据：138 期望 [ #1, #2, end#2]
> **end-tag 序**（子脚本先闭合先执行）；139 期望 [external, #1, end#1]；137 期望
> 空 href 不执行体。
> **本轮三刀**：①zero-dom 新 `child_text_content`（直接子 Text 拼接，与递归
> text_content 并列）；②extract 源文本改 child_text_content + 执行序按「pre-order
> 先行者是后行者祖先 → 后行者先行」重排（区间序；this_idx 保持全体 script
> pre-order 序与 shim getElementsByTagName 对齐，执行序与序号解耦）；③SVG 命名空间
> script href 空串跳过不落内联臂（HTML script src="" 维持旧回落不扩大）。族内余页
> 运行时面不触：142 write 注入外链 / 143 write 双 `</script>` 提前闭合外层（语料
> 故意双写）/ 144/145 mutation 改 textContent+type 重触发 prepare。
> **验证**：zero-dom lib **868/868**；engine lib **3540/3540**（+4 钉：嵌套源+序+
> 序号 / 外链嵌套序 / 空 href 双面 / 无嵌套序不变）；webview lib **809/809**；
> 定向 **FILTER=execution-timing 与 s105 基线 diff 恰 4 行（全 F→P）**；全子树终扫
> **356P/185F/37T + 15 NotRun**（s105：352P/189F/37T+15N），行级 diff **恰 4 行**、
> **零 P→非P**（comm 校验）、T/N 持平零旁损；fmt clean + clippy 三 crate clean。
> 诊断路径留档：fail 形状分组 → harness 级三页语料核对 → 探针 dump 双取法源文本
> 实证（探针已删）。渲染面零依赖——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s106-svg-script-family-static-face.md](html-semantics-compat/evidence/2026-10-27-m2s106-svg-script-family-static-face.md)。
> **范围**：document/mod.rs ×1（新访问器）+ extract.rs ×1（三刀 + 4 钉）；解析器/
> 语料/runner 零变更；语料不可改维持。
> **挂账**：SVG script 族运行时面 4 页（142/143/144/145，script 生命周期与 DOM
> 变更联动，另立项）；execution-timing 余 ~42F（scheduler 语义长尾 + 063 runner
> 脚本记账）；数据模块全量支持（fetcher MIME 门 + extractor 类型捕获 + registry
> 合成源 + rewriter）；.py 执行器前置；import attributes 余项（dep 级校验/
> export-from/动态 with）；charset 族（Timeout 形）；svg-script-fragment-clone 3F；
> FIXME 组；接口对象基建面（承袭）；基线扫描宜空载（承袭）；testharness.rs
> /tmp/zw-hb*.txt 插桩来源待查（承袭）。execution-timing 137/138/139/140 本轮销账。
> **下一步**：s107 入场探针定刀位（候选：execution-timing 余 ~42F scheduler 语义
> 定刀 / 数据模块全量支持或 .py 执行器前置体量评估 / SVG script 运行时面 4 页体量
> 评估）。
> **切片间隙**：本轮 zero-dom/engine 面 + 单测；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 105 施工收口，import attributes 静态校验轮——其下一步 s106 入场探针定刀位已由 s106 收口，见上）**：
> **入场定谳**：三候选评估——execution-timing 50F 异质（scheduler 排序长尾 + SVG
> integration-point 解析器面 + 138/139/143 疑似变换自伤）非单根因切片，否；063 =
> runner 脚本记账深坑，否；**import attributes 静态校验 3 Fail 页**最小高信定刀。
> 判据（语料原文核对）：invalid-type-attribute-error（4 外链模块 type 值
> notARealType/""/js/javascript → ev.error.constructor 全 TypeError，
> onerror=unreachable 即元素须 fire load 不 fire error）；unsupported-attribute
> （未知 key → SyntaxError）；invalid-import-errors-order（3 内联模块全
> SyntaxError——属性键错误先于同句/后句 specifier 解析错误与 type 值错误）。
> type 值合法性 = HTML module type map 不注册 "js"/"javascript"（JS 模块以无属性
> 为默认形态），语料四值判据一致；合法面 {"json","css","text"}。
> **本轮一刀**：新 script-sandbox `import_attributes` 模块
> `validate_static_import_attributes`——逐静态 import 语句（复用 es_module 5 扫描
> 件提 pub(crate)）定位 specifier 字面量后探测 `with {…}` 子句：任一属性键 ≠ type
> → SyntaxError（全量先于 type 值校验）；type 值 ∉ map → TypeError；块未闭合/
> 值非字符串字面量 → SyntaxError。webview is_module 臂图收集前接线：违例走 s102
> 报告面（error_ctor 分派 + 元素 fire load + continue 体不执行含 prelude，s103
> 同口径）——校验先于图收集保证属性键 SyntaxError 胜过裸 specifier 的
> DepResolution TypeError。合法属性零行为变化（仍走图收集，数据模块 MIME 面维持
> 现状）。放新文件不进 es_module.rs（后者已 3470 行超 2000 行准则）。
> **验证**：script-sandbox lib **372/372**（+7）；webview lib **809/809**（+2：非法
> type 值 TypeError 钉 + 未知 key 胜 bare specifier 序钉）；定向 **FILTER=
> import-attributes 4/4 全 Pass**（3 目标页 F→P + empty-attributes-clause 保持
> P）；全子树终扫 **352P/189F/37T + 15 NotRun**（s104：349P/192F/37T+15N），行级
> diff **恰 3 行**（3 目标页 F→P）、**零 P→非P**（comm 校验）、T/N 持平零旁损；
> fmt clean + clippy 双 crate clean。**扰动实录**：首版 map 漏 "text"（语料
> text-module 族 32 处 `type: "text"`）一轮全扫 text-module 5 页翻转（含
> integrity/load-error-events P→非P）——按语料属性值分布统计收录后
> FILTER=text-module 与基线零 diff 愈合。教训：module type map 合法面以全语料
> 属性值分布定界，不凭 spec 印象。**测试坑**：回归钉首版 `import BAD with {…}`
> （default binding 无 from 子句——非法 JS，import_clause_shape_ok 正确拒绝致无
> 报告），改裸字符串 `import "BAD" with {…}`（corpus 同形）后过。渲染面零依赖
> （JS 静态校验 + webview 编译臂）——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s105-import-attributes-static-validation.md](html-semantics-compat/evidence/2026-10-27-m2s105-import-attributes-static-validation.md)。
> **范围**：import_attributes.rs 新模块 + es_module.rs（5 扫描件 pub(crate)) +
> lib.rs（注册）+ webview.rs（接线 + 2 单测）；解析器/语料/runner 零变更；语料
> 不可改维持。
> **挂账**：import attributes 余项——数据模块全量支持（fetcher MIME 门
> testharness.rs:4904-4964 + extractor 类型捕获 + registry 合成源 + rewriter，
> 跨 crate 大切片）、`export … from` 属性子句与动态 import() with 选项、dep 级
> 属性校验；execution-timing 族 **50F**（scheduler 语义，另立项——063 为其中
> 一页）；.py 执行器 / 非 Error 抛出值 ev.error=null 身份 / 动态 import reject
> 形 / error-type-3 instantiation 优先级；charset 族（Timeout 形）；
> svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移/setAttribute('src')/type
> 门/children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜空载（承袭）；
> testharness.rs /tmp/zw-hb*.txt 插桩两行来源待查（承袭）。import-attributes 3 页
> 本轮销账。
> **下一步**：s106 入场探针定刀位（候选：execution-timing scheduler 族定刀 /
> 063 Timeout 深探 / 数据模块全量支持或 .py 执行器前置体量评估）。
> **切片间隙**：本轮 script-sandbox/webview 面 + 单测；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 104 施工收口，frameset 文档 document.body 语义轮——其下一步 s105 入场探针定刀位已由 s105 收口，见上）**：
> **入场定谳**：承袭「116 frameset 文档语义缺口」（s97 记账）——execution-timing/116
> 1F：`assert_array_equals` 期望 `['document.body: <FRAMESET>']` 实得 `<BODY>`。判据
> 机制：head 内动态插入外链 find-body.js，文档解析完后记录 `document.body.localName`
> ——frameset 文档（head 后 `<frameset>` 无 body）spec 语义 body element = html 元素
> 第一个 body **或 frameset** 子元素。根因：解析树（html5ever spec 树构造）frameset
> 文档无 body（Rust DOM 正确），shim `document.body: _wrapSelector('body')`（part06
> :4900）静态 selector proxy 恒报幽灵 BODY。**候选对比**：063（cloneNode deep——
> clone 插入触发 body 内联脚本重复执行 + pending=1 挂起，runner 脚本记账深坑）与
> module 尾片（import attributes/.py 执行器基建体量）均大；116 单 getter 分派定刀。
> **本轮一刀**：part06 `document.body` 改 getter——`__zw_query_match('body')` 未命中
> 且 `'frameset'` 命中 → frameset proxy；否则原 body proxy（普通文档 identity/行为
> 零变化——proxy 按选择器缓存跨访问同对象；均无保持 body proxy 旧兜底，不新增
> null 分派）。webview +1 单测（frameset 文档 localName==='frameset' / 普通文档
> 'body' / identity true）。
> **验证**：webview lib **807/807**（+1）；engine lib **3533/3533**；定向
> **FILTER=116 1F→1P**；全子树终扫 **349P/192F/37T + 15 NotRun**（s103：348P/193F/
> 37T+15N），行级 diff **恰 1 行**（116 F→P）、**零 P→非P**（comm 校验）零旁损；
> fmt clean + clippy 双 crate clean。**探针取证**：①裸 execute_script 走轻量
> bootstrap（`__zw_query_match` 未定义）——shim 语义取证必须走页面脚本路径；②探针
> 脚本置 `</html>` 后 → 解析器建隐式 body，document.body 合法地是 body——判据文档
> 形状须与 116 同形（脚本在 head 内）。渲染面零依赖（reftest 自有 onload 提取
> tag 无关、layout 读 Rust DOM 不经 shim）——reftest/product-smoke 未跑。账本见
> [evidence/2026-10-27-m2s104-frameset-document-body.md](html-semantics-compat/evidence/2026-10-27-m2s104-frameset-document-body.md)。
> **范围**：part06.js ×1 + webview.rs ×1（1 单测）；解析器/语料/runner 零变更；语
> 料不可改维持。
> **挂账**：execution-timing 族 **50F**（scheduler 语义，另立项——063 为其中一页：
> cloneNode deep + runner 脚本记账重复执行）；module 尾片余项（import attributes /
> .py 执行器 / 非 Error 抛出值 ev.error=null 身份 / 动态 import reject 形 /
> error-type-3 instantiation 优先级）；charset 族（Timeout 形）；
> svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移/setAttribute('src')/type 门/
> children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜空载（承袭）；
> testharness.rs /tmp/zw-hb*.txt 插桩两行来源待查（承袭）。116 本轮销账。
> **下一步**：s105 入场探针定刀位（候选：module 尾片体量评估（import attributes
> 或 .py 执行器前置）/ execution-timing scheduler 族定刀 / 063 Timeout 深探）。
> **切片间隙**：本轮 engine .js shim 面 + webview 单测；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 103 施工收口，module graph 取回/解析失败面轮——其下一步 s104 入场探针定刀位已由 s104 收口，见上）**：
> **入场定谳**：s102 残余①——dep 级取回失败空存根兜底：图内任一 dep fetch 失败
> 旧版 `registry.register(spec, "")` 后模块体**照常执行**，网络级失败面丢失。spec
> 语义（fetch-a-single-module-script / resolve-a-module-specifier）：图内取回失败
> → 元素 fire **error** + 体不执行 + 无 window 报告；图内裸 specifier 解析失败 →
> window 报 **TypeError** + 元素照常 fire **load** + 体不执行。判据页定谳：
> error-type-1 双模块（第一 syntaxerror.js parse→window SyntaxError+onload，s102
> 已达；第二 error-type-1.js 图内 404 → onerror、无 window 报告，log.length==3）、
> specifier-error（裸 specifier → ev.error=TypeError + log.length==1 + 元素不
> fire error）。关键事实：prelude 急切求值非入口注册模块（R5182）——体不执行必须
> 连 prelude 一并跳过，否则已取回依赖经 prelude 二次报错且误触 onload unreachable。
> **本轮三刀**：①engine `script_report_module_failure` 加 `error_ctor` 形参
> （编译/求值面 "SyntaxError"、解析面 "TypeError"）+ 新
> `script_dispatch_script_event_by_ordinal`（无 src 内联模块元素级派发）。②webview
> fetcher 分支：收集 `is_dep=true`（此处 spec 均为被运行模块的依赖——旧 false 使裸
> specifier 误作相对 URL 取回）、首 Err 捕获 `(failure, spec)` break，按类分派——
> DepResolution → TypeError 报告面（元素 load）；TopFetch|DepFetch → 元素 error
> （外链 R2944 `module_cache_key`=src 绝对化匹配；内联 ordinal）后 `continue`（编
> 译+prelude 一并跳过）；无 fetcher 路径与 `compile_module_contract` 动态 import 契
> 约路径不变。③+2 单测（dep 404 → onerror+体 undefined+无 window error+继续；裸
> specifier → TypeError+onload+体 undefined+继续）。
> **验证**：webview lib **806/806**（+2）；engine lib **3533/3533**；定向
> **error-type-1 1F→1P**（log=[SyntaxError,1,2] 链闭合）+ **specifier-error
> 1F→1P**、error-type-2 保 P；全子树终扫 **348P/193F/37T + 15 NotRun**（s102：
> 342P/180F/41T+27N）；**P→非P 5 页逐页归因非契约回归**（全为 s102 存根假阳诚实
> 化）：css/json/text-module load-error-events 各 1 子测试（inline,200 臂——
> import attributes/非 JS module 缺口；同页 `src,200` ×2 两版同 Fail、404 臂全保
> P）、nested-imports（delayed-modulescript.py 端点）、json-module/parse-error
>（parse-error.json 不在语料——s102 Pass 系「空存根→export 检查失败→碰巧
> SyntaxError 面」双重假阳）；连带 F→T 1 页（parse-error-location.tentative，体未
> 注册）。**增益**：fetch-error-2 F→P、import-subgraph-404 T→P（s102 F→T 族翻回
> 真 Pass）、load-error-events-inline N→P×2、integrity ×3 T→Fail+Pass（404 臂真
> P）、content-type-checking N→F+P2、http-accept T→F×2、module.html/valid-content-
> type N→F×7；fmt clean + clippy 双 crate clean；runner testharness.rs 零变更；
> reftest/product-smoke 未跑（无 render/layout 面）。账本见
> [evidence/2026-10-27-m2s103-module-graph-failure-face.md](html-semantics-compat/evidence/2026-10-27-m2s103-module-graph-failure-face.md)。
> **范围**：webview.rs ×1（fetcher 分支失败分派 + 2 单测）+ script_gen.rs ×1
>（error_ctor 形参 + ordinal 派发 helper）；shim/语料零变更；语料不可改维持。
> **挂账**：module 尾片（.py 执行器——nested-imports/integrity 族 load 期望臂、
> http-accept-header；import attributes + CSS/JSON/text module——load-error-events
> 200 臂、valid-content-type、content-type-checking Fail 面、parse-error 族；非
> Error 抛出值 ev.error=null 身份（error-type-3）；动态 import 已错模块 reject 形
>（instantiation-error-1）；error-type-3 instantiation 优先级）；116 frameset 文
> 档语义缺口；063 Timeout 承袭；charset 族（Timeout 形）；svg-script-fragment-clone
> 3F；FIXME 组（bufTop 漂移/setAttribute('src')/type 门/children-changed 其余入口）；
> 接口对象基建面（承袭）；基线扫描宜空载（承袭）；testharness.rs /tmp/zw-hb*.txt
> 插桩两行来源待查（承袭）。目标 2 页 + 连带增益本轮销账。
> **下一步**：s104 入场探针定刀位（候选：116 frameset 文档语义缺口 / 063
> Timeout / module 尾片余项——import attributes 或 .py 执行器前置体量评估）。
> **切片间隙**：本轮 webview/engine .rs 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 102 施工收口，module 失败报告面轮——其下一步 s103 入场探针定刀位已由 s103 收口，见上）**：
> **入场定谳**：s101 终扫 73 行 `page script threw`，35 页 declared=0——module
> 编译/求值失败在 strict 下整页 abort，掩盖页面其余用例。spec 语义（run a module
> script）：fetch **成功**后的编译/链接/求值失败 → 向 window 报异常（window error
> 事件 ev.error=抛出值，parse 失败为 SyntaxError）+ 元素照常 fire **load** + 页
> 面继续；仅**网络级取回失败** → 元素 fire **error**。判据页：module/error-type-1
> （parse→window SyntaxError+onload；network→onerror）、instantiation-error-3/4/5。
> **本轮四刀**：①engine `script_report_module_failure(msg, ordinal)`——
> `reportError(new SyntaxError(msg))`（shim reportError part01 既有面，ev.error 携
> Error 实例）+ `getElementsByTagName('script')[ordinal]` 元素 load 派发（文档序
> script 序号 = extract_script_csp_info 同口径，适配无 src 内联模块）。②webview 编
> 译失败臂：strict abort → 报告面 + continue（strict 亦不 abort——prepare 阶段失败
> 面非内联体抛错）。③webview 执行期失败臂：`strict && !is_module` 才 abort；module
> 执行期异常（求值抛错/变换后代码 parse 失败——compile_module_script 变换为词法面
> 不做全量 parse，error-type-1 syntaxerror.js `%` 体过变换 sandbox parse 期炸实
> 证）→ 同①报告面 + 继续。④外链 classic/module 取回失败两臂：补
> `script_dispatch_script_event(&abs, "error")`（R2944——旧注释声称 spec error 面
> 但实现仅 warn 静默跳过）。
> **验证**：webview lib **804/804**（+2：编译失败 window SyntaxError+onload+继续
> 钉、外链取回失败 onerror+继续钉）；engine lib **3533/3533**（含他流并入用例）；
> 定向：error-type-2 **1F→1P**、error-type-1/3 abort→子测试级 Fail（断言真跑）、
> instantiation-error **3/4/5 三页 1F→1P**；全子树终扫 **342P/180F/41T + 27
> NotRun**（s101：326P/220F/14T+18N），行级 diff **F→P 12 页 + 零 P→非P**
>（comm 校验）；**F→T 27 页归因**（非回归）：原 declared=0 abort 族跑完后测试体
> 因 feature 缺失（import attributes `with{type:"css"}`、.py CGI 端点、referrer
> 语义）未注册 → runner「completed without reporting (tests:0)」映射 Timeout——
> 状态更保真（window error 已报、页面继续），翻回需尾片；fmt clean + clippy 双
> crate `-D warnings` clean；runner testharness.rs 零变更；reftest/product-smoke
> 未跑（无 render/layout 面）。账本见
> [evidence/2026-10-27-m2s102-module-failure-report-face.md](html-semantics-compat/evidence/2026-10-27-m2s102-module-failure-report-face.md)。
> **范围**：webview.rs ×1（两失败臂 narrow + 取回失败投递 + 2 单测）+ script_gen.rs
> ×1（helper）；shim/语料零变更；语料不可改维持。
> **挂账**：module 尾片（.py 执行器前置 + import attributes；本面新暴露残余——dep
> 级取回失败仍空存根兜底（error-type-1 的 404.js 臂需「图中任一取回失败 → 元素
> error」语义，动 M2-S4E 存根契约须独立切片全扫审计）、非 Error 抛出值 ev.error=null
> （error-type-3 对象身份）、bare specifier 运行时 TypeError（specifier-error）、
> 动态 import 已错模块 reject 形（TypeError vs SyntaxError））；116 frameset 文档
> 语义缺口；063 Timeout 承袭；charset 族（现 Timeout 形）；svg-script-fragment-clone
> 3F；FIXME 组（bufTop 漂移/setAttribute('src')/type 门/children-changed 其余入口）；
> 接口对象基建面（承袭）；基线扫描宜空载（承袭）。F→P 12 页 + 祛 Mask 8 页本轮
> 销账。附带发现：testharness.rs 既有已提交 `/tmp/zw-hb*.txt` 调试插桩两行（非本
> 会话所加，来源待查）。
> **下一步**：s103 入场探针定刀位（候选：module 尾片 dep 级取回失败元素 error 面
> （存根契约变更，独立切片）/ 116 frameset 文档语义缺口 / 063 Timeout）。
> **切片间隙**：本轮 webview/engine .rs 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-27 M2 切片 101 施工收口，外链模块 import 收集基轮——其下一步 s102 入场探针定刀位已由 s102 收口，见上）**：
> **入场定谳（两层根因）**：nomodule-set-on-external-module-script 1F 整页 abort
> （`Module ./cocoa-module.js does not provide an export named default`，declared
> tests: 0）。①webview——ExternalModule 静态 import 收集基 = 页面 URL；spec 语义
> import 相对 **requesting module 自身 URL** 解析（HostResolveImportedModule
> referrer 为模块自身），模块体在 resources/ 子目录时其相对 import 解析到页目录 →
> 取回 404 → 空存根 → 假编译错。②runner——inline_local_scripts 连外链 module 也
> 内联（s94 保 type），engine 走 InlineModule 臂，①修复不触发、原始 src 解析基准
> 整体丢失。**方案取舍**：A) 跳过 module 内联——parser module 臂 load/error 投递
> 依赖 s100 投递尾，外置即丢（module/load-error-events 2P 回退）被否；B) data 属性
> 透传 origin——跨 crate 形状变更；取 **E1) 内联投递时重写模块体相对 spec**（投递
> 形态适配器职责，同 .sub.js 替换先例）。
> **本轮三刀**：①webview `module_dep_base`——ExternalModule 臂收集基 =
> `resolve_document_url(doc_url, src)`（InlineModule 维持页面 URL）。②runner
> `rewrite_inline_module_import_specs`——仅 `./`/`../` 相对 spec 以模块自身
> wpt_root 相对目录为基归一 root-absolute（lexer 感知提取定位 + 引号包裹整体替换；
> 裸 specifier 维持 s87 解析失败语义、root-absolute/scheme 不动、查询串保留；动态
> import() 走运行时链归 module 尾片）。③module 图**钻石依赖拼写别名**——collect
> 循环防护（按解析后 URL）命中时按**当前拼写补注册**同一源（`fetched` URL→源缓
> 存）：registry 按导入方体内原文拼写查表，旧版仅注册首拼写 → 第二拼写 lookup
> 落空 compile 失败（imports-self-inner 自导入实证；旧文档明记「已知限制」——本
> 轮连带解除的共享路径缺陷，②重写使拼写分歧显形）。
> **验证**：webview lib **802/802**（+2：收集基钉 dep 仅模块相对位置可达红→绿 +
> 拼写别名钉）；runner bin **229/229**（+1：helper 相对/上溯/root-absolute/裸/
> scheme 四态 + 端到端 module 重写而 classic 体不动）；定向 FILTER=
> nomodule-set-on-external-module **1F→1P**（两轮——仅①仍同错钉死根因②、
> ①+②+③ 全绿）、FILTER=module/imports **5 Pass 0 Fail**（中途态 5P→1F 假红
> 由③归位）；全子树终扫 **326P/220F/14T + 18 NotRun**，diff vs s100 基线**恰
> 1 页 1 行**（本页 Fail→Pass）、**零 P→非P**；fmt clean + clippy 双 crate
> `-D warnings` clean（ulimit 兜底沿 s91 口径）；reftest/product-smoke 未跑
> （无 render/layout 面）。账本见
> [evidence/2026-10-27-m2s101-external-module-import-base.md](html-semantics-compat/evidence/2026-10-27-m2s101-external-module-import-base.md)。
> **范围**：webview.rs ×1（收集基 + fetched 缓存补注册 + 2 单测）+ testharness.rs
> ×1（重写 helper + 调用点 + 1 单测）；shim/语料零变更；语料不可改维持。
> **挂账**：module 尾片（.py 执行器前置；text-module/css-module/json-module
> load-error 变体页同族汇入；动态 import() 相对模块解析基同列）；module 编译错误
> error 事件面（compile 失败须 error 事件而非 strict 整页 abort——本页「must run」
> 臂已销账、失败臂仍挂）；116 frameset 文档语义缺口；063 Timeout 承袭；charset
> 12F（.py）；svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移/setAttribute
> ('src')/type 门/children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜
> 空载（承袭）。nomodule-external-module 1F 本轮销账。附带发现：testharness.rs
> 既有已提交 `/tmp/zw-hb*.txt` 调试插桩两行（非本会话所加，来源待查）。
> **切片间隙**：本轮 runner/webview .rs 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-26 M2 切片 100 施工收口，经典脚本 load/error 事件投递保真轮；施工跨 429 断点，断点后终扫复跑与断点前扫描 bitwise 一致——其下一步 s101 入场探针定刀位已由 s101 收口，见上）**：
> **入场核账**：s99 挂账 load-event 投递保真族——load-error-events-1 11 臂（parser
> defer/sync/async × 200/404 + dynamic ×4 + no-src 内联臂）+ nomodule-sync
> 「must run」。两面根因：①投递面——parser 外链被 inline_local_scripts 内联改写丢
> onload/onerror（s94 保真只留 nonce/nomodule/type），且内联后无取回环节，spec
> 「成功执行 fire load / 取回失败 fire error」随之丢失；②服务面——load-error-events.py
> 为 wptserve CGI 端点，语料 pull 面不含 .py，dynamic 200 臂取回落通用缺文件 404 →
> error 路径误触发。
> **本轮三刀**：①投递保真——script_event_handler_attr 提取回写 on* 内容属性（`&`/`"`
> 转义同 nonce 先例）+ 投递尾（200 体后 fire load、非 200 只 fire error 且脚本体
> 不执行；document.currentScript.dispatchEvent——R2934 编译链触发 on* 与
> addEventListener 两路，事件不冒泡不扰 window load）；src 查询串先剥离再归一。
> ②fetch 通道 .py fixture 静态等价（wpt_load_error_events_response，fetch/内联共
> 用）：test 参数 `^[a-zA-Z0-9_]+$` 门（缺席/非法 400）、`*_load`→200 +
> `<test>.executed=true`、其余 404 + assert_unreached 体（status 拒 eval）；路径精
> 确匹配 classic 端点，module 族变体不匹配归 module 尾片。③extract
> attr_from_open_tag 值面从**原文**切片——旧版从 ASCII 小写镜像切值，
> `onLoad(t1)`→`onload(t1)` 失效、base64 nonce 同面受损；lower 镜像等字节长偏移通用。
> **验证**：runner bin **228/228**（+2 单测）；engine lib **3530/3530**；
> FILTER=load-error-events le1 **11/11 全 Pass**（页完成无 Timeout）；
> FILTER=nomodule nomodule-sync **1F→1P**；全子树终扫 **325P/221F/14T + 18
> NotRun**，diff vs s99 恰 3 页 F/T→P（le1 2F+1T→11P 全清 / module
> load-error-events 2F→2P——parser module 臂 on* 保真连带 / nomodule-sync
> 1F→1P）、**零 P→非P**（/tmp/n99.txt vs /tmp/n100s.txt）；execution-timing/122
> 缺行系 teardown 粒度方差（s93 先例，单跑仍报既有 Fail）；fmt clean + clippy 双
> crate `-D warnings` clean（ulimit 兜底沿 s91 口径）；reftest/product-smoke 未跑
> （无 render/layout 面）。账本见
> [evidence/2026-10-26-m2s100-load-event-delivery-fidelity.md](html-semantics-compat/evidence/2026-10-26-m2s100-load-event-delivery-fidelity.md)。
> **范围**：testharness.rs ×1（fetch arm + fixture + inline 保真/投递尾 + 2 单测）+
> extract.rs ×1（attr 值切片）；shim/语料零变更；语料不可改维持。
> **挂账**：module 尾片（.py 执行器前置；text-module/css-module/json-module
> load-error 变体页同族汇入）；module 编译错误事件面（nomodule-set-on-external-
> module-script 1F）；116 frameset 文档语义缺口；063 Timeout 承袭；charset 12F
> （.py）；svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移/setAttribute('src')/
> type 门/children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜空载（承
> 袭）。load-error-events classic 全臂 + nomodule-sync 本轮销账。
> **切片间隙**：本轮纯 runner/engine .rs 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-26 M2 切片 99 施工收口，script 布尔 IDL has 可见性轮——其下一步 s100 入场探针定刀位已由 s100 收口，见上）**：
> **入场核账**：s98 下一步候选入场探针——nomodule-set-on-async-classic-script
> 两行**同报** `assert_true: expected true got false`，两测试首断言同为
> `assert_true(supportsNoModule)`（`"noModule" in script` 探针）；若 load 门
> （`waitForLoadEvent = new Promise(r => window.onload = r)`）失灵应报 Timeout
> 而非断言失败——门正常，缺口在 `in` 可见性。**根因**：元素 proxy has trap
> （part05 生效版，R129 定谳）白名单无 _REFLECTED_BOOL 面——get trap（part04
> R3038/R3040）值面覆盖全表、`in` 探测恒 false（「'id' in el 仍 false」既有
> documented 限制同类面）。**本轮接线**：has 白名单新增 R5300 分支，
> `async`/`defer`/`noModule` 三名 **tag 门 SCRIPT**（':' 后缀容差同 R5266）——
> 全员表列名重蹈 R5249 menuitem 假阳性先例（接口成员归属按 tag 门控）；三名恰
> 为 HTMLScriptElement 接口成员（spec：reflected boolean，
> https://html.spec.whatwg.org/multipage/scripting.html#dom-script-nomodule）。
> **验证**：FILTER=nomodule 本两行 **2F→2P**——测试 1（noModule=false）四断言
> 全过（动态 appendChild src 执行 + onload 投递 + load 门 resolve）；测试 2
> （noModule=true）!executed/!loaded/!errored 全过（s96 prepare 门阻塞执行且不
> 投递 load/error——动态臂阻塞面实测）；全子树终扫 **314P/227F/15T + 18
> NotRun**，净 +2 Pass、行级 diff 仅本两行、**零 P→非P**（/tmp/s99-s1-rows.txt）；
> engine lib **3530/3530**（guard）；shim 拼接 node --check OK；纯 shim .js 变更
> fmt/clippy/reftest/product-smoke 不涉（沿 s91-s98 口径）。账本见
> [evidence/2026-10-26-m2s99-script-bool-idl-has-visibility.md](html-semantics-compat/evidence/2026-10-26-m2s99-script-bool-idl-has-visibility.md)。
> **范围**：part05.js ×1（has trap 一分支）；runner/语料零变更；语料不可改维持。
> **挂账**：load-event 投递保真族——parser 外链被 inline_local_scripts 内联投递
> 时 `onload`/`onerror` 内容属性被丢弃（s94 保真只留 nonce/nomodule/type）→
> nomodule-set-on-synchronously「must run」1F（脚本执行到位、`assert_true(loaded)`
> 假红）+ load-error-events 族 7F/3T 同族汇入；module 编译错误事件面
> （nomodule-set-on-external-module-script——缺 default export 的编译错误冒出为
> page script threw，spec 须落 error 事件）；116 frameset 文档语义缺口；063
> Timeout 承袭；module 尾片（.py 前置）；charset 12F（.py）；
> svg-script-fragment-clone 3F；FIXME 组（bufTop 漂移 / setAttribute('src') /
> type 门 / children-changed 其余入口）；接口对象基建面（承袭）；基线扫描宜空载
> （承袭）。nomodule-async 2F 本轮翻绿销账。
> **切片间隙**：本轮纯 engine shim 面（has trap）；下一挂账 load-event 保真族跨
> runner（inline_local_scripts）+ engine（执行后投递）两面。
>
> **▶ 上轮状态（2026-10-26 M2 切片 98 施工收口，write 注入 harness runner 面接线轮——其下一步 s99 入场探针定刀位已由 s99 收口，见上）**：
> **入场核账**：s97 挂账定谳——102 页脚本全执行但 write 注入 testharness.js 经
> fetch 通道取回 RAW 副本（`hook=undefined/state=null` 超时转储）。**完成机制核
> 实**（pinned testharness.js）：`notify_result → all_done()（要求
> test_environment.all_loaded）→ complete() → completion 回调 → 探针全局
> __zw_harness_complete`。**静态/write 两路不对称**：静态标签被 prepare_harness_html
> 内联替换（补丁 replacen + reporter 装配 + 生命周期脚本尾补调钩子），102 初始
> 文档只有 write 串里的 src——补丁与 reporter 都到不了注入副本，且 `document.open`
> 换文档后 load 不重发，`all_loaded` 恒 false，末测 done 后永不 complete。**本轮
> 接线**（testharness.rs fetch 静态回退特判）：`/resources/testharness.js` 服务
> 补丁源（`apply_harness_completion_patch` 与静态内联共用 replacen）+ eval 尾自调
> `__zw_mark_harness_loaded()`（注入副本求值完成即置 all_loaded；此刻
> tests.length=0，all_done 为 false 无误完成）；`/resources/testharnessreport.js`
> 服务 reporter 装配快照（上游 report 裸 loader，注册的 completion 回调到不了探针
> 全局）；reporter 提 `HARNESS_REPORTER_SOURCE` 常量、静态/fetch 共用装配面。
> **影响面核查**：write/dynamic 注入两路径的页面全语料仅 102 形（静态标签一律内
> 联替换走不到 fetch 通道），blast radius 有界。**验证**：execution-timing s97
> 97P/51F/2T → **98P/51F/1T**（逐行 diff 仅 102）；全子树终扫 **312P/229F/15T +
> 18 NotRun**，净 +1 Pass、行级 diff 仅 102 Timeout→Pass、**零 P→非P**
> （/tmp/s98-s1-rows.txt）；102 结果行 `Pass :: scheduler: defer script after
> initial onload event`（写入全家桶依序执行、eventOrder 断言通过后 t.done() 完
> 成——非空转判绿）；runner bin 单测 **226/226**（guard）；纯 runner .rs 变更，
> fmt/clippy/reftest 不涉（沿 s91-s97 口径）。账本见
> [evidence/2026-10-26-m2s98-write-harness-runner-face.md](html-semantics-compat/evidence/2026-10-26-m2s98-write-harness-runner-face.md)。
> **范围**：testharness.rs ×1（fetch 静态服务特判 + 装配常量/公共函数提升）；
> engine shim 零变更；语料不可改维持。
> **挂账**：116 frameset 文档语义缺口；063 Timeout（hook=function pending=1）承
> 袭；module 尾片（.py 前置）；charset 12F（.py）；svg-script-fragment-clone 3F；
> nomodule-async 2F（window.onload promise 门）；load-error-events 族（脱离树副本
> 元素事件保真）；FIXME：写流中页面脚本结构变更 bufTop 漂移、嵌套 write 内含外链
> 近似文档序、setAttribute('src') 通路、type 门、children-changed 其余变异入口
> （append()/textContent=/.text 已连接面）；接口对象基建面（承袭）；基线扫描宜空
> 载（承袭）。102 本轮翻绿销账。
> **切片间隙**：本轮纯 runner 面（.rs）；下一挂账 nomodule-async/load-error-events
> 回 engine shim 面，跨面间隙。
>
> **▶ 上轮状态（2026-10-26 M2 切片 97 施工收口，document.write 写入脚本执行面轮——其下一步 s98 入场探针定刀位已由 s98 收口，见上）**：
> **入场核账**：s96 HEAD 基线 **288P/251F/17T + 18 NotRun**（scripting-1 口径
> 574 行）。s97 聚焦 document.write 族：spec #document-write-steps 把写入标记交
> 解析器插入点（内联同步、外链按文档序阻塞写流内其后脚本、嵌套写入落在当前求
> 值窗），本引擎回放走 innerHTML/insertAdjacentHTML（inert）——写入脚本须显式
> 执行面。**本轮两刀**：①part06 `__zwDocWriteRunFace`（write 入口惰性安装，显式
> open/close 流与隐式流共用）：缓冲尾段子树收集（bufTop = 现长 − 基点，4096 节点
> 预算）——基点取**首轮 innerHTML 往返后**口径（dbg 探针 `kids=8 base=9` 实证序列
> 化/重解析对空白文本节点有损耗：9→7，写前快照使 bufTop 落负、收集恒空）；内联
> 槽位 `__zwDocWriteDoneSlots` 防重；求值窗 `__zwDocWriteEvalSlot`（part04 prepare
> then 内设置/清除）——窗内嵌套 write 的面走即时 pass（只执行新增内联、src 槽位
> 留续流链、不动记账，009「external before/dw/after」序）；外链 src 槽位阻塞链
> （水位线占位停本槽位 + `__zwDocWritePendingSlot` 挂链 + plain wrapper 重建
> handle 域副本**全属性透传**（012 include-N onload）+ `_zwPrepareSrcScript(copy,
> cont)` 续流清链/推进/续跑）；close 流接管 runner 无 settle 环境（基点 0，全家桶
> 依序）；②part04 insertBefore 统一出口挂 R387 同口径门（连接态→nomodule→
> for/event→run-once→内联 eval / `_zwPrepareSrcScript`）——018 testlib.addScript
> firstInParent 形态。**主发现（水位线占位）**：src 槽位 prepare 前不占位，续流
> 回调条件推进（`wm === slot`）永不成立 → 重入重复 prepare fresh 副本 → 无限
> fetch（102b/102c 90s 执行超时实证）；修后插桩定谳 close 流全序
> `close-takeover→face→slot0 内联→slot1 src→prepare→then→cont→face(wm=2)→slot2
> 内联→face-exit:wm=3`，102b/102c 残留 Fail 系探针缺 testlib.js（`log` 未定义）
> 作者缺陷，非引擎回归。**验证**：execution-timing corpus-only s96 83P/73F/3T →
> **97P/51F/2T**；全子树终扫 **311P/229F/16T + 18 NotRun**，净 +23 Pass、
> **零 P→非P**（comm 校验空；终扫 /tmp/s97-scripting1-final.txt 定为下轮基线）；
> 033/041/068/096 仅断言消息面变化（Fail→Fail 更深断言）；102 Fail→Timeout
> （页脚本全执行，write 注入 harness 未接完成探测——runner 面挂账）；116
> Timeout→Fail（frameset 文档 `document.body` 报 BODY——既有引擎缺口显形）；
> engine lib **3530/3530**（guard）；shim 13 part 真实序拼接 node --check OK；
> 纯 .js 变更 fmt/clippy 不涉；reftest 未跑（无 render/layout 面，沿 s91-s96
> 口径）。账本见 [evidence/2026-10-26-m2s97-document-write-run-face.md](html-semantics-compat/evidence/2026-10-26-m2s97-document-write-run-face.md)。
> **范围**：engine shim ×2（part06 执行面 + open/close/write 记账 / part04
> insertBefore 门 + prepare then 求值窗标记）；testharness.rs 上轮 writer 单元
> 截断修复随本切片验证（007-012 rawtext 语义）；无其他 .rs 触碰；语料不可改
> 维持（探针页 11 页已删）。
> **挂账**：102 runner 面完成探测接线（write 注入 harness `__zw_mark_harness_loaded`）；
> 116 frameset 文档语义缺口；063 Timeout（hook=function pending=1）承袭；module
> 尾片（.py 前置）；charset 12F（.py）；svg-script-fragment-clone 3F；
> nomodule-async 2F（window.onload promise 门）；load-error-events 族（脱离树副本
> 元素事件保真）；FIXME：写流中页面脚本结构变更 bufTop 漂移、嵌套 write 内含外链
> 近似文档序、setAttribute('src') 通路、type 门、children-changed 其余变异入口
> （append()/textContent=/.text 已连接面——insertBefore 本轮已挂）；接口对象
> 基建面（承袭）；基线扫描宜空载（承袭）。020/048 本轮翻绿销账。
> **切片间隙**：本轮纯 engine shim 面；下一挂账 102 接线在 runner 面（.rs），
> 跨面间隙。

> **▶ 上轮状态（2026-10-26 M2 切片 96 施工收口，script.src prepare 触发 + for/event 门轮——其下一步 s97 入场探针定刀位已由 s97 收口，见上）**：
> **入场核账**：s95 HEAD（8c997860f）基线 **281P/256F/19T + 18 NotRun**（scripting-1
> 口径 574 行）。s96 入场探针自 256F 选 runner 可达 classic 动态 src 两簇：
> execution-timing 022/024（append 空脚本后连接态设 src——src set 无 prepare 触发，
> 探针实证非延迟非通道慢）+ 149 三个 dom_inserted for/event 臂（R387 无门）。
> **本轮两刀**：①`_zwScriptForevBlocked` for/event 门（双属性在场且非
> 「for≈window 且 event≈onload|onload()（ASCII 剥白 CI）」即中止；R377 fallback/
> 内联臂/R387b 三路径统一接入）；②R387b 执行体提升 `_zwPrepareSrcScript` 单点
> + part05 set trap SCRIPT.src 专用分支（反射写属性 + connected → prepare；
> `_reflectedUrlAttr` 分支加 SCRIPT/src 排除）。**主发现（跨 part 作用域结）**：
> helper 定义于 part04 appendChild trap 闭包，part05 set trap 是另一函数作用域
> 不可见——探针 DIAG `hasFn=undefined` 定谳，`globalThis._zwPrepareSrcScript`
> 导出解结（_zwNodeContains part03 导出先例）。探针教训：首轮 2s 轮询误报
> Pass（teardown 竞态），改 onload+阶梯时点 + fetch 插桩定谳「从不 fetch」；
> 插桩复核 B 臂全链路：TRAP→conn=true→PREPARE-CALLED→fetch→执行于 load 前。
> **验证**：探针页 9/9 全绿；execution-timing 全目录 022 F→P、023 T→P、
> 024 F→P 目录内零回归；script-for-event 两文件 34/34；149 **6F→3F**（dom 臂
> 翻绿，残余 3F 全 parser 臂=document.write 挂账）；change-src-attr-prepare-
> a-script **T→P**（spec 属性变更步骤正典页）；全子树终扫 **288P/251F/17T
> + 18 NotRun**，diff vs 基线 **F/T→P 7 行 5 页、P→非P 0 页**（终扫
> /tmp/s96-final-scan.txt 定为下轮基线）；engine lib **3530/3530**（guard）；
> 纯 .js 变更 fmt/clippy 不涉；reftest 未跑（无 render/layout 面，沿 s91-s95
> 口径）。账本见 [evidence/2026-10-26-m2s96-script-src-prepare-and-forev-gate.md](html-semantics-compat/evidence/2026-10-26-m2s96-script-src-prepare-and-forev-gate.md)。
> **范围**：engine shim ×2（part04 helper×2 + R387 门/调用改造 / part05 URL
> 分支排除 + SCRIPT.src 专用分支）；无 .rs 触碰；语料不可改维持（探针页已删）。
> **挂账**：execution-timing document.write 族（149 parser 臂 + 005-012/018/
> 026/027）；020 data: URL（fetch 通道 data: 支持未证）；048 sel 形 script[0].src=
> 自身缺口；module 尾片（.py 前置）；charset 12F（.py）；svg-script-fragment-clone
> 3F；nomodule-async 2F（window.onload promise 门）；load-error-events 族（元素
> onload 投递保真）；FIXME：setAttribute('src') 通路、type 门、children-changed
> 其余变异入口（insertBefore/append()/textContent=/.text 已连接面）；接口对象
> 基建面（承袭）；基线扫描宜空载（承袭）。
> **切片间隙**：本轮纯 engine shim 面；下一挂账同流同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 95 施工收口，script.text IDL + children-changed 记账轮——其下一步 s96 入场探针定刀位已由 s96 收口，见上）**：
> **入场核账**：s94 HEAD（47bc3dffd）基线 **263P/270F/23T + 18 NotRun**（scripting-1
> 口径 574 行）。s95 入场探针自 270F 聚类选 runner 可达 classic 脚本文本语义两簇：
> script-text.html 6F（.text getter=child text content / setter=string replace all，
> 全 detached 纯 IDL 面）+ emptyish-script-elements 6F（解析期空源 bail-early 不置
> already-started → 补 Text 子须执行）。排除 runner 上限：script-charset-01（.py
> 执行器）、script-text-modifications（slow.py trickle 时序）。**本轮两刀**：
> ①script.text IDL（get trap 直接子 Text walk——comment/PI/元素子不计；set trap
> null→"null" 归一后委托 textContent——R3907 同口径）；②appendChild trap SCRIPT
> 父 children-changed→prepare（头部 pre 快照 + 尾部触发块：pre 空 + post 非空 →
> connected → already-started 门 → nomodule 门 → classic eval + 置位）；
> already-started 记账 = part06 __zw_set_current_script 投递时按 __zwSelector 记
> _zwScriptStarted（空源 bail/门压制不投递不记——与 spec prepare 置位时机对齐；
> handle 形沿 R387 _zwRanScripts）+ cloneNode 传播（R3901/R3902b 同型）。
> **回归与记账返工（本轮主发现）**：首轮纯 pre/post 近似出 execution-timing
> 059/060/062 三行 P→F——执行中脚本自摘子（removeChild/innerHTML=''）再补文本、
> started 脚本浅克隆后补文本，pre 快照捕获在变异后误判「未启动」；返工即投递
> 记账 + 克隆传播 + 执行后置位（pre/post 降第二道近似），三页回绿且连带解锁
> 克隆/补文本族：061/064/065 file Timeout→全绿解挂、036 Timeout→subtest Fail
> 显形、066 F→P、127/128 F→P。
> **验证**：定向 script-text 6P/0F、emptyish 7P/0F、059/060/062 回绿、120-126
> 与基线逐行一致；全子树终扫 **281P/256F/19T + 18 NotRun**，diff vs 基线
> **F/T→P 14 行 + 解挂 4 页 + 066 F→P、P→非P 0 行**（14 行 = script-text 6 +
> emptyish 6 + execution-timing 127/128 2），域账 **282P/256F/19T**（+1 惯例）。
> 单测腿：engine lib **3530/3530**（guard）；纯 .js 变更 fmt/clippy 不涉；
> reftest 未跑（无 render/layout 面，沿 s91-s94 口径）。账本见
> [evidence/2026-10-26-m2s95-script-text-and-children-changed.md](html-semantics-compat/evidence/2026-10-26-m2s95-script-text-and-children-changed.md)。
> **范围**：engine shim ×3（part03 get trap / part04 appendChild 钩子+克隆传播 /
> part06 投递记账）；无 .rs 触碰；语料不可改维持（探针页已删）。
> **挂账**：execution-timing 编号套件 78F（036 显形 deep clone 族 / 122-126
> type-async 面 / 005-012 document.write 族）；module 尾片（.py 前置）；charset
> 12F（.py）；svg-script-fragment-clone 3F；nomodule-async 2F（window.onload
> promise 门）；load-error-events 族（元素 onload 投递保真）；FIXME：children
> changed 其余变异入口（insertBefore/append()/textContent=/.text 已连接面）未挂
> 触发；接口对象基建面（承袭）；基线扫描宜空载（承袭；s95 终扫
> /tmp/s95-final-scan.txt 定为下轮基线）。
> **切片间隙**：本轮纯 engine shim 面；下一挂账同流同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 94 施工收口，prepare 双门 + 投递保真 + module 缓存键轮——其下一步 s95 入场探针定刀位已由 s95 收口，见上）**：
> **入场核账**：s93 HEAD（43f10a3e7）基线 **211P/282F/25T + 22 NotRun**
>（scripting-1 口径 540 行）。s94 入场探针自 282F 聚类选 runner 可达 classic
> 属性面：nomodule face 7F + script-for-event face 13F。**本轮三刀**：
> ①extract prepare 双门（`script_suppressed_by_nomodule`：nomodule 内容属性在
> 场压制 classic、module 不受影响——module/nomodule-attribute 以 this.js 顶层
> this 区分执行形态；`script_blocked_by_for_event`：双属性在场 → ASCII 剥白后
> for≈"window" 且 event≈"onload"|"onload()"（ASCII case-insens），任一属性缺
> 失照常执行——语料 Script 15/16 单属性臂定谳）；②shim R387 动态插入钩子
> nomodule 门（三执行分支条件前缀；不置 run-once——spec 中止在 already-started
> 置位前，摘属性可重来）；③runner 投递保真扩口（inline_local_scripts 保留
> nomodule——新 `open_tag_has_attr` bool 属性裸形在场判定，与 attr_from_open_tag
> 的 nonce 裸形视缺失分工；保留 type="module"——s92 挂账兑现）。
> **附带根因返工（本轮主发现）**：type 保真暴露 R5182 潜伏缺陷——`__moduleCache`
> 文档级记忆化键误用页面 URL，同文档第二枚 module script 被首枚记忆化命中短路、
> 模块体零次求值（currentScript-null file Timeout + probe-a~d 最小探针二分定谳
> 任意双 module 页挂起；潜伏态，语料此前无双内联 module 页）；修复 = 键参数分离
> `compile_module_script(source, url, cache_key, registry)`（内联 = 页面 URL 挂
> ordinal 片段、外链 = src 绝对化；三生产调用方语义不变）+ learning 沉淀
>（bugs/2026-10/2026-10-26-module-cache-key-page-url-collision.md）。
> **验证**：定向 nomodule FILTER 4F→P、script-for-event 18 行全绿（13F→P）、
> 探针 4 页全绿、currentScript-null 全绿（连带基线 Fail→Pass——module 求值
> currentScript=null 语义成立）；全子树终扫 574 行 **263P/270F/23T + 18 NotRun**，
> diff vs 基线 **F→P 23 行、P→非P 0 行**（for-event 13 + nomodule 4 +
> currentScript-null 1 + module-vs-script-2 1 + module/load-error-events 3——
> 页内 type=module 标签在，缓存键解锁，非 classic load-error 族）；module 族
> 多页首次以 subtest 粒度完成（type 4 例、imports 5 例、integrity inline 1、
> module script-for-event 18 例、empty-attributes-clause 1 例等翻绿）。
> 历史口径重述 **211P/282F/25T（540 行期）→ 263P/270F/23T（574 行期，行数增长
> = 粒度完成非口径变化）**，域账 **264P/270F/23T**（+1 惯例）。单测七腿：
> engine **3530**（+3）/runner **226**（+76i）/webview **800**/sandbox v8
> **365**（+1）/quickjs **260**（+1）；fmt/clippy 四 crate clean；reftest 未跑
>（无 render/layout 面，沿 s91-s93 口径）。账本见
> [evidence/2026-10-26-m2s94-prepare-gates-and-delivery-fidelity.md](html-semantics-compat/evidence/2026-10-26-m2s94-prepare-gates-and-delivery-fidelity.md)。
> **范围**：engine ×2（extract 双门 + shim 钩子）、script-sandbox（签名+单测）、
> webview（缓存键接线）、runner（投递保真）；css/layout/renderer 零改动；语料
> 不可改维持（probe 临时页已删）。
> **挂账**：async nomodule 2F（window.onload promise 门——window load 派发
> face）；sync 臂 1（元素 onload 属性投递丢失 / 元素 load 事件 face）；
> nomodule-set-on-external-module-script（外链 module 装载面）；module 族新显形
> Fail（charset-01/03、integrity 外链、single-evaluation-1/2、instantiation-6/7、
> specifier-error、evaluation-error-2、fetch-error-2、import-attributes 部分
> ——.py 执行器前置项不变，subtest 账已立）；execution-timing 80F / 顶层 65F /
> 完整接口对象基建面（承袭）；基线扫描宜空载（teardown 粒度负载敏感，承袭；
> s94 终扫 /tmp/s94-final-scan2.txt 定为下轮基线）。
> **切片间隙**：本轮 engine/sandbox/webview/runner 面；下一挂账同流同面，间隙开放。
>
> **▶ 上轮状态（2026-10-26 M2 切片 93 施工收口，classic 脚本属性面批刀轮——其下一步 s94 入场探针定刀位已由 s94 收口，见上）**：
> **入场核账**：s92 HEAD（2cf2a5c5c）基线 **201P/291F/25T**（the-script-element
> 子树 517 行）。s92「下一步 module 尾片盘点」入场探针后**转向**：module 尾片
> 43F 簇（css-module 20 + json-module 11 + text-module 8 + import-attributes 4）
> 被 wptserve .py 执行语义缺口封顶（serve-custom-response.py 等以静态文本投递，
> runner 无 .py 执行器），转 runner 可达 classic 属性面。**本轮两刀**：
> ①shim part03 R3019 占位区 `HTMLScriptElement` 接口对象 + 静态 `supports(type)`
> ——HTML #dom-script-supports 四值精确匹配（'classic'/'module'/'importmap'/
> 'speculationrules'→true，JavaScript MIME 类型/空白沾染/大小写变体全 false；
> spec 明示不做 ASCII case-insensitive、不剥空白；webReader 取回 spec 原文定谳，
> 语料为旧版无 importmap/speculationrules 断言，按 spec 现行实现两相兼容）；
> shimming 前 grep 定谳 shim/Rust 侧零 HTMLScriptElement 全局引用，`||` 守卫
> 无冲突面。②extract `trim_ascii_html_ws`（TAB/LF/FF/CR/SPACE）替换两函数的
> `str::trim`——type 剥离收窄到 ASCII whitespace，\x0B/\x85/\xA0/ /　
> 尾随不再误判 JS；调用方仅脚本提取循环两处，+单测 ASCII 照剥/非 ASCII 不提取
> 双形态。**验证**：定向 script-supports 5/5、script-type-whitespace 5/5 全绿；
> 全子树复扫（FILTER=scripting-1 加宽口径）540 行 211P/282F/25T+22 NotRun，
> diff vs 基线 9 行 F→P（两面全数）+23 行非 Diff 逐行归因：1 行 noscript Pass
> = 口径加宽（本来就绿）、22 行 = 超时页 teardown 粒度——**stash 复归位 s92
> 代码重建复跑 load-error 族 bitwise 同现（diff 空）+当前二进制全扫/单跑行集
> 一致**定谳非附带；基线缺行系 s92 扫描会话后台负载早停。历史口径重述
> **210P/282F/25T（517 行恒等，+9P −9F）**，域账 **211P/282F/25T**（+1 惯例）。
> 单测：zero-engine **3531/3531**（lib 3527=3526+1；engine 自 s92 零外来变更）；
> fmt/clippy clean（ulimit+timeout shell 兜底）；reftest 未跑（无 render/layout
> 面，沿 s91/s92 影响面口径）。账本见
> [evidence/2026-10-26-m2s93-classic-script-attribute-faces.md](html-semantics-compat/evidence/2026-10-26-m2s93-classic-script-attribute-faces.md)。
> **范围**：engine ×2（shim 一处 + extract 一函数族）；runner/webview/
> script-sandbox 零改动；css/layout/renderer 零改动；语料不可改维持。
> **挂账**：完整接口对象面（HTMLScriptElement 仅静态 supports，constructor/
> instanceof 同一性缺，`__zwHtmlTagIface` 全表无真实 class 同族挂账）；module
> 尾片 43F 簇（.py 执行器为前置，含 propagate-nonce-inline-module 1F 与
> module 路径 CSP 检查点缺位承袭）；execution-timing 80F / 顶层 65F（既有账）；
> runner 报告粒度受会话负载影响（超时页 teardown，基线扫描宜空载跑——s93 终扫
> /tmp/s93-final-scan.txt 定为下轮基线，FILTER=scripting-1 口径 540 行）。
> **下一步**：s94 入场探针定刀位（候选：execution-timing 80F scheduler 语义 /
> 顶层 65F / 接口对象基建面）。
> **切片间隙**：本轮 engine 面；下一挂账同流同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 92 施工收口，CSP nonce 传播轮——其下一步 s93 module 尾片盘点入场转向 classic 属性面，已由 s93 收口，见上）**：
> **入场核账**：s91 HEAD（abdd1b178）基线 **197P/295F/25T**。残面：
> string-compilation-nonce-{classic,module} must-not-inherit 4F（s88
> 挂账 CSP 面）——setAttribute onclick 两臂 spec 语义：事件处理器不
> 继承触发脚本 nonce → 体内 import() 的 module fetch 被 `script-src
> 'nonce-correct'` 拒 → TypeError reject；红因 = 引擎动态 import 宿主
> 解析路径无 CSP 检查、无 nonce 传播通道（import 成功 →
> promise_rejects_js 断言失败）。**既有基座**（security-hardening
> M2-s1~s7）：meta CSP 解析、`check_script(nonce,hash,url)`、violation
> 事件、eval/attr 门（`unsafe-eval` 放行页不激活）——缺口仅在
> import fetch 检查点。**本轮三刀**：①script-sandbox
> `DynamicImportState.nonce` 槽位 + resolver 契约升三参（空串↔None
> 双射）+ `set_dynamic_import_nonce`/`__zwSwapImportNonce` swapper
> （trait 默认 no-op，QuickJS 降级；空契约路径既有 type_error reject
> 零改动命中判据）；②webview 逐脚本 nonce 刷新（spec 动态 import 继承
> 触发脚本 nonce，hostimportmoduledynamically-retrieval）+ resolver
> 检查（violation 入 connect 共享队列 + 返空串）；③shim 求值窗——
> 处理器传 `''`（不继承）、timer 注册期捕获（字符串编译继承）、R387b
> 窗换 `''`（动态插入无 nonce）。**返工（投递保真）**：首轮全子树
> 2 行附带——propagate-nonce-external-{classic,module} 翻红，ZWDBG
> 双探针（resolver 收参 + 刷新序列，跑完即撤）定位：页面外链测试脚本
> 经 runner `inline_local_scripts` 内联投递丢原标签 nonce 属性 → 刷新
> 读 None → 槽位空。修复：投递保留 nonce（复用 extract
> `attr_from_open_tag` 升 pub，`&`/`"` 转义；上方 harness 注入块
> csp_nonce_attr 同款保真先例）。**验证**：nonce 两页 12/12（4 红臂
> 翻绿 + 8 inherit 臂在 CSP 检查开启下保持绿）；propagate-nonce 4 页
> 3 绿 1 红（inline-module 1F = s91 前既有 module 路径面）；终扫
> **201P/291F/25T**，diff vs 基线恰 4 行全落 nonce 族、族外零移动，
> **string-compilation 族 46P/0F/0T 完全清盘**（s88 入场 3P/3F/8T）。
> 单测六腿：runner **226**、engine **3526**、sandbox v8 **364**（+2
> 新测）、quickjs **259**、webview **800**；fmt/clippy 四 crate
> `-D warnings` clean（ulimit+timeout shell 兜底同 s91）；reftest 未
> 跑（无 render/layout 面，无政策快路径返 None，沿 s91 影响面口径）。
> 域账自此按 **202P/291F/25T** 记（raw 201P + 惯例 +1）。账本见
> [evidence/2026-10-26-m2s92-csp-nonce-string-compilation.md](html-semantics-compat/evidence/2026-10-26-m2s92-csp-nonce-string-compilation.md)。
> **范围**：script-sandbox ×2 / engine ×2（shim + extract 一词 pub）/
> webview / wpt-runner runner 侧五 crate；css/layout/renderer 零改动；
> 语料不可改维持。
> **挂账**：propagate-nonce-inline-module 1F + module 路径动态 import
> CSP 检查点缺位（es_module 重写走 `__zw_compile_module` shim 路径
> 绕开检查——与 classic 槽位路径正交）；`inline_local_scripts` 不保留
> `type="module"`（投递语义保真，本刀语料不受影响）；security-csp 域
> 旁证（他流域控制面，下次跑批建议重基线）；QuickJS 后端降级维持。
> **下一步**：s93 module 尾片盘点——the-script-element 余账
> （propagate-nonce-inline-module 等 module 路径面）按 s86 切片序④
> 入场先探针定刀位。
> **切片间隙**：本轮 script-sandbox/engine/webview/runner 面；下一
> 挂账同流同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 91 施工收口，动态 import() 重写器词法感知 + module 面收口轮——其下一步 s92 CSP nonce 面已由 s92 收口，见上）**：
> **入场核账**：s90 HEAD（8c4a575dd）the-script-element 全子树复扫
> **173P/294F/30T**（497 行）。残面：string-compilation-module
> eval 1F + 整页 1T、base-url-inline-module / nonce-module /
> base-url-external-module 整页 3T、base-url-external-classic 整页 1T
> （语料缺 dynamic-import/scripts/——fetch_dir_html depth-1 不下钻）。
> **根因**（STEPS 探针串实证）：`rewrite_dynamic_imports` 纯文本扫描
> 把**字符串字面量内** `import(` 也改写——字面量被 Function/eval/
> setTimeout 字符串体编译后在全局作用域执行，体内 `__zw_dynamic_import
> .call({ referrer: _importMeta.url }, ...)` 引用 IIFE 局部绑定
> `_importMeta` → ReferenceError，import 永不发生；WPT 子测试名（显式
> 字面量）在 s90 扫描被污染成 `__zw_dynamic_import.call(...)` 同源实证。
> **本轮三刀**：①es_module 重写器 `RewriteCtx` 栈词法感知——Literal
> （引号定界/转义对）/TemplateExpr（`${}` 内照常重写、花括号计深）/
> 行注释/块注释，栈空代码态照旧；5 拷贝点改 `char::len_utf8` 步进
> （首版按字节切遇 WPT 语料 `ś` panic、runner 批次 rc=101 全灭——返工
> ①）+多字节单测钉死；已知限制：含引号正则字面量误入 Literal 态、
> 漏改走 V8 原生宿主回调语义仍对；②语料 fetch_raw 补 scripts/ 5
> loader（gitignore 本地数据）+ part04 R387b 外链经典脚本求值窗
> `__zwSwapImportReferrer(自身绝对 URL)` finally 换回（concept-scripts
> -base-url）；③v8_runtime swapper 无参调用 = 纯 getter + part01
> 字符串 timer 注册期捕获基（fire 在求值窗外槽位已被改写）+ part03
> `_zwCallHandlerSwapBase(fn, thisArg, ev, base)` 拆通用形、
> `_zwCallHandlerDocBase` 变薄壳。**验证**：全子树基线/修后双扫 diff
> 归因配对核清零误伤：**+24P +1F −5T**——module 5（eval F→P+4 拆臂）、
> base-url-inline-module 5、base-url-external-classic 5、
> base-url-external-module 5、nonce-module 4P+2F（must-not-inherit
> 拆臂显形既有红面非回归）；族外 27 行派生名恢复 + import-meta 1 行
> 同名异写状态不变（名恢复即刀A 生效直接证据）。族终态 **42P/4F/0T**。
> 单测 engine **3526/3526**、webview **800/800**、sandbox v8 **362/362**
> （含新增 4 测）、quickjs 全绿；fmt clean、clippy 双 crate
> `-D warnings` clean（用户 systemd scope 本环境不可用，ulimit 24G +
> timeout 900 shell 兜底）。域账自此按 **198P/295F/25T** 记（raw
> 197P + 惯例 +1）。账本见
> [evidence/2026-10-26-m2s91-module-context-lexer-aware-rewrite.md](html-semantics-compat/evidence/2026-10-26-m2s91-module-context-lexer-aware-rewrite.md)。
> **范围**：script-sandbox ×2 / engine shim ×3 文件；css/layout/
> renderer 零改动；语料不可改维持（探针页跑完即删）。
> **挂账**：nonce-classic/-module must-not-inherit 4F（CSP nonce 语义
> 未建模，s88 已挂）；正则字面量限制（漏改走原生回调语义对、换基精度
> 降）；语料 fetch 深度工具局限在案；QuickJS 后端动态 import 不支持态
> 维持。
> **下一步**：s92 二选一定刀位——CSP nonce must-not-inherit 4F（引擎
> CSP 语义建模，s88 挂账合并）或 module 尾片盘点；开工先探针。
> **切片间隙**：本轮 script-sandbox/engine 面；下一挂账同流同面，
> 间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 90 施工收口，string-compilation 动态 import() 字符串编译轮——其下一步 s91 module 上下文面已由 s91 收口，见上）**：
> **入场核账**：s89 HEAD（44e23748f）the-script-element 全子树复扫
> **158P/294F/33T**（485 行）。string-compilation 族基线 3P/3F/8T：
> classic 整页 T（字符串 setTimeout handler 被静默丢弃永不 fire，堵死
> harness 串行队列连累 4 臂 pending）、promise-result 2F（执行中换
> `<base>` 后 import 基走错）、base-url-inline-classic / nonce-classic
> 整页 T、module 系 4T + module eval 1F、base-url-external-classic 1T。
> **本轮三刀**：①part01.js `setTimeout/setInterval` 补字符串 handler
> `new Function` 注册期编译（HTML spec TimerInitializationSteps 的
> WebIDL callback 转换；语法错同步抛同 Chrome）——classic 页解堵拆臂；
> ②engine extract 新增 `script_effective_base_urls`（zero_dom 树序，
> `<base href>` 更新 current_base、script 记基，与 extract_page_scripts
> _indexed 同序号空间）接 webview `run_page_scripts_impl`——外链经典
> 脚本 referrer 用自身 src、内联用捕获表（concept-scripts-base-url：
> parse 时刻定格，动态 base 不回溯）；③script-sandbox 新增
> `register_dynamic_import_referrer_swapper`（`__zwSwapImportReferrer`
> 返旧换新，捕获 DynamicImportState Arc 不借 &mut self）+ shim
> `_zwCallHandlerDocBase`（inline/反射 handler 求值窗换当前文档基、
> finally 换回零外溢）接四处编译点——reflected/UA 臂 import() 用求值
> 时刻文档基（event-handler-processing-model）。**返工两笔记账**：
> 原文偏移扫 `<script` ≠ DOM 序号（harness 注入 + 字符串字面量过计数，
> DEBUG 探针实证后改 zero_dom 树序）；body-reflect `var fnB` 循环闭包
> 读末轮绑定（R2946 单测红，改 `_zwWrapHandlerDocBase` 参数绑定）。
> **验证**：全子树 stash 基线/修后双扫 diff 恰 31 行全落族内零误伤：
> **+20P −2F −3T +2F（nonce must-not-inherit 从整页 T 拆臂显形，非回归）**
> ——族 3P/3F/8T → **18P/2F/5T**（classic 5/5、promise-result 4/4、
> nonce 继承臂 4/4、base-url-inline-classic 5/5 全绿）。单测 engine
> **3526/3526**、sandbox v8 **358/358**、quickjs **255/255**、webview
> **800/800**（首跑 1F 系 target-disk-guard 清理 zero-image-decoder
> 独立进程二进制，补建全绿非代码回归）；fmt clean、clippy 三 crate
> clean（test-guard 包裹）；reftest **716/716** rc=0。域账自此按
> **174P/294F/30T** 记（raw 173P + v8-code-cache +1 惯例；F 双扫同
> 294，对 s89 账面 295F 为截断方差）。账本见
> [evidence/2026-10-26-m2s90-string-compilation-base-url.md](html-semantics-compat/evidence/2026-10-26-m2s90-string-compilation-base-url.md)。
> **范围**：engine shim ×2 / engine extract / script-sandbox ×2 /
> webview 六文件；css/layout/renderer 零改动；语料不可改维持（探针页
> gitignore 数据跑完即删）。
> **挂账**：module 上下文 5 文件 T（`<script type="module">` 页基座，
> module 面单独探）；base-url-external-classic 1T（dynamic
> createElement('script') 外链页面加载器）；nonce-classic must-not-
> inherit 2F（CSP nonce 语义未建模，s88 已挂）；QuickJS 后端
> classic 动态 import 不支持态维持（swapper trait 默认 no-op 降级）。
> **下一步**：s91 the-script-element 余账——module 页基座先行探针
> （区分页面加载失败 vs import 永不落定），再决 external-classic
> 动态加载器刀位。
> **切片间隙**：本轮 engine/script-sandbox/webview 面；下一挂账同流
> 同面，间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 89 施工收口，execution-timing stylesheets-blocking 轮——其下一步 the-script-element 基面 classic 面已由 s90 收口，见上）**：
> **入场核账**：s88 基线（HEAD=4fdaf03e5）execution-timing 目录当拍
> **48P/94F/8T**——s86 记「12F 单簇」细分为 106-\* ×10 + 107-\* ×2 的
> 「scheduler: stylesheets blocking \*」目标行；其余 82F 编号套件预存红
> （s88 基线同态非回归）。四根：①语料根——`fetch_dir_html` 扩展名表缺
> `*.css`，css/background.css（`#test{position:fixed}`）与 css/import.css
> （`@import url("background.css")` 无分号）从未入语料；②装填根——
> `set_active_external_css` 生产恒空（唯一调用点是单测），getComputedStyle
> 桥读空级联；③@import 根——进程内无递归展开，且无分号语句文本拼进合并
> 级联后 css-syntax at-rule 消费遇裸 `{` 吞块，把**下一张表首条规则**吞掉
> （106-import 实证：双表 fetch 序正确探针仍 `static`）；④107 根——隐式
> write 走 `body.innerHTML=基线` 重放，原 `#test` 注册元素被 plain 产物
> 替换（无 `__zwSelector`）→ position 不在 UA 默认表返 `''`。
> **本轮一刀（四文件）**：webview `load_html` 同步发布外链样式表文本
> `set_active_external_css`（css-less 传空串复位防跨文档泄漏）+
> `resolve_external_css` @import 递归队列（对齐 R2411：相对导入表 URL、
> css_seen 防环、FIFO 文档序）；css_urls 新增 `strip_import_statements`
> （Tokenizer span 切除顶层 @import 语句，接线两处拼接点，css-parser
> 未动回避跨流改写）；engine shim getComputedStyle `position` 初始值
> `static` 回落（css-position-3，R5009 float 同形）；runner lib.sh 扩展
> 名表补 `*.css`。FIXME：@import 臂不走 CSP/Mixed 门（async_load 既有
> 同态）；级联顺序近似（子表追加在导入表文本后，逐位拼接归基建轮）。
> **验证**：execution-timing 子树 **48P/94F/8T → 62P/80F/8T**，stash 基线
> 逐行 diff 恰 **14 行 Fail→Pass**（106×10 + 107×2 + 150×2 同根红利）、
> Pass→Fail 0 行、T 不变；the-script-element 全子树 **158P/295F/33T**
> ——非目标 144 行 Pass 逐位不动（零误伤），F 差 1 为 TLA 截断点漂移
> 报表方差；dynamic-import 金丝雀与 stash 基线 bitwise 同态（对 s87
> 账面漂移为环境方差不记本刀）。单测 webview **799/799**（含探针三钉
> + css_urls 4 钉）、engine **3523/3523**；fmt + clippy 三 crate clean；
> reftest **716/716**（清缓存全量重编）。域账自此按 **159P/295F/33T** 记
> （raw 158P + v8-code-cache +1 惯例，F −13 与 P +14 之差同截断方差）。
> 账本见 [evidence/2026-10-26-m2s89-execution-timing-stylesheets.md](html-semantics-compat/evidence/2026-10-26-m2s89-execution-timing-stylesheets.md)。
> **范围**：生产面 webview ×3 / engine shim / runner lib.sh 五文件；
> script-sandbox/layout/css-parser 零改动；语料不可改维持（css 两文件
> 已按 pin 补拉本地，gitignore 数据命令留档）。挂账：编号套件 82F
> （trickle pipe/多文档时序）、webview 第 2 跑单失败未复现 flake。
> **卡点（非本流）**：`make test` 预存红维持 s86 记录——quickjs
> zero-renderer `mutation_arm_…reset_window`（归 perf/renderer 流未解）。
> **下一步**：s90 the-script-element 基面（s86 切片序③——string-compilation
> 余 4 臂挂 runner 计时器/Function Proxy/事件派发建模、nonce CSP、
> base URL、delay-load trickle、credentials M3；明细见 s88 账本 §4）。
> **切片间隙**：本轮 webview/engine/runner 面；下一挂账同流多面
> （shim/runner/CSP），间隙开放。

> **▶ 上轮状态（2026-10-26 M2 切片 88 施工收口，classic-script dynamic import 轮——其下一步 stylesheets-blocking 12F 簇已由 s89 收口，见上）**：
> **入场核账**：s87 遗留 classic 根 `Error: Not supported` 面当拍
>（HEAD=a2b8e71e8 全子树基线复扫 138P/313F/34T，域账面 −1P 为
> v8-code-cache 未上报同缺）细分：propagate-nonce-{inline,external}-
> classic 2F、string-compilation-classic 1F+1T、inline-event-handler 1T、
> base-url/nonce-classic 3T、css-module/import-css-module-dynamic 1F。
> **s87 归因勘误**：credentials 12F/1T 整族走 iframe + HTTP cookie
> handler（fetch set-cookie.py → iframe appendChild →
> fetch_tests_from_window），**非** classic import 根（本轮修复对其零
> 移动反证）；delay-load-event 1F 须异步取回 + trickle 时序建模。
> **本轮一刀（共享路径）**：script-sandbox 注册 V8 原生
> `set_host_import_module_dynamically_callback`（isolate slot 持
> referrer+resolver），三态消费与 `__zw_load_module` 同款契约（返空→
> 逐次新 TypeError / 空码→TypeError 携已解析 URL / `resolved\x1fiife`→
> 按需最小安装 prelude helpers + TryCatch 编译求值）；未注册 resolver
> 保 V8 默认 `Error: Not supported` 零外溢；`Sandbox` trait 新增
> `set_dynamic_import_resolver/referrer` 默认 no-op（QuickJS 降级不变）。
> webview `__zw_compile_module` 闭包体提升 `compile_module_contract`
> 共享函数，原生 resolver 同契约；每 classic 脚本前刷 referrer=文档 URL。
> **验证**：the-script-element/ 全子树基线/修复后双扫 bitwise
> **138P/313F/34T → 144P/308F/33T**，diff 恰 6 行全红→绿零误伤
>（nonce-classic ×2 + nonce-external-module + string-compilation-classic
> :: eval 4×F→P；inline-event-handler T→P；css-module/import-css-module-
> dynamic F→P 域外红利——无 type CSS module 动态 import 现按契约 reject
> TypeError）；fetch-error 16P/script-error/credentials 逐位不动。单测
> v8 **357/357**（+5 classic 原生 import 钉：成功/eval 字符串/取回失败/
> 图断/无 resolver 保真）、quickjs 255/255、webview 792/792；fmt +
> clippy `-D warnings` 两 crate clean。域账自此按 **145P/308F/33T** 记。
> 账本见 [evidence/2026-10-26-m2s88-classic-dynamic-import.md](html-semantics-compat/evidence/2026-10-26-m2s88-classic-dynamic-import.md)。
> **范围**：生产面 script-sandbox/webview 两 crate；runner 零改动；
> apps/browser 与 renderer 宿主未动（未注册 resolver 行为不变）；语料
> 不可改维持。FIXME 挂账：classic 上下文重复失败 `__moduleFailures`
> 同对象记账未接；目标模块体 import.meta 无注入。**卡点（非本流）**：
> `make test` 预存红维持 s86 记录——quickjs zero-renderer
> `mutation_arm_…reset_window`（归 perf/renderer 流未解）。
> **下一步**：s89 execution-timing stylesheets-blocking-scripts 12F 簇
>（s86 切片序②）；string-compilation 余 4 臂/nonce CSP/base URL/
> delay-load/credentials 残面归基面③、CSP、runner 建模、M3 各账
>（明细见账本 §4）。
> **切片间隙**：本轮 script-sandbox/webview 面；下一挂账 runner 面
>（stylesheets-blocking-scripts 时序），间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 87 施工收口，dynamic-import must-reject 族轮——其下一步 classic 面已由 s88 收口，见上）**：
> **入场核账**：s86 切片序①当拍 dynamic-import 子树 **6P/49F/11T**
>（FILTER=the-script-element/module/dynamic-import）。must-reject 24F
>「单根」经当拍细分为三根：①错误类型根 18F——`__zw_load_module` 宿主返空
> 一律普通 `Error`，spec fetch-a-single-module-script 取回失败须
> TypeError；②对象身份根 4F——parse/求值失败不缓存，重复 import 每次新
> 对象（spec module map 失败记账：parse/求值/依赖解析失败复抛同对象，
> fetch 期失败逐次新对象）；③MIME 根 2F——wptserve
> `?pipe=header(Content-Type,…)` 响应头模板未建模，wrongMimetype 按扩展名
> 误判 scripty。**本轮一刀（三面）**：es_module prelude `__zw_load_module`
> 失败三态（宿主返空→逐次新 TypeError 不入缓存 / `resolved\x1f` 空码标记
> →TypeError 入 `__moduleFailures` 复抛同对象 / eval 抛错→原错误对象入
> 缓存复抛）；webview `collect_module_deps_recursive` 返
> `ModuleGraphFailure` 三分类（TopFetch/DepResolution/DepFetch——裸
> specifier 无 import map 即解析失败），`__zw_compile_module` 失败两态
> 回传（fetch 失败返空 / 解析失败返空码标记）——renderer/tab 旧宿主返空
> 契约降级不劣化；runner `wpt_data_script_fetcher` MIME 派生序新增最高
> 优先 `?pipe=header` 臂。**验证**：子树 **6P/49F/11T → 28P/27F/11T**；
> the-script-element 全子树 stash 基线复扫 bitwise——**116P/335F/34T →
> 138P/313F/34T，diff 恰 22 行全 Fail→Pass、全落 dynamic-import/ 目标
> 族零误伤**（TLA 3 案页脚抛 rc=1 截断两轮同态、`v8-code-cache.html`
> 未及上报，runner 行为先于本轮）；域账自此按 **139P/313F/34T** 记。
> 单测：es_module 新增失败身份语义四态钉（取回 TypeError×新对象/图断
> TypeError×同对象/parse SyntaxError×同对象/求值 Error×同对象+体恰跑
> 一次）+ v8 352/352 + quickjs 8/8 + webview 792/792；fmt clean + clippy
> 三 crate clean（video_registry map_entry 门禁已由渲染流 `9d50e4a91`
> 同刻自修解除，见卡点）。账本见
> [evidence/2026-10-25-m2s87-dynamic-import-must-reject.md](html-semantics-compat/evidence/2026-10-25-m2s87-dynamic-import-must-reject.md)。
> **范围**：生产面 script-sandbox/webview/runner 三文件 + 契约单测；
> apps/browser 与 renderer 宿主未动（返空契约降级路径行为不变）；语料
> 不可改维持。**卡点（非本流，碰头记录）**：`make test` 预存红维持 s86
> 记录——quickjs zero-renderer `mutation_arm_…reset_window` 本轮复跑仍红
> 0.63s（page_scripts.rs 无新提交，归 perf/renderer 流未解）。
> **碰头记录（已化解）**：video_registry map_entry 代修与渲染流 `9d50e4a91`
> 同刻撞车（同款 entry API 改写）——保留上游版，本流重复提交 rebase
> skip 移除，零净损；渲染流 guarded-clippy 已全绿佐证门禁解除。
> **下一步**：s88 classic-script dynamic import 重写——classic 脚本
> `import()` 未重写，裸 V8 无宿主回调 reject `Error: Not supported`
>（credentials 12F + delay-load-event/propagate-nonce-classic/
> string-compilation-classic 同根），配 `check-cookie.py`/`set-cookie.py`
> runner handler 仿真 + 跨源 cookie jar；instantiation 2F 归 module 基面
> 切片④。
> **切片间隙**：本轮 script-sandbox/webview/runner 面；下一挂账同面
>（script-sandbox 重写），间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 85 定证收口，s84 根因证伪轮——其下一步挂账经 s86 核账已被 s79/s80 消费，见上）**：
> s84「gBCR materialize 滞后」根因三路定证**证伪**（s53 先例零代码轮，
> 插桩临时已全部回退/删除，工作树回 s84 基线）：①take_probe 链直插
> （ZW_PROBE_TRACE）——corpus complex 逐轮 `flush 即 apply → version
> 同拍 bump → 门即开 → changed=true → refresh fired=true`，GATE MISS
> 0 次（fired=false 仅 html 未变轮，正确 no-op），s84 三处漏批候选
> （version 门时序 / runner_probe_html 基线 / timer 泵批次归属）无一
> 成立；②Rust 复现测试（复刻 probe loop 双结构逐拍采样）——round 0
> rect 即非零（normal dialogA@8,100,784,18 …、slots dialogA@8,100,0,36
> …），12 轮 + +100ms 稳定，s84「三拍恒零、+100ms 才有效」不存在；
> ③TD 点击坐标实测——normal 嵌套 20/20 Pass、clickOn 落点非零命中
> 正确目标。**红图修正**（make + FILTER 当拍实测）：closedby 家族
> **104P/19F**——closedby 27P/0F、stacked 4P/0F、start-open 2P/0F、
> **simple 6P/0F（s84 记 2F 过时复绿）**、corner-cases 29P/1F（s53
> keep）、bounds-clicking 12P/2F（R-s31 渲染流）、complex 24P/16F
> （16F 全 shadow-slots，normal 20/20 Pass）。**归账**：16F=declarative
> shadow 联合切片（s51 用户门禁维持）、2F=渲染流、1F=keep——本流
> 工作面（engine shim/webview/runner）无剩余可施工面。s84 伪值推断
> （存疑不下死论）与「红图当拍复核」教训见账本。账本见
> [evidence/2026-10-25-m2s85-closedby-rootcause-falsified.md](html-semantics-compat/evidence/2026-10-25-m2s85-closedby-rootcause-falsified.md)。
> **范围**：零代码轮（s53 先例），不改生产面；corpus 不可改维持
> （正常面下我方 rect 同步可见，无需「新鲜度」修复）。
> **下一步**：testdriver-Actions 支持面（s78 挂账：hash-name-dynamic
> 5F + 指针/键盘合成输入族——runner 语料 testdriver.js 缺席面，runner
> 面本流工作面）。
> **切片间隙**：本轮零代码；下一挂账 runner 面亦本流，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 84 分诊收口，closedby 同红假设重估轮——s85 证伪其根因，见上）**：
> A 族 closedby×slot 16F 挂账依据（crbug 417571176 同红，2026-04）重估
> **过时**：wpt.fyi chrome-158.0.8122.0 summary 实证 closedby 家族全绿
> （complex 40/40、simple 6/6、closedby 27/27、bounds 14/14、corner
> 30/30、stacked 4/4、start-open 2/2）——closedby 已 ship，**23F 转可
> 施工**（complex 16 + simple 2 + bounds 2 + corner 1）。**根因隔离**
> （三步插桩探针，临时已删）：失败首腿全是 light-dismiss 误触发——
> 首测点击 popover 的 ld 落点 px=0/py=0（corpus clickOn 2-rAF 后采样
> rect 恒零）→ s30 backdrop retarget → popover pass 误隐；gBCR 时序
> 探针证 showModal/showPopover 后 sync/2raf/4raf 三拍 rect 恒零、
> +100ms 才有效，且序列化 htmlLen +23（closedby+open）与 rect 转有效
> 同拍——mutation 批 materialize（apply→render→rect 快照）在 rAF 驱动
> 节拍整链滞后 ~5+ 探测轮。**下一步**：Rust 侧定证 take_probe
> flush/version-gate/refresh 链哪环漏批（候选：mutation_version 门
> 时序、runner_probe_html 基线、timer 泵与 rAF 拍批次归属），修复 rect
> 新鲜度（closedby 23F 与 abspos-dialog-layout 5F 共桥——本轮顺带观测
> `:modal` UA position:fixed 未上、dialog 盒 in-flow）。账本见
> [evidence/2026-10-25-m2s84-closedby-parity-recheck.md](html-semantics-compat/evidence/2026-10-25-m2s84-closedby-parity-recheck.md)。
> **范围**：零代码轮（s53 先例），不改生产面；corpus 不可改（2-rAF
> 采样是上游语料行为，Chrome 同步 rect 语义使其天然正确——修复须落
> 我方 rect 新鲜度）。
> **下一步**：s85 修复片——Rust 契约测试复现 gBCR 滞后 + take_probe
> 链定证 + materialize 时机修复；验证门 closedby 家族 23F 红转绿 +
> 域/pin 逐位持平。
> **切片间隙**：本轮零代码；下一挂账 webview/runner 面（rect 快照链
> ——本流工作面），间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 83 收口，handle 形元素 matches complex 读面轮）**：
> s82 遗留 matches complex 面分诊**推翻非平凡判定**：probe 实证 gap 非
> 同 turn 窗口而是**永久**——post-drain `matches('#anc span')` 仍 false；
> qS 倒查命中并登记 R100 锚，但 t6 锚臂 host `__zw_matches` 读原始
> dom_html 快照（apply 后锚不可解析），'0' 无条件 return 短路。s82 已
> 落双形态父链解析 + compound 匹配器，JS 侧判定材料齐备（无需 Rust
> 回调）。**修复**：part05 `_zwHandleMatchesComplex` 三态契约（true/
> false/null——null=有不可判 complex，落回锚臂 host 全语义，pseudo 面
> 不劣化）+ part04 t6 臂重排（R134 → s83 JS 走 → 锚臂 → regex）。列表
> 逐 complex 判定；compound 带 pseudo/组合器非后代/子代保守不判（防
> over-match）；48 跳封顶；不越 shadow 边界。**关键序**：JS 走必须先
> 于锚臂——锚臂 '0' 短路后置补偿不可达（probe 定位）。契约
> dom_bridge_tests `handle_element_matches_complex_s83`（18 断言）。
> **验证**：probe 同 turn + post-drain 全 Pass（临时已删）+ engine lib
> **3523P/0F** + 域（FILTER=embedded-content）**976P/252F/30T 逐位
> 持平 REG 0** + interactive pin **439P/44F/0T 逐位持平** + fmt clean
> + clippy 仅 s78 既有警告。账本见
> [evidence/2026-10-25-m2s83-handle-matches-complex.md](html-semantics-compat/evidence/2026-10-25-m2s83-handle-matches-complex.md)。
> **范围**：兄弟组合器（`+`/`~`）保守 false（反链无前兄弟指针，语料
> 0 需求）；锚臂 host 快照底层未动（JS 走已覆盖可判面，host 修复仅剩
> pseudo complex 面，无已知消费方）。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成
> 面，启动前查 oracle master 碰撞记录）。
> **切片间隙**：本轮 js_dom_shim 两件 + 契约 dom_bridge_tests；下一
> 挂账 layout 集成面需跨流协调，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 82 收口，handle 形元素 closest 读面轮）**：
> s81 范围注记的「同 turn insert 身份缺口」分诊升级：createElement 产物
> 是 handle 形代理（`__zwSelector` 恒 null），closest 的 `!sel` 门使其
> **永久盲**（非同 turn 窗口——已 append 排水后仍 null；身份永驻 handle
> 形）。分诊矩阵实证：matches 单 compound true（t6 臂）、complex false
> （stale 缺口）、closest ×3 全 null；语料需求 0（117F × createElement
> 39 处 × closest/matches 共现 = 0），立项按 spec 定义 + 真实站点
> slice（jQuery/Swiper 创建元素 `closest`）。**修复双臂**
> （part05 `_zwHandleClosest` + part04 closest 入口 handle 分支）：
> ①R100 锚回（曾 sel 向解析过）→ host `__zw_closest` 全语义；②JS 单
> compound 祖先走——父链双形态解析（sel→`__zw_parent` 视图文档；handle
> →`_zwNodeParent` 反链），48 跳封顶（R125 口径），nodeType≠1 停
> （不越 shadow 边界——spec closest 祖先轴不含 shadow host），complex/
> 伪类保守 null（`_matchCompoundOf` 不求值 pseudos，防 over-match）。
> 契约 dom_bridge_tests `handle_element_closest_s82`（15 断言）。
> **验证**：probe 4 面 Pass（临时已删）+ engine lib **3522P/0F** +
> 域（FILTER=embedded-content）**976P/252F/30T 逐位持平（REG 0——
> 语料 0 需求预判实证）** + interactive pin **439P/44F/0T 逐位持平** +
> fmt clean + clippy 仅 s78 既有 concat! 警告（不在本 diff）。账本见
> [evidence/2026-10-25-m2s82-handle-closest.md](html-semantics-compat/evidence/2026-10-25-m2s82-handle-closest.md)。
> **范围**：matches complex stale 面（handle 形锚回 host 快照 miss）
> 记遗留不建——需 Rust 视图文档 matches 回调或 query_all 回退，非平凡。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成面，
> 启动前查 oracle master 碰撞记录）或 matches complex stale 面分诊。
> **切片间隙**：本轮 js_dom_shim 两件 + 契约 dom_bridge_tests；下一
> 挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 81 收口，查询面 id 改名可见性轮）**：
> R125（s79）读侧只接 gEBI；查询面（querySelector/querySelectorAll/
> matches/closest）同 turn 改名 `#new` miss、`#old` stale hit。probe
> 分轮定位**根因补刀**：`el.id` IDL 反射读走纯快照（与 getAttribute
> 的 R2995 lw 链同 turn 分歧）——`_matchCompoundOf` ids 校验读
> `p.id` 故 token 形态拉真全灭。**修复两刀**：①part03 id 臂 lw 化
> （共享路径，DOM 反射 spec 同步语义；`_zwRawIds`/handle 形保持）；
> ②part05 s81 helpers + 六挂点双臂补偿——stale 剔除（lw 现值复验，
> 全形态）+ 单 compound 拉真（R125 表候选 + in-doc 门 + full
> compound 复验 + 子树门；qSA 文档序归并）。R3190 属性语义不动
> （补偿只动选择器 id 约束）。契约 dom_bridge_tests
> `query_id_rename_same_turn_visibility_s81`（17 断言）。
> **验证**：细粒度 probe 12 面 Pass（临时已删）+ engine lib
> **3521P/0F** + 域（FILTER=embedded-content）**976P/252F/30T 逐位
> 持平（REG 0 / GAIN 0——s80 预判无直接语料依赖，实证）** +
> interactive pin **439P/44F/0T 逐位持平** + fmt clean。
> 账本见
> [evidence/2026-10-25-m2s81-query-id-rename-visibility.md](html-semantics-compat/evidence/2026-10-25-m2s81-query-id-rename-visibility.md)。
> **范围**：complex 祖先段约束 pull-in 不接（single 门，stale 剔除
> 仍全形态生效）；同 turn insert 元素无 `__zwSelector`（批末 apply
> 才有身份）上行 closest/query 盲区——结构性身份缺口与改名无关
> （无 id 对照面同样 null 实证），后续独立切片。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集
> 成面，启动前查 oracle master 碰撞记录）或 s81 遗留同 turn insert
> 身份缺口分诊。
> **切片间隙**：本轮 js_dom_shim 四件 + 契约 dom_bridge_tests；
> 下一挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 80 收口，跨批 selector 迁移 journal 轮）**：
> s79 遗留 runner 缺陷 C（hash-name-reference-dynamic 3P/5F）定位并修复。
> s79 假说「序列化/apply 只经客户端缓存属性」**证伪**：Rust 侧 attr-trace
> 实证 mutations 队列不 drain（1→9 单调）、同键 lw 读亦通——丢点在
> **apply 解析**：全以创建期 sel 入队（proxy 闭包 sel 冻结），批 2 改名
> 落管线 doc 后，批 3 起旧键属性写 `find_by_selector` 恒 miss 走
> lenient skip 静默丢写；序列化无 name / 视图查询 miss / gEBI 旧键落空
> 均为下游表现。**修复（s80 journal）**：`apply_dom_mutations_full` 新参
> `sel_renames`——SetAttr(id) 落地记（改名前 sel → `#新id`），失配臂
> `journal_find_sel_rename` 追链（4 跳封顶）重试；直查命中优先（新元素
> 抢占旧 id 不劫持）。持有方 `RenderPipeline::sel_renames`，导航边界清、
> 同页 `render_html` 重建不清（selector→selector 映射不受 slotmap 换代
> 约束——首版在重建点清空致语料无效，复跑定位后修正）。契约
> dom_bridge_tests `test_apply_sel_rename_journal_cross_batch_attr_s80`。
> **验证**：engine lib **3515P/0F** + 域（FILTER=embedded-content）
> **976P/252F/30T（GAIN 5 / REG 0，hash-name-dynamic 3P/5F→8P/0F）** +
> interactive pin **439P/44F/0T 逐位持平**。范围：journal 只挂
> SetAttr/RemoveAttr 失配臂（SetText/SetStyle 等无语料实证面暂不扩）。
> 账本见
> [evidence/2026-10-25-m2s80-cross-batch-sel-rename-journal.md](html-semantics-compat/evidence/2026-10-25-m2s80-cross-batch-sel-rename-journal.md)。
> **下一步**：`querySelector('[id=…]')` 同 turn 改名可见性（共享查询
> 视图面）或 720-stretch 渲染展开 22F（需布局几何，归 layout 集成面，
> 启动前查 oracle master 碰撞记录）。
> **切片间隙**：本轮 js_dom_bridge + pipeline + webview 导航清点 +
> 契约 dom_bridge_tests；下一挂账同面或 layout 集成面，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 79 收口，IDL id 改名 R125 对称臂轮）**：
> s78 未解决项 hash-name-reference-dynamic 5F 分诊——**控制面原记
> 「testdriver-Actions 面」证伪**：runner 自带 test_driver.Actions stub，
> 8 行点击全链路可达（ZW_TD_DEBUG resolve 迹：首点击 `#area` 全通，首个
> DOM 变更后 map 解析失败）。5F 真实面 = shim 两缺陷（本轮修复）+ runner
> 属性传播缺陷（未解决，后续切片）。**修复（R5279 两刀）**：①part04
> `.id=` set trap 补 R125 sel-based id 覆盖表登记（与 setAttribute('id')
> 对称——缺登记同 execute getElementById miss 新 id；latest-wins 缺写使
> 「改名回原 id」被残留旧条目毒化）；②part06 getElementById 第二条
> pending-by-id 兜底环补 `_r125Overridden` 门（快照命中被覆盖表剔除后
> 不得经该环复活）。契约 part90 `idl_id_rename_getelementbyid_r5279`
> （8 断言，撤任一刀对应转红）。**验证**：engine lib **3512P/0F** + 域
> （FILTER=embedded-content）**971P/257F/30T 持平** + interactive pin
> **439P/44F/0T 逐位持平**（R5279 只增 id 覆盖登记，语料面零数字变化）。
> **新诊断（未解决）**：runner 命令 turn 的 resolve 时点**属性写丢失**
> （`setAttribute('name')` 不在 live 态亦不入序列化——id 写有客户端缓存
> 可见、name 写不可见，疑序列化/apply 管线只覆盖客户端缓存属性）+
> **覆盖表缓存代理空子树**（re-parse 代际后 gEBI 命中代理
> `querySelectorAll('area')` → 0）——hash-name-dynamic 5F 实际归属此组合
> 面；resolve 迹仪器化方法已验证（ZW_TD_DEBUG + 临时探针，跑完即删）。
> 账本见
> [evidence/2026-10-25-m2s79-idl-id-rename-r125.md](html-semantics-compat/evidence/2026-10-25-m2s79-idl-id-rename-r125.md)。
> **下一步**：runner 属性传播缺陷切片（refresh 管线属性 mutation 应用 +
> 代际代理失效，面在 tests/wpt-runner + shim 视图生命周期）或 720-stretch
> 渲染展开 22F（需布局几何，归 layout 集成面）。
> **切片间隙**：本轮 part04/06 shim + 契约 part90；下一挂账 runner 面
> 在 tests/wpt-runner，间隙开放。
>
> **▶ 上轮状态（2026-10-25 M2 切片 78 收口，F4 image-map 命中轮）**：
> s67 八族地图 F4 收口（the-area-element 三文件 area-coords 32F +
> area-shape 17F + area-processing 14F）。失败面修正：控制面原记 84F 系
> 旧账——实测 68F 中 5F 是 hash-name-reference-dynamic 的 testdriver-Actions
> 面（runner 语料缺 testdriver.js，指针/键盘合成输入族，本切片外），
> image-map 实际可收面 63F。**根因**：runner headless 无渲染管线——host
> `__zw_elementFromPoint`（R2924 HitTestCache）与 `__zw_getBoundingClientRect`
> 均缺席，elementFromPoint 缺席臂此前只有 SVG 形状回落，usemap img 形状
> 命中面为空 → 63F 全 null。**修复（R5278 三刀）**：①解析核（part03）——
> float-list 逐字解析（分隔集 ASCII 空白+逗号+分号、`\v` 非空白、垃圾跳读、
> 浮点前缀截断含指数 `.4`/`1e1`/`2in`、error→0）+ shape 枚举（ASCII
> case-insensitive，missing/invalid → rect）+ 点入形状（rect min/max 归一
> 半开框、circle d²≤r²、poly on-edge 预检**边界含入** + even-odd 半开边
> 射线——star (101,101)/(199,101)/hitPoly (119,101) 三行 corpus 定谳、
> default 恒真）+ hash-name reference；②命中面（part06 `_zwImgMapHitSel`）
> ——elementFromPoint 缺席臂（SVG 后置臂，产品路径零波及），img 框 (0,0)+
> IDL 尺寸近似（FIXME 真实几何归 layout 集），无形状覆盖/usemap 不可解析 →
> img 自身；③契约 part90 两测试（65 断言）。**排障留档**：裸跑 runner 缺
> Makefile 目标的 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`，interactive pin 假红
> 7 行（滚动/点击时序面）；stash 验证基线同红排除代码回归后改走 make 目标
> ——runner 数果实测必须走 Makefile 目标。**验证**：域（make + FILTER）run3
> **907P/320F/31T → 971P/257F/30T**（GAIN 64 = area 63 + usemap-casing
> 加成 1 / 0 P→F；audio-lazy T→P 为已记抖动族顺风向；裸跑/make/全域截断跑
> 三方 embedded-content 数字逐位一致）+ interactive pin **439P/44F/0T 逐位
> 持平**（make 目标重跑后）+ engine lib **3499P/0F** + clippy/fmt clean。
> 账本见
> [evidence/2026-10-25-m2s78-image-map-hit.md](html-semantics-compat/evidence/2026-10-25-m2s78-image-map-hit.md)。
> **下一步**：720-stretch 渲染展开 22F（需布局几何，归 layout 集成面）或
> testdriver-Actions 支持面（hash-name-dynamic 5F + 指针/键盘合成输入族）。
> **切片间隙**：本轮 part03/06 shim + 契约 part90；下一挂账 720-stretch 需
> layout 面协同，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-25 M2 切片 71 收口，F1b SVG 固有尺寸轮）**：
> s67 八族地图 F1b 收口（naturalWidth-naturalHeight-width-height 的 SVG
> data URL 族 77P/181F + svg-img-density-vs-zoom 1P/8F）。**根因**：runner
> 位图解码面不解 SVG——host `__zw_get_image_size` 返回 viewBox 原始宽高
> （缺省 100×100），非 CSS-images natural dimensions 组合表；resourceState
> 无固有轴元数据，密度校正无法逐轴表达语料 data-density 语义（attr 轴校正、
> 默认/AR 轴不校正）；display:none 未渲染 IDL 面（whatwg#11287）恒密度校正。
> **修复（R5277 五刀）**：①`__zwSvgPayload5277` data:SVG payload 提取
> （percent/base64/raw）；②`__zwSvgNaturalDims5277` 固有尺寸核（无引号 attr
> 按 HTML tokenizer 语义、length 非负 number|px、viewBox AR、w+h/w/h/none
> 组合表）；③sweep SVG 核前置 + 覆写面——携 axes（本核已写）幂等跳过、
> host 态（无 axes）原位覆写不重派 load、非 SVG 维持 host 幂等；④密度逐轴
> 折算（attr 轴按选中密度校正、默认/AR 轴恒原始；无 axes 旧态默认 both，
> raster 语义不变）；⑤未渲染 IDL 臂——display:none + loaded 回原始未校正
> intrinsic（内联样式直读近似，FIXME 样式表/继承面）+ avail 传播携 axes。
> **运行期 fast path 撤除留档**：fetch 臂 SVG 前置在 129 img 模板克隆风暴下
> 超 `_deferBudget=256` 静默丢回调 → inflight 泄漏 → load 链死锁（run5 全域
> 1001/1258 行截断实证），同步 dispatch 变体仍抖（77P/181F 不稳）——终版
> 撤除，运行期 SVG 语义归 host 解码器切片。契约 part89 两测试（27 断言，
> (h) 覆写面 load 不重派钉）。**验证**：域 **747P/479F/32T → 907P/320F/31T**
> （GAIN 160 = 160 F→P / 0 P→F；唯一 T 变化 video-loading-lazy-autoplay
> T→F 为 s70 已记环境抖动行翻回）+ naturalWidth 文件 77P/181F → **236P/22F**
> （3× 稳定）+ interactive **439P/44F/0T 逐位持平** + engine lib
> **3496P/0F** + clippy/fmt clean。账本见
> [evidence/2026-10-25-m2s71-svg-intrinsic-dims.md](html-semantics-compat/evidence/2026-10-25-m2s71-svg-intrinsic-dims.md)。
> **下一步**：F4 image-map 命中测试 84F（elementFromPoint 几何）优先，或
> 720-stretch 渲染展开 22F（需布局几何，归 layout 集成面）。
> **切片间隙**：本轮 part03/04 shim + 契约 part89；下一挂账 F4 亦 engine 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-25 M2 切片 70 收口，F2 source-set 族轮）**：
> s67 八族地图 F2 收口（parse-a-srcset 49 + select-an-image-source 13 +
> update-the-source-set 84 + relevant-mutations 46 + current-pixel-density
> 21 + 邻族 7）。**根因**：srcset/sizes/picture-source 选择面仅 R4162/R2419
> 「首 URL」近似——描述符语法/DPR 择面/media+type 门/relevant mutations
> 触发/密度校正全缺。**修复（R5277 九刀）**：①part03 共享选择核
> （parse-srcset/parse-sizes/create-source-set/update-source-set/
> select-image-source 五核，spec §4.8.4.3.7-12 逐字）；②候选 URL 就地绝对化
> （`_zwResolveFetchUrl` 文档基含 `<base href>`——available-images 键/请求
> URL/currentSrc 同源；raw/绝对键并存把同图拆两个 avail 表项，
> image-loading-lazy-available P→F 实证）；③全状态写入面切选中 URL（sweep
> srcset-only img / `.src` IDL 链 / revalidate）+ 密度入 resourceState +
> naturalWidth 密度折算；④relevant mutations 触发面（src/srcset/sizes 恒重
> 验、crossorigin/referrerpolicy content-state 枚举比对、SOURCE 变迁 → 其后
> img 兄弟、picture 树插入/移除）+ restart 收窄为选中 URL 变更（同 URL 命中
> 照常重派 load——spec maybe-omit-events 门）；⑤null selection（src/srcset
> 移除 → broken，error 仅在仍有关联时派）；⑥detached 更新延一步（重读
> loading——src-then-lazy-load 时序）+ lazy 视口门只拦 fetch 不拦 avail
> settle（avail 先于 lazy 决定）+ scrollIntoView 视口旗标近似（F5 几何收口
> 前债务标记）；⑦broken 无 width/height 属性 → IDL 0（s68 误正改 0——
> 300/150 是 UA icon 非 IDL 面）；⑧css-parser media_query.rs MQ4
> error-handling——非空 inner 无法解析 → 整条不匹配，不再退化空条件 all 恒
> 匹配（`all and (!)` 面 + matchMedia false 对齐，钉定测试撤换）；⑨extract.rs
> 候选收集接解析核。契约 part88 五测试。**验证**：域 **607P/678F/37T →
> 747P/479F/32T**（GAIN 133 = 133 F→P / 0 P→F；run3 捕获唯一 P→F 由②收口
> run4 复验翻回；run3→run4 差恰 2 行）+ interactive **439P/44F/0T 逐位持平**
> + engine lib **3484P/0F**（+5 契约）+ css-parser lib **3003P/0F** +
> clippy/fmt clean。账本见
> [evidence/2026-10-25-m2s70-source-set.md](html-semantics-compat/evidence/2026-10-25-m2s70-source-set.md)。
> **下一步**：F1b naturalWidth SVG 固有尺寸/密度校正尾片（210F 余量）或 F4
> image-map 命中测试 84F（elementFromPoint 几何）——按共享路径收益取一。
> **切片间隙**：本轮 part03/04/05 shim + extract.rs + css-parser media_query
> + 契约 part88（part85 键位随绝对化更新）；下一挂账 F1b/F4 亦 engine 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 67 收口，embedded-content 开片分诊轮）**：
> 两件事。**① 域分诊八族地图落档（819F 全归因）**：F1 img 静态加载管线
> ~260F（settle 只接 `.src` IDL setter 通道 R56h，parser 静态 img 无
> fetch/load 事件、naturalWidth 恒 0）+ F2 select-an-image-source/currentSrc
> /srcset/sizes ~300F（currentSrc 恒 ''，全族算法未实现，独立大切片）+
> F3 img.decode() 56F + F4 image-map 命中测试 84F（elementFromPoint 几何
> 底座→渲染流记账回流）+ F5 lazy-loading ~49F（视口几何缺失，F1 后重
> 分诊）+ F6 canvas 53F（getContext options 序 + Tab 序，逐案分诊）+
> F7 object/embed 窄面 25F + F8 杂项 2F。域刷新基线 **458P/819F/40T**
>（执行案 231 与 M1 一致；subtest 1060→1317 为 R5235 帧驱动后此前
> Timeout 悬挂案完成注册）。**② F7 null 反射收口（R5274 一刀）**：part04
> get-trap IFRAME 分支后补 OBJECT/EMBED 分支——contentDocument/
> contentWindow 恒 null（spec「does not represent a nested browsing
> context → null」尾步）+ getSVGDocument 返 null 函数（GetSVGDocument
> Support）；EMBED 无 contentDocument/contentWindow 成员保持 undefined。
> 契约 part84 两测试（钉①五臂 + 钉② IFRAME R130 面不回归）。**验证**：
> 域 **458P/819F/40T → 461P/816F/40T**（GAIN 3 / REG 0）+ interactive
> 439P/44F/0T 逐位持平 + lib **3469P/0F**（+2 契约）+ 翻转力实证（撤刀
> → 钉①红，恢复字节一致）。账本见
> [evidence/2026-10-24-m2s67-embedded-content-triage.md](html-semantics-compat/evidence/2026-10-24-m2s67-embedded-content-triage.md)。
> **下一步**：F1 img 静态加载管线开片（~260F 最大可收口单族；DCL 扫描
> settle + loading=lazy 跳过 + setAttribute('src') 通道，R5254 两段式
> 先例）。
> **切片间隙**：本轮 part04 get-trap + 契约 part84；下一挂账 F1 亦
> engine shim/host 桥面，间隙开放，开工前查 oracle master 碰撞记录
>（2026-10-24 已查：oracle master 记「engine 面占用标记已退场清除」，
> 无新占用）。
>
> **▶ 上轮状态（2026-10-24 M2 切片 66 收口，text-level-semantics 开片轮）**：
> 两件事。**① the-a-element 13F 两族全收（R5273 三刀）**：a.text
> getter/setter = textContent 精确委托（9F；option 的 strip-and-collapse
> 不适用）+ a/area toString 品牌检查（4F；WebIDL es-stringifier step 2，
> **探针实证 trap 臂才是 createElement 产物 toString 实际供给面**——原型
> defineProperty 不达，双侧同步防直装路径）。**② 共享路径根修（part05
> overlay 种子）**：a.text 委托暴露预存缺陷——textContent=（sel 形）写后
> 同 turn childNodes.length===0 而 firstChild 已见新值（探针钉定与 .text
> 无关）。根因 = 注册文本节点无 pending link，overlay added 并入跳过而
> removed 剔除生效，childNodes 与 firstChild 两视图分裂（R5269 probe62
> childLen=0 同根源）。修 = `_zwOverlayPendingChildNodes` 入口
> `_zwTextElsBySel` 种子（命中即以注册节点为基；appendChild 并入/
> removeChild 剔除/apply 后不双计均保持）。**验证**：域 **34P/0F**
>（exit 0 首次）+ links 119P/20F/4T（stash 二分与 HEAD 逐值一致，预存）
> + document-metadata 59P/59F/18T + tabular 130P/4F + interactive
> 439P/44F + engine lib **3463P/0F**。账本见
> [evidence/2026-10-24-m2s66-text-level-a-element.md](html-semantics-compat/evidence/2026-10-24-m2s66-text-level-a-element.md)。
> **下一步**：embedded-content / scripting-1 / microdata 按挂账序开片
>（分诊轮先行）。
> **切片间隙**：本轮 part03/part04/part05 shim + part05 overlay；下一挂
> 账 embedded-content 分诊未定，间隙开放，开工前查 oracle master 碰撞
> 记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 65 收口，http-equiv 分诊 + content-language pragma 轮）**：
> 两件事。**① http-equiv-and-name 2F 分诊（本域 shim 面最后未分诊项，
> 分诊轮全归档）**：-1 = CanvasText 系统色 × color-scheme applied-color
> 链 → 渲染流记账回流（与 s64 color-scheme 23F 同桶）；-2 = 本流修复。
> **② 修复面（R5272，dom crate 两刀）**：①`parse_selector_chain` 复合伪类
> 名字边界——名字取 `(`/`#`/`.`/`[`/`:` 最早命中、仅 `(` 按含参取 args，
> 旧 `r.find('(')` 扫过整段余串把 `:root:lang(de-DE)` 的 `root:lang` 当
> 名字 → SyntaxError；②`matches_lang` pragma-set default language 回落
>（spec「language of a node」Otherwise 分支）——祖先链全无 lang 时吃
> `<meta http-equiv=content-language>`（含逗号失效、首 ASCII 空白 token、
> 文档序最后胜出；空 lang = explicitly unknown 不回落；HTTP 头回落
> FIXME），R3281 共享权威面 style-system `:lang()` CSS 匹配同步吃到。
> 回归测试 zz_r5272×2。**验证**：域 **59P/59F/18T**（+1 NotRun 为 CSP
> tentative 行与本修复无关）+ interactive 439P/44F + style-system lib
> **2449P/0F**（共享权威零回归）+ zero-dom lib **868P/0F**（+2）。账本见
> [evidence/2026-10-24-m2s65-http-equiv-content-language.md](html-semantics-compat/evidence/2026-10-24-m2s65-http-equiv-content-language.md)。
> **下一步**：text-level-semantics 开片（M1 基线 21P/13F），分诊轮先行。
> 余账全归档：color-scheme 23F + http-equiv-and-name-1 1F + 管道/应用面
> ~27F 渲染流、CSP tentative + parse-world 底座。
> **切片间隙**：本轮 dom crate（query.rs/lang_dir.rs）；下一片
> text-level-semantics 分诊轮不改引擎生产面，间隙开放，开工前查 oracle
> master 碰撞记录（2026-10-24 已查：渲染流在 ::first-line/::selection/
> paint 面，无 dom/shim 占用）。
>
> **▶ 上轮状态（2026-10-24 M2 切片 64 收口，document-metadata 分诊 + link/style 窄面轮）**：
> 三件事。**① color-scheme 归属核对（定档渲染流）**：10F 死因 =
> corpus 打包缺口（缺 support/compute-root-color-scheme.js，上游有）
> ——按「打包缺口本地补」先例恢复（gitignored 本地数据，**步进
> 52P/54F/18T → 52P/66F/18T** +12F 归因入账）；断言面 23F 塌缩单根
>（inline CanvasText 系统色 × color-scheme 解析 + colorScheme IDL +
> root 应用色）= applied-color → 渲染流记账回流。**② link/style 事件
> 族分诊（35F+18T 四类）**：资源管道 ~13F + applied CSSOM ~13F + CSP
> tentative 底座 7F+5T + parse-world 底座 1F 挂账；shim 窄面 7F 本轮
> 全收。**③ 修复面（R5271）**：①link.fetchPriority 枚举反射（合法
> ascii 小写、missing/invalid → auto，2F）；②link.relList supported
> tokens（link types 表 20 关键字，supports() 1F）；③style.sheet
> [SameObject] WeakMap 缓存 + 直接子指纹失效（有效子变更换新对象 /
> 空 fragment 身份保持）+ connected 门（不在文档返 null）+
> StyleSheet 基接口 brand（instanceof 可达，LinkStyle 2F）。**验证**：
> 域 **58P/60F/18T** + tabular 130P/4F（终版二进制）+ interactive
> 439P/44F + lib **3458P/0F**（CSSOM 契约零回归）。账本见
> [evidence/2026-10-24-m2s64-metadata-triage-linkstyle.md](html-semantics-compat/evidence/2026-10-24-m2s64-metadata-triage-linkstyle.md)。
> **下一步**：http-equiv-and-name 2F 分诊（本域 shim 面最后未分诊项）；
> 域收口后 text-level-semantics 开片（M1 基线 21P/13F）。余账全归档：
> color-scheme 23F + 管道/应用面 ~27F 渲染流、CSP tentative + parse-world
> 底座。
> **切片间隙**：本轮 part04/part06 JS shim；下一挂账 http-equiv 分诊
> 未定，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 63 收口，document-metadata base 族轮）**：
> document-metadata 第二实施切片：**43P/63F/18T → 52P/54F/18T**
>（GAIN 9 / REG 0），the-base-element 13/13 全绿。**修复面
>（R5270 三刀）**：①`_zwDocBaseURL` scheme 排除——spec 文档基 URL
> 算法末步 data/javascript scheme **立即返 fallback**（非继续找下一
> base，base-data 案 1 期望 document.URL 实证）。②pending 增补 base
> 先扫——R161 归并把 pending 追加快照尾树序不保（head.prepend 新
> base 先于快照 base 评估，案 3 实证）；corpus 未钉 pending valid
> base 树序案，先扫近似记已知限制。③BASE href IDL（R5008 块内分
> 支）——缺失/空串 → 文档地址；解析失败 → **原属性串**
>（base_href_invalid corpus 钉定 "//test:test"，首版「失败 → 文档
> 地址」翻红回退修正）；LINK 保持 R5008 口径（harness resolveUrl
> 消费面）。**验证**：the-base-element 13/13 + tabular 130P/4F +
> interactive 439P/44F + lib **3456P/0F**（_zwDocBaseURL 共享面
> a/form action、baseURI、img src 零回归；+2 为 oracle 流新增）。
> 账本见
> [evidence/2026-10-24-m2s63-base-url-family.md](html-semantics-compat/evidence/2026-10-24-m2s63-base-url-family.md)。
> **下一步**：meta color-scheme 10F 归属核对（疑 style-system/渲染
> 面）；link/style 事件 30F+18T 分诊（runner file 通道能力上限）；
> style type/media + CSP tentative 待分诊。
> **切片间隙**：本轮 part03；下一挂账分诊未定，间隙开放，开工前查
> oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 62 开片，document-metadata title.text 族轮）**：
> 两件事。**① tabular-data 余 4F 分诊定档（不修）**：processing-model-1
> 四案全是表格布局 span 几何（colspan/col span=1000 字面列数摊宽、
> rowspan=0 不参与行高分布；offsetWidth/clientHeight 读），控制面排除
> 清单明确归 rendering-compat——本流收口定格 **130P/4F**，余案归因
> 挂账（DOM 语义面 IDL clamp 15 断言已全绿）。**② document-metadata
> 开片（M1 基线 31P/75F/18T → 43P/63F/18T，GAIN 12 / REG 0）**：
> Mission 序 tabular-data 后下一域，域内从未切片。**修复面（R5269，
> title.text 族 12F 全收）**：①title.text getter（part03）——child text
> content 直接 Text 子原样拼接（comment/元素子不含、无空白归一；
> title.text-01 断言 .text="TEXT"≠textContent）；sel 形 SetText 后融合
> 视图不载新文本——walk 空且 lw 非空才直取 latest-wins（appendChild
> 变更链 walk 非空自然排除，lw 不可直接作信号——title.text-01 首版
> 回退实证）。②title.text setter（part04）——R3907 option.text 同口径
> 委托 textContent。③document.title setter（part06）——改经元素
> textContent setter 落地（自带宿主写 + 本地文本子注册；旧裸
> __zw_set_text 只写宿主，本地文本子 stale 实证）。**验证**：
> the-title-element 13/13 + tabular 130P/4F + interactive 439P/44F +
> lib 3454P/0F（R3035 共享面零回归）。账本见
> [evidence/2026-10-24-m2s62-title-text-family.md](html-semantics-compat/evidence/2026-10-24-m2s62-title-text-family.md)。
> **下一步**：base 族 9F（HTMLBaseElement.href IDL + base 感知 URL 解析）；
> meta color-scheme 10F 归属核对；link/style 事件 30F+18T 分诊
> （runner file 通道能力上限）；style type/media + CSP tentative 待分诊。
> **切片间隙**：本轮 part03/04/06；下一挂账 base 族亦 shim/net 面，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 61 收口，insertRow tbody 目标 + XML 导入链轮）**：
> tabular-data 域第四实施切片：**127P/7F → 130P/4F**（GAIN 3 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3454P/0F**。**修复面（R5268）**：①table.insertRow append 位
> （part03）——删 `hasSection` 分支（旧：rows 空 + thead/tfoot 存在时新
> 行**直挂 table**），统一「`_zwTblChildIdxLast` 最后 tbody，无则新建
> tbody 挂 table」——spec append 位 = 最后 tbody 元素子，thead/tfoot
> 不是插入目标（table-insertRow thead/tfoot 案 2F 同修）；连带多 tbody
> append 位从「最后有行的 tbody」纠正为「最后 tbody 元素子」。②快照
> 元素级 getElementsByTagName（part02）——`_zwParseEl` 缺方法（仅 doc 级
> 有），委托 querySelectorAll 一行补齐（tBodies.html XML documentElement
> 查询面）。③importNode 快照域产物物化（part06）——`_zwParseEl`/
> `__zwIsMEl` 源 deep import 经 `_wcRebuildAsHandle` 物化为 handle 树
> （t8q adoptNode 同判据同重建器；raw _zwMEl 副本 append 进活树后
> `.tBodies` 读 undefined 实证）；浅克隆与真 wrapper 源不进分支零变化。
> **验证**：probe 6 案全绿（已删）+ table-insertRow 3/3 + tBodies 1/1 +
> 两域复跑零回归。账本见
> [evidence/2026-10-24-m2s61-insertrow-tbody-xml-import.md](html-semantics-compat/evidence/2026-10-24-m2s61-insertrow-tbody-xml-import.md)。
> **下一步**：tabular-data 余 4F 全在 processing-model-1——rowspan-0×2
> （clientHeight 渲染门）+ col-span-limits×2（span 处理模型），待分诊；
> cells/tBodies HTMLCollection 类型一致化（无断言，低优先）。
> **切片间隙**：本轮 part02/03/06；下一挂账分诊未定，间隙开放，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 60 收口，table.rows HTMLCollection 轮）**：
> tabular-data 域第三实施切片：**122P/12F → 127P/7F**（GAIN 5 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3454P/0F**（+5 为 oracle 流新增测试）。**修复面（R5267）**：
> ①`_zwMakeHTMLCollection` 加 **rebuild 钩子**——[SameObject] 集合的 live
> 读重建形态：每次属性读重算成员表（排序/过滤天然最新），不注册写侧
> live 维护（无 matches 不进 `_zwLiveCollections`；与 matches 形态互斥
> 守卫，既有调用方零变化）。②rows getter（part03）裸数组 → 集合形态：
> `_zwRowsCollCache` WeakMap 缓存（R3908 selectedOptions 同口径，键 =
> 元素 proxy；rebuild 闭包持 sel/handle 原始值防自持环）+ rebuild 每读
> 重算 `_zwTableRowsAll`/`_zwTblChildAll`（s59 地基复用）。table-rows
> 断言面全落既有完备工厂：instanceof（`_zwHCPrototype` 链）、named
> getter（id 全元素 + name 仅 HTML-ns）、ownKeys（[indices…, names…]
> 集合序）、gOPD、canonical 索引；live 断言（removeChild 后 namedItem
> →null）由重建天然过。**范围界定**：仅 rows 收进集合；cells/tBodies
> 保持数组（类型一致化留挂账）；`_rowList`/`_cellList` 消费面经 trap
> 语义兼容零改动。**验证**：table-rows 5/5 全绿 + 连带面逐文件复核零
> 回归（remove-row/insertCell/rowIndex/sectionRowIndex/cells/cellIndex/
> createTBody/table-insertRow 持平）。账本见
> [evidence/2026-10-24-m2s60-table-rows-htmlcollection.md](html-semantics-compat/evidence/2026-10-24-m2s60-table-rows-htmlcollection.md)。
> **下一步**：tabular-data 余 7F——table-insertRow 2F（无 tr 时插入位
> 语义：spec **末 tbody**，现实现首 tbody）+ tBodies 1F（DOMParser XML
> documentElement.getElementsByTagName 缺）可并片；处理模型 4F 待分诊。
> **切片间隙**：本轮 part03/05；下一挂账亦 shim 面——间隙开放，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 59 收口，tr 族索引/集合读侧轮）**：
> tabular-data 域第二实施切片：**94P/40F → 122P/12F**（GAIN 28 / REG 0），
> interactive-elements 全域 **439P/44F/0T** 复跑逐值不变，zero-engine
> lib **3449P/0F**。**根因（R5266）**：tr 族六 getter（rowIndex/
> cellIndex/sectionRowIndex/rows/tBodies/cells）旧实现经 selector 查询
> 三缺陷——detached script-created 子树不可解析恒空 / 匹配全部后代非
> 直接子 / 无 namespace 过滤。**修复面**：①part03 新地基四件——
> `_zwIsHtmlNsEl`（**严格** xhtml 串判定，探针实证 createElementNS("",
> x) 产物 namespaceURI 读 **null**、解析树/createElement 读 xhtml 串；
> 区别于 `_zwTblChildIdx` 宽松口径不动）+ `_zwTblChildAll` 直接子收集 +
> `_zwTableRowsAll`（spec rows 排序：thead 行 → 直接 tr + tbody 行 →
> tfoot 行）+ `_zwIdxOfSelf` 身份计位（childNodes 缓存 proxy 恒等）。
> ②六 getter 重写：rowIndex 门槛 **parent**（table）或 section+**grandparent**
> table（非最近祖先，table>div>tr 案 -1）；sectionRowIndex 父 table/section；
> cellIndex 父 HTML tr 直接 td/th 子；handle-only created 全覆盖。③has
> trap 名册（part05，R5249 tag-gate 模式）——`"cellIndex" in th` 等恒
> false 补六名。**连带收口**：remove-row 3F + thead/tbody/tfoot rows 3F
> + table-insertRow 1F（`_rowList`/`_cellList` 转读修好的 rows/cells）。
> **lib 测试契约修补**（part15 R3243 三测试）：rows/cells 读侧走
> childNodes 融合视图后，测试换代点只 bump 宿主代际未调 shim 侧挂钩的
> 缺口显形（跨场景 pending 桶残留，探针实证 len=5 含建行）——按
> part12/38/41 既有契约补 `__zw_reset_pending_state` + 两处同场景续跑
> 步先重播种 pristine 快照。**验证**：五目标文件 45 断言全绿；域
> 122P/12F/0T；interactive-elements 439P/44F/0T（REG 0）；lib 3449P/0F；
> fmt/clippy 干净。账本见
> [evidence/2026-10-24-m2s59-tr-index-family.md](html-semantics-compat/evidence/2026-10-24-m2s59-tr-index-family.md)。
> **下一步**（s60 已收口）：table-rows 5F 已清；余挂账 table-insertRow
> 2F / tBodies 1F / 处理模型 4F / cells-tBodies 类型一致化。
>
> **▶ 上轮状态（2026-10-24 M2 切片 58 收口，tabular-data create/delete + setter 族轮）**：
> tabular-data 域首个实施切片：**67P/67F → 94P/40F**（GAIN 27 / REG 0），
> interactive-elements 全域 98 files **439P/44F/0T** 复跑逐值不变。
> **修复面三件（R5265）**：①table create/delete 派生族（part03）——
> createTBody 插**最后一个 HTML-ns tbody 之后**（createTBody.html 15 案
> 钉定，嵌套不作锚）、createTHead 有守卫+before tbody/tfoot、createTFoot
> 有守卫+append、createCaption 有守卫+首子、delete 三族删首匹配直接子；
> 门统一 `_realTag` 后缀取 `.split(':').pop()`（createElementNS 前缀案
> 放行）。②caption/tHead/tFoot getter 收紧（`_zwTblChildIdx` 精确索引 +
> 非 HTML ns 排除）。③IDL setter 族（part05 set trap 新分支 + part04
> set trap 补 receiver）——null 删首匹配、非 section/caption 形态
> TypeError、localName 不符/祖先环 HierarchyRequestError、caption 插
> **首子**（caption_001 firstChild 钉定）、thead/tfoot 有既有先删后原位
> 插（防 selector 重解析误删同形新子）。**本轮 bug 两个同 commit 修**：
> caption 插入段困嵌套 if/else 末臂永不可达（探针入口插桩实锤
> setter 路径从未调 insertBefore；修=卫语句+平铺共享段）；createTBody
> 位置误诊为包装身份断裂（同型元素 identity 断言失败优先怀疑顺序——
> 12 探针逐因子排除后插桩抓 `at:0` 实锤）。**验证**：目标案全绿
>（createTBody 15/15、caption_001 5/5、tHead 3/3、tFoot 2/2）；全域
> interactive-elements 439P/44F/0T（REG 0）；zero-engine lib 3448P/0F。
> 账本见
> [evidence/2026-10-24-m2s58-tabular-create-delete-family.md](html-semantics-compat/evidence/2026-10-24-m2s58-tabular-create-delete-family.md)。
> **下一步**（s59 已收口前两见）：tr 族 21F 已清；余挂账 table-rows
> 5F / remove-row 3F + section rows 3F / insertRow / tBodies 1F / 处理
> 模型 4F。
>
> **▶ 上轮状态（2026-10-24 M2 切片 57 收口，selectAll inert 作用域 + 钉案轮）**：
> 45F 全量复核（逐案读测试源钉断言机制）+ D 族唯一非几何门案收口
>（R5264），全域 438P/45F/0T → **439P/44F/0T**（GAIN 1 / REG 0）。**方案
> 取舍**：s46 五桶分组沿用并逐案核对——`inert-node-is-unselectable`
> 断言链 `document.execCommand('SelectAll')` →
> `getSelection().toString()`，此前 execCommand selectAll 未接（permissive
> no-op → toString 恒 ''），且模态 inert 遮罩是**纯 DOM 树行走**（区别于
> D 族其余案的 gBCR/elementFromPoint 几何门）。**修复**（part06）：
> execCommand 新增 selectall 分支——`_zwDialogModalTopSel()` 命中 → 栈顶
> 模态 dialog 子树，否则 body；复用 `_selection.selectAllChildren`（R2804
> 面）；不派 beforeinput/input（选区命令不经编辑宿主事件序，与 undo/redo
> 同口径）；`_zwQueryCommandState` 补真实反射。**钉案成果**（跨流协调
> 交付物）：余 44F = 渲染流门 ~21F（abspos 居中 6 / default-color 2 /
> overlay 2 / top-layer-position 1 / non-modal 1 / multiple-centered 1 /
> scroll-height 1 / visibility-hidden UA 1 / bounds-clicking 2 /
> modal-selection 拖选几何 1 / dialog-inert 1 / ancestor-is-inert 1 /
> inert-svg-hittest 1 / showModal top-layer 绘制序 1 / closed-details-
> layout-apis 1）+ shadow 底座 20F（closedby-complex 16 + focus-previous-
> outside 3 + form-submit-shadow 1，declarative shadow 待用户点名）+
> corner-cases ESC 1F（s53 定性 divergence 不弯就）。**验证**：目标
> 1F→1P；全域 439P/44F/0T；zero-engine lib 3448P/0F；对照面
> not-highlighted（reftest 视觉）/ with-modal-dialogs-and-iframes /
> 两 crash 测试（语料不载）零波及。账本见
> [evidence/2026-10-24-m2s57-selectall-inert-scope.md](html-semantics-compat/evidence/2026-10-24-m2s57-selectall-inert-scope.md)。
> **下一步**：本流 shim 面清空——跨流协调（渲染流按名取案，飞书已通知）
> + declarative shadow 联合切片待用户点名。**切片间隙**：本轮零 .rs 变更
>（仅 JS shim），下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 56 收口，light dismiss 两腿轮）**：
> s49 定性「指针管线深水」的
> dialog-light-dismiss-pointer-capture.html（touch/mouse 双腿）**2F → 2P
> 全收口**（R5263），全域 436P/47F/0T → **438P/45F/0T**（GAIN 2 / REG 0
> / Timeout 0）。**方案取舍**：s49 三洞试错后回退定性指针管线深水；本轮
> 复核推翻前提——管线 capture 重定向面已全通（part04 setPointerCapture
> pending override + part06 `_zwProcessPendingCapture` up 派发前换防 +
> 尾簇 12 换防后取 capturedSel），真缺口唯一：light dismiss 决策仍是
> R4300 单腿（pointerdown 即关），capture 无从影响。spec 已演进两腿
>（down 记 nearest clicked dialog 祖先，up sameTarget 比对后关），实现
> 以**两态分类**等价替换身份模型：down `_zwDialogLdPick` 反文档序首个
> open+any 定类（'in' 链内 / 'out' 链外）存槽，up
> `__zw_dialog_light_dismiss_up` 重分类同态比对，相等且 'out' 才关
>（选择器保持 R5234 口径）；全腿组合等价（in/in、out/in、in/out 不关，
> out/out 关）——capture 面（down 'out' + capture 把 up 重定向 dialog 内
> 'in'）由此不关门。槽卫生：down 终态必写槽（早退也清）+ up 读取即清。
> popover pass 维持 down 腿单发（R5234 绿面零变化）。**验证**：目标
> 2F→2P（pointerId 全管线默认 1 与 `st.capture['1']` 对齐实证）；全域
> 438P/45F/0T；part60 R4 契约两腿化 + out→in capture 门腿（r4d）+ R6
> down+up 对（r4d 首轮 fallback `'button'` 命中 l1 首按钮误关，改
> `'#inner3'` 唯一 id——契约侧 bug）；zero-engine lib 3448P/0F；
> fmt/clippy 清。账本见
> [evidence/2026-10-24-m2s56-dialog-light-dismiss-two-leg.md](html-semantics-compat/evidence/2026-10-24-m2s56-dialog-light-dismiss-two-leg.md)。
> **下一步**：B 族清空，挂账序——C 族渲染流 20F 跨流协调（归渲染流）；
> declarative shadow 联合切片待用户点名。**切片间隙**：本切片为 engine
> shim 面（part01/part06），下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 55 收口，xhr-response graft 轮）**：
> s54 分诊轮挂账的 xhr-response 1F **窄域收口**（R5262），F 族
> name-attribute.html 三案全清（**18P/0F**），全域 435P/48F/0T →
> **436P/47F/0T**（GAIN 1 / REG 0）。**方案取舍**：s54 定性 parse-world
> 基面三件套升级（波及 DOMParser/XHR/responseXML 全语料）vs 底座挂账；
> 本轮复核出第三条窄域路线——场景需求全落 `xhr.response.body` 单节点，
> 而 `_zwParseEl` 的 R3019 mutTree 桥（`_ensureMutTree`）本就把该节点惰性
> 物化为 wrapper-world `_zwMEl` 实体子树，仅差物化时机（惰性→主动）、
> 接口原型接线（裸 `_zwMEl` 不接线，R125 在 `_zwMBuildNode` 只覆盖解析
> 子）、`doc.body` 指向（仍是快照壳）。**修复**（part03 XHR HTML 文档
> 分支）：body `_ensureMutTree()` 主动物化 + R125 配方接线
>（`__zwHtmlTagIface.body` → `HTMLBodyElement.prototype`）+ `doc.body`
> 换实体节点。读语义零变化（旧 firstChild 等读面本就惰性经同一 mtree）；
> DOMParser 语料零波及（graft 仅 XHR `responseType='document'` HTML MIME
> 分支，`_zwParsedDoc` 工厂与 `_zwParseEl` 基面未动）；head/documentElement
> 保持快照面（场景不触及）。wrapper-world 实体面经 s54 探针全链验证
>（innerHTML 写解析接 R4100、qSA 返真实 parentNode 链完整节点、open IDL
> + toggle 队列全通）。**验证**：name-attribute 17P/1F → **18P/0F**；
> xhr 语料 stash 前后双侧同跑 net-api xhr 切片 **580P/14F/21T 逐位一致**
>（基线二进制 R5262 锚点 0 / graft 后 1 双验）；escaping.html
>（html-syntax，XHR document 读面消费方）4 子测试全 Pass；zero-engine
> lib 3448P/0F；探针页已删。账本见
> [evidence/2026-10-24-m2s55-xhr-body-graft.md](html-semantics-compat/evidence/2026-10-24-m2s55-xhr-body-graft.md)。
> **下一步**：F 族清空，续挂账序——B 族 pointer-capture 2F 分诊（s49
> 定性指针管线深水）或 C 族渲染流 20F 跨流协调；declarative shadow
> 联合切片待用户点名。**切片间隙**：本切片为 engine shim 面（part03），
> 下一挂账开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 54 收口，name-attribute 3F 分诊轮）**：
> F 族 name-attribute.html 三案分诊完成——**2F shim 收口 + 1F 底座缺口
> 挂账**，全域 433P/50F/0T → **435P/48F/0T**（GAIN 2 / REG 0）。
> **R5260**（mutation event and toggle event order → Pass）：R4009 toggle
> 事件任务队列旧 in-place 去重使 e0 解析期合并事件留队首；spec「queue a
> details toggle event task」合并语义为**移除已排 task + 重排队尾**（
> oldState 取首排程 pre-state）。修复换新 entry 对象 + old 承接
> prevEntry.old——旧 timer 经 fire 闭包身份守卫（`!== entry`）自动作废，
> 兼容 part81 `__zw_setTimeout` stub 环境；spec 推演 FIFO [e3,e2,e1,e0]
> 对齐测试期望。**R5261**（connected-in-template → Pass）：plain wrapper
> 模板 content 视图（part03 `_zwContentView`，模板 childNodes 内联存储
> 形态）缺 ParentNode 元素族 getter，`content.firstElementChild` 恒
> undefined、setup 断言即败；part04 `_tplContent`（sel/handle 形态，
> R151）有全套故 `template-in-disconnected` 本来就过。补
> firstChild/lastChild/children/firstElementChild/lastElementChild/
> childElementCount 六 getter（R151 同款形状）；探针全链：inner
> instanceof HTMLDivElement、innerHTML 解析互斥 [1,0]、qSA 全通。
> **xhr-response 1F 挂账**：`xhr.response.body` 是 `_zwParseEl` 快照（
> protoChain `_zwParseEl > Object > null` 无接口原型），场景链需三件套
> 探针逐步实证——①instanceof HTMLBodyElement（R130 setPrototypeOf 会剪
> 掉 `_zwParseEl.prototype` 全部方法面）；②innerHTML 写解析（ctor own
> data property 赋值仅覆写字符串，R3019 mutTree 桥只读 rewire 无
> setter）；③open IDL（qSA 经 `_zwWrapQ` 返回 `_zwParseEl`，`e.open`
> undefined）。定性 parse-world 写面切片，波及 DOMParser/XHR/responseXML
> 全语料，单独立项排期。**验证**：name-attribute 单案 15P/3F →
> **17P/1F**；zero-engine lib 3448P/0F（part60 R4100 契约 per-element
> 形态不断言跨元素序，重排队零回归）；探针页已删；fmt/clippy 不适用（
> diff 仅 JS shim）。账本见
> [evidence/2026-10-24-m2s54-details-name-3f-triage.md](html-semantics-compat/evidence/2026-10-24-m2s54-details-name-3f-triage.md)。
> **下一步**：parse-world 写面三件套切片（接口原型双链保留 / innerHTML
> setter 进解析 / open IDL）排期，或续挂账序 B 族 pointer-capture 2F、
> C 族渲染流 20F；declarative shadow 联合切片待用户点名。**切片间隙**：
> 本切片为 engine shim 面（part03），下一挂账开工前查 oracle master
> 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 53 收口，零代码轮）**：corner-cases
> 「focusin calls showModal, modeless」ESC 1F 复审定性——**spec 一致行为，
> 测试期望分歧**。四重证据同向（ESC 后 dialog 重开 open=true）：live
> spec close-the-dialog 恢复门（anchor 后代 OR wasModal → focusing steps
> → focusin → 处理器 showModal；showModal 前置守卫无一命中、无重入门）；
> Chromium main 源码（Close() 焦点恢复在 is_closing_ 释放后、showModal
> 无守卫）；直探 Chromium 151 与 **Chrome for Testing 158.0.8118.0
>（wpt.fyi 同 build）**真 ESC——皆重开；我方 runner 探针序列与直探逐步
> 同构。**反证与矛盾**：wpt.fyi 同 revision（c645564b0e，2026-10-23
> run）chrome-158 + firefox-161 summary 双双 30/30 绿（压缩格式已验证
> c=[passes,total]）——与同 build 直探矛盾只能落 wptrunner 环境层（页面
> 可见语义内构造不出 closed 读点世界）；s41 依据「Chrome-parity 同红」
>（crbug 417571176，2026-04）已被双引擎绿取代（该 crbug 的 CL 即 wpt
> #58337 断言形式修复）。**收口**：1F 保留标 spec-consistent known
> divergence，不弯就（s41 R5244 试错实测 REG 4 + 违背 spec/直探）；
> 全域 433P/50F/0T 零变化。账本见
> [evidence/2026-10-24-m2s53-corner-cases-esc-divergence.md](html-semantics-compat/evidence/2026-10-24-m2s53-corner-cases-esc-divergence.md)。
> **下一步**：F 族 name-attribute 3F 分诊（mutation events / XHR 解析 /
> template 附着底座缺失——评 shim 可收口面 vs 底座缺口）；B 族
> pointer-capture 2F（s49 定性指针管线深水）、C 族渲染流 20F 维持跨流
> 挂账；declarative shadow 联合切片待用户点名。**切片间隙**：本轮零代码
> 未触 engine 生产面，间隙维持开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 52 收口）**：R5257 focus 滚动进视口
> + R5258 moveBefore——**interactive-elements 文件级 Timeout 清零**（50F
> 全为明确 Fail）。**分诊**（本轮刀前）：shim 面红名单三案——
> focus-after-close 文件级 Timeout（6/7 Pass 悬挂末腿）、
> movebefore-setup 1F（`new_parent.moveBefore` TypeError）、showModal 1F
> （rendering on top——top-layer 绘制序渲染流面，转 C 族记账）。**R5257**
>（focus-after-close Timeout → 7 子测试全 Pass）：探针双根因——proxy
> focus() 无滚动步（spec focusing steps 尾 UA 滚动面）+ 视口 scroll 事件
> 只派 window（document 监听永收不到）。三修：①focus 尾 nearest 滚动
>（gBCR 出界才滚，R3047/R3051 面复用）；②几何 fallback——动态创建元素
> 无布局几何（runner gBCR 零盒实证），读 proxy inline style（handle-only
> 可达）position 非 static + top 可解析 → 文档 y 顶对齐（FIXME: 样式表
> 定位随渲染流底座）；③no-scroll 变体 `__zwFocusNoScroll` 旗——R4600
> dialog 内部 focusing steps（show/showModal/close 恢复/fixup）包旗不滚
>（Chromium close 恢复不滚，同测试 scrollTop==0 断言面）；④`_zwFireScroll`
> 补 document 级派发（非冒泡 bubble 腿门控，window 不双计）。**R5258**
>（movebefore-setup 1F → 1P）：Element.moveBefore = WebIDL 校验 + 委派
> insertBefore 机械——免 removal steps 语义（shim 移动面本无 dialog 删除
> 钩子，组栈/open 态天然原地保持；FIXME: 跨文档同文档门随 parser 批次）；
> ESC 断言链走既有 R5236/R4300 组栈序。**契约**（part83）：nearest 滚动
> 位 + 双通道恰一次/showModal-close 零滚动 + 焦点恢复/moveBefore open 态
> 保持 + ESC 关组栈顶/NotFoundError 零滚动。**验证**：focus-after-close
> **Timeout → 7 全 Pass**、movebefore-setup **1F → 1P**；全域 98 files
> 483 subtest **433P/50F/0T 对 s51 逐位 REG 0 / GAIN 3**；zero-engine lib
> 3448P/0F（+契约 part83）；fmt/clippy clean（test-guard 包裹）；release
> runner 含 R5257×4/R5258×1 锚点。账本见
> [evidence/2026-10-24-m2s52-focus-scroll-movebefore.md](html-semantics-compat/evidence/2026-10-24-m2s52-focus-scroll-movebefore.md)。
> **下一步**：declarative shadow 联合切片仍待用户点名（渲染流协调）；
> 挂账序续 C 族渲染流 20F（top-layer 绘制序 showModal 1F / modal-selection
> 1F / inert 族 4F——elementFromPoint/hit-test 缺口）与 B 族指针管线深水、
> F 族 name-attribute 3F 逐案分诊。**切片间隙**：本切片为 engine shim 面
>（part01/04 + 契约 part83），下一挂账 C 族为渲染流面需跨流协调，开工前
> 查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 51 收口）**：R5256 dialog 属性移除面
> + A/H 分诊改记。**分诊**（本轮刀前）：A 族 closedby×slot 16F（
> dialog-popover-closedby-complex 全部「shadow DOM slots」变体）与 H 前置
> 4F **同根**——探针实证 `<template shadowrootmode=open>` 在页面视图被
> 拆包平铺（part03:7639「parser 占位」互证）、shadowRoot null、shim
> attachShadow 虚拟树不下传 dom（attach_shadow 生产零接线）、shadow 内容
> rect=0/hit-test null（几何面未通）；clickOn 坐标驱动下 shim 虚拟单边
> 提升会 REG 现存 4 Pass（点击落 (0,0) 全员误关）。**改记**：A+H 统一
> 「declarative shadow parser 提升 + shadow 几何渲染流联合切片」，渲染流
> 半边维持用户决策门禁（web-components 归档流明示等点名协调
> rendering-compat；与 parser form pointer 缓行决策同域）。**R5256 两刀**
>（dialog shim 面）：①open 属性移除面关闭步——`_zwDialogAttrClosedR5256`
>（part01）清模态/组栈 + close/toggle 排队，无 cancel/无 beforetoggle/
> 焦点不恢复/returnValue 不变（tentative 断言契约；旧注记「不派 close」
> 已被 2025 attribute change steps 变更推翻）；removeAttribute 尾钩 +
> IDL falsy 臂双接通，**close watcher 门**（pre-snapshot 模态态/组栈双
> 信号——属性直开无 watcher 仅隐藏，首版 REG dialog-open 一案当场修正）；
> ②detached 文档 dialog 三方法——showModal 抛 InvalidStateError、show
> 静默返、close no-op（dialog-active-document 旧产物 TypeError 面）。
> **契约**（part82）：模态移除/IDL falsy/wasOpen 门/同任务 toggle 合并/
> detached 三方法/属性直开零 close 六腿。**验证**：close-via-attribute
> **2F → 2P**、active-document **1F → 1P**、dialog-open 维持 3P；全域
> 98 files 483 subtest **431P/51F/1T 对 s50 逐位 REG 0 / GAIN 3**；
> zero-engine lib 3447P/0F（+契约 part82）；fmt/clippy clean（test-guard
> 包裹）；release runner 含 R5256 锚点。账本见
> [evidence/2026-10-24-m2s51-dialog-attr-removal.md](html-semantics-compat/evidence/2026-10-24-m2s51-dialog-attr-removal.md)。
> **下一步**：declarative shadow 联合切片待用户点名（渲染流协调）；续
> dialog shim 面红名单（focus-after-close 1T / showModal 1F /
> dialog-inert 1F 等逐案分诊）。C 族渲染流 20F 与指针管线维持跨子系统
> 单独排期。**切片间隙**：本切片为 engine shim 面（part01/03/04），
> 下一挂账预期仍 shim 面或待决策项，间隙开放，开工前查 oracle master
> 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 50 收口）**：toggleEvent 1T 收口
> （R5254 parser 初始 open details toggle 补偿两段式）。**分诊**：
> toggleEvent.html 文件级 Timeout——11 async_test 中 9 Pass、pending: 2
> 永挂（t9 依赖 inline ontoggle 在 parser 声明 open 时 resolve promise；
> 末案 DOMParser 产物 ontoggle）。根因：R4009 toggle 队列只挂写路径尾钩
>（IDL setter / setAttribute / removeAttribute），parser 初始 open 由
> host 解析直建不经 JS 写路径；innerHTML / createElement-HTML / 包装
> 世界三解析面已接 R4100 扫描，**唯主文档初始 HTML 与 DOMParser 快照面
> 缺**。**两段式**（单段直扫两难均实证：注册期直扫 task 派发早于页面
> 脚本——runner 面 arm 到真线程 timer，t9 resolver 未就位永挂；DCL 直
> 扫分不清 parser open 与运行时 open——activation-behavior coalesced 案
> toggle 计数 2≠1 回归）：注册期 mark（`__zwDetailsInitOpenMark5254`，
> 页面脚本前文档纯 parser 态，open 者身份 key 入集）+ DCL fire
>（`__zwDetailsInitOpenFire5254`，capture listener，只对「初始 open 且
> 当前仍 open」走既有 `_zwToggleQueueNotify4009` 入队，标记集一次性消
> 费）；`__zw_reset_form_state` 尾清集重标记（跨导航）。**刀二
> DOMParser**：快照架构死穴——扫描快照 ≠ 用户 querySelector 手持快照，
> 实例属性 handler 不可达（首版直调计数 0 实证）；改 parseFromString
> 返回前扫 `details[open]`（打 open IDL getter + setTimeout task）+
> `_zwParseEl.prototype` ontoggle accessor 按身份键（`_zwEvKey`，同源
> 重解析稳定）登记 `_zwParsedToggleH` 全局表，task 经身份键寻址派发。
> **契约**（part81）：(a) 主文档初始 open 恰好一次 toggle（closed→open
> + target.open 真值）+ 同态写不重派（t9 断言面）；(b) 初始 closed 不
> 派；(c) DOMParser 产物收到 toggle + target.open 真值；(c2) 无 open
> details 零 task。时序模拟：记录式 `__zw_setTimeout` stub + 手动
> `__zw_fire` + 手动派 DCL。**验证**：toggleEvent.html **Timeout → 11
> 子测试全 Pass**；activation-behavior coalesced 案两段式回绿；全域
> 98 files **483 subtest 428P/54F/1T 对 s49 逐位 REG 0 / GAIN 2**
>（fail 集逐位一致，子测试总数 482→483 为 Timeout 不再吞 2 案）；
> zero-engine lib 3446P/0F（+契约 part81）；fmt/clippy clean
>（test-guard 包裹）；release runner 含 R5254 锚点（grep 实证）。
> 账本见
> [evidence/2026-10-24-m2s50-parser-open-toggle.md](html-semantics-compat/evidence/2026-10-24-m2s50-parser-open-toggle.md)。
> **下一步**：按挂账序候选 A 族 closedby×slot 16F（Chrome-parity 同红
> 钉住，slot 分配 shim 面）或 H 前置 4F（declarative shadow parser +
> 渲染流联合切片）；F 族 name-attribute 3F 与 DOMParser 快照架构
>（live 文档面）维持挂账。**切片间隙**：本切片为 engine shim 面
>（part01/02/03 + callbacks.rs 注册尾），下一挂账 A/H 均预期跨
> parser/shim/渲染流，间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 49 收口）**：B 族 light dismiss 5F 全挂账
> 零净变更 + F 族分诊 + R5253 收口 details auto-expand 1F。**B 族定性**：
> bounds-clicking 2F——探针实证 elementFromPoint 对 dialog 自身盒命中返
> null（modal+modeless 同断、后代正常），渲染流命中缺口并 C 族记账；
> pointer-capture 2F——R5252 两腿模型（down/up 双 target 判定 + capture
> retarget 救回）试错三洞（backdrop retarget 使 down 腿 contains=true 短
> 路漏记、capture 残留未隐式释放误救回、corner-cases modal 组入口永挂），
> git 回退定性**指针管线深水**，需先有指针序列契约面再动；corner-cases
> 1F 既有挂账（modeless ESC 竞态）。**F 族分诊**：closed-details-layout-
> apis 1F 渲染流面；name-attribute 3F（mutation events / XHR 解析 /
> template 附着底座未实现）；toggleEvent 1T 文件级 Timeout（9 子测试
> Pass 后 completion 未触发，未深查待分诊）。**R5253 两刀**（part02
> `_scrollToAnchorForHash`）：几何量测前插 spec 展开步（target 链逐个
> setAttribute('open','')，toggle/name 组链自动）+ 仅真展开时挂 rAF 补偿
> 臂（属性 mutation 批应用后重入：展开步 no-op + 既有滚动按新几何落位，
> 既有滚动面零变化）。**契约环境差异注记**：bridge 契约 sandbox 无帧驱
> 动/布局供给面，滚动落位不可建模（帧驱动 + 手动 tick 仍红）——契约钉
> (a) 展开/(b) 链外不误开/(c) no-crash，落位面由 WPT 单案承载。**验证**：
> auto-expand 单案 **Fail → Pass**；interactive-elements 全域 98 files
> 482 subtest **426P/54F/2T 对 s48 逐位 REG 0 / GAIN 1**；zero-engine
> lib 3445P/0F（+契约 part80）；fmt/clippy clean（test-guard 包裹）；
> release runner 含 R5253 JS 串锚点。账本见
> [evidence/2026-10-24-m2s49-fragment-details-expand.md](html-semantics-compat/evidence/2026-10-24-m2s49-fragment-details-expand.md)。
> **下一步**：按挂账序候选 A 族 closedby×slot 16F（Chrome-parity 同红
> 钉住）或 H 前置 4F（declarative shadow parser+渲染流联合切片）；
> toggleEvent 1T 分诊可作小切片插入。C 族渲染流 20F 与指针管线两腿模型
> 需跨子系统协调，单独排期。**切片间隙**：本切片为 engine shim 面
>（part02 fragment 导航），下一挂账 A/H 均预期跨 parser/shim/渲染流，
> 间隙开放，开工前查 oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-24 M2 切片 48 收口）**：D 族 focus/inert 10F
> 逐案单跑取证后重分诊——可收口面仅 focusability 1F（R5251）。**重
> 分诊**：focus-previous-outside 3F + form-submit-dialog-shadow 1F →
> **挂账（H 族前置）**——首层根因是 `<template shadowrootmode=open>`
> 声明式 shadow root 从未被提升（`host.shadowRoot` 返 null，
> `querySelector` 即炸），shim 全 part 无 shadowrootmode 处理
>（attachShadow 命令式已有），4F 记入 H 族「parser+渲染流联合切片」
> 范围，R4600 焦点恢复 shadow/slot 边界是其后续面；dialog-inert 1F +
> modal-dialog-ancestor-is-inert 1F + inert-svg-hittest 1F +
> inert-node-is-unselectable 1F + modal-dialog-selection 1F → **挂账
>（渲染流面）**——elementFromPoint 对 open dialog 返 null / 点击穿透
> inert 祖先 / 选择窗口空串，同底座 = 命中测试几何链缺失，与 C 族
> top-layer/UA 布局 15F 合并记账。**根因**（focusability）：Tab 可聚焦
> 判定两处均无 editing host 分支——runner `dispatch_tab_focus` 页面候选
> 序选择器不含 contenteditable 宿主（本案观测路径，testdriver
> send_keys TAB）；host `FocusManager::scan_node` 同缺口（MoveFocus 效果
> 面）。spec：editing host 是 focusable area（HTML §focusable-area）。**落
> 地（R5251 两刀）**：runner dispatch_tab_focus 候选选择器尾补
> `[contenteditable]` + host 门（`isContentEditable`（R3187 枚举态）真
> 且父不可编辑 → 入零序；嵌套 editable 非 host / `contenteditable=false`
> 孤岛不入序；显式 tabindex 优先原臂，原生可聚焦元素落原 else 臂——首版
> 误让 button 落 CE 臂 continue 掉，回归当场修正）；dom FocusManager 补
> `contenteditable_state`/`is_editable`/`is_editing_host` 三 helper +
> scan_node 零序分支。**验证**：dialog-focusability 单案目标子测试
> **Fail → Pass**；interactive-elements 全域 98 files 482 subtest
> **425P/55F/2T 对 s47 逐位 REG 0 / GAIN 1**；zero-dom lib 866P/0F
>（+3 新单测，fail-first 先行）；zero-engine lib 3444P/0F（+契约
> part02 R5251）；fmt/clippy clean（test-guard 包裹）；release runner
> 含 R5251 JS 串锚点。账本见
> [evidence/2026-10-24-m2s48-editing-host-focusable.md](html-semantics-compat/evidence/2026-10-24-m2s48-editing-host-focusable.md)。
> **下一步**：按挂账序转 B 族 light dismiss 5F 分诊（invoker/backdrop
> 面）；H 前置 4F 与渲染流面 5F（并入 C 族记账）维持挂账；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）。
> **切片间隙**：本切片为 runner 页面序脚本面（testharness.rs）+ dom
> FocusManager 面（crates/dom）——下一挂账 B 族 light dismiss 预期
> engine shim 面（part01 dialog 事件），间隙开放，开工前查 oracle
> master 碰撞记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 46 收口）**：余 60F 归因分诊落档（六族
> 地图——A closedby×slot 交叉 16F（crbug 417571176 Chrome-parity 同红）、
> B light dismiss 5F、C top-layer/UA 布局 15F（渲染流面）、D focus/inert
> 10F、E dialog API 语义 5F（可 shim 收口候选）、F details 零散 4F/1T +
> G 历史 menu 面 3F）+ 族 G 顺手收口（R5249）。**根因**（三半边「IDL 名
> 无 tag 门」同模式）：①part05 has trap track IDL 列 label/kind/srclang/
> default（media 切片 3 加）无条件命中——历史 menu/menuitem（menuitem 已
> 删为 HTMLUnknownElement）`'label' in el` 假阳性；②set trap R3069 臂
> FLAT 表 type 无 tag 排除——`menu.type=` 误写内容属性；③同臂 label 显式
> 列（R3069-fix）无 tag 门。**落地（R5249 双刀）**：has trap kind/srclang
> 收 TRACK 门、label/default 收 TRACK+OPTION 门（HTMLOptionElement 读侧
> R3069-fix 对称）；R3069 臂条件首项 MENU/MENUITEM 负门——赋值落 expando、
> 内容属性不变，name 臂自带门与 FLAT 其余名零变化。**验证**：
> contextmenu-historical 单案 **11/11 Pass**；interactive-elements 全域
> 98 files 482 subtest **423P/57F/2T 对 s45 逐位 REG 0 / GAIN 3**（GAIN
> 即三案）；lib **3442P/0F**（+2 契约 part78）；契约翻转力实证（tag 读恒
> TRACK → 钉 1(a) 红；负门短路 → 钉 1(c) 红）。**已知边界**：menu.type/
> label 读侧 FLAT 读仍返内容属性值（真浏览器 undefined，测试无此面）；
> track 接口其余成员 in 仍无条件（corpus 无 menu 面观测）。gates：clippy
> --workspace -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s46-historical-menu-idl-gate.md](html-semantics-compat/evidence/2026-10-23-m2s46-historical-menu-idl-gate.md)。
> **下一步**：按挂账序 E 族 dialog API 语义 5F（close-via-attribute 移除
> open 属性触发 closing/close 事件 2F 为首靶，active-document/showModal
> throw/movebefore 跟进）；parser form element pointer 4F **已缓行**
>（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆 parser 批次、
> M2 收口后尾段单独立项）；declarative shadow DOM 同打捆待触发。**切片
> 间隙**：本切片为 shim 面，下一挂账 E 族亦 shim 面——间隙开放，开工前查
> oracle master 碰撞记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 45 收口）**：s37 已知边界「点击
> summary-less details 内非激活后代不翻转」收口（2026-10-23 重新定性：
> spec 无 summary 翻转是 UA affordance 非 MUST，corpus 无观测面——性质
> 同 s42/s43：spec 正确性收口 + 契约钉住）。**根因**（通道分裂）：details
> 激活面两条通道各断半边——①`.click()` 合成通道 part04 click 默认动作段
> 只处理 receiver 自身三站（LABEL/SUMMARY/DETAILS），非激活后代无 nearest
> 上溯；②JS dispatch 通道 part03 R112 post act:details 门
> `_zwUaDispatch === true` 挡 `new MouseEvent`（无 UA 印记、无 synthetic
> 标）。**落地（R5248 双刀）**：part04 nearest 上溯分支——receiver 非
> 三站且 `_zwFindClickActivation` 命中 act:details 时
> `_zwDetailsActivateR4400` 翻转（spec the-details-element activation
> behaviour + legacy-pre-activation nearest ancestor (or self) with
> activation behavior；notPrevented 门内；receiver 三站让位防双翻；交互
> 后代 act:button/act:a/act:summary/checkbox 命中时 type 豁免自然不落）；
> part03 门放宽 `(_zwUaDispatch === true || _zwSyntheticClick !== true)`
> ——JS dispatch 进同款翻转，`.click()` 合成标保持不进（单翻互斥），
> preventDefault/connected 门由消费块入口守卫继承；hasSum 门槛让位不变
> （summary 站优先，R4009 面不回归）。**验证**：探针七案修后全翻且单翻
> 互斥成立；interactive-elements 全域 98 files 482 subtest **420P/60F/2T
> 对 s44 逐位 REG 0 / GAIN 0**（R4400 既有面无回归，corpus 无观测面无
> GAIN）；lib **3440P/0F**（+2 契约 part77）；契约翻转力实证（nearest
> 分支短路 → 钉 1(a) 红；门收紧 → 钉 1(c) 红）。**已知边界**：corpus 无
> 观测面（无 GAIN）；detached details 后代点击不翻转（R112 connected
> 门，spec 无条件但 corpus 无观测面）维持现状。gates：clippy --workspace
> -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md](html-semantics-compat/evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md)。
> **下一步**：按挂账序其余余案归因；parser form element pointer 4F
> **已缓行**（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆
> parser 批次、M2 收口后尾段单独立项）；declarative shadow DOM 同打捆
> 待触发。**切片间隙**：本切片为 shim 面，下一挂账开工前查 oracle
> master 碰撞记录（间隙开放）。
>
> **▶ 上轮状态（2026-10-23 M2 切片 44 收口）**：s39 剩余 1F「focus-after-close
> Complex dialog usage focus 恢复红（activeElement 得 div）」收口。**根因**
> （六层钻透）：Complex 案跨 testdriver 多命令 execute_script，宿主 apply
> 换代发生在 show 与 close 之间——①静态树节点 handle 形 proxy 无
> `_zwNodeParent` 反链记账（R91 只记 JS 动态操作）→ ②isConnected 反链
> miss + rect 探测兜底空 → 双假阴性 disconnected → ③R114 detached 臂误入
> → ④activeElement getter 的 R4600 focus-chain retarget 上行解析出
> **runner 投影容器形**——BND dump 实证 `_shadowHandles` 跨执行残留注册
> （`__n1/__n3/__n5/__n8`，meta hostSel:null、hostHandle 指向换代后复用的
> 活 handle）→ 误穿边界回吐投影 host DIV；⑤host tagName 防线被复用活
> handle 绕过、⑥`hostSel==null` 判据亦不成立（页面自建 shadow 的动态 host
> meta.hostSel 同为 null）——均不可判据。**落地（R5247，判据回归 spec）**：
> part04 getter + part01 `_zwIsConnected` 前置复核——`__zw_selector_for_handle`
> 宿主持久反查（不随换代清）+ querySelector 同一性核验（命中同 handle 且
> `__zw_contains` 在档才短路 true；`_zwRemovedHandles` 守门保持 remove 语义；
> shadow 树内锚宿主扁平 selector 命中别元素不短路——R91 反链 shadow 穿越
> 权威判定保持；part04 复核须前置——反链循环含中途 return 后置不可达）；
> part06 activeElement getter retarget 语义前提门——锚经同一性核验确认
> light 树直返锚（spec 焦点链 retargeting 仅当锚身处 shadow 树才穿边界；
> 判据与 stale 注册形态解耦），host 解析空壳兜底回落原锚。**验证**：
> focus-after-close 单案 **6/6 Pass**（Complex + shadow 双案同时翻绿）；
> interactive-elements 全域 98 files 482 subtest **420P/60F/2T 对 s43 逐位
> REG 0 / GAIN 1**（Fail 清单 diff 仅 Complex 案一行）；lib **3437P/0F**
>（+2 契约 part76）；契约翻转力实证（撤前置复核 → 钉 1(a) 红；light 门
> 无条件短路 → 钉 2(b) 红）。**已知边界**：focus-after-close 文件级
> Timeout（dont_scroll：focus 不滚 viewport → scroll 永不发 → promise
> 永挂）维持 s39 既有独立挂账；dialog-focus-previous-outside 3F 既有挂账。
> gates：clippy --workspace -D warnings（test-guard 包裹）/fmt clean、零
> 新增依赖。账本见
> [evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md](html-semantics-compat/evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md)。
> **下一步**：按挂账序 summary-less details 内非激活后代翻转（s37 邻近
> 缺口，真浏览器会翻转；2026-10-23 重新定性：spec 无 summary 翻转是 UA
> affordance 非 MUST，corpus 无观测面，优先级维持挂账序）；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）；declarative
> shadow DOM 同打捆待触发。**切片间隙**：本切片为 shim 面，下一挂账
> summary-less 翻转亦 shim 面——间隙开放，开工前查 oracle master 碰撞
> 记录。
>
> **▶ 上轮状态（2026-10-23 M2 切片 43 收口）**：挂账「replaceWith
> self-removal 父桶记账」收口（s26 既有挂账，R371 模板残留）。**根因**：
> replaceWith sel 臂 remove 步只覆盖 apply 面（Remove/RemoveHandle
> mutation）与 sel/handle 字符串标记表（parentNode/isConnected 读），合并
> 视图子读（childNodes overlay / R322 归并查询）按**父桶 removed identity**
> 剔除——桶缺 removed 项 → 被替换者（快照 sel 节点）在合并视图残留至 apply
> 换代，视图 ≠ spec 终态。**落地（R5246 一刀）**：remove 步后经 canonical
> 记账入口 `_zwHCLiveInvalidate([], [被替换者 proxy], 父sel, null)` 直调
> （父桶 removed + 全局 pending-removed + live 集合失效一体；子树展平含
> template 内容子）；proxy identity 取基列表条目（`_r371Self`，与 overlay
> 剔除同一缓存对象）；记账先于 R371 rekey → dst 桶必存在，rekey 走合并分支
>（promote 分支保留护栏）。**验证**：契约
> test_dialog_replacewith_parent_bucket_accounting_r5246（part75 新文件，
> R1 corpus 同形全树视图退出 + R2 body 直子面）+ 既有
> r321_replacewith_fragment_flatten 断言更新（kids 前导宿主残留 DIV 消失，
> 视图即 spec 终态）；interactive-elements 全域 98 files 482 subtest
> **419P/61F/2T 对 s42 逐位 REG 0 / GAIN 0**（挂账本就「对扫描无害」，
> corpus 无观测面——切片性质同 s42：spec 正确性收口 + 契约钉住）；lib
> **3434P/0F**。**已知边界**：sel 臂 replaceWith 的 MO record 面（插入/
> 移除两段，R117 handle 臂已有）独立挂账不扩面。gates：clippy
> --workspace -D warnings（test-guard 包裹）/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s43-replacewith-parent-bucket.md](html-semantics-compat/evidence/2026-10-23-m2s43-replacewith-parent-bucket.md)。
> **下一步**：按挂账序 summary-less details 内非激活后代翻转（s37 邻近
> 缺口，真浏览器会翻转）、focus-after-close Complex dialog usage focus
> 恢复深层（s39 剩余 1F）；parser form element pointer 4F **已缓行**
>（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆 parser
> 批次、M2 收口后尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **切片间隙宣告义务（2026-10-23 用户决策，配合
> [oracle 流解封协议](archive/incremental-rendering-oracle/master.md)）**：每个
> shim 面切片 push 后，若下一挂账不触 engine 生产面（余下挂账均为 shim
> 面），在「当前状态」宣告切片间隙开放；开工下一切片前先查 oracle
> master 碰撞记录有无占用标记，有则等待或改排其他 shim 挂账。
>
> **▶ 上轮状态（2026-10-23 M2 切片 41 收口）**：挂账「ESC 组栈边缘乱序」
> 定性收口（非代码轮）。**探针钉定**（zz-probe-s41，验证后已删）：
> corner-cases「focusin calls showModal, modeless」ESC 案事件序 =
> show() 聚焦空 modeless dialog（subject fallback）→ close-the-dialog
> 焦点恢复尾步 anchor 门命中（焦点锚=subject 后代）→ prevFocused
> focusing steps 触 focusin → 处理器 re-entrant showModal 重开。**spec
> 对账**：恢复门（锚 shadow-including 后代 OR wasModal）、show()/
> showModal() 双记录 prevFocused、focusing steps「not focusable →
> nothing」与现行文本逐字一致——R4600/R5236 无漂移。**试错回退**
> （R5244 subject 非 focusable 门，未合入）：目标案 GAIN 1 但 REG 4
> （dialog-focus-shadow「show:」三臂 + focus-previous-outside——Chrome
> 断言面「show() 无 focus delegate 聚焦 dialog 自身」，whatwg/html#8199
> 只移除 TAB 序可聚焦），两面矛盾 Chrome 选边聚焦面。**Chrome-parity
> 同红证据**：crbug 417571176（父 376516550 closedby 特性）2026-04
> 仍 "still failing according to wpt.fyi"，上游已改一次 assertion 仍未
> 过；本项目按 Chrome 拉齐，修复后随 corpus 刷新自然转绿。**收口**：
> 挂账关闭（语料钉住 + crbug 引用入账）；interactive-elements 全域
> 98 files **419P/61F/2T 对 s40 逐位 REG 0**、lib 3432P/0F、零代码净
> 变化。账本见
> [evidence/2026-10-23-m2s41-esc-stacked-parity.md](html-semantics-compat/evidence/2026-10-23-m2s41-esc-stacked-parity.md)。
> **下一步**：按挂账序非模态 close 不出组栈、R371 模板残留、
> summary-less details 内非激活后代翻转（s37）、focus-after-close
> Complex dialog usage focus 恢复深层（s39 剩余 1F）；parser form
> element pointer 4F **已缓行**（2026-10-23 用户决策：与 declarative
> shadow DOM（H）打捆 parser 批次、M2 收口后尾段单独立项）；
> declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 40 收口）**：挂账 custom command
> （`--` 前缀）CommandEvent 派发收口（s24 defer）。**spec 钉定**：现行
> button activation behavior——validity Custom 态对任意元素 true →
> fire command 事件（command=值本身大小写保留、source=invoker、
> cancelable=true）→ Custom 态 return 无内建行为；派发形状
> bubbles=false/composed=true/isTrusted=true/无 relatedTarget（上游
> button-event-dispatch 语料钉死）。**落地（R5243 两刀）**：①part05
> `_defineEventSubclass('CommandEvent','Event',[command,source])`
> （ToggleEvent 同厂）；②part01 `_zwInvokerCommandActivate` custom
> 分支构造 CommandEvent + trusted 印记（R312 通道）经
> `_dispatchWithBubble` 在 target 派发后 return——分派在既有调用方
> 门内（disabled/preventDefault 零新写），`cmdCustom` 原值与 `cmd`
> 小写拆分、内建关键字面不变。**集群**：interactive-elements 全域
> 98 files 482 subtest **419P/61F/2T 与 s39 逐位打平**；dialog 集群
> 82 files 对 s35 账本 **REG 0 / GAIN 5**（全为 s39 既有收益）；
> forms/the-button-element 18 files 54 subtest 对 s24 逐位一致
> **REG 0**（corpus 零 custom command 消费）。**已知边界**：现行
> spec「invalid → Unknown 不派事件」未跟进（维持 R4400 auto 映射，
> corpus 钉住，跟进涉 dialog/details 内建关键字重构独立切片）；
> popover 关键字 + oncommand IDL + button command/commandFor IDL
> 反射维持挂账。gates：lib **3432P/0F**（+1 契约 part73
> test_invoker_custom_command_event_r5243）、clippy/fmt clean、零
> 新增依赖。账本见
> [evidence/2026-10-23-m2s40-custom-command-event.md](html-semantics-compat/evidence/2026-10-23-m2s40-custom-command-event.md)。
> **下一步**：按挂账序 ESC 组栈边缘乱序、非模态 close 不出组栈、
> R371 模板残留、summary-less details 内非激活后代翻转（s37）、
> focus-after-close Complex dialog usage focus 恢复深层（s39 剩余
> 1F）；parser form element pointer 4F **已缓行**（2026-10-23 用户
> 决策：与 declarative shadow DOM（H）打捆 parser 批次、M2 收口后
> 尾段单独立项）；declarative shadow DOM 同打捆待触发。
>
> **▶ 上轮状态（2026-10-23 M2 切片 39 收口）**：挂账 focus-in-shadow
> 跨流收口（focus-after-close「moved to shadow root」案 +
> dialog-focus-shadow-double-nested）。**探针钉定**（zz-probe 五迭代，
> 验证后已删）四层根因：①sel 模板 content 子列表 per-read 重建，跨读
> mutation 落空（spec content getter 恒返同一 fragment）；②content
> 条目 parentNode 指模板 wrapper，replaceWith 落错列表（探针 D
> `pidIsView=false`）；③`_zwMEl.replaceWith` 只有 remove 半边，替换者
> 落空；④focus delegate 非 dialog 分支缺 get-the-focusable-area 递归，
> 嵌套 shadow host 被跳过后焦点落树序后一 focusable（探针 E
> `osraeTag=BUTTON`≠autofocus 锚）。**落地（R5241 四刀 + R5242 一刀 +
> 收尾失效）**：①part04 content 子列表首读物化缓存（缓存数组即活
> 树）；②parentNode/parentElement getter 槽位消费补 parentHandle 分支
> （focus chain 上行达 shadow 边界）；③`_tcParent = _tplContent`（条
> 目父 = 视图本体）；④part03 replaceWith 补插入半边（plain 父门控，
> host proxy 面零变化）；⑤part01 focus delegate 递归（dialog 分支
> seq-focusable 优先序不动）；⑥innerHTML= setter sel 模板缓存失效
> （R5196 lib 红面收口，视图身份不动）。**集群**：
> double-nested **2F → 2P 全绿**；focus-after-close **2F+1T → 1F+1T**；
> interactive-elements 全域 98 files 419P/61F/2T 对 s38 账本
> **REG 0 / GAIN 3**；forms 域 401 files 3902P/707F/20T/1U
> **REG 0 / GAIN 3**（GAIN 为 label 文件尾 3 案 s38 跑时截断未注册，
> 共同 48 案逐位一致）。**已知边界**：focus-after-close 余 1F
> （Complex dialog usage focus 恢复，既有）；content 缓存不回灌非
> innerHTML= 的宿主侧子树变化。gates：lib **3431P/0F**、clippy/fmt
> clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s39-focus-in-shadow.md](html-semantics-compat/evidence/2026-10-23-m2s39-focus-in-shadow.md)。
> **下一步**：按挂账序 custom command `--` 前缀 CommandEvent defer、
> ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留、summary-less
> details 内非激活后代翻转（s37）、focus-after-close Complex dialog
> usage focus 恢复深层（s39 剩余 1F）；parser form element pointer 4F
> **已缓行**（2026-10-23 用户决策：与 declarative shadow DOM（H）打捆
> parser 批次、M2 收口后尾段单独立项，触发条件见
> [决策记录](html-semantics-compat/evidence/2026-10-23-user-decision-parser-form-pointer-defer.md)）。
>
> **▶ 上轮状态（2026-10-23 M2 切片 38 收口）**：挂账 appendChild move
> 后兄弟读 stale 收口。**探针钉定**（zz-probe 两迭代，验证后已删）：
> R334 appendChild/insertBefore wire 异步 drain，兄弟读无 pending 补偿
> （parentNode 有槽位补偿、兄弟没有）；两遍调用序列进一步钉出**共享视
> 图插位缺陷**——R182 重定位 pos 在旧位剔除前计算，剔除后未同步，
> cleanup `insertBefore(input, dialog)` 形态视图成 [dialog, input,
> script]（真树 [input, dialog, script]）；shadow 容器案露第三层——
> 上一子测试 stale parentSel 槽未覆写。**落地（R5240 三刀）**：①part04
> 兄弟 trap pending gate（sel 槽→融合视图定位 / 容器 ariaSync→
> registry 定位）；②part05 overlay 旧位剔除 pos 同步（**共享路径**，
> childNodes/children 全消费面受益）；③容器挂载路径 dual-form 槽覆写
> （R97 noWire 同款形状）。**集群**：focus-after-close **3F+1T →
> 2F+1T**（moved-in-between 案全绿）；interactive-elements 全域 98
> files 416P/64F/2T 对 s37 账本 **REG 0 / GAIN 1**；forms 域 4627
> subtest 逐位一致（REG 0）。**已知边界**：moved-to-shadow-root 案兄
> 弟断言过、红点推进到 `shadowRoot.activeElement`（focus-in-shadow 跨
> 流缺口，与 dialog-focus-previous-outside 同族，新挂账）。gates：lib
> **3430P/0F**（+1 为渲染流新增）、clippy/fmt clean、零新增依赖。账本
> 见 [evidence/2026-10-23-m2s38-pending-move-sibling.md](html-semantics-compat/evidence/2026-10-23-m2s38-pending-move-sibling.md)。
> **下一步**：按挂账序 parser form element pointer 4F（用户决策门禁，
> 单独立项征询）、custom command `--` 前缀 CommandEvent defer、ESC 组
> 栈边缘乱序、非模态 close 不出组栈、R371 模板残留、summary-less
> details 内非激活后代翻转（s37）、focus-in-shadow
> shadowRoot.activeElement + focusing steps 跨 shadow 恢复（s38 新，与
> dialog-focus 族跨流缺口同族）；declarative shadow DOM（H 定性后）待
> parser/渲染流联合切片。
>
> **▶ 上上轮状态（2026-10-23 M2 切片 37 收口）**：挂账 anchor-with-inline
> 导航缺口收口。**探针钉定**（单案隔离；多案探针 onhashchange 竞争污
> 染作废）：裸 a 后代 click 也不导航——R112 post 段 nearest activation
> 无 act:a 分支（落 input-change 兜底）；svg 案露出第二根因——summary
> 豁免祖先 walk 不认 SVG ns a[href]（`_zwIsInteractiveContentEl` ns
> 门），summary 误翻转。**落地（R5239）**：①R112 post 补 act:a/act:area
> 分支执行 `_zwAnchorActivate`（receiver 自身让位防双导航）；②summary
> 消费面祖先 walk 补 A/AREA href 判定（不扩共享交互函数）。**集群**：
> anchor-with-inline-element **Timeout 全红 → 5P/0F**；A/B 对照 summary
> 域 REG 0 / **GAIN 5**（窄面 20P/0F/0T 全绿）、details/label 逐位一
> 致；interactive-elements 全域 98 files 415P/65F/2T（Timeout 2 文件均
> 既有挂账）、dialog 子集对 s35 REG 0 / GAIN 1；forms 4627 subtest 计
> 数逐位一致。**已知边界**：点击 summary-less details 内非激活后代不
> 翻转（同根邻近缺口，真浏览器会翻转，记待办）。gates：lib **3429P/
> 0F**、clippy/fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s37-anchor-inline-activation.md](html-semantics-compat/evidence/2026-10-23-m2s37-anchor-inline-activation.md)。
> **下一步**：按挂账序 parser form element pointer 4F、custom command
> `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不出组
> 栈、R371 模板残留、summary-less details 内非激活后代翻转（s37 新）；
> declarative shadow DOM（H 定性后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 36 收口）**：挂账 impl-doc wrapper
> requestClose 收口。**探针钉定**：dialog-requestclose 面 5 files 78P/1F，
> 唯一红正是挂账项；合成 doc（`_zwMEl` 产物）dialog 四方法全缺，R125
> 原型链已接 HTMLDialogElement.prototype 但 prototype 空占位。**落地
>（R5238）**：`HTMLDialogElement.prototype.requestClose`（R4100
> details.name 同款 plain 世界供给惯例）——spec「request to close」
> 步骤照抄：open 缺 return、ownerDocument ≠ 主文档（非 fully active）
> return；trap 世界走 part04 R4200 版先于原型命中，双世界不重叠。
> **集群**：requestclose 面 **78P/1F → 79P/0F**（EXIT 1→0）、dialog
> 集群 82 files 对 s35 账本 **REG 0 / GAIN 1**（351P/59F/1T）、forms 域
> 4627 subtest 对 s34 基线计数逐位一致。**已知边界**：detached doc
> dialog 的 close/show/showModal 三方法仍缺（挂账范围外，探针 3F
> 实证）。R5237 撞渲染流，改 R5238。gates：lib **3429P/0F**、clippy/
> fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s36-wrapper-requestclose.md](html-semantics-compat/evidence/2026-10-23-m2s36-wrapper-requestclose.md)。
> **下一步**：按挂账序 anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边
> 缘乱序、非模态 close 不出组栈、R371 模板残留；detached doc dialog
> close/show/showModal 三方法（新）；declarative shadow DOM（H 定性
> 后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 35 收口）**：挂账 E 余项 ESC 栈序
> 收口，**closedby 双语料面全清**（simple 6/6 + complex normal 面 s34
> 已绿）。**落地（R5236）**：`__zw_esc_dialog_cancel` 两段式——①
> `_zwTopLayer`（showModal+showPopover 共享插入序）反走，栈顶 popover
> 命中即 hidePopover、先碰 open dialog 切组栈；②R4300 组栈原逻辑不动
> （惰性补登记 + none 挡板 + 尾→头首个 open 组）。spec 钉定：close
> watcher 组栈，none dialog 组挡「再往下」不挡「更晚注册」——两段拼接
> 与 watcher 组栈语义在所有混合序一致。**集群**：82 files 411 subtest
> **350P/60F/1T**，同 key 对 s34 ON 基线 diff **REG 0 / GAIN 5**；回归
> 面 canceling 族 8P/0F、keyboard 77 键 A/B 逐位一致。设计迭代：首版
> 统一扫描破 part60 R2（lib test 抓住），两段式恢复。gates：zero-engine
> lib **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s35-esc-unified-toplayer.md](html-semantics-compat/evidence/2026-10-23-m2s35-esc-unified-toplayer.md)。
> **下一步**：按挂账序 impl-doc wrapper requestClose（s36 已收口）、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close
> 不出组栈、R371 模板残留；declarative shadow DOM（H 定性后）待
> parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 34 收口）**：挂账 F 帧驱动快照新鲜
> 化。**落地（R5235）**：Makefile `testharness-html-semantics` 入口默认
> `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a/R3254-KP4 既有开关，
> keyboard 套件同款入口模式；直接调 runner 维持 OFF 旧路径）。**对账三
> 面**：dialog filter 82 files 411 subtest **337P/73F/1T → 345P/65F/1T
> REG 0 / GAIN 8**（closedby-complex normal dialogB/popoverB——快照新
> 鲜后命中循环拿到真实盒）；testharness-html 合成面 7 files 14 subtest
> 逐字节一致；forms 域 401 files **4627 subtest 逐位一致**。**同日续勘
> 归因修正**：**挂账 G 移除**——spec 原文（#blocked-by-a-modal-dialog）
> 就是「栈顶模态 + flat tree 后代豁免」无祖先链豁免，gate 只判栈顶与
> spec 一致，三层栈探针是人造面（误关真因 = 快照滞后已修）；**挂账 H
> 重新定性**——最小探针实证 `<template shadowrootmode>` declarative
> shadow root 不支持（shadowRoot null、template 当活 DOM=R371 伪象），
> shadow slots 16F 归因 declarative shadow DOM 缺口（parser + shadow 树
> 渲染，后者归渲染流），修 light dismiss flat contains 是修在伪象上。
> **挂账 I（新）**：全量 testharness-html-semantics（2594 用例）>5400s
> 跑不完且 `--json` 缓冲到最后超时全丢——runner 增量输出或 make 分域入
> 口。gates：runner Makefile 一行无 Rust 变化、零新增依赖。账本见
> [evidence/2026-10-23-m2s34-raf-frame-driven-default.md](html-semantics-compat/evidence/2026-10-23-m2s34-raf-frame-driven-default.md)。
> **下一步**：E 余项 ESC 栈序（simple 5F——统一 top-layer 序覆盖
> popover 候选，涉 R4300 组栈改造）或按挂账序 impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留；declarative shadow DOM
> （H 定性后）待 parser/渲染流联合切片。
>
> **▶ 上轮状态（2026-10-23 M2 切片 33 收口）**：light dismiss 两遍模型
>（挂账 E 第一刀）。**落地（R5234）**：`__zw_dialog_light_dismiss_check`
> 两遍重写——①popover pass（`_zwTopLayer` 反走自顶向下，stale/摘属性/
> 断连跳过，contains 即级联停，否则 `_zwHidePopover`）；②dialog pass
>（反文档序首个 open+any 一锤定音：backdrop 命中自身 → requestClose；
> contains → break 内层挡外层；否则 requestClose——一次 pointerdown 至
> 多关一个 dialog）；③shadow 守卫（targetSel 解析不到 → 保守跳过）。
> **集群**：82 files 411 subtest **337P/73F/1T**，同 key 411 对 s32 逐位
> diff **REG 0 / GAIN 16**（closedby-complex 的 dialogA/outside-all/
> popoverA 面）。**探针钉定零盒根因 = 快照滞后**：`waitForRender`（双
> rAF）在 shim OFF 模式是同步 stub 不驱动 render，测试体内 gBCR 永远读
> 上一拍快照；`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a 既有开关）
> 实验实证 closedby 46 子测全出 **25P/21F** 对 OFF 同 key **回归 0
> 增益 +8**（normal dialogB/popoverB 全绿）——未默认启用（全集群对账
> 独立切片，挂账 F）。**新挂账**：F 帧驱动快照新鲜化（runner）、G
> runner modal retarget 多模态规则（下层 modal 盒内点击误 retarget
> 栈顶）、H shadow 影域 light dismiss（shadow slots 16F，target 解析
> 穿 shadow）、E 余项 ESC 栈序维持（simple 5F）。gates：zero-engine lib
> **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s33-light-dismiss-two-pass.md](html-semantics-compat/evidence/2026-10-23-m2s33-light-dismiss-two-pass.md)。
> **下一步**：挂账 F（帧驱动快照新鲜化 + 全集群对账，解锁 normal
> dialogB/popoverB 8 案与 simple Timeout 面已实证）或按挂账序 G/H、
> impl-doc wrapper requestClose、anchor-with-inline 导航缺口、parser
> form element pointer 4F、custom command `--` 前缀 CommandEvent defer、
> ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 32 收口）**：popover 有效性语义。
> **红因链破案**：closedby-complex 40F 统一死于 `matches(':popover-open')`
> SyntaxError（host 引擎无该伪类 → promise_test 未处理拒绝全红）；
> closedby-simple 1F = `resetDialogOuterTest` 前置 `hidePopover()` 过度
> 抛出，文件级 Timeout = replica promise_test 不接体 rejection、其余 5
> 体同点 reject 挂死 completion callback（`pending:5 tests:6` 实测吻合）。
> **spec 钉定**：hide popover 算法仅在 validityResult 为 DOMException 时
> 抛——hidden/disconnected 态不匹配返 false 布尔 → hidePopover 静默
> no-op（popover-utils「should not throw」语料权威）。**落地三件**
>（R5233）：shim `_zwHidePopover` lenient 化（非 popover
> NotSupportedError / showing+断连 InvalidStateError / 其余静默）+
> matches 陷阱 `:popover-open` 裸伪类特判（:focus/:modal 同款 JS 侧特判）
> + part09 契约测试翻转与三态断言。**集群**：82 files 411 subtest
> **321P/89F/1T**，同 key 406 对 s31 逐位 diff **REG 0 / GAIN 0**；
> closedby-simple Timeout 解体 → 6 子测收敛 5F+1P；complex 40F 红因迁移
> 为统一行为缺口。**探针钉定两缺口**（一次 outside click → dA/dB 全关、
> pA/pB 不关）：**挂账 E（下一切片主目标）popover light dismiss 未建**
>（R3071 defer 至今）+ **dialog light dismiss 链停**（一次 click 应只关
> topmost clicked dialog 链）；simple 第二 ESC 断言面（ESC 栈序）同挂。
> focus-after-close.html Timeout 既有挂账维持。gates：zero-engine lib
> **3429P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s32-popover-open-semantics.md](html-semantics-compat/evidence/2026-10-23-m2s32-popover-open-semantics.md)。
> **下一步**：挂账 E（popover light dismiss + 链停语义，engine click 路
> 径）或按挂账序挂账 A（强制同步布局冲刷）、impl-doc wrapper
> requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 31 收口）**：dialog showModal 布局
> 域。**谜题破案**：s30「dialog[open] computed block 但 gBCR 全零」是探
> 针伪象——gBCR 读 renderer 上一帧 rect 快照（rect_bridge），open 同 tick
> 读拿到旧布局；等帧后全部有盒。探测环刷新顺序（flush→timers→刷新）钉
> 死 stale 窗口：abspos-dialog-layout 9 案中 6 案红因 = showModal 同 turn
> 测量双侧 stale（expected 300/got 0），**挂账 A（本流）：强制同步布局
> 冲刷**（gBCR/offset* 读前冲刷 + 重排，CSSOM view 语义）。**`:modal`
> 通道落地**：模态性只在 shim JS 栈、DOM 序列化不携带 → 镜像 :focus 注
> 入通道（R4241）建全链——dom `Document.modal_dialogs`/`is_modal_dialog`
> + pipeline `inject_modal_state`（fresh-parse 后注入）+ webview 转发 +
> runner 探测环轮询 `_zwDialogModalAllSels()`（shim 全量导出，栈序、嵌
> 套模态皆匹配）+ matcher `:modal` 臂——作者 `dialog:modal` 规则自此真
> 实匹配。**UA 定位注入实验矩阵**（回归 0 纪律定性）：规范照抄（inset
> 0 + margin auto）= 9 回归（布局对 inset 指定 + size auto 走满视口拉
> 伸，light-dismiss 盒外点击全族失效）；inset auto 近似 = 3 回归（出流
> 重叠、selection/editing 点击几何漂移）；不注入 = **0 回归 0 增益**
>（320P/85F/2T 与 s30 逐位打平）→ 终态不注入。**挂账 B（渲染流）：
> abspos 收缩/fit-content 语义补齐**，UA 规则随该切片按规范回填（A 形
> 态实验已端到端验证轮询链）；s30「ancestor-is-inert 红因 = UA
> position:absolute 缺失」修正为「注入受布局收缩缺口阻断」。挂账 D：
> getComputedStyle 路径 `:modal` 注入（低优）。gates：zero-dom lib
> **863P/0F**、zero-engine lib **3428P/0F**（+1）、zero-style-system lib
> **2446P/0F**（+1）、clippy -D warnings、fmt clean、零新增依赖；渲染/
> UA 样式净变化为零免 product-smoke。账本见
> [evidence/2026-10-23-m2s31-dialog-modal-channel.md](html-semantics-compat/evidence/2026-10-23-m2s31-dialog-modal-channel.md)。
> **下一步**：挂账 A（强制同步布局冲刷，解锁 abspos/non-modal 布局语
> 料同 turn 断言面）或按挂账序 popover 有效性语义（`:popover-open` 选择
> 器 + hidePopover not-showing 抛 + popover-simple 余 hang）、impl-doc
> wrapper requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 30 收口）**：rendering inert 域第一
> 刀——hit-test 切面收口。宿主指针命中（runner `resolve_pointer_target`
> 候选循环）不排除模态 inert 子树，点击模态外元素照常命中。修复三件：
> ① 命中候选循环接 inert 排除（跳过 `!_zwDialogModalFocusGateOk(e)`——
> s29 门语义与 hit-test inert 排除同源直接复用，门补 globalThis 导出供
> 页面域调用，缺省回落原行为）；② backdrop 命中回落：候选全排除且栈顶
> 模态存在 → target=栈顶模态 dialog（新增 part01 `_zwDialogModalTopSel`；
> ::backdrop 无布局盒的近似）——真 Chrome 仲裁（ancestor 语料独立探针）
> 实测「点击 inert 区域 target=HTML」（backdrop 被 display:none 不渲染、
> 穿透到根），回落读 styleSheets 文本扫 `::backdrop{...display:none}`
> 命中则 target=documentElement；③ light dismiss backdrop 臂
>（`__zw_dialog_light_dismiss_check` 加 px/py）：backdrop 回落 target=
> dialog 曾压制 dismiss——target=栈顶模态 dialog 本身且落点在 gBCR 外 =
> 点击 backdrop → 照发 close request（真 Chrome md:dialog 且关闭）。
> 账本：dialog 集群 82 files **320P/85F/2T**（s29 同集 317P/87F/3T），
> 同 key 回归 **0**、增益 2（blocks-mouse-events F→P、light-dismiss-
> touch F→P 连带）+ backdrop-receives-element-events **Timeout→Pass**
>（3T 基线之一收口，backdrop 命中面直接语料）。gates：engine lib
> **3427P/0F**、clippy -D warnings、fmt clean、零新增依赖。账本见
> [evidence/2026-10-23-m2s30-modal-hit-test.md](html-semantics-compat/evidence/2026-10-23-m2s30-modal-hit-test.md)。
> **本轮钉定的相邻归因**：ancestor-is-inert 红因已换——dialog showModal
> 后 UA `position:absolute` 缺失（探针 pos=static、盒 (101,201,50x50)
> 落进父 div 内容区），归 abspos/layout 账（与「showModal 居中」同根），
> 本轮命中面修复已为该案铺平；inert-svg-hittest `:active` 断言引擎恒
> false（matcher 运行时伪类臂缺 active/hover），归交互态伪类账；
> selection 两案依赖指针拖选→Selection 链路，挂账；dialog-inert 依赖
> 居中布局，归 abspos 账。
> **下一步**：dialog UA showModal 布局（absolute + inset:0 + margin:auto
> 居中——abspos-dialog-layout ~10F 与 ancestor-is-inert/dialog-inert 同
> 根）或按挂账序 popover 有效性语义（`:popover-open` 选择器 +
> hidePopover not-showing 抛 + popover-simple 余 hang）、impl-doc
> wrapper requestClose、anchor-with-inline 导航缺口、parser form element
> pointer 4F、custom command `--` 前缀 CommandEvent defer、ESC 组栈边缘
> 乱序、非模态 close 不出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-23 M2 切片 29 收口）**：模态焦点门接页面
> focus()——s28 门只覆盖宿主驱动路径（pre-focus 步、mousedown 默认动
> 作），页面直调 `el.focus()` 不在门内，inert-node-is-unfocusable 13F。
> 修复四件：① `part04 focus()` trap 接门（已聚焦 no-op 之后、失连臂之
> 前；失连元素门内跳过保 R114/T55 detached 契约）；② 门位在 R4003b
> label 前递**之后**——前递源（label 本就非 focusable area）不受模态
> inert 门，inert 语义作用在焦点落点，前递目标经递归 focus() 再入 trap
> 自行过门（inert-label-focus 断言面钉次序：门前置时该案 PASS→FAIL，
> 移位后归位）；③ showModal 栈推进移到 focusing steps 之前（嵌套模态
> 子树 focusing steps 过门）；④ R4600b inert 分支 focus fixup 落 body
> 走 `__zwModalGateBypass`（fixup 是 UA 级焦点回落非 focusing steps，
> Chromium 同款 activeElement===body）。门包含判定补 shadow-including：
> `proxy.contains` 只覆盖 light DOM，既有契约 R4600 R8（delegatesFocus
> 影子树内锚）暴露缺口——新增 part01 `_zwDialogShadowInclContains`
>（parentNode 上行遇 shadow root（`__zwHandle` 命中 `_shadowHandles`）
> 换 host 继续、frag 容器经 `_shadowHandleMeta` 跳 host，路数同 part06
> activeElement retarget，64 步 guard 防反链环）。契约
> `test_dialog_modal_focus_gate_s28` 增 G5/G6（页面 focus() 拒焦／失连
> 归解析节点所有权）。dialog 集群 82 files 407 subtest **317P/87F/3T**
>（s28 同集 303P/101F/3T），逐 subtest diff 同 key 回归 **0**、增益
> **14**（inert-node-is-unfocusable 13F 全清 15/15 +
> remove-dialog-should-unblock-document 连带 F→P）；gates：engine lib
> **3426P/0F**（含 G5/G6）。账本见
> [evidence/2026-10-23-m2s29-page-focus-gate.md](html-semantics-compat/evidence/2026-10-23-m2s29-page-focus-gate.md)。
>
> **▶ 上轮状态（2026-10-23 M2 切片 28 收口）**：corner-cases 余 2F 收口
> ——先建真 Chrome 基准仲裁（playwright-core + chromium-1243，CDP 真实
> 输入层；test_driver 桥接补 ButtonType/特殊键映射），语料基线 29/30
> 唯一失败＝esc-modeless。两案定性：esc-modeless 非 bug（真 Chrome 事件
> 流与引擎逐条一致，上游期望 closed 在 Chrome 自身即失败，记账 Chrome
> parity 非 gap）；click-modal 引擎分歧（真浏览器不可能聚焦模态外元素，
> runner pre-focus 步泄焦）→ 修复＝模态 inertness 焦点切面：`part01
> _zwDialogModalFocusGateOk`（栈顶 open+模态 dialog 子树外非 focusable
> area，stale 栈项跳过，stacked modal 仅栈顶生效）+ `part06
> _zwIsFocusableSel` 接入（宿主 pre-focus 步与 mousedown 默认动作共用
> 门）。契约 `test_dialog_modal_focus_gate_s28` G1–G4。集群 **303P/101F
> /3T**，回归 0 增益 1（corner-cases **29P/1F == 真 Chrome 29/30**）。
> 账本见
> [evidence/2026-10-23-m2s28-modal-focus-gate.md](html-semantics-compat/evidence/2026-10-23-m2s28-modal-focus-gate.md)。
>
> **▶ 上轮状态（2026-10-23 M2 切片 27 收口）**：corner-cases 毒化链根因
> 收口（R4620 批内 selector 时序）——根因在宿主共享 apply 路径而非调用方：
> JS 侧同 dispatch 的 op 引用批开始时的树态（spec insertBefore 的 ref 节点
> 对象不因前序 remove 失效），而 drain 顺序重放时前序 Remove 摘除 dialog →
> ref script 的 `:nth-child` 索引漂移（3→2）→ INS target 失配 warn+skip →
> dialog 永久 detached、同批 SetAttr 连锁失配（remove_child 已清 id_map）→
> 记录毁灭后 29 案 setup 全部 closedBy=none、ESC close request 找不到
> dialog（探针宿主 warn 实锤，R361 stash 走不到——target 先失败）。修复＝
> `apply_dom_mutations_full` 批前预解析 insert-adjacent 族四变体
> （Html/Text/Element/SelElement，同构失配语义）target 为 NodeId（不随树
> 态漂移）+ 四臂失配查表兜底（ref 被批内摘除时维持 skip，语义对齐 JS 抛错
> 面）。契约 `dom_bridge_tests::test_apply_ins_target_pre_remove_index_drift_r4620`
> （修复前红/修复后绿）。dialog 集群 82 files **302P/102F/3T**（s26 同集
> 272P/135F，+30P/−33F），逐 subtest 对 s26 账本 diff 同 key 回归 **0**、
> 同 key 增益 27（全落 corner-cases 毒化案）；corner-cases **1P/29F→28P/2F**，
> 3T 全为 s26 基线既有；focus-shadow 50P/0F、closedby 27/27、requestclose
> 75P 维持；九域全量留待下轮刷新（本轮证据＝集群逐 subtest diff）。gates：
> engine lib **3416P/0F**（含新契约）、clippy -D warnings、fmt clean、零新
> 增依赖。账本见
> [evidence/2026-10-23-m2s27-r4620.md](html-semantics-compat/evidence/2026-10-23-m2s27-r4620.md)。
> 余 2F＝`focusin calls showModal` 变体（focusin handler 内 re-entrant
> showModal 的 ESC/light-dismiss 判定边缘；clicking/modal 键 s25 同 key
> 既有）。
> **下一步**：corner-cases 余 2F（re-entrant showModal 边缘，同域小残差）
> 或按挂账序推进 inert 计算态 13F、abspos/layout ~10F（rendering 域）、
> popover 有效性语义（`:popover-open` 选择器 + hidePopover not-showing
> 抛 + popover-simple 余 hang）、impl-doc wrapper requestClose、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不
> 出组栈、R371 模板残留。
>
> **▶ 上轮状态（2026-10-22/23 M2 切片 26 收口）**：dialog focusing steps 家族
> 收口（R4600 + rollover 挂死修复 R4600c/d）——spec show/showModal step 的
> run the dialog focusing steps、
> close-the-dialog 的焦点恢复落地：`_zwDialogFocusingStepsR4600`（autofocus
> 委派 → focusable area 递归判定（tag 表 + disabled/hidden/inert 子树传播/
> closed dialog 排除/delegatesFocus）→ focus 派 focusin）+
> `_zwDialogRestoreFocusR4600`（previously focused 捕获与恢复）+
> activeElement JS-observable（part06 focus chain retargeting：shadow 树内
> 沿 parentNode 上行解析焦点锚）。R4600b：focus-shadow 余 4F 根因＝R371
> replaceWith 重键的 `_r371Next` 后继快照取到**换行空白文本节点**（nodeType
> 3 无 selector），元素级 QSA R322 归并锚点只认 sel 字符串 → 恒尾插 → 焦点
> 扫描先命中后随兄弟；修复＝快照经 `_childNodeList(父sel)` 元素扫描推进到
> 最近后随**元素**兄弟。rollover 会话 R4600c/d 修 s26 树全量暴露的 3 文件
> 确定性整文件挂死（dialog-autofocus 族，load+focusin 双到齐缺 focusin
> ——runner 预注入 autofocus 先于页脚本，focusing steps 事件相位派发时
> 页监听器未注册且对已聚焦元素不重派）：part06 load 分支补 spec
> flushing-autofocus-candidates（processed flag + 首个合格候选 defer
> task，已聚焦态补发事件相位）+ part01 focusable area 补 pending-removed
> 门（元素级 QSA 无 overlay，同步摘除后仍命中）。testdriver click/down
> focus 步骤经 `_zwFocusSel` 门。
> part66 `test_dialog_focusing_steps_r4600` R1–R13 新契约。九域对账
> 5070→**5116P**（68.32%，Timeout 173 持平、subtests 7488 不变）：
> R4600c/d 隔离对照 **+3P/0 回归**（dialog-autofocus 族 3 挂死文件转
> Pass）；余 +43P 为 R4600 主体（focus-shadow 50F→0、closedby 27/27
> 维持）；同 key 回归 27 全落 corner-cases 毒化链（下述，s26 树已
> 同态）。corner-cases **1P/29F**（s25 28P/2F）：R4600 使 focusin 真实
> 触发音量 handler（remove+reinsert 共享 dialog），test#0 通过但毒化共享
> dialog → 后续 setup 全部 closedBy=none/conn=false——**新暴露**非新引入
> （s25 缺口休眠因 focusing steps 无操作），毒组合＝[reinsert#1+show+drain+
> ESC#1+focus+reinsert#2]，E1-E8 单因子全过；三层缝隙移交下轮：①R334 重
> 插 wire 完整性 ②ESC 派发锚 activeElement 反查 miss ③`_zw_esc_dialog_cancel`
> QSA-miss 顺栈 purge。另发现未修：R371 rekey 时 dst 桶不存在 → 被替换
> template 不入父桶 removed 合并视图残留（对扫描无害，spec 违例候选下轮）。
> **下一步**：corner-cases 毒化链三层缝隙（R334 重插 wire 完整性/ESC 派发
> 锚点/组栈 purge）；余挂账 inert 计算态 13F、abspos/layout ~10F（rendering
> 域）、popover 有效性语义（`:popover-open` 选择器 + hidePopover not-showing
> 抛 + popover-simple 余 hang）、impl-doc wrapper requestClose、
> anchor-with-inline 导航缺口、parser form element pointer 4F、custom
> command `--` 前缀 CommandEvent defer、ESC 组栈边缘乱序、非模态 close 不
> 出组栈、R371 模板残留。
>


> **▶ 拆分动机（2026-10-08 预留编号立项，用户「候选 goal 也立项吧」）**：继
> net-api（79.7% 收口）之后的下一个"整域收割"级主攻——html/semantics 覆盖
> forms/embedded-content/tabular-data 等真实站点核心语义面；form-validation 已
> 打过样（monthly 高亮项），全语义面从未系统圈过。
>
> **▶ 基线事实（立项时点，M1 盘点复核）**：表单校验/控件状态面有存量
> （form-validation + html-syntax M 系遗产）；各子域（forms 约束校验/控件状态/
> 提交语义、tabular-data 表格模型、embedded-content 媒体与 iframe 元素属性等）
> 通过率未知——M1 分域基线定形。
>
> **▶ 域边界（先例：R4114-N 深结构归档条款）**：本 goal 只做 **DOM/语义/API 面**
> （元素属性/IDL 接口/约束/状态机）；布局渲染差异（table 布局、multicol、控件
> 视觉）归 rendering-compat。已归档 html-compat 的余账关系 M1 盘点核对防重复
> 计账。

## Mission

以 WPT html/semantics corpus 分域为验收标准，按子域切片推进（建议序：forms
约束与控件状态 → tabular-data → document-metadata/grouping/text-level →
embedded-content → links/microdata），每域独立收口出数字。

**关键约束**：
1. **WPT 标尺先行**：M1 纯资产切片零源码改动
2. **分域收口制**：一个子域一个数字，不设总通过率门禁（corpus 太大）
3. **轻量修复优先**：约束校验/IDL 面走 shim；触 DOM parser 核心的深结构改动
   单独立项征询

覆盖范围（十个初始子域，fetch 脚本 DIRS 同清单）：forms（最大，含深层子目录
按 web-animations 先例追加）/ embedded-content / document-metadata /
grouping-content / text-level-semantics / tabular-data / interactive-elements /
scripting-1 / links / microdata。

### 排除（明确不在范围内）

- **table/multicol 等布局渲染差异** — rendering-compat 域
- **iframe 浏览上下文树** — navigation-compat M3 门控域
- **媒体元素播放语义** — media 系已归档 goal 域（embedded-content 中的媒体
  元素属性面归本 goal，播放管线不碰）
- **ARIA/无障碍映射** — 未立项域，挂账

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| engine shim | 元素 IDL 面/约束校验/状态机语义 | part 系 JS 语义 |
| dom/style-system | 属性解析与反射（不碰 parser 核心与布局） | 轻量优先 |
| WPT 资产 | html/semantics 十子域导入（分批） | fetch 脚本 + 账本回填 |

### 依赖约束（run-rules §9 碰撞管理）

- **与 rendering-compat**：布局/绘制差异记账回流（table 布局是其活跃工作面），
  本 goal 不越界修渲染
- **与 navigation-compat M3**：embedded-content 的 iframe 面只做元素属性，
  frame 树语义挂其门控
- **与 css-parser 流（本会话 fuzz 修复侧）**：CSS 解析面零交集

---

## Done Criteria

- **DC-1** 十子域导入 + 分域基线 + csv planned 行回填（每域独立数据行）
- **DC-2** 逐域收口：forms / tabular-data / document-metadata / grouping /
  text-level / interactive-elements / scripting-1 / links / microdata /
  embedded-content 各自"分级通过率不再下行两个连续轮次 + 余案归因挂账"
- **DC-3** 回归测试账本纪律（imported-tests.txt）；make test 全绿、make reftest
  零回归、product-smoke 逐字节恒值

## 里程碑

M1 导入与分域基线（✅ 2026-10-15）→ M2 forms 域（最大杠杆，多切片）→
M3 其余九域依序推进 → M4 总收口归档。

## 轮次账

- **2026-10-31 M2 切片 175**：detached 节点选择器身份盗用根修轮——枚举
  镜像钩首刀 P2F=16 勘根两刀（管线等值轻量臂实证换代理论证伪 → mutation
  日志暴露 showModal/close 打在镜像句柄上）定谳：detached 镜像
  `unique_selector_for_node` 盗用在树同 `tag.class` 真元素唯一选择器
  （uniqueness 门 doc 查询、真元素顶替）→ sel↔handle 表顶替 → normal 段
  全打镜像。根修 = 引擎可达门（parent 链不达 root 返 None）+ 钉测 +
  枚举钩撤下（hooks-on 实证注册恢复零 Pass 增益，s176 引擎侧 shadow
  寻址承接）。engine lib 3617P/10F（+1 钉测）/webview lib 830P/0F/
  interactive-elements 437P/26F P2F=0 与 s173 终态逐键同态/forms
  96P/12F 持平。账本
  [2026-10-31-m2s175-detached-selector-identity.md](html-semantics-compat/evidence/2026-10-31-m2s175-detached-selector-identity.md)。
- **2026-10-28 M2 切片 124**：module 网络面收口轮（link rel=modulepreload
  取回面）——勘根四项定谳：modulepreload referrerpolicy 族三面缺口
  （echo-referrer.py 端点 / shim 取回面 / 元素级 referrerpolicy 通道）+
  cross-origin 页实缺 script.py 端点（非 https 传输——www1:8444 别名已
  剥）；module-import-referrer TLA 根因精确化（依赖模块 TLA 不可用，
  build_dep_iife 同步 IIFE——深转换架构面保留）；dynamic import() 透传
  = css-module 复合保留；classic 外链 SRI 无语料需求。施工五件：shim
  `_zwModulePreloadMaybeStart`（同步内联取回——mp5 探针实证 setTimeout
  延后臂不轮转、import() 先行覆写 referrers）+ `_mo_notify` LINK 触发
  臂（顶层 + 下降）+ classic 动态 import() module map 记忆化（恰好一次
  求值 + 失败不入缓存）+ runner 端点仿真 ×2 + pin ×2。域家族 **6 行
  （4F+2T）→ 15/15 全 Pass**；全量 A/B +15P/−4F/−2T 全落家族真翻红 0
  （6653P/1145F/105T → 6668P/1141F/103T，行数 +9 = 挂起页未完成注册
  子测试现完成打印，口径已记账）；sandbox lib 377/0 + engine lib
  3570/0 + runner 256/0。账本
  [evidence/2026-10-28-m2s124-modulepreload-face.md](html-semantics-compat/evidence/2026-10-28-m2s124-modulepreload-face.md)。
- **2026-10-25 M2 切片 86**：scripting-1 开片分诊轮——s85 下一步
  「testdriver-Actions 支持面」过时（s79 已证伪 s78 注记、s80 journal
  已收口 hash-name-dynamic 8P/0F，FILTER 复跑 8/8 复证）；当拍红图
  **169P/335F/86T**（语料 +195 后），失败族四切片序：①dynamic-import
  must-reject → ②stylesheets-blocking-scripts 12F 簇 → ③the-script-
  element 基面 65F → ④module 尾片。一刀（R5286，runner）：moving-between-
  documents 53 案 helper 运行期 iframe 开第二文档——单文档 runner
  不可执行面归 navigation M3 门控，`html_semantics_case_skipped` 增
  helper 标记臂 + 契约单测。域 **169P/335F/86T → 117P/335F/34T**
  （−52P/−52T 全落 helper 案，F 逐位不变；墙钟 47→24min）。账本
  [evidence/2026-10-25-m2s86-scripting1-triage.md](html-semantics-compat/evidence/2026-10-25-m2s86-scripting1-triage.md)。
- **2026-10-24 M2 切片 69**：F3 img.decode() 收口（R5276 三刀）——根因
  decode IDL 无实现（get trap 无臂 + SVGImageElement 原型无方法）。
  ①共享解码核 `__zwImgDecodeCore`（IMG trap/detached 动态臂/SVG 原型三方
  复用，调用时捕获候选 + 恒异步 settle + 双点变异与活跃性复检 + 自驱
  fetch 链）；②part03 detached 产物动态臂 + SVGImageElement.decode；
  ③part04 trap 臂瘦身为闭包供给。契约 part86 两测试。域
  **549P/738F/35T → 607P/678F/37T**（GAIN 58；残留 2F=quick-attach 时序
  归 F2、2 案 F→T 预存环境抖动）；lib **3477P/0F**（+2 契约）。账本
  [evidence/2026-10-24-m2s69-img-decode.md](html-semantics-compat/evidence/2026-10-24-m2s69-img-decode.md)。
- **2026-10-24 M2 切片 68**：F1 img 静态加载管线收口（R5275 六刀）——
  根因 img settle 只接 `.src` IDL 通道（parser 静态 img 无 fetch/settle）
  + template 内容漏抓 + broken 终态缺失。part04 sweep/getter 面 + part05
  R56h restart 语义 + extract.rs template 递归 + testharness.rs 生命周期
  拆三任务链 + available-images `_defer` 化。域 **461P/816F/40T →
  549P/738F/35T**（GAIN 88）；naturalWidth 30P/228F→48P/210F + delay-load
  7 案全绿；interactive **439P/44F/0T 持平**；lib **3474P/0F**（+2 契约
  part85）。账本
  [evidence/2026-10-24-m2s68-img-static-loading.md](html-semantics-compat/evidence/2026-10-24-m2s68-img-static-loading.md)。
- **2026-10-24 M2 切片 67**：embedded-content 开片分诊轮——域刷新
  458P/819F/40T 八族地图（F1 img 静态加载管线 ~260F / F2
  select-an-image-source ~300F / F3 decode 56F / F4 image-map 命中 84F
  渲染流回流 / F5 lazy ~49F / F6 canvas 53F / F7 object|embed 窄面 25F /
  F8 杂项 2F）+ F7 null 反射收口（R5274 一刀，part04 get-trap
  OBJECT/EMBED 分支——contentDocument/contentWindow/getSVGDocument 恒
  null 面）+ 契约 part84 两钉 + 翻转力实证。域 **461P/816F/40T**（GAIN 3
  / REG 0）；interactive 439P/44F/0T 持平；lib **3469P/0F**（+2）。账本
  [evidence/2026-10-24-m2s67-embedded-content-triage.md](html-semantics-compat/evidence/2026-10-24-m2s67-embedded-content-triage.md)。
- **2026-10-23 M2 切片 46**：余 60F 归因分诊六族地图落档（A–G，含
  Chrome-parity 同红族 A/B 与可 shim 收口候选 E）+ 历史 menu 面 IDL 归属
  收口（R5249 双刀，shim part04/part05）——has trap track IDL 列
  kind/srclang/label/default tag 门（TRACK 与 TRACK+OPTION）+ R3069 写臂
  MENU/MENUITEM 负门（FLAT type 与显式 label 列无 tag 门致 menu.type=/
  menu.label= 误写内容属性）。契约 part78 两钉（menu 排除五臂 + track/
  option/FLAT 面四臂不回归）+ 翻转力实证。contextmenu-historical
  **11/11 Pass**；interactive-elements 全域 98 files **423P/57F/2T 对 s45
  逐位 REG 0 / GAIN 3**；lib **3442P/0F**（+2 契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s46-historical-menu-idl-gate.md](html-semantics-compat/evidence/2026-10-23-m2s46-historical-menu-idl-gate.md)。
- **2026-10-23 M2 切片 45**：s37 已知边界「点击 summary-less details 内
  非激活后代不翻转」收口（R5248 双刀，shim part03/part04）——part04
  click 默认动作段 nearest activation 上溯分支（`.click()` 合成半边，
  receiver 三站让位 + 交互后代 type 豁免 + notPrevented 门继承）+
  part03 R112 post act:details 门放宽至 JS dispatch 通道
  （`_zwSyntheticClick` 标互斥防双翻，preventDefault/connected 门由入口
  守卫继承）。契约 part77 两钉（翻转/互斥六臂 + 让位/豁免三臂）+ 翻转力
  实证。interactive-elements 全域 98 files **420P/60F/2T 对 s44 逐位
  REG 0 / GAIN 0**（corpus 无观测面）；lib **3440P/0F**（+2 契约）；零
  新增依赖。账本
  [evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md](html-semantics-compat/evidence/2026-10-23-m2s45-summaryless-details-descendant-activation.md)。
- **2026-10-23 M2 切片 44**：s39 剩余 1F focus-after-close Complex 案收口
  （R5247，shim part01/part04/part06 三处）——跨代 handle proxy 孤儿
  isConnected 假阴性修正（宿主持久反查 + querySelector 同一性核验前置
  复核）+ activeElement 焦点链 retarget light 锚误穿收口（light 锚直返
  门，判据与 stale `_shadowHandles` 投影注册形态解耦）。契约 part76 两钉
  （孤儿 isConnected / light-shadow retarget 双向）+ 翻转力实证。
  focus-after-close 单案 **6/6 Pass**；interactive-elements 全域 98 files
  **420P/60F/2T 对 s43 逐位 REG 0 / GAIN 1**；lib **3437P/0F**（+2 契约）；
  零新增依赖。账本
  [evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md](html-semantics-compat/evidence/2026-10-23-m2s44-orphan-isconnected-retarget.md)。
- **2026-10-23 M2 切片 43**：replaceWith self-removal 父桶记账收口（R5246
  一刀，s26 既有挂账「R371 rekey 时 dst 桶不存在，不记父桶 removed」）——
  sel 臂 remove 步补 `_zwHCLiveInvalidate` 直调（父桶 removed identity +
  全局 pending-removed + live 失效一体），合并视图残留终结、视图即 spec
  终态；记账先于 rekey，promote 分支留护栏。契约 part75
  test_dialog_replacewith_parent_bucket_accounting_r5246（R1/R2）+ part24
  r321 kids 断言更新（宿主残留 DIV 消失）。interactive-elements 全域
  98 files **419P/61F/2T 对 s42 逐位 REG 0 / GAIN 0**；lib **3434P/0F**
  （+1 契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s43-replacewith-parent-bucket.md](html-semantics-compat/evidence/2026-10-23-m2s43-replacewith-parent-bucket.md)。
- **2026-10-23 M2 切片 42**：非模态 close 不出组栈收口（R5245 一刀，
  s23 已知保守项）——close-the-dialog 组栈出栈移出模态臂，modeless
  show/close 循环 stale 项无限增长终结；契约 part74
  test_dialog_group_stack_hygiene_r5245（R1–R4 行为不变量：循环/栈序/
  全关 no-op/模态臂保持）。interactive-elements 全域 98 files
  **419P/61F/2T 对 s41 逐位 REG 0 / GAIN 0**；lib **3433P/0F**（+1
  契约）；零新增依赖。账本
  [evidence/2026-10-23-m2s42-nonmodal-close-stack.md](html-semantics-compat/evidence/2026-10-23-m2s42-nonmodal-close-stack.md)。
- **2026-10-23 M2 切片 41**：挂账定性轮——「ESC 组栈边缘乱序」收口为
  Chrome-parity 同红（corner-cases「focusin calls showModal, modeless」
  ESC 案：crbug 417571176、父 376516550，2026-04 wpt.fyi 仍红）。探针
  钉定事件序 + spec 逐字对账（R4600 恢复门/R5236 组栈零漂移）；R5244
  假设门（subject 非 focusable no-op）GAIN 1 / REG 4（dialog-focus-shadow
  Chrome「show() 聚焦 dialog 自身」面）试错回退，零代码净变化。全域
  419P/61F/2T 对 s40 逐位 REG 0、lib 3432P/0F、零新增依赖。账本
  [evidence/2026-10-23-m2s41-esc-stacked-parity.md](html-semantics-compat/evidence/2026-10-23-m2s41-esc-stacked-parity.md)。
- **2026-10-23 M2 切片 40**：custom command（`--` 前缀）CommandEvent
  派发收口（R5243，s24 defer）——part05 CommandEvent 接口
  （`_defineEventSubclass`，command=''、source=null，无 relatedTarget）
  + part01 `_zwInvokerCommandActivate` custom 分支派发后 return
  （command=原值大小写保留、source=invoker、cancelable=true、
  bubbles=false/composed=true、R312 trusted 印记；分派在调用方
  disabled/preventDefault 门内零新写）。interactive-elements 全域
  98 files **419P/61F/2T 对 s39 打平**；dialog 集群对 s35 **REG 0 /
  GAIN 5**（s39 既有收益）；forms/the-button-element 对 s24 逐位一致。
  gates：engine lib **3432P/0F**（+1 契约 part73）、clippy/fmt 过、
  零新增依赖。账本
  [evidence/2026-10-23-m2s40-custom-command-event.md](html-semantics-compat/evidence/2026-10-23-m2s40-custom-command-event.md)。
  挂账：现行 spec invalid→Unknown 面（跟进涉内建关键字重构）、
  popover 关键字、oncommand IDL、command/commandFor IDL 反射、
  ESC 组栈边缘乱序、非模态 close 不出组栈、R371 模板残留、
  summary-less details 非激活后代翻转、focus-after-close Complex
  dialog usage 深层。
- **2026-10-23 M2 切片 27**：corner-cases 毒化链根因收口（R4620 批内
  selector 时序）——根因在宿主共享 apply 路径：drain 顺序重放时前序
  Remove 摘除移动语义节点 → 后续 INS ref 的 `:nth-child` 索引漂移 →
  target 失配 warn+skip → dialog 永久 detached + 同批 SetAttr 连锁失配
  （JS 侧 op 引用批开始时树态，spec insertBefore 的 ref 不因前序 remove
  失效）。修复＝`apply_dom_mutations_full` 批前预解析 insert-adjacent
  族四变体 target 为 NodeId + 四臂失配查表兜底；契约
  `test_apply_ins_target_pre_remove_index_drift_r4620`（前红后绿）。
  dialog 集群 82 files 272P→**302P**（−33F），同 key 回归 **0**、增益 27
  （corner-cases 1P/29F→**28P/2F**，余 2F＝re-entrant showModal 边缘）；
  3T 全为 s26 基线既有；九域全量留待下轮刷新。gates：engine lib
  3416P/0F、clippy/fmt 过、零新增依赖。账本
  [evidence/2026-10-23-m2s27-r4620.md](html-semantics-compat/evidence/2026-10-23-m2s27-r4620.md)。
  挂账：corner-cases 余 2F（re-entrant showModal）、inert 计算态 13F、
  abspos/layout ~10F（rendering 域）、popover 有效性语义、impl-doc
  wrapper requestClose、anchor-with-inline 导航缺口、parser form element
  pointer 4F、custom command `--` 前缀、ESC 组栈边缘乱序、非模态 close
  不出组栈、R371 模板残留。
- **2026-10-22/23 M2 切片 26**：dialog focusing steps 家族（R4600）+
  rollover 挂死修复（R4600c/d）——show/showModal 跑 spec dialog
  focusing steps（autofocus 委派 → focusable area 递归判定 → focus 派
  focusin）+ close-the-dialog 焦点恢复 + activeElement 焦点链
  retargeting + R4600b replaceWith 空白锚点（focus-shadow 50F→0）；
  rollover 会话修 3 文件确定性整文件挂死：part06 load 分支补 spec
  flushing-autofocus-candidates + part01 focusable area 补
  pending-removed 门。九域 5070→**5116P**（68.32%），Timeout 173 持平，
  R4600c/d 隔离对照 +3P/0 回归；同 key 回归 27 全落 corner-cases
  毒化链（R4600 激活 focusin → 毒化共享 dialog，新暴露非新引入）。
  make test 76 腿全绿、clippy/fmt 不涉（纯 .js）、零新增依赖。账本
  [evidence/2026-10-22-m2s26-r4600.md](html-semantics-compat/evidence/2026-10-22-m2s26-r4600.md)。
  挂账：corner-cases 毒化链三层缝隙（下一切片首选）、inert 计算态
  13F、abspos/layout ~10F（rendering 域）、popover 有效性语义、
  impl-doc wrapper requestClose、anchor-with-inline 导航缺口、parser
  form element pointer 4F、custom command `--` 前缀、ESC 组栈边缘
  乱序、非模态 close 不出组栈、R371 模板残留。
- **2026-10-22 M2 切片 25**：corner-cases hang 排查（R4500）——横切根因
  收口：shim `close()` void 违约（透传 wasOpen boolean）× 语料
  testharness cleanup abort 语义（返非 undefined / 抛错均
  `tests.abort()`）→ file Timeout 吞全文件；s23「detach 窗口身份链」
  归因作废。修复 part04 三处 close 包装 + part14 两处旧断言更正 +
  part60 void 族契约 `test_dialog_void_return_r4500`（R1-R3）。九域
  5016→5070P（同 key 回归 0、翻转 0），Timeout 180→173（closedby
  1P→27/27、corner-cases 1P→28P/2F 等 7 文件解除吞没），+55F 为新上报
  真断言非回归。余 popover-closedby-simple 1 hang（cleanup hidePopover
  抛 not-showing → abort 同族，popover 域挂账）。gates：engine lib
  3394P/0F、webview 785→786P/0F、page-runtime 136P/0F、integration lib
  787P/0F、clippy/fmt 过、零新增依赖；中途 stash→rebase→pop 并入远端
  共享面提交（floats + workers 呈现管线，同动 part04.js 无冲突）。证据
  见 [evidence/2026-10-22-m2s25-r4500.md](html-semantics-compat/evidence/2026-10-22-m2s25-r4500.md)。
- **2026-10-22 M2 切片 24**：ToggleEvent.source / commandfor 簇
  （R4400）——toggle-source 三文件 3/10→10/10，九域 5008→5016P（+8 回归
  0）。含横切修复：user_actions.rs plan 通道私有批回灌共享队列全史 +
  prevented request-close 丢弃 invoker source。证据见
  [evidence/2026-10-22-m2s24-r4400.md](html-semantics-compat/evidence/2026-10-22-m2s24-r4400.md)。
- **2026-10-22 M2 切片 23**：dialog closedby 簇（R4300，rollover 接续
  ——上会话 429 中断遗留实现+契约 WIP，本会话核验收口）——两段推进：
  ①runner 通道：inline_local_scripts 根路径按 wpt_root 内联（域外助资
  产 `/common/top-layer.js`、`/close-watcher/resources/helpers.js` 等
  可达，harness 本体先前已被 replace_script_source 消费无双注入）+
  send_keys uE00C Escape 臂（`__zw_dispatch_key_at_active` 焦点元素三
  形态派发：`__zwSelector`/`_proxyCache` 反查/wrapper 直派；未取消 →
  `__zw_esc_dialog_cancel` 默认动作）+ click 命令 light dismiss 前步 +
  goal 脚本 5 助资产 fetch；②产品语义：closedBy IDL（小写化 + auto 派
  生 + 反射）+ `_zwDialogClosedByState` + close watcher 组栈
  `_zwDialogModalStack`（show/showModal 压组、首个 open 组即停、组内
  none 挡下探、markup/IDL open 惰性补登记、stale 顺栈清出）+
  `__zw_esc_dialog_cancel` 重写（closedby 资格门 + 组栈序 +
  `_zwDialogRequestClose`）+ `__zw_dialog_light_dismiss_check`（任何
  open+any dialog 包含链外落点 → close request；`__zw_pointer_down_
  sequence` 头部钩与 runner click 共享）+ `__zw_reset_form_state` 清
  栈。part60 R1–R5 契约。九域 4980→5008P（68.02%），对 s22 逐 subtest
  diff 净 +28P、回归 1：interactive-elements 202→227P（canceling 族假
  Fail 转真断言：ESC 主断言/组栈 none 挡板 4P/light dismiss 9P/
  cancel-events 族新上报 4P/closedby 逐案 7P/non-modal-canceling 1P）
  + scripting-1 +3（SVG script scheduler 意外翻绿稳定，机制未定位观
  察账），余七域 0 变化；回归 1（bounds-clicking (1,1)：dialog UA
  abspos 布局/hit test 缺口，归 rendering 域）。zero-engine --lib
  3386P/0F、page-runtime 136P/0F、webview 785P/0F、integration
  html_compat 21P/0F、clippy（test-guard 20G 包裹）/fmt 过、零新增依
  赖。挂账：corner-cases ~28 案 file Timeout（focusin remove+reinsert
  → showModal → ESC close 不达，detach 窗口身份链）、ToggleEvent
  .source/commandfor 8F 真断言（下一切片）、ESC 组栈惰性补登记边缘乱
  序（保守）、非模态 close 不出栈（读侧 purge 兜）、bounds-clicking
  (1,1)（rendering 域）、余 s22 挂账维持。账本
  [evidence/2026-10-22-m2s23-r4300.md](html-semantics-compat/evidence/2026-10-22-m2s23-r4300.md)。
- **2026-10-22 M2 切片 22**：dialog 元素簇（R4200，跨午夜会话）——两段
  推进：①part01 dialog 语义主体重构：toggle 事件任务队列（R4009 同款
  per-element 去重合并、preOpen 调用方写前捕获）+ close 事件排队异步
  （spec queue an element task，trusted 通道，旧同步派发 onclose 早于后
  续脚本）+ show/showModal 请求态语义（modal 上 show 抛/非模态上
  showModal 抛，旧静默切态）+ beforetoggle 同步可取消 + 写后重查 +
  close-the-dialog 核心共用 + `_zwDialogRequestClose`（cancel 可取消 +
  重入 busy 守卫，part04 DIALOG tag 门分发）；②共享基建：part03
  Promise.withResolvers polyfill + `_realTag` sel 印章（detach 窗口）、
  part05 `_insertAdjacentVariadic` sel 项（prepend(parsedEl) 丢失）、
  part04 matches 裸属性 lw + addEventListener `{signal}` per-record +
  set trap open moAttr 尾钩、part01 show/showModal/close 调用点 MO
  record（aria helper 共用防双计）。part60 T1–T8 新契约 +
  part14 ④⑧/part59 W3 契约同步（close 排队断言）。九域 4887→4980P
  （67.54%），对 s21 逐 subtest diff 净 +93P、回归 0，全落
  interactive-elements（109→202P）：requestclose 0→75P、toggle-events
  0→12P、close/close-event-async/requestclose-2/-recurse/-closedby/
  submit-dialog-close-event 各 +1。zero-engine --lib 3378P/0F、
  page-runtime 136P/0F、webview 785P/0F、integration html_compat 21P/0F
  （包名 zero-integration-tests 口径补正）、clippy/fmt 过、零新增依赖。
  挂账：closedby 簇 ~85F runner testharness helper 缺口（下一切片）、
  inert 计算态 ~15F、focusing steps ~15F（wrong global realm 面）、
  abspos/layout ~10F（rendering 域）、impl-doc wrapper requestClose、
  MO 内部写仅覆盖 dialog open（保守面）、`_zwSelSeenTag` 幽灵 sel 误
  服务（保守已知）、其余 s21 挂账维持。
- **2026-10-21 M2 切片 21**：details name= 互斥簇（R4100）——spec
  exclusive accordions 双 prong（others 开写面关其他 open / given 插入与
  name 变更面自身让位）+ 组口径（同树根 + 同名非空 name + DETAILS）。
  核心结构发现：innerHTML-into-sel 解析产物 `_zwMEl` plain wrapper 与
  t8r 物化 identity 元素混合同树——统一 node-based 核心
  `__zwDetailsExclusivityNode4100`（childNodes DFS 组枚举、`_elKey`/对象
  身份双态）+ R4009 队列第 4 参 plainNode 通道 + 解析扫描顺序插入模拟
  （树序第一 open 胜出，CeAttach 整批一次扫描）+ 六挂接面 + 解析面三处。
  part01 `_REFLECTED_NAME_TAGS` details + part03 原型 name accessor。
  取舍：解析面排程 toggle（spec 字面）与 test 3 期望序互斥，取 spec 口
  径维持 test 3 F。part60 T1–T8 契约。九域 4873→4887P（66.29%），对 s20
  逐 subtest diff 净 +14P、回归 0，全落 interactive-elements（95→109P）：
  name-attribute 2P/14F+Timeout→15P/3F、fragment-insertion 1P/0F。
  zero-engine --lib 3374P/0F、page-runtime+webview 785P/0F、integration
  html_compat 21P/0F、clippy/fmt 过、零新增依赖。挂账：toggleEvent 尾部
  ontoggle（document-parse 面）、name-attribute test 3 期望序互斥观察、
  xhr/template setup error、dialog 元素簇（下一切片）。账本
  [evidence/2026-10-21-m2s21-r4100.md](html-semantics-compat/evidence/2026-10-21-m2s21-r4100.md)。
- **2026-10-21 M2 切片 20**：details/summary 深层面（R4009，rollover 接续
  ——上会话 429 中断遗留 WIP 本会话续推收口）——toggle 事件任务队列
  （part03：去重/合并/净零/同态不排程/detached；choke point part04 `open`
  IDL setter + attr 尾钩）+ summary 激活行为（part03 bubble 站钩 + part04
  post 分支，首个 summary 元素子 + 交互豁免含祖先面）+ ToggleEvent 接口
  （part05）+ 宿主空格路由（webview SUMMARY → plan_summary）。**宿主双
  toggle 缺陷重设计**：plan_summary followup toggle 与队列双派被集成测试
  证伪（`toggle:true,toggle:true`），移除 followup 发射权归队列，断言
  `click:false,toggle:true,click:true` 不变、事件形状转正。part60 T1–T9。
  九域 4855→4873P（66.09%），对 s19 逐 subtest diff 净 +18P、回归 0，
  全落 interactive-elements（77→95P）：toggleEvent 9P/0F、
  activation-behavior 9P/0F、anchor-without-link 2P/0F、
  details-keyboard-activation 1P/0F。zero-engine --lib 3358P/0F、
  page-runtime 136P/0F、webview 785P/0F、renderer activation 9P/0F、
  integration html_compat 21P/0F、clippy/fmt 过、零新增依赖。挂账：
  details name= 互斥簇 18F+Timeout（下一切片首选）、toggleEvent 尾部
  ontoggle 内容属性面、ToggleEvent.source/commandfor、anchor-with-inline
  synthetic click 导航缺口（导航域）、parser form pointer 4F、
  navigation-compat M3 项维持。
- **2026-10-21 M2 切片 19**：label 本体激活转发（R4008，rollover 双触发
  两会话同树协作合账，commit 88793f87d）——shim 交互豁免身份比较（bubble
  站挂钩 + R155 post 段，target 即 label 本体不豁免）+ 宿主面上移（renderer
  activate_label_at 既有 + webview user_actions plan 链递归 Activate，
  labelable 过滤防环/取消抑制/disabled noop）；shim 不承接宿主 generic
  Event（首发 shim finally 段转发被 renderer 双激活集成测试证伪后重设计，
  checkbox 双翻）。part60 W1–W6 重写 + integration 三面新测。九域
  4848→4855P（65.91%），对 s18 逐 subtest diff 净 +7P、回归 0：
  proxy-modifier tentative 4F 全翻绿（主面）、label-forwarded-click-pointer
  -properties 未上报→2P/0F（isTrusted trusted 面随宿主派发通道翻绿）、
  simulated-click-inert 翻绿。zero-engine --lib 3355P/0F、webview lib
  785P/0F、renderer activation OK、integration html_compat 21P/0F、
  clippy/fmt 过、零新增依赖。协作分工：对侧会话代码提交 + 全量 corpus，
  本会话 kill 冗余全量轮 + docs 合账 + 账本独立复算。挂账：parser form
  element pointer 表规则 4F 同根（深结构单独立项）、details 键盘激活 /
  summary toggle coalescing、navigation-compat M3 项维持。
- **2026-10-21 M2 切片 18**：label.form IDL（R4007，rollover 接续）——
  `.form` getter 分发区补 LABEL 分支，`_zwLabeledControl` 取控件后委托
  控件 `.form`（R4005 关联面全继承），label 自身 form 属性不参与。
  part60 契约 1 测（W1–W3）。九域 4838→4848P（65.82%，账本条目口径），
  对 s17 逐 subtest diff 净 +10P、回归 0：form.html label.form 8 案
  翻绿（7P/10F→15P/2F）、dialog autofocus 两案意外翻绿（焦点链联带）。
  zero-engine --lib 3325P/0F、make test 76 腿 22222P/0F 全绿、clippy/fmt
  过、零新增依赖。挂账：parser form element pointer 表规则 4F 同根
  （深结构单独立项）、proxy-modifier tentative 4F、details 键盘激活 /
  summary toggle coalescing、navigation-compat M3 项维持。
- **2026-10-21 M2 切片 17**：form owner 动态关联（R4005/R4006，rollover
  接续）——`.form` getter 重写（JS 视图 parentNode 三族统一上行 + form
  attr 控件所在树查找 + latest-wins 读 + OBJECT gate）、R52 消零 purge
  WeakRef 复活（identity 断修复、泄漏口径不变）、gEBI pending 窗口同 id
  树序取首 + `_r125Overridden` 改名门。part60 契约 1 测（W1–W4）。九域
  4774→4838P（65.90%），对 s16 逐 subtest diff 净 +64P、回归 0：
  form_attribute.html 43P/57F→100P/0F 满绿、association.window.js 翻绿、
  object.form/form-elements-filter +1+1、select-tab* 意外翻绿 2、
  load-error-events 上报展开 +2。zero-engine --lib 3310P/0F、make test
  全腿绿（GPU adapter 94P/2F 容差漂移结构不阻塞非本轮、归 rendering 流
  观察账）、clippy/fmt 过、零新增依赖。全量轮 5400s 限被兄弟流并行负载
  顶爆、10800s 放行记 evidence。挂账：label.form IDL（form.html 余 10F，
  下一切片）、form_owner_and_table parser 指针表规则 2F（深结构）、
  proxy-modifier 4F、details 键盘激活 / summary toggle coalescing、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 16**：label 激活行为面（R4004）——交互内容判定
  存在性走 has 句柄（get_attr_handle 缺失返空串致恒真误判）、R368 盖章
  属性回填、label 激活转发三路归一（bubble 站挂钩新设 + post/trap 迁移
  `_zwLabeledControl` for 感知 + 交互 target 豁免补齐 + 双闸互斥收敛
  自转发死循环与 post+trap 双转）+ plainParent hop/R11 媒体补链/DFS 控件
  查找/盖章子事件方法转 live。part60 契约 1 测。九域 4710→4774P，对 s15
  逐 subtest diff 净 +64P、回归 0：clicking-noninteractive-* 57F 全数
  翻绿、意外翻绿 7 案（label-inside-anchor/proxy-click Timeout→Pass、
  dialog 系 5 案含 inert-label-focus 观察账消账）。zero-engine --lib
  3289P/0F、make test 主矩阵绿 + browser bin 腿既有失败 stash 对照定谳
  非本轮、clippy/fmt 过、零新增依赖。make test 与 clippy 并行触发 guard
  总内存超限（8GB 预算下编译峰值 ~8.8GB，16GB 放行）记 evidence。挂账：
  form_attribute 动态关联 ~50F（下一切片）、proxy-modifier tentative 4F、
  details 键盘激活 / summary toggle coalescing（M1 起既有）、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 15**：labels IDL 簇 + label.focus() 前递
  （R4003/R4003b/R4003c，rollover 双触发两会话同树合记）——part03
  `_liveQueryCollection` 泛化（自定义快照 + NodeList 原型 target +
  `__zwNL` 印记透读）、part04 labeled control 算法 + labels 反查活集合
  （[SameObject] per-element-key 缓存居 part01 共享作用域）+ focus 前递
  + body/html offsetParent→null。part60 契约 3 测。九域 4683→4710P
  （64.00%），对 s14 逐 subtest diff 净 +27P、回归 0（labelable-elements
  +21、button/input-labels +1+1、forward-focus +4；inert-label-focus
  失败模式漂移 +0，run2 二进制未含 R4003c）。zero-engine --lib
  3273P/0F、make test 58 腿绿 + browser bin 腿 xvfb 复跑 665P/0F、
  clippy/fmt 过、零新增依赖。并发碰头信号（共享 target 指纹 E0425、
  run kill 史 ×4、06:43 一次死因未考证）记 evidence。挂账：
  clicking-noninteractive-* 57F（下一切片首选）、form_attribute 动态
  关联 ~50F、proxy-modifier 4F、inert-label-focus 负载超时观察账、
  navigation-compat M3 项维持。
- **2026-10-20 M2 切片 14**：meter 实际值算法 + double setter（R4002）——
  part04 `_zwMeterActualValues`（六 IDL getter 缺省/钳制 + 浮点整串解析，
  前导 + 跳过为语料钉死行为；R5009 片 e 旧前缀解析修正）+ 六 setter
  WebIDL double 转换（非有限 TypeError）+ best representation 写属性 +
  METER 移出 value 纯反射族（part03 host-read 委托支/part05 tag-gate
  不变）。part60 契约 1 测（W1–W5）。meter 簇 14P/48F→62P/0F，forms
  3688→3736，九域 4662→4683P（63.48%），对 s13 逐 subtest diff 净 +21P。
  **跨流窗口 −27P 对照实验归因、meter 排除**：B2 提交树 16 变更文件复跑
  与 B1 全量账本持平（账本绑定提交树）；无 meter 对照重建原样复现；
  scripting-1 −24 主假设归 script-sandbox 模块缓存三部曲（8458c9223/
  a85c7ade4——安静机恒 Timeout 而 s13 负载下 18P，方向与边界漂移相悖）、
  interactive-elements −3 签名匹配 s7 runner 通道 realm 家族（Fail
  "wrong global"）与窗口候选 3ccfdc479/b2fde4965/368ce9158 并存两说，
  commit 级 bisect 因旧 commit v8 源码构建指纹受阻留属主流。本切片由
  rollover 双触发并发会话同树推进，记账已合并（见切片账本说明）。make
  test 59 腿 58 绿 + 1 既有无 display 环境腿（xvfb 同二进制 665P/0F）、
  clippy/fmt 过、零新增依赖。挂账：labels IDL ~30F（下一切片）、
  form_attribute 动态关联 ~50F（中深）、clicking-noninteractive-* 57F、
  module/dialog 跨流碰头两项（属主流核账）、navigation-compat M3 项维持。
- **2026-10-19 M2 切片 13**：selected autocomplete value 算法 + HTMLCollection
  named access（R4001）——part04 category 全表 + token 算法（INPUT/SELECT/
  TEXTAREA getter，FORM/其余保持枚举反射；field token 小写为 corpus+Chrome
  67/67 钉死行为）、part05 setter 缓存旁路、part03 `_liveQueryCollection`
  named property 补齐（`document.forms.X` 可达）、part03 `.form` 表补
  FIELDSET（named access 修复暴露 vacuous-pass 真值化）。part60 契约
  W1–W5；part15 R3204 input 缺省断言 'on'→''（spec 修正）。九域
  4581→4662P（63.20%），对 s12 逐 subtest diff 净 +81P、回归 0（−11 条
  上报漂移全为 runner 超时边界，P 零损失）。zero-engine --lib 3245 全过、
  make test 全腿绿、clippy/fmt 过、零新增依赖。挂账：meter 净化 44F
  （下一切片）、labels IDL ~30F、form_attribute 动态关联 ~50F（中深）、
  encoding 三簇/double-submit 归 navigation-compat M3 维持、R3204 类
  既有契约钉旧行为风险记入（动枚举反射分发区先跑 part10/part15）。
- **2026-10-19 M2 切片 12**：populateForm 同 turn 二次插入 sibling 断链
  修复（R4000）——`_zwPendingParsedForSel` 候选收集 `candCount > 1`
  提前 return 缺陷删除（签名盖章截断致第二批 pending 节点 hits 恒 0，
  兄弟导航断链；歧义保守语义由 hits 唯一性判定承担不变）。part59 契约
  1 测（W11）。解锁切片 11 flag 契约：form-submission-algorithm t2/t3
  Fail→Pass，九域 4579→4581P（62.10%），对 s11 逐 subtest diff 净
  +2P、回归 0（账本绑定 rebase 后合并树，对 rebase 前复跑 0 变化）。
  zero-engine --lib 3224 全过、clippy/fmt 过、browser bin 腿 :102
  665P/0F、quickjs 矩阵腿全绿、零新增依赖。挂账：
  form-submission 余 F 归 iframe 导航基建簇（navigation-compat M3）、
  既有小挂账维持、二批插入后 body.childNodes 融合视图丢 base 尾节点
  （corpus 无消费面，观察项）。
- **2026-10-19 M2 切片 11**：firing submission events flag + dialog
  returnValue WebIDL 化 + 宿主 click 坐标注入（R3999）——每 form 粒度
  `__zwFiringSubmissionEvents`（spec 步 5.1/5.2/5.4/5.7，入口早退 +
  窗口置位 + finally 清位，仅激活/requestSubmit 通道）；returnValue
  setter DOMString 转换（null→"null"，undefined→"" 兼容例外）；泛型
  click offsetX/offsetY 派发期注入（与 image selected coordinates 同
  口径，事件类保持泛型不改 R108 面；并行 regression-guard 流协作归并，
  账本排查记录 3）。part59 契约 3 测 + part14 期望修正。九域
  4577→4579P（62.07%），对 s10 逐 subtest diff 净 +2P、回归 0：
  dialog-return-value 翻绿、dialog-form-submission 8/8。zero-engine
  --lib 3193 全过、make test 全腿绿（browser 腿 :102 复跑 664P/0F）、
  clippy/fmt 过、零新增依赖。挂账：populateForm 同 turn 二次插入 sibling 断链
  （R3254-K3 面，挡 form-submission-algorithm t2/t3，flag 契约已就位）、
  既有小挂账维持。
- **2026-10-18 M2 切片 10**：form 提交 construct-entry-list 对齐 + dialog
  提交分支（R3910）——constructing entry list flag 共享（submit 入口
  早退先于 submit 事件、构造窗口置位/finally 清位）、formdata 事件共享
  条目存储派发（handler 增删改写提交条目）、action 解析按 spec 顺序后移
  （旧位序丢 formdata/dialog 面）；method=dialog 分支（无祖先 return、
  image 坐标、optional value、_zwDialogClose）；close-the-dialog 步 1
  无 open 早退（四处共享路径）；SubmitEvent/FormDataEvent 构造器
  WebIDL 化。part59 契约 6 测。forms 3602→3605P、dialog-element
  38→47P、九域 4565→4577P（62.05%），对 s9 账本逐 subtest diff 净
  +12P、回归 0。zero-engine --lib 3169 全过、make test 全腿绿
  （browser 腿 :102 复跑）、clippy/fmt 过、零新增依赖。挂账：
  form-double-submit-requestsubmit 36F→Timeout（iframe 导航基建，归
  navigation-compat M3）、submit-dialog-close-event 挂起维持、
  firing submission events flag（t2-t4）、image 坐标 offsetX 面（t5）、
  dialog-return-value returnValue setter String 强转 1F。
- **2026-10-18 M2 切片 9**：selectedness/selectedIndex 语义（R3909）——
  selectedness setting（受影响者胜 affKey + size 1 非 disabled 回落）、
  ask-for-a-reset 汇流钩（`_mo_notify` childList 触发、批插依序、innerHTML
  容器合流、Blink 重回落）、selected 属性变更步两半、selectedIndex setter
  （可全不选）、select.value setter 双形态真态写（旧 guard 双重排除丢写面
  根因修复）、form.reset 复位算法、select.remove(n)/add 祖先检查 + 元素代理
  add、select value-missing（placeholder 四条件）、Option() 构造器
  （属性只随 defaultSelected + clean 层）；读侧 selectedIndex/value/视图
  全改 JS 真态，pristine-only size 1 跳 disabled 回落。part59 契约 12 测。
  select-value-selectedOption +8、selected-index 13/13、select-validity
  6/6 等**净 +34P**；forms 3569→3602P、九域 4531→4565P（61.59%），对
  s8/s6 账本 diff 均零回归。zero-engine --lib 3138 全过、make test 全腿绿、
  clippy/fmt 过、零新增依赖。挂账：option.form 1F、text-setter 1F、
  replaceChild sel 形落尾缺口、t8g 墙钟钉 clippy 并行负载边缘越限观察账。
- **2026-10-18 M2 切片 8**：HTMLOptionsCollection/option list 基建
  （R3908/R3908b）——`_zwListOfOptions` 统一构造（嵌套 optgroup 排除、
  select/hr/datalist 整枝）、HTMLOptionsCollection 代理（length/索引
  setter、replace 取先删后插序规避 overlay identity miss）、
  `_zwIsSelectTag` 前缀判定六处、selectedOptions 代理三 trap + 视图归一、
  option.selected dirty selectedness（不反射；宿主激活脚本
  script_gen.rs 分流属性写）。part58 契约（W1–W9）。common 三文件
  15P、selectedOptions 簇 18P、script-mutation 5P 等**净 +54P**；forms
  3515→3569P、九域 4477→4531P（61.13%），对 s7/s6 账本 diff 均零回归。
  zero-engine --lib 3111 全过、make test 全腿绿（browser 腿 :102 复跑，
  s6 同款环境归因）、clippy/fmt 过、零新增依赖。挂账：ask-for-reset 3F、
  selected-index 7F、add/remove 4F、constructor dirtiness 3F、validity 7F、
  通用 replaceChild sel 形 oldChild 落尾缺口（独立排查）。
- **2026-10-18 M2 切片 7**：option 元素文本/反射语义（R3907）——
  `.text` getter「collect text value + strip and collapse」（script
  命名空间排除模型、子列表递归、NBSP 保留）、`.label`/`.value` 无 ns
  存在即采纳 + collapse 回落（R122 实例层判定）、`.text` setter
  textContent 委托；part03 助手 4 个（R3028 latest-wins 先探、64 层
  防环）+ part04 setter 支，part57 `test_option_semantics_r3907` 契约
  （W1–W7；part16 近 2000 行上限新切片文件）。option 族 36→101P
  （+65P），forms 3450→3515P、九域 4409→4477P（60.40%），对 s6 账本
  逐 subtest diff 零回归。zero-engine --lib 3107 全过、clippy/fmt 过、
  零新增依赖。**stash 对照实验**排除 dialog +3P 因果（runner 案级
  realm 身份非确定性，归 wpt-runner 通道挂账；d11 detached-document
  showModal 不抛为 `_zwIsConnected` 预存缺口）。挂账：option 余簇 7F
  异根（selectedness dirtiness 5F、form owner 1F、isConnected 面 1F）、
  HTMLOptionsCollection/option list 基建簇 ~45F+。
- **2026-10-17 M2 切片 6**：文本控件选区语义（R3906）——selection
  setter「set the selection range」算法 + value setter 选区折叠 +
  异步 select 事件（part06 `_zwScheduleSelectEvent`）+ textarea 换行
  归一修正 + attr 回落 latest-wins + defaultValue= 真脏态守卫 +
  type 变态复位 direction 归一（初版裸 'none' 满账本暴露 80F 同名
  回归后修正，作废首轮取账重跑）+ runner variant meta 展开（R329
  机制复用）。part16 `test_textfieldselection_r3906` 契约（W1–W9）。
  textfieldselection 784P/0F、type-change-state 462P/0F；select-event
  0→270P、selection-not-application variant 化 183+79P；forms
  2848→3450P、九域 3807→4409P（4409/7412 = 59.48%，+602P，分母
  +499），对 s5 账本逐 subtest diff 零回归。zero-engine --lib 3100
  全过、clippy/fmt 过、零新增依赖；make test 三个 browser shell
  间歇失败归因为环境面（X :100 catlink 桥死亡 + load 抖动，隔离与
  main CI 双证）。挂账：editing 管线 `_textSelection` 裸 'none'
  观察账；066.html 预存 hang。
- **2026-10-17 M2 切片 5**：select.showPicker() spec 检查序（R3905）——
  part03 方法分发区新支（disabled→InvalidStateError 先于
  being-rendered→NotSupportedError（display:none + cv:hidden 祖先）先于
  NotAllowedError 门、`_zwTransientActive` 同步消费、multiple/size 不抛），
  part16 R3905 契约单测（9 场景 + 抛错面激活不消费钉针）。show-picker
  六文件 0→7P、customizable-select/nested-options 联带 +1P，forms
  2840→2848P、九域 3799→3807P（零回归；分母 6913 不变）。
  `make test` 全绿、clippy/fmt 过、零新增依赖。教训：corpus 全量轮
  禁与 make test/clippy 并行（首轮负载噪声作废）。
- **2026-10-17 M2 切片 4**：valueAsNumber/valueAsDate setter 抛错面
  （R3904）——part04 两 setter 按 spec setter steps 重排（valueAsNumber
  ±Infinity→TypeError 先于适用性、不适用→InvalidStateError、NaN→''；
  valueAsDate WebIDL `Date?` 转换先行（undefined→null）、不适用
  （含 datetime-local）→InvalidStateError、null/undefined/Invalid Date
  →清空），part16 R3904 契约单测（步序钉针 + 三清空面）。typeerror
  两文件 8F + datetime-local-valueasdate 1F 翻绿、同族 invalidstateerr
  两文件联带 +2P，九域 3788→3799P（零回归；分母 6913 不变）。
  `make test` 全绿、guarded-clippy/fmt 过、零新增依赖。
- **2026-10-16 M2 切片 3**：input.showPicker() spec 检查序 + 瞬态激活消费
  （R3903）——part03 方法分发区新支（不可变检查先于激活门/消费、readonly
  适用 12 类型表、NotAllowedError 门、`_zwTransientActive` 同步消费、无
  focus），latest-wins 属性读三级回落，part16 契约单测。show-picker 三文件
  0→129P（user-gesture 63 / disabled-readonly 44 / does-not-focus 22），
  forms 2697→2829P、九域 3626→3788P（零回归；分母 6656→6913 系 img
  naturalWidth 文件完整上报）。`make test` 全绿、guarded-clippy/fmt 过
  （fmt 既有违规系并行会话遗留，见切片账本）、零新增依赖。
- **2026-10-16 M2 切片 2c**：stepUp/stepDown 全七类型 spec 步进算法 +
  valueAsNumber/valueAsDate IDL 全类型面（R3902c）——part03 `_parseHtmlDateValue`
  date/week/datetime-local 回读校验 + `_formatHtmlDateValue` week/
  datetime-local 新支 + stepUp/stepDown 分支重写（`_reflectedTypeEnum` 类型门、
  `_suDefaultStep`/`_suScale` 缺省表、`_suParseBound` 类型域解析、`_suKInt`
  相对容差近整判定、步进基与 spec 步 3/4/5/7/8/9/10；sel 路径 min/step 走
  `__zw_get_attr_lw` latest-wins 读）+ valueAsNumber getter 全类型，part04
  valueAsNumber setter date 族/month 扩面，part16 R3317 契约修正（方法恒
  暴露 + InvalidStateError + 离格 snap→6）。stepping 簇 +68P、联带 IDL 面
  +64P，九域 3494→3626P（零回归）。`make test` 全绿、guarded-clippy/fmt 过、
  零新增依赖。
- **2026-10-16 M2 切片 2b**：dirty checkedness/value + files IDL +
  `:active`/`:hover` + compound pending 回落（R3902b）——query.rs
  Active/Hover 识别臂，part01 dirty/files 两 map，part03 files/indeterminate
  `_realTag` 门 + 非 file 恒 null + checked 脏态优先，part04 setAttribute/
  removeAttribute('value') 已脏不覆写 + clone 传播 dirty value/checkedness/
  indeterminate + files setter（FileList 存引用/null no-op/否则 TypeError），
  part05 `.checked=` 置脏 + DataTransfer.files FileList 视图，part06 compound
  pending 回落（`[` 预门）。cloning-steps/untrusted-key-event/files/
  checkable-active-space 四簇 +125P，九域 3369→3494P（零回归）。
  `make test` 全绿、guarded-clippy/fmt 过、零新增依赖。
- **2026-10-16 M2 切片 2a**：focus/focus-visible 面（R3902）——part06
  focus-visible 模态旗标，part04 焦点派发接线 + matches() `:focus`/
  `:focus-visible` 分支（isConnected 连接性判定），part01 rAF 预算
  64→512（promise_test 串行链 double-rAF 断链根因）。focus
  -dynamic-type-change 双文件 0→120P、forms +121P、九域 3248→3369P
  （零回归）。`make test` 全绿、clippy/fmt 过、零新增依赖。
- **2026-10-15 M2 切片 1**：input value IDL 面（R3901）——part01 四值模式/
  净化/类型迁移助手，part03 getter 模式分派，part04 setter 分派 + setAttribute/
  cloneNode 接线，part05 `input.type=` setter，用户编辑通道
  （`__zw_text_input`/`__zw_text_delete`/execCommand/script_gen 三脚本）净化
  绕行旗标。the-input-element 528→951P、forms 1897→2320P、九域全量
  2715→3248P（+533，零回归）。中途 13 回归（textarea 缓存写误删）当日定位
  恢复（经验已沉淀 docs/learnings/bugs/2026-10/）。`make test` 全绿、
  clippy/fmt 过；csv 行回填全量数字。零新增依赖。
- **2026-10-15 M1**：fetch 脚本九域+叶子助资产（jsdelivr CDN 回落先落入
  lib.sh 共享路径）；runner 通道 `testharness-html-semantics`（九域顶层递归
  扫描 + skip 规则双侧同源）；基线 42.63%（2715/6369，1151 案），分域账与
  失败簇归因 evidence 落档；csv 数据行回填。挂账：span-limits /
  downloading-resources 两处挂死排除（FIXME）。`make test` 全绿、
  clippy/fmt 过、零新增依赖。

## 依赖约束备注

form-validation 遗产为 forms 域起点；html-syntax-compat（编号 40，已收口）的
parser 面资产为消费基础。
