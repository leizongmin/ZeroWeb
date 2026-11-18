# 刀 A1b：动画通道残面 R1–R3（2026-11-18 凌晨）

## 靶面

刀 A1 收口（e40480aa9）后 mask-position 页残留 20F 分解（b0=
results-knifen2-b1，bin `8833233e`）：

| 池 | 形 | 数 |
|---|---|---|
| R1 | Transitions 端点钳制 at(1) 采纳 raw 端点原文（initial/unset/mismatched ×2 方法） | 6 |
| R2 | WAAPI keyframe 端点 `inherit` 未解析父 computed（FIXME(kf-inherit-waapi)） | 7 |
| R3 | WAAPI 层数失配动画零覆盖产出（mismatched-layers 全 at） | 7 |

## 根因（三站三刀）

**R3（shim 单位门误杀层数失配）**：`part03.js` `_makeAnimation` 的
`_zwUnsafe` ④ 门按**整串单位 multiset** 比较帧端点——`'0px 0px, 10px 10px'`
（px,px,px,px）≠ `'20px 20px'`（px,px）→ 动画整体不登记（返回 0 无
entry），全 at 值域回落基值。CSSA 同形正常（真样式表通道无 shim 门）
实证 Rust apply 链（mix_mask_position LCM 展开、canonicalize、gCS 串化）
全无恙——唯 shim 门独病。探针 R2 二分四形状叠同元素：层计数相等（2→2/
3→3）正常产出、失配（1→2/2→1/2→3）全部零覆盖（残留前动画值）。
修：position 族逗号列表（mask/background/object-position）改比**逐层
单位种类集合**——顶层逗号分割（函数内逗号不算）逐层提单位 signature、
跨帧比较种类集合（忽略层数）：全纯 px 等同种类安全放行，真单位种类
失配（px↔% 层）维持 ④ 原判。无单位值（none/关键字）原 continue 保形。

**R2（WAAPI inherit 端点父链不可达）**：`start_animation_with_underlying`
的刀 K1 inherit 解析需要 `parent: Option<&ComputedStyle>`——js_anim
通道 `start_animation` 传 None → 'inherit' 回落 'initial' 同式解析
（FIXME(kf-inherit-waapi) 在册）→ from[inherit] 全 at 值域错（at(0)
got '0 % 0 %'，mids got ' ' 双垃圾）。修（shim 侧，注册时点改写帧端点）：
扫 `_zwSpec.kf[].p` 发现 'inherit' → 父 identity 解析 → 父该属性
computed 改写进帧值；父不可达/读值空保持原文（Rust 回落兜底零回归）。
父 identity 三层落定（探针 R2C 实证链）：
1. corpus 元素 **handle-only**（模板克隆/createElement 面）——
   `__zw_selector_for_handle` 反解空、wrapper `__zwSelector` 槽空，
   sel 链 `__zw_parent(sel)` 不可用；
2. 走 `_zwNodeParent[handle]` 父反链（R5181，MutationObserver childList
   added 批逐子镜像）——值是 **{parentSel, parentHandle, ...} 对象**，
   首版整对象串化 "[object Object]" 垫空（命中性失效根因），按字段提取；
3. `parentHandle` → `__zw_get_computed_style_by_handle` 读父 computed
  （handle 通道 gCS，与 sel 通道 `__zw_get_computed_style` 并存双形态）。

**R1（Transitions 端点钳制 echo 欠混合形）**：`transition_registry::query_core`
`eased == 1.0 → return None`（echo 级联 to，v3 A/B 定谳的端点序列化
形差防线）——initial/unset at(1) 期望 `calc(0% + 20px)` 系数形、
mismatched at(1) 期望 LCM 展开形，echo 级联形欠混合/展开位。WAAPI
同形 at(1) 已走插值臂实证同通道正确。修：例外清单 +mask-position
（rule-* 四属性与 rule-inset 八槽同款例外臂）——插值臂 t=1 产 mix 形
经 canonicalize 与期望对齐；纯 px 对插值臂 t=1 与 echo 恒等零扰动。

## 验证（corpus 单页探针，bin-knife-a1b 谱系）

| bin | md5 | 码面 | mask-position 页 |
|---|---|---|---|
| b0 | `8833233e` | 刀 A1 前在册 | 176P/196T |
| bin-knife-a1b v1 | `499b4f2e` | +R3 shim 门 | **183P**（+7 mismatched WAAPI 全翻） |
| v2 | `27317410` | +R1 例外臂 | **189P**（+6 Transitions at(1) 全翻） |
| v3 | `1689833c` | +R2 inherit（sel 链版未命中） | 189P（R2 三层根因排查中） |
| v4 | `2cddb000` | +R2 反链对象字段提取（终态） | **196P/196T 全清** |

跨页 P2F 抽查：background-position-interpolation 页 fa17c8dc（pre-A1b）
10P/186F = `2cddb000` 10P/186F **逐位一致**——R3 门放宽该页零位移
（bg-position 无插值臂，登记后离散回落 echo 与 B0 静态同值）。

## A/B 六腿账

电池 03:42:45–03:49:07（六腿 rc=1 全为 corpus 失败常态），b0=
results-knifen2-b1（`8833233e`，刀 A1 前在册）→ b1=results-knife-a1b-b1
（bin-knife-a1b 终态 `2cddb000`，03:41:16 构建）。ab-compare 全目录：

```
TOTAL: gained=2374 lost=1 regressed=1 pages_gain=153 pages_loss=1
```

**本刀通道净账（vs b0，五对照腿零容忍成立：masking/fonts/gaps/multicol/
shapes lost=0）**：

| 腿 | 增益 | 归属 |
|---|---|---|
| masking | +336 | 刀 A1 已账 +273（mask-position +166、mask-image +107）→ **本刀净 +63**：mask-position +20（R1–R3 三站全清）、clip-path +25、mask-border +21、mask-image −3（见核注） |
| gaps | +54 | column/row-rule-inset +22×2（页内翻面 22/22 全为 WAAPI from[inherit] 行）、row-rule-color/width +5×2 |
| multicol | +22 | column-count +5、column-rule-color +5、column-rule-width +5、column-width +7（7/7 WAAPI inherit） |
| shapes | +17 | shape-image-threshold +3、shape-margin +5、shape-outside +5、shape-outside-path +4（5/5 WAAPI inherit） |

合计 **+156**：mask-position R1/R3 靶面 + R2 inherit 父链反链改写的属性
无关跨页杠杆（gaps/multicol/shapes 抽页核验翻面行 100% 为 WAAPI
from[inherit]）。

**mask-image −3 核注**：`from [inherit] to [url]` WAAPI at(−0.3/0/0.3)
三行 vs 刀 A1 终态（fa17c8dc）Pass→Fail。harness 期望侧是静态元素
inline `style.maskImage='inherit'` 后读 gCS（interpolation-testcommon.js
:383-387）——本引擎静态 computed 的 'inherit' 关键字解析对 mask-image
返 'none'（父链有 `.parent{mask-image:url(blue)}` 应返 blue）。刀 A1 时
actual 侧（E5 initial 回落）与 expected 侧（静态缺陷）同错同值 'none'
恰一致故 Pass；本刀 R2 修 actual 侧为 spec 正确 blue（CSSA 链同证）后
两侧分歧 → Fail。根子在静态 cascade 关键字通道（mask-image initial/
inherit 臂缺失，刀 O 同款注册面形态），非本刀回归——vs b0 该三行恒
Fail，零容忍不受影响。

**他流账（不入本刀）**：
- fonts +856（font-valid +306、font-computed +232、system-fonts +192、
  shorthand-prevention +104 等）→ 刀 N/n3 流：font 简写/supports 提交
  e7e5a877d、3da1e3bbd 晚于 b0 bin 构建，静态序列化面本刀无通道可达。
- typed-om +96 与 lost=1（scrollbar-width.html 'thin' 关键词行
  34/34→33/34）→ knife-P 流窗口法证：part10 WIP（03:37:24 mtime，184+
  行在飞）早于电池 bin 03:41:16 构建被扫入；scrollbar-width 为静态
  关键词面，本刀改动无通道可达。

**外来码面处置**：transition_registry.rs 的 ZW_TRANS_DEBUG eprintln 块
（03:36:36 mtime，兄弟 fork 探针插桩）commit prep 时摘除恢复原
`return canonicalize(...)` 形，本文件 diff 纯净 +6；js_anim_registry.rs/
part01f.js/es_module.rs/part10 外来 WIP 一律不入本刀 pathspec。

## 收尾挂账

- R4（allow-discrete 54F，transition_behavior 字段建模）→ A1c 池不变。
- mask-image 静态 cascade 'inherit' 关键字臂缺失（刀 O 同款注册面形态，
  修则 −3 归零且两侧归位）→ A1c/关键字通道刀候选。
- bg-position 列表插值臂（引擎侧缺臂，R3 门放宽后登记但离散回落，
  background-position 页 10P/186F 与 pre-A1b 逐位一致）→ 后续刀候选。
- knife-P typed-om 面（~826F 预估翻面）并行流，互补不入本刀账；
  其 scrollbar-width 'thin' 一行回退由其 WIP 自带，非本刀面。
