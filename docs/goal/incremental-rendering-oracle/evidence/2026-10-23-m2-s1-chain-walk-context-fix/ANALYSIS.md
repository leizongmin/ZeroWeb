# M2 切片 1：增量链走上下文修复——伪元素挂载与容器链同源（反例 B/C 收口）

**运行身份**：基线 HEAD `fd94eef92`（上轮 M1 style 基线），本轮改动 `crates/style-system/src/lib.rs`（伪元素块提为 `attach_pseudo_elements` 共享 helper + 增量链走补伪元素挂载与容器上下文）与 oracle 测试模块。
**环境/守卫**：`./target/test-guard --compile-first … cargo test -p zero-style-system`；fmt --check、定向 bench-gate（`ZERO_WEB_BENCH_CRATES=zero-style-system`）、`make test` 全量。

## 根因与修复

`compute_styles_incremental` 的祖先链单节点重算原先直接调 `compute_element_style_internal`，与全量递归相比缺两块上下文：

| 缺口 | 机制 | 生产可达性 |
|---|---|---|
| 反例 B：伪元素丢失 | `::before/::after/::marker/::placeholder/::selection/scroll-marker 族/::first-letter` 计算只存在于 `compute_styles_recursive` 内联块；链走重算的祖先样式（无伪字段）直接覆盖 `styles` 中的完整条目 | **可达**——`paint_only_incremental`（pipeline/mod.rs:2334）与 getComputedStyle 代际缓存增量 cascade（computed_style_cache.rs）两个消费者的每次增量都会链走重算祖先 |
| 反例 C：容器上下文丢失 | `@container` 条件与 cq 单位按 `self.container_chain` 求值（lib.rs `compute_element_style_internal` 起步处 `container_context_from_chain`）；增量路径从不压链 → 条件恒 false | **可达**——同上两个消费者；容器页 + paint-only mutation / gCS 增量即触发 |

修复（共享路径，非调用方补丁）：

1. 伪元素块（原 lib.rs 650–1022，373 行）**原样提取**为 `StyleSystem::attach_pseudo_elements`，全量递归与增量链走调用同一实现（zeroweb-guidelines #6：双路径共享等价断言）。
2. 增量链走（自根向下）每算完一个祖先：挂伪元素 → `parent_custom` 推进 → 是查询容器（尺寸或 style-only，R4124/R4582 同判据）则 `container_entry_for` 压链 → 进入变更子树递归前弹回本项压入的条目（多 changed 项各自干净链起，≡ 全量递归平衡 push/pop）。

## 验证

| 检查 | 结果 |
|---|---|
| oracle 套件（9 序列） | **9/9**——B/C 两步由 Divergent 翻转为 Equal（含 CSSOM 同帧变更步：`::before` 规则移除后祖先伪样式消失、容器收窄后 @container 失效，钉死"重算"而非"沿用旧值"） |
| 反例 A（`:only-child` 兄弟失效，API 契约缺口） | 保持 Divergent 检出 ✓（未修复，属调用方 changed 集完备性契约） |
| style-system 全量 | 2445/2445 PASS |
| fmt / clippy / make test | 本轮收尾跑（见提交说明） |
| 定向 bench-gate | `ZERO_WEB_BENCH_CRATES=zero-style-system`（见提交说明；无伪规则页 helper 早退于 `is_q` 探测，理论开销≈0） |

## 过程记录

- C 的 CSSOM 收窄步初版 changed 只传 `[p1]` 而失败——归因：CSS 变更影响所有匹配 `p` 的兄弟，p2 不在增量范围（反例 A 同族的 changed 集完备性契约），**不是**容器修复回归；按契约补全为 `[p1, p2]` 后 Equal。oracle 对契约违约的检测再次有效。
- 提取手术第一次因插入点误算导致块复制（编译期 unclosed delimiter 暴露），`git checkout` 回滚后按正确顺序重做；块内逻辑零改动（移动 + 参数化）。

## 边界与未覆盖

- 反例 A 保持开放：结构型伪类/兄弟选择器的失效需要调用方传完整 changed 集；生产现路由（单目标 paint-only mutation）不可达，若未来 mutation 路由扩展需先补此契约。
- `:has()` 全量回退计数（DC-2"全量回退有计数"）、Shadow/slot/字体失效序列：下一切片。
- 布局几何快照（M1 剩余）、帧复用/缓存生命周期（M3）、检查成本（M4）未动。
