# M2 s441 — 红池 48 键伪红定谳清偿（green-2x2.png 资源丢失）+ fetch_raw 自愈化 + 基带重测 8113P/206F/6T/1U（零源码改动）

日期：2026-11-15（凌晨续，s440 收口后同日轮）。承前：s440 挂账三件——
relevant-mutations.html 46F 大块红勘根、zzz-s433 探针残留清理、键面记账
口径（本轮起基带 8065P/254F/7T/1U）。

## 勘根定谳：46F 大块红 = 资源缺失伪红，非代码回归

s440 双跑 sweep 的 stable-F 大头 relevant-mutations.html 46F 与 s126 入场
形态逐字吻合（40 资源缺失 + 6 代码红）。勘本地资源面：s126 前置动作生成
的 `wpt-data/images/green-2x2.png`（**gitignore 本地资源，不入库**）在本
树已丢失，s440 轮末 00:23 已重生成（先于本轮，时窗证据：sweep 日志
00:12-00:15 全红 → images/ 目录 mtime 00:23）。本轮 solo 复验：

- `FILTER=the-img-element/relevant-mutations`：**121P/0F/0T**（113 +
  8，与 s126 收口面逐键同；s293 11-05 亦 0F——丢失时窗 11-05 ~ 11-14
  22:41，丢失向量不可考，机器重启与 fetch 脚本均为增量式可排除）。
- 全域双跑 join **8113P/206F/6T/1U** vs s440 fix join
  8065P/254F/7T/1U：**恰 48 键 F→P，零 P→F、mismatch 0、其他零位移**。
  48 = relevant-mutations.html 46 + relevant-mutations-lazy.html 1
  （width attribute changes，lazy 臂依赖同资源）+ adoption.html 1
  （adoption is from appendChild，picture>source@green-2x2 换选面）——
  三页均为 green-2x2.png 消费者（语料内共 6 页消费，全在
  the-img-element），全数资源归因闭合。
- stable-T 7→6：唯一移除键 = zzz-s433-probe-N7-localName.html——s440
  轮末 00:21 已删探针文件（本轮核实全树零 zzz-s433* 残留），键面自然
  消失。**s440 挂账第二件（探针残留清理）实为已完成，本轮复核销账**。

## 根修：fetch_raw 上游通道取代本地生成（散账 s126 资源分发门）

s126 曾误判上游缺席而本地生成 PNG（72 字节 8-bit RGB）。本轮实测上游
pin（315976933）**在档**：jsdelivr 301 落地 2×2 16-bit RGB，字节与本地
版不同但断言面等值（relevant mutations 族只断言
load/error/currentSrc/naturalWidth，无像素断言）。施工一笔：

- `25-html-semantics-compat.sh` ③k 块新增 `fetch_raw
  "images/green-2x2.png"`——与 green-1x1/green-16x16/red-16x16 同通道，
  fresh checkout 与资源丢失均由 make 前置 fetch 步自愈，48 键伪红不复
  发。s126 挂账「资源入库策略待用户拍板」对**本资源**消解（无二进制入
  库——上游 pin 拉取，语料 gitignore 政策不变）。
- 本地文件换为上游字节（跨环境一致），solo 复验 121P/0F/0T 同绿。
- 自愈端到端验证：删文件 → 跑 fetch 脚本 → 恢复且与上游逐字节 cmp 同。

## 基带重测与红池重盘（206F）

新基带（双跑 join，可复现口径）：**8113P/206F/6T/1U**。红池分布：

| 块 | 键数 | 归属 |
|---|---|---|
| forms/the-select-element | 147 | **GB-20261031 簇 D 用户门**（customizable-select 主体 + filterable/popover tentative） |
| interactive-elements/the-dialog-element | 16 | abspos 居中 6 + closedby 3 + top-layer/inert/showModal 栈序——**布局/top-layer 渲染域为主**（goal 契约边界外） |
| embedded-content/the-img-element | 13 | error.html 7（破图 alt）+ svg density 3 + pixel-density 3——decoder/renderer 耦合面，挂账在案（s126/s226） |
| forms/customizable-combobox | 12 | tentative 规格页（beforefilter/:filtered/键盘指针面） |
| tabular-data/processing-model-1 | 4 | col-span 1000/1001 限值 + rowspan-0 clientHeight ×2 |
| embedded-content/the-video-element | 4 | **GB-20261114-VIDEO 用户门**（P1+P2） |
| 余 tails | 10 | input 3 / label 2 / embed 2 / details 1 / textarea 1 / audio 1 |

stable-T 6：style-error-01（s414 定谳引擎分歧）/ audio+video lazy source
deferred ×2 / sizes-auto / update-media / select-home-end-detailed
（load 族渐进）。stable-U 1 维持。

**分诊结论**：in-domain 非门非跨域块仅剩 combobox tentative 12 + tabular
4 + tails 10（≈26 键小面）；两大块（select 147 + video 4）挂用户门、
dialog 16 属渲染域。红池分诊在现状门禁下实质收窄至小尾巴逐件勘根。

## 验证汇总

| 检查 | 结果 |
|---|---|
| solo relevant-mutations（本地 8-bit PNG） | 121P/0F/0T |
| solo 同上（上游 16-bit PNG） | 121P/0F/0T |
| 全域双跑 join（资源复位 + 探针已删） | 8113P/206F/6T/1U |
| vs s440 join 逐键差 | 恰 48 F→P 零其他位移 |
| 自愈路径（删文件→fetch→cmp） | 恢复且字节恒等 |
| 全桥套件绿窗轮末收口跑 | **1596P/0F exit 0**（test 17.54s；s438 基线 1579 +17 = 组合树新钉，零失败） |
| fetch 脚本 bash -n | 绿 |

工件：/tmp/s441/{solo-rm,solo-rm-upstream,leg1,leg2,bridge}.log、
leg{1,2}.ks、fixr.{pass,fail,timeout,unsupported,1.sw,2.sw}、
upstream-green-2x2.png。

## 组合树复验（push 前 rebase 入他流 fb81665dd 后双跑位移检查）

基线双腿测于 ae91fbe94+docs 树；push 前 rebase 入他流
**fb81665dd**（ip 空块垫底 br 方向性壳携，part01f.js +123，engine 共享
面）→ 组合树（74c998e92）复跑双腿 join **8113P/206F/5T/1U + mismatch
1**：stable-Fail / stable-Pass 集合与基线**逐键恒等**（diff 空），
fb81665dd 对本语料零位移；T 6→5 + mismatch 1 = 可选 select
select-home-end-pagedown-pageup-detailed.optional.html 文件级键
Fail↔Timeout 单腿置换（s439 在册负载族渐进退化形态，簇 D 门内）。
**基带 8113P/206F/6T/1U 维持**（T 面按该族 5↔6 振荡记账）。

## 结论

s441 收口（零源码改动）：红池 48 键伪红定谳资源丢失并根修（fetch_raw
上游通道自愈化，s126 资源分发门对本资源消解）+ zzz 探针销账 + 基带重测
8113P/206F/6T/1U + 全桥套件 1596P/0F 绿窗收口。挂用户门维持：
GB-20261114-VIDEO + GB-20261031（簇 D）+ s137。下一步：小尾巴逐件勘根
（combobox tentative 12 / tabular 4 / tails 10）或候门决项（video P1+P2
/ 簇 D A2）；dialog 16 红块属渲染域，移交/协同归 rendering 流记账。
