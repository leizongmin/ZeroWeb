# 收口后复跑 — SAB 面回归发现与修齐（+ fetch 资产缺口补齐）

**日期**: 2026-10-19（rally 编码轮，收口后例行复跑触发）
**套件**: `make testharness-encoding`（WPT 315976933870b34d6ea30e3f6643403edae678ba，全语料 36 案）
**原始数据**: 本轮 `/tmp/encoding-final.json`（复跑命令 `./target/test-guard --per-proc-mem 4 --total-mem 8 --time-limit 900 -- ./target/release/zero-wpt-runner testharness-encoding --json`）

## 结果

| 指标 | 收口（2026-10-02 M3/M4） | 回归态（本轮初跑） | 修齐后（本轮终跑） |
|---|---|---|---|
| subtests | 12070/12075 = 99.96% | 1402/1477 = 94.92%（分母塌缩） | **12071/12075 = 99.97%** |
| 残差 | 5（transfer/SAB ×4 + encoding_rs ×1） | 75 | **4**（transfer/SAB ×3 + encoding_rs ×1） |

**净 +1 正漂移**：encodeInto detached-output 残差 ×1 被收口后其他流的引擎语义修齐
顺势清偿（encodeInto 110/111 → **111/111**）——非本 goal 改动，跨 goal 正漂移记账。

## 回归发现（两独立根因）

### 根因 1：runner 页面环境 WebAssembly.Memory 被桥接对象覆写（主因，跨 goal 回归）

- **形态**：`/common/sab.js` 的 `createBuffer('SharedArrayBuffer')` 全部抛
  `sabConstructor is not a constructor`——encodeInto SAB 变体 ×54、
  textdecoder-streaming SAB 变体 ×16、decode-utf8 整案脚本抛错（分母 12→1）、
  textdecoder-copy SAB ×1，合计约 −80 案。
- **探针定位**（临时 zz-rally-probe.any.js，用后即删）：window 通道页面环境
  `typeof WebAssembly === 'object'` 但 `WebAssembly.Memory === undefined`、
  键集 `_modules,_instances,…` = dom_bridge 桥接 polyfill 形态；`SharedArrayBuffer`
  本身存在（V8 原生）。收口时（M1 JSON 证）同一环境跑的是真 SAB 读写断言。
- **引入点**：`b785548dc`（2026-10-17，webcrypto M6 残留收口）把 runner 案例预热从
  `execute_script` 改 `execute_script_with_dom`（testharness.rs:7849，修
  [SecureContext] 页锚初始化竞态）——`execute_script_with_dom` 前置注入
  `generate_dom_api_polyfill()`，桥按既有契约覆盖原生 V8 WebAssembly，而桥不实现
  `Memory` 构造器。2026-10-17 起所有经 runner 页面路径的 corpus 的 SAB 面皆中招
  （当前盘面仅 encoding corpus 消费 sab.js——见根因 2 的同族盘点）。
- **修复**：dom_bridge.rs 桥安装时回挂原生 Memory 构造器（`__zwPrevWasm__.Memory`
  保留到桥对象；compile/instantiate 宿主桥语义不变）。常驻断言
  `test_polyfill_webassembly_native_memory_preserved`（dom_bridge_tests.rs）。
  修复后探针：桥键集含 `Memory`，sab.js 探测链路恢复。

### 根因 2：fetch 资产缺口——/common/subset-tests.js 未显式补拉（本 goal 资产域）

- **形态**：api-invalid-label（3421 子测试）+ textdecoder-fatal-single-byte（7168）
  两案整案塌缩为单个 `load support /common/subset-tests.js` Fail（`No such file
  or directory (os error 2)`），分母 −10587。
- **根因**：`scripts/goals/30-encoding-compat.sh` 只显式补拉 `common/sab.js`；
  subset-tests.js 系 M2 时点 wpt-data 的既有驻留（其他 goal fetch 带入），非本
  脚本拉取面——资产不自足，fresh clone 不可复现收口分母。
- **修复**：fetch 脚本照 sab.js 先例显式补拉 `fetch_raw "common/subset-tests.js"`，
  注记两案消费关系。修复后两案复原：api-invalid-label 3421/3421、
  textdecoder-fatal-single-byte 7168/7168。

## 修齐后残差（4/12075 = 0.03%，与收口分类同族收敛）

| 案 | 归属 |
|---|---|
| streams/decode-utf8 ×2（transferred Uint8Array/ArrayBuffer chunk） | transfer 结构面（workers/transfer 域，重入条件不变） |
| textdecoder-copy ×1（SharedArrayBuffer 变体） | 同上（SAB infra） |
| textdecoder-mistakes ×1（fatal stream: iso-2022-jp） | encoding_rs 上游契约（随升级重评） |

encodeInto detached-output ×1 清偿（跨 goal 正漂移，见上）；transfer/SAB 结构面
残差由 4 收敛为 3（encodeInto 臂清空）。

## 跨域漂移核对（run-rules §10）

- **wasm/jsapi 通道**（zero-web 父目标 page-wasm 已归档面，收口 715/719 = 99.4%）：
  Memory 回挂使该通道页面环境从「桥无 Memory」（10-17 起回归态）恢复「桥 + 原生
  Memory」。本轮 `make testharness-wasm` 实跑未完成——wpt-data 语料本 clone 未
  驻留，fetch 遇网络超时（curl 28，30s 零字节）中止。方向判断为恢复性：回挂即
  还原 10-17 前（各 corpus 上次跑绿时）的页面环境形态，且对桥对象仅属加属性
  变更。留重跑注记：下次 wasm 通道复跑时核对 715/719 基线。
- net-api 六 corpus / 渲染 reftest：本修复不触渲染域与 net 协议面，未复跑
  （Makefile 门禁由 make test 覆盖接口面）。

## 门禁

- cargo fmt --all -- --check 零 diff；runner release 重建通过。
- clippy `-p zero-engine --all-targets -D warnings` 零告警（test-guard 包裹）。
- make test 全绿（exit 0，76 个 `test result: ok` 段，零 failed）。
- 修复面常驻断言：`test_polyfill_webassembly_native_memory_preserved`
  （engine lib 130 dom_bridge 测试同批全绿）。

## 交接清单

- 桥 Memory 回挂为 engine 共享面修复，惠及所有 sab.js 消费 corpus；后续 goal
  复跑遇 SAB 形态 Fail 先核对 runner 页面环境 `WebAssembly.Memory` 是否在位
  （探针模式见本文根因 1）。
- fetch 脚本自足性原则：goal 脚本对 `/common` 依赖一律显式补拉，不依赖其他
  goal fetch 的驻留残留（sab.js/subset-tests.js 两先例）。
