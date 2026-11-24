# kc5r84 verdict——wrap 链闭账四臂合并刀（F2P=6 精确命中／P2F=0）

日期：2026-11-24。face＝四文件单刀（双实例合并稿，NOTES-kc5r84.md §零 仲裁）：

- **A 面（3 文件）**：part03.js 守卫③ 裸关键字豁免 +column-fill（刀 E，:17099
  注释含 ASCII 标记 `kc5r84`，md5 6fdc3b4d）；animation.rs
  `apply_single_property` 委托臂 +column-wrap（:5383 扩表，eab8b131）；
  computed_style.rs gCS 串化臂 `"column-wrap" => column_wrap_str(...)`＋
  helper（Auto/Wrap/Nowrap）＋use 补 ColumnWrapComputedValue（c78e254c）。
- **B 面（1 文件）**：transition_registry.rs `register_from_style_change`
  `columns` 简写位且 allow_discrete 时 governing 追加 reset-only
  `"column-wrap"`（同层位/时长/延迟/timing；plain 位不扇出，
  css-transitions-1 §starting-of-transitions）＋`#[used] static
  KC5R84_FACE_MARKER` 指纹（r82 形制承 1a68c9fbc，1cc0a6a0）。

bins：b0＝r84-b0-runner **72b3a4d4**（HEAD 3829434b7 纯基线，strings 'kc5r84'
=0 双门）、b1＝r84-b1-runner **791e5bfa**（四文件 face，strings 指纹 1/3 门＋
Compiling zero-engine=1）。构建链：实例 B 首版 E0382（`tf` 主 insert move 后
columns 分支复用）→主 insert `tf.clone()` 修复＝merged 面唯一码面修正（NOTES
§零）。

## 根因（四臂清单缺口，前证定谳）

1. **gCS 串化缺臂**：`serialize_computed_property_impl` 无 column-wrap 臂 →
   CSSA/WAAPI 通道 gCS 恒 '' ＝ discrete-no-interpolation 42 wrap 键 vacuous。
2. **动画写回缺臂**：`apply_single_property` 尾 `_ => {}` → 翻转输出静默丢弃。
3. **shim 守卫③ 独病**：column-fill 裸 'auto' 端点中 regex 不在豁免表 →
   WAAPI 不注册 → fill initial→auto t≥0.5 恒 underlying 'balance'（r83 残 4F；
   同页 rule-style/span 不中 regex 全 Pass＝门独病法证形同刀 C/D）。
4. **简写扇出缺 reset-only 长手**：`transition-property: columns` 按简写计算值
   差分登记——reset 页 columns 前后同值 → 跳过；css-multicol-2 §4.5 columns
   展开含 reset-only column-wrap（不入简写计算值串），静态级联复位 nowrap→auto
   独立变化无过渡管辖 → start 侧直接跳终值。

静态写面（apply_advanced :2260 解析臂）、inherit 臂（inherit.rs）、简写扇出计算
值层（shorthand/mod.rs）已在；插值离散翻转 t≥0.5→to 既有（css-values-4 §8.5.6）。

## 探针门（发射前，probe-results/ 16:46 实证）

- discrete-no-interpolation：b1 168/168 全 Pass（42 wrap 键 un-vacuate 无翻面
  ＝P2F 探针先证零险；fill 组 7/7——b0 形 3P+4F＝t<0.5 两侧同 'balance' 偶合
  Pass，r83 账只记 4F 侧修正）。
- column-wrap-reset-interpolation：b0 Fail / b1 Pass（B 面扇出登记实证）。
- columns-shorthand-reset-wrap：b0 3F → b1 恰 1P/2F（multicol3 翻；multicol1
  败在 `rule.style.columnWrap` 首断言＝CSSOM 声明块 getter 对简写-only 块不物化
  column-wrap，gCS 臂不可达；multicol2 声明块级简写复位缺——两者摘除出本刀
  预测，归 r85 CSSOM 声明块 face）。

## 电池与 judge（六腿全 RC=0，16:47:30–17:10:33；systemd transient
MemoryMax=24G/RuntimeMaxSec=7200，flock /tmp/zw-kc5r84-battery.lock，JSON_DIR
唯一写者；发射行树面 FACE-REGISTRY/FACE-PART03 md5 与收账时逐字恒等）

- **G2 P2F=0** 三 corpus（fail-stop 通过）。
- **G3 F2P=6＝predicted 6 精确命中**（窗 ±4），页分布 4/1/1 全对：
  discrete-no-interpolation 4（fill WAAPI at 0.5/0.6/1/1.5）＋
  columns-shorthand-reset-wrap 1（multicol3）＋column-wrap-reset-interpolation 1
  （Animate and reset column-wrap）；n=1613 P→P=1400。
- **G1** 六键恰预注册清单，无 OTHER 页。
- **G4** at (0.5) 边界键 1（informational）。
- **G5 values** n=9882 恒等 9881；唯一漂移 random-computed「Shared globally:
  random(--identifier, a, b)」b0 Pass→b1 Fail＝双稳态采样键第五代（r81c F/F、
  r82 P/P、r83 b0F→b1P、r84 b0P→b1F）；双臂 3× 复核（probe-random-r84.sh 六
  快照 probe-results/random-recheck/）b0=F/P/P、b1=F/F/P——**双臂各自跨次翻面
  ＝双稳态坐实非 bin 相关**（预注册分支 1 命中）＝SAMPLING-FLAKE 归档。
- **G5 transitions** n=1506 hard-P2F=0；truncation-asym=22（event-dispatch 20＋
  animate-with-color-mix 2＝r83 truncation-asym 族页集复现）按 r83 条款归档；
  F2P=0。

judge-r84.py **VERDICT PASS**（judge-r84.out 在档；本 session 独立键级直算
multicol/values/transitions 三腿与 judge 输出逐字吻合＝双算对账）。

## 残面（r85 候选）

1. **CSSOM 声明块 face**：`rule.style.columnWrap` getter 对简写-only 声明块不
   物化 column-wrap（multicol1）＋声明块级 `columns` 简写复位不覆写同块在前
   `column-wrap`（multicol2）——两键同域。
2. 缺语料页 column-wrap-interpolation.html 补页（'wrap'/'nowrap' 端点组；
   守卫③ 豁免表若扩需另轮预注册）。
3. neutral at==endTime 精确边界残面（r83 移交，独立非本刀域）。

链证据：.acceptance/wptfyi-css/kc5r84/（NOTES 全链＋bins＋results-b0/b1＋
judge-r84.out＋probe-results/random-recheck 六快照）。
