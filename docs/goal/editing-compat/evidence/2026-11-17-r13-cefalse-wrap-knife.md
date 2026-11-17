# R13 cE=false wrap 刀收口（R13-CEFALSE-WRAP，2026-11-17）＝fb 条目 362＋indent 四目标键翻绿

刀体 **b65398b99**（`fix(editing): R13 刀——cE=false 选区 wrap 命令整面纳入`；
part01g.js 三处＋part134_r13_cefalse_wrap_pins.rs 5 形钉测，一提交常驻，
15:24 随 docs push 入 origin）＋门禁批 zw-r13 首发 **ALL-DONE**
（head=b65398b99，bin 777936555f545d98fc8267f8662ea18b，15:29:08 发射、
16:22:07 完走，wall ~53min）。F2P 恰 5 键（fb ×1＋indent ×4）零新增红。
普查归域档见 [census](2026-11-17-r13-cefalse-wrap-census.md)。

## 根因与修复面（三处，一提交）

1. **fb 可编辑链守卫收窄**（part01g `__zw_ce_fb_apply` :158-167）：守卫拦截
   条件由「链上遇非可编辑元素即拦」收窄为「非可编辑节点且**不被 active
   range 含界**才拦」——含界判定＝同 root ∧ `start ≤ node 起点` ∧
   `node 终点 ≤ end`（`_zwJusBpCompare` 双端比较，与收集臂同款）；fully
   contained 非可编辑链节点不再拦、照走 wrap 臂，部分含/界外维持拦截
   （零树变更返 true，ret [true] 录音面不变）。含界预判输入提升循环外
   （shim 循环不变量）。
2. **fb/indent node list 过滤器扩展**（`_zwJusGetContainedNodes` 谓词两处）：
   vendored spec「if it is editable」排除面收窄为 Chrome delta——
   `isEditable(node) || node.nodeType === 1`，fully-contained 非可编辑
   **元素**照纳入（第二谓词 allowed-child 照旧把关；ws 文本子维持原排除，
   三 span 之间 ` ; ` 隔断文本由 Chrome 录音形保留在原位）。indent.html
   三连形 swcss T/F 双键翻绿域。
3. **indentNodes wrap 新建臂容器形分叉**（`_zwFbqIndentNodes` 增
   `zwWebkitWrap` 形参）：swcss=true 产 webkit-indent-blockquote 形
   `<blockquote style="margin: 0 0 0 40px; border: none; padding: 0px;">`
   （indent.html 143/144/145 三行 accepted 皆收该形；p-margin/div-margin
   形未被三行全收，census 档对账）；swcss=false 恒 plain blockquote；
   outdent 复用臂（:888/:890）不传旗标维持 plain（outdent.js idx 4/5
   swcss T/F 对行 accepted 同形——Chrome 重缩进与 CSS 模式无关）。
   既有 FIXME（CSS-mode margin-div 臂未移植）保留，swcss=true margin-div
   族 ×2 非本刀域。

钉测 `part134_r13_cefalse_wrap_pins` 5 形 1 fn：fb 362 ws 内保留形＋indent
145/146/143 精确串＋守卫保留面（部分含拦截）。clippy
`-p zero-engine --lib --tests -D warnings` 干净。

## 门禁批（zw-r13，首发 ALL-DONE，systemd unit zw-r13）

| 腿 | 门 | 结果 |
|---|---|---|
| L0 runner 复建 | bin 锚＋刀体特征串 ×2 | bin=77793655… ksig margin: 0 0 0 40px=2 zwWebkitWrap=2 |
| L1 fb 主门 | 5043/3/0＋Fail 集==fbc3 减恰 1 cE 键＋零新增红 | 5043/3/0（wall 729s）翻绿恰条目 362 `?5001-last` compare 三形键，余 3 键（dl qI ×2＋dPS qV ×1）恒一 |
| L2 indent 主门 | 1308/21/0＋Fail 集==census 减恰 4＋零新增红 | 1308/21/0（wall 19s）翻绿恰 143/144/145/146（abc 含 ×2＋cE 含 ×3），余 21 键恒一 |
| L3 img 腿 | 962/0/0 | 962/0/0（i7i1 收官面恒一） |
| L4 fs 腿 | 2415/7/0＋Fail 集==fbc2 锚 | 2415/7/0 SET-IDENTICAL |
| L5 event 腿 | 176/4/0＋残红恰 Chrome 同红 4 键 | 176/4/0 四键逐在（evt-reject 真差异 3 键翻绿维持） |
| L6 ie 腿 | 1476/1852/3＋Fail/T 集双恒等 | 1476/1852/3 SET-IDENTICAL（inputevents 事件计数面零扰动） |
| L7 enginelib | 4137/0＋pin-in 3/3 | 4137 passed 0 failed（+1=part134 入册；r13_cefalse_wrap＋evt_reject＋img_breakout 三 pin 全 ok） |
| L8 mt 压轴 | 10090/2/0＋残红恰 X37（multiplier2） | 10090/2/0（wall 1798s）残红恰 subscript+formatblock 双键恒一 |

批工件 `.acceptance/dps/r13-*`（fb/indent/img/fs/event/ie/lib/mt 日志＋
resolved/newred＋launch/tail 脚本＋build.log＋ledger）；tail.sh md5
f2e4f2ad668e5512df12818d92b335ad chmod 400 冻结；发射器 exec 前 md5 断言；
基线在位核定（fbc3-fb 5042/4＋evtr-census-indent 1304/25/0＋fbc2-fs
2415/7/0＋evtr-comb-ie 1476/1852/3）批首自断言过。

## writer 429 中断与后继轮独立复核

writer session 15:42 429 限额中断（批已脱离存活完走），本轮（后继轮）按
账本协议代跑尾链复核——**全部独立复算**（非读 tail.sh 结论文本）：

- 字节绑定：各腿 bin 恒 777936555f545d98fc8267f8662ea18b（fb/mt log 尾
  echo＋盘上 release 二进制现值四同）；head=b65398b99 批域，现 HEAD
  e4fb2c42e 对 crates/＋Makefile 零 diff（他流 docs 漂移域外，判读域闭合）。
- 计数复算：八腿 Pass/Fail/Timeout 全部逐 log `grep -c '^Pass|^Fail|^Timeout'`
  复算恒等。
- Fail 集 judge 复算：fb/indent `comm -13/-23` 新增红=0/翻绿集与批内
  intermediate 文件逐字节 cmp 恒等；fs/ie Fail 集与锚 `diff` 恒等；ie
  Timeout 集恒等；event 4 键逐 `grep -qF` 在册；mt 残红恰 X37 双键；
  lib `test result: ok. 4137 passed; 0 failed`＋三 pin 行 `... ok` 逐在。

## 残红与后续

- fb 残 3＝dl qI ×2＋dPS qV ×1（非本刀域，fbc3 以来恒一）。
- indent 残 21＝census 族带（rtl/span-br/tbody/br/blockquote ×9/ol 嵌套
  ×5/ul-br/swcss margin-div ×2/cE=false 嵌套 editable），维持在册。
- 下一步＝**fb apply 侧 ~32 族＋fontSize T1 ×3 普查归域开工**（同 ilb/img
  普查工法：族分类→刀序→探针→钉→fresh 复核腿→门禁批）；exec1 474 域决策
  挂账维持。
