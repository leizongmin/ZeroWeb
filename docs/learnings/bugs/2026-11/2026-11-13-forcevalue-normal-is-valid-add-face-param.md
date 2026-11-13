---
date: 2026-11-13
modules: engine
---

# forceValue 的 'normal' 是合法 ADD 面参数，不能当移除面拦截

## 问题

bold 700/900 toggle-off 语料残余定谳时，曾假设"newValue='normal' 意味着
移除面、forceValue 不该包裹"，在 `_zwVaForceValue` 入口加了 bold/italic
非激活值早退门（复用 `_zwFmtActivated` 判定）。face 全量对跑产生
**P2F=84、F2P=0**：bold?2001-3000 的 h3 族 32 断言、italic?1-1000 的
address/cite/dfn/var 族 52 断言整腿红。

## 根因

vendored setSelectionValue 的 nodes pass 对每个 contained 节点无条件调
`forceValue(node, command, newValue)`，包裹面（tag face 或 css style）由
**command** 决定，newValue 只影响并值判据（sibCrit）和等值早退门。ADD 面
（如 address 内 italic ADD 产出 `<i>`）走的就是
`forceValue(tx,'italic','normal')` 这条调用——'normal' 在这里是 ADD 流程
的正常参数，不是移除信号。按激活值拦截等于把 ADD 面的包裹臂一起关掉。

## 解决

撤掉该门（该毒根最终定谳在 wrapper/handle 桥层，见
docs/goal/wptfyi-editing/evidence/2026-11-13-dc2-bold-toggleoff-universe-diag.md）。

## 避免方式

- 动 forceValue/setSelectionValue 前先确认调用点传的是什么：移除与添加
  的区别不在 newValue 字面值，在 execCommand toggle 计算出的目标值与
  当下 state 的组合。
- 判断某条 forceValue 调用是否该包裹，看 `command` 的包裹面分支，不看
  newValue 是否在激活值集合里。
- 想拦截"移除面"时，先在单腿探针页对照一个已知 ADD 用例
  （如 `foo<address>[bar]</address>baz` + italic，期望 `<i>`）再下手；
  gate trace + face 全量对跑（P2F/F2P 计数）是方向错误的快速反证。
