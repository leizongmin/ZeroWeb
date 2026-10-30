# M4 form 面引擎阻塞：shim 双计 + materialized form submit 失效

**日期**: 2026-10-31
**影响面**: form36 通道全部 form 面（encode-form-common.js 依赖页 ~58 文件）；
href 面 / encoder API 页 / decode 面（enc31）不受影响，脚本 face 正常。
**结论**: 上游 WPT 标准 helper `encode-form-common.js` 的「脚本建 DOM + load 后
getElementsByTagName 重查询 + collection[0] submit」形态在本引擎当前实现下
必然挂死。属引擎共享路径缺陷，修复是独立工作项（两项，见文末），M4 form
面数据在该修复落地前无法产出。

## 症状

- 旧分片数据（只设 `ZW_SCRIPT_TIMEOUT_MS=600000`，case 预算缺省 30s）：form
  面全灭，每 variant 切片恰 1 合成 Timeout——case 超时预算缺陷的一层。
- 修正 env（补 `ZW_CORPUS_CASE_TIMEOUT_SECS=600`）+ 补拉 `/common/blank.html`
  （pin 315976933870b34d6ea30e3f6643403edae678ba）后探针页仍挂：5 个 pending
  测试无完成回调，`state={"loaded":true,"pending":5,"phase":2/3}`，iframe
  load 事件从不发生。**第二层才是根因**。

## 二分台账（diag 探针页，scratch 已清理）

| 探针 | 形态 | 结果 |
|---|---|---|
| A/B/C、diag12 P1/P2、diag13 V1–V4 | 静态 HTML 或 JS 创建 iframe+form、直接引用原始元素 submit（含 load 监听器内、acceptCharset、双 iframe、onload 后赋值） | 全 Pass |
| diag6/9（真实 helper + 埋点面包屑） | helper 微缩页，重查询后 frames[0]/forms[0] submit | 挂死：面包屑停在 `submit-ok`，`iframe-load` 永不出现 |
| diag14 | 动态建 1 iframe，t0/DCL/load 三点普查 | t0=1 个 → DCL 起 2 个（getElementsByTagName 视图） |
| diag15 | 重查询 + collection[0] submit（helper 最小复现） | 挂死 |
| diag16 | expando 身份探针 + collection[last] submit | `[0]`=新 proxy（无 expando）submit **静默无效**；`[1]`=原始 proxy（有 expando）submit 正常；body.innerHTML 只含 1 份标记 |
| diag17/18 | 双路径对照 | `document.getElementsByTagName`=2；`documentElement.getElementsByTagName`=1；escape-hatch `__zw_native_get_elements_by_tag_name`=1——**真实树只有 1 个节点，是 JS wrapper 分裂** |

## 根因链（引擎架构三层叠加）

1. 管线为「全解析后统一执行脚本」（`run_page_scripts_impl`，webview.rs
   :3413）：脚本 `createElement`/`appendChild` 经 shim wrapper（handle 身份）
   记入 `_zwPendingAdded`（part05.js:11183）并落 mutation 队列；webview apply
   后真实节点 materialize 进树。
2. **缺陷①（双计）**：load 时 apply 走 `apply_dom_mutations_and_render`
   （webview.rs:1248），该路径**不下发** `__zw_apply_generation_bump` 换代
   通知（仅 `apply_mutations_subset`:2616 与 `apply_pending_shared_mutations`
   :4898 两处下发）。shim 的 R382 ghost 驱逐（part05.js:10713–10725）被代际
   门挡住（「同 turn 条目零成本直过」），pending 条目永不触发
   `_zwHandleInHostSnapshot` 真含判定 → 元素已在视图基座（快照）里、pending
   overlay 仍按 R51c 并入原始 proxy → 文档级枚举**每个动态元素渲染两次**
   （[快照新 proxy, pending 原始 proxy]，顺序见 diag18 doc{}）。
3. **缺陷②（submit 失效）**：快照侧新 proxy 的 `form.submit` 经 R3048 路由
   （part04.js:11713，`_realTag(sel, handle) === 'FORM'`）能取到函数且执行无
   异常，但对 materialized form 导航不发生（parse-time form 的同型 proxy
   正常，diag12 对照）。submit 内部对 apply-materialized 节点的静默 no-op
   点未最终定位（`_zwFormSubmitNavigate`/form_activation 一层，独立排查项）。
4. helper 的 `frames[0]`/`forms[0]` 恰取快照侧新 proxy（缺陷①的顺序效应）→
   submit 无效（缺陷②）→ iframe load 永不到 → 全部 form 面测试 pending。

## ~~修复方向（独立轮次，共享路径）~~（已被并行流收口，见下节）

1. `apply_dom_mutations_and_render` apply 成功后补发
   `__zw_apply_generation_bump`（对齐既有两处），使 R382 驱逐生效、枚举
   单一化。——**未实施**：并行流 s168 已从 shim 侧收口同一根因（下节），
   该点不再改生产路径（工程注记：R383a 闩使 fuse/枚举正确性不再依赖该
   apply 点的换代通知）。
2. 排查 materialized form 的 submit 静默 no-op 点并修复。——**缺陷②未
   独立复现**：为缺陷①同一根因的下游表症（双计状态下 named-target 分支
   从双渲染 iframe 枚举取首条 proxy，身份态不一致），单一化后自然消失。
3. 修复落地后重跑 form36 form 面（`/tmp/zw-enc2-m4/m4-form-batch.sh` 就绪，
   env 已带 case 预算 600s），再走 tally 判读 DC-4。——**已执行，全绿**。

## 复核与收口（2026-10-31，并行流 s168 后）

**上轮诊断时点差异**：本 evidence 的探针与挂死结论基于 01:17 二进制
（早于 05:58 `daf3bc4c5` 与 06:09 `88ca4f5db` 两笔 s168 提交）。重建
runner 至 `4740787f5`（06:30 二进制）后按原形态复核：

- **diag19 型最小复现不再成立**：helper 形态页（script 建
  iframe+form+input，load 后重查询 `forms[0].submit()`，form.target 指名
  iframe）真实导航 named iframe，iframe load 正常派发；原始 expando
  proxy 对照路径同通。（探针页用后即删，未入库。）
- **四族 form 面抽检全绿**：big5 14,362P、euc-jp 7,392P、shift_jis
  7,393P、euc-kr 17,048P——上轮全灭的同型页面。
- **全量 58 job 批跑**（`m4-form-batch.sh`，12 lane，env
  `ZW_SCRIPT_TIMEOUT_MS=600000` + `ZW_CORPUS_CASE_TIMEOUT_SECS=600`）：
  ~2 分钟全部 rc=0（上轮同批全灭）；per-family merge 8 shard 文件数门
  全绿（batch 内联 merge 有 `c.split`→`c[0].split` 形状 bug 在统计打印
  崩一次，独立 `merge-fix.py` 重跑补齐；shard 内容首写即正确）。

**定谳（根因链修正）**：双缺陷为同一根因的两层表症——R51c/R382 机制下
跨代际 pending 桶并入枚举 overlay 造成 ghost 双计（缺陷①），双计状态下
named-target 提交分支消费双渲染枚举首条 proxy 静默失效（缺陷②）。
并行流 `daf3bc4c5` 的 shim 侧 R383（跨代际桶不并入 overlay）+ R383a
（apply 配对环境闩——游标>0 置闩恒跳并）+ webview 两 apply 点游标即时
发布（含 runner 页面管线 `run_page_scripts` 路径）使枚举单一化，两层
表症一并消失。本 goal 未改任何通道/引擎代码，M4 数据为纯数据轮次。

**终局 tally（DC-4 判读，tally_dc4.py 全 shard 汇总）**：

| shard | files | subtests | 状态分布 |
|---|---:|---:|---|
| enc30（一期根面） | 36 | 12,075 | 12,072P/3F |
| enc31 decode | 34 | 360,431 | 360,431P/0F |
| form36-euc-jp | 10 | 101,334 | 101,334P |
| form36-iso2022-jp | 10 | 93,880 | 93,879P/1T |
| form36-shift-jis | 15 | 138,301 | 138,301P |
| form36-euc-kr | 15 | 239,960 | 239,960P |
| form36-big5 | 16 | 193,181 | 193,181P |
| form36-schinese | 2 | 303 | 303P |
| form36-big5-encoder | 1 | 13 | 13P |
| form36-iso2022-enc | 1 | 12 | 2P/10F |
| **合计** | 140 | **1,139,490** | **1,139,476P** |

- **DC-4 全分母门 PASS**：1,139,476 / 1,152,394 = **98.879% ≥ 98.8%**
  （通过线 1,138,566）；passes 与全可收口天花板 1,139,476 **逐位相等**。
- 非 Pass 恰 **14** = 3F SAB（decode-utf8 transferred ×2 +
  textdecoder-copy ×1）+ 10F iso-2022-jp-encoder 流式模型簇（M1 挂账）+
  1T data:iframe（36 号在册契约挂案）——**全部在册残差，零意外**。
- **零回归**：68 文件 form/href 面 766,958P/1T 与 36 号在册账逐位一致；
  enc31 decode 360,431P/0F 与 31 号收口账恒等；enc30 12,072P/3F 不变。

## 过程事件（非根因，记账）

- shift_jis 整分片 03:16 静默黑死一次（rc=1、0B 输出、无守卫消息、无内核
  OOM 记录）；逐文件隔离重跑 15/15 恢复 href 面数据，死因未定（单次环境性）。
- enc31 euc-kr 族分片 05:45 再现静默黑死（rc=0、0B stdout+stderr、~10 分钟，
  同族形态）；逐文件隔离重跑 22/22 恢复（11 个 decode 页全真数据 1.8MB/页，
  `_chars-*`/errors 页 `[]` 为一期 31 号已收口族、runner 谓词不匹配，符合
  预期）。两次黑死均无守卫消息与内核 OOM 记录，疑与该族数据规模相关，
  逐文件隔离为既证恢复手段。
- rally 双 spawn 竞态再现一轮，按 memory 规则处置（终止重复体、弃半成品），
  处置细则已沉淀 `rally-double-spawn-race`。
- 本轮 diag 探针 18 页全部落在 wpt-data gitignored 区，未入库，收尾清理。

## M4 可跑面收口数字（2026-10-31 判读）

- **enc31 decode 面**：385 cases / **360,431P / 0F**——与 M0 判读账
  「decode 34 文件 360,431 = 31 号 100%P」精确吻合（euc-jp/iso-2022-jp/
  shift_jis/big5 整族分片 + euc-kr 逐文件恢复；schinese 族 `[]` 为预期：
  gb18030/gbk-decoder 为 `.any.js` 窗口腿，30 号在收）。
- **form36 非 form 面（href + encoder API + big5-enc-ascii）**：12 文件
  补跑 + 三族旧分片 href 剔除合成行后 43,275×3 + 43,264 + 43,513 +
  303 + 13 + 12 = **228,955 案账**（Pass 228,945 / Fail 10）。
- **tally 汇总**：grand 589,436 子测试 / 589,423P / **非 P 恰 13** = 3F SAB
  （decode-utf8 transferred-buffer ×2 + textdecoder-copy ×1）+ 10F
  iso-2022-jp-encoder 流式模型簇——与在册 14 项残差之 13 项逐条吻合，
  零意外残差；第 14 项（1T data:iframe）位于 engine-blocked form 面池内。
  可跑面通过率 99.9978%。
- **DC-4 全分母门**：FAIL（589,423 / 1,152,394 = 51.15% < 98.8%）。
  engine-blocked 池 = 1,139,490（预期本地三源）− 589,436（实跑）=
  **550,054 子测试**（form36 form 面 + data:iframe），待引擎两缺陷修复
  落地后由 `m4-form-batch.sh` 重跑补齐，再重算 DC-4。
  ——**已补齐并翻绿**，终局判定见上文「复核与收口」节（98.879% PASS，
  非 P 恰 14 全在册）。
