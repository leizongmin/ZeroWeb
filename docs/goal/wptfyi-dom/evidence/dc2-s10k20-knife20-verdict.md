# DC-2 切片 10 刀⑳收账——R522 TT parser svg 变异域＋动态 CSP 指纹＋CSP 政策闩锁（k20 系三提交合账）

**日期** 2026-11-25　**判定** PASS（TT 277→226＝净 +51 face，NEW=0/P2F=0；其余四腿 Fail/Timeout 集逐键 SET-IDENTICAL）

## 提交链（判读绑定）

| 提交 | 内容 | 判读二进制 | 电池 |
|---|---|---|---|
| k20 刀体 `7c9cb581e`（rebase 前 36fbdf7c6） | R522 TT parser svg 变异域：prepare 连接门三重假真切断（R522e pending-removed 墓碑前置＋R522b getter true 父链核＋R522 parentNode null 早退）＋R522d append 父墓碑消费点唯一化＋R522c replaceChild wrapped 文本子委派 insertBefore/removeChild＋FIX-D sel 域 parsed Text/Comment ChildNode before/after/replaceWith＋FIX-E 撤刀⑫ script 容器 `__zwFragNoExec` 硬阻断回归 k19 provenance 快照门 | `2a5dfa1eaeb440662f3ad43e41c843e4` | v2 |
| k20 补丁① `9f4486302`（rebase 前 e4f28829d） | 动态 meta CSP 指纹失效（013 P2F：enforcement 首调缓存谎报，k19 借 detached 抢置空转假绿、刀体修复后裸露）＋R522c 委派序换 remove→insert＋R87 注册判别补 adjustRemove 前置（Range-mutations-replaceChild 四面 P2F） | `5fe2e31bf658ba7565fcd3c7d0a34793` | v3 |
| k20b 补丁② `12924df1d`（rebase 前 78c89992b） | TT CSP 政策闩锁（014/015 P2F）：meta 交付政策入文档清单后移除元素不回收（CSP3 #meta-element），k20 指纹重扫只覆盖动态注入方向、移除方向误关 enforcement；k19 首调缓存系 lib 装载期 createPolicy 经 `__zwTtCspState` 扫描恰质 poll 为 ON 的偶然绿。置位点入 `_zwTtParseCsp`（任一扫描路径单调置位），`_zwTtEnforced()` 返回闩锁值，指纹重扫只为捕捉 OFF→ON 动态注入 | `5cbce26fbd875c7a9b4c8e461ffb6d73` | v4 |

rebase 谱系：两次 pull --rebase 改写 36fbdf7c6→733faa06e→7c9cb581e／e4f28829d→a7767fc94→9f4486302／78c89992b→12924df1d；判读绑定字节不变（part14 blob `b8197ea0`、part166 blob `eaf105e2` 跨 rebase 恒等）。

## 三轮判读链（锚＝k19v2 终态 TT F 277 / WC F 1116 / DP F 222 / SAN F 50 / DOM F 364）

| 电池 | 时窗(UTC) | TT | DOM | WC/DP/SAN | 结论 |
|---|---|---|---|---|---|
| v2（刀体） | 07:41–08:12 | 277→256（GONE 22/NEW 1＝013 P2F） | 364→368（NEW 4＝Range-mutations-replaceChild P2F） | SET-IDENTICAL | 刀体两 P2F 定谳→补丁① |
| v3（补丁①） | 08:28–09:03 | 277→242（GONE 37/NEW 2＝014/015 P2F） | 364=364（replaceChild 归零） | SET-IDENTICAL | 013/replaceChild 兑现，014/015 新 P2F→补丁② |
| v4（补丁②，终判读） | 09:36–10:11 | **277→226（GONE 51/NEW 0）** | 364=364 | F/T 集逐键恒等 | **k20 系合账净 +51** |

v4 明细：TT Pass 1805→1856（+51）、Timeout 65 键集恒等；GONE 51 分页＝HTMLElement-generic 28＋script-enforcement-004 16＋003 4＋002 2＋012 1（014/015 各 1 面亦清）；NEW=0。WC/DP/SAN/DOM Fail 集与 Timeout 键集逐键恒等；Pass 漂移（WC −1632／DOM −629）＝已知 gethtml 页预算截断与 Range-mutations-deleteData 停摆噪声族（deleteData 页 Timeout +1，同 dataChange/replaceData 停摆家族，F 集恒等非回归）。

## 机制要点

1. **R522 连接门假真切断**：detached 变异窗口（splitText/normalize/removeChild/nodeValue 等中点）prepare 时 pending 变异未 drain、`__zw_contains` 裸 dom_html 反链 mutation-blind 假真—— parentNode null 早退＋pending-removed 墓碑前置＋getter true 父链核三重切断，抢置 started 章消失，append 时点 prepare 完整走 k19 trust 快照门。append 父墓碑消费点唯一化（R522d）消除 detached 子树内 append 复活父墓碑导致的变异中点提前派发。
2. **动态 CSP 指纹**（补丁①）：`_zwTtParseCsp` 每次 gate 调用按「全部 meta http-equiv/content 拼接指纹」失效重扫——动态注入 meta 即时生效（012/013），静态页指纹不变走缓存零行为变化。
3. **CSP 政策闩锁**（补丁②）：`enforced` 位单调闩锁——任一扫描路径（sink 名单门/执行门）见到 require-trusted-types-for 即永久 ON；移除 meta 元素不回收已交付政策（CSP3 meta-element 语义）。近似边界：meta 在任何扫描前已被移除的页无法追溯收集（shim 装配早于 DOM 组装，无解析期扫描钩子），语料无此形态。
4. **HTMLElement-generic −28**：FIX-E（撤 `__zwFragNoExec` 硬阻断）兑现 14 面（innerHTML/innerText/textContent/src/script.text/script.srcdoc sink 救援可达）＋闩锁兑现 14 面（页中移除 meta 后 enforcement 保持）。

## 常驻钉测

- part165（刀体）30 face svg 变异域全绿＋动态 CSP 三向（C0/C1/C2）。
- part166（补丁②）4 face：PRIME latch 直读（createPolicy 质 poll 后移除 meta `__zwTtEnforced()` 恒 true）＋L2 014 HTML 形阻断＋L3 015 SVG 形阻断＋L4 保信方向（trusted 源 fused 恒等照常执行）＋N1 无 meta 负面页（独立沙箱）。门禁批内 4/4 全绿（`cargo test -p zero-engine --lib s10k20`＝part165 2＋part166 2，test-guard --compile-first 包裹）。

## 质量门禁

fmt 零 diff；clippy `-p zero-engine --lib --tests -D warnings` 0 warning（k20b）；k20b 提交 lei-pre-commit-guard PASS；v4 电池发射器 `.acceptance/dps/s10k20b-battery-v4.sh`（EXPECT_MD5+EXPECT_HEAD 断言、flock 单例、五腿 test-guard 包裹）。

## 残面挂账（刀㉑ 输入）

TT 残 226F：secondary-document 30／from-literal 21／HTMLElement-generic 残 20／block-string-assignment Document-write 10／default-policy-report-only 9／createHTMLDocument 8／script-enforcement-008/009 各 5 等。刀体提交注记挂账的 002 insertAdjacentText／003 normalize／004 cloneNode/cloneContents 域已被 v2–v4 判读实测吸收（002/003/004 仅余 svg 域 4/4/16 面＝k19 已知残域，非新账）。

## 累计

DC-2 累计 83,670→**83,721**（k20 系净 +51，五腿电池口径）；距 ≥99,947 差 16,226。
