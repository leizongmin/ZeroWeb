# editing-compat — 簇 A（cloneNode 前置成本）归因收口：填充/物化逐节点 JS glue 定量（2026-10-30）

承接 [M2 次轮自旋簇归因](2026-10-30-m2-r2-spin-cluster-attribution.md) 簇 A 挂账：
runConformanceTest 每数据行 `documentElement.cloneNode(true)`（include/tests.js 5582/5631）
的性能簇。本轮目标：把「~54ms/次克隆」定位到可修的机制层，产出修复 scope 与基线。
方法：同树四轮计时探针（临时页 `zz-clone-cost-probe.html` 经 editing run 通道单案
复跑，**测完即删**，未入语料盘点；计时放 test name——Pass 行不打印 assert message）。
归因纪律 #35 全程适用：单段微基准不作为慢的证明，以下每条结论都有全链与分解双侧证据。

## 1. 第 1 轮：单次克隆阶段分解 + 字节 vs 节点

页面形态与 run 语料页同构（harness 内联 + 50 节点级 contenteditable 宿主）：

| 阶段 | 耗时 | 说明 |
|---|---:|---|
| ih 读（documentElement.innerHTML 序列化） | 0.2ms | 融合序列化可忽略 |
| set（新 div.innerHTML = ih，全链填充） | 30.6ms | host 重解析 + registry 填充 |
| clone1/clone2（cloneNode(true) ×2） | 65.2 / 62.6ms | ≈ 2×set + 逐节点物化 |
| q（克隆件 querySelector） | 1.2ms | 消费面可忽略 |

字节 vs 节点分解（同页第二轮）：`setBigText`（200KB 单文本节点）**2.5ms**——
解析器非字节二次方；`set2000span`（2000 个 `<span>a</span>`，~50KB）**2184ms**
（~1.1ms/节点）；`setBody` 28.0ms vs `resetBody`（同串第二次重设）102.7ms
（3.7×）；`cloneBody` 136ms。**结论：放大与节点数挂钩、与字节数无关**；
同一容器的重设比首次贵 3.7×。

## 2. 第 2 轮：S1-S8 原语分层（2000 span 定标）

| 层 | 耗时 | 判定 |
|---|---:|---|
| S1 host 解析+深 JSON（`__zw_parse_html_subtree` 裸调） | 1.3ms | 线性，免罪 |
| S2 裸宿主队列（2000 create + 2000 text + 4000 append_child_handle） | 1.4ms | ~0.23µs/调，免罪 |
| S3 `document.createElement('span')` ×2000（JS wrap 层） | ~27ms，桶 [8.5\|6.3\|6.7\|5.2] | ~13µs/次桶平（线性），免罪 |
| S4 预建 span `appendChild` ×2000（JS append trap 全链） | ~572ms，桶 [134\|114\|145\|179] | ~286µs/次，**主项**（桶微增） |
| S5 全链 set（innerHTML= ，含 t8r 物化） | 3236ms | 主项 |
| S6 裸 `__zw_set_inner_html_handle` | 0.0ms | 队列 push，免罪 |
| S7 S5 产物 `cloneNode(true)`（deep 臂 + 再物化） | 3376ms | ≈ S5——克隆贵在物化 |
| S8 同 div 重设（替换 2000 旧子） | 13217ms | 首设 4×——移除侧逐子成本 + 全局状态增长 |

host 侧三层（解析/队列/wrap）全部线性且合计 <30ms——**2 秒级成本全部在 JS
shim glue 层**。

## 3. 第 3 轮：setter 内部相归因（裸回调拦截）

setter 裸调用的 5 个宿主回调是 call-time 全局查找 → monkey-patch 可拦截。记录
累计耗时 + 首/末 append 时间戳（时间戳在裸回调内部取，**窗口含相邻迭代 trap
glue**——B/C 相边界受此污染，判读见 §5）：

| 相 | S5（首设） | S8（重设） | 说明 |
|---|---:|---:|---|
| A（起点→首个 append：parse + plain 建树） | 80ms | 83ms | 与 S1+建树一致 |
| B（首 append→末 append 窗口） | 670ms | 2366ms | 迭代 trap glue |
| C（末 append→终点） | 1637ms | 3002ms | 尾迭代 glue + 尾钩 |
| 拦截宿主回调合计 | 2.3ms（parse 1.3 + append 1.0/2000 调） | 同量级 | **残差 ≈ 100% 纯 JS** |

拦截合计仅 2.3ms——setter 的 2.4-3.2s **全部是 JS 侧 trap/hook 执行时间**，
宿主桥调用本身可忽略（~0.5µs/调）。

## 4. 第 4 轮：单次操作成本表（元素 proxy）

| 操作 | 单次成本 |
|---|---:|
| `__zwHandle`（R351 短路键） | 0.41µs |
| `getPrototypeOf`（trap） | 0.57µs |
| `'nodeType' in el` | 0.33µs |
| nodeType / tagName / childNodes / textContent / getAttribute | 2.3-3.8µs |
| parentNode / firstChild | 3.4-3.8µs |
| form（trap 内 `_realTag` + 原型走查回落） | 6.4µs |
| **createElement + createTextNode + append text + append span（全新 pair）** | **960µs** |

属性读全部 µs 级（R351 时代的 78µs/读已被既有短路消除）——**慢不在读，在
append 链**：全新 pair 960µs 与 §3 的 B+C 合计（~2.3s/2000 对）同量级互证。
S4（仅 span append）286µs/次 vs pair 960µs——差额 ~650µs 落在 text 子 append
一侧。**机制注记（2026-10-30 二轮修正）**：复核 part06.js:4014 后确认主文档
`createTextNode` 产物是 **handle 形**（`__zw_create_text` + `_textHandles` +
`_wrapHandle` proxy），text 子与 element 子同走 appendChild trap 的 handle
子分支、同 wire `__zw_append_child_handle`（第 3 轮拦截 nA=2000 只计 element
append 是拦截面缺口，非分支差异）；本文初稿「plain 节点走 plain 子分支」的
机制断言有误，~650µs 差额的精确分支归因（trap 内 s95 `_zwS95ChildText` 文本
门 / text 相关 `_zwRegisterTextEl` 面 / 或纯逐调用 glue 噪声）**未定**，
需 trap 内部插桩（`Object.defineProperty`/裸回调全量计时）才能收口——修复
设计不建立在未证机制上。

## 5. 结论与修复 scope

1. **机制定位**：cloneNode deep 臂 = `__zw_set_inner_html_handle`（队列 push，
   0ms）+ `_zwFragmentAdded`（parse+plain 建树，80ms/2000）+ t8r 同款
   `_wcRebuildAsHandle` 逐顶层物化（part04.js ~12811 与 ~6377）——物化逐节点
   `createElement`+`createTextNode`+`appendChild`，每节点付 appendChild trap
   全链 glue（element 子 ~286µs/次，text 子一侧 +~650µs 差额待插桩归因，
   见 §4 修正注记）。innerHTML setter 对 handle 容器同款物化（t8r），
   故 clone ≈ 2×set。
2. **放大项**：重设 2-4×（S8、resetBody 3.7×——移除侧逐子成本叠加）+ 全局
   状态温和超线性（S4 桶 134→179、reset 增幅随累计创建数增长）。非解析器、
   非宿主桥、非属性读。
3. **消费者影响换算**：~100 节点语料页单次克隆 30-65ms（第 1 轮实测）×
   bold 类案 ~760 数据行 × 4 变体页 ≈ 分钟级/案——与 M2 预算窗
   「3048 行需 ~300s」定量吻合。R5342 + 300s 预算已解除墙钟截断（终账
   2,406P/12,308F/0T），**本簇现为纯性能余量项，非正确性/解锁项**。
4. **修复 scope（下轮立项）**：appendChild trap 热路径（part04.js 6622 起
   ~450 行语义密集 dispatch，每调用 ~25-40 个 try/catch 内 helper 调用的
   分布式成本，无单一 villain——286µs ≈ 全部子步之和）。候选：①循环不变量
   提出（`_realTag(sel,handle)` 每调用算 3-4 次、CE/mo/live-range 门控前置，
   guideline #26 既有先例 t8，预期 1.5-2×）；②常见形态快速门（handle 子 +
   handle 父 + registry 空 + 无 observer，预期 5-10×，须逐义务核对等价）。
   **约束**：该 trap 承载 ~15 项语义义务（R177 校验/Document-Attr 拒绝、
   s95 SCRIPT 门、R368 plain 盖章、R334 sel 子路径、R97 fragment 展平、
   容器门、R51 反链、R50 live 集合失效、R3909 select 复位、MO 记录、
   range adjust、CE connect、s131 form 键、picture/img mutation）——按
   guideline #35 须整体替换 A/B（全链等价实现对照，非单点微基准），跨域
   抽查 dom + html-syntax + selection + web-components 后方可落。
   **本轮不动手**（风险面与验证量需独立整轮）。
5. **尾钩单修收益有限**（不支持顺手小修）：s131 sweep `.form` 读 6µs × n、
   CE `_elConnected` 父链走查 ~15µs × n、R3k 命名注册 plain 遍历——合计
   ≤100ms/2000 量级，占总成本 <5%；主项是逐节点物化 glue 本身。

## 6. 复验入口

- 探针四轮已删（`wpt-data/editing/run/zz-clone-cost-probe.html` 不存在——
  残留会污染全量基线扫描）。重建方式：本文件 §1-§4 的探针脚本描述可复原。
- 单案通道：`make testharness-editing-run-one FILTER=zz-clone-cost-probe`。
- 修复验收基线：S5 ≤300ms/2000 span、S7 同量级、S8 ≤1.5× S5、bold 四变体
  单案墙钟对照（当前 ~300s 预算占用 → 目标 <100s）、全量 run 面零 verdict
  级回归（逐案折叠变体 diff）。
- 二轮修正（2026-10-30）：§4/§5 机制注记按 part06.js:4014 复核修正
  （text 子为 handle 形）；~650µs 差额分支归因待 trap 内部插桩，修复设计
  不建立在未证机制上。

## 7. 修复落地（2026-10-30 三轮，guideline #35 整体替换 A/B）

承接 §5 scope，四笔共享路径修复（part03/part04/part06），全部经探针
A/B 归因锁定后落地。**测量方法论**（两轮探针 ~15 个对照象限沉淀）：
单段微计时会漏掉 trap 间 gap 的成本（本轮 17.3ms/call 的真凶正藏在
所有段计时器都照不到的 `typeof` 全局查找里），最终口径 = 入口/出口
单对计时 + closure 持有累计对象（不走 globalThis，避免探针自身污染
被测路径）+ 阶梯 stub（L1 只读父 / L2 加子扫描）逐层排除读取项；
`performance.now()`（µs 精度）替换 `Date.now()`（ms 精度，clock 基准
实测 x1000 仅 0.1ms，精度足够）。

### 7.1 根因 1：未注册 globalThis 属性 miss 17.6ms/次（part06）

remove 侧 `AdjustRangesForRemove`（part03 262 流程）每次调
`typeof globalThis.__zwSelectionFollowRemoval` 判断 selection 联动
回调是否注册。该属性原在 `_getSelection()` 内部惰性注册——**从不调
getSelection 的页面永远不注册**，`typeof` 打到 globalThis 解析 miss
慢路径（命名属性/宿主扫描走遍累计页面状态），实测 **17.60ms/次**
（装饰器报 17-18ms 而所有段计时器全 µs 级的"测量悖论"，即此调用落在
段间 gap；直接插桩 `"undefined|17.60"` 钉死）。已注册属性命中是 ns 级
（`__zwLiveRanges` 对照）。

修复：注册块从 `_getSelection()` 体内（原 part06:10048-10115）提升到
文件顶层函数声明前，缩进去 2 层；函数体首行
`if (!_selection || !_selection._ranges || _selection._ranges.length === 0) return;`
无 selection 安全 no-op；`if (!globalThis.__zwSelectionFollowRemoval)`
幂等门保留。效果：remDbg 17.3ms→0.009ms/call，p3t 纯跑 move
4553→~112ms（0.45ms/move，-97.5%）。

### 7.2 根因 2：R155 全表反查 O(registry)（part04 调用点 + part03 查询点）

R155（part03 `_parentNodeFor` 16993 区）在 `_zwNodeParent` 反链缺失时
对 `_handleChildren` 全表 `for...in` 反查 × 逐子 `__zwHandle` get-trap
读，实测 **6022 次调用 0 次命中、~2.5ms/次**——innerHTML bulk 桶替换
（part04:12840 `_handleChildren[handle] = _ihMat`）不记反链，批量解析
产物子代反链全缺，反查纯浪费。调用链钉死（栈采样
`_parentNodeFor ← Object.get ← _elConnected ← Object.set`）：part04
innerHTML setter CE 连接环 12957-12967 对 `_ihMat` 逐子
`_elConnected(_ihCw)`，父记账 `_ihPc !== true` 时允许 per-child 升级。

修复（调用点手术，O(1) 反向索引方案否决——`_handleChildren` 变更面
~30 处散布两文件，回归风险不可控）：`_ceParentConnected` 的 false 分支
权威（sel 父走 host `__zw_contains` 判定、handle 父走 `_ceConn` 记账），
而 DOM spec `concept-connected`（connected ⇔ shadow-including root 是
document）下子代刚装入本容器、root 与父同一，**父断 ⇒ 子必断**，
per-child 树上行冗余；实测升级路径在批量流恒不触发（R155 miss →
null 父 → false）。循环整体抬进 `if (_ihPc === true)` 门。效果：R155
0 entries（三次全 0），S5 2941-2984→315ms、S8 5853-6429→829ms、
p1b-set2000span 1228-1280→216ms。

### 7.3 根因 3：插入侧二次方（part03 trap 键控根验证 + `_realTag` 前置）

append/remove trap 的兄弟定位与根验证每调用全量扫 `_handleChildren[ph]`
（`indexOf`/`in` 扫描），2000 子时 O(n²)——S4 1638ms、p3-append-text
9606ms。修复三件套：Fix 1 `_realTag` 盖章前置（part04 AREA 门加
`nodeType === 1` 先决，text/comment 子跳过 tag 解析——消费门本就只要
元素）；Step 1 惰性 newIndex 闭包 + 插入侧 `_handleChildren[ph].indexOf`
快速路 / 移除侧 `_oph262`/`_oreg262.indexOf` 快速路；Step 2 键控根验证
（`sameParent263Verified`/`sameNode262Verified`/`sameParent262Verified`
handle 匹配跳过根核查，`ensureInsRoot263`/`ensureRmRoot262` 惰性）。
效果：S4 1638→208ms、p3-append-text 9606→268ms（-97%）、adjIns≈0。

### 7.4 并行流撞车与合并（2026-10-30，同日双流独立归因簇 A）

rebase 时发现并行流已落 `fc12a1ff8`（R263b/R262b 预筛门——range 注册表
非空时 O(n) childNodes refresh + 索引扫的 pass1 保守超集预筛，tape 归因
主项 s35 占 73-86%）。两流对同簇独立归因到**不同 villain**：他们修
AdjustRangesForInsert/Remove 内部的注册表循环；本轮修 trap 层 sibling
定位/根验证/globalThis 探测/R155 反查。机制互补、效果部分重叠（本探针
R=1 下两边的快道各自消除同款 O(n)/append 读）。合并方式（part03 唯一
冲突区 = R262 头部）：保留上游 R262b 预筛全文，`rmRoot262` 急切根游走
改回本轮惰性 `ensureRmRoot262()`（null 父早退路径免一次根游走；预筛两处
同根比较与 R351 根缓存写侧 `_rzr0/_rzr1` 同步换惰性调用——缓存行直引
`rmRoot262` 在惰性化后会缓存 null，致 R351 identity 校验假失败、rule①
调整丢失，为合并时发现的隐患）。R263 插入侧无冲突（上游预筛不需要
insRoot，本轮 `ensureInsRoot263` 闭包原样保留）。合并基线复测见 §8 附注。

## 8. A/B 结果（新二进制，探针 3 样本同二进制，guideline #35）

| 指标 | 修复前（同探针） | 修复后 3 样本 | 均值 | Δ |
|---|---:|---:|---:|---:|
| S4 appendChild×2000 | 1638 | 208.9/208.4/205.3 | 207.5ms | -87% |
| **S5 全链 set** | 2941-2984 | 318.0/315.2/311.0 | **314.7ms** | **-89%** |
| S7 S5 产物 cloneNode | 3376 | 362.3/355.4/364.6 | 360.8ms | -89% |
| S8 同 div 重设 | 5853-6429 | 832.3/832.4/822.5 | 829.1ms | -86% |
| p1b-set2000span | 1228-1280 | 221.4/215.7/212.2 | 216.4ms | -82% |
| p1-clone1（50 节点语料形） | 65.2 | 18.0/17.8/17.4 | 17.7ms | -73% |
| p3-append-text | 9606 | 268.2/273.2/264.2 | 268.5ms | -97% |
| p3t 纯跑 move×250 | 4553 | 121.4/125.6/123.4（q2） | 123.5ms | -97% |
| R155 反查 | 6022 次/0 中 | **0/0 ×3** | 0 | 消除 |

样本间极差 ≤2.3%（S5 311.0-318.0），测量稳定。对照 neuter 反事实基线
（§5 时代 R155/入口环双 neuter）：S5 335-338 / S8 875-884——修复后
全面优于 neuter 基线（n-ner 本身也省不掉调用开销）。

**验收线进度**：S5 314.7 vs ≤300（差 4.9%，余项 = 逐节点物化 glue
常数项）；S8 829 vs ≤1.5×S5=472（未达，remove 侧每子 ~250µs 为下轮
热点）；S7 与 S5 同量级 ✓。bold 四变体墙钟与全量回归见 §9。

### 8.1 合并基线（上表 = 本轮修复在旧基座；本节 = 叠加 R263b/R262b 后）

对 `fc12a1ff8`+`19b098b39` 基座 rebase 合并后首样本：S5 322.6 /
S8 854.1 / S7 361.3 / S4 212.9 / p1-clone1 18.1 / p3t-q2 132.6 / R155 0
——与本轮旧基座三样本均值同水平（+1-8% 噪声级，预筛门 R=1 时每 move
+~40µs 根游走为 p3t 微增主因），无叠加回退；对面 evidence 的 S5 残差
465-480 与本轮 322 的差额（~-145ms）即本轮 part04 CE 门 + trap 快道在
合并基座上的独立贡献。互补性成立：两流快道在探针面各自消除同款
O(n)/append，语料真页（注册表常驻数百条）里 R263b 预筛仍独立吃重。
