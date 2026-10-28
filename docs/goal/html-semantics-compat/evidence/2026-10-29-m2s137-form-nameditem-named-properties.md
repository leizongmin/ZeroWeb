# M2 s137 · FORM named properties 语义补全——form-nameditem 10F→1F 收口 9（8 F→P 真翻红 + 剩 1F fix#19c 冲突挂待决，全量零回归）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s136（激活门 type 读 lw 融合读）

## 入场定刀位

控制面 s136 候选②：form-nameditem 残余 10F 勘根收口。该文件 17 子
测试（10 Fail / 7 Pass），覆盖 FORM named properties 全语义面
（named getter 序 / gOPD 描述符 / expando 遮蔽序 / non-configurable
invariant / past names map / RadioNodeList 形态）。

## 勘根定谳（10F 分组）

1. **gOPD 不见 named 属性**（single / multiple / expando
   already-existing 三案）：元素 proxy 的 gOPD trap 只报 expando，
   named 命中无描述符臂。
2. **多匹配返首元素非 RadioNodeList**（item / indexed 两案）：旧
   R3254-K3 named 臂 `_formControls` 首命中直返。
3. **expando 优先序缺失**（added later 案）：R3042 expando 读在
   named access 之后——后加同名控件遮蔽已存在 expando。
4. **proxy invariant 违例**（non-configurable 案）：get trap 不尊重
   target non-configurable（`Object.defineProperty(form, k,
   {configurable:false})` 经 default defineProperty 落 target 后，
   get/gOPD 不报即 V8 TypeError）。
5. **past names map 缺失**（past names 案）：改名后读 miss。
6. **「Named elements should override builtins」**（1 案）：31 内建
   名全部被 named 覆盖的要求与 fix#19c 站点兼容偏离（reset 等不被
   遮蔽，有 form-interaction fixture 实测回归证据）正面冲突——
   **挂待决（须用户拍板）**，本轮不施工，保持 Fail。

## spec 序与语义定谳

- named getter（legacy platform object ordinary get 序）：
  non-configurable target own（invariant）→ own expando → named →
  past names map。
- named property 描述符：`{value, writable:false, enumerable:false,
  configurable:true}`（[LegacyUnenumerableNamedProperties]）。
- **past names 语义修正**（施工中定谳）：past 条目仅在元素**仍是
  listed 控件**时有效——保的是改名后身份（rename→read 命中），
  remove 后读须 undefined（语料双面断言；初版「脱离 listed 后仍
  返回」理解反了，施工中按语料改正）。
- named 集合实例稳定化：`gOPD(form,name).value === form[name]` 须
  同一对象（real browser 每 form 每 name 缓存同集合引用）。
- set/defineProperty/delete 写面：named 命中一概拒绝（non-writable
  ——严格 TypeError / sloppy 静默；delete 同）；expando 走 store
  （delete 真删）与 target（non-configurable defineProperty 落
  target，delete 按 invariant 报 false）双轨。

## 施工（part04.js 1 处 + part05.js 4 处，无 Rust 变更）

- **part04 get trap FORM gate 内重写**（s119 索引臂保留）：①
  invariant 臂（target non-configurable own → `_t[prop]` 直读——
  accessor 形态安全）→ ② own expando 臂 → ③ named 臂（命中 +
  past 记账）→ ④ past 回查臂（**listed 复核**：`_formControls`
  逐项 `===` 身份比对）→ ⑤ s119 索引臂。
- **part05 顶层**：`_zwFormNamedLookup` helper（id/name 匹配；恰 1
  → 元素；>1 → `_zwMakeCollection(hits, false)` + per-form per-name
  实例缓存 `_zwFormNamedCollCache`（WeakMap 键 = 元素 proxy，成员
  逐项身份比对不变复用、变更重建，换代清空同 `__zw_reset_pending_state`）
  + `var _zwFormPastNames = {}`。**IIFE 顶层声明**——初版误放
  get trap 函数体内（每次属性访问重建且 part05 trap 不可见，同
  `_zwSubmitBusy` 注释判例），施工中搬家。
- **part05 gOPD trap**：invariant 臂（target non-configurable 如实
  上报）+ FORM named 臂（四字段描述符）。
- **part05 has trap**：FORM named 臂（`in` 可见性；namedItem 案
  `"namedItem" in form` 须 false——排除名单同 get gate）。
- **part05 set trap**：else-if 链末端（反射 IDL 先行，fix#19c 偏离
  同侧）named 命中 `return false`。
- **part05 新增 defineProperty/deleteProperty trap**（handler 原无
  此二键）：defineProperty——named 命中 false，否则
  `Object.defineProperty(_t, ...)`（non-configurable expando 落
  target，WPT new-name2 案语义）；deleteProperty——① target
  non-configurable false → ② expando store 真删 → ③ named false →
  ④ target ordinary 兜底。

## 验证

- 定向：form-nameditem **16P/1F**——10F 收口 9（8 F→P +
  expando 遮蔽序案修复），唯一 Fail = override builtins（挂待决）。
- 九域键级 join（对 s136 域 log）：resetting-a-form 14 /
  the-form-element 108 / form-control-infrastructure 121 /
  constraints 920 / the-input-element 1644 / the-select-element 314 /
  the-option-element 108 / the-textarea-element 41 /
  the-button-element 56——**恰 8 F→P、零位移、零 GONE**；textarea
  1 条 NEW 为 dup-key 副本假阳性（两侧各 2 份同 Pass，s136 同款）。
- 全量 join：7914/7914 行守恒，**恰 8 Fail→Pass 真翻红、new
  non-Pass 0**（6805P/1004F/102T → 6814P/996F/102T）；3 条 NEW
  （textarea / srcset `\n` 变体 / execution-timing 149）逐一核验
  均为 dup-key 副本假阳性（两侧各 2 份同 Pass，净零）。
- engine lib：3584 用例 **连续两轮 3584/0**（间歇 details flake
  未复现；挂账口径不变）。
- runner bin：**255/1F**（76i）——唯一 F =
  `trend_csv_check::checked_in_suites_csv_passes_generator_contract`，
  定谳为**并行流 5a5da454b（encoding-legacy-mb DC-3）对
  docs/compat/trends/wpt-suites.csv 的写回损坏**，非本轮引入（本轮
  零 Rust 变更）：① 头部注释行 2/4 被_csv 模块 quote 包裹（违反
  文件头第 6 条「禁止 csv 模块整体回写补引号」自明规则）→ **本轮
  已机械恢复两行原样**（修复后第一炸点消除）；② encoding-legacy-mb
  2026-10-28 数据行（文件行 209）字段语义错位——rate 位落
  `completed`、status 位落 `active`、evidence 列被 note 文本顶替
  （漏写 evidence 路径）——**跨流数据行账务内容，不越权代写**，
  归口 encoding 流处置（挂账）。
- clippy（zero-engine --all-targets -D warnings）零告警；零 Rust
  变更 fmt 不适用；shim 拼接（13 part）node --check 过；探针页
  （identity/corpus-clone 两页）验后删；语料零改动。

## 挂账（本切片更新）

- **「Named elements should override builtins」1F 挂待决（须用户
  拍板）**：spec 全覆盖（named 遮蔽含 reset/elements/length/action
  等 31 内建名）vs fix#19c 站点兼容偏离（reset 不被遮蔽防宿主
  `typeof f.reset==='function'` guard 失效，有实测回归证据）。
  施工面已按偏离侧落地（排除名单 get/has/gOPD/set/defineProperty/
  delete 六 trap 同款），拍板翻转时改名单即可。
- **encoding-legacy-mb CSV 行错位归口 encoding 流**（runner bin
  唯一 F，见验证节②；pending encoding 流 DC-4 或用户处置）。
- **RadioNodeList live 语义近似**：多匹配集合为构建时快照（成员
  变更重建近似 live；语料无 live 断言面）。
- **past names 条目不显式清理**：listed 复核天然过滤失效条目；
  内存面单 form 条目数 = 曾命中名数（可控）；改名/重组的显式
  清理语义 corpus 无面。
- **expando store 与 target 双轨**：`form.x = 5`（set trap）入
  store；`Object.defineProperty(form,'x',{...configurable:true})`
  落 target——后读 store 优先，defineProperty 值不可见（语料无面，
  pre-existing 类同缺口）。
- 承袭不变：part03 raw 链（UA 指针激活）type 读通道未勘、engine
  lib 默认并行 details flake 独立勘根（s135/s136 双数据点）、
  reset.html form element pointer 面（parser form pointer 待决项
  缓行）、part03 raw 链 reset 臂无还原臂、form-submission-0 186F、
  execution-timing 构成面、lazy-loading 26F、meta color-scheme 21F、
  写流盖章断链、part02 setAttribute 绕 notify、041 nested write、
  deep clone 非 HTML ns、css-module corpus 门（待决）、www2 可达性、
  module-import-referrer TLA、dynamic import() with 透传、classic
  外链 SRI、143 write、defer 多写完成序、接口对象基建面、FIXME 组、
  /tmp/zw-hb*.txt、FileList 缺索引访问、live 树逗号列表 qSA 根
  缺口、渲染流面让位累积、s131 插入步缺口细化、radio 现象⑤/
  checked 属性增设/defaultChecked 无独立 corpus 面、s135 已知限制
  3 条；**待决（须用户拍板，既有）**：css/text 数据模块语料资源
  缺失、green-2x2.png 语料资源分发、parser form element pointer
  4F 缓行（2026-10-23）。

## 过程教训

- **探针断言逐条跑**：assert 系列抛错即中止——多条数据线要拆
  独立 test() 块，否则第二条数据永远带不出来（identity 探针首版
  踩）。
- **join 提取管道自身要先验**：`sed 's/^Pass /Pass\t/; s/^Pass\t//'`
  第二个表达式连分隔符一起删 → 状态列丢失 → MOVE 永不触发 → 九域
  「零位移」假绿一轮；修正后 8 F→P 才显形。对账管道的空输出/零
  位移要先喂已知差异样本再信。
- **awk `$1=""` 触发行重构**：字段赋值使 `$0` 按当前 OFS 重建，
  多空格塌缩——提取 key 前后两侧同构变换才可比；剥前缀用
  `sub(/^[^ ]+ /, "")` 而非字段赋值。
- **消息尾 tab 毒化 tab 分割**：runner 原样输出消息（`for='  WINdow\t`
  行尾带 tab）→ `key\tst` 变 `key\t\tst` → FS='\t' 时状态落 $3；
  对账前先扫空状态行。
- **并行流文件损坏的边界**：机械损坏（注释行 quote 包裹，恢复
  原样即可验证）就地修复；数据行语义错位（无原样、涉及他流账务
  归口）不越权代写，定谳归口 + 挂账。门禁唯一 F 用「非本轮引入 +
  定谳 + 归口」口径如实记录。
- **行为语义先对语料再动手**：past names map「remove 后须
  undefined」与直觉（past = 曾用名永久有效）相反——初版按直觉
  实现会在施工后定向跑时翻车；语料逐断言读完再定实现。
