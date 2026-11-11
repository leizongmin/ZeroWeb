# DC-2 light-dark() 图像上下文批（11-11 午后）

## 批次与提交

提交 `8c166641d`（14 文件 +472/−44，pre-commit guard PASS 14 文件 472 行零发现）：css-color-5
§funcdef-light-dark 图像上下文贯穿四属性（background-image / mask-image 共用、
list-style-image、border-image-source）：

- **css-parser**：三解析器 `LightDark { light, dark }` 变体（`parse_extended_visual.rs`
  BackgroundImageValue + `parse_extended_border.rs` ListStyleImageValue /
  BorderImageSourceValue 同构）——恰两参顶层逗号（`split_top_level_comma` + 右段复切
  验 arity）、支内 none 合法（#valdef-light-dark-none）、色值支/单参/三参/空参段经
  递归解析 `?` 自然拒绝。
- **style-system**：cascade 期按 `style.color_scheme_dark` 选支展开（`bg_image_value_to_computed`
  加 dark 参数 + `bg_image_branch_to_computed` 支内语义：嵌套 light-dark 递归选支、
  支内 None → 新变体 `TransparentImage`）——computed 期一次定案，`inherit` 继承的
  自然是选中支，动态 scheme 变化经 cascade 重算重选；background-image / mask-image /
  border-image-source 三臂接线，list-style-image 臂（apply.rs）内嵌 resolve 同语义。
- **engine**：`TransparentImage` 序列化 `image(rgba(0, 0, 0, 0))`（computed_style.rs
  三处：image_layer_list / list_style_image / border_image_source；区别顶层 none）；
  paint 五臂补 exhaustive（border.rs / effects.rs ×2 / text.rs / text_list.rs——透明
  图像同 none 不绘制）。
- **shim 写臂**（invalid 组）：part11-color.js `__zwImageNormalizeDecl` 浅校验
  （仅 image 四属性且整值顶层 light-dark( 前缀介入：恰两参+支形态白名单 none/url()/
  *-gradient()/嵌套 light-dark 递归；无括号单词=named 色/垃圾 ident 拒；rgb()/hsl()
  族色值函数拒）+ part06 `_zwNormalizeDeclValue` 接线（undefined=不介入 passthrough、
  null=拒）；白名单外函数形态（image-set()/cross-fade() 等不支持面）与多图层混合
  放行 passthrough 零回归。

## 测量

**共享树提前兑现（v11，zeroweb-2-70 记录轮，build 06:06 取工作树全态含本批在树
hunk）**：color corpus **10,808 → 10,840（净 +32），逐页零回归**，values 2,213P 零漂移。
逐页归属：

| 页 | v10 → v11 | 归属 |
|---|---|---|
| light-dark-image | 1/32 → 30/32（+29） | 本批（30P = 32 − 2 挂账，全数兑现） |
| system-color-compute | 25/27 → 27/27（+2） | 70 的 fill/stroke gCS 读出 hunk（同树在飞，非本批） |
| light-dark-currentcolor-in-color | 0/1 → 1/1（+1） | 同上（fill/stroke 面顺带兑现 currentcolor 序列化；归属待 70 提交定谳） |

**v12 终账**（本批提交 `8c166641d` 落库后自跑，results-v12/）：
待回填。

## 验证链

- 端到端单测 `test_get_computed_style_light_dark_image` 22 断言全绿（dark/light 选支、
  支内 none→image(rgba(0, 0, 0, 0))、linear/radial/conic/repeating 渐变 rgb() 规范化、
  嵌套 light-dark 递归、invalid（色值/三参/单参）声明拒回落 none、inherit 按父
  scheme 选支继承）。
- css-parser parse 形态表 3 测试（`test_parse_background_image_light_dark` /
  `_invalid` / `test_parse_list_style_border_image_light_dark`）全绿。
- engine 全量 3,837P / 7F（7F = master.md 已定谳 main 既有欠账组 img/svg settle +
  websocket 族，与本批零交集）；style-system 3,083P、css-parser 2,511P 全绿
  （共享树含 fe grid G2a 在飞 hunk 同态编译测试）。
- shim 25 part 拼接 `node --check` 语法验证过（part06.js 为拼接片段，单文件
  check 不适用）。
- clippy 三 crate：零新告警（唯一 warning 在 parse.rs:98 = fe grid 面文件）。

## 挂账（本批不追）

- **content 图像 light-dark 1F**（light-dark-image 页）：ContentValue 全链
  （parse/apply/序列化/computed 枚举）翻 match 与本批同体量，单独小批。
- **image-set()/cross-fade() 嵌套 1F**（同页 "nested image-set" 子测试）：图像函数
  解析面缺位（`image-set(-webkit-cross-fade(light-dark(...),...)...)`），需
  image-set/cross-fade 全套解析+序列化+内部 light-dark 替换。
- 一致性 dark FieldText 5F（UA 表单样式建模，沿前批挂账）。

## 协作纪要

多实例同树窗口（本 session + fe grid G2a + e2 + 70 fill/stroke）：本批 14 文件按
pathspec add + computed_style.rs **hunk 级拆分提交**（5 hunk 定谳：本批 3 处
TransparentImage 入库，70 的 fill/stroke 臂 @@ -336 与 fe 的 grid_line @@ -3415
留工作树各自提交）——`git diff` 导出 + `git apply --cached` 精准暂存，工作树他人
hunk 未裹走。号段：v11 先启先得归 70（其 build 已含本批树面 hunk 提前兑现），
v12 归本 session（fe 仲裁）。e2 曾误认本批为 fe 半成品申请接管，经 SendMessage
边界澄清后确认七处文件零改动让位。
