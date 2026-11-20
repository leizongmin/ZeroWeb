# EV · 刀 APC-2（computed 面 webkit 小写 w 读形 + CSSOM 写门三值）— 2026-11-20 rally-cont

靶面 = css/css-ui/appearance-cssom-001.html 残面 166F（APC 收账后 175P/341）。
预期满贯 341（+166 零 lost）。

## 机注定谳（zwprobe2 镜像 wpt-data 探针，CS-1b b1 bin＝含已提交 APC face）

### 指定面全通（PROBE-14/16/17/20/21/22/23 反证排除）

`setProperty('-webkit-appearance', v)` 后 IDL 双形读（WebkitAppearance/
webkitAppearance）、gPV、IDL setter 双形、invalid→removeAttr→valid 序列、
invalid 态直读——**全部正确**。_battery 消息里的 `value ""` 并非空串缺失，
而是 computed 断言复用同一 assert 描述串的误导。

### computed 面小写 w 单缺口（PROBE-24/25/26/27 定谳）

| 形态 | WebkitAppearance | webkitAppearance | gPV('-webkit-appearance') |
|---|---|---|---|
| valid 写后 gCS | 值 ✓ | **''✗** | 值 ✓ |
| invalid 写后 gCS | 'button'（initial）✓ | **''✗** | 'button' ✓ |
| 无写 initial gCS | 'button' ✓ | **''✗** | 'button' ✓ |
| unprefixed 写后 gCS | 值 ✓（别名回读通） | **''✗** | 值 ✓ |

根因＝part01.js gCS proxy **plain get 陷阱**用 `_camelToKebab(p)` 直查：
大写 W 形天然产 '-webkit-appearance' ✓；小写 w 形产 'webkit-appearance'
（丢前导 '-'）host 查询落空串。has/getPropertyValue 双 trap 已有
`/^webkit([A-Z])/` 特例（APC face 臂③），plain get 漏同款特例＝读写不对称。

### 失败簇映射（b1 JSON 期望数组 census 实证）

- x147 Invalid 环：期望数组 `["button"]`＝computed initial 断言（specified
  `[""]` 断言本已过——PROBE-15 佐证）。
- x9 Compat 环：期望数组 `["<value>", "auto"]`＝computed 断言（array 含 ""
  仍挂＝actual 非 空串而是 webkit 名 miss 的 ''——PROBE-24 形态）。
- x4 Values 环（none/auto/textfield/menulist-button＝页 values 全集 4 值）：
  computed 回读断言。
- x6 Invalid 环 `[""]`＝specified 断言：**值门误收** push-button/
  slider-horizontal/square-button（写入成功→specified 读回值∉[""]）；
  页 arrays 实证（values=4 值/compat=9 值/invalid=150 值，三值仅在
  invalid_values）→CSSOM 写门拒收零连带面。

## face 双臂（2 文件）

1. **part01.js gCS plain get webkit 特例**：`/^webkit([A-Z])(.*)$/` →
   '-webkit-' + lower + kebab（同 has/gPV trap 形；`--` 免归一、`_kpC`
   同源消费）。CSSOM-1 §camel-cased-attribute webkit 遗留形双 IDL 暴露。
2. **callbacks.rs `__zw_appearance_normalize_decl` 写门三值拒收**：
   push-button/slider-horizontal/square-button 返 "null"。parse_appearance/
   渲染管线保留（css-ui-4 compat-special 关键字，paint/text.rs 按钮绘制
   消费，AppearanceValue→AppearanceComputedValue 全链不动）；语料全集仅
   appearance-cssom-001 消费此三值（grep 实证）。

## 电池（zw-apc2-battery，r1）

FACE-MARKER 'APC-2 webkit-legacy-lowercase-get'（part01.js include_str!
ASCII 注释字节）；主靶 css-ui + 哨兵 masking/shapes/typed-om（typed-om
护对端 T3D/OM-1 新刀）。8 腿 b0/b1。

RECONCILE 18:34:41（8 腿 18:25:42-18:34:41，rc=1=含预期失败，产物完整）：

- css/css-ui/appearance-cssom-001.html: P 175->341（**+166 满贯**，341/341）
- css/css-ui/outline-017.html: P 1->0（−1，见下法证）
- **TOTAL gained=166 lost=1**；零 ASYM 页（双侧断言数全等，零 phantom gain）

判别：FACE-MARKER b0=0 b1=1（'APC-2 webkit-legacy-lowercase-get'）；M2 b0
md5 `b78081f2` ≠ b1 `4499a13c`；RIDE 1 文件（es_module.rs 对端 WIP）S1/S3
双点零扰动；哨兵 masking/shapes/typed-om 双向零位移（typed-om 护对端
T3D/OM-1 新刀）。

### outline-017 −1 solo 四象限法证（不入账）

同二进制 solo 复跑（archive bin + positional filter）：b0-solo 0/3、
b1-solo 0/3——`outline-offset is animated as a length` 两侧同报
`expected 2px but got 2.01px`。电池窗内 b0 恰好采样在 2px 前过线＝
load-window 动画计时 flake（2.01px=进度略过采样点），与 appearance CSSOM
面零机制交叠，双向不入账（CS-1b 先例）。
续开轮 8 轮交错复测补强（b0/b1 逐轮交替，消负载梯度）：b0 轨迹
0→1→1→0×6——r1 同 bin 复现与 b1 逐字节同签名 `2.01px` 形、r2/r3 同 bin
通过＝同二进制双向翻面实锤（b1 0/8 窗为电池余载期同环境态）；双侧
ASYM=0（3 subtests 恒）非完备性缺口。

## 收账

- **净 +166 零入账 lost 零 phantom**。归属全兑现：x160＝part01.js gCS
  proxy plain get webkit 小写 w 形映射臂（Invalid 147 + Compat 9 + Values 4
  的 computed 断言）；x6＝callbacks 写门拒收 push-button/slider-horizontal/
  square-button（specified [""] 断言）。与探针页级推算逐项一致。
- appearance-cssom-001 341/341 满贯＝CSSOM-1 §camel-cased-attribute 双 IDL
  暴露（WebkitAppearance + webkit 遗留小写形）+ css-ui-4 页判 invalid 三值
  CSSOM 写通道拒收（parse_appearance/渲染管线保留，写读分层见 face 注释）。
- master.md 账头见「刀 APC-2 收账」行（face 提交 + 收账同轮）。
