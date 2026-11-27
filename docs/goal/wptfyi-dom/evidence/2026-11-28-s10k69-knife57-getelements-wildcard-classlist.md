# 刀57（s10k69）：getElements '*' 非元素补门 ＋ classList.supports 无表抛 TypeError

- 日期：2026-11-28
- 刀体：8a620f6a8（`fix(engine): getElements '*' 非元素补门＋classList.supports 无表抛 TypeError＝dom 233F−8`）
- 二进制绑定：bin `74b68e49b4b5`（rebuild1-4 四次构建 md5 恒定，rebase 重放后复验）
- 电池：chain69（san→dom→tt→ref），发射 21:58:38Z，EXPECT_BIN/EXPECT_HEAD 前置断言过

## 根因一：getElements wildcard 3F（Element-getElementsByTagName 1F ＋ Element-getElementsByTagNameNS 2F）

### 排查链（探针 zz-k57-probe{,2,3,4,5,6,7}，跑后删除未入库）

1. probe1（sel 域 host div）：`getElementsByTagName('*')` 正常 7 元素——sel 域无毒。
2. probe2（detached createElement/innerHTML 形）：typeof 正常——基本构造无毒。
3. probe3（复刻 page setup 精确形）：typeof 正常——**只查 typeof 漏了 length**（教训：断言带精确期望）。
4. probe4（单外域子 TEST:AÇ 残留）：正常——外域单子无毒。
5. fixture 全文重读：wildcard 测试前 15 个测试，两类清理形态（t1-t4 add_cleanup 批量 vs 外域元素逐个 add_cleanup）。
6. probe5（无 cleanup 残留复刻＋干净复刻）：双双 typeof 正常——**mutation 残留假说出局**。
7. probe6（检查点全序列复刻）：撕开真相——`gEBT(*)` 返回 `ELS=[P,A,B,EM,U,#text,#comment]` len 7＝**5 元素＋Text＋Comment**。原失败消息 `got object "string" length 7` 的 "string" 是 format_value 渲染误导，非 typeof。
8. probe7（fingerprint）：element 为 handle 形（`__n0`）；同元素 `QSA(*)`=5 纯元素 vs `gEBT(*)`=7——分叉在集合构建层非 walk 层。

### 机制闭合

- `document.createTextNode`/`createComment`/`createProcessingInstruction`（part06:7763/7770/7790）返回 `_wrapHandle(handle)`——**带 `__zwHandle` 的完整 proxy 但 nodeType 3/8/7**。
- `_handleChildren[handle]` 存全节点树（`_handleSubtreeNodes` 的 nodeType 过滤为证）→ text/comment 是父容器的 pending 子，进 `_zwPendBucket` 作用域桶。
- `_zwMakeCollection` 构建期并入（part05:15111）逐候选跑 `matches`＝`_zwLiveMatchesFor('*')`→`_zwFilterByTagNameNS` '*' 快道。
- '*' 快道 s433 身份判定（`__zwSelector != null || __zwHandle != null`，前提「proxy 构造上恒元素」）将 text/comment wrapper 误判元素→保留→7。
- 具名 tag 查询不受影响：主循环身份通过后 localName/tagName 比较天然滤除 text（'#text' ≠ 'pre'）。仅 '*' 语义面（spec qualifiedName '*' 无比较步骤）破。
- gEBTns 空 nsWant 变体（`('', '*')` 等）：主循环 ns 读 null → nsActual ''==nsWant '' 放行＋localName '*' 恒真→text/comment 保留（`Empty string namespace` got 2）。

### 修复（刀体四处）

| 文件 | 位置 | 改动 |
|---|---|---|
| part05.js | `_zwFilterByTagNameNS` 前 | 新增 `_zwNonElNodeEntry(el)`：`_textHandles`/`_commentHandles`/`_piHandles` 三表＋`__zwIsText` 印记判读 |
| part05.js | '*' 快道 push 条件 | `&& !_zwNonElNodeEntry(_se)` |
| part05.js | 主循环身份通过后 | `if (_zwNonElNodeEntry(el)) continue;` |
| part05.js | `_zwDocAllElements` push | 同门（文档级 `getElementsByTagName('*')` 同族面） |
| part01c.js | move/wrap 流 `__zw_create_text` | 补 `_textHandles[_b2h] = true` stamp（三表完备性前提；全创建点 stamp 审计闭合） |

- QSA 不受影响：`_handleSubtreeNodes`（part05:10055）本有 nodeType===1 过滤。
- R12/s433 既有保护不回退：FORM named-first gate 面未触（三表判读不经 proxy 属性读，`__zwHandle` 走 R351 内部键短路）。

## 根因二：classList.supports 5F（Element-classlist）

- 判据：`e.classList.supports("a")` 五 ns 形（HTML/XHTML/MathML/XML null/foo）`assert_throws_js(TypeError)`。
- spec dom-domtokenlist-supports 步骤 2：关联属性无 supported tokens → TypeError。我方无表臂恒返 false（旧注释假设「基线不测 supports」与语料矛盾）。
- 修复：`_classListProxy`（part03:19448）无表臂翻 `throw new TypeError(...)`；a/area relList（spec 实有 tokens、表未钉 R5271 FIXME）调用点传 `[]` 空表保持恒 false 不误抛；link relList/sandbox/controlsList/blocking 四有表路径字节不变。
- 语料爆炸半径 grep 收口：`.supports(` 消费面全为有表页（link-rellist/preload helper/sriharness/common.sub.js 均 link；sandbox/controlsList/blocking 各自表格）。

## solo 验证

| 页 | 前 | 后 |
|---|---|---|
| dom/nodes/Element-getElementsByTagName.html | 1F/19 行 | 19P |
| dom/nodes/Element-getElementsByTagNameNS.html | 2F/16 行 | 16P |
| dom/nodes/Document-getElementsByTagNameNS.html | 14P | 14P |
| dom/nodes/Element-classlist.html | 5F/1420 行 | 1420P |
| dom/nodes/Document-getElementsByTagName.html | — | 18P |
| dom/nodes/Element-getElementsByClassName.html | — | 3P |
| dom/nodes/Document-getElementsByClassName.html | — | 1P |
| dom/nodes/Document-createProcessingInstruction.html | — | 12P |

- 基线锚：chain68 dom 腿 `/tmp/s10k68-leg-dom.log` 三 F 行（wildcard/Empty string ns/('*','*')）全数清零。
- 电池基线：chain68 dom 233F → 预期 225F（−8：3 gEBT 族＋5 classlist）；census 预期 84,520→84,528（ΔP＝+8 翻面±截断噪声带）。
- html curated 语料仅 controlsList.tentative.html 含 supports 面（testharness.rs 843 页清单核）；solo 文件参数对 testharness-html 类别不产行（语料 curated 非单层扫描），四有表消费页由表路径字节不变＋grep 收口兜底。

## 经验

- 失败消息的 format_value 渲染可误导类型归因（"string" 实为集合含非元素节点的渲染碎片）——归因前先程序化展开集合元素身份。
- 断言探针必须带精确期望（len/元素清单），只查 typeof 会让 len=7 现象漏过四轮（probe1-4 全中此坑）。
- s433 身份判定的前提「proxy 构造上恒元素」在 text/comment/PI wrapper 上不成立——身份判定≠元素判定，三表补门是同构面的通用修正。
