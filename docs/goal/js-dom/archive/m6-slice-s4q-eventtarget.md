# M6 S4q — QuickJS EventTarget（R63）

**日期**: 2026-08-16
**commit**: `21f6c4b2`
**里程碑**: M6 QuickJS 原生绑定移植（js-dom goal DC-7）第八切片
**证据**: [evidence/2026-08-16-r63-quickjs-s4q-eventtarget.json](../evidence/2026-08-16-r63-quickjs-s4q-eventtarget.json)

## 目标

S4q EventTarget：addEventListener/removeEventListener/dispatchEvent 原生——
QuickJS native 对象获得事件交互能力（交互式页面的基础）。

## 实现

- **监听器存储**（镜像 V8 gc.rs LISTENERS）：线程局部
  `(NodeId ffi, type) → Vec<(capture, Persistent<Value>)>`——单列表保**全局
  注册序**（spec 派发按注册序，capture/bubble 混合按 add 顺序）。
- **addEventListener**：非 callable 回调静默忽略（spec 不抛）；capture 经第三参
  truthiness（options 对象形态延后）。
- **removeEventListener**：移除最早匹配条目（callback identity 经 Persistent
  restore 到当前 ctx 后 JS 严格等——同 Persistent 来源恒等成立）。
- **dispatchEvent**：**target 站派发**（无 capture/bubble 链——祖先虚站基建镜像
  V8 R40 延后）；注册序触发；轻量 plain 事件对象（type/target/currentTarget
  三字段，Event 构造器族属 S5q 域）；**listener this = target** 经 `Args::this`
  + 低层 apply（rquickjs 无 call_with_this 高层封装——本轮 API 发现）；回调
  异常吞掉不中断（spec 报告后继续，console 报告延基建）；返 true。

## API 发现

`Function::call` 无 this 绑定能力；正确路径是 `Args::new` + `args.this(obj)` +
`args.push_arg(...)` + `args.apply(&func)`（低层 JS_Call 携 this_func）。

## PoC 断言

注册 A/B → dispatch 按注册序触发（`a:click:true|b`——序 + e.type + this ===
target 一次验证）；remove A → 再 dispatch 仅 B。

## 验证

engine quickjs **1419** / v8 **2153** 零回归；webview quickjs wiring 绿；
clippy quickjs 矩阵零警告；fmt 无 diff。rebase over CI-watchdog docs `7a3be5cc`
零冲突。

## M6 累计（R57→R63 七轮十一切片）

4 全局工厂 + 13 属性（6 setter）+ 11 方法（属性 4 + mutation 2 + 查询 2 +
EventTarget 3）。元素可达（查询）+ 可变（mutation/写入）+ 可交互（事件）。

## 剩余

S5q customElements 五件套 + lifecycle 四件套；S0q 续 weak/finalizer；S1q 复合
对象；S4q 完整化候选（capture/bubble 祖先链 + Event 构造器族 + DOMException
基建）。
