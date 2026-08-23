# R209 Evidence — iframe/detached 工厂节点方法面 + surround/insert 叶子 newParent 异常链

**日期**: 2026-08-24
**切片**: M4——R208 解锁的 r1+ 轮新聚类（testNodes 方法面缺失 + host 不按 spec 抛异常）
**改动面**: `part03.js` / `part05.js` / `part06.js` / `part01b.js`（shim 四件）+ `dom_bindings/dom_exception.rs`（native 实例 legacy 常量）+ `part21.rs`（回归单测）

## 一、根因（探针链实证：win.run() + testNodeInput 直驱 22 形态）

R208 修复查询缓存后，surround/insert 的 r1+ 轮跑到断言层，暴露两大簇：

1. **工厂节点形态缺方法面**（探针逐形态实测 `typeof`）：
   - iframe `createTextNode` 产物（`paras[0].firstChild` 等 own=doc 域）：缺
     compareDocumentPosition / hasChildNodes / cloneNode / substringData / splitText
   - iframe `createElement` 产物（`paras[0]` / `detachedPara1` / `detachedDiv`）：缺
     compareDocumentPosition / cloneNode(deep) / isEqualNode / insertBefore
   - detached doc `createDocumentFragment` 产物（`docfrag`）：缺 cloneNode / isEqualNode
   - detached doc `createProcessingInstruction` 产物（PI 域）：缺 CharacterData 方法面
     （appendData 族 + 叶子 mutation 抛）
   - iframe doc 无 `doctype`（Range-test-iframe.html 有 `<!doctype html>` 但
     `_zwMakeIframeDoc` 不解析——common.js `doctype = document.doctype` 得 undefined）
   - `textContent` setter / `append` / `prepend` 的内联 Text 字面量：轻量形态无方法面
2. **host surroundContents / insertNode 不按 spec 抛**：
   - 叶子 newParent（Text/Comment/PI）：spec 步骤 5 `newParent.appendChild(fragment)`
     必抛 HRE——host 旧吞错/defer-no-op（探针 "HOST DID NOT THROW"）
   - `insertNode` 无 Text startContainer 的 splitText 路径（spec 步骤「split it with
     offset」）——折叠 surround 的树中间态与模拟分歧
   - `startContainer === node` 无 HRE 早退（spec 步骤 1 的「or is node」分支）
3. **DOMException 实例缺 legacy code 常量**：common.js `getDomExceptionName` 经
   `for (prop in e)` 找 `^[A-Z_]+_ERR$` 且 `e[prop] === e.code` 反查异常名——实例上
   这些常量不可枚举 → "Exception seems to not be a DOMException"

## 二、修复（七件）

| # | 位置 | 内容 |
|---|------|------|
| ① | part05 `doc.createTextNode` | 重写：完整方法面（cDP/hCN/cloneNode/isEqualNode/splitText 本地版 + `_zwAttachCharacterDataMethods` appendData 族 + `_zwMDefineSiblings`）+ 叶子 mutation 族抛 HRE |
| ② | part05 `_zwIframeCreateElement` | 补 contains/cDP/isEqualNode（属性+子递归）/cloneNode(deep)（属性复制+子递归）/insertBefore（ref=null 尾插语义） |
| ③ | part05 `_zwMakeIframeDoc` | `<!doctype>` regex 提取 → 静态 DocumentType（name/publicId/systemId + 完整节点面 + cloneNode 经 implementation）挂 `doc.doctype` getter（**不入 childNodes**——restoreIframe 清理循环/ referenceDoc removeChild 语义零扰动，探针实证入 childNodes 反而 -330P） |
| ④ | part05 textContent setter + append/prepend（含 docEl.append） | 内联 Text 字面量统一改经 `doc.createTextNode`（方法面完整） |
| ⑤ | part03 detached doc | fragment 补 isEqualNode/cloneNode(deep)；PI n7 补 `_zwAttachCharacterDataMethods`；`_zwAttachCharacterDataMethods` 对叶子 nodeType（3/4/7/8/10）挂 mutation 族 HRE 抛（removeChild 先 NotFound 校验后 HRE——r126 spec 校验序）；doctype dt 补同款 |
| ⑥ | part06 `surroundContents` | 叶子 newParent 先走 insertNode 树变更（折叠路径 split 中间态与模拟对齐）再抛 HRE；Document/Doctype newParent 抛 InvalidNodeTypeError（spec 步骤 1） |
| ⑦ | part06 `insertNode` | Text startContainer 的 splitText 路径（split → insertBefore(node, tail) → setEnd(parent, newOffset) 折叠同步）；`startContainer === node` HRE 早退 |
| ⑧ | `dom_bindings/dom_exception.rs` | native `fill_instance` 补 code≠0 对应的**单个** legacy 常量（可枚举，`legacy_name_for_code` 反查表）——与 polyfill 侧 A/B 对齐 |

**part01b**：DOMException 实例挂 code≠0 对应的**单个** legacy 常量（可枚举，
`_ZW_DE_LEGACY_BY_CODE` 反查表）。

spec 依据：
- https://dom.spec.whatwg.org/#dom-range-surroundcontents
- https://dom.spec.whatwg.org/#dom-range-insertnode
- https://dom.spec.whatwg.org/#concept-text-split
- https://dom.spec.whatwg.org/#dom-node-pre-insert

## 三、验证链

- **单文件**：surroundContents **645P/1195F → 733P/1107F（+88P）**；insertNode
  **487P → 628P（+141P）**（native 侧 surroundContents 同为 733P 逐计数一致）
- **全量（polyfill）**：R208 基线 50796P/4238F/21T → **51096P/3937F/22T（净 +300P/-301F）**
- **全量（native 对照）**：**51098P/3937F/20T**——status flips 仅 2（均为既存
  harness flaky 的 Timeout↔Pass：insertBefore-iframe-crash / EventListener-incumbent-
  global-subframe-2，历史轮次同形态）；passive-by-default 的 only-in 差异为用例
  subtest 名 wording 漂移（既存），非行为分歧
- **逐 subtest 转移比对**（surroundContents before/after 同键）：
  Pass→Fail 198（全为 positionTests 的「模拟侧修好、host 侧真实缺口暴露」——
  1,x 族非折叠文本 extract 语义是下一层缺口，非回归；Fail→Pass 286）
- **engine 单测**：2348 全绿（新增 `test_iframe_testnodes_method_face_r209`
  ——13 形态方法面 + doctype 可读 + surround 叶子异常/树中间态 + insertNode
  自引用 HRE + DOMException legacy 常量五断言组）
- **fmt / clippy**：`cargo fmt --all -- --check` 零 diff；`cargo clippy --workspace
  --all-targets -- -D warnings` 零警告
- **make test**：全绿 except `window_surface_present_smoke`（XOpenDisplayFailed
  无显示环境，R203–R208 同款豁免，run-rules §10）

## 四、过程教训

1. **两侧对称性破坏 = 净回归**：单侧（模拟 or host）修好会使 positionTests 的
   树比较分歧——baseline 里「双方同样坏」的 Pass 是假绿；修一侧必须把另一侧
   的 spec 行为同步拉齐（本轮 surround HRE 链的四次迭代：提前抛 → 吞错 →
   先变更后抛 → 校验序）
2. **修复顺序即语义**：`removeChild` 的 NotFound 校验先于叶子 HRE（r126 抓回）；
   surroundContents 的 InvalidNodeTypeError（步骤 1）先于树变更、叶子 HRE
   （步骤 5）后于树变更
3. **结构挂点零扰动**：doctype 入 childNodes 看似 spec（文档序）实则扰动
   restoreIframe 清理节奏（-330P 实测）——getter-only 满足消费面即可

## 五、commit

（落盘时待填——见 master.md R209 行）
