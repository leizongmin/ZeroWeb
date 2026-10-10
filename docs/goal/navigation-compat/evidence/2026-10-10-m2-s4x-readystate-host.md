# M2-S4X — runner readyState 状态宿过渡（2026-10-10）

**通道**: `make testharness-navigation`（test-guard 包裹；FILTER 单案最小复现迭代）
**跨域验证**: navigation 全量 per-subtest 精确 diff 对 S4W 终态
[2026-10-10-m2-s4w-strict-narrow-family.txt](2026-10-10-m2-s4w-strict-narrow-family.txt)；
net-api 全量（14,234 行）+ csp-inheritance（56 行）before/after 双臂
（/tmp/zw-before-{net,csp-inh}.log vs /tmp/zw-after-{net,csp-inh}.log，本轮后不留档）
**前序**: [2026-10-10-m2-s4w-strict-narrow-family.md](2026-10-10-m2-s4w-strict-narrow-family.md)

## 切片内容

| 改动 | 文件 | spec/WPT 锚点 |
|---|---|---|
| **runner readyState 状态宿过渡**（t8m renderer/tab_scripts 路径镜像）：① `run_page_scripts_strict` 前置 `script_set_ready_state("loading")`（页面脚本语义位置 = parser-inserted classic script）；② 生命周期 timer 任务内原子过渡——interactive + readystatechange → DCL → complete + readystatechange → load → pageshow（内联进既有单串生命周期脚本，时序不变） | `testharness.rs`（runner 单一 chokepoint） | [dom-document-readystate](https://html.spec.whatwg.org/multipage/dom.html#dom-document-readystate) 三态；WPT navigate-history-push-not-loaded「Document must not have loaded yet」 |

**为什么此前缺**：`__zwReadyState` 状态宿（t8m）只接了 renderer/tab_scripts 两条宿主路径，
testharness runner 路径未注入——getter 缺省 "complete"（该缺省对「全解析后统一执行脚本」
的 run_script 模型注释为准确，但对语义位置在解析期的页面脚本不准）。

## 跨域爆炸半径测量（S4W 记账项的兑现方式）

S4W 记账：全语料 21 案断言 document.readyState、共享 runner 语义变更须独立轮次跨域测量。
本轮 before/after 双臂落地：

| suite | before | after | Δ |
|---|---|---|---|
| testharness-navigation（本 goal 全量） | 394 P / 476 | **395 P / 476 = 83.0%** | +1（预期案） |
| testharness-net-api（fetch/xhr/url/... 六 corpus） | 14,234 行 | 14,233 行 | **+1**：xhr/sync-xhr-and-window-onload Timeout→Pass（红利——「sync XHR should not fire window.onload synchronously」此前 case 级 Timeout） |
| testharness-csp（inheritance 叶，56 行） | 56 行 | 56 行 | **零变化**（逐行恒等） |
| testharness-web-animations | — | — | N/A：readyState 引用案（document-timelines）不在本地导入运行面（timing-animation-compat 子集外）；exit 1 = 空集既有语义 |

**零 Pass 回归**：navigation diff 恰一行（push-not-loaded Fail→Pass）；net-api diff 恰两行
（同一案 Timeout→Pass 改写）；csp 逐行恒等。

**不修的面**：readystatechange 派发复用 t8m `script_transition_ready_state` 同款语义
（document 上派发）；reftest 路径（reftest_scripts.rs）不注入——reftest 页面无 readyState
断言面，零触碰。

## 数字

| corpus 域 | S4W 后 | 本轮 | Δ |
|---|---|---|---|
| 全量 | 394 P / 476 = 82.8% | **395 P / 476 = 83.0%** | +1（navigation-history-push-not-loaded 收口） |
| navigation-api | 234 / 255 | **235 / 255**（92.2%） | +1 |

## 质量门禁

- `cargo clippy --workspace --all-targets -- -D warnings`：零 warning（testharness.rs 变更轮）。
- `cargo fmt --all -- --check`：零 diff；`git diff --check`：零问题。
- `make test`：全绿（数字见本轮终态行）。
