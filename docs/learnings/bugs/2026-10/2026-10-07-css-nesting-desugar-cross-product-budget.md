---
date: 2026-10-07
modules: css-parser
---

# CSS 嵌套 desugar 的父×子选择器叉乘需设预算——病态输入指数放大

## 问题描述

`fuzz_css_parser` 连续三周在 fuzz.yml 周跑中超时（libFuzzer exit 70，
`timeout-e33b1829…`）：280 字节输入使 `Parser::parse_stylesheet` 在本地 debug 档
耗时 26s+、内存 35GB（fuzz CI 5s 超时即崩）。误判过两轮：先以为是 NUL 转义死循环，
后以为是最小化输入里的反斜杠模式。

## 根因分析

`desugar_selectors` 对嵌套规则做父级×子级选择器叉乘（无 `&` 的嵌套 = `父级 后代
本选择器` 逐个 `prepend_descendant`，含 `&` 则逐父级 `substitute_amp`）。嵌套链每层
的输出作为下一层的父级——层宽 m、链深 d 时结果 m^d 指数放大，且每个结果整链 clone。
fuzz 输入的 NUL 噪声让逗号分隔的病态选择器大多合法存活（`&.`、`.` 均可解析），凑出
5-6 层 × 宽列表的组合。定位手段：`ZW_CSS_NESTING=0` A/B（开 271ms 指数 / 关 0ms）
一击锁定嵌套路径；tokenizer 单测先行排除词法层（全部微秒级）。

## 解决方案

`MAX_DESUGARED_SELECTORS = 1024` 预算：`desugar_selectors` 每轮 sel 处理后检查
`out.len()`，超限**整条规则丢弃**（返回空列表；`compile_parsed_style_rule` 对空
`own` 本就丢弃规则，子嵌套拿空父级连锁为空）。部分截断无规范语义故不做；预算检查放
循环末（而非每处 push）使"父级 ≤ 预算"的归纳界成立。css-nesting-1 Security
Considerations 明确允许实现对此类组合设上限。

经验：解析器的组合性展开（叉乘/笛卡尔积/递归 materialize）必须有输入无关的上界；
fuzz 崩溃工件先做"tokenizer 与 parser 分层计时 + 语义 kill-switch A/B"再读代码，
比盲跑最小化器快且安全（本次最小化曾把机器拖到 OOM，见仓内无人值守运行安全准则）。
