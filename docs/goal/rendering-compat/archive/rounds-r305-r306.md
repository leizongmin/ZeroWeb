# R305–R306 — Phase A IFC 统一 spec-rfc 设计 + Phase 0 探针实证（归档自 master.md）

> 归档说明：本文件为 master.md「最近轮次详细记录」中 R305–R306 的逐轮详细记录，于 doc-maintenance 轮（2026-06-19）归档——master.md 最近轮次窗口收窄为最近 20 轮（R307–R326），R305–R306 作为第 21–22 轮迁出。R305–R306 的核心结论（Phase A 几何基线 ≠ fontdue render baseline，§6.3A「frag.y+height」方向证伪；Phase A 真正杠杆从「offset 语义统一」重定为「Gate 2 放宽 + multicol 墙②」）仍以浓缩形式保留在 master.md「综合裁决」杠杆穷尽表（Phase A IFC font_size 解锁 | R125/R198/R205/R206）。本归档仅为可追溯性保留，archive 区不修改。

---

### R305 — Phase A IFC 统一 spec-rfc 设计文档产出（read-only 设计，基线 loose 438/490 / strict 296/490 持平）

**承接**：R304 DEFER taffy 升级后，转向最大结构性 lever Phase A IFC 统一。本轮按 spec-rfc 工作流产出**设计文档**（不落地代码）。

**产出**：`docs/goal/rendering-compat/phase-a-IFC-unification-design.md`（335 行，11 章，Spec Lint 14 Pass / 1 Warning / 0 Fail）。

**read-only 精读结论——三处墙精确定位（代码行号实证）**：
- **三条 IFC 路径**：compute_final（engine.rs:1668 真实 styles IFC）→ Gate 2（engine.rs:1910）→ paint use_stored（text.rs:807）Path A 渲染 vs Path B 空 styles 重跑（text.rs:846）。
- **两个 Gate**：Gate 1（R207 narrow，engine.rs:1720-1749）决定哪些容器进 IFC；Gate 2（R84 安全子集，engine.rs:1910 `lines.len()<=1 && is_pure_ahem`）决定哪些容器**存** inline_layout。**关键事实**：`store_font_sizes_from_ifc`（engine.rs:1152）不受 Gate 2 限制广泛建立 per-node font_size map，Gate 2 只限完整行盒存储。
- **墙①**（large-font 簇根因）= Gate 2 多行限制：ifc-008 inner-div 2 行→不存→Path B 16px。
- **墙②**（multicol 反向依赖 R198/R209/R213）= multicol 永远走 Path B（use_stored=!multicol_info，text.rs:807），放宽 Gate 2 让内层容器存行盒后 multicol-fill-auto 0.63→9.15 回归；机制疑点（font_size map 不受 Gate 2 限→回归非 map 变化，疑 paint 分支/几何变化）降级为假设 A2 待 Phase 3 探针。
- **墙③**（v_offset/baseline 语义分歧）= Path A 用 `is_ahem?0:font_size`（text.rs:1208）vs Path B 用 `baseline_fs`（text.rs:1225），多行非-Ahem 不一致——**架构性**，只要 Path B 存在两套语义就不可收敛（R206 broad 翻 FAIL 直接原因）。

**推荐方案**：**baseline-resolved 单一权威行盒**——InlineLayoutLine/Fragment 加 `baseline_y` 字段，compute_final 对所有过 Gate 1 容器存行盒，paint 永远消费 stored（消灭 Path B，仅 flex/grid/table 保留重跑），删除 Gate 2 启发式改用「font_size 同源」不变量。multicol 经 Phase 3 探针定 M1（消费 stored 做列分配）/M2（保守 fallback）。

**5-Phase 实施计划**（每 Phase 独立可合并、零 count 回归硬门禁）：P1 加死字段 baseline_y 建测量基线（净 0 验证）；P2 paint Path A 改用 baseline_y（R207 子集仍 PASS 验证 A3）；P3 read-only 探针 multicol 墙②；P4 删 Gate 2 多行限制 + multicol 方案；P5 删 Path B 死代码 + engine.rs 拆分（3969→抽 inline_finalization.rs ~400 行）。

**对优先级队列影响**：Phase A 有了可执行的架构蓝图 + 分阶段回归门禁（区别于 R125/R198/R205/R209/R213 五轮单点死锁——它们都在试图**单点**修 font_size 而 Path B 仍在）。设计文档落盘供后续多轮接力。next = R306 执行 Phase 1（加 baseline_y 死字段 + compute_final 计算，净 0 验证，零渲染变化）。read-only 设计，无代码/reftest 变更，基线持平。

### R306 — Phase A Phase 0 探针实证：geometric baseline ≠ fontdue render baseline，§6.3A「frag.y+height」方向证伪（read-only 探针，基线 loose 438/490 / strict 296/490 持平）

**承接**：R305 spec-rfc 设计文档把 Phase 0 定为「实测 glyph 基线耦合探针」，因 §6.3A 发现 `GlyphPrimitive.y`=基线、`frag.y/offset/glyph.y` 经验性耦合，原 Phase 1「加 baseline_y 字段 = frag.y+height」前提不稳。本轮执行 Phase 0 实证探针。

**探针设计（env-gated，零默认回归）**：text.rs:1208 stored Path A 的 `v_offset` 原为 `is_ahem ? 0 : font_size`。加 env `PHASEA_BL=1` 临时改用文档化基线不变量 `v_offset = frag.height`（即假设 baseline = frag.y + height，types/mod.rs:387 注释 + apply_vertical_alignment `run.y = baseline_y - run.height` 推导）。对 stored 单行纯 Ahem 用例 font-051（`div{font:100px/1 Ahem}` → "FAIL" 4 字 ×100px = 400×100 黑矩形）A/B 实测。

**实证结果（决定性）**：
| 模式 | font-051 diff | 裁决 |
|------|---------------|------|
| BASELINE（v_offset=is_ahem?0:font_size，默认） | **0.00% PASS** ✓ | 当前 offset load-bearing 正确 |
| PROBE（v_offset=frag.height，PHASEA_BL=1） | **16.67% FAIL**（80000/480000 px，max ch 255）✗ | 文档化「frag.y+height」**渲染错误** |

font-051 单行 line-height:1 Ahem：IFC 算 `frag.height=line_height=100`、`max_ascent=max(80,100)=100`、`frag.y=100-100=0`。PROBE 把 offset 从 0 改成 height=100 → glyph_y 下移 100px → 黑矩形整体错位 16.67%。**当前 offset=0 才正确**。

**关键推论 1 — stored Path A 的 `else { frag.font_size }` 分支是死代码**：Gate 2（engine.rs:1910 `lines.len()<=1 && is_pure_ahem`）保证**只有纯 Ahem 单行容器存储** inline_layout（R207 narrow 扩展的是 Gate 1，Gate 2 的 is_pure_ahem 守卫未动）。故 stored 片段 `frag.is_ahem` **恒为 true**，`v_offset` 恒为 0，`else` 分支永不执行。stored Path A 的 offset 实际是常数 0。

**关键推论 2 — geometric baseline ≠ fontdue render baseline**：types/mod.rs:387「基线 = frag.y + height」是 IFC 的**几何基线**（apply_vertical_alignment `run.y = baseline_y - run.height` 推导成立）。但 fontdue 光栅化 Ahem glyph 时，`GlyphPrimitive.y`（被 cpu/mod.rs:33 `glyph_top_left` 当 baseline）+ fontdue 自身 glyph 度量（y_offset/bitmap_height）的组合，使 **offset=0（非几何 height）** 产出与 chromium 一致的位图。即 fontdue Ahem 的「render baseline」与 IFC「geometric baseline」差一个 fontdue-metric-dependent 常量。`baseline_y` 字段若存几何基线（frag.y+height），paint 直接用会**重演 16.67% 错误**。

**对设计文档的纠正（§6.3A / §0 / §7.1 Phase 1 作废）**：
- 原 Phase 1「paint Path A 改用 `frag.y+height` 基线 / 加 baseline_y 死字段=几何基线」**实证证伪**——会破坏 font-051 等 stored Ahem 用例（R207 子集）。
- 真正可行的统一方向（二选一，替代原 baseline_y 字段）：
  - **(A) 存 render glyph_y**：InlineLayoutFragment 加字段存「compute_final 用同款 offset 校准（is_ahem?0:font_size）算出的最终 glyph_y（= 传给 fontdue 的 baseline）」，paint 直接消费，绕过 offset 语义分歧。但 stored 路径 is_ahem 恒 true → glyph_y = content_y + frag.y + 0，对 multicol/非 Ahem 无新信息。
  - **(B) 保留 paint 端 offset 校准**：stored frag 已携带 is_ahem + font_size，paint 端 `is_ahem?0:font_size` 校准不动；统一靠「让更多容器进 stored」（Gate 2 放宽）而非「改 offset 语义」。
- 两方向都把 Phase A 的真正杠杆从「offset 语义统一」**重定**为「Gate 2 放宽覆盖多行/非纯-Ahem」——而这恰是 R209（PHASEA_MULTILINE）已试、被墙②（multicol-fill-auto 0.63→9.15 回归）阻塞的方向。offset 语义**不是**阻塞点（Path A offset 对 stored Ahem 已正确）。

**与历史轮次的一致性**：R209 已用 Gate 2 多行放宽 + offset=0（未改 offset）测 ifc-008：8.18→4.17%（改善但未过，残余=换行精度）+ multicol-fill-auto 回归。本轮探针**补齐了 offset 语义这一维度**——确认 offset 不需改、不能改成 frag.height，R209 的 ifc-008 残余 4.17% 与 offset 无关（是换行/列宽精度）。Phase A 真正硬阻塞 = 墙②（multicol 反向依赖）+ 换行精度，**非** §6.3A 假设的 offset/baseline 语义。

**意义**：Phase 0 探针以最小代价（env-gated A/B，已回退）**证伪了 R305 设计文档的核心假设**（geometric baseline 可作 render baseline），避免在错误前提下实现 Phase 1（加 baseline_y 字段 = 几何基线）而破坏 R207 子集。这是 spec-rfc「先思考再编码，不假设」原则的实证体现——§6.3A 已自我标记前提不稳（「无法仅靠读码推导」），本轮探针把「不稳」转为「证伪」。设计文档须据本结论修订（§6.3A 加实证裁决、Phase 1 重定向为 Gate 2 放宽 + multicol 墙②，非 offset 字段）。

**本轮为 read-only 探针**：env-gated 代码改动（text.rs:1208 PHASEA_BL 分支）**已 100% 回退**（git diff 仅余并行 agent 的 README.md WIP，非本轮）；revert 后重编译 + font-051 复测 **0.00% PASS** 确认恢复。未改默认行为，未跑全量 make reftest（探针仅改 stored Ahem offset，font-051 A/B 已充分裁决；默认路径零变化）。基线 loose 438/490 / strict 296/490 / chromium-Oracle ~35.6% 持平。next = 据本结论修订设计文档（§6.3A/Phase 1 重定向），或 pivot 到 multicol 墙②的 layout 侧 column-aware IFC（Phase A 真正硬阻塞，R131 谱系）。
