# 会话碰头记录与看护交接（2026-10-21 03:0x）

本 session（延续轮）恢复时发现另一并行 session 在同一 clone（ZeroWeb-2）活跃推进
同一 svg-compat goal：已提交并推送 M1 基础设施 `7b9499062`（testharness-svg 通道 +
Makefile target + 深挖 fetch；内容与本 session 前一轮暂存的改动一致，pre-commit
guard 隔离扫描 PASS）、补 fetch 脚本头 skip 规则同源账（工作区未提交）、写
`2026-10-21-m1-baseline.md` 骨架（盘点数字与本 session 独立计算一致：2144 文件
= 1240 `.svg` + 397 无 harness html + 489 含 harness + 18 helper；渲染断言面
76.4%；可执行子集本地规则副本 486 案），并于 03:01 以 test-guard 包裹重启基线
（`/tmp/svg-m1-baseline.json`，14400s 墙钟）。

按 run-rules §9 碰头处理：本 session **退避为看护方**——不改共享面（evidence
骨架 / master.md / CSV / 脚本头账均归活跃 session 所有），仅监视基线完成。

因本环境 session 随查询结束被清理（跟踪式后台任务两轮随 session 终止被杀），
看护改为 detached 收尾分析器（已在本 session 接手分析后主动停用）。

## 基线 v1 定谳（03:01-03:17，EXIT=1 = 含失败正常语义；本 session 解析）

**486 案 1527 子测试：134 Pass / 1339 Fail / 36 Timeout = 8.8%**，全绿案 20/486。
runner 实跑集合与本地规则副本 486 案逐案一致（零静默误伤）。分目录 P/F/T/U：

types 70案 0/512/4（接口面全缺）、animations 275案 1%（SMIL）、path 29案 0%
（getTotalLength 面）、interact 20案 12%、geometry 13案 19%、styling 14案 40%、
historical.html 64/68（负向历史面，缺接口反通过）。top 失败簇与 M2 计划切片对位：
`.baseVal` undefined 267（S2 值类型）、getBBox 171（S4）、getTotalLength 80+（S4）、
setCurrentTime/pauseAnimations 48（S3）、SVGAnimatedEnumeration 19（S2）、
createSVGLength/Matrix 20（S3）。

## 活跃 session 收口进行时（本 session 彻底退避）

03:17 基线完成后，活跃 session 03:22 补拉 `wpt-data/resources/SVGAnimationTestCase-testharness.js`
（gitignored；animations 275 案的根级 helper——v1 中 createSVGElement/smil_async_test
`is not defined` ~185 fail 即此缺口）并将重跑基线（v2 数字必然优于 v1，以 v2 为准）。

## 剩余缺失 helper 清单（本 session 全量盘点，7 项）

`common/rendering-utils.js`（9 案，渲染断言面——rendering-compat 域，不补）、
`css/support/{interpolation,parsing,computed}-testcommon.js`（15 案，CSS 插值/解析面
——rendering-compat 域）、`resource-timing/resources/observe-entry.js`（1 案，
resource-timing 域）、`resources/testdriver{,-vendor}.js`（4 案，runner 有自备
testdriver 面，按既有各 goal 口径记账）。均不建议 svg-compat 域补拉。
- 追记（03:06，第二轮恢复）：会话终止反复连坐基线跑（前两轮均死于 teardown），
  本轮加 **detached 守护**（`/tmp/svg-m1-guardian.sh`，setsid 脱离会话生命周期）：
  当前跑干净完成则退出；中途被杀则同参数（test-guard 4G/8G/14400s + --json）自动
  重启。守候目标 = wrapper pid 1212834。诊断产物不入库（.acceptance/tmp 惯例）。
