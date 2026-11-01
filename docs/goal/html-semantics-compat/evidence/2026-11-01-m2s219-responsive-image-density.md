# M2 s219 — 响应式图片密度面（density-corrected intrinsic dimensions）

日期：2026-11-01。切片号 s219（承 s218）。

## 入场定刀位

s218 控制文档下一步候选「current-pixel-density·sizes·srcset·decode 响应式
图片族 ~29F 候选」。本刀收口 current-pixel-density 密度子面。

## 勘根

s218 A/B 日志 embedded-content F 分布下钻 + 语料单跑分型：

- **current-pixel-density/basic.html 11F** = 8 PNG 密度案例（本轮靶面——
  spec density-corrected intrinsic dimensions 密度从未进入 Rust
  image_sizes 面，clientWidth 恒 intrinsic 256）+ 3 SVG data-URL（intrinsic
  缺失 decoder 耦合，挂账不动）。
- error.html 7F = 破图 alt 渲染（layout 流域，挂账）。
- sizes-auto = 文件级 Timeout（68 案未上报独立面，挂账）。
- parse-a-srcset "data:,a " 1F = URL serializer 非 ASCII 百分号编码
  （`_zwResolveFetchUrl` 面爆炸半径大不入刀，挂账）。

三缺口定刀：① shim x 描述符非有限密度弃候选（parse-a-pixel-density-
descriptor）；② shim natural-dim 密度守卫放行 +Infinity（sizes 0px → 密度
w/0 = ∞ 合法计算态 → dim/∞ = 0）；③ Rust `build_img_intrinsic_all` 密度
折算（per-Node 挂点，webview URL 解码缓存零触——密度只折 layout 固有尺寸
map，不进 fetch/decode 键）。

## 施工（shim 2 文件 + rust 3 文件，语料零改动）

- **part03.js**（~2512）：x 描述符解析 `dv` 非有限（`9e999…x` →
  parse +Inf）→ `error = true` 弃候选（spec parse-a-pixel-density-
  descriptor 有限性门；与 shim 既有 isFinite 门同口径收窄）。
- **part04.js** `_zwImgNaturalDim`（716-774）：resourceState 臂（733）与
  host 快照臂（759）两处密度守卫 `isFinite(d) > 0` 形改 `d > 0`——+Infinity
  放行（dim/∞ = 0），NaN/≤0 仍回退。
- **part04.js** `__zwFireStaticImgSettle`（1187，勘根后探针补刀第四处）：
  静态 img settle 密度入账 `isFinite(_zwImD)` 拒 Infinity → `_zwImDen` 恒
  1 → resourceState.density 丢密度。同改 `> 0` 放行。此点为首轮三处漏网
  ——靶语料 i0px 案 width/naturalWidth 恒 256 定谳（探针分层面拆断言 +
  `__zwSelectImageSource5277`/`__zwCreateSourceSet5277` 链值探针锁定断点
  在 settle 入账非选择链）。
- **extract.rs** 新 `img_selected_pixel_density(doc, img_id, vw, vh)`：
  srcset/sizes 密度解析+选择最小镜像（候选文法同 shim：w 纯数字非零、x
  有限非负（`parse_srcset_real` 拒 '+' 前缀 + isFinite）、h 兼容须 width、
  描述符冲突弃候选；sizes 顶层逗号括号深度切分 + px/vw/vh/vmin/vmax/裸 0
  字面值，media 前缀项跳过回落 100vw；密度归一 w/size + default src 补挂；
  同密度去重 + ≤DPR 最大否则最小）。**全候选非法且 src 缺席 → 返 +Inf**
  （无选中源 → broken → dim/∞ = 0——corpus '9e999…x' clientWidth 0 面；
  有 src → None 走 default source 密度 1 不折算）。
- **pipeline/mod.rs** `build_img_intrinsic_all`：img 循环 effective src 命
  中 sizes/no_ratio 后接密度折算（f32 除；no_ratio 为 `(Option,Option)` 逐
  轴 map，None 维保持 undefined）。比率 map 不折（(w/d)/(h/d)=w/h 不变）。
- **tests/pipeline.rs** R5386 `test_img_density_corrected_intrinsic_scaling_r5386`：
  十 img 钉面——1.6x→160、2x→128、10000x→0.0256、'9e999…x'→0（broken）、
  512w+256px→128、256w+512px→512、256w+1px→1、sizes 0px→0（+Inf）、全候
  选非法但有 src→256（default source 密度 1）、plain src→256。

## 验证

- shim 拼接 15 part → node --check SYNTAX_OK。
- R5386 红绿：`git stash push extract.rs mod.rs` 跑红（1.6x got 256x256
  断言失败——旧版因目标行为失败）；pop 跑绿 1 passed。
- zero-engine 全量 **3684P + 0F**（s218 基线 3677 + R5386 + 并行会话
  main 6 案；零回归）。日志 evidence/logs/s219-zero-engine-full.log。
- 靶语料 solo（FILTER=current-pixel-density）**8P/13F**：8 密度案例全翻绿
  （1.6x→160/2x→128/1x/512w+256px→128/256w+512px→512/256w+256px/256w+1px
  →1/plain→256，width/clientWidth/naturalWidth 三面同绿）；余 13F 分型 =
  3 PNG expect-0 案（10000x/9e999…x/sizes 0px——width/naturalWidth 半面
  已绿，clientWidth 红于 layout 1px 钳，见挂账）+ 3 SVG data-URL + 7
  error.html。日志 evidence/logs/s219-density-target.log。
- **探针定谳记录**（临时 probe 文件已删，输出存 evidence/logs/s219-probe.log）：
  i0px 三面拆分（width 256→修后 0、naturalWidth 同、clientWidth 恒 1）+
  i9e（width/naturalWidth shim 面原生 0、clientWidth 256→修后 1）+
  i10000x（width/naturalWidth 0、clientWidth 1）+ rect 探针（i0px 盒实值
  1px 非 0px）+ 选择链值探针（sizes parse 0 / created density Infinity /
  picked density Infinity / currentSrc 同步）。
- embedded-content 域 A/B（A = s219 修复后、B = s218 基线日志）：共享键
  **1257/1257**，漂移恰 **5 F→P**（1.6x/2x/512w+256px/256w+512px/256w+
  1px——恰密度非退化 PNG 五案），**P→F = 0**，键集差 0。日志
  evidence/logs/s219-embedded-content-wip.log。方向注记：join A B 后
  `$2=="Pass" && $3=="Fail"` = F→P 改善（$2=A 侧），复核口径与 s218 同。
- clippy 0 警告（needless_borrow + collapsible_if 两处修复后）fmt 过。

## 性能注记

密度解析入口 `doc.get_attribute(img_id, "srcset")?` 早退——srcset 缺席 img
零成本；srcset 存在时 O(attr 长) 单遍。zero-engine 无 bench 目标覆盖本面，
不做 bench-gate（渲染关键路径未触——折算发生在 build_img_intrinsic_all
既有 img 循环内，无新增遍历）。

## 挂账新记

- **layout-engine 替换元素零/亚像素固有盒钳 1px**（sizing.rs:2899-2900/
  3136 `w.max(1.0)`）——corpus current-pixel-density 3 PNG expect-0 案
  （10000x/9e999…x/sizes 0px）clientWidth 1≠0 唯一根；width/naturalWidth
  已绿。渲染流车道不入刀，移交 layout-engine 侧勘根（Chrome clientWidth 0）。
- error.html 破图 alt 渲染 7F（layout 流域，承勘根）。
- sizes-auto 文件级 Timeout（68 案独立面）。
- parse-a-srcset "data:,a " 1F（URL serializer 非 ASCII 百分号编码）。
- SVG data-URL intrinsic 3F（与 img naturalWidth SVG viewBox 族 22F 同源
  decoder 耦合，renderer 流 ③A 相邻避碰撞）。

## 承袭挂账（不动）

img naturalWidth SVG viewBox 族 22F；svg-img-density-vs-zoom 7F；decode
2F；update-the-image-data 簇/img-picture-ancestor 4F/update-media/
source-relevant-mutations/update-the-source-set（选源响应面另勘根）；
lazy-loading 族 ~24F；part06 `_zwFieldsetDisableFocusReset` legend 扫描
共享（s215）；part03 `_zwMElCheckValidity` FORM 分支短路（s210）；onblur
flake 族采样（s209~）；beforeinput.tentative；非 canvas 面一般渲染门
FIXME(rendering)（s218）；flat tree slot 细化 FIXME(flat-tree)（s218）；
用户门禁三项（GB-20261031 customizable select 177F / form-nameditem
override builtins / form owner table 缓行族）。
