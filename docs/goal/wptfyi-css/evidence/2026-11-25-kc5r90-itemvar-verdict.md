# kc5r90 verdict——random-item() 选位感知 var 预写＋注册 custom 替换放行落表（2026-11-25）

## 结论

- **刀面双臂**：K1＝css-parser `rewrite_random_item_var_items`（css-values-5
  #funcdef-random-item：仅**选中项**不可解才 IACVT，未选中不可解项忽略）——整值
  单调用 `random-item(...)` 形逐节预写：key 节 var 先代入再判 key（spec
  substituted before being parsed；不可解 → 选位不可定整调用 fail-closed 返
  None）；item 节不可解以哨兵 `zw-item-unresolved-sentinel` 占位（槽位不变，
  选位数学不动；命名避 'random' 词根——`decl_value_has_random`/替换器快路径均为
  裸 windows(6) 子串扫描）。求值器 `eval_random_item_call` 的 `var(` 全拒保持
  fail-closed 兜底。K2＝style-system 4.7 循环 `--*` blanket skip 改注册放行：
  `registered_properties.contains_key(prop)` 进支路，先 `resolve_env_and_var`
  补 4.5 `--*` 整键跳过，替换值 **`Arc::make_mut` 落 step-4 gather 表**
  （`self.custom_properties`＝gCS 直读 :3686＋步骤 6 var 源 :3428；
  `resolved_cascaded` 的 `--*` 键无人消费＝死端）。
- **同轮门修订（v1→v2，r85/r86 先例）**：v1 bad3603b K2 写 `resolved_cascaded`
  死端——探针① b1≡b0 全五键 message 恒等（'0px'/''＝gCS substitution_view 拒
  raw 归 @property initial）直证；v2 60a045c0 落表修全绿。
- **judge/verify 双算 PASS，NET +8**（P 6709→6717）：刀内 6 键 F→P（预注册
  §一 表 5 行，:78/:79 同行两键——§五 '5' 系行计数转写，as-run 以 6 键全名集
  硬门）＋P2F=0＋GONE/NEW=0（9882/9882）＋非 random 页逐键恒等＋预注册
  G-EXTRA 双候选全兑现 report-only。

## F2P 账（6 刀内＋2 EXTRA）

| 键 | 形 | 臂 |
|---|---|---|
| unresolvable-argument 'Selecting a resolvable item…' | inline color `random-item(fixed 0.999, var(--missing), rgb(1,2,3))` | K1 |
| computed 'with a var()-derived <random-key> resolves' | `--length: random-item(var(--fixed-zero), 5px, 15px)` | K2＋K1(key var) |
| computed 'selecting var(--a)/var(--b) into a registered <length>' | item 含 var，resolve-after-selection | K2 |
| computed 'into a registered <length>/<number>' | 无 var 直替换 | K2 |
| EXTRA 'Nested in fixed value random() …' | raw 同串 → K2 异 ordinal 异值 | K2 |
| EXTRA 'random() in different custom properties … not shared' | not-equals 兑现 | K2 |

P 持形全稳：IACVT 黑（哨兵选中不写回→保原文→下游拒函数形）、fallback 通路、
dashed-ident/element-scoped 跨属性共享（`canonical_key_text` dashed 键不含属性
名→同 R 同选位）、empty-item、recompute stability；held-F 2 持（未注册
§9.4 raw）；写面 serialize 15＋valid 21 双臂全 P。

## 机制链与勘误

- 机制：4.5 var 站（`--*` 整键跳过）→ 4.7 random 站（K1 钩子＋K2 支路＋落表）
  → 4.85 简写第二趟（payload 具体化后 gate 不命中）→ 步骤 6/gCS 消费。
- §四 勘误：'random() shared by custom property in var()' b0 即 Fail
  （held-F 非 P），双臂恒 F 无 P2F；F 形根因留后续轮独立勘。
- 哨兵计数 3（r89 轮 8）：c1e98b9c7 语料 cron 换代后哨兵页 F 键 3→1＋ric 修订
  前缀 2 持；judge 脚本首版 G3 把 EXTRA 当硬门系脚本误写（预注册 report-only 在
  电池前），修正非门放宽。

## 证据链（.acceptance/wptfyi-css/kc5r90/ 本地）

NOTES-kc5r90 §零-§七（布防＋as-run 全账）／probe-r90.sh＋probe2 双臂 10 腿
RC=0／battery-r90.log（B0 e36e61f5 指纹=0／B1 60a045c0 指纹=1）／judge-r90.out
／verify-r90.out／bins/{r90-k1k2-runner 作废, r90-k1k2-v2-runner 落库}；
代账 kc5r89/bins/GENERATIONS.txt r90-k1k2＋r90-k1k2-v2 两行。

## 残面移交

1. 'random() shared by custom property in var()' held-F 根因勘（b0 形即红）；
2. random-item 嵌入形（非整值单调用）FIXME 不展开（语料缺席）；
3. margin computed 通道 6 位截断精度形（r89 移交，r91 候选）。
