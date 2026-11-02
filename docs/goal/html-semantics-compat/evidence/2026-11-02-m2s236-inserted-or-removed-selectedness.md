# M2 s236 — inserted-or-removed innerHTML selectedness（身份无关状态键 + notify 钩序）1F→P（连带 +1F）

日期：2026-11-02。切片号 s236（承 s235）。三层根、两次施工位（part03 状态键归一 + part01 钩移位）。

## 入场定刀位

s235 下一步候选：inserted-or-removed 1F（innerHTML 插入 SELECT 的 OPTION
selectedness 未生效），先核对 s228 挂账「innerHTML sel 容器 identity 架构」
是否同根定车道。

## s228 挂账核对（相关不同根，车道独立）

s228 账本两层根：selectorFor nth === 跨族判定 + innerHTML t8r 物化仅
handle 容器（sel 容器产物留 `_zwMEl` 无身份解析视图）。本切失败是
selectedness **计算面**（option 节点 querySelectorAll 可查、
selectedOptions 视图空、selectedness 归一错人），非 testdriver identity
面——共享「sel 容器产物无身份」这一架构背景，但失败机制与修复面均不同，
车道独立施工；架构物化面仍归 s228 挂账不动。

## 勘根（仪器：探针 assert_true(false, dump) + `__zwS236Trace` 临时插桩，验后全撤）

失败面（inserted-or-removed.html 5 子测，入场 4P+1F）：

- **根 1（subtest 4「Inserted by innerHTML」）**：`_zwListOfOptions`
  走查 tag 探测链死路——无身份 pending 解析视图（innerHTML= 直写 select
  产物，R304，无 sel 无 handle）`_realTag(null,null)` 回落
  `_tagFromSel(null)` 恒 `'DIV'` → OPTION/OPTGROUP 全落空 → 列表空 →
  `selectedOptions[0]` undefined。
- **根 2（根 1 修复后暴露）**：无身份 option 共享 null 状态键——
  `_zwOptKey(null,null)` 恒同键，`_optSelectedDirty`/`_optSelectedClean`
  跨对象交叉污染；且读侧（视图 hit）走 wrapper 本地属性、写侧（dirty 层）
  对 plain 死路，读写模型分裂。修复 = `_zwOptStateKeyOf`（plain 每对象
  stamp `'plainOpt:pN'`；有身份恒等于旧 `_zwOptKey` 零回归）+
  `_zwOptSelOf`（dirty → clean → `_zwOptSelState` → 本地 hasAttr 四层
  统一读）替换全部读点（4 处：selectedView hit / valueMissing /
  optionsCollection selectedIndex / select selectedIndex trap）与写点
  （6 处）的裸键。
- **根 3（subtest 5「insertBefore .selected=true」中程回红定谳）**：
  `_mo_notify` childList 块内 R3909 钩原挂头部——先于
  `_zwHCLiveInvalidate`（bucket）与 R51 `_zwNodeParent` 反链登记。
  insertBefore(dirty B) 时 setting 走 `_zwListOfOptions('#p5')` 见旧列表
  `[A,C,D]`（B 未入 bucket、反链未登记），affKey=@B 但 B 不在列 → C 未按
  last-wins 清脏 → notify 后视图 `[B,A,C,D]` 中 B dirty-true + C
  attr-true，last 胜出 = C（签名 `expected "B" got "C"`）。插桩 trace
  实证钩确已跑、affKey 确已对——是**时点**错，非键错。修复 = 钩移到记账
  之后（part01），s111 script 臂同构先例（「原子插入」语义：全部记账完成后
  按 post-mutation 树触发）。修复后 trace
  `["container:aff=undefined","perPoint:@__n0"]` 定谳双臂分工：innerHTML
  plain 批（无身份，逐点臂 `_zwOptNearestSelect` 恒 miss）经容器臂；
  带身份 insertBefore（反链已登记）经逐点臂、受影响者绝对胜出。
- 容器臂 affKey 注释同步改写：原记「父链未注册（R51 反链登记在本钩之后）
  恒 miss」理由随钩移位过时，plain 无身份才是逐点臂 miss 的恒常真因。

## 施工（+113/-32，两文件）

`crates/engine/src/js_dom_shim/part03.js`（+98 净）：
1. `_zwListOfOptions` tag 臂：sel/handle 皆 null 时读本地 tagName
   （`_zwDetailsCollect4100` mixed-tree 走查先例；wrapper.tagName 是解析期
   真 tag，spec tagName 不可变无陈旧面）。
2. 新 helper `_zwOptStateKeyOf` / `_zwOptSelOf`（紧邻 `_zwOptKey`）。
3. `_zwSelectednessSetting`（affected 键 + post-loop last 键）、
   `_zwSelectResetOnMutation` 逐点臂（`_affK`/`_insList[0]`/`_ido`/`_ico`）、
   `_zwOptAnyDirty`、`_zwOptWriteSelState`、`_zwOptCaptureSelDefaults`、
   `_zwSetSelectSelectedIndex` 键归一（全部 `k != null` 守卫）。
4. 4 读点换 `_zwOptSelOf(...) === true`。
5. FIXME(s236) 注记：re-fallback 清理臂保持 identified-only（plain
   hasAttr 恒 false 会误清属性背书真态，attr 保护门失效）。

`crates/engine/src/js_dom_shim/part01.js`（+14/-8 净）：
6. R3909 钩移位至 HCLive/bucket + 反链登记之后，s236 注释 +
   spec ask-for-a-reset 链接；catch 恢复原空体。

## 验证

- solo inserted-or-removed **5/5 全 Pass**（修复前 4P+1F；根 3 中程回红
  一轮插桩定谳修复）。
- 全树 A/B（`evidence/logs/s236-ab.log` vs s235 基线
  `evidence/logs/s235-ab.log`）：**8098 键 join，key 集零增删，恰 2 键
  F→P，零 P→F**——
  1. 目标键 inserted-or-removed innerHTML；
  2. 连带键 customizable-select/select-value-selectedOption「HTMLOption
     .selected updates select.value and select.selectedOptions」（同根
     selectedness 状态键收益，solo 复跑稳定 Pass，同文件 slotting 余 F
     双侧同态非本切）。
- checkable-active-onblur-with-click following-label：F(A 全树)→P(B 全树)
  →F(B solo)——在册 s196/197 flake 族负载侧摆动（点击/blur 时序敏感），
  双向有档不归因不计（与 select/option 机制无通道）。
- node --check 全 shim concat 语法干净；`__zwS236Trace` grep 零残留；
  探针文件 zz-s236-probe.html 验后删。

## 残余挂账（本切不动，FIXME/注释已注记）

- plain option `t.value` 读值死路：value attr / 子文本链对无身份视图不可达
  （探针实证 selected plain option `val:''`，spec 应 "Second"）。
- `_zwOptIsDisabled` optgroup-disabled 继承对 plain 惰性。
- re-fallback 清理臂 identified-only（attr 保护门优先，见 FIXME）。
- 架构物化面归 s228 车道（innerHTML sel 容器 identity）。

## 下一步候选

email-set-value 1F（文本控件 selection ↔ window.getSelection 集成面，
中面非单行修）；或 color.window.js 语料缺口（补资产 flow）；或 plain
value 死路补面（本轮新账，与 s228 车道交叉评估后定刀）。
