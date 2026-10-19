# 复跑第二轮 — iso-2022-jp 残差清偿 + wasm 通道原生形态还原

**日期**: 2026-10-19（rally 编码轮，收口后第二轮复跑）
**套件**: `make testharness-encoding`（全语料 36 案）+ `make testharness-wasm`（jsapi 31 案）
**原始数据**: 本轮 `/tmp/encoding-rerun.log`（修前核对）、`/tmp/encoding-full2.log`（修后首跑）、
`/tmp/wasm-rerun.log`（回归态）、`/tmp/wasm-fixed.log`（修后）、`/tmp/encoding-final3.log`（终态二跑）

## 结果

| 通道 | 修前（10-19 首轮终态） | 本轮终态 | 变化 |
|---|---|---|---|
| encoding 全语料 | 12071/12075 = 99.97%（残差 4） | **12072/12075 = 99.98%（残差 3）** | iso-2022-jp ×1 清偿 |
| wasm/jsapi（跨 goal 核对） | 未跑成（10-17 起回归态） | **715/719 = 99.4%** | 基线逐字还原 |

## 发现 1：iso-2022-jp fatal-stream 残差是本方 shim 缺陷，非 encoding_rs 契约

- **原记账**：「encoding_rs iso-2022-jp fatal-stream 状态契约 ×1（随升级重评）」——归因有误。
- **spec 事实**（Encoding Standard `decode()` 步骤 1）：decoder 重置只发生在 `do not flush`
  未置位（非 stream 调用入态）时；stream 中途 fatal 抛错**不重置** decoder，转义态保留
  （iso-2022-jp 是唯一状态合法存续的解码器，WPT 测试注释明示）。
- **本方缺陷**：shim part02.js legacy host 路径在 fatal 错误分支**无条件重建 handle**
  ——stream 面错误同样重建，丢失 ESC ( J / ESC ( I 转义态；flush 后 0x7e 得 `~`
  而非 `‾`（Roman）/ 0x42 得 `B` 而非 `ﾂ`（半角片假名）。
- **修复**：`if (!streamFlag)` 才重建 handle（flush 面重建仍等价 spec reset——下次
  decode 步骤 1 本就重置）；stream 面保留 handle 续用原转义态。host 侧
  text_encoding.rs 注释同步（原「spec fatal 错误后 decoder 不再可用」表述系误读）。
- **验证**：textdecoder-mistakes 全案 85/85 绿（含 utf-8/utf-16 既有 fatal-stream 面
  零回归——它们的错误均在 flush 面或 JS 状态机等价重置路径）；全语料 12072/12075。
- **常驻锁定**：`js_dom_bridge_tests::test_text_decoder_legacy_labels_encoding_compat_m2`
  增 fatal-stream 转义态保留断言（corpus 文件系 gitignored 拉取资产，本地单测等价锁定）。

## 发现 2：wasm/jsapi 通道塌缩 141/460——10-19「恢复性」判断对 jsapi 面不成立

- **背景**：10-19 首轮证据判断「桥 Memory 回挂即还原 10-17 前页面环境形态」——对
  sab.js 消费者（只需 `WebAssembly.Memory`）成立，对 wasm/jsapi 通道**不成立**。
- **回归态实测**：`make testharness-wasm` 141 Pass / 319 Fail / 分母塌缩 460（基线
  715/719）。失败整面性（Branding/name/length/Empty buffer 等最基础接口全灭）。
- **根因**：dom polyfill 的 WebAssembly 段（`__WASM_BRIDGE__` 宿主自动桥接，生产
  execute 路径既有契约）**整体替换**原生 V8 WebAssembly；桥仅 stub 实现（Module 为
  描述对象、exports 空数组、参数映射受限）。page-wasm 基线 715/719（2026-09-12）系
  runner 预热走裸 `execute_script`（无 dom polyfill）时以**原生 V8 接口面**跑出；
  `b785548dc`（2026-10-17）预热改走 `execute_script_with_dom` 后桥覆盖原生，通道塌缩。
- **修复（生产桥契约不动）**：
  1. dom_bridge.rs 桥安装时把覆盖前原生引用以不可枚举键 stash 到桥对象
     （`Object.defineProperty(globalThis.WebAssembly, '__zwNative__', …enumerable:false)`）；
  2. runner testharness.rs 预热（with_dom）后经锚点整面回挂原生（还原 b785548dc 前
     全部 runner corpus 的页面环境形态）。不能在 with_dom 前加裸 `execute_script`
     预存——会先装 shim 丢失 b785548dc 的 SecureContext 页锚初始化语义。
  3. shim 无任何 WebAssembly 引用（grep 实证），回挂无捕获时序风险。
- **验证**：`make testharness-wasm` 715 Pass / 0 Fail / 4 Timeout（Timeout 与基线记账
  同四案，分母 719 逐字还原）；`make testharness-encoding` 12072/12075 零回归
  （sab.js 面原生 Memory 语义与 10-19 桥回挂等价）。
- **常驻锁定**：`dom_bridge_tests::test_polyfill_webassembly_native_reference_stashed`
  （stash 锚点 + 不可枚举双断言；restore 语句与锚点键名双向锁定——单侧改名即测试红）。
- **生产语义**：不回挂（runner 专属），浏览器页面继续走 `__WASM_BRIDGE__` →
  webview host → wasm-sandbox 底座；锚点键不可枚举，不进接口面遍历。

## encoding_rs 升级重评（重入条件结账）

- 上游 0.8.35 → 0.8.42（2026-09-24）含 legacy multibyte streaming chunk-boundary
  修复（`0280af8e0`，PR #126）与 iso_2022_jp.rs 改动，但 **0.8.40+ 全系要求
  rust 1.88**（crates.io rust_version 实证），工作区 MSRV 声明 1.85——升级需 MSRV
  政策联动，属用户拍板面。
- **残差已由本方 shim 修复清偿，升级不再绑定任何残差**——维持挂账，动机除名；
  未来 workspace MSRV ≥ 1.88 时可顺带评估（streaming 修复对本仓空块 guard
  workaround 或有化简空间，届时重审 text_encoding.rs:90 注记）。

## 修后残差（3/12075 = 0.02%，全为 transfer/SAB 结构面，记账不变）

| 案 | 归属 |
|---|---|
| streams/decode-utf8 ×2（transferred Uint8Array/ArrayBuffer chunk） | transfer 结构面（workers/transfer 域） |
| textdecoder-copy ×1（SharedArrayBuffer 变体） | 同上（SAB infra） |

## 门禁

- cargo fmt --all -- --check 零 diff；clippy `-p zero-engine -p zero-wpt-runner
  --all-targets -D warnings` 零告警（test-guard 包裹）。
- make test 全绿（exit 0）。
- 单测：`test_text_decoder_legacy_labels_encoding_compat_m2`（增 fatal-stream 断言）、
  `test_polyfill_webassembly_native_memory_preserved`（既有）、
  `test_polyfill_webassembly_native_reference_stashed`（新增）全绿。

## 交接清单

- runner 页面环境现恒为「dom polyfill（DOM/桥面）+ 原生 WebAssembly（整面）」——后续
  goal 复跑遇 wasm/jsapi 形态 Fail 先核对 `WebAssembly.__zwNative__` 锚点链路是否完整
  （桥对象键 `__zwNative__` + testharness.rs restore 语句，双向改名即断）。
- wasm/jsapi 通道回归监控建议：跨 goal 触碰 dom_bridge.rs wasm 段或 runner 预热序列的
  变更，随变更跑一次 `make testharness-wasm`（语料本 clone 已驻留，通道分钟级）。
