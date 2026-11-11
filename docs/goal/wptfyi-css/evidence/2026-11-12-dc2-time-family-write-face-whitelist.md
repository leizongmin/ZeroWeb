# DC-2 g11：<time> 族写面文法门（validate_decl_value 白名单扩 4 属性）

日期：2026-11-12（凌晨批）
基线：HEAD `c4bce8a46`（g10 time 维度批入账后）。

## 勘根

minmax-time-invalid.html 52F（"should not set" 全失败 = 非法形被接受）三面勘定：

1. 宿主侧 css-parser `parse_time` 对全部 52 形**本就正确拒绝**（g10 批已落：
   畸形参 parse None + `calc_expr_is_pure_time` 拒裸 Number/Length 叶）——
   `min()`/`min(1s, )`→None、`min(0)`/`min(0px)`/`min(1s, 0)`→purity false。
2. 语料写臂是 shim inline style 面（part03 `setProp`），T54 写面校验经
   `__zw_validate_decl_value` 委托宿主 `validate_decl_value`——白名单
   `["transform","rotate","scale","translate","z-index","margin-left"]`
   **不含 <time> 族** → 放行原文透传落库，宿主拒收逻辑不可达。
3. apply 侧裁决四臂**早已存在**（apply_advanced:710/724/812/826：
   transition-duration/delay、animation-duration/delay——`parse_time_list`/
   `parse_non_negative_time_list` + `split_top_level_commas` 括号深度感知，
   负 delay 合法、duration 非负），仅白名单未接入。

## 结果

- **css-values 语料：2,516 → 2,572，净 +56，逐页零回归**（0 页失 pass；
  F 3,077 → 3,021）。
  - minmax-time-invalid.html：**0P/52F → 52P/0F 单页全清**（语法桶 24 + 类型
    桶 28）。
  - percentage-without-context.html：+4——`calc(sign(50%) * 1s)` 四属性同值
    家族（animation-duration/delay、transition-duration/delay 各 1）写面正确
    拒收（percentage 无上下文 spec invalid），"should not set" 全翻。

## 改动面

- `crates/style-system/src/property/apply.rs`：`VALIDATED_PROPERTIES` 6→10
  （+transition-delay/transition-duration/animation-delay/animation-duration）。
  零 apply 逻辑改动——四臂既存，白名单接入即启用。
- **g11 语料跑捕获一处回归并同批修正**：css-animations 对 v3 基线逐页 diff
  唯一退化页 `animation-duration-computed.html` 5→4（`Resolved value of
  animation-duration:auto with animation-timeline:auto`）——根因 = css-animations-2
  §propdef-animation-duration `<time># | auto#` 的 `auto` 关键字（csswg#6530
  滚动驱动动画时长）被白名单 apply 裁决拒收（`parse_animation_duration` 纯
  plain 形不识 `auto`），写面丢弃后 resolved 值回落初值。修正：`animation-duration`
  逐 top-level 逗号项校验（`apply_advanced::split_top_level_commas` 提升为
  pub(crate) 复用），`auto` 项放行、其余项走原裁决；delay 与 transition-duration
  无 auto 关键字照拒。shorthand `auto, 1s` 混合形同步放行（pin 案在测）。
- `t54_validate_decl_value_tests` +3 钉：52F 代表集拒收（语法桶 + 类型桶）、
  合法形放行（plain/ms/负 delay/min/max/round/calc/`<time>#` 逗号列表 + auto
  三案 + 无 auto 关键字两案拒）、var()/未实现函数逃逸门。
- 已知残余（本批不扩，白名单使其写面可见、无语料页驱动）：`parse_animation_duration`
  纯 plain 形（数学函数形 duration 槽拒收——spec 允许数学形，minmax-time 语料
  页仅写 transition-delay，动画页无数学形 duration，挂账另刀）。

## 门禁

- style-system lib：`cargo test -p zero-style-system` **2,515P/0F**（含本批
  +3 钉：时间族畸形拒收代表集 / 合法形放行含 auto 案 / var 与未实现函数逃逸
  门；t54 模块 7P）；clippy `cargo clippy -p zero-style-system` **零告警**。
- 语料两轮（g11 初跑 → 发现回归 → 修正 → g11b 终跑；均为 systemd scope +
  test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 3540`，
  transcript/results 唯一目录 results-g11、results-g11b，全部阶段 exit=0）：
  - **css-values（主账）**：2,516 → 2,572 净 +56 逐页零回归；g11b 修正后
    复跑对 g11 **逐页 byte-identical**（253 页 0 diff——auto 刀对 values 面
    零扰动旁证）。
  - **css-transitions**：对 results-v3 陈旧基线（185P，g4..g10 累积漂移期内）
    **185 → 198 净 +13，页级与 test 级零回归**（g11b 117/118 页——
    inherit-height-transition 本轮 runner 时窗未落账，g11 同页 0P 无 pass
    流失）——其中 transition-delay-invalid.html 0→5 与
    transition-duration-invalid.html 0→5 为本批白名单直接翻账（"should not
    set" 写面拒收生效），currentcolor-animation +1 / events +2 为期内其他批
    累积漂移。F 1,751 → 2,304 增量为语料页新增子测（runner 子测抽取随批次
    增长），非既有 pass 流失（test 级 losses=0）。
  - **css-animations**：对 v3 基线 **225 → 236 净 +11，页级与 test 级零回归**
    ——animation-delay-invalid 0→5、animation-duration-invalid 0→6 为白名单
    直接翻账；g11 初跑 animation-duration-computed 5→4（auto 关键字拒收回归）
    经修正后 g11b 复位 5→5，test 级 losses=0。
- make test 全量本批有意推迟：同树他流 color WIP（part11* 未提交 hunks）会
  污染归属；本批面由单测 + clippy + 语料三套件两轮覆盖（g10 同例在案）。

## 碰头与协作记录

- 前轮 g10 批（time 维度+opacity 数学臂）由 429 rate-limit 中断遗留工作树，
  本 session 开工时双 spawn 竞态对手方已完成三提交（5fc0d203a fmt /
  f06979d02 g10 批 / 8409a3b7b 入账 + c4bce8a46 证据补全）并 push——本流
  认领件：make test g10d 权威账核对（3881P/10F）+ r307_tree_order_identity_domains
  stash A/B（干净 HEAD 同红，与对手方 db74aad18 worktree 探针互补定谳 CSSOM
  写回刀窗口）——账目已由对手方并入 8409a3b7b/c4bce8a46，无重复劳动。
- 本批工作面 = style-system 单文件；对手方 color WIP（part11-color.js +
  part11-computed-color.js +37 行）零交集，css-color 对照轮 contaminated
  不跑（color 账归彼流批次）。
