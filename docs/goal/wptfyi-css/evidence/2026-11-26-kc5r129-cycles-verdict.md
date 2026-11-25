# kc5r129 verdict——function-cycles 刀：依赖图环四修（2026-11-26）

## 结论

- **刀域**（css-variables-1 §3.4 cycles「If there is a cycle in the
  dependency graph, all of the properties in the cycle are invalid at
  computed-value time」×css-values-5 §cyclic-substitution-contexts；判据页
  dashed-function-cycles.html 4 Fail 键）：**style-system custom_function.rs
  会话域单文件——元素级管线零扩散**。
  1. **修 A（KEY1「Observing property cycle locally」）**：会话入口
     incoming 依赖图环分析→环成员预置 `SCOPE_IACVT` 遮蔽标记（影子绑定
     `let incoming = &incoming_eff` 一次改线全部下游视图——参数部分作用域/
     三层视图/inherit 父帧/缺省域）。根因＝元素级 gather 表保留环原文
     （`gather_custom_properties` 纯级联聚合无环检测），会话 var 代入撞
     交替环 → Err → 消费声明 invalid；预置标记后标记分支
     （computed.rs:559「标记键按缺席处置」）→ 消费侧 fallback 生效。
  2. **修 B（KEY2「Using cyclic values with no fallback」）**：局部声明
     `final_raw` 环预置——成员跳过求值直接落 iacvt 形账（raw_scope 留
     原文＋scope_out 遮蔽标记，与循环内逐条求值的既有 Err 臂同形）。根因
     ＝环命中 fallback 破环通道（`resolve_var_reference_at` 环命中侧
     fallback 解出成功→cycle_hit 清除）产出伪值「3 1」；环成员资格先于
     fallback 逃逸。
  3. **修 C'（KEY3「Cycle through local, other function, fallback in
     function」）**：`substitute_dashed_functions` **产物重扫臂** Cycle→
     本调用 Invalid 收口（环不再整链上抛）——内层调用 invalid → 外层局部
     按 Invalid 臂落遮蔽标记 → 消费侧 fallback 生效。检出点在实参替换/
     局部展开/预检的保持整链传播（:694 局部臂 Err 不动——「Cycle through
     unused local」判据要求 result 字面 FAIL-result 不外泄，改局部臂必
     P2F）。
  4. **修 D（KEY4「Function in a cycle with its own default」）**：参数
     缺省补嵌套 dashed 调用展开（`expand_default_dashed_calls`——调用栈
     push 态传缺省展开，栈上再入即检出）→ `Ok(None)` 整调用 invalid。
     根因＝缺省值此前不展开（"--f(13px)" 字面绑定 --y，result 10px 不
     引用 → 调用成功）。三缺省臂（IACVT 实参/缺席实参/typed 强制失败）
     同形接线；嵌套 Invalid → 参数值遮蔽标记（同 IACVT 实参口径）。
  5. **环分析器 `detect_var_reference_cycles`**（会话域纯文本函数）：主
     引用位建边、**fallback 子树整段排除＝惰性求值语义**——「Local with
     self-cycle in unused fallback」守成键要求 fallback 不参与环成员资格
     （主链可解即非环；若静态含 fallback 边则该键 FAIL-result 泄漏 P2F）。
     DFS 三色＋显式灰栈，回边命中灰节点→灰栈自命中点至栈顶整段记成员
     （自引用环＝单元素段）。
- **电池代**：b1＝gen22 `r129-cycles-runner`（md5
  4da2ff918e70f2574a9a74f06aaa0967，FACE=1＋CARRY-K128=1 双指纹独立重核）；
  b0＝k128 归档 bin `r128-fnlayer-runner`（md5 db8cb4d1，ZW-KC5R128-
  FNLAYER-FACE=1）洁净锚。
- **judge VERDICT PASS＋verify VERIFY-PASS**＝预注册通过：
  - G1 双腿 RC=0（b0 00:47:59 / b1 00:48:12 各 13s）。
  - G5 键 presence 521==521 GONE=0 NEW=0；pages 43==43。
  - G2 P2F=0（豁免集空）；held-F 显式 10 键两臂恒 Fail；红线 4 键恒
    Pass（mixin-cycle k112 判例＝本刀环语义域重点守成锚）。
  - G3 F2P observed=4 knife=4 out-of-registry=0 missing=0（registry 程序化
    生成防转写漂移）。
  - G4 SET-IDENTICAL 非豁免页 diff=0。
  - NET css-mixins **P 476 → 480 = +4**（b0 census {Pass 476/Fail 44/
    Timeout 1}＝k128 b1 终态净值 pristine 精确复现；b1 {Pass 480/Fail 40/
    Timeout 1}）；净恒等式 +4 == F2P − P2F == 4 − 0。
- **F2P=4 全落 dashed-function-cycles.html**（22 判据 template 中 4 Fail
  全翻，18 守成键零扰）。刀前失败形态：KEY1 Invalid（交替环 Err）／KEY2
  「3 1」（fallback 破环伪值）／KEY3 Cycle（整链上抛杀外层 fallback）／
  KEY4「10px」（缺省字面绑定再入不检出）。
- **P2F 面防核**：修 A/B 会话域表克隆零元素级扩散（css-variables 套语料
  不在本套语料，元素级环判缺位维持现状＝已知 FIXME 域）；修 C' 收口语义
  对元素级 var(--tmp, PASS) 消费两语义同向（Cycle 与 Invalid 都落
  IACVT→fallback——6 个环经 --tmp 判据键全恒 Pass 实证）；修 D 展开缺省
  全语料唯 KEY4 一形（grep 证零其他缺省-dashed 形）；「Cyclic defaults」
  var 互指缺省环走既有 marker 链零扰（钉测守成）。
- **钉测**：style-system 新钉 8＋改钉 2＝**2731P/0F**——新钉
  incoming_property_cycle_shadows_with_fallback（KEY1）/
  local_var_ring_without_fallback_is_invalid（KEY2）/
  unused_fallback_self_reference_is_not_a_cycle（惰性边守成）/
  local_self_cycle_uses_fallback／cross_function_local_cycle_falls_back
  （KEY3）／cycle_through_unused_local_propagates_cycle（:694 局部臂整链
  传播守成）／default_reentry_cycle_invalidates_call（KEY4）/
  cyclic_var_defaults_still_shadow（「Cyclic defaults」守成）；改钉
  cyclic_call_contained_as_invalid（k112 自引环：Cycle→Invalid）＋
  function_recursion_cycle_poisons_declaration（k112 T12：断言
  Cycle→Invalid——收口语义，消费声明 IACVT 同向）。
- **门禁**＝css-parser 3415P/0F＋style-system **2731P/0F**（2723 基＋8
  新钉）＋engine 4310P/3F＝既有对端红名单恒定（dialog_focus_s330/
  expando_r3046/p99s_inline_custom_property——k127/k128 账面同名恒定，
  k128 stash 归因定谳非本流引入；本刀 engine 侧仅 #[used] 指纹静态）＋
  clippy RC=0＋fmt RC=0。
- **挂账**：元素级 gather/compute 管线 css-variables-1 §3.4 环判缺位
  （本刀会话域兜住函数消费路径；非函数元素级环语义维持现状——影响
  css-variables 套计数，独立刀评估）；字符串字面内 var( 形似体文本级
  扫描（has_inherit_call 同口径 FIXME——tokenizer 级收严）。
- 判据页 <https://drafts.csswg.org/css-variables-1/#cycles> ＋
  <https://drafts.csswg.org/css-values-5/#cyclic-substitution-contexts>。

## 证据链

- 探针：probe-b1-full census {Pass 480/Fail 40/Timeout 1}（43 页 521 键）；
  registry 差分 GONE=0 NEW=0 F2P=4 P2F=0（f2p-registry-k129.py）。
- 电池：battery-k129.log LEG b0 RC=0 / LEG b1 RC=0；B0=db8cb4d1（FACE-K128=1）
  B1=4da2ff91（FACE-K129=1 CARRY-K128=1）。
- judge：G1..G5 全门 PASS；verify：(4,4,0) 关系预测逐位＋净恒等式。
- 工件：/tmp/zw-k129/（gates/build/probe/battery 四脚本＋judge/verify/
  registry＋logs）；归档 bin .acceptance/wptfyi-css/k129/bins/
  r129-cycles-runner；电池结果 .acceptance/wptfyi-css/k129/
  results-k129-b{0,1}/。
