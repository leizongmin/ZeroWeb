# formatBlock div>p 摆位刀（vetoed 直亲内联成员外提三防线，2026-11-10）

## 刀体

- **缺陷定级（残 23F 主目标，div>p ×2）**：`<div>[foo<p>bar]</p></div>` + formatblock `<p>`（formatblock.js :1754/:1759 双 dps 变体）录 `<p>[foo</p><div><p>bar]</p></div>`——Chrome 段落摆位把部分覆盖段落的替换块落块侧旁（新块 hoist 出 div，div 保留未触的 p>bar）；本引擎 spec splitParent 爬升遇 prohibited 后代刹车后新块 wrap 落块内产嵌套形 `<div><p>foo</p><p>bar</p></div>` 全红。
- **blockExtend 域的成员构成（TRACE 实证）**：blockExtend 把 range 端点爬到块边界后，SLC 块本身成为 contained 成员——探针 `afterFilter=[foo,<p>]`：foo（内联 text）与 **p 本体**（非列表 SLC + 无 prohibited 后代 → 过滤器通过；getContainedNodes 含 p 后跳过其后代 bar）双成员入 nodeList，bar 不在列表。同 tag no-op splice 对 p 成员不命中（其祖先链是 div 非 p）。
- **修复**：part01g `__zw_ce_fb_apply` 值爬升环加 Chrome delta 臂——vetoed formattable **直亲** + **内联成员** + 新块为其允许子 + recordValues 全 null 时 `splitParent([node])` 外提，wrap 尾环落宿主层；块成员不外提留位，走尾环 SLC 溶解臂在**原块内**溶解重 wrap 产出 `<div><p>bar</p></div>` 录音形。h1→p 外壳形（新块非允许子）不外提维持 spec 停——fixDisallowed 允许子臂拆父产出合并录音形（:1744 现行绿面保持）。
- **三防线（首版两缺陷 TRACE 定谳后加）**：
  1. **直亲门**（`node.parentNode === anc`）：splitParent 只拆直亲链——`<div><table><tr><td>[foo]</table></div>` 录音 div 落 td 内（:1794），首版无门时臂在 vetoed div 触发把 foo 跨 td/tr/table 撕出（got `<div>foo</div>` 表格全拆，P2F ×2）；
  2. **块成员门**（`_zwJusIsInlineNode(node)`）：块成员（SLC 本体）外提会连锁拆壳——splitParent move-out 臂移出 p 后空壳删除 → 被替换块整体消失（首证 got `<p>foo</p><p>bar</p>` 丢 `<div><p>bar</p></div>`）；
  3. **值门**（recordValues 九命令全 null 才触发）：styled 块 forecolor 等真值恢复经 forceValue 落内联载体，外提后载体 × wrap × fixDisallowed 链丢内容（`<div style=color:blue>[foo<p>bar]</div>` 探针 foo 失踪实证）。
- **TRACE 诊断法（临时插桩，commit 前撤除零残留）**：part01g 页可见全局 `__zwFbqTrace`（filter/splice/values/climb/arm/wrap 逐事件）+ 探针页 `assert_unreached` 打轨迹全文——首轮 F2P=0 + P2F=3 静态推演失败后改实证取数，TRACE 三事件（afterFilter 构成 / ARM-FIRE ×2 / hostTree 中间态）一次性定谳三缺陷根因。

## 预验（脏树测量域 = 提交域字节恒一）

- 探针 zz-probe-fbwrap.html（7 行 dump census，语料条目格式直跑 runConformanceTest）：B1/B2 出录音形 `<p>foo</p><div><p>bar</p></div>`（=语料期望形）；A1/A2 合并面 `<h1>foo<br>bar</h1>` 恒一（:1744 绿面）；C1/C3 与 pre-arm 字节恒一（值门/直亲门生效）；C2 无损外提形 `<p>foo</p><div style="background-color:yellow"><p>bar</p></div>`（背景声明不入九命令——probe-only 形变，无语料行，无数据丢失）。
- fb 全页 5025P/21F judge vs boolface 门 bfl-fb-gate：**F2P=2（恰 div>p 双 dps 行）P2F=0 lost 0 VERDICT OK**（fbw-fb-post2.log；5023/23 → 5025/21）。

## 门禁批（v2 前五腿 + v2-tail 尾段合并判读）

批 batch-fbwrap-gate-v2（锚 head **8259b07d8** + 九点 md5：part01g 新锚 `3683c329`，part01 `73e18900`/part01c `fb8aa0d8`/part01d `f4bc3bf8`/part01e `4415bf2f`/part01f `6e8eb934`/part01h `97531eb0`/part06 `05dd0f67`/bridge `d47f2e09` 沿推送树；特征串摆位刀×1 + 块成员门×1 + inlineNode 门×1 + trace 残留=0 + wrap-only×4 + harvest×2 + carryProp×2 + fb_indeterm 06×3 0G×1 + fbqRet2×3 + boolean 读面刀 01d/01g + TEMP-TRACE=0；ensure_solo + flock）。

- **v2 前五腿过门（13:24 发射）**：probe fbw 58P/19F（judge vs fbw-probe-post3 幂等零翻越）→ fb 主门 5025P/21F（judge vs bfl-fb-gate **F2P=2 键面恰 div>p 双行 + P2F=0**；幂等 vs fbw 预验 F2P=0 P2F=0）→ mt 20182P/2F（judge vs bfl-mt 键级零翻越）→ iol 1759 / iul 2061 零漂移。
- **v2 ABORT（14:21，非行为红）**：ip 腿 test-guard 900s 超时 P=0——跨界租户毒化（mykernel `make check` + ZeroWeb-2 clone cargo build 同机并发；wallclock-contention 先例形，boolface 批同腿 ~4min 完成 P=6989）。
- **v2-tail 全绿（14:23 发射，14:53:57 TAIL-ALL-DONE）**：ip **6989**（=boolface 同数，judge vs bfl-ip-post 零漂移）→ ihtml 1446 零漂移 → exec×3 零漂移 → enginelib 3827P/**7F**（名册双门恒一）→ battery 七腿 420/880/1104/552/3456/2607/1455 全同数 F=0 T=0 → binary sig 全在产物（含摆位刀标记）。
- 合并判读 = v2 前五腿 + tail 六腿全过，锚域字节恒一（tail 首行锚复读 head + part01g md5 复读 OK）。

## 挂账更新

- **残 23F → 21F**：div>p ×2 收口。残 21F 族级（fbw-fb-post2 实测）：compare 面照树红 ×5（blockquote ×2 / section-hgroup ×1 / h1 合并形 dps-p ×1 / p 命令 span cE=false ×1）+ qV·qI 读面 ×16（table ×4 / h1 ×2 / dl ×2 / p ×2 / blockquote-inner qV-after · div · del · xmp · listing · quasit 各 ×1）——其中 blockquote ×2 compare + section-hgroup compare 定性为 **Chrome per-paragraph 不合并族**（部分覆盖多段各自换块不合并，与 div>p 摆位同根不同形），待刀。
- **② 外层 restore 载体类（值命令）**：多块 styled census（C1/C2/C3）pre-arm 无损嵌套、post-刀 C1/C3 字节恒一（值门/直亲门拦）、C2 无损外提形——**外层 restore 载体链未再现数据丢失**；挂账收窄为「C2 形（背景块 × 多块）Chrome 真值未录音，现走外提形」。
- **③ `_zwJusWrap` 载体内插块 hostkids=0 机制定谳**：本轮 TRACE 面未再现（wrap 尾环 hostkids 正常），维持挂账原状。
