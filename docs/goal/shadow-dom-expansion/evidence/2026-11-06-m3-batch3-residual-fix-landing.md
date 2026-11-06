# M3 Batch 3 残面修复落地——链资格 O(1) 计数化 + handle 读增量索引泛化

**日期**: 2026-11-06
**前置**: [wedge-watchdog rootcause](2026-11-06-m3-batch3-wedge-watchdog-rootcause.md)
（残面代码级定位：ShadowRoot 句柄读每测试落 `with_query_view_doc` × 链增量资格
检查 O(全队列) 线性扫；Q1 队列加速 11× 互证）；上一 session 离树 WIP
（stash `m3-residual-wip`）由本轮认领收口（fmt/clippy 修复含内）。

## 改动面（callbacks.rs 单文件，+455/−35）

1. **`ViewDocKey.sih_prefix`**：条目构建/链步时增量维护 `[0..count)` 内
   `SetInnerHtml` 出现次数（链步 = 旧条目计数 + `[prev_count..count)` 新段增量，
   R342 同窗清零以 `min` 收口；全量重建 = 单次全扫），链资格判定由每调用
   全量线性扫降为 `sih_prefix == 0` O(1)。语义逐位一致（同前缀同谓词）。
2. **`tail_iah_pending`**：live_ok 判定的 pending 尾 `[applied..count)`
   `InsertAdjacentHtml` 存在性增量扫描状态（`TAIL_IAH_SCAN` thread-local，
   lo/hi 窗口 + 计数，左剔右并摊销 O(新增)；Arc 身份 + drain 代际 + 回缩检测
   防御；applied 回退时窗口覆盖）。
3. **t8c 泛化（`HANDLE_TAG_MEMO` 同款，f3c4b952e/PR #94 先例）**：
   `HandleStateIndex`（text/tag/inner_html/attrs/attr_order 五 HashMap +
   `apply(&DomMutation)`）+ `HANDLE_STATE_MEMO`（共享队列，键含 mut Arc、
   drain 代际、reg_epoch、已扫水位，含就地截断检测）/ `HISTORY_STATE_MEMO`
   （已应用历史，clear_mutation_history 显式失效）——`__zw_get_text_handle` /
   `__zw_get_attr_handle` / `__zw_has_attr_handle` / `__zw_attr_names_handle` /
   `__zw_get_inner_html_handle` / `__zw_style_rules_handle` / 历史 tag·text 读
   全走双层终态索引合并（原全队列逆序/正序扫，gethtml 语料 ~75 native 读/子测试
   × O(队列) → 页级 O(n²)）。

等价性核对（对原实现逐函数）：text 4 臂 / innerHTML 5 臂 / tag 2 臂 /
attr latest-wins（Remove = 即时 absent = 值空）/ 名序算法（Set 去重保序 +
Remove 摘除 + 重设尾追加）逐一比对一致；跨层合并仅 1 处已注记 FIXME 边
（current 层 remove 后重设 history 层名，序落 history 位——dataset 枚举序
边缘，语料无钉面）。

## 验证

- fmt 干净；clippy `-p zero-engine --all-targets -D warnings` 0 告警
  （修复 WIP 自带 collapsible_if 1 处）。
- `cargo test -p zero-engine`：**3,761P / 5F**——5F 为 HEAD 干树既有（见下）。
- 新增单测 3：`handle_state_memo_latest_wins_layers` /
  `handle_state_memo_in_place_truncation_rebuilds` /
  `handle_state_history_layer_append_and_clear`。

### 全通道 A/B（WC 通道 ×1，同机同参 test-guard `--per-proc-mem 4 --total-mem 8 --time-limit 3600`）

before = HEAD 二进制（66dd8eeb3+2，08:29 备份 provenance）、after = WIP 二进制
（08:43 构建，md5 与 `/tmp/zw-runner-wip-m3residual` 一致）。语料 392 案
逐子测试身份比对（22 个 zz-* 诊断探针页行名带时间戳、逐行身份不可比，聚合
单列）：

| 面 | before (P/F/T/other) | after (P/F/T/other) | 逐行身份比对 |
|---|---|---|---|
| 语料 392 案 | 7,088/1,622/9/8 | 8,582/1,622/9/8 | **P→F=0、F→P=0、vanished=0**；唯一 delta = gethtml.html **+1,494 行全 Pass** |
| zz-* 探针 22 案 | 33,361/13/5/0 | 37,384/13/3/0 | 行名动态仅聚合：F/other 零变化，T −2 |
| 合计 414 案 | 40,449/1,643/14/8 | 45,966/1,643/12/8 | — |

零回归门通过：语料级逐子测试严格 0 回归（P→F/T 为 0），F/T/other 总量不变。

### gethtml.html 楔死解除定量（语料组合总数 6,908 = 17 safelisted×384 + 95×4，算术确证）

子测试由 9 层嵌套循环静态生成（语料 `runAllTests`），总数固定 6,908；
历史口径 4,506 = ×8 页级楔死时 testharness 超时补记的截断面（非总数）。

| 轮次 | 乘数 | 窗 | gethtml 结果 | 占全量 |
|---|---|---|---|---|
| M2 基线（rootcause 前） | ×8 | 240s | 4,506F + 1T（页级楔死 0 Pass，超时补记） | ~65%（伪） |
| rootcause 轮复测（05:56 build6） | ×8 | 240s | 3,678P/0F/1T，截于 3,679（rootcause 记 82% of 4,506 口径） | 53% |
| 本轮 before（HEAD） | ×1 | 30s | 2,268P/0F/1T | 33% |
| **本轮 after（WIP）全通道** | ×1 | 30s | **3,762P/0F/1T（54%）** | 54% |
| 本轮 after solo | ×1 | 30s | 3,443P/0F/1T（solo 与全通道差 ~8% = 页预算 wall-clock 方差） | 50% |
| 本轮 after solo | ×2 | 60s | 4,819P/0F/1T，行名唯一无重复 | 69.8% |
| 本轮 after solo | ×3 | 90s | 5,958P/0F/1T | 86.3% |
| 本轮 after solo | ×4 | 120s | **6,908P/0F/0T——全量真实上报，页级 Timeout 消失（整页含收尾在预算内完成）** | 100% |

吞吐定量：修复前 ×8（240s 窗）跑 3,679 行（二次方拖累），修复后 ×2（60s 窗）
跑 4,819 行——1/4 预算跑得更多；行数随预算平滑延伸（×1 3.4-3.8k → ×2 4.8k
→ ×3 6.0k），无固定截断面。3F referenceTarget（rootcause ×4 时真实 Fail，
build6 05:56 早于 d8f473ff8 06:30）已由 dom 层先行（簇 C，d8f473ff8）翻绿——
本轮 ×2/×3 均 0F 且 4,819 > 3,417 覆盖其位置。

**残余斜率定量**：三点拟合一致（×2→×3 时间比 1.5 > 行数比 1.24；×3→×4
时间比 1.33 > 行数比 1.16）——剩余小斜率超线性项存在，斜率 ~0.0019
ms/test²（修复前 knee 0.0355/×8 截点反推同量级），**改善 ~19× 未归零**。
归因：本刀覆盖读面（链资格扫 + 句柄读索引）与 live_ok 尾扫，I12 判定的
「I 相写路径组合增长」主项仍在（attachShadow × setHTMLUnsafe 每写 O(N)
视图重建），未被本刀触碰——后续若需 ×1 全量，方向 = 写路径增量化
（rootcause 修复候选③ live-aware 直读面），不在本刀范围。

×4 全量锚点：120s 预算内 6,908 行全 Pass、0 Fail、**页级 Timeout 消失**
（整页含 teardown 干净收尾）——楔死解除与全量可达双证闭环。×1（30s）达
54%，×1 全量需写路径增量化或 mega-case 页预算语义（rootcause 已记
wptrunner 语义面），挂账 M3 尾段候选。

### 泛化回归面（跨通道烟测）

- html-semantics FILTER=`forms`（`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`，HEAD vs
  WIP）：2 files/2 rows 全 Pass，0 翻转。域太薄无区分度，仅作烟测；
  HandleStateIndex 泛化的回归防线以 WC 全通道逐行 0 回归 + engine 单测为准。
- attachment：484P/170F 前后同态（safelist 61f84064e 已在两轮生效，170F 的
  context 排除归 m3c2，本轮不触）。

## HEAD 既有 5F（跨流，与本改无关）

`cargo test -p zero-engine --lib` 干树（66dd8eeb3，stash 前后同 5F）复现：
part06 R2809 / R2810 / R3276 + part51 p99s + part56 p56——签名 = cssText
声明块尾分号（Blink 形 `color: rgb(9, 105, 218);`）vs 测试期望旧形。
序列化形家族归因：**s249（652a78e00，2026-11-03）`_zwSerializeDecls` 末声明
带分号 + 颜色归一**（认领声明时点的「q16/s299 系嫌疑」以此为准收窄）；
adopted/insertRule 路径经 `_ruleFromText` 重建（part06.js `_zwAdoptedCombinedText`
推 `rules[j].cssText`）。归属 html-semantics 流（s 系）bisect 精确翻转点后修
（更新 5 测试期望或回退路径形）。

## 碰撞记录（run-rules §9）

- 同 session 双 resume（Batch 2 同型）：本轮（新 session）与被 rally resume 的
  上一 session（d72338b4）并行同一 clone。上轮 evidence 明示「本流不代 pop」
  → 残面 WIP 由本轮认领落地，零编辑面冲突。A/B 前后均以「改动全在位→重建→跑」
  顺序自证对应源；solo ×1 复跑由上流 session 以同 md5 二进制副本接力完成
  （09:02，`gethtml-after-solo-x1.json`），本轮直接采信（provenance 已核）。
- 二进制纪律：长跑期间无重建（before/after/solo 三轮同一对二进制），
  runner 二进制副本进 `/tmp` 不重建。
- **构建混杂排查（单变量保持证明）**：共享 clone 上另有两刀并行（bb114f730
  qState shim 刀 part01e.js，08:47 提交；35bc60eaf s300 PNG eXIf 刀
  render-foundation，09:10 提交）。特征串法证：before/after 两二进制 grep
  `shellHome`/`shellSuccessor`/`png_orientation`/`exif_text` 全 0——两刀改动
  均不在本轮任一二进制；part01e.js mtime 09:00:39 晚于 08:43 after 构建
  （决定性证据为特征串，mtime 辅助）。A/B 严格单变量（仅残面 WIP）成立。
- 代码 commit 由上流 session 09:10 收口（f11555070，+455/−35 与本轮 A/B
  验证面逐行一致，未触 goal doc）；本轮交付 A/B 证据与文档面，不重复提交。
- 质量门禁按实际影响收窄：clippy/tests 以 `-p zero-engine` 定向（workspace
  全量门延后至 5F 归属流修复后，本提交不含其修复面）。

## 结论与门值影响

- 残面修复落地，M3 Batch 3 ①完成：语料级 0 回归 + gethtml ×1 从稳定楔死
  2,268 → 3,762P/0F（54% of 6,908），行数随预算平滑延伸 ×2 4,819（69.8%）
  → ×3 5,958（86.3%）；修复前 ×8（240s）3,679 vs 修复后 ×2（60s）4,819，
  1/4 预算跑得更多。残余斜率 ~19× 改善未归零（写路径增长主项不在本刀面）。
- 全量口径勘误：语料组合总数 6,908（9 层嵌套循环算术确证）；历史 4,506 =
  ×8 页级楔死超时补记截断面，非总数。rootcause「82%」「×1 全量 ~4,5xx」
  预期随口径一并修正：**全量真实上报锚点 = ×4 solo（6,908P/0F/0T，120s
  预算内含收尾完整跑完）**；×1 全量需写路径增量化（rootcause 修复候选③
  live-aware 直读面，挂账 M3 尾段候选），mega-case 页预算语义备选
  （rootcause 已记 wptrunner 语义面）。
- 簇 C referenceTarget 3F 已翻绿：rootcause ×4 复测（05:56 build6）的 3F
  早于 dom 层先行 d8f473ff8（06:30）；本轮 ×2/×3 全 0F 且覆盖其位置——
  goal doc ④「待通道复测」可结，engine glue 为深化项非本 3F 修复面。
- M4 复测口径更新：gethtml 以「真实 Pass 行数 / 6,908」计（超时补记 Fail
  不计入通过分母），全量真实上报锚点 = ×4 solo（6,908P/0F/0T）。
- goal doc 状态头与 Batch 3 段由并行 session 收口（5fa247f0a，总数勘误
  6,908 + ×1/×2/×3 延伸链 + 簇 C 3F 翻绿，与本 evidence 定量一致）；本文档
  为定量细节面（A/B 表格、混杂排查、斜率拟合、×4 锚点）。
- 下一步：M3 余项 ② attachment 170F 复测（m3c2 协同）/ ③ 簇 D 剔池决策 /
  ④ 簇 C engine glue（深化项）；写路径增量化（候选③）挂账 M3 尾段候选；
  5F 归 html-semantics 流。

## 上流互证补记（d72338b4 rally resume，09:35）

- **独立 ×1 复算**：以同 md5 二进制副本 solo ×1 复跑 =
  **3,628P/0F/1T**——与 solo 3,443 / 全通道 3,762 同带；×1 三样本 ~9% 宽
  均为墙钟方差、无固定截断面，互证上表「延伸平滑」结论。
- **时序污染披露**：本互证跑（~09:15-09:17）与 ×3 solo 起步段并行约
  2.5 分钟，双向扰动；污染方向只会压低行数——本 ×1 结果仍高于净跑 solo，
  ×3 5,958 的「平滑延伸」判定对该 confound 不敏感；×4（09:25-09:31）在
  本流停手后净跑，锚点不受影响。
- **filter 口径**：本互证用子串 filter `gethtml`，连带命中
  reference-target/tentative/gethtml-serialization.html（3F，簇 C ④ 深化面
  另页）与 gethtml-ordering.html（3P）；上表 solo 用窄 filter
  `declarative/gethtml.html`——两口径在 gethtml.html 本页无差。
- **provenance**：solo 产物在 `.acceptance/shadow-m3-residual/`
  （gitignored）：x1 09:04 / x2 09:12 / x3 09:24 / x4 09:31；碰撞记录中
  「09:02」为 x1 run 起步时刻；本互证原始 JSON 留 /tmp（会话内一次性）。
