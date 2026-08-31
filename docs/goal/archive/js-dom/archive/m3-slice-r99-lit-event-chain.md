# M3 R99 — lit 事件链落地（_zwMEl 事件面）+ fonts.ready 惰性化

**日期**: 2026-08-17
**Commit**: `bfb64b2aa`
**Milestone**: M3（真实 SPA / Web Components 端到端验收）
**前置**: R97（首渲染）、R98（响应式更新链）

## 背景与目标

R98 后 lit 组件的渲染与响应式更新已闭环，事件交互（`@click`）是 Web Components
「真实运行」的最后一段。探针实证带 `@click` 的组件**首渲染直接中止**
（renderRoot 仅 marker、hasUpdated:false）。

## 根因与修复

### 1. `_zwMEl` 解析节点缺事件面（事件链主根因）

lit EventPart（class z）的 `_$AI` commit 调
`this.element.addEventListener(name, this, t)` / `removeEventListener`——
解析节点（template.content 子树的 `_zwMEl`）缺这两个方法，commit 抛
TypeError 使整次 render 中止。

**修复**（part03 `_zwMEl`）：本地事件面——
- listener 数组存储；对象 listener 走 `handleEvent` 协议（lit 传 part 自身）
- capture/once 选项（boolean | object 双形态）
- 派发序 = 注册序；once 派发后移除
- 返回值 = `!(cancelable && defaultPrevented)`（spec dispatchEvent）

### 2. fonts.ready fallback 竞态（CI flake 池 ⑤ 修复，附带）

part06 顶层**无条件** `setTimeout(0)`（R34xx fonts settle 兜底）在每次 shim
注入时经 setTimeout polyfill 注册瞬时 `_t_` 键，与 renderer reset 断言竞态
（`renderer_js_worker_document_reset_...` 自 8/16 起 3+ CI 轮次失败，CI 守护
记录归因完整、建议「注册条件化」）。

**修复**：惰性化——`fonts.ready` 改 thenable 包装（then/catch/finally 首调时
`_armFontsFallback()` 一次 + 委托底层 Promise）。无 font 消费的页面（绝大多数，
含 renderer reset 测试）零注册；消费页面 settle 语义不变。Renderer reset 测试
本地 20/20 通过，fonts 单测 27 全绿。

## 验收

- **lit e2e 组 F（新，`lit_event_chain_lands`）**：
  `btn:BUTTON|t0:0|dispatch:true` → post-drain `t1:1|count:1`
  （click 派发 → handler `_inc` → count+1 → 响应式重渲染按钮文本）
- **lit 全组 6/6 绿**
- **WPT dom A/B**：nodes 6673=base（0 新增 fail + 22 条既存 flake 消退）、
  collections 48、traversal 1595、events 236
- **单测**：engine v8 2205 / quickjs 1431 / integration 781 / renderer 134
  （含修复前 make test 偶发失败的 reset 测试）；`make test` exit 0
- **质量**：fmt / 双矩阵 clippy / pre-commit-guard PASS

## 教训

1. **框架的 listener 形态是对象**（part 自身实现 handleEvent）——事件面实现
   不能只支持 function listener。
2. **shim 顶层无条件宏任务注册是 reset/断言类测试的系统性竞态源**——惰性化
   （消费时 arm）是通用修复模式，零语义损失。
3. e2e 探针统一用 `new Event()`（modern 构造器）——legacy `createEvent` 工厂
   在部分 execute 上下文不可用。

## M3 进度

WC 端到端全链闭环：customElements 五件套 + lifecycle + Shadow DOM + 真实 lit
（首渲染 R97 + 响应式 R98 + **事件交互 R99**）。DC-2 第二项（Web Components
代表性页面真实运行）**实质达成**。

剩余：SPA 框架端到端（React/Vue 之一）——DC-2 第一项；可选 lit 综合验收页
资产化收口。
