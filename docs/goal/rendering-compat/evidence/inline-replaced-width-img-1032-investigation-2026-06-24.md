# inline-replaced-width `<img>` 子集 — §10.3.2 下一 lever 调研（doc-side read-only）

**日期**: 2026-06-24
**性质**: read-only 测试文件核查（doc-maintenance），未跑 reftest（agent 正在跑 R548 normal-flow A/B，并发 OOM 风险）；为 R548 后的下一 lever 备料
**相关**: master.md item 4 第 3 子项（纠正 round-2 对 5 用例的不准描述）

## 背景

R545 把 normal-flow 残余 `inline-replaced-width（8）` 整体判为「inline `<svg:svg>` 范围外」。round-2 doc-side
核查纠正：该组混合，002/003/004/008/009 为 inline svg:svg（范围外，goal line 99），但 **001/006/012/014/016
是 `<img>` PNG 且资产存在**（support/blue96x96.png / blue15x15.png / swatch-green.png，R318 已贯通 img）= in-scope。

本轮读 5 个 img 用例的 title + 关键 css，**精确确认它们测 CSS2.1 §10.3.2 inline replaced width 的不同子情况**
（纠正 round-2 item 4「012/014/016=显式 w/h override」的不准描述）：

| 用例 | title 关键词 | 测的 §10.3.2 子情况 |
|------|-------------|---------------------|
| 001 | auto margin-left/right + **intrinsic** width | inline replaced 用 intrinsic 宽（96×96 img），auto margin→0 |
| 006 | auto margin + **percentage intrinsic width** | `<img width="50%">` HTML 属性百分比宽解析（15×15 img） |
| 012 | replaced inline **wrapping around floats** (% widths) | inline replaced % 宽（width:100%）+ float 包裹上下文 |
| 014 | replaced inline with **% widths** | `width:200%` / `width:50%` 百分比宽 |
| 016 | width - inline replaced + **max-height** | `width:auto; height:100px` + max-height 交互（aspect-ratio 约束） |

## 共性 = §10.3.2 百分比宽 + width:auto/max-height 交互

5 案主轴 = **inline replaced 元素的百分比宽解析**（006/012/014）+ **width:auto 与 max-height/aspect-ratio 交互**（001/016）。
即 ZeroWeb 的 `apply_replaced_element_sizing`（tree.rs）+ inline 宽路径可能未正确处理 **inline 上下文**的：
1. 百分比 width（相对 containing block）on inline replaced；
2. width:auto 时由 height + intrinsic aspect-ratio 推导（§10.3.2 §10.4 链）。

R318/R325 修了 img 固有尺寸（HTML 属性 + SVG data URI + 解码固有），但**是否覆盖 inline 上下文的 % 宽与 auto/max-height**
待 R548 后 code agent 跑 `reftest-upstream inline-replaced-width --jobs 3` 逐案区分 SVG-fail（skip）vs img-fail（修 §10.3.2）。

## 建议（next-lever，R548 后）

1. 先跑 `reftest-upstream inline-replaced-width --jobs 3` 确认 001/006/012/014/016 当前 FAIL（R545 标其为
   normal-flow 残余，应 FAIL；但需 fresh 确认，因 R525–R547 可能已间接修部分）。
2. 对 FAIL 的 img 案，LAYOUT_DUMP 看 img 计算宽 vs 期望，定位是 % 宽未解析还是 auto/max-height 推导缺。
3. seam 方向：`apply_replaced_element_sizing`（tree.rs）inline 分支 + engine inline 宽路径（非 col/table 域）。
4. 预期 yield ~3-5（取决于多少已间接修复）；与 R546「分母真实性」暴露模式同族（缺真运行，非渲染必坏）。

## 不改 master.md 的原因

agent 正活跃验证 R548（normal-flow A/B PID 66512 进行中 + 即将提交 R548+master.md 条目）。
本调研写 evidence（additive、零冲突），待 R548 落地、master.md 安全后折进 item 4 第 3 子项
（纠正「012/014/016=显式 w/h override」→ 实为 % 宽 / float+%/ auto+max-height）。
