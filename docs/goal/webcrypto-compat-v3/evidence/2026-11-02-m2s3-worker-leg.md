# M2-3 worker 腿接线（2026-11-02）

## 范围与结果

Mission 第 4 条「crypto 面接入既有 workers 通道，全子树 window/worker 双腿执行」
落地。runner 新增 `tests/wpt-runner/src/testharness_webcrypto.rs`（独立模块，
2000 行约束 + 域内聚，testharness_webmessaging/workers 先例），engine 侧
worker 全局 crypto 绑定补齐（part05.js），全子树 179 个 `.any.js` 案双腿执行。

**全量新分母 360 案 / 68,645 子测试 = P 40,572 + F 29,066 + T 7**
（[m2s3 锚](2026-11-02-m2s3-full.json.gz)）：

- window 腿 181 案：P 26,386 + F 14,536 + T 3——与 [M2-2 锚](2026-11-01-m2s2-full.json.gz)
  **逐字节全等**（状态逐项 0 变化），零回归红线保持。
- worker 腿 179 案：P 14,186 + F 14,530 + T 4；全绿案 78；与同源 window 腿
  **状态族 parity 177/179**（仅 hkdf/pbkdf2 两案饥饿族差异，见下）。
- 对本切片首轮（flatten 修复前，worker 腿接线已就位）对照：subtest 翻转
  979 例（None→Pass 862 + None→Fail 111 + Fail→Pass 1 + Timeout→None 5），
  **Pass→非 Pass 0 例**；翻转面精确落 flatten 修复族 worker 腿
  （wrapKey 299、supports×4 201/172/148/95、pbkdf2 34、derived_bits_length 30）。

## 双腿分道与执行形态

- window 腿：`.any.js`/`.window.js` 全量原样（M1 台账键不变，跨轮对账锚）。
- dedicated worker 腿：无 `// META: global=` 缺省 window+dedicatedworker 双变体
  （WPT test_format）；显式 tokens 含 `worker` shorthand 或 `dedicatedworker`
  即含 dedicated 腿；仅 sharedworker/serviceworker → 无 dedicated 腿。台账键 =
  worker 变体 URL 名（`foo.https.any.worker.js`——upstream wptserve 对外命名，
  与磁盘无碰撞；window 键保持源文件名）。window 腿在前 worker 腿在后逐案相邻。
- worker 腿执行（testharness_workers 通道同构）：页面
  `fetch_tests_from_worker(new Worker('/…any.worker.js'))` 聚合；worker 顶脚本
  经 fetcher `.any.worker.js` → 磁盘 `.any.js` 映射 + ClassicWorkerHandler 同构
  包装（GLOBAL flags + importScripts testharness/META 依赖 + 语料正文内嵌 +
  done()）；`// META: timeout=long` 升 timeout_meta（window 腿同款）。
- engine 配套（part05.js）：`wctx.crypto = globalThis.crypto`（同一实例镜像，
  getRandomValues/randomUUID/subtle 全 host 桥绑定，[SecureContext] 门禁随页锚
  scheme——WebCrypto spec §9 Crypto 接口 [Exposed=(Window,DedicatedWorker,
  SharedWorker)]）+ `_zwWorkerInterfaces` 追加 TextEncoder/TextDecoder/
  CryptoKey/SubtleCrypto/QuotaExceededError 镜像（后者为 harness
  assert_throws_quotaexceedederror 按 `self.QuotaExceededError` 取构造器比对
  `e.constructor` 所需——worker 面缺镜像曾致 getRandomValues 族 "wrong global"
  误判 78 案，修复后全绿）。
- 页锚 scheme 判定 `webcrypto_case_uses_http_page` 随 worker 键继承（historical
  等无 `.https.` 中缀案走 http 页 → worker 全局 subtle/CryptoKey/SubtleCrypto
  不挂载 → 镜像守卫天然跳过，worker 腿 `"in self"` 断言面同 window 腿成立）。

## 修复一：导入源内联扁平化（strict 闭域 + 顶层 let/const 闭域）

worker 腿首轮全量后 window 全绿但 worker 全红的 6 案归因出 `_zwInlineImportScripts`
（part05.js）direct eval 内联的两个脚本语义缺口——upstream `importScripts` 是
独立脚本（顶层声明落全局），eval 内联后：

1. **strict 指令闭域**：eval 源指令位生效 → strict direct eval → 顶层
   function/var 声明闭在 eval 作用域。命中 supports×4（util/supports.js 与
   modern-algos/util/supports.js 均 `'use strict'` + 2 顶层 function；zzprobe
   探针实证 `probeHelperTest is not defined`）。
2. **顶层 let/const 闭域**：direct eval 词法声明永不落 eval 外（strict 与否
   同）。命中 wrapKey_unwrapKey（`let wrappingKeyData`/`let toWrapKeyData`，
   60s 探针实证 `wrappingKeyData is not defined`）、derived_bits_length（4 个
   顶层 `const`）、generateKey/failures_*（algorithm_registry.js 顶层 const）、
   derive_key_and_encrypt（3 个顶层 `let`）。

修复：`_zwFlattenToplevelDeclarations`（part05.js 新增，eval wrap 前应用）——
① 指令位 `'use strict'` 等长空白抹除（保行列归因；声明落 sloppy eval 外层
变量环境与用例体共享；strict 内部规则随抹除失效，FIXME(inline-strict) 记账
口径——语料面无 strict 行为断言）；② 深度 0 语句位 `let`/`const` 等长替换为
`var`（多声明符 `,` 形态随 var 语义一并可见；`in self`/重复声明/TDZ 可观测
差异记账；顶层 class 无法等长替换仍闭域——corpus helper 面未现，
FIXME(toplevel-class) 记账）。嵌套内联源随 `_inner` 整体覆盖。

**影响面核查**：全语料（wpt-data 全树）importScripts 目标静态扫描——
strict 头/列 0 let-const 命中面仅在 WebCryptoAPI helper（上列），workers/
dedicated-workers/canvas 等其余通道 **no-op**（零行为变化）。原
FIXME(strict-import) 注记修正（其「指令位置失效」表述与 zzprobe 实证相反）。

修复后探针复跑：supports 双腿全等（200/171/77/53 + 域红 Fail 全等）、
wrapKey 299/299、derived_bits_length 29/29、ecdh 40/40+31/31、
generateKey/failures_AES-CTR 618/618、derive_key_and_encrypt 双腿 parity。

## 挂账：worker 消息投递空闲饥饿（引擎侧缺陷，非本切片域）

worker 腿残 T 4 案中 2 案为**新归因引擎缺陷**（hkdf 0/2110、pbkdf2 699/8632+
case T；窗口腿同案全绿 3661/8632）：

- **触发面**：worker isolate 内注册大量 async promise_test（2110/8632）。
  最小复现探针（已删档，本节留档）zzprobe10：2110 个 `promise_test` 各做一次
  SHA-256 digest——window 腿 2110/2110 即时全绿，worker 腿 30s 仅 42 完成
  （~0.7s/条量化）；对照 zzprobe9 单测试内 1000 路并发 digest worker 双腿全绿、
  zzprobe7 setUpBaseKeys 形态 Promise.all（6 import + ECDH generateKey）
  worker 全绿、zzprobe8 subsetTest 延迟定义 21 测 worker 全绿。
- **机理归因**：上游协议 worker 端每测试完成即 `postMessage({type:"result"})`
  逐条派发（testharness.js:302-308）；观察面呈 **worker→页消息投递与宿主
  事件循环耦合**——忙 worker（pbkdf2 逐 op CPU ~83ms）投递跟上 op 节奏
  （12 条/s），闲 worker 退化 ~0.7s/条（hkdf 定义涓流期 0 条）；与算子种类、
  并发路数、注册速率均无关（对照探针逐一排除）。
- **处置**：引擎侧 worker 消息泵空闲退化修复属独立切片（script-sandbox/
  event-loop 域），本切片不实施。worker 腿接线不受阻（其余 177 案 parity），
  hkdf/pbkdf2 worker 腿在修复切片内翻绿。
- 另 2 案 worker T（failures/successes Hybrid-KEM）与窗口腿同源（域门控，
  D1 待拍板 ML-KEM 域，非接线缺陷）。

## 其余验证

- `cargo build --release --bin zero-wpt-runner` 干净（make browser-build 不含
  tests/ crate；runner 重建即带 engine shim 重编译）。
- 无人值守跑全程 test-guard 包裹（`--per-proc-mem 4 --total-mem 8
  --time-limit 1800`）。首轮全量 runner 空输出 exit=1 一次（双流 0 字节），
  同命令复跑干净、结果完整——沿用 M2-1 flake 判据记账（WSL2 环境）。
- runner/shim 共享面对账：远端推进 `4f044f3fe..22acb33eb`（part04/part06
  域），与本切片触面（part05.js/testharness.rs/testharness_workers.rs/
  main.rs/新模块）diff 零交集。
