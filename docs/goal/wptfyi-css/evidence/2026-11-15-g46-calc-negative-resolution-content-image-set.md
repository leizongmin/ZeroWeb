# g46 calc 负分辨率存储形回读 + content×image-set 刀

2026-11-15。driving: g45 挂账 image-set-computed 残 4F——`calc(-1 * 1x)` 期望
computed `0dppx` 得 none（2F）与 `content: image-set(...)` CSS.supports false
（2F）。均 B0（g44）既有失败非 g45 回归。

## 1. triage

- **calc 负分辨率 "none" 根因（探针实证）**：css-parser 四点全绿（fold
  `Some(Some(-1.0))`、parse Some、decl_invalid false、canonical
  `calc(-1dppx)`）——丢属性发生在**存储形回读**：写面 canonical 化存的
  `calc(-1dppx)` 求值期再解析时，calc 内一元负号维度字面被 `parse_factor`
  neg 重写落通用 `0 - x` 臂 → `BinaryOp(Number(0), Sub, Resolution)` → 纯
  Resolution 维度门拒裸 Number 叶 → fold None → 选项 invalid → 整值 parse
  None → 属性不应用 → computed "none"。Length 叶有保族重写先例
  （`0px - x`），Resolution 缺同等臂。
- **content×image-set**：parse_content `<image>` 域（url/gradient/filter）缺
  image-set 臂（css-content-3 #content-property：content-list 接受
  `<image>`）；写面门（g45 part06 content 含 image-set( 子串路由）与回调层
  判定器均通——decl_invalid 层分割 `split_top_layers` 括号感知，多选项
  image-set 恒单层不腰斩。

## 2. 实现

- **共享解析器 neg 保族重写**（css-parser `values/types.rs` `parse_factor`）：
  neg 臂加 `CalcExpr::Resolution(_)` → `Resolution(0) - x`（css-values-4 §8
  维度一元负号，镜像 Length `0px - x`）；语义分界保持——字面
  `calc(0 - 1dppx)`（number−resolution 混型）不经本臂、纯度门仍拒（钉测
  断言 `fold("0 - 1dppx") = None`）。语料暴露面核查：css-values 页面
  `calc(-<n><res单位>)` 形零命中，爆炸半径 corpus-zero。
- **computed 负值钳 0**（engine `computed_style.rs`）：`image_set_to_css`
  分辨率提取后 `dppx.max(0.0)`（css-values-4 §6.4 无负 `<resolution>`；WPT
  oracle `calc(-1 * 1x)` → `0dppx`；字面负值 parse 期已拒，此处仅收 calc
  折叠负值）。
- **content `<image>` 域**（css-parser `values/parse_extended.rs`）：
  `ContentValue::ImageSet(ImageSetValue)` 变体 + parse 臂 +
  `parse_content_image` 域扩展（filter() 内层同收）；单实现点——新增
  `parse_image_set_value`（pub，css-parser `values/parse_extended_visual.rs`）
  承载别名剥除/选项文法，`parse_background_image` 层臂改调同源（单实现）。
- **style-system 直通**：`ContentComputedValue::ImageSet` 变体 +
  `apply_advanced` 双 match 臂直通（主臂 + filter 内层递归臂）。
- **engine computed 串化**：`content_to_css` ImageSet 臂复用
  `image_set_to_css`（分辨率 dppx 归一/渐变 computed 形同一实现点）。
  paint 面 ContentComputedValue 既有兜底臂天然覆盖（无 E0004 新增臂）。

## 3. 单测与门禁

- css-parser **3,227P/0F**（+1 钉 `test_g46_negative_calc_resolution_roundtrip`：
  负维度/负乘积折叠、原始形 parse+decl_invalid、canonical 存储形回读、
  canonical 投影、混型分界 `0 - 1dppx` 仍拒、content image-set 选项模型
  （192dpi Plain 保形 dppx 2.0 + 渐变 1x）、`-webkit-` 别名、负字面拒收、
  filter() 内层、单实现点同源）。账面更正：g45 证据记 3,227P 实为 3,226P
  （HEAD `git show | grep -c '#\[test\]'` 实证，±1 转抄漂移；本轮 HEAD
  3,226 + 1 钉 = 3,227）。
- style-system lib **2,538P/0F**；engine lib **4,013P/0F**（+2 钉：负 calc
  钳 0 双形、content image-set computed oracle 串化）。
- fmt `--all --check` clean；clippy css-parser + style-system +
  engine(--features v8) `--all-targets -D warnings` 全零告警。
- bench-gate 未跑（neg 重写单臂增量，同 g42-g45 刀轮口径，收口轮统一补）。

## 4. 受控 A/B（B0 = results-g45-b1 g45 提交态；B1 = results-g46-b1）

| corpus | B0（g45-b1） | B1（g46-b1） | 净 |
|---|---|---|---|
| css-images | 3087P/128F | 3091P/124F | **+4P/−4F** |
| css-values | 4038P/1549F/6T | 4038P/1549F/6T | **±0 零回归** |

- 逐页（ab-g42.py，案数恒同 3215/5593；grand fixed +4 regressed 0 net
  **+4**）：image-set-computed **F 4→0 全页清零**（image-set 双页 parsing
  0F + computed 0F 收口），其余页零漂移，css-values 恒同（attr-security 3F
  未动如预期）。

## 5. 残面挂账

- css-images 124F 旧账：position-invalid 5F / conic-calc-angle-invalid 3F
  细形（g43 起）+ 其余既有页。
- css-values attr-security 3F：css-values-5 #attr-security 污点传播刀
  （g45 证据 §5 定谳，非本轮回归）。
- content image-set 渲染面（替换图像绘制）FIXME 挂账（computed 串化已通，
  paint 兜底臂 no-op）。
- g45 账面 ±1 计数漂移更正见 §3（不回改 g45 证据原文，本轮账面为准）。

## 6. 运行记录

- corpus 以 systemd service `zw-g46-corpus` 单实例发射（flock + 同树
  cargo/rustc cwd 守卫 + CSS_CORPUS_TRANSCRIPT 唯一路径）；runner 二进制
  重建 62s，provenance 以 mtime 法证（binary 04:06:48 新于全部源文件，
  0 newer——g46 未触 .js，字符串标记法不适用）；双套件 exit=0，单位尾态
  inactive 正常。他 clone 进程在案（ZeroWeb cargo test、ZeroWeb-3 editing
  battery）不属本树，无竞争。
