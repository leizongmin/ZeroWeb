# DOM 与 Shadow DOM控制面

**2026-11-22 · Active · DC-2 首簇进行中**。当前基线 68,790 / 107,971（缺口 39,181；coverage-2026-11-22 快照口径），目标净增 35,000 达 ≥99,947。DC-1 已完成归档：六腿试跑与 CI 记分管线逐目录对齐（一致性定谳），缺口三分法分解定谳。\
**卡点** 无阻塞。语料 pin `31597693` 与快照分母 `a3830cd8f7` 的漂移面（~12,000st）单列待决：bump 共享 lib.sh WPT_REV 属跨流共享面，DC-3 前统一处理。\
**下一步** DC-2 首切片：Reference Target property-reflection 三文件（property-reflection / -idl-setters / -imperative-setup，5,536st）——实现 `Element.referenceTarget` / `ShadowRoot.referenceTarget` IDL 反射 + `shadowrootreferencetarget` 声明式属性（矩阵注册展开即记分，行为面逐簇收敛）；伴生修 `new CustomElementRegistry()` Illegal constructor 误伤。

[执行契约](../wptfyi-dom.md) · [95% 冲刺计划](../../compat/wptfyi-95-plan.md)

| 里程碑 | 状态 | 缺口与证据 |
|---|---|---|
| DC-1 基线与试跑 | ✅ 已归档（2026-11-22） | 基线 68,790/107,971 + 缺口三分法 + 六腿试跑一致性定谳（本地=CI 管线，dom +224/domparsing +28 引擎漂移）：[evidence/2026-11-22](evidence/2026-11-22-dc1-baseline-and-trial.md) |
| DC-2 按簇收敛 | 待执行 | 优先序见证据文：Reference Target → nodes/ranges 失败面 → sanitizer/trusted-types → domparsing XML → idlharness wrapper → quirks（协作）→ domxpath/CE/observable → webidl |
| DC-3 全量对账 | 待执行 | 验收线 ≥99,947 项通过 |

## DC-1 关键定谳（2026-11-22，防后续轮重走）

- **语料不是主杠杆**：dom/nodes、dom/ranges、dom/traversal、dom loose static 在 pin `31597693` 已齐（本地语料 = pin 静态面全集）；「补语料」预期收益 ~500st。「缺口主体是引擎工作」（~34,500）与「变体映射/runner 包装面」（~2,600：dom idlharness window/worker 虚拟形态 2,646 等）。三方对账方法与逐簇表见证据文。
- sanitizer-api / trusted-types / webidl / xml 四目录 **pin 在、语料可 fetch**——0% 状态是「语料+通道+API」三缺，不是漂移。
- quirks 大头（unitless-length 4,784 + hashless-hex-color 2,060）是样式系统写面（rendering 流域），且 mode 页为 iframe 形态（runner 无 iframe 文档管道）——动手前按 goal 划界协作。
- shadow-dom/focus + focus-navigation（248）为 wc 通道政策性排除（渲染级专项泊位，42/45 号流共享面），本 goal 不擅自解排。
- 语料 fetch 必须先 `source ~/use-proxy.sh`（首射直连超时实证）。

## 碰撞记录

- 2026-11-22：开工时树内有 CSS 流 INSETS-V4 刀 WIP（layout-engine 4 文件，staged 空 blob 索引损坏）——已 stash 过境（`insets-v4-wip-transit-dom-dc1-build`，patch 备份 `.acceptance/dps/insets-v4-wip-transit-1519-backup.patch`，md5 `dedf2067dc4d58732c3365611d047214`），保证 DC-1 基线二进制干净锚定 `6d7407da2`；试跑链完成后已 pop 归还（md5 复核一致）。未触碰其字节。
- 共享写面其他碰撞暂无。`lib.sh WPT_REV` bump 为跨流共享面（见卡点）。
