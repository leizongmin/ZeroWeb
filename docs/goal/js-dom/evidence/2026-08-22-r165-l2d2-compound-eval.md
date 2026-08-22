# R165 Evidence — L2-d2 compound 匹配器基建 + gate 评估结论（M1）

**日期**: 2026-08-22
**Commit**: `296c173b5`（rebase 后；原 `f1c731459`）
**切片**: M1 L2-d2 尝试——compound 本树查询（结论：回退到 L2-d1 基线，匹配器基建落地）

## 评估过程（两轮迭代）

### 尝试 1：完整 compound（`.cls[attr]` 等）

- 实现：`_queryTreeByCompound`（tag/#id/.classes 多个/`[attr="v"]` 含单引号，
  compound 内组合，节点局部判定）
- 结果：ParentNode-querySelector-All **1072P/902F**（基线 1937P/37F）——
  大规模回归
- probe 定位：`docQ:1` 但 `detQ/fragQ/elQ:0`——**element/fragment 上下文的
  identity 依赖 wrapper 语义**，真实节点返回破坏其消费面

### 尝试 2：收窄纯 `#id`（getElementById 形态）

- 结果：1936P/38F——仅 `#id-li-duplicate` 一处回归
- 根因：**duplicate-id** 场景树首命中与 JSON 往返首命中的 **identity 分歧**
  （WPT 断言特定对象）

## 结论

compound 迁移**不能按形态逐个 gate**——element/fragment 消费面的 wrapper
依赖 + duplicate-id identity 分歧需要 **L2-d3 统一匹配器 + identity 桥**
（wrapper ↔ 真实节点双向映射），而非 per-form 切换。已落地：

- `_queryTreeByCompound` 匹配器基建（tag/#id/.class/[attr=v]，守卫同 L2-d1）
- queryBody gate 维持 **L2-d1 纯 tag 基线**（全中性）

## 基线（回退后全中性）

| 面 | R164 基线 | R165 |
|---|---|---|
| ParentNode-querySelector-All | 1937P/37F | **1937P/37F** |
| 全量 dom WPT | 9516P/347F/18T | **9516P/347F/18T** |
| zero-engine | 2310 | 2310 全绿 |

## L2-d3 方向（R166+）

统一匹配器（part05 `_matchComplexAgainst` + nodeInfo）+ **identity 桥**：
- 查询面统一经一个入口（wrapper 与真实节点的身份映射表）
- 或 element/fragment 查询全部迁到本树（消费面同步切换）——影响面大，
  须 RFC 级切片计划

## 验证

- `cargo test -p zero-engine`：2310 全绿；fmt/clippy 干净；双路径全量一致
