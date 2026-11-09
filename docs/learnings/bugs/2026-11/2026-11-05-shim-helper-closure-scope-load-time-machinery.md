---
date: 2026-11-05
modules: engine
---

# shim helper 定义在运行核闭包内——载入期机具消费时缺位 throw 撕裂 sweep

## 问题

s288 给 img 事件派发加 Event 形态 helper（`globalThis.__zwImgMakeEvent5277`），
初版把赋值放在 part03 `__zwImgRunUpdate5277`（运行核）函数体开头，part04 的
img sweep/drain 派发点经 `globalThis.*` 消费。family A/B 瞬间 **REGRESSED=15**：
eager/invisible/delay-load/in-scroller ×8/multicol/relevant-mutations-lazy ×2/
relevant-mutations/svg-density 全 P→F（失败面 = 窗序/装载断言：img load 在
window load 前未到、「The images should block the window load event」false、
in-scroller 等 load 超时）。

## 根因

shim 的执行时序分两层：

- **载入期机具**（part04 sweep/drain/window-load 链）：页面加载即执行，
  不依赖任何 img update run 先行。parse 期 `src` 属性回填走 R4004 直写
  host、**不走 wrapper setter 钩子**（s287 勘根实证），lazy 图视口门外
  不启动 run——这两类页面上**没有任何 `__zwImgRunUpdate5277` 执行过**。
- **运行核**（`__zwImgRunUpdate5277` fetch 臂）：只有触发 update run 才执行。

helper 赋值在运行核内 → 未跑过 run 的页面上 `globalThis.__zwImgMakeEvent5277`
为 undefined → part04 sweep 派发点 `undefined('load')` throw。而 sweep 的
循环体整体包在**单个 `try { ... } catch (_eSweep) {}`** 里，一抛全弃——
sweep 正是 parse 期 img 的主要 settle 通道，一断则本文件所有依赖 sweep
settle 的图 load 全丢，窗序断言面随之崩。solo 目标测试（historical）全绿
掩盖了它：该测试自己触发过 run，helper 已就位。

## 解决

helper 赋值提升到 part03 **载入期作用域**（与其他 `globalThis.__zwImg*`
入口函数同级、`__zwImgRevalidate5277` 邻位），运行核内只消费。二分定位
路径：先摘 proto 挂载仍红（排除 Event.prototype 嫌疑）→ 提升 helper 后
eager solo Pass → family r2 REGRESSED=0。

## 避免方式

1. shim 内新增 `globalThis.*` helper 时先盘点**全部消费点的执行时态**：
   只要有一个消费点在载入期机具（sweep/drain/MO 回调/scan），赋值就必须
   在载入期作用域（partNN 顶层或 IIFE 直执行段），不可放进任何「可能
   永不执行」的函数体。
2. part04 sweep/drain 的单 try 包裹体是**爆炸半径放大器**：任何新引入
   的 throw 点（含 undefined 调用、断言 helper）都会把局部缺陷放大成
   整域 settle 丢失。改 sweep/drain 路径时，回归检查不能只跑目标面
   solo，必须跑 family 级 A/B。
3. 目标测试自身触发过相关机具 ≠ 机具在全部页面时态下可用——solo 绿
   之后 family A/B 仍是必经门禁。
