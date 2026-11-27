# s10k60 刀㊼：解析视图 context 激活＋san 基线键名归一——sethtml-safety 9F 清零＋html5lib-basics td-context 1F 翻绿

日期：2026-11-27。刀体 commit `63e34ae4c`（4 文件：part13-sanitizer.js＋part03.js＋js_dom_bridge.rs＋callbacks.rs，79 增 16 删）。guard 扫描 PASS（0 发现 0 提醒，绑定暂存 blob）。刀号 ㊼ 与 html-semantics 流 s10k48 刀㊼（fa75fd1f5）同字异 session——本仓刀号按本流序列自增，撞号以 commit 标题＋evidence 目录消歧，本刀独占 s10k60。

## 刀位与基线

控制面点名簇：san 残面 sethtml-safety 9F（刀㊻b 后 san 池最大簇）。新基线 solo 实测 **739P/39F**（ver-san60-base.log＝刀㊻b 收账 739P/39F 零漂移复认，k147 零外溢）。

## corpus ground truth

- `sethtml-safety.sub.dat` #3/#9-15：显式 config 面＋空 config 面——`{elements:[...]}` 与 `{}` 期望树全剥 base/embed/frame/iframe/object/script（XHTML）＋svg script/use（#built-in-safe-baseline-configuration 语料 SAFE_BASELINE verbatim）。
- `sethtml-unsafety.sub.dat` #0/#1：`#document-fragment script`——spec fragment parsing context=script → tokenizer script data state → 整串单 text（#1 `<p>Hello</p>` 期望 `| "<p>Hello</p>"`；#0 `test` 期望同构）。

## 三重根因与订正

### ① san 基线臂键名失配（8F）

`_zwSanBaselineRemoveEls` 条目键名 `ns` vs `_zwSanSameEl` 比较器读 `namespace`——恒失配使三条消费臂（verdict safe 强制基线臂 part13:1052-1056＋removeUnsafe 剔除臂 967-973）**自诞生未生效**。默认实例 removeElements 路径（键名正确）兜住无 config 面；显式 config 面暴露。订正：数组字面量＋构造点 `b.ns`→`b.namespace` 两处。

### ② 解析视图 context 从未生效（#1）

写链两侧分叉：引擎 Rust 侧 `replace_inner_html_unsafe` 从引擎树取 `local_name()`（小写，正确）→ html5ever ScriptData → 单 text；shim 视图侧 `_zwFragmentAddedUnsafe` 传 `_realTag` 大写形 `'SCRIPT'` → html5ever `tokenizer_state_for_context_elem`/`reset_insertion_mode` 均按 interned **小写**名匹配（`local_name!("script")`，markup5ever `From<&str>` intern 不归一）→ 大写恒落 Data/InBody——ctxTag 参数自 M3 清单 1 引入以来**从未生效**。探针 D1 定谳：script ctx unsafe 实际子树 `1:P:"el(1)"`（P 元素）非单 text。

订正（Rust `parse_html_subtree_deep_json(_unsafe)`）：`context_local` 进 `to_ascii_lowercase()` 归一。深臂 shim `_zwMBuildBodyTree` 直传原始串（ctxTag 在值时去 `'<body>'` 包裹——rawtext 态下包裹原样进 text 产物；包裹仅 body 寻址的全文档路径需要）。

### ③ innerHTML 视图同 context 补齐（matches face）

`.innerHTML matches .setHTMLUnsafe`（html5lib-basics #5 table face）在 ② 翻绿后暴露：setHTMLUnsafe 视图产 `tbody>tr>td`（InTable 隐式包裹激活）而 innerHTML 读面视图（`_zwFragmentAdded` 3 参调用不传 ctxTag）仍 body context 丢 `<td>`——此前「双错相等」掩盖。探针 P1 定谳：A 面 tbody>tr>td 真代理、B 面 `table>#text`。

订正：`parse_html_subtree_deep_json` 增 `context_local` 参（opt-in 不物化形，与 unsafe 变体同形）＋回调读 args[4]＋`_zwFragmentAdded` ctxTag 透传（handle/sel 双域，`_zwFragmentAddedUnsafe` 同款；**TEMPLATE 容器保守排除**——contents owner document 的 scripting disabled＋noscript 序列化印章与全文档路径强耦合（R5001 face），不并入本轮激活）。

## 判读

| 臂 | 锚 | 实跑 | 归因 |
|---|---|---|---|
| sethtml-safety 页 | 32P/1F（刀体① 后） | **33P/0F** | ② 后 #1 翻绿页全绿 |
| san 全臂 | 739P/39F | **749P/29F/0T** | F 四象限 comm：翻绿 10（sethtml-safety 9＋html5lib-basics td-context）、**新回潮 0**、common 29；ΔP+10=ΔF−10 闭合 |
| dom 臂 A/B | pre-knife bin（58c120fec 重建 `517c26fb2a6f`） | pre **57325P/361F/12T** / post **57524P/361F/12T** | **F·T 恒等**＝刀零 dom 影响；P+199＝Range-mutations 截断噪声族（dom-leg 基线既知形） |
| dom 臂旧锚勘误 | s10k7（11-24）496F | 361F | −135 全属 11-24→58c120fec 间**他流已落修复**（98188dfbd MO-attributes 回归修复、f634df5f2 清零红测试等均为 HEAD 祖先）——电池锚含刀窗前已落码增益，对锚增量≠对已推增量；dom 臂新锚 **361F** |
| tt60 | 2290P/14F/18T | **2290P/14F/18T** | 零漂移 |
| ref60 | 725/725 | **725/725**（Layout 521/521＋Text 204/204） | 零漂移 |

bin 字节绑定：chain60 发射断言 `0227ef9ac2bf`＝commit 63e34ae4c 字节；A/B 后复原重校同 md5 恒等。

## fmt 既有漂移记档

`cargo fmt --all -- --check` 15 区既有 diff（style-system/if_function.rs 11＋css-parser/mixin_apply.rs 1＋engine callbacks.rs:4876/css_wire.rs:392/js_dom_bridge.rs:5875 各 1）——HEAD 版本同区已 diff（`git show HEAD:` 复核），属已推代码在当前 rustfmt（style_edition 2024＋max_width 120）下的漂移，非本轮引入；本轮编辑区零新增。按精准修改纪律不顺手重排，挂账待专刀统一 sweep。

## 残相挂账

- san 残池 **29F**（刀㊼ 后）：svg-animate SMIL 1F（页可达上限 21/22）；余 28F 簇 census 待下轮（declarative-shadow-root 7F、custom-elements-is 8F 等旧簇以新基线实测为准）。
- 解析视图 context 激活的进一步面：TEMPLATE 容器（safe/unsafe 双路径）保守未激活；sel 域 outerHTML 产物视图（`_zwFragmentAdded` hostHandle=null 无 hostSel 形）未透传。
- census：+10 → **84,368**，距 DC-3 校准线 ≥94,093＝9,725。
