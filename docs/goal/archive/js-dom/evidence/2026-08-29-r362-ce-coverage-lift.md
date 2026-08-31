# R362 — DC-4 覆盖率提升：custom_elements.rs 89.0%→91.9%（<90% 候选清单清零）

**日期**: 2026-08-29
**切片**: DC-4 覆盖率持续提升（custom_elements.rs——最后一个 <90% 源码文件）
**改动面**: `dom_bindings/tests_ce.rs`（新，3 测试）+ `mod.rs`（模块注册，v8 门控）

## 1. 背景

已知 Fail 集合余 13 全部深结构/基建域（R360/R361 巡检定域），本轮转 DC-4 覆盖率线。
custom_elements.rs 89.0%（154/173）是覆盖率矩阵最后一个 <90% 文件——`<90% 提升候选`
清单中唯一项。

## 2. 改动

新增 `tests_ce.rs`（原生沙箱直连 Rust 侧 custom_elements 模块，不经 shim 字符串桥）：
1. **connect/disconnect lifecycle**——native appendChild/removeChild 驱动
   `notify_connect_after_insert`（connect 臂 + R3271 fast-path 跳过 div）与
   `notify_disconnect_after_remove`；嵌套子树（collect_custom_subtree 多层 pre-order）
   逐层派发；**已连元素移入 detached 容器**形态覆盖 disconnect 臂（was=true 且新 parent
   未连——removeChild 走的是另一函数，本臂需 insert-into-detached 形态）+ 移回 body 的
   再 connect。
2. **attribute 派发**——native setAttribute/removeAttribute 驱动
   `read_attr_change_context`（old 预读三态 null→v1→v2→null）+ `notify_attribute_change`；
   div fast-path（无连字符 tag 不派发）对照。
3. **守卫臂**——polyfill hook 未注册时全路径静默不抛。

## 3. 观测 finding（⚠️ R363 勘误撤销）

~~嵌套 insert（`ce.appendChild(inner)`）触发父子双 connect~~ → **R363 勘误：装配伪影**
（两段 join 拼接使首条元素双渲染，见 evidence/2026-08-29-r363-ce-double-connect-erratum.md
——逐段读数复刻实证每连接态真转恰一次派发，mark/unmark 簿记 spec-correct，无 registry
簿记缺陷）。CE registry 专项中本项删除；专项剩余内容 = per-realm registry 路由。

## 4. 验证

| 门 | 结果 |
|----|------|
| custom_elements.rs | **89.0% → 91.9%（+2.9pp，154/173→159/173）——<90% 候选清单清零** |
| dom_bindings 源码总覆盖 | 94.35% → **94.43%**（+0.08pp，超 R36 基线） |
| 全部 dom_bindings | 96.01% → **96.12%** |
| engine 单测 | v8 2490（+3）/ quickjs 1472 全绿 |
| clippy / fmt | v8 + quickjs 双矩阵 `-D warnings` 零警告 / 无 diff |

## 5. 后续

覆盖率矩阵已无 <90% 文件；dom_bindings 覆盖率进入 94%+ 平台期，后续提升点为 dispatch_connect
内部守卫（98/101/111/114/119 行——需 hook 缺失/非函数/空 pairs 各形态）低 ROI。DC-4 持续
维护。主线剩余：M5/M7 default-on（待用户点名）。
