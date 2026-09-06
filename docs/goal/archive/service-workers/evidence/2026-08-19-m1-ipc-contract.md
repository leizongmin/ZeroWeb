# M1-4a Service Worker IPC Contract

**日期**：2026-08-19
**状态**：M1-4a complete
**前置**：[M1-3c page bridge](2026-08-19-m1-page-bridge.md)

## 0. 协议

在 `IpcMessageKind` 末尾追加：

- `ServiceWorkerRequest(ServiceWorkerRequestParams)`；
- `ServiceWorkerResponse(ServiceWorkerResponseParams)`。

严格遵守 bincode 判别值纪律：未在既有变体中间插入。

Request operation：

- Register：script URL、optional scope、renderer document URL；
- Snapshot：registration ID；
- Unregister：registration ID；
- ActivateWaiting：registration ID。

Response result：

- Registered ID；
- registration snapshot；
- boolean；
- empty；
- typed error。

snapshot 只含 ID、规范化 script URL、scope 与 lifecycle state。协议不传 script source、
JS 引擎对象、cache body 或凭据。

## 1. 信任边界

- Register 的 script/document URL 必填；
- script/document/scope 每项最多 64 KiB；
- browser handler 必须调用 `validate()` 后再抓取或写 manager；
- renderer 提供的 document URL 只是声明，browser 仍需与 tab 导航 authority 对比；
- error code 固定为 InvalidArgument/NotFound/InvalidState/Network/Script/Capacity/Internal。

## 2. 验证

新增 5 项协议测试：

- register request round-trip；
- snapshot response round-trip；
- typed error round-trip；
- snapshot/unregister/activate-waiting ID operation round-trip；
- oversized URL fail closed。

`zero-protocol` 全套 **298/298** 通过，`--all-targets -D warnings` clippy 通过。

## 3. 未完成边界

- browser 尚未处理 request；
- renderer JS worker 尚未发送 request；
- request/response correlation 将复用 `IpcMessage.id`；
- manager ownership、script fetch 与 tab security authority 在 M1-4b 接入；
- peer binary 多进程验收在 handler 接线后执行。
