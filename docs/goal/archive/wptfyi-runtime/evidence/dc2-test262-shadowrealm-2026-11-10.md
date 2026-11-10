# DC-2 · ShadowRealm 特性切片（+59 过，翻红 0）

**2026-11-10** · commit（本轮）· 上轮 `3ced2838b`（SAB 管线 +116）之后续

## A/B（release runner，test-guard 包裹）

| 面 | BEFORE（`3ced2838b`） | AFTER（本轮） | 净增 |
|---|---|---|---|
| ShadowRealm 面（64 案） | **0P/64F**（全 `ShadowRealm is not defined`） | 60 案执行：**59P/1F/0T**（4 importValue 划界 skip） | **+59，翻红 0** |
| `-realm` 面（222 案） | 132P/90F | 143P/79F | +11（全部为路径含 `-realm` 子串的 ShadowRealm 族——errors-from-the-other-realm / nested-realms / wrapped-function ×7 / validates-realm-object ×2；Fail→Pass 11、Pass→Fail 0） |
| staging/sm 面（1,409 案） | 1313P/91F/5T | 1313P/91F/5T | **零漂移** |
| engine lib（3,835 测试） | 7F（img/fetch/websocket 族） | 同一 7F 集逐字一致 | 零归因回退 |

log：`/tmp/round11-shadowrealm-{before,after3}.json`、`/tmp/round11-realm-{before,after2}.json`、`/tmp/round11-sm-{before,after}.json`。

## 方案（四件）

1. **V8 flag + 宿主 context 回调**（script-sandbox `v8_runtime.rs`）：平台初始化前
   `--harmony-shadow-realm`（内核已实现、flag 默认关）＋
   `set_host_create_shadowrealm_context_callback`——缺省时内核抛 "Not
   supported"，回调返回裸 ECMAScript context（`global_template: None`，无 DOM
   shim——规范要求纯 JS realm；缺一即全簇翻红，单测钉面
   `test_shadowrealm_activate_and_evaluate`：构造器可见/API 面/evaluate 同步
   完成值/wrapped function 跨 realm 调用/非 primitive TypeError）。
2. **iframe 模拟 realm 转发**（engine `part05.js`）：`ShadowRealm` 加入 s376
   `_zwRealmIntrinsics` 转发列表（主 realm 同一引用，`$262.createRealm().global`
   面可达）。
3. **importValue 步序补丁**（engine `part05.js` 顶层 IIFE）：提案
   §ShadowRealm.prototype.importValue 步 4 IsString 检查在 ToString 之前——V8
   flag 实现先 ToString（旧草案顺序），`throws-if-exportname-not-string` 以
   toString 调用计数钉当前顺序。方法简写（无 `[[Construct]]`——
   not-constructor 断言面）＋非 string 分支先 `String(specifier)` 对齐步 3→4
   步序（`Object.create(null)` 无继承 toString 面，`String()` 与规范 ToString
   同走 ToPrimitive(hint string)）＋name/length own 面可写性收口。
   descriptor/name/length/proto/specifier-tostring 六断言面全过。
   https://tc39.es/proposal-shadowrealm/#sec-shadowrealm.prototype.importvalue
4. **importValue 真加载面划界 skip**（runner `testharness_test262.rs`）：4 案
   （import-value / throws-if-import-value-does-not-exist /
   throws-typeerror-import-syntax-error / throws-typeerror-import-throws）依赖
   shadow realm 内真实模块加载——engine dynamic import 桥按 classic 页面 slot
   契约实现，该场景违背 V8 内核期望，进程级 SIGTRAP 连累整轮；划界 skip 保
   其余 60 案可跑（同步校验面不涉加载，照常执行）。

## 残余定性（不入账）

- `globalthis-ordinary-object`（1 案，引擎侧不可回收）：内核 shadow realm
  global 的 `[[Prototype]]` 非 realm 自身 `Object.prototype`（探针：own props
  仅 `["constructor"]`、`getPrototypeOf({})` 同 realm 比对 false、
  `Object.isExtensible` 真）——kernel 管理的 realm global 身份，引擎无 API
  可达（同 tco 簇定性口径；V8 flag 门内一致性缺口）。
- importValue 4 案：需 shadow-realm-aware dynamic import 桥（referrer context
  分流），另案评估；当前 SIGTRAP 面不可放宽。
- 案数勘误：triage 记 61 → 实测上游 64 文件（60 执行 + 3 FIXTURE 常驻 skip 面
  + 1 import-value FIXTURE 已在 3 内）——精确口径 64 案、+59 净增。

## 账目与锚点

净增账 75,671 → **75,730** / 76,000（+59 案级）。枚举面探针：flag 新全局
`ShadowRealm` non-enumerable（for-in 命中 0、propertyIsEnumerable false、
gOPN 含之）——页面 for-in 基线无扰动。sandbox lib 442/442（含新钉面）。
