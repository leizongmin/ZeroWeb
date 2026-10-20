# FileAPI M2-P3 — readAs* 读取状态机（2026-10-11）

## 修复内容（js_dom_shim part02.js FileReader 重写 + part05.js EventTarget 接线）

**435 → 467（+32P）= 93.6%**（35 案 499 子测试，0 Timeout）；reading-data-section +
FileReader/ + fileReader.any.js 簇 **53/53 全绿**（M1 基线时点该簇 32F）。JSON 全量：
[2026-10-11-m2-p3-readas-state-machine.json](2026-10-11-m2-p3-readas-state-machine.json)。
复现：`make testharness-fileapi`。

spec 依据：https://w3c.github.io/FileAPI/#reader-general-update-steps 。

1. **读取状态机**：readAs* 同步置 LOADING；LOADING 期间重入读抛 `InvalidStateError`
   DOMException（FileReader-multiple-reads ×4）；loadstart →（bytes settle）→ progress →
   load → loadend；result 在 loadstart/progress 阶段恒 null、readyState 恒 LOADING，
   result 赋值 + DONE 在 progress 之后、load 之前（filereader_result 记序）。
2. **事件按 spec 逐 task 派发**（setTimeout 承载）：runner `__zw_test_setTimeout` stub
   为确定性 FIFO 队列（R3254-E2 先例）；无 host 环境回落 microtask（零回归）。
   **关键定位**：首版用 Promise `.then` 链逐事件间隔——filereader_result 的 progress
   变体 4 案仍挂「Not expecting event, but got loadend」：microtask 链上事件与
   promise_test 完成回调同队列竞速，尾事件先于 done-part 时 EventWatcher 撞
   `waitingFor=null`；loadstart 变体（多一跳 progressStep）恰使 done-part 先行而侥幸
   通过——两变体结构性不对称纯属微任务排队巧合。改 task 承载后页内 promise 链
   （wait_for → 断言 → 下一 wait_for）在事件间完整 drain，与浏览器 task 语义一致。
   定位用临时探针 `zw-probe-events.any.js`（EventWatcher 形态复刻 + once-listener
   对照），用后即删。
3. **progress 仅在有字节被读出时派发**（空 Blob 无 progress——filereader_events 口径）。
4. **abort 真取消**：代计数 `_zwGen`——abort/重读后旧读链全部 task 静默（不补发
   load/progress/loadend）；abort 本身同步派发 abort + loadend（filereader_abort 口径）。
5. **Determining-Encoding**（#determining-encoding）：BOM 嗅探（EF BB BF / FE FF /
   FF FE）优先 → encoding 参数（`__zw_text_encoding_of` 全表校验，与 TextDecoder 同源）
   → type 的 charset 参数（`_zwParseMimeType` params）→ 缺省 UTF-8；bytes 经
   TextDecoder（encoding-compat M2 host decoder，BOM 剥除 ignoreBOM=false 缺省）解码。
   收 UTF-16BE/LE BOM ×2、UTF-16 标签、windows-1252 type charset ×2 + 参数覆盖 ×1。
6. **readAsDataURL 空 type → 'application/octet-stream'**（spec datatype 缺省）×2。
7. **事件对象保真**：ProgressEvent（bubbles/cancelable 恒 false——
   progress_event_bubbles_cancelable）经 dispatchEvent 派发（listeners + on* handler
   同键去重）；part05 接 FileReader 原型链到 EventTarget.prototype（XHR R114 同款），
   ctor 初始化 `_et_listeners`——EventWatcher 的 `watchedNode.addEventListener` 前提。

## 回归门禁

- zero-engine lib **2987P / 0F**（test-guard 包裹，RUST_MIN_STACK=32MiB——深嵌套管线
  测试栈前提，Makefile 同款）；桥接测试零改动全绿（execute checkpoint drain 吸收
  task 粒度变化）
- net-api 通道逐 face 与 M2-P1 记录**逐值恒等**：url 4429/6065、fetch+xhr 2060、
  streams 905、eventsource 33、mimesniff (Blob/File) 腿 865（上游语料冲突仲裁记档不变）
- make test / fmt / clippy：见 master.md 当轮记录

## 残余失败（P4 零散面，32F）

Blob.bytes() 5、detached ArrayBuffer 4、slice 多字节解码 6、filelist 空表 4、
dom.window 平台对象 3、url-with-fetch/xhr 边界 5、url-format origin 1、
File-constructor 2、Blob-constructor/Blob-newobject/Blob-slice 零散 2——
与 M2-P1 收口时点的 P4 记账一致，本轮零漂移。
