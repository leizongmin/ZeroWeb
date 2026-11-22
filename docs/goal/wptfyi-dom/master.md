# DOM 与 Shadow DOM控制面

**2026-11-23 · Active · DC-2 切片 5 刀③b① 已落地（TT 属性 sink 六 setter 通道：part05 `_zwSetAttributeNodeCore` 入口钩〔单点覆盖 setAttributeNode/NS + fresh 层 NamedNodeMap.setNamedItem/NS，ordering＝TT 检查先于 InUse/同属性早退〕、part03 `_r122Set` 传播钩〔Attr.value/Node.nodeValue——检查在传播 try 外〕、Attr.prototype.textContent accessor 新挂〔spec dom-node-textcontent attribute 特例〕、part14 `_zwTtAttrSinkData`〔type+spec sink 名〕+ default policy 回调参数序修正〔input,type,sink〕+ 单次透传旗标；set-attributes-require-* 三页 287P/249F→**536P/0F＝+249P/-249F 全清**、探针 13P/0F、lib 4249P/3F 精确回基线、san 323P 恒等、③a 探针 16P/0F 恒等、no-require 页 202P/0F〔textContent 传播附带 +23P〕）**。刀③a sink enforcement 基座已入链（script sink wrap 安装面回退记档在案）。DC-1 基线 68,790 + 切片 1/2/3/4/5①②③a③b① 净增 +65/+8,640/+405/+70/+933/+232/+13/+249 ≈ 79,397 / 107,971（coverage 快照口径，DC-3 空载复锚），目标净增 35,000 达 ≥99,947。DC-1 已完成归档。\
**卡点** 无阻塞。语料 pin `31597693` 与快照分母 `a3830cd8f7` 的漂移面（~12,000st）单列待决：bump 共享 lib.sh WPT_REV 属跨流共享面，DC-3 前统一处理。\
**下一步** DC-2 切片 5 刀③b② HTML sink 族（本轮进行中）：innerHTML/outerHTML/insertAdjacentHTML/document.write/ShadowRoot.innerHTML/iframe.srcdoc 属性反射/script.src 属性反射/script.text|innerText|textContent〔TrustedScript〕——block-string-assignment 族基线 31P/82F（15 页可跑）。后续：③ script sink wrap 重设计（native toString 伪装 + 错误归因保持——QuickJS eval 内建约束下或需引擎宿主桥臂；lib 11 失败面 3a 证据文在册）、④ .headers sidecar 17 页 runner 预处理微刀。微刀挂账（刀②③a③b 证据文分域在册）：handle 域 foreign ns（写链 handle 树 proxy namespaceURI 恒 XHTML，三级回落，解锁 javascript-url 残 22F/names 残 7F/animating 组）、parseHTML 文档级骨架消毒 26F、Node-multiple-arguments 180F＝replaceWith/after/before 多参 API 缺失（独立微刀，tt-enforced TypeError 断言叠加其上）、sanitizer 语义细节残留（config 24F/modifiers 14F/get 2F/boolean-defaults 2F）、report-only violation 事件面（SecurityPolicyViolationEvent）、document getElementsByTagNameNS/getAttributeNode（namespace.html 页级 1F）、runner 模块脚本通道、attribute-via-attribute-node 页 `<script src="x">` 404 载入挂起（engine 载入完成面刀位，六通道就位后回补即翻）。切片 5 刀③b① 证据见 [evidence/2026-11-23-dc2-tt-knife3b-slice5](evidence/2026-11-23-dc2-tt-knife3b-slice5.md)、刀③a 见 [evidence/2026-11-23-dc2-tt-knife3a-slice5](evidence/2026-11-23-dc2-tt-knife3a-slice5.md)、刀② 见 [evidence/2026-11-22-dc2-sanitizer-knife2-slice5](evidence/2026-11-22-dc2-sanitizer-knife2-slice5.md)、刀① 见 [evidence/2026-11-22-dc2-tt-knife1-slice5](evidence/2026-11-22-dc2-tt-knife1-slice5.md)。

[执行契约](../wptfyi-dom.md) · [95% 冲刺计划](../../compat/wptfyi-95-plan.md)

| 里程碑 | 状态 | 缺口与证据 |
|---|---|---|
| DC-1 基线与试跑 | ✅ 已归档（2026-11-22） | 基线 68,790/107,971 + 缺口三分法 + 六腿试跑一致性定谳（本地=CI 管线，dom +224/domparsing +28 引擎漂移）：[evidence/2026-11-22](evidence/2026-11-22-dc1-baseline-and-trial.md) |
| DC-2 按簇收敛 | **切片 1 A/B/C + 切片 2 D 面 + 切片 3 ranges + 切片 4 san/tt 通道 + 切片 5 刀① TT 工厂 + 刀② Sanitizer 构造面已落地（2026-11-22/23；ref-target 33P→98P→8,738P；切片 3 dataChange 188F→0F；切片 4 通道 +70P；刀① tt +912P/san +21P；刀② san 探针 +232P/-253F PI 页全清，余 F 分域挂账移交刀③+微刀）** | 切片 1–5② 证据：[slice1](evidence/2026-11-22-dc2-rt-slice1.md) · [slice2](evidence/2026-11-22-dc2-rt-slice2.md) · [slice3](evidence/2026-11-22-dc2-ranges-slice3.md) · [slice4](evidence/2026-11-22-dc2-san-tt-slice4.md) · [slice5 刀①](evidence/2026-11-22-dc2-tt-knife1-slice5.md) · [slice5 刀②](evidence/2026-11-22-dc2-sanitizer-knife2-slice5.md)；优先序全表见 [DC-1 证据文](evidence/2026-11-22-dc1-baseline-and-trial.md)：~~切片 2 D 面（注册门）~~ → ~~nodes/ranges 失败面~~ → ~~sanitizer/trusted-types 通道~~ → ~~TT 工厂刀①~~ → ~~Sanitizer 构造面刀②~~ → domparsing XML → idlharness wrapper → quirks（协作）→ domxpath/CE/observable → webidl |
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
