# WPT A/B 电池 CASE_WALL_S 截断窗制造假回归——Pass 集比对把「未跑到」记成 REGRESSED

## 问题

2026-11-20 刀 WAC 电池（css-masking 主靶 + transitions/animations 哨兵）原始对账出现
`css-animations/event-dispatch.tentative.html: P 2->0 REGRESSED=2`，而该页与刀面
（mask-border WAAPI composition 准入）无任何代码路径交集。

## 根因

该页在全部历史观测中均**页面级 Timeout**：leg 的 `CASE_WALL_S=60` 在页中途杀掉
harness，窗内跑到第几个子测随机器时序波动（观测窗内条目数 12~19）。ab-compare
按「子测名 → status」Pass 集做差：b0 窗恰达第 17 子测（含 2 个偶发 Pass），
b1 窗 12 条即断未到达——比较器无法区分「失败」与「没跑到」，把截断差记成回归。
同类陷阱：wpt-runner 语料 JSON 的子测 `status` 是字符串枚举（`"Pass"`/`"Fail"`/
`"Timeout"`），按整数（0=Pass）解析会静默得到全零 Pass 计数。

## 解决

- **仲裁法**：对可疑 REG 页所在语料，双臂提高 `CASE_WALL_S`（本次 240s）复跑，
  若 SET-IDENTICAL 即定谳窗口伪影，不转抄进收账。本次复跑 gained=0 lost=0
  regressed=0，电池终判回到 +168 零 lost。
- **判读纪律**：语料腿 rc 只留痕；门 = 计数==期望 + 对账零漂移双证之外，
  对每个 REG 追问「页是否页面级 Timeout/子测数两侧不等」，不等则先仲裁再收账。
  事件派发类长页（event-dispatch、逐 case 等真实定时器的页）是高危候选。
- **证据核对**：解析 JSON 前先 `python3 -I` 打印原始 `status` 值域再写判读脚本。

## 适用范围

`make reftest` / wpt-runner 语料腿、`.acceptance/wptfyi-css/ab-compare.py`
A/B 电池判读、任何「Pass 集差 = 回归」口径的工具——只要采样存在墙钟截断，
Pass 集差就必须先排除截断深度差。
