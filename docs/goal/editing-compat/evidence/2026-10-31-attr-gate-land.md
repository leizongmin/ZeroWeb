# 融合序列化 attr 层 gap 落地——inline-host 880F 解封至 800F

**日期**: 2026-10-31
**前置**: [insertParagraph 簇落地](2026-10-31-insertparagraph-cluster-land.md)（归因 attr 层 gap 为 inline-host 0P/880F 封顶根因，本轮修复面）

## 1. 修复面（4 文件，1 提交）

| # | 位置 | 内容 |
|---|---|---|
| ① | callbacks.rs `__zw_pending_attr_sels`（新桥） | journal 未应用尾 `[cursor..)` 的 attr 类变更（SetAttr/RemoveAttr/SetStyle/RemoveStyle）去重 sel 清单；版本戳 `len:cursor`。消费方 shim 融合序列化门（R380/R3254-E2）加第三臂 |
| ② | part04.js outerHTML sel 分支 + part05.js `_zwFusedInnerHtmlSel` | 门臂 `|| _zwPendingAttrDirty(sel)`——纯 attr 变更开门，序列化从融合视图取 latest-wins attr 读（与属性读路径同层，读写两层同步） |
| ③ | part05.js R383/R383a | `_zwOverlayPendingChildNodes` **跨代际桶不并入**（stamp ≠ 当前 apply 代际），配 `_zwApplyPairedEnv()` 域闩（见 §3.4） |
| ④ | part05.js `_zwPendingSubtreeDirty` + `_zwPendingAttrDirty` | `__zw_contains` 返串 `"1"`/`"0"`，`"0"` 真值（仅空串 falsy）——旧 truthy 判定使非前缀键恒命中（PSD，既有 bug）与祖先维度恒开门（本轮新臂，初版）；均改 `=== '1'` 字符串比对 |

配套 webview.rs：`apply_pending_shared_mutations` / `apply_mutations_subset` 在 `applied_mutations` 推进处即时 `publish_mut_applied_cursor`（原只在 R348 重绑注册站发布，apply 与发布间存在同 turn 窗口，R383a 闩首判滞后一批）。

## 2. 语料账（同驱动同参，R5342 自适应墙）

| 语料 | 落地前 | 落地后 |
|---|---|---|
| insertparagraph-in-inline-editing-host | 0P/880F | **80P/800F** |
| insertparagraph-with-white-space-style | 616P/488F | 616P/488F（零位移） |
| insertparagraph-in-editing-host-cannot-have-div | 420P/0F | 420P/0F（零位移） |

inline-host 800F 残余归因（失败行抽样）：① 跨子测状态渗漏——"followed by text" 族尾部 `abc` 文本节点（测试自建 finally 移除）在融合容器视图跨子测残留；② content-bearing 案 span 内容丢失（D1 族 text-view 生命周期，与 marker 剥离同族）。新簇证据已齐，下轮候选。

## 3. 归因链（bisect 关键实证）

修复非单点，四层叠加，逐层 A/B 钉死：

1. **门不开**（原始 gap）：`removeAttribute` 后 attr 读返 false、序列化仍带旧 attr——R380 门只认结构桶/子树脏。①② 直修。
2. **全队列扫描**：初版 ① 扫 append-only 全队列，已 apply 的 SetAttr 使 attr 门对整页恒开（per-read 开销膨胀 + overlay 重并成倍增串，语料慢 6×+ 的第一来源）。加 cursor 过滤。
3. **跨代际桶幽灵重并**：门开后 apply 落座条目仍留桶（pa2b 只作废不清桶），overlay 基底 + 幽灵双计——t4 探针实证 4 br（2 基底 void 形 + 2 幽灵 `_zwMOuterHtml` 非闭合计）。R383 桶级跳并直修。
4. **truthy-"0" 连坐**：`__zw_contains` 返串，truthy 判定使 `_zwPendingAttrDirty` 祖先维度与既有 `_zwPendingSubtreeDirty` 非前缀键**恒命中**——每次门开走全页融合/BUild 路径（语料慢 6×+ 的第二来源、`<br></br>` 非闭合串页级扩散根因）。`=== '1'` 直修（同 `_zwHandleInHostSnapshot` 既有惯例）。

### R383a 域闩（t8r-4 臂②语义保护）

R383 首版无条件跳并使 engine lib `t8r4_k3_invalidate_nowire_and_ce_subtree` 臂②翻红（A/B stash 实证 1359P/11F → 1358P/12F，唯一新增即此案）：该臂**裸调 `__zw_apply_generation_bump`**（镜像生产通知调用点，但无配对 apply），跨代际桶条目仍 pending，跳并把 pending 子从视图丢失（`firstChild` → null）。

生产 webview apply 与换代成对（两调用点均 apply 后发 bump），共享队列 FIFO 消费保证「跨代际桶条目必已落基座」；engine 纯测试环境游标（`MUT_APPLIED_CURSOR`）恒 0。域判别用**游标闩**：`__zw_pending_attr_sels` 版本戳 cursor 分量 > 0（本页发生过 apply 发布）置闩恒跳并，engine 测试永 0 恒并；桥未注册视作配对环境（语料修复语义优先）。闩后 t8r4_k3 复绿、语料账逐位复原。

**中间账留档**（R383a 首版按父 sel 查已应用结构清单，非闩）：inline-host 退化 20P/860F——失败形态 br 缺失（非双计）：per-subtest 动态建 host 为 handle 寻址父（`AppendChildByHandle`），journal sel 键清单无法覆盖，跨代际桶错误并入已失效 wrapper（序列化为空串）；且游标发布滞后窗口（R348 重绑前）使首批判定失真。据此废弃 per-parent 清单方案（`__zw_applied_struct_parents` 桥未提交即撤），收敛为闩 + webview apply-time 游标即时发布。探针矩阵 zz-probe-attr7（removeAttribute 断言窗）单案 Pass、zz-probe-attr11（语料同形 static host）单案 Pass。

## 4. 门禁

| 门 | 结果 |
|---|---|
| engine lib（test-guard 包裹） | 3617P/**11F**——失败集与 HEAD 逐条 diff **identical**（11 案已知红；t8r4_k3 由 R383a 复绿） |
| webview lib | 822P/1F（`test_document_scrolling_element_r3259`，pre-existing 已知红） |
| fmt/clippy | clean |
| rustfmt 行宽 | 4 文件均 <2000 行 |

## 5. 跨域回归电池

| 面 | 本轮 | R5331/前基线 | 判读 |
|---|---|---|---|
| dom（testharness-dom） | 见 §5.1 | 56,049P/576F/15T（R5331） | — |
| selection（testharness-selection） | 见 §5.1 | 无可比全分母历史账（archive M1 为 2,970 子测切片；selection-compat v2.0 M1 未启动，分母 33,845/20 文件） | 本轮 101 文件全分母为**首个全面账**，作后续基线 |

### §5.1 终账

| 面 | 本轮（R383a 终版二进制） | 对照 | 判读 |
|---|---|---|---|
| dom | **55,948P/527F/14T**（57,382 行） | R5331 56,049P/576F/15T | F **−49**；14T 全为已知重族（Range-mutations-×3、webkit-animation-×4、EventListener-incumbent-×2、invoke-legacy、replaceChildren、Event-dispatch-on-disabled、abort-signal-any、cross-origin-named-access），无新 T 族 |
| selection | **27,589P/23,591F/5T**（80,383 行） | 本 session 前轮（R383 无条件版）27,590P/23,599F/5T（51,194 行，箱体截断） | P **−1**、F −8、T 集相同——+29k 行纯 F 区截断释放，零回归信号；101 文件全分母为 selection 面**首个全面账**（archive M1 为 2,970 子测切片，selection-compat v2.0 M1 未启动），作后续基线 |

dom 面 F 净改善与 P 净增并存、T 集恒定，判无回归；selection 面 P 分毫级位移在截断释放噪声内。

## 6. 挂账

1. **part04 outerHTML sel 分支 BUILD 路径 void 处理缺失**：`</tag>` 无条件闭合（R308 void 表只在 part03 `_zwMOuterHtml`/`_zwMSerialize`）。当前仅 void 元素自身持 pending attr 时命中，语料未触发；与 800F 残余簇同轮处理。
2. **inline-host 800F 残余簇**（§2）：跨子测文本渗漏 + D1 族 text-view 生命周期，证据已采样，下轮首位。
3. **renderer 私有队列路径的 R383a 语义**：renderer 直连不发布游标 → 闩不置位 → 跨代际桶恒并（pre-R383 行为）。editing 语料走 webview 不受影响；renderer 编辑面若暴露同形 ghost 需单独对账。
4. join 精细化三族、italic/bold 组、void-element、空 li no-op、Shift+Enter——承前挂账不变。
