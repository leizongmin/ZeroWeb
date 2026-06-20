# R381–R393 逐轮详记（header 治理迁出）

> 本文件于 R395（doc-maintenance）从 `master.md` header（line 3）迁出。
> 因 header 经 R385 治理瘦身至 ~4500 字符后，R386–R394 持续逐轮追加，膨胀至 18902 字符
> （违反 goal doc「master.md 不允许无限增长」「header 保持简洁」治理原则）。
> 结构化结论保留在 master.md「综合裁决」表（每轮一行）；本文件为逐轮 verbose 备查记录。
> 更早逐轮详记见 [`rounds-r351-r380-header.md`](./rounds-r351-r380-header.md)（R351–R380）、
> [`rounds-r314-r334.md`](./rounds-r314-r334.md) 等。

---

## R394 — DC-14 分母真实性 gap（N_imported 仅上游 ~5-6%）

渲染侧 plateau 5 轮确认后（R384/R389/R390/R392/R393）转查 DC-14 分母去子集化。经 GitHub API
（~/use-proxy 代理）fetch 上游 web-platform-tests/wpt@master 实测：

| 目录 | N_imported（本地） | N_full（上游） | 覆盖率 |
|------|-------------------|---------------|--------|
| css-flexbox | 60 | 586 | 10% |
| css-multicol | 60 | 537 | 11% |
| css-writing-modes | 60 | 855 | 7% |
| css-fonts | 57 | 391 | 15% |
| css-grid | 18 | 63 | 29% |
| css-position | 24 | 149 | 16% |
| css-tables | 54 | 203 | 27% |
| css-text-decor | 39 | 356 | 11% |
| CSS2 | 131 | ~5000-7000（43 子目录旧套件） | ~2-3% |
| **总计** | **503** | **~8000-10000** | **~5-6%** |

**关键**：self-source 443/490、chromium-Oracle 42.1% 均基于此 ~5-6% 子集分母，非全量真通过率，
不构成 DC-14 达标证据（goal line 329/843）。R388-R393 所有相对进展（+12 一致等）是子集口径。
完成全量导入是多会话基础设施（fetch ~7500-9500 新案 + reftest 10-20× 时长 + oracle 全量重抓
+ ZW 全量下真通过率未知）。

详见 [`../evidence/r394-dc14-denominator-gap-2026-06-22.txt`](../evidence/r394-dc14-denominator-gap-2026-06-22.txt)。
零代码变更（read-only API fetch + 对账）。

---

## R393 — line-box 半行距垂直定位（strut_ascent）非 clean lever

承接 R392 CONTINUE 查 `apply_vertical_alignment` 的 `strut_ascent = line_height*0.8` 是否可修。
实测 text-emphasis-style-property-012（line-height:5）：ZW CJK 文本 y=64、chromium y=69 →
**ZW 文本比 chromium 高 ~5px**。把 `line_height*0.8`（baseline=64）改教科书半行距
`(L+font)/2`（=48，文本上移）会**反向发散**（ZW 已偏高）。故 `0.8` 启发式本就比教科书更近
chromium，~5px 残余是 font-metric 噪声**非可修 bug**。R392 text-emphasis 真因=paint 侧标记
line-box 顶相对定位（非文本 baseline）。**line-box 垂直定位加入 plateau 第 5 项确认**
（R384/R389/R390/R392/R393），单会话 clean-win 5 轮穷尽。
详见 [`../evidence/r393-linebox-halfleading-not-lever-2026-06-22.txt`](../evidence/r393-linebox-halfleading-not-lever-2026-06-22.txt)。
零代码变更（read-only 调查）。

---

## R392 — text-emphasis 实现 net-negative（line-box 定位阻塞），已 100% 回退

R391 后扫 near-pass 集群发现 32 个 text-emphasis 测试完全未实现（疑缺失特性 clean win）。
实现全栈（style-system parse/store/inherit + paint render_fragment! 每 glyph 渲染标记）。
过程中修一个继承 clobber bug（简写入 is_inherited 致继承循环用父 None 覆盖长手 Mark；
简写应不入 is_inherited）。实测 **net-negative**：32 案 z_vs_chr 1.44%→1.52%、
chr<1% **15→9（-6）**。根因=32 案**全用 `line-height:5`**，暴露 ZW line-box 垂直定位与
chromium 分歧（标记须 line-box 顶部相对定位，ZW line-height 半行距分布与 chromium 不同
= IFC 垂直 positioning / Phase A 谱系）。标记渲染了但位置错（紧贴文本上 vs chromium line-box 顶）。
**已 git checkout 9 文件全回退**，零残留（grep=0），build 干净，self-source 443/490 零回归。
text-emphasis **非 clean win，阻塞于 line-box 定位**，勿再以单会话重试。
详见 [`../evidence/r392-text-emphasis-linebox-blocked-2026-06-22.txt`](../evidence/r392-text-emphasis-linebox-blocked-2026-06-22.txt)。
本轮零净代码变更。

---

## R391 — R388+R389+R390 后诚实 chromium-Oracle 基线锁定

全量重跑 cross-validate（475 案，oracle=R388 全量+R389 6 flexbox；R390 swatch 在 ref 不影响 oracle）。
**锁定诚实基线**：广义 chr<1% 一致 **200/475 (42.1%)**、严格 self-pass&chr<1% **177/475 (37.3%)**、
污染 46.5%、z_vs_chr 分布 <1%=200 / 1-5%=220 / 5-10%=22 / >10%=33。
**关键纠正**：R388 报的 205/43.2% 含 R389 才暴露的 **5 个假一致**（css-flexbox
flex-abspos-inset-cross-size-001 / nested-001/002 / aspect-ratio-cross-size-001 /
cross-size-border-box-001，图片 404 时 ZW-test 与 oracle 均全白 → 假一致 ~0.7-0.87%；
R389 图片加载后真实 abspos-flex-width bug 显现 → 9-19% 真发散，移出 chr<1%）——非 ZW 回归，
是度量变诚实。故真实诚实广义一致 = 42.1%（非 43.2%）。所有资源解析 gap（Ahem/图片/swatch）已修，
**无假一致/假失败残留，此为后续多会话架构工作的可信对比基线**。
self-source 443/490 (90.4%) / strict 295/490 (60.2%) 零漂移。
详见 [`../evidence/r391-definitive-baseline-2026-06-22.txt`](../evidence/r391-definitive-baseline-2026-06-22.txt)
+ [`../evidence/cross-validate-full-2026-06-22.txt`](../evidence/cross-validate-full-2026-06-22.txt)。
零代码变更（read-only 重测）。

---

## R390 — 系统资源解析审计完成 + 补齐 css-multicol swatch（第三处资源 gap）

R388/R389 暴露「资源解析 gap 制造假状态」模式，本轮脚本扫 1023 test/ref 全部资源引用
（link/src/url()），1099 引用中 42 缺失。**裁决**：除 R388（Ahem，真 +12 一致）/ R389（../support/，
honesty）外，**其余缺失资源均 honesty-only 或 out-of-scope**——writing-modes test-*.png 方向图
（8 float-contiguous，test 已 PASS，须上游下载）；/common/*.js（async reftest，属 JS 执行类非渲染）；
/fonts/*.ttf/.woff（css-fonts 特性，rustybuzz 未接生产 R330/R332 已证净负）；缺失 ref 文件
（loader 已 warn 跳过不计分母）。**本轮修复**：css-multicol/support/ 补 swatch-blue/orange/yellow.png
（从 css/CSS2/*/support/ 复制，md5 一致=标准 WPT swatch，8 multicol ref 用）。实测
**self-source 443/490 零变化零回归**（验证：移除 swatch multicol-clip-001 仍 PASS 0.56%；
有 swatch 0.22%——仅降发散量不翻 pass/fail = honesty-only）。**结论**：R388 是唯一带来真增益的
资源修复；单会话 clean-win 经 R384（oracle-无关）+ 资源审计双确认穷尽。
详见 [`../evidence/r390-resource-audit-2026-06-21.txt`](../evidence/r390-resource-audit-2026-06-21.txt)。
零 Rust 变更。

---

## R389 — 正确 oracle 下重扫 + ../support/ 图片路径（第二处资源解析 gap，9 css-flexbox 案）

承接 R388 CONTINUE，在可信 oracle 下重扫 near-pass/gross 案找 clean win。
**结论：plateau 在正确 oracle 下仍成立**——R384 self-source 穷尽是 oracle-无关（测 z_vs_ref），
故仍有效；R363/R354 结构性归因经正确 oracle 复核不变；large-font 簇（R388）是唯一被 oracle
证伪的归因。候选逐案证伪：font-family-name-025=字体回退噪声（test 依赖未装 CSSTest/Verdana，
ZW vs chromium 选不同 fallback）；flex-abspos-inset-nested 原疑 false-neg，像素实证推翻
=**两者均退化全白（图片 404）**。发现**第二处资源 gap**：9 css-flexbox test/ref 的
`<img src="../support/1x1-green.png">` 在当前 repo 布局下解析为不存在的 `css/support/`
（实际在 `css/css-flexbox/support/`，WPT 约定为同目录 `support/`；这些是旧快照测试）。修复
=`../support/`→`support/`（9 文件，资源引用修复非渲染变更）+ 重抓 6 oracle。
**self-source 全量 443/490 零变化零回归**（3 false-self-pass→真 self-pass，3 退化 self-fail→真 self-fail，
抵消）。6 案真实根因=abspos flex 容器宽度解析（top/bottom inset definite height，无 left/right；
ZW shrink-to-fit 1px vs chromium 更宽）——**taffy-blocked 结构性（R363/R97/R304 DEFER），非 clean win**。
详见 [`../evidence/r389-rescan-oracle-imgpath-2026-06-21.txt`](../evidence/r389-rescan-oracle-imgpath-2026-06-21.txt)。
零 Rust 变更。

---

## R388 — chromium Oracle 大面积损坏（Ahem 未加载）→ large-font 簇「发散」是 oracle 镜像

承接 R387「lever=stored-path Y 定位」做像素级诊断，发现**反转**：ifc-008 的 ZW 渲染**本就正确**
（实心绿 200×200，0% 红），报告的 7.93% chromium 发散全部来自一张**损坏的 oracle 截图**
（85% 红底=fallback 细 X 字形）。**根因**：oracle 截图 06-18 21:37 抓取，系统
`$HOME/.local/share/fonts/Ahem.ttf` 06-20 03:20 才安装；`chromium-oracle-shot.mjs` 用 `file://`
无法解析上游 reftest 的绝对路径 `/fonts/ahem.css` → Ahem 永不加载 → chromium 退回 fallback 字体
→ 几何崩溃。**108 个 Ahem 依赖 reftest 的 oracle 全损**（css-multicol 27 / css-writing-modes 24 /
css-flexbox 12 / CSS2/fonts 11…）。**实证**：重抓 oracle（http server root=wpt-data，自包含不依赖
系统 Ahem）后 ifc-008 Z_vs_chr 7.93%→0.50%、font-051 8.32%→0.84%、downloadable-font-scoped
20.22%→1.52%。**推翻 R385「fontdue 度量死路」+ R387「large-font lever=layout Y」对该簇的归因**
——两者在追 oracle 镜像。**修复**：`chromium-oracle-shot.mjs` file://→内嵌 HTTP server
（root=DATA_ROOT，+~60 行含 MIME/路径逃逸防护），全量重抓 503 oracle。**cross-validate 复测**
（apples-to-apples，唯一差异=oracle）：广义 chr<1% 一致 **193→205/475（40.6%→43.2%，+12/+2.5pp）**、
严格 self-pass&chr<1% 170→177（+7）、污染 48.0%→46.0%、gross ≥5% 53→50；**26 案改善**
（large-font/字体簇）vs **14 案「退步」=正确 oracle 揭示先前被 fallback 偶然掩盖的真实发散**
（css-fonts 多，非 ZW 回归）。net 计数温和（+12）但**定性价值大**：① large-font 簇证正确、不再是
lever；② DC-14 度量自此可信（无 108 损坏 oracle 系统偏差）；③ R385/R387 false lead 关闭。
详见 [`../evidence/r388-oracle-ahem-invalidation-2026-06-21.txt`](../evidence/r388-oracle-ahem-invalidation-2026-06-21.txt)
+ [`../evidence/cross-validate-full-2026-06-21.txt`](../evidence/cross-validate-full-2026-06-21.txt)。
本轮代码变更：仅 `chromium-oracle-shot.mjs`（JS 工具，非 Rust）；零生产/reftest 行为变更（ZW 本就正确）。

---

## R387 — fontdue Ahem 光栅化完美（16/48/100px）→ 推翻 R385「字体层死路」归因

（⚠️ R388 进一步证该归因对 large-font 簇亦错=oracle 镜像）R174 AA 基准仅测 DejaVu 48px（W/i），
未覆盖 100px 大字号 Ahem。本轮扩展 `glyph_aa_dump.rs` 加 Ahem 多字号实证：fontdue 在 16/48/100px
渲染 Ahem **完美**（width=height=advance=font_size，w/fs=1.000 adv/fs=1.000，精确 1em 方形）——
此 fontdue-perfect 结论 R388 仍成立（且 ZW 实心绿进一步印证）。原推断「真 lever =
compute_final_inline_layouts 的行盒/片段 Y 定位」**被 R388 推翻**（large-font 簇 ZW 正确，
发散=oracle 损坏）。本轮代码变更：glyph_aa_dump.rs 加 Ahem 诊断（~25 行，工具保留）；
零生产/reftest 影响。详见 [`../evidence/r387-fontdue-ahem-perfect-2026-06-21.txt`](../evidence/r387-fontdue-ahem-perfect-2026-06-21.txt)。

---

## R386 — Phase A 设计 §7 phased plan 闭环核查——无可行剩余单会话 phase

逐个核查 phase-a-IFC-unification-design.md §7.1 的 6 phase：Phase 0（glyph 基线探针）= R305 已做；
Phase 1（paint Path A 用 frag.y+height 基线）= **R306 证伪**（font-051 A/B：frag.height offset
→16.67% FAIL vs 默认 offset=0→0.00% PASS；geometric baseline ≠ render baseline）；Phase 2
（multicol 墙探针）= 已做；Phase 3（删 Gate 2 多行限制）= **R355 已做**；Phase 4（删 Path B 空 styles
重跑死代码）= **risky 非死代码**——Path B 当前消费者含 Wall 3 混合内容（storage gate
`inline_finalization.rs:308` 排除存储）+ multicol + flex/grid/table + 非 block-level，删除会回归；
Phase 5（engine.rs 拆 inline_finalization.rs）= **已做**（文件存在）。**Phase A 设计无安全剩余单会话
phase**。结合 R384（47/47 系统证伪）+ R385（fontdue 度量死路），plateau 现从所有维度
（clean-win / Phase A phased plan / fontdue 度量层）确认。本轮零代码变更（read-only §7 核查）。

---

## R385 — 关闭 R355-regression lead（fontdue 度量噪声）+ header 治理归档

R381 发现 R355 使 ifc-008 chromium-Oracle 从 1.82% 退步到 7.89%，疑 stored-path line.y offset
miscalibrated。本轮像素级实证 ifc-008 ZW-test vs oracle：发散 **7.80% 散布于 rows 18-250/cols 8-240**
（非均匀偏移），bbox 差异=ZW 内容更宽（fontdue 100px Ahem advance 偏大）+ 起始更低。**结论：
stored-path 发散=散布 fontdue 度量噪声，非可修 offset bug**（line.y 逻辑本身正确；pre-R355 overrides
path 偶然更近 chromium 是度量巧合）。与 R225/R375 advance-width 死路 + R174 fontdue 度量噪声一致
——**ifc-008/009 的 chromium 改善须等字体层突破，非 Phase A narrow slice 能解**。
详见 [`../evidence/r385-ifc008-stored-path-metric-noise-2026-06-20.txt`](../evidence/r385-ifc008-stored-path-metric-noise-2026-06-20.txt)。
另：**header 治理**——header 原 ~17859 字符含 R351-R384 全部逐轮详记（违反 goal doc「master.md
不允许无限增长」），本轮把 R351-R380 详记迁出 [`rounds-r351-r380-header.md`](./rounds-r351-r380-header.md)，
header 瘦身至 ~4500 字符（留 R381-R385 + 综合裁决指针）。本轮零代码变更（read-only 像素分析 + lead
关闭 + 文档治理）。

---

## R384 — 47/47 决定性 plateau 核查——单会话 clean-win 100% 系统性证伪

把当前 47 个 self-source 失败逐个对照 ruled-out lever，**0 个未覆盖**：每案映射到一个已证伪 lever
（baseline-export R316 / multicol paint-side R157-R317 / balance R199-R322 / column-aware IFC Phase 1
R381-A1 / advance-width R225-R375b / font-weight R229b / WM clearance R114b×4 / clear+margin R333 /
spec-conflict R365 / fit-content-on-flex R364 / 各 taffy-blocked R97/R304 / Phase A R354 / multicol mixed
R383-R109 / 大字体 R378 / 退化 R369b）或多会话硬核（Phase 3 嵌套 breaking、Phase A broad、Phase 2
multicol）。**这把「单会话穷尽」从逐案断言升级为系统性证明**——未来会话勿再逐案扫描找 clean win
（R307-R384 已穷尽所有角度）。真实 lever 仅剩多会话架构，且 R383 证其依赖链（Phase A → multicol
混合内容）；Phase 3 嵌套 breaking（R109-independent，真 block 子）是唯一可独立推进的多会话硬核
但无安全单会话首步。本轮零代码变更（read-only 系统核查）。

> ⚠️ R395 补注：本「单会话穷尽」结论是**子集范围（490 案）**——R394 实测当前导入仅上游
> ~5-6%（503/~8000-10000）。全量集合含未检新失败模式，clean-win 面在更大分母下可能重开；
> 故本结论不可外推为「全量 WPT 集合单会话杠杆穷尽」。

---

## R383 — LAYOUT_DUMP 深度诊断纠正 Phase 2 混合内容前提——根因 = R109 entanglement，前置依赖 Phase A

承接 R382 Phase 2 spec（统一 column-flow），对 multicol-block-no-clip-002 做 LAYOUT_DUMP 深查：
5 个 `<span>`（display:inline）经 **R109（inline→block converter）被转成 block-level LayoutBox**，
与 h4 共 6 block 子被 multicol 按原子 block 分配到 3 列（blue+h4→col1、orange+pink→col2、
yellow→col3，各列顶 y=28/48）；ref 期望 inline 作**单一 IFC 跨列流动**（blue 4 行+h4+orange 1 行
填 col1 至 balance 高 → orange 余+pink 溢 col2 → pink 余+yellow 填 col3，span 跨列分裂）。
**根因 ≠「inline 未分配」（R382 误判），= R109 entanglement**：spans 已是 block 盒，统一
column-flow 即使实现仍按 block 分配，**修不了混合内容案**。真修复须 **先 Phase A（inline 内容作
流动 IFC / R109 解转换）再 multicol 列碎片化——两多会话 lever 依赖（Phase A → multicol）**。
已纠正 multicol-phase2-unified-column-flow-spec.md（§0/§6.5 A1/§11 加 R383 警示）。
**R109-independent 的 multicol 失败（嵌套 breaking Phase 3，真 block 子）可独立推进**。
本轮零代码变更（read-only LAYOUT_DUMP 诊断 + spec 纠正）。

---

## R382 — Phase 2 multicol 统一 column-flow spec-rfc 产出 + probe

承接 R381 Phase 1 gate 关闭（A1=FALSE，pure-inline 无目标）后的 Phase 2 路由，用 spec-rfc 标准
模式产出 [`../multicol-phase2-unified-column-flow-spec.md`](../multicol-phase2-unified-column-flow-spec.md)
（415 行，lint 23 Pass/1 Warning/0 Fail，门禁通过）：layout 侧 `flow_children_into_columns` 统一列流，
按文档序把 block+inline 子逐列流动（取代三段分离模型），存 `LayoutBox.inline_multicol_columns`，
paint 消费，env `MULTICOL_UNIFIED_FLOW` 门控。**Probe 结果**：A3 已解决（IFC **无需扩展**——列切片
逻辑在统一流 `flush_inline_run`，IFC 只产宽度换行行盒；`inline/mod.rs:1462 break_items_into_columns`
是垂直书写模式命名碰撞非 multicol）；A1 部分 probe（multicol-block-no-clip-002 像素分析：内容挤
左上角未跨 3 列，范围成立；block-vs-inline 精确归因待深 probe）。范围：单层 balance height:auto
混合 block+inline 子（目标 ~6-8 案）；排除嵌套 breaking（Phase 3）+ inline 跨列断裂（Phase 2c）。
**下一步**：实施 Commit 1（block 等价 byte-identical 安全网）→ Commit 2（inline-run + paint 消费，
chromium-Oracle 门禁）。

---

## R381 — DC-14 fresh cross-validate 揭示 R355/R362 在诚实指标上退步

master.md 引用的 chromium-Oracle ~35.6% 来自 2026-06-18（pre-R355）陈旧数据。本轮用当前代码 +
固定 06-18 oracle 截图重测全量 475 案：chromium-Oracle 真一致 **169→170（35.6%→35.8%）**基本持平，
self-source 头条 442→443 **夸大**真实进展。逐 case 对比 5 个 fixed 案 z_vs_chr：
**R368(ifc-011) -10.08pp / R373(clear-inline-001) -2.35pp = 真 DC-14 改善**；
multicol-count-computed-004(R358) flat +0.15pp；但 **R355(ifc-008) +6.07pp / R362(ifc-009) +5.07pp
= REGRESSED**——两案从 false-NEGATIVE（pre：ZW-test≈chromium 1.82/2.35% 但 ZW-ref 错→self-fail）
转 false-POSITIVE（self-pass 但 chr 7.89/7.42%），R355 报告「ifc-008 -4.01% chr 改善」**与实测矛盾**
（provenance 存疑）。净 DC-14 = -1.14pp 近中性。**关键教训：self-source 头条非 DC-14 进度可靠代理
——R355/R362「+1 clean win」在诚实指标上是退步；后续 Phase A slice 须一律用 chromium-Oracle z_vs_chr
验证（不只看 self-source 翻转）**。fresh cross-validate 已存
[`../evidence/cross-validate-full-2026-06-20.txt`](../evidence/cross-validate-full-2026-06-20.txt)，
详见 [`../evidence/r381-dc14-fresh-crossvalidate-2026-06-20.txt`](../evidence/r381-dc14-fresh-crossvalidate-2026-06-20.txt)。
plateau 在诚实指标上成立（35.8% 真一致，需多会话架构突破）。本轮零代码变更
（read-only cross-validate + 对比分析）。
