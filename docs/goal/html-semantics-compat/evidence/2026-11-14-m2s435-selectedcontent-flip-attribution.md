# M2 s435 — selectedcontent-listbox 红利案翻绿机制归因（零源码改动轮）

日期：2026-11-14。前情：s433 全域双跑 stable-P +2 中「红利案」
`selectedcontent-listbox :: <select size=4> should support <selectedcontent>.`
F→P 双跑一致，当时注记「页面无 form 读路径，与本轮改动直链未逐案归因」。
本切片全树 bisect + 刀级 A/B 定谳：**翻绿属同窗他流 site-opt t68（PR #302）
cd29066a3 的 innerHTML 融合 getter 空串折叠修复，与 s433 FORM named-first
六刀零因果**——s433 记账归属更正（驱动案 nameditem 真翻绿；红利案系
时序相关、机制无关的他流贡献）。

## bisect 归因（criterion 二值化，语义反转）

判据脚本 bad iff 页面 PASS（git-good = 旧行为 F / git-bad = 新行为 P，
first bad = 首个翻绿提交）；`make testharness-html-semantics
FILTER=selectedcontent-listbox` solo 跑（make 门禁入口），build 失败
exit 125 skip。范围 [9f66f2bbe（s432 终态基线）..HEAD（3b4747cd2）]，
6 腿定谳：

| 腿 | 判据 | 结果 |
|---|---|---|
| 313ab873f（R5527 docs） | PASS | bad |
| aaadeec3f | no-Pass rc=2（脏腿：runner 运行失败非干净 F，后被父腿补验覆盖） | good |
| 376dfb3c1（t68 merge） | PASS | bad |
| 005619c62 / 98f4b3ac6 | PASS | bad ×2 |
| **cd29066a3（t68 sel/handle 双族注册表 sync，site-opt 流 PR #302 合入）** | **PASS** | **first bad = 首个翻绿** |

父提交补验：d1545546b（cd29066a3^）solo **F**，失败签名与 s432 基线三腿
（sweep2/4/6 存证）逐字同——`assert_equals: <selectedcontent> should
respond to assigning to option.selected. expected "" but got "one"`。
工件 /tmp/s435/{bisect-main.log,bisect-log.txt,bisect-legs.log,
parent-run.log,ab-run.log,head.log}。

## 机制勘根（断言②链 + 刀级 A/B）

测试三断言：①初始 selectedcontent.innerHTML === 'one'（解析期同步，
两代均过）；②`optionOne.selected = false` 后 innerHTML 立即 === ''
（**基线红面**）；③click 回填（②红即中止未达）。

基线红链（四步，全部代码据点在案）：

1. spec：非 multiple select、`option.selected = false`、display size=4
   （≠1 无回落）→ 清空后 selectedness 全 false（HTML §4.10.7 ask for a
   reset——shim `_zwAskForReset` + `_zwSelectednessSetting`，s324 期实现）；
2. selected setter 步尾同步（part03 `_zwOptNearestSelect` →
   `_zwSelectedContentUpdate`，s324 期实现——本窗口 diff 零触碰）→
   `_zwScCloneInto(sc, chosen=null)` → `sc.innerHTML = ''`（单记录
   replace-all 通路，part03 `_zwScCloneInto` 注记）；
3. **宿主写异步 apply**（part05 融合臂头注）——写记入
   `_zwPendingByParent` 桶（removed=[唯一文本子]，added 空），同 turn
   尚未 drain；
4. 断言立即 GET `selectedcontent.innerHTML` → sel 域融合 getter
   `_zwFusedInnerHtmlSel`（part05）三臂门激活（桶 removed 非空 + stamp
   同代际）→ pending 融合视图 walk 产出**合法空串** →
   **旧折叠 `return _r380Out !== '' ? _r380Out : null` 把空串折叠成
   null** → 调用方回落 host 直读 → host 快照未 drain 仍含 'one' →
   断言得 'one' 红。

cd29066a3 修复 hunk（part05 `_zwFusedInnerHtmlSel` 尾）：空串是合法融合
结果、原样返回（t68 四面之「removeChild 序列化空串保留」；t67-minrepro
caseC 同签名「innerHTML 读回陈旧内容」）——同构于本页断言②形态。

**刀级 A/B（hunk 级因果钉死）**：HEAD 仅回退该一行（`return _r380Out;` →
旧折叠）→ solo **复红同签名**；还原后 solo P（head.log 在案）。窗口内
cd29066a3 其余 573 行（fragment wire 补齐/drain 后 identity 反查/QSA class
归并/removeChild 空串保留其余臂）与本页零交集——静态标记页无 fragment
插入、无 removeChild、无动态 QSA。

## 证据矩阵

| 树 | solo 判据 | 结果 |
|---|---|---|
| 9f66f2bbe（s432 终态） | s432 sweep2/4/6 存证 | F 同签名 |
| d1545546b（cd29066a3^） | 本轮补验 | F 同签名 |
| cd29066a3 | bisect 判据腿 | P |
| HEAD + 仅回退折叠行 | 本轮 A/B | F 同签名 |
| HEAD（含折叠修复） | 本轮 head.log | P |

## 结论与影响

- s433 账本「stable-P +2（驱动案 + 红利案）」的红利案归属更正：
  selectedcontent-listbox 翻绿 = 同窗他流 t68 cd29066a3 的融合 getter
  空串折叠修复；s433 六刀贡献 = nameditem 驱动案（该案机制已由 s433
  账本施工面自证）。两案 stable-P +2 净账不变，归属注记以本账为准。
- 方法论注记：单案翻绿的「红利案」在双流并行窗口内**不可默认归属本流
  施工**——组合态窗口含他流合并时，翻绿案须 bisect 归因后再记账
  （归因纪律 run-rules #10 的翻绿面推广）。
- 无源码改动、无 corpus 账面位移（判据案本就在册 P/F 翻转记录，全域
  口径零变化）；探针无残留（A/B 行已还原，工作树干净）。
