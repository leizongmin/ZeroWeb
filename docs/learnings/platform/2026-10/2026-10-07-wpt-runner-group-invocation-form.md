---
date: 2026-10-07
modules: tests/wpt-runner
---

# zero-wpt-runner 组级过滤的正确调用形态：testharness-dom + 路径，不是 `run <组名>`

## 问题

对 WPT named-access 组做回归复验时，直觉调用：

```
cargo run --release --bin zero-wpt-runner -- run named-access
```

日志显示命令"成功"结束，但结果为 **0 条匹配**（0P/0F/0T）——空转。两轮（postrebase、postmerge）都以空转日志收场，若只看 exit code 会误判为"通过"。实际该组基线为 43P/1F/2T。

## 根因

`zero-wpt-runner` 的子命令语义：

- `run <过滤词>` 在 **curated 常驻断言集**（约 1343 条）里按词匹配——`named-access` 这个词不命中任何 curated 条目，于是 0 匹配静默返回。
- 上游 WPT 原始组（如 `html/browsers/the-window-object/named-access-on-the-window-object/`）要走 **`testharness-dom <组内路径>`** 形态，按目录路径加载并执行 testharness 用例。

两个入口面向不同测试资产（curated 导入集 vs 上游原始组），过滤词互相不可替代。

## 解决

组级回归复验用：

```
cargo run --release --bin zero-wpt-runner -- testharness-dom browsers/the-window-object/named-access-on-the-window-object
```

结果按 Pass/Fail/Timeout 逐条列出，与基线（如 43P/1F/2T）逐条同位比对。

## 如何避免

- 跑 WPT 回归前先确认目标用例在哪个资产层：curated 集（`run` + 过滤词）还是上游原始组（`testharness-dom` + 路径）。
- **exit 0 不等于有覆盖**：0 匹配空转也是 exit 0。判定前先看结果的 P/F/T 计数是否与预期基线同量级，0/0/0 一律视为调用形态错误。
- 空转日志留痕后应立即用正确形态重跑，不要把空转日志当通过证据归档（本例两轮空转日志均留痕并重跑纠正）。
