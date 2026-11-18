# M2 s426 复核账——styleerr 弃案复核轮（候选再勘根 → 维持 s414 弃案判决，
# 404/MIME 失败类区分构形评估后弃置）+ 池标签修正（防再推导）

- 日期: 2026-11-14；基线: HEAD d1545546b（PR #301 navigator 平面对象合并后，
  无新到货），驱动: CONTINUE 指令按既定流程续下一个施工候选。本轮零源码
  改动：池余标签「s164 FIXME（styleerr）」触发候选再勘根，独立重推到
  s414 证据链后维持弃案，并固化一层 s414 未覆盖的反再推导结论。

## 一、再勘根触发与重推链

- 触发: s424 归因收口枚举把 document-metadata/the-style-element/
  style-error-01.html 1T 标为「s164 FIXME（styleerr error 事件臂，记账不
  扩刀）」——该标签读作未裁决的范围决策；池空环境下按候选重开勘根。
- 独立重推（本轮）: ①corpus 实读——动态创建 style（createElement +
  appendChild）+ `@import "support/css-red.txt"`（text/plain，资产在册
  21B）断言 error；②同目录 style_events.html 2P（静态臂 s164：404 → 聚合
  error fire）、style-load-after-mutate.html 1P（动态臂 s162：load 已派，
  无 import 链）；③根路径 `/support/css-red.txt` 本仓缺失 →
  style-load-after-mutate 的 import 实为 404 网络失败类（与
  style-error-01 的 MIME 拒绝类不同类）；④part03.js:6782 FIXME 块 + 账本
  evidence/2026-11-12-m2s414-style-import-event-divergence.md 定谳链命中
  ——s414（2026-11-12）已完成同面深勘：wpt-metadata live 复核
  style-error-01 唯一条目 = crbug 1232491（**error 语义面无全绿引擎**，
  引擎分歧非缺口）、style-load-after-mutate 零条目（全引擎绿）；probe3b
  实证动态 error 臂可施工（d_err=1）但必翻转后者 → **弃实现**。

## 二、新增评估：404/MIME 失败类区分构形（本轮超越 s414 的一层）→ 弃置

- 构形: `_zwSsVisitCss` settle 值携带失败类（`!_ssR.ok` → `error:http`，
  judge MIME 拒绝 → `error`；消费方 `!== 'load'` 判式向后兼容）——动态臂
  按类分派：404 类不派事件（load 已由 s162 派，CSSOM import 失败语义），
  MIME 类派 error → style-load-after-mutate（404 类）保 load、
  style-error-01（MIME 类）翻 Pass，**两案可同时绿**——s414「两案互斥」
  在聚合 error 语义下成立，本构形绕开互斥。
- **判决: 弃置**。理由：①上游期望本身 stale——wpt-metadata 分歧记录在案
  （crbug 1232491），全引擎按 CSSOM 一致语义走（@import fetch 失败 → 规则
  styleSheet 置 null、父 sheet 照常 load 完成）；本构形为翻键引入**无引擎
  采用、spec 不定义**的特设事件规则——语料形合而规不合，与勘根修共享路径
  的车道纪律相悖（s392 navigation M3 排除域同款判决逻辑：不可赢/分歧面不
  施工）。②s414 再开锚（crbug 1232491 收敛或语料出现单案面）未触发，
  判决条件未变。③s426 记录此构形评估结论本身即为反再推导结论——后续
  session 不再重试该逃逸门。

## 三、判决与池标签修正

- **维持 s414 弃案**：style-error-01 1T = 引擎分歧面在册（非缺口、非互斥
  逃逸门可翻）， FIXME 注记与再开锚维持不变。
- 池标签修正（控制文档归因收口枚举）：「s164 FIXME（styleerr error 事件
  臂）」→「s414 分歧定谳弃案（上游分歧 crbug 1232491 + 两案互斥；s426
  复核维持，404/MIME 区分构形已评估弃置）」。
- 组合态漂移补记（上一轮已完成未落账）: PR #301 navigator 平面对象
  （engine shim part02.js 共享面）合并后 runner 重建 + s423/s424 收口面
  drift solo（button-click-submits 12/12、toBlob.jpeg/png 1/1）零漂移。

## 口径

- 域 corpus 计数零位移（零源码改动；style-error-01 1T 维持在册，标签
  迁移：候选→弃案定谳）。在册红维持 s424 终态口径 202F+13T / 201F+14T。
- 用户门维持：GB-20261031（簇 D）、GB-20261114-VIDEO（video P1/P2/P3）、
  s137（form-nameditem）。

## 下一步

- 与 s425 相同等待面：渲染流 listbox 形态 carve-out 到货 → engine
  act:option 臂施工；dialog R-s31 簇/img density/embed sizing face 到货
  复验；门决策后 video P1+P2 或簇 D A2 分诊续。本车道红池确认**全部**
  归域且无再推导空间（本轮为最后一个未定谳标签的关闭轮）。
