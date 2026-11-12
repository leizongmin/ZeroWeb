---
date: 2026-11-12
modules: tooling,testing
---

# 门禁批 judge 伪绿两形：页滤空转 rows=0 与 grep 捕获 test result 摘要行

## 问题描述

同日门禁批两处脚本伪象，都把「实际未验证」伪装成「绿」：

1. **judge 空转伪绿**：批内 judge 以页子串 `"run/multitest-ab.html?"` 过滤日志行，但 ab 语料日志行实际形如 `Pass editing/run/multitest-ab.html :: <key>`（页名后无 `?`；mt 语料才是 `multitest.html?1-1000` 分页形态）。过滤命中 0 行 → base/post 双侧 rows=0 → 集合运算下 P2F/lost/po_fail 全空 → judge 判 OK。腿计数（10092/0/0）真实，但「与基线逐行恒等」这一断言实际未执行。
2. **名册提取吞摘要行**：`grep '^test .* FAILED'` 除逐名失败行外还命中 cargo 汇总行 `test result: FAILED. 3905 passed; ...`（前缀同为 `test `），提取名单混入第 8 行垃圾 → 逐名 diff 伪失配 ABORT（反方向伪象：真绿报红）。

## 根因分析

- 两形共同根因：**断言链里没有「过滤器确实命中了东西」这一环**。judge 的比较逻辑只在命中集合上运算，空集是合法输入但不是合法验证；grep 提取的名单直接进 diff，多捕一行只是名单变长，单看提取输出无法发现。
- 页滤串从旧脚本（mt 形态）复制到新语料（ab 形态）时未对照实际行格式复核；`FAILED` 模式只想到逐名行忘了 cargo 还有同前缀汇总行。

## 解决方案

- judge 在比较前先断言 **rows > 0 且 base/post 行数与腿计数一致**（`len(base) == P+F`），空集直接 fail：「没比上」与「比上且恒等」必须是两种输出。
- 名册提取用全尾缀锚定：`grep '^test .* \.\.\. FAILED'`（`...` 是 cargo 逐名行独有），或提取后 `wc -l` 对账 failed 计数。
- 复用旧 judge/提取脚本时，先 `grep -m1` 目标日志核对行格式再落脚本；腿计数判读（P/F/T）与逐行 judge 是两条独立证据，前者绿不能替后者背书（本例两条都绿时 judge 仍可能空转）。

## 案例

2026-11-12 unwrap R5450 收口：gate v4 L6 逐名 diff 伪 ABORT（形 2，离线修正重提取闭合）；换基重锚批 B1 ab judge rows 0->0 空转（形 1，离线修正页滤 `editing/run/multitest-ab.html` 重判 10092 rows 恒等）。两处都是腿计数绿、judge 层伪象，离线重判才拿到真证据。
