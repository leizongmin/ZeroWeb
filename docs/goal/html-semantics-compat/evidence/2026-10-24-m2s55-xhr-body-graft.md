# M2 切片 55（s55）——xhr-response 底座缺口收口：R5262 XHR 文档 body graft wrapper-world 实体节点

日期：2026-10-24。s54 分诊轮挂账的 parse-world 三件套（接口原型 / innerHTML 写解析 / open IDL）以**窄域 graft** 收口，name-attribute.html **18P/0F 全绿**。

## 方案与取舍

s54 分诊给出两条路线：parse-world 基面三件套升级（波及 DOMParser/XHR/responseXML 全语料）vs 底座挂账。本轮复核发现第三条窄域路线——场景需求全部落在 `xhr.response.body` 一个节点上，而 `_zwParseEl` 的 R3019 mutTree 桥（`_ensureMutTree`）本就把该节点惰性物化为 wrapper-world `_zwMEl` 实体子树（`__zw_parse_html_child_nodes` + `_zwMBuildNode`，inert 印章同盖），仅差三件事：物化时机（惰性→主动）、接口原型接线（裸 `_zwMEl` 不接线——R125 在 `_zwMBuildNode` 只覆盖解析子）、`doc.body` 指向（仍是 `_zwParseEl` 快照壳）。

修复（part03 XHR HTML 文档分支，`self.response = new _zwParsedDoc(docText)` 后）：body 的 `_ensureMutTree()` 主动物化 + R125 配方接线（`__zwHtmlTagIface.body` → `HTMLBodyElement.prototype`）+ `self.response.body` 换实体节点。**读语义零变化**——旧 body 的 firstChild/childNodes 读面本就惰性经同一 mtree；**DOMParser 语料零波及**——graft 仅在 XHR `responseType='document'` HTML MIME 分支，`_zwParsedDoc` 工厂与 `_zwParseEl` 基面未动。head/documentElement 保持快照面（场景不触及，不为一次使用扩面）。

wrapper-world 实体面经 s54 探针全链验证（connected-in-template 同面）：innerHTML 写解析接 R4100 扫描、qSA 返真实节点（parentNode 链完整，互斥簇可走树）、open IDL + toggle 队列全通。

## 验证

- 探针页 zzz-probe-s55.html（identity / 全链 / 快照读面三子测试）全 Pass，**已删**
- name-attribute.html **17P/1F → 18P/0F**（F 族三案全收口）
- 全域 interactive-elements 98 files **435P/48F/0T → 436P/47F/0T**（GAIN 1 / REG 0；439 计数含未删探针 3P）
- xhr 语料回归对账：stash 前后双侧同跑 net-api xhr 切片 **580P/14F/21T 逐位一致**（REG 0；基线二进制 R5262 锚点 0、graft 后 1 双验）
- escaping.html（html/syntax，XHR document 读面消费方）4 子测试全 Pass（XMLHttpRequest 面在内）
- zero-engine lib **3448P/0F**

## 未解决项

- F 族 name-attribute 清空。挂账序：B 族 pointer-capture 2F（s49 定性指针管线深水）、C 族渲染流 20F（跨流挂账）、declarative shadow 联合切片待用户点名、parser form pointer 4F 用户决策缓行
