# 组成态巡检二跑 — HEAD 全新重建零漂移（2026-10-20）

f1032b7c2 组成态下 `make testharness-navigation TIME_LIMIT=2700` 复跑（runner 依赖链
全新 release 重建 36.76s，非复用旧二进制），全量对
[final 账本](2026-10-20-composition-recheck-final.txt) 状态行集合（空白归一 + 排序）
**逐行恒等**：409P/35F/26T/14NR，零漂移。make exit 1 = the-iframe-element sandbox
双案慢性抛错（定性见 [recheck](2026-10-20-composition-recheck.md)），不作组成态门禁。

- 兑现 [A/B 陈旧二进制 learning](../../../../learnings/bugs/2026-10/2026-10-11-ab-baseline-rebuild-stale-release-binary.md)
  的反向纪律：本账本由 cargo 指纹全新重建的二进制产出，排除旧 release 复用干扰。
- 非 Pass 75 案逐案与 master.md 挂账核对一致（M3 iframe/bfcache 载体 47、DOM 焦点
  16、渲染域 6、runner 形态/任务模型 6），无新可切片项，M2 收口态维持。
- 并发记录：跑测期间同 clone 有 `testharness-html-semantics` 长跑（11:14 起、
  time-limit 10800，资源余量充足），结果不受影响（逐行恒等即证）。

原始输出：[2026-10-20-composition-patrol.txt](2026-10-20-composition-patrol.txt)。
