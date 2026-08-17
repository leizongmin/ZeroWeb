# 页面渲染兼容性 — WPT Reftest 驱动的渲染正确性目标

**版本**: v1.0
**日期**: 2026-06-06
**状态**: Active / Continuous（2026-07-29 用户推翻 agent 自我暂停，恢复持续推进）
**执行模式**: 轻量修复优先（永不停）；遇需用户决策项或深结构方向 → 记入「待用户决策」清单 → 跳过 → 继续其他轻量修复
**父目标**: `docs/goal/zero-web.md`（ZeroWeb 总体目标）

> **说明**
> 本文档是 ZeroWeb 页面渲染兼容性的专项目标执行契约。目标是以 WPT reftest 通过率为验证标准，将 ZeroWeb 的 CSS 渲染输出对齐到 Chromium（Chrome/Edge）水平。本文定义了使命、边界、完成标准、执行协议和文档治理规则，供后续 `rally run` 会话作为稳定输入使用。

> **🔗 当前阻塞 + 可执行方案（2026-07-25）**：reftest ~57% plateau（自主 clean-lever 穷尽）。四大阻塞（Phase A / font-stack / P1b / P3）方案见 [`rendering-compat/blockers-resolution-plan-2026-07-25.md`](rendering-compat/blockers-resolution-plan-2026-07-25.md)。**实施入口**：Phase A 首切片（pre-authorized）= [`rendering-compat/phase-a-slice1-inline-block-linebox-mechanism-2026-07-25.md`](rendering-compat/phase-a-slice1-inline-block-linebox-mechanism-2026-07-25.md)；P1b 独立 RFC = [`rendering-compat/p1b-rfc-2026-07-25.md`](rendering-compat/p1b-rfc-2026-07-25.md)。运行时控制面板 = [`rendering-compat/master.md`](rendering-compat/master.md)。
>
> **🧭 方向裁决（2026-07-28）**：当前执行策略改为"转投高收益项目 + plateau-guard"。后续 agent 不应再把 WPT 95% 当作短期冲刺目标，也不应在已反复证伪的单点切片上循环。允许继续推进的工作仅限：（1）有明确 driving test、低风险、A/B 零回归的 CSS2/parser/selector clean lever；（2）产品/legacy smoke 的可见稳定性修复；（3）为 Phase A 完整 inline-box-model / IFC coherence 输出可回退实施设计。暂跳过：旧 Phase A 首切片、R109 单点、37-form-controls 单点、font-stack rebuild/M18、P1b JS Bridge 深改、P3 真窗口/GPU 验收、inline SVG/SVG intrinsic sizing、sticky/scroll-snap/动态滚动。旧 2026-07-25 blocker 文档保留作历史依据，不再作为默认开工入口；最新执行方向以 `rendering-compat/master.md` 顶部裁决包为准。
>
> **▶ 轻量修复优先裁决（2026-07-29 用户两次指令）**：用户指令——「**永远不要停止，把待决策的记录到文档，在我没决策之前，继续推进其他剩余任务**」+「**我们主要做轻量修复，调整文档方向确保不会跑偏**」。据此：(1) **主线 = 轻量修复**：沿用 `2026-07-28 方向裁决` 允许范围——CSS2/parser/selector clean lever（driving test + 低风险 + A/B 零回归）、产品/legacy smoke 可见稳定性修复、文档与代码不一致的纠偏（本 goal 滞后严重）；每修 net≥0 即 land。(2) **永不停**：轻量修复持续做，遇需拍板事项记「待用户决策」清单并跳过，继续下一个轻量修复，不因单项阻塞停 goal。(3) **深结构护栏·防跑偏**（纠正早些「一律放行深结构」措辞）：font-metric 生产激活+A/B（R2202 dormant 已落地、**勿继续推激活**）、vertical-mode native R1043、taffy replaced-element border-box R2174、Phase A slice-3 IFC 深构造、font-stack C-dep rebuild 等**深结构多会话方向不自主开工**，记待决策清单等用户点名；clean lever 九重穷尽（R2183-R2190）后若暂无新轻量候选，做文档纠偏 + plateau-guard，**勿借机跳深结构**。(4) 本文「执行协议」生效——CONTINUE 是默认输出、未完成/证据不足/状态不一致是继续信号而非 BLOCK、自主修复不等用户逐步指令。
>
> **▶ 主线切换裁决（2026-08-04 用户决策）**：用户裁决——「工作切回父目标 zero-web（恢复 P1 DOM/JS Bridge 原生化），渲染兼容性先缓一缓」。据此：(1) **本 goal 降频守成**，不再作为自主主线推进：保留低频 plateau-guard（`make test` triple-guard 周期拉长——父目标侧有 .rs 变更或每 ~10 轮跑一次，守护 13192 全绿基线）。(2) **待用户决策清单原样保留**：vertical-mode R1043 / taffy R2174 / Phase A IFC / font-stack C-dep / srcset 等深结构继续等点名；用户点名任一深结构 → 立即切回本 goal。(3) 文档纠偏停止主动排程（design-doc 引用核验已近收官），仅在有 .rs 变更引发 drift 时顺手修。(4) 本裁决不推翻 2026-07-29「永不停」指令——守成 + 待命即执行形态，恢复时零成本；父目标主线（P1a 事件循环/fetch/Observer 真实化）与渲染侧无冲突。
>
> **▶ 字体栈实施裁决（2026-08-09 用户决策）**：用户批准 font-stack coherence rebuild（`fontdue-replacement-scoping.md` v0.2.3 / `unified-font-stack-design.md`）——**接受 HarfBuzz C 依赖**（FreeType 已 default-on），本 goal 由降频守成**恢复主动实施**。执行形态：拆分为独立可验证切片（度量统一 → 光栅统一 → 塑形 HarfBuzz → 字体回退逻辑），每片 kill-switch + 结构签名 gate + 全量 oracle A/B，net≥0 才落地；第一刀选最小可度量收益切片。与父目标 P1a（engine DOM 桥）工作面不重叠，可并行。深结构清单其余项（vertical-mode R1043 / taffy R2174 / Phase A IFC / srcset）仍等用户点名。
>
> **▶ 字体栈当前进展（R3252-F east-asian+position-OT·2026-08-12）**：resolved-face fallback 与 `font-size-adjust` 已从 computed style 贯通到 layout advance、engine callback/cache、paint glyph size 和最终 raster；`ZW_SHAPED_FALLBACK` **default-on**，`=0` 回滚。**R3245-F** two-value `font-size-adjust` 全栈贯通。**R3246-F** `letter-spacing: 0em` ligature 抑制修复。**R3247-F** CSS `font-size` 绝对关键字 + HTML `<font size>` 映射。**R3248-F** CSS `font-synthesis` shorthand + 4 sub-properties。**R3249-F** CSS font-family quoted generic name 区分。**R3250-F** HTML `dir` attribute → CSS `direction` + `unicode-bidi` presentational hints（HTML §14.3.5）+ `font-variant-numeric` OT feature injection（`ordn`/`zero`/`lnum`/`onum`/`pnum`/`tnum`/`frac`/`afrc`）+ BiDi paragraph direction 从 CSS `direction` 贯通到 UBA。**R3251-F** `unicode-bidi: plaintext` → BiDi paragraph level auto-detect（UAX #9 HL4）+ CSS `font-variant-caps` 全属性支持（parsing/inheritance/OT injection: `smcp`/`c2sc`/`pcap`/`c2pc`/`unic`/`titl`）。**R3252-F** CSS `font-variant-east-asian` 全属性支持（parsing/inheritance/OT injection: `jp78`/`jp83`/`jp90`/`jp04`/`smpl`/`trad`/`fwid`/`pwid`/`ruby`）+ CSS `font-variant-position` 全属性支持（parsing/inheritance/OT injection: `subs`/`sups`）。Oracle A/B：css-fonts `84/282 (29.8%)` 保持，writing-modes `2/81 (2.5%)` 保持；reftest `687/687`。当前 Oracle 差异瓶颈：固有字体渲染差异（~1-3%）+ complex shaping（ligatures/BiDi/variable fonts）+ @font-face web font 依赖。详细执行态见 [`rendering-compat/master.md`](rendering-compat/master.md)。
>
> **▶ 字体栈最新进展（R3424-F author face layout advance·2026-08-14）**：显式 `@font-face` 的初次layout与paint IFC现共享ordered face advance；text node override键、ordered list主face恢复和generic/author边界均已闭合。`ZW_AUTHOR_SHAPED_LAYOUT=0`可回滚，generic/system继续使用既有估算路径，避免全局shaped-layout的历史性能回归。css-fonts Chromium Oracle 282案为19改善/19微退/244持平，rounded净改善`23.33pp`，pass/credible/strict/near保持`83/78/50/33`；self-source `280→282/287`。`font-size-adjust-009/010/011`及固定Noto Sans资源已资产化；reftest`687/687`、product smoke、workspace clippy、完整`make test`和性能绝对预算均通过。variable-font gate仍default-off。详细证据见[`rendering-compat/master.md`](rendering-compat/master.md)。
>
> **▶ 组合态回归修复（R3426-F CJK contiguous opt-in·2026-08-14）**：远端 CJK form-layout 修复再次把普通字体的 per-character CJK 连续模式改为 default-on，重开 R3388 已记录的 advance wall。最新代码 A/B 仍为净负：welcome `16.43%→23.93%`（`+7.50pp`，击穿严格 `<20%` 产品门）；css-text self-source 虽多 1 个宽松 pass（`1754→1755/1826`），可信 strict 却从 `528→410`（-118）；已有 Chromium Oracle 107 案通过数保持 `7`，仅两案微改善，不能覆盖产品退化。现恢复 `ZW_CJK_CONTIGUOUS=1` 显式 opt-in，Ahem 路径与远端 baseline/atomic-inline/leading-space 修复全部保留；默认 product smoke 全 viewport 重新通过；reftest `687/687`、可信 strict `573`，bench `16/16`、workspace strict clippy 与完整 `make test` 均通过。禁止在 layout/paint advance 尚未全局同源前再次默认开启。
>
> **▶ 字体性能回归修复（R3427-F hmtx hot path·2026-08-17）**：保留真实 hmtx 换行与 Chrome parity 收益，消除逐字符字体字节 hash/`Arc` clone/thread-local borrow、单次布局重复全文档 font override 收集，以及 taffy 文本回调重复 family matching。face cache 现按进程唯一 `font_instance_id` 批量测量；布局全阶段共享一次预计算 node-to-font map。medium layout 同机定向 p95 `~546→~411ms`，完整门禁相对巡检最坏 `850ms` 下降约 41%；16/16 microbench、绝对页面预算、RSS、form-input 通过，异构硬件相对门仅 WARN，`≤141ms` 残余继续跟踪。reftest `687/687`、可信 strict `552`、welcome `16.61%` 与全 viewport 产品门通过。
>
> **▶ 字体性能增量（R3428-F hmtx character cache·2026-08-17）**：完整文本 run cache 改为 `(完整 font_id 链,size,char)` 加法缓存，避免 8,000 文本节点页面反复复制字体链和原文，并让不同 run 共享字符宽度；fallback chain 与 `unicode-range` 更新会清缓存，`ZW_HMTX_CHAR_CACHE=0` 可回退。hmtx 模块独立后 `loader.rs` 恢复 2000 行。同一 release 二进制 A/B：medium layout/total p95 均下降 16%，RSS `160.4→155.5MiB`；完整性能绝对门、reftest `687/687`、可信 strict `552`、welcome `16.61%` 与全 viewport 产品门通过。`≤141ms` 相对目标尚未关闭。
>
> **▶ 字体栈增量进展（R3254-F relative-font-weight·2026-08-12）**：**R3253-F** 补齐 CSS `font-variant` shorthand（展开并重置 ligatures/caps/numeric/east-asian/position）与 `font-stretch` keyword/percentage computed storage/inheritance，并让 `font` shorthand 按 CSS Fonts 4 §4 重置全部 variant 子属性和 stretch。**R3254-F** 将 `font-weight: bolder/lighter` 从 getComputedStyle 私有后处理前移到 style-system computed 阶段，按父元素绝对字重映射为 100/400/700/900；layout、paint、gCS 现消费同一结果，engine 删除重复父链 DFS。Oracle A/B：css-fonts 仍 `84/282`，目标页 `bolder 11.31%` / `lighter 10.91%` 持平；两页依赖本机未安装的 `CSSTest Weights` 字体，故当前 Oracle 无像素翻转但零退化。验证：相对字重映射表/根节点/多层父链与既有 gCS 测试通过，workspace clippy clean，reftest `687/687`；全量测试仍受既有 browser 表单快照时序、无 wgpu adapter 与 QuickJS 缺 `libclang` 环境门阻断。
>
> **▶ 字体栈增量进展（R3255-F font-stretch-face-matching·2026-08-12）**：`@font-face font-stretch` 描述符现按 keyword/percentage 解析并穿过 engine 提取、WebView async fetch/drain、browser/renderer/WPT 三宿主注册；render-foundation 新增统一 width-specific alias 与 CSS Fonts width-first matching，normal width 同时保留旧 alias，layout ordered faces、paint 主 face、控件/列表/效果文本全部消费 computed `font-stretch`。CSSOM 同步支持 `font-stretch` 百分比与 `font` shorthand stretch 序列化。上游 `font-stretch-01..18` 为可信 strict `18/18`、全部 `0.00%`；Chromium Oracle A/B：01..11 从 `2.41%→1.54%`，12..18 保持 `1.54%`，合计改善约 `9.57pp`、零回归，目录门仍 `84/282`（剩余 1.54% 光栅差异未跨 1% 阈值）。workspace clippy clean、reftest `687/687`、产品 smoke 全 viewport 通过。
>
> **▶ 字体栈增量进展（R3256-F historical-forms·2026-08-12）**：CSS `font-variant-alternates: historical-forms` 已完成 parse/computed/default/inherit/initial/known-property/CSSOM 全生命周期，并在 shaping feature precedence 中注入 OpenType `hist=1`；`font` 与 `font-variant` shorthand 均重置/展开 alternates，函数式 alternates 继续等待 `@font-feature-values` 独立切片。上游 `font-variant-alternates-02` 可信 strict `0.00%` 且已写入 `imported-tests.txt`；Chromium Oracle paired A/B `6.23%→6.20%`（改善 `0.03pp`），全 css-fonts 保持 `84/282`、无扩散回归；reftest `687/687`、workspace clippy、产品 smoke 全绿。同期 `src:local()` 的 Linux `Arial→Liberation Sans` 替代实验使目标页 `7.38%→7.84%`，已完整回退并沉淀平台经验。
>
> **▶ 字体栈增量进展（R3265-F font-kerning·2026-08-12）**：补齐 CSS `font-kerning: auto | normal | none` 的 computed/default/inherit/initial/registry、`font` shorthand 重置与 CSSOM 序列化；shaping 按 writing mode 注入横排 `kern` / 竖排 `vkrn`，`none` 同时禁用两者，显式 `font-feature-settings` 继续保持最高优先级。上游 `font-kerning-04` 与 `FontWithFancyFeatures.otf` 已资产化；目标 5 案 self-source 可信 strict `2/5→3/5`，04 从 `0.07%→0.00%`，Chromium Oracle 仍 `1/5`，全 css-fonts 保持 `84/282`、credible `74`、strict `54`，无目录级回归。reftest `687/687`、workspace default/QuickJS clippy、产品 smoke 全绿；`make test` 并发态仅既有 real HTTP fetch 时序失败（隔离及 browser 串行全绿），workspace 串行态仅本机无 wgpu adapter 的 compositor recovery 测试阻断。
>
> **▶ 字体栈增量进展（R3267-F synthetic-italic-ipc·2026-08-12）**：R3264 已让 GPU 消费 `GlyphPrimitive.synthetic_italic`，但 renderer IPC 未序列化该字段，browser/compositor 接收端与单进程 WebView transform 又固定写 `false`，导致生产端判定的合成斜体在进程/宿主边界静默丢失。本轮给 `IpcGlyph` 增加向后默认的 `synthetic_italic`，贯通 renderer export、browser/compositor restore 与 WebView transform，并在五个边界各加回归断言；`font-synthesis-style` 上游 WPT 已资产化。同期 synthetic bold 完整实验因 `font-synthesis-08 4.08%→4.23%`、禁用类仅各改善约 `0.01pp`，全 css-fonts 另有 `font-face-local-not-family +0.01pp`，按净负门禁完整撤回。验证：workspace default/QuickJS clippy、reftest `687/687`、产品 smoke desktop/375/320 与表单性能全绿；`make test` 仍受既有 real HTTP fetch 与多进程表单快照时序阻断。
>
> **▶ 字体栈增量进展（R3273-F font-feature-values·2026-08-12）**：完整实现 `@font-feature-values` 专用 AST/parser 与 `@stylistic`、`@styleset`、`@character-variant`、`@swash`、`@ornaments`、`@annotation` 六类 alias；StyleSystem 按 computed family、layer 声明顺序和源序合并 alias，生成 `salt/ssNN/cvNN/swsh+cswh/ornm/nalt`，`font-variant-alternates` 支持规范组合值、继承、shorthand 与 CSSOM，显式 `font-feature-settings` 仍最终覆盖。`ZW_FONT_FEATURE_VALUES=0` 可回滚。上游 03–19 与 layers 共 18 案已资产化；目标 20 案 self-source 可信 strict `2/20→20/20`，Chromium Oracle 17 个函数式案均改善约 `0.03–0.06pp`（swash-14 `6.92%→6.86%`、layers `6.87%→6.81%`），全 css-fonts `alternates-order 9.05%→8.94%`，目录计数保持 `84/282`、credible `74`、strict `54`，无关键扩散回归。验证：parser `2827/2827`、style `2161/2161`、engine `2019/2019`、workspace default/QuickJS clippy、reftest `687/687`、产品 smoke 与表单性能全绿；`make test` 仍仅受既有 real HTTP fetch 与多进程表单快照时序阻断。
>
> **▶ 字体栈增量进展（R3278-F table shaped intrinsic·2026-08-12）**：table auto layout 的直接文本 cell 现复用 IFC 字体 resolver 与 `AdvanceSource`，以真实 shaped max-content 取代逐字符启发式；many-to-one ligature 与 ZWNJ 等可忽略控制字符折叠 run 使用 paint 同源总 advance，复杂 offset mapping 仍 fail-closed。`ZW_TABLE_SHAPED_INTRINSIC=0` 可回滚。`font-feature-resolution-001/002` self-source `2.18%/2.81%→0.55%/1.36%`；Chromium Oracle `10.62%/12.91%→9.83%/12.32%`。全 css-fonts 282 案 A/B：2 改善、6 微退、274 持平，总 diff `-0.96pp`，目录 `84/282`、credible `74`、strict `54` 均保持。TableCell inherited 32px IFC 实验使 Oracle 恶化至 `13.07%/16.78%`，已完整撤回，后续须随 table paint ownership/line metrics 一并处理。验证：layout `1364/1364`、engine `2022/2022`、workspace default/QuickJS clippy、reftest `687/687`、产品 smoke 与表单性能门全绿；`make test` 两个并发时序失败均已隔离复现，real HTTP 串行通过，form fixture 重建独立 renderer 后通过。
>
> **▶ 文字排版增量进展（R3288-F plaintext line direction·2026-08-12）**：`unicode-bidi: plaintext` 不再于断行前重排整个 DOM text run；现先按逻辑顺序软换行，再按 `<br>` 分隔段落的首个 strong 字符决定基方向、逐 fragment 恢复视觉字符顺序，并在 `text-align:start` 时解析段落 start 边。paint Path B 通过容器 override 保留 plaintext 语义，显式 left/right 仅关闭自动 start 对齐，不恢复错误预重排。`ZW_PLAINTEXT_LINE_DIRECTION=0` 可回滚。上游 `bidi-plaintext-br-001` self-source `0.93%→0.88%`，Chromium Oracle `2.15%→2.04%`；writing-modes 81 个 Oracle 案 A/B 为 1 改善、0 恶化、80 持平，总 `-0.11pp`。剩余差异主要是 Arial/`10ch` 字体度量墙；跨多个 mixed-direction fragment 的完整逐行 UBA 仍有 FIXME。验证：layout `1365/1365`、engine `2022/2022`、workspace default/QuickJS clippy、reftest `687/687`、产品 desktop/375/320 smoke 与表单性能门全绿；`make test` 唯一 real HTTP 并发时序失败串行通过。
>
> **▶ 文字排版增量进展（R3289-F plaintext inline owner·2026-08-12）**：layout IFC 现将 `unicode-bidi:plaintext` inline owner 持久化到 `LayoutBox`，paint Path B 空 styles 重跑时按 owner 恢复；同 owner 的连续 plaintext fragments 在最终行内先按 identity source range 恢复折叠后的词间空格、合并逻辑文本，再执行一次 UBA，避免逐词独立重排或吞掉空白。`ZW_PLAINTEXT_LINE_DIRECTION=0` 同时回滚。相对 R3288，`bidi-plaintext-001/005/011` 各改善 `0.01pp`，增量净 `-0.03pp`；相对 gate 全关闭，writing-modes 81 案为 4 改善、0 恶化、77 持平，总 `-0.14pp`。上游 plaintext 12 案 self-source `12/12` 通过但均为 approximate；001–011 已登记常驻资产。验证：layout `1366/1366`、engine `2027/2027`、workspace default/QuickJS clippy、reftest `687/687`、产品 desktop/375/320 smoke 与表单性能门全绿；`make test` 唯一 real HTTP 并发时序失败串行通过。跨不同 style owner 的整行 UBA 与 L4 glyph mirroring 仍待后续切片。
>
> **▶ 文字排版增量进展（R3319-F bidi override mirroring·2026-08-12）**：容器级 `unicode-bidi:bidi-override` 现从 computed style 进入 layout IFC，并在 paint Path B 空 style map 重跑时恢复；RTL override 按指定方向直接生成视觉序，UAX #9 L4 镜像使用 `unicode-bidi-mirroring`，`visual_to_logical` 仍指向原始源码。`ZW_BIDI_OVERRIDE=0` 回滚 override，`ZW_BIDI_MIRRORING=0` 单独关闭 L4。专用 WPT `bidi-glyph-mirroring-001` self-source `0.07%→0.06%`，002 保持 `0.10%`，两案均 approximate；writing-modes Chromium Oracle 81 案全持平、rounded `0.00pp`。全局恢复 RTL Path B 实验为 2 改善/15 回归/65 持平、`+0.16pp`，normal UBA 同开 L4 的收窄实验仍为 `+0.04pp`，均已撤回；`isolate-override` 因 isolation boundary 尚未建模不在本切片宣称范围。验证：layout `1367/1367`、engine `2033/2033`、workspace default/QuickJS clippy、reftest `687/687`、产品 desktop/375/320 smoke 与表单性能门全绿；`make test` 唯一 real HTTP 并发时序失败串行通过。
>
> **▶ 字体栈增量进展（R3325-F @font-face unicode-range·2026-08-12）**：CSS tokenizer 现生成结构化 `UnicodeRange` token，`@font-face` descriptor 支持单值、闭区间和 `?` wildcard；闭区间随 engine/WebView/browser/renderer/WPT 字体加载 metadata 到达 FontLoader。face 选择、advance 与 legacy raster fallback 均先检查声明范围，注册时清 shaping cache；`ZW_FONT_UNICODE_RANGE=0` 回滚。Chromium Oracle css-fonts 为 5 改善、0 回归、278 持平，rounded 总差异 `-2.89pp`：`size-adjust-02 5.45%→4.41%`、`font-face-unicode-range-nbsp 2.41%→1.27%`、`size-adjust-01 7.88%→7.37%`、`font-face-unicode-range-2 5.31%→5.19%`、`size-adjust.tentative 10.87%→10.79%`；目录通过数仍 `84/282`。3 个 driving WPT 与 4 个字体资源已资产化，self-source 均为 approximate，故收益以 Chromium Oracle 为准。验证：parser `2828/2828`、FontLoader `46/46`、WebView `579+17`、engine `2043/2043`、workspace default/QuickJS clippy、reftest `687/687`、产品 desktop/375/320 smoke 与表单性能门全绿；`make test` 唯一 real HTTP 并发时序失败串行通过。当前 resolver 每 CSS family 仍只暴露一个 matched face，同 family 多个 disjoint unicode-range face 留后续切片。
>
> **▶ 字体栈增量进展（R3326-F same-family unicode-range faces·2026-08-13）**：FontLoader 只为显式 `@font-face` alias 发布有序 `:face=N` 键；`resolve_font_faces()` 按既有 stretch→weight/style 规则返回同一变体全部 face，layout/paint ordered shaping 按声明顺序消费。系统字体不发布列表，alias 第二 face 不再被误标为 bold。真实 Lato 双 face 测试证明同 family `"Aa"` 按 `U+41-5A` / `U+61-7A` 分别落不同 font ID；`ZW_FONT_UNICODE_RANGE_FACES=0` 回滚。现有 WPT 无同 family 分片案，故采用真实字体机制测试；既有 unicode-range 2 案保持。css-fonts Chromium Oracle 283 案全部持平，rounded `0.00pp`。验证：FontLoader `47/47`、layout `1368/1368`、workspace default/QuickJS clippy、reftest `687/687`、产品 smoke/perf 全绿；`make test` 唯一 real HTTP 并发时序失败串行通过。
>
> **▶ 字体栈增量进展（R3327-F first available font metrics·2026-08-13）**：`ex` 不再于 parser 阶段降级为固定 `0.8em`；FontLoader metric map 按 CSS Fonts first available font 规则跳过不能匹配 U+0020 的 `@font-face`，向 style-system 提供真实 x-height 与 `0` advance aspect。`ch/ex` 按 CSS family 顺序取首个可用 face，并使用 `font-size-adjust` 后的 used-font scale；`em` 仍绑定 computed font-size。`ZW_FIRST_AVAILABLE_FONT_METRICS=0` 回滚旧常量路径，`ZW_PERFONT_LINEHEIGHT` 仍独立默认关闭。css-fonts Chromium Oracle A/B 为 3 改善、1 回归、279 持平，rounded 总差异 `-8.91pp`，通过数 `84→86/282`：first-available-font-001 `4.42%→0.65%`、002 `1.85%→0.63%`、font-size-adjust-units-001 `5.34%→1.07%`，唯一回归 order-001 `16.38%→16.73%`。3 个 driving WPT 与 Revalia/AD 字体已资产化。验证：parser `2828/2828`、style `2163/2163`、FontLoader font tests `125/125`、engine `2043/2043`、layout `1368/1368`、WebView `579+17`、WPT runner `150/150`（10 ignored）、workspace default/QuickJS clippy、reftest `687/687`、产品 smoke/perf 全绿；`make test` 唯一 real HTTP 并发时序失败串行通过，本机全 render-foundation GPU suite 因无 wgpu adapter 不在默认门禁口径。
>
> **▶ 字体栈增量进展（R3328-F glyph-derived x-height + adjusted normal line·2026-08-13）**：FontLoader 在旧 OS/2 表缺 `sxHeight` 时从 `x` glyph bbox `yMax / unitsPerEm` 推导 aspect，避免 shaping fail-closed 为 computed size；`FontMetricProvider` 始终暴露 font ID 与 ex/ch aspect，但 ascent/descent/gap 仍受 `ZW_PERFONT_LINEHEIGHT=1` 控制。非 Ahem 的 `font-size-adjust + line-height:normal` 保持 TextRun specified size供逐 face shaping，只用 primary used size计算 normal 行高，修复 glyph 放大但行盒仍 20px 的重叠。`ZW_FONT_METRIC_GLYPH_FALLBACK=0` 与 `ZW_FONT_SIZE_ADJUST_NORMAL_LINE=0` 回滚本切片。css-fonts 严格 A/B 为 3 改善、1 回归、279 持平，rounded `-10.33pp`，pass 保持 `86/282`：font-size-adjust-order-001 `16.73%→6.35%`、size-adjust-01 `7.37%→7.32%`、first-available-001 `0.65%→0.64%`，唯一回归 size-adjust-02 `4.41%→4.52%`。order-001 已资产化。验证：engine `2047/2047`、layout `1370/1370`、font tests `126/126`、workspace default/QuickJS clippy、reftest `687/687`、产品 smoke/perf 全绿且 welcome `15.90%` 不变；`make test` 受既有 real-HTTP 及远端新 multiprocess form tests 的 60s 并发时序超时中止，受影响模块与渲染门禁独立全绿。
>
> **▶ 字体栈增量进展（R3329-F adjusted generic shaped advance·2026-08-13）**：generic family 的 legacy contextual paint 兼容路径原本无条件把 HarfBuzz advance 改写为逐字符 paint width；R3328 后 layout 对活跃 `font-size-adjust` 已使用 shaped advance，导致 `quick` 等 fragment 的 layout/shaping 为 `110.089px`、paint 仅消费 `97px`，同一 monospace face 内字符位置失真。现仅对 generic + active adjustment 使用 shaped advance，普通 generic 文本保持 legacy 策略；`ZW_SHAPED_ADJUSTED_GENERIC_ADVANCE=0` 回滚。trace 现逐 fragment 在约 `1e-5px` 内闭合。css-fonts 严格 A/B 为 3 改善、1 回归、279 持平，rounded `-1.17pp`，pass 保持 `86/282`：size-adjust-03 `16.58%→15.46%`、order-001 `6.35%→6.33%`、font-size-adjust-014 `3.75%→3.71%`，size-adjust-02 微退 `4.52%→4.53%`。size-adjust-03 已资产化。验证：engine `2048/2048`、workspace clippy、reftest `687/687`、产品 smoke/perf 全绿且 welcome `15.90%` 不变；`make test` 未重复运行，沿用 R3328 已记录的 real-HTTP / multiprocess form 并发超时限制。

> **▶ 字体栈增量进展（R3330-F @font-face size-adjust descriptor·2026-08-13）**：CSS parser 现结构化保存 `@font-face size-adjust` 百分比，engine/WebView/browser/renderer/WPT 加载链将 face scale 注册到 FontLoader；无 `font-size-adjust` property 时，shaping、layout advance、paint advance 与 raster 统一消费 descriptor used size，property 活跃时按 CSS Fonts 5 precedence 完全覆盖 descriptor。首版只放大 glyph 曾使 `size-adjust-03 15.46%→16.03%`，trace 暴露 60px glyph 仍落在 40px legacy fragment（`Quick` layout `112px`、paint `170px`）；最终以 glyph used size 偏离 specified size作为 absolute shaped advance 条件，三方闭合。`ZW_FONT_FACE_SIZE_ADJUST=0` 回滚。css-fonts Oracle A/B 为 2 改善、0 回归、280 持平，rounded `-2.33pp`，pass/credible/strict 保持 `86/76/54`：size-adjust-03 `15.46%→13.55%`、size-adjust.tentative `10.79%→10.37%`。验证：parser `2829`、engine `2049`、WebView `579+17`、WPT runner `151`（10 ignored）、font `127`、workspace clippy、reftest `687/687`、产品 smoke/perf 全绿且 welcome `15.90%` 不变；`make test` 的 QuickJS clippy 通过，V8 workspace 仍仅 8 个既有 real-HTTP / multiprocess form renderer-ready 时序失败。

> **▶ 字体栈增量进展（R3337-F size-adjust normal line metrics·2026-08-13）**：FontLoader metric map 现携带 first available face 的 descriptor scale；`FontMetricProvider` 仅为非 100% `size-adjust` face 暴露缩放后的真实 ascent/descent/line-gap，`line-height:normal` 在 `font-size-adjust:none` 时消费该度量。普通字体仍遵守 `ZW_PERFONT_LINEHEIGHT` 默认关闭，不重开已证净负的全局 hhea 路径；unitless/length line-height 与 computed `font-size` 不变，property 活跃时继续覆盖 descriptor。`ZW_FONT_FACE_SIZE_ADJUST_NORMAL_LINE=0` 回滚。css-fonts Oracle A/B 为 1 改善、0 回归、281 持平，rounded `-1.92pp`，pass/credible/strict 保持 `86/76/54`：size-adjust.tentative `10.37%→8.45%`；上游 self-source 同向 `7.58%→6.14%`。用例已通过标准 importer 常驻。验证：layout `1372`、style `2163`、engine `2049`、WebView `581+17`、WPT runner `161`（10 ignored）、font `128`、workspace clippy、reftest `687/687`、产品 smoke/perf 全绿且 welcome `15.90%` 不变；`make test` QuickJS clippy 绿，V8 browser `284/8/1` 仍仅相同 real-HTTP / multiprocess renderer-ready 时序失败。

> **▶ 字体栈增量进展（R3341-F size-adjust relative metrics·2026-08-13）**：engine 向 style-system 的 first-available `FontRelativeMetrics` 现保留 descriptor scale；computed 阶段在 `font-size-adjust:none` 时只对 `ex/ch` 应用 `size-adjust`，`em` 与 computed font-size 保持不变。任何 `font-size-adjust` property 值继续完全覆盖 descriptor，避免双重缩放；`ZW_FONT_FACE_SIZE_ADJUST_RELATIVE_UNITS=0` 回滚。css-fonts Oracle A/B 为 1 改善、0 回归、281 持平，rounded `-0.04pp`，pass/credible/strict 保持 `86/76/54`：size-adjust.tentative `8.45%→8.41%`；上游 self-source 同向 `6.14%→6.10%`。复用 R3337 已导入的常驻 WPT。验证：style `2164`、engine `2049`、WebView `583+17`、WPT runner `161`（10 ignored）、workspace clippy、reftest `687/687`、产品 smoke/perf 全绿且 welcome `15.90%` 不变；`make test` QuickJS clippy 绿，V8 browser `284/8/1` 仍仅相同 real-HTTP / multiprocess renderer-ready 时序失败。

> **▶ Oracle 基础设施进展（R3344-F webfont-ready capture·2026-08-13）**：对 css-fonts 当前 top-worst 做图像审计后，确认 font-size-adjust-012/013 的 ZeroWeb self-source 均已通过 `0.94%`，且本地 AhemEx250/500 资源与 alias 加载有效；旧 Chromium oracle 却显示 fallback X/A 而非 AhemEx 方块，`13.58%/16.24%` 因此是损坏 oracle，不是渲染逻辑回归。根因是两条 DC-14 捕获脚本只等待 `networkidle0` 与 `<img>`，未等待 CSS Font Loading 的 face swap。`capture-oracle-per-dir.mjs` 与 `chromium-oracle-shot.mjs` 现以 2 秒 bounded `document.fonts.ready` 等待后再截图，坏字体不会阻塞批量任务。验证：两脚本 `node --check`、CLI help 与 `goto→images→fonts→screenshot` 静态顺序检查通过。当前环境没有 Chromium 可执行文件，012/013 及同类 webfont oracle 尚待可用捕获环境刷新；刷新前不得把这些旧 shot 用作收益裁决。

> **🔬 候选裁决（R3345-F table text metadata / HTML UA defaults·2026-08-13）**：资源完整的 font-feature-resolution-001/002 图像显示 table cell 几何按继承的 32px 布局，但 cell 直接文本因 `text_node_font_sizes` 为空在 paint IFC 回退 16px；最小 style 测试确认 table→tbody→tr→td 继承正确，缺口位于 final IFC 的 TableCell eligibility。实验把 TableCell 加入 metadata 收集后语义转正，但 Chromium Oracle 反而 001 `9.83%→13.07%`、002 `12.32%→16.78%`，因放大 glyph 暴露 feature/raster 与 cell geometry 的既有偏差。继续补 HTML UA `table{border-spacing:2px}` 与 `td/th{padding:1px}` 后，css-fonts table 簇改善（UA-only：001 `9.83%→8.87%`、002 `12.32%→12.12%`，font-weight 三案亦各约 `-1.1pp`），但 css-tables self-source baseline `105/115`：padding-only `103/115`、spacing-only `104/115`、组合 `103/115`。三候选均跨目录净负，已全部回退，工作树零源码残留。结论：cell paint metadata、feature/raster 与 UA geometry 必须作为协同切片解决；禁止单独重开任一开关以追 css-fonts 排名。
>
> **▶ WPT 资产进展（R3346-F ic-height fixed webfont·2026-08-13）**：用 `ORACLE_DUMP_ALL=1` 展开 css-fonts 全候选后，排除了 Chromium 不支持 `font-synthesis-position` 的旧 shot、平台 `local()` family、待重抓 webfont、table 双路径及 generic raster 簇。`font-size-adjust-ic-height` 的 test/ref 已导入，却遗漏其固定 `NotoSansCJKjp-Regular-subset-chws.otf`，ZeroWeb 因 test/reference 分别落不同 fallback 路径形成 self-source 假绿。标准 importer 现将该字体写入 `imported-resources.txt` 并保留测试账本；Chromium Oracle 单案 `2.85%→2.44%`，完整 css-fonts 1 改善、0 回归、281 持平，pass/credible/strict 保持 `86/76/54`。self-source `2.60%→4.60%` 反向变差，进一步证明资源缺失时不得以 test-vs-ref 单独裁决。该切片零引擎源码变更。
>
> **▶ Oracle 可靠性进展（R3373-F shaping cache descriptor isolation·2026-08-13）**：共享 shaping cache 的内容寻址 key 原只含字体字节 hash，遗漏 `@font-face size-adjust`；同字节 Ahem face 在不同 loader 中带 150%/默认 descriptor 时发生缓存碰撞，使完整 css-fonts `--jobs 8` 的 `size-adjust-03` 假降至 `16.65%`，单案与 `--jobs 1` 则为 `18.37%`。key 现同时包含每个 face 的 descriptor scale，回归测试用共享 cache 的双 loader 锁定 24px/16px glyph size。修复后 css-fonts `--jobs 1/8` 数值完全一致：`size-adjust-03 18.37%`、`size-adjust-01 11.65%`，pass/credible/strict 保持 `86/76/54`；旧并行数值属于虚假改善，不再作为历史收益基线。同期将上游 `font-synthesis-weight-webfont-bold` 与固定 `Lato-Bold.ttf` 纳入账本，资源独立 Oracle `1.10%→1.08%`，self-source 均为 strict `0.00%`。
>
> **▶ WPT 资产进展（R3374-F separator fixed webfont·2026-08-13）**：css-fonts `url()` 资源完整性扫描发现 `separators` 缺少固定 `separator-test-font.ttf`，导致自定义字体断言走 fallback。标准 importer 补齐 test/ref 与资源账本后，Chromium Oracle `0.74%→0.71%`，self-source 保持可信 strict `0.00%`，完整 css-fonts pass/credible/strict 保持 `86/76/54`，无回归。同期 `IcTestFullWidth.woff2` 因现有 FontLoader 不支持 WOFF2 而无像素变化，`Exo-DemiBold.otf` 使目标 Oracle `2.72%→2.95%`，两项均按门禁回退。
>
> **▶ 字体栈增量进展（R3375-F WOFF2 decoding·2026-08-13）**：引入 pure-Rust、MIT、MSRV 1.85 的 `wuff 0.2.8`，FontLoader 统一支持 WOFF2→sfnt 解码，`ZW_WOFF2=0` 可回滚；真实 `IcTestFullWidth.woff2` 测试覆盖解码及原始 FontLoader 入口。导入 `NotoNaskhArabic-regular.woff2` 与 `shaping-001` 后，Arabic shaping 28 案为 20 改善/0 回归/8 持平，总 `-9.01pp`、可信 strict `4→7`；全 css-text 1826 案为 37 改善/5 微退/1784 持平，总 `-9.98pp`、strict `574→581`，pass 保持 `1762/1826`。解码扩大字体 surface 后暴露 indexed glyph 越界 panic，现于共享 raster 入口按 sfnt `numGlyphs` 校验并 fail-closed。css-fonts Chromium Oracle 282 案逐像素持平；Arabic shaping 尚无 Chromium shot。
>
> **▶ WPT 资产进展（R3380-F Naskh shaping corpus·2026-08-13）**：按当前 HEAD 复验 `ZW_WOFF2=0/1`，将 R3375 的全部 20 个改善案通过标准 importer 纳入常驻账本，test/ref 路径完整且 test key 20/20 唯一；A/B 保持 20 改善/0 回归/8 持平、总 `-9.01pp`、可信 strict `4→7`，其中 `shaping-009/010/011` 达 strict `0.00%`。NKo 固定 WOFF2 试验使 `020/021/022` 从 `1.31/1.13/1.05%` 退至 `1.35/1.34/1.29%`，已完整回退；其跨 inline boundary shaping 以及 Mongolian vertical corpus 等 Phase A/vertical 路径闭合后再导入。
>
> **▶ 文本度量进展（R3381-F Unicode NSM zero advance·2026-08-13）**：layout estimator 过去把 Arabic nonspacing marks U+0654/U+0670 各算 `0.5em`，令真实 shaping 仅 `26.48px` 的 `NBSP + marks` fragment 虚增至 `180px`。现用既有 `unicode-bidi` 的 `BidiClass::NSM` 将 nonspacing mark 独立 advance 置 0，shaper仍负责 glyph offset；`ZW_ZERO_WIDTH_NSM=0` 回滚。目标 `shaping-arabic-diacritics-002` `7.53%→6.21%`；全 css-text 21 改善/10 微退/1795 持平，总 `-2.64pp`，pass 保持 `1762/1826`、可信 strict `581→582`，无 pass flip。当前 css-text 无 Chromium shot；复用 R3380 常驻 WPT并新增 estimator 单测。
>
> **▶ WPT 资产进展（R3382-F NSM line-break strict flip·2026-08-13）**：将 R3381 唯一新增 strict flip `line-break-anywhere-overrides-uax-behavior-011` 经标准 importer 常驻；默认 `0.00%`，`ZW_ZERO_WIDTH_NSM=0` 为 `0.52%`，test/ref 无外部字体依赖。其余改善案已跟踪、属于 vertical，或仅微量变化，未扩成低信号 CI 集。Arabic driving case 的剩余差异已定位为 inline bidi 层未消费 `unicode-bidi:isolate`，受 Phase A 护栏约束。
>
> **🔬 候选裁决（R3383-F full-width space / inline owner transform·2026-08-13）**：审计确认 `text-transform:full-width` 的共享转换未把 U+0020 映射到 U+3000，且 inline 元素收集分支未在行断前应用自身 `text-transform`。实验补齐两点后，全 css-text self-source pass 保持 `1762/1826`、可信 strict `582→592`；但 driving `text-transform-fullwidth-009` 对 reference 与 Chromium Oracle 均精确持平（`5.19%` / `6.64%`）。fresh Chrome 127 重抓 `css-text/text-transform` 后，107 案 Oracle 为 4 改善/9 回归/94 持平，总差异 `+0.24pp`，pass/credible/strict 均保持 `7/5/4`；最大回归 `capitalize-016 +0.14pp`。按 Oracle net≥0 门禁，源码与测试已完整回退。结论：U+0020 phase-2 whitespace 处理和 inline transform ownership 不能作为两个局部补丁合并启用；须等 inline style ownership/whitespace phase 协同闭合，禁止重试该单点组合。
>
> **▶ 字体加载进展（R3384-F data URI webfont·2026-08-13）**：复用 render-foundation 既有 data URI parser 抽出公共字节解码，base64/percent payload 可供图片与字体共同消费；WebView async loader 与 WPT loader 现直接加载 `@font-face src:data:`，不发网络请求，并保留 family/weight/style/stretch/size-adjust/feature/unicode-range metadata。WPT fresh-loader cache key 纳入 data source，避免不同内联字体碰撞；`ZW_DATA_FONT=0` 回滚。同步补齐 Chrome 127 全 css-fonts 284 张 oracle（0 失败），runner 当前 282 个可比案基线为 pass/credible/strict `87/77/54`，纠正 R3344 的旧 fallback shot。串行 css-fonts A/B 为 2 改善/0 回归/280 持平，总 `-0.09pp`：`font-synthesis-style-binary 3.60%→3.52%`、`font-synthesis-weight-binary 3.31%→3.30%`；feature descriptor 案 `3.42%` 持平。self-source pass 保持 `280/287`、可信 strict `152→155`。三个 binary WPT 已常驻。验证：WebView `594+17`、WPT runner `165`（10 ignored）、workspace default/QuickJS clippy、`make test`、reftest `687/687`、产品 smoke/perf 全绿；render-foundation 非 GPU 测试通过，完整 GPU suite 仍因本机无 wgpu adapter 不可运行。
>
> **▶ 字体度量进展（R3385-F rex root x-height·2026-08-13）**：CSS Values 4 `rex` 现以 typed `LengthValue` 保留至 computed/calc 阶段；StyleSystem 在根元素完成后缓存 root first-available font 的 used x-height，并用独立 `FontRelativeContext` 同时传 current/parent/root 度量。`font-size` 中 `ex/ch` 现按规范读取父元素字体，普通 `ex/ch` 仍读当前元素字体，`rex` 始终读根字体且支持 `calc()`；`ZW_ROOT_FONT_UNITS=0` 回滚。`rex-in-monospace` self-source `0.22%→0.00%`（strict flip），fresh Chromium Oracle `1.33%→1.16%`。串行 css-fonts 282 案仅该用例改变：1 改善/0 回归/281 持平，总 `896.98→896.81pp`（净 `-0.17pp`）；pass/credible/strict 保持 `87/77/54`。self-source pass `280/287` 保持、可信 strict `150→151`。test/ref 与 `ExTest.woff` 已常驻。验证：parser `2843`、style `2172`、layout `1373`、workspace default/QuickJS clippy、`make test`、reftest `687/687`、产品 smoke/perf 全绿。
>
> **▶ 字体度量进展（R3386-F rch root zero advance·2026-08-13）**：沿用 R3385 的 root metric 生命周期，新增 typed `LengthValue::Rch`、`CalcContext.root_ch_width` 与 `FontRelativeContext.root_ch_width`；StyleSystem 在根元素完成后缓存 root first-available font 的 used U+0030 advance，`font-size`、`line-height`、普通 length 与 `calc()` 均从同一根字体度量解析 `rch`，不受当前/祖先 family 或 font-size 影响。`ZW_ROOT_FONT_UNITS=0` 同时回滚 `rex/rch`。`rch-in-monospace` self-source `0.15%→0.00%`（strict flip），fresh Chromium Oracle `0.92%→0.84%`。串行 css-fonts Oracle 282 案中，除同一 kill-switch 下既有 `rex` 改善外，仅 `rch` 改善 `0.08pp`、零回归；pass/credible/strict 保持 `87/77/54`。self-source pass 保持 `280/287`、可信 strict `150→152`。test/ref 已通过标准 importer 常驻。`rcap/ric/rlh` 分别需要 root cap-height、ideographic advance 与 line-height provider，不并入本切片。
>
> **▶ 字体度量进展（R3387-F cap/rcap root cap-height·2026-08-13）**：CSS Values 4 `cap/rcap` 现以 typed `LengthValue` 保留至 computed/calc 阶段；FontLoader 将既有 OS/2 cap-height provider 纳入 first-available family metric map，StyleSystem 分别按当前/父/root 字体 ownership 解析普通长度、`font-size` 与 root unit，并缓存 root used cap-height。`ZW_ROOT_CAP_UNITS=0` 独立回滚本切片，不影响已落地的 `rex/rch`。`rcap-in-monospace` self-source `0.15%→0.00%`（strict flip），fresh Chromium Oracle `0.92%→0.84%`。串行 css-fonts Oracle 282 案仅该用例改善 `0.08pp`、0 回归；pass/credible/strict 保持 `87/77/54`。self-source pass `280/287` 与 7 个既有 mismatch 不变，可信 strict `152→153`。test/ref 已通过标准 importer 常驻。`ric/rlh` 继续作为独立 provider/lifecycle 切片。
> **Post-rebase 复验**：合入远端 form-control 渲染提交后，Oracle 差分仍仅 `rcap -0.08pp`；self-source 当前组合态为可信 strict `153→154`，pass `280/287` 与 7 mismatch 不变。
>
> **🛡️ 组合态回归修复（R3388-F non-Ahem CJK contiguous gate·2026-08-13）**：远端 form-control 提交把 CJK/SEA per-char 断行的无空格模式从 Ahem 专用全局启用，重开了代码注释已记录的 advance-wall 回归。现默认恢复 Ahem-only，普通字体新语义保留为 `ZW_CJK_CONTIGUOUS=1` 显式实验。A/B：welcome `15.96%→22.70%`（实验开启净退 `+6.74pp`）；全 css-text self-source pass 均为 `1762/1826`，但可信 strict `580→466`（-114），且出现 2 个 pass→fail。故默认 gate-off 是产品硬门与规范集共同裁决，不是单纯回退偏好。
>
> **▶ 字体度量进展（R3389-F ic/ric root ideographic advance·2026-08-13）**：CSS Values 4 `ic/ric` 现保留 typed 单位至 computed/calc；FontLoader 将既有 U+6C34 horizontal advance provider纳入 first-available family metric map，StyleSystem 按 current/parent/root ownership 解析普通长度、`font-size` 与 root unit，并缓存 root used ideographic advance。缺 U+6C34 时沿用规范 `1em` fallback；`ZW_ROOT_IC_UNITS=0` 独立回滚。`ric-in-monospace` self-source `0.15%→0.03%`，Chromium Oracle `0.94%→0.92%`。串行 css-fonts Oracle 282 案仅该用例改善 `0.02pp`、0 回归；pass/credible/strict 保持 `87/77/54`，self-source pass `280/287`、可信 strict `154` 与 7 mismatch 均不变。既有 WPT/WOFF2 资产账本归属更新为 R3375/R3389-F。`rlh` 仍需独立 root line-height lifecycle。
>
> **🧪 净负实验裁决（R3390-F lh/rlh used line-height·2026-08-13）**：审计并试作 typed `lh/rlh`、parent/current/root line-height context 与 root lifecycle，随后完整回退源码。CSS Values 4 要求 `lh/rlh` 用于 `font-size`、`line-height` 等 font-affecting 属性时按父元素 computed metrics 消解循环；fresh Chrome 127 probe 在 `rlh-in-monospace` 中给出 body 子级 `19px`、monospace 祖先内子级 `23.3846px`。当前 ZeroWeb 默认 `normal=1.164em` 只能得到近似值，且 layout per-font provider 默认关闭；root-fixed 与 parent-computed 两种候选均使 fresh Chromium Oracle `0.96%→0.98%`（净退 `0.02pp`）。root-fixed 虽令 self-source `0.15%→0.00%`，但与 Chromium 循环语义不符；parent-computed self-source为 `0.22%`。故不提交半正确实现，等待 root/parent used `line-height:normal` 与 Chromium 同源后再恢复。经验见 [`lh-rlh-needs-used-line-height-provider.md`](../learnings/bugs/lh-rlh-needs-used-line-height-provider.md)。
>
> **🧪 净负实验裁决（R3397-F font-language-override·2026-08-13）**：完整试作 `font-language-override:normal|"<tag>"` 的 parse/computed/inherit/shorthand/CSSOM、大小写敏感 OpenType language tag、shaping callback/cache 与 browser/renderer/WPT 三宿主桥接，随后完整回退源码。真实 Libertine WOFF 机制测试证明默认 `fi` 为 1 glyph、`"TRK"` 为 2 glyph、规范不生效的 `"trk"` 仍为 1 glyph；treatment PNG 与 `font-feature-settings:"liga" 0` reference 字节完全一致。fresh Chromium 串行 A/B 却为 01 `0.78%→0.79%`、03 保持 `0.74%`，两案聚合净退 `0.01pp`。说明 language-system 选择已正确，剩余差异在分离 glyph 的定位/advance/光栅路径；不以 self-reference 覆盖 Oracle 门禁。经验见 [`font-language-override-needs-chromium-glyph-positioning.md`](../learnings/bugs/font-language-override-needs-chromium-glyph-positioning.md)。
>
> **▶ WPT 资产进展（R3399-F Noto Sans JP variable default instance·2026-08-13）**：资源完整性审计发现 `font-weight-normal-variable` 的 test/ref 已在 full corpus，但固定 `NotoSansJP.subset.ttf` 未登记，fresh checkout 会让两侧同走 fallback。现通过标准 importer 将 test/ref 与字体写入常驻账本；同一 HEAD 移走/恢复字体的 fresh Chromium 串行 A/B 为 `0.36%→0.35%`，完整 css-fonts 保持 pass/credible/strict `87/77/54`，self-source 为可信 strict `0.00%`。该用例 test 的 `font-weight:normal` 与 reference 的 `"wght" 400` 均使用 variable font 默认实例，本切片不宣称 variable-axis plumbing，仅保证固定测试资产完整。
>
> **▶ WPT 资产进展（R3403-F RobotoExtremo variable capability fixture·2026-08-13）**：资源完整性审计继续发现 `synthetic-bold-out-of-capabilities-range` 的 test/ref 已存在，但固定 `RobotoExtremo-VF-wght-400-500.subset.ttf` 未登记，导致两侧同走 fallback。标准 importer 现登记 test/ref/font；同一 HEAD、同一 release runner 的缺资源/有资源 Chromium 串行 A/B 为 `0.50%→0.48%`，完整 css-fonts pass/credible/strict 保持 `87/77/54`，self-source 为可信 strict `0.00%`。字体只由该 test/ref 引用；本切片仅保证 variable capability fixture 可再生，不宣称 synthetic bold 或 variation-axis 实现。
>
> **🧪 净负实验裁决（R3405-F dynamic font-variant fixture·2026-08-13）**：审计发现 `font-variant-{caps,east-asian,ligatures,numeric,position}` 共用的两段 JS generator 与 `gsubtest-lookup3.otf` 缺失，旧 Chrome/ZeroWeb 均渲染近空页而形成 `0.00%` 假绿。试作补齐完整资源闭包并 fresh 重抓 5 张 Chrome Oracle；同一 fresh Oracle 下，缺资源 baseline 为 `1.27/2.54/4.94/3.80/0.46%`（合计 `13.01pp`），资源完整 treatment 为 `2.82/5.55/10.50/8.27/0.95%`（合计 `28.09pp`），5/5 回归、净退 `15.08pp`。完整 self-source 中 ligatures `7.08%`、numeric `5.52%` 亦直接 mismatch。故资源/账本完整回退，不把动态 fixture 资产化；fresh Oracle 本地保留，等待 DOM generator、GSUB shaping 与字体路径协同闭合。经验见 [`dynamic-wpt-fixtures-need-fresh-oracle-before-import.md`](../learnings/bugs/dynamic-wpt-fixtures-need-fresh-oracle-before-import.md)。
>
> **🧪 净负实验裁决（R3406-F Inter variation descriptor fixture·2026-08-13）**：`font-variation-settings-descriptor-01/02` 共用的 `Inter.var.subset.ttf` 缺失；试作补齐字体并 fresh 重抓两张 Chrome Oracle。Oracle 百分比 off/on 均四舍五入为 `0.78/0.77%`，但直接比较 ZeroWeb PNG 与 fresh Oracle 的精确像素后，01 为 `2157→2177`（+20 px）、02 为 `2137→2158`（+21 px），合计净退 41 px。原因是字体改变了默认 face，而 ZeroWeb 目前只有 `@supports` 声称支持 `font-variation-settings`，未把 `slnt/wght` axis 贯通 computed/shaping；self-source `0.00%` 是 test/ref 同时忽略 axis 的假绿。故 Inter 资源与账本完整回退，fresh Oracle 本地保留，待 variation-axis contract 闭合后再导入。
>
> **🧪 净负实验裁决（R3407-F Deseret case-mapping fixture·2026-08-13）**：从 css-text 固定字体缺口中选取不依赖 vertical/inline boundary 的 Deseret `capitalize/uppercase/lowercase` 三案；Rust Unicode scalar case mapping 已使资源完整 self-source 三案均 strict `0.00%`。补齐 `NotoSansDeseret-Regular.ttf` 并 fresh 重抓 Chrome 后，以同一 fresh Oracle 做资源 off/on A/B：baseline `6.99/3.87/3.47%`（总 `14.33pp`），treatment `9.75/5.08/4.45%`（总 `19.28pp`），3/3 回归、净退 `4.95pp`。说明 case mapping 正确，但固定字体暴露 layout/paint glyph geometry 与 Chrome 的差异；资源与账本完整回退，fresh Oracle 本地保留。
>
> **▶ 字体回退进展（R3408-F author font fallback·2026-08-13）**：R3243 为止住普通 CJK 长文多 face shaping 性能回归，将 `ZW_SHAPED_FALLBACK` 改为显式 opt-in，却同时截断了显式 `@font-face` family 列表并关闭其 `font-size-adjust` used size。`font-size-adjust-013` 运行帧因此把 primary 缺失的 `A` 落到默认字体，而非 secondary AhemEx。现以 `ZW_AUTHOR_FONT_FALLBACK=0` 为 kill-switch，仅当解析到多个非 generic author face 时保留有序 fallback 并恢复 adjustment；generic/system 文本仍维持单 face 快路径。fresh Chromium 串行 css-fonts 282 案 A/B 为 7 改善、0 回归、275 持平，总差异 `909.51→905.07pp`（净改善 `4.44pp`）：`size-adjust-02 9.87%→6.20%`、`font-size-adjust-013 17.19%→17.01%`、009/010/011 各 `10.37%→10.25%`、012 `14.09%→13.98%`、`font-face-unicode-range-2 5.18%→5.06%`；fresh corpus pass/credible/strict 保持 `83/78/50`。`font-size-adjust-013` self-source 保持 approximate `0.94%` 并已常驻账本；剩余第三组缺图属于既有 inline-block ownership，不在本切片扩修。经验见 [`author-font-fallback-must-not-share-generic-perf-gate.md`](../learnings/bugs/author-font-fallback-must-not-share-generic-perf-gate.md)。
>
> **▶ Oracle 可靠性进展（R3409-F shaping cache loader/descriptor isolation·2026-08-13）**：共享 shaping cache 的内容寻址 key 已包含字体字节、face index 与 `size-adjust`，但遗漏 `unicode-range` 和 loader-local `font_id`。前者会改变 fallback 选择；后者直接存于 cached `ShapedGlyph`，不同 loader 若以不同顺序加载相同字体，内容 key 相同却会把旧 loader 的 ID 解释为另一张字体。初版只补 range 后，最新组合态仍使 `font-stretch-01..05` 在 `--jobs 1/8` 间漂移 `2.42%↔2.79%`。最终每个 face key 同时包含 local ID 与 range；两组共享 cache 回归测试分别锁定相反 range 的 fallback ID 和相反加载顺序的 local ID。fresh css-fonts Oracle `--jobs 1/8` 的 282 个逐案百分比完全一致，汇总均为 pass/credible/near `83/78/33`；本切片不改变正确串行像素，不宣称兼容率收益。经验见 [`shaping-cache-font-face-metadata.md`](../learnings/bugs/shaping-cache-font-face-metadata.md)。
>
> **▶ 字体回退进展（R3410-F per-face feature descriptor·2026-08-14）**：ordered fallback shaping 过去只在 FontLoader 入口合并 primary face 的 `@font-face font-feature-settings`，随后把同一 feature vector用于所有 run；secondary descriptor 被忽略，primary descriptor则错误泄漏。现按 CSS Fonts 4 precedence为每个 resolved face独立合并 descriptor，再由 caller feature覆盖；resolved vector进入每个 face 的 cache key，descriptor 注册同步清 cache。共享 cache 双 loader 机制测试以相同 Lato 字节、互斥 `unicode-range`、secondary `liga=0/1` 证明 descriptor 分别产出 2/1 glyph，caller `liga=1` 可重新启用 ligature。`ZW_PER_FACE_FEATURES=0` 回滚。fresh css-fonts Oracle 282 案 A/B 全部逐像素持平，pass/credible/near 均 `83/78/33`；self-source off/on 同为 `280/287`，可信 strict/可疑/near/fail 均 `164/40/76/7`。现有上游 corpus 没有 secondary fallback descriptor driving case，故本切片按规范机制与零回归落地，不宣称像素收益。经验见 [`fallback-face-feature-descriptor-ownership.md`](../learnings/bugs/fallback-face-feature-descriptor-ownership.md)。
>
> **🧪 Dormant 实验裁决（R3411-F italic/oblique face category·2026-08-14）**：`italic-oblique-fallback` 是固定 `markA.ttf`、fresh Chromium Oracle `3.49%`、self-source `3.36%` 的明确 matching 缺口。实验把静态 `@font-face` 加载链从 `is_italic:bool` 升级为 Normal/Italic/Oblique 三态，shared resolver按 CSS Fonts 4实现 `italic→oblique→normal` 与 `oblique→normal→italic`，并以 `ZW_FONT_STYLE_MATCHING_V2=0` 回滚。resolver单测与全 workspace编译通过，但同一 release runner 的完整 css-fonts 282 案 Oracle A/B 全部逐像素持平，目标页也保持 `3.49%`；说明 inline span owner 的 resolved face未进入最终 paint glyph，三态 plumbing在当前生产路径 dormant。源码已完整回退，不提交无像素作用的实现。后续须与 inline owner face ownership/Phase A 协同，禁止单独重试 resolver。经验见 [`italic-oblique-matching-needs-inline-face-ownership.md`](../learnings/bugs/italic-oblique-matching-needs-inline-face-ownership.md)。
>
> **🔬 候选裁决（R3412-F synthetic small-caps expansion·2026-08-14）**：`small-caps-letter-spacing-001` 使用无 `smcp` 的 Ahem，self-source `0.13%`、Chromium Oracle `1.85%`，表面像 spacing 缺口，实际要求 synthetic small-caps 把 `ß` 展开为 `SS`。ZeroWeb 当前只注入 OpenType `smcp`，缺 feature时没有 case expansion。正确修复会改变字符数、advance、断行和 source range，必须在线断前建立一对多映射并让 layout/shaping/paint共同消费；paint侧补 spacing或 Ahem特判都会制造半正确实现。故本轮不提交源码，留待 inline text transform ownership协同切片。经验见 [`synthetic-small-caps-needs-pre-line-break-expansion.md`](../learnings/bugs/synthetic-small-caps-needs-pre-line-break-expansion.md)。
>
> **▶ WPT 资产进展（R3413-F imported font resource closure·2026-08-14）**：新增 `make audit-imported-font-resources`，逐项扫描 `imported-tests.txt` 的 test/ref 直接 `url()` 字体引用并要求存在于 `imported-resources.txt`。首轮立即发现 3 个 fresh-checkout 缺口：`font-feature-resolution-*` 的 `fonts/Lato-Medium-Liga.ttf`、plaintext bidi 11 案的 `fonts/sileot-webfont.woff`、`font-synthesis-style` 的 `fonts/Lato-Medium.ttf`；本机 full corpus此前掩盖了账本缺失。三项已通过标准 importer登记，闭包门禁现 PASS。该切片不改变当前机器像素，只保证常驻 WPT 在 fresh sync 后仍消费相同固定字体。
>
> **▶ WPT 资产进展（R3414-F resource closure automatic gate·2026-08-14）**：`fetch-wpt-data` 过去只执行资源同步，不验证常驻 test/ref 的字体引用是否全部进入账本；漏项在本机 full corpus存在时仍会静默通过。现把 `audit-imported-font-resources.sh` 接到同步步骤之后，所有 `reftest` / `reftest-oracle` / `import-wpt` 的 fetch前置链都会自动验证闭包，缺失即失败。手动 `make audit-imported-font-resources` 入口继续保留。
>
> **▶ WPT 资产进展（R3415-F linked stylesheet font closure·2026-08-14）**：字体闭包门此前只扫 test/ref 内联 `url()`，`<link href="/fonts/*.css">` 中的字体可绕过。现同时解析 test/ref 直接引用的 stylesheet并扫描其字体 URL；相对路径按浏览器 web-root 语义归一化，`..` 不得逃出 WPT root。常驻用例当前主要命中 `/fonts/ahem.css → fonts/Ahem.ttf`，手动门与 `fetch-wpt-data` 自动链均 PASS，未发现新的账本缺口。
>
> **📋 待用户决策清单（遇需拍板项在此追加，跳过并继续其他轻量修复）**：
> - 格式：`- [ ] <事项> — 为何需用户（深结构 / 许可证 / 破坏性操作 / 改 Mission / 超大下载）— 建议 — 追加时间`
> - **深结构方向（用户 2026-07-29「主做轻量修复」指令划入护栏，等点名，不自主开工）**：
>   - [x] ~~font-metric 生产激活+A/B — R2202 dormant 基础设施（webview+renderer，env `ZW_PERFONT_LINEHEIGHT=1`）已落地未激活；深 plumbing + 需 product-smoke A/B 量化 CJK 收益 — 待用户授权激活并跑 A/B~~ ✅ **已完结（2026-08-01）**：用户授权后 A/B 完成 = **net 负，保持 dormant**（welcome 英文 −0.44pp；morning 中文零变化——全显式 line-height 无 normal 行，「CJK lever」假设证伪）；证据 `evidence/font-metric-activation-ab-2026-08-01.md`（R2393）
>   - [ ] vertical-mode native R1043 — 四层协调深改，R1043 谱系停止条件曾触发 — 等点名
>   - [ ] taffy replaced-element border-box sizing R2174 — 深 multi-session — 等点名
>   - [ ] Phase A slice-3 IFC 深构造（IFC 单一权威化）— 深 architectural，设计已就绪 — 等点名
>   - [x] ~~font-stack coherence rebuild + Phase A IFC line-box-metric 统一（R2025 user-blocked；RFC-ready [`unified-font-stack-design.md`](rendering-compat/unified-font-stack-design.md) v0.2.3）~~ ✅ **已批准（2026-08-09 用户决策）**：接受 HarfBuzz C 依赖，恢复主动实施，分片执行中 — **⚠️ R2869 勘误 R2867（历史依据，不改变批准）**：Skia/raster C-dep **非** font-wall unlock（R1560 real-skia-safe A/B net-24 已证伪；光栅层 R1068/R1159 FreeType default-on 已对齐 chromium）；font-wall 残余在 **layout/metric coherence（Phase A IFC）**，须 **full font-stack rebuild（layout/paint/wrap metric coherence）整体做**（isolated slice 全 net-negative：line-height ×3 + advance ×4 + raster ×1，不可切片），二者皆 deep multi-week user-gated；DC-2~5/2026 65% oracle absent 此授权 = unreachable
>   - [ ] 响应式图片 srcset / `<picture>` / CSS `image-set()`（R2412 发现）— `extract_img_resources` 仅取 `<img src>`，不解析 srcset/source；srcset-only 图缺抓、其余仅次优分辨率。正确选源须 DPR+`sizes`+布局（layout-dependent）+ painter effective-src plumbing — 深，须 RFC+布局集成 — 等点名
> - 真正需用户拍板的 4 类（不兼容/闭源许可证、破坏性 git/文件操作、改 Mission/Done/范围、超大磁盘网络下载工具审批无法覆盖）同上格式追加。当前该 4 类无悬而未决项。
>   - [x] ~~**Mission 95% 的时间账本校准（A1）** — 改 Mission/Done/范围 — Ladybird 7 年/8 人全职/428 贡献者才到同源 93.33%（2026-08-05 实测，官方算法复算），ZeroWeb 当前 oracle ~57% + G0 单维护者；95% 作为短期冲刺目标与幂律现实不匹配是 plateau 反复的根源之一~~ ✅ **已拍板（2026-08-07）**：采纳分阶段里程碑（2026 65% → 2027 80% → 长期 95%），Mission 已更新
>
> **~~⏸️ 旧暂停裁决（2026-07-29，agent 自设；已被上方用户指令推翻，不再约束执行，仅作历史留档）~~**：当时 agent 判定 clean-lever 穷尽、改为「转其他 goal + 低频 plateau-guard」、要求结构性方向须用户点名授权。**此判定与更早的 `2026-07-16 默认决策边界`（已授权上述结构性方向）冲突，agent 当时选了更保守的一方并自我停手，用户 2026-07-29 明确推翻并要求持续推进。**

---

## Mission

以 **上游 WPT 真实 reftest 通过率 95%+** 为长期愿景（核心 CSS 领域与 Chromium 一致），并采用**分阶段里程碑**校准执行预期（2026-08-07 用户拍板 A1；决策依据 [`ladybird-timeline-calibration-2026-08-07.md`](rendering-compat/ladybird-timeline-calibration-2026-08-07.md)）：

| 阶段 | 目标（oracle 一致率） | 说明 |
|---|---|---|
| 2026 年内 | **65%** | 从当前 ~57% 起步；轻量修复 + 守成形态 |
| 2027 | **80%** | 结构性缺口（IFC 等）解耦后 |
| 长期 | **95%** | Ladybird 同口径参考：8 人全职 + 400 贡献者 7 年才到同源 93.33%——95% 是多年级愿景 |

分阶段目标不降低长期 Mission；每阶段达标即验收，plateau 属幂律曲线预期内（不是失败信号）。

**关键约束**：所有验证必须基于从上游 WPT 仓库（`https://github.com/web-platform-tests/wpt`）导入的**真实 reftest**，不允许使用手写 inline reftest 替代或充数。通过率统计的分母是上游 WPT 目录中**所有**属于范围内、不在 skip list 中的 reftest case，不允许人为缩小导入范围。

**⚠️ 优化目标 = chromium Oracle 一致率，非同源通过率（DC-14，2026-06-16 实测确立）**：reftest runner 当前用 ZeroWeb 自渲染 ref 作参考（`reftest.rs:278-283` `run_reftest_with_base` 把 test 与 ref 都经同一 `RenderPipeline` 渲染），同源通过率含 **46.5% 假通过**（全量实测，见 `evidence/cross-validate-full-2026-06-16.txt`）——真实「与 chromium 一致」通过仅 ~37%。**同源通过率（当前 436/489）不再作为优化目标或达标依据**；优化目标改为「chromium Oracle 一致率」，修复优先取 `evidence/analyze-pollution-2026-06-16.txt` 的 18 个真 bug 候选，每项修复用 `scripts/cross-validate.py` 验证（而非仅看同源通过）。**★ R669 起 chromium Oracle 已集成为一等 harness 指标**：`make reftest-oracle [DIR=...]` 直接报 per-dir chromium-Oracle 真一致率 + top 发散修复候选（DC-14 独立 Oracle 项 ✅，见下），取代 post-hoc cross-validate.py 作主测量路径。

覆盖范围：

1. **渲染器图元覆盖** — CPU 渲染器和 GPU 渲染器必须支持所有 13 种 `RenderPrimitives` 图元类型，浏览器必须正确消费所有图元
2. **CSS 2.1 核心**（`css/css2/`, `css/CSS2/`）— 渲染兼容性的基石
3. **Flexbox + Grid**（`css/css-flexbox/`, `css/css-grid/`）— 现代布局引擎必备
4. **Positioning + Float + Table + Multicol**（`css/css-position/`, `css/css-float/`, `css/css-tables/`, `css/css-multicol/`）— 传统布局模式完整覆盖
5. **文字排版全套**（`css/css-text/`, `css/css-writing-modes/`, `css/css-fonts/`, `css/css-text-decor/`）— 文本渲染正确性
6. **布局正确性** — Margin 折叠、BFC、Float 布局、滚动容器等核心 CSS 2.1 布局行为
7. **高级视觉效果** — text-shadow、多背景图层、clip-path、backdrop-filter 等

执行方式：**交替推进** — 每轮执行同时扩展上游 WPT 真实 reftest 导入范围和修复发现的渲染缺口，直到目标通过率达标。

运行环境：**CPU 软件渲染 + GPU 渲染都必须通过** 上游 WPT 真实 reftest 验证。

参考基准：**Chromium（Chrome/Edge）** 的渲染输出作为 reftest 的参考截图来源。

### 优先级修订：Legacy Static Web（HTML 3.2/4 + CSS1/2）

**背景记录（2026-06-26）**：用户反馈 `http://172.27.46.54:8000/testpage.htm` 一类老式静态页面渲染效果差。该页面不是 IE1 专属兼容目标，而是典型的 HTML 3.2/4 + CSS1/2 静态网页模式：`BODY BGCOLOR/TEXT/LINK/VLINK`、`TABLE BORDER/CELLPADDING`、`TR BGCOLOR`、`IMG ALIGN=TOP`、`FONT SIZE`、标题/段落/列表/链接等基础结构。当前 `rendering-compat` 主线以 WPT reftest + Chromium oracle 为核心，虽已覆盖部分 CSS2/presentational hints，但没有把这类老式静态网页作为独立产品验收面。

**裁决**：在不降低 WPT/DC-14 最终目标的前提下，将 **HTML 3.2/4 常见静态文档 + CSS1/2 常见布局** 提升为短期高优先级推进面。理由是：

- 这类页面大量依赖 UA stylesheet、HTML presentational attributes、基础 block/inline、表格、图片、列表和链接颜色，修复通常比 multicol/writing-modes/font-feature 等现代或结构性子域更局部。
- 用户可见收益更直接：静态文档、内网页、说明页、老式工具页不需要 JS/现代 CSS，也能暴露基础排版/绘制链路问题。
- 该方向不是完整 CSS2 达标的替代品；完整 CSS2 `chr<1%` 仍是长期目标，但短期应优先让 legacy static pages "可读、布局不崩、核心语义可见"。

**Legacy Static Web Tier 1 范围**：

- HTML presentational hints：`body bgcolor/text/link/vlink/alink`、`table border/cellpadding/cellspacing/width/height`、`tr/td/th bgcolor/align/valign/width/height`、`img width/height/align`、`font size/color/face`、`hr` 基础属性。
- UA stylesheet 基线：`h1`-`h6`、`p`、`ul/ol/li`、`b/strong`、`i/em`、`a`、`table/tr/td/th`、`font`、`hr` 的默认 display、margin、font-size、font-weight、font-style、text-decoration、border/padding 语义。
- CSS1/2 常见模式：颜色/背景、字体大小与继承、普通流、inline formatting 基础、表格基础布局、替换元素尺寸与 baseline/vertical-align、margin/padding/border、float/clear 基础。
- 明确暂不扩展到 IE 专属行为或浏览器 bug 兼容；quirks mode 只按标准/Chromium 可解释行为推进。

**验收方式**：新增 `legacy-html` 产品 smoke fixture 集，至少包含 20 个 HTML 3.2/4 + CSS1/2 静态页面（真实录制 + 合成最小页各占一部分），使用 Chromium 参考截图做 oracle，并在 ZeroWeb CPU 路径输出截图后做像素对比。该 fixture 集不替代 WPT 通过率，但作为短期修复优先级和回归门禁；每次修复必须同时说明它对应的 WPT/CSS 规范点或 legacy fixture。

---

## Support Envelope

### 在范围内

| 领域 | 具体内容 | 说明 |
|------|----------|------|
| WPT reftest 基础设施 | 导入上游 WPT reftest、解析 test list（含 fuzzy 注解）、截图对比、通过率报告、CI 集成 | 详见 evidence/ |
| Chromium 参考截图 | 自动化 headless Chromium（Puppeteer/Playwright）截图工具链 | 详见 evidence/ |
| Reftest 分类容差 | 布局类严格容差、文字类宽松容差、WPT fuzzy 注解覆盖 | 详见 evidence/ |
| CSS 2.1 渲染 | 盒模型、颜色、背景、边框、margin 折叠、inline formatting、BFC、浮动清除、基础定位 | 详见 current-baseline.md |
| Inline formatting 所有权 | 文本节点、inline 元素、inline-block、`<br>`、混合中英文文本单一权威 | 详见 current-baseline.md |
| Flexbox 渲染 | 所有 flex 属性的正确布局和绘制 | 详见 current-baseline.md |
| Grid 渲染 | 所有 grid 属性的正确布局和绘制 | 详见 current-baseline.md |
| Float 布局 | 完整的 float 布局算法，float exclusion、clear、BFC 触发 | ✅ 核心 float 定位、clear、float containment 与 inline exclusion 已实现（R895 / DC-11） |
| Table 布局 | 完整的 table layout 算法，table-layout: auto/fixed、border-collapse、spanning | ✅ 表格网格构建、auto table layout、colspan、border-spacing、匿名表格盒已实现 |
| Multi-column 布局 | column-count/column-width 的实际列排布、column-rule、column-span | ✅ column-count/column-width、column-gap 和基础列分配已实现 |
| 文字排版 | OpenType shaping、BiDi 算法、CJK 排版优化、text-align justify、word-break/overflow-wrap、writing-mode、vertical text | ✅ 已集成 rustybuzz、unicode-bidi、CJK line-breaking；残余详见 current-baseline.md |
| Position 定位 | absolute/relative/fixed/sticky 的精确坐标计算 | ✅ fixed 已修复（R324）；sticky 静态部分已验证（R1982）；残余详见 current-baseline.md |
| Reftest 验证 | CPU 软件渲染模式 + GPU 渲染模式的截图对比 | 详见 current-baseline.md |
| 产品静态页面视觉 smoke | `apps/browser/assets/welcome.html` 等内置静态页面、录制的真实静态文章页和图片密集静态站点必须通过 ZeroBrowser/WebView 路径与 Chromium 参考截图对比 | ✅ 已建立产品 smoke 证据链；详见 dc-progress.md DC-13 |
| 渲染器图元覆盖 | CPU 渲染器和 GPU 渲染器必须能够渲染所有 `RenderPrimitives` 类型（fills、rounded_rects、gradients、shadows、images、strokes、path_fills、path_strokes、transforms、clips、filters、blend_modes、glyphs） | ✅ **已实现（M7）**：CPU + GPU 均已实现全 13 种图元渲染并附单测；详见 current-baseline.md |
| 浏览器图元消费 | `append_webview_primitives()` 必须将所有 `RenderPrimitives` 类型传递到渲染器，不能静默丢弃 | ✅ **已实现（M7）**：遍历全 13 字段无丢弃；详见 current-baseline.md |
| 渐变渲染 | 线性渐变、径向渐变、锥形渐变、重复渐变的 CPU + GPU 渲染 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 阴影渲染 | `box-shadow` 的高斯模糊阴影渲染（offset + blur + spread + color） | ✅ **已实现（M7）**；详见 current-baseline.md |
| 图片渲染 | 背景图片（`background-image`）、`<img>` 元素、`list-style-image` 的图片解码和渲染 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 线段/路径渲染 | `StrokePrimitive`（线段）、`PathFillPrimitive`（路径填充）、`PathStrokePrimitive`（路径描边）的渲染 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 变换渲染 | CSS 2D transform（translate、rotate、scale、skew、matrix）的正确应用 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 裁剪渲染 | `overflow: hidden/clip` 的矩形裁剪，`border-radius` 的圆角裁剪 | ✅ **已实现（M7）**；详见 current-baseline.md |
| 滤镜渲染 | CSS filter（blur、brightness、contrast、grayscale、hue-rotate、invert、opacity、saturate、sepia、drop-shadow） | ✅ **已实现（M7）**；详见 current-baseline.md |
| 混合模式渲染 | `mix-blend-mode` 的 16 种混合模式（normal、multiply、screen、overlay、darken、lighten 等） | ✅ **已实现（M7）**；详见 current-baseline.md |
| Margin 折叠 | 相邻块级元素 margin-top/margin-bottom 的正确折叠算法 | ✅ **已实现（R323 实测）**；详见 current-baseline.md |
| BFC（Block Formatting Context） | `overflow: hidden/auto/scroll`、`display: flow-root`、浮动等正确创建 BFC，隔离浮动和 margin 折叠 | ✅ margin 隔离已实现（R323 实测）；详见 current-baseline.md |
| 替换元素布局 | `<img>`、`<video>`、`<iframe>`、`<canvas>` 的固有尺寸计算和 `object-fit` | ✅ **已实现**；详见 current-baseline.md |
| 滚动容器 | `overflow: scroll/auto` 的可滚动容器，滚动偏移的正确应用 | ✅ 静态部分已验证（R1982）；残余详见 current-baseline.md |
| text-shadow | 文字阴影（offset + blur + color） | ✅ 已实现 text-shadow paint 图元生成与渲染；详见 dc-progress.md DC-12 |
| 多背景图层 | `background-image` 多层叠加渲染 | ✅ **已实现**；详见 current-baseline.md |
| clip-path | CSS clip-path（circle、ellipse、polygon、inset） | ✅ **已实现（M9）**；详见 current-baseline.md |
| backdrop-filter | 元素背后内容的滤镜效果 | ✅ **已实现（M9，R894 实测验证）**；详见 current-baseline.md |
| CSS mask | CSS 遮罩效果 | ✅ **已实现（M9）**；详见 current-baseline.md |
| 重复渐变 | `repeating-linear-gradient`、`repeating-radial-gradient` | ✅ **已实现**；详见 current-baseline.md |

### 不在范围内（明确排除）

- **非 CSS 渲染领域的兼容性**：JS/DOM API 兼容性、网络协议兼容性、安全策略兼容性不在本目标范围内（由父目标 `zero-web.md` 覆盖）
- **Canvas / WebGL / WebGPU**：不在本目标 reftest 范围内
- **动画/交互的帧级正确性**：CSS animation/transition 的视觉正确性验证不作为 reftest 核心指标（但如果有 reftest 覆盖则需通过）
- **性能优化**：本目标关注渲染正确性，不关注渲染性能（由父目标的性能基准体系覆盖）
- **Chromium 专属行为**：只对齐标准规范行为，不复制 Chromium 的 bug 或非标准行为
- **新 crate 依赖的大规模引入**：最小化新依赖，仅在必要时引入许可证兼容的 crate
- **SVG 文档/内联 SVG 渲染**：不在本目标范围。作为 `<img>` / CSS `url()` 图片资源参与页面渲染的 SVG 栅格化属于"图片子资源与替换元素"范围，至少要覆盖产品静态 smoke 中的 Logo 场景

### 依赖约束

- **原则**：最小化新依赖引入
- **许可证**：如果必须引入新 crate，仅接受 MIT / Apache-2.0 / BSD 许可证
- **评估标准**：新依赖必须论证"不引入则无法达成 reftest 目标"的必要性
- **Taffy 迁移裁决（2026-07-16）**：用户已裁决 `taffy 0.7 → 新版 taffy` 应尽早推进，取消旧记录中的"暂缓/未决"状态。迁移不是一次性大爆炸升级，必须先设计并拆分为可回退切片，重点核查 `computed_style_to_taffy()` 适配层、baseline、intrinsic sizing、flex/grid/table/multicol、margin collapse、abspos/fixed/sticky 等行为差异；每个切片都必须用 Chromium oracle reftest、产品 smoke 和现有单测做 A/B，确认 net≥0 且无关键产品回归后才能落地。
- **默认决策边界（2026-07-16）**：为避免执行中反复请求人工决策，以下事项默认已批准继续推进：兼容许可证下的字体/光栅化/shaping C/C++ 依赖调研与小切片试验；R1035/LayoutNG 的本地源码、sparse checkout 或人工片段路线；vertical writing-mode 的 native/scoped/env-gated 改造；table、multicol、R109、Phase A 等结构性多会话工作。只有以下情况需要重新询问用户：不兼容许可证或闭源商业 SDK、大量磁盘/网络下载且工具审批无法覆盖、改变 Mission/Done Criteria/范围边界、破坏性 git/文件操作。

---

## 当前能力/缺口基线

**当前能力/缺口详细基线**：详见 [current-baseline.md](rendering-compat/current-baseline.md)（完整能力矩阵和已知缺口表）。

**关键状态摘要**（截至 2026-08-07·R2863）：
- ✅ **已完成**：CPU/GPU 渲染器全 13 种图元（M7）、浏览器图元消费（M7）、Margin 折叠（R323）、BFC margin 隔离（R323）、Float 核心布局（R895）、Position fixed（R324）、外部样式表加载（R213）、图片子资源贯通（R318）、产品 smoke 证据链
- ⚠️ **P1-严重缺口**：Inline formatting 所有权分裂、Layout/Paint IFC 双路径、滚动容器（「浏览器层 glyph 重排」R2004 已修复——`transform_webview_primitives` 逐个映射仅 scale+offset+clip 无 sort/reorder + 单测 `transform_webview_primitives_preserves_glyph_order` 守护，详见 current-baseline.md / DC-13；不再列 open）
- 📊 **测试基线**：总测试数 13495 全绿（`make test` R2563 周期复跑 + R2572-R2577 六连 lever 各轮零回归 13190/0/74 精确持平 + R2592 text-decoration shorthand thickness 接线 +1→13191 R2597 持平确认 + R2637 registry box-dimension initial-value 纠偏 + 守卫测试 +1→13192 + R2638 column-gap initial-value 纠偏 + 守卫测试 +1→13193（rendering-compat 侧 held）；R2638 后经父目标 zero-web P1a DOM/JS Bridge + 缺失 Web API 系列（R2704-R2863）+291 推进至 13484（R2862 plateau-guard 复跑确认）；R2873 var()-in-shorthand pending-substitution +8 单测 → 13492；R2878 两值 background-size +1 单测 → 13493；R2879 background 简写 gradient+color 拆分 +2 单测 → 13495（零回归）；74 ignored = 网络型 real_website_compat 用例），覆盖率 95.46% line / 96.94% function / 94.88% region

---

## Done Criteria

以下条件**全部满足**时，方可判定本目标完成。

**详细进度**：DC-1~14 完整进度详见 [dc-progress.md](rendering-compat/dc-progress.md)。

### DC-1: WPT Reftest 基础设施就位

- [x] 能够从上游 WPT 仓库 fetch 并解析 reftest test list（**扩展**现有 `manifest.rs`，不重写）
- [x] 解析上游 WPT MANIFEST.json 中每个 reftest 的 `fuzzy()` 元数据，并传递给像素对比引擎
- [x] 能够用 CPU 软件渲染器对 ZeroWeb 渲染输出截图（**复用**现有 `render_scene_to_framebuffer`）
- [x] 能够用 GPU 渲染器对 ZeroWeb 渲染输出截图
- [x] **自动化 headless Chromium 截图**：通过 Puppeteer/Playwright 脚本自动在 headless Chromium 中渲染 reftest HTML 并截图，作为参考基线（零手动操作）
- [x] **Viewport 对齐**：ZeroWeb 截图和 Chromium 截图在相同 viewport 尺寸下捕获（默认 800×600，可配置）
- [x] **JS 执行支持**：Reftest harness 在截图前通过 `script-sandbox` V8 runtime 执行页面 JavaScript
- [x] **分类容差机制**：支持按 reftest 分类设置不同像素容差阈值（布局类 ≤ 0.1%，文字类 ≤ 0.5%）；优先使用 WPT fuzzy 注解；容差锁定不可放宽
- [x] **范围外 reftest 过滤**：导入时自动过滤或标记范围外 reftest（SVG、Canvas、WebGL），维护 skip list 文件
- [x] 通过率报告按 WPT 目录分类输出（文本 + JSON 格式）
- [x] Reftest 运行可通过单一命令执行——`make reftest`（Makefile:74，test-guard 包裹 `cargo run --release --bin zero-wpt-runner -- reftest`）
- [x] CI 管线中集成 reftest 运行（至少 CPU 模式）——`.github/workflows/ci.yml` `reftest` job（workflow_dispatch：fetch-wpt-data + reftest-smoke 快门禁 + 全量 CPU reftest --format json + 报告 artifact 上传）+ `.github/workflows/weekly.yml` `reftest-trend` job（schedule + dispatch，周记录趋势）

**状态**：✅ **全部就位**——fetch/parse test list（manifest.rs）、MANIFEST.json fuzzy 元数据、CPU+GPU 截图、headless Chromium oracle 抓取、viewport 对齐（800×600）、V8 JS 执行、分类容差锁定（DC-14）、范围外 skip list、文本+JSON 报告、单一命令（`make reftest`）、CI 集成（ci.yml `reftest` + weekly.yml `reftest-trend`）全实现；详见 dc-progress.md

### DC-2: CSS 2.1 核心通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css2/` 和 `css/CSS2/` 目录导入**全部**范围内 reftest（排除 skip list 中的范围外 case）
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] 覆盖：盒模型、margin 折叠、BFC、inline formatting、颜色、背景、边框、基础定位
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

**状态**：详见 dc-progress.md

### DC-3: Flexbox + Grid 通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-flexbox/` 导入**全部**范围内 reftest
- [ ] 从上游 WPT 仓库 `css/css-grid/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

**状态**：详见 dc-progress.md

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-position/`、`css/css-float/`、`css/css-tables/`、`css/css-multicol/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

**状态**：详见 dc-progress.md

### DC-5: 文字排版通过率 ≥ 95%（基于上游真实 WPT reftest）

- [ ] 从上游 WPT 仓库 `css/css-text/`、`css/css-writing-modes/`、`css/css-fonts/`、`css/css-text-decor/` 导入**全部**范围内 reftest
- [ ] 上游 WPT 真实 reftest 通过率 ≥ 95%
- [ ] CPU 软件渲染模式 + GPU 渲染模式均达标
- [ ] **不允许**用 inline 手写 reftest 替代或充数

**状态**：详见 dc-progress.md

### DC-6: Quirks Mode 完整实现

- [ ] CSS parser 在 quirks mode 下正确调整解析行为
- [ ] Style system 在 quirks mode 下应用特定样式规则
- [ ] Layout engine 在 quirks mode 下实现特定布局行为
- [ ] DOM parser 的 quirks mode 状态正确传递到 CSS parser → style system → layout engine 链路

**状态**：✅ 实质已实现（CSS parser + style system 两层活跃；layout-engine 无独立 quirks 层由 style-system 预烘焙覆盖）；详见 dc-progress.md

### DC-7: 测试与质量不可退让

- [x] 所有现有测试持续全绿（`cargo test` 零失败）—— held baseline **13495/0/74**（74 ignored = real_website_compat 网络型用例，见下条）
- [x] **真实网站测试保留 `#[ignore]`**：`tests/integration/src/real_website_compat.rs` 中的真实网站兼容性测试因本地网络不稳定，保留 `#[ignore]` 标记，不计入本目标通过率统计。其余所有测试零 `#[ignore]`
- [x] 所有新增渲染修复必须有对应单元测试覆盖，**且把对应的上游 WPT reftest 用例导入常驻断言集（测试资产化，2026-08-06 落地）**：`make import-wpt TEST=<wpt 路径> REF=<ref 路径> [NOTE="R21xx 备注"]` —— 文件本体进入 `tests/wpt-runner/wpt-data/`（独立 repo），条目记入 `tests/wpt-runner/imported-tests.txt` 账本（随修复提交），manifest 自动重新生成
- [x] `cargo build` 零错误、`cargo clippy` 零警告（R2865 `cargo clippy --workspace --all-targets -- -D warnings` 全绿复跑确认；R2873 var()-in-shorthand 改动后 workspace clippy 复跑全绿，held baseline 13492/0/74）—— **DC-7 全部子项现已闭环**
- [x] Reftest 通过率报告持久化到 `docs/goal/rendering-compat/evidence/wpt-trends/`（`scripts/record-wpt-trend.sh` → `trend.csv` 绝对数 + JSON 快照；本地 `make reftest-trend`，每周 CI 自动记录，2026-08-06 落地）
- [x] 每轮执行的 reftest 通过率变化可追溯（`evidence/wpt-trends/trend.csv` 历史记录，含日期/模式/绝对数/git_sha，2026-08-06 落地）

**状态**：详见 dc-progress.md

### DC-8: CPU 渲染器图元覆盖 100%

**状态**：✅ **已完成（M7）** —— CPU 渲染器已实现全 13 种图元，详见 dc-progress.md

### DC-9: GPU 渲染器图元覆盖 100%

**状态**：✅ **已完成（M7）** —— GPU 渲染器已实现全 13 种图元（非 CPU passthrough），详见 dc-progress.md

### DC-10: 浏览器图元消费完整性

**状态**：✅ **已完成（M7）** —— `append_webview_primitives()` 遍历全 13 字段无丢弃，详见 dc-progress.md

### DC-11: 布局正确性

**状态**：✅ Margin 折叠、BFC 创建、Float 布局、Position fixed、替换元素、百分比高度、Auto margin 居中、min/max-width/height 已实现；⚠️ Position sticky、Overflow scroll/auto 残余属 host 层 interactive 特性；详见 dc-progress.md

### DC-12: 高级视觉效果

**状态**：✅ text-shadow、多背景图层、重复渐变、border-image、clip-path、backdrop-filter、CSS mask 已实现；[~] 打印媒体查询部分实现（R1981/R1991/R1992）；[ ] scroll-snap 需宿主层滚动输入路由；详见 dc-progress.md

### DC-13: 产品静态页面视觉 smoke

**状态**：✅ welcome.html、Legacy Static Web smoke、URL 导航外链 CSS、图片子资源、via-webview 路径、viewport 覆盖、自动检查、glyph 重排保护、证据持久化已实现（R1600/R1601/R658/R213/R318/R662/R1597/R1598/R2004）；⚠️ morning.work、wintertc.org fixture 录制待完成；详见 dc-progress.md

### DC-14: 真通过标准（anti-false-pass）— 验证可信度门禁

> 本 DC 防止 reftest 通过率被「同源假通过」「宽容差」「子集分母」污染。**DC-2~13 的通过率数字只有在本 DC 同时满足时才可信、才计入达标判定。**

**状态**：✅ 独立 Oracle、非平凡性检查、严格容差三态分类、容差锁定、分母真实性（R484 全量去子集化）、GPU 非 passthrough、内联 smoke 不计达标均已实现；详见 dc-progress.md

**关键事实**：字体光栅化（fontdue ≈ chromium）非渲染差异主因；多行 y 堆叠（R630）和字体归因三证推翻（R631）证实行盒度量为真因；详见 dc-progress.md 和相关 evidence 文件。

---

## 活跃里程碑（M7-M11）

**历史里程碑**：M2-M6 已完成或已过时，详见 [archive/milestones-history.md](rendering-compat/archive/milestones-history.md)。

### M7 — 渲染器图元覆盖 + 浏览器图元消费（✅ 已完成）

**目标**：消除渲染管线最大的视觉输出缺口 — 让 CPU/GPU 渲染器和浏览器 `append_webview_primitives()` 能处理所有 13 种 `RenderPrimitives` 图元类型。

**状态**：✅ **已完成（DC-8 CPU 13/13 + DC-9 GPU 13/13 + DC-10 浏览器消费全 13 字段，均附 framebuffer 像素断言测试）**

### M8 — 布局正确性（Margin 折叠 + BFC + Float + Replaced Elements）（⚠️ 部分完成）

**目标**：实现 CSS 2.1 核心布局算法，使块级布局结果与主流浏览器一致。

**状态**：✅ Margin 折叠、BFC margin 隔离、Float 核心布局、Position fixed、替换元素、百分比高度、Auto margin 居中已实现；⚠️ Position sticky、Overflow scroll/auto 残余属 host 层 interactive 特性

### M9 — 滚动容器 + 高级视觉效果（✅ 基本完成）

**目标**：实现滚动容器功能和高级 CSS 视觉效果。

**状态**：✅ text-shadow、多背景图层、重复渐变、border-image、clip-path、backdrop-filter、CSS mask 已实现；⚠️ scroll-snap 需宿主层滚动输入路由

### M10 — 上游 WPT 真实 Reftest 导入与验证（✅ 已完成）

**目标**：从上游 WPT 仓库导入**全部**范围内真实 reftest，建立可信的渲染正确性验证基线。

**状态**：✅ **已完成（R484 全量去子集化 ~9967 reftest + R669 chromium Oracle harness + DC-14 三态分类）**

### M11 — 全量冲刺 + 上游真实 WPT Reftest 通过率达标（⚠️ 进行中）

**目标**：修复所有剩余渲染缺口，达到上游真实 WPT reftest 各领域通过率 ≥ 95%。

**状态**：⚠️ **自主 clean-lever surface 经 6 vein definitively 穷尽**——R2572 订正旧「8 angle 穷尽」框架（过早），续经 4 法 land 六连 lever R2572-R2577（counters() / ::marker / list-style-type:string / border-image-outset / border-image-width / word-break:break-word；directed probe + Explore-agent fan-out + exhaustive field 审计 + exhaustive variant 审计）；R2578 exhaustive value-variant 审计（全值枚举变体→消费核验）= clean lever 零产出（残余全 false-positive / deep / host-layer / 0-test）；R2581 missing-property 批量核验（60 常见 CSS 属性→51 未应用全 deep/niche/host-layer）+ R2582 伪元素 parse-vs-apply 审计（19 伪元素→16 未 apply 全 Phase A IFC-deep/host-layer/niche/OOS）亦 = clean lever 零产出。**6 vein（directed probe + agent fan-out + exhaustive field + exhaustive variant + missing-property 批量 + 伪元素 parse-vs-apply）rigorous 证 clean-lever surface definitively 穷尽**。活跃自主面仅 ① 低频周期 plateau-guard（R2577 `make test` 13190/0/74 绿）+ ② 文档纠偏；**唯一推向 95% = 用户点名授权深结构专项**（最高 value = Phase A IFC line-box-metric 统一，first-letter/first-line 亦属此 territory；次 = R1043 vertical-mode / R2174 taffy border-box / font-stack C-dep / Phase 2 multicol fragmentation / individual+3D transforms；受字体度量 / 布局结构性 plateau 限制）。详见 `rendering-compat/master.md` R2572-R2582。

---

## Final Output Protocol

### 输出规则

| 情况 | 输出 | 说明 |
|------|------|------|
| Done Criteria 全部满足，目标能力达到 production-ready 水平 | `DONE` | 见下方"DONE 允许条件" |
| 进展仍可推进，还有未完成的工作 | `CONTINUE: <下一步>` | **这是默认输出** |
| 遇到真正的外部阻塞（依赖不可用、平台不支持） | `BLOCK: <原因>` | 罕见使用 |
| verify 发现未满足条件但进展仍可推进 | `CONTINUE: <下一步>` | 返回执行，不是 DONE |

### DONE 允许条件

**同时满足以下所有条件时才允许输出 DONE**：

1. ✅ Done Criteria DC-1 到 DC-14 全部满足（**DC-14 真通过标准是 DC-2~13 通过率数字的可信度前提**）
2. ✅ CPU 渲染器 + GPU 渲染器均支持全部 13 种 `RenderPrimitives` 图元类型
3. ✅ 浏览器 `append_webview_primitives()` 正确消费并渲染所有图元类型
4. ✅ 所有四个 WPT 领域（CSS 2.1、Flexbox+Grid、布局模式、文字排版）通过率均 ≥ 95%（基于真实上游 WPT reftest，且为**严格容差真通过率**、reference 为 **Chromium 独立 Oracle**、分母为上游全量——即满足 DC-14）
5. ✅ Margin 折叠、BFC、Float 布局、滚动容器等核心布局行为与 Chromium 一致
6. ✅ CPU 软件渲染 + GPU 渲染双模式均达标
7. ✅ `cargo build` + `cargo test` + `cargo clippy` 全通过
8. ✅ 有结构化的 reftest 通过率报告作为自动化证据（包含真实 WPT reftest 结果）
9. ✅ master.md 内部自洽，archive 已建立，进度已归档
10. ✅ 产品静态页面视觉 smoke 通过
11. ✅ 渲染能力本身达到可验证的 production-ready 质量

### 禁止输出 DONE的情况

即使以下情况中部分条件看起来"还行"，也**不允许**输出 DONE（包括但不限于）：

- ❌ CPU 或 GPU 渲染器不支持全部 13 种图元类型
- ❌ GPU 渲染器是 CPU 渲染器的 passthrough 封装
- ❌ `append_webview_primitives()` 丢弃任何图元类型
- ❌ ZeroBrowser 对 WebView glyph 做会改变布局语义的后处理重排
- ❌ 只通过了手写 inline reftest，未使用上游 WPT 真实 reftest
- ❌ reftest reference 由 ZeroWeb 自渲染（同源），未接入 Chromium 独立 Oracle（DC-14）
- ❌ 通过率含同源假通过而未做非平凡性检查（DC-14）
- ❌ 分母为子集，非上游全量（DC-14）
- ❌ 容差过宽松
- ❌ 无 reftest 证据，或 reftest 存在未分析的失败项
- ❌ 无实际代码/测试进度（仅有文档和计划）

### BLOCK 策略

- "未完成、证据不足、暂时无法验证通过率、文档状态不一致" 都是**继续推进的信号**，不是 BLOCK 的理由
- 即使遇到困难，如果仍有可能推进，输出 `CONTINUE: <下一步>`
- 只有在真正无法继续（外部依赖不可用且无替代方案、平台根本性不支持）时才输出 BLOCK
- 缺少 coverage 测量手段、缺少统一统计脚本、缺少报告链路 — 这些是要继续推进的工作内容，不是 BLOCK 的理由

---

## Execution Protocol

### 高收益执行模式（2026-07-28 裁决）

当前目标进入 **plateau-guard + 高收益推进** 模式：

1. **守住已获得收益**：每轮先确认 `make test`、产品 smoke、legacy smoke 没有新回归；reftest 作为回归守卫和机会性扫描，不再作为短期 95% 冲刺。
2. **只接 low-risk clean lever**：只有同时满足"明确 driving test、根因清楚、改动面小、A/B 无新失败、产品 smoke 无结构回归"的修复才继续落地。
3. **及时跳过死胡同**：同一方向连续 2-3 轮 empirical 扫描 negative、或需要高风险架构重写但没有新设计时，立即记录结论并转向，不继续消耗会话。
4. **Phase A 只做设计后实施**：完整 inline-box-model / IFC coherence 是潜在高收益架构方向，但必须先写可回退实施设计，包含 kill-switch、结构签名 gate、三态 A/B 门禁和净负回退策略；禁止直接按旧 `phase-a-slice1` 开工。
5. **明确暂跳过项**：font-stack rebuild/M18、P1b JS Bridge 深改、P3 真窗口/GPU 验收、R109 单点、37-form-controls 单点、inline SVG/SVG intrinsic sizing、sticky/scroll-snap/动态滚动不作为当前 rendering-compat 主线。
6. **文档优先级**：入口文档定义长期目标；当前执行方向以 `docs/goal/rendering-compat/master.md` 顶部"当前裁决"块为准；archive/evidence 只作历史证据，不覆盖最新裁决。

### 自主执行原则

执行代理必须：

1. **自主探索**当前渲染管线状态，识别能力缺口
2. **自主导入** WPT reftest，扩大覆盖范围
3. **自主运行** reftest，分析失败原因
4. **自主修复**渲染错误，不等待用户逐步指令
5. **自主添加**测试，新修复必须有对应单元测试
6. **自主验证**，运行 reftest + `cargo test` 确认修复有效
7. **自主归档**，完成的里程碑记录到 archive
8. **持续推动**，直到 Done Criteria 全部满足

### 交替推进策略

每轮执行的工作模式：

1. **扩展基础设施**：从上游 WPT 仓库导入更多真实 reftest case，扩大覆盖范围
2. **运行上游真实 reftest + chromium Oracle 交叉验证**：同源通过率仅作自一致性参考；**优化目标 = chromium Oracle 一致率**，修复优先取真 bug 候选（chromium 大幅不一致但同源「通过」的用例），每项修复**用 chromium Oracle 验证**而非仅看同源通过
3. **修复渲染缺口**：优先修复被同源假通过掩盖的真实缺口
4. **补充测试**：为每个修复添加单元测试
5. **验证回归**：确保修复不破坏已有通过的 case
6. **更新文档**：更新 master.md 状态和 evidence

### 遇到问题时的处理原则

1. **已知失败测试**：不允许留给下一轮。遇到 flaky test、遗留失败、环境脚本问题时，当作当前任务的一部分修复
2. **Reftest 失败分析**：每个失败 case 必须分析根因（CSS parser 错误？样式计算错误？布局算法错误？绘制错误？）
3. **技术决策**：在 master.md 中记录关键决策及其理由（如是否引入新依赖、选择哪种实现方案）
4. **依赖问题**：优先自行解决；只有真正无法解决时才 BLOCK
5. **范围变更**：如果发现目标需要调整，在 master.md 中记录并说明理由，但不修改本文件（除非 Mission 本身变化）

### 当 verify 发现缺口时

- 默认输出 `CONTINUE: <下一步>` 并返回执行
- 不输出 DONE 或大段解释
- 如果仍有可能推进，就不结束

---

## Document Control / Archive Policy

> **📄 2026-07-29 结构性精简**：本文件从 982 行精简到 ~460 行。详细内容转子文档：DC 进度→ [`dc-progress.md`](rendering-compat/dc-progress.md)、当前能力/缺口基线→ [`current-baseline.md`](rendering-compat/current-baseline.md)、已完成里程碑→ [`archive/milestones-history.md`](rendering-compat/archive/milestones-history.md)。**精简前完整原文（982 行，零删减保底）→ [`archive/rendering-compat-pre-slimdown-2026-07-29.md`](rendering-compat/archive/rendering-compat-pre-slimdown-2026-07-29.md)**。`master.md` 同期精简（750→~135 行），完整原文→ [`archive/master-pre-slimdown-2026-07-29.md`](rendering-compat/archive/master-pre-slimdown-2026-07-29.md)。

### 文档控制平面

本目标采用**两层文档控制平面**：

#### 入口文档（稳定、不频繁修改）

- **路径**：`docs/goal/rendering-compat.md`（本文件）
- **职责**：定义本目标的 Mission、Done Criteria、执行协议和文档治理规则
- **修改条件**：仅在目标本身发生实质性变化时修改（如调整 WPT 覆盖范围、修改通过率目标、调整技术路线）
- **禁止行为**：每轮执行不重写本文件；日常进度、证据、活跃里程碑更新写入 master.md

#### 运行时控制平面（持续演进）

- **路径**：`docs/goal/rendering-compat/master.md`
- **职责**：当前真实状态的唯一控制面板，包含：
  - 当前活跃里程碑及其完成状态
  - 当前各 WPT 目录的 reftest 通过率数据
  - 已导入的 reftest case 数量和分类
  - 已发现和已修复的渲染缺口清单
  - 当前能力矩阵和已验证项
  - 下一步计划
  - 未解决问题列表
- **治理规则**：
  - master.md 是持续演进的增量控制面板，不是一次性交付物
  - 不允许无限增长 — 过时内容必须重写、压缩或迁移到 archive
  - 各章节之间必须自洽（活跃里程碑、Done Criteria、通过率数据、Latest Evidence 不能互相矛盾）
  - 如果出现矛盾（如"通过率未达标但证据声称全部满足"），执行代理必须先纠正文档和状态评估再继续

#### 归档区域（历史记录）

- **路径**：`docs/goal/rendering-compat/archive/`
- **职责**：存储已完成里程碑的详细过程、关键决策、验证结果、commit hash 和历史证据
- **性质**：archive 是历史记录区，不是当前状态的来源

#### 证据区域（验证数据）

- **路径**：`docs/goal/rendering-compat/evidence/`
- **职责**：存储 reftest 通过率报告、失败截图对比、覆盖率数据等验证证据
- **性质**：持续追加，不修改已有证据文件

### 文档治理原则

1. master.md 各章节必须自洽 — 活跃里程碑、Done Criteria、通过率数据、Latest Evidence 不能互相矛盾
2. 如果发现矛盾，执行代理必须先纠正文档再继续
3. master.md 不允许无限增长 — 过时内容必须压缩或归档
4. archive 是只追加的 — 不修改已归档内容
5. 所有验证证据必须以结构化形式持久化（reftest 报告、截图、覆盖率数据）

---

## 单文件行数限制

- 单个 `.rs` 文件不超过 2000 行
- 如果超过，按职责拆分为多个模块
