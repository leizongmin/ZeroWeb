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
