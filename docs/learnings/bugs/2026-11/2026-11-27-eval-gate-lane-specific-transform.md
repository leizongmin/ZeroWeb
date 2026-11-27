---
date: 2026-11-27
modules: engine,webview
---

# 运行时全局包装是车道无关的，文本级门变换是车道特定的——每条执行车道都要独立落点

## 问题描述

刀㊾（s10k50）把 TT eval 门从刀㉙的「运行时替换 `globalThis.eval`」重形为
「arg-transform：`eval(expr)` → `eval(__zwTtEvalArg(expr))`，语法 callee 保持
原生 eval＝直接 eval 语义保形」。首版只落了 part03 script prepare 钩（shim
车道），solo 单页组立刻系统性回归：目标页 0P/2F、eval 家族五页 15F——门完全
不触发，页面退未门控域照旧放行。

## 根因分析

两种门形的拦截位置不同：

- **运行时全局包装（刀㉙）**：装一次即替换全局绑定，之后**任何车道**执行的
  `eval(...)` 调用都经包装函数——车道无关。
- **arg-transform（刀㊾）**：文本级变换，只改写**经它过的那段文本**——车道
  特定。哪条车道执行这段代码，哪条车道就必须在执行前变换它。

而执行车道不止一条：zero-wpt-runner 从**原始 HTML** 抽脚本（`extract_page_scripts_sched`）
经 webview.rs Rust 侧车道直执行，根本不经过 part03 的 DOM prepare（刀㉙返修四
钉测已实证「装置钩子对语料页面永不触发」）；part03 prepare 只及 shim 车道
（动态插入、引擎钉测域）；桥接钉测直驱沙箱则两条都不经过。首版只落 shim
车道＝runner 全语料裸奔。

## 解决方案

门形从全局替换改为文本变换时，逐一盘点执行车道并各自落点：

1. **runner 车道**（webview.rs）：交付体求值前装置（armed 查询单独 execute，
   非 armed 页零额外传串）→ armed 才经沙箱回环 `__zwEvalArgTransform`；
2. **shim 车道**（part03 prepare）：MaybeInstall 之后同形钩；
3. **钉测道**（bridge pins 直驱沙箱）：片段先经变换再执行（语料字节流镜像）。

fail-safe 形：变换异常/空返一律交付原文（armed 页该文本退未门控域，FIXME
边界），保证误扫描最多漏点、永不破坏语法。宿主 header-face 装置点在变换点
之后（armed 迟到），该车道保留运行时包装形——同一门允许双形并存，以装置
时点划界。判断准则：改门形先问「这个门的拦截位置从运行时挪到了文本层，
还有哪条车道在执行没经变换的文本」。
