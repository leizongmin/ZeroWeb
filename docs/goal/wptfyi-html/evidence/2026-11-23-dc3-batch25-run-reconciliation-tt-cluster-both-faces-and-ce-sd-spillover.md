# DC-3 · 批二十五：run 45063183178 收账——TT/Sanitizer 刀簇双向账（Sanitizer IDL 面曝光＋tt/san 大额兑现）＋CE·SD −2,772 溢出＋dom/ 停摆形回位（批二十四归因降级）

2026-11-23。run 45063183178（11-22 19:52:13Z dispatch，~2h06m）对批二十四
run 45049069340 同分片逐键 A/B；快照 coverage-2026-11-22（442d0e6a1
回写，UTC 同日覆写批二十四文件，generated 21:56:11Z，aligned a3830cd8f7
分母恒 2,278,575）。窗口码变（批二十四 checkout 之后落 main）唯一簇＝
对端 TT/Sanitizer 刀：abfa08966 ②面复原＋7c0b0bf4f headless 泵
（apps 面构建图外）＋6346b9981 ③a 基座＋efb757973 ③b① 六 setter——
全部位移按排除归因该簇，逐刀细分归对端 bisect。

## 目标级账

html 面 109,408 → **109,427（+19）＝净 +38,331/基线 71,096，验收
≥152,096 余 42,669**。

## Sanitizer IDL 面曝光（html/dom +18 主构成）

- `?exclude=(Document|Window|HTML.+)` 变体键 16 子测试 Fail→Pass
  （同注册集 1,628）＝Sanitizer interface object 全套曝光
  （existence/prototype/constructor/@@unscopables/方法签名）——②面
  复原＋③a part13 构造面兑现；nameset 548dd3→569d8f＝状态变化所致，
  三键仍 DISTINCT＝过滤生效。
- partial-updates/tentative 两键 +4（sanitize-nested 2→4、
  template-for-sanitization 0→2）；setHTMLUnsafe 家族 ±对冲
  （runScripts +2、setHTMLUnsafe −2，webappapis 组行 ±0）。

## 邻账兑现（对端刀簇正向面）

- **tt 套件 943 → 1,281（+338）**；快照组行 918 → 1,256＝两口径精确
  同值——③b① 兑现全在 aligned 清单内路径，批二十四清单外差不再扩大。
  对端 runner 口径 +249（set-attributes-require 三页），coverage 口径
  +338 量级吻合按面归因。
- **san 套件 60 → 323（+263）**；快照组行 39 → 281（+242）——差 63＝
  sethtml-with-trustedtypes 域清单外/zw_only 口径，两口径各自闭合。

## CE·SD −2,772 溢出（本批主负信号）

- `property-reflection-idl-setters.html` **2,910 → 225（−2,685）**：
  注册数不变（2,910），P→F 整面翻红——IDL 元素反射 setter 矩阵
  （ariaLabelledByElements/ariaActiveDescendantElement/
  interestForElement 等）全红。
- property-reflection −15、DSD attachment −68／basic −3、clonable −2；
  custom-elements/reactions +1 对冲。
- **探测不对称**：对端自家 runner 全桥（1,735 键 html-semantics 通道）
  fail=3 持位＝本溢出对其看门狗不可见，coverage web-components/
  shadow-dom 套件为唯一探测器。归 CE·SD 账本＋对端定谳修复，本流
  不修。

## dom/ 停摆形回位 ＝ 批二十四归因降级

dom/ranges 42,300 → 42,420（**+120 部分回位**：dataChange 947→1,016、
replaceData 1,001→1,052，页级 TO 形仍在不完全注册；vs 批二十二余
−230）。**窗内无任何 dom 侧修复落码而回位**＝停摆形对负载/时序敏感
（非确定码行为）——批二十四 4fb594667 机制归因降级为「嫌疑形＋timing
不稳定并存」，定谳权在对端（wptfyi-dom 账本）。

同款停摆形再现 editing/run：**−220**（delete.html?7001-last −46／
multitest.html?9001-last −174，均 P→None 未注册＋注册缩水，尾分片形）
＝批二十四 editing +96 部分回吐，timing vs 码两假说并存记档。

## 观察清单（批二十四承接口）全持位

webappapis 1,540／per-global 6 注册 3P/3F／dat shapes 逐字节同（write
TO 恒定续持）／interestfor −3／合成 Fail 扫描（os error 2 清零持位、
fetch-module 2 键＋import-maps 3 键持位）——全持位。track-cues 持位
Pass 第二连演（flaky 判定待第三演）；onblur flaky +2（checkable-
active-onblur-with-click 3P→5P，快照 +1 记账粒度）。

## 其余组漂移（各流域域）

css-animations −14／css-backgrounds −1／css-borders +7／css-writing-
modes −1／web-animations ±3／dom/nodes −1（MutationObserver-attributes
42→41）——各流域 watch 不深挖。

## 面差对平与快照管线观察

组差合计 **−2,286**＝zw_passes 1,930,725 → 1,928,439 逐项闭合（tt
+338、san +242、CE·SD −2,772＋shadow-dom −2、dom/ranges +120、
editing/run −220、html/dom +18、html/semantics +1、css/动画面 −10 等）。
**环比护栏未拦截 −2,286 总回落**（快照照常回写 main）——护栏判定域
观察记档，归 wptfyi-data 域。

[逐文件 JSON](2026-11-23-dc3-batch25-run-reconciliation-tt-cluster-both-faces-and-ce-sd-spillover.json) ·
[批二十四证据](2026-11-23-dc3-batch24-run-reconciliation-perglobal-editing-knife-and-dom-collision.md)
