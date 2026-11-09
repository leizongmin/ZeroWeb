# M2 切片 376（s376）——dialog 域剩余 3 案勘根定性轮（零源码改动）：bounds-clicking 2F 归 R-s31；corner-cases 1F 上游 Chrome 同败

日期：2026-11-10（承 s375 挂账：closedby-bounds-clicking 2F 流内候选）
域：html/semantics/interactive-elements/the-dialog-element（dialog-closedby-bounds-clicking / dialog-closedby-corner-cases）
性质：**勘根定性轮（零代码改动）——bounds-clicking 2F 定谳 R-s31 UA dialog 定位簇（改挂渲染流）；corner-cases 1F 定谳上游 Chrome 同败面（本地 chromium 159 实测 + spec 链推演双证）**

## 勘根一：dialog-closedby-bounds-clicking 2F（(1,1) 两子测误关）

**案面**：dialog `closedby=any`，作者样式 `dialog { inset: 0 auto auto 0; width: 100px; height: 100px }`（无 position——靠 UA `dialog { position: absolute }` 使 inset 生效锚定视口原点），子 #box 200x200 margin:20 溢出。14 子测（show/showModal × 7 坐标）：ZW 仅 (1,1) 两测红（expected open got closed=误关），其余 12 测全绿——bounds 判定机器（s372 flat contains 面）语义正确：内点 (99,99) 不关、溢出后代 (150,150)/(199,199) 不关、外点 (250,250) 关、单轴出界 (150,1)/(1,150) 关。

**探针证据链**（zz_probe_s376c，勘根毕删）：
1. dialog rect=(8,20,100,100)——**pos=static**（UA absolute 缺席），文档流内 body margin 8px + 前置偏移 12px 起排；(1,1) 落在 dialog 外。
2. pointerdown 事件链 `pd@document<BODY> | pd@html<BODY> | pd@body<BODY>`——点击 (1,1) 几何正确命中 BODY（非 dialog 后代）→ light-dismiss 分类机按「命中物非 dialog 后代→关」执行——**语义链每环都正确，几何差异纯粹来自布局**。

**定谳**：Chromium 里 UA `dialog { position: absolute; inset-inline 0; margin: auto; width/height: fit-content }` 簇使作者 inset 生效锚定 (0,0)，点击命中 dialog。ZW 缺 UA 定位簇 → dialog 落流内偏移位。与 modal-dialog-ancestor-is-inert、abspos-dialog-layout 6F、multiple-centered-dialogs、non-modal-dialog-layout、modal-dialog-scroll-height、top-layer-position 同根（R-s31 挂账簇；s372 先例已定性：定位注入需布局侧 abspos 收缩/margin-auto 居中配套，单独注入 position:absolute 有盒宽塌缩 + 全 modal corpus 布局回归风险）。**从「流内候选」改挂 R-s31 渲染流簇，shim 无近路**。

## 勘根二：dialog-closedby-corner-cases 1F（focusin calls showModal, modeless + ESC）

**案面**：20 子测 19 绿 1 红。红子测链：setup 先 `outside.focus()` 再挂 focusin listener（listener 错过首 focusin——dialog 从未变模态）→ openDialog('modeless') 走 `dialog.show()`（非模态，spec show() 尾步 run dialog focusing steps → 焦点迁入 dialog）→ ESC → close（wasModal=false，焦点锚在 dialog 内 → anchorOk=true）→ spec 同步焦点恢复 outside → focusin → listener showModal() → open 属性已移除 → showModal 成功重开 → 最终 open=true。案期望 `respondsTo(modeless)=true`（closed）。

**探针证据链**（zz_probe_s376d/e + 本地 chromium 实测 zz_probe_corner.mjs，勘根毕删）：
1. ZW modeless ESC 通道本身工作（无 listener 时 cancel→close→open=false）。
2. ZW 真案链败态=ESC 后 `open=true`（重开发生），与 spec 算法逐环吻合（show() focusing steps 迁焦点 + close 同步焦点恢复——spec「close the dialog」焦点恢复为同步步，仅 close 事件异步排程）。
3. **本地 Chromium 实测**（puppeteer-core + trusted ESC 经 CDP Input 通道，复刻上游 setup 顺序）：pre `open=true log=[openDialog open=true]`（listener 同样错过首 focusin）→ ESC 后 `open=true log=[...,focusin->showModal ok]`——**上游 Chrome 同样重开（respondsToEsc=false）**；对照臂（无 listener 纯 modeless）ESC 后 open=false 正常关闭。

**定谳**：上游案期望 `respondsTo(modeless)=true` 与真 Chrome 行为矛盾（Chrome 实测 open=true）→ **该子测在上游 Chrome 也是 FAIL**（wpt.fyi result_status=FAIL 查询命中该文件 chrome run 5159743996559360 佐证）。spec 语义链（show 尾 focusing steps + close 同步恢复 + 重开链）在 Chrome 落地同形。**非本仓缺陷，挂账转「上游已知失败面」——修它需要违 spec 行为，不施工**。

## 勘根过程坑（入册）

**探针静默空集坑**：自建探针 HTML 源码不含 `testharness` 字串（漏写 `<script src="/resources/testharness.js">` 引用）→ runner `html_semantics_case_skipped` 谓词 `relative.ends_with(".html") && !source.contains("testharness") → skip` → FILTER 空集静默 exit 1（「Using 8 test job(s).」后零输出零报错）。s376 勘根烧 4 轮（FILTER 形态/枚举机制/skip 谓词依次排除后钉死）。自建探针最小形态必须含 testharness 引用。

## 域口径变化

dialog 域真案 18F 中 3 案定性完毕（bounds 2F 归 R-s31 簇、corner-cases 1F 归上游同败面），非翻绿非回归——**域计数零位移**（s375b 口径 464P/22 Fail 行维持）。上游同败面在账本单列口径（不入「本仓可施工」残红清单）。

## 挂账（移交下一切片）

1. **R-s31 UA dialog 定位簇**（渲染流）：bounds-clicking 2F 新增归入（共 10+ 案投影：abspos-dialog-layout 6F、multiple-centered-dialogs、non-modal-dialog-layout、modal-dialog-scroll-height、top-layer-position、modal-dialog-ancestor-is-inert、bounds-clicking 2F、dialog-showModal top-most 嫌疑）。
2. **上游已知失败面**（单列口径）：dialog-closedby-corner-cases「focusin calls showModal, modeless + ESC」1F——Chrome 实测同败，待上游案修正或 spec 语义变化后重勘。
3. 流内剩余候选：select-dialog-mode-focus 2F（focus 行为面）、dialog-overlay 2F（overlay CSS 属性 computed 值面）、dialog-form-submission 亚像素坐标、closed-details-layout-apis（content-visibility 渲染流）、inert hit-testing 穿透近似面（无红案钉面不扩刀）、基建面 T=3+probe8/9/10 转正决策承继。
4. 勘根产物已删（zz_probe_s376[a-e] 探针 + zz_probe_corner.mjs + cdp_pipe.py 均为临时工具，gitignore 区/本地）；验证与探针日志 /tmp/s374/（本地）。
