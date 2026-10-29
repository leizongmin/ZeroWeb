# R5326 — 上轮 429 中断 WIP 取证收口（无 corpus 变化轮）

- **日期**: 2026-10-29
- **性质**: 调查/取证轮——前一会话（R5326 起步，API 429 限流中断）遗留 3 处未提交
  hunk，逐 hunk 像素级定谳后**全量回退**；根因重新定位。corpus 16590 案 **809F 不变**
  （FIXED=0，REGRESSED=0），无 trend 行。
- **基线**: R5325 交付态 809F（clean HEAD 本地复跑复核一致）

## 上轮遗留 WIP（已回退）

3 处 hunk，语义目标 = block-in-inline-insert-009 家族（✗1.08%）：

1. `crates/layout-engine/src/inline/collect_items.rs:2004` — walk（
   `collect_flat_inline_children`）删 `fragment_node_ids` 成员表过滤（R109 split
   容器成员的元素子不在表内属预期，不按表过滤；R3997 容器作用域同论）。
2. `crates/layout-engine/src/r109.rs:47` — `shrink_r109_anon_blocks` 分裂边改
   `!last → border_right=0` / `!first → border_left=0`（中间片段双边开放、单片段
   双保留；CSS2 §9.2.1.1）。
3. `crates/layout-engine/src/tree.rs:3887/4034` — 匿名块片段 ctx_node 选取要求
   `has_preserved_content`（跳过纯空白首项 + 子树 DFS）+ 单 Inline 片段（len==1）
   双标 first+last。

## 定谳（逐 hunk）

- **corpus A/B（同树全量）**：WIP 树 809F fail-list 与基线**逐案一致**；全量
  16590 案 mismatch % 逐值 diff 仅 2 案移位，均经同二进制复跑证伪为本轮无关 flake
  （见下文 flake 清单）。WIP 对 corpus **像素零影响**。
- **hunk 1（成员表过滤删除）**：insert-009-nosplit-ref 页 REFTEST_DUMP PNG
  base vs WIP **字节级一致**（cmp 同）——「nosplit ref 页文本段 IFC 收集 0 项、
  匿名块 784×0 整段丢失」诊断不可复现：该页匿名块片段收集不经
  `collect_flat_inline_children`（成员经 flatten 路径文本保留），walk+过滤路径
  对这些页是死路。
- **hunk 2+3（分裂边/首末标记）**：layout-dump 实证 insert-009 测试页 **R109
  split 根本未触发**（div 仍嵌套在 `#target` span 下，无匿名块片段）——
  `shrink_r109_anon_blocks` 与首末标记在该页零执行；「测试页蓝脊贯穿全块」非
  中间片段双边保留，而是未分裂嵌套渲染的形态。
- **真根因**：`inline_child_is_pure_block_carrier`
  （crates/layout-engine/src/inline_block_split.rs:335）只认**直接** block 子的
  纯块载体——insert-009 形态 `span(蓝边) > span#target(青边) > [inline…, div×4]`
  的载体链深两层，外层 span 的子是 inline（#target）→ 载体判定 false → body 层
  split 不触发。放行该家族需：**多层 inline 载体链升格 + 带边框 wrapper 的
  inline 级片段化**（边框/背景按 §9.2.1.1 分片绘制到各匿名块）——纯块载体升格
  现有实现「载体自身不生成可见盒」，带框 wrapper 不能静默升格，属 ledger 既有
  深结构排除域（R5225 分叉对、inline_block_split.rs FIXME「文本+块混排 wrapper
  另切片」同域）。
- **chromium 定谳（本地 headless 151.0.7922.173）**：insert-011 test ≡ ref ≡
  oracle-shots 逐字节一致。期望形态实证：外层 span 若以 block 子结尾，其**末边
  （border-right）移到 block 后的空片段带**绘制（ref 手工建模为空 span；ZW 对空
  inline 段走 collapse-through 不生成行）——单片段/末片段右缘语义以「inline 是否
  以 inline 内容结尾」为准，非「最后生成的匿名块」。#33 自洽 PASS 教训又一例：
  insert-011 现绿（0.56%）但 test/ref 双侧均偏离 chromium。

## 回退依据

准则 4（每行变化可追溯需求）+ 准则 5（bug 修复测试复现目标行为）：三 hunk 的
驱动诊断均被像素证据证伪，corpus 零效果、无 witness、无 pin——保留即无验证面
的行为变更。语义上 hunk 1（R3997 容器作用域）与 hunk 2 的中间片段双边开放方向
正确，全文存档于本文件附录，深结构轮可按需重启。

## 本轮分诊记录（809F 择轻扫描，均不轻）

- `text-transform-fullwidth-004/005`（0.12/0.17%）：vertical-rl 域，挂账避开。
- `letter-spacing-ligatures-005`（0.55%）：css-text-3 书法连接（cursive joining）
  抑制字距——ZW 全仓无 joining/cluster 概念（grep 实证），先行特征缺失，非字距
  单点修复可绿。
- `outline-028`（0.31%）：outline 绘制特征缺席（test/ref 均无框）。
- `clip-text-background-table-cell`（0.28%）：gradient 机制域。
- **multicol 8 案 ×1.04%**（column-height-007 / column-wrap-no-constraints-002 /
  multicol-fill-balance-003/006 / multicol-span-all-children-height-009 /
  nested-past-fragmentation-line / orthogonal-writing-mode-spanner /
  spanner-in-opacity）：各差恰 5000px，逐案 diff 定位为**嵌套 multicol
  column-fill:auto 分片填充/溢出顺序**机制（fill-balance-003: ZW 在
  ref 空白带多涂一根 25×200 内列条；column-height-007: 对角错位条）——单一域
  内聚簇，值得专项轮，非本轮余量可修。
- **shadow `:host` 簇 ×5000px**（featureless-003/004、lang-pseudo-class-across-
  shadow-boundaries、content-visibility slot ×2）：`:host` 全仓无实现（matcher/
  pipeline grep 零命中）；严格解析下含 `:host` 的规则整条失效 → featureless-003
  四 div 全落基类色（red red green green，chromium 全绿）。shadow 级联隔离特征域。
- **5000px 观察记档**：≥20 异域案差恰 5000px 系各案独立机制下的巧合同值（多为
  100×100 fixture 方格的半幅面积），**无共享机制**，勿循值追猎。

## flake 清单（同二进制复跑实证）

- `block-in-inline-insert-011`：0.56% ↔ 1.46%，骑 1.00% 阈值翻转 ✓/✗——
  fail-list 归因时先复跑再定谳。
- `css-text-decor/text-decoration-inset-orthogonal-block-001`：0.45% ↔ 0.83% ↔
  1.08%（基线二进制四连跑实测跨阈），任意轮可闪红。
- `css-sizing/contain-intrinsic-size/contain-intrinsic-size-014`：0.10% ↔ 0.33%
  （良性，绿区内抖动）。

## 附录：已回退 WIP 全文（深结构轮可重启的 groundwork）

见本会话记录或 git 历史；要点：hunk 1 = 删 collect_items.rs walk 内成员表过滤
（R3997 容器作用域语义自洽）；hunk 2 = r109.rs 分裂边 `!last → border_right=0` /
`!first → border_left=0`；hunk 3 = tree.rs 片段 ctx_node 取
`has_preserved_content` 首个可保留文本（含 DFS）+ 单片段双标 first/last。
重启前提：多层载体链升格 + 带框 wrapper 片段化落地后，三 hunk 才有可复现驱动面。
