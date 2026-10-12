# performance-api-compat — 运行时控制面

> 状态头（每轮先核对再追加，依据日期：2026-10-13）

**当前状态**：M2-S1 实现完成（bench-gate 静机收口挂起，见卡点）；M2-S2
event-timing 实现完成（0/68 → 55/72 = 76.4%，余 17 跨域挂账）；M2-S3
navigation-timing 自洽面实现完成——**66/95 → 81/95（85.3%）**。实现：shim
`part01b` `PerformanceNavigationTiming` 接口（Illegal constructor + 全属性
toJSON + toStringTag）+ 每文档单实例条目（init 入 `_perfEntries`——getEntries*
立即可检索；数值属性 unavailable=0 与 timing L1 桩同口径，type 缺省 'navigate'，
真值归 navigation-compat/M3）+ name 访问时惰性 getter（main location 对象
part05 才装配，同文档生命周期内不变，po-navigation 断言 name ===
location.toString()）+ observe 钩子 nav 条目**两种形态无条件回放**
（po-navigation：entryTypes 无 buffered 也须收到——Chrome nav 条目
buffered-always 语义）+ legacy `performance.navigation` 四枚举常量
（TYPE_NAVIGATE/RELOAD/BACK_FORWARD/RESERVED）。余 14 失败全为真值/环境/net 面
（attributes-values 时间戳、redirect 链、content-encoding、host_info 多主机
URL——M3 挂账）。

**卡点**：bench-gate 静机收口未遂（第 7-11 轮）——环境争用定性证据链已闭合：
① 碰撞方进程实锤（渲染流 rally 多次在测量窗口内启动编译/reftest，loadavg
0.81→11.5 两分钟跳变）；② pre-M2-S2 树 A/B 当前环境同样 FAIL（load_html_simple
112µs > 预算 104µs）；③ 同指标跨分钟漂移 2.8x（395µs→142µs，真实回归不可能使
测量变快）；④ ms 级 page/* 指标全轮 PASS。结论：mb/* FAIL 为环境系统性，非
变更回归。**待渲染流 rally 收敛或深夜停摆窗口重跑，PASS 后关 M2-S1**（不使用
ZW_BENCH_ALLOW_BUSY=1；不做基线 relax）。

**下一步**：① 静机窗口重跑定向 bench-gate 关 M2-S1（唯一挂起点，不阻塞其余
面）；② M3 resource-timing 条目生成——需 net 装载钩子（host→JS entry buffer
桥），缺面分项见 M1 报告 §缺面 3；③ navigation-timing 真值面与 navigation-compat
流域协调（契约依赖条款）。

**M2-S3 门禁（2026-10-13）**：make test 全绿（71 targets；R2821 单测断言更新
——clearMarks+clearMeasures 后 buffer 余 navigation 条目，真实浏览器语义）；
fmt/clippy 零告警；兄弟 corpus 守卫：resource-timing 10/113 与 M1 基线逐分一致
（buffer-full 语义零回归）、performance-timeline 失败集 PRE/POST 完全一致
（po-observe/po-resource/droppedentriescount/navigation-id-initial-load 为既有
失败，非新增）、event-timing 修复面抽查绿、longtask 13/13。hr-time/user-timing
无预存基线行（归 timing goal 账本，§10 不越权改写），失败面（crossOriginIsolated/
coarsening、mark/measure 语义）与 nav 条目无交集——nav 条目只影响
'navigation' 类型查询与 observe 回放，类型作用域查询不受影响。

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
| M2-S1 longtask 条目生成 | **实现完成**；bench-gate 静机复跑后收口 | [M2-S1 基线](evidence/2026-10-12-m2s1-longtask-green.json) |
| M2-S2 event-timing 条目生成 | **实现完成**（55/72，余 17 跨域挂账） | [M2-S2 记分](evidence/2026-10-12-m2s2-event-timing-green.json) |
| M2-S3 navigation-timing 自洽面 | **实现完成**（81/95，余 14 真值/环境面） | [M2-S3 记分](evidence/2026-10-13-m2s3-navigation-timing-green.json) |
| M3 resource-timing host 管道 + navigation 真值 | 待启动（缺面分项见 M1 报告） | — |

## 基线账本

| 日期 | corpus | 子测试 Pass/Total | 备注 |
|---|---|---|---|
| 2026-10-12 | resource-timing | 10/113 | buffer 语义面自洽部分；条目生成缺面 |
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
