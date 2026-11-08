# M3 R3b：manual slot 分配通道 + B6 补充池通道

**日期**: 2026-11-08
**切片**: 45 号 M3 R3b（slot-fallback-content-006 缺口收口）+ B6（testharness
渲染耦合补充池通道）
**提交**: e5f84ad0b（R3b 主体）、9d307d1b0（活过滤断言修正）、cfe55be04（跨流
cfg 门修复）、60928d40c（B6 通道）、30ea4c630（quickjs-ren-lib 腿测试引擎
时序盲区修复）；push 前 pull --rebase 由 88302c3bf/c272e84bb/79d703f48/
3d0a2b2e8/1b5640e65 重放，正文门禁账保留运行时原哈希、以本行做新旧映射
**结论**: R3b 落地——`slotAssignment:'manual'` wire 全参透传 + `assign()`
渲染镜像（手动表 + 读侧逐节点活过滤）+ serialize→re-parse marker 回环 +
Manual 模式渲染消费；B6 通道落地（`testharness-shadow-dom` runner 子命令 +
Makefile 目标，10 案显式清单与 fetch 脚本两侧同源）。门禁账见 §4。

## 1. R3b 变更面（88302c3bf）

R3 归因改判（[R3 evidence §3](2026-11-08-m3-r3-mutation-arm.md)）指向的
缺口体：`attachShadow({slotAssignment:'manual'})` + `HTMLSlotElement.assign()`
的渲染镜像。JS shim 侧语义完备（42 号 `__zwManualAssigned` 列表 + 抢占 +
slotchange），Rust 渲染侧此前零消费。本片四层接线：

- **dom/shadow.rs**：
  - `ShadowRootData.slot_assignment` 全参透传（`attach_shadow_full` 七参，
    named 默认；`copy_subtree_from` 随拷贝还原——命令式 manual root 的
    setHTML 克隆保语义面）。
  - 手动表 `manual_slot_assignments`（NodeId 键控 raw 列表）+
    `set_manual_assigned_nodes`（整体替换语义，spec assign slottables）。
  - 读侧 [`Self::get_assigned_nodes`] 手动臂：manual 表命中 +
    `slot_shadow_context` 门（named root 上 assign 列表 inert）+
    **逐节点 parent==host 活过滤**（spec find slotables——被移出宿主的
    slottable 失配自身，宿主内条目保持；全移出才塌回回退内容）。
  - `assigned_slot` 反查同门（manual 表扫 + 同 inert 门 + 同活过滤）。
  - serialize→re-parse 回环：manual root 的 slot 以
    `data-zw-manual-assigned="#id #id"` marker 回写分配关系（spec 无声明式
    对影——manually assigned nodes 不落序列化产物，ZeroWeb 内部通道）；
    parse 侧 `restore_manual_slot_assignments` 还原并摘除（marker 不留
    存活属性面）。降级：任一被分配节点无 id/引用不可解析 → 不写/放弃
    marker，slot 走回退内容（与真实浏览器 assign 状态非持久化一致）。
  - `resolve_slots` manual root 整树跳过（spec：manual 模式 light 子不参与
    自动分配，重算会清手动表）。
- **dom/parser/declarative_shadow.rs + serializer.rs**：
  `shadowrootslotassignment` 枚举捕获与 spec 序回写（serializable 与
  clonable 之间）。
- **engine**：`DomMutation::AttachShadow{,OnHandle}` wire 携带
  `slot_assignment`；新 `SlotAssign` 变体（句柄优先/selector 回落 shadow
  包含查询）；`__zw_slot_assign` 回调；shim `assign()` 臂批量同步引擎表
  （self+victim——跨 slot 抢占由 shim 清对方列表后各自调引擎）。
- **单测 6**：manual 表渲染镜像 / named inert / 回环还原 / 无 id 降级 /
  引用失配跳表 / 枚举序列化序。

## 2. 活过滤断言修正（c272e84bb）

回环测试原断言「移出宿主的被分配节点塌回回退内容」把整表清空，与读侧
逐节点 parent==host 门和 spec find slotables 逐项过滤不符。修正为两步：
B 移除后列表塌到 `[A]`（宿主内条目保持），A 也移除后才塌回回退内容。
 Scoped 验证：`./target/test-guard -- cargo test -p zero-dom slot` 过滤
25P/0F（含 R3b 全部 6 新单测）。

## 3. B6 补充池通道（M0 evidence §3/§6 M3 项 4）

- 42 号 `wc_case_skipped` 渲染级排除域的 10 案（几何命中/滚动/焦点/
  caret/Highlight/交互）落独立通道：runner 子命令 `testharness-shadow-dom`
  （`SHADOW_RENDER_POOL_CASES` 显式清单——与 fetch 脚本
  `45-shadow-dom-rendering.sh` `SUPPLEMENT_POOL` 段两侧同源，非目录递归，
  shadow-dom/ 其余面仍归 42 号 WC 通道防双计）+ Makefile 目标
  `testharness-shadow-dom`（fetch 走 45 号语料脚本，幂等）。
- `accesskey.tentative` 交互面随 nested-hover 口径挂账（runner 无真实输入
  注入，结果按静态等价口径记账，不记真通过）。
- 基线首跑 `--json` 捕获落本 evidence §5，门值实测回填控制面。

## 4. 门禁账（2026-11-08，净树 head=c272e84bb）

**legs1（zr3-r3b-gates-074138）**：
- `cargo fmt --all -- --check`：净。
- workspace v8 debug 腿：**3814P/6F**——6 案（img_runtime_
  deferred_settle_r5275 / svg_img_settle_density_r5277 /
  test_fetch_filtered_response_type_matrix /
  test_iframe_service_worker_messages_dispatch_to_iframe_container /
  test_t8k_websocket_error_nonterminal /
  test_t8k_websocket_event_flow_open_message_close）与 R3/R4 evidence
  HEAD 环境失败名单逐字一致（fetch/websocket/img 域），零 P→F。
- quickjs clippy 腿（14 crate `-D warnings`，与 workspace 腿并行）：净。

**legs2（zr3-r3b-gates2-074559，补 legs1 make 中断的余腿 + 渲染面）**：

| 腿 | 结果 |
|---|---|
| renderer bin (v8) | RC=0 |
| renderer lib (v8) | RC=0 |
| browser bin (v8, xvfb) | **742P/1F**——1F = `default_actions_work_without_javascript`（R4 evidence §4 同案同败：fetch/credentials 桥域，git stash 探针证 HEAD 既有；742P 与 R4 逐字一致，零 P→F） |
| browser tests (v8, xvfb) | → legs3 |
| render-foundation GPU 条件腿 | → legs3 |
| quickjs 测试腿（5 crate / renderer bin / renderer lib） | → legs3/legs4 |
| 其余门腿（reftest/矩阵/smoke/clippy/bench） | → chain5 |

**legs3（zr3-r3b-gates3-075234）**：

| 腿 | 结果 |
|---|---|
| browser tests (v8, xvfb, 软跑) | rc=101，6F 全部为 cdp_focus_governance.rs:277 **陈旧二进制守卫 panic**（88302c3bf 07:48 落 engine shim，debug renderer bin 为 07:42 旧建，守卫拒跑 0P/6F）——基础设施时序问题非行为失败；净 HEAD 重建 bin 后双腿复核挂 chain5 |
| GPU 条件腿 | gpu-probe available → gpu-renderer rc=0、gpu-solid-fill rc=0 |
| quickjs 5-crate 腿 | rc=2：s336-s339 四钉 E0433（`crate::V8Sandbox` 缺 cfg 门，881a4a4012 跨流引入非 R3b 面）→ 79d703f48 整钉 `#[cfg(feature = "v8")]` 修复（R5334 前例同型机械修） |

**legs4（zr3-r3b-gates4-075521，head=79d703f48 净 HEAD）**：

| 腿 | 结果 |
|---|---|
| quickjs 5-crate 腿（cfg 修复后） | rc=0 |
| quickjs renderer bin | rc=0 |
| quickjs renderer lib | **369P/1F**＝`page_scripts::mutation_arm_does_not_apply_across_document_reset_window`（panic :1836「未观测到滞留窗」，11-03/11-04 即在册早于 R3b 非本轮引入）——**已根因修复（30ea4c630），定性改判**：当日插桩取证（settle@~700ms/队列恒空/250ms 时 worker 确认在忙＝忙臂 eval 未启动）证明固定 250ms 放行+裕量是 v8 时序形态——quickjs 下 worker boot+shim 安装+快照构建超裕量，prio 复位在臂启动前到达即在命令边界合法内联落定（滞留旧臂被复位臂清队丢弃，t8-D1 语义照常）＝**测试侧引擎时序盲区，非产品缺陷、非对端碰头红**（推翻本行前记「规则 9 不硬解」定性）；改轮询 execution_count 确认臂 eval 开跑后复位，单测双腿绿（quickjs 2.26s/v8 1.63s），净套件复核 370P/0F 见 legs5；净 HEAD browser 双腿复核挂 chain5b |
| reftest 全量 / DC-4 矩阵 / product-smoke / clippy-v8 / bench-gate | → chain5（legs4 在此腿 CHAIN-FAIL 中止） |

**chain5b（zr3-r3b-chain5b-0808，净 HEAD 复核链，head=79d703f48）**：

| 腿 | 结果 |
|---|---|
| debug renderer bin 净 HEAD 重建 | rc=0（08:08:09 落盘） |
| browser bin (v8, xvfb, 软跑) | rc=101，**742P/1F** 同在册 `default_actions_work_without_javascript`（faces 与 legs2 逐字一致，零 P→F） |
| browser tests (v8, xvfb, 软跑) | rc=101，6F 仍为 cdp_focus_governance.rs:277 mtime 守卫拒跑——apps/renderer/src 有对端在流 probe 编辑，守卫按 mtime 再拒（行为面 6P/0F→? 未得复核，M1 evidence §同套件在册 6P/0F）；净树复核挂后链 |
| reftest 全量 | **rc=0（725/725 零回归）** |
| DC-4 矩阵 | rc=2，**32P/3F**：F＝display-contents-shadow-dom-1（1.97% 在册）+ vw-units（2.08% 在册）+ **slot-fallback-content-006（4.90%，23543/480000 px differ）**——006 未按预期翻绿，非链错属矩阵实测，另行调查（006 是 R3b 目标案，直接推翻「R3b 已收口」结论，见 §6） |

**chain6（zr3-r3b-chain6-0814，head=79d703f48 树）**：product-smoke rc=0
（08:14:14）；clippy-v8 于 08:14:40 链死亡（unit 消失无 rc 账，bench 未跑，
B6 3d0a2b2e8 亦未在其树）——clippy/bench 缺口由 legs7 补。

**legs5（zr3-r3b-gates5-081923，head=3d0a2b2e8+1b5640e65 树，本流收尾链）**：
fmt rc=0；quickjs renderer lib（带 30ea4c630 修复）**rc=0 370P/0F**（338s）
——legs4 唯一红案修复后全绿；余腿（reftest×2/smoke/clippy/bench/B6 基线）
与 chain5b/chain6/B6 首跑内容重复，ren-lib 落账即停链不重跑（修复仅动
page_scripts 测试面，上述腿结果内容等价传递）。

**legs7（zr3-r3b-gates7-082700，净树 d34a3404d 收尾）**：clippy-v8 rc=0；
bench-gate（dom/engine 定向）rc=0（08:30:20 落账，控制面「PASS」句据此）。

## 5. B6 基线（3d0a2b2e8 通道首跑实测，2026-11-08 08:13）

10 案 101 子测 **15P/86F**（14.9% 子测通过率；rc=1 预期——基线记通过率非
land 门禁）。json 留档 `.acceptance/b6-baseline.json`（gitignored 面）、
text 账 `zr3-b6-baseline-082500.log`：

| 案 | 子测面 |
|---|---|
| offsetParent-across-shadow-boundaries | 12F/10P（部分过） |
| offsetTop-offsetLeft-across-shadow-boundaries | 2P/1F |
| focus-within-shadow | 1P |
| scroll-restore-shadow | 1P |
| scroll-to-the-fragment-in-shadow-tree | 8P/2F |
| Document-caretPositionFromPoint.tentative | 14F |
| DocumentOrShadowRoot-prototype-elementFromPoint | 41F |
| HighlightRegistry-highlightsFromPoint | 4F |
| MouseEvent-prototype-offsetX-offsetY | 3F |
| accesskey.tentative | 2F（helper `pressAccessKey` ReferenceError——helper 解析缺口，按静态等价挂账不记真通过） |

几何命中簇（elementFromPoint/caretPositionFromPoint/Highlight/offsetX-Y）
零过，符合 runner 无输入注入/布局几何查询管道的预期口径；滚动/焦点单测
面已过。B6 门值待 M4 与控制面回填对账。

## 6. 下一步

1. **006 矩阵实测翻查**（R3b 目标案未翻绿，最高优先）：4.90% px differ
   归因——R3b 渲染镜像哪层缺口（marker 回环？Manual 消费臂？shim assign
   时序），修后矩阵 33P/2F 复测。
2. 净树 browser tests 复核：cdp_focus_governance 6F 为 mtime 守卫拒跑
   （对端在流编辑所致），净树+净 bin 重跑确认行为面（M1 在册 6P/0F）。
3. M4 复测收口：DC-4 数值门逐批对账（20 池 + 邻域 9）+ B6 门值回填。
