# editing-compat — join paragraphs 簇清零：skipWs 方向反转根因 + wire 探针定位（2026-10-31）

承接 [行内壳变体收敛](2026-10-31-join-inline-shell-family-convergence.md)
（fix9 = 5,698P/1,820F 基准）。本轮交付 **paragraphs fd/select-boundary 簇
（632F）清零**：`_zwCeSkipWs` 方向反转根因定位 + 方向参数化修复，
join 三案 **5,698P/1,820F → 6,330P/1,188F（75.8%→84.2%）零回归**。

## 1. 残余 1,820F 两簇（fix9 账）

- **paragraphs fd 316F + select-boundary 316F**：`\n` 块间空白几何
  （`<div normal>abc[]</div>` + 换行空白文本 + `<div nowrap>def</div>`）——
  实际 `"\n<div nowrap>def</div>"`：左块内容消失、`\n` 残留、壳丢失。
- lrlp 三方法各 396（**无**空白分隔几何——语料为 JS 串接直邻
  `` `abc[]<div…>` ``，无文本节点）：与本轮根因无关，保持不动。

## 2. 探针定位（wire 追踪 + JS 阶段标记）

单案探针页（临时，已删）：`zw-probe-fd.html`——setup
`<div style="white-space:normal">abc[]</div>\n<div style="white-space:nowrap">def</div>`
+ `sendDeleteKey()`，期望壳形态。插桩（全部临时，已移除）：
callbacks.rs 两个 wire 回调 + js_dom_bridge.rs apply 臂 eprintln
（`ZW_WIRE_TRACE=1`），另注册 `__zw_wire_trace_note` 通道供 shim JS 侧
（不可读 env，typeof 探测门控）打阶段标记。

- **第一轮 trace**：整个动作唯一 wire op = `remove_child_at_path path=0,0`
  → 引擎 apply 命中 host/0/0 = **"abc" 文本（caret 侧内容）**；探针 Fail，
  actual `"\n<div nowrap>def</div>"` 与语料残余同形。join 应有的 def 移除
  /分隔消费/壳 insert 全部不在 wire 上。
- **加阶段标记后 trace**：
  `joinApply(el, dst=左块, srcP=el, sn=左块)`——**startNode = caret 侧块
  自身**；takeLine 递归把 "abc" 提取为首行 → 左块 removeChild("abc")
  → emit1 寻址 path [0]+i 0 → 引擎删 "abc"。

## 3. 根因：`_zwCeSkipWs` 方向恒反向

`_zwCeSkipWs(n)` 跳空白文本时恒走 `previousSibling`——这是 **backspace
前邻臂**（8718，传 `bn.previousSibling`）的正确方向；但被三个**后邻臂**
复用：fd 块界（8816）、fd select（8800）、backspace select（8742），均传
`x.nextSibling`。`</div>\n<div>` 几何下从空白文本继续 backward 回走命中
**前邻**（如 `fnx = skipWs(左块.next = "\n") = 左块自己`）→ 后邻查找返回
caret 侧块 → `joinApply(el, 左块, el, 左块)` → caret 侧内容被当 join 首行
提取移除。lrlp 无空白分隔（next 直接是元素节点，skipWs 原样返回），
396×3 不动即是该根因边界的反向印证。

## 4. 修复（part01.js，净 +8/-3）

`_zwCeSkipWs(n, back)` 方向参数：`back ? previousSibling : nextSibling`；
backspace 前邻臂传 `true`，三个后邻臂走默认 forward。

## 5. 验证账

- 探针修后 **Pass**；wire 序列正确化（sn=右块，def 移除寻址 `[2,0]`
  精确命中，i 取 splice 前序语义核对）。
- shim 全量拼接 `node --check` ✓。
- **全套三案**（test-guard `--per-proc-mem 4 --total-mem 8 --time-limit
  5400` + `ZW_SCRIPT_TIMEOUT_MS=300000`）：**6,330P/1,188F（84.2%）**——
  对 fix9 逐测试 ID：**F→P 632 / P→F 0**（632 = paragraphs fd 316 +
  select-boundary 316 全清），结果行 7,518/7,518 恒等。
- 分面账（F 计）：paragraphs backspace/fd/select-boundary = **0/0/0**；
  lprl = 0/0/0；lrlp = 396×3（唯一残余）。
- **插桩移除后干净二进制复跑：逐位复现 drift=0**（7,518 结果行 diff
  为空）——插桩零语义影响确认。
- **合并终验（rebase 拉入并行流 854629e5c（R5351 orphan inline 字体）+
  d5ec12d94（s167 外链槽位级联）后 union 树复跑）**：**6,330P/1,188F
  逐位复现 drift=0**——本轮账与 union 树无交互漂移。
- `cargo fmt --all --check` ✓；clippy（runner crate）0 warning。
- make test（`CARGO_BUILD_JOBS=2`）engine lib **3,611P/11F = 已知红集**
  （9 fetch 域 + r113 + s24 渲染流债）零新增。

## 6. 剩余

lrlp 1,188F = 引擎收敛簇残余（§1 evidence ④ 形态：①`abcabc` caret 侧
文本双计 56/方法 + ②行内 run 后继文本壳落引擎丢失）——JS 侧形状 Node
复算正确，属 D1/apply 引擎收敛域，下一轮首位候选。
