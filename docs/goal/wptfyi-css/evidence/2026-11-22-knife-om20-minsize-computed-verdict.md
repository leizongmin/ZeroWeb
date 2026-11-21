# 刀 OM-20 判读收账 · min-size computed 'auto' 分面保形 6 靶（2026-11-22）

## 判读门（05:16:57 GATE PASS）

```
[css-typed-om] keys b0=12503 b1=12503 P 12440->12448 F2P=6 P2F=0 new=0 gone=0
[cssom]        keys b0=1630  b1=1630  P 888->888    F2P=0 P2F=0 new=0 gone=0
[css-sizing]   keys b0=5700  b1=5700  P 3239->3239  F2P=0 P2F=0 new=0 gone=0
[css-values]   keys b0=1385  b1=1385  P 984->985    F2P=0 P2F=0 new=0 gone=0
TARGET-GAIN om20-minsize F2P=6 (expect 6)
GRAND F2P=6 P2F=0 new=0 gone=0 excluded=5
OM20-GATE-PASS
```

- 恰集 6/6 全 F2P（computed/get-auto-min-size.html 六靶），无
  TARGET-MISS、无 NON-TARGET-F2P、无 P2F-LIST。
- cssom SET-IDENTICAL（888→888）＝CALC-SIZE-RESOLVED-S3
  resolved-min-size 7 靶 string gCS 合同零扰动——本刀分面设计的核心
  验证点。css-sizing 恒同；css-values P +1＝排除域 random flake
  （OM-17 doc_seed 定谳域）＋zz-om20probe 探针翻转 3，账面自洽。

## 机制与 face

- b0 双根因：flex/grid 4 靶＝引擎文本 'auto'（S3 例外保形）而 keyword
  map 缺臂回落基类；block 2 靶＝S3 resolved 值门（baf6b16d7，csswg
  #11716）string gCS '0px' 被 typed 面继承出 CSSUnitValue。
- 分面合同：string gCS＝resolved 值（cssom 7 靶合同，不动）；typed
  computedStyleMap＝computed 值（css-sizing-4，min-* 保持 'auto'）。
- face＝part10-typed-om.js 单文件单臂（patch md5
  `46f6a9430b3cf6b03cdfb5bc213bbe2e`）：mapGetAll computed 面 min-* 读
  `__zwOmComputedEl.style` inline 声明文本——无声明或 'auto' 归
  CSSKeywordValue('auto')，inline 数值声明落回原文解析（OM-4 cursor
  声明文本源先例）。FIXME：stylesheet 声明面 shim 不可见挂账。

## 电池与二进制锚

- 电池 `knife-om20-minsize-computed-battery.sh`，log
  `knife-om20-minsize-battery-050338.log`（05:03:38 发射含跨 clone 静默
  等待 ~3min→05:16:57 GATE-DONE；14 腿全 rc=1 正常尾）。
- 前置闸实证：PREFLIGHT-BUSY-WAIT busy=6（兄弟 csinterp-s2a-r4 电池腿
  在飞）→ 05:06 对端收尾后自动进入构建——跨 clone 墙钟隔离首次实战。
- b0r md5 `e3195594d60753676a2d3d14d76318ff`（＝OM-19 组合复锚
  merged-runner 同字节——115adf54d HEAD 构建，跨电池二进制身份互证）／
  b1r md5 `1bccb0294a92d5222697524cdbc0279d`——M2 distinct ✓，
  FACE-MARKER `刀 OM-20` b0r=0/b1r=1。
- 探针页 zz-om20probe（gitignored）：READINGS 双读数＋R4/R5/R6 类形
  合同（undeclared keyword/declared-px Unit/flex keyword）。

## 残面与挂账

- typed-om 语料残池 40F−6＝34F（census 口径）：scroll-snap-align 4F
  结构性不可赢（前账）、get-auto-min-size 余 0、余散布 1–2F 族
  （break/transform/writing-mode/serialization 转义/declared 边角）。
- stylesheet 声明 min-* 计算面（shim 不可见）挂账。
