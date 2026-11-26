# DC-2 切片 10 刀㉜ verdict——media reactions 簇收口（customized-builtins HTMLMediaElement 28F 全清）

**2026-11-26 · 记录腿 HEAD 即本刀码提交（k31 码刀 `cbcccab7c` 之上；修改 2 文件 +29/−1，纯 .js shim，fmt/clippy 不适用）**

## 结论

- **WC 语料净 +28**（F 987→956 zz-excluded 口径）：`custom-elements/reactions/customized-builtins/HTMLMediaElement.html` **28 行全翻绿零残面**（audio/video 各 14：src·crossOrigin·preload·autoplay·loop·controls·defaultMuted × add/replace）——刀㉛ 移交的族内最后残池清空，customized-builtins reactions 17 页全绿。
- **零新败**：NEW Fail 行＝0；唯一 NEW Timeout＝`shadow-dom/reference-target/tentative/event-path.html`（页级停摆，solo 双侧同形定谳非本刀，见下）；gethtml −69P＝巨页截断带（F 集恒等签名，k31/k30 同注记口径）。
- **四邻电池**：domparsing **行级 SET-IDENTICAL**（222F/1313P/1T）＋sanitizer-api **行级 SET-IDENTICAL**（50F/706P/1T）＋dom **F/T 364/12 恒等**（−144P＝Range-mutations dataChange −70/replaceData −74＝master.md 碰撞记录在案停摆漂移族，零 NEW 行）。
- **累计 84,154（距 ≥99,947 差 15,793）**。

## 根因三站（码刀 2 文件 +29/−1）

| 站 | 根因 | 刀 |
|---|---|---|
| ① 升级在途 ctor throw | `class X extends HTMLAudioElement` 的 `super()` 命中 part03 media 包装 ctor：升级在途（`_ceRunCtor` 设 `_zwCeExisting`）时 `__zwIfaceSuperCreate` 返 null（在途门）→ 穿透落 `_znMwCtor.apply`＝**Illegal constructor throw** → 用户 ctor 体在 super() 即中止 → 'constructed' 反应永不落（28 行全部首断言 `expected ["constructed"] got []` 主根因；winErr=["Illegal constructor"] 探针实证）。generic 接口 stub 同态**静默穿透**，media 包装独有 throw。原型/章均已先设（`protoIsCustom:true`＋`is` 属性在）＝connected/attributeChanged 照常派发、唯 constructed 缺席 | part03 media 包装：`_zwSupM` null 时消费 `_zwCeExisting` 返回既有元素（与 HTMLElement ctor 桥 derived-ctor this 注入同语义）；`new klass()` 形态在途门不命中仍走 Illegal throw（audio_constructor 断言面不变） |
| ② 映射缺口 preload/defaultMuted | `_zwCeReflectionAttr` 对 'preload'（不在 FLAT/BOOL 表）与 'defaultMuted'（dirty/default 态不入 _REFLECTED_BOOL）返 null → set trap 头 `_ceSetAttr=null` → sink 双门跳过（k31 根因①同型）。探针定谳：preloadSet/defaultMutedSet=[] 而其余五属性全派发 | part03 映射补 `preload→preload`＋`defaultMuted→muted`（sink 双门＋moAttr 真写门保安全，非媒体元素分支 tag 门内零写零派发） |
| ③ defaultMuted handle 漏 moAttr | part05 defaultMuted 分支 handle 路径不设 moAttr → sink 外门 `if (moAttr)` 跳过（k31 根因③ htmlFor handle 面同款） | part05 两 handle 路径（设/移除）补 `moAttr='muted'` |

## 验证面

- 探针链：zz-k32-probe1 自含页（不经 reactions.js helper）四相定谳——修复前 `logAfterCreate:[]`＋winErr=["Illegal constructor"]＋preload/defaultMuted 零派发；修复后 `["constructed"]`＋winErr 清零＋七属性全派发；`new klass()`（AUDIO tag＋is 记录）与自主元素对照双不变。探针页交付前已删。
- 判读纪律：语料指纹 `@ 315976933870` 恒等；行提取/排序一律 `LC_ALL=C awk`＋C-collation（comm 双侧静默零输出 rc=1 复演踩坑，dom 腿 diff 改用 awk merge-diff 定谳）；zz 探针页剔除口径＝路径含 `/zz-`（行首锚 `/^zz-/` 漏剔 k31 探针 43 行的坑已修正）。
- **event-path Timeout 归因定谳**：该页零 customElements.define、零 media 元素——本刀三修复无代码路径触及（映射需 registry entry、页无 define；ctor 修复需 audio/video cbi）。`git stash` 双向 bisect：修复前 solo 复跑同形（9F+1T+2NotRun，同一测试行 "toggle event propagates... deeper shadow root" 超时）＝既有页级停摆面（master.md 碰撞记录停摆漂移族同款，对时序敏感非本刀确定码行为），修复恢复后归位。
- WC 电池墙钟 ~6min（04:48→04:54）＋三邻腿 ~9.5min。
- 质量门：电池全腿走 make testharness-* 守卫入口；lei-pre-commit-guard 提交前扫描在案。

## 残池移交

- 无族内残面——customized-builtins reactions 簇（17 页）全绿收口。
- 近邻挂账不变：connected-callbacks shadow host 簇 8F／Node replaceChild 跨文档 F（刀㉚ 承继）；TT residual 101F 池（B 臂：createHTMLDocument 5F／block-string-assignment 13F／svg-script-set-href 7F／script-enforcement-008/009 等）。
- 同 face 相邻缺口（本刀未触，corpus 无钉）：`muted` IDL set handle 路径同款漏 moAttr（part05 muted 分支，非本页 observed 集）——后续 WPT 出现 media cbi observed 'muted' 面时同款一行为补。
