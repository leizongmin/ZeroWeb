# 刀 TEXT-INTERP-F2 判定笔（2026-11-22 05:5x，实施流）

## 判定 TOTAL：gained=5 lost=0 → P2F=0，**入账 +4**（+1 flake 域剔除）

## 电池档案

- unit `zw-knife-text-interp-f2`（service 模式 systemd-run），log
  `knife-text-interp-f2-battery-0520.log`；EXIT rc=0 05:46:53。
- face：`knife-text-interp-f2-face-draft.patch` md5 `29437f5f52adb26726f90815439dadd5`
  单文件双 hunk（parse.rs 解析臂＋钉测），草案直贴（patch-only 前向 apply 协议）；
  landed hunk 体与草案恒等（仅 @@ 函数上下文后缀差）；入库 f3001b563。
- bins：b0 `3f5a0bdb6c9eb48a0459908e1a83ef00`（1m07s）/ b1 `164b8589f1b1d53915ed761a0d628904`
  （55.8s）；M2 门过（md5 异）。
- FM 门偏差：face 无串字面量（注释不烙二进制）→ 以 patch md5 恒等门＋M2 双证替代
  binary FACE-MARKER 门（claim NOTES 偏差注记）。
- RIDE 1 文件＝s419（es_module.rs），S1/S2/S3 md5 恒等零扰动。
- 语料腿（CASE_WALL_S=60，六腿全 rc=0）：values 3m07s/2m48s、css-text 2m33s/2m31s、
  transitions 6m51s/6m46s。
- 快照：`.acceptance/wptfyi-css/snap-knife-text-interp-f2-0547/`（JSON/bin/ride 全套）。

## RECONCILE 逐页

- **主判据精确兑现**：`css-values/calc-rounds-to-integer.html P 64->68 (+4)`——
  4 案逐条 fail→pass：
  - `should accept "calc(1.1e1)"` / `"calc(10)"` / `"calc(10.1)"` / `"calc(1e1)"`
    （hyphenate-limit-chars CSS.supports 前置臂，TEXT-INTERP verdict 遗留 #1 恰集）
- `css-values/random-computed.tentative.html P 119->120 (+1)`——翻转案
  `Shared globally: random(--identifier, a, b)`（共享随机性等值断言），与 face
  零机制交集；该页 r3 对账即列「既有剔除域」（tentative＋random 域）——
  按「flake 域窗内双向皆不入账」剔除，不入 tally。
- css-text gained=0 lost=0、css-transitions gained=0 lost=0（哨兵零漂移；
  parse 单域变更下 css-text calc 形案位未现 F2P＝该语料无 calc 形声明案位）。
- ASYM：无。

## 机制与规范

`parse_hyphenate_limit_chars` 整数分量循环扩 calc() 形：
`values::parse_math_function` → `calc_expr_is_number` 门（长度/百分比叶即拒）→
`eval_calc(&expr, None)` 常量求值 → `v.round() as i64`（computed 时整数化）。
规范：css-values-4 §calc-type-checking（整数属性的 calc 值接受 <number> 计算式，
computed/used 时圆整）。复用既有 API（parse.rs mathfn 求值族），零新依赖。

## 入库与门禁

- 定向 fmt：rustfmt --check parse.rs（repo config）clean（apply.rs:6105 漂移为
  R5890 存量池域，非本刀）。
- 定向 clippy：`cargo clippy -p zero-style-system --all-targets` guard 包裹
  status=0 零发现。
- lei-pre-commit-guard：PASS（1 文件 50 行，0 发现 0 提醒）。
- 提交：face f3001b563（pathspec 单文件，HEAD blob 双标记核验）；
  docs 本笔。

## 遗留挂账更新（TEXT-INTERP verdict 四条现状）

1. ~~calc() 分量接受~~ → **本轮闭缺（+4 入账）**。
2. lenient 委托臂缺口 → **已随 fa89cbb31 落库闭缺**（animation.rs lenient fn
   hyphenate-limit-chars 委托臂）；CSSA/WAAPI sample 增益为测量欠账
   （css-animations 语料窗，b0 须 pre-fa89cbb31 bin 或 fset 对照），非 hunk 域。
3. inheritance 页余量断言（css-text 域，TEXT-INTERP-r2 解剖笔 lane 已开
   css-text 161F 四类普查——supports 门 11／setter 过松 8 等）。
4. M4 探针（hyphens/text-align-last 离散采样之谜）下轮参考。
