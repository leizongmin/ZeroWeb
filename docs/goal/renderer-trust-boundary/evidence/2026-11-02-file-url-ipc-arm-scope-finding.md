# file:// IPC 直读臂无范围策略——确认缺陷（DC-2 类）与修复设计

**日期**：2026-11-02
**性质**：确认缺陷（① 预执行核查连带发现：核查 browser 侧 file:// 链时发现 IPC 臂无 origin/范围策略）+ 修复设计（本切片只核查与记档，修复为下一切片）。
**核查面**：`apps/browser/src/fetch_proxy.rs`（enqueue file:// 臂 :276-287、`fetch_service_worker_request` file:// 臂 :513-517）、`crates/protocol/src/message.rs`（FetchParams :883）、`apps/renderer/src/ipc_fetch.rs` / `ipc_image_batch.rs`（renderer 请求发送面）、`crates/security/src/context.rs`（check_resource_url :342）。

## 缺陷陈述

browser 进程的两个 file:// 直读臂对 renderer 请求**无任何 origin/范围校验**——`FetchParams` 仅携 request_id/url/method/headers/body（`message.rs:883`），不带 initiator/document 上下文；臂前唯一门 `check_resource_url` 只做 HSTS 升级 + 混合内容（`context.rs:342`）。凡 renderer 发出 `FetchRequest(file://任意路径)`，browser 即以自身特权读本地文件并把字节回传。

威胁模型（renderer 已被攻破）下的链路：renderer 任意构造 file:// FetchRequest → browser 读任意本地文件 → 字节回 IPC → 经 renderer 直连网络外传（seccomp 拍板「网络面不动」）。无需任何页面参与。这正是 DC-2「普通页面进程申请读取…特权操作不可执行」的反面：本地文件读属 browser 特权操作，现对 renderer 全量开放。

## 实证反例（headless 真实进程，一次性行为探针不落仓）

http-origin 页面（`http://127.0.0.1` 本地服务）+ file:// 三面探针：

| 面 | 结果 | 判定 |
|---|---|---|
| `<img src="file:///…/pixel.png">` | **naturalWidth=1，加载成功** | 跨 origin 本地文件读确认（唯一 file:// 读取器即 browser 臂，renderer 直读路无 file:// 分叉，成功即证走臂） |
| 页面 JS `location.href = "file:///…/secret.txt"` | **导航成功，document.body 渲染出本地文件全文** | Chrome 对此显式拒绝（Not allowed to load local resource）；本仓无 scheme 迁移策略 |
| `<iframe src="file:///…">` 后读 contentDocument | `null`（SOP 拦截） | 直接 DOM 回读链不成立（SOP 成立，记档） |
| `<script src="file:///…secret.js">` | 未执行（`__leak=null`） | script 子资源另有未证门（记档观察，不追） |

CDP `Page.navigate` 到 file:// 亦成功（该路经 browser `navigate()` 属 sanctioned，本就是须保留的产品流——地址栏/会话恢复/CDP）。

## 请求面核查（谁可达臂）

- **主文档**：`navigate_with`（`runtime.rs:3366`）→ `AsyncPageLoad` → `IpcAsyncFetchHost` → 臂。`zero://` GET 已走 `TabNavigateRequest` 带 initiator 委托 browser（`runtime.rs:3390`）——file:// 无此委托，直接本地取。
- **图片**：`IpcImageBatch::issue`（`ipc_image_batch.rs:144`）发 FetchRequest，无 scheme 门——实证即此面。
- **SW 拦截 fetch**：`fetch_service_worker_request` 臂同形无门。
- **模块体动态 import**：renderer worker 本地直读（① 在册面），不经本臂。

## 修复设计（下一切片，server-authoritative，M1 模式）

策略锚点取 browser 自身知情面，不用 renderer 自报 initiator（被攻破 renderer 可伪造）：

1. **per-tab 最近 document URL（browser 自己服务过的）**：镜像 `note_hsts_response` 的 document 响应钩子（`fetch_proxy.rs:571`，仅 document resource_type 触发）记 `last_document_url`——browser 只在自己真实读过并回传 document 时更新，renderer 无法伪造。
2. **per-tab sanctioned 导航目标**：browser 发起导航处（`process_backend.rs:2012` `navigate()`——地址栏/会话恢复/CDP/history）记待决目标；renderer 自发导航（页面 JS/链接）无此标记。
3. **臂上策略**（两臂同规则）：
   - document 类 file:// 请求：放行仅当 (a) 命中 sanctioned 目标，或 (b) `last_document_url` 为 file://（file→file 链接/JS 导航，Chrome 允许）。
   - 子资源 file:// 请求：放行仅当 `last_document_url` 为 file:// 且请求路径在其目录子树内（Chrome same-dir-or-below parity）。

修复后：http 页面 img/导航读本地文件全被拒（F2 收口）；被攻破 renderer 在 http 文档 tab 下零 file:// 读、在 file 文档 tab 下仅子树读（F1 收口至 Chrome 同面残余）。既有产品流保全：地址栏/CDP/会话恢复（sanctioned）、file 页内子资源与 file→file 导航（锚点匹配）——`cdp_file_url_navigation` 两腿回归应保持绿（CDP 导航 = sanctioned 路）。

连带影响 ① 决策：本修复落地后，landlock 直读臂（模块体动态 import）成为 file:// 面仅存的无范围读取通路——①-a 翻默认的价值随之上升（见[决策包](../2026-11-02-decision-package-landlock-default-and-file-scope.md)）。

## 范围

- 修复实现 + 常驻回归（策略单元 + cdp 集成反例/对照）为下一切片；本切片仅核查、实证、记档。
- 策略面 = 多进程 IPC 臂（GUI `fetch_proxy::enqueue` + headless `proxy_fetch`）。单进程嵌入模式（zero-webview 库直连 ResourceLoader，renderer/browser 同进程）不在本面：威胁模型中被攻破对象是独立 renderer 进程，单进程无该信任边界，Threat model 不适用即不适用本策略；headless `#[cfg(test)]` 进程内 webview 同理（仅测试构建）。
- script-src file:// 未执行的门（哪条路径拦的）记档观察不追；iframe SOP 成立不追。
