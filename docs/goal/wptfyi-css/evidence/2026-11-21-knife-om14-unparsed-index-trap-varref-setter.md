# 刀 OM-14：unparsed 索引 setter 越界门 + varref variable 可写 2 靶（2026-11-21 10:2x 收账）

## 池

OM-13 后 typed-om 残池 121F/64 页，本刀取 2 页 2F：
cssUnparsedValue-indexed-getter-setter 1F（`set [len+1]` 不抛＝OM-13
探针 zzP11 挂账兑现）、cssVariableReferenceValue-variable 1F
（`.variable` 只读，`'--bar'` 更新面不绿；`''`/`'bar'` 旧态 strict
赋值 TypeError 已绿）。OM-13 挂账标注「需 Proxy 结构改造，sentinel
热路径下风险高，单独立刀」。

## face（patch `b2139bb4`，单文件 part10-typed-om.js +26/−2）

- **A. CSSUnparsedValue 索引 setter 越界门**（css-typed-om-1
  §dom-cssunparsedvalue-__setter__）：Array exotic 索引写不可拦截，
  Proxy set 陷阱为合同唯一实现面——canonical 数值串键 n > length 抛
  RangeError（"Index out of range"）、n == length 追加放行；仅
  `typeof k === 'string'` 时做 canonical 判定，length/负索引/非
  canonical 串/符号键透传。实例本体仍 Array exotic，
  isArray/instanceof/迭代/展开 node 冒烟实证。
- **B. CSSVariableReferenceValue.variable 可写**（§dom-cssvariablereferencevalue-
  variable）：set 臂同构造器校验——非串或非 `--` 前缀 TypeError
  （"Invalid custom property name"）且原值不变。

v1 face（patch `666c777b`，前会话 09:40 电池 GATE PASS）trap 臂
`Number(k)` 对符号键抛 TypeError，与本面「符号键透传」注释相悖
（node 冒烟实证）；本轮收账前补 `typeof k === 'string'` 门重跑电池，
判读字节与提交字节重绑定（v1 电池不入账）。

## 电池（knife-om14-battery.sh，log knife-om14-battery-1013.log）

b0＝face 剥除（OM-13 提交态 69852d1b1 树）、b1＝树面＋face，M2 异
md5 ✓，FACE-MARKER OM-14 注释 b0=1→b1=4（b0 侧 1 处＝OM-13 挂账
注释）＋_zwOmUnparsedIndexTrap 0→2。10:13:05 build 起、10:21:38
收账（8.5 分钟，全程 test-guard 包裹：build per-proc 6/total 14/
1800s＋leg per-proc 4/total 8/3000s＋CASE_WALL_S=60）。哨兵＝
css-variables＋css-logical（Proxy 包装 unparsed 热路径双哨兵——
custom prop/var() 值逐构造点入 Proxy）。

判读（knife-om14-judge.py）：TARGET-GAIN **2/2**（两页各 1/1 恰集
F2P，NAME-DIFF gone 合计恰 2 new=0）；STAY-FAIL 1/1（engine 转义域
roundtrip 面按账保 Fail）；typed-om P 12285→12287（**+2**）/F
121→119（−2）；双哨兵 SET-IDENTICAL（variables 56 页/logical 86 页
——Proxy 结构改造风险面清偿）；**OM14-GATE PASS rc=0**。TRAP-NOOP
state=done rc=0（b1 face 态即提交态，树 md5 `d457e5e5` 与判读绑定）。

## 残面

- engine 自定义属性转义域（跨流挂账）不变：roundtrip 1F＋var 名
  反斜杠截断＋gCS 替换域。
- typed-om 残池 **119F/64 页**续分解；gCS typed 通道 ~69F、marker
  18F 维持既有序挂账。

## 验证面注记

JS shim 单文件（include_str 入 engine 二进制），fmt/clippy 不适用；
行为验证＝电池三 corpus（typed-om 359 页＋variables 56 页＋logical
86 页）＋Proxy 透传面 node 冒烟（isArray/instanceof/迭代/展开/
符号键读写/越界抛型）。

[控制面](../master.md)
