# 本地 Chromium 作 getComputedStyle 序列化 oracle

日期：2026-08-05
相关模块：`zero-engine` getComputedStyle 序列化（`crates/engine/src/js_dom_bridge/computed_style.rs`）、reftest oracle 工具链

## 问题描述

`getComputedStyle` 的 CSS 属性序列化需要逐字节对齐 Chromium 输出（computed CSS 值正确性维度）。
长期判定「简写属性（outline/border/flex/columns/list-style/...）序列化需 Web chromium oracle，
本地不可验证」，导致这些简写被反复 defer（R2737-R2753 共 17 轮把「oracle-verifiable surface」
限定为 gradient/box-shadow/border-image-source，简写全 defer）。

## 根因

「需 Web oracle」是误判——本机已装 Chromium headless 二进制（`/usr/bin/chromium`，版本 150），
即项目 product-smoke/reftest 既定 oracle 工具链。getComputedStyle 的确切输出串可直接用它提取，
**无需联网**。前序结论把「无本地 oracle 文件」等同于「需 Web」，漏掉了「现场跑 chromium 提取」这条路。

## 解决方案：headless `--dump-dom` + 结果写 DOM

chromium headless 的 `console.log` **不被 `--dump-dom` 捕获**。正确做法：把待提取的
`getComputedStyle` 结果写进一个 DOM 元素（`<pre>`），再 `--dump-dom` 取序列化后的 HTML，
sed 剥标签即得纯文本结果。

```bash
# 1) HTML 里：遍历元素 × 属性，把结果拼成文本塞进 <pre id=result>
#    （key 点：用 textContent 写 DOM，不要用 console.log）
# 2) 跑 headless dump
chromium --headless --disable-gpu --no-sandbox --virtual-time-budget=3000 \
         --dump-dom /tmp/oracle/extract.html \
  | sed -n '/===ORACLE===/,/<\/pre>/p' \
  | sed 's/<[^>]*>//g; s/&quot;/"/g'   # 剥标签 + 反转义
```

要点：
- `--virtual-time-budget=3000` 让脚本（含 getComputedStyle 同步调用）跑完再 dump。
- 用 `getPropertyValue(prop)` 取串（比 `.camelCase` 更稳，覆盖未知属性返 '' 可跳过）。
- 一个 HTML 可批量覆盖多元素 × 多属性，一次 dump 拿全部 oracle 值。
- 提取出的确切串直接作为 TDD 测试的 `assert_eq!` 期望值——本地 oracle 验证闭环。

## 关键发现（land 7 项中 2 处真 diverge）

oracle 提取后才发现 ZeroWeb 既有的两处真实 diverge（前序被「需 Web」挡住未暴露）：
1. **outline-width**：旧误套 border-width 的 `border-style:none → used 0px` 规则，但
   Chromium 对 outline-width **不归零**（保留 computed medium→3px）。`outline-width` default
   应返 `"3px"` 不是 `"0px"`。
2. **flex-basis**：`flex: <number>` 省略 basis 时 spec §7.1.1 = `0%`（百分比），旧 expand_flex
   用 `"0"`（→`0px`）致 `flex: 1` flex-basis diverge（Chromium `"0%"`）。

另外确认了**易错的序列化顺序**（无 oracle 极易写反）：
- `outline` = `<color> <style> <width>`（color 在前）
- `border` = `<width> <style> <color>`（width 在前）——两者顺序**相反**！

## 如何避免

- 凡是「需对齐 Chromium 某确切输出」的序列化/解析正确性工作，**先用本机 chromium headless
  dump 提取 oracle**，再 TDD。不要因「没有 wpt oracle文件」就 defer。
- 同模式可用于：任意 CSS 属性 computed 值、CSSOM `rule.style` 序列化、`getBoundingClientRect`
  等布局输出——把结果写 DOM 再 dump。
- 注意 `--dump-dom` 不抓 console，结果必须落 DOM。
