# fb apply 侧 SLC 溶解声明 carry 刀（color:blue 数据丢失修复，2026-11-10）

## 刀体

- **缺陷定级（数据丢失级，非兼容面）**：`<div style=color:blue>[foo]</div>` + formatBlock `<p>`：execCommand 全返 true 但内容序列化清空。语料 :1834-1857 八行（4 entry × stylewithcss true/false 双变体）全红，RET 期望 [true,true,true] 却"假绿"——part06 异常壳掩盖 apply 内部破坏。
- **根因（TRACE 级实锤，临时插桩后移除）**：apply 解包臂 `vals2 record→dissolve→restore` 把溶解 SLC 的声明 forceValue 到裸文本上——本引擎 cssFlag 读面走 font 臂造 `<font color>` 载体；共享尾 wrap 在文本处造新块，**块落进载体内**（disallowed 父）→ fixDisallowedAncestors 允许子臂 record+splitParent 拆父（载体溶解）+ restore 对**块**二次 forceValue 再裹载体#2——链条终态宿主清空、GOT=""、SEL 塌 host@0。TRACE 逐点：`collected [div[color:blue]]` → unwrap 后 `forceValue forecolor=blue on t"foo"` 载体落 host → `wrap done block=p hostkids=0` → `fixdone tree=null`。
- **bisect 面谱**（fbl2 探针）：color:blue / font-size / background-color / fb=pre / fb=h1 / fb=div / styled-p→div 全丢；bold / underline 幸存（specified 读面 0 值不入 restore，见挂账②）；b/span 载体形（无 formattable 祖先 split）无恙。丢失与目标 tag 无关、与 cssFlag 无关。
- **Chrome 对齐**：Chrome formatBlock 把被替换块的内联声明转移到替换块本体——语料双变体同形 `<p style="color:rgb(0, 0, 255)">[foo]</p>`，**无内联载体**（无 font/span 包裹）。
- **修复（part01g）**：解包臂溶解前收割块级声明 `_zwFbqHarvestBlockValues`（forecolor/hilitecolor/fontsize/fontname 四值命令；**boolean 命令不入收割**——specified 读面错读他族声明，hilite 形探针 `bold=yellow` 实证，挂账②）；等值匹配的 vals2 三元组抑制 restore（子树更深自有值照常 forceValue）；wrap 成块后 `_zwFbqCarryProperty` 映射 `setStyleProp` 落块本体（CSSOM 归一 `color:rgb(0, 0, 255)` 形恰值），fixDisallowed 前落——块 specified 即值，restore 爬升面自然短路。
- 读面（`__zw_ce_fb_value`/`__zw_ce_fb_indeterm`）、wrap-only 块（xmp/listing 走 else 臂不溶解）、veto/收集/裂父各环零改动。

## 预验（脏树测量域 = 提交域字节恒一）

- fbl4 探针（floss2 九形）：H=`<p style="color:rgb(0, 0, 255)">foo</p>`（语料期望串恰值）、L=`<p style="font-size:large">`、O=`<pre style=...>`、P=`<h1 style=...>` 载体全消；I/J/K/M 零漂移；N（hilite）维持原坏形（挂账②域）。
- fb 全页 5015P/31F→**5023P/23F**，judge vs fbw-fb-gate（5015/31 基线）F2P=8 P2F=0 lost 0 VERDICT OK（fbl6-fb-post.log）；8 flip = color:blue 4 compare + 4 qV-after（qV-after 翻绿 = 修后树有 `<p>` 读面 "div"→"p"，修复直接后果）。
- fbwrap 探针（30 形）残 5F 全落 blockquote×3 + div>p×2 既有族，color:blue/xmp/listing 行全绿。

## 门禁批

批 batch-fbcolor-gate-v1（锚 head **5bd748fea** + 九点 md5：part01g 新锚 `d5b1e849`、part01e `4415bf2f` 新增点、part01 `6fca2f31`/part01c `fb8aa0d8`/part01d `9fbab2af`/part01f `6e8eb934`/part01h `97531eb0`/part06 `0a7329fb`/bridge `b6b9a516` 沿推送树；特征串 fbqRet2×3 + fb_indeterm 3/1 + wrap-only×4 + harvest×2 + carryProp×2 + **trace 残留=0**；ensure_solo + flock）。

- 首射 ABORT @fb 行级白名单（诊断期自伤非行为红）：白名单谓词漏 `queryCommandValue("formatblock") after` 形（16 flip 中 4 行属 qV-after，见预验）；计数/judge/幂等当时已全绿。谓词扩双形（compare ∨ qV-after，域匹配不变）整批重射，源字节零改动。
- 重射：**ALL-DONE 2026-11-10 09:45:20 全绿**（08:36:02 发射，69 分钟，solo 窗，fbc-* 日志族）。
  - probe：f6bwin 2092P/0F + k11 224P/17F judge 零漂移。
  - fb 主门：5023P/23F（期望恒一）；judge vs fbr4b **F2P=16 / P2F=0 / lost 0 VERDICT OK**；行级 flip 白名单 16 = xmp/listing compare ×8 + color:blue compare ×4 + color:blue qV-after ×4；幂等 judge vs fbl6 预验 F2P=0 P2F=0。
  - mt：20182P/2F judge 键级零翻越 VERDICT OK。
  - 单文件腿（vs F1' 终态零漂移）：iol/iul/ip/ilb/ihtml + exec×3 judge 零漂移。
  - enginelib：failed=7 名册双门恒一；battery 七腿 420/880/1104/552/3456/2607/1455 全同数 F=0 T=0；binary sig 全在产物（含 harvest）。

## 推送重锚（rebase 并入组合树）

刀体+docs 经 rebase 推为 **ff1ad58b7**（刀体）/ **25e0d0cfe**（docs v1.87）。入站域（10fab5c5a..6fc7279e7 九笔：s377 dialog overlay + T54 engine-shim 共享名折叠 + R5491 layout fill 上溯 + test262/script-sandbox module 预检）：**part01 +12（`73e18900`）/ part06 +15（`d9aaada5`）触边**（T54 词尾折叠、s377 overlay gCS 面），刀核 part01g `d5b1e849` / part01d `9fbab2af` / part01e `4415bf2f` / part01f `6e8eb934` / bridge `b6b9a516` 恒一 → 按推树纪律重绑 fb 腿（重绑不扩）：组合树重建二进制直跑 formatblock 全页 **5023P/23F**，judge vs 批门 **F2P=0 / P2F=0 / lost 0 VERDICT OK**（fbc2-fb-rebind.log，REBIND-DONE 09:55:10）——刀判读绑定推树字节恒一。重锚后锚值：part01 `73e18900`、part06 `d9aaada5`、part01g/bridge 恒一；下一刀批脚本按现树重取锚。

## 挂账

- **① boolean 命令 specified 读面错读**：`_zwVaSpecifiedValue(div,'bold')` 对 `background-color:yellow` 形返 'yellow'（他族声明窜值）——hilite 探针形（N）仍产出双空 span 坏树（五条 boolean restore 载体链）。语料无 fb×styled-div-boolean 行，无 compare 面；数据丢失面收敛待 boolean 读面独立刀。
- **② 外层 restore 载体类**：多块形（如 `<div style=color:blue>[foo<p>bar]</div>` + <p>）外层 record/restore 在裂父后仍走 forceValue 载体 + wrap-in-carrier + fixDisallowed 链——本刀未接（无语料行，P2F=0 门下零回归）。与 ① 同根（载体+块嵌套+拆父），修复应并刀。
- **③ `_zwJusWrap` 载体内插块 JS 侧 hostkids=0 机制**：TRACE 实证 wrap 后宿主 JS 侧即空（font[foo] 载体消失），wrap 内部序列化/接线细节未定谳（M-wire 壳缺失疑点同 :1029 注）——carry 刀绕过该链，机制定谳挂账。
- 残 23F 维持 fbwrap 刀挂账：blockquote ×4 / table ×4 / div>p ×2 / qI 泄漏 ×2 / 录音伪影 ×4 + listing 窗界 ×1 + del qV-after ×1（本刀残面比对 fbwrap 刀挂账只减不增）。
