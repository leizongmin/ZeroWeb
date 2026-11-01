# M2-S217：image-map 命中面视口矩形修正 + object 不支持 image map 臂

- **日期**：2026-11-01
- **目标面**：misc 残面勘根续推——embedded-content 域 F 分布重勘根定刀
  usemap-casing 族（the-img-element 7F + the-object-element 8F = 15F，
  单根）
- **状态**：收口——R5384 红绿实证；靶语料 16/16 全 Pass（15F 精确翻绿
  + img 正案保持）；embedded-content 语料 A/B 共享键 1257/1257，漂移
  15 = 恰 15 Fail→Pass 全靶面，零 P→F 键集差 0；zero-engine 3676P+0F

## 勘根

- **入场**：s216 A/B 日志（s216-embedded-content-wip.log）F 分布下钻
  ——img 族 103F/canvas 53F 中单文件最大 = usemap-casing 15F（img 7 +
  object 8，同根）；forms 域大头 customizable select 族 171F 全处用户
  门禁（GB-20261031）不动。
- **失败形态**：两文件全部 15 负案 + object sanity 单跑详情 = `Cannot
  read properties of null (reading 'parentElement')`——**不是 map 匹配
  错，是 elementFromPoint 返 null**（语料断言 `element.parentElement`
  在 null 上抛）。
- **单根定谳**：s78 `_zwImgMapHitSel`（part06 elementFromPoint 的
  host-bridge 缺席臂）以 `(0,0) + IDL width/height` 近似 img 边界——
  runner 布局集成面就绪后 gBCR 有真实视口 rect，语料 img/object 文档
  流纵向堆叠（首元素 y≈8、后续 y≈108/208…），除首个元素外全部在旧边
  界检查（`y < h` = y < 100）即 `continue` → 命中遍历落空 → null。正
  案（img 首元素 #sanityCheck）恰在 (0,0) 近似域内故恒 Pass——正负案
  分叉即根因指纹。FIXME(image-map)「真实几何归 layout 集成面」的前置
  条件（布局 rect 可得）已成熟，本轮在该臂内落地。
- **object 面**：image map 处理模型 spec 只定义 img；object 不支持
  image map（语料标题钉「it does not support usemap」）——命中点应返
  object 自身。旧实现查询列表仅 `img` 且无回落臂 → object 面全 null。
- **map 匹配语义核实**：`nm === ref || mid === ref` 已是精确匹配
  （hash-name reference 解析 s78 既有），大小写负案不是本刀缺口；语
  料 Unicode 陷阱形态（ß/ĸ/İ/ﬁ 等）在精确匹配下天然满足。

## 施工

（shim 1 文件 + rust 测试 1 文件；语料零改动）

- **part06 `_zwImgMapHitSel` 重构**：
  ① 查询列表 `img` → `img,object`（文档序，qSA 逗号 selector list 标
  准语法——R5384 绿态同证可用）；
  ② 边界几何改 gBCR 真实视口矩形（`el.getBoundingClientRect()` 非
  零宽高即采用 left/top/width/height），零矩形回落 `(0,0)+IDL w/h`
  旧近似（bridge/reftest 无渲染环境 R5278 钉面同形保持）；
  ③ area coords 按 img 原点偏移换局部坐标（`x - left, y - top`——
  旧 (0,0) 近似下偏移恒 0 同形）；
  ④ 新 object 臂：命中即返 object 自身不查 usemap；
  ⑤ 注释同步（s217 修正语义 + FIXME 收窄为「浏览器渲染路径真实
  hit-test 归 layout 集成面」+ spec 锚 image-map-processing-model）。
  img 无 usemap 跳过（不自命中）旧形保持。
- **part09.rs R5384**（`test_img_map_hit_viewport_rect_object_arm_r5384`
  ）：dom_html 三元素纵向堆叠语料同形（img#a usemap=#m 精确命中 /
  img#b usemap=#sImPlE 对 simple+SIMPLE 双不匹配 / object#o
  usemap=#m）+ rect bridge mock（`__zw_getBoundingClientRect` 按
  selector 路由 "8,8,100,100"/"8,108,100,100"/"8,208,100,100"——
  part01.rs test_layout_geometry_e2e 先例同款，不在
  register_dom_callbacks 测试注册）。四断言：①map 精确命中 →
  AREA#ar；②偏移判别点（视口 x=105 超出 coords 值域、减原点后局部
  97 在域内 → AREA#ar——无偏移实现/旧 shim 均红）；③负案 → IMG#b
  非 null；④object → OBJECT#o。

## 验证

- **红绿实证**（R5384）：`git stash push part06.js` 还原跑红——②
  `left: "null" right: "AREA#ar"`（旧 shim 边界检查即 continue，旧版
  因目标行为失败）；stash pop 恢复跑绿——1 passed（四断言全过，
  qSA('img,object') 逗号路径同证）。
- 拼接 shim（15 part include 序）node --check SYNTAX_OK。
- zero-engine lib 全量 **3676P+0F**（3675 基线 + R5384；R5278 既有
  image-map 钉面 parse+elementFromPoint 两测试全绿——零矩形回落臂
  兼容性实证）。
- **靶语料 solo**（FILTER=usemap-casing）：**16/16 全 Pass
  MAKE_EXIT=0**（15F 精确翻绿 + img 正案保持），日志 evidence/logs/
  s217-usemap-target.log。
- **embedded-content 语料 A/B**（A = s217-embedded-content-wip.log
  修复在位；B = s216-embedded-content-wip.log 即 pre-s217 基线；LC
  ALL=C，(key,state) 对全测试名整行键 sort -u join）：共享键
  **1257/1257，漂移 15 = 恰 15 Fail→Pass 全落 usemap-casing 靶面**
  （非靶 F→P 0），**P→F=0，键集差 0**；A 侧 1038P/190F/30T（ΔF
  恰 −15）。B 侧既有 205F+30T（lazy-loading 族等）两侧同态。日志
  evidence/logs/s217-embedded-content-wip.log（MAKE_EXIT=2 = 失败
  存在正常形态）。
- guarded-clippy 0 警告；`cargo fmt -p zero-engine -- --check` EXIT=0。
- 其余域零触面：改动仅 elementFromPoint 缺席臂 img/object image-map
  路径，forms 域无 img/object 命中面，不重跑。

## 挂账

- img naturalWidth-naturalHeight-width-height 22F（SVG viewBox 内在
  尺寸/宽高比族——image-decoder 面，与 renderer-trust-boundary 流
  ③A image-decoder 切片工作面相邻，暂不入刀避免跨流碰撞，观察）。
- canvas focusability 002/003/004 ~14F（canvas 回退内容可聚焦族，
  tentative 语料，候选）。
- current-pixel-density/sizes/srcset/decode 族 ~29F（响应式图片面，
  候选）。
- part06 `_zwFieldsetDisableFocusReset` legend 扫描碎片共享（s215 承
  袭，观察）。
- part03 `_zwMElCheckValidity` FORM 分支短路形态（s210 承袭）。
- onblur flake 族持续采样（s209~s216 在册摆动键承袭）。
- beforeinput.tentative.html 既有面（录入事件族独立评估承袭）。
- 上游 customizable select 特性面 177F 挂起待拍板承袭（GB-20261031）。
- form owner table / parser form element pointer 缓行族（用户决策承
  袭）。
- form-nameditem override builtins 待决（承袭）。
- input-type-button click() 返回值面（共享通路高风险，未入刀，承袭）。
- 多进程 renderer SetDomSnapshot entries 化（s174 低优先承袭）。
