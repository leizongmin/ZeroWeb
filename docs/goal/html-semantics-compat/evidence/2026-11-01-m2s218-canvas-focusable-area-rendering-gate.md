# M2-S218：canvas 回退内容 focusable-area 渲染面门

- **日期**：2026-11-01
- **目标面**：embedded-content 残面续推——the-canvas-element
  canvas-descendants-focusability 族 19F（001 渲染 canvas 负案 4F + 002
  hidden canvas 5F + 003 hidden 组 5F + 004 shadow host 5F，单根）
- **状态**：收口——R5385 红绿实证；靶语料 29/29 全 Pass（19F 精确翻
  绿 + 005/006 既有 Pass 保持）；embedded-content 语料 A/B 共享键
  1257/1257，漂移 19 = 恰 19 Fail→Pass 全靶面，零 P→F、键集差 0；
  zero-engine **3677P+0F**（3676 基线 + R5385）

## 勘根

- **入场**：s217 账本挂账候选（canvas focusability 002/003/004 ~14F +
  001 同族 5F）。六文件全读：001/002 常态、003/004 tentative，断言模
  式统一 = `element.focus()` 后 `document.activeElement !== element`
  （负案红 = 不该聚焦的当前可聚焦）。
- **失败形态**：001 渲染 canvas 内裸 section/div/span/a（无 tabindex）
  恒可聚焦（4F）；002 `<canvas hidden>` 内全体（含 button/tabindex/a
  [href]）恒可聚焦（5F）；003 渲染 canvas 内 hidden 组恒可聚焦（5F，
  display:contents 组既有 Pass）；004 canvas 内 shadow host（attach
  Shadow + 命名 slot、light 子无 slot 属性）light 子恒可聚焦（5F）。
  solo 002 失败明细：5F 全焦点断言红，setup 面 `getClientRects()
  .length === 0` 断言不红——layout 对 canvas 子代无盒记录，rect 面真
  实反映。
- **单根定谳**：part04 focus() 只有 disabled 门（s212）+ T55/R114 失
  连臂，无 focusable area 渲染判定——spec focusing steps 对非 focus
  able area no-op，focusable area 要求「being rendered 或 being used
  as relevant canvas fallback content」+ 固有可聚焦核心。canvas 回退
  豁免语义（canvas.html#being-used-as-relevant-canvas-fallback-cont
  ent）：渲染 canvas 的回退内容可聚焦（button/a[href]/tabindex 豁免
  「未渲染」）；未渲染 canvas 内全不可聚焦；canvas 内 display:none 链
  不可聚焦、display:contents 可聚焦；shadow host light 子不在 flat
  tree 不可聚焦。
- **rect bridge miss 约定核实**（callbacks.rs R150）：真实 bridge
  miss → 空串（selector 无 NodeId 或 snapshot 无条目），hit 才有
  "x,y,w,h"——零盒 hit 与 miss 可区分，支撑「无布局记录不可判」放行
  语义。
- **首版设计回归与修正**：首版把固有可聚焦核心拒绝放 canvas 面之外
  （非 canvas 零盒 hit 判 false），zero-engine 全量 3 红（T55/T57/R4
  600）——持久化测试上下文 rect mock 泄漏使门「可判」，无 tabindex 的
  div/body focus 目标被核心臂全拒、焦点迁移序断言空。修正：**判定面
  收进 canvas 回退候选（最近 canvas 祖先存在）**，非 canvas 元素恒
  undefined 放行（一般「being rendered」门无语料覆盖，FIXME(render
  ing) 在册），T55/R114/T57 焦点迁移契约不受扰，语料 19F 面不受影响。

## 施工

（shim 1 文件 + rust 测试 1 文件；语料零改动）

- **part06 `_zwFocusableAreaGate(sel, handle)`**（`_zwIsFocusableSel`
  之后新增，同域内聚）：返 true/false/undefined。判定序：
  ① rect bridge 缺席或 handle-only 身份 → undefined 放行；
  ② `closest('canvas')` 无 → undefined 放行（非 canvas 面本轮不判）；
  ③ 固有可聚焦核心 `_zwIsFocusableSel`（同 part 谓词，IIFE 提升）→
  false 拒（回退候选面裸无 tabindex 元素恒非 focusable area）；
  ④ flat tree 走查：祖先链 `_shadowRoots[ancSel]` shadow host（键 =
  hostSel，同 part05 flat-tree 上溯先例）→ false；
  ⑤ canvas 渲染判定：canvas 自身 gBCR 非零 → 渲染中；miss/零 →
  false（后代非 relevant fallback）；
  ⑥ display:none 链走查（自身 → canvas 独占路径）：hidden 属性 + 计
  算 display 双通道（gCS 动态通道），display:contents 非 none 不截断
  → 全过返 true。
  FIXME(flat-tree)：shadow host 判定未做 slot 分配细化（默认 slot 重
  纳入 light 子），有 host 祖先即判出——语料钉 named-slot 形态。
- **part04 focus() 插门**（T55/R114 失连臂后、`_navMarkFocusChanged`
  前，连接面专用）：`_zwFocusableAreaGate(sel, handle) === false` →
  return（activeElement 不动、不派焦点序）；undefined 放行保既有环
  境契约；失连臂先行 return 不受扰（detached 焦点所有权契约优先）。
- **part09.rs R5385**（`test_focusable_area_rendering_gate_r5385`）：
  dom_html 三 canvas 同形语料（cv1 渲染：btn1/sec1/display:contents
  dc1；cvHidden hidden：btn2；cv3 渲染：hidden tabindex hid1 +
  shadow host host3>span1）+ rect mock 只给 canvas 视口 rect（cv1
  8,8、cv3 8,300 各 300×150；canvas 子代与 hidden canvas 空串 = 真实
  bridge miss 约定）。attachShadow + 命名 slot 建树（004 同形）。六
  断言：①btn1=true（回退豁免正案）；②sec1=false（核心拒绝）；③dc1
  =true（display:contents 不截断）；④btn2=false（canvas 未渲染）；
  ⑤hid1=false（display:none 链）；⑥span1=false（shadow host light
  子不在 flat tree）。

## 验证

- **红绿实证**（R5385）：`git stash push part04.js part06.js` 还原跑
  红——② `sec1=true`（旧 shim 无门恒可聚焦，旧版因目标行为失败）；实
  证后 stash pop 恢复跑绿——1 passed（六断言全过）。
- 拼接 shim（15 part include 序）node --check SYNTAX_OK。
- **首版回归捕捉**：首版全量 3674P+3F（T55/T57/R4600 焦点迁移序红）
  → 判定面收 canvas 回退候选修正 → 全量 **3677P+0F**（R5278 既有
  image-map 钉面、T55/T57/R4600 焦点契约、模态 inert 门全绿）。
- **靶语料 solo**（FILTER=canvas-descendants-focusability）：**29/29
  全 Pass MAKE_EXIT=0**（19F 精确翻绿 + 005/006 sandboxed iframe
  srcdoc 形态既有 Pass 保持），日志 evidence/logs/s218-canvas-target
  .log。
- **embedded-content 语料 A/B**（A = s218-embedded-content-wip.log
  修复在位；B = s217-embedded-content-wip.log 即 pre-s218 基线；
  LC_ALL=C，(key,state) 对 sort -u join）：共享键 **1257/1257，漂移
  19 = 恰 19 Fail→Pass 全落 canvas-descendants-focusability 靶面**
  （非靶 F→P 0），**P→F=0，键集差 0**；A 侧 1057P/171F/30T（ΔF 恰
  −19）。B 侧既有 190F+30T（lazy-loading 族等）两侧同态。日志
  evidence/logs/s218-embedded-content-wip.log（make 报错 = 失败存在
  正常形态）。
- guarded-clippy 0 警告；`cargo fmt -p zero-engine -- --check` EXIT=0。
- 其余域零触面：门仅作用「最近 canvas 祖先存在 + rect bridge 在位」
  的连接元素 focus()，forms/interactive 域无 canvas 回退候选面，不重
  跑。

## 挂账

- 非 canvas 面一般「being rendered」渲染门（display:none 普通元素
  focus() 拒绝）——本轮 FIXME(rendering) 在册，无语料覆盖不入刀。
- flat tree slot 分配细化（默认 slot 重纳入 light 子）——FIXME(flat
  -tree) 在册，语料钉 named-slot 形态已覆盖。
- mousedown 默认动作聚焦面（`_zwFocusSel`，part06）仍只过固有核心谓
  词未过渲染门——渲染中点击面与真实 hit-test 归 layout 集成面，观察。
- img naturalWidth-naturalHeight-width-height 22F（SVG viewBox 内在
  尺寸/宽高比族——image-decoder 面，与 renderer-trust-boundary 流
  ③A image-decoder 切片工作面相邻，暂不入刀避免跨流碰撞，承袭）。
- current-pixel-density/sizes/srcset/decode 族 ~29F（响应式图片面，
  候选，承袭）。
- part06 `_zwFieldsetDisableFocusReset` legend 扫描碎片共享（s215 承
  袭，观察）。
- part03 `_zwMElCheckValidity` FORM 分支短路形态（s210 承袭）。
- onblur flake 族持续采样（s209~s217 在册摆动键承袭）。
- beforeinput.tentative.html 既有面（录入事件族独立评估承袭）。
- 上游 customizable select 特性面 177F 挂起待拍板承袭（GB-20261031）。
- form owner table / parser form element pointer 缓行族（用户决策承
  袭）。
- form-nameditem override builtins 待决（承袭）。
- input-type-button click() 返回值面（共享通路高风险，未入刀，承袭）。
- 多进程 renderer SetDomSnapshot entries 化（s174 低优先承袭）。
