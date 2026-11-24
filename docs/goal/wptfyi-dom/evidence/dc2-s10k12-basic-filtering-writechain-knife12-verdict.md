# DC-2 切片 10 刀⑫ 判读证据——setHTML 写链视图恒重建＋detached write 快照遍历＋dataAttributes 名单放行刀

- 日期：2026-11-25
- 刀面：sanitizer-api `sanitizer-basic-filtering.html` Fail 64。
- 根因族：写链三域（element handle 域／ShadowRoot handle 域／parse 文档域）两 bug＋sanitizer config 一臂——均非 sanitizer 本体：α 视图清空捷径、β 物化 iterate-mutate、A data-* 无条件剥除。

## §1 实施面（3 shim 文件＋钉测）

1. `js_dom_shim/part05.js`（α＋TT 章）：
   - `_zwSetHtmlUnsafeImpl` 旧 `'<'` 捷径：handle 域只在值含 `<` 时重建 `_handleChildren` 视图，否则清空（`else if (handle && _uVal.indexOf('<') < 0) { _handleChildren[handle] = []; }`）——纯文本/消毒后净串（无 `<`）写入后 text/comment 子在 childNodes/getHTML 读面整丢。改为 handle 域恒重建＋删清空分支（无 `<` 输入只产 text/comment，重建循环 passthrough 零成本）。
   - TT-enforced 门控补章：函数头新增 script 容器检测（`_realTag` 大写 `SCRIPT`／`:SCRIPT` 后缀），命中且 `globalThis.__zwTtEnforced()` 为真时对容器 proxy 盖 `__zwFragNoExec` 章（s120 先例：章经 proxy set/get 存取；`_zwScriptMaybeStartDynamic` 首门消费）——spec trusted-types #enforcement-in-scripts（setHTMLUnsafe 对 script 容器设源须丢 trustworthiness）。
2. `js_dom_shim/part03.js`（β）：
   - detached doc `write` 物化循环 `for (j...) _tree.appendChild(added[j])` 边遍历边 mutate——appendChild 摘除守卫（旧父 removeChild）splice 活数组 `added`，`j++` 跳位丢约一半顶层子（T,C,T→T,T／C,C→C1／C,T→C／P,C1,C2→P,C2 七形态手工模拟全验证）。改 `var _uSnapW12 = added.slice()` 快照遍历，附 spec #dom-document-write 链接。全库同型排查：其余 `.appendChild(added[/kids[/arr[` 三处均为 cloneNode 形安全。
3. `js_dom_shim/part13-sanitizer.js`（A）：
   - data-* 过滤臂旧 `cfg.dataAttributes !== true` 无条件剥除，显式 attributes 名单（`attributes:['data-x']`＋`dataAttributes:false`）无法放行 data-x。新增 `_zwSanAttrAllowHit(list, local, ans)` helper（按 ans 精确匹配；XHTML 条目对 null-ns 实际属性放行），data-* 臂先查 cfg.attributes＋元素条目 attributes 双名单，未命中才 `_zwSanDropAttr`，附 spec #data-attributes 链接。
4. `js_dom_bridge_tests/part158_s10k12_writechain_pins.rs`（新）：九组常驻钉测——① handle 域纯文本三 sink 形（setHTMLUnsafe('text')／'a  b'／setHTML('plain text')）② comments 双路径（comments:false 剥后 serialize+reparse 合并 'T(a  b)' 单 text＝语料期望；unsafe/comment:true 全保留）③ dataAttributes:false＋显式名单（P 只剩 data-x）④ detached write 三形（'a <!-- c --> b'→T,C/T；双 comment 保；'<p>x</p>tail'→E<P>|T(tail)）⑤ SR.setHTML 纯文本。
5. `js_dom_bridge_tests.rs`：include 接入。

## §2 验证链

- 探针定位：语料页临时探针（gitignored）＋`assert_true(false, msg)` 树形转储；RAW childNodes vs NodeIterator 区分「树里没有」vs「遍历跳过」；直调引擎回调 `__zw_parse_html_subtree_unsafe` 定谳发射端完好（Rust `deep_entry_json_unsafe` 发 T/C/E 含 comment，JS 物化器 `_zwMBuildDeepTreeNode` 处理齐全）——责任在消费端写链。
- 钉测：`./target/test-guard --compile-first -- cargo test -p zero-engine --lib test_s10k12_writechain_knife12_pins` **1 passed**；终版代码复确认 1 passed（电池后重编译验证）。
- 本刀文件 `rustfmt --check` clean；clippy `-p zero-engine --lib` 0 warning（touch lib.rs 强制重检取证）。
- D/E 簇定谳挂账（48F，本刀不动）：解析器不产 foreign namespace——svg/math 元素全 xhtml ns、属性 ns 全 null（探针 R7/R8/R9）——parser 侧深项另刀。

## §3 电池（release 绑定，五腿 solo）

- 发射：`.acceptance/dps/s10k12-launch.sh`（md5 断言先行：v1 `7202718678…`→TT 回归修复重建后 v2 `563e5997d436c8f30e2a18ef3208a7e6`）；pre 锚＝k11 v2 电池（其后仅本刀三 shim＋钉测）。
- v1 电池（18:24:59Z–18:55:45Z）中途判读：SAN GONE=39/NEW=0 达标，但 **TT NEW=2**（trusted-types/script-enforcement-001/003 两脸 Pass→Fail，`Number of messages (RUN) expected 0 but got 1`——script 执行了）→定性＝α 修复移除旧捷径的偶然通过，暴露 TT enforced 页 setHTMLUnsafe script 容器信任链缺章→§1 补章修复→快验两脸 Pass→v1 作废、v2 重射全五腿。
- v2 终态（/tmp/s10k12-*-post-knife12-v2.log，18:59:00Z–19:29:43Z）：

| 腿 | pre (k11 v2) | post (k12 v2) | GONE | NEW | 结论 |
|----|----|----|----|----|----|
| WC | 18267P/1120F | 18385P/1118F | **2** | **0** | declarative-shadow-dom-opt-in document.open ×2＝β 级联（detached doc write 路径）；P+118＝截断落位（F 恒等面零回归） |
| DP | 1313P/222F | 1313P/222F | 0 | 0 | SET-IDENTICAL |
| SAN | 584P/172F | 623P/133F | **39** | **0** | F2P=39 精确/P2F=0/NET +39 |
| TT | 1544P/546F | 1548P/542F | **4** | **0** | v1 回归已修（NEW 2→0）；ShadowRoot.setHTMLUnsafe policy ×3＋knife3b2 probe9 ×1＝α 级联 |
| DOM | 57514P/496F | 57414P/496F | 0 | 0 | F 集逐键恒等；P−100＝截断落位 |

- SAN GONE 39 键分组（逐脸闭合 α/β/A）：basic-filtering 18（α text/0 ×4＋comments/1 ×4；β comments/0 parse+parseHTMLUnsafe ×2＋顺爆 namespaces/0 ×2；A dataAttributes/2 ×6）＋sethtml-tree-construction 12＋sethtml-context 6＋html5lib-basics 2＋sethtml-safety 1（后 21 全 α 形——非串输入/纯文本/上下文形均踩视图清空捷径）。

## §4 收账

- 刀⑫ 净增益 **+45**（SAN 39＋TT 4＋WC 2）；累计 83,143→**83,188**；距 ≥99,947 差 ≈16,759。
- 二进制字节绑定：终版 md5 `563e5997d436c8f30e2a18ef3208a7e6`；判读全部绑定该 md5 断言发射器。
- 残面（刀位候选）：basic-filtering 残 46F 主体＝D/E parser foreign namespace（svg/math 元素 ns＋xml:/xlink:/xmlns: 属性 ns，探针 R7/R8/R9 定谳 parser 侧深项）；javascript-url/parseHTML 族 22F；gethtml.html Timeout 页预算面。
