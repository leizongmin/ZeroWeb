# M2 扩面基线 + 失败簇归因 + 门值回填（DC-3，2026-11-06）

## 基线方法

- A/B 两轮全通道：`before` = HEAD 规则基线（392 案前账 323 案），
  `after` = 本轮扩规（fetch 脚本扩批 + `wc_case_skipped` 转正收窄）后
  同一 HEAD 引擎二进制全量重跑。两轮均 test-guard 包裹
  （`--per-proc-mem 4 --total-mem 8 --time-limit 3600`）。
- 中间轮 after1 发现 `wc_case_skipped` 遗留 `"shadow-dom/reference"` 前缀
  项吞掉 `reference-target/`（starts_with 前缀碰撞），修复（移除前缀项 +
  精确路径跳过 `reference/empty.html`）后以 after2 为定版基线。
- 证据 JSON（gzip）：`2026-11-06-m2-baseline-before.json.gz` /
  `2026-11-06-m2-baseline-after.json.gz`（同目录）。

## 全通道基线（after，@ pin 3159769338）

| 面 | 案 | 子测试 | P | F | 通过率 |
|---|---:|---:|---:|---:|---:|
| custom-elements | 175 | 3,848 | 2,821 | 1,018 | 73.31% |
| template | 28 | 667 | 583 | 84 | 87.41% |
| sd-top | 64 | 666 | 622 | 43 | 93.39% |
| sd-untriaged | 54 | 251 | 153 | 97 | 60.96% |
| **sd-decl-top（新）** | 20 | 5,334 | 6 | 5,326 | 0.11% |
| **sd-decl-tentative（新）** | 36 | 55 | 1 | 53 | 1.82% |
| **sd-reftarget（新）** | 15 | 142 | 0 | 139 | 0.00% |
| 合计 | 392 | 10,963 | 4,186 | 6,760 | 38.2% |

其余状态：Timeout 9、Unsupported 6、PreconditionFailed 2（合计恰平账
10,963）。

## 零回归核对（A/B 逐案逐子测试）

- custom-elements / template / sd-top：before/after 完全一致，零回归。
- sd-untriaged：56c→54c、253s→251s、155P→153P，F 97 不变——唯二差集
  `untriaged/resources/blank.html`、`bobs_page.html`，即本轮新规则有意
  排除的 fixture 页（拉取不扫描），非回归。
- 结论：**旧面零回归**，扩面为纯增量。

## 新面失败簇归因（M3 输入）

按消息签名逐子测试分类（5,518F）：

| 簇 | 失败数 | 根因 | 证据签名 |
|---|---:|---|---|
| **A：unsafe 解析/序列化 API 族** | ≈5,296（96%） | `Element.setHTMLUnsafe`/`Element·ShadowRoot.getHTML`/`Document.parseHTMLUnsafe` + `createContextualFragment` 零实现 | `wrapper.getHTML is not a function`（2,298）+ gethtml.html 特征检测 `expected false/true got undefined`（2,208）+ setHTMLUnsafe/parseHTMLUnsafe 散点（~790） |
| **B：片段解析路径未物化** | ≈157 | innerHTML/createContextualFragment 路径未接 `materialize_declarative_shadow_roots`（s168） | `The template should be left as a <template> element`（110）+ attachment 断言（~47） |
| **C：referenceTarget** | ≈139 | `referenceTarget` IDL + `shadowrootreferencetarget` 属性未实现 | 特征签名 5 + setHTMLUnsafe 依赖 93 + 宿主结构 null 链 41 |
| **D：adoptedStylesheets/module 链 + template IDL** | ≈54 | module script → constructable stylesheet 载入链、`shadowRootAdoptedStyleSheets` IDL、`shadowrootclonable` clone 物化、template IDL 反射 | modulepreload 共享断言、`Clone has a shadow root`、IDL 反射 assert |
| env（Timeout/Unsupported） | 12 | 环境受限 | — |

簇 A 内部结构：`gethtml.html` 单文件 4,507 子测试（decl-top 84%），
`declarative-shadow-dom-attachment.html` 654，`declarative-shadow-dom-opt-in.html`
119（117F+1P+1T，该页 1P 即既有唯一 decl-top 通过）。

**M3 收敛路径（按翻转规模排序）**：

1. **簇 A+B 同根首刀**：实现 unsafe 三 API（setHTMLUnsafe/getHTML/
   parseHTMLUnsafe）+ 片段解析物化接线，预期翻转 ~5,300 子测试。
2. 簇 C：referenceTarget 语义（IDL + 声明式属性 + 目标重定向），或如实
   挂账并按修订规则重算门。
3. 簇 D：逐项甄别环境受限部分（module 载入链涉及 script-sandbox 面），
   可修项入 M3 尾段，不可修项按实剔池。

## 实测池修正与 DC-4 门值回填

**实测新面池 = 5,334 + 55 + 142 = 5,531 子测试**，显著低于 M0 按
wpt.fyi 分母的估计 ≈14,500。差值归因：wpt.fyi 分母来自全实现浏览器，
同文件生成更多子测试；本引擎缺 API 时页面脚本早抛、后续子测试不再生成。
**对照组证据**：untriaged 域（无 API 依赖）wpt.fyi 分母 251 = 本地实测
251，两侧一致；差值全部集中在 API 依赖域（declarative/reference-target），
与簇归因自洽。

**门值回填（按 M0 修订规则）**：

- M0 初版 75.0%（wpt.fyi 池估计口径）→ **M2 实测修订 31.4%**
  = (621 + 5,531 × 76.0%) / 15,350。
- 全量推进决策维持不变：M0 决策依据是同批文件在 wpt.fyi 全实现口径的
  可达面（≈14.3k ≥ 8k 阈值）；文件集合未变，untriaged 对照组证明可达
  文件两侧一致，本地实测 5,531 是实现态下限，随 M3 实现增长
  （unsafe API 落地后早抛页面将展开更多子测试）。
- M4 收口对账：按届时实测池重算门值、与届时实测通过率对账；池随实现
  展开属如实口径，不属"调门替代收敛"。

## 并行流碰撞记录（run-rules §9）

本轮全程与 editing-compat rally 会话共享同一 clone（各自探针与
`target/release/zero-wpt-runner` 重建；另有 zeroweb-m3-postproc.service
css-v2-m3 watcher 活动）。本轮改动面为 WC 通道规则 + goal 文档，与
editing 通道无共享文件语义冲突；二进制重建存在交错但 before/after 均
以"改动全在位→重建→跑"的顺序自证对应规则源。按 §9 记录碰撞、不强制
消解。
