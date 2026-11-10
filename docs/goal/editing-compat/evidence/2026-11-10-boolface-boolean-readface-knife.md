# boolean 命令 specified 读面刀（幻值恢复面 + 块级背景 carry，2026-11-10）

## 刀体

- **缺陷定级（坏树级 + 数据丢失级复合）**：`_zwVaSpecifiedValue` 的 prop 映射对全部非值命令走 else 臂读 `background-color`——五条 boolean 命令（bold/italic/underline/strikethrough/subscript）在带背景声明的祖先形上全部错读幻值。fb 溶解形（`<div style="background-color:yellow"><b>[foo]</b></div>` + formatBlock `<p>`）：recordValues 九命令爬升对 dissolve 块记下五条 boolean 幻值三元组 → 溶解后 restoreValues 对裸文本五连 forceValue，且幻值 'yellow' 经同一 else 臂属性映射落成**背景 span 载体**——门禁前探针实证五嵌套 `background-color:rgb(255, 255, 0)` span 坏树；纯背景形（无 boolean 载体）更走 wrap-in-carrier→fixDisallowed 拆父链内容全失（carry 刀挂账 N 形双空 span 的真根因——非 boolean 载体单独致损，而是幻值恢复 + 背景 carry 被 hilitecolor display 门挡两缺陷复合）。
- **第二面（爬升刹车缺失）**：b/i/u/s/strike/sub/sup 标签形 specified 读恒 null——recordValues/restoreValues 祖先爬升穿壳到根：该停不停（真 boolean 载体不刹车误记 null）与不该停乱停（背景声明幻值刹车）双向失真。
- **对齐面（vendored implementation.js getSpecifiedCommandValue :2201-2337）**：subscript/superscript 只读标签（无 vertical-align 面）；strikethrough/underline 的 style 声明面先于 s/strike/u 标签面（声明设置 text-decoration 但不含该线值时直接 null，不落标签）；bold/italic 走 relevantCssProperty（fontWeight/fontStyle）声明面 + b/strong/i/em 标签面。修复逐臂同构。
- **修复**：part01d `_zwVaSpecifiedValue` 头部补五条 boolean 族读面臂（在 backcolor/hilitecolor display 门之后、值命令 prop 映射之前）；声明读复用 `_zwPlainDecl`（style attr 权威面，与 vendored element.style 同域）。part01g `_zwFbqHarvestBlockValues` 对 hilitecolor 在 spec 读面 null（块级 display 门）时直读 `background-color` plain 声明补收割——Chrome fb carry 面转移被替换块内联声明不分 display。harvest 头注同步（boolean 不入收割的理由由「读面错读」改确为「Chrome carry 面只转移值声明」）。
- 语料影响面：**零**——语料（fb 全页 + mt + 单五腿）无背景块 × boolean 记录/恢复交叠行，judge 门禁 F2P/P2F 全 0 实证；修复面全部落在探针形。

## 预验（脏树测量域 = 提交域字节恒一）

- 探针 zz-probe-boolface.html（8 行 88 断言，语料条目格式直跑 runConformanceTest）：修复前 4 绿（color 块 ± b/em/sub 载体控制行）+ 4 红（背景块 ± b/u/s 载体四形，坏形见刀体）；修复后 **8/8 全绿**（bf2-probe-fixed.log，rc=0）。
- fb 全页 5023P/23F judge vs carry 刀门 fbc-fb-gate：F2P=0 / P2F=0 / lost 0 **VERDICT OK**（bf2-fb-post.log）。

## 门禁批

批 batch-boolface-gate-v1（锚 head **cbbb55373** + 九点 md5：part01d 新锚 `f4bc3bf8` / part01g 新锚 `c842e062`，其余 part01 `73e18900`/part01c `fb8aa0d8`/part01e `4415bf2f`/part01f `6e8eb934`/part01h `97531eb0`/part06 `d9aaada5`/bridge `b6b9a516` 沿推送树；特征串 fbqRet2×3 + fb_indeterm 3+1 + wrap-only×4 + harvest×2 + carryProp×2 + trace=0 + strike 读面×3 + boolean 读面刀 01d/01g 标记 + text-decl×2；ensure_solo + flock；对端 clone site-optimizer bench lane 在飞由 ensure_solo 腿前等待消化）。

- **首射全绿 ALL-DONE 2026-11-10 11:50:11**（10:31:21 发射，79 分钟，bfl-* 日志族）：
  - probe：boolface 88P/0F（judge vs bf2 预验幂等零翻越）+ f6bwin 2092/0 + k11 224/17 judge 零漂移。
  - fb 主门：5023P/23F（期望恒一）；judge vs fbc-fb-gate **F2P=0 / P2F=0 / lost 0 VERDICT OK**；幂等 judge vs bf2 预验 F2P=0 P2F=0。
  - mt：20182P/2F judge vs fbc-mt F2P=0 P2F=0 post-only 0 lost 0 VERDICT OK。
  - 单文件腿（vs fbc carry 终态零漂移）：iol 1759/iul 2061/ip/ilb/ihtml + exec×3 judge 零漂移。
  - enginelib：failed=7 名册双门恒一；battery 七腿 420/880/1104/552/3456/2607/1455 全同数 F=0 T=0；binary sig 全在产物（含 boolean 读面刀标记）。

## 挂账

- **② 外层 restore 载体类（值命令，未接）**：多块形（如 `<div style=color:blue>[foo<p>bar]</div>` + <p>）外层 record/restore 在裂父后仍走 forceValue 载体 + wrap-in-carrier + fixDisallowed 链——boolean 幻值面已随本刀消失，值命令（forecolor 等）真值恢复的载体链未接（无语料行，P2F=0 门下零回归）。挂账③（`_zwJusWrap` 载体内插块 hostkids=0 机制定谳）同前。
- 残 23F 维持：blockquote ×4 / table ×4 / div>p ×2 / qI 泄漏 ×2 / 录音伪影 ×4 + listing 窗界 ×1 + del qV-after ×1（本刀残面只减不增，fb 页 23F 逐行 judge 恒一）。
- indent 27F/img 23F 零散边界族维持待刀；selection-compat-v2 划转 fontSize T1 ×3 待接收。
