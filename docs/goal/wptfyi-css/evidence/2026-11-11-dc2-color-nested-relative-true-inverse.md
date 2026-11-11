# DC-2 color 嵌套 relative 真逆直转账（本批面 +5）

2026-11-11。接续 [RCS+var 入口放行批](2026-11-11-dc2-color-var-origin-rcs-entry.md)
后的 color 挂账桶「嵌套往返精度 5F + pow 通道 1F」。

## 批内容（part11-computed-color.js +31 行，提交 8774b164c）

1. **OkM1/OkM2 数值真逆矩阵**：Gauss-Jordan 13 位字面（`_zwCCOkM1t`/`_zwCCOkM2t`）。
   发表逆 `OkM1i` 与真逆偏差 7.6e-4（链内致 rgb 投影往返 e-5 级输出噪声）、
   `OkM2i` 本系真逆（偏差 5.5e-8）。发表逆仍供序列化投影面（8 位语料锚按发表
   常数判定，全局换真逆会漂锚）。核验：M1t·M1 偏差 1.3e-13、M2t·M2 偏差
   3.3e-12、xyz65(0.99,0.88,0.77) 全链往返 ~7e-12（语料容差 ±1e-4）。
2. **`_zwCCOriginInSpace` 嵌套直转臂**：origin 为 relative-out oklab/oklch 且
   目标空间 xyz/xyz-d50/xyz-d65 时，lab 坐标经 M2t 立方回 LMS、M1t 直配 xyz65
   （d50 再经 D65toD50），绕过 rgb 投影往返。lab/lch native cbrt 自逆不入臂；
   hsl/hwb 越界往返留 out-of-gamut 桶。
3. **decl 面裸数学函数形放行**：`pow(l, 1)` 等直写形（函数名白名单 + 括号
   平衡），求值归 resolve 期 `_zwCCEvalRel`（css-values-4 § math functions）。

## 多实例仲裁与采纳口径

本批由同 session 双 spawn 竞态的前轮实例开发：rrnest 探针（11:03）时二进制含
**错误 M2t**（4 案嵌套未翻、仅 pow 翻正），11:08 修正矩阵后实例停摆未及重建。
本 session 按「孤儿爆发杀 agent 留自足脚本零重跑采纳」处置：清杀两个孤儿
resume，采纳修正后的工作树认领件，重建复验（rrs2）。修正前探针账目保留
（results-rrnest：2480P/28F）。

## 验证（rrs2，css-color 全量 + values/typed-om 漂移门）

- css-color **10,900 → 10,905 净 +5 逐页零回归**：color-computed-relative-color
  6F→1F——`color(from oklab(from color(xyz-d50 0.99 0.88 0.77) l a b) xyz-d50
  x y z)` oklab/oklch × d50/d65 四案全翻（精确 0.99 断言）+ `pow(l, 1)` 形
  1 案；残余 1F = hsl 嵌套（归 out-of-gamut 挂账桶，预期保持）。
- css-values **2,213 → 2,218 +5**：clamp-color-computed 5 案翻正（裸函数形
  白名单连带），零回归。
- css-typed-om 9,872P 零漂移。
- make test：复用同树前证（zvvf-make-test.log，3848P/7F；7F = main 既有
  websocket 族欠账）——本树相对该证仅差 3 行矩阵字面量，其唯一消费臂（嵌套
  直转）无单元测试覆盖，行为面由 rrs2 corpus 验证。
- 证据产物：`.acceptance/wptfyi-css/results-rrs2/`（三语料 JSON），
  日志 `logs/rrs2-*.log`。

## 挂账

color 桶残余：hsl 嵌套往返 1F（out-of-gamut 桶）、系统色 27F、out-of-gamut
20F、opacity calc 20F 等（见 [DC-3 预备缺口地图](2026-11-11-dc3-prep-reconciliation-runbook.md)）。
