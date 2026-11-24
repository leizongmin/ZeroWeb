# DC-2 切片 10 sanitizer 刀⑭ 判读证据——parseHTML 文档骨架保真（doc 级子列表挂载）

**日期** 2026-11-25 · **判定** 刀⑭ 落地，sanitizer 净 +20（70F→50F，parseHTML 22F→2F），五腿 join 零回归

## 刀面（4 文件，+283/−2，commit fa3134e8d）

1. `crates/engine/src/js_dom_shim/part03.js`：`_zwK14ExtractDoctype`——源串 DOCTYPE 提取铸 plain mock（nodeType 10＋name/publicId/systemId；isEqualNode 走 `_zwIsEqualNode` nodeType-10 字段面，期望侧 `implementation.createDocumentType` 真 handle 跨域配对）＋globalThis 导出（part 间非同闭包，函数声明不互见）；s180 static（`Document.parseHTMLUnsafe`，双路径单点漏斗）挂载块——DOCTYPE push＋`d.appendChild(d.documentElement)`（createHTMLDocument 同款文档级挂载语义）＋body 树首文本子前导 WS trim（空则 shift；真文档解析 before-html 模式吃除骨架间 WS）＋headEl childNodes 清零（工厂无条件 title 子，html5lib 期望 head 无子）。
2. `crates/engine/src/js_dom_shim/part13-sanitizer.js`：构造器 `__zwCommentsExplicitTrue`／`__zwDataAttrExplicitTrue` 显式真值印记（实例 dict 缺省 comments:true／dataAttributes:true 是 boolean-defaults `get()` 钉面不可动）；漏斗 `_zwSanParseHtmlImpl`——safe 注释剥离（`_zwK14StripComments` 递归；剥后 serialize 相邻文本并文自然合并＝tc3 safe "ab" 形）＋safe data-* 局部覆写（过滤调用前置 false、调用后恢复实例态；raw 显式 true 豁免）＋骨架 verdict 过滤（`_zwSanElementVerdict` 同源语义：elements 定义但未列→remove＝head 剥/html 剥整文档清空）＋DOCTYPE 从原始输入注入 unshift。
3. `crates/engine/src/js_dom_bridge_tests/part160_s10k14_parsehtml_skeleton_pins.rs`（新，140 行）：十三钉常驻（挂载遍历／head 无子／跨域 isEqualNode 双向／DOCTYPE 双路径／骨架 verdict 三形／注释 safe 剥并文＋unsafe 保留／unsafe 属性形／body 起始标签属性不并入／工厂流零扰 createHTMLDocument title 完好／空输入边界）。
4. `crates/engine/src/js_dom_bridge_tests.rs`：include 接入。

## 根因链（probe_tmp_k14 定谳）

- 单根因：detached doc 文档级 `childNodes` 是构造期静态空数组（part03 工厂 IIFE），`write()` 产物只落 body 活树 `_tree`（`body.childNodes` 活 getter 读 `_tree.childNodes`）——主文档迭代器以返回文档为根只达根节点，22 案同报 "Additional nodes at the of node2"；all tc2（空期望）双路假通过。
- corpus 机制：`assert_subtree_equals` 用 `document.createNodeIterator(doc)` 跳根对遍历＋isEqualNode 逐对＋尾查；expected 侧 build_node_tree 经 context div/new Document 包裹（探针 harness 初版直接根在 html 元素＝跳根错位假红，修正后对齐）。
- 次生面五项（probe W 系定谳）：headEl 工厂无条件 title 子；DOCTYPE 全丢（W5 null）；漏斗骨架零配置过滤（W9 docElKids 恒 2）；safe 注释不剥＋文本不并（W8）；safe data-* 缺省不剥（dbg safeAttrs [data-xyz,id]）；document tc `<!DOCTYPE html>\ntext` 前导 `\n` 未吃（dbg bodyKids ["\ntext"]）。
- `_zwIsEqualNode` 元素分支 ns/prefix undefined-null 归一＋属性序无关配对现成可用（bodyEq 双向 true 直证跨域配对；刀⑬ 钉③ 同源）。

## 电池判读（v1，发射器 `.acceptance/dps/s10k14-launch.sh`，md5 断言 `7904ea87822749290576aaf9c4c7cce4` 先行）

二进制 md5 `7904ea87822749290576aaf9c4c7cce4`（工作树刀⑭ 面 release 构建，＝commit fa3134e8d 字节域）；窗 20:58:15Z–21:29:03Z（30.8 分钟）；pre 锚＝k13 v1 五腿 F 集（1118/222/70/542/496，cp 至 /tmp/k14-*-preF.txt）；判读器 `.acceptance/dps/s10k14-judge.sh`（LC_ALL=C awk 提取＋comm 前对账＋在飞腿 EXIT 守卫）。

| 腿 | pre (k13 v1) | post (k14 v1) | GONE | P2F | 终态 |
|---|---|---|---|---|---|
| WC | 18454P/1118F | 18454P/1118F | 0 | 0 | F 集逐键恒等（T 20） |
| domparsing | 1313P/222F | 1313P/222F | 0 | 0 | F 集逐键恒等 |
| **sanitizer** | **686P/70F** | **706P/50F** | **20** | **0** | **F2P=20 精确/P2F=0/NET +20** |
| TT | 1548P/542F | 1548P/542F | 0 | 0 | F 集逐键恒等（T 64/NR 17 截断族） |
| dom | 57547P/496F | 57515P/496F | 0 | 0 | F 集逐键恒等；P−32＝截断落位族 |

**join GONE=20 / P2F=0 / 新见 F=0** —— 零附带损伤。SAN 判读与单腿真值预检（v2 单跑 706P/50F，同二进制）逐项一致；P 706＝686+20 恒等对账。

### SAN GONE 构成（20，全 sanitizer-parseHTML.html）

22F→2F：all 块 0/1/3/5 双路 8＋safe 块 0/1/2/3/5/6/7/8 共 8＋unsafe 块 2/3 共 2＋document 双路 2。safe tc0 `<script>hello` 翻转无玄机：safe 块期望本就 script 剥除（baseline remove）＋骨架挂载匹配 script-less 树；探针 c6 的 FAIL 预测对应 unsafe 块 tc0（期望 head>script 归置）——该案仍红 ✓。

### 残 2F 分布（挂账后续刀位）

- `parseHTMLUnsafe testcase 0, "<script>hello"`——script→head 归置 raw 路径（漏斗 safe 路已通，raw 臂 body>script）
- `parseHTMLUnsafe testcase 1, "<html onload=...><body onload=...>"`——html/body 属性捕获（docEl 无属性面，须补 attr 面＋消毒配置过滤）

## 验证链

- probe_tmp_k14 corpus 矩阵 15/17（c6/c7＝挂账两形）；dbg 三项（safe attrs id-only／unsafe data-xyz+id／document tc body 首子 "text"）。
- 钉测 `test_s10k14_parsehtml_skeleton_knife14_pins` 1 passed；fmt `--all --check` 零 diff；clippy zero-engine 0 warning。
- 全量 zero-engine lib 4272 passed＋1 failed＝`test_dialog_focus_restore_declarative_shadow_s330`——stash 刀体后 HEAD 态同红＝前科红（非本刀回归）。
- 临时探针 probe_tmp_k14.rs／k14dbg_tmp.rs＋include 行已删（commit 前纪律）。
- 刀体先入库：commit fa3134e8d（guard PASS 0 发现）先于发射；二进制 md5 断言绑定发射域。
