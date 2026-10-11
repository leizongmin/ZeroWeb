# FileAPI M2-P5 — 元素代理面专轮（残余 5F 收口）（2026-10-11）

## 修复内容（part03.js 元素代理 trap + part05.js HTMLCollection 原型 + part02.js 一处交界）

**488 → 493（+5P）= 98.8%**（35 案 499 子测试，0 Timeout）。JSON 全量：
[2026-10-11-m2-p5-element-proxy.json](2026-10-11-m2-p5-element-proxy.json)。

1. **元素代理 String 化 = spec class 串**（+4P）：trap 的 `Symbol.toPrimitive` /
   `toString` 返回值由 tag 名（`String(bodyEl)` → 'body'）改为
   `'[object <IfaceName>]'`（与既有 `Symbol.toStringTag` 同表 `__zwHtmlTagIface` 查询）。
   spec 依据：真实 DOM 元素无 @@toPrimitive，`String(el)`/`el.toString()` 恒
   `'[object HTMLBodyElement]'`（Object.prototype.toString → toStringTag）。R34xx 原
   tag 名形态的消费面仅 assert 消息构建（值语义非契约）；QuickJS 非抛前提保持（函数
   仍返回）。收：File-constructor「HTMLBodyElement in fileBits」（BlobPart USVString
   转换 24 字节）、「Using object fileName」（'[object HTMLBodyElement]'）、
   dom.window「attributes」（+Attr.prototype `Symbol.toStringTag='Attr'`——
   '[object Attr]' 13 字节）。
2. **HTMLCollection value iterator 每步重读 length**（+1P）：spec value iterator 语义
   ——旧实现迭代创建期快照恒读 `els.length`，`Object.defineProperty(list, 'length',
   {get: throws})` 劫持不生效；改每步经 getter 读 `coll.length`（劫持 getter 异常
   原样传播）。收 dom.window「overwritten 'length'」。
3. **select.length + select[i] 索引项**（+1P，净 +1 计入 select 案）：HTMLSelectElement
   supported property indices——sel 经 host 查询（`_selectOptions` 同源）、handle
   （detached createElement select——dom.window 形态）经 JS 侧 `_handleChildren` 子树
   递归收集 OPTION。热路径护栏：数字开头快检先于 `_realTag`（trap 每属性 get 都过）。
4. **BlobPart sequence 平台对象分支**（part02）：`@@iterator` 缺失时以**元素代理品牌**
   （`__zwSelector`/`__zwHandle`）判定——品牌可判则按 length+索引 getter 物化；
   普通对象（`{0:'FAIL',length:1}`）仍 TypeError（上游 non-objects 簇钉死——
   M2-P4 结构判据回退的可行解：品牌判据）。

## 记账更正（M2-P4 evidence 一处错误归因）

M2-P4 evidence 原句「url-with-xhr『XHR with a fragment』随之翻绿」**不实**——该案
自 M1 起持续失败（p4/p4b/p4d 三份运行日志均在 Fail 列），UUID 修复实际只翻绿
url-format 1 案（+1P 算术不受影响）。本轮更正记录。

## 残余失败（6F，跨域挂账不变）

- FrozenArray\<MessagePort\> 1F：postMessage transfer 后 message event 的 `e.ports`
  为空（webmessaging ports 结构化转移面）。
- url-with-fetch/xhr 5F：Request 构造/revoke 生命周期（Request 对 blob URL 强引用）
  + fetch/xhr fragment（net-api Request 语义面）。

## 回归门禁（变更影响面全量复跑）

- **dom 通道（js-dom DC-3）**：54305P/167F → 54303P/169F。diff 逐条甄别：
  ①insert-adjacent ×4 Pass→Pass 仅断言消息文本变化（`__n3431` →
  '[object HTMLHeadingElement]'——format_value 串化，spec 更正确形态）；②
  Event-timestamp-high-resolution 族 Pass→Fail ×2 为**既有 flake 族**（R150 ceil
  量化 vs performance.now 竞速——FILTER 隔离复跑 3 次失败类型逐次漂移
  （Keyboard+Focus / 5F / 4F），与本次变更无因果；基线运行该族本就有 2F）。
- **html-syntax 通道**：61217P/73F → **逐字节恒等**。
- **html 通道（forms/focus/input-event）**：14P/0F 恒等。
- **selection 通道**：2965P/5F——Pass 数与 uievents-compat M3-tail31 记录
  （2026-10-06「2965P 无新增 Fail（stash 基线比对 anchor-removal/editing/event
  恒值）」）完全一致，失败集同批已知面。
- **keyboard 通道**：18P/7F+1T——失败全部 css-scroll-snap 几何面（渲染域已知挂账，
  keyboard-page-scrolling 归档 goal 域）。
- **uievents 通道**：test-guard 1200s 墙钟超时未出结果（语料规模，基线亦不可得，
  双侧一致不可判定——挂账下轮 TIME_LIMIT 放宽复跑）。
- zero-engine lib **2987P / 0F**（test-guard，RUST_MIN_STACK=32MiB）。
- 纯 .js shim 变更无 .rs——按门禁表仅验证受影响构建与行为（zero-engine + 上列
  通道），未触发 fmt/clippy/workspace 全量门禁。

## canvas 通道说明

R34xx 原始定位面（canvas WPT assert 消息）——本仓无 canvas 语料（2dcanvas 未拉取、
无 Makefile target），无法复跑；消息文本变化不影响断言通过性（Pass/Fail 与消息
内容无关），风险记档。
