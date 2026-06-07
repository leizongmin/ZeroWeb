# ZeroWeb 深度审查报告 — Protocol / IPC 模块

> **摘要**
>
> **审查范围**：`crates/protocol/`（Channel、Transport、Process Manager、Message）
>
> **关键发现**：共发现 3 个问题（中 1 / 低 2）
>
> **最高优先级**：SharedMemoryChannel 使用 Vec::pop 实现 LIFO 语义，消息接收顺序与发送顺序相反
>
> **验证状态**：已验证（2026-06-07）— 1 verified, 2 dismissed

## 审查上下文

| 字段 | 内容 |
|------|------|
| **审查对象** | protocol/channel.rs、protocol/transport.rs、protocol/process.rs、protocol/message.rs |
| **审查维度** | 实现缺陷、可靠性 |
| **代码版本** | main 分支，commit f5eb85b |

---

## 问题清单

### 中优先级（Major）

#### IPC-01 [实现缺陷] SharedMemoryChannel 使用 Vec::pop 实现 LIFO 语义

- **位置**：`crates/protocol/src/transport.rs:127`
- **置信度**：0.90
- **状态**：verified
- **描述**：`SharedMemoryChannel::recv` 使用 `Vec::pop`（从尾部弹出），但 `send` 使用 `Vec::push`（从尾部插入），导致消息接收顺序为 LIFO（后进先出）而非预期的 FIFO（先进先出）。测试 `test_message_ordering`（308-322 行）验证了此 LIFO 行为，但这对于 IPC 通道来说是不正确的——消息应按发送顺序接收。
- **触发条件**：快速连续发送 3 条消息 A、B、C，接收顺序为 C、B、A。
- **代码证据**：
  ```rust
  fn recv(&mut self) -> Result<IpcMessage, ProtocolError> {
      inbox.pop().ok_or_else(|| ProtocolError::Channel("没有可用消息".into()))
      // Vec::pop 从尾部弹出 → LIFO
  }
  ```
- **影响**：IPC 消息处理顺序错误，可能导致导航命令、网络请求等乱序
- **建议修复**：使用 `VecDeque` 替代 `Vec`，`recv` 用 `pop_front` 实现 FIFO。

---

### 低优先级（Minor）

#### IPC-02 [可靠性] RendererHandle::poll 阻塞等待消息

- **位置**：`crates/protocol/src/process.rs:197-208`
- **置信度**：0.70
- **状态**：dismissed
- **描述**：`poll()` 方法调用 `self.recv()` 阻塞等待消息。方法名暗示非阻塞轮询，但实际行为是阻塞。
- **dismiss 原因**：阻塞 poll 是单线程进程通信的刻意设计。已有 try_recv() 方法提供非阻塞读取。方法命名虽可改善，但行为非 bug。若渲染进程挂起，poll 将永久阻塞。
- **建议修复**：重命名方法或使用非阻塞读取。

---

#### IPC-03 [实现缺陷] bincode 反序列化无格式验证

- **位置**：`crates/protocol/src/serialize.rs:11-13`
- **置信度**：0.55
- **状态**：dismissed
- **描述**：`deserialize` 直接反序列化原始字节，仅依赖 bincode 内置验证。
- **dismiss 原因**：Protocol 用于父子浏览器进程间的可信 IPC，不是不可信网络输入。PipeTransport 已有 16 MiB MAX_FRAME_SIZE 限制，对内部通信添加额外验证是过度设计。对于来自不可信渲染进程的 IPC 数据，缺少额外的格式/大小限制。
- **建议修复**：添加反序列化数据的最大大小限制。

---

## 统计总览

| 维度 | 高 | 中 | 低 | 合计 |
|------|----|----|----|------|
| 实现缺陷 | 0 | 1 | 1 | 2 |
| 可靠性 | 0 | 0 | 1 | 1 |
| **合计** | **0** | **1** | **2** | **3** |
