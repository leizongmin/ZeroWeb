# M2 S4c pathData API——getPathData/setPathData + SVGAnimationElement 控制面 stub（2026-10-21）

S4b-2（evidence/2026-10-21-m2-s4b2-paint-geometry-landing.md）收口后接续。
源码面单文件 part03.js（pathData helpers 四件 + SVGGeometryElement 方法安装
+ SVGAnimationElement begin/end stub + `_zwSvgPathParse` Z 尾数死循环根修）；
wpt-data 侧补拉 `/common/rendering-utils.js` helper 一件。编码与定向验证在
前一审 session 完成（5 小时限额中断），corpus 复跑、DC-4、对账与收口由接续
session 完成。

## 实现

### getPathData(options)（spec SVG2 paths.html §InterfaceSVGPathData）

- 返回普通 dictionary 对象序列 `{type, values}`——每调用重建（fresh array/
  segment/values 三层 fresh，WPT fresh-array 定谳面）；type 保留 d 属性原文
  大小写（m/l 小写面定谳）；A 旗标出 1/0 数值。
- 隐式重复组逐组拆段：M 后续组为 lineto（大小写随 moveto，spec
  §PathDataBNF）；部分尾组丢弃、完整组保留（§PathDataErrorHandling）；
  字母无完整组（'M 10 20 L'）→ 无段产出；路径须以 moveto 开头（bare 'Z'
  → 空序列——malformed-d 定谳面）。
- `{normalize: true}`：绝对 M/L/C/Z 缩减——H/V→L、相对→绝对、S/T 平滑反射
  （无前序 C/Q 时控制点 = 当前点，S4a flatten 同口径）、Q/T 升阶 cubic
  （2/3 控制点提取）、A 分解 cubic（端点→中心参数化 F.6.5 + 半径缩放校正
  F.6.6 + span >90° 等分多段；首段起点/末段终点取精确端点防漂移）、Z 闭合
  复位。normalize 精度 epsilon 0.0005 实测余量充足。
- attribute base value 语义：CSS d property / SMIL / Web Animations 动画态
  均不入 getPathData（css-d-vs-attribute 定谳）；d 属性缺失/空 → 空序列。
- 形状元素（rect/circle 等价路径展开）未实现（FIXME(svg-compat)，corpus
  暂无消费者）。

### setPathData(data)

- sequence\<PathDataSegment\>（iterable 协议，Array.from 承接——数组/generator
  展开/Float32Array 定谳面）+ 逐段 dictionary 校验：type 单字符且 ∈
  MLHVCSQTAZ（多字符非已知命令别名）、values 精确 arity（M/L/T 2、H/V 1、
  C 6、S/Q 4、A 7、Z 0——结构 API 不做隐式重复拆分）、数值 Number 收敛后须
  有限（NaN/Infinity 段无效）、Z 须空 values。
- 段级错误 → 合法前缀截断（spec §PathDataErrorHandling；unknown-X/extra-
  values/too-few/NaN/Infinity/Z-with-values/MM 定谳面全过）；前缀空或首段非
  moveto → 移除 d 属性（leading-non-moveto/wrong-arity/空 sequence 定谳面）。
- 序列化：type 原文（Z/z 规范化 'Z'）+ 值单空格连接；A 旗标位 ToBoolean
  （2/5→1、false→0——flags-coerce 定谳面）。
- 写入走 setAttribute/removeAttribute——MutationObserver d 属性记录（含
  oldValue/newValue 与空 sequence 移除记录）随之原生派发（mutation-observer
  两案全绿）。

### 前置与根修

- **SVGAnimationElement begin/end 控制面 stub**（spec animate.html
  §InterfaceSVGAnimationElement）：beginElement/beginElementAt/endElement/
  endElementAt——SMIL 时钟未接入（FIXME(svg-compat M4)），仅控制面存在性
  （S3 pauseAnimations stub 同口径）；setPathData-during-animation 案只消费
  beginElement 可调用性 + 属性 base 值读写。float 校验同 setCurrentTime。
- **`_zwSvgPathParse` Z 尾数死循环根修**：groupCnt 无 'Z' 映射 → 'Z' 后尾随
  数字走 else 分支 cnt=undefined → 消费不推进 → 死循环（S4a 期潜伏缺陷，
  corpus 无输入命中故未暴露；'Z 1 2' 类输入挂死整个 runner）。修复：cnt
  undefined → 错误截断（前缀保留语义）。对既有终止输入零行为变化（构造性
  论证：undefined 分支此前不可达），path/parsing 116P/0F 复跑佐证。
- **wpt-data 补拉 `/common/rendering-utils.js`**（upstream master，19 行
  waitForAtLeastOneFrame，2026-10-21 取）：M1 期「缺失 helper 7 项不补」
  决议的定向修订——彼时消费面均属渲染/CSS 域（out of scope）；本切片
  setPathData-during-animation 为 DOM 面 in-scope 消费者，照 M1
  SVGAnimationTestCase 补拉先例办理。其余 6 项 helper 维持不补。

## 两轮修出（首跑 51/56 → 复跑 54/56）

1. `setPathData-malformed` multi-char type：「MM」→ U='' → 
   `'MLHVCSQTAZ'.indexOf('')` 恒 0 假命中 → 按 arity 2 收敛成段。修正：U 空
   串先排除。
2. `setPathData-malformed` Z-with-values：Z 走 arity-0 分支绕过 values 校验
   → 'Z [1,2]' 误收。修正：values 先统一 Array.from + 精确 arity 校验
   （Z 须空）。

## 验证

- **path/interfaces 0P/56F → 54P/2F**：segments/normalize/malformed/
  plain-object/overrides-d/mutation-observer/detached/empty-or-missing/
  malformed-d/css-d-vs-attribute/during-animation 全族；唯二 F =
  setPathData-css-d.tentative 两案（CSS `d` property 计算值序列化
  `'path("...")'`——CSS 域协同挂账，S4a 期同口径划界）。
- normalize 数值面：H/V 链、S 反射、Q/T 六连升阶、60° 单弧（含 x-axis-
  rotation 10°）、180° 大弧二分——全部 0.0005 epsilon 内。
- 回归面：path/parsing **116P/0F**（Z 根修零回归实证）、getBBox 全量
  **146P/2F**（唯二 F 均既有挂账：-02 tspan Ahem 字形度量、-03 image
  intrinsic size）、-05 15/15、-09 24/24 维持。
- 全 corpus 对账（S4b-2 基线 1046P/650F/40T → 本轮原始全量
  **1104P/590F/41T/0NR = 63.6%**，分母 1736→1735）：逐子测试零 Pass
  回归、零 DOWN 文件；+58P/-60F 中 54 属 path/interfaces 靶面，4 属两个
  时序敏感案波动（smil-values-empty-segments 2 子、use-load-error-events
  .tentative 2 子——单案复跑呈 F/P 抖动，非本切片断言面，不记入归因
  升级）。探针剔除同分母 **1070P/590F/41T = 62.9%**（S4b-2 基线
  1012P/650F/40T = 59.5%）。Timeout 40→41 = syncbase-escaped-dots
  F(2)→T(1) 分类移动（见卡点）。runner 二进制 mtime 晚于 part03.js
  末次修改，corpus 全程一次跑完（guard 1800s 内）。
- DC-4：`cargo fmt --all --check` EXIT=0；定向 `cargo test -p zero-engine`
  （test-guard 4G/8G/3000s）**3372P/0F，EXIT=0**。

## 卡点与划界

- **CSS d property（setPathData-css-d.tentative 2F）**：`getComputedStyle`
  的 `d` 属性计算值 + `'path("...")'` 序列化 + CSS cascade 覆写语义——CSS
  解析/样式域协同（S4a 期 getTotalLength CSS d 2 子同挂账），M4 归口。
- **syncbase-escaped-dots F(2)→T(1) 分类移动（机制已溯）**：该案调
  `trigger.beginElement()` 后监听 `beginEvent`（SMIL syncbase 语义）。
  pre-S4c：方法不存在 → TypeError 快速 2F；S4c 后：stub 存在但 no-op →
  等 beginEvent 不至 → 30s Timeout（pre/post 单案对照实证）。属 SMIL
  时钟未接入挂账面的预期分类移动，无 Pass 损失；M4 SMIL 评估归口。
- **形状等价路径**（FIXME）：SVGGeometryElement.getPathData 对
  rect/circle 等的等价 path 展开（Chromium 有为，corpus 暂无断言消费者）。
- **SMIL/Web Animations base value 域划界**：getPathData 恒 attribute base
  （动画态两案全绿验证）；animVal 通道与 SMIL 时钟接入归 M4 SMIL 评估。
- 既有挂账继承：-02 tspan Ahem 字形度量、-03 image intrinsic size（M4
  定稿归口，见 S4b-2 evidence）。
