# g45b attr 污点经自定义项传播刀

2026-11-15。driving: g45 挂账 css-values attr-security 3F——
attr-security-animation / attr-security-transition / attr-universal-selector
各 1F，期望 computed `background-image: none` 得
`image-set(url("https://does-not-exist.test/404.png") 1dppx)`。均 B0（g44）
既有失败非 g45 回归（g45 image-set 文法位放行前被文法拒意外遮蔽，放行后
暴露）。

## 1. triage

- 页面链（三页同构）：`--s: attr(href)`（custom property 内 attr()）+
  `background-image: image-set(var(--s))`——attr() 产物（元素属性文本）经
  var() 进入取 URL 属性。css-values-4 §attr security：attr() 产物不得作为
  图像/URL 源，期望整条声明 IACVT 回落初值 none。
- **根因（代码定谳）**：attr 替换发生在**预级联站**（style-system
  compute_element_style_internal 的 author_attr_active 块，
  substitute_attr_values_deferring）——`--s: attr(href)` 替换成功后词痕即刻
  消失，值文本已为 URL。既有两站均失察：预级联站只拒**直取**属性上的
  attr()（tainted && fetches_url(prop)，custom prop 不在 fetches_url 集）；
  4.75 重裁站只认**剩余 attr 词**（值里已无 attr 可认）。
- 首版补丁（step 4.65，级联文本上找 attr 词自定义项 + `var(` 前缀引用）
  未生效：cascaded 到达该站时词痕早已消失，`has_attr_word` 恒空。b2/b3
  两轮 corpus 4038P 同形即为该版（含构建竞态因素，见 §5）——未提交，
  本轮以重写版为准。

## 2. 实现

- **污染名登记**（预级联站 Owned 替换臂）：替换成功且 `tainted` 且 prop
  以 `--` 开头 → 记入 `tainted_customs`（hints 以 "" 调用不在面内）。
- **var() 取数拒收**（五路 retain_mut 之后、仍处 author_attr_active 块）：
  `tainted_customs` 非空时，对取 URL 属性
  （[`attr_target_fetches_url`]：background / background-image /
  border-image / list-style-image / cursor）声明值引用任一污染名者整条
  丢弃（IACVT 回落初值 none）。引用匹配 `var_refs_custom`：`var(` 后跳
  空白取 custom-ident 全等比对（`var(--s2)` 不误伤 `var(--s)`）；hints 与
  `--` 项自身不在丢弃面。跨元素继承同名污染项不在本站视野（FIXME：
  污染旗标随 custom map 传递后按消费站收口）。

## 3. 单测与门禁

- style-system lib **2,539P/0F**（+1 钉
  `test_attr_taint_via_custom_property_drops_url_target`：正钉污染链
  background_image 丢弃为空 + 负钉未引用污染项不牵连）。
- fmt `--all --check` clean；clippy `-p zero-style-system --all-targets
  -D warnings` 零告警。
- commit `c65741a62`（style-system lib.rs +42、tests/basic.rs +55，显式
  pathspec 提交——同暂存区共存对端 g46 批次 8 文件，防卷入）。

## 4. 受控 A/B（B0 = results-g46-b1 修前二进制；B1 = results-g46-b2）

| corpus | B0（g46-b1） | B1（g46-b2） | 净 |
|---|---|---|---|
| css-values | 4038P/1549F/6T | **4041P/1546F/6T** | **+3P/−3F** |
| css-images | 3091P/124F | 3091P/124F | **±0 零回归** |

- 逐案（results-g46-b1 vs results-g46-b2）：attr-security-animation /
  attr-security-transition / attr-universal-selector 3 案 Fail→Pass，
  全页 NEWFAIL=0（零新回归）。对 g44-b1 基线 css-values 4041P/1546F =
  完全回零（G0=4041P/1546F）。
- 归因：04:06:48 → 04:17:59 二进制间唯一内容差为本修（touch 强制
  style-system 重建；css-parser/engine 未重编译保持上代）——+3 全落本刀。

## 5. 运行记录与法证教训

- **陈旧二进制三连**：b2/b3 两轮 corpus 复用 04:06:48 二进制（log 零
  `Compiling` 行）——g46 构建起跑早于本修 lib.rs 终态落树（04:06:20），
  cargo 已按旧快照编译 style-system；此后提交不改变工作树内容，mtime
  判 fresh 不重建。**mtime 相对源码编辑时刻的法证存在盲区**（构建中途
  源码落树、产物晚于源码但内容为旧快照），行为探针（页账 4038P 恒同）
  才是终判。解除：`touch` 源文件破 freshness → g46b2 轮真重建（产物
  04:17:59）。
- corpus 以 systemd service 单实例发射（flock + 同树 cargo/rustc cwd
  守卫）：本侧 b4 unit 两次被守卫正确弹掉（对端 zw-g45b4-build 在飞），
  采认对端 g46b2 磁盘终态为本刀 B1，未双跑。

## 6. 残面挂账

- 跨元素继承链污染（父元素 `--s: attr(href)`、子元素
  `background-image: var(--s)`）本站不覆盖——FIXME 挂账（corpus 暂无
  暴露页）。
- css-images 124F 旧账：position-invalid 5F / conic-calc 3F 细形（g43 起）
  + 其余既有页。
- content image-set 渲染面 FIXME（g46 挂账承接）。
