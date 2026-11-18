# g45 image-set 解析与串化刀

2026-11-15。driving: css-images image-set-parsing 114F（66 canonical-mismatch +
48 should-not-set 双断言面）+ image-set-computed 36F（CSS.supports 支持门失败）。
image-set() 此前整值不可解析：写面恒拒 → computed none，支持门恒 false；本刀
单实现点落 css-parser（parse 臂 + canonical 串化 + invalid 判定），style-system
加变体直通，engine computed 串化 + shim 双写面门扩展（g43/g44 既有通道复用）。

## 1. 实现

- **parse 臂**（css-parser `values/parse_extended_visual.rs`）：
  `BackgroundImageValue::ImageSet(ImageSetValue)`——options 逐
  `<image-set-option> = <image> [ <resolution> || <type()> ]?`（css-images-4
  §image-set-nota）；`-webkit-image-set(` 解析期别名剥除；顶层逗号字节态机
  （引号+转义+括号配平）；选项图像限 url/引号串/渐变递归（嵌套 image-set/
  `none`/其他函数拒收）；双 resolution/type 拒收；分辨率明文字面负值拒收
  （calc 可负），dpi÷96/dpcm×2.54÷96/x=dppx 恒等（types.rs 既有约定复用）；
  calc 臂三态折叠 `fold_resolution_typed`（None=文法/型别 invalid、
  Some(None)=静态不可折叠保形、Some(Some)=dppx 基折叠；`+ -`/`- -` 符号归一
  重试不放宽共享 parse 面）；`url("")` 空串合法（calc-infinity-nan 钉面）；
  type() 原文保形不转义还原（转义感知闭引号探测）。
- **canonical 串化**（`values/serialize_gradient.rs`）：
  `canonicalize_background_image_specified` ImageSet 臂恒 Some 投影
  （已 canonical 形恒等回放）；`image_set_specified_to_css`——函数名恒
  `image-set(`、url 引号化 `url("…")`（`\`/`"` 转义）、渐变走
  `serialize_gradient_specified_value`、缺省补 `1x`、Plain 原文保形、Calc
  折叠走 `format_dimension_inner` §10.6 canonical（NaN/±∞ 语料形态构造性
  恒等——零回归保障）、type() MIME 重引号化置分辨率后。
- **invalid 判定扩接**（`background_image_decl_invalid`）：image-set 子串含
  + parse 失败 → 整声明拒收；双写面复用 g43/g44 通道（IDL setProp part06
  `_zwNormalizeDeclValue` 扩 content/border-image-source/cursor 含 image-set
  臂 + setAttribute `_zwStyleAttrDeclFilter`），`__zw_gradient_normalize_decl`
  加 cursor 拒收臂。
- **style-system 变体直通**：`BackgroundImageComputedValue::ImageSet` +
  `bg_image_value_to_computed` 直通臂（CSS.supports 支持门经 parse 臂自动
  放行——computed 页 36F 支持门侧）。
- **engine computed 串化**（`js_dom_bridge/computed_style.rs`）：
  `image_set_to_css`——分辨率归一 dppx（`format_resolution_dppx`，缺省
  1.0，非折叠 calc 保形 FIXME）；painter effects/text 防御臂（背景/蒙版
  no-op、clip:text 纯色提取 continue）。

## 2. 单测与门禁

- css-parser **3,227P/0F**（+4 钉：canonical 页面形 29 组、invalid 页面形
  24 组、投影契约钉演进、NaN 幂等/多字节 URL 面）。
- style-system lib **2,538P/0F**；engine lib **4,009P/0F**（+2 computed
  串化钉：分辨率归一 5 断言、渐变+type+webkit 别名 3 断言）。
- fmt `--all --check` clean；clippy css-parser + style-system +
  engine(--features v8) `--all-targets -D warnings` 全零告警。

## 3. 受控 A/B（B0 = results-g44-b1 g44 提交态；B1 = results-g45-b1）

| corpus | B0（g44-b1） | B1（g45-b1） | 净 |
|---|---|---|---|
| css-images | 2941P/274F | 3087P/128F | **+146P/−146F** |
| css-values | 4041P/1546F/6T | 4038P/1549F/6T | **−3（attr 污点暴露，见 §5）** |

- 逐页（ab-g42.py，案数恒同 3215/5593；grand fixed +146 regressed 3 net
  **+143**）：image-set-parsing **F 114→0 全页清零**（144/144 Pass）；
  image-set-computed F 36→4（fixed +32）；css-values attr-security-animation/
  attr-security-transition/attr-universal-selector 三页 0F→1F。

## 4. 残面挂账（image-set-computed 余 4F，B0 既有失败非本轮回归）

- **content×image-set 支持门（2F）**：`content: image-set(url(…) 192dpi,
  linear-gradient(black, white) 1x)` CSS.supports false——content 属性自身
  解析面未接 image-set（css-content-3 `<content-list>` 含 `<image>`），
  需 content parse 臂另行接线。
- **calc 负分辨率折叠/钳制（2F）**：`image-set(url(…) calc(-1 * 1x))` 期望
  computed `0dppx`（css-values-4 负 resolution 计算值钳制），得 none——
  共享 CalcParser 数×分辨率乘法折叠或 computed 钳制缺口，且链上任一环
  拒收即整值落 none，需逐环核。

## 5. attr() 污点暴露（css-values −3，非代码回归）

三页机制同型：`div { --some-string: attr(data-foo); background-image:
image-set(var(--some-string)) }`——B0 时 image-set 不可解析恒 none（**碰巧
过**）；g45 image-set 可解析后 var/attr 替换链产出合法图像集，css-values-5
[#attr-security](https://drafts.csswg.org/css-values-5/#attr-security) 污点
传播（attr() 跨属性上下文经自定义属性注入 → 计算值期 invalid）未实现即
暴露。拒 var() 于 parse 期是 spec 错误方向（css-variables-1：含 var() 声明
parse 期恒合法），正修为独立 attr-security 刀（污点标志随替换链传播）。
attr-security.html / attr-security-if.html / attr-security-transition-interpolated
四页内其余 image-set/var/attr 组合案全数 Pass 未受影响。

## 6. 运行记录

- corpus 以 systemd service `zw-g45-corpus` 单实例发射（flock + 同树
  cargo/rustc cwd 检查 + CSS_CORPUS_TRANSCRIPT 唯一路径）；同 session 并发
  incarnation 同脚本竞速，先到者持锁执行，双套件各 exit=0，unit 尾态 failed
  为脚本收尾行截断（两段 corpus 本体 exit=0 佐证于日志），results-g45-b1
  JSON mtime 与日志时间戳互证单写者。
