# M2 切片 357：插入臂 prepare 中止序对齐——remove-src-attr-prepare-a-script F→P

**日期**: 2026-11-09
**目标对**: `scripting-1/the-script-element/remove-src-attr-prepare-a-script.html`（s356 挂账残 10F 首选，src 属性族近邻）
**结果**: 单案 F→P；域 join det **F→P=1 / P→F=0 / T 双向 ∅**；语料口径 the-script-element **10F→9F、5T 不变**（probe 噪声剔除后 321 键）；残 9F 全部在册零新增红。

## 勘根定谳

1. **测试面**：createElement script → type='invalid'（data block）→ src=flag-setter.js → innerText 内联体 → `.append()` 入 body → 断言双旗标恒 false（didExecute/innerTextExecuted）→ type='' → removeAttribute('src') → 再断言双 false。spec 判据（pin 自 PR #10188 评注链）：插入期 prepare 在 **type 分类步中止**（invalid 非 JS MIME essence → return，无执行、不置 already-started）；后续 `type=''` 与摘 src **均非 prepare 触发面**——触发面只有三处（post-connection 插入、**src 属性变更且 value 非 null**、children changed），type 变更永不在列（curl spec 原文逐字核对）。
2. **红态定位**（自建 probe14 分相探针，域跑已删）：`assert_false: A2 innerText after insert expected false got true`——插入瞬间内联体已被 eval，与 src 摘除/type 置空无关（A3-A6 未及执行）。**根因不在触发面而在执行臂**。
3. **执行臂定位**：`.append()`/replaceChildren 共享插入通道 M2-S4P 臂（part05）classic 内联臂只有 `textContent 非空 + run-once 未置` 两道门——**无 type 分类门、无 src 属性门**。对照面：appendChild（R387/part04）与 insertBefore（s97/part04）臂在 049/066 corpus 形上不内联 eval（green 钉面）；probe15（classic+src+text 经 .append()）复现同红 → 现象钉死在 M2-S4P 臂。
4. **次生缺陷**：part01 s109 属性臂对 src 记录不区分 set/remove——摘除记录也触发 maybe-start-dynamic；type='' 后摘 src → classify=classic → src 已空落内联分支 → **同步 eval 体**（首修仅加 blocked 门后探针预演 A6 必红的静态推导）。spec：摘除 value=null 不触发。
5. **连带发现（同臂第三缺陷，随修）**：M2-S4P 臂对 src+体+classic 形内联 eval 且**抢先置 run-once**——把 s111 notify 臂的外链取回挡死（probe15 实证 didExecute 恒 false）。spec：src 分支优先，体忽略、外链取回。

## 施工（JS shim 双刀，Rust 零改动）

1. **part05 M2-S4P 臂**：module 委派臂后加两道门——①`classify==='blocked'` → continue（spec 中止序：无执行、不置 run-once）；②`hasAttribute('src')` → 委派 `_zwPrepareSrcScript`（gate 内 already-started/nomodule/for-event/空 src/module 臂完整重检——spec src 分支语义，与 R387b appendChild 面同口径）后 continue。无 src+classic+体 → 内联 eval 原样（create-script-set-location 判据面不变，solo 复验 Pass）。
2. **part01 s109 属性臂**：src/href/xlink:href 记录触发前加**写后属性在场门**——属性已不在场 = 摘除记录，跳过触发（spec value 非 null 条件的写后状态近似；setAttribute('src','') 空串仍触发 → 落 s356 空 src error 臂，语义保持）。

## Blast radius / 回归定谳

- 摘除触发抑制面 corpus 普查：removeAttribute('src') 形 4 页——049（摘除时 fetch 在飞，spec 语义 = 照常执行，green 保持）、066（clone 摘除后补体，摘除时刻体空 → 今日无观察差）、115（parser defer 摘除，already-started 已置 → 今日无观察差）、remove-src-attr（本切片目标）。049/066/115 经 execution-timing 全子域复验全 green。
- M2-S4P src 委派面：src+无体形（change-src-attr 插入相）今日走 s111 臂 fetch，改后走同 gate 委派（单点复用、run-once 幂等）——solo 复验 Pass；无 src+体内联 eval 面（create-script-set-location，browsers/history 域）零漂移——testharness-navigation solo Pass。
- probe15 async B1（didExecute=true）= **spec 真值**：外链取回已在飞，摘除不取消（049 同判据「removeAttribute( src ) before it runs」期望照跑一次）——探针诊断面非回归。
- s96 src IDL setter 直呼面、s95/s111 children-changed 臂、part06 写流 src 臂零接触。

## 验证

- **红态**：solo remove-src-attr `assert_false: expected false got true`（s356-b 域账）+ probe14 A2 分相定位（会话记录，探针已删）。
- **绿态 solo**：remove-src-attr Pass；probe14 双测全 Pass（blocked 面：插入跳过/type 置空无触发/摘除无触发/异步双零）；probe15 sync 全 Pass（classic 面：内联不 eval/摘除不触发）；change-src-attr Pass；create-script-set-location Pass（navigation 域）；execution-timing 全子域 147P/3F（红 = 052/143 长挂 + probe12 自建单列，精确既有集）。
- **域 join**（LC_ALL=C + (key,state) sort -u）：A=s356-b 320 键 vs B=s357-b 323 键——A-only 恰 1（remove-src-attr Fail）、B-only = remove-src-attr Pass + probe14 2P + probe15 1P+1F（探针噪声，commit 前删除）；**P→F=0、T→F=0、零意外位移**；残 9F = probe12/13 自建单列 2 + 143/052 长挂 2 + custom-element-exception + json-module parse-error-location + import.defer 三案。
- **shim 22 part 拼装 node --check SYNTAX-OK**（纯 JS 无 Rust 改动，clippy/fmt 不适用面）。
- **make test**：后台跑（结果见控制文档轮次账）。

## 挂账 / 下一步

- **s97 insertBefore 臂（part04 10324）与 R387 内联臂（part04 8552）同类缺口在册未修**：s97 无 blocked 门、两臂对 src+体+classic 形内联 eval（R387 因 fused read 形态在 049 corpus 面无观察差，green 钉面未动）——无 corpus 钉面不扩刀，语料扩展出钉面再收。
- custom-element-exception、json-module parse-error-location、import.defer 三案（tentative，独立切片）、143/052 流式物化长挂、probe12/13 自建单列、T=5 长尾（load-error-events-3、code-cache-base-url、dynamic-imports-credentials-setTimeout、instantiation-error-8、script-crossorigin-network）在册。

## 日志

- evidence/logs/s357-b-script-element.log（域跑 B 面，join 基线 s356-b-script-element.log）
