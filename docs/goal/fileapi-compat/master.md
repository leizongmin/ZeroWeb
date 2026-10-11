# File API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../fileapi-compat.md](../fileapi-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-11（M2-P4 部分收口）

## 当前状态

**目标 Active（M1 + M2-P1/P3/P4 完成——可执行子集 97.8%，残余 11F 全部跨域）**：
35 案 499 子测试 **488 Pass**（M1 62.3% → M2-P1 87.2% → M2-P3 93.6% → M2-P4 97.8%，
单调上行，0 Timeout）。
M2-P4（10-11）：Blob.bytes() +6、slice [Clamp] 恰半取偶 +6、detached buffer 按 part
空（根因 postMessage 先 detach 后 clone 的序错）+4、FileList 接口原型化 +4、
createObjectURL path UUID v4 +1。探索后回退记账：indexed-properties 结构判据不可行
（上游钉 `{0:'FAIL',length:1}` 普通对象 TypeError），须元素代理品牌。证据：
[evidence/2026-10-11-m2-p4-scattered.md](evidence/2026-10-11-m2-p4-scattered.md)。

里程碑台账：M1（10-11）通道 + 甄别 + 基线 62.3%；M2-P1（10-11）构造器 WebIDL
+124P → 87.2%（endings 簇随构造器收口，上游语料冲突 chromium 仲裁记档见下）；
M2-P3（10-11）readAs* 状态机 +32P → 93.6%（事件 task 语义，见 learning
docs/learnings/platform/2026-10/2026-10-11-wpt-eventwatcher-task-dispatch.md）；
M2-P4（10-11）零散面 +21P → 97.8%。逐轮证据见 evidence/。

## 缺口清单（残余 11F，全部跨域挂账）

| 簇 | F | 归属 |
|---|---|---|
| 元素代理 String 化（String(el)='body' vs '[object HTMLBodyElement]'、Attr/toStringTag、select.length/索引） | 5 | 元素代理面专轮（js-dom 通道须复跑） |
| url-with-fetch/xhr（Request 构造/revoke 生命周期、fetch fragment） | 4 | net-api Request 语义 |
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

- DC-2 门禁参照 webstorage 先例：分级通过率不再下行两个连续轮次 + DC-3 逐修复
  回归测试（本 goal 以 testharness-fileapi 通道 + evidence JSON 为常驻断言集）+
  make test 全绿 / make reftest 零回归 / product-smoke 恒值。当前 97.8% 且残余 11F
  已全部定域挂账——本 goal 可执行面已收口，达标后走 M3 归档评审。
- 数字维护：每轮修齐后向 `docs/compat/trends/wpt-suites.csv` 追加行
  （M2-P4 行 2026-10-11 = active，499/488）。
- 元素代理专轮动 @@toPrimitive/toString 时：同轮复跑 js-dom + html-syntax 通道
  （String(el) 现返 tag 名被 R34xx canvas 断言消息消费，移除影响面全语料）。
- 事件派发粒度经验：FileReader 族事件必须 task 承载（setTimeout），microtask 链与
  promise_test 完成回调竞速——后续 shim 内新增异步事件接口（XHR 进度、IDB 等）照此先例。
