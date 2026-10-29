# M2 s143 · 查询视图已应用前缀重放双计——implicit-submission 提交按钮键翻红（游标修复 + flatten 鬼影随根消退）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s142（name/target 属性锚 + pending 签名 +target）

## 入场定刀位

s142 挂账刀 A′ 候选①：implicit-submission.optional「Submit event with a
submit button」1 键 Pass→Fail 的勘根收口。s142b 曾定谳为「提交导航触发
(iframe+form) pending 片段 body 真实再物化两份逐字副本」（既有旧疾，
stash 对照基线同现）。本轮勘根推翻该归因，钉死另一根因并收口。

## 勘根（假设演进 + 探针证伪链，探针页 zz-probe-s143 验后删）

1. **初判 flatten 鬼影（证伪为该键根因，修复保留）**：s142b 归因的
   「双重物化」候选机制——`_mo_notify` 汇流点把 (iframe+form) 片段展平
   入账，后代条目无槽/无 handle/无 MatParsed，apply 代际 bump 的 K3、
   t8r-4 两臂都清不到 → 永久滞留 pending 表（探针实证：populateForm 后
   pend 常驻 IFRAME/P/FORM/P/INPUT/INPUT 展平条目，BUMP 两次才归零）。
   修复后定向跑该键仍 Fail → 滞留为真但不承担该键，因果解耦。
2. **次判刀 A′ overlay 滞留窗口（同上证伪，修复保留）**：R379/pa2b 的
   apply 换代通知走 execute_script 常规队列，用户动作脚本（优先通道或
   沙箱直执）可反超 → shim overlay 补偿状态在 host apply 落地后仍未清算
   的窗口被读取穿透（探针实证窗口存在）。刀 A′ = script 执行边界把幂等
   `__zw_apply_generation_bump` 拼进脚本头（守卫 = 共享队列无未应用尾）。
   修复后该键仍 Fail → 因果解耦。
3. **v25→v27 探针链钉死真根因**：v25 修探针自身 f0err 后给出关键时间
   窗——提交事件 handler 执行中查询干净，提交导航后查询出双份；host 侧
   REFRESH dump 证基座 cached_html 恒单份（无真实再物化）；v26 证 qSA
   命中的两个 wrapper 均为位置选择器（`:nth-child(2)`/`:nth-child(4)`），
   后者在基座不存在；v27 RAW 证 `__zw_query_all` 本身返回并集 = 视图内
   插入被应用了两次。
4. **源码四件套闭环**：`with_query_view_doc` 的查询视图 = 基座 html +
   队列 structural mutation 重放 **`[0..count)`**；共享队列 append-only
   （R100，应用不消费）；基座经 R55/R348 注册站换新 Arc = 最新
   cached_html（**已含已应用插入**）→ 重放把已应用的 InsertAdjacentHtml
   再插一次；既有 id 去重只覆盖带 id 片段，targetted-form 语料的
   `<iframe><form>` 无 id → 双份同 target 表单（位置 2/4）→ qSA 2
   wrapper → `form.submitButton` 2 元素集合 vs `event.submitter` 单元素
   → WPT 断言翻红。s142 的属性锚使 `event.submitter` 语义正确化（单
   元素），基线的「双向一致错误」（结构路径钉单 → 集合也单 → 相等）
   被揭穿，故该键在 s142 前一直 Pass——旧疾揭露的实质是视图双计。
5. **触发面拓扑**：execute_dom_script 沙箱路径执行期间用私有队列（私有
   Arc）→ 尾部 R4400 回灌共享队列 + applied_mutations 游标推进 → R348
   重绑共享队列 + 新 Arc → 重绑后的异步续延查询（时线中 bump 后到位者）
   触雷。executor 优先通道队列每批清空天然正确；dispatch_dom_event_inner
   每次新 Arc 同理触雷。私有队列纪元内 `[0..count)` 全为本脚本 pending，
   重放自 0 起本就正确。

## 施工（4 文件，+129/−11）

- **`crates/engine/src/js_dom_bridge/callbacks.rs`**（核心）：新增
  `MUT_APPLIED_CURSOR` thread_local + `publish_mut_applied_cursor()`——
  webview 侧 `applied_mutations` 游标的桥侧镜像。`with_query_view_doc`
  四点接入：`applied = cursor.min(count)` 钳制（防未来乱序防御）；重放
  区间 `[applied..count)`（原 `[0..count)`）；live_ok 直读判定扫描同区间
  收窄（区间内无 InsertAdjacentHtml 才可直读 live）；链步（chain）路径
  不变仍用 prev_count（链步前提 = 同 Arc 同游标纪元，游标推进必换 Arc
  必 O(1) miss）；全量重建起点 = applied。R57 文档注释补 s143 重放区间
  说明。无须加游标缓存键项：游标只在 script 边界推进，推进必伴随
  re-register 换 Arc，O(1) 命中天然按纪元隔离。
- **`crates/webview/src/webview.rs`**（4 发布点）：run_page_scripts_impl /
  execute_script_with_dom / dispatch_dom_event_inner 三处共享队列注册站
  紧邻 `publish_live_query_doc` 发布 `applied_mutations`；导航复位
  （队列清空 + 游标归零处）发布 0。
- **`crates/webview/src/webview/user_actions.rs`**（3 发布点）：executor
  分支先发布 0 清同线程陈值；私有队列注册站发布 0；R348 重绑块发布
  `applied_mutations`（含 R4400 回灌推进）。
- **`crates/engine/src/js_dom_shim/part05.js`**（真实记账修复保留）：
  汇流点展平后代盖 `_zwMoFlatRoot` 根印 + bump 内根不在 pending 表时
  随根同批作废（splice + set delete + id 表清除 + 全桶剔除）。刀 A′
  （user_actions.rs script 边界幂等 bump 拼头）保留。两处注释均改写为
  不承担 submitButton 键的准确表述（原注释误记为该键根因）。

### 语义后果（设计审视，两项均为既有设计的对齐而非新行为）

- live_ok 区间收窄：apply 落地后的查询更多直读 live（视图更新鲜），与
  R102「无 pending structural mutations 时直读 live」设计一致。
- subset 路径 apply Err 仍推进游标、cached_html 不更新（webview 既有实
  现）：旧全量重放会在视图里复活被丢弃批；新尾重放后丢弃批与 pump 错
  误路径「丢弃批、refresh_if_html_changed 按 shim 真相兜底」的既定设计
  对齐。

## 验证

- 定向：implicit-submission.optional FILTER 3 跑全 Pass（目标键
  Fail→Pass 稳定）。
- 目录 A/B：form-submission-0 全目录 248P/69F 对基线 +3P 零坏位移。
- **全量 join**（对 s142 fixed 真基线 /tmp/zw-s142-full-fixed.log，
  7994 行 = 7037P/858F/99T 与 s142 在册吻合）：`(key,state)` 对级
  `LC_ALL=C sort -u` 后双侧各 **7961 对**、join **7951 键**——
  **恰 1 键 Fail→Pass = implicit-submission.optional「Submit event with
  a submit button」**（基线 1 Fail → 修复后 1 Pass），零 P→F、零键消失
  零新增、行数守恒 7994 = 7994（**7038P/857F/99T**，ΔP=+1/ΔF=−1 精确
  闭合）。s142 记载的 26 键 base-only + 2 键 Fail→Timeout 本轮零重现
  （两侧键集完全相等，该轮为多跑波动）。
- 分类假象核验（key 级 state-set 纪律）：form-submit-action 10 键
  state-set 双侧逐字节同形（每键恰 {Fail,Pass} 各 1——runner 同文件
  多跑 flake，s141 已定性）；execution-timing/149 一子测试名内嵌真实
  TAB 字符致 TSV 分列错位（双侧字节一致）——均为分类工具假象非位移。
- 门禁（**rebase 后合并树复跑**——提交前 git pull --rebase 并入并行
  流 5 提交：net-api M2-S6/S8/S9/S10 + layout ×2 + rendering 文档，
  含真实代码面，故全部门禁在合并树上重出）：
  - clippy（zero-engine + zero-webview + zero-wpt-runner
    --all-targets -D warnings）零告警零错误。
  - engine lib **3591/9F**（总数 3597→3600，+3 并行流新测试）——
    8 名单与 s140 定谳基线**逐字节相同**（fetch_sync_wire_preserves_
    response_contract / test_fetch_appends_cross_origin_request_origin /
    test_fetch_forbidden_headers_r3221_r3222 /
    test_fetch_passes_request_credentials_to_host /
    test_iframe_service_worker_messages_dispatch_to_iframe_container /
    test_request_headers_guard_r3223 / test_request_body_readers_r2982 /
    test_response_request_constructors_r2968）；第 9 位轮换：details
    flake 本轮通过（在册间歇），**新面孔
    test_request_signal_passthrough_r3045**——该测试出自 2026-08-09
    295720ac8（旧提交，rebase 前树含本切片改动时通过），并行流
    dc0689f45「Request signal 身份族（M2-S8）」改动破坏其 passthrough
    契约（delta=并行流，与本切片无关）→ 归口 net-api 流勘根。
  - runner bin **258/0F**（+2 并行流新测试；trend_csv_check 被并行流
    收口，基线改善）。
  - fmt 合并树：本切片 4 文件零 diff；既有 js_dom_bridge.rs 7 处（并
    行流 net-api/url 区域，不重排）+ testharness.rs 3 处（并行流
    layout 提交带入，归口该流）。
  - shim 13-part 规范序拼接（part01,01b,02..07）node --check 过
    （104239 行）；探针页验后删，语料零改动。
- 过程注记 ×2：①首轮全量跑误用删插桩前构建的陈旧二进制（ZWDBG 逐泵
  日志 56 万行 → test-guard 单进程 4GB 超限被杀，产物作废）；以最终
  源码重建后重跑（二进制 mtime 晚于全部源码改动 + `strings` 无 ZWDBG
  实证）——无人值守跑必须确认二进制新鲜度后再读数。②rebase 前后全量
  各跑一次：**两跑逐字节相同**（7038P/857F/99T、7961 对、7951 键、
  恰 1 F→P 同键）——并行流未动 html/semantics 面，本节数字对两树同
  时成立，最终以合并树跑为准。

## s142b 归因修正（控制面同步）

「提交导航触发片段 body 真实再物化两份逐字副本」重新定性：submitButton
键无须真实再物化即可完整解释（查询视图双计，基座恒单份）；flatten 鬼
影滞留为同族 overlay 双视向量（已随根消退修复）。s142b v3 的
body.innerHTML 双副本观察是否含独立「真实再物化」成分，待鬼影修复后
重新观察定谳；刀 A′ 挂账据此改记（见下）。既有预存残留提及：
part04.js `__zwDbgS`/firing 为既往切片已提交探针残留（s141 及更早），
非本轮引入，未动。

## 挂账（本切片更新）

- **刀 A′（片段双重物化）改记**：submitButton 键已由游标修复收口，该
  刀不再承担该键；已施工部分（script 边界幂等 bump 拼头 + flatten 鬼影
  随根消退）为 shim overlay 滞留面/记账正确性收口。「真实再物化」重探
  降为观察项（鬼影修复后 v3 形态复核），crash→hang 递进面（sel 消费失
  配）归刀 C 邻域。
- **刀 C re-entrant submission hang**（不变）：submit handler 内
  re-entrant requestSubmit/submit 形态 hang（runTest2 矩阵 +
  form-submission-algorithm 第 9 子测），firing submission events 重入
  防护独立语义收口。
- **刀 B runner 多跑+首遍挂**（不变，本轮再实证）：form-submit-action
  10 键 state-set {Fail,Pass} 双态；s141 探针实证单文件 runner 执行
  3 遍、遍 1/2 全挂遍 3 才绿。
- **accept-charset w1252 + `&#NNNN;` 实体回退 + lone surrogate USV**
  （候选②，18 案）不变。
- **engine lib 9F 环境面**（合并树复验：8 名单逐字节重验不变 + 第 9
  位轮换 details flake / test_request_signal_passthrough_r3045，后者
  **归口 net-api 流**勘根——M2-S8 Request signal 身份族改动破坏
  R3045 契约测试，与本切片无关）。
- 承袭不变：「Named elements should override builtins」1F 挂待决；
  encoding-legacy-mb CSV 行错位归口 encoding 流；echo 回显体 HTML 解析
  面观察；RadioNodeList live 语义近似、past names 条目不显式清理、
  expando store/target 双轨、part03 raw 链 type 读通道未勘 + reset 臂
  无还原臂、reset.html form element pointer 面（待决缓行）、
  execution-timing 构成面、lazy-loading 26F、meta color-scheme 21F、
  写流盖章断链、part02 setAttribute 绕 notify、041 nested write、
  deep clone 非 HTML ns、css-module corpus 门（待决）、www2 可达性、
  module-import-referrer TLA、dynamic import() with 透传、classic 外链
  SRI、143 write。
- **待决（须用户拍板）**：css/text 语料资源、green-2x2.png、parser
  form pointer 缓行（2026-10-23）。

## 下一步（s144 候选）

①刀 C re-entrant submission hang 语义收口（firing submission events
重入防护 spec 步骤）；②刀 B runner 多跑机制勘根；③accept-charset
w1252 + 实体回退 + lone surrogate USV 18 案；④「真实再物化」观察项
复核（鬼影修复后 v3 形态）。
