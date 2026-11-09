# M2 切片 369：全 strict 车道迁移——eval wrapper 自 runner classic 车道退役（域零漂移）

**日期**: 2026-11-09
**目标对**: s368 挂账「全 strict 车道迁移」（设计已勘定：sandbox in-execute catch 面、quickjs 腿平行、reportError 原值透传、跨脚本词法重声明早错语料面 A/B）
**结果**: 一刀落地后域 the-script-element **零漂移**（公共 1136 键零状态变化）、s368 结果保持（load-error-events-3 T→1P）、全语义面 sweep **script 车道零 P→F**（ feared 重声明早错语料信号未现）；s335 `[error, body-μt, load]` 序由 sandbox in-execute catch 面双腿测试钉死。

## 施工（6 文件）

1. **script-sandbox/lib.rs**：trait 新 `execute_classic(code, catch_js)`（default
   `EngineUnavailable`，parse_check 前例）；`__zw_e` 名与 engine tail 两侧跨 crate 契约
   （双侧注释）。
2. **v8_runtime.rs**：`execute` 塌为 `execute_with_catch(code, None)`；新
   `execute_classic` = `execute_with_catch(Some(catch_js))`。
   - **MicrotasksExplicitGuard**：rusty_v8 Isolate::new 默认 `MicrotasksPolicy::Auto`，
     顶层 `script.run` 解栈时**自动排空微任务**——早于宿主侧 catch 面，序面被破坏
     （首版红态 `body;catch;load;` ≠ `catch;body;load;`）。catch 车道 run 前置 Explicit、
     Drop 恢复 Auto（裸指针镜像 IsolateEnterGuard 形态；声明先于 scope 链 = 析构晚于
     scope 链）。嵌套宿主回调子 run 在 kAuto 下本就不自动排空（JS 栈非空），全库零影响。
   - `run_catch_face`：挂异常原值→global `__zw_e`→`tc.reset()`→compile+run catch_js
     （吞错+reset）→`perform_microtask_checkpoint()`→native `delete __zw_e`。
   - compile-fail 与 runtime-fail 两点均进 catch 面（timeout watchdog kill 臂不进，
     与 wrapper 外层面同构）。
   - `v8_try_catch_message!` 宏内 `let try_catch = $try_catch` **消耗实参**——msg 提取用
     reborrow `&mut *try_catch`，catch 面在 msg 提取**之后**跑（face 会 reset TryCatch，
     顺序双向耦合）。
3. **quickjs_runtime.rs**：同构 `execute_with_catch`；catch 面 = 挂 `__zw_e`=caught 原值 →
   `eval_global_raw(catch_js)`（JS_Eval GLOBAL 吞错）→ 有界 job 排空（10_000）→
   raw eval delete 片。
4. **script_gen.rs**：`script_run_classic_page_strict` 退役；`script_run_classic_page_impl`
   收 3 参（report_tail 机器拆除，字节级基线输出不变）；新
   `script_classic_catch_tail(source_url)` = IIFE 包 `STRICT_UNCAUGHT_REPORT_TAIL` +
   `__zw_SRC_EXPR__` 替换（IIFE 防 wrapper 的 `var _zwCs` 全局污染；reportError 透传与
   error 后 load 派发机器原样随迁）。
5. **js_dom_bridge_tests/part09.rs**：s335 守卫迁移 `execute`（设索引）+
   `execute_classic(body, catch_tail)`，断言面不变（`body|error:boom|mt|load` 序 +
   Err 文本 + `__zw_e` 跑后即删）；s336 同迁（五参 adapter 断言不变）。
6. **webview.rs**：strict classic 臂塌二叉（strict → 直编译 `full` = classic_code +
   `//# sourceURL`（换行先行）；非 strict → 前置串零漂移）；执行臂
   `execute(script_set_current_script(idx))` + `execute_classic(&full, &catch_tail)`；
   sentinel 尾读块整删（Err → combined_err → first_strict_error 同契约）。

## s335 序保真机制

`[catch 同步, 体内 μt, load μt]`：Explicit 策略下 checkpoint 由 catch 片尾
`perform_microtask_checkpoint` 显式触发——catch_js 同步体先于体内已排队 μt（catch 片
enqueue 在体 throw 之后 = FIFO 尾），catch_js 内排的 load μt 再后。V8/quickjs 双腿各一
persistent 上下文测试钉死（含 compile 期 SyntaxError 同进 catch 面）。

## 验证

- **单测**：sandbox all-targets **440P/0F**（含新 3 测：V8 GDI catch 序、V8 compile-fail
  进 catch、quickjs 镜像）；engine classic 守卫 **8P/0F**（s335/s336 新机制同断言面）。
- **solo ×3**（红绿锚点）：load-error-events-3 **1P**（s368 结果保持）；
  checkpoint-after-window-onerror **2P**（s335 时序面）；evaluation-order-1 **4P**。
- **域 join**（A=/tmp j368 = s368-b 1137 键 vs B=本刀域跑 1137 键；LC_ALL=C 全键稳定态
  join；**本轮新坑：A 侧文件是 s368 轮 pipe 稳定态分类文件非 raw 行，直接按「剥首词」
  建键得到 `:: subtest` 假键集 common=0 假象——join 前先核两侧文件格式**）：
  - 公共 1136：**1128 Pass|Pass + 5 NotRun|NotRun + 3 Timeout|Timeout，零状态变化**；
  - A-only 1 = le3 文件级 Timeout 键；B-only 1 = 子测 Pass 键——s368 位移原样保持
    （item 38 人工合并判读）；
  - 残 T=3 基建面 trio 逐名同构（credentials-setTimeout / instantiation-error-8 /
    script-crossorigin-network，s102/s124 挂账）；域残 F=0；零 P→F。
- **全语义面 sweep A/B**（A=evidence/logs/s366-b 快照 8417 键 8078P/285F/43T/11NR vs
  B=本刀 8478 键 **8144P/283F/40T/11NR**；全键 join）：
  - 预期已知位移 ✓：le3 文件T→1P（s368）、code-cache-base-url 文件T→6P（s367）；
  - 其余三处逐一 triage 归因**非本刀**：
    ① select-home-end-pagedown-pageup-detailed.optional.html A 4 named Fail → B sweep
    file-level Fail = **sweep 态瞬态捕获**（本刀 solo 复跑 4 named Fail 与 A 逐名同构，
    页存活非死）；
    ② checkable-active-onblur-with-click following-checkbox P→F = **s368 二进制同红**
    ——git stash A/B 协议：摘除本刀改动重建 release runner 后 solo ×2 皆 **4F/3P 逐子测
    同构**（s366-b 以来的页面/testdriver 漂移 + sweep 载荷方差；s369 二进制 solo ×2 同形）；
    ③ select-event.html A-only 文件级 Timeout = A 面瞬态（B sweep + solo **270P** 全绿）；
  - **script 车道面零 P→F**——设计期最重的「跨脚本词法重声明早错」语料信号未现
    （8478 键全查；eval 局部 accessor 导出的 R201 机器随 wrapper 退役，无页依赖）。
- **make test**：v8 workspace 腿聚合 **9331P/7F**（abort 于 js_dom_bridge 腿 3821P/7F，
  EXIT=2；7 红既有集逐名同构零新增红——img_runtime_deferred_settle_r5275 /
  svg_img_settle_density_r5277 / fetch_filtered_response_type_matrix /
  iframe_sw_messages_dispatch / response_body_readable_stream_r2967 /
  websocket_error_nonterminal / websocket_event_flow）；P 较 s368（9329）+2 属并行流
  组合态漂移；quickjs 腿未达同形态。
- fmt PASS、clippy --all-targets 净（首版两测内冗余 `use crate::Sandbox` 清理）。

## 挂账更新

- s368 挂账「全 strict 车道迁移」→ **本刀销账**。
- 「直编译车道 reportError 透传与抛错后 load 派发增量」→ 随全车道迁移统一收口
  （STRICT_UNCAUGHT_REPORT_TAIL in-execute 承载，s335/s336 守卫同断言面迁移实证）。
- 基建面 T=3 维持 s102/s124 挂账；probe8/9/10 转正决策维持。
- MicrotasksExplicitGuard 面：宿主后续如需「execute 后宿主侧排空」语义，注意 Explicit
  车道内 checkpoint 由 catch 面显式触发，catch_js 缺省（`execute`）车道维持 Auto 旧行为
  零漂移。

## 日志

- evidence/logs/s369-solo-{le3,ckpt,evo1}.log（绿色 solo ×3）、
  evidence/logs/s369-b-script-element.log（域 B 面）、
  evidence/logs/s369-full-semantics.log（全语义面 sweep B 面）——入库。
- /tmp/s369-tri-{selhe,chkob,chkob2,selev}.log、/tmp/s369-ab-{build,chkob-1,chkob-2}.log
  （triage 与 stash A/B 工件）——tmp 不入库，数字在文。
