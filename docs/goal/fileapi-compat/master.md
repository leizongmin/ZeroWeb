# File API 兼容 — 运行时控制面板（master.md）

**入口文档**: [../fileapi-compat.md](../fileapi-compat.md)
**创建日期**: 2026-10-08（goal 立项） | **最后更新**: 2026-10-11（M2-P1 收口）

## 当前状态

**目标 Active（M1 + M2-P1 完成，下一切片 M2-P3 readAs* 状态机）**：可执行子集
35 案 499 子测试 **435 Pass = 87.2%**（M1 基线 62.3% → +124P，单调上行）。
M2-P1（10-11）：Blob/File 构造器 WebIDL 转换层（sequence/联合/字典/type 判据/
接口对象）+ endings 随收口 + Blob.arrayBuffer→ArrayBuffer + `_zwConsumeBodyBytes`
Uint8Array 交界；**上游语料冲突经 chromium 实测仲裁**——mimesniff parsing.any.js
(Blob/File) 腿 865 子测试钉 parse-serialize 与 FileAPI/chromium 钉的范围判据互斥，
保留 FileAPI 语义（net-api 通道 mimesniff 该腿转失败已记档，fetch+xhr/streams/url
零回归）。证据：[evidence/2026-10-11-m2-p1-constructor-webidl.md](evidence/2026-10-11-m2-p1-constructor-webidl.md)。

里程碑台账：M1（10-11）通道 + 甄别 + 基线 62.3%；M2-P1（10-11）构造器 WebIDL
+124P → 87.2%（endings 簇随构造器收口）。逐轮证据见 evidence/。

## 缺口清单（M2 剩余簇）

| # | 簇 | 规模（F） | 状态 |
|---|---|---|---|
| P1 | Blob/File 构造器 WebIDL 转换（含 endings） | ~150 | ✅ M2-P1（→87.2%） |
| P3 | readAs* 读取状态机（result 阶段 null、multiple-reads 队列、abort、Determining-Encoding、readAsDataURL type 串） | ~31 | ⬜ 下一切片 |
| P4 | 零散面：Blob.bytes() 缺失 5、detached ArrayBuffer 4、slice 多字节解码 6、filelist 空表 4、dom.window 平台对象 3、HTMLBodyElement String 化 2、url 边界 6、事件面 2 | ~33 | ⬜ |

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
  make test 全绿 / make reftest 零回归 / product-smoke 恒值。
- 数字维护：每轮修齐后向 `docs/compat/trends/wpt-suites.csv` 追加行
  （M2-P1 行 2026-10-11 = active，499/435）。
- M2-P3 动 FileReader 时同步核对 `result` 阶段语义（loadstart/progress 为 null）与
  `_start/_done/_fail` 状态机，spec 依据
  https://w3c.github.io/FileAPI/#reader-general-update-steps 。
