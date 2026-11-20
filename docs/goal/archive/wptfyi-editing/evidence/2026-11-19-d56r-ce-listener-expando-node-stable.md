# d56r · ①(b) 刀落地与定谳——监听面跨族调和＋expando 节点稳定键：机制兑现、套件不可交付、root root 重定向

**2026-11-19 · d56 轮后续（①(b) 落刀轮）**。上轮 d56b 定谳机制链（attach 挂 W1、
键链换代派发落 W2/W4、监听器桶键孤儿化 → bi=0 族）后，本轮落两段刀并全链 A/B。
结论：**合并面机制兑现（隔离账 7 键翻面、哨兵三零回归），但 input-events 套件净
-61 不可交付，两刀已回退**（完整 diff 存证于
[2026-11-19-d56r-knife-ab.patch](2026-11-19-d56r-knife-ab.patch)）。root root 上溯
一层到 **wrapper 身份跨查询不稳定**（刀 C，下轮主线）。

## 1. 刀设计（两段，match-only 方针内）

### 刀 A：监听器注册/派发面节点 id 桶调和

- **桥**：新宿主回调 `__zw_node_id(key)`（callbacks.rs）——key→NodeId（u64 十进制串），
  LIVE_QUERY_DOC 优先、miss 回落查询快照 re-parse（与 gBCR 同源）；`@` handle 键与
  不可反查键返空串（零行为变化）。
- **注册面**（part04 元素臂 attach）：record 同引用双写 `_zwNidBuckets['id:<nid>'][type]`
  （R143 同款 fn+capture 去重），record 带 `_zwNid` 印记；全局 `__zwNidCount` 计数兼
  零成本门。removeEventListener / once 派发移除 / signal-abort 三处同步跨桶清理。
- **派发面**（part03 `_dispatchToListeners`）：station 桶 ∪ nid 桶合并（`_zwStationList()`
  每段实时取，R331 重克隆语义保持）；合并去重两级——record 引用级 + fn+capture 同型级；
  R111「removed 则跳过」检查跨桶化（`_zwLive`）；once 移除跨桶；
  `event._zwNidFired` per-dispatch 印记（`_dispatchWithBubble` 入口复位）保同一逻辑节点
  经多站点出现的单次触发。
- **spec**：dispatch 以 event path 节点为单位 invoke listener 表，wrapper 家族身份不参与
  ——dom §concept-event-dispatch；listener 属节点 listener 表——dom §concept-event-listener。

### 刀 B：expando 桶键节点稳定化

- 设计动机（后被证伪，见 §2）：`_expando[key]` 按 wrapper 键存——churn 后 harness
  `isListeningToInputEvents` 守卫旗跨族读空 → 每测重新 attach → stale 积累。刀 B 使
  expando 键经 NodeId 稳定：`_zwExpandoKey(key)`（part01）——sel 键经 `__zw_node_id`
  反查 NodeId 共享桶（首访问缓存）；`@` handle 键/不可反查键回落 wrapper 键。19 处
  `_expando[key]` 代码位（part03×1 / part04×12 / part05×6）切换键基。

## 2. 机制定谳链（本轮诊断与勘误）

| 探针 | 结论 |
|---|---|
| zz-diag-d56r1（hr 键单测页，双基座） | 刀前 gBIlen=0 → 刀后 gBIlen=1、document 捕获层 1 次派发——**合并面单测兑现** |
| zz-diag-d56r2（got=4 键单测页，D56DIAG2 派发漏斗记账） | 单测 1 次派发 1 次点火——单测派发面干净 |
| zz-iso 20 键隔离账（刀 A+B，40 子测） | **7 翻 PF→PP**（bs-03/06/07/10/11/12、fd-02），0 翻破坏，20 稳 FF（无 join 族 caret 面＋gtr 值形域，另一刀），13 稳 PP——**合并面本身干净** |
| **勘误①：套件级 got=2 面不是 stale 积累单因** | harness 每测进 churn 分支（`gEditor !== document.querySelector(...)` 在 wrapper 身份漂移世界恒真）：`isListeningToInputEvents=false`（旧行）→ remove 用**本测新闭包**（fn 不匹配，spec 无操作）→ 守卫必假 → **每测必重挂**。每挂进 nid 桶（节点级、永不清）→ 合并面全可达 → got=N。**真浏览器身份稳定 → 分支永不跑 → 单监听器**——生命周期差异是结构性的，非桶键选择可修 |
| **勘误②：刀 B 键→nid 首访问缓存投毒** | `_zwKeyNidCache` 按选择器串缓存 nid，但选择器串跨 mutation 不稳定（nth-child 移位后同串解析到不同节点）→ 旧节点 nid 发给新节点 → `_zwSelPendingParent` 等选择机制 expando 跨节点串数据 → 删除不发生/结构错。P2F=95 中 **other=80** 为此类（innerHTML 断言面），got=2 仅 5 |
| 刀 B 性能面 | dom `Range-mutations-replaceData.html` 风暴页 86 键早停（page timeout 预算内少跑；已跑键全 Pass）——mutation 风暴不断产生新 wrapper 键 → 每新键一次 `__zw_node_id` 宿主往返。无错误结果，纯性能回退 |

## 3. A/B 账（exact-set，/tmp/d56r-{suite,dom,uievents,selce}-{knife,base}.json）

| 套件 | knife 键 | base 键 | F2P | P2F | 判定 |
|---|---|---|---|---|---|
| input-events 全量 | 6663 | 6664 | 34 | **95** | **净 -61，不可交付**（P2F 构成：other=80 刀 B 投毒、got=2 ×5、got? ×10） |
| dom | 57777 | 57861 | 0 | 0 | 零回归（replaceData 页 86 键早停＝性能面） |
| uievents | 2400 | 2400 | 0 | 0 | 零回归 |
| selection-CE（filter "contenteditable"） | 414 | 414 | 0 | 0 | 零回归 |

input-events F2P=34 明细：backspace +2、forwarddelete +1、non-collapsed-selection +3、
deleting-in-list-items +4、zz-probe b3p/b3q/b3r +2×3、d46b +3、d47b +1、d49a/e +2、
d53g/k +5、zz-iso 7（见 §2 隔离账）。跨族可达修复真实存在，但被 P2F 面淹没。

## 4. 哨兵

- enginelib（刀 A+B 在树时）：`cargo test -p zero-engine --lib` 经 --compile-first
  **4214P/0F**（绿）。
- dom / uievents / selection-CE：见 §3，三零回归——**合并面与 expando 重键在
  非 input-events 语料世界无扰动**（刀 C 落地时刀 A 合并面可复用的安全证据）。

## 5. root root 与刀 C（下轮主线）

本轮定谳：bi=0 族（d56b 链）与套件 got=N 面共享同一 root root——**wrapper 身份跨查询
不稳定**（同一节点两次 query 返回不同家族 wrapper，`===` 判 churn）。真浏览器语义：
节点不变 → wrapper 不变 → harness churn 分支永不跑、单监听器、expando wrapper 键天然
节点稳定。刀 C（wrapper 身份稳定化）落地后：churn 分支消失（生命周期对齐）、expando
无需重键（刀 B 整体作废）、派发站键与 attach 键有望经稳定 wrapper 归一（刀 A 合并面
或可简化退役，其安全域已由本轮哨兵证明）。

刀 C 设计前必读：R52 receiver identity 保持（`_proxyCache` 消零清）、d56b 换代链
（W1→W4 触发点）、方针 #32（多族 wrapper 结构属性 match-only——身份稳定不统一结构，
两段刀均在方针内的先例）。风险面：身份稳定使缓存失效策略从「mutation 即弃」变
「节点存活即留」，需核对所有依赖换代清态的代码位（slice47 压实豁免、s284、R4005）。

## 6. 运行产物

- /tmp/d56-clean-runner（刀 A+B v4 干净版）、/tmp/d56-wip0-runner（base 基线）、
  /tmp/d56-diag2-runner（D56DIAG2 诊断版）
- /tmp/d56r-{suite,dom,uievents,selce}-{knife,base}.json（四套件双腿，systemd-run
  d56r-final2.service 串行）
- /tmp/d56r-compare.py（exact-set 对账器）、/tmp/d56r-final-ab.sh（运行电池）
- 诊断页（gitignored wpt-data）：zz-diag-d56r1/r2（本轮）、zz-diag-d56n/p（上轮遗留）
- 刀 A+B 完整 diff：[2026-11-19-d56r-knife-ab.patch](2026-11-19-d56r-knife-ab.patch)
  （已回退，538 行）
