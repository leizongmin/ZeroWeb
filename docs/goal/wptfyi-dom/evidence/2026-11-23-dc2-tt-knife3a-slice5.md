# DC-2 切片 5 刀③a 证据：Trusted Types sink enforcement 基座（2026-11-23）

## 交付面（part14-trusted-sinks.js 新件 + part04 两臂挂接 + part13 名单导出）

1. **part14-trusted-sinks.js**（新链尾件，IIFE + globalThis 契约，~230 行）：
   - enforcement 状态惰性扫描（页内 meta CSP——`require-trusted-types-for`
     且含 `'script'`；Report-Only 不 enforce；装配早于 DOM 时无法读到 meta，
     故首次 sink 调用时扫描并缓存——sink 调用必发生在 script 执行中）。
   - `_zwTtSinkValue(type, value, sinkName)`：sink 管道核心——TT 实例
     （part12 is* 内部品牌）→ 数据串直接用；其余 String 化 → default policy
     存在走其 createX（回调抛错/返 undefined → TypeError、返 null → 空串）；
     无 default policy → TypeError。
   - `_zwTtAttrSinkType(elem, attrNS, name)`：属性 sink 判定表（语料
     support/attributes.js trustedTypeDataForAttribute 全表反推）：
     on\<known-handler\> @ HTML/SVG/MathML（attrNS=null）→ TrustedScript；
     iframe srcdoc → TrustedHTML；HTML script src → TrustedScriptURL；
     SVG script href（attrNS ∈ {null, xlink}）→ TrustedScriptURL；
     其余 → null。事件名单复用 part13 `_zwSanEventHandlers`（新增导出，
     前缀匹配被语料负例 ondoesnotexist 证伪）。
   - script sink 包装：eval / Function（prototype/name 静态面保留）/
     setTimeout / setInterval——首参字符串与 TT 实例走 TrustedScript 管道，
     函数参直透传；包装恒安装、enforcement 惰性判定（非 enforcement 页
     零行为变化——回归网约束）。
2. **part04 挂接两臂**：setAttribute（R116 名字归一后、写入前）、
   setAttributeNS（NS 校验后、写入前）——`__zwTtSinkValue` 缺席或判定
   null 时零行为变化（短路条件保护）。
3. **js_dom_bridge.rs**：include 链 part13 后追加 part14（修正一次插入
   错误：初版误将 part13 include 重复入链，链序核读后归位
   part12→part13→part14）。

## 探针轮（zz-knife3-probe.html，临时件不入库）

- 刀前基线 10/10（probe1-10）：createElementNS 直建元素 ns 读面真实可得
  （SVG/foo ns + tagName——**handle 域 foreign ns 缺口只影响写链解析产物，
  不影响本刀**）、meta CSP 可读、工厂/defaultPolicy/is* 就绪、基线零
  enforcement（setAttribute onclick/srcdoc/xlink:href 全不抛）。
- 刀后 18/18 全绿（删 3 个刀前基线案后）：probe11-15 enforced TypeError
  全臂（onclick/srcdoc/script src/xlink:href/href-null-ns）、probe16-17
  负例（ondoesnotexist / foreign attrNS）、probe18 TT 实例透传、
  probe19-20 eval/setTimeout 字符串抛、probe21 default policy 管道
  （policy 名须为 `'default'`——首跑探针写 'probeDefault' 误案修正后过）。

## 语料对账与 script sink wrap 回退

- **首轮（wrap 恒装态）tt 全腿**：996P/949F vs 刀② 基线 943P/1012F
  （+53P/-63F）——但 zero-engine lib 全量**爆 14F vs 基线 3F**：11 个新
  失败全落 script sink wrap 语义链（① 内建 eval/Function 包装后**错误
  归因位移**：p7_import_helper 断言 filename=helper.js lineno=2 → wrap 后
  helper-main.js lineno=0——间接 eval 编译归因取调用点，wrap 函数体非
  页面脚本语境；② setup 字符串定义全局可见性丢失：r216
  "setup:paras is not defined" / r214 "range-setup:ReferenceError"；③
  worker string timer 编译单元：worker_importscripts ×5 +
  worker_toplevel_function_reachable_from_string_timer）。
- **决策：回退 wrap 安装面**（保留 `_wrapScriptSink`/`_installScriptSinks`
  函数定义与 FIXME 记档，3b native 伪装 + 归因保持重设计——QuickJS eval
  内建不可换名约束下可能需引擎宿主桥臂）。回退后：
  - lib 全量 **4249P/3F 精确回基线**（3F = min_max + numeric_special +
    offset_path_m2 全预存挂账）✓
  - 探针 16P/0F（属性 sink 面全保；删 eval/setTimeout 案 2 例）
  - set-attributes-require-* spot **287P/249F vs 基线 274P/262F
    （+13P/-13F）**——本刀实收减免面
  - san spot 323P 恒等（F 329→328 为页预算截断落位抖动，P 侧零回落）；
    tt 非 enforcement 页（set-attributes-no-require 179P 等）恒等。
  - tt 全腿整腿数字（回退态）留 3b 与 wrap 重设计一并收。


## 挂账（3b+）

- HTML sink 族挂钩：innerHTML/outerHTML/insertAdjacentHTML/document.write/
  iframe srcdoc 属性通道（block-string-assignment-to-* 族余量）。
- setAttributeNode/setNamedItem/Attr.value/Node.nodeValue 通道
  （attributeSetterData 后 6 API）。
- .headers sidecar 形态的 17 页（runner 无 CSP 头通道——runner 预处理
  微刀：headers CSP 注入 meta 或旗标）。
- report-only violation 事件面（SecurityPolicyViolationEvent）。
- Node-multiple-arguments(-tt-enforced) 180F：主体是 replaceWith/after/
  before 多参 API 缺失（非 enforcement 面——独立微刀；tt-enforced 版的
  TypeError 断言叠加其上，多参先修才能吃到）。
