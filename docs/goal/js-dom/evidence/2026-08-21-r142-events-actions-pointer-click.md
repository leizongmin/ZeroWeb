# R142 — events no-focus-events 2F→0F（testdriver Actions 指针链 + Generic 激活态 + 点击 focus 步骤）

**日期**: 2026-08-21
**里程碑**: M4（WPT dom 上游基线扩展）
**驱动用例**: `dom/events/no-focus-events-at-clicking-editable-content-in-link.html`（2 subtest）

## 根因（三层）

用例经 `new test_driver.Actions().pointerMove(x,y,{origin}).pointerDown().pointerUp().send()`
合成指针点击 contenteditable 元素，断言 focus 事件序列恰好为
`[focus(target), focusin(target)]`（无冗余 blur/focusout）。三层缺口：

1. **testdriver stub 无 `Actions` 类**：`test_driver.Actions is not a constructor` ——
   且 `Actions` 在 `unsupported_testdriver_dependencies` 白名单外（整个文件被
   Unsupported 中性跳过，fail 集不显）。
2. **非分类目标激活整体哑火**：`ActionTargetState` 无「普通元素」态——
   span[contenteditable] 非 checkbox/radio/anchor/summary/option，Activate 分类
   落到 `NotApplicable` noop，click 事件根本不派发。
3. **点击 focus 步骤缺失**：真实浏览器指针激活序列对可聚焦目标先 focus 再 click；
   宿主 Activate 管线无 focus 语义（shim 的 element.focus() 已有完整
   focusout(旧)→focus(新)→focusin(新) 派发，R3247，但无人调用）。

## 修复（四处）

- **testharness.rs stub**：`test_driver.Actions` 链式构造器——pointerMove 记
  origin 元素，pointerDown/pointerUp no-op 记形，`send()` 对 origin 入队
  `click` 命令（宿主走既有 Activate 管线）；key 系列显式 reject。
- **依赖白名单**：`Actions` 加入放行表（连同本文件进 fail 集可见）。
- **`ActionTargetState::Generic`**（page-runtime）：普通元素激活 = 纯 click 事件
  （cancelable_event=click，mutation/effect 全空）；webview Activate 分类兜底
  从 `NotApplicable` noop 改为 `Generic`（真实浏览器对任意元素点击都派发 click）。
- **runner click 前 focus 步骤**：`apply_testdriver_command` 的 click 分支先执行
  `element.focus()`（shim R3247 派发 focus/focusin——WPT 期望的序列），再派发
  Activate。focus 失败不阻断 click（不可聚焦目标真实浏览器也派发 click）。
- **stub `selectorFor` 唯一化扩展**：同 tag 多实例时属性筛选器
  （`span[contenteditable]`）→ nth-of-type 兜底（origin 无 id 也能定位）。

## A/B 结果（polyfill / native 双路径）

| 套件 | 结果 |
|---|---|
| no-focus-events | **2P/0F 双路径 100%** |
| Event-dispatch-on-disabled-elements | 与基线一致（3P/1F 既有；「disabled 不派发」1F 是另一簇） |
| dom/events 全量 | 413P/36F——fail 集 vs R141：no-focus-events 消失（修复）；**+3 新可见**（click-on-absolute-pseudo / focus-event-document-move / handler-count——此前被 Actions 白名单挡为 Unsupported 中性，本轮放行后暴露为真实 Fail，clean 基线同形，非回归） |

## 集成测试更新（语义对齐）

`details_summary_activation_and_cancellation_conform_across_hosts` 的
`ignored.noop_reason` 断言从 `Some(NotApplicable)` 改为 `None`——非首个
summary 激活：click 事件照常派发（Generic），仅无 toggle 默认动作（观察串
`click:false,toggle:true,click:true` 三事件不变——第 3 事件来自第二次 #summary
点击的 preventDefault 分支，非 #second）。旧断言编码「连 click 都不派发」的
旧语义，与新 Generic 语义冲突，按真实浏览器行为更新。

## 门禁

`make test` 66 套件全绿（双矩阵）；fmt 零 diff；clippy v8+quickjs 双矩阵零警告。
单测：`generic_activation_dispatches_click_without_default`（page-runtime，plan
层断言：click cancelable + 全空 mutation/effect + preventDefault 无 rollback）。
