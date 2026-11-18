# s432：links 域 a/area target=_blank aux 臂施工（T 池 4 键全解挂，34 子测翻绿）

日期：2026-11-14 ｜ 车道：engine/dom（shim JS 三处）｜ 前序：s431 form
target=_blank aux 上下文（基建轮）

## 勘根定性（4 键 → 三层根因）

s431 移交的 stable-T 池 4 键（links/links-created-by-a-and-area-elements/
target_blank_implicit_noopener{,_base}.html + target_blank_useractivation.html
+ rel-attribute-clearing.html）全部 file 级 Timeout。三层根因递进揭破：

1. **裸 `opener` 标识符解析断**（implicit_noopener 双文件 26 子测挂的根因）。
   R206 合并 Function 形参表无 `opener`——子文档端点脚本裸名落宿主
   globalThis（主文档）→ undefined → 端点 `{ hasOpener: opener !== null }`
   ReferenceError 死于 postMessage 之前 → 零投递 → async_test 全 pending。
   s431 form 双胞胎端点用 `window.opener` 限定名（形参 `window` 已达）通过，
   分歧面恰在此。定性证据：zz-probe-aux2 探针实证 aux 端点脚本**已运行**
   （`__zwAuxEndpointRan===1` 主文档可见——aux 脚本 globalThis 即主 VM 全局）
   而 `typeof opener === "undefined"`。
2. **target/rel/URL 面无断**：勘根期插桩（后摘除）实证 `_zwAnchorActivate`
   `_blank` 臂触发、attr href 读出、URL 解析、flags 合成、`_zwLoadIframeEntry`
   全链无错（DBG=`T=_blank H=support/... ABS=https://... FL={"aux":true,
   "opener":null} LOADED`）——排除臂内断点，把疑点收敛到投递面。
3. **UA 指针 click 无锚激活默认动作**（useractivation mouse 臂挂的根因）。
   runner Actions 指针序合成的 click（`__zw_pointer_up_sequence` 两合成位）
   经 `__zw_dispatch_event` 只派事件不跑激活——页面 dispatchEvent/click()
   API 路径的激活臂（part04 R154/R106）不在宿主通道。zz-probe-c 探针实证：
   元素上 pointerdown/mousedown/mouseup/click 事件序全齐且冒泡、
   `Actions.send()` resolve，而 `_zwAnchorAuxSeq` 不增、aux 零投递。

**探针纪律**：勘根共 7 个探针文件（aux/aux2/bisect/endpoint/endpoint2/
probe-c/endpoint-c），全部 solo FILTER 跑 + 用后即删，未污染终验 sweep。
中途一轮 sweep 确有探针入库（join 出现 zz 键）——该轮只作勘根判读，终验
重跑干净链。bisect 探针渠道名错位（`?v=zz3a` → substring 剩 `v=zz3a`）
自伤一轮——探针自身也要对渠道名语义（memory s374 单 assert 启示）。

## 施工（三刀，全 shim JS，零 Rust 改动）

1. **part03.js `_zwAnchorActivate` `_blank` 臂**：读 target（own-expando
   优先 → attr 桥 → `<base>` target 回落）+ rel 语义矩阵（spec window
   opening steps——noopener 强制 = 显式 noopener/noreferrer 或无显式
   opener；noreferrer → referrer ''）。合成 aux entry 走
   `_zwLoadIframeEntry('__zwAuxAnchor:<seq>', _abs, flags)`；提交后消费
   瞬态激活并 return。static/IDL 双 href 形（attr 桥 latest-wins 同
   M2-S9 口径）。
2. **part01.js `_zwRunIframeScripts` aux 形参注入**：`flags.aux` 项向合并
   Function 形参表追加 `opener`/`close`（值 = win.opener/win.close 快照，
   与 finish 预置同门同序——先置后参）。非 aux iframe 形参表逐字节不变；
   `fn.call` → `fn.apply` 承载动态长度。
3. **part06.js `__zw_dispatch_event` 指针 click 锚激活臂**：派发尾
   （`_dispatchWithBubble` 后）`type==='click'` ∧ 未取消（ok）∧ 指针生成
   （pointerType 在场 ∧ pointerId ≠ −1）时 A/AREA 跑 `_zwAnchorActivate`。
   门设计：键盘合成 click（ENTER/SPACE 通道 pointerId −1，PE spec 非指针
   生成）与 webdriver 折叠 click（R108 宿主激活事务 pointerType null 走
   泛型分支，激活归宿主 `anchor_click_target` 管线）均不越界，无双激活。

**spec 锚**：HTML window opening steps（_blank noopener/noreferrer 缺省
强制）；run post-click activation steps（hyperlinked resource 导航 +
target=_blank aux 语义）；PE spec pointerId −1 = 非指针生成。

## 验证链

| 门 | 结果 |
|---|---|
| 25 part 拼装 node --check | OK（153,603 行首轮 / 153,625 行终稿含 part06 臂） |
| runner release 重建 | exit 0（1m08s / 1m39s 两轮） |
| solo probe-c（area IDL + mouse actions 双问探针） | area idl P + mouse actions P（修复后；修复前 mouse delivery=NO send=SENT ev=po,mo,mo,ck,body-ck SEQ 不增——定谳证据） |
| solo target_blank_useractivation | **4/4 P** exit 0（a.click/a.mouse/area.click/area.mouse 全绿；修复前 1P+3 pending 且 promise_test 顺序执行——a.mouse 挂阻塞后续两子测） |
| solo target_blank_implicit_noopener{,_base} + rel-attribute-clearing | exit 0 ×3（修复一刀后即绿） |
| 全域 sweep ×2 终验（探针清空后）+ stable join vs s431 组合态双基线 | **stable-P 8274→8308（+34 零消失，全落 links 域 13+13+4+4）/ stable-T 8→2（−6）/ stable-F 200→204（+4 簇 D 翻轮）/ 沾染 2→1（C-NEW 0）** |
| 归因 | +34 = implicit_noopener 13 + implicit_noopener_base 13 + rel-attribute-clearing 4 + useractivation 4（T 池 4 文件键全解挂）；T −6 = 4 目标文件键 + select-event/focus-dynamic 双轮翻绿（在册负载 flake 族 s292 判据，跨轮翻转不入账）；F +4 = customizable-select home-end 4 子测跨轮翻红（簇 D GB-20261031 在册，键盘时序面与 links 施工零交集）；C-GONE home-end.optional（同族翻轮） |
| **组合态整验（他流两笔合入后终稿树重跑）** | pull --rebase 合入 editing ip M2 壳族三臂（0ee05d644，part01f.js +208 行）+ css system-color（349a891ed，css-parser/computed_style/style-system）后：25 part 拼装 OK（153,829 行）+ release 重建 exit 0 + solo ×6 全绿（4 目标 + label-inside-anchor/rel-form-target 双哨兵）+ 全域 sweep ×2 join 对 s431 双基线——**stable-P 8308 逐键零漂移（+34 集合与预合并轮恒等）/ T −6 恒等 / F +1**：home-end 4 子测键本轮翻回 P、detailed.optional 文件键翻 stable-F、home-end.optional 文件键翻 T——簇 D 家族成员跨轮轮换（在册），与两笔到货及本切片零账面位移；沾染 2→0 |
| 探针残留 | 零（终验 sweep 无 zz 键，7 探针文件全删） |
| Rust 面 | 零改动（clippy/fmt 不适用；runner 重建用既有 Rust） |

## 判读

- s431 基建（aux entry 合成 + flags 预置通道）在本轮复用零改动——form/links
  双域同链验证基建口径正确。
- **裸标识符 vs 限定名**是 R206 形参遮蔽模型的系统性约束：子文档脚本裸名
  只达形参表内的名（window/self/parent/top/document/location/navigator/
  fetch 族/URL/caches/history + aux opener/close）。后续端点类勘根先核对
  端点读法（裸名即查形参表）。
- useractivation 4 子测的顺序挂起效应（promise_test 串行，首挂阻塞后续）
  是「3 pending」表象的成因——勘根时勿按子测独立归因。
- T 池 4 键孪生面（s431 下一步预期）全数兑现；dialog R-s31 簇/img
  density/embed sizing face 到货复验维持挂账。

## 下一步

s433 候选：红池分诊续（残余 stable-T 2 键 = 簇 D 家族；video P1+P2 或
簇 D A2 待门决策）；window.open 真弹窗链（s431 defer 项）评估轮。
