# R295 Evidence — iframe realm 的 Text/Comment 构造器 ownerDocument（双套件 100%，dom 全量 -2F）

**日期**: 2026-08-27
**切片**: M4——R295(d) Text/Comment-constructor 跨 globals 2F（原计划 (a) MO 剩余的转序切片）
**改动面**: `part05.js`（`_zwMakeIframeWin` 的 Text/Comment per-realm 构造器包装）+ `part23.rs`（+1 单测）

## 一、修复内容

### iframe realm 构造器（WPT Text/Comment-constructor "across globals"）

spec WebIDL：node 构造器（Text/Comment）产物的 ownerDocument = **该 realm 的
document**——`new iframe.contentWindow.Text().ownerDocument ===
iframe.contentDocument`。旧 `_zwMakeIframeWin` 直接转发主构造器（`Text:
globalThis.Text`）使产物 ownerDocument 恒主 document（断言「expected
Document with 1 child got 2」——iframe doc [html] vs 主 doc [doctype,html]）。

修：win 对象内联 per-realm 包装类——实例经主构造器建（完整方法面），ownerDocument
defineProperty 覆写为本 iframe 的闭包 doc。**首版教训**：闭包捕获 `win` 变量在
对象字面量求值时**尚未赋值**（`var win295 = win` 得 undefined）——改捕获函数参数
`doc`（稳定引用）。prototype 直连主构造器 prototype（跨 realm instanceof 保持）。

## 二、验证

| 套件 | R294 | R295 | Δ |
|---|---|---|---|
| Text-constructor | 15P/1F | **16P/0F（100%）** | +1 |
| Comment-constructor | 15P/1F | **16P/0F（100%）** | +1 |
| node-creation-realm / Event-dispatch-bubbles / Range-insertNode / Document-createTextNode / MO-inner-outer | 全基线 | 同 | 持平（iframe-realm 消费方 sweep） |
| engine 单测 | 2432 | **2433** | +1（r295 单测：ownerDocument + 双 realm instanceof） |

## 三、dom 全量（单跑）

| 域 | R294 | R295 | Δ |
|---|---|---|---|
| dom 全量 | 54070P/89F | **54070P/87F** | **-2F** |
| dom/nodes | 12692P/45F | **12693P/43F** | +1/-2 |
| 其余四域 | 同 | 同 | 持平 |

set-diff：消失的恰为两构造器 subtest，**零新增失败**。

## 四、MO 剩余簇归因补记（本轮取样）

- **MO-document parser 3F**：parse 期 record（masterMO 在解析中观察后续解析
  节点的插入）——需 parse-time MO 捕获基建（host 解析流与 MO 汇流点的时序
  耦合），深结构域。"removal of parent during parsing" 的 nextSibling 反向
  （"#s012" vs null）同源（解析流的后续 append record 字段）。
- **MO extractContents/surroundContents/inner-outer "2 children" 3F**：
  wrapper identity 域（R291 归因同族）+ record 来源定位（R294 教训延续）。

## 五、R296 靶点

- **(a) querySelector-All tree-order 4F**（内容树 wrapper identity——R167 桥
  归一覆盖缺口；MO "2 children" 同族或连带）。
- **(b) Node-insertBefore 1F**（host ref 校验域——apply 报「节点不是子节点」）。
- **(c) MO-document parser 3F**（parse-time record 基建——深结构，评估立项）。
- **(d) querySelector-mixed-case 1F / escapes 2F / scope 2F**（selector 引擎域
  小簇重聚类）。
- **(e) variant 基建最小支持**（解锁 ranges/in-shadow 2F + events/
  handler-count；低优先备档）。
