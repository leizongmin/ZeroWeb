# ihe2：ihtml D 族占位 br 消费刀门禁批判读（2026-11-16）

## 批终态

- **ALL-DONE 2026-11-16T14:19:47+0800** @head `36f3731b0`（刀体提交）@bin
  `defc09510f7692f41a5dcf0270af780b`（四腿字节恒一）。脚本
  `.acceptance/dps/ihe2-tail.sh`（md5 `acbebf02bd3f78ffdc90bf1603a8ef70`，
  chmod 400 冻结），发射 13:36:42（unit zw-ihe2，pid 969555），锚定 v2
  （LAUNCH_HEAD 冻结＋每腿 bin md5＋engine/Makefile 域漂移熔断）。
- writer session 全程存活，无代跑。mt 腿因并行 wptfyi-css 流 A/B 批（预编译
  bin）争抢，load_wait 多等 300s（13:45:08 才起跑）＋wall 2054s 慢化——计数
  门（10090/2/0＋X37 双键＋零新增红）全绿，争抢伪影排除。

## 四腿结果

| 腿 | 结果 | 判读 |
|---|---|---|
| L1 iht A | 1565 行 1547P/18F（wall 11s） | F2P=13 逐键消失＋零新增红 OK |
| L2 iht B | 1565 行 1547P/18F（wall 42s） | 同上 OK；A/B 红集 comm 恒等 |
| L3 mt | 10090P/2F/0T（wall 2054s） | 残红恰 subscript+formatblock X37 双键＋零暴露 OK |
| L4 lib strict | 4068P/0F（16.61s） | 13 钉入既有测试函数 +0，含渲染流 transition 刀 +8 OK |

原始日志独立复核（不依赖脚本回显）：`comm -3` A/B 红集恒等；`comm -13`
对 ihe1 基线零新增红；`comm -23` F2P 恰 13 键（D 族 12＋空根 1）；mt 残红
`grep` 恰 X37 双键。

## census 谱系

41F（ihc1 普查）→ 31F（ihe1 G 族刀）→ **18F（ihe2 D 族刀）**。

残余 18 族谱（ihe2-iht-a.log 红集逐键清点）：

| 族 | 键数 | 形 |
|---|---|---|
| A（断载荷/未闭合壳吞并，host 直子） | 5 | `foo[bar]baz` × ""/<b>/<b>abc（swcss 双态） |
| rawtext 壳内分选区 × 可见载荷 | 4 | xmp/script × `<b>abc</b>`（swcss 双态） |
| **C（块壳 p × 全段选区 × 隐形载荷）** | **4** | `<p>[foo]</p>` × space/span display:none×2swcss/comment——下一刀目标 |
| CE=false pre 链 | 1 | `<pre contenteditable=false><span contenteditable>[1234]</span></pre>` + `<pre>abc</pre>` |
| E（空行内壳丢失） | 2 | `<div>a[]b</div>` × `<span></span>` / `<span s1>b<span s2>` |
| F（CE=false 邻接） | 2 | `<i>Z</i>` × CE=false span 前/后 |

## 刀体语义（实现要点）

- 溶解臂全含门把零文本元素 span [0,0] 判为坍缩选区 [0,0] 全含 → 整壳替换
  载荷（实录 got=`abc`/`<!--abc-->`/`""`）。D 臂主门：零文本块壳 p × 元素
  端点坍缩（sc===壳）× 占位 br（裸 br 或无属性 span 壳包 br+注释）× 素文
  本/单注释载荷，分域六规则（文本载荷 caret/容器位序定删壳或留壳；注释载
  荷裸 br 落 caret 位、span 壳按 caret 位序解包或落壳内 br 前）；空根臂
  （sc===host 无子 → 载荷 verbatim）。序列化逐子镜像拼串＋round-trip 门。
- part124 +13 钉（ihb-d-* 12＋ihb-d-empty-root），红形与 corpus 实录逐字节
  同形。
- 他流交叉验证：html-semantics s516 收口轮（ce66c579d）将本刀 36f3731b0
  归因复跑其语料——零语义位移（谱系第十九例），runner 换代 +8,280。

## 下一步

C 族占位块壳保留刀（4 键）：根因已定谳（同一溶解臂全含门，非坍缩形——
p 壳 span [0,3] 被全段选区 [0,3] 判全含 → 整壳换载荷，got=` `/`<!--abc-->`/
`<span style="display:none"></span>`）；刀界已设计并落 staging 草稿
（`.acceptance/dps/ihe3-c-knife.part06-insert.js.txt`＋
`ihe3-c-knife.part124-insert.rs.txt`＋门禁脚本草稿
`ihe3-c-knife.gate.sh.draft.txt`）：块壳保留＋载荷落壳内＋空白/注释载荷垫
占位 br（结果壳内无元素子＝不可见空块）＋单空元素载荷不垫（corpus
display:none span 双 swcss 态实录均无 br）。预期 ihe3：1551P/14F F2P=4＋
mt 10090/2＋lib 4068/0。
