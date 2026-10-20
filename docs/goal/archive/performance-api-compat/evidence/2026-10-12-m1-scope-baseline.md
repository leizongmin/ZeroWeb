# M1 语料导入 + 可执行面甄别 + 基线（2026-10-12）

WPT_REV pin `315976933870b34d6ea30e3f6643403edae678ba`（goals/lib.sh 统一 pin）。

## 语料盘点

WPT 无 `performance-api` 顶层目录（curl 列目录 404 实测，pin 顶层 295 目录无此名）——
wpt.fyi 的 performance-api 标签对应四个独立顶层套件，与契约 Mission 四件套一一对应。
`tests/wpt-runner/scripts/goals/19-performance-api-compat.sh` 的 `DIRS` 已按此修正
（原 `( "performance-api" )` 拉不到任何语料）。

| corpus | 顶层案面 | 备注 |
|---|---|---|
| resource-timing | 97（.html 58 + .any.js 39） | +resources/ 支持资产 88 文件 |
| navigation-timing | 59（.html 58 + .window.js 1） | +resources/ 15 |
| event-timing | 72（.html 72） | +resources/ 2 |
| longtask-timing | 20（.html 14 + .window.js 6） | +resources/ 6 |
| timing-entrytypes-registry | 0 案 | 纯支持资产（registry.any.js + resources/utils.js），供四件套引用 |

合计 248 案面 + 111 支持文件落 `tests/wpt-runner/wpt-data/`（gitignored）。

## 可执行面甄别（runner 通道）

通道：`zero-wpt-runner` 新增四子命令（`testharness-resource-timing` /
`testharness-navigation-timing` / `testharness-event-timing` /
`testharness-longtask-timing`），复用 timing 四 corpus 扫描器（`run_timing_subdirs`
同构）：`.html` 直跑，`.any.js`/`.window.js` 以 window 变体执行。Makefile 四 target
（`testharness-*`，test-guard 包裹）。skip 规则见
`tests/wpt-runner/src/testharness.rs` `performance_api_case_skipped`。

结构性不可执行面（skip，不计入分母）：

| 规则 | resource | navigation | event | longtask | 理由 |
|---|---|---|---|---|---|
| manual/ref/idlharness | 1 | 1 | 3 | 1 | 真实交互 / 参照页 / WebIDLParser build 期资产 |
| iframe | 12 | 32 | 1 | 7 | runner 无多 frame 文档管道 |
| worker | 2 | 0 | 0 | 0 | runner 无 worker 文档管道 |
| .py server handler | 36 | 1 | 0 | 2 | wptserve 动态端点，静态语料不可执行 |
| dispatcher 跨窗会话 | 0 | 1 | 0 | 0 | bfcache/navigationId helper 机制 |
| CORS 整形 / web-bundle / 字典 | 3 | 0 | 0 | 0 | custom-cors-response / 未拉取支持目录 |

可执行分母：resource 43 + navigation 24 + event 68 + longtask 10 = **145 案**。
语义缺面（无条目生成管道）**不在 skip 之列**——用例照跑按真实 Fail 记账，这正是
基线要度量的对象。

## 基线（2026-10-12，首跑）

| corpus | 案 | 子测试 | Pass | Fail | Timeout | NotRun | 通过率 |
|---|---|---|---|---|---|---|---|
| resource-timing | 43 | 113 | 10 | 87 | 16 | 0 | **8.8%** |
| navigation-timing | 24 | 95 | 66 | 12 | 12 | 5 | **69.5%** |
| event-timing | 68 | 68 | 0 | 65 | 3 | 0 | **0.0%** |
| longtask-timing | 10 | 17 | 1 | 15 | 1 | 0 | **5.9%** |
| 合计 | 145 | 293 | 77 | 179 | 32 | 5 | **26.3%** |

逐案 JSON（`--json` 捕获）：同目录 `2026-10-12-m1-baseline-{corpus}.json`。

navigation-timing 69.5% 的构成：大量用例是「window.performance is defined」级
自洽断言（`test-*-attributes-exist` 族），不依赖导航条目真值即可通过；属性值
断言（`nav2-test-attributes-values` 等）按真实缺面 Fail。

## 缺面甄别（host 管道四缺口，M2/M3 挂账依据）

1. **longtask-timing（M2 首攻）**：无 `PerformanceLongTaskTiming`（`supportedEntryTypes`
   含 'longtask' 但永无条目——`assert_implements` 全灭）。判定管道本质是**任务循环
   计时**：shim 侧 setTimeout/raf/microtask checkpoint 均为 JS 层可控点，JS 层任务
   包裹（>50ms 判定 + entry 生成）无需 host 管道改造即可打通主案面
   （longtask-in-raf / longtask-promise / buffered-flag / longtask-attributes）。
2. **event-timing（M2 同批）**：无 `PerformanceEventTiming`——事件派发无
   startTime/processingStart/processingEnd 计时点，interactionId 未定义。shim 事件
   派发为同步 JS 路径，包裹 dispatchEvent 入口可在 shim 层产条目；0% 基线意味着
   全部 68 案是 M2 的直接增量面。
3. **resource-timing（M3）**：无 `PerformanceResourceTiming` 条目生成——资源装载
   （script/img/css）走 host fetch 桥，条目生成需 host→JS entry buffer 钩子；
   transferSize/nextHopProtocol/redirectTime 等真值需 net 层数据（挂账分项）。
   buffer 语义面（buffer-full-* 族 10 Pass）部分已有——getEntries 面自洽。
4. **navigation-timing（M3）**：无 `PerformanceNavigationTiming` 条目——导航生命周期
   真值挂 navigation-compat 流域；「attributes-exist」自洽面可用诚实缺省值条目
   （unavailable=0）先打通。

## M1 通道产物

- `tests/wpt-runner/scripts/goals/19-performance-api-compat.sh`：DIRS 修正 + registry 支持目录
- `tests/wpt-runner/src/testharness.rs`：四 corpus SUBDIRS + `performance_api_case_skipped` + 扫描器
- `tests/wpt-runner/src/main.rs`：四子命令
- `Makefile`：`fetch-wpt-performance-api` + 四 testharness target
