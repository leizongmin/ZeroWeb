# Web API shim、宿主桥与实现边界

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

查编码、压缩、WebCrypto、FileReader、messaging、导航与计时的 shim/真实宿主边界。shim part 编号不代表独立功能；跨 part 闭包、prototype 接线与宿主 callback 要一起核对。

### User Task Navigation

| 范围 | 相关知识与验证判据 |
| --- | --- |
| fetch、WebSocket、资源策略 | [网络](../web-data/network/index.md)，不在本文重复 wire 操作配方 |
| storage、Service Worker | [存储](../web-data/storage/index.md)、[Service Worker](../web-data/service-workers/index.md) |
| Canvas、媒体 | [Canvas](../graphics/canvas/index.md)、[媒体](../media/index.md) |
| 通用 API | `js_dom_bridge/crypto.rs` / `text_encoding.rs` / `compress.rs` 内嵌测试，WPT `testharness.rs` 的对应通道 |
| 产品验证 | [zeroweb-verify](../../../zeroweb-verify/SKILL.md)，入口未提交、本轮未实跑 |

## Gotchas
- WebCrypto 与 compression 的宿主 byte wire 是逗号分隔十进制字节，不是 UTF-8 文本；BufferSource 的 byteOffset/byteLength 必须随视图截取，不能对整个底层 buffer 计算。 (`crates/engine/src/js_dom_bridge/crypto.rs`, `crates/engine/src/js_dom_bridge/compress.rs`, `crates/engine/src/js_dom_shim/part01b.js`)
- shim randomFill 只有宿主返回足量随机字节才用 OS 随机；宿主缺失或失败返回空串会回退 Math.random，所以对象存在与 UUID 格式正确不证明 CSPRNG 安全性。 (`crates/engine/src/js_dom_shim/part01b.js`, `crates/engine/src/js_dom_bridge/crypto.rs`)
- 当前 getRandomValues 用 ArrayBuffer.isView 校验，未严格拒绝 DataView/浮点视图；注释的整数 TypedArray 限定不是已实现的完整输入契约。 (`crates/engine/src/js_dom_shim/part01b.js`)
- legacy TextDecoder host 表进程共享且最多 4096 个 FIFO 条目；逐出后未知 handle 返空 text、err=0，没有注释所称透明重建，不能把它当可恢复的跨块状态。 (`crates/engine/src/js_dom_bridge/text_encoding.rs`)
- decoder 空输入且非 flush 必须直接返回空结果保驻留 lead；给 encoding_rs 喂空块曾丢 Big5 等多字节 lead，flush 空块则仍须完成替换/fatal 收尾。 (`crates/engine/src/js_dom_bridge/text_encoding.rs`)
- compression 宿主先收完整字节再 flush，gzip、deflate/zlib、deflate-raw 是不同格式；损坏或空解压返回空串有失败与空结果歧义，不能据此声称真流式处理及可靠错误区分。 (`crates/engine/src/js_dom_bridge/compress.rs`)
- FileReader 的实现分布在 part02 状态机与 part05 EventTarget 接线；只载前者可能有 onload 却无 addEventListener，WPT EventWatcher 的消费面因此不可达。 (`crates/engine/src/js_dom_shim/part02.js`, `crates/engine/src/js_dom_shim/part05.js`, `git:b39c119222db50d9ead3c53cd7825f2500aa5555`)
- FileReader 同步置 LOADING，result 在 progress 后、load 前赋值，空 Blob 不发 progress；abort 增代使旧 task 静默，不能只把 readyState 改 DONE 后允许旧读链再发 load。 (`crates/engine/src/js_dom_shim/part02.js`)
- FileReader 编码选择是 BOM、显式合法标签、Blob type charset、UTF-8 的优先序；host 编码表缺失时回落 UTF-8，不能从 UTF-8 样本通过推出 legacy encoding 已接通。 (`crates/engine/src/js_dom_shim/part02.js`, `crates/engine/src/js_dom_bridge/text_encoding.rs`)
- 普通 structuredClone 不保用户类 prototype，Blob/File 分支保平台类型；消息必须先克隆再 detach transfer 列表，否则克隆会碰已脱离的 ArrayBuffer。 (`crates/engine/src/js_dom_shim/part02.js`, `crates/engine/src/js_dom_shim/part05.js`, `git:c4408c7f6d129eb1bb168f45b88d96a313447ad0`)
- MessagePort 转移是新对象接管对端、队列与启用态，原 port 脱离；收到的 ports 数组冻结，不能只给原对象标 detached 而不迁移配对关系。 (`crates/engine/src/js_dom_shim/part05.js`)
- location hash 导航载入前 replace、完全载入后 push，但锚激活来源恒 push；判据同时依赖 load 设置的标志与宿主临时 sourceElement，不能仅按 URL 相同文档判断。 (`crates/engine/src/js_dom_shim/part02.js`, `crates/engine/src/js_dom_shim/part06.js`, `crates/webview/src/webview/user_actions.rs`)
- Navigation API 的 navState 与 classic history.state 是不同槽，navigate 未传 state 会把 classic 槽置 null，getState 则可仍为 undefined；保存/返回 navState 要 structuredClone 避免后续修改原对象污染历史。 (`crates/engine/src/js_dom_shim/part02.js`)
- performance.now 有宿主时读 elapsed 毫秒，无宿主回落 Date.now，并粗化到 0.1ms；timeOrigin 惰性首访问缓存，不能把无宿主路径当同源单调时钟或把该 origin 当实际导航开始时间。 (`crates/engine/src/js_dom_shim/part01b.js`)
- PerformanceObserver 每个 observer 至多排一个 microtask flush，按 startTime 排序后以 observer 为 this 和第二实参；event 条目按未取整 durationThreshold 过滤，first-input 不用该门槛。 (`crates/engine/src/js_dom_shim/part01b.js`)

## Architecture
- NavigationBridge 只排跨文档目标 URL，worker 回调不 fetch/reload；runtime 主线程 drain 后执行导航，shim 内存 history 变化不等于宿主真的载入新文档。 (`crates/engine/src/navigation_bridge.rs`, `apps/renderer/src/js_worker.rs`, `apps/browser/src/tab_js_worker.rs`)
- WPT 按通道注入本地 fetch/script、WS、CSP 与 timer/rAF 分支；它验证的是明确 fixture 下的能力，不是所有宿主默认配置下的功能覆盖。 (`tests/wpt-runner/src/testharness.rs`, `crates/webview/src/webview.rs`)
- WebView 在 shim 就绪后才订阅 long-task 观察，观察器只投递耗时，页面任务尾再发 `__zwEmitLongtask`；shim 初始化不计入页面 longtask，观察器不得在引擎回调内重入 JS。 (`crates/webview/src/webview.rs`, `crates/script-sandbox/src/lib.rs`)
