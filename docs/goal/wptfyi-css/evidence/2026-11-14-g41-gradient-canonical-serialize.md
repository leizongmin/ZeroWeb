# g41 渐变声明值 canonical 串化刀

2026-11-14。css-images-4 [§serializing-gradients](https://drafts.csswg.org/css-images-4/#serializing-gradients)：linear/radial/conic-gradient 声明值写面 canonical 串化——配置前置、`in <method>` 后置、缺省法冗省、xyz→xyz-d65 别名展开、色标段 as-written。主收割面 gradient-interpolation-method-valid.html（B0 813F）。

## 1. 规则集（813 形 corpus 变换规则，JSON expected/got 对逐形定谳）

- **结构 canonical**：配置（角度/方向/`at <position>`/`from <angle>`）前置到函数名后，`in <method>` 紧随其后，色标段逐参 `, ` 连接 as-written。`||` 组合子方向词序保留 as-written（corpus 输入全水平前置）。
- **方法词归一**：全空间词表（lab/oklab/srgb/srgb-linear/xyz/xyz-d50/xyz-d65/display-p3/a98-rgb/prophoto-rgb/rec2020…）；`xyz` 串化展开为 `xyz-d65`（css-color-4 §9.1 别名）；hsl/hwb/lch/oklch 极坐标空间 hue 法（shorter/longer/increasing/decreasing）保留，shorter 缺省可省。
- **冗省**（#gradient-color-interpolation）：站含 predefined `color()` 色标 → 缺省法 oklab；全 legacy 站 → srgb；声明法==缺省法且 hue 缺省 → 整 `in` 子句省略。
- **渲染口径不变**（R2288）：未知空间/hsl/hwb/wide-gamut 渲染回退 Srgb；串化未识别 passthrough。

## 2. 实现与接线（最终收敛形态）

- **核心**（css-parser）：`parse_transform.rs` 渐变解析重构——`GradientSpecifiedParts { head, body, method }` 文本承载 as-written 片段（避免 ColorInterpolationSpace 枚举膨胀与 Copy derive 破坏；FIXME 色标级 canonical 归 computed 刀）；`parse_interpolation_method` 返回 `(space, hue, method_text, leftover)`，前缀/中缀/未命中三态，polar 分支三态（hue 词消费/畸形 passthrough/方法结束余量回填配置）；`specified` 三态 `(method.is_some() || !present).then(...)` 保 `in bogus` 契约。新文件 `serialize_gradient.rs`：`serialize_gradient_specified_value`（冗省判定 + 别名展开）+ `canonicalize_background_image_specified`（括号感知 split_top_layers，任一层非渐变/invalid 整值 None）。
- **写面单点接线**：`serialize_math.rs` `fold_decl_specified` background-image 臂 = `canonicalize_background_image_specified(v).or_else(|| fold_image_set_specified(v))`（g40 image-set calc 折叠回落，两臂互斥补全）。写面四站 + mutation replay 三站共享该分派。
- **shim 回调双通道**（grid_cssom G1 同型）：part06.js `_zwNormalizeDeclValue` background-image 臂 → callbacks.rs `__zw_gradient_normalize_decl` 回调 → 同一 Rust 单点；非渐变面原文 passthrough，回调未注册零漂移。
- 单测 5 钉：canonical_reorder（含 conic/radial/方向序）、method_redundancy_and_alias、passthrough_faces（in bogus/none/url/多层/invalid/light-dark）、identity_and_whitespace、fold_background_image_gradient_canonical_dispatch（分派臂 + image-set 回落 + None 原文）。

## 3. 双 spawn 竞态收敛与本轮事故

- **多重 spawn 爆发**：rate-limit 中断触发同 session 5 复苏位并发（21:38 + 22:05×2 + 22:07×2），任一身位在初版直臂（css_style_declaration named setter/setProperty 直接 fold）外并行实现同核函数的 shim 回调通道 + fold_decl_specified 臂。按孤儿纪律 SIGKILL 全部竞争身位（TERM 无效）、采纳自足脚本零重跑；写面架构**采纳收敛形态**（单点分派 + 双通道），初版直臂实测 runner 路径不生效已移除（css_style_declaration.rs 仅留分派指引注释）。
- **陈旧二进制事故（run 1 无效定谳）**：22:09:38 首轮 44s release 构建复用旧 engine rlib（fingerprint 漏检 css_style_declaration 编辑），corpus/探针 valid 页 585P/813F 与 B0 逐案恒等 → 判定旧态，结果作废。provenance 法证：nm 符号在（css-parser 新建）而行为 B0（engine 调用点未入二进制）。
- **run 2 收敛验证**：22:15 组合接线二进制，valid 页 1134P/264F 双探针窗复现（22:15:47 与 22:18:57 两独立窗恒等）+ 本轮独立 provenance 探针（当前二进制 1134/264）三重一致。

## 4. 受控 A/B 账（B0 = results-g40-b1 g40 提交态；B1 = results-g41-b2 干净重录）

| corpus | B0 | B1 | 净 |
|---|---|---|---|
| css-images | 887P/2328F | 1439P/1776F | **+552P/−552F** |
| css-values | 4041P/1546F/6T | 4041P/1546F/6T | **±0 零回归** |

- B1 口径注：初版 results-g41-b1 工件曾卷入本地合成探针页 zz-probe-g41-fold（cases=42，+4P/1F，探针页跑后已删不入库）——终账以探针删除后同二进制 41 案重录 results-g41-b2 为准（css-values 沿用 b1 终轮产物，字节级对照验证）。
- css-images 逐页：gradient-interpolation-method-valid **585→1134（+549）**、-computed 48→51（+3，computed 残面案见 §5）；其余页零漂移。
- REGRESSED=0：41 案键集逐页 DOWNS 空。
- 组合态 provenance：构建树含 34 在途 engine shim WIP（part01/part01f C4-revert，s438 在途 face）——css-images 档零交叉，双 corpus 键集逐案无第二页漂移佐证。

## 5. 残面清单（后续刀位）

- **G2 computed 刀**：gradient-interpolation-method-computed 881F——computed_style 通道无 canonical（探针页 computed 案实证：declared 已 canonical 而 computed 读回原文）+ 方向序/现代色保真/angle-in-method 归一。
- **A 刀 invalid 拒写面**：gradient-interpolation-method-invalid 292F（0P）——validate 白名单接入 + parse 严格性盘点（invalid 值现全放行，写面无拒写语义）。
- **解析缺口**：4-token 位置（`at left 10px top 50em`，parse_position_pair 仅 1-2 token）+ size-only radial 形；color-stops-parsing 60F；image-set-parsing 114F + image-set-computed 36F（6b 指派，C+D 刀）。
- 独立探针 anomaly：runner 对 color-stops-parsing/image-set-parsing 单页直跑零输出（corpus 路径正常计数），待查。

## 6. 门禁

css-parser lib tests 3217 全绿 0F（+dispatch 钉）+ engine lib tests 3959 全绿 0F + 双 crate all-targets clippy 0 error + fmt 净。engine all-targets 曾被 34 WIP part115:125 doc lint 阻断，本轮窗内由在途修复解除后复跑通过。
