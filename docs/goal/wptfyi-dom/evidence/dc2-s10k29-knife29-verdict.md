# DC-2 切片 10 刀㉙ verdict——TT eval 门全链（SHIMW universal eval 门＋strict 车道装置＋执行体免门遮蔽收口）

**2026-11-26 · 记录腿 HEAD `a842bf953`（返修三f `73f3af3c5`＋返修三g `a842bf953`，rebase 于他流 k135 `5a7597859` 之上）**

## 结论

- **TT 语料净 +25**：P 2116→2141，F 143→118，T 20 恒（键集 SET-IDENTICAL，TGONE=0/TNEW=0），P drop=0，语料 NEW=0。
- **GONE 25 行全 eval 族 11 页**：Window-block-eval-function-constructor 2／eval-csp-tt-default-policy(-mutate) 2+2／eval-csp-tt-no-default-policy 4／eval-function-constructor-untrusted 1／eval-with-permissive-csp 4／eval-reporting-no-unsafe-eval 3／eval-reporting-report-only 1（余 1F＝RO String 载体 deferred）／reporting-for-Window-eval 1／reporting-for-Window-function-constructor 4／tt-block-eval 1。
- **四邻电池**（web-components/domparsing/sanitizer-api/dom vs k28c v1 基线）：**DP/SAN/DOM SET-IDENTICAL**（FT 223/51/376 键集零差）；WC 唯一 GONE＝reference-target form.html 零输出页（无 TT meta、11-22 语料、非本刀域，两轮电池同形＝既有现象，挂账 css-shadow 流对账）。
- **累计 83,998（距 ≥99,947 差 15,949）**。

## 提交链

| 提交 | 内容 |
|---|---|
| `be7080635` | 刀㉙首笔：SHIMW universal eval 门（part14 §5.5 globalThis.eval 三 face 包装）＋Function 编译门＋part01 `__zwNativeEval/__zwDirectEval` 快照＋内部求值面旁路＋part177 四钉 |
| `61ecaa892` | 返修三/四：装置点搬初始 script 交付通道（script_gen.rs classic wrapper 前缀）＋TT face fresh parse |
| `73f3af3c5` | 返修三f（k29 配置 F 终形）：装置点落 strict 直编译车道体前（runner 语料全走此道，classic wrapper 前缀永不触发＝残页装置缺失根因）＋part06 写面三处 `(0,eval)` 遮蔽（internal-slot 双页 Timeout 根因） |
| `a842bf953` | 返修三g：脚本执行体免门遮蔽收口 8 处（part15 位置式内联体实证＋part03/04/05 巡检 7 处）＋webview.rs fmt 两行 |

## 配置矩阵（solo 五页判读，/tmp/s10k29/）

| 配置 | 装置点 | 残 eval 族页 | SVG internal-slot | 定谳 |
|---|---|---|---|---|
| A（首笔＋else 臂 reinstall） | 体前 reinstall | ✓ | ✗ Timeout | reinstall 毒化写面动态流 |
| B（61e uninstall-only，交付通道装置） | 初始交付通道 | ✗ | ✓ | runner 语料不走交付通道→残页门缺 |
| C（页首检查点） | doc 级页首 | ✗ | ✓ | 首脚本 else 臂卸门杀页首装置 |
| D（strict 车道体前＋卸-only） | strict 车道 | ✓ | ✗ | 残页✓但写面 `(0,eval)` 过门拦体 |
| **F（D＋part06 写面遮蔽）＝返修三f** | strict 车道 | ✓ | ✓（Fail1 基线态） | **全绿终形** |

## dp 位置式 20F 插曲与 bisect 假阳性教训

- 电池 dp 腿 NEW 20F（positional-methods-with-parser-options，k28c 同页 160/160 全 Pass）：part15 `_p15EvalScripts` `(0, eval)` 过门吞 EvalError（体为 String→TT face→default policy 无 createScript→`catch (_eEv) {}` 静默；测试自移除 script 后延后执行取消）——与 internal-slot 写面 Timeout 同漏网类。
- commit 级 bisect 假阳性：61e 20F／be7 160P／k133 纯 solo 160P 非单调；早前「k133＋be7 part01 回退全绿」被误读为 part01 元凶——实为该组合无装置点＝无门（配置态混淆）。页面内插桩（ZZ 面包屑 test）两配置同形 F=0 佐证内容无关。**教训：bisect 遇非单调即停步验「判别轴是源码还是构建配置态」，文件级回退须与纯 solo 交叉验证。**
- 返修三g 收口 8 处执行体（part15 位置式＋part04 R387 appendChild/s95 补文本/s97 insertBefore＋part05 s377 handle/s4p 增节点＋part03 s97 写入槽/module 缓存体）统一 `(typeof globalThis.__zwNativeEval === 'function' ? globalThis.__zwNativeEval : eval)` 遮蔽（part03 s109 臂/part06 返修三f 同形）；页面显式 eval 面仍走 SHIMW 门管辖域。

## 验证面

- 位置式页 solo 160/160 Pass（修复前 20F）；TT eval 族页（default-policy 24P／permissive 16P）全绿；SVG internal-slot Fail1（基线精确形态，「first arrived」已知行为）；HTML internal-slot manipulation 子测 Pass（较 full-A 改善）＋页级 T（基线恒形态，X2 中性化对照实证非本刀回归）。
- 质量门：fmt 0 diff（webview.rs 6101 两行补 fmt）／clippy engine 0／pins s10k29 6/6／lei-pre-commit-guard PASS（5 文件 26 行 0 发现 0 提醒）。
- 判读：judge-k29.sh vs k29pre（P2116/F143/T20 基线）line-level 多重集 comm＋judge-battery-k29.sh vs k28c v1 base-sorted keys（FT 页键）。
- 探针工作流：zz-* gitignored 探针页五轮面包屑归因（门状态 pre/post＋eval native-ok 直测），定谳后已删（树净，他刀 zz 不动）。

## Deferred 挂账

- report-only 余 1F（RO face String 载体）。
- runner 多页进程跨页状态泄漏（`__zwEvalGateInstalled` 不随页复位 FIXME，script_gen.rs 在册）。
- enforced 页直接 eval 间接化边界／纯内联事件处理器装置点／default-src 回退。
- HTML internal-slot 页 8 测 solo 悬挂（声明 sink TS 臂；X2 中性化对照实证非本刀回归，既有现象）。
- wc reference-target form.html 零输出页（无 TT meta，非本刀域）——css-shadow 流对账挂账。

## 证据文件

- 判读：`/tmp/s10k29/k29g-judge.txt`（TT 全腿）／`/tmp/s10k29/battery-k29-judge.txt`（四邻电池）；日志 `tt-full-k29g.log`／`battery-k29-{wc,dp,san,dom}.log`。
- 发射器：`/tmp/s10k29/launch-k29g.sh`（HEAD＋净树＋12 文件 md5 指纹断言）／`battery-k29.sh`（内层单锁串行）。
