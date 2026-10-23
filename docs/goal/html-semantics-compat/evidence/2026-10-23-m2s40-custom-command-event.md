# M2 切片 40 — 挂账：custom command `--` 前缀 CommandEvent 派发收口（2026-10-23）

## 范围

s24 挂账：invoker command 激活对 `--` 前缀 custom command 直落
no-op（`_zwInvokerCommandActivate` custom 不映射 auto、不派事件）。

## spec 钉定（html.spec.whatwg.org form-elements.html，2026-10 现行文本）

- button activation behavior：validity 判定 Custom 态对**任意元素**
  true → fire `command` 事件（CommandEvent，command 属性=值本身、
  source=invoker、cancelable=true）→ continue false return → target
  断连 return → **Custom 态 return**（无任何内建行为；dialog 目标
  不得 showModal/details 不翻转）。
- custom command keyword = 以 `--` 起头的字符串；command getter 对
  Custom 态返**值本身**（大小写保留——`--cAsE-cArRiEs` 语料断言面）。
- 派发形状（button-event-dispatch 上游语料钉死）：bubbles=false、
  composed=true、isTrusted=true、target=invokee、source=invoker、
  无 relatedTarget 字段（读 undefined）。
- 现行 spec 已无 auto 态（missing/invalid → Unknown → validity false
  不派事件）；本实现维持 R4400 旧 spec auto 映射（corpus 在账面钉住，
  本切片精准修改纪律不动，记为已知 spec 漂移）。

## 落地（R5243 两刀）

1. **part05 CommandEvent 接口**：`_defineEventSubclass('CommandEvent',
   'Event', [command='', source=null])`——ToggleEvent（R4009）同厂
   （`_defineEventSubclass`，原型身份 + init dict 缺省 + 构造器
   length=1 语义免费继承）；无 relatedTarget 字段。
2. **part01 `_zwInvokerCommandActivate` custom 分支**：`isCustomCmd`
   时构造 CommandEvent（bubbles=false / cancelable=true / composed=
   true / command=属性原值大小写保留 / source=invoker proxy）+
   trusted 印记（isTrusted=true + `_zwUaDispatch`，R312 通道——
   R4009 toggle trusted 同款）经 `_dispatchWithBubble` 在 target
   派发后 return。分派在 disabled/preventDefault 等调用方门内
   （part04 click() notPrevented 块、part03 R112/R4400 UA 段），
   门语义零新写。`cmdCustom`（原值）与 `cmd`（小写）拆分——内建
   关键字匹配行为不变。
3. CommandEvent 未装异常形态 → 维持旧 no-op（lenient）。

## 集群验证

- interactive-elements 全域 98 files 482 subtests：**419P/61F/2T，
  与 s39 账本逐位打平（REG 0）**。
- dialog 集群 82 files 409 keys 对 s35 账本逐位 diff：**REG 0 /
  GAIN 5**（GAIN 全为 s39 既有收益——double-nested 2 + requestclose-3
  1 + focus-after-close 2，无新漂移）。
- forms/the-button-element 18 files 54 subtests：**41P/11F/2T，对
  s24 账本逐位一致**（11F/2T 全为 form submit/reset 既有缺口；两
  corpus 文件只用 commandfor 缺省 command，custom 路径零消费）。
- 契约 `test_invoker_custom_command_event_r5243`（part73 新文件，
  R1–R6）：基面形状 / 大小写保留 + `--`/`---` 边界 / 非 `--` 前缀
  不派 / click preventDefault 抑制 / dialog 目标只派事件不 showModal /
  command 事件 cancelable=true。
- 数据 [2026-10-23-m2s40-custom-command-event.json](2026-10-23-m2s40-custom-command-event.json)
  （meta + interactive-elements 全域 + button 集群两次 dump）。

## 已知边界 / 挂账

- 现行 spec「invalid → Unknown 不派事件」未跟进（本实现 invalid →
  auto 映射，R4400 面 corpus 钉住）；跟进需同步 dialog/details 内建
  关键字面（`show`/`hide`/`toggle` 已从现行 spec 移除），独立切片。
- popover 关键字（show-popover/hide-popover/toggle-popover）与
  hide-all-popovers 前步维持 defer（headless 无 popover 栈面，
  s24 既有挂账）。
- `oncommand` IDL handler 未建（ontoggle 同族既有挂账）；
  button `command`/`commandFor` IDL 反射未建（本切片消费面只需
  内容属性读）。
- input[type=button]/form-associated submit 按钮的 invoker 门语义
  （现行 spec form owner 早退）未实现——corpus 零消费，R4400 面
  既有。

## 门禁

zero-engine lib **3432P/0F**（3431 +1 新契约）；clippy `-D warnings`
（test-guard 包裹全 workspace）、fmt clean；零新增依赖。runner
release 二进制重建（make 依赖链）。R5243 撞号检查通过。
