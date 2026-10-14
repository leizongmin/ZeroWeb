# Mutation、Range 与 Shadow DOM

<!-- verified_commit: e523b1c73b96360a6451948f0b748b7b4657750f -->
<!-- verified_at: 2026-10-14 -->

## Module Structure

覆盖原生树变更记录、Range、ShadowRoot 与 slot 分配。微任务与 JS observer 选项见 [事件循环](../../scripting/event-loop/tasks-and-checkpoints.md)；身份同步见 [快照与 live DOM](../../scripting/dom-binding/identity/snapshot-and-live-dom.md)。

### User Task Navigation
原生边界判据在 `crates/dom/src/range.rs`、`crates/dom/src/tests/tests_range.rs` 和 `crates/dom/src/mutation.rs` 内测试；本次只核对源码，不运行 observer/浏览器验收。

## Gotchas
- 原生 MutationObserver 只是 callback 容器，`process_mutations` 对全部 observer 通知同一批记录；observe 选项、subtree 过滤与微任务队列不能从此类型推断。 (`crates/dom/src/mutation.rs`, `crates/dom/src/document/mod.rs`)
- `take_mutation_records` 和 `process_mutations` 都用 mem::take 消费同一 pending 队列；先取记录再 process 不会再次递送原批次。 (`crates/dom/src/document/mod.rs`)
- `clear_observers` 只清回调列表，不清 pending 队列；不能把它直接视作完整 JS disconnect 的实现。 (`crates/dom/src/document/mod.rs`)
- Range 的 set_start/set_end 不检查偏移或边界顺序，仍返回 Ok；同容器收集仅按 children 切片，不能把任意 Text 范围当作已实现的部分文本提取。 (`crates/dom/src/range.rs`, `crates/dom/src/tests/tests_range.rs`)
- Range 文档头称文本偏移为 UTF-8 字节，实际 insert_node 按 chars 拆分并 clamp；它也不是 Web IDL 的 UTF-16 code unit 口径，桥接修改必须核对转换。 (`crates/dom/src/range.rs`)
- Text 容器 insert_node 保留 head 原节点并创建 tail，插入节点位于两者之间；越界偏移仍创建空 tail，不能按注释“文本不拆分”省掉节点。 (`crates/dom/src/range.rs`)
- 跨容器 Range 收集目前只拼 start 侧剩余 children 与 end 侧前部 children，没有完整公共祖先遍历；compare_boundary_points 也主要比较相同容器，不代表任意树序。 (`crates/dom/src/range.rs`)
- ShadowRoot 实际挂在 host.children 且 parent 指向 host；封装依赖查询路径显式跳过嵌套 ShadowRoot，而非物理独立树。 (`crates/dom/src/document/shadow.rs`, `crates/dom/src/document/mod.rs`)
- 原生 shadow_root 返回侧表中的 root，不检查 open/closed mode；closed 对脚本的可见性应在绑定层处理，不能通过此 getter 自动获得封装。 (`crates/dom/src/document/shadow.rs`)
- resolve_slots 只分配 host 的直接 light children，并取第一个匹配 slot；没有 slot 属性的子节点匹配 name 为 None 的默认 slot，不与显式 name="" 完全等价。 (`crates/dom/src/document/shadow.rs`)
- get_assigned_nodes 没有分配时返回空，不返回 fallback children；渲染消费者需自己选择 fallback，不能按 resolve_slots 的说明文字假设已展平。 (`crates/dom/src/document/shadow.rs`)

## Architecture
- attach_shadow 验证 host 是元素且无既有 root 后，同时设置 host 引用、父子关系和 host→root 侧表；新增其他创建路径必须保持三处身份一致。 (`crates/dom/src/document/shadow.rs`)
- `:target` 由 Document URL 的末个 # 后串百分号解码再查 id_map，DOM 查询与 CSS matcher共用该权威；导航层未注入 URL 时恒不匹配。 (`crates/dom/src/document/target.rs`, `crates/style-system/src/matcher/mod.rs`)
