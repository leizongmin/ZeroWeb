# M3 S2 SVGURIReference href 反射——xlink:href 兼容语义 + readonly 对象面（2026-10-21）

M3 S1（evidence/2026-10-21-m3-s1-hittest-landing.md）后 M3 第二切片。
linking/scripted 20F+3T 定性后划界：xlink-href-compat 12F + href-animate 4F +
href-mpath 1F 属 href 反射面（本切片），href-script 3F+markup 1F 属外部脚本
加载语义（S3 边界），a.ping 2F + a.rel-* 2T 属导航域（挂账）。实现纯
SVGURIReference href IDL 反射，单文件 part03.js 为主 + part05.js set trap
一处门。

## 实现（spec SVG2 linking.html#InterfaceSVGURIReference）

- **href accessor 安装**（part03.js，`_zwDefSvgProps` 数值表后）：10 接口面
  （SVGAElement / SVGAnimationElement（一次覆盖 animate/set/animateMotion/
  animateTransform/mpath 五元素）/ SVGFEImageElement / SVGFilterElement /
  SVGForeignObjectElement / SVGImageElement / SVGPatternElement /
  SVGScriptElement / SVGTextPathElement / SVGUseElement）prototype accessor
  （get trap R93 原型链回落以元素 proxy 为 this 派发，`this.hasAttributeNS`
  等走 R3024/R3217 同步面）。
- **读优先级**：null-ns href 在 → 读 href；否则 xlink:href 在 → 读
  xlink:href；都无 → `''`（case 4/7/8/9 定谳）。
- **写回 backed 属性**（`href.baseVal = v`）：backed 判定按「当时存在的
  属性」——href 在写 href、仅 xlink:href 在写 xlink:href（case 2/5 定谳：
  case 5 仅 xlink 在时 baseVal 写回 xlink:href，即便写后另设 href 会夺走读
  优先级）、都无新建 href（case 3）。
- **animVal 恒同 baseVal**（case 11/12 string 值面；SMIL 动画评估归 M4）。
  baseVal/animVal 每次 live 反查属性（removeAttributeNS 回落链 case 7/8/9/10
  免失效缓存）。
- **[SameObject]**：per-element 缓存（`_zwSvgAnimKey` 同款键形态 + 独立
  `_zwSvgHrefCache`），`el.href === el.href` 恒真；对象为 shim
  SVGAnimatedString 实例（instanceof 真）。
- **get trap A/AREA URL 分支 SVG ns 排除**（part03.js:~23040）：SVG a 此前
  吃进 HTMLAnchorElement URL 分解反射（href 返 string/pathname 等 URL 组件
  存在）——排除后 href 落 prototype accessor，pathname 等 URL 组件 IDL 于
  SVGAElement 不存在（真浏览器 undefined），HTML a/area 路径不变。
- **set trap SVG href readonly 门**（part05.js R3069 reflected 分支前）：
  `el.href = v` 对 SVG ns 元素静默忽略（readonly IDL setter sloppy 无效）；
  写面走 `el.href.baseVal = v` 实例 setter。旧 R3069 按 HTML a/area 语义写
  href 属性——SVG a 会吃进真浏览器没有的假 set→get round-trip。
- **discard 豁免**：spec SVG2 URIReference 元素表列 discard，但真浏览器未
  落地 SVGDiscardElement（WPT animations/historical 定谳
  `window.SVGDiscardElement === undefined`，discard 元素落 SVGElement 通用
  面）——不建接口、不装 accessor，表外回落即正确行为。

## 修出两轮

1. **SVGDiscardElement 接口新建触发 historical 回归**（r1 corpus 发现）：
   首版按上轮侦察结论「SVGDiscardElement 不存在需新建」建了三处表项 +
   装 accessor——r1 全量对账揪出 `svg/animations/historical.html::
   SVGDiscardElement` **Pass→Fail**（该案断言 `window.SVGDiscardElement ===
   undefined`——WPT historical 定谳浏览器不暴露该接口，上轮「spec 列了就该
   建」的判断与浏览器现实相反）。修正：回退三处表项 + 安装表移除，探针 P8
   改断言浏览器现实（13 元素 href 全量 + discard.href undefined）。
   r1 数字（含回归未修复态）：1223P/488F/42T/1753；r2 修复后定稿见下节。
2. **href-script 家族移动归因**（非修出、划界证据）：S2 反射落地后
   href-script-element 3 子首断言（href.baseVal）通过、转挂 `waitEvent(script,
   'load')`——文件 F(3)→T；href-script-element-markup 首断言同样通过后
   `loadedScript()` 断言本就满足（markup 脚本加载管线读取 href 属性优先级
   本已正确）——**1F→P 真·连带翻绿**。外部脚本 load 事件派发/dummyScript
   等待语义归 S3。

## 验证

- **探针 m3s2-probe.html 10/10 Pass**（对象面 + SameObject + backed 写回
  两向 + 读优先级 + removeAttributeNS 回落链 + qualified xlink:href +
  animVal 镜像 + 13 元素全量 + discard 豁免 + readonly 静默 + HTML a 不
  回归）。
- **靶面 xlink-href-compat 12/12 全绿**（基线 12F 全灭）。
- **全 corpus 对账 r2**（基线 = M3 S1 收口 1201P/504F/41T/1746 = 68.8%）：
  - 原始全量 **1224P/487F/42T/1753 = 69.8%**（+23P/−17F；分母 +7 =
    探针剔除口径外 m3s2-probe +10 子、href-script 3F 子 + href-mpath 1F 子
    转文件级 T 后分母 −4+1）。
  - 转移矩阵：1201 Pass→Pass（**零 Pass 回归**、零 DOWN）、487 Fail→Fail、
    41 Timeout→Timeout、**13 Fail→Pass**；Timeout 41→42 = href-script
    文件行（F(3) 子转 pending T，S3 边界挂 load 事件）。
  - 探针剔除同分母（1701→1698）：1156P/504F/41T = 68.0% →
    **1169P/487F/42T = 68.8%**（恰 +13P/−17F/+1T）。
  - +13P 聚类全部落 href 反射面：xlink-href-compat 12（靶面）+
    href-script-element-markup 1（连带，见修出 2）——无靶面外移动、无归因
    悬案。
- DC-4：`cargo fmt --all --check` EXIT=0；定向 `cargo test -p zero-engine`
  （test-guard --compile-first 4G/8G/1200s）**3373P/0F，EXIT=0**。

## 卡点与划界

- **href-animate-element 4F 不动**：首断言 `animate.targetElement` 当前
  undefined（S4c stub 未含）——targetElement 目标解析（href 优先级消费 +
  id 反查）随 M4 SMIL 批，本切片不前移（前移只会把快速失败转 30s 文件
  超时，无净增益且拖慢 corpus）。
- **href-mpath F(1)→T**：首断言 `mpath.href.baseVal` 已过，挂
  beginEvent/getCTM（SMIL + getCTM 均挂账面）。
- **S3 边界**：SVG script 外部脚本加载语义（load 事件派发 + dummyScript
  「等足够久」估算 + href/xlink:href 切换不重载）——href-script-element
  文件 T 回收在此。
- **导航域挂账**：a.ping 2F（`observe_entry is not defined` ReferenceError
  + ping 请求管线）+ a.rel-noopener/noreferrer 2T（window.open policy）。
- **CSS d property / 文本几何 / image 内在尺寸 / SVGLength 上下文** 等
  既有挂账不变（见 master.md 下一步计划 3-5）。
