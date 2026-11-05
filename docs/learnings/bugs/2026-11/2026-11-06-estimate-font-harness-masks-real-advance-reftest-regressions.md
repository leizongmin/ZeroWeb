date: 2026-11-06
modules: zero-engine,zero-layout-engine,wpt-runner
---

# estimate-font 测试 harness 掩盖真实 advance 下的 reftest 回归

## 问题描述

R5426 会话一：abs 盒 paint 直系文本种子臂（`ZW_PAINT_ABS_TEXT_SEED`）修复
content-177（abs span 直系文本落 16px 默认字号），在 `cargo test` harness 里
实测 2.27%→0.75% "pass"，测试全绿，看似修复完成。落地前 corpus 窗口
（make reftest-upstream 全量）A/B 定谳：该臂在真实字体窗口使
css-ruby/abs-in-ruby-{base,base-container,container} 3 案 ✓0.05%→✗1.04% 翻红，
content-177 自身 2.35%→2.90% 反而变差——harness 的"通过"是伪绿。

## 根因分析

两套环境文本度量域不同：

- runner `main()`（`tests/wpt-runner/src/main.rs`）设
  `ZW_RENDERER_AUTOMATION=1` 并注册 R1765 fontdue **实测 advance** 度量回调
  ——这是 reftest 语料窗口的度量真值；
- `cargo test` harness 两者皆无 → uniform estimate metrics（'m' = 0.584×fs
  等宽近似）。

estimate 等宽路径下，3em 文本 max-content 与可用宽的关系、折行触发点、字形
几何都和真实 sans-serif advance（≈0.6em/字符）不同：本案真实 advance 890 > 784
可用宽触发折行，estimate 单行放下两臂盒形重合——种子臂引入的折行分岔与
Path B 行几何耦合在 harness 里完全不可见，只暴露目标语义（48px 播种）恰好
"修好"单案。

## 解决方案

- **reftest 修复必须在 corpus 窗口 A/B 定谳**（`make reftest-upstream` 全量，
  比对 F 集合 movers），harness 单案针只用于快速迭代与回归护栏，不作为
  通过证据；
- 窗口 A/B 用 kill-switch 隔离单臂贡献（filtered runs），翻红案用
  REFTEST_DUMP PNG 逐臂取证归因；
- 涉及字体度量/折行/盒宽的修复，默认怀疑 estimate 侧结论，窗口复测后再
  判定落地。

## 关联

- R1765（runner fontdue 实测回调注册）、R4558（paint 直系文本种子 display 门）
- R5426 evidence：`docs/goal/rendering-compat/evidence/r5426-cdata-entity-and-abs-seed-revert-2026-11-06.md`
