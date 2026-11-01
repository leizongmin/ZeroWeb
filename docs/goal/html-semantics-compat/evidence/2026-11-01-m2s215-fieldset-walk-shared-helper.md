# M2-S215：fieldset 禁用传染走查共享化——`_zwDisabledFieldsetAncestor` 单源收编

- **日期**：2026-11-01
- **目标面**：s210/s211 挂账承袭——fieldset 禁用传染走查 shim 内多处同构
  内联复制，s212 边界集统一后漂移风险实证（host 侧 s214 三处分歧即前车），
  提取共享帮手收编谓词走查面
- **状态**：收口（**重构切片，行为保持 + 单点对齐**）——zero-engine
  3674P+0F（基线 3673 + R5382）；forms 语料 A/B 共享键 4725/4726 零漂移
  （唯一位移 = onblur flake 族在册摆动键，与本刀无耦合）；F→P=0；键集
  零差

## 勘根

- **走查同构面盘点**（LEGEND 扫描锚 grep）：谓词走查两处——①part03
  `_zwFeDisabled` fieldset 臂（s212 参考模型）；②part04 willValidate 臂
  内联（s193 disabled-001 钉面，DATALIST 早断 + fieldset 臂融合循环）。
  行为面第三处 part06 `_zwFieldsetDisableFocusReset`（s197 焦点复位，
  **动作融合**——非豁免命中即内联 unfocus 序，只可共享 legend 扫描碎片
  不可整走查，本轮不动）。s210 的 `_validityState` 内联复制 s211 已先行
  收编至 `_zwFeDisabled`（:12994 try 调用）。
- **s211 账本原句**：「fieldset 走查三处同构（`_zwFeDisabled` 帮手 +
  part04 willValidate 臂内联）——willValidate 臂共享化仍挂」。
- **不可掺入共享帮手的语义**：DATALIST 早断是 willValidate 特有 barred
  条款（§4.10.5.2.2）；`_zwFeDisabled` 的边界集臂把 DATALIST 当截断点、
  fieldset 臂当包裹穿行——若帮手掺 datalist 语义，plain `<datalist><option>`
  将误判禁用。帮手 = 纯 fieldset 臂，datalist 走查保留 willValidate 独立。
- **拆臂行为对齐点**（唯一行为变更，enumerated）：原 `_zwFeDisabled`
  融合循环 `else if` 使首父为 FIELDSET 时边界臂整体缺席（`_fdDepth` 随
  后 > 0 永不再中）——fieldset 隔在 option/optgroup 与其 select/optgroup
  祖先之间的形态不传播 select 禁用。拆独立走查后按 concept-option-
  disabled「后代」语义传播（与 s214 host 权威模型对齐——s214 Rust 面本
  就如此）。HTML 树构造不可产 fieldset 嵌 select 形态（in-select 模式
  忽略），仅脚手 DOM 面可达；语料零覆盖。

## 施工

（shim 2 文件 + rust 测试 1 文件；语料零改动）

- **part03 新共享帮手 `_zwDisabledFieldsetAncestor(sel, handle)`**（
  `_zwFeDisabled` 后，拼接序先于 part04/05/06 可见）：逐层 disabled
  fieldset 命中 → 走查位（祖先链直子）为该 fieldset **首个 legend 元素
  子**豁免（`_dfWalk === _dfLeg`，`_dfWalk` 随链推进 per-fieldset 语义，
  legend 包 p/嵌 fieldset 隔层、第 2+ legend 均不豁免）→ 豁免继续上行；
  未豁免返 true。guard 64。逐行镜像原 `_zwFeDisabled` fieldset 臂
  （变量 `_fd*`→`_df*`，`_df` 前缀 grep 无碰撞）。
- **part03 `_zwFeDisabled` 拆臂**：边界集臂原样拆独立走查（自首个父元素
  起，SELECT/OPTGROUP/HR/DATALIST/OPTION/包裹穿行语义逐行不动）+
  fieldset 臂移交帮手调用；`:12820` 处注释同步（「保留原内联走查不动」
  →「s215 起同源」）。
- **part04 willValidate 臂**：fieldset 臂移交帮手（try/typeof 调用形
  同 :12994 house style）；DATALIST 早断保留独立走查（`_wvDl*` 新前缀，
  原 `_wv/_wp/_guard` 变量随融合循环消亡——尾部无其他引用核对）；自身
  disabled/type/readonly 门原样后置（四条件 OR 和与顺序无关，位精确）。
- **part09.rs R5382**（`test_r5382_fieldset_walk_shared_helper_alignment`
  ）：脚手 DOM 面构形 `select[disabled] > fieldset[disabled] > option`
  （fieldset 经 markup 外置 + appendChild 移入——解析面不可产）三断言：
  ①option `matches(':disabled')` true——fieldset 隔层不缺席边界臂（旧
  shim 返 false 因目标行为失败，对齐钉证）；②`willValidate` false——
  disabled fieldset 祖先经共享帮手 barred（willValidate 面）；③select
  自身 disabled 直判 sanity。**修正记录**：首版 fieldset 未带 disabled，
  断言②设计错误（spec barred 列表不含 disabled select 祖先，willValidate
  正确值应为 true）——补 disabled 属性后成立，非实现回归。

## 验证

- 拼接 shim（15 part include 序）node --check SYNTAX_OK（首跑暴露插入
  点双重 `}` 括号失衡，修正后过）。
- R5382 单测 1 passed（断言修正后）。
- zero-engine lib 全量 **3674P+0F**（3673 基线 + R5382；disabled-001
  镜像、fm3、R5380 各腿、matches 四伪类臂全绿——触面回归零）。
- forms 语料 A/B（对 s213-forms-wip.log，LC_ALL=C，(key,state) 对全
  测试名整行键 sort -u join）：共享键 **4726 对**，**零漂移 4725** +
  位移 1 = checkable-active-onblur radio spacebar（onblur flake 族在册
  双向摆动键 s209~s213——本轮 B 侧 Pass → A 侧 Fail 摆走，:active/焦点
  语义与 disabled 走查无耦合）；**F→P=0**；键集差 0（两侧各 4726 对，
  beforeinput 方差本轮两侧同态未现）。日志
  evidence/logs/s215-forms-wip.log（5262 行 tee 全量；make 错误 1 =
  失败存在正常形态）。
- guarded-clippy 0 警告；`cargo fmt -p zero-engine -- --check` EXIT=0。

## 挂账

- part06 `_zwFieldsetDisableFocusReset`（s197 焦点复位）legend 扫描碎
  片与共享帮手内部同构（~8 行）——动作融合面，整走查不可收编；碎片级
  共享收益小、触 disabled-003 钉面，本轮不动（保留观察）。
- `__zw_has_attr` 非 lw 存量点位审计（s211 承袭）。
- part03 `_zwMElCheckValidity` FORM 分支短路形态（s210 挂账承袭，本地
  _zwMEl 树面无语料未动）。
- onblur flake 族持续采样（本轮摆走 1 键，族内双向闭环延续）。
- beforeinput.tentative.html 既有面（录入事件族独立评估承袭）。
- 上游 customizable select 特性面 177F 挂起待拍板承袭（GB-20261031）。
- form owner table / parser form element pointer 缓行族（用户决策承袭）。
- form-nameditem override builtins 待决（承袭）。
- input-type-button click() 返回值面（共享通路高风险，未入刀，承袭）。
- 多进程 renderer SetDomSnapshot entries 化（s174 低优先承袭）。
