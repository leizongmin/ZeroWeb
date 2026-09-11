# 事件循环与异步回调 spec 化 — 运行时控制面板（master.md）

**入口文档**: [../event-loop-spec.md](../event-loop-spec.md)
**创建日期**: 2026-09-07（goal 拆分 bootstrap）
**最后更新**: 2026-09-11（M3-S2 落地——renderer tick_observers per-task 重构，kill-switch
`ZW_RENDERER_TICK_PER_TASK` 默认 OFF；product-smoke 20.30% 超阈为干净 main 即有的跨流域漂移，已归因记档）

---

## 当前状态

**专项定位**：父目标 P1a 遗留面收敛（microtask checkpoint 简化版 = DC2 缺口②、host 侧
MO 通知端死路、IO/RO WPT 覆盖为零）。rAF 帧驱动切片已落地不重做；本目标三线 = IO/RO
WPT 基线 → MO host 触发（方案 C 设计已存在）→ checkpoint spec 化（kill-switch）。

**与兄弟 goal 的边界**：
- rendering-compat — crate 层面零重叠（渲染流域活跃面是 css-parser/style-system/
  layout-engine/render-foundation）；engine 属共享面，碰前 `git log` 核对
- webdriver — apps/renderer 共享：该流只碰 Automation 消息处理段，本流只碰 tick 排布段
  （page_scripts.rs/runtime.rs/js_worker.rs）；发现要碰对方段即暂停记入本表
- web-components — part01.js 无共享段（该流主力 part03/part04/part05 + dom_bindings），
  但同属 engine 共享大文件池，碰前互相 `git log` 核对

## 实测基线（2026-09-07 立项时）

### 现有实现

- ✅ rAF 帧驱动切片（kill-switch `ZW_RAF_FRAME_DRIVEN`，默认 OFF 同步 stub；js_worker.rs
  L40/L584/L680 + part01.js L3049-3070）——不重做，reftest 同步 stub 约束有效
- ✅ IO/RO B-gen 生产实现（part01.js L2568-2927）：observe 时 initial notification +
  `__zw_observers_tick` post-render 持续跟踪（threshold 越界/size-diff）；apps/renderer
  tick 接线完成（page_scripts.rs L353 + runtime.rs L708）
- ✅ 几何反馈基建：rect_bridge.rs 493 行同步 `__zw_getBoundingClientRect` + DOMRect
  真原型链（R3319）
- ✅ MO 双轨：polyfill MO（part01.js L742+ Proxy trap）观测 JS 驱动 mutation；dom 层
  `mutation.rs` + `pending_mutations` 记录端可用
- ⚠️ microtask checkpoint 简化版：v8_runtime.rs `perform_microtask_checkpoint`
  （L422/L486）仅 execute 末整批排空，非 spec per-task；无显式 task queue
- ⚠️ MO 通知端死路：engine 全仓零调用 `process_mutations`/`take_mutation_records`——
  host 驱动 mutation 不可观测
- ⚠️ MO 方案 C（hybrid 共享注册表 + host hook + NodeId↔handle 身份桥）设计完成
  （`zero-web/p1b-mutationobserver-host-trigger-design-2026-08-10.md`）未实施
- ⚠️ IO/RO WPT 覆盖为零（wpt-data 无 observer 目录、imported-tests.txt 零命中；现有
  测试全自写）

## 缺口清单

| # | 缺口 | 状态 |
|---|------|------|
| P1 | IO/RO WPT 用例覆盖为零（fetch 脚本 + 导入 + 基线） | ✅ 2026-09-11（基线 29.7% 落 evidence/） |
| P2 | 事件循环时序差距清单（对照 spec 逐条）未建立 | ✅ 2026-09-11（evidence/2026-09-11-m1-event-loop-gap-list.md） |
| P2.5 | IO/RO 语义修齐第一批（tick 接线 + threshold 越界 + 构造器校验/getter + root==target + documentElement client 尺寸） | ✅ 2026-09-11（切片 3a+3b+3c，基线 29.7% → 41.4%，evidence/2026-09-11-m1-slice3-observers-wiring-validation.md） |
| P3 | MO host 触发未实施（通知端死路） | 🔄 M2 MO-S1+MO-S2 ✅ 2026-09-11（identity 桥 + 排空点 + kill-switch 默认 OFF + sibling 双向透传/removed '#id' 回落/oldValue 真值；余 fragment flatten + quickjs 接线；MO-S4 待用户点名） |
| P4 | checkpoint 简化版（无 task queue、无 per-task checkpoint） | 🔄 M3-S1+S2 ✅ 2026-09-11（runner timer 泵 / renderer tick 双 per-task，kill-switch 默认 OFF；余显式 task queue + default-on 决策） |

## 已完成切片

### M3-S1 — runner timer 泵 per-task 边界（2026-09-11）✅

- `__zw_fire_due_timers` per-task 模式（kill-switch `ZW_TESTHARNESS_TIMER_PER_TASK=1`
  默认 OFF）：每次调用只派发首个到期 timer，未派发保留队列——probe 循环每迭代一个
  execute → 一 timer 一 task 一 checkpoint，消除批量派发违反（gap-list §1.2 第一行）
- A/B 双臂逐 subtest 零 delta：dom 全量 54,324P/163F/13T、MO 12 文件、IO、RO 四
  corpus；runner 单测 213P/0F + clippy/fmt 干净；生产路径与 reftest 同步 stub 约束
  零触碰（明细 evidence/2026-09-11-m3-s1-timer-per-task.md）
- 后续：M3-S2 renderer `tick_observers` per-task（生产面，需渲染 A/B）→ M3-S3 显式
  task queue（多队列 oldest-first）

### M3-S2 — renderer `tick_observers` per-task 重构（2026-09-11）✅

- shim `__zw_observers_tick_once(from)`：无状态游标协议——从 from 起 schedule 首个活跃
  observer 并返回下一游标，-1 = 耗尽（首版「返回是否有余量」协议因扫描不推进致死循环，
  已换游标制）；与 `__zw_observers_tick`（整批）并存
- renderer `tick_observers_with(ctx, per_task)`：per-task 下循环 execute→apply 逐
  observer（上限 64 轮），rAF tick 在队列排空后单独一 execute；默认 OFF 维持现合并
  tick。kill-switch env `ZW_RENDERER_TICK_PER_TASK=1`（OnceLock 进程缓存），
  `tick_observers` 包装读它，模式注入形态供测试双模式直设
- 验证：渲染器单测 2 案（per-task 对 2 活跃 observer 调 tick_once 3 次=2 schedule+1
  终止探测、游标越界返 -1；合并模式 0 次 tick_once + 交付面不变 1/1）+ `make test`
  19,153P/0F + clippy 零警告 + fmt 干净
- **跨流观察（rule 10 归因）**：product-smoke 在**干净 main（无本切片改动）**上即
  20.30% > 20.00% 阈值（REGRESSION），与本切片无关——渲染流域近期 font/letter-spacing
  提交漂移所致，属 rendering-compat 域，本流不改不判
- reftest：本切片默认 OFF 且 reftest 单渲染路径不经 tick 循环，约束保持有效

### M2 MO-S2（第三批）— fragment flatten 验证 + quickjs 接线（2026-09-11）✅

- **fragment flatten 澄清与锁定**：native appendChild(DocumentFragment) 的树结构语义已由
  绑定层 `insert_with_fragment_flatten`（R3132）保证——逐子移动、fragment 不入树，排空侧
  每个 child 各产一条 childList record（addedNodes=[该子]，prev/next 正确）。新增 v8 门控
  集成测试锁定（f1/f2 入树 + MO 记录可达）。**批派发粒度差异**（浏览器单记录 N addedNodes
  vs 本实现 N 记录）记 MO-S3 候选（同 target+type 连续记录合并，无当前 driving 用例不冒进）
- **quickjs 接线（DC-7 对等收口）**：三处 `sync_render_after_native_dom` 调用点 gate 从
  v8 扩到 any(v8, quickjs)，`sync_render_after_native_dom`/`drain_native_mutations_to_mo`
  同步扩——quickjs native 写此前既无重渲染也无 MO 通知；mo_host_trigger 测试模块门控同扩
  （fragment 用例保留 v8-only——quickjs 绑定面缺 `__zw_native_create_document_fragment`
  工厂，DC-7 缺口记档）
- **验证**：v8 3P + quickjs 2P（attributes/remove 双引擎端到端）+ 双引擎 clippy 零警告 +
  `make test` 19,153P/0F + fmt 干净

### M2 MO-S2（第二批）— childList nextSibling 树反推导（2026-09-11）✅

- dom 层 MutationRecord 无 next_sibling 字段（结构缺口不动）——排空侧从**当前树 +
  record 自身数据反推**：prev 存在 → `doc.next_sibling(prev)`（移除后即被移除节点旧
  next）；prev None 且有 added → `doc.next_sibling(added[0])`；首子移除型（prev None +
  removed 非空）→ 移除后新首子即旧 next；非 childList 恒 None
- JS 入口追加 nextSel 参数 → record.nextSibling proxy
- 验证：webview 集成移除中间子断言 `ul/1/proxy/a/l`（prev=#a + next=#l 双臂）+
  `make test` 19,150P/0F + clippy 零警告 + fmt 干净

### M2 MO-S2（第一批）— childList previousSibling 透传 + removed '#id' 回落（2026-09-11）✅

- Rust 排空：`DrainedMutation` 扩 prev 臂——previous_sibling 经 `unique_selector_for_node`
  （在树）；removed 节点已脱树、query 唯一性必败 → 回落 `stable_selector_for_node` 的
  '#id' 形态（属性派生不依赖树位置），非 id 形态丢弃（脱树语境无稳定语义，误配比缺失糟）
- JS 入口：`__zw_mo_notify_native` 追加 prevSel 参数 → record.previousSibling proxy
- 验证：webview 集成三段（attributes 二次写 oldValue='c1' 透传 + childList 移除
  `ul/1/proxy/a`——target/removedNodes.length/removed proxy/prevSibling.id）+
  `make test` 19,140P/0F + clippy/fmt 干净
- 遗留（第二批已收 nextSibling）：fragment added flatten 语义；quickjs 接线

### M2 MO-S1 — host 侧 mutation 通知 identity 桥 + 排空点（2026-09-11）✅

方案 C hybrid（设计片 p1b-mutationobserver-host-trigger-design §5）首切片落地：

- **JS 入口**（part01.js MO 段）：`globalThis.__zw_mo_notify_native(sel, type, attrName,
  oldValue, addedSels, removedSels)` → 组 record（attributes/childList/characterData；
  characterData 自带 target——R188 分支跳过该型）→ 复用 polyfill `_mo_notify` 单注册表
  派发（options 过滤/subtree 冒泡/oldValue 语义自动获益 = 设计 MO-S3 大半免做）。
  addedSels/removedSels = '|' 分隔稳定 selector 串，逐个 `_makeProxy` 包 proxy
- **Rust 排空点**（webview.rs `drain_native_mutations_to_mo`，挂在
  `sync_render_after_native_dom` 尾）：`take_mutation_records()` 排空 +
  `unique_selector_for_node` 逐条解析身份（唯一性校验；无身份 record 丢弃）→
  execute_script 投递。覆盖全部三个 native 写检测面（execute_script /
  execute_script_with_dom / dispatch_event 尾）。**去重天然成立**：polyfill apply 路径
  自更 cached_html，不进 native 写检测分支（设计 §3 双重通知风险消解）。重入安全：
  投递前 cached_html 已同步，回调内再写经下一轮检测再排空
- **kill-switch**：`mo_host_trigger` 字段（env `ZW_MO_HOST_TRIGGER=1` 初值）+
  `set_mo_host_trigger()` setter（测试直设避免进程 env 竞态），**默认 OFF**（DC-3
  行为时序变更门禁）
- **身份形态契约**（实施中发现）：shim MO 注册键 = `'s:' + __zwSelector`，selector 为
  CSS 形态（'#t' 含 '#'）——排空侧必须传 `unique_selector_for_node` 原样输出，传裸 id
  即投递丢失（无报错静默 miss）
- **验证**：JS 入口单测（engine part03 `test_mo_native_notify_entry_m2_s1`——attributes
  透传 + target proxy + childList addedNodes 展开 + options 过滤不变）+ webview 集成
  （`tests/mo_host_trigger.rs`——native setAttribute → polyfill MO 收 record + OFF
  死路保持）+ `make test` 19,140P/0F + clippy 零警告 + fmt 干净
- **A/B（trigger ON vs OFF）双 corpus**：
  - WPT testharness corpus：IO 94/122/1 = 94/122/1、RO 19/32/6 = 19/32/6——逐 subtest
    零 delta
  - **MO 主 corpus**（dom/nodes MutationObserver 12 上游文件，既导入面——js-dom goal
    R45-R51/R188/R189 账本）：OFF 135P/3F/0T = ON 135P/3F/0T，138 subtests 逐条零
    delta（3F 为 R188 已定性 parser-interleaving 深域，非本域）
  - A/B 零回归证据成立；全量 make test ON 臂留 default-on 决策前补做（无 default-on
    计划，须用户点名）
- **MO 验证面澄清**：WPT MO 用例已随 dom/nodes 套件导入运行（12 文件常驻）——无需
  另建 testharness-mutation-observer 导入切片；MO-S2 的标尺 = 该 corpus 的 3F 收口
  （parser-interleaving 属增量解析管线，跨域）+ native 面新增集成测试
- **遗留（MO-S2 候选）**：① oldValue/added/removed 的 NodeId 真值捕获（dom 层
  record_mutation 未记录 old_value——排空侧无法透传，需 dom 层小改）；② quickjs 路径
  sync_render 未接线（v8-gated，DC-7 对等后续）；③ WPT mutation-observer 导入子集
  作为 MO-S2 验收标尺（kill-switch OFF 下走 polyfill 路径，native 面仍靠集成测试）

## 下一步计划

1. **M3-S2：renderer `tick_observers` per-task 重构**（生产面，实施要点已勘察
   2026-09-11）：
   - shim 增 `__zw_observers_tick_once()`（只 schedule/dispatch 首个活跃 observer；
     返回是否仍有余量）——现 `__zw_observers_tick()` 单 execute 批派发全部 observer
     + rAF 回调（gap-list §1.2 第二行违反点）
   - renderer `page_scripts.rs tick_observers`（L353）：kill-switch
     `ZW_RENDERER_TICK_PER_TASK` 下循环 execute→apply 逐 observer（observer 回调可改
     DOM，每轮须走 apply_recorded_mutations）；默认 OFF 维持现合并 tick
   - 风险与验收：帧 pacing 变化（每帧多轮 execute+apply）→ 需 bench-gate +
     product-smoke + reftest（observer-free 页面应零变化）三面 A/B；验收标尺薄的
     风险已在——记录清楚再动
2. **M3-S3：显式 task queue**（多队列 oldest-first，timer/network/UI 分源）——
   timer 顺序由 host 线程完成时序决定（gap-list §1.3），生产 timer 顺序保证 +
   `_defer` fallback 语义收口随本切片评估
3. **M2 MO-S3（候选）**：排空侧记录批派发合并（同 target+type 连续 childList 记录 →
   浏览器语义单记录——fragment append N 记录粒度差异；当前无 driving WPT 用例，冒进
   缓行）+ 排空批「单 execute 多 record」的 execute_script 次数优化（现逐条投递）
4. **跨流协调项（非本流可闭合，记录待碰头）**：
   - engine apply 路径稳定 selector 唯一性（RO observe-001..020 / IO handle 族根因）
   - 几何真值簇（scroll offset / transform / zoom / clip-path 参与 IO 几何）——渲染流域
5. **本流后续小修候选**：detached doc 初通知抑制 + takeRecords 真排队模型（同做）；
   scroll-margin 臂

**待用户决策清单**：
- runner viewport 校准（1280×800 → 上游 WPT 校准 800×600）：牵动全部 testharness
  套件的绝对几何期望（一次性大重校准），非轻量修复
- requestIdleCallback 真实 idle 时序（已在 goal 范围外条款）
- MO-S4 + escape-hatch 收敛（设计文档 §6 决策门禁：生产路径主干变更须用户点名）
- M3 per-task（timer 泵/renderer tick）与 task queue 的 default-on 时机（A/B 零回归
  后仍须点名——行为时序变更门禁）

### M1 切片 3c — IO/RO 语义修齐第二批 + 实验回退记账（2026-09-11）✅

- 落地两小修：① target-is-root（root==target → spec skip-to-step-11 臂：rect 留零 +
  isIntersecting false + ratio 0，仍派初通知）；② documentElement.clientWidth/Height
  viewport 化（part04 element trap；html 根 client 尺寸 = viewport 而非内容高——
  empty-root-margin 全簇 + scrollTo 族翻绿，IO +7 零回归）
- 实验后回退（详见 evidence「实验后回退」节）：observe 初通知就绪门 + rect 读
  handle→selector 回落——zw-ro-probe 诊断证实 **RO observe-001..020 族真根因 = engine
  apply 路径给 createElement 元素派 `div` 类裸 tag 稳定 selector，与既有同 tag 元素
  歧义致反查断裂**（js-dom/WC 流 apply 域协调项）；且就绪门时序变化使 observe-007/017
  回退。回退后 IO 94/217 = 43.3%、RO 19/56 = 33.9%（合计 41.4%），全量零回归
- 质量门禁：zero-engine + integration 3,455P/0F；后续剩余聚类全部落在几何真值 /
  viewport 校准 / 稳定 selector 唯一性三处跨流域 → 本流 IO/RO 轻量修复面收敛，
  主力转 M2

### M1 切片 3a+3b — IO/RO 动态跟踪接线 + 语义修齐第一批（2026-09-11）✅

- 切片 3a：runner probe 循环接 `__zw_observers_tick`（对齐 renderer `tick_observers`）
  + IO threshold 越界判定 spec 化（`_thresholdIndex` + `_crossed` OR 臂——thresholdIndex
  或 isIntersecting 任一变化即派发；旧双侧比较在默认 [0] 下永不判越阈 = 65× 簇根因）
- 切片 3b：IO 构造器校验（threshold RangeError/TypeError、rootMargin SyntaxError
  DOMException）+ `root`/`thresholds`/`rootMargin` getter + IO/RO `observe()` 非 Element
  TypeError（nodeType 判据——第一版 handle/selector 判据误伤 shadow/detached 元素已修正）
- 基线：IO 67→87（30.9%→40.1%）、RO 14→19（25.0%→33.9%）、合计 29.7%→38.8%（+9.1pp）
- 质量门禁：make test 19,134P/0F + clippy 零警告 + fmt 干净；明细与翻绿清单落
  `evidence/2026-09-11-m1-slice3-observers-wiring-validation.md`
- 残留取序（3c+）：几何真值簇（transform/zoom/scroll-offset——多数属渲染流域，碰头
  协调）、runner viewport 校准（800×600 vs 1280×800，**待用户决策**，牵动全部套件）、
  documentElement.clientHeight viewport 化、zero-area ratio=1 + edge-inclusive、
  target-is-root、detached-doc 初通知抑制

### M1 切片 2 — 事件循环时序差距清单（2026-09-11，纯文档）✅

- `evidence/2026-09-11-m1-event-loop-gap-list.md`：checkpoint 调用点精确盘点
  （v8_runtime.rs:422/486 唯一两点）+ HTML spec 事件循环算法逐条对照 + 差距→里程碑映射
- 关键发现：① 常规路径（每 `<script>` / 每异步回调独立 execute）边界已吻合 spec；
  ② 违反点集中在**批量派发**——runner `__zw_fire_due_timers`（N timer 一个 execute）、
  renderer `tick_observers`（IO/RO/rAF 全部回调一个 execute）；③ part01.js 行号已漂移
  （MO 现 L2113+、IO L2866+、rAF L3388+，契约 9/7 快照过时）
- 待用户决策清单新增登记：requestIdleCallback 真实 idle 时序（已在 goal 范围外条款）

### M1 切片 1 — IO/RO WPT 基线（2026-09-11，零源码改动纯资产）✅

- `tests/wpt-runner/scripts/fetch-observers-subset.sh`（WPT `3159769338` pin，110 IO +
  40 RO top-level .html 全量 + resources 全目录；.window.js 包装形态不拉取）
- runner 入口 `make testharness-intersection-observer` / `make testharness-resize-observer`
  （`observers_case_skipped` 内容规则筛减：ref 页 / source 含 `<iframe` / v2 / resources；
  内容规则而非名字规则——document-scrolling-element-root.html 名字无 iframe 但内容用）
- **基线**：IO 81 cases / 217 subtests / **30.9% Pass**；RO 33 cases / 56 subtests /
  **25.0% Pass**（合计 29.7%）。明细 + 失败聚类：
  `evidence/2026-09-11-m1-observers-wpt-baseline.{md,json}`
- 账本：114 用例记入 `imported-testharness.txt`（`EVLOOP-M1-observers-baseline`）
- 关键归因：IO 最大失败簇（65×）= 初通知后动态跟踪面不可达——testharness runner 不调
  `__zw_observers_tick` 且无渲染循环；runner 环境限制已记入 evidence，切片 3 修

## 下一步计划

1. **M2 MO-S2 余项**（previousSibling 透传 + removed '#id' 回落已落地，2026-09-11）：
   - dom 层 MutationRecord 补 next_sibling 字段（spec removedNodes.next——dom 层结构
     缺口，排空侧已留 DrainedMutation 扩位）+ fragment added 语义对齐（polyfill R47
     同款：addedNodes = flatten 子）
   - quickjs 路径 sync_render 接线（v8-gated，DC-7 对等——quickjs native 写后既无
     重渲染也无 MO 通知，属更大的 quickjs native 写收口面）
   - MO-S4（escape-hatch 联动验证）须用户点名（设计 §6）
2. **M3**：task queue + per-task checkpoint（kill-switch → A/B → default-on）
3. **跨流协调项（非本流可闭合，记录待碰头）**：
   - engine apply 路径稳定 selector 唯一性（RO observe-001..020 / IO handle 族根因）
   - 几何真值簇（scroll offset / transform / zoom / clip-path 参与 IO 几何）——渲染流域
4. **本流后续小修候选**：detached doc 初通知抑制 + takeRecords 真排队模型（同做）；
   scroll-margin 臂

**待用户决策清单**：
- runner viewport 校准（1280×800 → 上游 WPT 校准 800×600）：牵动全部 testharness
  套件的绝对几何期望（一次性大重校准），非轻量修复
- requestIdleCallback 真实 idle 时序（已在 goal 范围外条款）
- MO-S4 + escape-hatch 收敛（设计文档 §6 决策门禁：生产路径主干变更须用户点名）

**碰撞管理**：碰 engine 前先 `git log --since="14 days ago" -- crates/engine/
crates/script-sandbox/` 核对渲染流域活跃面。

## 里程碑状态

| 里程碑 | 状态 |
|--------|------|
| M1 — WPT 基线 + 时序差距清单 | ✅ 2026-09-11（基线 29.7% + 差距清单；语义修齐 3a+3b+3c → 41.4%；剩余聚类为跨流域协调项，主力转 M2） |
| M2 — MutationObserver host 触发 | 🔄 MO-S1 ✅（identity 桥 + 排空点 + kill-switch OFF）；MO-S2（派发深化 + WPT 标尺）进行中 |
| M3 — checkpoint spec 化 | 🔄 M3-S1 ✅ + M3-S2 ✅（runner timer 泵 / renderer tick_observers 双 per-task，均 kill-switch 默认 OFF）；余显式 task queue |

## 验证基线

- 测试基线：立项时点全绿（`make test` / `make reftest` 入口，经 test-guard 包裹；
  禁止裸跑 cargo test）。2026-09-11 切片 3a+3b 后：make test 19,134P/0F；切片 3c 后
  engine + integration 3,455P/0F
- IO/RO 用例面：基线 29.7%（IO 30.9% / RO 25.0%）→ 切片 3a+3b+3c 后 **41.4%**
  （IO 94/217 = 43.3% / RO 19/56 = 33.9%），明细见 evidence/
- 质量门禁：`cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` 全过；
  时序变更必须 kill-switch + 全量 A/B 零回归；渲染相关门禁（product-smoke/bench-gate）
  在 tick 排布变更轮按 run-rules §12 判断是否需要（本切片仅 runner probe 循环加一次
  shim 调用 + JS 语义，未触渲染管线，product-smoke 不适用）
