# shadow-dom 扩面 — 语料滚动 + 通道重审承接

**版本**: v1.0
**日期**: 2026-11-04（立项，coverage-2026-11-02 对账驱动，用户「按建议立项」）
**状态**: Active——立项，M0 未启动
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
  + Rust `resolve_slots` 消费）——收口时按 Support Envelope 排除，等用户
  点名专项；不在本 goal DC。

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

- **渲染级 composed tree 不进本 goal**：shadow 树进样式/布局/绘制、
  `:host`/`::slotted` 选择器命中、Rust `resolve_slots` 接线——维持
  web-components 收口排除条款（rendering-compat 交界，用户点名专项）。
- **custom-elements 不重开主账**：64.67% 是 web-components 通道在账
  通过率，本 goal 只做同通道扩面时的零回归门 + 滚动新面顺带入账。
- **template 面**归 html-semantics-compat 通道既有账，不动。

## 里程碑

- **M0**：差集甄别 + declarative 重审 + 数值门（DC-1）。
- **M1**：语料扩批 + 通道四件套（DC-2）。
- **M2**：扩面基线分类（DC-3 前半）。
- **M3**：JS 语义簇收敛批（DC-3 后半）。
- **M4**：coverage 复测收口（DC-4）。

## 依赖约束（run-rules §9）

- WC 通道与 fetch-web-components-subset.sh 的 skip 规则两侧同源，改动
  同步；扩面后 custom-elements/template 既有账逐 subtest 零回归。
- 与 rendering-compat（活跃 rally）交界：凡甄别出样式/布局耦合簇，先
  归因落 evidence 再碰头归口，不双改。
- 上游滚动走 pin 对齐 + blob-hash 对账（goal-35 先例：文件数一致 ≠ 内容
  一致）。
