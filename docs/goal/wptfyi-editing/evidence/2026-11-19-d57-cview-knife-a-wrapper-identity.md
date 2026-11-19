# d57 · 刀 C-view 落地——查询串粘性 canonical：wrapper 身份跨 materialize 稳定、churn 分支结构性消失

**2026-11-19 · d57 轮**。上轮 d56r 定谳 root root＝wrapper 身份跨查询不稳定（canonical
串换代），刀 C 设计定型。本轮落 **C-view（query_match 粘性 canonical）＋刀 A 重放**
（listener nid 合并面），刀 B 维持作废。结论：**机制全链兑现**（d57a census 翻面点
归零、harness churnBranch=false、刀 A 单测页 gBIlen=1 保持、20 键隔离账 5 键翻面），
套件 A/B 账见 §3。

## 1. 刀设计（match-only 方针内）

### C-view：查询返回点 canonical 粘性（css_wire.rs）

- **病灶链**：innerHTML 写积 pending journal → 首键 mutation materialize 快照 →
  `div` 最小形失去唯一性 → `unique_selector_for_node_ctx` 重算翻结构路径
  （`html:nth-child(1) > …`）→ shim `_wrapSelector` 按串建 proxy → 同节点两代查询
  返回不同 wrapper → corpus harness `gEditor !== document.querySelector(...)` churn
  分支每测重挂、监听器孤儿化（d56r §5 root root）。
- **刀面**：`query_match_selector_doc_ctx` 返回点接入粘性层——thread_local
  `STICKY_QUERY: HashMap<原始查询串, canonical>`（cap 65536 溢出清空）＋导航清空钩子
  `reset_sticky_query`（webview `load_html` 文档换代块、renderer `js_worker.rs` 与
  browser `tab_js_worker.rs` 的 `url_changed` 分支三处调用，与 `bump_mut_drain_gen`
  同批 per-page 语义）。
- **d57a 勘误①（键基）**：粘性键首版按 `(html_ptr, NodeId)`——d57a census 实证粘性在
  **每个 materialize era 内**成立（t1→t3 `div` 稳、u1→u3 路径形稳）但跨 drain 必失：
  mutation drain 重建快照 Arc、`Arc::as_ptr` 变化 → 键失配 → 重算翻面。对象身份键
  在 re-parse 世界结构不可跨代。
- **终版键设计（勘误①修正）**：按**原始查询串**粘性。陈旧条目由命中时**自验证**淘汰
  ——memo 串经 `find_by_selector[_cached]` 须解析回本次找到的节点才可信，否则重算
  刷新。自验证保证**服务串恒解析到服务节点**（resolution 不变量＝query_match
  canonical 的真实契约，历史 `<option>` 歧义 bug 即该不变量破坏；唯一性只是其实现
  捷径，非独立不变式——`div` 歧义后首匹配仍是 wrapper，粘性 `div` 无害且合法）。
  查询串键跨页面内 drain 存续（内容验证门），粘性使 `===` churn 消失 → harness
  生命周期对齐。spec：DOM wrapper identity per node（同一 Node 返回同一 JS 对象）。
- **d57 勘误②（页面作用域）**：查询串键首版跨页存活——selce 套件
  modify-around-non-editable-span 4 键 P2F（单跑 A+C 16/16 全过、knife-A-only 单跑
  0 fail＝纯 C-view；套件上下文才现形）：旧页 memo 条目（path-form canonical）在新页
  自验证通过（解析仍命中同逻辑位）但换 wrapper 家族键 → 下游家族敏感面（selection
  modify walk 跨非可编辑 span 跳位）行为翻转。自验证只保证 resolution 正确，不保证
  与新页全新计算同 canonical 形。修正＝`reset_sticky_query` 导航清空（页面内 drain
  不清，d57a 翻面消除不受影响——v3 电池实证）。

### 刀 A 重放：listener nid 合并面（callbacks.rs / part01/part03/part04）

与 d56r §1 刀 A 全同（`__zw_node_id` 桥、part04 元素臂双写、part03
`_dispatchToListeners` station∪nid 合并＋两级去重＋跨桶 remove/once/abort），
完整 diff 见 [2026-11-19-d56r-knife-ab.patch](2026-11-19-d56r-knife-ab.patch) 减刀 B
面。刀 B（expando 节点键）**维持作废**：C-view 后 query 家族 wrapper 键天然持久，
唯一残留跨族面是 CE 键管线（`selector_for_page_node_handle` ctx=None 全新算路径形）
的 station 键——由刀 A 合并面兜住派发；expando 状态跨族共享（bs-03/06 的
selection pending 面仍依赖）不在本轮刀面内。

## 2. 机制定谳链（d57a/d56r 单测页）

| 探针 | 刀前（base） | 刀后（A+C） |
|---|---|---|
| zz-diag-d57a T1 census t4-postKey（materialize 点） | `q.sel=html:nth-child(1)>body:nth-child(2)>div:nth-child(1)`、`gEd===q:false` | `q.sel=div`、`gEd===q:true` |
| zz-diag-d57a T2 churnBranch | `true`（每测重挂分支必进） | **`false`** |
| zz-diag-d56r1（hr 键，attach+派发单测） | gBIlen=0 | gBIlen=1（刀 A 派发面兑现） |
| zz-diag-d56r2（got=4 键派发漏斗） | got=N | 单测 1 派发 1 点火 |
| zz-iso 20 键隔离账 | 13 PP/20 FF | 18 PP/22 FF（bs-07/10/11/12、fd-02 翻 PP；bs-03/06 全量套件内翻 PP 而隔离页内 FF——见 §3 F2P 明细；余 FF＝join 族/无 join caret 面，§3.1） |

## 3. A/B 账（exact-set，knife=本刀终版（干净二进制 d57-final-runner，含勘误②页面级清空）/ base=d56r base 腿复用，同 HEAD 9f47b0575）

| 套件 | knife 键 | base 键 | F2P | P2F | 判定 |
|---|---|---|---|---|---|
| input-events 全量 | 6667 | 6664 | **816** | 20 | **净 +796，可交付**（P2F 构成：join 族 18 + zz-diag 诊断页 2，见 §3.1） |
| dom | 57864 | 57861 | 0 | 0 | 零回归（Range-mutations 风暴页早停 3 键＝纯性能面，shared 键全对齐；d56r 刀 A+B 同面 86 键更宽） |
| uievents | 2400 | 2400 | 0 | 0 | 零回归 |
| selection-CE（filter "contenteditable"） | 430 | 414 | 0 | **0** | 零回归（键差 16＝zz-diag-modify-span 诊断页；勘误② 4 键 P2F 已由页面级清空修复归零） |

input-events F2P=816 明细（bi=0 族结构性翻面——C-view 身份稳定 + 刀 A 派发合并
双条件齐备，d56r A+B 因 churn 每测重挂使 got=N 淹没全部赢面）：deleting-in-list-items
+368（Delete,ul/ol 各 105、Backspace,ul/ol 各 79）、d47b +79、d53g/k +189、
non-collapsed-selection +37、b3p/b3q/b3r +30、backspace +10、forwarddelete +3、
zz-iso +7（bs-03/06/07/10/11/12、fd-02——A+B 7 键全数保住）、d49a/e +2、d54a/d56j
+4。键集差仅新诊断页（gitignored wpt-data）。v1（无页面级清空）与 v3 终版电池
F2P/P2F 逐套件完全一致——页面级清空对 corpus 赢面零扰动。

### 3.1 P2F=20 构成与 root cause（新定谳：join 面 materialize 竞态）

- zz-diag-d56.html ×2：d56 轮诊断残留页（gitignored），非上游语料，剔除后上游
  **净 P2F=18**。
- 18 键全为 join 族复制形（`<div>abcdefdef<p>ghi</p></div>`）：bs-06/13、b3p/q/r、
  d46b×5、d56j×3、d56jt1、non-collapsed-selection×5、backspace×1。
- 诊断（zz-diag-bs13 + part01 临时 marks，已回退）：CE 键管线 sel=path-form，
  DELentry `el=document.querySelector(pathSel)` 解析到 path-form wrapper
  （`elSameAsDivQ=false`——两族 wrapper 并存实锤）；shim 图 join **本身正确**
  （`_zwCeTakeLineFrom` 后 `srcPnow="abc<p>ghi</p>"`＝p 已失 def）；引擎终态
  `abcdefdef`＝remove 失真重放。
- **机制**：base 世界 join 面无 listener（bi=0）→ `getTargetRanges()` 不被调用 →
  join wire 批在单一代际一致重放 ✓。knife 世界刀 A nid 合并使 harness listener
  可达（bi=1 ✓、gtr 断言 ✓）→ harness 回调读 `getTargetRanges()` → **派发中引擎
  往返** → 撕开 pending-journal materialize 时序（innerHTML 写积 journal 在 join
  dispatch 与 mutate 之间落快照）→ join wire 落换代后快照重放失真 → 内容复制。
- **修复方向（下轮刀 D，独立 A/B）**：`__zw_ce_delete_impl` 入口（派发前）强制
  materialize——dispatch/GTR/mutate 三相同代际。blast radius 覆盖全部 816 赢面
  （文内臂 sc.nodeValue 路径同为 journal-pending 世界），须整套 A/B 重跑验证，
  不在本轮随 A+C 交付。

## 4. 哨兵

- enginelib：`cargo test -p zero-engine --lib` 经 --compile-first **4214P/0F**（绿，
  与 d56r 基线同量）。
- dom / uievents / selection-CE：§3 表三零回归——**刀 A 合并面 + C-view 粘性 +
  页面级清空在非 input-events 语料世界无扰动**。

## 5. 性能门禁（#23）

- 全量 bench-gate 三跑（`ZERO_WEB_BENCH_CRATES=zero-engine`）：run1 与电池并行
  （page/* 墙钟 6 FAIL，3× 负载噪声特征）；run2 仍与 selce 腿并行（11 FAIL）；run3
  空闲机收敛至 **2 FAIL 且均为 `mb/*` 纯 Rust criterion 基准**
  （dirty_tracking/mark_100_nodes_and_merge、medium_complexity_page/incremental_vs_full）
  ——该两基准不经 js_dom_bridge 回调（query_match 粘性层不可达），理论面即排除本刀。
- **静窗定向微基准定谳**（load 0.85，`cargo bench --bench engine_bench --` 过滤两键，
  knife 树自身）：mark_100_nodes_and_merge **1.72µs**（基线 1.758µs 级，较噪声窗读数
  -54%）；incremental_vs_full **3.467ms**（基线 3.143ms 级，预算 4.243ms 内，
  criterion 判 "No change in performance detected"）——**两 FAIL 归因测量窗噪声**
  （R5500 同款：基线 justification 已载「机器微基准噪声态漂移」先例；基线
  2026-11-11 @718e0f0e6 距今 1264 commit）。机器有两家长驻 codex agent＋并行流，
  静窗稀缺，全量 gate 复跑留待下轮窗口。
- 真实成本信号：dom Range-mutations 风暴页早停 3 键（57,864 vs 57,861，shared 键全
  对齐）＝粘性验证在 mutation 风暴下的最坏面，可忽略。

- /tmp/d57-final-runner（终版干净二进制：刀 A + C-view + 页面级清空，strings 核验
  `_zwExpandoKey|_zwKeyNidCache|zwDiagJoin` 零命中＝刀 B/诊断印记无残留、
  `__zw_node_id`/`_zwNidBuckets` 在）
- /tmp/d57v3-{suite,dom,uievents,selce}-knife.json＋.err（终版 knife 腿，systemd-run
  d57v3-final.service 串行）；base 腿复用 /tmp/d56r-*-base.json（同 HEAD）
- /tmp/d57-compare.py（exact-set 对账器）、/tmp/d57v3-final-ab.sh（终版电池）
- /tmp/d57-benchmini-knife.log（静窗定向微基准，两 FAIL 键归因证据）；
  /tmp/d57-benchgate{,2,3,-head}.log（全量 gate 三跑＋HEAD 跑被外负载污染记录）
- 过程产物：/tmp/d57-cview-runner（v1 对象键版）、/tmp/d57-cview2-runner（v2 无清空）、
  /tmp/d57-knifeA-only-runner（勘误② 二分）、/tmp/d57-diag-runner（join 面诊断版，
  part01 临时 marks 已回退）
- 诊断页（gitignored wpt-data）：zz-diag-d57a（本轮 wrapper 身份 census）、
  zz-diag-bs13（input-events join 面诊断）、zz-diag-modify-span（selce QM 流量诊断）、
  zz-diag-d56r1/r2（沿用）
