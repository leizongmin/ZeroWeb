# M2 s129 · input/change 事件属性 + UA trusted 面 + checkbox indeterminate + runner select 键接线 + 宿主 change-on-blur 接线（input-change-event-properties 11F→0F 定向 19F→0，真翻红 0）

日期：2026-10-28 · 切片：html-semantics M2 · 前序：s128（change-on-blur 共享面收割）

## 入场定刀位

s128 后全量基线（/tmp/s128-full.log，原始行 6758P/1051F/103T）
翻拣域/文件分布：`forms/input-change-event-properties.html` 整页
11F 单根清晰——input 事件 `composed` 断言全族失败（UI Events 事件
定义表 composed:true，shim 全部 InputEvent 构造缺）。同域 rider 五
面搭刀：①UA 派发语义事件（reset/submit/script error/激活 input/
change）isTrusted false（R312 `__zwTrusted` 通道修复，合成 `click()`
保持 untrusted——checkbox.html:37/radio.html:119 负案在账且当前
Pass，不得回翻）；②checkbox pre-click / canceled activation 缺
indeterminate 清除（checkbox.html 4F）；③runner scroll_key 分支无
SELECT 键导航接线（`script_select_key_action` 生成器 R3254-K5 已备
但零调用点——down-arrow 1F）；④radio.html t1（激活 input/change
trusted 同根①）。

## 勘根定谳（含排误）

1. **input/beforeinput composed:true 缺失**：part01 全文件 16 处
   InputEvent 构造（insert/delete/CE 系）+ part03 `_zwFireInputChange`
   + part06 `__zw_dispatch_event` beforeinput/input 分支与 plain input
   分支 + 5 处 radio/select 导航 `new Event('input')` + execCommand
   4 处。回归安全性核查：无 corpus 断言 input composed:false；change
   composed:false 已正确不动。
2. **UA trusted 面**：`_makeEvent` 已支持 `options.__zwTrusted`
   （R312）；`Event` 构造器透传 full options 给 `_makeEvent`；
   `_defineEventSubclass`/SubmitEvent 同透传。**dispatch 期翻面**：
   page-facing `dispatchEvent` 对 `isTrusted && !_zwUaDispatch` 强制
   翻 false（part05 R312 段）——`el.dispatchEvent(new Event('error',
   {__zwTrusted:true}))` 单印**不够**，须 `__zwMakeTrustedEvent`
   （`__zwTrusted` + `_zwUaDispatch` 双印，part03:14866 既有出口）；
   内部通道 `_dispatchWithBubble` 不翻面，`_makeEvent('change',
   {__zwTrusted:true})` 直接过。勘根中段曾误改 R96/S110 为单印形，
   探针/failure.html 复跑定谳后改双印形（过程教训入账）。
3. **checkbox indeterminate**：pre-click activation steps 清
   indeterminate + canceled activation steps 同步清（HTML spec
   checkbox-state；checkbox.html t3/t4/t6 断言面）。
4. **宿主 change-on-blur 断链（本切片最大勘根转折）**：定向跑
   input-change-event-properties 后 typing 8 子测从 s128 的
   `input composed expected true but got false` 变为 `Cannot read
   properties of undefined (reading 'bubbles')`——初判为施工回归，
   建探针页（gitignored，验后删）逐层取证：**input 派发正常
   （composed:true 已生效）**，undefined 的是 **changeEvent**——
   s128 该文件 typing 在第一断言（input composed）即失败，
   `assert_event(changeEvent,…)` 从未执行，change 缺失是**前存断链
   被本修复揭开的下一层**，非回归。再取证：el.blur() 后 change 正常
   派发（s128 part04 blur 臂与 `_userEdited`/基线键控全通）→
   test_driver.click(button) 的焦点迁移走**宿主管道**
   `_zwFocusSel → __zw_host_blur/__zw_host_focus`（part06 slice22
   钩子）——只派 blur/focusout/focus/focusin，**未接 s128 两臂**：
   `__zw_host_focus` 不记基线 → `__zw_text_change_due` 因基线
   undefined 恒 false；`__zw_host_blur` 无 change 派生。send_keys
   首键自聚焦即宿主通道，故 typing 面 change 永缺。s128 账本
   「change-on-blur 未接路线：宿主 `__zw_host_focus`」注记即此，
   本切片收口。
5. **探针页首跑 `test_driver is not defined`**：probe 漏引
   /resources/testdriver.js 与 testdriver-vendor.js（corpus 头部
   两行），补齐后取证正常——runner testdriver 适配依赖页面 script
   标签声明面。

## 施工（engine shim part01/03/04/06.js + runner testharness.rs）

1. part01：`__zw_text_input` 上方 s129 注释块（UI Events spec 链 +
   全文件 16 处同形说明）；16 处 InputEvent init（8 beforeinput
   cancelable:true + 8 input cancelable:false）加 `composed: true`。
2. part03：①`_zwFireInputChange` input 三形态 init 加
   `composed:true, __zwTrusted:true`、change 加 `__zwTrusted:true`
   （UA 语义事件 trusted，click 本身仍 untrusted）；②script 队列
   排水 error（S110 臂）→ `__zwMakeTrustedEvent('error')`；③R96
   fetch reject 臂 error → `__zwMakeTrustedEvent('error')`；④checkbox
   pre-click（`_r108Pre` 后）清 indeterminate（`delete
   _indeterminate[key]`）；⑤rollback checkbox 臂 checked 恢复后同清
   （canceled activation）。
3. part04：①form.reset() 派发 `new Event('reset',{…,__zwTrusted:
   true})`；②submit 派发 SubmitEvent/Event fallback init 均加
   `__zwTrusted:true`。
4. part06：①`__zw_dispatch_event` beforeinput/input 分支与 plain
   input 分支加 `composed:true`；②radio/select 导航 + typeahead 5 处
   `new Event('input',{…})` 加 `composed:true`；③execCommand 4 处
   InputEvent 加 `composed:true`；④**`__zw_host_focus` 记 text
   control change 基线**（镜像 part04 focus 臂，`__zw_text_change_
   baseline(k, sel, null)`）；⑤**`__zw_host_blur` change-on-blur
   派生**（blur→focusout 派发后 `__zw_text_change_due` 门 →
   `_dispatchWithBubble` change（bubbles，`__zwTrusted:true`），序
   blur → focusout → change 互操作序；与 part04 blur()/focus() 臂
   经 `_activeElKey` 幂等守卫互斥不双派）。
5. runner testharness.rs scroll_key 分支：keydown 未取消时先走
   SELECT/radio 焦点键消费门——`script_select_key_action(&selector,
   key_name)` 返 true（键被选中项导航消费）即跳过滚动默认动作；
   非 SELECT 返 false 照旧滚动（input-change-event-properties
   down-arrow 断言面；R3254-K5 生成器首次接线）。

## 验证

- shim 拼接（js_dom_bridge.rs 装配序 13 part）node --check 过；
  `cargo check -p zero-wpt-runner` 过；fmt 净（fmt 报 testharness.rs
  一处链式调用换行，`cargo fmt -p zero-wpt-runner` 修齐复检净）。
- 定向：`FILTER=input-change-event-properties` **11/11 全 Pass**
  （click ×2 + typing ×8 + select down-arrow）；`FILTER=the-input-
  element/checkbox.html` **6/6 全 Pass**（s128 4F→0）；radio.html
  t1 Pass（残余 3F = 组属主语义异根，s128 同额非回归）；
  reset-event.html Pass；button-events.html submit 子测 Pass
  （「type=button shouldn't trigger」Fail 与文件级 Timeout s128
  在账同额，异根不动）；fetch-src/failure.html Pass。
- 全量 A/B × /tmp/s128-full.log：原始行口径 **6758P/1051F/103T →
  6780P/1032F/103T**；键级（去重归一 tab）join **恰 19 Fail→Pass**
  （input-change-event-properties 11 + checkbox 4 + radio 1 +
  reset-event 1 + button-events submit 1 + failure 1，全 `1F→1P`），
  **真翻红 0、new non-Pass key 0**。唯一键数 +3 为 zw-probe-s129.html
  探针页三个 Pass 键（参跑时在语料、验后已删）——勘误（2026-10-28
  复算）：初记「149.html tab 归一伪影」有误，149.html 12 行结果行
  两轮逐字节一致零 diff。本轮无 guard 内存杀。
- engine lib **3570/0**；wpt-runner **256/0**（76 ignored）；
  clippy（zero-wpt-runner --all-targets -D warnings）零告警。
- 探针页 zw-probe-s129.html（gitignored wpt-data）四轮取证后**已删**
  ；语料零改动。常驻语料 input-change-event-properties 11 断言 +
  checkbox.html 6 断言 + failure.html 等即回归面（与修复同提交）。
  渲染面零依赖——reftest/product-smoke 未跑。

## 挂账（本切片更新）

- **input-change-event-properties / checkbox.html t1·t3·t4·t6 /
  radio.html t1 / reset-event / button-events submit /
  fetch-src/failure 收口移出候选**。
- **radio 组属主语义 3F 新入账**（radio.html「orphan tree /
  different form owners / different name」——组属主（tree/form/
  name 分组）判定缺失，异根让位）。
- **s128 挂账两项收口**：change-on-blur 未接路线之「宿主
  `__zw_host_focus`（Tab 键驱动失焦）」已接（`__zw_host_focus`/
  `__zw_host_blur` 双钩子）；「checkbox 即时 change（checkbox.html
  'fires click → input → change'）」已收口（t1 Pass）。
- 承袭不变：button-events「type=button shouldn't trigger submit or
  reset events」（异根）与该文件级 Timeout、empty.html/
  empty-with-base.html Timeout（error 从未派发，异根）、
  form-submission-0 186F、execution-timing 构成面、lazy-loading 26F、
  meta color-scheme 21F、写流执行身份盖章断链 FIXME、part02
  setAttribute 绕 notify 臂、041 nested write、radio 组排他其余置真
  现象、deep clone 非 HTML ns 面、css-module corpus 门（待决）、
  www2 可达性、module-import-referrer TLA、dynamic import() with
  透传、classic 外链 SRI FIXME、143 write、defer 多写完成序、接口
  对象基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、
  live 树 document 级逗号列表 qSA 根缺口、渲染流面让位累积
  （wrapping hard-wrap ×2、placeholder-lineheight、naturalWidth
  SVG viewBox 22F、current-pixel-density 11F）；**待决（须用户
  拍板）**：css/text 数据模块语料资源缺失、green-2x2.png 语料资源
  分发、parser form element pointer 4F 缓行（2026-10-23）。

## 过程教训

- **「undefined」报错先辨事件**：`Cannot read properties of
  undefined (reading 'bubbles')` 未指明是 inputEvent 还是
  changeEvent——多断言共享一 helper 的语料里，前序断言修复会揭开
  后序断言的前存失败，形态酷似回归。探针页逐事件 log 是最快定谳
  路（本轮 4 轮探针：缺 testdriver 引用 → 事件序取证 → el.blur()
  对照定谳宿主断链）。
- **R312 trusted 是双印通道**：`__zwTrusted`（构造面）+
  `_zwUaDispatch`（dispatch 免翻面）缺一不可；`_dispatchWithBubble`
  内部通道不翻面可单印。新 trusted 派发一律走
  `__zwMakeTrustedEvent`（dispatchEvent 面）或 `_makeEvent
  (__zwTrusted)`（内部通道）。
