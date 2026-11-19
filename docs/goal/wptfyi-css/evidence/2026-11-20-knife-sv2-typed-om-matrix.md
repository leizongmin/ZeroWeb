# 刀 SV2 typed-om 矩阵面：toMatrix + DOMMatrix 合成 + 相对单位守门（2026-11-20）

## Face 与落库形态

- face 两文件：`part10-typed-om.js` +193/−6、`part05.js` +4（translateSelf/scaleSelf
  可变族委托 rotate/translate/scale）。25 片 build 序拼接 ASSEMBLY-OK。
- part10 面：CSSMatrixComponent/DOMMatrixReadOnly 不可变方法族改合成形
  （translate/scale/skewX/skewY/rotateAxisAngle/rotate 以 `_zwOmMatMul16(this._m, 基阵)`
  与 this 合成，rotate 三轴 = rotateAxisAngle z→x→y 链）+ CSSTranslate/CSSPerspective
  `.toMatrix` 相对长度/百分比 TypeError 守门（`_zwOmAbsLen` 绝对长度 allowlist
  px/cm/mm/q/in/pt/pc，spec css-typed-om-1 §CSSTranslate.toMatrix/§CSSPerspective.toMatrix）
  + normalize var face（含 var() 声明解析降级可用）。
- 乘积序 FIXME(sv-compose)：`_zwOmMatMul16` 为 other×self 同族约定（part05
  `_domMatrixMultiply` 同索引形），语料期望侧与组件侧同族相消故 WPT A/B 恒一致；
  绝对序修正须与 part05 同步换序，不得单侧改序。

## 电池双跑（12 腿 6 语料×2 侧，knife-sv2-battery.sh）

- battery#1（03:49:21 发射 04:20:54 GATE-DONE rc=0）：gained=97 lost=3 regressed=4。
  typed-om 五页：normalize-tokens 6→16 (+10)、cssTransformComponent-toMatrix 0→8 (+8)、
  2d-flattening 0→4 (+4)、cssTransformValue-toMatrix 0→2 (+2)、relative-units 2→0 (−2)。
  b1 `2ea2adae` / b0 `84dd2e5f`（results 目录两次共用后被 battery#2 覆写，数字见本节）。
- battery#2（TypeError 根修后，04:45 发射窗 05:31:33 GATE-DONE rc=0）：b0 `8a3cbd78`
  / b1 `fee99384` M2 异 md5；gained=27 lost=15 regressed=15——typed-om 四页 +24 与
  battery#1 逐页恒（normalize +10/toMatrix +8/flattening +4/cssTransformValue +2 全
  F2P 零 lost），relative-units 零差行＝根修生效（b0/b1 双侧 2P：旧 FIXME 占位抛错
  退化为等价假过 → b1 真抛 TypeError，双侧同数）。
- **收账口径：typed-om +24**（双电池逐页稳定、跨 apply.rs census 态稳定）。
  css-animations 面差（battery#1 +76 / battery#2 −12）不转抄：同二进制内容跨电池
  P 摆动（Element-getAnimations 16↔1）＝动画时间线敏感页在多流 clone 并发负载下
  电池级 flake，且 animations 域主权已归刀 WA（`ea98cb8ae` 落库 +73）。2×2 因子
  （ride 新旧×face 有无）在并发负载下与噪声不可分辨，无判别力。

## P2F 法证与根修（battery#1 三页）

1. **relative-units −2 = face 真缺陷（已根修）**：组件 toMatrix 无相对单位守门
  （em 应 TypeError）；b0 的 2P 是旧 FIXME 占位抛错退化等价假过。根修后探针
  `probe-sv2-face-v2.js` 8 断言 ALL-PASS（含 TypeError 4 断言），battery#2 零差。
2. Element-getAnimations / animationevent-interface 两个 REG＝part06
  AnimationEvent.animation 机器缺口（b0 动画不真跑→事件路径退化假过，b1 动画
  物化→animationend 真派发触缺口）。180s 墙钟单页重跑定谳非超时。归让渡刀 WA
  （peer 已落 `ea98cb8ae` part05 AnimationEvent.animation 接口行 + 刀 WA2 计算值
  收口在飞）。
3. 哨兵零差：pva / fonts / shapes / transitions 两电池全无差行（刀 H var-reference
  fallback 消费面无 P2F）。

## 归因双锚点与 census 法证

- 双锚点：同电池 face-only 差（M2 md5 + patch strip/replay 往返）+ ffix2 独立
  faceless 电池数字与 b0 逐页同形（animations P=2/0/0、normalize P=6、toMatrix P=0）
  ＝增益随 face 走。
- census drift 两例均定谳零污染：battery#1 apply.rs（mtime 04:03:55，双 build 窗
  之后＝双二进制同源）、battery#2 part03.js（mtime 05:29:00，build 窗后腿段中点
  ＝peer 刀 WA2 在飞编辑）。ride census 17 文件显式清单（knife-sv2-ride-files.txt）。
- 树态法证：HEAD 移动（ea98cb8ae 触 part05 +4 AnimationEvent.animation、未触
  part10）后 worktree 两文件 diff 复核＝只含本刀 hunks（part05 blob f077b14b4
  与 HEAD 恒等＋本刀 +4；part10 HEAD 侧 7d688488a→HEAD 零变化），零抹除。

## 残面

- normalize-tokens 残 F、typed-om computed-opaque 大域 821F（top: scroll-padding/
  gap/cssRGB 各 28F）另刀；WAAPI getAnimations 对象桥归刀 WA/WA2 域。
- `_zwOmParseTransform` translate 分支静默丢第 3 参（FIXME 观测，非法 CSS 语法
  语料不达）。
