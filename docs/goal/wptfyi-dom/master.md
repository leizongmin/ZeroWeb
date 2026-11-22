# DOM 与 Shadow DOM控制面

**2026-11-22 · Active · DC-2 切片 3 已落地（Range-mutations dataChange 188F→0F，dom 通道 +405P/−188F/−1T 零回归；shim CharacterData 三 setter 补 replace-data 边界）**。DC-1 基线 68,790 + 切片 1/2/3 净增 +65/+8,640/+405 ≈ 77,900 / 107,971（coverage 快照口径，DC-3 空载复锚），目标净增 35,000 达 ≥99,947。DC-1 已完成归档。\
**卡点** 无阻塞。语料 pin `31597693` 与快照分母 `a3830cd8f7` 的漂移面（~12,000st）单列待决：bump 共享 lib.sh WPT_REV 属跨流共享面，DC-3 前统一处理。\
**下一步** DC-2 切片 4：sanitizer/trusted-types 语料+通道三缺面（DC-1 优先序第三簇；pin 在、语料可 fetch，动手前先核 runner 通道面）。RT 集群余量泊入后置：深语义 41F（form association 8F / event retargeting 12F / .labels 1F / basics invalid-ID 3F / gethtml 序列化 3F 等）+ accname testdriver 通道（`get_computed_label` stub 解锁 dom-mutation/label-descendant/label-for 3 Unsupported 案 + aria-labelledby 5F）；伴生修 `new CustomElementRegistry()` Illegal constructor 误伤顺延。切片 3 证据见 [evidence/2026-11-22-dc2-ranges-slice3](evidence/2026-11-22-dc2-ranges-slice3.md)、切片 2 见 [evidence/2026-11-22-dc2-rt-slice2](evidence/2026-11-22-dc2-rt-slice2.md)。

[执行契约](../wptfyi-dom.md) · [95% 冲刺计划](../../compat/wptfyi-95-plan.md)

| 里程碑 | 状态 | 缺口与证据 |
|---|---|---|
| DC-1 基线与试跑 | ✅ 已归档（2026-11-22） | 基线 68,790/107,971 + 缺口三分法 + 六腿试跑一致性定谳（本地=CI 管线，dom +224/domparsing +28 引擎漂移）：[evidence/2026-11-22](evidence/2026-11-22-dc1-baseline-and-trial.md) |
| DC-2 按簇收敛 | **切片 1 A/B/C + 切片 2 D 面 + 切片 3 ranges 已落地（2026-11-22；ref-target 33P→98P→8,738P；切片 3 dataChange 188F→0F、dom 全腿 57,472P/683F/12T→57,877P/495F/11T、wc 全腿真实面零变化 20,653P/1,235F/17T；零新增 F、逐文件 P drop=0）** | 切片 1/2/3 证据：[slice1](evidence/2026-11-22-dc2-rt-slice1.md) · [slice2](evidence/2026-11-22-dc2-rt-slice2.md) · [slice3](evidence/2026-11-22-dc2-ranges-slice3.md)；优先序全表见 [DC-1 证据文](evidence/2026-11-22-dc1-baseline-and-trial.md)：~~切片 2 D 面（注册门）~~ → ~~nodes/ranges 失败面~~ → sanitizer/trusted-types → domparsing XML → idlharness wrapper → quirks（协作）→ domxpath/CE/observable → webidl |
| DC-3 全量对账 | 待执行 | 验收线 ≥99,947 项通过 |

## DC-1 关键定谳（2026-11-22，防后续轮重走）

- **语料不是主杠杆**：dom/nodes、dom/ranges、dom/traversal、dom loose static 在 pin `31597693` 已齐（本地语料 = pin 静态面全集）；「补语料」预期收益 ~500st。「缺口主体是引擎工作」（~34,500）与「变体映射/runner 包装面」（~2,600：dom idlharness window/worker 虚拟形态 2,646 等）。三方对账方法与逐簇表见证据文。
- sanitizer-api / trusted-types / webidl / xml 四目录 **pin 在、语料可 fetch**——0% 状态是「语料+通道+API」三缺，不是漂移。
- quirks 大头（unitless-length 4,784 + hashless-hex-color 2,060）是样式系统写面（rendering 流域），且 mode 页为 iframe 形态（runner 无 iframe 文档管道）——动手前按 goal 划界协作。
- shadow-dom/focus + focus-navigation（248）为 wc 通道政策性排除（渲染级专项泊位，42/45 号流共享面），本 goal 不擅自解排。
- 语料 fetch 必须先 `source ~/use-proxy.sh`（首射直连超时实证）。

## 碰撞记录

- 2026-11-22：开工时树内有 CSS 流 INSETS-V4 刀 WIP（layout-engine 4 文件，staged 空 blob 索引损坏）——已 stash 过境（`insets-v4-wip-transit-dom-dc1-build`，patch 备份 `.acceptance/dps/insets-v4-wip-transit-1519-backup.patch`，md5 `dedf2067dc4d58732c3365611d047214`），保证 DC-1 基线二进制干净锚定 `6d7407da2`；试跑链完成后已 pop 归还（md5 复核一致）。未触碰其字节。
- 2026-11-22：f9f56f2c2（标题为 css 流 KC-4 收账，实为 stale 工作树回写——rendering R5911「第三流 f9f56f2c2 stale 回写＝blob 复核反向贴回」与 html-semantics 补账十四已各自定性并恢复本流域）携旧工作树抹及本流：master.md 退回 2026-11-09 代（rebase 冲突按本版整体重放，来件该文件零新增内容）＋删 DC-1 证据文与 dc1-*.log 等 10 件（原 blob `01e5eeb62` 核证后自 f131e559e 原样恢复，见随后碰撞恢复提交）。本流 DC-1/DC-2 账目无实际丢失。
- 共享写面其他碰撞暂无。`lib.sh WPT_REV` bump 为跨流共享面（见卡点）。
