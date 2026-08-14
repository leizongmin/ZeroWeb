# M4 Slice R19 — classList replace 校验顺序/去重/mutation + assignment readonly + identity 缓存

**日期**: 2026-08-14
**里程碑**: M4 — WPT dom 上游基线 + 按聚类驱动修复
**切片**: R19
**前置**: R16（classList toggle no-op + write runUpdate + replace 顺序，遗留 20F）/ R13（classList 去重 + contains 空白不抛）

## 问题

WPT `dom/nodes/Element-classlist.html` 剩 20F（R16 后），5 个 distinct 测试 × 5 节点类型（HTML/XHTML/MathML/XML/foo），实为 4 个 distinct bug：

1. **`classList.replace(" ","")` 抛错类型错误**（5 subtest）：期望 SyntaxError，实际抛 InvalidCharacterError。spec `dom-domtokenlist-replace` 校验顺序**特殊**——两 token 的空串 SyntaxError 先于两 token 的 ASCII 空白 InvalidCharacterError（区别 add/remove 逐参先空后空白），故 `replace(" ","")` 的 newT="" 先抛 SyntaxError（非 oldT=" " 空白）。
2. **`classList.replace("c","a") in "a b c"` → "a b a"**（5 subtest）：R16 去重循环 `j > i` 边界 off-by-one，splice 后 i 位置的重复 newT 没移除。
3. **`classList.replace("a","a","a")` mutation 时机**（5 subtest）：oldT===newT 存在时 `write(cur())` 被 runUpdate「值相同 return」吞 mutation，spec 要求返 true 时必触发 mutation。
4. **`classList = x` assignment 覆盖**（5 subtest）：classList 无 set trap 拦截 → 落入 generic expando 被串覆盖；且每次 get 新建 Proxy → identity 不等（WPT `assert_equals(e.classList, expect)` 要求 cached accessor 同一对象）。

## 修复

纯 shim 修复（part01/part03/part04），零碰撞：

1. **part03 `replace` 校验顺序**：改两阶段——先两 token 空串 SyntaxError，再两 token 空白 InvalidCharacterError（`globalThis.DOMException` 保 identity）。
2. **part03 `replace` 去重算法重写**：`splice(i,1,newT)` 后全局有序去重（seen 表，首个保留），统一覆盖所有 replace 情形（含 newT 在 oldT 前后）。
3. **part03 `write(arr, force)` 加 force 参数**：replace 返 true 时 `write(p, true)` 强制 setAttribute + notify，绕过 runUpdate「值相同 return」。
4. **part04 元素 set trap 加 `classList` readonly 分支**：return true（no-op，早于 className/generic fallthrough）。
5. **part01 `_clsProxyCache` + part03 `_classListProxy` per-element 缓存**：同 `_proxyCache` 模式，cache hit 返回同一 DOMTokenList proxy（spec cached accessor identity）。

## 验证

- **单测**：新增 `test_classlist_replace_validation_dedup_mutation_and_assignment_r19`（校验顺序 replace(" ","")→SyntaxError + 去重 replace("c","a")→"a b"/"c b a"→"a b" + oldT===newT 返 true/规范化 + assignment no-op + identity `===`）。v8 矩阵 pass。
- **6 个 classlist 测试全绿**（含 R19 + R13 + R3032 full + A/B 门 + consecutive ops + polyfill API）。
- **clippy 双矩阵**：v8 + quickjs 零警告；fmt 干净。
- **WPT Element-classlist.html 双路径**：R18 1400P/20F → R19 **1420P/0F（100%）**，双路径对等。
- **WPT dom/nodes 全量双路径**（完整 JSON 入 evidence）：

  | 路径 | R18 | R19 | Δ |
  |---|---|---|---|
  | Polyfill | 55.11%（2481P） | **55.55%（2501P）** | **+0.44pp / +20P** |
  | Native | 54.46%（2452P） | **54.91%（2472P）** | **+0.45pp / +20P** |
  | 双路径差 | 0.65pp | **0.64pp** | 收敛 ✓ |

## 决策记录

- **replace 校验顺序为何与 add/remove 不同**：spec `DOMTokenList.replace` 算法的 4 步校验是「两空串 → 两空白」，而 `add`/`remove` 是「逐参先空后空白」。WPT checkReplace(null," ","",...,"SyntaxError") 证实。这是 replace 的 spec 特例，需独立校验逻辑（不能用通用 check 函数）。
- **classList assignment strict 模式**：spec strict 下应抛 TypeError（readonly accessor 无 setter）。WPT `assignToClassListStrict` 无断言（仅执行），故 set trap 返 true（no-op）即可通过。proxy set trap 返 true = 赋值「成功」语义，strict 下不抛。
- **identity 缓存为何必要**：spec classList 是 cached accessor（`[SameObject]`），每次访问同一 DOMTokenList。WPT `var expect=e.classList; e.classList="foo"; assert_equals(e.classList, expect)`——若每次 get 新建对象则 `!==`。`_clsProxyCache[key]` 保证同元素 identity，与 `_proxyCache` 元素代理缓存同模式。

## 残留（非本切片）

- Element-classlist.html 全绿（0F），classlist 子集无残留
- 其他失败聚类见 master.md「剩余聚类」（iframe.contentDocument / querySelector-mixed-case / createEvent 6F / canvas proxy instanceof / polyfill appendChild 闭环）
