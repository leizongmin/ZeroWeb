# R336 — MO record 三处补口（2026-08-28）

## 复核定性与归因

R331 记档的「MO Timeout 族全量并发下慢、非死循环」假设被本轮**推翻**。四个文件级
Timeout（attributes / characterData / childList / inner-outer）逐个 single-run +
pending 计数归因，全部是**真实 record 缺口**导致的 async_test 挂起：

| 文件 | pending | 缺口 |
|------|---------|------|
| attributes | 1 | `Attr.value` 同值 set 不发 record（R122 幂等护栏吞 set） |
| characterData | 2 | handle 文本 CharacterData 四方法不发 record（R3034 只接 setter） |
| childList | 1 | （R335 已修，n91 splitText） |
| inner-outer | 1 | outerHTML record target 错（自身而非父）且 removed 空 |

## 修复

1. **part03 `_r122Set`**：同值分支只补通知不传播（互写环防护保持）——
   `_mo_notify(ownerSel, ownerHandle, {type:'attributes', attributeName, oldValue})`。
2. **part04 handle 文本四方法**：append/delete/insert/replaceData 编辑后发
   characterData record（oldValue 仅 `_mo_any_wants_char_old` 请求时捕获）。
3. **part04 outerHTML setter（sel 域）**：新增父 target record
   （removed=[自身 proxy] + added=[解析 wrapper]），R3031 自身 target record 保留兼容；
   wrapper 挂 `_zwSelPendingParent` 槽（R304 同款）保证融合视图可见性。

## 验证

- MutationObserver-attributes **42/42 全 Pass**（41P/1T → 42P）
- MutationObserver-inner-outer **3/3 全 Pass**
- MutationObserver-characterData 23P 全 Pass
- MutationObserver-childList 38/38 维持（R335）
- MO 全族 **134P/4F/0T**（fail 集恒等备档：cross-realm 1F + document parser 3F）
- engine v8 2474（+1 回归测试）/ quickjs 1467 绿；tab 38P + renderer R2929/R2930 绿
- fmt/clippy 双矩阵干净

## 教训

宏观定性（「并发下慢」）要随新证据持续复核：single-run + pending 计数是区分
「环境慢」与「record 缺口挂起」的最小判据。挂起测试的 pending 计数在修复过程中
逐步收敛（13→4→2→1→0），每一步都可独立验证。
