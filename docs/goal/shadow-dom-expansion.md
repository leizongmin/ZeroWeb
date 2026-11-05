# shadow-dom 扩面 — 语料滚动 + 通道重审承接

**版本**: v1.0
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）
**状态**: Active——M0/M1/M2 完成 + M3 Batch 1/2 落地（2026-11-06，门值 31.4% 回填；
Batch 2 簇 A unsafe 三 API：gethtml.html 预算内 2,210P/0F，全通道 A/B P→F=0、F→P 2,839），
Batch 3 楔死归因收口（90s 看门狗 × ShadowRoot 域 declarative 写路径二次累积；
看门狗随乘落地，×4 复测 gethtml 3,417 行仅 3F），
M3 余项：残面修复（t8c 协同）/ 簇 C engine glue / 簇 D 甄别 / attachment 物化臂 170F
**执行模式**: WPT 驱动（上游 shadow-dom corpus 为验收标尺）+ 语料扩面 + 按簇语义收敛
**父目标**: `docs/goal/zero-web.md`（真实可用浏览器——Web Components 面）
**前账**: `docs/goal/archive/web-components.md`（2026-09-11 收口，DC-1~5 ✅，
三目录 76.0%——slot 全链路/composed path/retarget 语义已落）

## coverage 对账（coverage-2026-11-02 基线，全量头条 67.54%）

| 面 | 全量分母 | zw 覆盖 | 通过率 | 未覆盖 |
|---|---:|---:|---:|---:|
| shadow-dom | 15,350 | 62/383 文件，621P | 4.05% | **14,729** |
| custom-elements | 4,356 | 175/206 文件，2,817P | 64.67% | 1,539（残余） |

通道现状：`testharness-web-components`（WC_TEST_SUBDIRS = custom-elements +
shadow-dom + template）已入账，但 fetch 子集窄（`fetch-web-components-subset.sh`
排除 shadow-dom/declarative、leaktests 等域），shadow-dom 只入 62/383 文件
——**web-components goal 收口后上游语料滚动 + 当时排除域未重审**是缺口主体。

前账遗留锚点（master.md 定稿）：

- reactions/ 剩余 fail（~207F）——media 联动/table insertRow/Range 交叉面，
  独立深项域。
- builtin-coverage（~126F）——customized built-ins innerHTML 解析实例化。
- **渲染级 composed tree**（shadow 树进样式/布局/绘制、`:host`/`::slotted`
  + Rust 侧 slot 消费）——**已拆专项 goal `shadow-dom-rendering`（45 号，
  2026-11-04 用户拍板立项）**；不在本 goal DC。

## Mission

1. **语料滚动扩面**（M0/M1）：FORCE 续拉对齐 pin + 上游差集甄别（收口后
   新增面、declarative 排除域重审——declarative shadow DOM 的 parser/
   template 层是否已可跑、哪些子面仍属渲染级排除）；产出批次与数值门。
2. **通道扩面**（M1/M2）：fetch 子集扩批 + skip 规则重审（两侧同源）+
   runner 通道 DIRS 扩面； weekly 分片接线核对。
3. **按簇语义收敛**（M2/M3）：扩面后失败簇逐簇归因——JS 语义簇（slot/
   retarget/composed path 深水区）就地修；渲染级耦合簇归账
   rendering-compat 或具名挂账。
4. **复测收口**（M4）：shadow-dom topdir 落数值门；custom-elements 通道
   既有账零回归。

## Done Criteria

- **DC-1（M0 对账）**：上游差集与排除域重审结论落
  `docs/goal/shadow-dom-expansion/evidence/`（逐目录：文件数、批次归属、
  declarative 重审裁定）；DC-4 数值门按决策规则落定并回填本文件。
- **DC-2（通道扩面）**：批次目录入常驻通道（fetch 子集 + skip 规则 +
  DIRS + Makefile target 四件套）。
- **DC-3（基线与收敛）**：扩面基线分类落 evidence（JS 语义 Fail 与渲染级
  环境受限分列）；至少一批 JS 语义簇就地收敛或具名挂账。
- **DC-4（coverage 复测）**：shadow-dom topdir 从 4.05% 升至数值门；
  custom-elements 既有账（64.67% 台账）零回归；头条不回落。

**DC-4 数值门决策规则（立项即定）**：M0 甄别后按地址池分档——地址池（
导入即得 + JS 语义可修）≥ 8k 子测试：全量推进，数值门 = 地址池 × web-
components 收口时同类簇通过率；< 8k：只导入 JS 语义面目录，渲染级耦合
目录如实标记挂账。

## 划界

- **渲染级 composed tree 不进本 goal**：已拆专项 goal 45 号
  shadow-dom-rendering（2026-11-04 用户拍板）——`:host`/`::slotted` 命中、
  布局/绘制消费归 45 号；本 goal 只收 JS 语义面与语料滚动，与其零重叠
  （shadow-dom 目录 fetch 子集分域记账）。
- **custom-elements 不重开主账**：64.67% 是 web-components 通道在账
  通过率，本 goal 只做同通道扩面时的零回归门 + 滚动新面顺带入账。
- **template 面**归 html-semantics-compat 通道既有账，不动。

## 里程碑

- **M0**：差集甄别 + declarative 重审 + 数值门（DC-1）。✅ 2026-11-06
- **M1**：语料扩批 + 通道四件套（DC-2）。✅ 2026-11-06
- **M2**：扩面基线分类（DC-3 前半）。✅ 2026-11-06
- **M3**：JS 语义簇收敛批（DC-3 后半）。
- **M4**：coverage 复测收口（DC-4）。

## 进展

### 2026-11-06——M0/M1/M2 完成（单轮）

- **M0（DC-1）**：上游全树 392 blobs 逐目录甄别 + declarative 排除域重审
  （parser 层 s168/s173 已落，旧排除条款过时）→ 转正 declarative/
  reference-target/untriaged 三域 + 顶层 2 新案；地址池按 wpt.fyi 口径
  ≈14,500 ≥ 8k → **全量推进**，初版门 75.0%。证据：
  [m0 差集甄别](shadow-dom-expansion/evidence/2026-11-06-m0-diff-triage-and-gate.md)。
- **M1（DC-2）**：通道四件套扩批（fetch 脚本 +124 TH 案、`wc_case_skipped`
  转正收窄、扫描/Makefile/weekly 零改动即接线）；修复
  `"shadow-dom/reference"` 前缀项吞 `reference-target/` 的 skip 碰撞。
- **M2（DC-3 前半）**：A/B 全通道零回归（custom-elements/template/sd-top
  逐案一致；untriaged 差值恰为 2 个有意排除 fixture 页）；扩面基线
  392 案/10,963 子测试/4,186P；失败簇 A~D 归因——簇 A（unsafe 解析/
  序列化 API 族，~5,300 子测试，gethtml.html 单文件 4,507）为 M3 首刀。
  **门值回填 31.4%**（实测池 5,531；wpt.fyi 估计 14,500 的差值 =
  缺 API 早抛截断，untriaged 对照域 251=251 自洽）。证据：
  [m2 基线与失败簇归因](shadow-dom-expansion/evidence/2026-11-06-m2-baseline-and-cluster-triage.md)。
- **碰撞记录（run-rules §9）**：本轮与 editing-compat rally 共享 clone
  （runner 二进制重建交错），改动面零共享语义冲突，未强制消解。质量门禁
  按实际影响收窄：clippy `-p zero-wpt-runner --all-targets -D warnings`
  0 告警通过（workspace 全量门因共享树 rally 活跃持锁按 §9 延后，
  本提交改动面不含 wpt-runner 以外代码）；lei-pre-commit-guard PASS
  （7 文件/467 行/0 发现）。
- **下一步（M3）**：簇 A+B 首刀——setHTMLUnsafe/getHTML/parseHTMLUnsafe
  三 API + 片段解析物化接线（innerHTML/createContextualFragment 接
  `materialize_declarative_shadow_roots`），预期翻转 ~5,300 子测试；
  其后簇 C（referenceTarget）实现或具名挂账、簇 D 逐项甄别。

### 2026-11-06——M3 Batch 1（片段解析 opt-in 语义）

- **簇 B 首刀落地**：opt-in 页（declarative-shadow-dom-opt-in，120 子测试）
  片段解析路径收敛——innerHTML/insertAdjacentHTML/createHTMLDocument/
  document.write(fresh) 不物化、template 存活、`shadowrootmode` 值还原源码
  小写、content 可读。**117F+1P → 108P/10F**（+107 翻转）。
- 根因三层：shim 查询源模板空形态化（WC-M3 既定）→ 引擎 skip 路径留下
  s173 预变换 `Open`/`Close` 可观测值 → shim 身份桥键失配丢 content 视图。
  修复 5 文件 +170/-8：materialize 全模板值还原（dom/shadow.rs）+
  `_zwQueryKey` template 键内容不敏感剥离（shim part03.js）+ 上轮已落的
  parser 物化接线/片段预变换与还原一并入库。
- **全通道 A/B 零回归**：392 案逐子测试比对 P→F = 0；旧五面
  （custom-elements/template/sd-top/untriaged/reftarget）逐案一致；
  decl-top +108P，池展开 +494 全在 gethtml.html（早抛解除→新子测试全落
  簇 A——unsafe API 族 Batch 2 目标面，池随实现展开口径与 M2 预测一致）。
  剩余 10F 逐条归因（canvas 待查 1、查询路径快照闭包 4、CFR 1、
  parseHTMLUnsafe 1、document.write allowed 1、iframe 2）。证据：
  [m3 batch1](shadow-dom-expansion/evidence/2026-11-06-m3-batch1-fragment-parse-optin.md)。
- **下一步（M3 Batch 2）**：簇 A unsafe 三 API（setHTMLUnsafe/getHTML/
  parseHTMLUnsafe + shim 接线 + declarative 物化语义对齐）——预期翻转
  gethtml.html 4,946 子测试主力；canvas 1F 与 document.write allowed
  路径顺带排查。

### 2026-11-06——M3 Batch 2（簇 A unsafe API 三件套）

- **簇 A 首刀落地**：setHTMLUnsafe/getHTML/Document.parseHTMLUnsafe 三 API +
  shim 接线（ShadowRoot.prototype 面、detached doc open/close/write 物化印记、
  attachShadow 声明式重挂分支）+ getHTML 序列化域（资格根模板形注入、
  serializable 资格、options 透传）+ shadow echo 剔除（innerHTML/outerHTML/
  getHTML 序列化三闸门：引擎子树内带 shadowrootmode 的 template 只能来自真根，
  按属性 presence 剔除）。
- gethtml.html：M2 基线 4,506F+1T（fail-fast 瞬报）→ **预算内真实执行
  2,210P/0F/1T**（oracle 潜在 ~4,5xx 子测试）。长尾定性收口：×1/×4 成对复跑
  行数平坦（2,056 vs 2,035，×4 注入生效页面 ~240s 超时）+ harness 超时时
  `pending:1` → 特定子测试**楔死**坐实（非成本爆炸），楔死身份挂 knee probe
  （runner 基建）。canvas ×4 append 修复（standalone 补 append 委托）全通道
  确认翻绿。
- 附加面外溢：attachment.html 0P/654F → 484P/170F（opt-in disallowed 校验臂
  整族翻转；余 170F 为 safelisted 物化臂挂账）、property-reflection +30P、
  clonable +2P、serialization +2P。
- **全通道 A/B 零回归**：392 案公共行逐子测试比对 P→F = 0，唯一转移
  Fail→Pass 2,839；custom-elements/template 既有账逐案零变化，DC-4 零回归门
  维持。opt-in 页 109P/9F（余 F 逐条挂账见证据）。DOMParser/XHR/detached doc
  查询视图翻 unmaterialized（opt-in 语义，URL 变体 iframe 维持物化）。
- 顺带基建：runner `ZW_TIMEOUT_MULTIPLIER`（wptrunner `--timeout-multiplier`
  同义面）；serializer ShadowRoot 布尔属性 spec 序回写（单测抓出首版引号
  bug）；zero-dom 新增物化片段/旗标序列化单测，885+ 全绿。
- 证据：[m3 batch2](shadow-dom-expansion/evidence/2026-11-06-m3-batch2-unsafe-apis.md)。
- **碰撞记录（run-rules §9）**：Batch 2 交付期同 session 双 resume 并行
  （04:19 与 05:31 两次 resume 同 `d72338b4`，先手侧未感知后手）。38b972751 /
  f98948c72 / lark 通知 ×2 由先手侧落地并推送；后手侧独立复算全通道 A/B
  （P→F=0、F→P 2,839 同值互证），并以最终二进制 ×4 filter 复验佐证楔死
  定性（4 文件 3,194P/3F/1T——含被 sweep 扫入的 zz-gethtml-slice 探针
  1,156P，非语料不计）。**探针清理冲突**：后手按研删纪律清 8 页
  （declarative/ 下 zz-cost/cleanup/h1/divx/hang/gethtml-slice/gethtml-trunc/
  knee-probe）——先手 evidence 挂账的 zz-knee-probe 已删，Batch 3 楔死身份
  需重建探针再跑。改动面零语义冲突，未强制消解。
- **Batch 3 楔死归因收口（2026-11-06 下午）**：探针法证链（knee×1/×4 + wedge2 +
  ABC/ABC2/IJ/I12）定性收口——**无固定楔死子测试**；冻结点 = 90s V8 看门狗墙钟
  （×1/×4 行数平坦坐实）× **ShadowRoot 域 declarative setHTMLUnsafe 组合写路径
  每测试 O(N) 增长**（I12 终分：attachShadow 单独、元素域 setHTML 单独均平坦，
  仅组合增长 1→21ms 边际超线性；t8c 读面索引不覆盖此写路径交互项）。runner
  侧看门狗随乘数放大已落地（testharness.rs `script_timeout_scaled_ms`），
  ×4 复测闭环：gethtml 3,417 行仅 3F（真实 referenceTarget Fail）、knee2 div×8
  3,072 全量完整、二次模型三内容定量自洽。证据：
  [batch3 rootcause](shadow-dom-expansion/evidence/2026-11-06-m3-batch3-wedge-watchdog-rootcause.md)。
- **下一步（M3 Batch 3 余项）**：①残面修复（已代码级定位）——ShadowRoot 句柄
  读每测试落 `with_query_view_doc` 视图路径 × 链增量资格检查 O(全队列) 线性扫
  （Q1 队列加速判别 + 代码互证）；修复候选：链资格 O(1) 计数器化 / ShadowRoot
  句柄读 live-aware 直读，均在 callbacks.rs = t8c 编辑面须协同落地，落地后重建
  knee/gethtml 复测（队列折叠兜底挂账）；
  ②簇 C referenceTarget——dom 层先行已由并行流落地（d8f473ff8，声明式值捕获+
  spec 序回写），待 engine glue 与通道复测；attachment safelisted 物化臂 170F
  甄别；簇 D 逐项甄别。

## 依赖约束（run-rules §9）

- WC 通道与 fetch-web-components-subset.sh 的 skip 规则两侧同源，改动
  同步；扩面后 custom-elements/template 既有账逐 subtest 零回归。
- 与 rendering-compat（活跃 rally）交界：凡甄别出样式/布局耦合簇，先
  归因落 evidence 再碰头归口，不双改。
- 上游滚动走 pin 对齐 + blob-hash 对账（goal-35 先例：文件数一致 ≠ 内容
  一致）。
