# s112：解析期外链 async/defer/trickle 调度 + 投递体内闭合转义

日期：2026-10-27。目标域：`scripting-1` execution-timing 慢取回外链时序族
（050/085/086/088/096/099/103/110/111/112 十页 pipe=trickle）+ script-defer /
delay-load-event-1 顺带面。单文件轮：`tests/wpt-runner/src/testharness.rs`。

## 入场定谳（探针分组）

trickle 族实为十页而非挂账估计的六页（~11 行估计失效）。按根因分组：

- **调度缺失族**（本轮主刀）：050/085/086/088/103/110/111/112——runner
  `inline_local_scripts` 把解析期外链一律原位内联直执，async/defer/trickle 三类
  时序语义全部塌缩为「立即同步」；
- **投递截断族**：099（include-11.js 的 srcdoc 串含字面 `</script`，投递面在该处
  提前封口，单元整体 SyntaxError 零日志）；script-type-and-language-js.js 43-45 行
  **注释内** `</script>` 同根因——`testParserInsertedDidNotRun`（定义于注释之后）
  未注册，页面首内联脚本 ReferenceError 短路（基线仅 1 行页级 Fail，456 子测不可见）；
- **write 面族**（不在本轮）：096 write(defer)、142 xlink:href、143 eventOrder。

## 机制（三件套）

1. **分类器 `script_sched_unit`**（预处理时刻）：nomodule/nonce/module 单元维持
   原路径（CSP 槽位/prepare 门/模块取回链各有专用面）；其余按 async 属性 →
   `async`、defer 属性 → `defer`、trickle 查询 → `block` 分类；归类冻结于预处理
   （spec prepare 定格，运行期 removeAttribute 不重分类——103/111/112 判据）。
   `trickle_delay_ms` 与 shim fetch 通道同语法同 30s 上限。
2. **页面侧调度器 `__zwSchedRegister`**：async=trickle 后 setTimeout 执行；defer
   入 `_s.deferred`，`__zwSchedArmDrain` 在 interactive readystatechange 之后、
   DCL 之前统一 drain（文档序）；block=trickle 后执行（解析阻塞近似——不暂停后续
   投递，判据页只断言 log 序，可观察面等价）。生命周期链接线：drain 臂装在 chain
   task1 内部（setTimeout FIFO 保证 interactive 面先跑——103 的 interactive 断言
   依赖此序）；DCL 派发幂等化（`_zwDclFire112` 旗标）改为 drain 完成回调；
   load 门加 `__zwSchedPending()` 条件。
3. **投递体内 `</script` 转义**：`</script` → `<\/script`（字符串/正则/注释各语法
   位 `\`+`/` = `/` 等值；与 any_js_window_wrapper 既有转义同口径）。

### 中间态回归（一乌龙，当日修复）

首轮终扫 load-error-events-1 四子测 Pass→Timeout：调度单元的 load/error 投递尾读
`document.currentScript`，而 setTimeout/drain 相位执行时 classic currentScript 已清
（getter 返 null，part06 R3258）——事件永不派发，event_test 挂死。修正：注册语句
同步原位执行时 currentScript 仍指投递元素，立即捕获为 `_zwEl` 传入闭包，投递尾改
`script_event_dispatch_tail_el(event, "_zwEl")`。**已知限制**：仅投递尾覆盖，体内
currentScript 读取在调度单元仍为 null（挂账）。

## 验证

- runner bin **238/0**（236 基线 + s112 分类/转义两钉：
  `inline_sched_classification_wraps_defer_async_trickle_s112`、
  `inline_body_closing_tag_escaped_s112`）。
- execution-timing 定向：22F→13F，050/085/086/088/099/103/110/111/112 九页
  Fail→Pass，归一 diff 零 P→非P（/tmp/s112-post-execution-timing.log）。
- 全 scripting-1 终扫 **398P/152F/37T+15N → 638P/367F/37T+15N**：
  - 十一页 Fail→Pass：execution-timing 九页 + **script-defer**（defer 单元不再抢跑
    t.step 前断言）+ **delay-load-event-1**（trickle(d3) 单元不再原位直执）；
  - **script-type-and-language-js 解锁**：页级 Fail（1 行）→ 456 子测全注册，
    229 Pass / 227 Fail——页保持 Fail，227 失败为先前不可见的真实 type/language
    分类缺口（候选独立切片），非 P→非P；
  - 归一 diff（/tmp/s112-final-scripting-1.log 存档，vs /tmp/s112-base-rows.txt）
    除上述外零变化，零 P→非P。
- fmt 过；clippy `-p zero-wpt-runner --all-targets -D warnings` 过（as_deref 误加
  被 needless_option_as_deref 抓回——src 本就是 `Option<&str>` Copy）。渲染面零
  依赖——reftest/product-smoke 未跑。

## 挂账

- 调度单元体内 `document.currentScript` 读取仍 null（投递尾已覆盖，体内读取需 shim
  侧 idx 通道，遇实证再立项）。
- `block` 类不真暂停解析（050 等价仅判据面；真解析暂停需 runner 投递面分段，成本
  远超收益，除非出现断言解析阻塞的语料）。
- script-type-and-language-js 227 失败（type/language 分类真实缺口）。
- 096 write(defer) / 142 xlink:href / 143 eventOrder（write 面族，原挂账不变）。
