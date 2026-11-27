# s10k71 刀59：MO subtree 冒泡补 handle 反链＋Range.deleteContents 兜底去重

- 日期：2026-11-28（UTC 23:12 发射）
- 基线：chain70（刀58 后 dom 232F＋12T；bin 2ec3aca260c7＝4bcb46c34）
- 刀体：172bf3d4e（直推无 rebase 重放；part01.js +20／part06.js +12）
- 字节绑定：bin d22d10dd9ad8＝172bf3d4e；发射断言＋终 md5 恒等（批中零重建）
- 电池：chain71 pid 3068712，17.5min，EXIT=1 与基线同形

## 目标键（3 键）

| # | 键 | 根因 | 修复 |
|---|----|------|------|
| 1 | ParentNode-replaceChildren :: "There should be a MutationRecord for the node removed from another parent node."（T） | part01 `_mo_notify` R3026 冒泡臂 `if (sel && ...)` 对 handle-only 整段 defer——detached 树（root=createElement）内 observe(a,{subtree}) 收不到后代 b 容器 replace-all 记录（探针定谳 callback 收 n=2 缺 #2：tgt=b rm=1 add=2）→ assert 异常 → Timeout | 冒泡臂补 handle-only 分支：`_zwNodeParent` 反链爬 handle 祖先逐级 `_mo_deliverToId('h:'+parentHandle, rec, true)`；doc 站不投（detached 不属 document）维持既有语义 |
| 2 | ParentNode-replaceChildren :: "There should be MutationRecords for the nodes removed from another parent node."（T） | 同上（双参形态同根因） | 同上 |
| 3 | MutationObserver-childList :: "childList Range.deleteContents: child removal mutation"（F） | part06 R3254-E2 同容器元素端点分支：removeChild 通道成功时（trap 自发带 sibling 锚记录＋host mutation 重读剔桶）兜底 splice 不命中，但兜底 `_mo_notify` 无条件发——sel 树每子双记录（探针定谳 n=2：#0 带 next=p4last ✓、#1 无 sibling spurious）→ 期望 n=1 断言败 | 桶判定去重：兜底 splice/notify 收进 `indexOf >= 0` 门内——仅通道静默失败/plain wrapper 未记账（桶未剔）时 relink＋notify |

spec 依据：MutationObserver observe 步骤——subtree 观察按 target 的树祖先链投递后代 mutation，与节点是否 in-document 无关（detached 树同有祖先链）。

## 判定（chain71）

- dom：57637P，232F+12T → **229F+10T**（−3 恰 3 目标键、零新增，comm 双向）
- san 5F／tt 32F 逐字节恒等（comm -3 diff_lines=0）；ref rc=0 204/204 100%
- solo：ParentNode-replaceChildren 31P/0F、MutationObserver-childList 38P/0F；MO 族 10 页零漂移（characterData 23P／attributes 42P／takeRecords 3P／inner-outer 3P／callback-arguments 1P／disconnect 2P／textContent 4P／sanity 16P／nested-crash 1P／cross-realm 1P）
- 探针双页（zz-k59-probe3/4，跑后已删）三岔定谳：probe3 修后 n=3（#2 tgt=b add=2 冒泡兑现）；probe4 修后 n=1（next=p4last 去重兑现）
- 探针自身 bug 两犯在案（值未提顶层变量 ReferenceError；promise/setTimeout 链在 runner 事件循环不可靠→改 corpus 同款 async_test＋callback 内注册带值 test——dump 嵌运行期注册名）

## 同簇残账（挂账续刀）

- MutationObserver-document 2F（parser script insertion／removal of parent during parsing）＝**parser 插入记录面**：shim `_mo_notify` 只在 JS mutation 发声（part01:9609 架构注记），R387 `_moReplayParseBacklog` 仅 body 直下脚本锚部分回放——期望记录形态与回放产物有差（解析积压回放精细形态校准），单独刀，本轮不展开。

## census

84,533 → 84,536（+3：1F＋2T 翻通过）。DC-3 校准线 ≥94,093，差 9,557。
