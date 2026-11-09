# F6a hex 色值大小写归一刀（2026-11-09，⑧ lane9 首刀）

## 立项

F6bN 刀收口后 mt 行级残 5F 中 F6a 2 键（DC-4 census 键级残红族）：

- multitest :2128 `foo<font color=#0000FF>[bar]</font>baz` + `[["delete",""],["inserttext","a"]]`
  期望 `foo<font color="#0000ff">a[]</font>baz`，实得 `#0000FF` 原样直传。
- multitest :2348 `foo<font color=#0000FF>[bar</font>baz]` 同链期望
  `foo<font color="#0000ff">a[]</font>`，实得同上。
- 附加出现面：multitest-ab.html（无窗整谱页）同两键各 1 行——合计行级 4 行。

## 定谳

幸存壳层由 DOM 祖先自然承接（part01 TypingStyle 轮 `_zwCeTypingTagsAt` 语义，
:9805 注）：delete 撤选区文本后空 font 壳幸存、caret 居壳内，inserttext typed
文本落进原壳——attr 原样直传，大写 `#0000FF` 保真到序列化。Chrome 录音期望形
为小写 canonical hex。

**判别证据链**（为何只做 hex case-fold、不动语义）：

1. forecolor 主动施加路径已产出 Chrome 一致形：harness forecolor 默认值即
   `#0000FF`（tests.js :4635），全部链形键过——我们 forecolor impl 写小写形。
2. 命名色壳在语料中恒溶解、无幸存录音形：blue（≡ link 参照 #0000ff 零 delta）
   :2143 期望整体溶解 `foo<a>a[]</a>baz`、brown（≠ 参照，F2 carry 域）
   :2153 同溶解——命名色不存在「幸存壳」形态，归一无从对照，不扩语义。
3. span 背景色族（:2133 `#00FFFF`）被 harness inline-style 归一遮蔽，非本域。
4. fontsize 同族键（:2123/:2343 期望 `size="4"`）与直传恒等（数值无大小写），
   故 F6a 族唯有 hex 色值暴露分歧—— fontsize 不在本刀域。

## 刀体

`crates/engine/src/js_dom_shim/part01.js` 两处：

1. 新增 `_zwCeNormalizeHexColorShells(sc, el)`（`_zwCeTypingTagsAt` 同款链走
   模式，guard 256）：caret 祖先链上 font 元素 `color` attr 值匹配
   `/^#[0-9a-fA-F]+$/` 且含大写时 `setAttribute('color', 小写形)`。
2. `__zw_ce_inserttext_mutate` 变异核 sc/so 提取后、F2 上下文解析前调用——
   typed 落壳前归一。execCommand 分支（part06 inserttext 臂）与键盘路径共用
   本核，两入口同时覆盖。

f2.carry 消费点（:9680 font 重建）本就写 parseSimpleColor 归一小写形，零交叉；
F6b script 族（幸存原壳恒消+重放）、F2 link carry、face/size 三路径无 font
color attr（N 族 size-only）零扰动。

## 批前预验（2026-11-09 19:0x，dirty 树 head=c439e8a8b + 刀体未提交）

- 二进制身份：`_zwCeNormalizeHexColorShells` 特征串 3 处在 target/release/
  zero-wpt-runner（学习 #28 先验，计数=源码 grep 计数）。
- probe-f6bwin：期望 2091P/1F（F6bN 判读 2089P/3F 基础上 F6a 2 键翻绿，残红
  仅 F1' fontsize(4) 1）；probe-f6b 17/0 回归护栏。

## 门禁批

batch-f6a-gate-v1（锚 38269884f，八点 md5：part01 8b20e03f /
part01c fb8aa0d8 / part01d 9fbab2af / part01f 6e8eb934 / part01h 97531eb0 /
part06 247d1370〔承 s366 并入域 rebase 重锚，F6bN 批锚 2fdb996f 为批时 head
域判读绑定有效〕/ bridge f7d65f32 / fixture 8a3f16e2；特征串门 f6a=3 +
逐代累加面）。

判读域：probe 五腿（f6bwin 2091/1 残红成分门 #0000FF=0/fontsize=1/sub=0 +
f6b 17/0 + f6bn 7/0 + f7m 149/0 + f7 9/0）+ k11 224/17 judge + ilb/ip/ihtml/
exec×3/iol/iul 零漂移（typing 共享核回归面，vs F6bN 终态基线）+ mt judge
（F2P 行级恰 4=F6a 两键×双页、P2F=0、rows 20184 恒一、行级 8F→4F 残
F1' fontsize(4) 2 行 + X37 2 行）+ calibrate 翻键 4 全落 #0000FF 白名单 +
enginelib 7-red 名册双门 + battery 七腿同数 + 二进制特征串五串。

## 判读（2026-11-09 20:41 门禁批终态，ALL-DONE）

batch-f6a-gate-v1（19:08:34 发射，ABORT 于 probe-f6bn 腿——脚本引用 F6bN 诊断件
zz-probe-f6bn.html，该 fixture 随刀撤除未入库，P=0 F=0 rc=2 工具页缺失签名，
非行为漂移）→ v1.1（19:19:45，砍幻影腿+前段补判传递，ABORT 于 mt F2P 门——
judge-dps.py key 剥页前缀，F6a 两键 ab+?8001-9000 双页 4 行去重为 2 键，键级
F2P=2=机制预期集，我预写 4 为行/键翻转混计，门限口径 bug 非行为漂移）→
v1.2（20:16:24，修正 calibrate 门，**ALL-DONE 20:41:03**）。三批同发射锚
38269884f，前段腿判读跨批传递。

判读域：

- probe：f6bwin **2091P/1F**（F6a 两键翻绿；残红成分门 #0000FF=0 / fontsize
  白名单=1 / sub=0）+ f6b 17/0 + f7m 149/0 + f7 9/0 + k11 224/17 judge OK。
- 语料腿零漂移（typing 共享核回归面）：ilb 1658 / ip 6989 / ihtml 1446 /
  exec×3 / iol 1759 / iul 2061 全 VERDICT OK。
- mt：20176P/8F → **20180P/4F**（assert 级行计数）；judge 键空间 rows 10092
  恒一 F 5→3，F2P 2=机制预期（F6a 两键，双页行级 4 翻）P2F 0 postF 0 lost 0
  VERDICT OK。
- calibrate：行级 flip 恰 4 全落 #0000FF 白名单（白名单外 0）；行空间
  rows 20184→20184 F2P 4 P2F 0 postF 0 lost 0。
- enginelib：3821P/7F 双门（failed total=7 + 名册逐名恒一：6 先例 + r2967
  外部债）。
- battery 七腿 420/880/1104/552/3456/2607/1455 全 F=0 T=0。
- binary sig 5/5（`_zwCeNormalizeHexColorShells` 在产物，批前预验先验 3 处）。

**判定：DC-4 红线全过（P2F=0 ∧ postF=0 ∧ lost=0 ∧ rows 恒一），F6a 两键
收口——mt 行级残 5F→3F（assert 级 8→4 行：F1' fontsize(4) 2 行 + X37 2 行），
键级残 3F = F1' fontsize(4)（下刀）+ X37×2（除名维持）。**

批脚本教训（同批三射，两射 ABORT 均脚本缺陷）：腿引用 fixture 先核存在性
（诊断件随刀撤除不留库）；judge 元组键/行两级翻转计量分清（judge 键空间剥页
前缀去重、calibrate 行空间含页前缀）——门阈值按判读工具的真实键空间预写，
不按行直觉。
