# fbret apply 内容模型边界 ret 刀收口（2026-11-10）

## 刀体

commit **0bc8d545b**（原 ac2b12294 → 3a307c259 → 0bc8d545b 三代 rebase 重植，八点 md5 锚逐代恒一，刀体字节恒等）：

- **part06**：`__zw_ce_fb_apply` 返回值捕获（fbqRet2）贯通 execCommand 尾返回——此前 apply 返回值被丢弃、尾恒 true；宿主内容模型守卫（part01g，7ab68347f 落地）拒形（span 容器拒 p :1864/:1871、p 容器拒 div :1876）语料 ret 断言行 ×3 恒红。仅 formatBlock 臂写入，其余命令 undefined→true 行为恒一（mt 语料 formatBlock ret 36/36 全 true、tentative 页无 ret 断言，回归面核验零 P2F 候选）。
- **part01g**：可编辑链守卫返 false→true 纠偏——该形 Chrome ret 录音 [true]（语料条目 362 选区跨不可编辑 span，Chrome 真意 wrap；本引擎不具该 wrap 能力只拦不报 false）。**预验反例在案**：首版贯通误翻 entry 362 ret 行（legs5 绿→红 P2F），纠偏后 b8 29P/1F 恰 legs5 基线（残=entry 362 compare 既有 F2F 挂账族）。两处注释纠偏：链守卫原注将 :1864 族误归于链守卫——实为链在内层 editing host 截停，由宿主守卫拒。

批前预验（release 二进制直验）：zz-fbq-b7 611P/37F（family 残红 0，36=legs5 残域 + 1 分片窗界伪影 old-b7 同红）+ zz-fbq-b8 29P/1F + zz-probe-f6bwin 2092P/0F + zz-probe-knife11-v2 224/17 judge 零漂移 VERDICT OK。

## 门禁批史（三批，争抢截断两例均验尸归档）

| 批 | 树 | 结果 |
|---|---|---|
| v1 | ac2b12294 | probe 绿 + fb 主门绿（4897/149 F2P=3 flip 白名单 3）+ **mt 腿 ABORT**：P 20023 vs 期望 20182 缺 159 全为末窗墙钟截尾（multitest-ab −143 行、?9001-last −16 行，两页各另添 1 Timeout 行）——webcrypto r4 lane（ZeroWeb-2，23:49-00:49 全窗重叠）+ 23:59 rebase 并入 test262 使 mt 腿重建二进制编译 CPU 叠毒；日志 batch-fbret-gate-v1-abort1.log |
| v2 | ac2b12294 构建 | 单二进制全链重射：**mt 20182/2 Timeout=0 judge 零翻越 VERDICT OK** + iol 1759 / iul 2061 行级恒一 OK + **ip 腿 ABORT**：test-guard 900s 截尾（webcrypto r5 lane 98.7% CPU 全程重叠，插入段历史墙钟本近帽沿）P=0 |
| v3 | **0bc8d545b**（含 layout s5483 并入，二进制重建） | 现树全链重射（终判绑定推送字节纪律；ip 帽提 1800s）：**ALL-DONE 03:25:37 全绿**，02:13:07 发射，solo 窗（webcrypto DC-1 已收账） |

并发席位让位记录：v1 ensure_solo 40min 帽 01:31 ABORT（零腿运行证据零污染，让位正确）；v1/v2 双脚本 flock 仲裁、多轮 pull --rebase 刀体重植字节恒一（八点 md5 锚逐代复读）均见 [ACTIVE-BATCH 账本](../../../../../.acceptance/dps/ACTIVE-BATCH.md)。

## v3 判读（单干净二进制，全腿）

- **probe**：f6bwin 2092P/0F + k11 224P/17F judge 零漂移（rows 229 恒一）。
- **fb 主门**：4897P/149F（期望 4897/149），rows 5046 恒一；judge vs legs5 基线 **F2P=3 / P2F=0 / post-only 0 / lost 0 VERDICT OK**；行级 flip 白名单 3 全命中 contenteditable=false×ret（div 容器拒 p ×1、span 容器拒 p ×1、p 容器拒 div 内 span ×1 三形）；family 残红=0。
- **mt**：20182P/2F（期望恒一），judge 键级零翻越（F2P 0/P2F 0/post-only 0/lost 0，rows 恒一）；残 2F 维持 X37 双键除名。
- **单文件腿**（vs F1' 终态 f1r-* 基线，行级恒一零漂移）：iol 1759P/2F、iul 2061P/4F、ip 6989P/206F（rows 7195 恒一，1800s 帽内全程）、ilb 1658P/33F、ihtml 1446P/119F——五腿 judge 全 VERDICT OK。
- **exec×3**：text-editor 1332P/474F（rows 1806）、never-throw 0P/1F（rows 1）、recursive 5P/1F（rows 6）行级恒一零漂移。
- **enginelib**：3823P/7F，7-red 名册双门恒一（6 先例 + r2967 外部债）。
- **battery 七腿**：420/880/1104/552/3456/2607/1455 全同数 F=0 T=0。
- **binary sig**：五特征串全在产物 + fbqRet2 ×3 计数恒一。

## 收口结论

apply 内容模型边界挂账族 4 行全收口：ret ×3 本刀翻绿（fb F2P=3 恰值）+ family compare/qV-after ×1 前刀宿主内容模型守卫（7ab68347f）已绿——legs5 既有账口径（宿主守卫后残 ret ×3 恒红）即本刀域。entry 362 ret 行 legs5 基线本就绿，part01g 链守卫返 true 纠偏为 ret 贯通后**防 P2F**（首版贯通误翻实证），非翻绿行；族内残红仅 entry 362 compare（既有 F2F 挂账族，不属本刀域）。挂账四族余三族：fb qV/qI 读面 Chrome delta 12 行、indent 27F、img 23F。
