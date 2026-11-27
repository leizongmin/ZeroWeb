# s10k59 刀㊻b：san 残面 SVG animating URL 属性面订正——javascript-url 12F 清零＋svg-animate 15F→1F

日期：2026-11-27。刀体 commit `22c2c9a47`（part13-sanitizer.js 单文件，34 增 24 删）。guard 扫描 PASS（0 发现 0 提醒，绑定 amend 后字节）。刀号 b 后缀消歧——「刀㊻」已被 html-semantics 流 `b3dca53f0` 占用（撞号三查：commit 标题 grep 全史 12 处均属他流）。

## 刀位与基线

控制面点名簇：san 残面 javascript-url 12F（上轮基线 san59＝**713P/65F**，页簇 census：svg-animate 15F、javascript-url 12F、sethtml-safety 9F、custom-elements-is 8F、declarative-shadow-root 7F…）。

## corpus ground truth（built-in-animating-url-attributes-list，12 变体全读）

判据页 `sanitizer-javascript-url.html`（html5lib-testcases，8 #data 块 × setHTML/parseHTML ＝ 16 子测）期望树：

| 元素 | attributeName 值 | 期望 |
|---|---|---|
| `animate` ×2（href / xlink:href） | 剥 | 元素保留、attributeName 剥除 |
| `animateMotion` ×2 | 保留 | **元素与属性全保留** |
| `animateTransform` ×2 | 剥 | 元素保留、attributeName 剥除 |
| `set` ×2 | 剥 | 元素保留、attributeName 剥除 |

姊妹页 `sanitizer-svg-animate.html`（11 testcase × setHTML/setHTMLUnsafe ＝ 22 子测）期望分档：

- setHTML（safe）：`set` + attributeName∈{href, xlink:href, ylink:href（未声明伪前缀）, zlink:href ×4（声明/未声明、父级声明、引用侧变体）} → **remove**（剥 attributeName）；hreflang / xref / xlink:href:x（双冒号）+ javascript: to → **inert**（保留）
- setHTMLUnsafe：attributeName 全保留（唯一例外首 case 期望 animate＝SMIL 动画真实发生）

## 探针定谳（zz-s10k59-probe A1-A10，DIAG-by-reject，跑后已删）

| 探针 | 刀前观察 | 结论 |
|---|---|---|
| A1 setHTML `<svg><set attributeName="href">` | `<svg><set attributeName="href">` 全保留 | 空配置下 set 不剥——javascript-url set 4F 挂因 |
| A5 animateMotion | attributeName 被剥 | animateMotion 误剥——4F 挂因（期望保留） |
| A6 animate | `<svg></svg>` 整删 | animate 误整删——4F 挂因（期望保元素剥 attr） |
| A9 显式 config（attributeName 在 allowlist） | `<set to="x" begin="0s">` **attributeName 被剥** | allow 条目 `"attributeName"` vs lower 化 local `attributename` **case 失配**＝svg-animate inert/unsafe 档 over-strip 根因 |
| A10 同树 selector `set[attributeName]` | null | 与 A9 吻合——attr 真被剥，**非 selector 缺陷** |
| A2/A3/A4/A7 | `set`/`set[attributeName]`/`svg > set` 在 detached div 上全部正常返元素 | selector 引擎无恙 |
| A8 `set[attributename]`（小写） | null | selector attr 匹配 case 敏感（行为正确） |

## 三重订正（commit 22c2c9a47）

1. **元素 verdict 层**：删 `animate` 整删臂（`_zwSanAnimHrefTarget` 随之零引用删除）——corpus 期望树方向订正：animate 元素保留。
2. **属性层第 4 臂重写**：`safe` 门内 SVG `animate`/`animateTransform`/`set`（animateMotion 换员出单）＋ attributeName 值 **exact 不 trim**（allowed/2 `" href "` 带空格对照面期望保留）＋ `href` 或 `*:href` 后缀（ylink:href/zlink:href 未声明伪前缀 remove 档；双冒号 `xlink:href:x` 与 `hreflang`/`xref` 自然排除）。unsafe 方法全保留。
   spec：https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#built-in-animating-url-attributes-list
3. **allow 条目 case 不敏感**：新增 `_zwSanSameAttrCi`（双 lower 比较），`_zwSanAttrAllowHit` 与第 5 臂全局/per-element allow 查换用；remove 方向保持 exact。负钉面普查（sanitizer-api 语料 mixed-case 条目仅 `"attributeName"` 一处）无 case 敏感反例。

## 判读（定向两页 solo → san 全臂 → chain59 兜面）

| 臂 | 锚 | 实跑 | 归因 |
|---|---|---|---|
| javascript-url 页 | 12F | **42P/0F 全绿**（中途 allowed/2 回潮 2F＝值 trim 误剥，去 trim 后清零——修正过程留痕） | animating 组 16 子测全 Pass |
| svg-animate 页 | 7P/15F | **21P/1F** | 余 1F＝unsafe animate 档 `a.href.animVal !== baseVal`（SMIL 动画引擎缺口，scope 外挂账） |
| san 全臂 | 713P/65F | **739P/39F/0T** | ΔP+26＝ΔF−26 闭合 |
| san F 四象限 comm | — | 翻绿 26 全落两目标页（12+14）、**新回潮 0**、common 39 零扰动 | — |
| tt59（chain59） | 2290P/14F/18T（tt58 真实池） | **2290P/14F/18T** | 零漂移——part13 过滤管道不在 TT enforced 解析 sink 依赖路径 |
| ref59（chain59） | 725/725 | **725/725**（Layout 521/521＋Text 204/204） | 零漂移 |
| dom 臂 | — | 跳过 | part13 不在 dom 臂依赖路径（sanitizer-api 语料在专用入口 `testharness-sanitizer-api`，dom 臂零 san 行） |

## bin 字节绑定订正

chain59 发射断言 bin `9e90cc38add0`＝commit `404db3885` 字节（首轮刀体）。撞号消歧 amend 后终版 `22c2c9a47` 与 404db3885 **仅注释差**（刀号字符 ㊻→㊻b，4 处），shim 语义零差——判读仍有效（先例：刀② chain13 bin 订正、刀㊺d 仅注释差验证形）。

## 残相挂账

- svg-animate 页 unsafe animate 档 1F：需 SMIL 动画引擎（`a.href.animVal` 偏离 baseVal）——动画引擎 scope 外，本页可达上限 21/22。
- san 残池 39F 后续刀位：sethtml-safety 9F、custom-elements-is 8F、declarative-shadow-root 7F、其余簇 census 待下轮。
- census 84,332＋26＝84,358 待下次专跑并入。
