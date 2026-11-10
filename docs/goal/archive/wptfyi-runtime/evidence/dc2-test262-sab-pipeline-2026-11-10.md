# DC-2 · $262.agent SAB 管线切片（+116 过，翻红 0）

**2026-11-10**。test262 Atomics/agent 面全量 A/B：**258P/118F/6T → 374P/8F/0T，
+116 Pass，Timeout 全回收，残余 8F 全部 ⊆ 基线失败集（翻红 0）**。
net-gain 75,555 → **75,671 / 76,000**。

## 根因：结构性 report 竞态

上游 `test262-provider.js` 的 agent 报告链路是 `postMessage` 载体：worker 侧
`report(msg)` 发消息＋`Atomics.add` 计数；主侧 `getReport()` 忙等自旋排计数、
`w.reports.shift()` 取消息。但 `w.reports` 只能被宿主消息派发填充，而消息派发
需要主线程让出（execute_script 间隙）——自旋期间计数已可见、消息未派发，
`shift()` 落空、计数已减、报告永久丢失。Chrome 在 wpt.fyi 同型 harness 上
agent 面 112/112 全失败同根因。

## 方案

### 1. SAB 报告环形缓冲（provider 重写，传输层替换不改 $262 契约）

INTERPRETING.md 只约定 report 字符串最终经 `getReport` 送达，不约定载体。
共享内存环形队列绕开消息派发时序：

- 布局：协调面保留上游 SLEEP_LOC 0 / START_LOC 1 / BROADCAST_LOC 2 /
  WORKER_REPORT_LOC 3；`RING_BASE 64`、`RING_STRIDE 4096`、`RING_MAX_WORKERS 8`、
  `RING_CAP 4094`；SAB 总量 131,328 字节。
- per-worker 子环单写单读，无 CAS：worker 写 `[len][UTF-16 码元...]` 后
  `Atomics.add` 发布（WORKER_REPORT_LOC+index 计数语义保留）；空间不足
  `Atomics.wait` 等主侧消费推进；超长（n+1 > 4094）显式 throw。
- head/tail 单调计单位、取模落位；发布与写数据同线程序＋seqcst 原子序，
  读侧见到计数即数据可见。
- 主侧 `ringRead` 按发布计数逐条读、读毕推进尾；`w.reports`/`w.onmessage`/
  worker `postMessage(String)` 报告路径全部移除。
- 关键坑：start 消息必须传**裸 SAB**（`i32a.buffer`）——Int32Array 视图会被
  宿主 clone 的 TypedArray 分支按普通视图拷贝脱管（共享内存语义丢失）；
  worker 侧 `new Int32Array(msg.sab)` 重建视图。broadcast 无此问题（上游
  safeBroadcast 本就走 `ta.buffer`）。

### 2. SAB 跨 isolate 传输管线（clone token + Rust registry + worker 物化）

- 通道能力门：`xfer.__zwSabChannel === true`，仅 Worker.postMessage 宿主分支
  置位（part05.js）；公开 `structuredClone` 与 same-realm postMessage 维持
  DataCloneError（part61.rs `test_m3_structured_clone_view_faces` 钉面不变）。
- token 协议：structuredClone SAB 分支遇通道门时把 SAB 暂存
  `globalThis.__zwSabExports[key]`，调宿主回调 `__zw_sab_export(key)` 换
  registry id，产 `{"__zwSabRef": id}` token 入 clone 结果。
- Rust 侧（script-sandbox/worker.rs）：进程级
  `SAB_BACKING_STORES: Mutex<HashMap<u64, SharedRef<BackingStore>>>`
  （SharedRef Send/Sync）＋`sab_export_by_key` 导出；worker 派发侧
  `sab_materialize_tokens` 扫 token、`SharedArrayBuffer::with_backing_store`
  物化到 `globalThis.__zwSabInbox[id]`；bootstrap `_zwWorkerReviveXfer`
  revive 分支换入 inbox 对象（同消息同 id = 同一对象身份；未知 id 留 token
  穿透）。
- 回归锚点：公开 clone 拒收钉面 4 单测过（part61 SAB faces + part64 三桥）。

### 3. V8 前台任务泵（waitAsync 面的关键前置）

`Atomics.waitAsync` 的 futex 唤醒解析经 isolate 前台任务 runner 投递，默认
平台不自驱——无泵则解析永不发生、promise 永悬挂。仓库此前无任何
`pump_message_loop` 调用。两处：

- worker isolate：`worker.rs` 命令循环 None 臂改 4ms `recv_timeout`，超时即
  `pump_message_loop(..., false)` 空转排前台任务。
- 主 isolate：`v8_runtime.rs` `execute_with_catch` 尾部泵一次（wait_for_work=
  false，空队列即返）。
- 泵后任务解析的反应（await 续体）是微任务，两处泵后各补
  `perform_microtask_checkpoint`——第一轮泵后 waitAsync 仍 30T 即缺这一步
  （任务解析了、续体没跑）。

### 4. test262 通道真线程 worker 宿主（runner 侧）

- `main.rs` `testharness-test262` 分支：`ZW_THREADED_WORKERS=1`（命令分派首
  语句、WebView 创建前，ZW_RENDERER_AUTOMATION 同 SAFETY 纪律）＋
  `test262_worker_host::install()`。
- 新文件 `tests/wpt-runner/src/test262_worker_host.rs`（242 行）：宿主线程
  持 WorkerRuntime 表＋8ms 事件泵；`HostCommand` Spawn/PostMessage/Terminate/
  TerminateAll；派发脚本经进程级 `DISPATCH_QUEUE` 交给主侧探针循环排空
  （`take_dispatch_scripts` → `execute_script`）。
- 案级回收：`testharness.rs` 每案首 `terminate_all()` 同步握手清上一案 worker
  线程与积压派发队列（非 test262 通道 HOST_COMMAND_TX 为 None，零操作）。

## 验证链

| 步骤 | 结果 |
|---|---|
| 单案探针（wait-index-value-not-equal） | agent 面 report 经环送达、断言收敛 |
| waitAsync 面（泵 A/B） | 泵前 30T → 泵 v1 70P/1F/30T（缺微任务检查点）→ 泵 v2 100P/1F/0T |
| 全 Atomics 面 A/B | 基线 258P/118F/6T → 终态 **374P/8F/0T**（+116，Timeout 6 全回收：5 Pass 确认＋good-views ×2） |
| 单测钉面 | part61 SAB clone 拒收 + part64 worker host 三桥测试 4 过 |
| dom 锚点 | 56,870P/642F/14T vs round9 56,989P/692F/14T——见下节定性 |

log 留痕：`/tmp/round10-atomics-before.log`（基线）、
`/tmp/round10-atomics-after.log`（管线后泵前 337P/11F/30T）、
`/tmp/round10-waitasync-after.log` / `-after2.log`（泵两轮）、
`/tmp/round10-atomics-after2.log`（终态）、`/tmp/round10-dom-face.log`。

## dom 锚点定性（主 isolate 泵改动无语义回归）

差 169 子测试**全部**落在 Range-mutations 族两页
（dataChange 795→745、replaceData 835→716）；r9-only-pass 119 条中
Range-mutations 占 **119/119**，非 Range 族翻红 **0**。该族已由 round9 证据
定性为 90s 看门狗截断边界漂移 artifact。本轮追加同二进制稳定性证据：Range
面连续两轮 2920P/212F vs 2898P/223F——同代码相邻两轮差 22 Pass，与跨轮
漂移同量级，确认噪声非回归。

## 残余 8F 定性（不入账）

全部 ⊆ 基线失败集，engine coerce 顺序/挂起语义缺口，非 SAB 管线范围：

- `retrieve-length-before-index-coercion` ×5 ＋ `wait/retrieve-length` ×1：
  TypedArray length 与 index coerce 顺序（engine 侧）。
- `cannot-suspend-throws` ×2：`Atomics.wait` 主线程不可挂起异常语义
  （engine 侧，与 canSuspend 检测相关）。

## 归簇更新

$262.agent 112 案簇（[归簇证据](dc2-test262-realm-intrinsics-2026-11-10.md)
SAB clone 106 可修＋agent dependent）→ **已回收 +116**（SAB clone 106 全量＋
agent 面 waitAsync/超时回收）。test262 剩余未回收簇：ShadowRealm 61（特性
切片）、tco 34（引擎侧不可回收）、TLA 残余 15、gc 15、Atomics coerce 残余 8。
