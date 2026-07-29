# 渲染兼容性目标 — 运行时控制面板

> **▶ 当前裁决（2026-07-29 用户两次指令：① 永不停 / 待决策记账 / 继续推进；② 主做轻量修复、调文档方向防跑偏）**
>
> - **主线 = 轻量修复**（用户第 2 次指令「我们主要做轻量修复」）：沿用 `rendering-compat.md` 的 `2026-07-28 方向裁决` 允许范围——(1) 有 driving test、低风险、A/B 零回归的 CSS2/parser/selector clean lever；(2) 产品/legacy smoke 可见稳定性修复；(3) **文档与代码不一致的纠偏**（本 goal 文档滞后严重，R2202 实证 `loader.rs:429` / `text_metrics.rs:344` 等多处注释过时误导——纠偏本身是高价值轻量活）。每修一个跑 `make test` + 相关 smoke + 必要 dir oracle，net≥0 即 land。
> - **永不停原则**（用户第 1 次指令）：轻量修复持续做；遇需用户拍板事项记「待用户决策」清单并跳过，立刻转下一个轻量修复，**不因单项阻塞停整个 goal**。
> - **深结构护栏·防跑偏**（用户第 2 次指令；纠正本 session 早些「一律放行深结构直接开工」的措辞——那会让后续 agent 跑偏去啃深改）：以下**深结构多会话方向不自主开工，记待决策清单等用户点名**——font-metric 生产激活+A/B（R2202 dormant 基础设施 webview+renderer 已落地、env-gated 未激活，属此类边界外，后续 agent **勿继续推其激活**）、vertical-mode native R1043、taffy replaced-element border-box R2174、Phase A slice-3 IFC 深构造、font-stack C-dep rebuild。clean lever 九重穷尽（R2183-R2190）后若轻量修复暂无新候选，做文档纠偏 + plateau-guard，**不要**借机跳入深结构。
> - **A/B 门禁照旧**：每切片守 kill-switch + 三态 A/B（self-source reftest + product/legacy smoke + 相关 dir oracle）net≥0 才 land；net 负 → 回退、记 evidence、转下一切片。**「net 负」是转下一切片的信号，不是停下等用户的理由。**
> - **真正需用户拍板才记「待用户决策」并跳过**（清单见 `rendering-compat.md` 顶部）：不兼容/闭源许可证、破坏性 git/文件操作、改变 Mission/Done/范围边界、超大磁盘网络下载工具审批无法覆盖——**加上**上述深结构方向。
> - **plateau-guard 附带**：每轮 `make test`/product-smoke/legacy 漂移检查顺带跑。
>
> 旧「⏸️ 暂停裁决（agent 自设）」「🧭 高收益转向（2026-07-28）」块保留在下方作历史依据；**本块覆盖它们，是后续 agent 的默认执行入口**。

> **📍 R2202（2026-07-29）用户推翻暂停裁决 + 3 方向查证 = 文档滞后是根因；定位真未完成 plumbing = 生产未接通 `set_font_metric_map`**。承接用户「永远不要停止，待决策记账、继续推进」，已翻转顶部裁决（⏸️→▶）+ rendering-compat.md 状态/裁决/待决策清单。查 3 候选方向（避免盲推）：① R2197 残余 → 已被 R2198 修 + default-on；② tab_size → R2183 实证 non-lever；③ font-metrics/line-height（R632）→ **代码已大量实施**（R885 `FontMetricProvider` trait + U1b-wiring `set_font_metric_map` + `resolve_normal_line_height` 走真实 ascent−descent+line_gap），**根因 = 文档/注释滞后于代码**（loader.rs:429 / text_metrics.rs:344 仍称 dormant，实际 runner 已接通）。**真未完成项定论**：`set_font_metric_map` 唯一调用方 = `tests/wpt-runner/src/reftest.rs:557`，**生产 webview RenderPipeline 从未调用**（webview.rs:879/885 仅 `set_font_resolver`）→ 生产 line-height:normal 走常数 1.164（DejaVu 真值=1.164 故 welcome 零差；CJK NotoSansCJK 真值≠1.164 = morning 中文 R632 残余 lever）。
>
> **📍 R2203（2026-07-29）R2202「下一切片」状态核对 = ✅ 已在同一提交落地（dormant），本块订正滞后文本 + 连带纠偏 active doc 的 3 处 stale 函数引用**。承接用户「文档 vs 代码不一致的纠偏 = 高价值轻量活」，逐文件核对 R2202「下一切片：webview 生产路径补 `set_font_metric_map`」实际状态：commit `37c8e353`（即承载 R2202 文本的同一提交）**已落地 dormant 生产接通**——① `apps/renderer/src/main.rs:173` 启动期调 `webview.set_font_metric_map(font_loader.build_line_metric_map())`；② `crates/webview/src/webview.rs:901-906` `set_font_metric_map()` 镜像 `set_font_resolver`（存储 + env-gated 下推 pipeline）；③ `webview.rs:884-886` layout 更新时重推。kill-switch = env `ZW_PERFONT_LINEHEIGHT=1`，默认关 = dormant = 与接通前逐字节等价（零回归）。**故 R2202 原文「下一切片：补 set_font_metric_map」为 stale**——wiring 已 DONE，**唯一剩余 = 激活**（翻 env gate / default-on + product-smoke A/B 量化 CJK 收益），属深结构边界，**记待决策清单等用户授权，不自主开工**（顶部裁决一致）。**连带 active doc stale 纠偏（零源码行为变更）**：current-baseline.md:22/:32 + dc-progress.md:115 三处称 `append_webview_primitives()` 在 `app_render.rs:1512`（字段 `1526-1840`），实证函数已于更早提交抽取到 `apps/browser/src/app_render_primitives.rs:17`（字段迭代 `~31-487`，`app_render.rs:1512` 现为无关 UI 搜索提示代码）→ 已订正。archive 内同名引用按 append-only 规则不动。**待决策新增（沿用 R2202）**：是否授权系统性「文档 vs 代码实际状态」核对（文档滞后正导致反复盲推空转；本轮仅修已实证 3+1 处，未做全量扫描）。
>
> **📍 R2204（2026-07-29）✅ 新代码 clean lever LANDED — CDO `<!--` / CDC `-->` token 化 + stylesheet 顶层忽略（CSS Syntax §4.1.1）+ 连带 active doc 行号纠偏**。承接 R2203「轻量修复候选」verify-then-fix 入口，做**代码级**（非 wpt-data 依赖）fresh probe，发现并修复一个 CSS Syntax 合规缺口：**bug** = legacy `<style><!-- ... --></style>` 包裹（HTML 3.2/4 静态页常见，DC-13 Tier 1）会**丢掉全部内部规则**——tokenizer 不识别 CDO/CDC，`<!--` 落 `Error('<')`，顶层 `consume_rule` 当选择器解析失败后 `skip_malformed_qualified_rule` 一路消耗到 `{...}` 块（`skip_simple_block`），**吞掉紧跟其后的真实规则**。**fix**（tokenizer.rs）：`<!--`→`Token::Comment`、`-->`→`Token::Comment`，复用既有 `skip_whitespace` 跳过 Comment 的 ignorable 通道 → 顶层 `parse_stylesheet` 每轮 `skip_whitespace` 自动跳过；`<` 非紧跟 `!--` 时退回 `Delim('<')`。**6 个 driving 单测**（tests/cdo_cdc.rs：单/多规则包裹、仅 CDO 无 CDC、裸 CDC、真实 style 块、`<`-非-CDO 回退）全绿。**门禁**：scoped css-parser 2589/0；**全量 `make test` 12692/0/74**（vs R2201 12686 = +6 新测试，零回归，零 panic/OOM）；`cargo clippy --workspace --all-targets -D warnings` 干净；`cargo fmt --all --check` 干净。**意义**：这是 R2183-R2190「九重穷尽」**之后**首个新代码 lever——证「exhausted」结论局限于 **reftest-visible**（需 wpt-data/oracle 验证）lever；**unit-test 驱动的 CSS Syntax 合规缺口**经定向代码阅读仍可发现，验证用户「永不停止」+ 定向 code-reading fresh probe（非仅 wpt-data reftest probe）的价值。**连带 active doc 行号纠偏**（本轮同一批 verify-then-fix）：rendering-compat.md:38 自渲染 ref `reftest.rs:230-232`→`:278-283`（`run_reftest_with_base`）；current-baseline.md 外链 CSS/图片子资源 stale 行号（`:256`/`:265`/`396/421/448`/`370/395/423`）→ 当前（`prepare_page_subresources` :263 / `resolve_external_css` :363 / `fetch_image_subresources` :402 / `fetch_url` :515）。**下一轮可接续**：继续 CSS Syntax §4/§5 合规缺口定向 code-reading probe（如 `<`/`>` 在 media-query 比较运算符、CDC 在非顶层上下文的 parse-error 行为），或跑 scoped reftest（css-text-decor/css-fonts 近 95% dir）量本 lever 实际 WPT footprint。
>
> **📍 R2205（2026-07-29）✅ 第二个连续 CSS Syntax 合规 lever LANDED — BOM (U+FEFF) stylesheet 首字符剥离（CSS Syntax §3.3 输入预处理）**。承接 R2204「下一轮可接续」入口做代码级 probe。**bug** = stylesheet 首字符若为 U+FEFF（BOM），FEFF（`!is_ascii()`）被 `is_ident_start` 当 ident 首字符 → 污染紧跟其后的首个选择器标签名（`"\u{FEFF}body"` ≠ `"body"`）→ 规则在但选择器失配，`body` 样式不应用。覆盖面：external CSS 经 `net::charset::decode_with` 已剥 UTF-8/UTF-16 BOM，但 **inline `<style>` 文本**（html5ever 不剥文档中段 FEFF）与直接 `parse_stylesheet`/`load_html` 调用仍可能带首 BOM（UTF-8-with-BOM 是 Windows 编辑器默认，legacy/内网页常见）。**fix**（`crates/css-parser/src/tokenizer.rs` `Tokenizer::new`）：按 §3.3 剥首个 U+FEFF（`input.strip_prefix('\u{FEFF}')`），中段 BOM 作 ZERO WIDTH NO-BREAK SPACE 是合法 ident 字符保留。**4 个 driving 单测**（tests/bom_handling.rs）TDD red→green 实证（修复前 `left: Some("\u{feff}body")` vs `right: Some("body")`）。**门禁**：scoped css-parser 2593/0；**全量 `make test` 12696/0/74**（vs R2204 12692 = +4 新测试，零回归）；clippy + fmt 干净。**probe 负结果**（省后续 agent 重探）：① MQ Level 4 比较运算符 `<`/`>`/`<=`/`>=` 与 `(600px <= width <= 1000px)` 全范围语法**已完整实现**（`MediaFeatureOp` + `parse_op`/`parse_leading_value` + 多测，media_query.rs）；② `Token::UnicodeRange` 变体存在但 lexer **从不发射**（niche，`@font-face unicode-range` 专用，未追）。**意义**：CSS Syntax 合规 vein 连续两轮出 lever——R2204 是 §4.1.1 token 层（CDO/CDC），R2205 是 §3.3 输入预处理层（BOM），不同层各有缺口，**「reftest 九重穷尽」≠ 「CSS 合规穷尽」**再确认。**下一轮可接续**：CSS Syntax §3 预处理其他边角（CR/LF/FF 归一化已做、U+0000 NULL 处理）；或 `@font-face unicode-range` 的 `U+` 发射（若有 WPT footprint）；或 scoped reftest 量 R2204/R2205 实际 WPT 收益。
>
> **📍 R2206（2026-07-29）✅ 第三个连续 CSS Syntax §3 合规 lever LANDED — 原始 U+0000 NULL → U+FFFD 替换（§3.3 输入预处理收尾）**。承接 R2205「下一轮」入口。**bug** = 原始（未转义）NULL 落默认 `_ => Token::Error` 分支，顶层 Error 触发 `skip_malformed_qualified_rule` **吞掉相邻规则**（probe 实证：`"p{...}\0\ndiv{...}"` 规则数掉到 1，div 规则丢失）——与 pre-R2204 CDO bug 同根（顶层非 at/非选择器 token 触发畸形规则恢复吞块）。§3.3 规定**所有** NULL→FFFD。**fix**（`crates/css-parser/src/tokenizer.rs` `Tokenizer::new`）：BOM 剥离后，若含 NULL 走归一化分支把所有 `\0` map 成 `\u{FFFD}`（FFFD 是合法 ident 字符，并入相邻标识符，与 chromium 一致）；无 NULL 走原零开销 fast-path（`!input.contains('\0')` 早 return）。**4 个 driving 单测**（tests/null_handling.rs）TDD red→green 实证（修复前规则数=1）。**门禁**：scoped css-parser 2597/0；**全量 `make test` 12700/0/74**（vs R2205 12696 = +4 新测试，零回归）；clippy + fmt 干净。**probe 负结果**：① CR/CRLF/FF→LF 归一化**已正确处理**（`consume_string_content` 把 `\r`/`\x0C` 当串终止符；escape 行连接处理 CRLF），无需补；② 转义 NULL `\0`→FFFD **已在 `consume_escape`**（tokenizer.rs:441/489）。**意义**：CSS Syntax §3.3 预处理层至此 **spec-完备**（BOM 剥离 R2205 + NULL→FFFD R2206 + CR/CRLF/FF 已做）；连续三轮（R2204 §4.1.1 CDO/CDC、R2205 §3.3 BOM、R2206 §3.3 NULL）证明「reftest 九重穷尽」远不等于「CSS Syntax 合规穷尽」，定向 code-reading 持续出 lever。**下一轮可接续**：§3.3 既完，转向 §4 token 层剩余（`Token::UnicodeRange` 的 `U+` 发射——`@font-face unicode-range` 专用，先核 WPT footprint）；或 §5 parser/selector 边角；或 scoped reftest 量 R2204-R2206 实际 WPT 收益。
>
> **📍 R2207（2026-07-30）scoped reftest 量证 R2204-R2206 实际 WPT 收益 + wpt-data 经代理补齐（无代码变更，验证/测量轮）**。承接 R2206「量 R2204-R2206 实际 WPT 收益」入口。**网络**：直连 github 超时，source `~/use-proxy.sh`（`192.168.1.212:7078`）后 `make fetch-wpt-data` 成功（`leizongmin/zeroweb-wpt-data` v1.1，83M，gitignored 可再生）。**量证结果**：① **R2204 CDO/CDC 有真实 WPT 收益**——`css/CSS2/syntax/sgml-comments-000/001/002.xht`（CSS CDO 在 `<style>` 块的直接 reftest）self-source **3/3 PASS，全 0.00% diff，全「真通过-可信」**（DC-14 三态），确认 R2204 把这 3 案从 fail→pass。② **CSS2/syntax dir 274/275 = 99.6%**（self-source，275 案），唯一 hard fail = `uri-013.xht`（6.02%）。③ **BOM/NULL（R2205/R2206）无专门 WPT reftest**（wpt-data 全量 grep 无 FEFF/`\0` 专用案；这些是 robustness/spec 合规，由 unit test 守，reftest footprint≈0，符合预期）。④ css-values dir **0 reftest 案**（该域多为 unit-style 非 reftest）。**uri-013 诊断**：`consume_url`（tokenizer.rs）**已有专门 uri-013 处理**（无引号 url 遇 `"`/`'` 按 §5.4.7 consume-a-string 并入，注释引「driving: uri-013 #three」）——已**部分**实现，残余 6.02% = intricate bad-url/unterminated-string error-recovery（多 div `#one`..`#eight` 各种 `url(foo"bar`/`url(()`/`url([{}])` 边角），**非 clean lever**（复杂错误恢复，1 案，高风险），**deferred**。**结论**：R2204 CDO/CDC 经 reftest 量证有真实收益（+3 syntax 案）；R2205/R2206 是 spec 合规/robustness（unit-test 守，reftest footprint 预期为 0）。wpt-data 现已就位 + release build 已缓存，后续轮次 scoped reftest 成本低（无需重新 fetch/全量 build）。本轮无 `.rs` 变更，不重跑 make test（HEAD 代码自 R2206 未变）。**下一轮可接续**：回到 code-reading vein（reftest 量证已确认 §3 收益，syntax dir 已 99.6% 近顶，uri-013 intricate defer）——`@font-face unicode-range` 已确认需 font-stack（user-blocked，跳过）；§5 parser error-recovery（at-rule/declaration 边角，非 uri-013）或 value-parser 层定向 probe；或继续 scoped reftest 其他 dir（css-position/css-text）找离散 fail。
>
> **📍 R2208（2026-07-30）✅ Selectors Level 4 属性大小写修饰符 `[attr=val i]` LANDED（跨 css-parser + style-system 双 crate）**。承接 R2207「回到 code-reading vein」。**bug** = `[type="text" i]` / `[lang|="en" s]` **整条规则被丢**——`consume_attribute_selector`（parser.rs）取值后仅当紧跟 `]` 才消耗，遇 `i`/`s` 时 `]` 未消耗 → 残余 `i` `]` 破坏选择器解析（driving: attribute_case_flag，TDD red 实证规则数=0）。**fix**（跨 2 crate）：① css-parser `AttributeSelector` 加 `case_insensitive: bool` 字段；② parser 重构 6 个 value-matcher arm（移除各自 `]` 消耗），统一在取值后消耗可选 `i`/`s` 修饰符 + `]`，新增 `consume_attr_case_flag` helper（`i`→true）；③ style-system matcher 加 **additive** 早 return 块——`case_insensitive` 时强制 ASCII 大小写不敏感（覆盖文档语言默认），既有 HTML/XML 分支不动（零回归）。**`s` 修饰符**当前仅被 parser 消耗保留规则，其「强制敏感」语义未在 matcher 强制（走文档默认，`s` 罕见，注释标注后续按需补）。**5 个 driving 单测**（4 parser 规则保留 `tests/attribute_case_flag.rs` + 1 matcher `i`-flag 端到端 `test_matches_attribute_case_insensitive_flag_i`：XML 模式下 `[title="es" i]` 匹配 `title="ES"`、`[class~="active" i]` 匹配 `Btn Active`）。**门禁**：scoped css-parser 2601/0、style-system 1992/0；**全量 `make test` 12705/0/74**（vs R2207 12700 = +5 新测试，零回归）；clippy + fmt 干净。**意义**：R2204-R2206 后第 4 个 clean lever，**首个跨双 crate + 实现完整 Selectors L4 特性**（非仅 tokenizer 边角）——证 code-reading vein 跨 CSS 规范域通用。**probe 负结果**（省后续 agent）：css-position reftest 失败全深（dynamic-change/dynamic-relayout=interactive JS、in-inline=R109/Phase A、abspos-center=`writing-mode: vertical-rl`=R1043 blocked、relative-percent-inset=复杂定位+fixed 数学、replaced-object-backdrop=backdrop-filter underimplemented）；hex 颜色 3/4/6/8 位已全；shorthands 全面（gap/outline/columns/border-image 等）；`:is`/`:where`/`:has` 已支持。**下一轮可接续**：继续 code-reading 其他 CSS 规范域（CSS Values 单位/calc 边角、Selectors nth 公式边角、或更多属性 matcher 边角）；或 scoped reftest `css/selectors` 量 R2208 的 `[attr i]` WPT footprint；TDD red→green + 全量门禁 net≥0 land。

> **📍 R2201（2026-07-29）fresh-session plateau-guard = make test 12686/0 绿，R2198 default-on 代码跨全新构建无漂移，暂停态基线稳定**。新 session 承接，按顶部暂停裁决允许的「低频 plateau-guard」跑 `make test`（test-guard 包裹，排除 zero-render-foundation GPU crate）作漂移守护 + DC-7 持久化证据。**结果：12686 passed / 0 failed / 74 ignored，EXIT=0**（HEAD `f07ffcce` = R2198 default-on 代码；自 R2198[commit 2a290494]后仅 5 个 docs 提交，无 `.rs`/`.toml` 变更；全新依赖 + 工作区构建后重跑，与 R2200 基线 12686/0 字节级一致 = 零环境/工具链漂移）。**结论**：暂停裁决下基线经独立 fresh build 复核稳定，Phase A slice 2+3 default-on 未腐化；自主可推进面仍 comprehensively exhausted（reftest 9 重 + IFC Path A/B 4/4 safe DRY + multicol/float 双路径 vein + R2200 legacy outlier font-wall）。下一步不变 = 待用户点名授权结构性方向（font-stack C-dep / Phase A IFC 单一权威深实现[设计 `phase-a-inline-box-model-full-design-2026-07-29.md` 已就绪] / vertical-mode native R1043 / taffy replaced-element border-box sizing R2174）后转主动推进；期间仅低频 plateau-guard。本轮无代码变更，不重复跑昂贵的全量 oracle（R2199 今天刚跑过同代码）。


## 最近轮次摘要

> **📍 R2203（2026-07-29）doc-code 不一致纠偏 LANDED（零源码行为变更）= R2202「下一切片」状态核对 + active doc stale 函数引用订正**。承接用户「文档 vs 代码不一致的纠偏 = 高价值轻量活」+ R2202 根因（文档滞后致反复盲推空转）。核对实证：(1) R2202「下一切片：webview 生产路径补 `set_font_metric_map`」= **stale**——commit `37c8e353`（承载 R2202 文本的同一提交）**已落地 dormant 生产接通**（`apps/renderer/src/main.rs:173` 启动期调 `webview.set_font_metric_map(...)`；`webview.rs:901-906` 镜像 `set_font_resolver` env-gated 下推；kill-switch `ZW_PERFONT_LINEHEIGHT=1` 默认关 = 零回归）；wiring DONE，唯一剩余 = **激活**（深结构，记待决策清单不自主开工，与顶部裁决一致）。(2) current-baseline.md:22/:32 + dc-progress.md:115 三处称 `append_webview_primitives()` 在 `app_render.rs:1512`（字段 `1526-1840`），实证已于更早提交抽取到 `apps/browser/src/app_render_primitives.rs:17`（字段迭代 `~31-487`；`app_render.rs:1512` 现为无关 UI 搜索提示代码）→ 已订正；archive 内同名引用按 append-only 规则不动。本轮零 `.rs`/`.toml` 行为变更（仅注释/文档），HEAD 自 R2198 起代码字节未变，R2201 fresh-session `make test` 12686/0 已复核稳定，故本轮不重跑昂贵全量 test/oracle（边际价值≈0）。**fresh probe 结论**：CSS2/parser/selector clean lever 仍 exhausted（rendering path 零 TODO/FIXME；近 lever R2132/R2133 均 tokenizer 深逃逸分析，已被 R2183-R2190 九重穷尽）。

> **📍 R2200（2026-07-29）legacy outlier 19-testpage 残余诊断 = font-wall（text wrap-point），非离散 bug，plateau 第九重确认**。R2199c 后扫 legacy 51 fixture diff 排序：46-frameset 100%（OOS frameset 未支持）+ **19-testpage 17.23%** + 20-mixed 11.49% 为 top 非 OOS outlier（余 ~5% font-wall 基线）。19-testpage 经 slice-2 已从 22.39→17.23%，**残余诊断**：LAYOUT_DUMP 第二表 Product A 行 td h=36.6（slice-2 前为 55=3 行 a/i/b 堆叠，现 36.6≈2 行）—— 「A linked product description with italic and bold text」（~50 字符）在 531px 列 ZW **wrap 2 行**，chromium **1 行**（设计 §1.2 期望 ~20px 单行）= **glyph-width/metric 差异致 wrap-point 偏移**（estimate_char_width vs chromium 真实度量，R876 metric-coherence 谱系 / font-wall，**非离散 bug**）。diff 集中 y[0-240] 表区（25-56%），y>240 为 0%。**结论**：legacy outlier 残余 = font-wall（metric coherence），与 reftest plateau 同根（font-stack/user-blocked）。**plateau 第九重确认**（reftest 8 重 + product/legacy outlier 全 font-wall）。rendering-compat 自主 clean lever + 产品稳定性 + Phase A 三项 redirect「立即可做」**全部完成/穷尽**；残余 = font-stack rebuild（user-blocked，R2025 勿推）+ 结构性方向（redirect「暂跳过」）。后续 = 低频 plateau-guard，await redirect 更新/新方向授权。

> **📍 R2199（2026-07-29）✅ default-on plateau-guard 全量 chromium-Oracle 复核 = net-POSITIVE（+66），slice 2+3 default-on 经全语料验证无回归**。R2198 default-on 后跑全量 `make reftest-oracle`（10397 cases）做 DC-7 持久化证据 + 漂移检查。**聚合（DC-14 三态）**：oracle-pass（z_vs_chr<1%）**5969（58.8%）** vs R2185 baseline 5903（58.2%）= **+66 案 / +0.6pp**；credible 5848（57.6%）vs 57.0% = +0.6pp；strict 真通过 414（4.1%）= 持平；不一致 **4177** vs 4243 = **−66**（mismatch 减少）。**default-on 经全语料验证 net-positive 无回归**——slice-2 block-stacking fix 使多 inline block 容器 WPT 案更接近 chromium（a/i/b 不再块级堆叠），oracle 一致率上升。top high-diff 案全为 pre-existing（box-display insert-inline-in-blocks = JS DOM-mutation known-blocked / fonts/generated-content font-wall），无新回归。结论：Phase A slice 2+3 default-on **收官验证通过**（product-smoke + legacy + make test + 全量 oracle 四重绿）。

> **📍 R2199b/R2199c（2026-07-29）fresh probe post-default-on = plateau 再确认 + catalog `background-root-018` niche lever + anchor fix 实证 NO-OP**。R2199 全量 oracle 后做 R1820 式 fresh probe（勿盲信「exhausted」）扫 top 非 blocked 高 diff 案：① backdrop-filter（replaced-object-backdrop 100% / backdrop-inherit-rendered 47%）= backdrop-filter underimplemented（R894 谱系，非 reftest lever）；② **`background-root-018.xht`（43%）= 真 lever 候选但 niche**——body `background: url(cat.png)` + html `background: transparent` 时 body bg 传播到画布，CSS §14.2.3 要求图像定位相对**根元素 padding-box**（margin 16 + border 5 = 21px，匹配 ref `21px 21px`），ZW `paint_bg_image_in_origin(..., layout.x, layout.y)` 锚到根 border-box/0（视觉实证猫从 (0,0) 起，应 (21,21)）→ 整页 tiling 偏移 = 43% diff。**R1428 谱系**（root margin 偏移已修，border offset 未覆盖）。R2199c anchor fix 实证 = NO-OP，真因 = 画布传播 bg-image tiling 相位，深于 R1428 anchor，**ruled out 非 lever**。③ white-space-mixed-001（39.75%）= font-wall/IFC 谱系。**plateau 再确认**：自主 clean lever 仍 exhausted（与 R2183-R2186 + R2184 skip-list 审计 + 本轮 fresh probe 一致）；显著 pass-rate 进展 await user 授权结构性方向。

> **📍 R2198（2026-07-29）✅ Phase A slice 2+3 翻 default-on LANDED 🎉 — struct-check paint_skip-aware 解窄屏残余，slice 2 多 inline block-stacking fix 进入默认渲染路径**。承接 R2197（slice 3 dormant，残余窄屏 multi-line `<a>` struct 假阳性阻 default-on），本轮解残余 + 翻 default-on：- **struct-check paint_skip-aware**：`check_sibling_overlaps`（struct_check.rs）加 `paint_skip: &HashSet<NodeId>` 参数，跳过 paint_skip orphan box 对的 overlap 判定。- **paint_skip 通路**：`render_to_framebuffer_with_layout_with_base` 重构为 private `render_with_layout_inner`（4-tuple fb+root+paint_skip+html）+ 两个 pub wrapper。- **翻 default-on**：3 处 gate `== Ok("1")` → `!= Ok("0")`。kill-switch `ZW_PHASEA_MULTI_INLINE=0` 紧急回退。**默认路径（DEFAULT，无 env）全门禁绿**：make test **12686/0**；**product-smoke 全 8 fixture PASS**（welcome 17.03% / wintertc / morning item-tag:3 / **窄屏 375+320 全 PASS**）；**product-smoke-legacy 51/51 struct PASS** + 19-testpage 22.39→**17.23%（−5.16pp）** + 20-mixed 13.13→**11.49%（−1.64pp）**。


---

## 轮次详记归档

> **R2167–R2197 + R2199c 轮次详记已归档**（2026-07-29 文档结构性精简）：原 master.md 顶部轮次详记区（R2167→R2197 + R2199c，~190 行 blockquote，含多行续行）已逐字保留 → [`archive/master-pre-slimdown-2026-07-29.md`](./archive/master-pre-slimdown-2026-07-29.md) 顶部轮次详记区（零删减）。更早轮次归档见 [`archive/`](./archive/)。

## 通过率快照

- **make test**：12705 passed / 0 failed / 74 ignored（R2208 +5 = `[attr=val i]` parser+matcher 新单测，零回归）
- **reftest oracle**：58.8% oracle-pass（5969/10397，+0.6pp vs R2185 baseline），57.6% credible
- **product-smoke**：welcome 16.84% / wintertc / morning item-tag:3 全 PASS
- **product-smoke-legacy**：51/51 struct PASS，19-testpage 17.23%（−5.16pp），20-mixed 11.49%（−1.64pp）

---

## 当前状态概览

| 维度 | 状态 | 说明 |
|------|------|------|
| 渲染管线 | ⚠️ 全链路贯通但非一致 | HTML→CSS→Style→Layout→Paint→Composite 可运行；但 layout IFC、paint IFC 和 ZeroBrowser glyph 消费仍存在多套坐标/度量路径，`welcome.html` 已暴露用户可见错位 |
| WPT Runner | ✅ reftest 级 | 1,341 个手写 TestCase + 685 个内联 reftest（13 目录 ≥50） |
| Reftest Harness | ✅ 可用 | 分类容差、per-test fuzzy 注解、match/mismatch 模式 |
| CPU 软件渲染 | ✅ 全量图元 | render_full_scene() 支持全部 13 种图元（fills, rounded_rects, gradients, shadows, images, strokes, path_fills, path_strokes, glyphs, clips, transforms, filters, blend_modes） |
| 产品/真实静态页面视觉 smoke | ✅ 证据已持久化·持续修复 | welcome/morning.work/wintertc fixture + product-smoke + chromium Oracle 工具链就绪；**证据已持久化 `evidence/product-static/`**；**post-R632 diff**：welcome **16.16%**、wintertc 13.59%、morning-work 800×600 **18.15%**；★R630 修复用户痛点「文字堆叠看不清」；残余 diff = 中文字体度量（R633 Phase A 死锁）+ R109 IFC + hljs（需 JS），非证据缺口 |

---


## 下一步

> **轻量修复主线（用户 2026-07-29 裁决）**：持续做有 driving test、低风险、A/B 零回归的 CSS2/parser/selector clean lever；产品/legacy smoke 可见稳定性修复；文档与代码不一致的纠偏。每修一个跑 `make test` + 相关 smoke + 必要 dir oracle，net≥0 即 land。

### 待用户决策清单

- **font-metric 生产激活+A/B**：dormant 基础设施 webview+renderer 已落地（R2202）、env-gated 未激活，属深结构边界外，后续 agent 勿继续推其激活，等用户明确授权
- **vertical-mode native R1043**：深结构性方向，等用户授权
- **taffy replaced-element border-box R2174**：深结构性方向，等用户授权  
- **Phase A slice-3 IFC 深构造**：深结构性方向，等用户授权
- **font-stack C-dep rebuild**：user-blocked（R2025 踩坑勿推），等用户明确授权
- **系统性「文档 vs 代码实际状态」核对**：R2202 发现文档滞后正导致反复盲推空转，等用户授权

### 轻量修复候选

**当前活跃轻量主线 = 文档 vs 代码不一致纠偏 + CSS Syntax 合规缺口定向 code-reading probe**（R2202 根因：文档滞后致反复盲推空转；用户授权「文档 vs 代码不一致的纠偏 = 高价值轻量活」）。

★ **R2204 重要订正**：「CSS2/parser/selector clean lever 九重穷尽」结论**局限于 reftest-visible lever**（需 wpt-data/oracle 验证者）；**unit-test 驱动的 CSS Syntax 合规缺口**经定向代码阅读仍可发现——R2204 即证：CDO/CDC token 化缺口被「rendering path 零 TODO/FIXME」式扫描漏掉，但经读 tokenizer 主 dispatch + `skip_whitespace`/`skip_malformed_qualified_rule` 行为即定位修复。故后续 agent 不应把「reftest 九重穷尽」当成「所有 CSS lever 穷尽」。

下一轮可接续的具体入口（逐条 verify-then-fix，**不**做 pending 的全量系统性审计，只修已实证项；优先 unit-test 驱动，不强依赖 wpt-data）：
- CSS Syntax §4/§5 其他合规缺口定向 probe：media-query 比较运算符 `<`/`>`/`<=`/`>=`（MQ Level 4）token 化与 parse；CDO/CDC 在**非顶层**上下文（qualified rule prelude / declaration value）应 parse-error 而非一律忽略——核实现状是否需收紧。
- 量本 R2204 lever 的实际 WPT footprint：scoped `make reftest-upstream FILTER=css-syntax`（需 wpt-data，网络 fetch；不强求）看 css-syntax dir 是否有 `<!--` 相关 case 翻绿。
- 文档 vs 代码行号漂移续修（沿用 R2203/R2204 verify-then-fix 模式）。
- 每修一个：仅文档→跳过昂贵 make test；连带 `.rs`→跑 scoped test-guard + 必要时全量 `make test`，net≥0 land。

期间不要借机跳入深结构（见「深结构性方向」与「待用户决策清单」）。

### 深结构性方向（等用户授权）

以下方向不自主开工，记待决策清单等用户点名：

1. **Phase A IFC 统一**（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)）— 解 large-font（ifc-008/009/011）+ welcome/morning.work 文本度量残余。R207 narrow 已证 font-051 +1 可行；需多轮 set-diff 收敛 broad 应用 + 守 multicol-fill-auto 反向依赖。
2. **Phase 2 嵌套 multicol fragmentation**（[`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)、[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)）— 解 multicol-breaking（css-multicol 最大失败聚类）。R319 证纯 inline 迁移零增益，真实价值在嵌套 / 混合内容碎片化。
3. **baseline-export 真修复** — taffy 0.12.1 已支持 flex §8.5 first-baseline；fresh 实测 baseline-000~008 + flexbox-baseline-synthesis 全 near-pass font-wall，非结构性缺口。**勿再以 baseline-export 为 lever**。
4. **DC-9 blend_mode** — paint-isolation 架构（offscreen 子树渲染 + source/dest 双纹理 blend pass），低 reftest footprint（~2-4 案）。

---

## Done Criteria 进度

**Done Criteria 进度**：详见 [rendering-compat.md §Done Criteria](../rendering-compat.md#done-criteria)

---


## 通过率详细快照

### reftest 通过率（最新）

- **self-source 全量目录**：css-grid 32/48=66.7% + css-position 63/95=66.3% + css-tables 77/112=68.8% + css-flexbox 368/496=74.2% + css-multicol 195/451=43.2% + css-text-decor 244/246=99.2% + css-fonts 282/284=99.3%。**7 目录聚合 self-source 1148/2032=56.5%**
- **7 目录全量 chromium-Oracle 真一致 47.5%**（post-font-wall 全幅，旧 stale 36.4%）：grid 20/49=40.8% / position 57/97=58.8% / tables 74/115=64.3% / flexbox 298/497=60.0% / text-decor 118/242=48.8% / fonts 100/282=35.5% / multicol 157/452=34.7%（最低，chr<1%，聚合 **824/1734=47.5%**）
- **self-source strict**：295/490 (60.2%) @ 锁定 0.1%/0.5%（DC-14 真通过口径，pre-grid-expand）
- **chromium-Oracle 广义一致**：200/475 (42.1%) @ chr<1%（R391 锁定诚实基线）；严格 self-pass&chr<1% **177/475 (37.3%)**；污染 46.5%

### 产品 smoke（最新）

- **welcome**：16.84%（R632 line-height override 改善，自 R371 16.98%）
- **wintertc**：13.59%（R227+R255 后）
- **morning-work**：18.15%（R630 多行 y 分行 + R632 line-height 让中文行级度量差异诚实显现）
- **fullpage**：48.65%（R255 ua_default_display 修 4× 幻影盒 89.14%→48.65%）

### 测试覆盖率

- **cargo test**：7800+ 测试全部通过
- **cargo clippy**：`cargo clippy -- -D warnings` 通过
- **零 #[ignore] 测试**：仅 real_website_compat.rs 有 59 个 #[ignore]（因本地网络不稳定）

---


## 历史轮次归档索引

> **为避免 master.md 无限膨胀，逐轮详记按 era 迁出 `archive/`**（内容 100% 保留，未去重、未重排）；master.md 现仅保留当前活跃状态 + 最近 5 轮摘要 + 跨会话架构入口。**2026-07-29 精简前的完整 master.md**（含 R2167–R2201 全部轮次详记 + 旧章节，零删减保底）→ [`archive/master-pre-slimdown-2026-07-29.md`](./archive/master-pre-slimdown-2026-07-29.md)。逐轮详记按 era 分档：
>
> - **R2167–R2197 + R2199c**（2026-07-29 瘦身迁出，含多行 blockquote 续行）→ [`archive/master-pre-slimdown-2026-07-29.md`](./archive/master-pre-slimdown-2026-07-29.md) 顶部轮次详记区
> - **R2086–R2095**（reftest autonomous forward 穷尽调查）→ [`archive/rounds-r2086-r2095-plateau-exhaustion.md`](./archive/rounds-r2086-r2095-plateau-exhaustion.md)  
> - **R1651–R2025**（逐轮详记，~660 行）→ [`archive/rounds-r1651-r2025-accumulated-detail.md`](./archive/rounds-r1651-r2025-accumulated-detail.md)
> - **R1579–R1601**（font-wall plateau era）→ [`archive/rounds-r1579-r1601-fontwall-plateau-era.md`](./archive/rounds-r1579-r1601-fontwall-plateau-era.md)
> - **R1602–R1650**（lever scan / chrome127 era）→ [`archive/rounds-r1602-r1650-lever-scan-chrome127-era.md`](./archive/rounds-r1602-r1650-lever-scan-chrome127-era.md)
> - **R142–R302** → [`archive/rounds-r142-r302.md`](./archive/rounds-r142-r302.md)
> - **R894–R990**（multicol Phase 2 / harness JS vein / R109 §9.2.1.1 backfill / aspect-ratio / R990 ascent era）→ [`archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md`](./archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md)
> - **R991–R1093**（multicol spanner·Phase 2 / logical props / vertical-mode / FreeType C-dep / ::first-letter Phase A / nbsp·word-spacing / plateau era）→ [`archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md`](./archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md)
> - **R569–R881**（master preamble 摘要段）→ [`archive/rounds-r569-r881-master-preamble-summaries.md`](./archive/rounds-r569-r881-master-preamble-summaries.md)
>
> 更早 era（R11–R718）的归档清单见上述文件内链接。逐轮结论摘要亦见顶部「综合裁决」表。

