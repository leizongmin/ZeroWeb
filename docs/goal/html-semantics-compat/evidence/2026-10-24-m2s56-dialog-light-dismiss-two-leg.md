# M2 切片 56（s56）——B 族 pointer-capture 2F 收口：R5263 dialog light dismiss 两腿模型

日期：2026-10-24。s49 R5252 定性「指针管线深水」的 dialog-light-dismiss-pointer-capture.html（touch/mouse 双腿）**2F → 2P 全收口**，全域 GAIN 2 / REG 0。

## 方案与取舍

s49 当轮在三洞试错后回退、定性指针管线深水。本轮复核推翻前提：指针管线的 capture 重定向面**已全通**——part04 主帧 setPointerCapture proxy trap 写 `_zwPtrState.pending`（active NotFoundError 门 + pointerevents2 §9.3 pending override）、part06 `_zwProcessPendingCapture` 于下一指针事件派发前换防、up 序列尾簇 12 在换防后取 `capturedSel`。真正缺口唯一：light dismiss 决策仍是 R4300 单腿（pointerdown 即关），capture 无从影响判定。

spec 已演进为两腿（light dismiss open dialogs：pointerdown 记 nearest clicked dialog 祖先，pointerup sameTarget 比对后才关）。实现以**两态分类**等价替换 spec 身份模型：down 腿 `_zwDialogLdPick` 反文档序首个 open+any dialog 定类（'in' 包含链内 / 'out' 链外，backdrop 属主与盒外均 'out'）存槽；up 腿 `__zw_dialog_light_dismiss_up` 重分类后 sameTarget 同态比对，相等且 'out' 才关（选择器保持 R5234 反文档序首个 open+any 口径）。全腿组合推演等价：in/in、out/in、in/out 均不关，out/out 关——capture 面（down 于 backdrop 'out' + setPointerCapture 把 up 重定向 dialog 内元素 'in'）由此不关门。槽卫生：down 腿终态必写槽（无 dialog / target 解析不到的早退也清，防跨交互残留），up 腿读取即清。

落点：part01 改造 + part06 up 序列 capturedSel 计算后插 up 腿调用（传 `capturedSel || upSel`，序在 pointerup 派发前与 down 腿对称）；popover pass 维持 down 腿单发零变化（R5234 绿面）。portal/chorded 早退分支不经过 up 腿——每次 up 前必有 down 重写槽，无残留可读。

## 验证

- 目标文件 dialog-light-dismiss-pointer-capture.html **2F → 2P**（touch/mouse 双 capture 变体；pointerId 全管线默认 1 与 `st.capture['1']` 对齐实证成立）
- 全域 interactive-elements 98 files **436P/47F/0T → 438P/45F/0T**（GAIN 2 / REG 0 / Timeout 0）
- part60 R4 契约两腿化（in/in 保持 / out/out 关 / 默认模态槽 null 不关）+ 新增 out→in capture 门腿（r4d）；R6 改 down+up 对，toggle source=null 断言保持
- zero-engine lib **3448P/0F**；fmt/clippy 清（.rs 有变更轮适用）
- R4 首轮 1F 自坑自愈：r4d up 选择器 fallback `'button'` 命中文档首个 button（l1 的，已关）致 contains false 误关——契约侧选择器 bug 非 shim 缺陷，改 `'#inner3'` 唯一 id 后过
- 插桩注意：runner 无过滤跑整个 html-semantics 语料（数千文件）非本轮全域口径，全域 = interactive-elements 过滤参数（教训复认）

## 未解决项

- B 族 pointer-capture 清空。挂账序：C 族渲染流 20F（跨流，归渲染流）、declarative shadow 联合切片待用户点名、parser form pointer 4F 用户决策缓行
- chorded/portal 早退分支的 up 腿 spec 覆盖（真浏览器 chorded pointerup 也走 light dismiss）为极边角，未实现，现有语料无断言面
