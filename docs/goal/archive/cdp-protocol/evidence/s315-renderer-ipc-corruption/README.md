# S315 — renderer IPC 流损坏确定性复现取证（victim 帧 + 基准序列化对照）

**日期**: 2026-09-14　**触发**: 渲染流 PR30（7bed4635c，siteopt/baidu-round2）入树后
cdp-e2e 门 33→21 绿确定性红（双跑同形态 + 手工复现 3/3 同步失败于 network.events 步）

## 证据链

1. **归因（机械）**：07:56 门活跑在 `ab80e0e01`+R4330-F 树（= merge 第一父）PASS 33 绿；
   08:16 同流程在 merge 后树（唯一 delta = PR30 diff）21 绿 12 回归。归因 PR30 时序变化
   暴露 renderer 侧流损坏窗口（非 PR30 序列化改动——其 message.rs 仅加 ScriptError 变体）。
2. **victim 帧**（browser-stderr-hexdump.log，transport.rs S79 诊断网 hex 增强）：
   renderer→browser FetchRequest 帧 len=81：
   `2500000000000000 3f000000 0001000000000000 1f00000000000000 <31B url> 0300000000000000 "GET" <尾部零字节>`
3. **基准序列化对照**（/tmp/zw-probe 独立 crate 对同一 IpcMessage 实测，79 字节）：
   `2500000000000000`（id=37）`19000000`（variant=FetchRequest=19）`3f00000000000000`
   （request_id=63）`1f00000000000000`（url_len）`<31B url>` `0300000000000000`（method_len）
   `"GET"` `0000000000000000`（headers 空向量）`00`（body None tag）
4. **结论**：线帧 ≠ 任何合法序列化——variant 4 字节（19 00 00 00）丢失、request_id 第 6
   字节 00→01、尾部多出零字节 = **帧中途字节插入/丢失（写侧流损坏）**。序列化/反序列化
   对称性经 probe 证伪排除（同一二进制同一类型）。
5. **级联**：browser 侧 reader `Deserialization error: io error:` 终止 → renderer 进程死亡
   （后续 browser→renderer 写全部 `写入帧头失败: Broken pipe os error 32`）→ 12 步连坐
   （network.events / dialog×2 / emulation×2 / screenshot×2 / page.setContent /
   page.second.lifecycle / target.attachDetach / runtime.releaseObjectGroup / viewport.verified）。

## 关键代码面（修复排查入口）

- `apps/renderer/src/compositor_publish_thread.rs` SharedWriter：per-instance `frame` 缓冲 +
  flush 时持 fd 锁原子写出（R3254-L2 注记「失败路径清空本帧缓冲——否则吞错后重试会从头
  重复写入残留字节（IPC 流损坏）」——**partial write + 重试路径与本案字节形态吻合**）。
- writer 拓扑（runtime.rs `with_io`）：主线程 transport / compositor publish 线程 /
  sw_runtime_host / indexed_db_handler / service_worker_client 五个 SharedWriter 实例共享
  同一 fd mutex；per-instance frame 均原子，但 **partial-write 残留 + 跨实例重试** 未被排除。
- PR30 改动点（时序暴露者）：`apps/browser/src/headless/session.rs` proxy_fetch 移交
  worker 线程（P7：会话线程在各等待点排空完成队列）+ `Runtime.exceptionThrown`（P3）。

## 复现命令

```bash
make cdp-e2e                    # 双跑均 21 绿 12 回归（REGRESSIONS 集合见 determinism-report）
# 或手工单跑：
node -e "spawn zero-browser --headless --remote-debugging-port <free>; execFileSync node scripts/capture-core-flow.mjs, CDP_ENDPOINT_URL=…"
# 失败步 = network.events 首个 locator.click（#btn-xhr）报 Broken pipe
```

## 状态

- S309 门结论在当前树失效；门维持 **RED**（挂 renderer IPC 流损坏根因修复，S316 起）。
- master.md #0 项从「复现监测」升级为「确定性复现已捕获 + victim 帧取证在案」。
