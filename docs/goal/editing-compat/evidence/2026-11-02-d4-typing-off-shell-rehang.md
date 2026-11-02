# D4 typing 拆壳重挂轮（F5 typing 尾 br 边界）

日期：2026-11-02。基线 g19（F6 轮终批）multitest 9,590P/502F。

## 语义定案（语料源推导）

multitest.js :2452-2483 六 fixture：`styleWithCSS-false + bold + insertText d`，
caret 于行尾文本内（`abc[]`），行带 `<br>` 占位，形族六变体。Chrome typing-style
重挂语义三则：

1. **拆最深 off 壳**——显式 toggle-off 的 tag 仍在 caret 祖先链上时，typed 段
   不继承该壳。in-place splice（`<b>abcd<br></b>`）qS-after 反（Chrome 期望
   `<b>abc</b>d`，typed 段裸层）。
2. **typed 段按壳内非 off 链重包裹**——off 壳到 caret 之间的非 off 壳链保留于
   typed 段（F4 `<b><i>abc[]</i></b>` + bold-off → `<i>d</i>`，i 非 off）。
3. **纯占位尾段丢弃**——caret 之后的尾段若无非空文本（纯 br 占位），拆壳时
   丢弃（F1/F2/F3/F5 期望无 br）；off 壳直后的裸 br 占位同消费（F3/F5 形）。

## 实现（三文件）

- **part01.js** `__zw_ce_inserttext_mutate` 文本节点分支新 offSplit 臂（D4 注记
  块）：`__zwTypingTagsOff` 非空且 range collapsed 时自 sc 上行找最深 off 壳；
  命中则 ① 尾段手工爬升摘取（自 sc 逐级摘后兄弟至 offSplit，每级以容器空壳
  clone 承接保持嵌套上下文形，空级不建壳，文档序深层在前）；② typed 段按非
  off 链 `_zwCeTypingInsert` 重包裹后挂 offSplit.nextSibling；③ 尾段无非空
  文本则丢弃并消费 offSplit 直后裸 br，有则原样回挂。FIXME：嵌套尾段重挂形
  （F6 期望 `<i>d<br></i><b><br></b>` 全形——separator br 与 typed 段同壳合并
  + 末占位 br 重挂 off 壳），当前嵌套尾段按占位丢弃。
- **part06.js** collapsed format toggle 臂 `__zwTypingTagsOff` 记账（state-on
  → push off 表并移出正表；state-off → 移出 off 表并回正表，sub/sup 互斥同
  步）；`_zwSync` 同生命周期清除（`__zwTypingTagsOff = null`）。
- **part01f.js** `__zw_ce_delete_apply` 快照扩到三表（snap + tt + ttoff），动作
  后 reanchor + putRaw + 双表回写（同 F6 Fix C 语义）。

## 过程根因两笔（探针定位）

1. **shim DocumentFragment.appendChild 无移动语义**——zz-probe-d4x X4 实证：
   `frag.appendChild(br)` 后 `br.parentNode` 指 fragment 但源父 `b.innerHTML`
   仍含 br（host 桥 append 不摘源父 + JS 侧登记双写）。同根解释 X1/X2
   `Range.extractContents` 尾兄弟不迁移（fragment 收集同 primitive）。首轮
   D4 爬升依赖隐式 detach 全部失效（尾段原地滞留）。修复：爬升内显式
   `removeChild` 后再入 fragment（part01.js D4 注记）。**fragment-append 移动
   语义与 extractContents 共享根修复挂账**（共享路径修复面，涉 host 桥
   append_child_handle 摘源 + JS 登记簿一致性，影响面广独立轮）。
2. **尾段 null 时 `_zwD4Scan(null)` 抛 TypeError**——F3 形（off 壳即 sc 直接
   父、sc 无后兄弟，尾段 null）臂中断于 scan，dEl 已插入而丢弃臂未达，
   dispatcher 吞异常回落致 `d<br>` 滞留。修复：`if (tailSeq)` 门。探针
   zz-probe-d4f3.html S0-S3 分步定谳（S2 证明 toggle 面干净：DOM 不变、
   sel txt@3、toff=["b"]）。

## 探针终态（zz-probe-d4br.html，七案）

| 形 | GOT | 语料期望 | 判 |
|---|---|---|---|
| F1 `<b>abc<br></b>` | `<b>abc</b>d` | 同左（双期望其一） | ✓ |
| F2 `<i><b>abc<br></b></i>` | `<i><b>abc</b>d</i>` | 同左 | ✓ |
| F3 `<i><b>abc</b><br></i>` | `<i><b>abc</b>d</i>` | 同左 | ✓ |
| F4 `<b><i>abc<br></i></b>` | `<b><i>abc</i></b><i>d</i>` | 同左 | ✓ |
| F5 `<b><i>abc</i><br></b>` | `<b><i>abc</i></b><i>d</i>` | 同左 | ✓ |
| F6 `<b><i>abc<br></i><br></b>` | `<b><i>abc</i></b><i>d</i>` | 嵌套全形 | 挂账 |
| CTRL `<b>abc</b>` | `<b>abc</b>d` qS-after=false | 语料无此案 | 行为合理 |

qS_after 全 false（off 态读数一致）。探针 zz-probe-d4br/d4x/d4f3 入库
（gitignore add -f），zz-probe-d4ctl 删（PLAIN-2nd 观察=探针伪象，语料
每 case 全新 host 不触此形）。

## 门禁（gate20 批，systemd user unit + test-guard）

**multitest 9,590P/502F → 9,588P/486F**——主信号逐案核对（?9001-last 溢出桶）：

- **D4 六 fixture 主翻转**：F1-F5 compare innerHTML + qS-after 全数 Fail→Pass
  （5+6=11 恰桶内 F→P 全额）；F6 compare 保持 Fail（嵌套尾段挂账定谳一致）、
  qS-after Fail→Pass。主 case 行 F→P=0/P→F=0/new=0/gone=0 完美恒等。
- **溢出桶 18 断言消失（5F+13P）定谳环境截断非回归**：消失集与 g18 缺失集
  精确子集（g18 −27 ⊃ g20 −18），翻转序列 g13-g17 present（五种引擎状态）→
  g18 absent（F6 FixA/B diff，与消失 fixture 无因果路径）→ g19 present（Fix C
  diff，同无因果）→ g20 absent——随无关 diff 翻转即 flake 签名；消失 fixture
  为 delete/forwarddelete 无格式命令形 + `<p><b>abc[]</b></p>` typing 形，
  与 D4 臂触发条件（collapsed insertText + off 表非空）不相交（delete 形无
  format 命令、caret 元素 offset）；主行零位移。按 jc 墙钟预算截断同款先例
  记档不挂账，后续批自证（g18→g19 已自行恢复一次）。
- inserttext **2,779P/230F 恒等**（D4 臂同域激活面零位移，F2 面保持）。
- 恒等电池全绿（用例级 verdict diff=0）：ipara 6,728P/467F、forecolor 2,279P/48F、
  wss-ip 1,104P/0F、wss-ilb 552P/0F、run-ilb 1,641P/50F、join 三案
  3,456/2,607/1,455P/0F、chd 420P/0F、ipihost 880P/0F、void 双语料 72+48P/0F、
  iloop 1,619P/142F、illist 1,910P/155F（F1 面保持）；engine lib rc=0
  **3,684P/0F**。

净账：multitest 桶内 F→P 11（D4 主翻转）、P→F 0、消失 18（5F+13P）环境截断
定谳（见上）；语义探针 5/6 语料逐字 + F6 挂账。

## 残余挂账

- F6 嵌套尾段重挂形（1 fixture）。
- TypingStyle 7 案（CE-false span ×4 + 全段 `<b><i>` 双壳 ×2 +
  styleWithCSS-ipara qS ×1）——F6 轮挂账，本轮未并入。
- shim DocumentFragment.appendChild 移动语义 + Range.extractContents 尾兄弟
  迁移（共享根，独立轮）。
