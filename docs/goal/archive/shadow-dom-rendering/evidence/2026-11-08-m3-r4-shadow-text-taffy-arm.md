# M3 R4 前半：shadow Text 布局通道 taffy 侧（`ZW_SHADOW_TEXT_BOX` kill-switch 落地）

**日期**: 2026-11-08
**轮次**: 45 号 M3 R4 切片（shadow Text 布局通道——taffy 侧机制 + 门控验证 + 回归根因定谳）
**结论**: R4 taffy 侧机制落地但默认关（opt-in `ZW_SHADOW_TEXT_BOX=1` 才生效）：
布局树构建已验证正确，但行级 paint 行走尚未感知 shadow 匿名叶高度，默认开启
会让 slot-fallback-content-004 严格比对回落（✓0.86%→✗1.75%）。paint 侧为
R4 第二臂，下一片完成后翻默认。

## 1. 本片交付

- `crates/layout-engine/src/tree.rs`：`push_shadow_text_leaf`——shadow 树内
  文本节点的匿名 taffy 叶，flex/grid 匿名文本项同款构造
  （`new_leaf_with_context` + node_map/taffy_to_dom 登记，测量回调按 text
  NodeId 取内容）。`collect_shadow_children` / `collect_shadow_slot_children`
  共 6 个文本臂接入（shadow 根直挂文本 ×2、被分配 light 文本 ×2、slot 回退
  文本 ×2），纯空白文本跳过（与 flex 匿名项同口径）。
- `crates/layout-engine/src/tree/runtime_flags.rs`：`shadow_text_box()` 旗标
  ——`ZW_SHADOW_TEXT_BOX` opt-in（显式 `=1` 才开，沿用 inline 侧 `opt_in`
  惯例；与既有 `enabled` 的默认开/`=0` 杀语义相反），6 个调用点全门控。
- 顺带（门禁阻塞性修复，来自上轮已推 303479ea2 的 lint 漏网）：
  `crates/dom/src/document/shadow.rs:228` needless_borrow（clippy 1.99
  `-D warnings` 下报错）。

## 2. 门控矩阵（r4c，2026-11-08）

`make reftest-upstream FILTER=shadow-dom`（35 案，DC-14 self-source 严格容差
= 唯一可信达标指标；退出码 2 为既有 F 面常态）：

| 腿 | 真通过-可信 | 真通过-可疑 | 近似通过 | 不一致 | 004 | 其余 F |
|---|---|---|---|---|---|---|
| flag 关（默认） | 8 (22.9%) | 4 (11.4%) | 20 (57.1%) | 3 (8.6%) | ✓ 0.86% | display-contents-shadow-dom-1 1.95% / vw-units 2.08% / 006 4.83% |
| flag 开（=1） | 8 (22.9%) | 5 (14.3%) | 18 (51.4%) | 4 (11.4%) | ✗ 1.75% | 同上 3 案全稳（006 4.83→4.90 微移） |

- **flag 关 = base 零漂移**：三态分类、失败案名、004 行全等（与 r4b base 腿
  `/tmp/zeroweb2-r4b-base-shadow-dom.log` 及 HEAD 基线一致）——kill-switch
  默认路径零行为变化，满足逐批不回落门。
- **flag 开 = R4 生效面恰为 004 翻转**：仅新增 004 ✗，3 既有挂账 F 不动。
- 前轮 A/B（r4b）同构互证：css-shadow 111 案头条 55/111 双侧不变，但
  真通过-可信 40→37（3 案像素差被文本行位偏移推出严格容差、仍落宽松档）
  ——default-on 无头条收益（零 P/F 翻转）却有严格面损失，坐实默认关决策。

## 2b. 全量门禁（r4d/r4e，2026-11-08）

- `make reftest`（本地 725）：**725/725 全绿**（Layout 521 + Text 204），零回归
  （r4d-local725.log，REFTEST-RC=0）。
- `make product-smoke`：PASS（morning-work/article 320 宽 struct-check
  0 issues + sibling-overlap + count[.article=1]）。
- `make test`（workspace）：零引擎 js_dom_bridge 测试二进制 3814P/6F——
  img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 /
  test_fetch_filtered_response_type_matrix /
  test_iframe_service_worker_messages_dispatch_to_iframe_container /
  test_t8k_websocket_error_nonterminal / test_t8k_websocket_event_flow_open_message_close。
  6 案全落 fetch/websocket/img 加载事件域（s338/s339 credentials 透传与 window
  身份桥邻域），与本片 diff（layout-engine 私有 shadow 文本臂 + dom 一行等义
  lint + rustfmt 规范化）无机制性交集——判 HEAD 既有失败，归 engine bridge
  通道对端碰头账，本片不扩 scope 处理。
  **归因实证（detached worktree 探针 @48e973ae9）**：origin/main tip 本机同跑
  `cargo test -p zero-engine --lib js_dom_bridge::tests::` = 1379P/6F 名单全同；
  该 commit 即 CI run 42404172408 的 headSha，其 build-and-test 全平台绿（仅
  benchmarks 门挂）——6F 为本机环境面既有失败，与本流已推/未推提交及本片
  diff 均无关。本片过滤面同口径 1382P/6F（+3P 来自 303479ea2 新钉），
  P 集合只增不减，零 P→F。
- 运行环境注记：r4d 首跑 workspace 腿 Error 127（target/test-guard 瞬时缺失，
  双 agent 并发构建同一 target/ 所致；前台重建即恢复）。本轮存在同会话双
  agent 并发（rally 双 spawn），门禁采认协议见 rally-double-spawn-race。
- `make bench-gate`（r4i，提交后树上补跑）：microbench 1/1 PASS + absolute
  page-total/retained-form budgets PASS（welcome p95 8.08ms / morning 50.08ms /
  medium 296.82ms / form-input p95 0.0216ms）；WARN 基线硬件不匹配
  （AMD 9700X vs i5-13500H）——相对门禁未启用，绝对门禁全过，默认关路径
  无布局性能面变化。

## 3. 004 回归根因（定谳）

三层几何中前两层正确、第三层（行级 paint）不感知 shadow 匿名叶：

1. **taffy 树层——正确**。探针法证：host 节点获得正确高度（含嵌套匿名叶
   计入），slot 回退文本叶挂在正确父下，margin collapse-through 语义正确。
   （`ZW_TAFFY_TREE_DUMP=1`，004 变异终态页）
2. **LayoutBox 提取层——正确**。`extract_layout`（engine.rs）对
   `is_anonymous_text_item` 已按 taffy 几何产盒（flex/grid 匿名项既有路径）。
3. **inline_finalization 行级 paint 层——不感知**。IFC paint 行走按 DOM 序
   自累计 Y（不读 taffy 几何），shadow 匿名叶不改变其步进——文本行按
   「shadow host 高度为 0」的旧序列绘制。

**band 定位**（004 严格比对）：REF 期望行带 21/55/89/123/157/191/225；
TEST 实绘 SLOT1 首行 71–81（应 55–65，整体下移一行 = 前一 host 行高未计入）、
第二行 105–115（应 89–99）。偏移 = 行带步进差，非字体/宽度差。

**mini 隔离**（`.acceptance/` 本地件 mini/a-shadow-fallback.html vs
b-real-div.html）：同一布局结构的声明式 shadowrootmode 页与真 div 页，
R4-on 下回退文本渲染逐位一致——taffy 侧机制本身像素级正确；差异全部来自
第 3 层 paint 行走。

## 4. 附带发现（pre-existing，非本片引入）

**shadow host 不绘制自身背景**：mini a2（shadow 宿主带 background 灰条）vs
b（真 div 同样式）diff = 14112 px 逐位同位（恰为 host 灰条带区域），且
base 与 WIP 二进制双侧同现——host 元素自背景绘制缺失为既有缺口，与 R4 无
关，挂渲染级缺口清单（`::slotted`/host 自绘面，M3 后续切片或另立）。

## 5. 已具名未解 F 面（本片后不变）

- slot-fallback-content-006（4.83–4.90%）：R3 变异应用臂
  `resolve_slots` 统一挂接（`crates/dom/src/document/shadow.rs` ~269 FIXME：
  attachShadow/childList/slot 属性变异不重跑 resolve_slots）。
- shadow-style-invalidation-vw-units（2.08%）：iframe 载体面（M1 evidence §4
  勘误归因），具名挂账。
- display-contents-shadow-dom-1（1.95%）：display:contents × shadow 组合面。

## 6. 下一步

1. **R4 第二臂（paint 侧）**：inline_finalization 行走感知 shadow 匿名叶
   行位（读 taffy 几何或 flat-tree 序），004 翻回 ✓ 后翻
   `ZW_SHADOW_TEXT_BOX` 默认开，复跑门控矩阵逐批不回落。
2. **R3 变异应用臂**：resolve_slots 统一挂接（命令式 attachShadow/
   childList/slot 属性变异后重跑），006 及命令式 ~26 案翻绿面。
3. B6 补充池通道（testharness 渲染耦合 10 案计门挂此）。
