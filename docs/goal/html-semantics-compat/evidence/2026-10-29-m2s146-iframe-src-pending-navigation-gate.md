# M2 s146 · iframe.src pending 导航门——src setter 对 pending 解析 iframe 误判 detached（algorithm t10 收口 + t11 连带落定）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s145（刀 D 祖先锚定结构路径）

## 入场定刀位（含候选①归口甄别）

s145 下一步候选①（accept-charset w1252 + `&#NNNN;` 实体回退 + lone
surrogate USV 18 案）**归口甄别后跳过**：form-url-encoding-compat
控制面（2026-10-29 立项，用户决策）明确「s138 挂账 ④（windows-1252
accept-charset + `&#NNNN;` 实体回退 + lone surrogate USV，urlencoded2
4 案）归口本 goal M1，25 号流不重复施工」——s145 挂账清单承袭时未
对照该归口声明，本轮修正：候选①改记归口 encoding 流 M1，25 号流不
施工（run-rules §9 工作面不重叠）。取候选②：algorithm t10 iframe.src
异步装载面（s144 定谳独立挂账：not-connected 检查已在位，残面 =
装载 + loadPromise 解析链）。

失败形态：t10「Cannot navigate (after constructing the entry list)」
触发**文件级 Timeout**（`pending:2`——t10 挂死阻塞 t11 一并未完成）。

## 勘根（探针五轮二分定谳）

1. **挂点不在装载链本体**：runner 静态兜底 `fs::read(root.join(clean))`
   存在且 `wpt-data/common/dummy.xhtml` 在位（200 可达）；直调
   `__zw_reload_iframe(iframe, '/common/dummy.xhtml', {})` 装载成功
   （P0f）；src 属性写入正常（P0g）。loadPromise/onload 通道由
   t1-t9 同链 Pass 旁证（其装载源 = form 提交导航 reload，无门）。
2. **挂点 = set trap 导航门**：`iframe.src = …` 走 part05 R3069 反射
   分支尾钩，门条件 `_zwIframeNavigationConnected(sel, handle)` 对
   populateForm 产物恒 false → `__zw_reload_iframe` 永不触发（探针
   P0a/P0b：drain 后 location 恒 about:blank、load listener 零触发）。
3. **门失明的机制**：`_zwIframeNavigationConnected` sel 臂依赖
   `__zw_contains('html', sel)`；该回调走
   `with_query_doc_live_aware(&snap, true, …)`（R102，live_ok 硬编码
   true）——live doc 与 cached_html 快照**均不重放 pending structural
   mutations**（重放是 `with_query_view_doc` 的独有语义）。同任务内
   insertAdjacentHTML 产物（pending 解析节点）对 contains 恒不可见
   （P0e/P0i：populateForm 后立即查锚定 sel 返 '0'）→ 门把已插入
   connected 树的 iframe 误判 detached。
4. **为什么现在才冒**：src setter 门（navigation-compat M3 引入）的
   既有消费形态是静态/已物化 iframe；t10 是首个「insertAdjacentHTML
   产物 + 同任务连环设 src + loadPromise」三要素齐备的语料。t10 的
   语义本体（formdata remove → cannot-navigate 中止）s144 已收口，
   本轮残面纯为装载触发时序。

## 施工（part01.js 1 处，+14 行）

`_zwIframeNavigationConnected` sel 臂**前置 pending 解析节点判定**：
`_zwPendingParsedForSel(sel)` 签名命中（挂 `_zwSelPendingParent` 槽
= 插入点已定且目标在 connected 树内）即返 true——spec「navigation
starts once the element has a browsing context through insertion into
a connected tree」对 pending 已插入形态成立。常态零开销（helper 无
pending 解析节点即早退）；签名歧义保守返 undefined 落原 contains
判定；handle 臂（createElement 链，`_zwNodeParent` 本地 registry）
不在病灶不动。载入 R102 live/cached 双路径与 s141 pending 双身份
修复的既有语义对齐（「pending 已插入 = connected」非新增状态）。

## 验证

- 探针 zz-probe-s146（验后删）：修复前 P0a-P0j 五轮二分（装载未触
  发 → 门挂 → contains 两拍失明）；修复后 P0k（pending 同任务设
  src 即装载）+ P0l（完整 t10 形态：wasLoaded 门 + submit 中止 +
  loadPromise resolve）全 Pass。
- 定向 form-submission-algorithm **11/11 全绿**（t10 Pass + t11
  「Submission URL should always have a non-null query part」Pass
  ——被 t10 挂死阻塞的 pending 案连带落定）。
- 定向 form-submission-0 全目录 **279P/63F/0T**（vs s145 277P/63F/
  1T）；(key,state) 对级 join：**位移恰 3 条**——文件级 Timeout 行
  消失（lost 1）+ t10/t11 两键 Pass（gained 2），零键丢失零坏位移。
- **全量 join**（对 s145 /tmp/zw-s145-full.log 7986 对）：s146
  **7070P/851F/98T**（结果行守恒 8019=8019，键对数 7986=7986）。
  位移 6 条全归因：主刀 3 条（algorithm 文件 Timeout → 文件 Pass +
  t10/t11 Pass）；在册 flake 报法 2 条（src-then-lazy-load 键
  Pass→文件 Timeout——s144/s135/s136 在册负载时序 flake，刀 B 邻
  域；link-multiple-load-events media 键本轮未及上报——两轮同为文
  件级 Timeout，挂起时序下方键集报法漂移，非代码位移）。零真 P→F。
- 门禁：clippy zero-engine --all-targets -D warnings 零告警；shim
  13-part 规范序拼接 node --check 过（109667 行）；engine lib
  **3593/9F**（总数 +1 并行流新测试；失败名单与 s145 的 9F 逐字节
  相同——8 已知环境面 + R3045 归口 net-api；details 在册间歇本轮
  采样回绿）；runner bin **258/1F**（trend_csv_check 行 196 并行流
  f62bf89aa 归口不变）；本轮零 .rs 文件改动（fmt 无新面）；
  重建后 `grep -a` 核验新 shim 串进 runner 二进制（执行体证据）。
  验证集对齐 s142/s145 同面先例（定向 + 全量 join + lib + runner
  bin，无 reftest 义务——行为面非渲染）。

## 挂账（本切片更新）

- **t10/t11 收口**（本轮主刀）。form-submission-0 目录 Fail 面不变：
  form-submit-action 40F/urlencoded2 6F/text-plain 6F/multipart 6F/
  submission-checks 3F/input-type-change 2F = 已知基线面。
- **候选① 改记归口**：accept-charset 18 案归 form-url-encoding-compat
  M1（该流控制面明文），25 号流移出候选池；控制面挂账清单同步。
- **R102 live_aware(true) 回调簇理论缺口**（新记，低优先）：同款
  live_ok 硬编码的回调（sibling/child_nodes/template_contents/
  element_exists 等）对「同任务 insertAdjacentHTML 后立即查询」形
  态均有与 `__zw_contains` 同构的 pending 失明窗口——既有全量绿面
  说明语料覆盖未踩；发现踩坑语料再按本轮同型补 pending 臂，不做预
  防性扩面（影响面 = 既有「查询反映 pending 前」断言语义，R3190
  先例）。
- 承袭不变：刀 B runner 多跑 + 首遍挂（本轮新增两例同型报法漂移观
  察：src-then-lazy-load、link-multiple media 键）；C2 input-type-
  change 2F；engine lib 9F 环境面；runner trend_csv_check 行 196
  （归口 encoding 流）；「真实再物化」观察项；无锚链碰撞理论面盘点
  （刀 D 残余）；待决三项（css/text 语料资源、green-2x2.png、parser
  form pointer 缓行）。

## 下一步（s147 候选）

①刀 B runner 多跑机制勘根（多跑/首遍挂/键集报法漂移三面收口，含
details flake ≈40% 翻转率强化观察——本轮再添两例同型）；②无锚链碰
撞理论面影响面盘点（刀 D 残余，无语料面，盘点性质）；③全量基线翻
拣下一单根面（63F 目录基线面之外的文件级 Timeout 族——csp-blocked
报法漂移同面）；④按 encoding 流 M1 进度对账 form 面 46 文件能力
缺口（不施工，仅对账边界）。
