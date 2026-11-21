# 刀 MATHFN-LENPERC 判定：A/B 零位移 no-op——face 不入库（+0，P2F=0 穢无扰动）

日期：2026-11-21 23:54 发射–00:05:04 GATE-DONE（r4）· ZeroWeb-2 clone · patch-only
预注册：`.acceptance/wptfyi-css/NOTES-knife-mathfn-lenperc-claim.md`（预注册 +22-24 判空）

## 电池时间线

- r3 23:49 首发即死 rc=5 FM-FAIL：marker 写成 `//` 注释行——注释不进 rodata，
  strings 恒 0（G1a M3-FAIL 同谱系教训的第二形态：不光要 ASCII，必须在代码字面）。
- 修复＝模块级 `#[used] static MATHFN_LENPERC_FACE_MARKER`（types.rs 3292 行
  MATHFN_FACE_MARKER 同款先例），patch 重生成 md5 `04b8adb1` 换绑。
- r4 23:54 发射：M2 双臂 md5 异 ✓（b0 `2bb02b1a…`/b1 `ab819c395fc398eeafb72e06e519295f`）、
  **FACE-MARKER b0=0/b1=1 ✓**、S1/S3 ride 零扰动（es_module s419 相消），
  四腿（values/sizing × b0/b1）rc=1 常态，00:05:04 RECONCILE。

## 判定：gained=0 / lost=0，双臂逐案字节级全等

- values b0/b1 pass 6420/9874 全等；sizing 3240/5709 全等；五 failing 页
  （minmax serialize/used-value 族 20+29+10+12+21=92F）fail 集合逐案相同。
- face 确定烙入（FM+M2 双证）→ 零位移＝**行为 no-op**，非构建/ census 伪影。

## 根因（读码 00:0x）

style-system 六臂（width/height/min-/max- ×2，apply.rs:221-270）统一走
`parse_length_or_math`（apply.rs:36）——parse 失败后本就有 parse_math_function
兜底，产出同为 `LengthValue::Calc`。本 face 在 parse_length 内加 math 臂与该
兜底恒等 → 解析层本不缺 math。**预注册把缺口定错层**。

## 真实缺口定位（follow-up 刀 MATHFN-SERLAP 预研，fail message 三形）

1. computed 串化保形：`'min(20px, 10%)' as a computed value should serialize as
   'min(20px, 10%)'`——gCS computed 通道对 Calc 槽须按 css-values-4 §math
   序列化保 min()/max() 函数原形（现役 length() 直通解析/calc 形）。
2. used-value 串化：`'min(1% + 1px)' as a used value should serialize as '2px'`
   ——used 通道全解析（% 基 100px 语境）。
3. used-value 布局等价：`min(20px, 10%) should be used-value-equivalent to 20px`
   ——layout resolve_length 需对 min/max 表达式求值。

## 处置

- face 剥除（电池 cleanup 已反剥，types.rs 树净实证）；patch/脚本留
  `.acceptance/wptfyi-css/` 供 SERLAP 刀复用电池模板；不入库不记 tally。
- 负结果价值：+92F 池机制重定位（parse ✓ → serialize/used 两层），省一次错层入库。
