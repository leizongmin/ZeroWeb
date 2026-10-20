# M3——postMessage transfer/ports + structured-clone 语义（2026-10-21）

全量 **678/714 = 95.0%**（678P/27F/8T/1PF，175 案）。证据 JSON：
[2026-10-21-m3-transfer-ports-structured-clone.json](2026-10-21-m3-transfer-ports-structured-clone.json)
（runner `testharness-workers --json` 原样输出）。

## 数字对照

| 阶段 | Pass/Total | 率 | 备注 |
|---|---|---|---|
| M2 切片三收口 | 513/563 | 91.1% | 上一切片台账 |
| 本切片收口 | 678/714 | 95.0% | M2 残余 M3 簇（multi-port 7F、messageport 1F+2T、second-argument-null、imagedata、structure-message 1T、termination-with-port 1T）全数翻绿 |

分母 563→714（+151）：`workers/semantics/structured-clone/dedicated.html` 的
structured-clone battery 支撑资产（`html/webappapis/structured-clone/` 三 harness
文件 + `common/sab.js`）本切片补齐，battery 全量展开产出真实子测试。
Timeout 11→8（Worker-messageport×2、Worker-structure-message、
Worker-termination-with-port-messages 随 ports 面落地翻绿）。

## 修改面

### 1. 结构化克隆算法扩展（part02 `_zw_structured_clone`）

- **transfer 通道**：函数加 `xfer` 参数（serialize-with-transfer map 近似）——
  payload 内 MessagePort 引用转义为「同一转移体」（同一次 postMessage 内多处引用
  与 transfer 列表共享一个 clone；spec serialize-with-transfer 步序）。
- 包装对象克隆保持类型：Boolean/Number/String/BigInt（battery compare_*/BigInt 面）。
- Error 克隆：constructor/message/cause 递归保持，非 own message 不安装，自定义
  own 属性不迁（battery compare_Error 五面）。
- 视图面：DataView fresh-buffer 三元组重建；resizable buffer 身份保真
  （maxByteLength/resizable 随迁）；OOB TypedArray/View 序列化 DataCloneError；
  SharedArrayBuffer 非 COI 拒绝。
- 数组洞保持（稀疏数组 length/洞；非索引 own 属性同迁）。
- FileList/ImageData/ImageBitmap/OffscreenCanvas 克隆保持类型
  （ImageData data 独立拷贝；ImageBitmap wire 快照；OffscreenCanvas 尺寸保真，
  位图内容记账 FIXME）。
- 公开 `structuredClone(value, {transfer})` 二参面（内部 `__zwXfer` 供 postMessage
  注入转移映射）。

### 2. transfer 列表三相重构（part05）

- `_zwValidateTransferList`（新）：全表校验先行——不可转移项/非 object 元素
  DataCloneError、closed/source port、closed 位图、detached buffer（零长视图构造
  探针）；**不产生部分转移副作用**（multi-port「failed postMessage 后重发仍可用」
  面）。postMessage 各路径在 clone 前调用（validate-then-consume = spec 步序）。
- `_zwTransferPort`（自 consume 抽出）：配对改挂 + 队列随迁 + 原 port 脱离；
  global 构造器被 delete 后经原型 constructor 仍可达。
- `_zwIsMessagePort`：port 品牌印记（`_zwPortBrand` 不可枚举 own）——transfer 期间
  delete `globalThis.MessagePort` 的 battery 面跨删除稳定。
- ImageBitmap/OffscreenCanvas 纳入 transferable；转移后源关闭（ImageBitmap close/
  源 bitmap `_closed`）。

### 3. postMessage 转移面（Worker / DedicatedWorkerGlobalScope / MessagePort）

- 三处 postMessage 统一：0 参 TypeError（WebIDL message 必参）、第二参联合
  （sequence<object> or StructuredSerializeOptions——null → 字典分支缺省）、
  sequence 元素 object 型判定（null/原始元素 TypeError）。
- 投递载体 microtask→**task**（setTimeout 0；spec「queue a global task on the
  posted message task source」）——背靠背两条 worker 消息间页面 promise 链可改挂
  onmessage（Worker-structure-message 双发 Timeout 实证根因）。
- MessageEvent 携带 `ports`（freeze）。

### 4. 接口类化

- `ImageBitmap` 裸对象 → 全局接口类（非法构造 TypeError、toStringTag、close()
  width/height 归零、transfer 副作用）。
- `ImageData`/`MessagePort` 原型 `Symbol.toStringTag`（String(port) ===
  '[object MessagePort]' 面）。
- Blob 判定经装载期快照 `_zwBlobCtorSnap`（globalThis.Blob 被 delete 后构造/克隆
  路径仍可达——battery「interface deleted from the global」面）。

### 5. 语料补充（70-workers-compat.sh）

- `html/webappapis/structured-clone/`（battery harness 三文件）+ `common/sab.js`
  单文件拉取——dedicated.html 页面级 `<script src>` 依赖，缺文件即 ReferenceError。

### 6. 引擎单测（part61）

9 案新增：包装对象/稀疏数组、Error 三相、视图面（DataView/resizable/OOB/SAB）、
FileList+delete 后 Blob、ImageBitmap 面、detached buffer 再转移+OffscreenCanvas、
port 转移五错误形态、payload port + transfer 同一转移体往返、postMessage 第二参
联合形态。`cargo test -p zero-engine --lib worker` 275/275。

## M3 切片修正（会话内）

- **OOB TypedArray 判定**：引擎按 spec 把 OOB 视图 TypedArray getter clamp 为 0
  （DataView getter 抛 TypeError → catch 兜底，此前碰巧过），当前值无法区分真 0
  与 clamp 0 → resize 往返探针（resize 到 maxByteLength 恢复创建期 offset，量毕
  缩回；同步窗口不可观察），offset > 当前长 → DataCloneError。FIXME：部分跨度
  fixed 视图（offset ≤ 当前长但 offset+span 越界）未检出，battery 无此形态。
- battery「MessagePort transfer 三案」失败根因：旧 `_zwConsumeTransferList` 内联
  校验在 clone 期 port 已改挂（`_zwDetached`）后对合法转移误抛 closed/detached →
  校验先行（validate-then-consume）后消费位不再校验。

## 残余 36（27F + 8T + 1PF，29 案）分簇

| 簇 | 案×子测试 | 缺口 |
|---|---|---|
| ErrorEvent 溯源（P7 后半） | report-error-\* 6×2 + Worker-nested-importScripts-error 1 = 13F | importScripts 调用点/被导入脚本 filename/lineno 归因（脚本装载表） |
| 基建 302/baseurl（P8 记账） | baseurl/alpha worker-in-worker 1F、use-base-url 1F、dedicated-worker-from-blob-url 1F + baseurl/alpha 2T、location/redirect 1T | runner 无重定向语义；blob URL base 解析 |
| 环境值面（P3 残余） | Worker-location.sub 1F（`:80`）、search_empty 1F（`"?"`）、platform 1F、userAgentData 1F、rAF 1F、resource timing 1F、nested_sync_xhr 1F | 值域对齐/依赖面 |
| URL 编码 | semantics/encodings/003 1F、worker-url-encoding 1F | 文档编码 → worker URL 解码管线 |
| terminate 深度 | Worker-terminate-forever 1F、Worker_terminate_event_queue 1T | terminate 中断执行语义（同 realm 近似固有边界） |
| 排除面记账（P6） | exposure.any.js SharedWorker 1F、ctor-\{1,null,undefined\} 3T、navigator/007 1T | SharedWorker 挂账；等待型页面（M1 基线即有） |
| battery 先决条件 | structured-clone dedicated 1PF（ReadableStream 子类） | ReadableStream 非 transferable（上游同形态 precondition，非失败） |

## P2 契约门核验

本切片全部改动落在 engine 页面管线 shim（js_dom_shim）与 wpt-runner 测试通道，
未触 zero-protocol / apps/renderer IPC 契约与 zero-page-runtime 协议面——
transfer/ports 语义在同 realm 近似内闭合，P2 契约协商门未触发；独立 isolate/
线程化形态如后续立项再对契约（master.md 架构现状修正记账）。
