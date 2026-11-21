# 刀 G1a 收账——interpolate-size 计算值存储＋gCS 读回（+2，GATE PASS 零 P2F）

日期：2026-11-21 23:25 发射–23:45 GATE-DONE · lane g1a-v2 · ZeroWeb-2 clone · 本流续开轮代发落库
域：css-values-5 §interpolate-size（`numeric-only | allow-keywords`）计算值承载与 CSSOM 回读
预注册：`.acceptance/wptfyi-css/NOTES-knife-g1a-interpolate-size-storage.md`

## face（四文件，+64 行；首发 22:07 CJK 注释 marker M3-FAIL rc=7，本流续开修为 `#[used] static` ASCII 字面（KA2_FACE_MARKER 先例）后重发）

1. `style-system/property/computed_style.rs`：`InterpolateSize` 枚举（NumericOnly 默认 | AllowKeywords）＋ `ComputedStyle.interpolate_size` 字段（非继承）＋ `as_str()` canonical 串。
2. `style-system/property/default_impl.rs`：Default NumericOnly。
3. `style-system/property/apply.rs`：`"interpolate-size"` apply 臂（allow-keywords 字面判定，其余 numeric-only；FACE-MARKER `#[used] static INTERPOLATE_SIZE_G1A_MARKER`）。
4. `engine/js_dom_bridge/computed_style.rs`：`"interpolate-size"` 串化臂 `as_str()`（FACE-MARKER `#[used] static INTERPOLATE_SIZE_G1A_BRIDGE_MARKER`）。
动画插值消费未接线——FIXME: 随 S2-symbolic/K-B 接通道。

## 电池 v2（log `knife-g1a-battery-v2-2325.log`，unit zw-knife-g1a-battery-v2）

- b0r build 43s / b1r build；patch md5 `abd2a646` 绑定（重生成后换绑）；M2 双臂 md5 异
  （b0r `7b6a2f7f…` 见 bin md5 行 / b1r `573aad26862215415e936eb3454ef320`）。
- **FACE-MARKER `INTERPOLATE-SIZE storage` b0r=0 / b1r=2 ✓**（22:07 首发同门 b1r=0 M3-FAIL——
  函数内 `let _ =` 静态被 release 优化剔除，`#[used]` 保 rodata）。
- RIDE：es_module.rs（s419 在飞）双臂同载相消；b1r 与 cwcv-v3 电池 b1 同 md5 互证。
- 八腿？三语料 ×双臂（css-values/css-sizing/css-variables）；RECONCILE 23:45:23。

## 判定（judge-knife-g1a-interpolate-size-judge.py rc=0）

- **GRAND F2P=3 / P2F=0**；主靶 `interpolate-size-computed.html` **2F→2P**（`Property
  interpolate-size value 'numeric-only'`＋`'allow-keywords'`——预注册恰集）。
- 第 3 例 `random-computed.tentative.html: Shared globally: random(--identifier, a, b)`＝
  **已知掷币 flake**（doc_seed=指针地址跨 run 摇号，settle NOTES random() 预研节在案；
  刀 H/KA2 判例同形剔除）→ 真增益 **+2**。
- 哨兵 css-variables drift=0 SET-IDENTICAL ✓；which-value 6F 维持 F（判别消费归
  S2-symbolic 挂账不变）；calc-size 七页零位移 ✓。
- **累计 tally +12966 → +12968**。
