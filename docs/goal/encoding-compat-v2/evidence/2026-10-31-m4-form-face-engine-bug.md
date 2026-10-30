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

## 修复方向（独立轮次，共享路径）

1. `apply_dom_mutations_and_render` apply 成功后补发
   `__zw_apply_generation_bump`（对齐既有两处），使 R382 驱逐生效、枚举
   单一化。
2. 排查 materialized form 的 submit 静默 no-op 点并修复；验收 = form36
   任一 form 面单页跑通全 Pass + enc30/31/href 面零回归 + `make test` +
   `make reftest` 全绿。
3. 修复落地后重跑 form36 form 面（`/tmp/zw-enc2-m4/m4-form-batch.sh` 就绪，
   env 已带 case 预算 600s），再走 tally 判读 DC-4。

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
