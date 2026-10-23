# M2 切片 34 — 挂账 F：html-semantics 套件帧驱动 rAF 默认启用（2026-10-23）

## 范围

s33 挂账 F：`waitForRender` 快照滞后（shim OFF 模式 rAF 同步 stub 不驱动
render，测试体内 gBCR 读上一拍快照）的 runner 侧收口——html-semantics
套件 make 入口默认启用 `ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（R2713a/
R3254-KP4 既有开关，keyboard 套件同款入口模式），使双 rAF 走真实帧派
发、probe 拍间刷新 rect 快照。

## 落地（R5235）

Makefile `testharness-html-semantics` target 加
`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1`（keyboard 同款：runner 二进制语义
不变、套件入口层开启；直接调 runner 不带环境变量时维持 OFF 旧路径）。

## 对账

| 面 | OFF | ON | 同 key diff |
|---|---|---|---|
| dialog filter 82 files 411 subtest | 337P/73F/1T | 345P/65F/1T | **REG 0 / GAIN 8** |
| testharness-html 合成面 7 files 14 subtest | 14P | 14P | **逐字节一致** |
| forms 域 401 files 4627 subtest | 3899P/707F/20T | 3899P/707F/20T | **逐位一致（REG 0 / GAIN 0）** |

GAIN 8 = closedby-complex normal DOM nesting 的 clicking dialogB/popoverB
全案（s33 探针实证的快照滞后面，帧驱动后快照新鲜、命中循环拿到 dialogB
真实盒，不再误 retarget）。forms 域 4627 subtest 双跑逐位一致 + 合成面
逐字节一致：帧驱动对该两面零行为变化（rAF 直接用户语料全在 dialog 域）。

## 侦察（挂账 G 根因钉定，未动）

`_zwDialogModalFocusGateOk`（part01.js:1068）反走模态栈只判**栈顶**的
shadow-including contains——嵌套 modal 栈中下层 modal dialog 作为栈顶的
DOM 祖先被 gate 误拒 → runner `resolve_pointer_target` 命中循环排除其盒
（best 空）→ 回落栈顶模态 backdrop 语义（`msel`）。spec 嵌套语义应为
top-layer 元素及其 shadow-including 祖先链豁免。gate 被 focus gate /
hit-test inert 排除 / 指针命中三路共用（part06 同源引用），修需独立切
片全对账（含 uievents/pointer 语料面）。

## 已知边界

- 全量 `testharness-html-semantics`（无 filter，2594 用例）实测
  >5400s 跑不完（`--json` 缓冲到最后、超时全丢）——全量对账基建缺口
  记挂账 I（runner 增量输出或 make 分域入口），不阻本切片（分域对账
  已覆盖 rAF 直接行为面：waitForRender 的 3 个 html 语料全在 dialog 域）。
- pkill 误伤记录：本轮清全量跑链时 `pkill -f test-guard` 模式过宽，
  误杀并行流一个 `cargo test --no-run` 编译进程（缓存无损、对方自然重
  试）。后续按 pid 精确杀。

## 门禁

runner 侧 Makefile 一行 + 注释，无 Rust 代码变化、零新增依赖；runner
release 二进制未动（语义开关在环境变量层）。
