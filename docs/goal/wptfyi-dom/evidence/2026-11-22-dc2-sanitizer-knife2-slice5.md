# DC-2 切片 5 刀②：Sanitizer 构造面 + setHTML/setHTMLUnsafe/parseHTML 挂接

日期：2026-11-23（跨 11-22/23 会话窗）
范围：`crates/engine/src/js_dom_shim/part13-sanitizer.js`（新建 ~1080 行）、part03/04/05 挂接、`js_dom_bridge.rs` 深条目 ns 字段
规范：https://wicg.github.io/sanitizer-api/

## 交付面

1. **Sanitizer 接口全量**（part13，自包含 IIFE）：canonicalization（string/对象
   元素属性简写、""→null、空 dict marker 补齐、键序）、TypeError 校验矩阵
   （三互斥对、去重、rWC 可替换件门、per-element 双方向规则、data-* 规则）、
   get() 排序输出（ns null 先→ns→name；PI 按 target）、十 modifiers 全带
   boolean 返回值（方向内移动 + 冗余剥除）、removeUnsafe 基线补面。
2. **默认配置 121 元素/58 属性全表**（#built-in-safe-default-configuration，
   语料 sanitizer-default-config.html verbatim python 提取嵌入）。
3. **过滤引擎**：mock 域 materialize-then-filter（物化→verdict→序列化→既有写链，
   双端同态保持）；safe 基线（8 元素 + on* 表）、内建 javascript: 剥离
  （navigating href/action/formaction + animating animate 整删）、rWC unwrap、
   注释/PI 面、per-element attributes。
4. **挂接**：Element.setHTML/setHTMLUnsafe（part04 trap，globalThis 契约
   late-bound）、ShadowRoot.setHTML/setHTMLUnsafe（part03 dispatcher，mock 域
   就地过滤）、script 类容器 context check（safe 产出空）。
5. **Document.parseHTML**（static 新增）+ **parseHTMLUnsafe options 包装**
   （part03 s180 单参实现之上，无 options 直通零行为变化）。

## 数字

| 探针轮 | 总账 | 相对增量 | 内容 |
|---|---|---|---|
| 轮 1 | 91P/582F | — | 初版（跨 part 直调断裂） |
| 轮 3 | 275P/398F | +184P | globalThis 契约修复（IIFE 化 + __zwSanBridge 导出 + __zwSanSetHtml late-bound） |
| 轮 9 | 323P/329F | +48P/-69F | deep_entry ns 透传 + parseHTML 片段臂 |
| 轮 13 | 323P/329F | 恒等 | verdict 返回值重构 + parseHTMLUnsafe options 包装（逐文件零漂移） |

对账口径：`make testharness-sanitizer-api FILTER=sanitizer`（27 文件 673 案，
runner 无逐文件汇总行，从 Pass/Fail 行聚合）。

逐文件亮点（轮 9）：sanitizer-processing-instructions 33→45P **全清 0F**；
sanitizer-javascript-url 6→20P；sanitizer-basic-filtering 31→44P；
sanitizer-names 6→13P；sanitizer-config 47P/24F（canonicalization 主体过）；
sanitizer-default-config 1P/0F 全过；sanitizer-removeUnsafe 2P/0F 全过；
sethtml-tree-construction 4→63P。

回归网：dom interface-objects spot **23P/0F 恒等**（EXIT=0）；js_dom_bridge
全量 **4249P/3F**（三红逐一归因非本刀：`test_offset_path_m2_canonical_lerp`
＝MOTION-2 lane 挂账在案〔css 控制面 stash 法先证〕；
`min_max_single_arg_collapses_to_argument_terms`＝本刀 stash 法 HEAD 复红
实证既有；`test_get_computed_style_numeric_special`＝我方 b643 keys fail=3
挂账在册）；tt 全腿 **943P/1012F/68T 与刀① 基线逐位恒等零漂移**
（179 文件；EXIT=1＝Fail 常态留痕，腿门＝计数==期望+judge 零漂移双证。
首跑 per-proc 4G 守卫阈值被 runner 杀——刀② setHTML trap 走 Sanitizer
物化路径后 tt 大页内存峰值过线，放宽 8G/16G 重跑收数；无 options 直通
零行为变化预期兑现）。

质量门禁：改动面 fmt 净（`rustfmt --check` 漂移全在 callbacks.rs:851
MUT_APPLIED_CURSOR 等非本刀 hunk 与 css WIP 停泊域）；clippy -p zero-engine
12 errors 全既有（calc-size/animation/computed_style 域，本刀三 hunk
3974/4087/6660 均无告警）。

## 作用域拓扑勘误（本轮最大勘验）

外层 shim IIFE（part01 开 `(function(){` → part06 尾 `})();`）把 part03/04/05
顶层函数锁在内部；链尾 part（part12/13）只走 globalThis 契约（part01c
`__zw_ce_justify_*` 先例、part12 文件头明文）。顶层 function 声明不进 global
（trap 的 typeof 检查 false 而 globalThis 注册成功＝轮 1 Sanitizer 47P 实证）。
修复三步：part05 尾 `__zwSanBridge` 导出块（setHtmlUnsafeImpl/fragmentAdded
Unsafe/serialize/buildBodyTree/realTag 五函数，IIFE 内 part03 函数提升可见）→
part13 IIFE 化 → part04/part03 调用点改 `globalThis.__zwSanSetHtml`。

`Document.parseHTML(Unsafe)` 是 **static** 成员（首轮挂 prototype 致 "not a
function"）；part03 s180 已有单参 parseHTMLUnsafe（detached doc + write 机器），
part13 包装两参形。

## deep_entry ns 透传（引擎桥共享面）

`deep_entry_json`/`deep_entry_json_unsafe` 元素条目加 `ns` 字段（非 HTML ns
才发，零字节膨胀）；`_zwMBuildDeepTreeNode`/`_zwMBuildDeepEntry` 透传
`_zwMEl` snap.ns 通道。物化 mock 树此前恒 XHTML 兜底：sanitizer 允许表按 ns
匹配（string shorthand=XHTML，SVG `<svg>` 不得命中 XHTML "svg" 条目）、
names 语料断言、html5lib 树比对（isEqualNode ns 六元组）全歪。camelCase 由
解析器 adjust SVG tag names 保证（`foreignObject` 等，探针 probeI 实证）。

## 挂账（后续刀）

1. **handle 域 foreign ns**（独立刀）：写链解析产物（innerHTML/setHTML 写链）
   的 handle 树 proxy `namespaceURI` 恒 XHTML——`_zwSelNs(sel, handle)` handle
   臂短路归 ''，host `__zw_get_ns` 仅 sel 寻址。修法需 DomMutation::CreateElement
   ns 字段 + 历史表 + 快照回落三级（`__zw_get_tag_handle` 同构）。解锁：
   javascript-url SVG/MathML 组残 22F、names 残 7F、basic-filtering foreign 案、
   animating 组；trustedtypes -1P 疑同域。
2. **parseHTML 文档级骨架消毒**（26F）：语料期望树含 html/head/body 管控
   （elements 不含 head → head 删）。write 后真实树过滤臂已试死（probe12 实证）：
   detached doc 产物 proxy attributes **读**正常（len/idx/item 三通）而
   `removeAttribute`/PI `remove()` **静默失败**（注释 `remove()` 同机器却工作
   ——写面按节点类型分机器）。需先修 detached doc 写机器或走引擎臂。
3. **svg-animate 页 22 案**（页预算截断剩 1F）：SMIL `beginEvent` 动画事件栈
   + `SVGAnimatedString.baseVal/animVal` 域外（引擎 SVG 动画能力）。
4. 语义残留：sanitizer-config 24F、modifiers 14F、get 2F、boolean-defaults 2F
   （构造/修改器细节微刀）；TT 面（sethtml-with-trustedtypes 129F）= 刀③
   sink enforcement 主域。

## 证据日志

- /tmp/dc2s5-knife2-probe1.log（轮 1）→ probe3/9/10/13（全程）
- /tmp/dc2s5-knife2-dom-spot.log（dom spot）
- /tmp/dc2s5-knife2-tt-spot2.log（tt 全腿终跑；首跑 tt-spot.log 4G 阈值被杀留痕）
- /tmp/dc2s5-knife2-jdb-full.log（js_dom_bridge 全量）
- 探针页 zz-knife2-probe.html（多轮 ground-truth dump，已删）
