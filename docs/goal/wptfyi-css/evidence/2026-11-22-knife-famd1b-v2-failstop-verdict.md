# 刀 FAM-D1-b v2 判定：P2F 红线 fail-stop（2026-11-22）

## 电池档案

- unit `zw-knife-famd1b-intake` 07:24:53 发射（SERIALIZE-WAIT 让位 stretch 电池至
  07:33:54 接窗）→ 08:04:35 EXIT rc=0，四语料×双臂八腿全 rc=0。
- face v2 patch md5 `909fd034c6283f4b7439485c977c2594`（单文件
  `crates/engine/src/animation.rs` +88/−12）；PRESTRIP 新步兑现（animation.rs
  树面 v1 face 先回 HEAD 再 apply-check）；FM/M2/S1/S3 门全常态：
  b0-runner `a7f5cb8f07135a84c250592cdb6d4f8a` / b1-runner
  `5ad8e217cf43ee94d836a42dab08658d`（B1 build 47.89s＝v2 patch release 编译验证过），
  ride 2 文件（abspos.rs stretch face、es_module.rs s419 留守）双臂零扰动。
- 产物 pristine 快照 `snapshot-knife-famd1b-intake-0805`（witness 留证，
  results-knife-famd1b-intake/ 同存）。

## RECONCILE 账面（页级）

| 语料 | 位移 |
|---|---|
| css-values calc-interpolation.html | P 23→45 **+22**（CSSA/CS/WAAPI 三车道 ∞/calc 形，INF/CALC 行全 listing fail→pass） |
| css-values zz-famd1-intake-probe.html | P 2→13 **+11** |
| css-sizing | 零位移 |
| css-animations Element-getAnimations.tentative.html | P 1→17 **+16** |
| css-animations | gained=16 lost=0 |
| css-transitions | gained=0 lost=0 |
| **TOTAL** | **gained=49 lost=0** |

F2P 主判据（A=∞ 案 INF listing、B=纯 px calc CALC listing、probe/getAnimations 面）
**全兑现**——增益面真实，fail-stop 由红线独立触发。

## 红线（VERDICT: FAIL-STOP）

1. **P2F-CS-REGRESSION**：`css-transitions/CSSTransition-effect.tentative.html ::
   After setting a transition's effect to null, style is updated`——b0 Pass →
   b1 Fail，msg `assert_equals: expected "100px" but got "0px"`。
2. ASYM 形态页 ×3（event-dispatch.tentative 双语料＋event-order.tentative）——
   b0≠b1 断言数，M4-a r2 先例同域（截断/时序域），挂起零折算，不影响定谳。

同页交换对称：`a new transition can be started` b0 Fail(150px) → b1 **Pass**——
净 Pass 数双臂恒 4/10，页内一翻红一翻绿。

## 判类：确定性 stale-value 真机制回归（非 flake）——双流独立收敛

witness 车道（`NOTES-knife-famd1b-witness.md`）分析链，本流 face diff 独立复核吻合：

1. 测试形：left 0px（cascade）→ transition 100s → left=100px（cascade 变）→
   `transition.effect = null` → 期望 gCS(left) 回落级联 100px。
2. face 前 `apply_single_property` 无 position 臂 → 动画覆盖**永不写 left** →
   effect null 后 gCS 恒见级联值（b0 Pass 的结构性原因，非引擎正确）。
3. face 补 `"left"|"top"|"right"|"bottom"` 臂（动画覆盖首写 left，t≤0 写
   from 0px）→ effect 移除路径**不清动画覆盖槽**（预存缺口，此前被无臂形态
   掩盖）→ from 0px 残留 → gCS 0px。

本流 solo 探针实证（famd1b 冻结 bin 单页双臂各 2 轮，CASE_WALL_S=30）：

| 探针 | P/10 | style-is-updated | new-transition-started |
|---|---|---|---|
| b0-r1 / b0-r2 | 4/10 恒 | **Pass** 恒 | Fail 恒 |
| b1-r1 / b1-r2 | 4/10 恒 | **Fail** 恒（0px） | Pass 恒 |

确定性复现（4/4 与电池窗一致），flake 排除；交换对称同样确定性＝
new-trans F→P 为同机制真增益（动画覆盖开始实际生效），非噪声。

## 处置

- v2 face **弃置不入账**（tally 维持不動）；face 留树作 D1-b2 基座（patch 文件
  不回贴）；battery/patch 工件留 `.acceptance/wptfyi-css/`（famd1b-intake-*.sh/
  patch/NOTES-witness/snapshot-0805）。
- tally 状态行 +13269 不动（fail-stop 无收账）。

## D1-b2 处方（witness 预注册，本流采纳）

- 修位＝effect 移除/取消/无 fill 结束路径**恢复被覆盖属性的级联/underlying 值**
  （animation.rs 效果清理段 restore 覆盖槽，非仅停止 tick）。
- 判据候选：CSSTransition-effect.tentative 全页零漂移（b0 形 4/10 且
  style-updated 持 Pass）＋本刀 49F 增益面保持（calc-interpolation +22、
  probe +11、Element-getAnimations +16）＋new-trans 真增益面保持。
- 池背景：values calc/∞ 插值族与 Anim/WAAPI keyframe apply 面缺口即本刀增益面；
  D1-c（% /em 语境叶求值）域保持拒收。
