# R385 — js-dom goal 收官判定矩阵（DC-1~DC-8 逐项 verify）

**日期**: 2026-08-31
**HEAD**: `90ac9c654`（R385 收官轮）
**性质**: 终局输出协议（goal 入口文档 Final Output Contract）要求的 DC 逐项对照

---

## DC-1: 原生 DOM 为双引擎唯一生产路径（架构闭环）— ✅ 全项满足

| 条目 | 状态 | 锚点 |
|------|------|------|
| `ZW_NATIVE_DOM` 双 feature 默认开启 | ✅ | `WebViewConfig::default()` v8 分支 `native_dom: true`（R384，用户批准 2026-08-19）；quickjs-only 分支 `true`（R382）。生产 `run_page_scripts` 双 feature 均默认 `install_native_dom_bindings`（webview.rs:1703） |
| kill-switch 已移除 | ✅ | R384b（commit `222b2d6c9`）：env 读点 / `native_dom_enabled` / `install_dom_bindings_if_enabled` / config 字段 / builder / webview 五守卫 / Drop 无条件化 / runner+testharness 消费点全删；`native_dom=false` 回退路径不存在 |
| V8 L2 polyfill-live 合一 | ✅（实用收口） | R296–R359 L2-d3 路线 A 全切片（R359 定案 d3d-r3/d3e 实用收口点）+ R309/R310/R322 查询读 live + R380 sel 域融合 innerHTML；「持续萎缩的 polyfill 桥」保留为 M4 测试资产消费面（设计取舍，非未完成项） |
| V8 S6 高层 API 去字符串 | ✅（superseded） | R361 定案：default-on 使页面 globalThis 由 native 供给，S6「在将被删除的路径上建一次性桥」失去对象（RFC v0.3 已同步） |
| S7 polyfill 桥死代码删除 | ✅ | kill-switch 回退路径死代码随 R384b 全删；`__zw_*` 回调剩余部分是 M4 基线（55808 subtest）与既有 e2e 的消费面，其萎缩随上游用例面收敛持续（RFC v0.3 §修订历史） |

## DC-2: 真实 SPA / Web Components 端到端跑通 — ✅ 全项满足

- Vue 3 e2e：`tests/integration/tests/e2e_vue_library.rs`（mount / reactive+event / reconciliation），v8 + quickjs 双 feature 3/3（R100 + R339）；R385 make test 18502P 内含复验绿。
- Web Components：`e2e_web_components.rs`（8 组件组）+ lit 库 `e2e_lit_library.rs`（6/6），`fixtures/lit/lit.bundle.js` + `fixtures/vue/vue.global.js` vendored。
- 验收资产常驻 `tests/integration`，`make test` 内运行，断言可复现。

## DC-3: WPT dom 上游基线 — ✅ 全项满足

- 上游真实用例导入：`tests/wpt-runner/wpt-data/dom/`（abort/collections/events/lists/nodes/ranges/traversal 8 子目录 + 根散用例；**0 内建充数**）。
- 按子分类通过率报告：[2026-08-31-r385-m4-dc3-per-subdir-report.md](2026-08-31-r385-m4-dc3-per-subdir-report.md)（55,808P / 12F / 15T，subtest 通过率 **99.95%**；Fail 集合与 R380/R384 定档恒等零新增）。
- 账本：`imported-tests.txt`（js-dom 条目 72+）；每修复经 `make import-wpt` 资产化。
- evidence/ 持久化：130+ 报告文件（R1 起 41.25% → 99.95% 全程可追溯）。

## DC-4: 测试与质量不可退让 — ✅ 全项满足

- `make test` **18502P / 0F 全绿**（R385；含 Xvfb 下 zero-browser 411P——35 轮 X11 环境项收口；无任何「已知失败」豁免项）。
- clippy 双矩阵（v8 workspace + `--features quickjs`）`-D warnings` 零警告。
- dom_bindings 覆盖率：源码 **94.07%** / 全部 95.87%（R385 实测，vs M0 基线 93.14% 持续提升）。
- 每迁移/修复带单测 + A/B 对照（`tests_ab_compare.rs` 双 feature 参数化骨架 R0 起）+ driving WPT 资产化。

## DC-5: 性能不退化 — ✅ 全项满足（R385 收口）

- perf-gate default-on 后对照：真空窗（load1=0.2）**GATE PASS 38/38（NEW=0）**，与 M7（R382）A/B 判定同口径（`ZERO_WEB_BENCH_CRATES=zero-engine,zero-webview,zero-script-sandbox`）——[evidence](2026-08-31-r385-dc5-bench-gate-closure.md)。
- JS→DOM 桥基准持续记录：webview 三指标全 PASS（complex_page 528µs/inject_css 157µs/resize_and_render 143µs，均在基线预算内）；native 读 ~15.6x（RFC §4 S0 bench）锚点保留。
- 全量 113 档的基线陈旧（geo-mean 1.5x，9 天/230+ 提交）归 GB 流常规 re-capture，非本 goal A/B 判定面（噪声签名三证见 evidence §2）。

## DC-6: 文档治理就位 — ✅ 全项满足

- master.md 自洽（active milestone 收敛态 / 状态表 / evidence 账本 / 下一步互不矛盾；本轮同步 R385 记录）。
- archive/ 108 文件（milestone/切片全程归档）；evidence/ 130+ 文件。
- RFC `p1b-v8-native-bindings-rfc.md` **v0.3**：TBD-5 已更新为「双引擎」并关闭；S6/S7 状态终局对齐（本 R385 落地）。

## DC-7: QuickJS 原生绑定等价 — ✅ 全项满足

- rquickjs 原生 DOM 绑定镜像 V8 S0–S5：`crates/engine/src/quickjs_dom_bindings.rs`（3084 行）S0q–S5q 全量（R57–R78）；escape-hatch `install_native_bindings_quickjs` + webview 接线。
- GC/生命周期：strong+reset 换代唯一正确形态定案（R74；rquickjs 缺 finalizer 记 TBD 不阻塞）。
- 双 feature A/B：WPT 双路径对等差 0.02pp（R76）；driving 测试在 v8/quickjs 两矩阵均绿（R385 make test：quickjs 矩阵 763+612+144+126 全绿）。
- QuickJS 页面路径 default-on 走 native（R382 flip），不经 `__zw_*` polyfill 桥。

## DC-8: Canvas path-objects JS 侧 API 语义 — ✅ 全项满足

- 用例导入：205 用例入库 `wpt-data/html/canvas/element/path-objects/`（R56→R56i 九轮）。
- roundRect panic：total_cmp 修 NaN 排序（R56，backtrace 实证根因）。
- DOMPoint 断言精度/形状精度/语义校验：R56–R56f 切片族（nonzero fill rule/arcTo 切线/arc 环带/曲线自适应/isPointInPath rule）。
- 收敛复核：**202P/0F/3 NotRun**（R337 勘误：3 NotRun = 套件互斥 skip + reftest 格式面，无 Fail）；报告持久化 evidence/（m8-slice-* 9 文件）。

---

## DONE 允许条件对照（Final Output Contract §DONE）

1. ✅ DC-1~8 全部满足（本矩阵）
2. ✅ 目标能力达生产可用质量：双引擎 native 唯一生产路径（default-on + kill-switch 删除）+ Vue/lit/WC e2e 跑通 + WPT dom 基线 99.95%（55,808 subtest，0 内建充数）
3. ✅ 真实代码/测试/验收证据直接对应目标能力（本矩阵逐项锚点到 commit/文件/测试名）
4. ✅ `make test`（含 quickjs 矩阵）18502P/0F + clippy 双矩阵 `-D warnings` 零警告 + product-smoke struct 六门 PASS（R384）+ bench-gate GATE PASS 38/38（R385）
5. ✅ master.md 自洽，archive（108 文件）+ evidence（130+ 文件）已建立，milestone 全部归档
6. ✅ RFC v0.3 与 master.md 一致；TBD-5 已更新为双引擎并关闭

**判定：DONE 允许条件 1–6 全部满足。**

---

## R386 附录（2026-08-31 续轮）— DC-1「唯一生产路径」多进程面补片

R385 收官后接续核对发现一处 verify 盲区：DC-1 的 default-on/kill-switch 子项已闭合，
但「生产路径默认安装并使用原生绑定」实际只覆盖 **webview 进程内路径**——生产浏览器的
页面脚本走多进程 worker 沙箱（`tab_js_worker` / renderer `js_worker`），二者从未调用
`install_native_bindings*`，页面 JS↔DOM 仍只走 polyfill 桥（`external_script` 早 return
使 webview 安装面不可达）。

**R386 补片**（commits `513a1345d`/`ff092140b`/`a3b2d7459`；详见
[2026-08-31-r386-multiprocess-worker-native-bindings.md](2026-08-31-r386-multiprocess-worker-native-bindings.md)）：

- worker bootstrap 从 `dom_html` 快照 re-parse 安装原生绑定（双引擎 escape-hatch）；
- `SetDomSnapshot` refresh-only 快路径（engine 新 API；quickjs 全量重 install 会覆盖
  shim `Event`/`DOMException` 全局——`form.reset` preventDefault 回归实证后修复）；
- `ResetDocumentState`/`Shutdown` 清绑定线程局部（R3334/R74 悬垂家族 worker 版）；
- 顺带修 renderer js_worker v8+quickjs 组合态 `js_config` 双 move（R84 同款）。

**验证**：`make test` 18504P/0F（EXIT 0）；worker native 测试双 feature ad-hoc 全绿；
clippy v8/quickjs/组合三矩阵零警告。DC-1 多进程生产路径形态与 webview 路径对齐。
