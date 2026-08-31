# R360 — Document-URL 重定向语义（X-Zero-Final-URL 消费链；17→14 已知集历程的 -1）

**日期**: 2026-08-29
**切片**: M4 轻量修复（已知 Fail 集合巡检轮——realm/adoption 族归因 + 修复件）
**改动面**: `testharness.rs`（runner fetch handler 重定向生成器）+ `part01.js`（iframe 加载
final-URL 头消费）+ `part03.js`（URL/documentURI getter 读槽）+ `part24.rs`（+1 单测）

## 1. 巡检归因（15 集合逐个定域）

| 文件 | 失败形态 | 定域 |
|------|---------|------|
| Document-URL | `contentDocument.URL` 恒原 URL（无重定向跟进） | **本轮修复**（基建+shim 三件） |
| remove-and-adopt-thcrash | `window.open()` 无 popup 通道 | 环境基建（R325 已记档，不追） |
| Node-isConnected "Test with iframes" | iframe 子文档节点 connected 语义 | **探针深挖后转档**（见 §4） |
| MutationObserver-document 3F | parse-time record 管线缺失 | 深结构备档维持（R311 定性） |
| MutationObserver-cross-realm / create-element-realm / node-realm×2 | creation realm 跨 realm 语义 | 深结构备档维持 |
| querySelector-mixed-case indoc | handle-append 视图桥 | R220/L2 域维持 |
| remove-next-sibling-during-replace-with | 克隆 script 插入期执行 | R328 残余维持 |
| Event-dispatch-target-moved | sel 移动锚点失配 | 随 L2 记档维持 |
| click-on-absolute-pseudo | CSSPseudoElement API 缺 | css-pseudo 域不追 |
| event-global-onerror | 跨 realm onerror 恢复链 | 深结构备档维持 |
| Range-mutations data 族 ×2 | 注册表 GC 压力 + P1 前置 | R350/R353 已归档维持 |

## 2. 修复（Document-URL 三件链）

1. **runner fetch handler**：`/common/redirect.py?location=X` 内置等价生成器（wpt-data 无
   静态文件；同 R141 encoding.py 模式）——读 `?location=`（相对按请求原点解析）、取目标
   文件体、附 `X-Zero-Final-URL` 绝对最终 URL 头。
2. **shim `_zwFinishIframeEntry`**：解析响应 wire 的头域（`name\x1evalue` 分隔），命中
   `X-Zero-Final-URL` 即覆盖 effective URL——`doc._zwURL` 槽 + `entry.url`（history/URL
   面同源）。
3. **shim detached doc URL getter**：`URL`/`documentURI` 读 `_zwURL` 槽优先、缺省
   'about:blank' 不变（detached/createHTMLDocument 从不设槽——spec 非浏览上下文文档语义
   保持；iframe 加载后槽即最终 URL）。

## 3. 验证（landing 门）

| 门 | 结果 |
|----|------|
| 全量 dom sweep（polyfill，333 文件） | **55484P/16F/15T——真实 Fail 集合 15→14（Document-URL 退出），零新增零回归**（Timeout +1 轮转） |
| 目标件 | Document-URL 0P/1F→**1P/0F** |
| engine 单测 | v8 2486（+1 `test_iframe_final_url_header_r360`：带头 wire → doc.URL/documentURI 反映最终地址）/ quickjs 1471 全绿 |
| integration | 781P 全绿 |
| 文件级门 | QSA 1975 / matches 669 / appendData 384 / MO-attributes 42 / getElementsByTagName 19 / getElementsByClassName 3 / Element-children 2 / ParentNode-children 1 全持平 |
| clippy / fmt | engine + wpt-runner 双矩阵 `-D warnings` 零警告 / 无 diff |

## 4. Node-isConnected 探针深挖（转档记录，未 land）

探针实证 iframe 子文档 connected 语义缺三层独立链路：① main-doc handle proxy append 进
iframe doc body 走 R112 串行合并（无 parentNode 链接）→ stamp 无法从 proxy 写入（proxy
defineProperty/set trap 拦截）；② iframe 工厂 plain 元素自身缺 `isConnected` 面；
③ detached body 树对 plain 工厂子不回链 parentNode。已实现 stamp+getter 原型后验证不达
（stamp 经 proxy trap 不可达）——**三层耦合超出轻量切片边界，整组回退**；转档为
「iframe 子文档 connected 语义」专项（需 proxy stamp 协议 + plain 链回接成对设计）。

## 5. 后续

- 已知 Fail 集合余 14：深结构/基建域（realm 族 5、MO parse-time 3、Node-isConnected
  iframe 语义、sel 锚点、pseudo、redirect 外 1、Range data 族 Timeout 2）。
- M2（S6）前置重估、M5/M7 default-on（待用户点名）为主线。
