# s10k51 · MutationObserver-attributes 双记录回归根因定谳与修复

日期：2026-11-27。任务源：dom 热路径复验（s10k51 气球刀验证臂）唯一新 F 键
`MutationObserver-attributes`（「mutation records must match expected 1 but got 2」）。

## 结论（定谳）

**根因＝刀③b①（efb757973，TT 属性 sink 六 setter 通道）把 `Attr.prototype.textContent`
升原型 accessor（setter 即 `_r122Set`）时漏清第四处 own 直写点**：part04 setAttribute 的
R122 绑定表值同步臂（原 `part04.js:4814`）`_r122SA2.textContent = v`。刀③b① 清理了
`_zwMakeAttr`、`_r122Bind` 同步、`_r122Set` 三处，part04 这处漏网。

### 双记录机制链（确定性，非时序）

`attr.value = 'c03'`（bound Attr，绑定表已有 `_zwAttrBindings` 条目）：

1. `_r122Set`（part03）非同值臂：`_r122V='c03'`、`data='c03'` → `oe.setAttribute('class','c03')`；
2. part04 setAttribute 落地并发**记录①**（old=`'c01 c02'`）；
3. R122 绑定同步臂 `_r122SA2._r122V = v`（直写绕 setter，无害）→ **`_r122SA2.textContent = v`
   命中刀③b① 新原型 accessor → `_r122Set('c03')` 重入**；
4. 重入时 `_r122V === 'c03'` → **R336 同值臂**（「只补通知不传播」）→ `_mo_notify` 发
   **记录②**（old=`'c03'`）。

刀前 `textContent` 是 own 数据属性，直写 inert；刀后升 accessor 使该行变回环重入。
全 shim 摸排确认这是唯一漏网写点（其余命中均为 Text/Comment/PI/element 路径）。

### 判读陷阱记录（观测假象两例，防重蹈）

- **「消息扰动翻面」是伪观测**：bisect 后给语料 `checkRecords` 断言消息嵌
  `mrl.map(...)` JSON——`mrl` 是 host 记录列表非 JS 数组，`.map` 异常杀回调 → 子测试
  无 `done()` → 页 Timeout → `grep '^Fail'` 空＝看似「全绿」。三次此类观测均未查
  Timeout/完成形。教训：**失败翻面结论必须全量 census（P/F/T 三计数＋rc）**。
- **探针页不复现＝绑定态差**：最小/精确循环/全序列拷贝探针页全部 Pass——双发需
  `_zwAttrBindings` 已有该 Attr 绑定（原页前序子测试建立），fresh 页拷贝不带前置
  绑定态，:4814 同步臂 `if (_r122SA2)` 不命中。

## 定位过程（二分 10 探针）

`/tmp/s10k51/mo-probe.sh`：checkout shim 至候选 commit → 重建 runner → 单页跑计 F。
dc1→HEAD 提交序二分：6346b99817（刀③a）fail=0 ↔ efb757973（刀③b①）fail=1 相邻对，
7 探钉收敛到刀③b①。机制由代码读地定谳（上述链），修复后运行时证据闭环。

## 修复

`part04.js` R122 绑定同步臂删 `textContent = v` 直写（注释记重入根因与 s10k51 挂账）。
`_r122V`/`data` 同步保持；textContent getter 恒取 `_r122V` 派生，无需 own 镜像。

## 验证（bin `9c2cd0025399`，HEAD 30379aeab + 本刀）

- **原页**：修前 41P/1F（raw 3 连稳定复现）→ 修后 **42P/0F/0T rc=0**（+1P）。
- **MO 族 12 页全量**：attributes 42P/0F 修复，其余 11 页计数与 dc1/复验双基线恒等
  （childList 1F、document 2F＝既有面，双基线逐键核对非本刀引入）。
- **刀③b① 自身验收面**：default-policy 264P/0F、no-default-policy 272P/0F
  （刀时 264P，+8 为刀③b① 之后其他码刀面变化，pre-fix 臂 07:32Z 同值 272P 排除本刀）、
  error-cases 8P/0F、no-require 202P/0F、no-require-trusted-types-for 18P/0F——零漂移。
- **attributes.html**（textContent 读面）：73P/2F，2 F 键与 dc1 基线逐键恒等
  （+8P 为后继码刀面）。
- **TT 电池重面两臂**：ver-tt2 臂（2249P/18F/19T）测试窗与本轮前台 MO 族＋验收页批量并发重叠，
  新红恰＝墙钟敏感 navigate-to-javascript-url 001-004/008 五页（此族 pre-fix 臂已有 005/006
  Timeout 脆性）＝**争抢伪象嫌疑**；solo 双二进制复跑 nav-001（修复面 `9c2cd0025399` 与
  HEAD 面 `923ccfb3bd3d` 同败 1F）＝**单页模式内在面**（页 inline meta CSP 的
  javascript: 导航阻断在 solo 上下文不成立，与修复无关）。终判两 clean 臂（均空闲机零并发）：
  **ver-tt3（修复面 `9c2cd0025399`）＝2249P/18F/19T；ver-tt4（HEAD 面 `923ccfb3bd3d`，
  stash 修复后 make 重建、run-bin md5 断言在案）＝2249P/18F/19T，Fail 键集逐键恒等**
  ＝修复 TT 零漂移。nav5 新红＝**族级时序脆弱面非本刀回归**：同一 X 面三臂
  07:33 过（幸样本）/tt4 败/tt3 修复面败，族内 005/006 三臂恒 Timeout；
  挂账族级复跑终判（TT 席 s10k50b 同域）。
- 语料 `mutationobservers.js` 复原 md5 `5eaf5df8122b` 与备份恒等；探针页 ×4 已删。

## 工件

- /tmp/s10k51/：mo-probe.sh（二分探针）、mo-fixed.log（修后原页）、mo-patch-census.log
  （raw 第三基线）、ver-tt2.log（TT 重面臂）。
- 账本：.acceptance/dps/s10k51-t262-balloon-hunt.md（07:58Z TT 臂判读回执＋本刀回执）。
