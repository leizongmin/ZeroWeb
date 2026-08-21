# R154 — nearest-activation 止步 + A/AREA click hash 默认动作 + _zwMEl boolean reflected

**日期**: 2026-08-22
**里程碑**: M4（WPT dom 上游基线扩展）
**commit**: `92f301bd2`
**驱动用例**: `dom/events/Event-dispatch-single-activation-behavior.html`（132 subtest）

## 根因与修复（三件）

### ① nearest-activation 止步（single-activation 主要削减源）

**根因**：`_zwFindClickActivation`/`_zwPreClickActivation` 的上行遍历只识别
INPUT[checkbox/radio] 一种激活元素——途中经过**有自身 activation behavior 的其他元素**
（A/AREA[href]、LABEL、DETAILS/SUMMARY、INPUT/BUTTON submit 族）时继续穿透，翻到更上层的
checkbox/radio。spec `concept-event-dispatch` legacy-pre-activation 的「nearest ancestor
(or self) with activation behavior」是**任意**激活元素的最近者。WPT single-activation 的
`A 在 INPUT[checkbox] 内 click` 期望只激活 A 的 hash 导航、父 INPUT 不翻——旧版翻父使
activated 收到错误对象（4F 类型错配）+ LABEL/checkbox 隧道连锁。

**修复**：遍历每站先经 `_zwHasOwnActivationBehavior(tag, sel, handle)`（五类激活元素枚举，
与 node.click() post-activation 段同源）判定——命中即止步返 null（INPUT 翻转定位不穿透；
A 的 hash 导航由 ② 的 default action 承接）。

### ② proxy 侧 click() 的 A/AREA[href^="#"] 默认动作

**根因**：proxy `click()`（part04 dispatchEvent 默认动作段）只有 checkbox/radio、submit、
popover 三类默认动作，缺 A/AREA 片段导航——WPT 的 `window.onhashchange(e.newURL)` 收不到
href 字符串（activation 空数组，18F）。

**修复**：默认动作段补 A/AREA 分支（镜像 node.click() 本地版）：`location.hash = href.slice(1)`
→ 既有 `_setLocationHash` 链路（hash 变更 + history entry + **异步** hashchange 派发 +
滚锚）。`onhashchange` 基建（R2932 window IDL handler + R3006 hash setter）已完备，只缺
click 入口。

### ③ `_zwMEl` 的 boolean reflected accessor

**根因**：clone/解析子树产物是 plain object（无 get trap）——`cb.checked` 读 undefined，
click() 翻转了属性但断言读不到（clone 形态的 checked 断言恒 false）。

**修复**：`_zwMDefineBooleanReflected`（checked/disabled/selected/hidden/required/open/
multiple/readonly/autofocus/novalidate 十属性）defineProperty getter 读属性存在性、setter
写/删属性——与 proxy 侧 part03:7197 boolean reflected 分支同源语义。

## A/B 验证

| 项 | 结果 |
|----|------|
| single-activation | **107P/25F**（vs R153 85P/47F：-22F；类型错配 4F→0、A/AREA 空激活 18F→0、LABEL 隧道连锁 -6） |
| 剩余 25F 归因 | LABEL(child) 12 + checkbox/radio@FORM 12 + radio@LABEL 1——**clone 子树形态**的 activation 链（沙箱 createElement 单测全过但 WPT 的 `template.content` clone 路径仍有差异：sel-based innerHTML 异步 mutation vs runner 逐脚本 apply），下轮定向 |
| 全量 dom 套件 | **6279P/275F/18T**（vs R153 6257P/297F：净 +22P/-22F，fail 集合 diff 零新增） |
| `make test` | 66 套件全绿 |
| fmt / clippy | 零警告 |
| 单测 | r154 两件（a-in-checkbox：父不翻 + hash 字符串激活；checkbox@form 含 clone 产物：翻 checked + inline oninput 链） |

## 技术要点沉淀

- **Rust 多行字符串注释坑第三次出现**：注释行尾误加 `\` 会吞掉后续代码（V8 completion
  value 变成无关函数体）——本轮复现脚本两次踩中，均已修。后续新增测试注释行一律**裸换行**。
- **沙箱单测 vs testharness 环境差异**：sel-based proxy 的 `innerHTML` 走异步 mutation
  （沙箱单测中 host 快照不更新 → querySelector 查不到），testharness runner 每脚本后 apply。
  复现 WPT 行为时用 `createElement` + 手工建树（同步可见），或接受此差异单独归因。

## 未收（记入 R155 候选）

- single-activation 剩余 25F（LABEL/checkbox@FORM 的 clone 子树 activation 链——需复现
  template.content clone 路径的 input 事件触发差异）
- Element-matches.html 整页 error（R154 计划 (b) 未动——本轮时间用在 ① 的连锁削减）
- Attr-prefix 2F / MO-document 3F / realm·adopt 族
