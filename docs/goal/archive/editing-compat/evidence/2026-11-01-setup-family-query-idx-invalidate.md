# setup-exception 族收口——M-form 树 attr 写入点补祖先查询索引失效（R307 attr 变体）（2026-11-01）

## 0. 结论速览

| 门禁 | gate8 基线 | 本轮 gate9 | 位移 |
|---|---|---|---|
| fontsize 真页 | 2,211P/211F | **2,301P/121F/0T** | F→P 90、P→F 0 |
| forecolor 真页 | 2,142P/185F | **2,233P/94F/0T** | F→P 91、P→F 0 |
| fontname 真页 | 1,984P/241F | **2,074P/151F/0T** | F→P 90、P→F 0 |
| backcolor 真页 | 875P/151F | **965P/61F/0T** | F→P 90、P→F 0 |
| hilitecolor 真页 | 1,054P/164F | **1,144P/74F/0T** | F→P 90、P→F 0 |

五语料合计 **8,717P/501F** 对 gate8 8,266P/952F：F→P **451**（90/91/90/90/90 逐语料）、P→F **0**、分母逐位恒等。setup 异常断言（`Setup must not throw an exception`）五语料 grep 全零（修复前每语料恒 96）。

残余族普查（501F，五语料合计）：compare innerHTML **256**、queryCommandValue **231**、queryCommandIndeterm **9**、queryCommandState **5**（上轮 return value 60 + checks for modifications 30 + setup 族全数回收）。

## 1. 根因链（单文件单机制洞）

editing conformance harness `setupDiv`（tests.js:5283）对表行语料（如 `<table><tbody><tr data-start=1 data-end=2><td>foo<td>bar<td>baz</table>`）的选区构建序列：

1. `testDiv.querySelector('[data-start]')` → 命中查询索引 `_zwNodeIdx`（首查建 DFS 索引，键 tag+id+outer——R355 统一键构造），返回树内**真节点** TR（`__zwIsMEl` 印章、3 子、父 TBODY）；
2. `tr.removeAttribute('data-start')` → `_zwMElRemoveAttribute` 改 attrs 数组——**不失效**祖先挂槽的 `_zwNodeIdx`/`_zwQWrapMap`；
3. `testDiv.querySelector('[data-end]')` → `_zwMQueryAll` 现序列化（outer 已无 data-start）产新键 → 陈旧索引恒 miss → `_zwMFindRealNode` 返 null → `_zwMWrapCached` 回落 `new _zwParseEl(info)`——**孤儿 wrapper**（parentNode null；`_ensureMutTree` 对 tr 上下文 outerHTML 解析产 0 子）；
4. 调用方 `range.setEnd(orphan, "2")` → part06.js setStart/setEnd `_nodeLength=0` 界检抛 IndexSizeError（R288 元素容器精确校验）→ 该 browserTest 行全部 setup 断言败（每语料恒 96 明细行）。

R307 既有失效只覆盖 append/remove 两树结构写点（part03 旧 :18680/:18934 及 fragment/part05 面）；attr 写入同族缺陷（键含 outer、attr 变更即键失真）漏网——R307 注释「靠 outer-key 差异近似」只保证旧键 miss，不保证新键命中真节点。

## 2. 修复（part03.js +21/−0）

`_zwMElQIdxInvalidate(node)` helper（R307 同款沿父链遍历，depth 64 防环：`_zwNodeIdx = null` + `_zwQWrapMap.clear()`）挂五个 attr 写入点：

- `_zwMElSetAttribute` / `_zwMElRemoveAttribute`（NS 变体 `_zwMElSetAttributeNS`/`_zwMElRemoveAttributeNS` 经委托同覆盖）；
- IDL 直写三件 `_zwIdSet` / `_zwClassNameSet` / `_zwSlotSet`（绕过 setAttribute 直写 attrs 数组；classList 写面同经此处——R307 注释「同步点审计：setter/_zwIdlSet 反射/classList 写/解析初灌」在册）。

失效后下次查询重建索引（现 outerHTML 键）→ `_zwMFindRealNode` 命中真节点 → `_zwBridgeGet` miss 补登（R308）返回节点本体——查询产物与 traverse/mutation 面跨面 identity 统一（R167 归一语义）。

## 3. 定谳手法

- **切片页复现收敛**：forecolor.html `browserTests.forEach` sed 成 `slice(0,N)`——slice(0,60) 0 setup fails / slice(0,61) 16（第 61 browserTest 行 = 首个 data-start 表行；数组真实索引 node eval 计数，多行条目按行数会错）。状态依赖：单发页不可复现（前置累积态必需）。
- **ZZDBG 诊断抛**（tests.js setupDiv 临时插桩，异常经 "Setup must not throw" 消息携带 JSON）：
  - 修复前 `{"sn":"TR","so":"1","en":"TR","eo":"2","sl":3,"el":0,"same":false,"eParent":"null","sIsM":"boolean","eIsM":"undefined","ss":"ok","err":"IndexSizeError"}`——终点为孤儿（非 M-form 非 handle-proxy、0 子、无父）；
  - 修复后 `{"el":3,"eParent":"TBODY","same":true,"eIsM":"boolean","ss":"ok","se":"ok"}`——终点为真节点（identity 归一命中）。
- 插桩已整块还原（`return [startNode, startOffset, endNode, endOffset];` 原样恢复）；切片/探针页均为 gitignored 临时工件不入库。

## 4. 验证批（gate9 串行，test-guard 包裹，证据 /tmp/zw-edit-v/g9-*）

- **五语料**：见 §0 表。整行分母逐位恒等（2,422/2,327/2,225/1,026/1,218）。
- **位移电池（other 面）逐位恒等**：join-paragraphs 3,456P/0F、join-lrp 2,607P/0F、join-lpr 1,455P/0F、wss-ilb 552P/0F、wss-ip 1,104P/0F、chd 420P/0F、ipihost 880P/0F、void-ip 72P/0F、void-it 48P/0F、inline-host 宽 filter 0/0（伪欠账定谳同前轮）。
- **ilb（run 面）正向连带**：1,633P/58F → **1,641P/50F**（F→P 8、P→F 0——同族 setup 形连带回收）。
- **engine lib 3,667P/0F**：gate8 3,664P/1F 的 s24 红已由 rebase 窗 PR #267（site-opt/s24-flip-red）修复，差额 +3 测试为同窗并行流新增；零失败。
- **webview lib 834P/0F**：`env -u` 五代理变量清空跑法（shell HTTP(S)_PROXY 继承致 fetch_url_invalid_host 假红——既有 memory 在案）。
- **wcomp 4,022P/1,146F/5T**：计数级恒等。
- **hsem 6,905P/660F/162T**（gate8 6,903P/662F/162T，总 7,727 恒等）逐案归因：
  - F→P 4：form-action-reflection-with-base-url ×2 + the-output-element/mutations ×2——均 attr/mutation 变异邻近面（action 移除后 IDL 反射回落 document URL；output 派生值随后代变异重算），与本修复方向一致、因果相容；
  - P→F 2：checkable-active-onblur ×2——`:active` 时序 flakes 既有家族（gate8 账在册成员同族）；独立单语料复跑 Checkbox 案转 Pass、Radio 案仍 Fail，两跑异果定谳间歇 flakes，非确定性回归（本修复面为查询缓存失效，与 ：active 清除无因果路径）。
- **bench rc=0**：absolute page-total / retained-form budgets passed（硬件 mismatch WARN 既有，relative gates 不适用）。

## 5. 证据文件

- 批日志：`/tmp/zw-edit-v/g9-*.out`（五语料 + 电池）、`g9-status.log`（串行账）、`g9-enginelib.log`/`g9-webviewlib.log`/`g9-wcomp.log`/`g9-hsem-run.log`/`g9-bench.log`、`g9-active-rerun.log`（:active 复跑定谳）——tmp 清理后失效，终数已录本文档；
- 切片复现：`zz-fc-slice.html`（slice(0,61)）、修复前后对照 `slice0-60.out`/`slice0-61.out` vs `slice-fix2-61.out`（853P/23F/0 setup → 868P/24F/0 setup）。

## 6. 下一步接口

- compare innerHTML **256F**（apply 缺口②续刀）、queryCommandValue **231F**（fontname sans-serif 族为主刀）；
- enabled 端点门扩其余语料（bold tt 同形同期望已实证，本轮五值命令域外不动）；
- ilb 残余 50F 为表格族 insertlinebreak 既有失败（前轮定谳），setup 族连带回收后维持既有账。
