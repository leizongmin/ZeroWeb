# shadow-dom 扩面 — 语料滚动 + 通道重审承接

**版本**: v1.0
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）
**状态**: Active——M0/M1/M2 完成 + M3 Batch 1/2 落地（2026-11-06，门值 31.4% 回填；
Batch 2 簇 A unsafe 三 API：gethtml.html 预算内 2,210P/0F，全通道 A/B P→F=0、F→P 2,839），
Batch 3 残面修复落地（2026-11-06：语料级 A/B 0 回归，gethtml ×1 从稳定楔死
2,268 → 3,762P/0F，×1~×4 solo 随预算平滑延伸 3,443/4,819/5,958/6,908P
（×4 = 120s 窗全量真实上报，页级 Timeout 消失），无固定截断面——吞吐 >11×
与 Q1 队列 11× 互证，×1 全量差 = 写路径残余斜率（~19× 改善未归零）+ 预算
语义非病根），
M3 余项：簇 C engine glue / attachment+null 链修复落地与复测
（dom 面，m3c2 协同）/ rendering-compat 归口（adoptedStyleSheets 机制面 ~19F）
——簇 D 剔池决策已落地（module 载入链 27 案 27F+1T 出池，回池条件 =
script-sandbox 模块载入）
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
- **下一步（M3 Batch 3 余项）**：①残面修复——**已落地（见下节）**，候选
  「链资格 O(1) 计数器化」与「句柄读增量索引」均已实现（队列折叠兜底未动用，
  挂账撤销）；
  ②attachment 170F 甄别收口（2026-11-06 傍晚）——两个机制面：safelisted 臂 68F
  = setHTMLUnsafe 后声明式模板从不物化（gethtml 声明式象限通过系 attachShadow
  重建+序列化遮蔽虚构链）；disallowed 臂 102F = context 父模板被错误物化消费
  （spec：context 元素不附着）。修复面在 dom/ = 簇 C 流活跃面，已按 §9 交付
  证据待其落地复测：
  [attachment triage](shadow-dom-expansion/evidence/2026-11-06-m3-batch3-attachment-170f-triage.md)；
  ③簇 D 甄别收口（2026-11-06 傍晚，52F 四面拆分）——module/fetch CSS module
  载入链 ~17F 环境受限坐实（modulepreload onerror 实证，剔池候选挂账
  script-sandbox）；adoptedStyleSheets 机制 ~14F 与 rendering-compat 边界耦合
  归口挂账不双改；clone 面 ~3F（shadowrootclonable 克隆重建）+ IDL 反射属
  shadow-dom 范围内可修小刀（M3 尾段候选）；null 链 ~18F 与 attachment 机制一
  同根（静态物化不生效），随 m3c2 修复复测定格：
  [cluster D triage](shadow-dom-expansion/evidence/2026-11-06-m3-batch3-cluster-d-triage.md)；
  ④簇 C referenceTarget——dom 层先行已由并行流落地（d8f473ff8）；通道复测
  随本轮闭环：gethtml ×2/×3 solo 全 0F 且覆盖 ×4 旧 3F 位置（旧轮 build
  05:56 早于 d8f473ff8 06:30），3F 翻绿坐实，余 engine glue 深化项。

### 2026-11-06——M3 Batch 3 残面修复落地

- **落地内容**：stash `m3-residual-wip` 认领收口（§9 同 session 双 resume
  碰撞，Batch 2 同型）——链资格 O(1) `sih_prefix` 计数化 + `tail_iah_pending`
  增量扫描 + t8c 读索引泛化 `HandleStateIndex`（双层终态合并，HANDLE_TAG_MEMO
  同款）；callbacks.rs 单文件 +455/−35，新增单测 3，fmt/clippy（
  `-p zero-engine`）干净。同 clone 双 session 零编辑面冲突收口：本 session
  修复验证（fmt/clippy/单测/A-B），上流 session 提交代码（f11555070，内容与
  本 session 验证版零 diff）并接力 solo 复跑 ×1/×2/×3。
- **A/B 验证（WC 通道 ×1，同机同参守卫）**：语料 392 案逐子测试身份比对
  **P→F=0、F→P=0、vanished=0**，唯一 delta = gethtml.html **+1,494 全 Pass**；
  gethtml ×1 从稳定楔死 2,268 → 3,762P/0F。语料组合总数勘误 6,908
  （17 safelisted×384 + 95×4，算术确证；历史「4,506」是 ×8 楔死超时补记的
  截断面）——×1/×2/×3/×4 solo 随预算平滑延伸 3,443/4,819/5,958/6,908P，
  F 恒 0、无固定截断面：二次楔死已除；×4（120s 窗）6,908P/0F/0T **全量真实
  上报、页级 Timeout 消失**（~115s 窗外推确证），×1 30s 不可达属预算语义
  非病根；吞吐 >11×（修复前 ×8
  240s 窗 3,679 行 vs 修复后 ×2 60s 窗 4,819 行）与 Q1 队列加速 11× 互证；
  attachment 484P/170F 前后同态。
  [landing evidence](shadow-dom-expansion/evidence/2026-11-06-m3-batch3-residual-fix-landing.md)
- **HEAD 既有 5F 归因收窄（跨流）**：`cargo test -p zero-engine --lib` 干树
  （66dd8eeb3，无 WIP）复现 5F——part06 R2809 stylesheets write / part51 p99s
  custom property case / part56 p56 adopted snapshot / part06 R2810 rule style /
  R3276 add-remove rule；签名 = cssText 声明块尾分号（Blink 形
  `color: rgb(9, 105, 218);`）vs 测试期望旧形。归因 **s249（652a78e00，
  2026-11-03）`_zwSerializeDecls` 末声明带分号 + 颜色归一**（adopted/insertRule
  路径经 `_ruleFromText` 重建），认领声明时点的「q16/s299 系嫌疑」以此收窄；
  归属 html-semantics 流（s 系）bisect 精确翻转点后修（更新 5 期望或回退路径形）。
- **簇 D 剔池决策落地（M3 余项③，2026-11-06 傍晚）**：module 载入链家族
  27 案（async-fetch-*×15 / modulepreload-*×7 / nested-async / nested-sync /
  placeholder-identity / clonenode-async-fetch / fetched-module）出池
  27F+1T——wc-after 逐文件 F 签名复核修正 triage 的 ~17F 估数，判据 =
  载入机制依赖（module/importmap 逐文件 grep 确认）× P=0 × 断言链整体
  依赖载入成功。两侧同源落地：runner `wc_case_skipped` 新增
  MODULE_LOAD_CHAIN_PREFIXES + fetch 脚本 `SD_DECL_TENTATIVE_MODULE_PARKED`
  停拉（回池条件 = script-sandbox 模块载入落地）。复测：parked 全部消失、
  留池 9 文件逐行与 wc-after 零移动（25F+2P 精确一致）。簇 D 54F → 25F
  留池口径（rendering-compat 归口 ~19F + clone/IDL 尾段候选 ~6F）；
  triage 面 1 计数 17F→27F 修正随 evidence。
  [park evidence](shadow-dom-expansion/evidence/2026-11-06-m3-batch3-cluster-d-module-chain-park.md)

### 2026-11-06——M3 尾段小刀开工（clone 面 + template IDL 反射，本轮 session 认领）

- **认领（§9 同 clone 多 session）**：簇 D triage 挂账的两把 shadow-dom 范围内
  小刀由本轮认领——① `<template>` shadowRoot* IDL 反射（shadowRootMode 枚举
  limited-to-known-values / shadowRootDelegatesFocus·Clonable·Serializable 布尔反射 /
  shadowRootReferenceTarget 字符串反射；declarative-shadow-dom-basic reflection 6F，
  特征检测断言 HTMLTemplateElement.prototype own property——现状全缺）；②
  cloneNode clonable 根克隆（`_zwDeepCloneEl` 克隆臂：clonable 根随宿主克隆、
  shadow 树恒深拷与 deep 参数无关、非 clonable 保持 null + 轻量 attachShadow
  工厂补 init clonable/serializable/delegatesFocus 状态位；shadow-root-clonable
  「attachShadow with clonable: true」1F 直翻——实测 `clone.shadowRoot` null）。
- **实测基线（solo 失败消息定界）**：clonable/serializable 相关全通道 ~10F 中
  4F 属静态/setHTMLUnsafe 声明式物化不生效面（`!!root` false）——m3c2 归口
  （§9 不碰）；本刀不触物化路径。并行侧请勿动 part03.js 三处编辑面
  （R194b 轻量工厂 init 字段 / `_zwDeepCloneEl` 克隆臂 / HTMLTemplateElement
  原型挂载段）。

## 依赖约束（run-rules §9）

- WC 通道与 fetch-web-components-subset.sh 的 skip 规则两侧同源，改动
  同步；扩面后 custom-elements/template 既有账逐 subtest 零回归。
- 与 rendering-compat（活跃 rally）交界：凡甄别出样式/布局耦合簇，先
  归因落 evidence 再碰头归口，不双改。
- 上游滚动走 pin 对齐 + blob-hash 对账（goal-35 先例：文件数一致 ≠ 内容
  一致）。
