---
date: 2026-10-29
modules: engine, tests/wpt-runner
---

# WPT encode-form 期望按 per-scalar fresh 编码出字节——整串流式编码对 stateful 编码必判 FAIL

## 问题描述

legacy-mb encode 面（form 提交编码 / href URL 序列化）的序列化语义初看是「把整串值按目标编码流式 encode 出字节再 percent-encode」。对无状态编码（euc-jp/big5/gbk/euc-kr/shift_jis/单字节）逐 scalar 与整串逐位相同，无行为差；但 ISO-2022-JP 这类 stateful 编码下，整串流式模型的产出与 WPT 期望结构不匹配，全部子测试 FAIL，且失败形态极具迷惑性（期望 `%1B%24BPP`，实际 `%1B%24BPP%1B%28B...`——多出来的 ESC 序列像「编码器 bug」，实际是模型选错）。

## 根因分析

WPT encode-form harness（`encoding/resources/encode-form-common.js`）的断言结构：

1. 期望值**逐码点独立预计算**：`encoder(String.fromCodePoint(i))` 产出空格分隔的字节 hex（`expect()` 转 `%XX`），每个 async_test 的期望只含本码点的字节序列。
2. 提交值 = 400 码点用 separator 拼接，回读 `location.search` 后按 `encodeURIComponent(separator)` split，逐段与逐码点期望相等。

要满足 split 后逐段相等，浏览器必须对**每个码点独立地**产出「该码点单独编码」的字节序列——Chrome 正是这么做的（wpt.fyi aligned stable c=[1000,1000]）。对 ISO-2022-JP：每个非 ASCII 码点产出完整 `ESC $ B + 双字节` fresh 序列（无跨码点模式驻留、无 end-of-queue flush 尾巴 `ESC ( B`）；ASCII 码点直接裸字节。整串流式编码则在每个 ASCII 码点前后插入 `ESC ( B`/`ESC $ B` 模式切换、条目结尾多 flush 尾巴——split 后每段都多出切换序列，判 FAIL。

规范层面自洽：HTML urlencoded serializer 的 percent-encode-after-encoding 作用于整个 name/value 串，WPT 的期望结构实际把「encode 的应用粒度」钉死在 per-scalar——这是上游测试与 Chrome 实现共同承认的事实模型，不是测试缺陷。

附：errors 变体的期望公式 `%26%23<codepoint>%3B`（无法映射码点 → HTML NCR `&#N;` 后按 percent-encode set 补编）同样按逐码点预计算，per-scalar 模型天然满足；encoding_rs 的 `Encoding::encode` 单字符调用即产出 NCR 字节。

## 解决方案

engine 的 host encode 桥（`crates/engine/src/js_dom_bridge/text_encoding.rs` `text_encode_percent`）按 per-scalar fresh 模型实现：逐 `char` 用 encoding_rs 独立 encode，ISO-2022-JP 剥除 per-scalar 输出尾部的 `ESC ( B` flush 伪迹（jis0208/katakana/roman 字节域均不含 0x1B，剥除条件精确）；replacement 与 UTF-16 系先按 get-an-output-encoding 归一为 UTF-8。落地后 form 面 550,053/550,054（唯一非 P 为既录 data: iframe Timeout 案）。

## 如何避免

- 在 encoding 域做序列化方向（encode）能力时，先读 harness 的期望计算方式再选 encode 粒度模型，不按「直觉的流式编码」实现。
- stateful 编码（ISO-2022-JP，未来若涉 HZ-GB 等）逐 scalar 输出会带 flush 尾巴；剥除条件必须按「编码产出的字节域不含 ESC」论证，不按样例枚举。
- 失败形态里成对出现的 ESC 序列（`%1B%24B` 开头 + `%1B%28B` 结尾）是「fresh vs 流式」模型差的指纹，先对模型再查编码表。
