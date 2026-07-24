# 渲染兼容性目标 — 运行时控制面板

> **🎯 当前状态速览（2026-07-25）**
>
> - **Reftest broad 一致率 ~57%**（plateau；2026-07-17 用户裁决接受）。自主 clean-lever 空间经 R1872–R1878 七轮 subdir 扫描 + R109 probe **definitive 穷尽**——唯 R1874 出 +7，残余全 font-wall / interaction-entangled / structural-deadlock。
> - **唯一 headline unlock = Phase A IFC 三路径统一**（line-box metric coherence；pre-authorized ruling #4，多 session，未实施）。阻塞 reftest 大盘 / 37-form-controls overlap / vertical-mode。详见 [`blockers-resolution-plan-2026-07-25.md`](./blockers-resolution-plan-2026-07-25.md)。
> - **font-wall** = Phase A layout-metric coherence spread（非 raster、非 advance 单点，R1764–R1769 三部曲收敛）。font-stack rebuild = user-gated ruling #2（M18 暂不优先，plateau-accepted）。
> - **legacy smoke 50/51 struct PASS**（唯一 FAIL = 37-form-controls = Phase A 阻塞，非回归）。
> - 浓缩结论见下方「## 综合裁决」「## 当前状态概览」；用户裁决见 📌 裁决包；四大阻塞解决方案见 [`blockers-resolution-plan-2026-07-25.md`](./blockers-resolution-plan-2026-07-25.md)。
>
> **📦 顶部 round-detail（R1651–R2025 逐轮详记）已归档（2026-07-25 master.md 瘦身）**：原 master.md 此处 R1651→R2025 的逐轮详记（~660 行超长 paragraph）已逐字迁出 → [`archive/rounds-r1651-r2025-accumulated-detail.md`](./archive/rounds-r1651-r2025-accumulated-detail.md)（内容 100% 保留，未去重）。更早轮次归档见 [`archive/`](./archive/)（R1578b 及更早 / R1579–R1650 / R125–R354 / R388–R430 等）。

## 综合裁决：两阶段 plateau 演进

> 本节为 doc-maintenance 轮的**浓缩结论**，置于控制面板顶部便于检索。逐轮详细记录见顶部 round-detail（R491–R511）与文末「最近轮次详细记录」指针；更早轮次归档见 [`archive/`](./archive/)（R484–R490 伪元素线、R342c–R480 DC-14 导入 era、R314–R334、R307–R313、R305–R306、R304、R303、R142–R302、R23–R139、R11–R20）。

**核心结论（2026-07-16 更新，折叠 R990–R1093；分两阶段 + 字体栈后续路线已授权推进）**：

> **★ 2026-07-06 追加（R990–R1093：font-wall 首次突破 + FreeType C-dep root unlock + plateau definitive；本块 supersede 下方 R660–R668「font-wall 彻底定性为永久 plateau」结论）**：
> 1. **font-wall 不再是「永久 plateau」**：R990（★★★ `apply_vertical_alignment` is_ahem-gated ascent ratio，non-Ahem 0.8→0.928）LANDED，全量 oracle **4530→4668（+138 pass）** = font-wall 首次实质突破；R1068（★ `freetype-raster` feature，Phase 2 FreeType 光栅化替 fontdue）LANDED **feature-gated default-off**，css-text Oracle **+24 credible**（357→381）+ welcome −0.28pp = 首个 font-wall 正 yield（C 依赖证据）。R1066–R1071 跨平台 de-risk 完成（cc crate 编译 FreeType2 C 源，三平台一致）。
> 2. **Phase A 非-Ahem = fontdue tight-ink = font-wall = C-dep 共同 root（R1088–R1090 三证）**：::first-letter 端到端实现正确但度量门控 −7（R1088）/ Phase 3 line-box store-gate+paint 公式扩展 linebox −1·css-text −14·css-text-decor −14（R1090）/ ::first-letter color-only net count-neutral（R1089）三变体均 net-negative **直到 C-dep 解锁**——确证 font-wall 是 Phase A 非-Ahem 路径的共同 root，单 session 不可绕过。
> 3. **★ FreeType C-dep 翻 default = ✅ LANDED R1159（2026-07-08）**：技术已验证（yield 全表测绘 +32 集中 text dir R1084；R1092 价值上修 +200~400 batch unlock；R1094 全 corpus 实测 +232 零回归）。`freetype-raster` 已 default-on（`crates/render-foundation/Cargo.toml` `default = ["freetype-raster"]`，12 下游 crate 经 workspace 自动获得，规避 R1138 toggle gotcha）。本机 A/B 实测：css-text oracle OFF 358→ON 382 = +24（复现 R1094）+ welcome OFF 16.57%→ON 16.29% = −0.28pp（复现 R1068）+ make test 12205/0 + clippy clean，零回归。**font-wall（+232）已解锁落地**；**Phase A 非-Ahem + ::first-letter 经 R1160/R1206 证伪未被 C-dep 解锁**（R1160 formula-only -1 / R1206 combined store-gate+公式 -22 post-FreeType，比 pre-FreeType R1090 -14 更差；R1090/R1095/R1160/R1206 四证 Phase A metric 放置杠杆死；R1206 关闭 R1205 留的「combined 未单独验证」最后 gap）。**R1158 / R1213 已清除跨平台构建技术问题并完成 7-target 验证**：macOS GPU headless adapter、Windows rusty_v8 advapi32、Windows freetype-sys zlib/include/aarch64 等问题均已修复；`scripts/*.ps1` 本地构建脚本自动检测 Strawberry Perl zlib.h。剩余仅为 freetype-sys 上游 ARM64 NEON 细节的 continue-on-error 安全网（待上游修或换系统 freetype），不影响当前 rendering-compat 主线。
> 4. **其余 clean wins（已 LANDED）**：R1085 nbsp(U+00A0) 保留（linebox +10/css-text +1/writing-modes +3）/ R1086 word-spacing 前导间隙（CSS2/text +1/css-text +1）/ multicol R1028–R1080 累计 +~20（column-span:all spanner +8、column-gap:normal +6、fragment clip +4、inline 列溢出 +3、positioned 后代 flush+clip +3）/ border+margin/padding/inset 逻辑属性 WM-aware（R1048–R1049）/ R1058 inline 垂直 margin 归零。
> 5. **plateau definitive（R1091–R1093）**：positioning + floats/floats-clear + abspos §10.3.7 + CSS2/box + margin-padding-clear + css-text-decor fresh scan 全目录确证——残余 100% font-wall / 深结构性（R109 §9.2.1.1 / multicol Phase 2 / baseline-export / writing-mode 垂直），autonomous plateau exhaustive 完成。逐轮详记见 [`archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md`](./archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md) + [`archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md`](./archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md)。


> **★ 2026-06-26 追加（R660–R668 doc-accuracy arc + M7 闭环 + font-wall 彻底定性；supersede 下方 R519-era 收敛叙述）**：
> 1. **M7（渲染器图元覆盖）已完成**：DC-8 CPU 13/13 + DC-9 GPU 13/13 + DC-10 浏览器消费全 13 字段，均附 framebuffer 像素断言单测（R660–R666）。原 goal doc「P0 致命未起步」描述已纠正（M7 milestone section 已收口）。
> 2. **DC-13 legacy-html smoke 里程碑**：20/20 fixture + `make product-smoke-legacy` 门禁（R656–R659）+ nav 子资源（外链 CSS R213 / img R318）✅。
> 3. **font-wall 彻底定性为永久 plateau**：R668 证伪「rustybuzz same-source」（最后未测 font-wall 修复路径）→ product-smoke ~17% diff + reftest 字体噪声为**不可消除**（R388 光栅化≈/R631 选择对齐零变化/R633 死锁分界/R643 fontdue-advance net-negative/**R668 rustybuzz refuted** 五证）。即 goal 的 95% reftest 目标受此永久噪声 + 结构性布局缺口双重约束。
> 4. **当前真实剩余工作**（非 doc）：(a) code 特性——Float/sticky/scroll 布局 + clip-path/backdrop-filter/mask 渲染（已解析未渲染）；(b) ~~DC-14 独立 Oracle~~ ✅ **R669 已落地**（`make reftest-oracle [DIR=...]` 一等 chromium-Oracle 指标；doc-maintainer spot-check `DIR=css-grid`=16/49=32.7% 复现 R560 基线 + self-source ~56.5%/46.5% 假通过对照；默认 reftest 仍 self-ref 作同源参考，reftest-oracle 为独立 Oracle 补充）——DC-14 仍 open 项 = 严格容差三态分类 / 非平凡性检查 / 容差锁定；(c) 结构性布局（multicol/writing-modes/Phase A IFC 统一）。**goal doc 各 DC checklist 现大面积准确**（R660–R670 stale-claim 纠正；R670 收口 R669 DC-14 独立 Oracle 项）。

1. **旧 plateau（R305–R323 / R384，≥10 轮一致收敛）**：在「子集分母 + 旧 lever 族」语境下，**near-pass clean-win frontier / POLLUTED 候选逐项 hunt / fontdue 度量（advance-width / AA / Bold）/ multicol paint 侧与 balance 二分 / baseline-export 3 机制 / column-aware IFC 纯 inline / Phase A font_size 单点** 等 lever 族均 ruled out 或 refuted。下表「已穷尽 / 证伪的杠杆」记录的就是**这一阶段**的结论。**该结论的范围限定**：它覆盖的是 *font/metric 精度类* 与 *POLLUTED-hunt 类* lever，**不**覆盖 R491 之后发现的 CSS-correctness 单点 lever 族（见下）。

2. **★ R491–R511 CSS-correctness lever era（旧 plateau 的反例，现已 CSS2 趋尽）**：R491（画布背景传播 §14.2）打破旧 plateau，此后 ~20 轮产出**一串连续 clean win**，来自一个**未被旧 plateau「ruled out」清单覆盖**的全新 lever 族——**CSS 规范正确性单点修复**：
   - R491 画布背景传播 §14.2（gross failure 修 8 案）
   - R498–R500 abspos sizing（fixed/absolute 全-inset stretch + right/bottom Px 位置；self-source +33、chr<1% +10）
   - R503/R504 CSS Appendix E paint-order（step 6 z-index:auto + 全局 positioned-descendant 延迟；+7/0）
   - R505 opacity<1 建立堆叠上下文（修 R504 回归）
   - R508 background-position 两值轴向（+4/0）
   - R509 `content: attr()` 生成内容解析（+3/0）
   - R510 border 简写重复组件值拒绝（+1/0）
   - R511 CSS §9.7 table-internal+float → block（0 self-flip 但 DC-14 diff 降 70%）
   - R525 batch_fills draw_order paint-order（css-flexbox net +5；修渲染管线核心系统性 bug）
   - R526 abspos flex 子元素不受 `order` 重排（flexbox-paint-ordering-003 +1/0；R525 暴露的 abspos-order 真实 bug，双站点 tree.rs:489+engine.rs:1156 一致修复）
   - （另：R428 flex min-size:auto LANDED；R429 flex shorthand type-awareness LANDED commit `88e11d2b`；R513c/R516c/R517 cascade 负值拒绝族累计 self-source normal-flow+values +10）
   - 反例/refutation：R502 presentational hints（实现正确但 zero-yield，已 revert）

   **R511 扫描结论**：**clean 单点 lever 在全 CSS2 目录已趋尽**（lists/colors 100%；box-display/borders/generated-content = IFC Phase-A；floats-clear = §9.7/margin-collapse/float-sizing/BFC 多结构性；visuren/text = fontdue+IFC）。即 CSS-correctness lever 族在 CSS2 内也走到了尽头。

3. **当前真实状态（R519 后·code/doc 两 agent 独立收敛）**：两阶段 lever（font/metric 精度类 + CSS2 CSS-correctness 类）均已在 CSS2/子集范围内穷尽。R512（CSS2 内）+ R513（§9.7-pattern dead-value 扫描**外推到 8 个非-CSS2 全量目录**，两套不同方法论）**收敛确认**：单会话 rally clean lever 在全目录趋尽。★ **cascade 负值拒绝 vein LANDED + 独立验证**：R513c（0b11a12d，narrow-scope，numbers-units-006 +1）→ **R516c（01f63d70，full-scope reject-to-initial，max-height-001/012/023/034/045/056 +6 normal-flow，doc-agent reftest 复核确认）**→ **R517（3c195635，em/rem/ch/% 全单位泛化，+3 height-089/max-height-067/max-width-089）** = **cascade 负值 vein 累计 LANDED self-source normal-flow+values +10**（R513c +1 + R516c +6 + R517 +3），supersede R512 §4「非 clean slice」裁决 + supersede R513c 的 preserve-prior framing——cascade 负值 vein 累计 substantial 翻转 WPT「negative illegal」簇，是 reopen CSS-correctness vein 的实例；另 **R518（6a1245e9）§9.7 inline-level blockification on float LANDED +5 floats-clear**（补全 R511 partial，doc-agent targeted reftest 复核确认）。**★ code/doc 两 agent 独立收敛（单会话 clean lever 空间已耗尽）**：R519（dda90da9，code-side read-only）扫 float-replaced-width/visuren/clear-applies-to/generate/css-sizing-colors 5 区全 ruled out（structural / fontdue / presentational-hints-blocked / is_block_level-load-bearing / abspos-static-pos-entangled），明言「levers exhausted, remaining = multi-session structural」；doc-side 3 ruling 亦全闭——§9.7 position-blockification（72f9f7d7，~0 driving test）/ percent max-width/min-height clamping（cc543587，R119 analog ~0 yield）/ intrinsic-keyword sizing（15ff5fa1，taffy-blocked）。**R515 flex-grow/shrink 负值拒绝 = ⛔ R524 运行时 REFUTED（FALSE LEVER）**——R515 §6 读 taffy `sum_flex_grow > 0` 门控误推「负值影响分布」，实为门控使负和短路 no-growth；R524 实现 seam + A/B = **BYTE-IDENTICAL ZERO-YIELD**（flex-grow-003 1.04% / flex-shrink-002 3.12% WITH==WITHOUT fix，primitives 实证 test1/test2 25px 未 grow 两种情况相同），fix 已 REVERT。flex-grow-003 paint-order 根因（#cover 绘于 flex items 前）后由 **R525 batch_fills 修复**（现 PASS）；flex-shrink-002 由 **R530** 修复（负值→initial）——driving tests 虽 PASS，R515 负值拒绝 lever 本身仍 zero-yield（R524 证伪）。**纠正 R517「R515 单会话唯一 genuine lever」——非 genuine；单会话 clean lever 现真穷尽**。剩余 forward-motion 见文末「下一步」——按 `master.md` 状态记录 + `CONTINUE: <下一步>` 续跑协议推进**跨会话架构任务**（Phase A IFC 统一 ≈ 释放 ~2200 fontdue/IFC near-threshold 案 + welcome/morning 文本度量残余，迄今最大 lever）；或 font-feature same-source 接入（R513 精确定位入口 `shaper.rs:82` 空 features）。

   **★ R525–R546 推翻「单会话 clean lever 全穷尽」断言（doc-maintenance 复核）**：上方 point 3「单会话 clean lever 全穷尽」与 R511「clean 单点 lever 在全 CSS2 已趋尽」是在 R519/R511 时间点下的判断，**但其后 R525–R546 又连续产出 ~15 个 clean win**（R525 batch_fills paint-order +5 / R526 abspos-order +1 / R527–R532 img-asset import +19 / R528 transferred-size +5 / R530 flex-shrink min-size +9 / R536 flexbox_flex 双 bug +52 / R537 nowrap inline-block +9 / R542 welcome-regression-fix +2 / R544 ex 单位 +31 / R545 inherit 关键字 +3 / **R546 缺失 ref 导入 +149** / R547 ex 0.8em Ahem 修正 +1 / R548 col/colgroup min/max-width +4 / R502-reopen presentational hints +7 / **R549-land border-width medium（实验验证 +14 净，block C 解死路，clean-land 进行中）**），其中 R544/R545 在 normal-flow（R511 曾判「趋尽」的目录）仍净 +34。**结论修正**：真正穷尽的是 **(a) font/metric 精度类**（advance-width / AA / Bold，多轮证伪）与 **(b) 结构性 multicol/baseline/IFC-Phase-A 类**（须多会话）；**未穷尽**的是 **(c) CSS-correctness 单点 + 缺失资产/参考文件类**——它开放-ended，每轮扫描都能补新缺口（R546 缺失 ref 即典型：渲染本就正确，缺的是分母真实性）。rally 优先级 = 持续扫 (c)（ROI 最高、风险最低），(a)(b) 留多会话。

   **★ R583–R588（2026-06-25，sizing 子分支连续 6 clean win 后全闭环）**：R546/R549 next-lever 队列的 sizing 子集 = R548/R177b/R119 谱系**全对称补全**（cell/table × W/H × min/max × percentage + 百分比高度三件套 R119+R587+R588），R589 scoped reftest 实证 `display:table` 013 簇 6/6 PASS。**残余 014 inline-table 经 PIL 定性 = inline-level box 渲染缺口（非 sizing，defer Phase A）**。即 (c) 谱系内的 sizing 子分支已穷尽，剩余 (c) 仅余缺失资产/参考文件类增量（开放-ended 但边际递减）；项目回 firm structural plateau，forward motion 全多会话架构（Phase A IFC 统一 / multicol fragmentation / baseline-export / writing-modes 轴 / inline-level box-model）。

   **★ R590–R591（2026-06-25，(c) doc/research 单会话 lever 全穷尽）**：R590 分母真实性双半扫描 = missing-ref **0/10232 彻底穷尽**（R546/R551/R552/R553 闭环）+ missing-asset **63 test/24 资产 hand-off code agent**（writing-modes swatch/quadrant 簇主导；资产导入工程非 doc/research 渲染 lever）。R591 non-CSS2 CSS-correctness 单点扫描 = **EXHAUSTED**（converter 层 `computed_style_to_taffy` comprehensive 无 applied-but-wrong 值 bug；grid near-miss 2 案 selfsource PASS、>1% 全是 chromium-oracle 亚像素噪声；scrollbar-width hardcoded 0 footprint）。即 (c) 谱系内 **sizing（R589 闭）/ missing-ref（R590 闭）/ non-CSS2 CSS-correctness 单点（R591 闭）三 doc/research 子分支全穷尽**；唯一剩余 actionable = R590 missing-asset code-agent 导入。**项目回 firm structural plateau**：forward motion 全多会话架构（Phase A IFC 统一 / multicol fragmentation / baseline-export / writing-modes 轴 / inline-level box-model 含 014 inline-table）。

**★ R592–R645（2026-06-25→06-26，R591「firm plateau」判定的精细化）**：R591「项目回 firm structural plateau」**被其后连续产出部分推翻**——单会话 clean lever 经两条 narrow vein（css-text near-miss / per-fragment inline）又产出 R639/R644/R645 + 行盒度量共 ~+100 reftest，**R647/R648 双双收口后回归 firm plateau**（字体度量/oracle 轴 + 结构性 multicol/baseline/IFC 轴）：

- **行盒度量（font-metric-independent 部分）LANDED +29 reftest**：R630（`d31cf03a`）修 paint Path B 多行 y 堆叠（解除 R246 限定，net +24，**直修用户痛点「文字堆叠看不清」**）+ R632（`0911a2ac`）修 Path B line-height override 忽略 CSS（net +5，welcome 16.98→16.16%）。**R633 定论 font 方向 4 层全证伪**（bolding R229c / selection R631 / rasterization R388 / metrics=Phase A 死锁）；font-metric-independent 行盒部分修完，残余（morning 中文 +0.99pp = frag.height 字体自然行高）须 IFC↔FontLoader 接口（Phase A 多会话）。
- **★ per-fragment inline-painting vein（首个 landed Phase A inline-fragment narrow slice）**：R639（`c7ff730b`）per-fragment inline-bg **owner-height 索引** LANDED（+13 self-source：text-indent-wrap-001/002 + content-175 + css-text +10）。R636（blanket）/R638（gated）两轮 revert 后，R639 用 `Painter.inline_heights: NodeId→box.height` 预扫描攻克 inline-ownership split blocker——**该索引是通用 Phase A 桥接机制**，后续 per-fragment inline border/outline/shadow（须有 driving test；R640 证当前 WPT 无 inline-border 多行测试故 defer）+ 真实 font 度量可复用。**实证 narrow slice 非「全有或全无」**。**R647 复核（text-decoration）**：text-decoration 已是 per-fragment（paint caller `text.rs:1107` 在 fragment 循环内，用 `frag_base_x` + 逐 fragment `text_width`，符合 CSS2 §16.3），非 R639-class bug——per-fragment inline vein 下一属性（bg LANDED R639 / border·outline·shadow 无 driving test / text-decor 已正确）单会话穷尽，owner-height 桥下一用途 = 真实 font 度量（Phase A 多会话）。
- **css-text near-miss 扫描 vein（R647 收口：3 clean win 后穷尽）**：R644（`bc512083`）Cc 控制字符可见占位 LANDED（control-chars 60/64，+~56；fontdue 对 Cc 返回 .notdef 空形致 test==空 ref 假败，渲染 em-square 占位即修）；**R645（并行 code agent LANDED `7ada4552`）** SEA 词典分词文字（Thai/Lao/Myanmar/Khmer）按字符断行 fallback **+7**（line-breaking-024~027 + word-break-normal-002/003/th-001）+ **附带修 pre-existing bug**：IFC 测量路径（`measure_text_content`/`remeasure_text_with_float_exclusions`/`remeasure_inline_only_containers` 3 处）从不传 white-space（`no_wrap` 恒 false）→ pre/nowrap 容器 box 高度被系统性高估（被「单 token 无法换行」掩盖）；修后 pre 容器测量与渲染一致，为后续 pre 相关修复奠基。**R647（`328bd1a3`，read-only）逐簇评估剩余 12 个 0.00% FAIL mismatch 簇 = 全非 clean 单 session lever**，分三类：**(a) feature implementation**（text-autospace 4 测全未实现但 advance 数学匹配 ref 0.125em×2，29 测 blast radius 须逐测 A/B，**最高潜在 yield +2~4**；-webkit-text-stroke / text-wrap-balance 精度 / skip-ink-vertical 组合；hanging-punctuation/word-space-transform/text-spacing-trim = CSS Text L4 + chromium 支持弱 = oracle-neutral defer）；**(b) entangled whitespace/pipeline**（control-chars VT/NEL/FF——collapse_whitespace + split_into_words 双重按 Rust 空白分割，CSS 仅认 TAB/LF/FF/CR/space，修须改两处核心 + 影响 NBSP）；**(c) 须 pipeline trace**（object-replacement U+FFFC）。forward 选项 = text-autospace feature（~80 行多 crate）/ 0.00% 簇方法论外推到 css-text-decor·writing-modes（**R652 实测关闭**：css-text-decor 仅 4 簇 = skip-ink moderate-to-significant feature（font API `query_glyph_metrics` 仅返 `(glyph_id, advance)`、无 per-glyph ink bounds——须先扩 font 抽象暴露 ink bbox + decoration paint 加 orientation 感知裁剪；R655 复核，非 quick win）/ writing-modes **0 簇**，全 Match-failed 5-88% 结构性 vertical block-flow R631/R109 territory → 0.00% 簇 quick-win 全目录穷尽） / 多会话 Phase A。详见 [`evidence/r647-csstext-cluster-scan-exhausted-2026-06-26.txt`](./evidence/r647-csstext-cluster-scan-exhausted-2026-06-26.txt)。
- **font-wall oracle blocker 精确分解（R642/R643，方向 CLOSED）**：text-heavy（text-indent-wrap/morning/welcome）oracle diff 主因 = **line-count（ZW 8 行 vs chr 6 行，advance-width 差）非 R639 bg**（PIL 实测 8v6≈6.2%）。R643 fontdue-advance 攻击 net-negative（fontdue≠HarfBuzz hmtx 系统差）→ fontdue-advance 单会话死路。**未测的 NotoSansCJK+fontdue-advance 组合 = 最高 ROI 多会话 narrow slice**（同字体 fontdue hmtx≈Skia HarfBuzz hmtx，Latin 无 shaping 差；R631 用启发式 advance 零变因启发式不改变 line-count）。

**精细化 plateau 判定**：firm structural plateau = 字体度量/oracle 轴（R633/R642/R643，须 Phase A IFC FontLoader）+ 结构性 multicol/baseline-export/IFC-Phase-A 轴（须跨会话推进）。**单会话 clean-win vein 双双收口**：(a) css-text near-miss（R644 +~56 / R645 +7 LANDED，**R647 逐簇评估剩余 12 全非 clean → 穷尽**，残余最高 = text-autospace feature +2~4）；(b) per-fragment inline painting 经 owner-height 桥（R648 复核 text-decoration 已 per-fragment → vein 单会话穷尽，下一用途 = 真实 font 度量 Phase A）。**残余单 session 选项** = text-autospace feature / 0.00% 簇方法论外推到 css-text-decor·writing-modes（**R652 实测关闭**：css-text-decor 仅 4 簇 = skip-ink moderate-to-significant feature（font API `query_glyph_metrics` 仅返 `(glyph_id, advance)`、无 per-glyph ink bounds——须先扩 font 抽象暴露 ink bbox + decoration paint 加 orientation 感知裁剪；R655 复核，非 quick win）/ writing-modes **0 簇**，全 Match-failed 5-88% 结构性 vertical block-flow R631/R109 territory → 0.00% 簇 quick-win 全目录穷尽）；**主轴 forward motion** 仍是按 rally 续跑协议推进 Phase A IFC 统一（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)）。

**基线（R427 css-multicol 扩到全量后——**10/10 目录分母全量（R626-R627 去子集化完成，含 writing-modes/css-text/CSS2）**；其中 7 目录有聚合 self/oracle 测量；strict/oracle broad 仍 pre-grid-expand 子集口径；7 目录聚合 oracle 629/1726=36.4% 见顶部主基线）**：

- self-source 全量目录：css-grid **32/48=66.7%（R546m 复测）** + css-position **63/95=66.3%（R546m 复测）** + **css-tables 77/112=68.8%（R546m doc-maintenance 复测确认，[evidence](./evidence/r546m-csstables-77-reverify-2026-06-24.txt)）** + css-flexbox **368/496=74.2%（R525–R542 +108，R546 doc-maintenance 复测确认，[evidence](./evidence/r546-flexbox-368-reverify-2026-06-24.txt)）** + **css-multicol 195/451=43.2%** + **css-text-decor 244/246=99.2%**（strict 76.4%）+ **css-fonts 282/284=99.3%**（strict 88.3%）。**7 目录聚合 self-source 1148/2032=56.5%**（文字容差抬升，strict 口径更低）；**混合口径（7 全量 + 2 子集），非全量真通过率**
- **7 目录全量 chromium-Oracle 真一致 47.5%（R1201 fresh post-font-wall 全幅，旧 stale 36.4% / R1199 partial 39.3%）**（font-wall 全 dir 影响，text/layout dir 皆复测 R1196/R1198/R1199/R1201）：grid 20/49=40.8%（R1201 fresh，旧 19/48=39.6%） / position 57/97=58.8%（R1201 fresh，旧 36/95=37.9%，+20.9pp） / tables 74/115=64.3%（R1201 fresh，旧 49/112=43.8%，+20.5pp） / flexbox 298/497=60.0%（R1201 fresh，旧 251/496=50.6%，+9.4pp） / **text-decor 118/242=48.8%（R1198 fresh post-font-wall，旧 70/242=28.9%）** / fonts 100/282=35.5%（R1196 fresh，旧 98/282=34.8%） / **multicol 157/452=34.7%（R1201 fresh，旧 106/451=23.5%，+11.2pp，仍最低）**（chr<1%，聚合 **824/1734=47.5%（R1201 fresh post-font-wall 全幅，旧 stale 629/1726=36.4%）**，multicol 下拽自 6-dir 49.7%）；布局类 5 目录 606/1210=50.1%（R1201 fresh） vs 文字类 2 目录 **218/524=41.7%（R1199 fresh）**；子集 42.1% 广义口径对 self-source 严重高估（下条 200/475 为 pre-grid-expand 子集口径）
- self-source strict：**295/490 (60.2%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径，pre-grid-expand）
- chromium-Oracle 广义一致：**200/475 (42.1%)** @ chr<1%（R391 锁定诚实基线）；严格 self-pass&chr<1% **177/475 (37.3%)**；污染 46.5%。**⚠️ R388 报的 43.2% 含 R389 暴露的 5 假一致（flexbox blank-blank），已纠正为 42.1%；pre-R388 35.8% 被 108 损坏 Ahem oracle 压低已修**
- 产品 smoke（R630/R632 后；以文末「当前状态概览」为准）：welcome **16.16%**（R632 line-height override，自 16.98%）/ wintertc 13.59% / morning-work 800×600 **18.15%**（R630 多行 y 分行后中文行级度量差异诚实显现，自 16.41% 上升；**非结构退步**）/ fullpage 48.65%（全文本度量结构性，非图片/CSS 缺口）

> **⚠️ 基线陈旧性（R546 doc-maintenance 复核）**：上述聚合基线是 **R424/R524 前后快照**，**未折叠 R525–R546 的 clean win**。本轮已实证修正 self-source 陈旧项（test-guard --jobs 3 复测）：**css-flexbox 260/496(52.4%)→368/496(74.2%)** + **css-grid 31/48(64.6%)→32/48(66.7%)** + **css-tables 75/112(67.0%)→77/112(68.8%)** + **css-position 61/95(64.2%)→63/95(66.3%)**（4/7 布局类目录已对齐；evidence r546-flexbox/r546m-cssgrid/r546m-csstables/r546m-cssposition-*-2026-06-24.txt）。**css-multicol(195/451)、css-fonts(282/284) 仍 stale**（multicol 大目录仅差 1 + 结构性低值；css-fonts R546→284/286 微差），待后续轮补测。R546（ab2c6f7a）导入 14 个缺失共享 ref 后 CSS2 子目录新口径：**normal-flow 636/746(85.3%)（R548 col lever +4）/ R502 **LANDED** A/B 证 **+7/0**（normal-flow +3→639/746 / backgrounds +3 / borders +1；positioning 0；block C 独立阻塞） / lists 151/151(100%) / css-text 375/408(91.9%) / css-fonts 284/286(99.3%) / generated-content 207/225(92%) / writing-modes 613/788(77.8%)**（+149 self，12 诚实暴露）。**oracle / 7-dir 聚合百分比仍 stale**（待 cross-validate 重跑，OOM-prone 故本轮未重测）；逐目录增量以**顶部 round-detail 为权威**。self-source strict 295/490(60.2%) 亦为 pre-grid-expand 子集口径，含 flexbox 子集故未随 R525–R542 上移——下次 strict 复跑须用全量目录。

> **post-R326 strict 再复验（2026-06-19 doc-maintenance read-only，test-guard 包裹 `ZERO_REFTEST_STRICT=1 ... reftest-upstream`）**：strict 仍 **295/490 (60.2%) / 195 fail**（zero drift vs R323）——确认 plateau 在 DC-14 诚实指标上成立：R324（position:fixed）/R325（img aspect）/R326（sticky）三处 DC-11 correctness 修复**均未**把任一 strict-fail 翻成 strict-pass（loose 亦经三 commit 各自复验 438/490 零回归）。详见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)。**顺带纠正**：旧文「295/490 (60.4%)」百分比过时（60.4% 为 R308 前 296/490 值；R308 font-size% 修复使 strict 296→295 后未同步百分比，295/490=60.2%）。

**已穷尽 / 证伪的杠杆（勿再以单会话重试）**：

| 杠杆 | 裁决轮 | 结论 |
|------|--------|------|
> **R125–R354 早期 plateau ruled-out 杠杆（near-pass / POLLUTED-hunt 三趟 / Phase A font_size 多轮 / multicol paint·balance·column-aware / baseline-export 3 机制 / advance-width plumbing·paint 单侧 / DC-9 blend / font-weight·@font-face·rustybuzz shaping / taffy 0.11 / clear+margin-collapse / WM-1 三角度 / backdrop 非 CSS / wintertc @media / 近-miss·属性·pixel-dump 审计）共 24 行已归档**：[`archive/ruled-out-levers-r307-r354.md`](./archive/ruled-out-levers-r307-r354.md)（内容 100% 保留，其综合结论见上方「综合裁决」point 1「旧 plateau」）。下列保留 **R355–R430**（Phase A narrow-slice 突破 R355/R358/R362/R364 + DC-14 per-dir 全量导入 R388–R430，近期更相关）。
| R355 paint 侧 line.y 补全 + ifc-009 残余根因定位（二次诊断修正） | R355/R356 | R355 paint 侧 line.y 偏移（text.rs:832）是 Gate 2（ca14d05）的 load-bearing 配套——stash 对照证 HEAD Gate-2-only ifc-008 4.17% FAIL → +fix **0.00% PASS**；self-source **439→440/490 净 +1 零回归**（commit 787677e）。R356 二次诊断修正 ifc-009（2.08%）真根因：**首轮「line_height=20 传播」是 remeasure 路径伪影**；paint 存储片段探针实证渲染路径 line_height 正确（h=100），真根因=**跨 block float 侵入未实现**——inner-div（与 float div2 兄弟，同 div1 子）的 IFC 只看自身直接 float 子，div2 不传播到 inner-div IFC → line 1 用满宽 300px 容 "X X"（第 2 个 X 恰被 float 覆盖致 top 巧合正确），line 2 仅 1 X → 右下 100×100 应蓝实橙。属 M4 完整 float 布局 / Phase A 所有权分裂，单会话高回归风险（耦合 R355 float guard），defer。详见 [`evidence/r356-ifc009-lineheight-float-overlap-pinpoint-2026-06-20.txt`](./evidence/r356-ifc009-lineheight-float-overlap-pinpoint-2026-06-20.txt)。**结论**：R355 是真 +1 clean win（已提交）；ifc-009 真根因=跨 block float 侵入（Phase A/M4 第三机制，独立于 font 度量），未修 |
| 低 diff 失败第三处 Phase A 簇确认（multicol-count-computed-004 存储丢色） | R357 | 像素级根因（无代码变更）：multicol-count-computed-004（2.00%）换行正确，唯一 diff=color——存储 IFC 片段（InlineLayoutFragment）**不携带 color 字段**，paint 存储路径所有片段用容器 color（black），REF 期望各 span 自身 color（粉/橙/紫/灰）。auto+auto 非多列（compute_column_info 正确返 None）。R357 当时判「per-fragment color 修复不安全（耦合 R335 abspos 回归）」**已被 R358 推翻**（加 abs-pos guard 即解）。三处独立诊断共享根因=paint 阶段丢/错 layout 的 per-fragment 语义。详见 [`evidence/r357-stored-ifc-color-loss-phase-a-cluster-2026-06-20.txt`](./evidence/r357-stored-ifc-color-loss-phase-a-cluster-2026-06-20.txt) |
| **R358 per-fragment color + abs-pos guard（plateau 第三次突破，+1 clean win）** | R358 | **纠正 R357「per-fragment color 不安全」结论**——加 **abs-pos guard**（abs-pos/fixed 片段保留容器 color）即解 R335 回归：`render_fragment!` 宏（text.rs 非多列路径）解析每个片段所属元素 color，position:Absolute/Fixed 者用容器 color（维持当前行为，不激活 R335 的 abspos 绿 X 错位显眼化），其余用自身 color（镜像多列路径 line 1033 frag_color）。验证：**multicol-count-computed-004 FAIL→PASS（2.00%→1.00%）**；self-source **440→441/490 (90.0%) 净 +1 零回归**（abs-pos-non-replaced-vrl/vlr 簇不变，writing-modes 53/59 持平；唯一他例 float-lft-orthog-htb-in-vlr-002 6.68%→6.53% 仍 FAIL=微改善非回归）；engine 1146/0（+1 单测 test_paint_per_fragment_color_for_spans）、clippy/fmt 干净。**启示**：R335 net-negative 是因无 guard 全量应用 per-fragment color；scoped guard（abs-pos 例外）把「Phase A 耦合」降为「单点 clean win」。abspos 文本错位（R336）仍 Phase A，但 per-fragment color 本身非阻塞 |
| border-bottom-width-006（2.86%）=匿名块盒生成缺失（R109 谱系），非 clean win | R359/R360/R361 | 三轮诊断收敛：① R359 误判渲染管线合成；② R360 纠正=两个 fill 均正确渲染（#test border (8,51,96,96) black、#reference bg (12,16,96,96) black，像素 (50,100) 确认黑），真根因是 #reference 定位到 (12,16) 而非紧邻 #test；③ R361 LAYOUT_DUMP + 代码追踪精确定位：body 含混合内容 `<p>`(block) + #test/#reference(inline-block)，**CSS §9.2.1.1 要求 inline-blocks 被匿名块盒包裹**（独立 IFC 容器），ZeroWeb 不生成此匿名 wrapper——inline-blocks 作为 body 直接子，`adjust_inline_block_positions`（engine.rs:880）对 body 跑 IFC 覆盖 taffy 位置，但 IFC 不感知 `<p>` block 兄弟 → #reference 落 line1(y=16 body 顶)、#test 落 line2(y=51)，顺序反转。taffy 块堆叠 / IFC 误定位**均错**，正解需匿名块盒生成（R109 匿名块级联谱系）。style/layout-box/paint 全正确。属匿名块盒生成架构（与 R255 ua_default_display 同 R109 谱系），单会话高风险。**结论**：clean-win 面经连续多轮（R359-R361）确认穷尽，forward motion = 多会话 Phase A / 匿名块盒生成 |
| **R362 跨 block float 侵入传播（plateau 第四次突破，+1 clean win）** | R362 | Phase A 第四个 narrow viable-slice：CSS float 侵入——祖先 BFC 内的 float 应侵入未建 BFC 的后代 block 的 line box。`compute_final_inline_layouts` 加 `ancestor_floats: &[FloatExclusion]` 参数，递归时把（祖先 float + 本容器直接 float）按 `f.y - child.y` 平移到子节点 box 坐标系传播（FloatExclusion 无 x 字段，IFC 仅按 left/right+width 缩减行盒）；**排除子节点自身**（float 不在自身 IFC 排除自己——float-005 回归实证后修正）。验证：**ifc-009 FAIL→PASS（2.08%→0.00%）**；self-source **441→443/490 (90.4%) 净 +1 零回归**（首轮 float-005 因 float 自排除缺失回归，加 self-exclusion 后恢复；全量无其它回归）；layout-engine 891/0（+1 单测 `test_r362_float_intrusion_propagates_to_sibling_block_ifc`）、engine 1146/0、clippy/fmt 干净。**启示**：Phase A 谱系（R355 多行存储 / R358 per-fragment color / R362 float 侵入）连续三个 narrow viable-slice 成功——paint/layout IFC 语义丢失可逐项 scoped 修复，非全有或全无；剩余 Phase A 项（abspos 文本错位 R336）同谱系可继续 |
| 上一轮 CONTINUE 标记的 3 个「未深查」候选 + 1 个 false-pass polluted 案精确诊断 | R363 | baseline 复验 self-source **442/490 零漂移**；4 案 REFTEST_DUMP+PIL 诊断 **0 clean win**（hit-rate 0/4，与 R352/R354 一致衰减）：① flex-abspos-inset-nested-001/002（8.33/18.75%）= **2 互相依赖 bug**（abspos §10.3.7 shrink-to-fit 缺失→inner-flex w=0 + flex 替换元素主轴固有比尺寸，单修任一无效）；② fixed-table-layout-with-percentage-width-in-flex-item（11.20%）= flex definite-size resolution（flex item 含 width:100% table 后代塌缩到 2px，Mozilla bug 1469649 谱系）；③ **multicol-contained-absolute（chr 16.33% / self 0.00% = FALSE PASS，首次定位根因）**= multicol `column-fill:balance` 不平衡单个 200px 子（ZW 392x200 单列 vs chromium 784x100 跨两列平衡），test/ref 同源假通过；④ multicol-fill-000/count-002 = multicol 平衡/分布结构性。**方法论纠正**：自写 PNG 解析器 alpha/filter 字节 bug 误判 oracle 为「黑图」，PIL 复核 oracle 内容有效——polluted 案诊断须 PIL 对 oracle。详见 [`evidence/r363-flagged-candidates-structural-2026-06-20.txt`](./evidence/r363-flagged-candidates-structural-2026-06-20.txt)。**结论**：clean-win 面经 R352/R354/R363 三轮 0/N 衰减穷尽，forward motion 须多会话架构 |
| **R364 table 显式 width 列冻结 + min-content floor（CSS 正确性修复，net-neutral self-source / chromium-Oracle 改善）** | R364 | table-cell-width-0（self 31.57% FAIL）诊断：definite-width 表的显式 width 单元格列被扩展填满块按比例撑大（td.big-positive 20px→529px）。修复 `compute_column_widths`（table.rs）：① 扩展填满块改为显式 width 列冻结（col_explicit 标记，Pass 1 收集），仅 auto 列按当前宽比例吸收剩余（全显式时回退比例扩展）；② `cell_used_width` 显式分支 `base.max(intrinsic)` floor 到 min-content。验证：TEST 侧列宽全修正（zero=9.6/positive=9.6/big=20，normal 吸收剩余）；**ZeroWeb-TEST vs chromium-Oracle = 11.14%**（DC-14 真指标，修复前 TEST 明显错）；self-source 全量 **442/490 净中性零回归**（table-cell-width-0 仍 29.63%——REF 侧 `width:fit-content` on flex container 渲染满宽，taffy 0.7 blocker R304 DEFERRED，TEST 正确但 REF 错=同源假阴性）；layout-engine 893/0（+2 单测）、engine 1146/0、clippy/fmt 干净。**裁决**：CSS 正确（显式列不吸收剩余 + 列宽 min-content 下限均规范行为），服务 DC-14 真指标方向，待 fit-content-on-flex 解锁即贡献 PASS；R363「fixed-table-in-flex 同 flex definite-size 谱系」结论对【flex 内 table】仍成立，本修复是【table 自身】显式列分布，独立 |
| plateau 再确认（4 新角度）+ 系统性 REF-side blocker 洞察 | R365 | baseline 复验 self-source **442/490 零漂移**；4 新角度诊断 **0 clean win**：① fit-content/max/min-content 关键字全 flex/grid（taffy-blocked），无 block/inline-block 失败案用 → 无杠杆；② **min-max-size-table-content-box（36.34%）= TEST+REF 双侧多 bug + spec 冲突**——min-height 表格 border-box（h=50 应 66），但改 content-box 回归 min-height-table（csswg-drafts #5336 两案冲突）+ REF inline-block shrink 不生效（R180 仅 definite-width 子元素生效）；③ multicol-columns-001 = multicol wrapping 精度（R128 结构性）；④ inline-block shrink gap 仅 1 失败案 REF 且双侧阻塞 → 零杠杆。**系统性洞察**：多失败案（table-cell-width-0 / min-max-size-table）TEST 侧可修对但 self-source 因 ZeroWeb 自渲染 REF 错（fit-content-on-flex / inline-block shrink）而**假阴性** → 实证 DC-14 self-source 含系统性假阴性。详见 [`evidence/r365-refside-blocker-insight-2026-06-20.txt`](./evidence/r365-refside-blocker-insight-2026-06-20.txt)。**结论**：clean-win 面经 R352/R354/R363/R365 四轮 0/N 衰减彻底穷尽，forward motion 须多会话架构 |
| ifc-011 簇 IFC 表面修复三维度探针（margin / border-box 尺寸 / width-shrink） | R366/R367/R368 | R366（inline vertical-margin 归零）net-neutral 回退；R367（inline-block border-box 尺寸 in IFC）net-negative 回退（ifc-011 11.27→13.73%）；**R368 重开「宽度维度」新表面修复并锁定保留**——只读探针（LAYOUT_DUMP）定位 ifc-011 真根因：span w=784 满宽拉伸（R180 shrink 因 span.children 空→content_max_w=0 失败）是 PRIMARY 缺陷，div h=60 是 taffy 块容器高度（解耦）。修复 `shrink_inline_blocks_to_content` 改用 `intrinsic_sizing::box_content_max_width`（按 DOM text_content + 字体度量，处理无子盒纯文本）：ifc-011 **11.27→1.23%**（-10pp），self-source **442/490 净中性零回归**，+1 单测。border-box ib_sizes 配套实验证伪（net -1，multicol-dynamic-change 0.97→1.05% 翻 FAIL，回退）。残余 1.23% = span2 x 重叠（需 border-box -1）+ glyph 度量 + height-grow（未试，对翻 PASS 无杠杆+cascade 风险）= 多会话。**R369 DC-14 升级**：cross-validate vs chromium-oracle 实测 R368 是真 DC-14 大胜（ifc-011 z_vs_chr 12.30→2.22%，非 self-source 假象）；border-box 终局证伪（z_vs_chr 2.22→2.50% +0.28pp 真 chromium 退步，x 位置与 glyph baseline 耦合），三层证伪彻底关闭。详见 [`evidence/r368-inline-block-text-shrink-2026-06-20.txt`](./evidence/r368-inline-block-text-shrink-2026-06-20.txt)、[`evidence/r369-borderbox-dc14-refutation-2026-06-20.txt`](./evidence/r369-borderbox-dc14-refutation-2026-06-20.txt) |
| DC-14 全失败案扫描：false-negative 亦结构性 | R369b | 对全 48 self-source 失败案跑 DC-14 chromium-Oracle 扫描（REFTEST_DUMP + cross-validate.py），找「self-fail 但 z_vs_chr 低」（false-neg：test 已≈chr 仅 ref 发散）的易修 ref bug。5 候选逐案证伪均结构性：flex-abspos-inset-nested-001/002（chr 0.73/0.74%）像素分析揭穿 ZW-test 与 chromium **均退化**（非 200×200，z_vs_chr 低仅因两者主体皆白）；baseline-multi-line-horiz-003/004 = baseline-export 聚类（卡点#4）；box-offsets-rel-pos-vlr-005 = WM 结构性。**结论**：false-negative 亦结构性，self-source 非因易修 ref bug 人为偏低，反向印证 plateau；DC-14 方法论=可信判据（后续修复应一律用 z_vs_chr 验证）但不改 forward-motion 结论。详见 [`evidence/r369b-dc14-scan-falseneg-structural-2026-06-20.txt`](./evidence/r369b-dc14-scan-falseneg-structural-2026-06-20.txt) |
| **R388–R430 DC-14 per-dir 全量导入 era（oracle Ahem 修复 R388 / 图片路径 R389 / text-emphasis line-box 阻塞 R392-R393 / grid 缺口簇+特性扫描 R396-R400 / css-position·tables·text-decor·fonts·flexbox·multicol 6 目录 pre-confirm+全量折回 R404-R430）共 18 行已归档**：[`archive/ruled-out-levers-r388-r430-dc14-perdir.md`](./archive/ruled-out-levers-r388-r430-dc14-perdir.md)（内容 100% 保留）。**汇总 oracle**（grid 39.6% / position 37.9% / tables 43.8% / text-decor 28.9% / fonts 34.8% / flexbox 49.2%，聚合 629/1726=36.4%）见上方「基线」节；**关键遗留裁决**：text-emphasis 缺失=Phase A line-box 死锁（R392/R393，勿单会话重试）；flex min-size:auto=ZeroWeb-side clean-win lead（R428，**已 LANDED R435**）；font-features 解锁=rustybuzz 生产接线须 Phase A 同源（R331/R332）。 |

**剩余 forward motion（R395 复排，DC-14 分母 gap 发现后）**：

> **⚠️ 子集范围警示（R395）**：下述「渲染架构」轨道（2-4）的目标都是子集分母下的失败聚类。R394 实测当前导入仅上游 **~5-6%**（503/~8000-10000），R384「单会话 clean-win 47/47 穷尽」、R351-R393 的聚类归因**全部基于此子集**。全量集合含未检失败模式，clean-win 面与各架构轨道的真实 ROI 都须在去子集化后重新评估——**不可把子集结论外推为全局穷尽**。

1. **【优先·8 目录已完成】DC-14 分母去子集化** — gating DC-14 硬门禁（goal line 317/843）：达标判定前必须用上游每目录**全量** reftest。最可操作的多会话增量 = Phase 2 目录全量导入（grid ✅ / position ✅ / tables ✅ / **flexbox ✅** / **multicol ✅** / text-decor ✅ / fonts ✅ / **writing-modes ✅**）→ 最后 css-text + CSS2（~5000-7000）。每批 reftest + chromium-Oracle 复测，监控真通过率。**状态**：**8/9 目录全量完成（5 布局类 + 3 文字类）**——R401 grid（62.5%/39.6%）+ R404 position（64.2%/37.9%）+ R408 tables（67.0%/43.8%）+ R425 flexbox（49.6% self / 49.2% oracle·最高）+ **R427 multicol（43.2% self / 23.5% oracle·布局类最低）** + R417 text-decor（99.2% self / 28.9% oracle）+ R422 fonts（99.3% self / 34.8% oracle）+ **R457 writing-modes（78.0% self / 5.6% oracle·全 9 目录最低，迄今最大下拉）**；**8-目录全量 oracle 26.8% 真一致**（聚合 673/2512，writing-modes 5.6% 把 7-dir 36.4% 下拽至 26.8%=迄今最大单目录下拉，超 multicol）。工具链（`discover-reftests-authoritative.py` 子目录递归 1568cb0 + git/trees recursive=1 10db810（每目录 2 调用，token gate 移除）+ `capture-oracle-per-dir.mjs`）可复用于余 1 目录。**下批：css-text（第 9 目录，完全 unblocked per R460，1914 test 文件规模；全量 dry-run + import + reftest + oracle）→ 最后 CSS2（~5000-7000，亦 2 调用）**。〔**R514 pre-confirm 已备基线**：本地 grep 1302 test-side reftest（白空间 354/i18n 158/text-transform 106/word-break 101/line-break 75/text-align 73/hyphens 55 + CSS Text 4 ~122）；核心 CSS Text 3 特性（white-space 全 6 值/word-break/text-transform/justify）**已接线**，未实现簇=line-break CJK(75)/hyphens(55,R513 dead-value)/CSS Text 4(~122)/shaping(28,rustybuzz)；**预测 oracle ~25-33%（straddle text-decor 28.9%）**，0 新 clean lever（精度 Phase-A + 未实现 CJK/Text-4 封顶，与 R513 exhaustion 一致）。详见 [`evidence/r514-csstext-preconfirm-2026-06-23.txt`](./evidence/r514-csstext-preconfirm-2026-06-23.txt)。〕详见 [`evidence/r427-cssmulticol-full-2026-06-22.txt`](./evidence/r427-cssmulticol-full-2026-06-22.txt) + [`evidence/r425-cssflexbox-full-2026-06-22.txt`](./evidence/r425-cssflexbox-full-2026-06-22.txt) + [`evidence/r421-cssfonts-full-2026-06-22.txt`](./evidence/r421-cssfonts-full-2026-06-22.txt) + [`evidence/r412-csstextdecor-full-2026-06-22.txt`](./evidence/r412-csstextdecor-full-2026-06-22.txt) + [`evidence/r405-dc14-three-dirs-complete-2026-06-22.txt`](./evidence/r405-dc14-three-dirs-complete-2026-06-22.txt) + [`evidence/r401-dc14-grid-full-authoritative-2026-06-22.txt`](./evidence/r401-dc14-grid-full-authoritative-2026-06-22.txt) + [`evidence/r394-dc14-denominator-gap-2026-06-22.txt`](./evidence/r394-dc14-denominator-gap-2026-06-22.txt)。
2. **【渲染架构·子集范围内穷尽】Phase A IFC 三路径统一** — paint 不重跑 IFC，直接渲染 layout 存储的行盒（R205/R207 viable slice 已证 font-051 可行；R355 多行存储 / R358 per-fragment color / R362 float 侵入 三个 narrow viable-slice 已成；broad 应用需多轮 narrow 精修 + 守 multicol-fill-auto 反向依赖墙）。设计文档 [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)。
3. **【渲染架构·子集范围内穷尽】Phase 2 嵌套 multicol fragmentation** — layout 侧 column-aware IFC + 嵌套列碎片化（R131/R201；R319 证纯 inline 迁移零增益，真实价值在嵌套 / 混合内容；R383 证混合内容修复前置依赖 Phase A / R109 解转换）。设计文档 [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)、[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)。
4. ~~**【渲染架构·子集范围内穷尽】baseline-export 真修复**~~ — **✅ 闭合（R1362，2026-07-13）**：taffy 0.12.1 已支持 flex §8.5 first-baseline；fresh 实测 baseline-000~008 + flexbox-baseline-synthesis 全 near-pass font-wall（worst 2.34%/2.18%），非结构性缺口。**勿再以 baseline-export 为 lever**。

**裁决（R395 更新）**：轨道 1（分母去子集化）是当前**唯一可推进且非纯架构改造**的前向路径——并行 agent 已在执行（css-grid 全量导入 in-flight）。轨道 2-4（渲染架构）在当前子集内已被 R384 等系统性证伪为单会话不可解，但其全局 ROI 须待轨道 1 揭示全量真通过率后再校准。**无需用户在「架构改造 vs 接受 plateau」之间二选一**——先推进轨道 1（既满足 DC-14 门禁、又为轨道 2-4 的优先级提供数据），是当前最高 ROI 的下一步。

> **分支审计（2026-06-19 doc-maintenance read-only）**：未合并分支 `fix/rendering-compat-stacking`（8959ddb，2026-06-12，自称 R61 / 基线 387/490）经核查**冗余**——其核心改动（painter positioned/z-index 堆叠排序 CSS 2.1 App. E + `sync_inline_child_boxes_from_ifc`）**均已在 main**（`crates/engine/src/paint/painter/mod.rs:56-78`、`crates/layout-engine/src/engine.rs:1219`），且 main 版本更完整（额外处理 stacking-context 创建 + z-index:auto tree-order）。该分支**非**未合并 plateau 杠杆，亦非活跃并行开发（06-12 后无后续 commit）；doc-maintenance 续以 main HEAD 为准，不并入未合并分支内容。

---
## 里程碑完成状态

| 里程碑 | 状态 | 说明 |
|--------|------|------|
| M1 — WPT Reftest 基础设施 | ✅ 完成 | 14/14 标准全部达成 |
| M2 — CSS 2.1 + Quirks Mode | ✅ 完成 | CSS parser + style system quirks 已实现；layout engine quirks 推迟到 M4 |
| M3 — Flexbox + Grid | ✅ 完成 | 179 个 reftest, 100.0% pass rate；Flexbox/Grid 无渲染缺口 |
| M4 — Float + Table + Multicol | ✅ 完成 | float + table + multicol 布局算法已实现；219 个 reftest, 100.0% pass |
| M5 — 文字排版 | ✅ 完成 | CJK 换行 + justify 修复 + float 堆叠修复 + 51 个 Text reftest |
| M6 — 全量扩展 | ✅ 完成 | 685 reftest, 13 目录全部 ≥50, 100.0% pass；unicode-bidi + CJK 换行已接入生产。⚠️ **rustybuzz TextShaper（GSUB/GPOS）已实现+单元测试，但未接入生产 paint/layout 路径**（R330 代码核查实证：生产 text.rs:1057 仍逐字符 `glyph_id=ch as u32`，TextShaper 仅 lib.rs 单测调用）→ ligature/kerning/alternates 生产未生效 |
| M7 — 渲染器图元覆盖 | ✅ 完成（管线层）⚠️ | CPU 渲染器：全部 13 种图元 ✅；GPU 渲染器：13 种图元**管线**已建（48 单元测试 ✅），**但浏览器全量 GPU 路径 `render_full_scene_gpu` 实际消费以 DC-9 表为准**——transform=✅ R285/ee8373a 已接入（`collect_transforms`+`apply_transform_filters_headless`）、blend=CPU no-op stub+GPU 丢弃（**唯一剩余 GPU 真实缺口**）、5 color-matrix 滤镜（grayscale/invert/saturate/sepia/hue-rotate）=✅ R286/94c773a 已落（`collect_color_filters` mode 3-7 全处理，parity CPU）、clip=no-op（engine 从不生成）；filter:opacity/brightness/contrast/blur 已落（f6fed44/fc86937/3a3530f）；浏览器消费：全部 13 种图元 ✅ |
| M8 — 布局正确性 | ✅ 完成 | BFC 检测 ✅；float clear ✅；margin 折叠(taffy 0.7 内置) ✅；<img> 固有尺寸 ✅；position:fixed ✅(adjust_fixed_to_viewport)；position:sticky 需宿主层（已标记 is_sticky，后续集成）；percentage height/auto margin/min-max-width 已有测试验证 |
| M9 — 高级视觉效果 | 🔧 进行中 | 重复渐变 ✅；多图层背景 ✅；clip-path 全形状裁剪 ✅(inset+circle+ellipse+polygon)；border-image ✅；text-shadow ✅；backdrop-filter ✅；CSS mask ✅(渐变蒙版裁剪+alpha衰减)；overflow 全图元裁剪 ✅；滚动容器 paint 偏移 ✅(scroll_x/scroll_y 字段 + paint 时子元素坐标偏移 + 3 个单元测试)；剩余：scroll-snap 行为（需宿主层输入路由）、滚动输入路由（需浏览器 app 集成） |
| M10 — 上游 WPT 真实 Reftest 导入 | ⏸ plateau + DC-14 Phase 2（10/10 目录分母全量·R626-R627 去子集化完成；7 目录有聚合测量） | 基础设施 ✅；**grid+position+tables+flexbox+multicol+text-decor+fonts 已扩到全量**（48/95/112/496/451/246/284；writing-modes/css-text/CSS2 亦已全量·R626-R627）；全量目录 self grid 64.6%/position 64.2%/tables 67.0%/flexbox 52.4%（probe landed R435）/**multicol 43.2%（布局类最低）**/text-decor 99.2%/fonts 99.3%（文字容差，strict 76.4%/88.3%），**chromium-Oracle 真一致 36.4%**（grid 39.6% / position 37.9% / tables 43.8% / flexbox 50.6% 最高 / **multicol 23.5% 最低** / text-decor 28.9% / fonts 34.8%，聚合 629/1726=36.4%；multicol 下拉自 6-dir 41.0%）/ strict 295/490 (60.2%, pre-grid-expand)；R305–R400 单会话杠杆穷尽（R351/R355/R358/R362 四次 plateau 突破），**flexbox 暴露潜在 clean-win 面**（min-size:auto=ZeroWeb-side R428 ✅ LANDED R435 / aspect-ratio=taffy-blocked），multicol 全簇 self-fail 74-90%=结构性需 Phase 2，DC-14 分母全量去子集化已完成（10/10，R626-R627）；达标需跨会话架构任务（见「综合裁决」） |

## 当前状态概览

| 维度 | 状态 | 说明 |
|------|------|------|
| 渲染管线 | ⚠️ 全链路贯通但非一致 | HTML→CSS→Style→Layout→Paint→Composite 可运行；但 layout IFC、paint IFC 和 ZeroBrowser glyph 消费仍存在多套坐标/度量路径，`welcome.html` 已暴露用户可见错位 |
| WPT Runner | ✅ reftest 级 | 1,341 个手写 TestCase + 685 个内联 reftest（13 目录 ≥50） |
| Reftest Harness | ✅ 可用 | 分类容差、per-test fuzzy 注解、match/mismatch 模式 |
| Manifest Parser | ✅ 扩展完成 | reftest 条目解析、fuzzy 元数据、HTML 链接提取 |
| CPU 软件渲染 | ✅ 全量图元 | render_full_scene() 支持全部 13 种图元（fills, rounded_rects, gradients, shadows, images, strokes, path_fills, path_strokes, glyphs, clips, transforms, filters, blend_modes） |
| Reftest CLI | ✅ 可用 | `cargo run --bin zero-wpt-runner -- reftest` |
| Skip List | ✅ 已创建 | `tests/wpt-runner/reftest-skip-list.txt` |
| Chromium 截图脚本 | ✅ 已创建 | `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` |
| WPT 导入脚本 | ✅ 已创建 | `tests/wpt-runner/scripts/import-wpt-reftests.sh` |
| 内联 reftest | ✅ 685 个 | 13 个目录全部 ≥50，覆盖 CSS 2.1、Flexbox、Grid、Position、Display、Box、Float、Table、Multicol、Text、Fonts、Text-decor、Writing-modes |
| JS 执行 | ✅ 已集成 | reftest harness 通过 V8 sandbox 在渲染前执行 JS（不修改 DOM） |
| GPU 渲染截图 | ✅ 已验证 | GpuRenderer headless + read_pixels()；685/685 reftest GPU 模式 100.0% pass |
| GPU 渲染器图元 | ✅ 全量 | 全部 13 种图元管线 + 48 个单元测试 + 浏览器 GPU 路径集成 |
| CI 集成 | ✅ 已接入 | GitHub Actions reftest job（CPU 渲染） |
| Quirks Mode | ✅ 完成 | CSS parser + style system + layout engine quirks 全部实现 |
| 外部 stylesheet 加载 | ✅ 已贯通 | R213 落地：URL 导航路径 `fetch_url()` 现抓取 `<link rel="stylesheet">`（extract_stylesheet_hrefs → base URL 解析 → http_client 抓取 → 合并级联），三条 fetch_url 分支注入；离线 fixture HTTP server（R212）支撑测试 |
| 图片子资源/ImageCache | ✅ 已贯通 | R214（PNG 抓取+解码+image_cache）→ R215（浏览器 render_cpu/render_frame 消费 webview ImageCache，最后消费 hop）→ R216（JPEG）→ R218（SVG 栅格化统一到 render-foundation decode_image_bytes）。`<img>` 经 URL 导航全链路 fetch→decode→image_cache→browser render→真像素贯通（DC-13 P1 闭环） |
| 产品/真实静态页面视觉 smoke | 🔧 证据已持久化·持续修复 | welcome/morning.work/wintertc fixture + product-smoke + chromium Oracle 工具链就绪；**证据已持久化 `evidence/product-static/`**（3 fixture × {ZeroWeb-CPU/chromium PNG + README 含 diff%/根因}，满足 DC-13 line 305，R281 审计）；**post-R632 diff（product-smoke 阈值0）**：welcome **16.16%**（R632 line-height override 改善，自 R371 16.98%）、wintertc 13.59%（R227+R255 后）、morning-work 800×600 **18.15%**（R630 多行 y 分行 + R632 line-height 让中文行级度量差异诚实显现，自 R373 16.41% 上升；**非结构退步**，残余中文字体度量 R633 独立）、fullpage 48.65%（R255 ua_default_display 修 4× 幻影盒 89.14%→48.65%）；★R630 修复用户痛点「文字堆叠看不清」（paint Path B 多行 y 分行）；残余 diff = 中文字体度量（R633 Phase A 死锁）+ R109 IFC + hljs（需 JS），非证据缺口；★**R658 DC-13 legacy-html smoke 凑齐 20/20 页**（R656 001 + R657 002-004 + R658 005-020，HTML 3.2/4 + CSS1/2 全特性覆盖），逐像素/逐行 ASCII 体检「结构性健康无 major bug」，diff% 2.70–20.87% 全归因字体墙（垂直漂移+AA），新增 `make product-smoke-legacy` trend-only 门禁纳入 product-smoke 路径（goal line 316）；★窄屏 375×667 viewport 亦验证（DC-13 line 322「桌面+窄屏两 viewport」）——R658 legacy 关键 5 页 + R659 welcome/morning/wintertc 3 产品 fixture 全部窄屏结构完整，窄屏 diff 升高（020 20.87→29.98% / wintertc 13.59→25.74%）纯字体墙放大（更多换行→更多字形像素），无响应式布局 bug |
| #[ignore] 测试 | ⚠️ 保留 | 59 个真实网站测试保留 #[ignore]，因本地网络不稳定。其余零 #[ignore] |

---

## Done Criteria 进度

### DC-1: WPT Reftest 基础设施就位

| 条目 | 状态 | 说明 |
|------|------|------|
| fetch 上游 WPT 仓库 | ⚠️ | 导入脚本已创建，内联 reftest 替代上游导入 |
| 解析 fuzzy() 元数据 | ✅ | manifest.rs 已扩展 |
| CPU 渲染截图 | ✅ | render_scene_to_framebuffer() 可用 |
| GPU 渲染截图 | ✅ | GpuRenderer headless + CPU 圆角叠加 |
| Chromium 参考截图 | ✅ | Puppeteer 脚本已创建（capture-chromium-screenshots.mjs） |
| Viewport 对齐 | ✅ | ReftestConfig 有 viewport 字段 + CLI --width/--height |
| JS 执行集成 | ✅ | V8 sandbox 在渲染前执行 JS（不修改 DOM） |
| 分类容差机制 | ✅ | ReftestCategory (Layout/Text/Unknown) + per-test fuzzy override |
| 范围外过滤 | ✅ | reftest-skip-list.txt 已创建 |
| 通过率报告 | ✅ | 文本 + JSON 格式，按分类输出 |
| 单一命令运行 | ✅ | `cargo run --bin zero-wpt-runner -- reftest` |
| CI 集成 | ✅ | GitHub Actions reftest job |

### DC-2: CSS 2.1 核心通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283，2026-06-18）**：下表原「通过率 ≥95% ✅ 100.0%」基于**内联 685 reftest**，直接违反 DC-14（goal line 319「内联 reftest 100% 仅作 smoke，不计达标」+ line 844「禁止 DC-2~5 以内联 100% 冒充达标」= DONE 阻断项）。**真实达标**须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差 0.1%/0.5%），当前诚实数 = **39.6% strict**（188/475，evidence/cross-validate-full-2026-06-17.txt）/ 90.2% self-source-loose（442/490 @ 1%/5%），**均 <95%，DC-2 未达标**。内联 100% 仅 smoke（DC-7 全绿基线）。

| 条目 | 状态 | 说明 |
|------|------|------|
| 导入 reftest 子集 ≥ 50 | ✅（smoke） | 179 个内联 CSS 2.1 核心 reftest（**不计达标分母**，DC-14 line 323） |
| 通过率 ≥ 95% | ❌ 未达标 | 内联 smoke 100%（179/179）不计达标；真实上游全量+chromium Oracle+严格容差 = 39.6% strict，未达 95% |
| CPU 模式达标 | ❌ 未达标 | 同上（reftest harness 走 CPU 路径，容差 10× 过松 R280，reference 同源自渲染） |
| GPU 模式达标 | ❌ 未达标 | GpuRenderer headless 可用（机制就绪），但真实通过率未达标 + 容差过松 |

### DC-3: Flexbox + Grid 通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原「Flexbox/Grid 通过率 ✅ 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| Flexbox reftest 子集 | ✅（全量已导入） | **502 权威对**（R425/R426 全量；runner 运行 496；**不计达标分子**，DC-14 line 323） |
| Flexbox 通过率 | ❌ 未达标 | 全量 self 52.4%（probe landed R435；布局类容差；multicol 43.2% 更低）/ strict 42.1% / **chromium-Oracle chr<1% 50.6%（7 目录最高）**（主拖 aspect-ratio transferred size 49 例 90% self-fail + flex min-size:auto 57 例 79% + shorthand 152 例 65%），未达 95% |
| Grid reftest 子集 | ✅（smoke） | 51 个内联 Grid reftest（基础+进阶+边界+M6 扩展，**不计达标分母**） |
| Grid 通过率 | ❌ 未达标 | 同 Flexbox，内联 smoke 不计达标，真实未达 95% |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280 + 同源 reference，真实通过率未达标 |

### DC-4: Positioning + Float + Table + Multicol 通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原「各项通过率 ✅ 全部 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达；且 multicol/table 含已知结构性死锁（multicol-breaking R131、table colspan R177b 部分修），真实 sub-领域通过率更低。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| Positioning reftest | ✅（smoke） | 50 个定位 reftest（基础+进阶+M6 扩展，**不计达标分母**） |
| Float reftest | ✅（smoke） | 50 个 float 布局 reftest（M6 扩展，**不计达标分母**） |
| Table reftest | ✅（smoke） | 50 个 table 布局 reftest（M6 扩展，**不计达标分母**） |
| Multicol reftest | ✅（全量已导入） | **455 权威对**（R427 全量；runner 运行 451；**不计达标分子**，DC-14 line 323） |
| 各项通过率 | ❌ 未达标 | multicol 全量 self 43.2%（布局类最低）/ strict 27.9% / **chromium-Oracle chr<1% 23.5%（7 目录最低，全簇 self-fail 74-90% 系统性，multicol/table 结构性死锁更低）**；内联 smoke 100% 不计达标 |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280 + 同源 reference，真实通过率未达标 |

### DC-5: 文字排版通过率 ≥ 95%

> ⚠️ **达标口径纠正（R283）**：原各目录「通过率 ✅ 100.0%」基于内联 reftest，违反 DC-14（line 319/844，内联不计达标）。真实达标须 ≥95%（上游全量真实 reftest + chromium Oracle + 严格容差），当前 39.6% strict 未达；且文字类容差 5%（R280）更过松，fontdue CJK 度量/line-height 噪声（R174/R187/R229b）是文字类残余 diff 大头。内联 100% 仅 smoke。详见 DC-2 纠正说明。

| 条目 | 状态 | 说明 |
|------|------|------|
| css-text/ reftest ≥ 50 | ✅（smoke） | 51 个（**不计达标分母**） |
| css-text/ 通过率 | ❌ 未达标 | 内联 smoke 100% 不计达标；真实上游全量+chromium Oracle+严格容差未达 95% |
| css-fonts/ reftest ≥ 50 | ✅（全量已导入） | **284 权威对**（R422 全量；上游 391=284 reftest+107 非reftest；**不计达标分子**，DC-14 line 323） |
| css-fonts/ 通过率 | ❌ 未达标 | 全量 self 99.3%（文字容差）/ strict 88.3% / **chromium-Oracle chr<1% 仅 34.8%**（主拖 font-features/variant 89 例仅 10%，rustybuzz 未接生产），未达 95% |
| css-text-decor/ reftest ≥ 50 | ✅（smoke） | 50 个（**不计达标分母**） |
| css-text-decor/ 通过率 | ❌ 未达标 | 同上（text-emphasis 等未实现 R232） |
| css-writing-modes/ reftest ≥ 50 | ✅（smoke） | 50 个（**不计达标分母**） |
| css-writing-modes/ 通过率 | ❌ 未达标 | 同上（vertical-rl clearance R114/R164 死锁） |
| CPU 模式达标 | ❌ 未达标 | 容差 10× 过松 R280（文字类 5%）+ 同源 reference，真实通过率未达标 |

### DC-6: Quirks Mode

| 条目 | 状态 | 说明 |
|------|------|------|
| CSS parser quirks | ✅ | 已实现：quirky color values（hashless hex + numeric colors）、unitless lengths（裸数字视为 px） |
| Style system quirks | ✅ | 已实现：percentage-height quirk、table height quirk（height → min-height）、inline width/height quirk 注释 |
| Layout engine quirks | ✅ | table/float layout 已在 M4 实现，quirks mode 通过 UA 默认 display 值和 table height quirk 生效 |
| DOM → style 链路传递 | ✅ | Document::quirks_mode() → tag_name 提取 → apply_quirks_mode_adjustments |

### DC-7: 测试与质量

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 零失败 | ✅ | 全部通过（59 个真实网站测试保留 #[ignore]） |
| 零 #[ignore] 测试 | ✅ | 仅 real_website_compat.rs 有 59 个 #[ignore] |
| 新修复有单元测试 | ✅ | quirks mode 颜色/长度/样式系统各新增单元测试 |
| cargo clippy 零警告 | ✅ | `cargo clippy -- -D warnings` 通过 |
| Reftest 报告持久化 | ✅ | evidence/reftest-report-2026-06-06.json/txt |
| 历史记录可追溯 | ✅ | 首份报告已持久化 |

### DC-8: CPU 渲染器图元覆盖（全部 13 种）

| 条目 | 状态 | 说明 |
|------|------|------|
| FillPrimitive | ✅ | 填充矩形（原有） |
| RoundedRectPrimitive | ✅ | 圆角矩形（原有） |
| GlyphPrimitive | ✅ | 文字渲染（原有） |
| GradientPrimitive | ✅ | 线性/径向/锥形渐变，逐像素插值 |
| ShadowPrimitive | ✅ | box-blur 近似阴影，含 blur_radius/spread_radius |
| ImagePrimitive | ✅ | RGBA 像素数据合成到 framebuffer |
| StrokePrimitive | ✅ | solid/dashed/dotted 线段 + LineCap |
| PathFillPrimitive | ✅ | 多边形扫描线填充 |
| PathStrokePrimitive | ✅ | 多边形描边 |
| TransformPrimitive | ✅ | 仿射变换后处理 |
| ClipPrimitive | ✅ | 矩形裁剪（像素级 discard） |
| FilterPrimitive | ✅ | blur + opacity + brightness/contrast/grayscale/invert/saturate/sepia/hue-rotate（apply_filter 全 8 color-matrix + blur，effects.rs；drop-shadow 仍 stub） |
| BlendModePrimitive | ⚠️ stub | draw_order 派发（cpu/mod.rs:250）但 `apply_blend_mode`（effects.rs:331-348）是 **no-op stub**（算 rect 后不应用，注释自承需 source+dest 双图层）——同 DC-9 blend（R278/R282 证），paint 生成但 CPU 不真混合。生产 footprint 低 |
| render_full_scene() 入口 | ✅ | 新函数，CSS painting order 渲染全部 13 种图元 |

### DC-9: GPU 渲染器图元覆盖

> **状态（R277 只读复核；2026-06-19 doc-maintenance 复核 committed HEAD b75035b 纠正：R285 transform / R286 5 color-matrix 滤镜均已落，原「transform WIP / 5 滤镜 GPU 丢弃」两项 stale）**：较 R211（2026-06-17 标 transform/clip/filter/blend 全 ⚠️「丢弃」）有实质推进。filter:opacity（f6fed44）/brightness/contrast（fc86937）/blur（3a3530f）已落地为独立 WGSL 后处理管线（ping-pong 区域读写，`render_full_scene_gpu` 经 `apply_color_filters_headless`/`apply_blur_filters_headless` 消费，非 passthrough，满足 DC-14）。clip 经 R220 实证为 no-op——engine 生产路径**从不生成** ClipPrimitive（`add_clip` 0 处非测试调用），overflow 裁剪在 paint 阶段由 `clip_all_primitives_to_rect` 预烘焙进图元几何，CPU/GPU 两路均空谈满足，**非真实缺口**。**真实剩余缺口 1 类（仅 blend_mode）**：(a) ~~transform~~ **✅ R285/ee8373a 已落**——`fs_transform`（逆变换重采样）+ `create_transform_pipeline` 已接入 `render_full_scene_gpu`（`collect_transforms`+`apply_transform_filters_headless`，guard 非空；单测 `test_gpu_full_scene_transform_translation`，匹配 CPU `apply_transform_post` clear-to-white 语义）；(b) **blend_mode**——paint 在 `painter/effects.rs:313` 生成 `BlendModePrimitive`，但 **CPU `apply_blend_mode`（effects.rs:331-348）是 no-op stub**（算 rect 后 `_=(left,top,right,bottom)` 仅消未用警告），GPU `render_full_scene_gpu` 同样不消费，需 source+dest 双图层新机制（R269 标记为比 opacity 大的独立特性，低 reftest footprint）；(c) ~~5 color-matrix 滤镜~~ **✅ R286/94c773a 已落**——`collect_color_filters`（renderer/mod.rs:2062）现处理全 8 mode（Opacity0/Brightness1/Contrast2/Grayscale3/HueRotate4/Invert5/Saturate6/Sepia7），parity CPU `apply_filter`，单测 `test_gpu_full_scene_filter_grayscale`/`_hue_rotate`/`_invert`。drop-shadow（CPU 亦 stub，GPU 同）仍 `_ => None` 丢弃。reftest harness 与 product-smoke 均走 CPU 路径，GPU 缺口不污染测量数字，仅影响浏览器 GPU 渲染模式。

| 条目 | 状态 | 说明 |
|------|------|------|
| FillPrimitive | ✅ | GPU 填充（原有） |
| GlyphPrimitive | ✅ | GPU 文字渲染（原有，atlas） |
| RoundedRectPrimitive | ✅ | GPU 片段着色器（WGSL corner discard） |
| GradientPrimitive | ✅ | GPU 渐变 shader（线性/径向/锥形 + 1D 渐变纹理） |
| ShadowPrimitive | ✅ | 半透明填充矩形（简化，不做 GPU blur） |
| ImagePrimitive | ✅ | GPU 纹理上传 + 采样（RGBA→texture→shader） |
| StrokePrimitive | ✅ | CPU 侧顶点生成 + GPU fill pipeline（solid/dashed/dotted） |
| PathFillPrimitive | ✅ | CPU 侧扇形三角化 + GPU fill pipeline |
| PathStrokePrimitive | ✅ | CPU 侧分解为粗线段 + GPU fill pipeline |
| TransformPrimitive | ✅ | R285（ee8373a）独立 WGSL `fs_transform`（逆变换重采样，匹配 CPU `apply_transform_post` clear-to-white 语义）+ `create_transform_pipeline`，**已接入** `render_full_scene_gpu`（`collect_transforms` + `apply_transform_filters_headless`，guard `!empty && headless_texture.is_some()`）；单测 `test_gpu_full_scene_transform_translation` |
| ClipPrimitive | ⚪ no-op | engine 生产路径**从不生成** ClipPrimitive（R220 实证），overflow 裁剪预烘焙进图元几何。CPU/GPU 两路均空谈满足，**非真实缺口** |
| FilterPrimitive | ✅（drop-shadow 除外） | **全 8 color-matrix + blur 已落（独立 WGSL ping-pong 后处理，非 passthrough）**：opacity（fs_color_filter mode0）/brightness（mode1）/contrast（mode2，fc86937）/grayscale（mode3）/hue-rotate（mode4）/invert（mode5）/saturate（mode6）/sepia（mode7，R286/94c773a，parity CPU `apply_filter`）/blur（fs_blur 三角核 2-pass，3a3530f）。`collect_color_filters`（mod.rs:2062）现处理全 8 mode（原「mode 0/1/2 其余丢弃」已过时）。**仍未落**：drop-shadow（CPU 亦 stub，GPU `_ => None`） |
| BlendModePrimitive | ❌ 丢弃 | paint 生成（effects.rs:313）但 CPU `apply_blend_mode`=no-op stub（effects.rs:331-348）+ GPU `render_full_scene_gpu` 不消费。**单 framebuffer post-process 架构上不可行**（R278 实证：apply 时元素子树已与 backdrop 合并进 framebuffer、不可分离，区别于 opacity/blur 的合法区域近似）→ 需 **paint-isolation 架构**（元素子树隔离渲染到 offscreen + source/dest 双纹理 blend 合成 pass）；render-foundation 现无 per-element staging buffer、paint 无 isolation group，**multi-round 架构 defer**。footprint ~2-4 case，非 lever |

> **DC-9/DC-14 parity caveat（R277）**：覆盖满足 ≠ CPU 像素 parity——(1) opacity=GPU RGB-darken 近似（R272，post-process 无法恢复背景）；(2) blur=GPU 三角核 separable 2-pass vs CPU 多遍 box（R277，算法分歧，非 ==CPU，见 `evidence/r277-dc9-gpu-blur-vs-cpu-boxblur-parity-2026-06-18.txt`）；(3) brightness/contrast=精确 parity（R273 正确 CSS 语义）。三者覆盖均达标（独立 WGSL 非丢弃），但 opacity/blur 属「覆盖达标非像素对齐」类。

### DC-10: 浏览器图元消费

| 条目 | 状态 | 说明 |
|------|------|------|
| transform_webview_primitives() 全 13 种 | ✅ | 新函数处理所有 RenderPrimitives 字段 |
| render_cpu() 使用 render_full_scene() | ✅ | 完整图元渲染替代旧版 3 种入口 |
| scale_factor 应用 | ✅ | 所有图元类型正确缩放 |
| offset 应用 | ✅ | 所有图元类型正确偏移 |
| clip_y 视口裁剪 | ✅ | fills + glyphs 应用 clip_y 裁剪 |
| CSS painting order | ✅ | shadows → backgrounds → borders → content → overlay → filters → blend_modes |

### DC-11: M7 验证

| 条目 | 状态 | 说明 |
|------|------|------|
| cargo test 全绿 | ✅ | 7800+ 测试全部通过 |
| cargo clippy 零警告 | ✅ | `cargo clippy -- -D warnings` 通过 |
| 新增图元单元测试 | ✅ | 渐变/阴影/图片/线段/路径填充/路径描边/变换/裁剪/滤镜/混合模式各有独立测试 |

---

## M1 里程碑详情

**目标**: 建立能够导入、运行、对比和报告 WPT reftest 的完整基础设施。

### M1 完成标准 (14 项)

1. ✅ fetch 上游 WPT 仓库（导入脚本 + 内联 reftest 替代）
2. ✅ 扩展 manifest.rs 解析 fuzzy() 元数据
3. ✅ CPU 软件渲染截图（render_scene_to_framebuffer）
4. ✅ GPU 渲染截图（GpuRenderer headless + CPU 圆角叠加）
5. ✅ 自动化 Chromium 截图工具（Puppeteer 脚本）
6. ✅ Viewport 对齐机制
7. ✅ JS 执行集成（V8 sandbox 执行 script 标签中的 JS）
8. ✅ 分类容差机制
9. ✅ 范围外 reftest 过滤 (skip list)
10. ✅ 按目录分类通过率报告（文本 + JSON）
11. ✅ 单一命令运行全部 reftest
12. ✅ 导入 CSS 2.1 核心 ≥ 50 个 reftest（115 个）
13. ✅ 记录初始通过率（100.0% 113/113）
14. ✅ 确认 #[ignore] 标记状态

### M1 已完成的基础设施

| 组件 | 文件 | 说明 |
|------|------|------|
| Manifest 解析 | `tests/wpt-runner/src/manifest.rs` | reftest 条目、fuzzy 元数据、HTML 链接提取 |
| Reftest 引擎 | `tests/wpt-runner/src/reftest.rs` | 分类容差、fuzzy 覆盖、match/mismatch 比较 |
| Reftest 数据 | `tests/wpt-runner/src/reftest_data.rs` | 159 个 CSS 2.1 核心 + Flexbox/Grid 内联 reftest |
| Reftest CLI | `tests/wpt-runner/src/main.rs` | `reftest` 子命令 + 文本/JSON 报告 |
| Skip List | `tests/wpt-runner/reftest-skip-list.txt` | SVG/Canvas/WebGL/动画过滤规则 |
| Chromium 工具 | `tests/wpt-runner/scripts/capture-chromium-screenshots.mjs` | Puppeteer headless 截图 |
| 导入脚本 | `tests/wpt-runner/scripts/import-wpt-reftests.sh` | 上游 WPT reftest 批量导入 |

---

## 初始 Reftest 通过率数据（M6 inline，不计达标 — 已归档）

> 2026-06-07 M6 基线：685 内联 reftest 100% 通过（CPU 软件渲染，800×600）。该 685 内联 reftest 自 DC-14 起明确**不计达标分母**（goal line 323/844；DC-2~5 各节均标「内联 smoke 100% 不计达标」），100% 仅作 smoke。逐目录/覆盖范围明细已迁出至 [`archive/m6-inline-reftest-baseline-2026-06-07.md`](./archive/m6-inline-reftest-baseline-2026-06-07.md)。**达标口径真基线见下节「上游真实 WPT Reftest 通过率」**（self-source 456/518 / chromium-Oracle ~42%）。

---

## 上游真实 WPT Reftest 通过率

> 早期上游 reftest 调查（R11–R20，2026-06-09/10，self-source 基线 74.7%）已归档至 [`archive/rounds-r11-r20-reftest-investigation.md`](./archive/rounds-r11-r20-reftest-investigation.md)。

**当前基线（R427 css-multicol 扩到全量后——10/10 目录分母全量·R626-R627 去子集化完成（含 writing-modes/css-text/CSS2）；7 目录有聚合测量；strict post-R326 复验仍零漂移，见 [`evidence/strict-baseline-reverify-post-r326-2026-06-19.txt`](./evidence/strict-baseline-reverify-post-r326-2026-06-19.txt)；7 目录聚合 oracle 629/1726=36.4%（布局类 38.4% / 文字类 32.1%）见顶部主基线；evidence [`r427-cssmulticol-full-2026-06-22.txt`](./evidence/r427-cssmulticol-full-2026-06-22.txt) + [`r425-cssflexbox-full-2026-06-22.txt`](./evidence/r425-cssflexbox-full-2026-06-22.txt) + [`r421-cssfonts-full-2026-06-22.txt`](./evidence/r421-cssfonts-full-2026-06-22.txt) + [`r412-csstextdecor-full-2026-06-22.txt`](./evidence/r412-csstextdecor-full-2026-06-22.txt) + [`r405-dc14-three-dirs-complete-2026-06-22.txt`](./evidence/r405-dc14-three-dirs-complete-2026-06-22.txt) + R404 [`r404-dc14-position-full-2026-06-22.txt`](./evidence/r404-dc14-position-full-2026-06-22.txt) + R401 [`r401-dc14-grid-full-authoritative-2026-06-22.txt`](./evidence/r401-dc14-grid-full-authoritative-2026-06-22.txt)）**：

- self-source 全量目录：css-grid **32/48=66.7%（R546m 复测）** + css-position **63/95=66.3%（R546m 复测）** + **css-tables 77/112=68.8%（R546m doc-maintenance 复测确认，[evidence](./evidence/r546m-csstables-77-reverify-2026-06-24.txt)）** + css-flexbox **368/496=74.2%（R525–R542 +108，R546 doc-maintenance 复测确认，[evidence](./evidence/r546-flexbox-368-reverify-2026-06-24.txt)）** + css-multicol 195/451=43.2% + **css-text-decor 244/246=99.2%** + **css-fonts 282/284=99.3%**（文字容差）；**7 目录聚合 self 1148/2032=56.5%**；聚合（7 全量 + 2 子集）；**混合口径，非全量真通过率**
- self-source strict **295/490 (60.2%)** @ 锁定 0.1%/0.5%（DC-14 真通过口径，pre-grid-expand）
- chromium-Oracle 真一致率 **~42.1% (broad, 200/475) / 37.3% (strict self-pass&chr<1%, 177/475)**（self-source 含 46.5% 假通过；R391 锁定诚实基线，pre-R388 ~35.8% 被 108 损坏 Ahem oracle 压低已修；⚠️ 全部基于 ~5-6% 子集分母 503/上游~8000-10000，非全量真通过率，不构成 DC-14 达标证据）。**7-目录全量口径更诚实**：oracle 629/1726=36.4%（grid 39.6% / position 37.9% / tables 43.8% / flexbox 50.6% / text-decor 28.9% / fonts 34.8% / **multicol 23.5% 最低**）

完整 plateau 分析、已穷尽杠杆表、4 条跨会话架构路径见顶部「综合裁决」节；逐目录 chromium-Oracle 污染分布见 `evidence/cross-validate-full-2026-06-18.txt`（flexbox 26% 污染最诚实，writing-modes 73% 最高）。达标需按 rally 续跑协议推进架构任务，单会话杠杆已穷尽。

---

## 已知关键缺口（当前活跃）

> 下表仅列**尚未解决**的缺口，与 R305–R323 plateau 框架 + 顶部「综合裁决」R990–R1093 块对齐（剩余 forward motion = 已批准推进的 C/C++ 字体栈方向 + 跨会话架构任务）。**已完成项**（Float/Table/Multicol 布局算法、OpenType shaping、BiDi、CJK 换行、justify、quirks mode、CPU/GPU/浏览器图元覆盖、外部 stylesheet、图片子资源/ImageCache、margin 折叠、BFC 检测+margin 隔离、`<img>` intrinsic sizing + object-fit）见「里程碑完成状态」「当前状态概览」，不再在此重复列。

| 缺口 | 影响范围 | 优先级 | 解锁路径 |
|------|----------|--------|------|
| **FreeType C-dep 翻 default（font-wall root unlock）** | font-wall + Phase A 非-Ahem + ::first-letter（R990 +138 oracle / R1068 css-text +24 feature-gated 已证） | **✅ LANDED R1159（default-on）** | `freetype-raster` feature R1068 LANDED default-off → **R1159 翻 default-on**（`crates/render-foundation/Cargo.toml` `default=["freetype-raster"]`，12 下游 crate 经 workspace 自动获得，规避 R1138 toggle gotcha）。本机 A/B 实测：css-text oracle +24（复现 R1094）+ welcome −0.28pp（复现 R1068）+ make test 12205/0 + clippy clean，零回归。R1094 全 corpus +232 零回归转生产默认；R1213 已完成 CI 7-target 验证。下游解锁：font-wall（+232 落地）+ Phase A 非-Ahem + ::first-letter 可重试 |
| **Paint IFC / taffy-IFC 架构分裂** | large-font（ifc-008/009/011）+ welcome/morning 文本度量残余（self-source 失败主因） | **P0** | Phase A IFC 统一：baseline-resolved 单一权威行盒（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)；墙②③ R125–R213 六轮死锁 + R306 几何基线证伪） |
| Multicol column breaking / 嵌套碎片化 | css-multicol 失败聚类（结构性） | P1 | Phase 2 嵌套 multicol fragmentation（layout 侧 column-aware IFC；paint 侧 R157/R198/R203/R122/R317 五轮证 net-negative 死路） |
| ~~Multicol / flexbox baseline-export~~ | ~~baseline-000~008 + flexbox-baseline（~10+ 案）~~ | **✅ 闭合（R1362）** | **已非缺口**：taffy 0.12.1 已支持 flex §8.5 first-baseline；R1362 fresh 实测 css-multicol/baseline-000~008（9 案 6/9<1%，worst 2.34%）+ css-flexbox/baseline-synthesis（5 案 3/5<1%，worst 2.18%）全 near-pass font-wall，非 baseline 结构缺口。R859 空 IB margin-edge baseline 已 LANDED。**勿再以 baseline-export 为 lever** |
| Writing-mode 垂直布局 | css-writing-modes 垂直 float/clearance 轴 | P1 | 精细轴交换（R57/R114/R164 谱系，clearance vertical-axis） |
| Inline-box 模型 | CSS2 linebox（vertical-align/行盒高度） | P1 | 与 Phase A IFC 统一耦合（v_offset/baseline 语义分歧） |
| DC-9 blend_mode（mix-blend-mode） | GPU backdrop 合成（~2-4 reftest 案，近零覆盖） | P2 | paint-isolation 架构（offscreen 子树渲染 + source/dest 双纹理 blend pass，R278 defer） |
| position: sticky | 滚动吸附（需宿主层输入路由） | P2 | host-runtime 层 sticky 偏移驱动；**R326 实测**：converter 把 sticky 映射为 taffy Relative，block-level 偏移已被 taffy 应用（scroll-0 应吸附场景 delta==inset）。缺的是 scrollport 相对钳制（normal 位满足 inset 时应 == static，当前 == relative），属架构性 |
| 产品 smoke 文本度量残余 | welcome 17% / wintertc 14% / morning 49% | P1（与 Phase A 同源） | Phase A IFC 统一（item-tag R109 inline→block + system-ui 字体度量，非图片/CSS 缺口） |
| ~~WebP 解码~~ + ~~CSS `url()` 背景图抓取~~ | 图片子资源残余 | ✅ done | ✅ **WebP**（R1793）+ **CSS `url()` 抓取 sync+async+inline**（R1794 sync + R1795 async + R1796 inline `style=`：`extract_css_image_urls` 扫外链+`<style>`+元素 `style=` 属性，排除 @font-face + data:；painter 4 处 `simple_hash(url)` latent bug 修为 `image_resource_key(url, document_url)`，`background-image`/`list-style-image`/`border-image-source` 跨 sync+async+inline 端到端可渲染）。残余 = `content/cursor:url()` + favicon（低 ROI） |

### DC-11 doc 复核（2026-06-19，read-only 代码核查；无代码/reftest 变更）

承接 R323 margin 折叠探针（纠正 goal doc「未实现」）与 R324 position:fixed 修复（commit 5b11fc2）后，本轮 read-only 核查 DC-11「布局正确性」其余项是否如 goal doc 声称的「未实现」。逐项代码核查 + 生产接线验证：

| 项 | goal doc 旧声明 | 代码实证 | 裁决 |
|----|----------------|---------|------|
| BFC | 「无 BFC 概念，overflow:hidden 不隔离浮动/不阻止 margin 折叠」 | `establishes_bfc`（margin_collapse.rs:33-76）全条件（overflow/float/abspos/flow-root/flex/grid/table/multicol）**接线生产**（engine.rs:2940/2988/3052）；`use_bfc_float_containment`（engine.rs:2992）落地 float containment；margin 隔离 R323 实测 6 探针全过 | **过时**（待 goal doc 纠正；并行 agent R324 note 标 BFC float containment 为下一调查项） |
| `<img>` intrinsic + object-fit | 「无固有尺寸，object-fit 在 paint 阶段处理但无实际图片数据」 | `apply_replaced_element_sizing`（tree.rs:165，HTML width/height 属性 + SVG data URI + 解码固有尺寸）+ `compute_object_fit_rect` 全 5 值（Fill/Contain/Cover/None/ScaleDown，text.rs:1582，img paint site text.rs:614 调用）+ R318 图片数据端到端贯通 | **过时**（待 goal doc 纠正；并行 agent R324 note 标 object-fit 为下一调查项） |
| 滚动容器 | 「无真正滚动容器，浏览器层手动偏移」 | `scroll_x/scroll_y` 字段 + paint 偏移（painter/mod.rs:465-471）+ overflow 裁剪（needs_clip/clip_all_primitives_to_rect，mod.rs:197/298）；app 层 scroll_offset per tab + wheel 路由 | **基本准确**（paint 偏移+裁剪已落地，非 layout 级真滚动容器；master.md 已如实标「简化处理」）→ 不改 |
| position: sticky | 「需宿主层动态调整」 | `is_sticky` 标记落地（engine.rs:606）。**R326 实测纠正**：converter（converter/mod.rs:286）把 `Sticky` 映射为 taffy `Position::Relative`，故 block-level sticky 偏移**已被 taffy 应用**（scroll-0 应吸附场景 delta==inset，新单测 `test_sticky_applies_inset_like_relative_at_scroll_zero` 实证）。旧注「偏移未应用」源于 `engine.rs:1948` 死代码（`#[allow(dead_code)]` 的 `apply_relative_offsets`）注释，非生产路径。缺的是 **scrollport 相对钳制**（normal 位满足 inset 时应 == static，当前渲染 == relative）——属架构性，非单点修复 | **部分过时**（R326 已纠正「偏移未应用」为「已应用、缺 scrollport 钳制」） |
| position: fixed | 「当前错误地映射为 absolute」 | `adjust_fixed_to_viewport`（engine.rs:2176）存在且调用；**R324（commit 5b11fc2）已修 fixed-inside-positioned-ancestor over-correction（`+=`→`-=`）** | R324 已处理（goal doc 纠正见并行 agent 提交） |

**结论**：goal doc DC-11 的 BFC + object-fit 两项「未实现」声明与代码现实矛盾（governance §1 自洽）。本轮将核查结论沉淀于本表（避免与并行 agent 活跃编辑 rendering-compat.md 冲突）；goal doc prose 纠正**已由 R325 执行**（BFC known-gaps line 378 + 替换元素 DC-11/support-envelope/known-gaps 三处，按 R323/R324 先例）。scroll/sticky 声明准确不改。本轮零代码变更（仅 read-only 核查）。

## IFC 统一技术参考（R69+ 代码级上下文 — 已归档）

> 三套 IFC 运行路径（measure / remeasure / paint）、paint-IFC override 覆盖缺口、R37–R68 已穷尽不可行路径表、存储 vs paint 基线差异、完成度清单、Taffy Fork 状态等代码级细节已迁出至 [`archive/r69-ifc-unification-technical-reference.md`](./archive/r69-ifc-unification-technical-reference.md)（无入站引用，归档前核查）。现代 Phase A 规划见 [`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)；当前 plateau 结论见顶部「综合裁决」表。

---

## IFC 之外的其他卡点（R68 时代前置 plateau 框架 — 已归档，保留锚点 stub）

> 本节为 R68 时代（pre-plateau，~320 轮前）的卡点分析框架，详细「影响/当前能力/缺失/关键失败测试/技术方向」+「卡点依赖关系/R69+ 推荐优先序」**已迁出至** [`archive/r68-other-blockers-framework.md`](./archive/r68-other-blockers-framework.md)。多数卡点已被顶部「综合裁决」表 + 「已知关键缺口」表以更准确的多会话架构结论取代；保留下列锚点摘要供 `multicol-phase2-unified-column-flow-spec.md` 等文档的「卡点 #N」引用解析（避免 dangling pointer）。

- **卡点 #2 Multicol Column Breaking**（~22 测试）：内容碎片化缺失——超高块级子需跨列拆分。→ 现代结论见综合裁决「Phase 2 嵌套 multicol fragmentation」+ [`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)。
- **卡点 #3 Writing-mode 垂直布局**（~10 测试）：垂直模式 float/clearance 轴交换。→ 综合裁决 R114/R164 谱系。
- **卡点 #4 Flexbox Baseline 对齐**（~3-5 测试，独立）：taffy 仅 ≥2 baseline 子才算基线。→ 综合裁决「baseline-export」（独立卡点，与 multicol Phase 2 解耦）。
- **卡点 #5 Table Border-collapse 精度**（~3 测试）：外边缘单元格边框减半。→ R177b 部分修。
- **卡点 #6 CSS 2.1 App E 堆叠顺序**（2-3 测试）：position:relative 后代 tree-order 排序。→ **R380 ruled out**（net-negative 回退）。
- **卡点 #7 Grid Max-content Sizing**（2-3 测试）：taffy grid max-content。→ R97/taffy-blocked（R304 DEFER）。
- **卡点 #8 Swatch 图像缩放精度**（~5 测试）：纯色 PNG 双线性伪影。→ niche。
- **卡点 #9 Position Fixed 视口定位**（1-2 测试）：→ **✅ R324/R98 已修**（`adjust_fixed_to_viewport`）。

---

## 技术决策记录

| 日期 | 决策 | 理由 |
|------|------|------|
| 2026-06-06 | 保留真实网站测试的 #[ignore] | 本地网络不稳定，这些测试不可执行 |
| 2026-06-06 | 扩展而非重写 manifest.rs 和 reftest.rs | 目标文档明确要求扩展现有模块 |
| 2026-06-06 | 使用内联 reftest 替代上游导入 | 避免网络依赖，53 个 CSS 2.1 核心 reftest 覆盖主要布局场景 |
| 2026-06-06 | mismatch 阈值设为 0.5% | 800×600 视口下，50×50 小元素差异约 0.52%，1% 阈值会漏检 |
| 2026-06-06 | 文字类 reftest 使用宽松容差 (5%/15ch) | fontdue vs Skia 字体渲染像素差异大 |
| 2026-06-06 | QuirksMode 在 StyleSystem 内部传递（不暴露为公开参数） | 保持公共 API 简洁，doc.quirks_mode() 在 compute_styles 入口处提取 |
| 2026-06-07 | quirks mode 颜色/长度解析通过函数指针分发 | parse_color_fn/parse_length_fn 模式避免重复 match 分支 |
| 2026-06-07 | apply_quirks_mode_adjustments 接受 tag_name 参数 | 需要按元素标签（如 table）应用不同的 quirks 规则 |
| 2026-06-07 | inline 元素 width/height quirks 暂不实现 | layout engine 将 inline 映射为 block，实际已生效；待 inline layout 正确实现后补充 |
| 2026-06-07 | UA 默认 display 值通过级联注入（Origin::UserAgent） | 最低优先级，可被作者样式覆盖；避免修改 ComputedStyle::default() |
| 2026-06-07 | Table 布局通过后处理步骤实现（类似 float） | taffy 无原生 table 支持，所有 table display types 映射为 Block 后重新定位 |
| 2026-06-07 | 修复 parse_display 缺失 table types 的 bug | color.rs 中有重复的 parse_display（缺 table types），通过 pub use color::* 被实际使用 |
| 2026-06-07 | Multi-column 通过后处理步骤实现（类似 float/table） | taffy 无原生 multicol 支持，column-count/column-width 容器的子元素在后处理中重新定位到各列 |
| 2026-06-07 | 多列均衡分配使用 shortest-column-first 策略 | 依次将每个子元素放入当前总高度最小的列，实现视觉均衡 |
| 2026-06-07 | CJK normal 模式下每个字符单独作为"单词" | CSS 规范要求 CJK 允许任意字符间断行，split_into_words 中 CJK 字符独立为词 |
| 2026-06-07 | text-align: justify 使用 effective_content_area 计算剩余空间 | 修复了原先用 container_width 忽略 float exclusion 的问题 |
| 2026-06-07 | Float exclusion 从 max 改为 additive stacking | 多个同侧 float 应累加宽度而非取最大值 |
| 2026-06-07 | rustybuzz 集成到 TextShaper | 优先使用 rustybuzz 进行 OpenType shaping（GSUB/GPOS），回退到 fontdue 逐字符映射 |
| 2026-06-07 | unicode-bidi 集成到 inline layout | RTL 字符自动检测并重排序，LTR 文本零开销 |
| 2026-06-07 | FontLoader 存储原始字体字节 | 供 rustybuzz Face::from_slice 使用，fontdue 仍用于 advance width 获取 |
| 2026-06-07 | ShapedGlyph 增加 x_offset/y_offset 字段 | 来自 rustybuzz 的 GPOS 定位偏移 |
| 2026-06-07 | 新增 render_full_scene() 替代 render_scene_to_framebuffer() | 旧函数仅支持 fills + rounded_rects + glyphs，新函数支持全部 13 种图元 |
| 2026-06-07 | 新增 transform_webview_primitives() 替代 inline 坐标变换 | 旧方式仅处理 fills + glyphs，新函数处理全部 13 种 RenderPrimitives 字段 |
| 2026-06-07 | 渐变使用逐像素插值 | 线性/径向/锥形渐变均在 CPU 上逐像素计算，无 GPU 依赖 |
| 2026-06-07 | 阴影使用 box-blur 近似 | 三次 box-blur 近似高斯模糊，性能与质量平衡 |
| 2026-06-07 | 路径填充使用扫描线算法 | 逐行扫描多边形边界，奇偶规则填充 |
| 2026-06-07 | CPU 后处理：Transform/Clip/Filter/BlendMode 作为后处理步骤 | 像素级后处理，不依赖 GPU；GPU 渲染器需独立实现 |
| 2026-06-07 | GPU 渲染器多管线架构 | 5 条独立 wgpu 渲染管线：Fill+Glyph、RoundedRect、Gradient、Image、Blur。每种管线有独立 WGSL shader 和绑定组布局。Mesh-based 图元（stroke/path）通过 CPU 侧顶点生成复用 fill pipeline。Phase-separated 架构避免借用冲突。 |
| 2026-06-07 | 浏览器 GPU 路径集成 render_full_scene_gpu | render_frame() 改用 render_full_scene_gpu 替代 render_scene_ext，GPU 渲染路径现在支持全部 13 种图元。GPU 渐变测试使用 ±3 容差应对 float→u8 精度误差。 |
| 2026-06-07 | Taffy 0.7 内置 margin 折叠 | 发现 taffy 0.7 已通过 CollapsibleMarginSet 实现 CSS 块级 margin 折叠，不需要额外后处理步骤。移除了自实现的 margin_collapse 后处理。 |
| 2026-06-07 | Float clear 后处理实现 | 在 adjust_float_positions() 中实现 clear:left/right/both。非 float 元素的 clear 属性将其推到对应侧浮动元素的底部之下。LayoutBox 新增 clear 字段。 |
| 2026-06-07 | 重复渐变：fract() tiling | CPU 渲染器用 fract(t/period) 实现重复渐变周期循环；GPU 渲染器通过 WGSL shader 中 fract(t) 实现，repeating 标志通过 param3 取负编码传递。 |
| 2026-06-07 | 多图层背景 Vec 迁移 | background_image 从 BackgroundImageComputedValue 单值改为 Vec，CSS 解析器新增 parse_background_image_layers() 处理逗号分隔，paint 按逆序渲染（CSS 规范最后一层在最底）。 |
| 2026-06-07 | clip-path inset 实际裁剪 | 新增 clip_all_primitives_to_rect() 对全部图元类型（fills/rounded_rects/gradients/shadows/images/glyphs/strokes）应用矩形裁剪，替代原来的虚线指示器。 |
| 2026-06-07 | CSS mask 渐变蒙版 | mask-image 复用 BackgroundImageValue 类型解析，渐变蒙版通过 clip_all_primitives_to_rect 裁剪到渐变边界 + 平均 alpha 衰减实现。URL 蒙版暂不支持（需图像加载基础设施）。 |
| 2026-06-07 | overflow 全图元裁剪修复 | paint_node/paint_node_in_rect 中 overflow:hidden/scroll/clip 原来仅裁剪 fills+glyphs，渐变/阴影/图片/线段等图元溢出容器边界不被裁剪。改为使用 PrimitiveCounts + clip_all_primitives_to_rect 裁剪全部 13 种图元类型。 |
| 2026-06-07 | 滚动容器 paint 偏移 | LayoutBox 新增 scroll_x/scroll_y 字段。paint_node 中当 overflow == Scroll 时，子元素坐标减去 scroll 偏移量。overflow:Hidden 不应用滚动偏移（非滚动容器）。3 个单元测试验证。 |
| 2026-06-07 | render_full_scene 切换到上游 reftest | 上游 reftest 从旧版 render_scene_to_framebuffer（仅 fills+rounded_rects+glyphs）切换到 render_full_scene（全部 13 种图元）。同时启用 ImageCache 从 base_dir 加载 PNG 图片。这使 reftest 结果更准确，但也暴露了之前被不完整渲染掩盖的布局差异。 |
| 2026-06-07 | skip_indicators 模式 | Painter 新增 skip_indicators 标志，RenderPipeline 新增 set_skip_indicators() 方法。当设为 true 时跳过全部 ~30 个 CSS 属性调试指示器（border-collapse 橙色标记、direction 箭头等），避免干扰 reftest 像素对比。 |
| 2026-06-07 | UA 默认样式扩展 | 新增 body{margin:8px}、h1-h6{margin+font-weight}、p{margin:1em 0}、ul/ol{margin+padding-left} UA 默认样式，对齐浏览器默认行为。 |
| 2026-06-08 | table row-group 行索引修复 | build_grid 中行组内行存储 rg_child_idx 但 get_row_box 在 table_box.children 查找，导致 tbody/thead/tfoot 内的行被静默丢弃。修复：TableRow 新增 row_group_index 字段，get_row_box/position_cells 根据此字段正确导航到行组内的行。 |
| 2026-06-08 | column-gap 属性映射修复 | converter gap.width 原先使用 style.gap（仅 gap 简写设置），改为使用 style.column_gap（column-gap 长写属性）。同时修复 gap 简写解析：`gap: 10px` 现在同时设置 column_gap 和 row_gap。使用 fallback 策略：column_gap 非 0 时优先，否则使用 gap。 |
| 2026-06-08 | background-image 固有尺寸基础设施 | Painter 新增 image_sizes HashMap<u64, (f32, f32)>（url hash → intrinsic dimensions）。RenderPipeline.set_image_sizes() 将缓存传递给 Painter。reftest runner 在渲染前构建 ImageCache、提取固有尺寸。修复了 background-size: auto 拉伸到容器大小的问题。 |
| 2026-06-08 | is_block_level / is_relative 标志 | LayoutBox 新增两个布尔标志。is_block_level 用于 float/clear 后处理（CSS 规范 clear 仅适用于块级元素）。is_relative 用于 table 布局后处理保留 position:relative 的 inset 偏移。 |
| 2026-06-08 | gap 简写 handler 修复 | gap apply handler 不再设置 column_gap/row_gap（由各自的 longhand handler 通过 shorthand expansion 设置），避免 HashMap 迭代顺序不确定性导致的值覆盖。 |
| 2026-06-09 | reftest 分类容差 bug 修复 | 上游 reftest FileReftestCase::to_config() 使用 Default::default()（1%/5ch），未调用 ReftestConfig::for_category()。所有测试被以严格布局容差（1%）衡量，导致文字类测试（5% 容差）大量误判失败。修复后通过率 68.6%→73.5%（+24 测试）。新增 ReftestConfig::with_viewport() builder 方法。 |
| 2026-06-09 | columns 简写解析修复 | `columns: 3`（单整数）被 parse_column_width 先解析为 column-width: 3px，阻止 parse_column_count 执行。CSS 规范要求整数优先解析为 column-count。交换解析顺序后，`columns: N` 正确设置 column-count: N。 |
| 2026-06-09 | 零高度浮动处理 | adjust_float_positions Phase 1 中 line_max_height 跳过零高度浮动元素（child_outer_height == 0），避免空浮动元素推进后续浮动的 Y 位置。 |
| 2026-06-08 | table 行组位置更新 | position_cells 后新增 update_row_group_positions 后处理。按视觉顺序（thead→tbody→tfoot）计算行组的 y 位置和高度，含 border-spacing。支持 position:relative inset 从行组传播到子行。修复 out-of-order-elements-collapsed-border（46.32%→通过）。 |
| 2026-06-08 | CSS 绝对长度单位 | parse_length() 新增 in/pt/pc/cm/mm/Q 单位支持，按 CSS 规范转换为 px（96 DPI）。修复了所有使用 `height: 1in; width: 1in` 的 floats-clear 测试（之前 in 单位被静默忽略，元素折叠为 0 大小）。副作用：CSS2/borders 中使用 1in(=96px) 大边框的测试暴露了布局精度差异。 |
| 2026-06-08 | CSS inherit 关键字完善 | border/background shorthand 正确广播 CSS-wide keywords（inherit/initial/unset）到所有子属性。inherit_property 扩展支持非继承属性（background-*, border-*, margin-*, padding-*），使 `border-bottom: inherit` 等显式继承生效。 |
| 2026-06-08 | is_block_level 修正 | table 内部 display types（TableRowGroup, TableRow, TableCell 等）从 is_block_level 中移除。CSS 2.1 规定 clear 属性仅适用于块级元素，table 内部元素不是块级元素。 |
| 2026-06-08 | 参考文件过滤 | reftest loader 跳过以 -ref/-reference 结尾的文件名，避免参考页面被当作测试用例运行。移除 1 个误计入的测试（float-nowrap-3-ref.html）。 |
| 2026-06-08 | XHTML CDATA 调查 | 调查发现 html5ever 在 HTML 模式下将 XHTML CDATA 标记（`<![CDATA[...]]>`）保留在 `<style>` 文本内容中。CSS 解析器遇到 `<![CDATA[` 时错误恢复路径触发 `skip_to_rbracket()`，贪婪吞噬后续所有 token，导致整个样式表提取 0 条规则。之前通过 CDATA 损坏的 .xht 测试（test+ref 都无 CSS）实际是虚假通过。 |
| 2026-06-08 | XHTML CDATA 清理实施 | `strip_cdata()` 在 `collect_stylesheets()` 中去除 CDATA 前后缀。揭示真实通过率 66.1%（之前 76.4% 含虚假通过）。真实修复：background-087/326/328 ✅。揭示的渲染缺口：writing-modes 42.4%（需 writing-mode 布局支持）、multicol 49.1%（需 column breaking）、floats-clear 新增 6 个差异。 |
| 2026-06-08 | empty-cells border-collapse 修复 | `empty-cells: hide` 仅在 separated border model 中生效。在 collapsed border model 中，空单元格仍需显示边框。修改 paint_node 两处 skip_empty_cell 条件添加 `border_collapse == Separate` 检查。 |
| 2026-06-08 | row-group/row box model 抑制 | CSS 2.1 Section 17.5.3/17.5.4：在 separated border model 中，table-row-group 和 table-row 的 border/padding/margin 无视觉效果。新增 `suppress_row_group_row_box_model()` 和 `zero_box_model()` 函数。 |
| 2026-06-08 | table cell explicit height 保留 | 有明确 height 且 overflow:hidden/scroll/clip 的单元格保持 taffy 计算的原始高度，不被行高覆盖。修复 table-cell-overflow-explicit-height 测试。 |
| 2026-06-08 | CSS 2.1 Appendix E 绘制顺序 | paint_node_in_rect 和 paint_node 中子元素分两轮绘制：先绘制非 float 子元素，再绘制 float 子元素。确保 float 内容视觉上在 block 背景之上（CSS 2.1 Appendix E）。 |
| 2026-06-08 | columns 简写顺序无关解析 | expand_columns() 双值模式改为自动检测哪个是整数（column-count）哪个是长度（column-width），而非硬编码 parts[0]/[1]。修复 `columns: 100px 6` 等逆序声明。 |
| 2026-06-08 | clearance 计算代码质量改善 | 澄清 CSS 2.1 §9.5.2 clearance 语义：零 clearance 仍然阻止 margin 折叠；clearance = max(0, clear_bottom - hypothetical_position)。后处理方式的局限性在于 taffy 已应用 margin 折叠。 |
| 2026-06-08 | 空 inline 元素 line-height | 空 inline 元素（如 `<span></span>`）生成零宽度 TextRun，其 line-height 仍贡献到行盒高度。修改 collect_inline_items 不再跳过空 inline 元素。 |
| 2026-06-08 | sibling combinators 文本节点跳过 | NextSibling (+) 和 SubsequentSibling (~) 组合器现在跳过元素间的文本节点，匹配 CSS 选择器规范行为。修改 matches_selector_recursive 和 matches_has_selector_chain。 |
| 2026-06-08 | CSS 绝对长度单位 | parse_length() 新增 in/pt/pc/cm/mm/Q 单位（96 DPI），background 简写分类器新增所有长度后缀。修复使用 1in 高度的 floats-clear 测试。 |
| 2026-06-08 | 径向渐变位置修复 | gradient_to_primitive 改用 resolve_position() 正确处理 Percentage（百分比）和 Px（绝对像素），替代旧的 length_to_f32/100 逻辑。修复相关测试用例。 |
| 2026-06-08 | 表格 min-height border-box | apply_table_size_constraints 正确处理 min-height/max-height 为 border-box 约束（减去 padding+border）。修复 min-height-table。 |
| 2026-06-08 | 表格单元格高度最小值 | CSS 2.1 规范中 cell height 为最小高度，改用 max(row_height, cell_content_height)。 |
| 2026-06-14~17 | R118–R227 逐轮技术决策 | 已归档至 [`archive/tech-decisions-r118-r227.md`](./archive/tech-decisions-r118-r227.md)（R118–R139 同见 `rounds-r23-r139.md`，R142–R227 同见 `rounds-r142-r302.md`）；当前 plateau 结论与 ruled-out 杠杆见顶部「综合裁决」 |

---

## 下一步

> R305–R323 已确认结构性 plateau（见上方「综合裁决」）。下列为 rally 可通过 `master.md` 记录 + `CONTINUE: <下一步>` 续跑承接的**跨会话架构方向**；单会话 rally 的 firm lever 已穷尽，**两条 narrow vein 经 R647/R648 双双收口**（css-text near-miss R647 穷尽剩余 12 簇 / per-fragment inline R648 text-decoration 已正确穷尽）。**残余单 session 选项** = text-autospace feature（+2~4）/ 0.00% 簇方法论外推到 css-text-decor·writing-modes（**R652 实测关闭**：css-text-decor 仅 4 簇 = skip-ink moderate-to-significant feature（font API `query_glyph_metrics` 仅返 `(glyph_id, advance)`、无 per-glyph ink bounds——须先扩 font 抽象暴露 ink bbox + decoration paint 加 orientation 感知裁剪；R655 复核，非 quick win）/ writing-modes **0 簇**，全 Match-failed 5-88% 结构性 vertical block-flow R631/R109 territory → 0.00% 簇 quick-win 全目录穷尽）；**主轴**仍是 Phase A。下列具体 item 1–4 已 CLOSED/stale（R635 标注，保留作历史参考）。

### 当前可推进的 ZeroWeb-side code lever（供并行 code agent，doc verified）

> **所有历史 code lever 已 CLOSED/stale（R635 标注）**：下列 item 1–4 经 doc-side read-only 实证定位 seam + A/B 验证后全部 LANDED 或 REFUTED，保留作历史索引（详情见各 evidence + git history）。**当前无活跃单会话 code lever**；唯两条 narrow vein 仍 open（见「综合裁决」末段）。勿再以这些为活跃入口重投。

| # | lever | 状态 | 落点（commit / 验证） |
|---|-------|------|----------------------|
| 1 | abspos root-CB fix（positioning 005/006） | ✅ LANDED `0436c039` | `resolve_abspos_against_root_cb`（engine.rs 步 11.7）；positioning 全量 A/B +2/0 零回归（005/006 FAIL→PASS，残差 0.44% = fontdue 绿文本噪声） |
| 2 | HTML presentational hints → CSS（CSS2 App D） | ✅ LANDED R502，+7/0 | `collect_presentational_hints`（lib.rs）；normal-flow +3 / backgrounds +3 / borders +1；block C（absolute-replaced-width-006）经 R549 border-width-medium 解锁（非 R490 死路） |
| 3 | flex-grow / flex-shrink 负值拒绝 | ⛔ REFUTED R524 | apply.rs 拒绝负值 = byte-identical zero-yield（taffy `sum_flex_grow>0` 门控已 no-grow）；driving tests 经 R525（batch_fills paint-order）/ R530（负值→initial）不同 fix 现 PASS |
| 4 | R546 缺失 ref 暴露的 12 候选（sizing / applies-to / currentColor / missing-asset） | 全 LANDED 或 REFUTED | R548 col/colgroup min/max +4 ✅ / R550 currentColor +4 ✅ `af9b59bc` / R549 border-width medium ✅ `ebff2595` / R583–R588 cell+table min/max applies-to ✅ / R587–R588 百分比 min-height+根 height ✅ / R601 missing-asset 23 资产 ✅ `747c8c44` / R590 missing-ref 0/10232 彻底穷尽 / inline-replaced-width-012 残留 1 案 structural / margin-collapse-vrl-022 R625 证伪（vertical-rl 布局结构性，非 axis-swap 单点）/ 014 inline-table defer Phase A（inline-level box 缺口，非 sizing） |

> **doc/research 4 大 lever vein 单会话穷尽**（R491–R546 谱系，详见下方「已 ruled out」）：(1) CSS-correctness unimplemented-clause（R491–R518，§9.7 float inline-level 块化补全）；(2) validity-rejection 负值族（R510/R513c/R516c/R517，cascade 累计 self-source +10 LANDED；calc() 负值须求值，out-of-scope）；(3) dead-value §9.7 探针（R513，8 个非-CSS2 目录 CLOSED 无 yield；font-feature 族单一硬阻塞入口 `render-foundation/src/font/shaper.rs:82` `features: Vec::new()`，待 Phase-A layout+paint 同源）；(4) inheritance（R516d，`image-rendering` 误标非继承但 0 driving WPT）。唯一遗留 actionable = font-feature 族解锁（Phase-A same-source 子集，多会话）。

### 跨会话续跑入口

- [x] **★ #1 = FreeType C-dep 翻 default = ✅ LANDED R1159（2026-07-08）**：`freetype-raster` feature R1068 LANDED default-off → **R1159 翻 default-on**（`crates/render-foundation/Cargo.toml` `default=["freetype-raster"]`）。用户已明确 freetype 编译问题解决，C-dep 默认开启路径解锁。本机 A/B 实测零回归：css-text oracle +24（复现 R1094）+ welcome −0.28pp（复现 R1068）+ make test 12205/0 + clippy clean。R1094 全 corpus +232 零回归转生产默认。**font-wall + Phase A 非-Ahem + ::first-letter 三组阻塞已按 R1159/R1160/R1206 的最新结论拆分**：FreeType default 已落地；Phase A / ::first-letter 不再按“等待 C-dep”处理，而按后续具体技术路径继续推进。
- [ ] **按 rally 续跑协议推进结构性 plateau**：7-目录全量 self 1148/2032=56.5% / oracle 629/1726=36.4%（grid/position/tables/flexbox/multicol/text-decor/fonts）+ 456/518 loose（含子集）/ 295/490 strict（pre-grid-expand）/ ~42% Oracle（broad, pre-grid-expand）是诚实基线。剩余提升需 Phase A IFC 统一 / Phase 2 嵌套 multicol / baseline 合成 或 taffy 升级，均为跨会话工程；这不是人工授权卡点，后续 session 应通过更新 `master.md` 记录阶段事实，并在未完成时输出 `CONTINUE: <下一步>` 传递执行意图。**★ R626-R627 状态更新（2026-06-25）**：(1) DC-14 分母全量去子集化已 10/10 目录完成（含 writing-modes/css-text/CSS2）；(2) stale-oracle 债清偿（R626b/c/d：flexbox/fonts/multicol/writing-modes oracle 重抓 post-R601，CSS2 staleness 低影响 ~0.2% 保留）；fresh-oracle chr<1% 准确数：flexbox 47.7% / fonts 34.4% / multicol 23.2%（R626d）；(3) **font_size 线 R627 定论关闭**——3 个 seam（inline_layout 门控 R84/R355 / gate-entry R125/R198/R205 / font_size-value 存储 R627）全 net-negative，根因 = layout-IFC vs paint-IFC **双路径重跑发散**（paint 带 overrides 重跑必与 layout 行断分歧，pre-wrap 宽度敏感实证 -15）；真修复 = Phase A IFC **统一**（paint 复用 layout 单次结果），但**又被 estimate-vs-fontdue 墙阻塞**（layout 用 estimate_char_width，paint 用 fontdue，非-Ahem 二者不一致；advance-width 统一 R225/R320 亦 net-negative）。**故 control-chars/large-font(非-Ahem)/welcome-morning 文本度量残余 = estimate-vs-fontdue 架构性**，须跨会话字体度量统一（非单 session）。**裁决**：单 session clean lever 谱系穷尽，forward motion 全部进入 rally 可续跑的架构任务（Phase A IFC 统一 + estimate-vs-fontdue 字体度量统一 / multicol fragmentation / baseline-export / font-features rustybuzz 同源）。

### 跨会话架构任务（按依赖序）

1. **Phase A IFC 统一**（[`phase-a-IFC-unification-design.md`](./phase-a-IFC-unification-design.md)）— 解 large-font（ifc-008/009/011）+ welcome/morning.work 文本度量残余。R207 narrow 已证 font-051 +1 可行；需多轮 set-diff 收敛 broad 应用 + 守 multicol-fill-auto 反向依赖（R198 墙）。 **★ R639 de-risking（首步可 incrementally 续跑，非 big-bang）**：`Painter.inline_heights`（NodeId→box.height 预扫描）是通用 Phase A 桥接机制，per-fragment inline-bg R639 已 LANDED（+13）+ per-fragment color R358 + 跨 block float 侵入 R362 各 LANDED——实证 Phase A **非「全有或全无」**，可按 narrow slice 逐项提交（每项独立 A/B 守回归、零回归即留），降低首步风险。
   - **★ R885 LANDED（§12.6 step-1 font-bridge，dormant 零回归，commit `d5b7e3ae`）**：`crates/layout-engine/src/inline/font_metrics.rs` 新模块 = `LineMetrics` + `FontMetricProvider` trait + `impl FontMetricProvider for FontLoader`（family→font_id→fontdue `line_metrics_full`→ascent/descent/line_gap）+ `InlineFormattingContext.font_metric_provider`（`Option<FontMetricProviderHandle>`，默认 `None`）+ builder。**dormant**：`apply_vertical_alignment` 仍走 `0.8·fs`（grep 证 0 production reads）。零回归证据：make test 全绿（layout-engine 949 含 4 新 font_metrics 测试断言真实 Ahem 度量 ascent≈0.8·size/descent≈−0.2·size）、clippy `-D warnings` clean、fmt clean、product-smoke welcome **16.11% ≈ baseline 16.16%**。关键洞察：`0.8` 对 Ahem **恰好正确**（ascent=800/upem=1000=0.8em），对真实字体（system-ui/DejaVu/NotoSansCJK）ascent≈0.928em 故基线偏低——bridge 暴露 per-font 真实 ascent 使 step-2 能按字体取正确值。
   - **▶ step-2（下一会话，§12.6 step-2 三方协调）**：① **5-layer FontLoader wiring**（R887 已测绘路径）：app 层创建处（`apps/renderer/src/main.rs:154 load_system_fonts()` / webview-demo / browser）把 `FontLoader` 包 `Rc` 经新增 `WebView::set_font_metric_provider(Rc<dyn FontMetricProvider>)`（镜像既有 `set_font_resolver`，webview.rs:746）→ `RenderPipeline::set_font_metric_provider`（pipeline.rs，新增字段，与 `font_resolver` 并列——注意 pipeline 现仅持 `font_resolver: HashMap<String,u32>` family→id **快照**，**不持 FontLoader 本身**，故须新线程 `Rc<FontLoader>`）→ `LayoutEngine::set_font_metric_provider`（新增字段 + setter，镜像 `set_viewport`）→ `compute_with_img_sizes` 内 `InlineFormattingContext::new`（engine.rs:1416/1545）调 `.with_font_metric_provider(Rc::clone(&p))`；② IFC 携带 container_font_family + per-run family（TextRun 加字段，`layout()` 从 `styles.get(&container).font_family` 读）；③ `apply_vertical_alignment` 用 `provider.line_metrics(...).ascent` 替换 `dominant_fs*0.8`/`container_font_size*0.8`（mod.rs:1571/1573/1583）+ paint v_offset（text.rs:1359 `font_size`→ink-height，concept ②，须 render-foundation 暴露 glyph ink bounds，R655/R876 待补）；④ 三态门禁 A/B：welcome <20% + linebox/css-text/normal-flow oracle 零回归 + self-source 不降，**净负即回退**（§12.4 已有 R834/R836/R849/R875 四次单点 net-negative 先例，须三方同改非单点）。**wiring 单独不可提交**（field 不读→无单元测试可验证），须与至少一处 read（concept ①）同提交；但 ① 单点已证 net-negative（R834），故 wiring 须待完整三方方案才落地 = 多会话。
   - **R887 验证 + css-position 清扫（负结果，2026-06-30）**：实测复核当前状态——product-smoke welcome **16.11%**（font-bridge dormant 零回归确认）、reftest-oracle css-position 97 案 oracle-pass 45.4%/strict 真通过 7.2%（self-source ~56.5%/DC-14 46.5% 假通过）、css2 75%（12 案小目录）。css-position top 候选清扫**无 clean 单会话 lever**：replaced-object-backdrop 100%/backdrop-inherit-rendered 47.5%=backdrop-filter（DC-12 未实现 OOS）；semi-replaced-stretch-input/other 21%/13.6%=form-control semi-replaced sizing（ZW 不建模表单控件内在尺寸）；position-absolute-dynamic-relayout/hypothetical-dynamic-change=JS 重排；abspos-in-inline-006=R109 结构性。**root-element 集群（position-absolute/fixed-root-element-flex/grid，4 案全 4.09%）经空间 diff 诊断 = dashed-border DASH PHASE 噪声非布局**：diff bbox x[7,779]y[27,559] 恰 = 元素 border 周长（inset 10/20/30/40），87% diff 集中 near-edge，采样像素纯黑白 on/off（ZW 在角 (10,27) 落 dash、chromium 落 gap）= 相位差；元素**已正确拉伸**（非 stretch bug）。R795 已校准 dash ratio（dash=2×width/gap=1×width，chromium 8px 实测），残余相位对齐 = 渐减 fiddly 任务（251 文件含 dashed border，但逐案 4%→2% 非 clean win，defer）。**结论：css-position 单会话 clean lever 已耗尽**（与 R870-R881 谱系在 normal-flow/positioning/backgrounds/float 已收割一致），forward motion = Phase A step-2（多会话）。
   - **R888 CSS2 全量清扫（负结果，2026-06-30，plateau 定论）**：reftest-oracle DIR=CSS2 = **6283 案**，oracle-pass 48.3% / strict 真通过仅 **1.5%（96/6283）**（strict 0.1%/0.5% 门限下字体光栅噪声主导，非真布局缺口）。Top 候选**全部非 clean 单会话 lever**：background-root-101/102/103（3×100%）= **JS DOM mutation**（ZW harness 跑 JS 但不把 DOM 变更反映到 layout → 渲染初态 vs chromium JS 后终态）；font-family-invalid-characters-003（100%）= CSS **parser 花括号 error-recovery**（fiddly）；pagination/float-page-break-inside-avoid-*-print（99%）= **@media print 分页**（OOS/未实现）；inline-svg-100-percent-in-body（97%）= **inline SVG**（OOS）；before-after-table-parts-001（93%）/collapsing-border-model-010b（89%）= generated-content+table / border-collapse **结构性**；bidi-008/009 簇（56-73%）= **BiDi 算法**（已集成 unicode-bidi 但复杂边界）。**两最大目录（css-position 97 + CSS2 6283 = 6380 案）清扫定论：单会话 clean code lever 全耗尽**，残余 = 多会话架构（Phase A step-2 / Phase 2 multicol / baseline-export）+ OOS（SVG/JS-DOM/print）+ fiddly（BiDi/parser-recovery/dashed-phase）。下会话勿重扫这两目录，直入 Phase A step-2 wiring。
   - **R889 step-2 wiring 实验（负结果 + 关键路径修正，2026-06-30）**：实测尝试 concept ①（strut 用真实 ascent 替 0.8）的最小 wiring（layout-engine `LayoutEngine::set_font_loader` + IFC `container_font_family`/`ascent_ratio_for` + `RenderPipeline::set_font_loader` + wpt-runner CPU 渲染路径包 `Rc<FontLoader>` 并 `set_font_loader`），编译通过、make test 绿。**A/B product-smoke welcome = 16.11% 完全不变（77305 px 一字不差）**。诊断根因：**主文本 IFC 路径不是 `engine.rs::adjust_inline_block_positions`（line 1443，仅处理含 inline-block 子元素的容器），而是 `inline_finalization.rs::compute_final_inline_layouts`（engine.rs:418 调用，内含 5 个 IFC 站点 495/748/867/1004/1022）**——welcome 正文走 inline_finalization，未触达实验 wiring 故无变化。**关键修正：R887 step-2 wiring 路径标注的 engine.rs:1416/1545 错误**（那俩是 adjust_inline_block_positions + fix_vertical_mode_abs_pos，非主文本路径）；**真 wiring 目标 = `compute_final_inline_layouts`（inline_finalization.rs）的 5 个 IFC 站点**，须把 provider 经 `compute_with_img_sizes`(engine.rs:418 调用处，self 可用) → `compute_final_inline_layouts` 新增 provider 参数 → 内部 5 站点 `.with_font_metric_provider`。已 revert 全部实验改动（clean R888）。**结论**：step-2 wiring 比 R887 测绘的更重（5 站点 + inline_finalization 调用链，非单站点），且 §12.4 strut 单点 net-negative 先例仍成立（welcome 走对路径后大概率仍退步，须三方同改）。下会话：wire `compute_final_inline_layouts` 5 站点 → A/B welcome（验证 concept ① 是否触达 + 方向），净负即回退。
   - **R890 step-2 full wiring 实验（决定性负结果：R72 empty-styles 是 Phase A 硬阻塞，2026-06-30）**：完整 wiring concept ①：`LayoutEngine::set_font_loader`(field+setter) + `RenderPipeline::set_font_loader`(store+forward) + `Painter::set_font_loader`(+font_loader field) + 所有 painter setup 站点传 `font_loader.clone()` + paint Path B IFC（text.rs:888）注入 provider + layout `compute_final_inline_layouts`(@495) 注入 provider + IFC `container_font_family`/`ascent_ratio_for`(关联函数避 borrow 冲突) + strut+per-run ascent 用真实 0.928 替 0.8 + wpt-runner 包 `Rc<FontLoader>`。编译通过、make test 绿。**A/B welcome 仍 16.11% 一字不变（3 次连续：R889 错路径 + 本轮 layout 路径 + 本轮 paint 路径全 77305 px）**。**PHASEA_DEBUG 插桩定论**：provider **确实工作**（`family=["sans-serif"] -> 0.9282 provider hit`，证 DejaVuSans ascent=0.928 与 §12.2 吻合），且已送达 paint Path B IFC（`provider=true`）；**但绝大多 call 是 `family=[] -> 0.8 fallback`**——根因：**paint Path B IFC 调 `ctx.layout(doc, node_id, &HashMap::new())`（text.rs:977，空 styles，R72 安全路径）→ `container_font_family` 永远空 → provider 无法解析 family → 回退 0.8 → 无变化**。**★ 决定性结论：(1) layout IFC 改动被 paint Path B 重跑覆盖（R889+本轮）；(2) paint Path B 用空 styles（R72）故无 container font_family → font-bridge 即使全 wired 也 fallback；(3) §12.4 R834/R836/R849/R875「strut/v_offset net-negative」先例极可能就是这些 empty-styles fallback 假象（实际度量从未真正改变），即 concept ① 在决定渲染的 paint 路径上**从未被真测过**。已 revert 全部（clean R889）。**▶ 真正解锁路径（已发现现成模式）**：`inline_finalization.rs::store_font_sizes_from_ifc`（line 124）**已用 override-map 模式绕过 empty-styles**——它把 layout IFC 的 font_size/is_ahem/letter_spacing/line_height 存进 `LayoutBox.text_node_*`，paint Path B 经 `with_font_size_overrides` 等 builder 读取。**concept ① 须照此模式**：在 layout IFC（有 provider，算出真实 strut/per-run ascent）把**每文本节点的真实 ascent（或 baseline_y）**存入 LayoutBox 新字段（如 `text_node_ascent`），paint Path B 经新 override map 读取，**绕过空 styles + provider family 解析**。另注：`store_inline_layout_results`(line 72) 已是完整 Phase A 统一机制（存权威行盒供 paint 复用）但 `#[allow(dead_code)]` + `TODO 基线计算修复后启用`——长期搁置。下会话首选 store_font_sizes_from_ifc 式 ascent override（narrow），失败再考虑 store_inline_layout_results 统一（broad）。
   - **R891 concept ② real-path 决定性测试（font-metric 单点杠杆对 welcome 终局证伪，2026-06-30）**：R890 发现 concept ①（IFC strut）是 no-op（empty-styles）后，转测 **concept ②（§12.2/R877 真路径 = `render_fragment!` `$baseline_offset`，text.rs 非存储 else 分支）**：把非-Ahem baseline_offset 从 `baseline_fs`（=font_size=1.0em）改为 `baseline_fs*0.928`（=真实 ascent，is_ahem gating 防 Ahem 回归），1 行改动。**A/B welcome = 77275 px（16.10%）vs baseline 77305 px（16.11%）= −30 px / −0.01pp**（方向略好但**可忽略**，在噪声内）。**★ 终局证伪**：Phase A font-metric **两个单点杠杆都对 welcome 无实质影响**——① IFC strut = no-op（R889/R890），② render_fragment baseline_offset = −0.01pp negligible（本轮，真路径）。结合 R834（d464f0ad "real-ascent lever REFUTED, pipeline-coherence not single-knob"）+ R877（真路径=render_fragment）+ §12.4（4 次单点 net-negative）：**Phase A font-metric 单点杠杆对 welcome 已穷尽证伪**。**真因（master.md R630/R632 已定位）**：welcome 16% 残余 = **CJK 字体度量（R633 Phase A 死锁）+ R109 inline→block 结构性 + hljs（需 JS）**，非 Latin font-metric（baseline_offset/ascent）；② 影响 CJK+Latin ascent 故仅 −0.01pp。**裁决**：font-bridge（R885）作为 enabling infra 保留，但 **welcome 不再以 font-metric 单点/三方为 lever**（② 测得三方也只会是 negligible 累加）。welcome 提升 = R109 IFC 统一（P1，多会话）+ hljs（JS 运行时）+ CJK 度量死锁，均非 font-metric 单点。已 revert（clean R890）。**▶ 下会话方向重定向**：font-metric 侧改为「**非 welcome 的 font-metric 敏感 fixture/reftest**」（如 linebox/css-text 非-Ahem 文本类，或 CJK 度量死锁的 R633 谱系），或转 R109 IFC 统一（welcome 真正 P1 缺口）；勿再以 welcome font-metric 单点/三方为 lever。
2. **Phase 2 嵌套 multicol fragmentation**（[`multicol-fragmentation-design.md`](./multicol-fragmentation-design.md)、[`column-aware-IFC-spec.md`](./column-aware-IFC-spec.md)）— 解 multicol-breaking（css-multicol 最大失败聚类）。R319 证纯 inline 迁移零增益，真实价值在嵌套 / 混合内容碎片化。
   - **R892 css-tables 扫描（第 3 个目录 plateau 确认）+ multicol 轨道深度复核（2022-06-30）**：实测 product-smoke welcome = **16.11%（< 20% DC-13 gate PASS）**；reftest-oracle css-tables 115 案 oracle-pass 53.9%/strict 5.2%。css-tables top 候选经逐案诊断**无 clean 单会话 lever**：table-row-group-color-inheritance-001（9.07%）经渲染 + 空间分析 = **table-fixup 结构性**（ZW 不渲染 200px Ahem "X"——table-row-group 直文本子的匿名 row/cell fixup 缺陷，非颜色继承）；min-max-size-table-content-box（4.27%）= subtle sizing 边缘（min/max-width **已实现**于 table_shrink.rs:109/119/211/221 + compute_column_widths，diff 是 content-box × grid context 边缘，非未实现 clause）；table-cell-width-0（20%）= R97 intrinsic sizing（taffy-blocked）；percentages-grandchildren-quirks-mode（14.93%）= quirks % height subtle；dynamic-*=JS；collapsed-border-vertical-*=writing-modes+border-collapse 复杂。**至此 css-position/CSS2/css-tables 三目录（6498 案）plateau 定论**，R870-R881 CSS-correctness vein 确认全收割。**multicol 轨道复核**（multicol-fragmentation-design.md v0.4）：碎片化算法 `assign_children_to_columns_*` **已存在**；paint 侧 4 轮（R157/R198/R203/R317）+ Round 1' baseline-export（R316）**全证伪**；真修复 = **layout 侧 column-aware IFC**（layout 期算 multicol IFC 行盒 → 按列高预算碎片化 → 存 LayoutBox → paint 消费，类 Phase A 统一但 multicol-specific），文档明示「下一步（多会话 spec-rfc）」。**裁决**：所有剩余轨道（Phase A 已证伪 / multicol / baseline-export / R109）均多会话架构 + 有 prior 证伪，无单会话 clean lever。welcome DC-13 gate 已 PASS（16%<20%），WPT DC-2~5 受字体光栅 strict 噪声 + 结构性主导。**▶ 下会话**：启动 multicol layout-side column-aware IFC 的 spec-rfc 设计（lei-spec-rfc skill），Phase 1 = 死字段 + 测量基线（净 0，守 multicol-fill-auto-001 sentinel，类 font-bridge R885）；或 baseline-export 的 taffy-internal/自建 first-baseline 设计。勿再扫 css-position/CSS2/css-tables（plateau）。
   - **R893 multicol 扫描（第 4 个目录 plateau）+ 策略转向 DC-11/12 feature 验证（2026-06-30）**：reftest-oracle css-multicol 452 案 oracle-pass **23.0%**/strict 1.5%（与记录 23.5% 一致，最低 oracle 目录）。Top 候选**全部非 clean lever**：column-balancing-paged-001-print（81%）=@media print OOS；multicol-rule-nested-balancing-003/004（37%/20%）=nested balancing Phase 2 hardcore；**multicol-span-all-children-height-00X（5 案 15-30%）= R700 已证结构性**；multicol-breaking-005（23.47%）经读源 = **nested multicol**（outer column-count:3 + inner column-count:2，both balance，内容跨列碎片化）= Phase 2 hardcore（非 wiring bug）；subpixel-column-rule-width（23%）=亚像素渲染 fiddly；multicol-contained-absolute/overflow-clip-positioned=niche abspos×multicol。**column-aware-IFC-spec Phase 1 已 R381 紧急停止**（0/16 案匹配单层+balance+明确高度+纯 inline），真 multicol 工作全需 Phase 2（nested/breaking/混合，多会话硬核）。**至此 css-position/CSS2/css-tables/css-multicol 四目录（7380 案）plateau 铁定**。**★ 策略裁决（R893）**：reftest/oracle 侧 plateau 铁定（font-raster strict 噪声 + 结构性簇，单会话 clean lever 全穷尽，多会话硬核轨道均有 prior 证伪）。**转向 DC-11/DC-12 feature gap 验证**——这些 DC 项是 feature 完整性（非 reftest 率），直接推进不受 font-raster 限制：DC-11 未完成项（Float 完整布局 / position:sticky / overflow:scroll-auto 真滚动容器）+ DC-12 未勾选项（border-image / backdrop-filter / CSS mask / scroll-snap / @media print——M9 声称 ✅ 但 goal checkbox 未勾，需实测验证真实状态 + 补缺口）。**▶ 下会话**：实测验证一个 DC-11/12 feature 的真实状态（渲染一个对应 reftest/fixture 看是否真工作），若 gap 可修则补 + 单测；勿再扫 4 个 plateau 目录。
3. **baseline-export 真修复** — taffy 0.8+ baseline_overrides（R304 DEFER 升级）或自建 inline-level-box baseline 合成；解 flexbox-baseline / multicol-baseline 聚类。
4. **DC-9 blend_mode** — paint-isolation 架构（offscreen 子树渲染 + source/dest 双纹理 blend pass），低 reftest footprint（~2-4 案）。

### 历史轮次详记归档（master.md 瘦身）

> 为避免 master.md 无限膨胀，**R569–R1093 的逐轮详记已全部迁出 `archive/`**（内容 100% 保留，未去重、未重排）；master.md 现仅保留「综合裁决」表的结构化结论 + 当前活跃状态 + 跨会话架构入口。逐轮详记按 era 分档：
>
> - **R569–R881**（原 master 顶部 preamble 摘要段，157 条）→ [`archive/rounds-r569-r881-master-preamble-summaries.md`](./archive/rounds-r569-r881-master-preamble-summaries.md)
> - **R894–R990**（multicol Phase 2 / harness JS vein / R109 §9.2.1.1 backfill / aspect-ratio / R990 ascent era）→ [`archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md`](./archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md)
> - **R991–R1093**（multicol spanner·Phase 2 / logical props / vertical-mode / FreeType C-dep / ::first-letter Phase A / nbsp·word-spacing / plateau era）→ [`archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md`](./archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md)
>
> 更早 era（R11–R718）的归档清单见底部「最近轮次详细记录」。逐轮结论摘要亦见顶部「综合裁决」表。

### 已 ruled out（勿以单会话重试）

near-pass(R307) / POLLUTED hunt 三趟复核 R299–R309 + R311 + R329 / fresh-xval(R311) / Phase A 4 路 font_size(R125–R206) / multicol paint 侧(R157–R317) / balance 二分(R199–R322) / column-aware IFC 纯 inline(R319) / **column-aware IFC Phase 1（pure-inline balance 明确高度）(R381)**：执行 column-aware-IFC-spec.md §10 gate「假设 A1」，扫描全 16 css-multicol 失败案结构（height/column-fill/blockchildren），**0/16 匹配** Phase-1 目标（单层+balance+明确高度+纯 inline）——每案或有 block 子元素、或 height:auto、或 column-fill:auto、或 breaking/嵌套；spec 自身协议「A1 不存在→紧急停止转 Phase 2」生效，Phase 1 零杠杆关闭，真实 multicol lever = Phase 2（嵌套/breaking/混合碎片化，多会话硬核）/ baseline-export 3 机制(R266–R316) / **advance-width(R225–R375b) definitive 关闭**：R375 hand-crafted DejaVu 表 morning 16.41→19.14% + R375b fontdue-actual advance（临时加 fontdue dep+缓存 Font+metrics.advance_width）16.41→19.08%，双 variant 均退步；fontdue-actual（最后未测变体）亦证伪。根因：accurate DejaVuSans advance 使换行偏离 chromium（system-ui≠DejaVuSans 或换行算法不同），0.55 启发式碰巧更近。advance-width 非 morning cascade 根因/ blend post-process(R278) / font-weight -Bold(R229b) / taffy 升级(R304) / inline-flex·inline-grid width:auto shrink-to-fit（R370：probe 实证 inline-flex width:auto 同 inline-block 拉伸到满宽 800，是真 bug，但**零杠杆**——全 48 失败案 + product-smoke fixture 均不用 inline-flex/inline-grid width:auto；fix 需 flex_row_intrinsic_width（非 box_content_max_width，flex row 须求和 block 子元素非取 max），复杂且无 reftest/smoke 收益，按 code-guidelines「不做零价值修改」不修，勿再以单会话重试）/ **percent max-width/min-height/min-width clamping（R119 analog，doc-agent 复核 ~0 yield，闭）**：engine.rs:1408 仅 `clamp_percentage_max_height`，无 max-width/min 平行函数——但 max-width-091(percent)✓ + min-height-091/092(percent)✓ 均 PASS（block width 定值→taffy 直接钳；min-height 是测量期 floor 非 content re-clamp）；R119 缺口唯一 max-height-specific（auto-height 内容测量 re-clamp），已修即完整 percent-clamp，无平行 lever，勿以 R119 类比重扫 / **intrinsic-keyword sizing（max-content/min-content/fit-content，R97 谱系，doc-agent 复核 = 非 clean 单会话 lever）**：121 测试文件用此三关键字，但**全集中在 taffy-blocked 上下文**（css-multicol/tables/flexbox intrinsic-size/table-intrinsic-size/flex-item-*-content），CSS2 block/inline-block 上下文**仅 1 案且为 crash-test**（inline-negative-margin-minmax-crash-001，非 sizing-correctness）→ memory「block/inline-block 可独立做」slice **无 dedicated driving test**（~0 可测 yield）；max-content/min-content parse_basic.rs 解析但 resolve 丢信号→0（R97/max-content memory），修复须保留信号+shrink-to-fit 触发，grid/flex/multicol/table 受 taffy 容器不 shrink 阻塞 = 多会话/结构性，勿以单会话重扫。 / **NBSP/Unicode-space collapse (R651 read-only 复核·非 lever)**：`collapse_whitespace`（inline/mod.rs:231）用 Rust `char::is_whitespace()` 折叠 NBSP(U+00A0)/U+3000 等，违反 CSS Text 3 §4.1.1（仅 TAB/LF/FF/CR/space 可折叠）——真 correctness bug，但 collapse 上下文（normal/nowrap/pre-line）**无 reftest 覆盖**（white-space-collapse-001 是 testharness JS `assert_equals(offsetWidth)` 测，非 reftest）；NBSP reftests（white-space-pre-031/032/034/035）全在 `pre` 上下文（preserve 路径不经 collapse）实测 PASS @2.64%。无 driving reftest → 非 lever（product-smoke 影响 negligible，NBSP 罕见于 fixture），defer；R647 category (b) 的 NBSP 角度据此关闭。 / **per-font ascent wiring for monospace（R1023 re-confirm R1005）**：fontdue 0.9.3 实测 DejaVuSans ascent/fs=0.9282，**DejaVuSansMono ascent/fs=0.9282（完全相同）**——R990 is_ahem-gated 常数 0.928（派生自 DejaVuSans）**对 monospace 也精确正确**（css-text-decor 簇用 `font: monospace`，行盒 ascent 已被覆盖）。R1005「per-font wiring 当前零 yield（ZW 只加载 DejaVuSans/Ahem，ascent 均已被 R990 常数覆盖）」结论**对 monospace 角度再确认**。per-font ascent wiring（R1004 step-2）须等 @font-face webfont 加载真起效（non-DejaVuSans 容器字体）才有 yield，premature 勿投入。 / **root-element abspos stretch（R1023）net-negative without concurrent flex-item-text fix**：css-position 4 案簇 position-{absolute,fixed}-root-element-{flex,grid}（均 4.05%）= 根 `<html>` position:absolute/fixed + 全 length inset + auto 尺寸，taffy 对 root 不应用 abspos stretch（root 无 CB）→ ZW shrinkwrap 到文本。实施 `apply_root_position_stretch`（engine.rs，gate：仅根 + abspos/fixed + 全 Px inset + auto 尺寸 + HorizontalTb → 设 Relative + 定值 size viewport−insets + left/top 偏移）**结构正确**（border 正确填充视口）。但 A/B **net 负**：4 案 4.05%→4.76%（border 增益被 body 塌缩 diff 反超）——body 是 html 的 Element flex item，含文本 + `<br>` 子。**真根因（R1023b 纠正，非原「flex+text per-word」）**：block 容器默认 build 路径（tree.rs:876）只收 Element 子、跳过文本节点 → block flex item 含 Element 子（br）时成 `new_with_children` 非 leaf → measure 不触发 → intrinsic width = Element 子之和（br=0）→ w=0 → 文本 wrap 到 ~0 宽垂直堆叠。纯文本 flex item（无 Element 子）成 leaf + has_inline_content 正确测量（不受影响）。**该「block flex item 含文本+Element 子塌缩」是 root-abspos yield 的前置阻塞**，仅影响 flex/grid item 是 block 容器且同时含文本 + inline Element 子的场景（罕见于真实网页主导航，驱动案为 rootpos 4 案 body 含 text+br）。已回退 root-abspos fix；root-abspos 须与该塌缩修复合做才 net-positive（修复须守卫不 double-count 高度，多 session）。 / **root-element abspos stretch R1024 重试仍 net-negative（body flex-item sizing 未解）**：R1024 已修 body 塌缩（body 作 flex item leaf，w 正确），重试 root-abspos stretch（R1023 dormant fix）—— 4 案 **4.05%→4.52%（+0.47pp net 负）**，仍 net-negative，已回退。**R1025 纠正**（原「has_inline_content 把 br 测单行高度」**错误**——measure_text_content 的 Element-with-inline-content 路径用 IFC 测量，IFC **识 `<br>` 强制换行**，body 高度正确）：rootpos 残差 = body 作 flex item 在 stretched html 内的 main-axis 分配 + 文本定位 diff（chromium 对 body flex-basis:auto 在 stretched html 内的 grow/shrink 与 ZW 不同），非 br 多行问题。body 现已正确测量（leaf + IFC），rootpos 残差是 flex-item-in-stretched-container 的 main-size 算法差异，结构性多 session。root-abspos 第三次回退，勿以「body 已修」或「br 多行」重试。

> **R512（2026-06-23，read-only 再验证）**：全目录 self 通过率再核实与基线一致（tables 77/112、text-decor 244/250、grid 31/48、fonts 282/284、backgrounds ~82%、box-display ~50%、box ~48%、cascade ~97.4%、values 65.4%）。新增两条 ruled-out 机制：① **负值拒绝（numbers-units-006）须 cascade-level 校验**——★ **SUPERSEDED by R513c（commit 0b11a12d）**：cascade-level 版本已 LANDED + 独立验证（numbers-units-006 1.92%→0.00% PASS，values +1 零回归），即本条所述「须 cascade-level」的 clean slice 已落地（见顶部 R513c 条）；下列原机制分析仍准确——`cascade.rs:140` max_by_key 选单一 winner，`height:1in;height:-1px` 中 -1px 后声明 order 高 → winner=-1px，1in 被整体丢弃；apply-time 拒绝只回退到 initial（Auto→0）而非恢复 1in，与应用 -1px（taffy 钳 0）对无内容 div 同给 0 高故 diff 不变（R512 实验已回退）。CSS 合法性须在选 winner 时判定——此即 R513c 实现，负值族现全闭。② **ex 单位（units-002/003/004）须 font metric**——parse_basic.rs 无 `ex` 分支（声明被丢），但 units-004 需 Ahem 实际 x-height（0.8em）；style-system resolve_length 无字体访问，0.5em fallback 永久不过 units-004，勿以 0.5em 投。★ **〔已推进·见顶部 R544/R547〕**：R544 LANDED ex→Em 常数（先用 0.5em），**R547 修正为 0.8em（Ahem 实测 x-height，反推自 units-004/numbers-units-012）过 units-004（values +1）**；但 font-metric 管线仍未接（resolve_length 仍无字体访问），0.8em 是 **Ahem 专用常数**——真字体 ex 仍近似，R512 本条「须 font metric」的深层判断仍成立（fontdue 度量接入 = 多会话 lever）。另确认上轮 Phase-A 方向 = R506 已裁定的 R247 deadlock 同墙，勿以单会话重试。详见 [`evidence/r512-plateau-verification-2026-06-23.txt`](./evidence/r512-plateau-verification-2026-06-23.txt)。

> **R513（2026-06-23，read-only）**：把 §9.7-pattern **dead-value 探针**（属性 parse+store+apply 齐全但 layout/paint 从不读 → 渲染零效果，补一处 consume 即翻 reftest = §9.7 模式）外推到 8 个非-CSS2 全量目录。**CLOSED 无 yield**：每个 dead value 落三桶之一——(1) rustybuzz same-source 墙（**font-variant-numeric** 死值 footprint 强 ≈11 对但 `shaper.rs:82 features: Vec::new()` 是 font-feature 族单一硬阻塞，须 Phase-A layout+paint 同源）；(2) pervasive 结构改造非单 consume 点（**caption-side** → §17.4 table-wrapper；**box-decoration-break:clone** → fragmentation；**unicode-bidi/isolation/will-change/hyphens** → IFC/stacking/词典集成）；(3) 零 reftest footprint（cursor/scroll-snap-*/scroll-padding/margin 静态光栅无效果）。已核实 IMPLEMENTED（勿再查）：empty-cells/border-collapse/table-layout/border-spacing/vertical-align(inline+cell 双上下文)/writing-mode/direction/z-index/gap/order/flex-basis/contain:text-wrap:nowrap 全有局部 consume。★ 新增 actionable 精确定位：`render-foundation/src/font/shaper.rs:82` 是 font-feature 族（释放 ~11+ 对）的单一工程入口（Phase-A same-source 子集）。与 R512（失败案分类法）两套方法论收敛到同一穷尽结论。详见 [`evidence/r513-noncss2-section97-deadvalue-scan-2026-06-23.txt`](./evidence/r513-noncss2-section97-deadvalue-scan-2026-06-23.txt)。

> **R516d（commit 37d5b348，doc-side read-only Explore scan；与 R516c code 同轮 doc+code 并行，R516c=cascade full-scope win 见顶部）inheritance-gap vein = EXHAUSTED（无 lever）**：核查 inherit.rs（`inherit_property` 字段拷贝）+ inheritance.rs（`compute_inherited_style_with_quirks` 单一继承 chokepoint）+ registry.rs `is_inherited`（34 属性）。两列表**完全对称**（每实现属性 implicit + explicit inherit/unset/revert 路径均正确）。唯一 spec 偏差 = **`image-rendering`** 误标非继承（`is_inherited` 漏 + `extended.rs:100` 反向断言），但 **0 driving WPT**（503 manifest 条目无一用）+ **无功能效果**（engine 仅画 debug indicator，从不用于真实 image sampling）→ ~0 yield（R502 先例）。其他 is_inherited 漏的属性（orphans/widows/text-justify/line-break/text-orientation/hanging-punctuation/font-variant-*/text-align-all/text-wrap-mode/white-space-collapse）**完全未实现**（无 parse/apply 路径），非继承 bug = 全属性实现工程非单点 lever。★ **R175 custom-property 是唯一真实继承 bug（已修）**；standard-property inheritance vein 已采尽。**★ 至此 doc/research 单会话 lever 4 大 vein 全穷尽**：(1) CSS-correctness unimplemented-clause（R491–R511 + **R518**：§9.7 float inline-level 块化补全 R511 partial；R511 CSS2/R513 non-CSS2）；(2) validity-rejection（R510/R513c/**R516c**/**R517**/R515，negative-value 族系统闭合，cascade vein 累计 LANDED：R513c narrow +1 + R516c full-px +6 + R517 em% +3 = self-source +10；所有长度单位负值现均拒，仅剩 calc() 负值须求值 out-of-scope）；(3) dead-value（R513）；(4) inheritance（R516d）。唯一未实现 genuine lever = **R515 flex-grow/shrink**（apply.rs:577-587，+2，已 de-risked + 系统枚举确证，pending code agent）。

### 已完成里程碑（参考，非当前活跃）

- M1–M9 基础设施 + 渲染器图元覆盖 + 浏览器消费 + 布局正确性 + 高级视觉效果：**已完成**（见下方「里程碑完成状态」「Done Criteria 进度」）。
- M10 上游 WPT reftest：基础设施完成，通过率 plateau（456/518 loose，grid R401 扩到全量后），达标需 DC-14 全量去子集化 + 上述跨会话架构任务。

---

## 最近轮次详细记录

> 全部逐轮详记已归档（master.md 仅保留顶部「综合裁决」表的结构化结论，避免无限增长）：**R569–R881**（顶部 preamble 摘要段）→ [`archive/rounds-r569-r881-master-preamble-summaries.md`](./archive/rounds-r569-r881-master-preamble-summaries.md)；**R894–R990** → [`archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md`](./archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md)；**R991–R1093** → [`archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md`](./archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md)；R335–R336 → [`archive/rounds-r335-r336.md`](./archive/rounds-r335-r336.md)；R314–R334 → [`archive/rounds-r314-r334.md`](./archive/rounds-r314-r334.md)；R307–R313 → 各单轮归档（[`rounds-r307.md`](./archive/rounds-r307.md) … [`rounds-r313.md`](./archive/rounds-r313.md)）；R305–R306 → [`rounds-r305-r306.md`](./archive/rounds-r305-r306.md)；R304 → [`r304-taffy-upgrade-deferred.md`](./archive/r304-taffy-upgrade-deferred.md)；R303 → [`r303-dc9-gpu-primitive-audit.md`](./archive/r303-dc9-gpu-primitive-audit.md)；R142–R302 → [`rounds-r142-r302.md`](./archive/rounds-r142-r302.md)；R23–R139 → [`rounds-r23-r139.md`](./archive/rounds-r23-r139.md)；R11–R20 → [`rounds-r11-r20-reftest-investigation.md`](./archive/rounds-r11-r20-reftest-investigation.md)；R118–R227 技术决策表 → [`tech-decisions-r118-r227.md`](./archive/tech-decisions-r118-r227.md)。逐轮结论摘要见顶部「综合裁决」表。

> **归档完成（R397–R399）**：原 master.md 的 4 块 R68 时代 stale 节已全部迁出 archive/（内容 100% 保留 + 指针回指）——R335/R336 全文详记 → [`archive/rounds-r335-r336.md`](./archive/rounds-r335-r336.md)；M6 inline 基线明细 → [`archive/m6-inline-reftest-baseline-2026-06-07.md`](./archive/m6-inline-reftest-baseline-2026-06-07.md)；R69+ IFC 技术参考 → [`archive/r69-ifc-unification-technical-reference.md`](./archive/r69-ifc-unification-technical-reference.md)；卡点 #2–#9 框架 → [`archive/r68-other-blockers-framework.md`](./archive/r68-other-blockers-framework.md)（master.md 保留锚点 stub，`multicol-phase2-unified-column-flow-spec.md` 的「卡点 #2/#4」live 引用仍可解析）。master.md 803→508 行（累计 −295 行 / ~−21KB）。

> **归档扩容（master.md 瘦身，R569–R1093 逐轮详记迁出）**：原 master.md 顶部 preamble 的 `**Rnnn(...)**` 摘要段（R569–R881，157 条）+ `## 下一步` 节的全部逐轮 H3 详记（R894–R1093）已逐字迁出 `archive/`（内容 100% 保留，未去重、未重排），分别落入 [`archive/rounds-r569-r881-master-preamble-summaries.md`](./archive/rounds-r569-r881-master-preamble-summaries.md) / [`archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md`](./archive/rounds-r894-r990-multicol-harness-r109-aspect-era.md) / [`archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md`](./archive/rounds-r991-r1093-multicol-spanner-freetype-firstletter-era.md)。master.md 1318KB→~199KB（−85%），现可整文件读取；`## 下一步` 保留跨会话架构入口 + 历史归档指针，结构化结论仍在顶部「综合裁决」表。

