# M1 布局切片：layout-engine 冷/温全量双路径几何快照 + 撤用增量臂边界固化（INC-03 布局面）

**运行身份**：基线 HEAD `defbb2625`（上轮 M2 切片 2），本轮改动 `crates/layout-engine/src/engine/tests/layout_incremental_oracle.rs`（新增）+ `engine/tests/mod.rs`（注册）。**纯测试切片，零生产代码变更**。
**环境/守卫**：`./target/test-guard --compile-first … cargo test -p zero-layout-engine`（oracle 5/5，crate 全量 2076/2076）；fmt、`make guarded-clippy`、`make test`（见提交说明）。

## 生产布局失效路由（源码核查，本切片的基线事实）

| 路由 | 位置 | 说明 |
|---|---|---|
| paint-only mutation | `pipeline/mod.rs:2072` `paint_only_incremental` | 增量 style + **布局整体复用** `cached_layout`（不进 layout-engine） |
| SetText/结构/布局影响属性 | `pipeline/mod.rs:2086` 起 | **快照全量重建**——`compute_incremental` 臂已在 slice14（2026-10-01）撤用：该臂复用 taffy 缓存、不跑全量后处理族（inline 锚 reported rect、margin 折叠逃逸 + abs 修正等），突变后布局 ≠ 同内容静态加载（repro：s14-a-unrelated，abs 兄弟 +10、锚 gBCR 漂移） |
| 增量布局入口 | `layout-engine/src/engine.rs:1518` `compute_incremental` | **生产不可达**；已知边界（`incremental_parity_experiment.rs` 头注，2026-08-09）：taffy 树样式是建树快照，mark_dirty 只重算布局不更新 taffy style → 样式变更保留旧几何 |
| 缓存纯度 | engine.rs `cached_state` 读点核查 | 仅 `compute_incremental`（:1527/:1556）读；`compute()` 只写（:1482）——全量计算不消费缓存 |

## 双路径定义与比较契约

- **冷路径**：每步 `invalidate_cache()` 后 `compute()`——≡ 生产快照全量重建语义。
- **温路径**：`cached_state` 保留跨步 `compute()`——护栏性质：钉住 compute() 输入纯度（缓存只写不读）；未来任何"全量里偷用缓存"的优化都会被检出。
- 样式输入走双路径（全量 vs `compute_styles_incremental`），几何比对前先断言样式相等——样式分歧单独报出，归因不混淆。
- **几何容差契约（M1 按输出契约固定，兑现 goal"不留到事后放宽"）**：逐盒 **f32 精确相等，无 epsilon**——两路径同进程对同一 (doc, styles) 运行同一布局代码，位级相同是应有语义。比较键 = pre-order 盒序列（`node_id` 非空的盒），字段 = x/y/边框盒宽高/内容盒宽高；匿名盒无 node_id 不直接入键，其错位体现在兄弟盒几何（间接检出）；序列长度/顺序差异也是分歧。
- `LayoutResult::paint_skip_node_ids` 不在本切片比对：增量臂恒空是撤用臂已知语义差（不重跑 compute_final 孤儿回填）——归 M3 生产帧切片。

## 序列与结果（5/5）

fixture 与 style 层 oracle 同构（html>body>(div#card-a.card>(p1+p2+span)+section)），含真实文本使行盒几何非平凡；CSS 含定宽/padding/行高/margin（几何非平凡）。

| 序列 | 步骤 | 期望 | 结果 |
|---|---|---|---|
| layout_style_width_change_cold_warm_consistent | `.card` 400→600→400（布局影响属性，changed=card-a） | Equal | PASS |
| layout_font_invalidation_completeness_consistent | `p.text` 14→20px（changed=p1,p2——changed 集完备性契约，反例 A 同族） | Equal | PASS |
| layout_structural_move_cold_warm_consistent | span 移入 `#sec-b`/移回（结构变更，changed=s1） | Equal | PASS |
| layout_viewport_change_rebuild_consistent | 800→400 视口（`set_viewport` 失效缓存，重建后一致） | Equal | PASS |
| layout_incremental_arm_style_change_diverges_boundary | 撤用增量臂：v1 缓存 + v2 样式 + mark_dirty(card-a) | **Divergent** | 检出 ✓ |

**全量基线自检**：Divergent 序列内先断言两个独立引擎同输入逐盒一致（非确定性排除），分歧才能归因到增量臂而非对照器失灵。

## 撤用增量臂边界（几何级固化）

`layout_incremental_arm_style_change_diverges_boundary`：v1 缓存存在时样式变更（.card 加宽）走真增量臂（`was_full_recalc == false` 断言），几何与全量参照分歧——`incremental_parity_experiment` 头注的 taffy style 快照边界在逐盒几何上的序列化扩展，slice14 撤用依据的回归锚。重新启用前置条件不变：该实验补齐后处理族 parity 用例并全绿（深结构范围，非本 goal 擅改）。

## 边界与未覆盖

- 覆盖面：layout-engine 层几何（LayoutBox 盒序列）。**未覆盖**：paint/帧复用、半透明帧、多次导航缓存释放（DC-4 → M3）；命中测试几何（M3 生产帧一并入账）；`compute_incremental` 修复不在本 goal 范围（深结构边界，已有撤用决策）。
- 布局影响型失效的**缓存粒度**（DC-3"缓存按真实输入失效或保留"）：生产现答案 = 全有或全无（paint-only 复用整帧 / 其余全量重建），无子树级布局缓存需对照；若后续引入子树级布局缓存，本 harness 的温路径即对照基座。
- bench-gate 未跑：零生产代码变更（测试与文档），性能关键路径未触碰。
