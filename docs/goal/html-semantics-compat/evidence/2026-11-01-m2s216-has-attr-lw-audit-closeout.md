# M2-S216：`__zw_has_attr` 非 lw 存量点位审计收口——唯一真裸点 autoplay sel 面修复

- **日期**：2026-11-01
- **目标面**：s211 挂账本体——`__zw_has_attr`（纯快照读）非 lw 融合的存量
  点位全量审计。背景语义：setAttribute 写 lw 层不落快照，同批纯快照读恒
  miss（s211 探针实测）；lw-first 三元（`__zw_has_attr_lw` consult 变更列
  表 → 回落 applied view）是 house 安全形态
- **状态**：收口——审计 39 命中行 = 2 注释 + 37 代码调用点；修复前唯一
  真裸点 = part06 `_zwMediaAutoplay` sel 面 autoplay 读，已修复；修复后
  37/37 lw-first 安全形态；R5383 红绿实证；embedded-content 语料 A/B
  共享键 1257/1257 零漂移 + 键集差 0；**s211 该挂账销账**

## 勘根（审计方法与全量分类）

- **审计单元**：`grep -n '__zw_has_attr(' part*.js` 行级命中（排除
  `_lw`/`_handle` 后缀变体——二者是不同函数，子串不误匹配）。
- **分类方法**：逐命中行判注释/代码；代码行取 ±窗口（前 7 后 2 行）
  查 `__zw_has_attr_lw(` 同现——同现即 lw-first 三元 fallback 臂（安全
  形态），否则真裸点。
- **全量分类定稿**（修复后复核定稿，命令可复现）：
  - grep 命中 **39 行** = **2 注释行**（part03:12245 s211 审计留注、
    part03:12825 R4400 探针留注）+ **37 代码调用点**。
  - 37 代码调用点**全部 lw-first 安全形态**（真裸点 0）：1 = 本轮修复
    的 autoplay sel 读（lw 三元 fallback 臂），36 = 原生即 lw-first/
    回落形态（part01 popover/open/popovertarget/commandfor/sandbox/
    iframe 属性 7 站、part03 表单/详略/类名/select/option 面 15 站、
    part04 属性反射/removeAttribute 存量/readonly 面 9 站、part05
    attribute 面/name 面 2 站、part06 loop 读 1 站、part01b~part08d
    其余 3 站——按 part 分桶计数，明细见 grep）。
  - **修复前状态**：37 调用点中 36 lw-first 安全 + **1 真裸点**（
    part06 `_zwMediaAutoplay` :11143 sel 面 `(typeof __zw_has_attr ===
    'function') && __zw_has_attr(sel, 'autoplay') === '1'`——纯快照
    读，parsed 元素同批 setAttribute('autoplay') 后该读恒 miss）。
  - 中间过程计数（40 站口径）为窗口分类脚本的行集差异，以本节复核定
    稿为准。
- **身份二分判据**（红绿确定性基础）：detached createElement 元素 =
  handle-only（`__zw_has_attr_handle` 直存直读，不受快照影响——a2fix1
  IDL+handle 路测试通过即证）；parsed 元素 = sel 身份（`__zwSelector`）
  ——桥测试环境 mutations 永不 apply 宿主快照，parsed 元素 setAttribute
  对裸读恒不可见。故裸读暴露面仅在 parsed（sel 身份）元素 + 同批 lw 写
  形态。

## 施工

（shim 1 文件 + rust 测试 1 文件；语料零改动）

- **part06 `_zwMediaAutoplay` sel 面 lw 融合读**（:11138 起）：sel 臂改
  `typeof __zw_has_attr_lw === 'function' ? __zw_has_attr_lw(sel,
  'autoplay') : __zw_has_attr(sel, 'autoplay')` 三元，同函数
  `_zwMediaLoadSequence` src 读（:11117）与 loop 读（:10955）对齐；
  handle 臂不动（直存直读本就正确）；加 s216 留注（审计收敛末点语义 +
  s211 探针依据）。行为面：parsed video/audio 同批 `setAttribute(
  'autoplay','')` + load 后 autoplay 生效判定从恒 miss 修正为 consult
  变更列表命中。
- **part09.rs R5383**（`test_media_autoplay_setattr_sel_lw_read_r5383`
  ）：a2fix1 同款 timer stub（`__zw_pending`/`__zw_timers`/
  `__zw_setTimeout`/`__zw_fire_due_timers`）+ `__zwVideoBridge` 记录
  stub；dom_html `<video id="v">`；JS 路径 = parsed `getElementById`
  取元 → 挂 play/playing/timeupdate 监听 → `setAttribute('autoplay',
  '')` + `src = '/media/movie_5.webm'`。checkpoint 断言
  `__calls.join('|') === "play:https://wpt.test/media/movie_5.webm"`
  （autoplay 经 lw 读命中 → load 序末 `_zwMediaAutoplay` 触发 play）+
  `paused === false`；泵后 `__ev.join(',') === "play,playing,
  timeupdate"` 事件链。

## 验证

- **红绿实证**（R5383）：`git stash push part06.js` 还原跑红——
  `left: ""  right: "play:https://wpt.test/media/movie_5.webm"`（裸读
  恒 miss，play 未触发，`__calls` 空，旧版因目标行为失败）；stash pop
  恢复跑绿——1 passed。红绿确定性由身份二分判据保证（桥测试环境
  mutations 不 apply 宿主快照）。
- 拼接 shim（15 part include 序）node --check SYNTAX_OK。
- zero-engine lib 全量 **3675P+0F**（3674 基线 + R5383；a2fix1
  IDL+handle 路同批绿——handle 臂未受扰）。
- **embedded-content 语料 A/B**（本切片最近域；A = 修复在位，
  B = stash pre-fix 基线；LC_ALL=C，(key,state) 对全测试名整行键
  sort -u join）：共享键 **1257/1257 对，零漂移**，键集差 0。预期分
  歧条件（sel 身份 + 同批 lw 写 autoplay + 读前宿主未回放）语料零覆
  盖（A 侧仅 4 行提 autoplay 且属 lazy-loading 既有 F 族，两侧同
  态）。日志 evidence/logs/s216-embedded-content-wip.log（A，
  1742 行）与 s216-embedded-content-prefix-baseline.log（B，1736
  行）；MAKE_EXIT=2 = 失败存在正常形态（205F+30T 为 lazy-loading 族
  既有面，两侧一致非本刀引入）。
- forms 域未重跑：修复点位仅媒体 autoplay 面（`_zwMediaAutoplay`），
  forms 语料无媒体加载序覆盖，零触面。
- `cargo fmt -p zero-engine -- --check` EXIT=0。

## 挂账

- **s211 `__zw_has_attr` 非 lw 存量审计挂账：销账**（本轮收口，37/37
  lw-first；回归性新建面约定 lw-first 即可，无需常设审计）。
- part06 `_zwFieldsetDisableFocusReset` legend 扫描碎片共享（s215 承
  袭，观察）。
- part03 `_zwMElCheckValidity` FORM 分支短路形态（s210 承袭）。
- onblur flake 族持续采样（s209~s216 在册摆动键承袭）。
- beforeinput.tentative.html 既有面（录入事件族独立评估承袭）。
- 上游 customizable select 特性面 177F 挂起待拍板承袭（GB-20261031）。
- form owner table / parser form element pointer 缓行族（用户决策承
  袭）。
- form-nameditem override builtins 待决（承袭）。
- input-type-button click() 返回值面（共享通路高风险，未入刀，承袭）。
- 多进程 renderer SetDomSnapshot entries 化（s174 低优先承袭）。
