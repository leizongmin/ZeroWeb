---
date: 2026-11-06
modules: wpt-runner, testing
---

# runner 日志混入 NUL 字节后 grep 静默空输出——统计与 A/B 前先 Python 解析

## 问题

对 `make testharness-net-api` 产出的逐行结果日志做 `grep -c "^Pass"` 统计与
A/B 对比时，grep 对**确实包含大量 Pass 行的日志返回空输出**（连子串匹配
`grep -c "Pass"` 也是空），无报错、退出码不醒目，连续误判"运行没跑出结果"，
浪费两轮排查。

## 根因

runner 输出流中混入了单个 NUL 字节（`\x00`，来源是用例页面脚本的原始输出）。
GNU grep 对含 NUL 的输入按 **binary file** 处理：`-c`/普通匹配输出被抑制
（行为随版本表现为空或 `Binary file matches`），管道下游 `wc/sed/awk` 拿到
空输入，结果与"日志为空"无法区分。

## 解决

- 分析 runner 日志一律先经 Python 读字节再 decode（`errors='replace'`），
  用 `re.finditer(r'^(Pass|Fail|Timeout|Skip) (\S+) :: (.*)$', text, re.M)`
  提取——本仓 net-api 基线 A/B（M0/M1a/M1b 三轮，2026-11-06）即此法。
- 快速判别：`od -c file | tail` 或 `python3 -c "print(open(f,'rb').read().count(b'\x00'))"`。
- 若必须用 grep，加 `-a`（`--text`）强制按文本处理。

## 避免方式

凡对 WPT/runner/测试守卫产出的日志做统计、过滤或 A/B，默认假设可能含
二进制字节；先 Python/`grep -a`，不信"grep 空输出 = 日志为空"。
