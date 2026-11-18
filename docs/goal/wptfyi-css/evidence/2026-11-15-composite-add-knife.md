# composite-add 刀——shim 端点预合成 + 引擎字族/sizing apply 臂（cadd knife）

2026-11-15。接 [jsanim gcs value 刀](2026-11-15-jsanim-gcs-value-knife.md) v2 挂账的
「composite add/accumulate 建模」：v1 曾把 composite≠replace 整体不登记（245 REG 全量
归因中 add/accumulate 107 条），v2/v3 守卫保位后合成插值面全折（fonts -180 等）。本刀
把 add/accumulate 语义在 shim 侧建模掉：注册前把 add 帧端点与 underlying 预合成，按
replace 语义登记，引擎零合成器改动即可插值。

## 机制要点

- **预合成恒等**：add 语义 = 动画值与 underlying 逐属性相加（Web Animations §5.4.4
  combining values；单动画单迭代 accumulate 与 add 同形）。harness 面先 inline 写
  underlying 再 animate、paused+seek 下 underlying 恒定，故
  `lerp(U⊕F, U⊕T, p) ≡ U ⊕ lerp(F, T, p)`（U 恒定时数学恒等）——shim 把
  `endpoint' = underlying ⊕ endpoint` 算好按 replace 登记，引擎线性插值直接覆盖。
- **可加值域（shim `_zwAnimParseAddVal`）**：`<number>` / px / `<angle>`
  (deg/grad/rad/turn) 单 token 与 font-size-adjust `<metric> <number>` 同 metric 对。
  `%` 拒（引擎无 % 插值臂，登记会以 0px 垃圾值换 B0 fail）；混合单位/calc/关键字/
  函数列表（transform MULTI 形）拒 → 整动画 all-or-nothing 回落 B0。
- **伪通过池自然过滤**：underlying 经 gCS 读取——引擎无序列化臂的属性（border-*-width
  族）读到空串解析失败不登记。v1 的 80 条 border-*-width REG 对应的 B0 双侧空串恒等
  伪通过池由此保位（零改动）。
- **underlying ⊕ 运算**：同单位相加；无单位 0 并入任意单位侧（css-values-4
  `<length> 0` 等价）；metric 失配拒。帧级 composite 覆盖 options 级；变换帧删
  composite 键；非法 composite 串维持不登记。仅改写注册 spec——KeyframeEffect 持
  原始 keyframes，getKeyframes()/effect JS 对象面零变化。
- **引擎 apply 臂（animation.rs `apply_single_property`）**：新增 font-size /
  font-weight / font-size-adjust / letter-spacing / word-spacing / text-indent /
  min/max-width/height 十臂，严格解析（失败不动样式 → canonicalize 恒等返 None →
  B0 回落，零回归方向）。插值臂补 font-weight（数值钳 [1,1000]，css-fonts-4）、
  font-size-adjust（同 metric 数值 lerp）、text-indent + min/max-sizing（px 严格，
  关键字端点回落离散 css-values-4 §8.5.6）。
- **既有护栏零改动**：canonicalize None 保 echo、fallback 检测器捕开区间离散、
  守卫 ②calc-size/random ③裸关键字 ④混合单位端点——全部在变换后帧上照常运行。

## A/B 定谳（B0 = bin-jsanim-b3 快照 v3，B1 = bin-cadd-b1 快照本刀）

五电池全量（B0 复用 results-jsanim-b3 存量，B1 = results-cadd-b1）：

**+75 gained / 0 lost / 0 regressed（7 页双边显式转移）**：
font-size-adjust-composition 28→59（+31，31/31 纯 add 面）、font-size-adjust-
interpolation 78→94（+16）、font-size-composition 0→6、font-size-interpolation-001
8→16、font-size-interpolation-003 4→8、font-weight-composition 0→5、
rotate-composition 4→9（+5）。motion/backgrounds/values 与 B0 逐字节持平
（values 其合成面 = border-*-width 伪通过池 + calc-size 守卫域，正确保位）。

16 组定向 `CSS_CORPUS_ONLY_SUBSTR='composition'` 双腿（results-cadd-b0-comp /
results-cadd-b1-comp）：

**+197 gained / 0 lost / 0 regressed（17 页 0→12 直翻）**：margin-*/padding-* 8 页
（css-box）、width/height/min/max-width/height 6 页（css-sizing）、letter-spacing/
word-spacing 各 12（css-text）、text-indent 0→5（css-text）。

**合计 +272 / 0 / 0**。master.md 前估 composite-add ~146G，实收 272G——定向 16 组
扫出的 box/sizing/text 面超原估（原估基于五电池子集）。

## 过程记录

- Phase 2 首跑事故：批脚本循环变量名 `GROUPS` 撞 bash 特殊只读数组（用户组 ID 列表），
  赋值被静默忽略 → `$GROUPS` 展开为 "1000" 单组 → corpus 脚本 usage exit 2。改名
  `_zwGroups` 重跑 32 腿全绿。教训：批脚本变量名避开 bash 特殊名（GROUPS/SECONDS/
  LINENO/FUNCNAME 等）。
- `cases=` 汇总差异（transforms b3 113 vs b1 107）系重试计数口径，页级 JSON 集合
  ab-compare 双边相等（B0-Pass 子测零 REGRESSED 证）——库存以页级 JSON 为准（v3
  同款教训再证）。
- provenance：bin-cadd-b1 快照（`strings` 验 `_zwComposeAddKeyframes`/`_zwAnimParseAddVal`
  2 处命中，B0 快照 0 处）；全程无中途重建。
- 共享树注意：本刀 A/B 与门禁期间，script-sandbox 流（s419 导入绑定 holder 修
  make-test SW 红）在同树 es_module.rs 有在途 WIP——提交走显式 pathspec，门禁红在
  script-sandbox 域按账本四分类归因，不混入本刀。

## 验证

- rustfmt --check 绿；`cargo clippy -p zero-engine --all-targets -- -D warnings` 绿。
- `cargo test -p zero-engine animation::` 97/97 绿（test-guard --compile-first 包裹）。
- corpus A/B 五电池 + 16 组定向全零回归（上节）。
- make test 门禁（zw-cadd-mkt2 workspace 腿 + zw-cadd-legs 剩余九腿单元，均
  test-guard/systemd 包裹）：workspace 本刀域全绿；剩余腿绿四红五全对号在册红账，
  browser tests 6F 为 mtime tripwire 伪红、quickjs crates 47F 为 s419 WIP quickjs
  形扩面（均不入本刀账）——细目见控制面本轮剩余腿账。
- spec 链接：https://drafts.csswg.org/web-animations-1/ （§5.4.4 combining values /
  §5.4.15 computed keyframes）、https://drafts.csswg.org/css-fonts-4/#font-weight-prop、
  https://drafts.csswg.org/css-values-4/ §8.5.6。
