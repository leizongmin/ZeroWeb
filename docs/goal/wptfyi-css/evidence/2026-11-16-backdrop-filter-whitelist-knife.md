# backdrop-filter 白名单刀——一行门线解锁 filter-effects 语料支持门

日期：2026-11-16（十五轮，wptfyi-css；承接 offset-position 刀 4427b121e 落地后
的树窗）。
改动面（2 文件，+12/−0）：`crates/style-system/src/property/registry.rs`
（known_properties 白名单 + `"backdrop-filter"` 一行）+
`crates/engine/src/js_dom_bridge_tests/part02.rs`（R2785 supports 钉扩
backdrop-filter 双向断言）。

## 机制要点

**目标面**：texp 转化率轮 supports 门桶 `backdrop-filter 943`（filter-effects
语料 `CSS.supports('backdrop-filter', …)` 前置门整页翻红）。法证：五站全在
——apply 臂（apply_advanced.rs:1820，R2762 同 filter 列表解析）、gCS 序列化
臂（bridge computed_style.rs:690 复用 filter_to_css）、WAAPI 插值臂
（animation.rs:530 与 filter 共用 interpolate_filter_list，R4272）、apply 写
回臂（animation.rs:1715）、初始/继承（inherit.rs:1530）——唯一缺口 =
`known_properties()` 白名单缺串（js_dom_bridge.rs `decl_supported` 门 =
`known_properties().contains` + apply 校验，缺串即 supports 短路假）。

**刀体**：白名单 `"filter"` 后加 `"backdrop-filter"` 一行（apply 校验沿
parse_filter_list 文法裁决真值，非法值仍拒）。同型扫描定谳其余 supports 桶
均非门线缺口：columns/counter-reset/offset-anchor 已全在；mask-border/shape-
outside/calc-size/row-rule 为真实现缺口（多站刀，后置）。

**css-motion-1/filter-effects 参考**：<https://drafts.fxtf.org/filter-effects-2/#BackdropFilterProperty>；
白名单门先例 = offset 簇刀（CSS.supports decl_supported 前置门）。

## 单测

- engine `js_dom_bridge::tests::test_css_escape_supports_r2785` 扩
  backdrop-filter 断言：`CSS.supports('backdrop-filter','blur(2px)')` 真 +
  `'bogusxyz'` 假（门线双向 + apply 文法双向）。定向腿 1 passed
  （zw-bdf-pins3，test-guard 包裹）。
- fmt 权威门（`cargo fmt -p zero-style-system -p zero-engine -- --check`）
  0 Diff；手撸 rustfmt --edition 2024 对 part02.rs 27 处假漂移 = 配置解析
  差异（trailing-msg assert 展开），cargo fmt 同二进制同配置判净——权威门
  作准。clippy 两 crate 净（唯一 warning = peer es_module
  too_many_arguments 在册连坐）。

## A/B 双面账（干净二进制重放）

- B0 = `bin-bdf-b0/zero-wpt-runner`（md5 e2fe2a8d）= bin-posp-b1 快照，
  其构建树 ≡ 落地提交 4427b121e 内容（knife-2 落地法证：提交 = 该 WIP 树
  原样入册），行为基线等价。B1 = `bin-bdf-b1/zero-wpt-runner`（md5
  37eb8df2）：WIP 树（白名单一行 + 钉）重建。哈希异 = 刀已入 B1。
- 批（zw-bdf-ab.service，00:19–00:31）：filter-effects（主面）+
  css-typed-om/cssom（白名单触面护栏）× {b0,b1}，test-guard 包裹，
  transcript 唯一名 `/tmp/zw-bdf-{side}-{corpus}.txt`，JSON 分目录
  `results-bdf-{side}`。覆盖面对称：33/359/177 页两侧全同（2,452/12,359/
  3,479 子测），免 corpus 面不等假差。

**结果（B0 → B1，三 corpus）**：

| corpus | gained | lost | regressed | pages_gain |
|---|---|---|---|---|
| filter-effects | +253 | 0 | 0 | 6 |
| css-typed-om | 0 | 0 | 0 | 0 |
| cssom | 0 | 0 | 0 | 0 |
| **合计** | **+253** | **0** | **0** | **6** |

- filter-effects +253 明细：backdrop-filter-interpolation-001..004
  +218（59/35/72/52——WAAPI 插值链与 composition 通道既有，门开即通）、
  parsing/computed +25、animation/composition-001 +10。
- 护栏 typed-om/cssom 0/0/0：白名单纯增量一行，触面零外溢。
- 残面注记：texp 943 桶全量面含散页（css-view-transitions/css-will-change/
  css-borders 等他 corpus backdrop 页 ~690），本批三 corpus 外页面由全量
  corpus 跑自然入账，未在本刀 A/B 断言面内。

## 门禁（zw-bdf-mkt.service 拆腿，00:32–00:4x，test-guard 包裹）

| 腿 | 结果 | 归账 |
|---|---|---|
| companion-build | 0 | — |
| workspace-test | 10097P/**1F** | `network_loading::same_origin_image_burst` 负载 flake——R5288/R5292/R5385 同族 ≥4 先例（peak=7 limit=6 形，standalone 0.61s 过），本窗 peer editing-run 电池（102% CPU）并发；solo 复跑 discharge 见下 |
| quickjs-clippy | 101 | peer es_module.rs `too_many_arguments(8/7)` s419 WIP 在册连坐 |
| renderer-bin | 0 | — |
| renderer-lib | 371P/**1F** | `js_worker::tests::renderer_js_worker_fetch_without_handler_resolves_error` 在册 worker-fetch renderer 形 |
| browser-bin | 741P/**2F** | tab 镜像 worker-fetch + `default_actions_work_without_javascript` 在册 |
| browser-tests | 771P/**2F** | 同上两形 |
| gpu-adapter / gpu-consistency | 0 / 0 | adapter 96P |
| quickjs-workspace | 775P/**1F** | `image_batch_e2e` 环境形红（缺件/负载峰值在册，solo ×3 前轮定谳） |
| quickjs-renderer-bin | 0 | — |
| quickjs-renderer-lib | 369P/**1F** | worker-fetch quickjs 形（在册，351.47s，与 mkt3 逐名同） |

**discharge 补账（01:0x）**：`same_origin_image_burst` solo 复跑 **1 passed
0.61s**（zw-bdf-discharge unit，test-guard 包裹，R5293 同签名 0.61s）——
四分类定谳 = 负载 flake，非本刀域红。es_module 六树态红本轮未现（peer s419
WIP 演化，仅 clippy lint 连坐余留，在册）。

刀域（style-system / engine 钉面）零新增红：`test_css_escape_supports_r2785`
（含本刀 backdrop 扩断言）与 R2762 gCS 钉 workspace 腿全绿。fmt 权威门
`cargo fmt -p zero-style-system -p zero-engine -- --check` 净收口（探针
文件删除后 rc=0）。
