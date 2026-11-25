# DC-2 切片 10 刀㉔收账——Document.write 写流 body 空清重置＋融合视图同步（TT 152→133，同簇 19F 全清）

**日期** 2026-11-26　**判定** PASS（TT 152→133＝净 +19 face，GONE 19 全 write 簇/NEW 0；WC/DP/SAN/DOM Fail 集逐键 SET-IDENTICAL，Timeout 键集 TT 65/WC 20 恒）

## 提交与判读绑定

| 提交 | 内容 | 判读二进制 | 电池 |
|---|---|---|---|
| 刀㉔ `1b8e2fb99`〔谱系 18f8721a6＝发射锚→push 前 pull --rebase 携入 css 流 k128 重写＝推送终态；patch-id 恒等验证〕（part04 四处：B1b getter 让位＋_tcSame 空写子树异值判定＋替换全部子消化 pending 桶＋空值置空基底缓存＋body 空清写流重置；part173 钉测新文件＋bridge_tests 注册行） | 写流清空重置主刀 | — | v8（暴露 3 NEW） |
| v8.1 `ab0a2b41e`（B1b 让位收窄写流上下文：`__zwDocWriteMaterialized` 粘性旗标 part06 write 物化置位＋part04 让位前置门） | 旗标收窄 | `af01e401912e534760b417fb8e3dc24b` | **v9（终判读）** |

发射器 `.acceptance/dps/s10k24-battery-v9.sh`（EXPECT_MD5+EXPECT_HEAD 双断言、flock 单例、五腿 test-guard 包裹、setsid detached）；墙钟 WC 7m17s/DP 13s/SAN 17s/TT 16m42s/DOM 11m13s（15:46–16:22Z，36min）。v8 电池（`1b8e2fb99`@`de744d761ff8f7990c4b69e7d50980f4`〔发射锚 18f8721a6→rebase 重写〕，14:59–15:36Z）GONE 19/NEW 3——NEW 3 全集 setHTMLUnsafe 页＝B1b 让位泛化回归，v8.1 收窄消除。

## 根因链（归因→刀→收窄）

1. **缺位面**：block-string-assignment-to-Document-write 10F 全形态 `expected "..." but got ""`/累积翻倍。Document.write 写流是「Base 重放＋累计 buffer 全量重插」模型（R5001 隐式 open 流），body 空清只移除已物化子，旧 Base/buffer 留存使**下次 write 全量重放复活已清内容**（语料 21 面每 face `innerText=''` 开场＋write 期待独立内容；探针定谳 F2 序列 post-clear kids=0→write kids=3 it=[F1F2]）。同页 `document.write with html assigned via policy` 面另断 innerText getter：写流物化（IAH host mutation）后 latest-wins 短路返 stale 写入值（got ""）。
2. **刀（part04 四处）**：body 空清时同步归零写流记账（buffer/Base/停车链/嵌套链/defer/visLen——镜像 open() 重置面，Base 置空串防 _dwbFirst 回落重新快照）；textContent 空写 sel-only 形补子树异值判定（`_tcCur` 元素级文本值不含子树文本，`''===''` 误判同值跳过整个替换域——修复块曾落死代码域三轮无效的排查教训）；替换全部子消化 pending 桶（added 清空＋T84 增量缓存作废＋桶删除）＋空值置空 `_zwChildBaseCache` 条目（R81 空值不注册 R5273 种子的 stale 复活面）；innerText getter B1b：diverged 且 pending 门激活（`_zwFusedTextSel` 非空）时跳过短路落收集器读融合子视图。
3. **收窄（v8.1）**：v8 电池 NEW 3 全集 setHTMLUnsafe 页＝B1b 让位泛化把**无 write 页**（appendChild＋innerText 读形）带进融合收集器，暴露陈旧 base＋块界 `\n` 伪影。`__zwDocWriteMaterialized` 粘性旗标（part06 write 物化置位）作让位前置门——旗标关恢复原 B1 diverged 恒短路（v7 恒等价），开保持 pending 让位。debug 复核：setHTMLUnsafe 2F＝v7 基线键名恒等（throws/null 两面基线红非本刀域）；钉面页 0F 保持。
4. **保信**：GONE 19 页分布＝Document-write.html 7＋block-string-assignment-to-Document-write 10＋zz-knife3b2-probe×2（刀③b② 时代遗留探针页，同簇连带转绿）；TT Pass 1930→1949（+19 恰＝修复面）；Timeout 65 键集恒。

## v9 判读（锚＝k23 v7 终态 TT F 152 / WC F 1116 / DP F 222 / SAN F 50 / DOM F 364）

| 腿 | Fail | GONE/NEW | Timeout 键集 |
|---|---|---|---|
| TT | **152→133（Pass 1930→1949，+19）** | **GONE 19 全 write 簇 / NEW 0** | 65=65 恒 |
| WC | 1116→1116 | 0/0 | 20=20 恒（v8 的 21＝争抢抖动回稳） |
| DP | 222→222 | 0/0 | 1=1 恒 |
| SAN | 50→50 | 0/0 | 1=1 恒 |
| DOM | 364→364 | 0/0 | 12→13（`Range-mutations-deleteData` 已知截断抖动——solo 复跑 564P 全 Pass 实证非刀因；WC `property-reflection-imperative-setup` solo 2910P 同证） |

## 常驻钉测

part173 两面域：P1 写流物化基形／P2 body 空清生效／P3 清空后写独立内容（无复活核心钉面）／P4 多参拼接／P5 非 body 清空不重置流（物化产物 `<b>` 子清重放复活对照——触发面收窄到 body 本体）。P0 排障记录：钉测 host 占位块元素会在 innerText 产块界换行（Chrome 同型），body 置空。TT 管道 enforcement 面由语料页 block-string-assignment-to-Document-write 电池 TT 腿常驻覆盖；门禁批内钉测 1/1 绿＋s10k2x 全家 12/12。

## 质量门禁

fmt 零 diff；clippy `-p zero-engine --lib --tests -D warnings` 0 warning（双提交各一次）；提交 lei-pre-commit-guard PASS（刀 3 文件 157 行 0 发现／v8.1 两文件 +13/-3 0 发现）；release 重建（test-guard 包裹 32s）。

## 残面挂账（次刀输入）

TT 残 133F：default-policy-report-only 9F／default-policy 页 3（script.src/div.innerHTML/script.text null）／eval-csp-tt-no-default-policy 4／createHTMLDocument 残 5／script-enforcement-008/009 各 5F／Document-write-appending-line-feed 6F（v7/v8/v9 三锚恒等基线红——write \n 追加语义面）等；setHTMLUnsafe 页基线红 2F（throws/null 面空读 `\n` 伪影——空 div 块界，非本刀域）。次刀亦可转 WC adopted-callback 66F。

## 累计

DC-2 累计 83,795→**83,814**（刀㉔净 +19，五腿电池口径）；距 ≥99,947 差 16,133。
