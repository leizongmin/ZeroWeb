# fb 残红 R3/R4 dl qI 不可满足互斥对定谳（2026-11-11，v4 批程内只读产物）

> 收口入库版（返修 r1，2026-11-11）：源产物与数据源（fb13-preresearch.md / batch-tq-rebind-v3.log / tqr3-fb-gate.log）在本地 `.acceptance/dps/`（不入库）；门期望修正 5042/4 与白名单 13 键恰集已由 v7 批全腿实证（batch-tq-rebind-v7.log）。

数据源：tqr3-fb-gate.log（v3 批 fb 主门完整 5046 行，修复页，现行树 d85dc7114 二进制）。
结论：**fb13 预研 F1 节「非矛盾对」断言被数据否证，dl 刀取消**；R3/R4 归录音会话自相矛盾伪影（#37 判据），除名挂账。

## 互斥对实证（tqr3-fb-gate.log 逐行）

`<dl><dt>[foo<dd>bar]</dl>` queryCommandIndeterm("formatblock") **before**：

| 行组 | setup | 录音 | 现行树状态 |
|---|---|---|---|
| C ×2 | dps=div/p, fb=`<div>`（待执行主命令） | **false** | Pass（公共祖先否决臂 → false） |
| D=R3/R4 ×2 | dps=div/p, fb=`<p>`（待执行主命令） | **true** | Fail（恒红，即挂账 dl qI ×2） |

before 断言点引擎可见状态恒等：dps 已设（两值在 C/D 两组交叉出现，非区分变量）、
树上 pristine dl、选区 `[foo<dd>bar]` 相同；唯一差异 = 待执行主命令目标（div vs p），
在 before 点未执行、引擎不可观测。→ 无任何确定性实现可同时满足四行
（false 恒定：D 翻红；true 恒定：C 翻红；随 dps 分叉：C-p 或 R3/R4 必翻红之一）。

## 对预研 F1 刀法的否证

预研断言「R3/R4 两录音一致，非矛盾对——fbVals=div 条目 qI 录音 false 所指为别的
断言行/形」。数据否证：Group C before 行与 R3/R4 before 行同形同断言。
spec 叶型比较刀（→true）修 R3/R4 的同时 Group C before ×2 必 P2F 翻红，净收益零，
P2F≠0 直接违门。**dl 刀（F1）取消。**

## 处置

1. R3/R4 → artifact（录音会话自相矛盾，X37 双行先例同构），除名逐键挂账；
   实现维持现行公共祖先否决臂（div 侧），part01g 不动。
2. R7（idx171 qV(dps) before 期望 'div' vs 链 'p'）→ 录音会话自身复位伪影
   （fb13 预研 F2 节预证，v3 批实测未翻绿坐实），随 v4 门修正一并除名挂账。
3. fb 修复页终态残红 4 = 真差异 1（R13 cE=false span wrap 能力缺口，Chrome ret=true
   且 wrap、我方 ret=true 不 wrap、语料三选一形全不匹配）+ 伪影 3（R3/R4/R7）。
4. ⑯「qV·qI 单块形读面族下一刀候选」随修复页溶解（del/quasit/xmp/listing/h5/h1
   合并形 9 键全翻绿），该候选族撤销。
5. 下一簇按残键数降序：indent 27F 普查（或 selection-compat-v2 划转 fontSize T1 ×3
   接收），R13 小刀可搭车或维持挂账。

## 门期望修正（v4 已采用）

fb 主门 5043/3 → **5042/4**；judge 白名单 10→9 键（R7 移出翻绿集）；
残红判读：dl qI ×2 + cE=false ×1 + R7 ×1（伪影 3 键不再计入真差异）。
