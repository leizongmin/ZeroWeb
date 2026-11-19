# 刀 ARITH：typed-om CSSNumericValue 算术 API + 类型代数（2026-11-19）

## 目标

css-typed-om 残面 arithmetic 族：arithmetic.tentative.any 47F +
add-two-types.tentative 26F + cssMathInvert-type.any 3F +
cssMathNegate-type.any 2F（~78F，CSSNumericValue 算术/型别 API 族）。
纯 part10 shim 面，与 face C engine gCS 回显层、AR animation-range 刀
（11 Rust 文件）、五形族三面零交叠。

## 根因（b1 电池残面法证，results-knife-faceb/）

- **类型 map 形状错**：`_zwOmUnitType`/`_zwOmMathType` 恒产 7 族 0 键 +
  phantom percentHint——`assert_numeric_type_equals` 逐键 assert_equals
  （undefined 语义），零键必须省略而非 0（type 族页 P0 根因）；sum/min/
  max/clamp 合并用 Math.max 粗粒度，product 无 percentHint 归并、invert
  负指数带 0 键。
- **无加法消解**：不存在 `add-two-types`（§add-two-types）——Sum/Min/Max
  构造器无类型校验（number+px 不抛、px²+percent 不抛）；percent 归并
  percentHint（% + 指数 1 单族）全缺。
- **`_zwOmArith` 六法恒构节点**：无参不恒等（期望返回 this）、numberish
  不展开、不可加不抛 TypeError、同单位 UnitValue 全体不化简（sum/sub/
  min/max 同单位、mul 非 number 至多一个、div 除数全 number）、div 除数
  number 0 值不抛 RangeError、sub 取负不剥 Negate 壳/div 取倒不剥 Invert
  壳（testsuite `assert_style_value_equals` 结构匹配面期望
  `CSS.number(-1)`/`CSS.number(0.5)` 字面 UnitValue）。

## 码面（1 tracked 文件：part10-typed-om.js）

1. `_zwOmUnitType` 重写：零指数键省略 map（number→{}；percent→
   {percent:1}；其余族键=1），css-typed-om-1 §cssnumericvalue-type。
2. `_zwOmTypeOfVal` + `_zwOmTypeAdd` 新增：加法消解（§add-two-types）——
   同族指数逐键相等；percent + 指数 1 单具体族 → percentHint 归并
   （族键保留、percent 键吸收）；% + number → percent 保形；双侧 % 指数
   相等保留；hint 冲突/指数不等/任一键缺于一侧 → strict 抛 TypeError、
   宽容面（type() 恒不抛）并集近似。
3. `_zwOmMathType` 重写（§type-of-a-cssmathvalue）：sign→空型；negate
   保形；invert 指数取负；product 指数累加 + hint 传播；sum/min/max/
   clamp/hypot/abs 逐对 `_zwOmTypeAdd` 宽容面。
4. 加法型构造器 strict 校验：`_zwOmMathCtor` 内 Sum/Min/Max/Hypot 逐对
   `_zwOmTypeAdd(strict)`；`_zwOmCtorStrict` 旗标门——calc 解析器经
   `_zwOmFnNode` 绕过（可加性解析期已裁决）；clamp 保持
   `_zwOmClampCompat` 非-相等兼容面不变。
5. `_zwOmArith` 重写：无参恒等返回 this；numberish 展开；sum/sub/min/max
   逐对加法校验（不可加 TypeError）；mul/div hint 冲突校验（§multiply/
   divide-two-types）+ div 除数 number 型 0 值 RangeError；UnitValue 全体
   化简（加/减/取小取大同单位、mul 非 number 成员至多一个、div 除数全
   number 型）——残渣 `_zwOmNumClean`；sub 取负（Unit 直取负、Negate 剥壳
   双重取负抵消）、div 取倒（number 直取倒数、Invert 剥壳）。

## 消费方风险核查（改码前审计）

- `_zwOmNumericCheck`/`_zwOmClampCompat`/`_zwOmValidateMath`/gCS 面
  `_zwOmUnitType` 消费（1798 famName 探测）全部真值迭代或逐键读——
  零键省略形状兼容（`pt[f] &&`/`!t[k]`/`fam[d]===1`）。
- 解析器 Sum/Min/Max/Product 构建（parseSum/parseProduct/fn dispatch
  全在 `_zwOmFnNode` 旗标窗内）→ strict 校验绕过；`toSum`/fold 的 Sum
  构建天然可加（同族门/单成员）。
- 算术方法折叠爆炸半径：typed-om 语料仅两个 crashtest 调算术链
  （px.mul(px).div(px)——无折叠路径，形状不变）；cssMathValue 序列化页
  构造器直构（Negate/Invert 构造器不动）零影响。
- Negate/Invert 构造器不动（折叠在算术方法路径）——calc(1/2) 解析
  Invert(number) 节点形状与 cssMathValue.tentative 期望 'calc(1 / 1)'
  序列化保形。

## A/B 电池（results-knife-arith）

- **14:40:38 首发失败 rc=2（patch-apply 期）**：combined patch
  （faceC + arithmetic 两 face，基线=HEAD）对树内 faceC+arith 态
  `git apply --check` 上下文不匹配（part10:184 faceC to() 首臂）——
  patch 生成基（HEAD）与校验基（在飞叠合态）错位；trap 归一回
  b1 树态，b0/b1 零构建零语料腿。faceC 面（先行在树）已由
  results-knife-facec 电池独立全额验证（+209/0/0），与本刀页域零交叠。
- 重发计划：faceC 先 pathspec 落库后，combined 中剥出 arithmetic
  纯 face patch（基线=faceC 落库后 HEAD），b0=HEAD、b1=HEAD+arith，
  复用 knife-arith-battery.sh 配方换 patch 名重发。

## 预期与残面

- 预期：arithmetic.tentative 47→~0F、add-two-types 26→~0F、
  cssMathInvert/Negate-type 5→0F；type() 形状自愈面（其它 numeric-objects
  type 断言页顺向搭车数待电池定谳）。
- 残面（后续刀）：face C engine computed gCS 回显层（S2 calc 同单元折叠/
  S3 % 保留/S4 em/cm→px 换算，grid 20F + scroll-padding 44F + logical
  族）；mapGetAll 负值包裹 computed 面误包（随 face C）。
