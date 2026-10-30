---
date: 2026-10-30
modules: wpt-runner, infra/ci
---

# WPT 语料跨目录依赖只在 fresh checkout 假失败，合法 JSON 不红任何 job

## 问题

coverage-2026-10-29 全量对账快照的 encoding 面从上一份的 361,578P 塌到
1,317P，全量头条通过率 58%→15.86%。塌了两天无人发现，靠人眼对账才定位。
期间 weekly.yml 分片 job 与聚合 job 全绿，快照照常回写 main——两道自动化
都没有报警。

## 根因

两层叠加：

1. **语料跨目录依赖只在干净环境暴露**。decode/encode 案面通过
   `../../resources/` 相对路径引用 `encoding/resources/decode-common.js`、
   `ranges.js`，而周任务语料 fetch（goals/31-encoding-legacy-mb.sh 的
   DIRS）没列这个目录。CI fresh checkout 下页面 404：decode 页零测试注册
   （tests:0 Timeout），encode 页 `rangesMisc is not defined` 页面级 Fail。
   本地不复现——goal-30 时代已把 encoding/resources fetch 过，目录里躺着
   旧文件，runner 两通道平扫叶目录时顺手命中。第一层修复
   （22b817e55，只补 /resources 与 /common 两文件）不够，两次 CI 运行失败
   模式逐字节一致，证明缺的正是整个 resources 目录。
2. **合法 JSON 的假失败不红任何 job**。runner 正常跑完退出 0，结果文件是
   合法 `--json`（Timeout/Fail 数字齐整）；分片 job 绿、聚合 job 正常合并、
   快照照常生成回写。绿 job 只证明"流水线没崩"，不证明"数字合理"。

## 解决与避免

- 修：3be29eb75 给 31-encoding-legacy-mb.sh 的 DIRS 补
  `encoding/resources`，并给 mb-decode/form-encode 分片配
  `script_timeout: 600000`（90s 默认对 encoding 全量面偏紧）。
- 防：wptfyi_gap.py coverage 增加环比护栏——与同目录最近一份既有快照按
  topdir 比对通过数，跌幅同时越过 ≥1000 与 ≥20% 双阈值即打 `::error::`
  注解、快照打回归旗标、以退出码 3 终止；weekly.yml 一行未改，快照回写
  步骤 `if: success()` 自动跳过，坏快照不覆盖上一份，artifact
  （`if: always()`）仍保留现场。确属预期时 `--no-guard` 重出。
- 通用结论：**只在干净环境暴露的失败，本地绿不能充当回归证据**；依赖清单
  （语料目录、vendor 路径）的完整性要靠"从零拉取"验证，不能靠"本地恰好有
  旧文件"背书。对账/统计类产物的自动回写链路必须配环比守卫——上游数据
  （wpt.fyi、语料、runner 行为）任何一层变化都可能把分子悄悄打掉。
