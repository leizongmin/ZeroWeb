# M2 s226 — source relevant mutations 移除/移动/插入臂 + settle 代际守卫（含纪元对象缺位回归修复）

日期：2026-11-02。切片号 s226（承 s225）。

## 入场定刀位

s225 下一步候选首选「update-media/source-relevant-mutations 选源响应面余案
勘根」。靶案 `source-relevant-mutations.html` 1F（s225 embedded-content 基线
唯一位移面之外的在册挂账项）。

## 靶案与 spec

失败键：*Neither the removing, moving, nor insertion steps for the source
element track the previous 'next sibling' pointer when triggering relevant
mutations for remaining child image elements of its old parent*。

spec（HTML §relevant mutations）：source 元素的移除步/移动（concept-node-remove
+ 随后的插入）/插入步都要对旧父 picture 内「其后 img 兄弟」触发 relevant
mutations——位置锚是**移除前捕获的 former next sibling**。shim 侧
`__zwImgTreeMutated5277(sel, handle, node, isInsert, afterAnchor)` 第 5 参即
该锚（s126 引入，part04 R188 传递）。

## 勘根（探针分型四缺口 + 一竞态）

1. **removeChild 锚为文本节点**：快照兄弟链含空白文本节点，文本锚在
   element-only children 枚举中身份恒 miss →「其后 img 兄弟」全部漏重验。
2. **insertBefore 移动缺旧父摘除半**：移动（moveBefore 同委派）把 source 移出
   picture 时只派新父插入钩；spec concept-node-remove 的移除步同样触发旧父臂。
3. **`source.remove()` 整臂缺位**：remove() 元素分支与 removeChild 域不同钩，
   relevant mutation 不因 API 入口分叉。
4. **afterbegin 位次链缺失**：`_insertAdjacentVariadic`（prepend/insertAdjacentHTML
   afterbegin）的 per-item nextSibling 链落 null → overlay 尾插近似使 prepended
   source 落 JS 视图末位；source-set 选择核按视图序首个 source 命中即返回，
   位次错即断言窗选源错（R4600 只覆盖 beforebegin 位型）。
5. **settle 竞态（probe12）**：链式 fetch 完成序 ≠ run 序——前 run 的链在后 run
   删除态之后落态，后 run 的 InvalidState 臂见「态已存在」跳写，陈旧 URL 永驻
   currentSrc（记录半断言窗读 s220 记录表）。

## 施工（shim 3 文件 +146/−9，零 Rust）

- **part03 fetch 臂 settle 代际守卫**：run 启动捕获 `_zwImgDecodeEpoch5277[key]`
  （s224 纪元表复用），四个 settle/失败点比对——被取代 run 的完成一律丢弃
  （状态不写、事件不派、在飞计数照平）。**纪元对象缺位 = 从未发生取代 → 非陈旧**
  （初版 `!_ep → stale` 连同捕获行对 undefined 对象取键 throw，见下回归段）。
- **part03 TreeMutated 三臂记录半并调**：SOURCE 移除臂兄弟环 / 插入臂 / 带锚
  移除臂在 `__zwImgRevalidate5277`（fetch 半）旁并调
  `__zwImgSelectionQueue5277`（s220 记录半）——断言窗微任务读记录表才见新选源
  （s221 IMG 臂同模式）。插入臂 `afterAnchor === null` → 保守全量重验
  （afterbegin overlay 尾插近似使位次不可靠；重验幂等，未受影响 img 重算同选源）。
- **part04 removeChild（R188）锚过滤为元素**：`_r188Next` 沿 nextSibling 跳过
  非元素节点得 `_r188NextEl`；MO record 的 nextSibling 保持原始节点（spec record
  语义不动），仅 img 钩传元素锚。
- **part04 insertBefore 移动形旧父摘除半**：捕获移动前旧父 sel/handle + wire
  nextSibling 快照（`_r334bRnext`），插入钩后补派旧父
  `__zwImgTreeMutated5277(oldSel, oldHnd, newNode, false, nextEl)`。
- **part04 remove() 元素分支**：与 removeChild 域同钩（锚过滤为元素同款）。
- **part05 `_insertAdjacentVariadic` afterbegin 头插链**：`_r5277abPrev`/
  `_r5277abFirst`——items 已反序（插入序），第 k 个插入项的后继 = 上一个插入项，
  首个 = 插入前原首子；三条插入路径（fragment 展开 / 直接项 / pending sel）同链。

## 回归与修复（decode 目录一轮红）

初版 settle 守卫使 `image-decode-path-changes.html` 文件级 Timeout（decode 目录
56P + 1T，基线 60P/0F；solo 复现 pending:4）。二分定谳：去守卫复跑全 Pass（唯一
病灶）。根因——该文件全部为纯 detached `new Image()` 流：零 bump →
`globalThis._zwImgDecodeEpoch5277` 对象不存在 → 捕获行 `undefined[key]` throw
进外层 catch → 每链启动即 `_imFail`、settle 全失 → 案 5 onload 永挂。修复 =
捕获行与 stale 闭包判空（`!_ep → false`）：对象缺位语义 = 从未有任何 run 被
取代。修复后 decode 目录 60P/0F/0T 与 s225 基线逐位一致，靶案保持 Pass。

## 验证

- 拼接 17 part（js_dom_bridge.rs 拼接序）→ node --check **SYNTAX_OK**（120735
  行；HEAD 拼接 120598 + 本刀净 +137）。
- 靶语料 solo **Pass**（修复前 1F；初版守卫与守卫修复后两轮均 Pass）。
- decode 目录 **60P/0F/0T 全绿**（s224/s225 面零回归——s226 动了 s224 纪元机制
  的 fetch 臂，该目录为必跑回归网）。日志 s226-decode-dir-nullsafe.log。
- the-img-element 目录级 **862P/75F/17T**（s225 基线 861P/76F/17T），953 键
  join 位移恰 **1 F→P**（靶案）**零 P→F** lost/gained 0。
  日志 s226-img-element-dir.log。
- embedded-content 全域 A/B（B = s225 基线日志）：**1078P/150F/30T**，共享键
  **1257/1257** 位移恰 **1 F→P**（靶案）零 P→F，Timeout 30→30 稳定。
  日志 s226-embedded-content-wip.log。
- zero-engine 全量 **3710P+0F** 持平（test-guard --compile-first 包裹）。
  日志 s226-zero-engine-full.log。
- 纯 JS 切片（+146/−9）fmt/clippy 不适用；release runner 重编译后跑靶（shim
  include_str 进二进制）。
- 回归证据链：s226-decode-dir.log（初版守卫 1T）→
  s226-decode-path-changes-repro.log（solo 复现 Timeout）→
  s226-decode-path-changes-noguard.log（去守卫 Pass，病灶隔离）→
  s226-decode-path-changes-nullsafe.log（修复后 0F/0T）。

## 合并树复验（rebase 并入并行流 e0d6948c0 F2 insertParagraph——part01 分片，与.part03/04/05 零文件交集）

release runner 重编译后四面复验：靶 solo **Pass** +
decode 目录 **60P/0F/0T** + embedded-content 全域 **1078P/150F/30T**（对 s225
基线 join 仍恰 1 F→P 零其他位移）+ zero-engine 全量 **3716P+0F**（基线 3710
+ 并行流新钉测 6，零失败）——并行提交零干扰。日志 evidence/logs/s226-merged-
source-relevant-mutations-solo.log + s226-merged-decode-dir.log +
s226-merged-embedded-content-wip.log + s226-merged-zero-engine-full.log。

## 承袭挂账（不动）

update-media 选源响应面余项（**source-relevant-mutations 1F 本轮收口并删**）；
not-broken 2F（renderer 流车道归因）；layout-engine 1px 钳（sizing.rs 渲染
流）；error.html 破图 alt 7F；sizes-auto Timeout 68 案；SVG data-URL intrinsic
3F + img naturalWidth SVG viewBox 族 22F（decoder 耦合 renderer 流 ③A）；
svg-img-density-vs-zoom 7F；lazy-loading 族 ~24F；part06
`_zwFieldsetDisableFocusReset`（s215）；part03 `_zwMElCheckValidity` FORM 短路
（s210）；onblur flake 族（s209~）；beforeinput.tentative；非 canvas 面一般渲染
门 FIXME(rendering)（s218）；flat tree slot FIXME(flat-tree)（s218）；modern API
img 树变更面余项（s221）；TreeMutated IMG 移除臂余项（s221——removeChild 文本锚
面本轮收口）；mimesniff 裸空白 value 理论缺口（s222）；image-alternate-no-store
known limitation（s225）；用户门禁三项（GB-20261031 customizable select 177F /
form-nameditem override builtins / form owner table 缓行族）。
