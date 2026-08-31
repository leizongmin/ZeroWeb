# R337 — events 备档复核 + DC-8 path-objects 勘误（2026-08-28，零源码改动轮）

## ① events 备档集 Timeout 复核（R336 同法）

| 用例 | 结果 | 归因 |
|------|------|------|
| Event-dispatch-on-disabled-elements | 4P/5T | sync click() 语义探针（7 form element button/input/select/textarea/optgroup/option/fieldset）全部 disabled 不派发 + enabled 派发（dOK/eOK）——**非语义缺口**。5 pending = 4×CSS transition/animation promise_test（真动画时钟依赖）+ 1×test_driver.click 实测（Activate 管线 disabled 判定，activation 域） |
| EventListener-invoke-legacy | 6/6T | 全部依赖 CSS transition/animation 30ms 真实触发——动画时钟架构项 |
| event-global-is-still-set-when-reporting-exception-onerror | 1F | 跨 realm（frames[0].Function 造 handler）+ 多层 onerror 恢复链——R295 family 深结构备档 |
| Event-timestamp-cross-realm-getter / EventListener-incumbent-global-* | T | frames undefined 跨 realm 深项（R331 已记） |

**结论**：events 备档集全部为「动画时钟 pump / 跨 realm / test_driver Activate 管线」
三类深结构/架构项，无轻量修复可达面——备档维持且定性精确化。

## ② DC-8 path-objects 勘误

全量跑 `testharness-canvas path-objects`（canvas 流 3 天零编辑，零碰撞核查通过）：

- **202 Pass / 0 Fail / 3 NotRun**（205 用例全执行）
- R56 记录的「剩余 3 深项」现状勘误：
  - `2d.path.stroke.skew` = NotRun（套件内 CTM 语义互斥用例的 runner 策略性 skip，非 Fail）
  - `roundrect.end.3` = **Pass**
  - `isPointInStroke.scaleddashes` = **Pass**
- 另 2 NotRun 为 reftest-format 用例（clip.scale，走 reftest/oracle 面）

**DC-8 收敛状态勘误为：202P/0F，3 NotRun 均为格式/互斥 skip，无 Fail。**

## 教训

备档定性也有保质期：R56 的「3 深项」在后续轮次实际已收敛，但文档表述未跟进。
巡检轮的价值在勘误与定性精确化——DC-8 的实际状态比记录更好。
