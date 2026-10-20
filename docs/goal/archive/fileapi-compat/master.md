# File API 兼容 — 运行时控制面板（master.md）

> **已归档（模式 A，2026-10-20）**：当前终态、历史验证与挂账保留；入口见 [归档目标](../fileapi-compat.md)。

**入口文档**: [../fileapi-compat.md](../fileapi-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-12（收口记账修正——csv completed 行 + evidence 计数对齐磁盘实际）

## 当前状态

**目标 Completed（2026-10-11 收口）**：可执行子集 35 案 499 子测试 **493 Pass =
98.8%**（M1 62.3% → M2-P1 87.2% → M2-P3 93.6% → M2-P4 97.8% → M2-P5 98.8% →
M3 复跑 98.8% 恒值，单调不下行，0 Timeout）。DC-1~3 逐项核对通过（收口判定：
[evidence/2026-10-11-m3-closure.md](evidence/2026-10-11-m3-closure.md)）——
本轮全量门禁：make test 71 suite 全绿、make reftest 708/708 零不一致、
product-smoke 15.40% ≤ 20% + struct 全 PASS、legacy smoke 39/39 struct PASS、
uievents 通道首次在 guard 墙钟内跑完（1795P/170F+T，TIME_LIMIT=3600，基线自此
建立；M2-P5 挂账销项）。残余 6F 全部跨域记账（Request 生命周期 5 → net-api、
ports 转移 1 → webmessaging），重入条件在案。

里程碑台账：M1（10-11）通道 + 甄别 + 基线 62.3%；M2-P1（10-11）构造器 WebIDL
+124P → 87.2%（endings 簇随构造器收口，上游语料冲突 chromium 仲裁记档见下）；
M2-P3（10-11）readAs* 状态机 +32P → 93.6%（事件 task 语义，见 learning
docs/learnings/platform/2026-10/2026-10-11-wpt-eventwatcher-task-dispatch.md）；
M2-P4（10-11）零散面 +21P → 97.8%；M2-P5（10-11）元素代理面 +5P → 98.8%；
M3（10-11）DC-2 恒值确认 + DC-3 全量门禁 + 收口。逐轮证据见 evidence/。

## 缺口清单（残余 6F，全部跨域挂账）

| 簇 | F | 归属 |
|---|---|---|
| url-with-fetch/xhr（Request 构造/revoke 生命周期、fetch/xhr fragment） | 5 | net-api Request 语义 |
| FrozenArray\<MessagePort\>（e.ports 转移后为空） | 1 | webmessaging ports 面 |

## 排除面（挂账，非待办）

- multi-global/第二文档族（url/ 全族 ~12 案，内容标记 `iframe`/`window.open`）→
  navigation-compat iframe 管道同门槛重入。
- dedicated worker 面 5 案 → workers-compat 管道域。
- streams 消费端 2 案（Blob-stream/TextStream）→ net-api streams 域同门槛。
- 表单提交回显 10 案（wptserve `.py`）→ runner 服务器回环，暂无计划。
- idlharness 3 案（WebIDLParser build 资产缺失）→ 上游资产导入评估。
- BlobURL 分区/HTTPS 5 案 → security-hardening 分区面。

## 上游语料冲突（记档，2026-10-11 仲裁）

mimesniff `parsing.any.js` 的 (Blob/File) 腿（865 子测试，mime-types.json +
generated-mime-types.json 全量输入 ×Blob/File 两断言）钉 MIME parse-serialize；
FileAPI corpus（Blob-constructor type_tests + File-constructor type 表）钉范围判据
U+0020~U+007E + ASCII 小写化。chromium headless 实测与 FileAPI 一致
（`TEXT/HTML;CHARSET=GBK` → `text/html;charset=gbk`、` image/gif ` 空格保留、
`gbk(` 不加引号）→ 保留 FileAPI 语义。**影响**：net-api 通道（已归档 goal）mimesniff
分从 100% 降至 ~54%（该腿 865 全挂），属上游语料间冲突非本仓回归；net-api CSV
历史行不改写，此记录为准。若上游 mimesniff 语料跟进 FileAPI 算法更新，该腿自然回绿。

## 后续重入条件（记账）

- **Completed 后重入**：残余 6F 两簇重入条件——Request/blob URL 生命周期随
  net-api Request 语义推进复验；e.ports 随 webmessaging 面推进复验。排除面
  （multi-global/worker/streams/表单回显/idlharness/BlobURL 分区）维持 M1 甄别
  记账。上游语料冲突仲裁（mimesniff 865 腿）如上游跟进 FileAPI 算法更新自然回绿。
- 常驻断言集：testharness-fileapi 通道（`make testharness-fileapi`）+ evidence/
  五份 JSON（m1-baseline / m2-p1 / m2-p3 / m2-p4 / m2-p5）——后续涉及
  js_dom_shim Blob/File/FileReader/URL.createObjectURL 的变更跑该通道防回归。
- 元素代理 String 形态已改 spec class 串（'[object IfaceName]'）——后续轮次若见
  dom 族 assert 消息文本 diff（format_value 串化元素）属预期，非回归。uievents
  通道基线 = 2026-10-11 M3 轮 1795P/170F+T（TIME_LIMIT=3600）；后续复跑低于
  TIME_LIMIT=3600 会撞 guard 墙钟。
- 事件派发粒度经验：FileReader 族事件必须 task 承载（setTimeout），microtask 链与
  promise_test 完成回调竞速——后续 shim 内新增异步事件接口（XHR 进度、IDB 等）照此先例。
