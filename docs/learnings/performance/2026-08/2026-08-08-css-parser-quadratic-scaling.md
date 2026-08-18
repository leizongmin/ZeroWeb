---
date: 2026-08-08
modules: zero-css-parser（tokenizer）, 性能门禁体系
---

# CSS 解析器规则数 O(n²) 超线性缩放（已修复，2026-08-08）

## 问题描述

性能门禁首跑时，`css_parse_by_size` 基准的 5000 规则档单次迭代耗时异常：criterion
warmup 至少 5 次迭代 + 5s 测量窗被拖到数十分钟，整个基准套件（79 个基准）预计 3+ 小时
无法完成。

## 根因分析

从 `target/criterion/**/new/sample.json` 原始样本（`times[i]/iters[i]` = 逐迭代耗时）提取：

| 规则数 | 单次迭代耗时 | 放大倍数 |
|---|---|---|
| 100 | ~5.8 ms | 1x |
| 500 | ~145 ms | 5x 规则 → **25x** 耗时 |
| 1000 | ~578 ms | 2x 规则 → **4x** 耗时 |
| 5000（外推） | ~14.5 s | 5x 规则 → 25x 耗时 |

`generate_base_css` 生成的是 5000 条同构规则（`.class-N { color: rgb(...); ... }`），
解析器对其耗时呈**二次方**缩放——疑似选择器/声明表内部线性查找（如按 selector 文本
线性去重/注册、或规则插入 O(n) 的数据结构）。**100 规则 5.8ms 本身也异常偏慢**
（~580µs/规则），说明存在常数因子很大的逐规则开销。

## 解决方案（本次落地）

- **测量侧**：criterion 显式收紧测量参数（`--warm-up-time 1 --measurement-time 3
  --sample-size 20 --noplot`，入 `bench-report.sh` 的 CRITERION_FLAGS 且计入 config_hash），
  全套基准从 3+ 小时降到 ~12 分钟。
- **根因定位**（2026-08-08 同日）：`tokenizer.rs` 的 `byte_offset()` 对**每个 token**
  执行 `source.char_indices().nth(pos)`——从字符串头重扫 pos 个字符（重复 UTF-8 解码），
  单次 O(pos)、总计 O(n²)。证据：`tokenizer_1000_rules` 纯分词 p95=583ms ≈
  `css_parse_by_size 1000` 全解析 p95=584ms——**整个 O(n²) 都在 tokenizer**；
  parser 层核实为 O(n) 摊还（无去重/注册/contains，前瞻有界）。
- **修复**：`Tokenizer` 新增 `byte_pos: usize` 字段，`consume()` 增量维护
  （`+= c.len_utf8()`），两处 `pos -= 1` 回退处成对减回退字符 UTF-8 长度；
  `byte_offset()` 改 O(1) 直读。`source` 字段随之删除（修复后无读取者）。
  所有 pos 变更点仅 3 处（grep 证实），无遗漏风险。

## 修复效果（perf-gate 实测，auto-tighten 已收紧基线）

| 基准 | 修复前 p95 | 修复后 p95 | 加速 |
|---|---|---|---|
| css_parse_100kb | 549.4 ms | 1.63 ms | ~337x |
| css_parse_by_size 100 | 6.2 ms | 237 µs | ~26x |
| css_parse_by_size 500 | 145.6 ms | 1.21 ms | ~120x |
| css_parse_by_size 1000 | 584.2 ms | 2.51 ms | ~232x |
| css_parse_by_size 5000 | 14.7 s | 13.2 ms | ~1115x |
| tokenizer_1000_rules | 583.5 ms | 0.99 ms | ~589x |

缩放从 O(n²)（100→1000 规则 6.2ms→584ms）变为线性（100→1000 规则
237µs→2.5ms）。全套基准时长随 5000 档从 ~5 分钟降到 ~1s。

## 如何避免

1. 新基准的输入规模要兼顾「覆盖真实量级」与「测量时间可控」——超线性算法的
   大档位会让 criterion 默认参数下测量时间失控。
2. 门禁体系第一次跑全量前，先抽查 1-2 个 crate 的单测耗时，确认量级再放全量。
3. 任何解析/匹配类代码，警惕按输入规模线性增长的数据结构里的 O(n) 查找
   （选择器去重、规则索引、样式表注册表）。
4. **每 token 调用的函数必须 O(1)**——从输入头重扫的模式（`char_indices().nth(pos)`、
   `chars().nth(i)`、`line_column_from_offset` 式线性扫描）在逐 token/逐字符循环里
   必然 O(n²)。需要偏移/行列号时增量维护或预计算，不要现扫。

## follow-up（未做，精准修改范围外）

- `line_column_from_offset`（tokenizer.rs）同为 O(offset)/次，但当前**无生产调用方**
  （仅测试用）——若未来在诊断/错误报告路径逐 token 使用，会重新引入 O(n²)；
  届时用「预计算行起始偏移表 + 二分」替代。后续优化优先在 perf-gate 指标上
  验证（csp_parse/ipc_deserialize 等 µs 级基准在共享机器上受另一条流 WPT 全量
  干扰，见 docs/specs/performance-and-resource-budget.md 负载守卫章节）。
