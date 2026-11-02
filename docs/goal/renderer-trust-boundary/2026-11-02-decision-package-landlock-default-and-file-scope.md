# 决策包：renderer landlock 默认值（① 后续）与 file:// 范围修复分层

**日期**：2026-11-02
**背景**：拍板 ①A（browser file:// 预解析 → LoadHtml 后翻 `ZW_RENDERER_LANDLOCK` 默认）经预执行核查（[证据](evidence/2026-11-02-slice1-preflight-fact-correction-and-headless-file-url-fix.md)）修正执行面后，遗留模块体动态 import 取舍待决；同轮连带发现 file:// IPC 直读臂无范围策略（DC-2 类确认缺陷，[证据](evidence/2026-11-02-file-url-ipc-arm-scope-finding.md)）。本包给出分层执行序与 ①-a/①-b 决策材料。

## 分层执行序（Layer 1 无须拍板，Layer 2 须拍板）

**Layer 1——file:// IPC 臂范围策略（确认缺陷修复，goal 授权范围内）**：按证据文档设计落地 server-authoritative 策略（sanctioned 导航 + last_document_url 锚点；document 面 (a)|(b)、子资源面 file:// 子树）。收口「http 页面读本地文件」与「被攻破 renderer 全量 file:// 读」两确认面，产品流（地址栏/CDP/file 页内导航与子资源）保全。**不依赖 ① 拍板，先行执行。**

**Layer 2——landlock 默认翻转（待本包拍板）**：Layer 1 后，file:// 面仅存的无范围读取通路 = renderer landlock 关闭时的模块体动态 import 本地直读臂（`js_worker.rs:2630` → `client.rs:643`）。翻默认即收口最后该臂。

## Layer 2 选项

| 选项 | 内容 | 代价与风险 | 行为影响 |
|---|---|---|---|
| **①-a 接受收窄翻默认** | landlock 默认开；file:// 模块体动态 import 被拒（`ZW_RENDERER_LANDLOCK=0` 操作员豁免） | 最小改动（默认值一行 + 文档）；landlock 生产生效已实证（双腿探针） | 现可用行为回归：file:// 页模块体动态 import 失败。Chrome 对 file:// 页模块加载本按 CORS 拒绝，收窄即 parity；仓库内 smoke/fixture 无 file:// 动态 import 使用（grep 核查零命中），回归面限用户本地文件场景 |
| **①-b 先泵化代理再翻默认** | worker→host fetch 走可泵等待，动态 import 经 browser 直读，行为不回归 | 中等改造：renderer 主循环等待机制重构 + fetch 代理接线；死锁约束——worker 线程执行 sandbox 回调，主循环 `reply_rx.recv_timeout` 同步等待，worker→host 同步 IPC 必死锁（代码核查在册） | 零行为回归；改造引入的调度回归风险须回归验证 |

**已评估不采纳**：动态 import 面按文档目录授予 landlock（landlock restrict 后不可扩权，renderer 进程长活承载多文档，安装期无法得知后续文档目录；除非 renderer 逐文档重启，成本高于 ①-b）。

**推荐**：①-a。理由：Layer 1 已把 file:// 特权读收口到 server 侧策略，landlock 直读臂是残余通路且仅剩模块体动态 import 一用例；Chrome parity 使收窄成为兼容性对齐而非能力缺失；仓库内无使用面；`ZW_RENDERER_LANDLOCK=0` 保留操作员出口。若用户认为 file:// 动态 import 须保持可用，选 ①-b，改造排独立切片。

## 请拍板

**① 后续（landlock 默认翻转）**：A. ①-a 接受收窄翻默认（推荐）／B. ①-b 先泵化代理再翻默认／C. 维持 opt-in 现状缓翻。拍板后 Layer 2 立即可执行（Layer 1 与拍板无依赖，已先行）。
