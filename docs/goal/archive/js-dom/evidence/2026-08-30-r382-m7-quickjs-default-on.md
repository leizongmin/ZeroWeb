# R382 — M7 QuickJS `native_dom` default-on（用户批准 GB-20260829 ③）+ 守门工具修复

**日期**: 2026-08-30
**执行序**: zero-web master.md 2026-08-30 批复块 ③——「先 default-off 全量基线 → 翻开关 → A/B net≥0」
**改动面**: `crates/webview/src/webview.rs`（flip）+ `crates/webview/src/tests/coverage.rs`（断言分域）
+ `crates/engine/src/js_dom_shim/part02.js`（DOMException identity）+ `crates/webview/src/tests/service_worker_runtime.rs`（flaky 修复）
+ `tests/wpt-runner/scripts/audit-imported-font-resources.sh`（reftest 前置工具修复）

---

## 1. 步骤① default-off 基线（commit `405623b98` 时点）

| 门 | 结果 |
|----|------|
| quickjs 矩阵（script-sandbox/webview/webview-demo/integration/wpt-runner） | **1676P / 0F** |
| bench-gate 定向（zero-engine/zero-webview/zero-script-sandbox，42 指标） | 空载跑 **41 PASS + 1 已知漂移项**（`webview_load_html_with_css` 1.06×——GB-20260824/CI-GUARD-20260821 记录的 7763/9V74 平台漂移族指标；带载首跑 12-14 FAIL 均为宿主 100% CPU 挂死进程污染，杀掉后空载重跑收敛到 1 项） |

**过程发现（R381）**：建基线时 `vue_mount_lands` 在 main HEAD 失败——归因 R380 commit
`238ecf96e`：pa2b apply 钩子刻意保留 pending 桶（identity 记账），R380 融合 innerHTML
把「桶非空」当「快照 stale」→ apply 后快照已含的子经 overlay 重复并入（双份
`<p class="msg">`——R309 identity 双源教训的代际表述）。修复：桶记账时盖 apply 代际
时间戳（`_pb.stamp`）+ `__zw_apply_generation_bump` 递增代际计数（`_zwApplyGeneration`）；
融合路径仅 `stamp === generation` 时触发。vue e2e 3P 恢复，driving 用例保持 Pass。

## 2. 步骤② 翻开关 + 步骤③ A/B 守门

- **flip**：`WebViewConfig::default()` 的 `native_dom` 在 `quickjs`-only 构建下默认
  `true`（cfg 双分支；V8 路径默认 `false` 不动，M5 独立）。kill-switch 字段保留
  （显式构造以调用方为准；删除留 M7 收尾子片）。
- **断言分域**：`test_native_dom_disabled_by_default_r3097` 拆为 quickjs-only 版
  `test_native_dom_enabled_by_default_quickjs_m7`（断言工厂已安装）+ v8 版（保持
  disabled-by-default 语义）。

### A/B 结果

| 门 | default-off 基线 | flip 后 | 判定 |
|----|-----------------|---------|------|
| quickjs 矩阵 | 1676P / 0F | **1676P / 0F** | 逐位一致，net 0 ✅ |
| reftest（687 案） | （被工具 bug 阻塞，见 §3） | **687/687 100%** | ✅ |
| v8 矩阵（cfg 门控不含 flip） | — | 657P + 1 flaky（修复后 8/8 稳定） | 无关 ✅ |
| bench-gate 定向 | 41/42 PASS（1 已知漂移项） | **GATE PASS 42/42 全在预算内（NEW=0，net≥0）**——空窗复跑法（load1≤1 才起跑）：中间两跑 INCONCLUSIVE（并行流污染，守卫正确触发）与 22 FAIL（loadavg 6.5 带载噪声）均被拒绝 | ✅ |

## 3. flip 暴露并修复的三件工具/测试缺陷（全部 clean-HEAD A/B 归因，非 flip 回归）

1. **part02.js SW register/update reject 的 DOMException identity**：裸 `DOMException`
   解析到 shim 闭包内构造器，而 `instanceof DOMException` 检查解析到 globalThis
   （M7 下是 quickjs_dom_bindings 安装的 native 构造器）→ 恒 false。改
   `new (globalThis.DOMException || Error)(...)`（R9 wrong-global 先例）。
   `navigator_registration` 等 webview quickjs 全量 611P。
2. **`navigator_controller_tracks_document_and_skip_waiting_replacement` flaky**：
   旧 controller 的 `redundant` 转换走宿主 setTimeout（每 timer 一个 OS 线程，无
   FIFO），与 controllerchange 事件派发线程级竞争（同二进制双态可复现）。契约改为
   事件事实即时定格 + `__firstController.state` 变 `redundant` 后（bounded interval
   轮询 2000×5ms）finalize。**8/8 连跑稳定（v8）+ 3/3（quickjs）**。
3. **`audit-imported-font-resources.sh` 假阳性 fatal**：裸 grep `href="…css…"` 命中
   `<script>` 内 JS 字符串字面量（R123 导入的 PI 用例 data 是测试断言文本，
   style.css 上游不存在）→ fetch-wpt-data 自 R3413-F 起对每轮 reftest 致命。改只取
   真实 `<link rel="stylesheet">` 标签。reftest 门解锁：**687/687 100%**。

## 4. bench-gate 空窗复跑记录

| 跑次 | 起跑 loadavg 1min | 结果 | 判定 |
|------|------------------|------|------|
| 1（并行流活跃） | ~8.5 | INCONCLUSIVE（suspect 标记，负载守卫触发） | 正确拒绝 |
| 2（带载窗口） | 6.5 起 | 22 FAIL（跨 engine/webview/page 均匀 2-3×） | 噪声不可信 |
| 3（带载窗口） | 6.6 起 | 1 FAIL（compositing 1.32×，其余 41 PASS） | 部分污染 |
| 4（空窗，load1≤1） | 1.15 | **GATE PASS 42/42（NEW=0）** | **可信，采纳** |

教训：共享机器上 bench A/B 的可信判定不是「跑一次看结果」而是「等真空窗」——
负载守卫只拦 loadavg 超阈的极端态，5-6 的中载窗口照样产出大面积噪声 FAIL；
`load1 ≤ 1` 起跑 + 单跑全 PASS 才是闭合判据。

## 5. 教训

1. **M7 flip 的 A/B 守门抓的全是测试/工具面缺口而非 native 路径行为缺陷**——
   1676P 逐位一致说明 quickjs 原生绑定生产路径与 polyfill 行为等价（DC-7 的
   生产路径形态收敛）。
2. 宿主 setTimeout 线程模型下「timer 间 FIFO」假设不成立——测试断言跨 timer 状态
   必须轮询收敛而非时序假设。
3. bench-gate 负载守卫（INCONCLUSIVE）在双流并行机器上是有价值的正确行为，不可绕过；
   补跑需等并行流空窗。
