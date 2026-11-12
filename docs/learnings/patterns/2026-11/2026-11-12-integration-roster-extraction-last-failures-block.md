---
date: 2026-11-12
modules: site-optimizer verify, cargo libtest
---

# 集成 roster 提取算法——取最后一个 `^failures:$` 块到 `test result:` 行收口

## 问题描述

集成门禁 roster 比对（T64 起）：从 `cargo test -p <crate> --lib` 全量日志提取
失败键清单与基线恒一比对。首版按「看到 `failures:` 开块、`test result:` 收口」
的状态机（toggle）提取，日志里失败键数量与 libtest 自报 `N failed` 对不上，
且多轮日志拼接后取到陈旧块。

## 根因

1. libtest 的 `failures:` 块在一份日志里可出现多次：被中止轮次的部分块、
   测试自身 stdout 里含字面 `failures:`（如打印表格/示例）、以及最终块。
   toggle 状态机遇到中间块就开始收键。
2. 唯一稳定的锚点是**最后一个**行首精确 `failures:`（`^failures:$`）——
   libtest 终局块总在 `test result:` 汇总行之前，中间块后面必然还有输出。

## 解决

```python
starts = [i for i, l in enumerate(log) if l.strip() == 'failures:']
s = starts[-1]                      # 最后一个块，不是第一个
e = 下一个 ^test result: 行          # 块收口
keys = log[s+1:e] 去空行             # 失败键，每行一条
```

并用 `len(keys) == N failed`（`test result:` 行自报值）做提取完备性断言：
数量不符 = 提取器错或日志截断，不得进入比对。
