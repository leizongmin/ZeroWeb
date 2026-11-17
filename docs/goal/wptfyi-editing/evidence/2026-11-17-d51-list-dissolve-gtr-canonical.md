# d51 列表壳溶解收口 + join 界规范化刀 A/B 证据（2026-11-17）

## 刀面

part01.js 三处（BS 溶解臂承接 d51 前半、FD 镜像收口 + 两处 join 界规范化）：

1. **FD 溶解闭包 takeLineFrom 重写**（FD 镜像 13476 区）：直挂
   `cc.insertBefore(_lfTx, cnx)` 跨容器移动 shim 内抛 HierarchyRequest（BS 臂
   T7/T8 探针实证 li 连行带走的根因）——改为 `_zwCeTakeLineFrom(_lfLi, _lfTx)`
   提取游离节点后逆序挂回 `cc`（BS 臂同型：尾值先写、提取、逆序 insertBefore
   循环、`_lfMoved` 锚、`_lfLi.parentNode === cnx` 守卫摘除）。
2. **B2 `_gtrEndF1`（非 collapsed 跨容器 join）界终文本锚前向规范化**：
   JoinEndOf 裸 `(leaf,0)` 对后邻前导折叠空白欠账（`<li>  def[]  </li>` +
   `  ghi` 形期望 `( ghi,1)`）；元素锚不变，preserve/preLine 域 canonical
   自守恒 no-op。
3. **B2 `_gtrEndF`（collapsed 块界 join）同规范化**：`abc[]<p> def</p>` 形
   后邻前导折叠空白不入界。

## 权威账（专项单文件 A/B，base=45051c160 前沿 d50 收口）

| 腿 | d50 base | d51 wip | 共键 F2P | P2F |
|---|---|---|---|---|
| backspace 主文件 | 78P/85F (163) | 79P/84F (163) | 1 | 0 |
| forwarddelete 主文件 | 96P/80F (176) | 98P/80F (176) | 2 | 0 |
| non-collapsed 三 variant（对 d50 wip） | 98P (207) | 101P (207) | 3 | 0 |

**净 +6（F2P 6 / P2F 0）**。F2P 明细：BS `abc  <ul><li> []def </li></ul>  ghi`
innerHTML（BS takeLineFrom 修复兑现）；FD `<div>abc <ul><li>  def[]  </li></ul>
 ghi</div>` 双形 promise 主键 ×2（_gtrEndF 规范化兑现）；non-collapsed ?Delete
`abc[<ul><li>]def` 列表选区面 content 子测 ×3（FD 溶解臂在保守 join bail 前截获：
li 首行溶入外层流，`<div>abcdefghi</div>` / 二 li 形 `<li>ghi</li>` 保留全对，
第二把刀 B2 镜像不做 BS 侧——BS 面 gtr 起点本走 canonical backward 无欠账）。

## 探针定谳（zz-probe-d51a）

- **T8（BS 列表界 repeat 位）= 语料契约全对**：bi=1/input=1/content
  `<div>abcdefghi</div>`/gtr `(abc,3)-(def,0)`——takeLineFrom 提取路径
  在合流派发下完全兑现。
- **T7（同形 first 位）**：bi=1/gtr 对但 content `abc<ul><li></li></ul>`、
  def 摘后挂回失败——首派发 caret_range 世界同步回灌+回植鬼域（d39 谱系，
  入口刀两拒不重攻），③ 停场。
- **T9（FD 列表界）**：entry `(t"",3)` 鬼锚、bi=0——同 ③ 域，FD 溶解臂
  首位不可达。
- T1 hr caret bi=0（d50 已录 ④ 臂 hr 形派发鬼，持位）。

## 语料语境残余分型（列表面，knife 后仍在册）

1. **plain 形 bi=0 鬼**（`abc<ul><li>[]def</li></ul>ghi` 主键）：语料位置
   派发鬼，DOM 不动——③ 域证据+1（探针 T8 同形同码路径全对，差异纯在
   语料派发站代际）。
2. **bi=2 双派簇**（ws 形 promise 键 `got 2`、`[]ghi` caret 后邻面）：
   d40-II [SameObject] 身份双派族（旧册 342F）——溶解臂使新面**进入**该停场
   簇（got 0 无变异 → got 2 变异正确），无新增红键（P2F=0），③ 域证据+1。
3. **FD ws 形块界 join 内容塌方**（`abc <ul><li>  def[]  </li></ul>  ghi`
   content got `<div>ghi</div>`）：**base 继承**（d50 base 同消息逐字相同，
   非 knife 产物）——JoinApply ws 形变异把 abc/li/def 整体吞没，promise 键
   已因界规范化翻 P、内容子测 2 键残余。共享 JoinApply 路径 96P 高危，
   记停车场不本轮攻。
4. caret-after-ul 面（`<ul>...</ul>[]ghi`）bi=2 + content `<div>ghi</div>`：
   base 同红（P2F=0），2/3 复合形。

## 哨兵

- enginelib：**4131P/0F**（test-guard --compile-first --per-proc-mem 4
  --total-mem 8 --time-limit 1500）
- selection CE 过滤（23 案）：对 /tmp/d49-selce-base.json 共键 **414 全同
  F2P=0/P2F=0**（本腿经 systemd service 内重定向跑通；首次 scope 前台跑
  撞 Bash 10 分钟帽、journal 重定向截断 4486/53k 键——service 内
  `sh -c exec >file` 为正解，方法论入册）
- input-events 全套件（86 案）：对 /tmp/d49-suite-base.json 共键 5442
  F2P=269/**P2F=0**；对 /tmp/d50-suite-wip.json 逐轮净差 **F2P=12/P2F=0**
  （knife 核心 +6＝主文件 +3 腿合计，另 6＝zz-probe-b3p/b3q/b3r/d46b 镜像
  面同形自增）

## 运行产物

- /tmp/d51-bs-wip.json（BS 主文件，本轮前半已测）
- /tmp/d51-fd-wip2.json（FD 主文件）、/tmp/d51-nc-wip2.json（non-collapsed）
- /tmp/d51-selce-wip.json（selection CE 过滤）、/tmp/d51-suite-wip.json
  （input-events 全套件）
- zz-probe-d51a.html（wpt-data 本地不入树）
