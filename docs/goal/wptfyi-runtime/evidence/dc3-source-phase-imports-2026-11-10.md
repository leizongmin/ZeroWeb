# DC-3 · source-phase imports 切片（+90 过，翻红 0）

**2026-11-10** · commit `91ab28c56`（`crates/script-sandbox/src/v8_runtime.rs` +48）

## 背景：DC-3 首跑对账缺口 61

weekly run [42801058132](https://github.com/leizongmin/ZeroWeb/actions/runs/42801058132)
（HEAD `3bfccdfa3`）快照已由 cron 落库
（[coverage-2026-11-10](../../../compat/wptfyi/coverage-2026-11-10.md)，与
`/tmp/dc3-artifacts/wptfyi-coverage/` 产物逐字一致）：test262 51,838 +
encoding 1,151,530 + wasm 190,526 = **1,393,894**，对验收线 1,393,955
**差 61**。三目录拆解：

| 目录 | DC-2 账面 | 快照实跑 | 差 | 定性 |
|---|---|---|---|---|
| encoding | +12,979 预告 | +12,979 | 0 | 精确对上 |
| wasm | +1,122 | +1,115 | −7 | 上游漂移（已过路径上游侧变动），不可归因本仓 |
| test262 | 51,948 | 51,838 | **−110** | Timeout 逐分片与 DC-1 持平（排除 CI 慢速截断）；失败案逐条归类见下 |

test262 失败簇归类后，可回收候选三个：**A** 模块加载 fetch-before-parse
顺序缺陷 18 案（触碰 encoding/wasm/dom 共享 module 管线）；**B** `$262.gc`
→ `TestUtils.gc()` 悬空引用 11 案（上游 wpt resources
test262-provider.js @ 8c04fb8d43 逐字同款 bug，`--expose-gc` 因改变产品
页面全局形状/fingerprinting 被否决，gc 桥需 webview→page-runtime→engine
→script-sandbox 四层 `register_scoped_callback` 接线）；**C**
source-phase imports 簇 91 案（本轮切片）。另 S11.13.1_A5_T\* 族
（with 语句 PutValue）62 案经 node 24（V8 13.6）对照同样失败——V8 引擎
侧不可回收，非本仓回归，排除。

C 簇 +90（F2P 实测）即覆盖 61 缺口；A、B 两簇合计 ~26 收益但共享面
代价高，defer 到下批，本轮不动。

## 方案（两件，均在 script-sandbox `v8_runtime.rs`）

1. **V8 flag 开启**（`ensure_v8_initialized()`）：`--js-source-phase-imports`
   ——V8 内核已实现 source phase imports（ECMA-262 §13.3.10.1.1
   EvaluateImportCall phase=source），本构建默认关旗。平台初始化前
   `set_flags_from_string` 开启，经典页/worker/renderer 全 isolate 一致
   生效（ShadowRealm `--harmony-shadow-realm` 同款先例）。
2. **with-phase 宿主回调＋phase 分派**：
   `set_host_import_module_with_phase_dynamically_callback` 注册
   `dynamic_import_source_phase_callback`——`kEvaluation`/`kDefer` 原样
   委托既有 `dynamic_import_callback`（行为与注册前恒等），`kSource`
   按 ECMA-262 §16.2.1.7.2 GetModuleSource 的 SourceTextModule 实现恒
   abrupt：`PromiseResolver` reject `SyntaxError`「source phase import is
   not available」。本引擎无 source-phase module record（`import.source`
   成功面宿主 API 不存在），FIXME 标注保留成功面挂账。

**V8 150 实测文档偏差（重要）**：rusty_v8 文档称 with-phase 回调
「only called for source phase」，实测注册后该回调接管**全部**动态
import，旧 `set_host_import_module_dynamically_callback` 不再触达。
第一版回调无视 phase 一律 SyntaxError 拒绝，dynamic-import 面
792P/149F → 675P/266F（所有普通 `import()` 模板全被误拒）；按 phase
分派后恢复。此后任何宿主动态 import 逻辑改动必须走 with-phase 回调
内的 phase 分派，不能假设旧回调仍在管线里。

## A/B（release runner，test-guard 包裹）

| 面 | BEFORE（CI `3bfccdfa3`） | AFTER（本轮本地） | 净增 |
|---|---|---|---|
| dynamic-import 面 | 792P/149F | **882P/59F** | **F2P +90，P2F 0** |
| module-code 面 | — | 逐案比对 | 零漂移 |
| import 面 | — | 逐案比对 | 零漂移 |
| staging 面 | — | 逐案比对 | 零漂移 |
| annexB 面 | — | 逐案比对 | 零漂移 |
| encoding 探针 | — | 全 Pass | 零归因回退 |
| dom 探针 | — | 全 Pass | 零归因回退 |
| wasm 探针 | — | 1,218/1,218 Pass（filter `constructor`；`json_custom_section` 误 filter 0 命中系探针写法，非回归） | 零归因回退 |

伴随验证：script-sandbox 452 测试全绿、runner 319 测试全绿、
`clippy -p zero-script-sandbox --all-targets -- -D warnings` 干净（守卫
包裹）。intl402 本地重跑放弃：设 time_limit 600s 低于 DC-1 实测 660s，
test-guard 会杀进程产出空 JSON；该面无 `import.source` 使用，构造性
不受影响，由 CI 全量覆盖。

## 预期与观察点

新 dispatch run
[42843305476](https://github.com/leizongmin/ZeroWeb/actions/runs/42843305476)
（HEAD `91ab28c56`，`run_wptfyi_coverage=true`）：预期 test262
51,838 + 90 = **51,928**，三目录合计 **1,393,984 ≥ 1,393,955**，余量
仅 +29——上游漂移掉的已过路径、CI 分片截断（builtins 预算 9000s 是
主要观察点）与 F2P 本地/CI 方差均按 −1/案吃余量。收尾后按控制面四步
对账：快照落库核验 → 三目录净增对账 → compare-topdirs 对 11-07 基线
与 11-09 快照双基邻域核对 → 守卫证据归档。

## 挂账（不入账，定性不变）

- gc 桥 +11：四层 `register_scoped_callback` 接线（跨共享面），下批。
- 模块加载顺序 +18（C 簇余 1）：fetch-before-parse 顺序缺陷，触碰
  encoding/wasm/dom 共享 module 管线，下批。
- S11.13.1_A5_T\* with-PutValue 62 案：V8 引擎侧（node 24 同败），不回收。
- defer 簇 5+71 案、tco 34（V8 未实现 PTC）、TLA 残余 15、ShadowRealm
  残余 5（1 kernel 侧 + 4 skip）、`import.source` 成功面（无宿主
  source module record，恒拒绝语义忠实 §16.2.1.7.2）等既有定性不变。

## 邻域回退核对（首跑快照 11-10 对双基 11-07/11-09，③步预做）

方法：coverage-2026-11-{07,09,10}.json（corpus revs 三快照全同
`04067ce9/28c152b7/31597693/cb6cc97a`；wpt.fyi aligned run rev
`8c04fb8d43`→`792c7512e3`→`77787fe5fe`）按 topdir 汇总 zw_passes 三方
比对，凡 10→09 回退的 topdir 再下钻 zw-results 产物逐路径比对
（`Pass` 串大小写敏感，`status=='PASS'` 恒 0——首查曾因此漏报）。

结论：**无可归因于 ZeroWeb 行为面的回退**。全部回退项定性：

| 回退 | 量 | 归因 |
|---|---|---|
| html | −95（semantics −96 + canvas +1） | **共享面碰撞**——gitignored 语料树内被提交的 3 个 probe `.html`（`42274e6e9`，html-semantics-compat 流 11-10 落库）触发 fetch lib.sh:62-67 幂等快路径（目录已含 `.html` 即跳过 API 列目录）：新 CI checkout 的 `the-script-element/module/` 只有 probe、64 个上游 module 用例永不落盘 → −126 Pass；叠加 +33 真实增益（closedby-complex +10=`b9f00f5b4`/s371、script-crossorigin-network +8=`a475060dc`/s383、dialog 簇 +7=s373 段施工、probe +3）与 web-components 通道 the-template-element −3 ≈ −96。11-09 无 probe、目录空、fetch 正常拉齐（126P/5F/8T 全记录）。修复归该流，本流不动（见 master.md 碰撞记录） |
| web-animations | −1 | CI 时钟采样方差：`DocumentTimeline/constructor.html` 一子测 Pass→Fail，`assert_approx_equals: expected 76415.8 ±0.001 but got 76415.7`——亚毫秒容差撞采样抖动，文件数/其余子测零变化 |
| wasm | −7 | 上游漂移（首跑对账节已记） |
| test262 | −110 | Timeout 持平 + 失败簇归类（背景节已记） |

其余 topdir 全平或净增：dom +120、editing +2、encoding +12,054、
wasm 通道 +1,122、css +17,624、shadow-dom +935（11-09 快照自身 −981
系该 run 分片波动，11-10 已回收）、WebCryptoAPI +800。终局快照
（run 42843305476）落地后按同方法复核一遍收口 ③。
