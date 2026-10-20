# performance-api-compat — 运行时控制面

> 状态头（收口快照，2026-10-13——goal **Done**，DC-1~3 全 ✅，控制面留档备查）

**收口快照**：可执行面 **191/293（65.2%）**（M1 基线 77/293 = 26.3%）——
longtask **13/13（100%）**、navigation-timing **81/95（85.3%）**、event-timing
**55/72（76.4%）**、resource-timing **42/113（37.2%）**。门禁：make test 全绿
（71 targets）、reftest 零回归（failed 0.0%）、product-smoke PASS、bench-gate
PASS 35/35。目标文档状态已置 Done；余量挂账带数字分型见下方 §下一步。

**当前状态**：M2 三 slice 全部实现完成（longtask 13/13、event-timing 55/72、
navigation-timing 81/95；M2-S1 的 bench-gate 静机收口已完成——2026-10-13，
见下文收口记录）；M3-S1
resource-timing 条目生成完成（10/113 → 35/113）；M3-S2 resource net 元数据桥
完成——**35/113 → 42/113（37.2%）**。实现：`_makeResponseFromWire` 挂
`__zwRtMeta`（响应对象携带，无全局态）——encodedBodySize = content-length
优先（非消费读，避免吞掉可见 header 破坏 net-api 面）/ TextEncoder body 字节，
decodedBodySize = encoded（本地服务无内容编码），transferSize ≈ headersWire
线长 + body 字节，nextHopProtocol 'http/1.1'（runner 内部服务管道协议语义）；
`__zwEmitResourceTiming` 增 meta 参数，fetch 包裹/sync XHR 经响应对象传入；
失败响应无 meta → size 0（spec）。**架构发现**：runner 语境下宿主派发口被
R387b 页面 fetch 通道整体旁路（探针实证零调用）——runner 条目生成走 fetch
包裹通道；宿主派发口包裹服务 renderer/浏览器路径。

**卡点**：无。**M2-S1 bench-gate 静机收口完成（2026-10-13 03:07）**——定向跑
（zero-script-sandbox,zero-webview）GATE PASS，35/35 指标全在预算内、exit 0、
非 suspect（渲染流停摆窗口 loadavg ~1），报告
`tests/benchmarks/results/benchmark_20261013_030742.json`。此前第 7-11 轮 FAIL
的争用定性证据链（碰撞方进程实锤 / pre-change 树 A/B 同 FAIL / 同指标跨分钟
漂移 2.8x / ms 级 page/* 全 PASS）保留如下备查：渲染流 rally 多次压测量窗口
（loadavg 0.81→11.5 两分钟跳变）；pre-M2-S2 树 A/B 当前环境同样 FAIL；同指标
跨分钟漂移 2.8x（395µs→142µs）；ms 级 page/* 始终 PASS——环境系统性而非变更
回归，与本次 PASS 结论互洽。本次测量树含 M2-S1～M3-S2 全部 shim 变更（longtask
观察器路径 + webview shim 装载/派发路径均在其覆盖面内）——五个 slice 的性能
门禁一并闭环。

**下一步**：无——goal Done，收口归档已完成（2026-10-13，DC-1~3 逐项核对 +
终态树复跑 191/293 恒等见 [evidence/2026-10-13-closure.md](evidence/2026-10-13-closure.md)）。
后续无里程碑；M3-S3 loader 面经归属核对**定性为跨域挂账**，
不在本 goal 动工，重入入口如下：
- **img 面**：Error.stack 探针定位 onload 派发链 = part05 R56h 页面 fetch 通道
  （事件已派、条目已产、initiatorType 已归正 'img'——5f5b7b8e5）；剩余失败为
  事件**序列**语义（image-sequence-of-events）与 iframe 案（多 frame 结构性
  skip 面）——img 域。
- **link/stylesheet 面**：runner 进内路径无 link 完成信号（`.sheet` 惰性构建、
  无外部 CSS 装载通道；async_load 的 take_link_element_events 仅 AsyncPageLoad
  路径消费）——补通道属 webview/html-semantics 域基建，本 goal 不认领。
- **net 真值余量**：secureConnectionStart/redirect 链/TAO——需真实网络栈遥测。
- **环境面**：{{host}} 多主机模板 URL。
- navigation-timing 真值：归 navigation-compat 流域（契约依赖条款）。

**已知缺口（M3-S1 记账）**：buffer-full-add-entries-during-callback 期望 3 得 2
——callback 内 sync XHR 条目的事件循环交错（同步入账 vs 延迟入账在
microtask/task 序上不可兼得，三轮建模取 12/13 面收购点）。

**DC-3 门禁闭环（2026-10-13 凌晨，静机窗口）**：make reftest **零回归**
（不一致 failed = 0.0%——551 真通过 + 47 近似通过 + Layout 506/506 + Text
203/203）；product-smoke PASS（exit 0，struct-check 0 issues）；make test 71
targets 全绿（每轮提交前）。DC-3 三项（make test 全绿 / reftest 零回归 /
product-smoke 恒值）全部实证。

**M3-S2 门禁（2026-10-13）**：make test 全绿（71 targets）；resource corpus
42/113（nextHopProtocol 4 桶清零 + invariants/test_resource_timing 面 +3）。

**M3-S1 门禁（2026-10-13）**：make test 全绿（71 targets）；兄弟零回归——
user-timing 561/570 与 M2-S3 轮逐分一致、performance-timeline po-resource 由
既有失败转 PASS（resource 条目使观察面成立，正向解锁）、buffered-flag 家族
全绿。buffer-full 模型三轮迭代：满即 fire（误）→ 旗标+次级队列+drain 快照
（10/13）→ 纯余量+episode fire+sync 延迟（12/13 定稿）。

**M2-S2 门禁（2026-10-12/13 收尾）**：make test 全绿（71 targets，0 failed）；
shim JS 拼接语法校验过；uievents/ 子域定向守卫 86.0%（191/222，落在
uievents-compat completed 基线带 87.1%，失败面为 boundary-events/layerX/
accesskey-hang 既有缺口，非派发机制回归）；pointerevents 全量守卫三度尝试
均未产出（40/60 分钟墙钟 + 进程消失，基础设施问题不归因变更）——以 make test
71 targets + event-timing corpus 本体（新代码最深消费面）+ uievents 子域
三重证据替代。

## 里程碑

| 里程碑 | 状态 | 证据 |
|---|---|---|
| M1 导入 + 甄别 + 基线 | **完成**（2026-10-12） | [M1 报告](evidence/2026-10-12-m1-scope-baseline.md) |
| M2-S1 longtask 条目生成 | **完成**（bench-gate PASS 收口，2026-10-13） | [M2-S1 基线](evidence/2026-10-12-m2s1-longtask-green.json)；benchmark_20261013_030742.json |
| M2-S2 event-timing 条目生成 | **实现完成**（55/72，余 17 跨域挂账） | [M2-S2 记分](evidence/2026-10-12-m2s2-event-timing-green.json) |
| M2-S3 navigation-timing 自洽面 | **实现完成**（81/95，余 14 真值/环境面） | [M2-S3 记分](evidence/2026-10-13-m2s3-navigation-timing-green.json) |
| M3-S1 resource-timing 条目生成 | **实现完成**（35/113） | [M3-S1 记分](evidence/2026-10-13-m3s1-resource-timing-green.json) |
| M3-S2 resource net 元数据桥 | **实现完成**（42/113） | [M3-S2 记分](evidence/2026-10-13-m3s2-resource-timing-meta.json) |
| M3-S3 loader 面 | **定性为跨域挂账**（img 域/webview 基建/net 真值/环境面——见状态头 §下一步） | Error.stack 探针 + reftest/product-smoke 门禁记录 |
| 收口归档 | **完成**（2026-10-13） | [收口归档记录](evidence/2026-10-13-closure.md)——DC-1~3 逐项核对 + 终态树复跑 191/293 恒等 |

## 基线账本

| 日期 | corpus | 子测试 Pass/Total | 备注 |
|---|---|---|---|
| 2026-10-12 | resource-timing | 10/113 | buffer 语义面自洽部分；条目生成缺面 |
| 2026-10-13 | resource-timing | **35/113** | M3-S1 后（三通道条目 + buffer 机制；buffer-full 族 12/13） |
| 2026-10-12 | navigation-timing | 66/95 | attributes-exist 自洽面为主；条目真值缺面 |
| 2026-10-13 | navigation-timing | **81/95** | M2-S3 后（条目 + 双形态 observe 回放 + 枚举常量） |
| 2026-10-12 | event-timing | 0/68 | PerformanceEventTiming 全缺 |
| 2026-10-12 | event-timing | **55/72** | M2-S2 后（分母 68→72：assert_implements 放行暴露遮蔽子测试） |
| 2026-10-12 | longtask-timing | 1/17 | PerformanceLongTaskTiming 全缺（supportedEntryTypes 例外过） |
| 2026-10-12 | longtask-timing | **13/13** | M2-S1 后全绿（containerTypes 归多 frame skip 面，可执行分母 9 案） |

## 跨流记账

- **event-timing 余 17 失败跨域挂账（2026-10-12，M2-S2 收尾定性）**：webdriver
  键位/Actions 缺口（SPACE U+E00D、Actions keyDown/addTick、fling/selection 手势
  ——keyboard goal R3254 / webdriver 域）×6；shadow DOM 派发管道（event-retarget /
  first-input-shadow-dom / shadow-dom-null-target）×3；events 域连击链不按位置
  重置（dblclick eventCounts 3≠2，UI Events §5.2.2 chain reset）×1；label 激活
  转发 hang ×1；runner harness 版本缺 `window_state_context` ×1；tentative API
  （targetSelector）×5 子测试。均非 event-timing 语义面，本 goal 不认领。
- **wpt-suites.csv 共享面修复（2026-10-12）**：cookies×2 + webmessaging×1 数据行
  含未转义半角逗号（17/21/17 列），`generate-website-metrics.py` 15 列校验硬失败
  ——本轮以「整字段加引号」机械修复（内容逐字保留，已程序化核对），共享管道恢复。
  属碰头信号记录，内容归各自流。
- navigation-timing 生命周期真值与 navigation-compat 流域共享（契约依赖约束条款）；
  本 goal 只认领 PerformanceNavigationTiming 条目面自洽语义。
- **web-animations 计数漂移（2026-10-12）**：timing-animation 归档账本记 52/963
  （2026-09-28），当前 main 实测 502/952——渲染流域两周工作的组合态涨数，本 goal
  不认领、不改写彼账本（归因纪律 run-rules §10）。
