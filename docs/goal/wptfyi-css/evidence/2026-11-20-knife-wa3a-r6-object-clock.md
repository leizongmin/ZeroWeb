# 刀 WA-3a r4-r6 对象时钟面收账（2026-11-20 09:42）

## 收账结论

- **A/B 净 +2（gained=2 lost=0 regressed=0）**，face 落库 `61d17aa54`（part04.js +84/−30，唯一 pathspec；收账时原 hash `44b23ba11`，push 前两轮 rebase 换新）。
- 局部账累计 +7529 → **+7531**。

## 三 fork 级联（NOTES-knife-wa3b-prep.txt 仲裁笔为凭）

| 代 | 作者 | 内容 | 结局 |
|---|---|---|---|
| r4 | 他 fork WIP（08:1x 级联接管发射） | currentTime 年龄桥（__zw_performance_now − createdAt，**1ms** 阈值）+ _ctOvr seek 遮蔽 + cancel null + finished 年龄 settle | 电池净 −4：gCT twin 22P 面 P2F **−3** + getAnimations 两页 Timeout（−2）+ effect-null gain +1 |
| r5 | 他 fork 09:00:21 落树 | `_settle` 移入 Promise executor（修 `_rf` 词法域 ReferenceError 被 catch 吞 → 自然 finish 永挂 → 两页 Timeout） | 页挂死全消 |
| r6 | 本 fork | **16ms** 量化（＝引擎 `FIRST_FRAME_QUANTUM_S` 同帧量子，transition_registry.rs:633-655）+ startTime 显式值覆盖面 `_stOvr`（fresh null＝CSS-Animations §5.4.1 play-pending unresolved；set 读回保臂；杀 playbackRate setter 内联 play() 的 _startTime 残值） | twin P2F 全消 |

r4 的 1ms 阈值 P2F 机制：sync 断言路径实测漂移 1.372ms > 1ms 阈值 → `localTime of a new transition` / `localTime reflects playbackRate immediately` / `progress of a new transition` 三断言 hold-0 期望被打穿；part03 playbackRate setter 内联 play() 置 `_startTime≈2.4ms` 残值，`_zwEffComputedTiming` 的 localTime=currentTime−st 与遮蔽 currentTime 产生残差（part03.js:16121/:16251）。r6 双臂：16ms 量子保 sync 断言面（sync 漂移 ~1-2ms 落 0）+ startTime 覆盖面杀 st 残值。

## 电池 knife-wa3a-r6（12 腿 = 6 corpus × b0/b1）

- 发射链 /tmp/wa3a-r6-legs.sh（unit zw-wa3a-r6-legs，09:09:08 起，09:42:57 GATE-DONE）；主电池脚本 marker 门曾因注释空格形自拒（FACE-MARKER-R6-FAIL b0=0/b1=0 rc=7），bins 已建有效后续腿零重建。
- bins：b0=`3823f761fefbcb375a4d678b2f70726c` ≡ r4-b0（构建确定性互证）；b1=`5224a14ccc2abd4a8ad2c6a8317a98d1`。M2 异 md5 过。
- 产物：`.acceptance/wptfyi-css/results-knife-wa3a-r6/`（b0/b1 json+err、s1/s2/s3-md5、ride-files）。

### 主靶 css-transitions

- b0 P=1028 → b1 P=1028，**断言级 gained=0 lost=0**。
- 判据页零差异：gCT twin 3P b1 全 Pass（J1 ✓）；Document/Element-getAnimations 双侧 2P/5F、6P/1F 页全报告零 Timeout（J3 ✓，r5 生效），exclusion 断言双侧 Pass。
- OTHER 16 项＝timing 页报告集窗噪（event-dispatch ×12、animate-with-color-mix ×4 None↔Fail/Timeout），非 Pass 翻转，不入账。

### 哨兵

- css-animations：+2/−0（event-dispatch `Redundant change ×2` None→Pass）。
- css-fonts 6524→6524、css-properties-values-api 958→958、css-typed-om 12024→12024、css-shapes 6026→6026：**零 diff**。

### 窗噪底实证

r6-b0 ≡ r4-b0（同 md5）：transitions P=1028→1028 delta=0，噪底仅 event-dispatch ×4 None↔Fail。本窗 b0/b1 间 16 项 OTHER 同类，置信非 face 效应。

### S3-MISMATCH 仲裁（A/B 有效性）

S1→S2 干净（b0 构建窗）；S1→S3 唯 animation.rs 漂移（9314a8aa→c3f343d2）。mtime 法证：双腿二进制 09:07:19 前固化，animation.rs 漂移发生在 **09:30:07**（b1 transitions 腿窗中段，属主流撤回其 WIP，树现≡HEAD）——二进制已固化，**A/B 不受污染**。ride 集由 16 降 15。

## 挂账 WA-3b（非本刀域）

1. **effect-null interrupt 断言**（CSSTransition-effect `it should be possible to interrupt` got 0）：r6 16ms 量子下回 b0 形；r4 曾借 1ms 实龄 +1（与 −4 捆绑，不入账）。
2. **CSSTransition seek 通道**：_ctOvr 仅改对象属性不达引擎时钟；CSSTransition-currentTime seek 后 computed 值与 transitionend 越界面需引擎 start_s 重写或求值 offset 注入。
3. **cancel 分类轮询**：finished vs engine-cancel 按 elapsed 界判 + transitioncancel 派发（引擎 Cancel 事件不达 shim，pipeline/mod.rs:1437-1458 仅内部向量）。
4. **Document/Element.getAnimations 运行集**：`getAnimations for CSS Transitions` 等五断言 expected 2 got 0（b0 亦红预存，运行中过渡浮出面）。
5. persistent slot 镜像配方（part03.js:17105 起）已定位未实施。

## 残留风险与备注

- 中断测试自旋退出时 JS age ∈[13,16) 风险窗（JS 钟落后引擎纪元 1-3ms）：本窗未触发误杀（transitions 零 lost）；若后续窗现形，r7 降阈 ~8ms 备用。
- 面已含 r4/r5 内容（patch 自树生成），face commit 以 NOTES 仲裁笔归属三 fork 级联。

## 独立交叉核与 +2 转抄争议（6aa1ee8e 续开轮补录，收账 commit 竞态窗互盲）

时序法证：face `61d17aa54`（原 `44b23ba11` rebase 换新）09:45:11、本文档收账 `44eef6957`（原 `de95e5ff0`）09:49:23，与
CROSS-VERIFY NOTES 笔（09:49 落盘）竞态互盲——非协议违规。本节为收账后
独立复算（r6 b0/b1 JSON 失败消息逐条，非转抄），NOTES 有三点证明全笔。

- **transitions 三分量**：gCT twin 22P/22 双臂恒（16ms 量化保 hold-0 ✓）；
  dga/ega 双臂零 Timeout（settle 修复 ✓）；trans-eff interrupt b0=b1=4P
  同消息 `got disallowed value 0`——16ms 量子把 await ready 后读数（微任务
  级，实龄 ~1-2ms）量化归零；r5 探针 <1ms 阈值实证 5P、r4 age 桥电池实证
  +1，增益在 16ms 阈下结构性不可达。机制细化（挂账 WA-3b 第 1 条）：帧量子
  阈值无法区分「同任务 fresh 读」与「微任务 await 后读」，hold-0 判据应按
  §5.4.1 play-pending 状态驱动（ready 即决后 `Promise.resolve().then()`
  微任务清 _pending；pending=true→0、pending=false→实龄，双面无阈值，
  ~5 行 shim 改动＝r7 候选）。
- **净 +2 应为窗噪**：①同文判据自斥——transitions 侧 16 项 None↔Fail
  「不入账」+「噪底仅 event-dispatch ×4」，animations 侧 event-dispatch
  None→Pass 同类却入账；②r1 前例同页（`3313f4269` 窗噪段：该页双侧窗口
  幸运值定谳）；③负载窗双向压制——b0 腿 09:17:25-09:20:25 与 TEV face
  编辑+node 仿真 CPU 窗重叠（该页 idle 史 11-12P，本窗 b0=0P/b1=2P），
  face 零因果路径（AnimationEvent 派发页无 TransitionEvent 涉面）。
  修正目标：WAC 收账（`51c47d868`）提交文锚 +7697（7529 基弃 +2）但文件
  落 +7699（7531 基含 +2）——+2 现仍在账，争议未决；solo idle 双臂探针
  （b0 `45ea52fe` / b1 `e9ece0a4`，预期双臂 ~11P 恒 delta 0）排 zw-tev
  电池退场后出硬证，修正口径为锚 7699 → 7697。face 落库无异议（零回归 +
  settle/双臂机制资产归 WA-3b）。
