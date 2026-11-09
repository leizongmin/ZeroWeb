# M2 s334：canvas 双面——CTR Illegal constructor + getContext('2d') settings 字典转换，the-canvas-element 2F→2P 零位移

日期：2026-11-07 ｜ 承接：s333（`2026-11-07-m2s333-audio-lazy-loading-gate.md`）

## 目标与定谳

canvas 残红 3F+1T（承 s329）勘根四分：

1. **historical.html「CanvasRenderingContext2D constructors」**：shim 急切注册（R49xx）与 `_zwMakeCtx2d` 懒兜底两站点均以裸 `function CanvasRenderingContext2D() {}` 建口——可构造。spec 接口无 [Constructor]/[LegacyFactoryFunction]，call/new 均应抛 TypeError（WebIDL es-interface-call）。R49xx 的 typeof 特性检测面不受影响（throw 函数 typeof 仍 'function'）；实例全程 `Object.create(prototype)` 产，无 shim 内工厂调用（grep 实证）。
   https://webidl.spec.whatwg.org/#es-interface-call
2. **2d-getcontext-options.html**：spec getContext 步**每次调用**都执行 CanvasRenderingContext2DSettings 字典转换（创建相与缓存相各一轮，测试双相断言）；成员按字母序 Get（alpha → colorSpace（枚举，ToString 可观察）→ desynchronized → willReadFrequently）；undefined/null → 默认零 Get；非对象 → TypeError；离格枚举 → TypeError。shim 现状：直读 `options.colorSpace` 两次（host op 传参 + `_cs` 记录），无转换算法——探针恰得 `["colorSpace","colorSpace"]`。
   https://html.spec.whatwg.org/multipage/canvas.html#dom-canvas-getcontext ｜ https://webidl.spec.whatwg.org/#es-dictionary
3. **toBlob.jpeg.html**：`toBlob(cb,'image/jpeg')` 恒产 PNG（shim 类型臂硬编 `image/png`，part04 toBlob 路径无 JPEG 编码分支；仓库无 JPEG 编码依赖，image-decoder 仅 decode 面）——**需引入编码依赖**（Cargo deps + 渲染管线归属核对），挂账不在本切片。
4. **security.pattern.fillStyle.sub.html T**：跨源取材（forEachCanvasSource → REMOTE_ORIGIN 资源）+ origin-clean/taint 模型面，挂账（超时为取材 lane 未完成，非 shim 死循环——runner 跨源主机映射与 taint 双面均缺）。

## 施工（part05 四站点）

- CTR 急切注册（R49xx 段）与懒兜底（`_zwMakeCtx2d` 头）两处：函数体改 `throw new TypeError('Illegal constructor')`；prototype 非写/非删 defineProperty 保持。
- `el.getContext` '2d' 分支顶部新增 `_zwCanvasCtxSettings(arguments[1])` 单点转换（字母序成员 Get + 枚举 ToString + null/undefined 零 Get + TypeError 两臂），host op 与 `_cs` 改读转换产物；缓存相与 webgl 冲突相均在转换后（spec 步序）。

## 验证

| 面 | 方式 | 结果 |
|---|---|---|
| 契约六臂 | `cargo test -p zero-engine --lib test_canvas_ctr_illegal_ctor_and_settings_s334` | new/带参/call 三形态 TypeError + typeof 'function' 保持 + 创建相 5 Get 字母序（colorSpace ToString 在位）+ 缓存相重转换且同实例 + null 零 Get + 离格枚举/非对象 TypeError——1/1 过，`logs/s334-contract.log` |
| canvas 域 A/B | `make testharness-html-semantics FILTER=the-canvas-element`，基线 `logs/s332-ec-after.log`（part05 自 s332 无他改，s333 面 part04/06 不触 canvas） | 120 结果行；(state,key) 归一 join：**2F→2P**（historical 全子测翻绿、2d-getcontext-options F→P），其余 67 对零位移（toBlob.jpeg F、security.pattern T 原样）——零 P→F。`logs/s334-canvas-after.log` |
| engine lib | `cargo test -p zero-engine --lib`（test-guard） | 3800P / **6R 与在册已知红集逐名 diff 为空**（img settle×2 / fetch matrix / iframe SW / websocket×2），零新红。`logs/s334-lib.log` |
| clippy/fmt | `-D warnings` + `cargo fmt --check` | 双绿。`logs/s334-clippy.log` |

## 挂账与移交

- toBlob.jpeg：JPEG 编码依赖引入（Cargo.toml + 渲染编码管线归属与渲染流核对后另立切片）。
- security.pattern.fillStyle T：runner 跨源主机映射 + shim origin-clean/taint 模型（createPattern 来源污染追踪 → toDataURL/getImageData 抛 SecurityError）。
- OffscreenCanvas.getContext（part05:9412 OffscreenCanvas.prototype.getContext）与 part04 静态克隆臂未同步字典转换——语料无面，FIXME 在码（同转换 helper 在 part05 闭包内，跨文件臂需移植）。
- **面外移交（本轮勘根证据）**：`object-attributes.html` 4F——① object.width/height = style-system presentational hints 臂缺失（HTML rendering §width/height 映射属性；img/table 同型已落 `crates/style-system/src/lib.rs` presentational_hints 收集器，缺 object/embed 臂；solo 实证 computed width 恒非 100px）→ 渲染流面，今日该 crate 活跃（shadow M1），按 §9 碰头纪律挂账移交不硬解，solo 证据 `logs/s334-object-attrs-solo.log`（4 子测全 Fail；域跑 s332-ec-after 有同键在册）；② object.contentWindow 族 = nested browsing context face（object 未建 browsing context），同属面外。
- 队列下一候选：microtasks 4F+2T（script-sandbox 微任务序 face）或 CSP style-element 8T。

## 已知限制

- settings 的 alpha/desynchronized/willReadFrequently 仅转换校验，未透传宿主存储择别（软渲染无 CPU/GPU 分岔，行为面无观察者——FIXME 记 helper 注释）。
