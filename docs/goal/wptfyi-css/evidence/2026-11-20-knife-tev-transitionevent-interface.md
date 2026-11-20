# 刀 TEV TransitionEvent IDL 面收账（2026-11-20 10:2x）

## 收账结论

- **A/B 净 +23（gained=23 lost=0 regressed=0 pages_gain=1）**，face 落库 `bdce03d4c`（part05.js +45，唯一 pathspec）。
- 局部账累计 +7697（r6 伪增益修正后锚）→ **+7720**。
- 主靶 css-transitions 单页 `transitionevent-interface.html` P 19→42（**42/42 全绿**）。

## 刀面机制（face 设计）

既有 `_defineEventSubclass('TransitionEvent', …)` 的 `_factory`（part05.js）两处不可表达：
`o[p[1]] != null ? … : 默认` 把 `null` 归入缺省（WebIDL es-dictionary：undefined 才是缺省、
null 是现值须转换）；且直赋属性 writable。face 追加专设构造器（+45 行，命名函数表达式
`_tevWrap = function TransitionEvent` 防提升劫持 + `_tevBase.prototype` 共享 + Reflect.construct
保子类 instanceof）：

1. **missing type**：`arguments.length < 1` → TypeError（`Missing type argument`）。
2. **只读描述符**：propertyName/elapsedTime 构造后 `Object.defineProperty` `writable:false`
   （assert_readonly 走原型链断 [[Writable]]===false，testharness.js:2136）。
3. **DOMString 转换**：`String(o.propertyName)`（null→"null"、false→"false"、true→"true"、
   数字→十进制串、[]→""、[1,2,3]→"1,2,3"、对象→"[object Object]"、valueOf→其返回值串化）。
4. **double 转换**：`Number(o.elapsedTime)`（''→0、false→0、true→1、[]→0、[0.5]→0.5、
   valueOf→数值）。
5. **非有限 TypeError**：NaN/±Infinity → TypeError（`The provided double value is non-finite`）；
   'sample'、[0.5,1.0]、对象（valueOf 返非数）→ ToNumber NaN → 同 TypeError。

## 电池 knife-tev（12 腿 = 6 corpus × b0/b1）

- b0 = HEAD + 全部 ride（tev face 剥除）；b1 = HEAD + 全部 ride + tev face
  （`knife-tev-face.patch` +56/−0，md5 `aca7ad72a15566a9bb14e7690726d0d4`，与提交时 live
  diff md5 同源双证）。
- bins：b0=`45ea52fe146088621659f7103c236d23` / b1=`e9ece0a4f574993ac925a495840dfc78`；
  M2 异 md5 过；FACE-MARKER `_tevElapsedTime`（ASCII 标识符）b0=0 / b1=2。
- RIDE 13 文件（含 part04.js r6 face 作 ride）S1/S3 md5 零扰动。
- 预校验：25 片 concat `node --check` PASS；node 忠实仿真 50 断言 ALL-PASS
  （工厂 isSuperCall super 臂逐字移植后 subclass instanceof 全通）。
- 产物：`.acceptance/wptfyi-css/results-knife-tev/`（b0/b1 json+err、s1/s2/s3-md5、ride-files）。

## 主靶 23 翻转逐条（b0/b1 JSON 权威提取，全数 Fail→Pass）

| # | 断言 | 机制臂 |
|---|------|--------|
| 1 | Missing type argument | missing-type TypeError |
| 2 | propertyName is readonly | 只读描述符 |
| 3 | elapsedTime is readonly | 只读描述符 |
| 4-11 | propertyName set to null / false / true / a number / [] / [1, 2, 3] / an object / an object with a valueOf function | DOMString `String()` ×8 |
| 12-17 | elapsedTime set to false / true / '' / [] / [0.5] / an object with a valueOf function | double `Number()` ×6 |
| 18-20 | elapsedTime cannot be set to NaN / Infinity / -Infinity | 非有限 TypeError ×3 |
| 21-23 | elapsedTime cannot be set to 'sample' / [0.5, 1.0] / an object | ToNumber→NaN TypeError ×3 |

## 哨兵五语料逐字节零差

css-animations 840→840、css-fonts 6524→6524、css-properties-values-api 958→958、
css-typed-om 12024→12024、css-shapes 6026→6026：双臂全等，零 diff。
主靶 css-transitions 1028→1051（+23 全在 transitionevent-interface 单页，其余 122 页零翻转）。

## 机器协议备注

- 发射链 knife-tev-chain 等 `zw-wa3a-r6-legs.service` 退场（90×30s cap）+ 60s 宽限后串行发射
  （机器独占协议）；10:18:28 GATE-DONE rc=0。
- b0 solo 探针（/tmp/tev-solo）曾撞 knife-optical-chain 并发窗，event-dispatch 1P/18 污染作废
  ——发射前须 systemctl 全量活跃单元核对（含非 `zw-` 前缀的 `knife-*` unit）。
- 派发路径兼容：part01 构造派发 2 参预转换分支与 face 构造器语义同源（DOMString/double
  已在派发侧转换），face 只补构造器入口。

## 并笔备注：+2 修正（同 commit 承载）

R4-ARBITER 10:3x 终裁 +2 伪增益成立（b0 冻结 bin `3823f761` solo 三档墙钟
60s/180s/2400s＝9→12→19 断言爬行轨迹，2400s 案含 2 Pass 与争端 +2 逐字同测试名＝
电池窗 b0 0P 系负载压制截断的报告完备性不对称）；本 fork 承接修正 commit，锚
+7699 → +7697 并入本笔（master.md header 修正 provenance 由 staged 批采彼并笔），
TEV 收账基于修正后锚：+7697 + 23 = **+7720**。
