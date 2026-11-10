# DC-2 · test262 剩余簇归簇定性＋iframe realm JS 内建转发修复

**2026-11-10** · commit（本轮）· 上轮 `d44d9d5ad`（TLA 深度门 +30）之后续

## 归簇定性（DC-1 基线失败面，修复校正后口径）

| 簇 | 案数 | 定性 | 回收面 |
|---|---|---|---|
| tco-*（PTC） | 34 | **引擎侧不可回收**——V8 未实现规范尾调用（Chrome 同；`Maximum call stack` 恒现），`$MAX_ITERATIONS` 深递归期望零栈增长 | 0 |
| module instn-star-iee-cycle | 2 | module 循环早期错误族（非 PTC），归 module cycle 簇 | 待归簇 |
| SAB clone | 106 | **可修**——真线程 worker（`ThreadedRuntimeCore`）已落地，但消息物理载体是 JSON 字符串（`structuredClone` → JSON → `__zw_worker_host_op('post')` → `PostMessage(String)`），SAB 共享内存语义不可达。报错源 = 引擎 shim structuredClone 的非 COI 拒收分支（workers-compat M3 有意设计，battery 钉 `crossOriginIsolated` falsy 面） | 下轮切片 |
| ShadowRealm | 61 | **特性切片**——`ShadowRealm is not defined`（构造器缺失）；prototype/evaluate 面（返回值包装/裸全局/nested realms/importValue）+ WrappedFunction 面 | 下轮候选（s376 内建转发面即其地基） |
| eval/intrinsics（createRealm/otherGlobal） | ~50 | **本轮已修**（s376） | +121 |
| encoding iframe 面 | ~330 | unsupported-labels 158 + single-byte-decoder-iframe 168 需 iframe charset→document.encoding→TextDecoder 管道；streams/realms 12 + invalid-realm 4 = realm 语义 | 独立切片 |

DC-1 证据勘误：「createRealm/installAPI 78」归簇正则（`reading 'prototype'|createRealm`）
宽收 `reading 'prototype'` 通用形态——真实 eval/intrinsics 族 ~50（本案 +121 计数
口径），其余 78−50 为杂项 prototype 读取失败。

## SAB 管线切片方案（已勘明，下轮实施）

rusty_v8 绑定齐全：`SharedArrayBuffer::get_backing_store() -> SharedRef<BackingStore>`
提取 ＋ `SharedArrayBuffer::with_backing_store(scope, &store)` 跨 isolate 重建。

1. structuredClone SAB 分支按**通道能力门**分叉（非 `crossOriginIsolated`——
   workers-compat battery 钉全局 falsy 面，改用 runner 通道内 flag）：开 → 产
   `{"__zwSabRef": id}` token 入 JSON；关 → 维持 DataCloneError。
2. 主 isolate 宿主 op：token → `get_backing_store()` → Rust 侧
   `Mutex<HashMap<u64, SharedRef<BackingStore>>>` registry 落账。
3. worker 派发侧（`worker.rs` PostMessage 处理）：JSON parse 后 walker 扫 token 位，
   HostCallback Scoped 形态 `with_backing_store` 物化同 backing store SAB 换入
   （worker 侧 `new Int32Array(sab)` 即共享内存）。
4. worker→main 反向：backing store 身份反查 registry 复用同 id。

预估回收 ~106（Atomics.wait/waitAsync/notify 全族）。

## s376 修复：iframe realm JS 内建转发面

**根因**：模拟 iframe win（`_zwMakeIframeWin`，part05）此前仅 DOM 构造器转发
（R140/R179/R181 先例），JS 内建不在 win 对象上——`$262.createRealm().global.eval`
→ undefined，test262 eval/intrinsics 簇 ~50 案「eval/Proxy/ArrayBuffer/
[TA.name] is not a function/constructor」。iframe 内联脚本的 `with(window)`
取内建走作用域链穿透（闭包全局），模拟 realm 全局面残缺。

**修复**（三件）：

1. **JS 内建转发**（R187「转发本体」同款——主 realm 同一引用，instanceof/
   identity 断言面自然成立）：Reflect/Map/Set/WeakMap/WeakSet/WeakRef/
   FinalizationRegistry/Promise/ArrayBuffer/SharedArrayBuffer/DataView/11×
   TypedArray/Atomics/JSON/Math/RegExp/Date/Number/String/Boolean/Symbol/BigInt/
   6×Error/parseInt 系/structuredClone/TextEncoder/TextDecoder/Intl 共 50 名
   （字面量已定义面 Object/Function/Array/Error/TypeError 不覆盖）。
2. **scoped eval**：`with(win){ return eval(src) }`——var/function 声明落 win
   （真实 realm indirect eval「var 落 realm 全局」近似；严格模式 eval 自建环境
   不落 win，spec 同）。副作用正效应：iframe 内联脚本裸 `eval()` 此前解析到
   主 realm 直穿，现按浏览器语义解析到子 realm eval。
3. **Proxy 可构造化**：win.Proxy 旧包装仅 `{ revocable }`（普通对象非函数），
   `new w.Proxy(t, h)` 报 not a constructor——改函数包装，构造/revocable 产物
   均保留 R187 realm 印记 + `__zwRealmOf` 注册表。

**单 realm 近似边界（不回收面，维持）**：跨 realm intrinsic distinctness
（`g.Array !== Array` 断言）与 per-realm 默认 this（sm/Array/from_realms 的
映射函数默认 this = 函数所在 realm 全局）需真实第二 context 语义。

## 对拍（release runner，test-guard 包裹，翻红 0）

| 面 | DC-1 基线 | 本轮 | 净增 |
|---|---|---|---|
| `-realm` 面（250 案，交集 222） | 33P/189F | 132P/90F | **+99，翻红 0** |
| staging/sm 面（交集 1409） | 1291P/113F | 1313P/91F | **+22，翻红 0** |
| multiple-evaluations 面 | 35P/7F | 35P/7F | 持平（private brand 跨 realm 语义另案） |
| dom 语料面（子测试口径，回归锚点，A/B 双跑） | 56,931P/689F | 56,989P/692F | +58 子测试 |

dom 锚点 +3/+4 Fail 勘误：`Range-mutations-dataChange` 族页**超时截断边界漂移**
（BEFORE 轮 `tests:799` 处截断、AFTER 到达 802/803）——截断点外的
`xmlTextNode.data +=` 边界断言到达即红（`Wrong start offset expected 0 but got 1`
引擎既有 XML Range 边界缺口），非语义翻红；同二进制过滤/全量两模式截断点本身
漂移 ±6 子测试。+58 子测试属 wptfyi-dom goal 账（其 DC-1 未开账），不入本 goal
净增账。

净增账 75,434 → **75,555** / 76,000（+121 全部为 test262 案级）。log：
/tmp/round9-realm-face.log、/tmp/round9-sm-face.log、
/tmp/round9-multieval-face.log、/tmp/round9-dom-face.log（AFTER）、
/tmp/round9-dom-face-BEFORE.log（A/B）、/tmp/round9-dc-{rerun,before-rerun}.log。

## 残余定性（挂账更新）

- `-realm` 面 90F：跨 realm distinctness/默认 this 面（上表不回收边界）+
  private brand 跨 realm + RegExp legacy accessor cross-realm receiver（7 案
  需真实第二 context 原型身份）——均指向真 realm 管道，单 realm 近似边界。
- ShadowRealm 61 + SAB 106 + encoding iframe ~330：下轮切片候选（按 106 > 61 > 330
  排序；SAB 方案已勘明成本最低路径）。
