# M2 s148 · input-type-change-submit 挂死根收口——`_validityState` type 读快照盲区（s136 同根第 2 处）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s147（realm shadow 自导航递归收口）

## 入场定刀位

s147 下一步候选④：form-submission-0 余下基线 Fail 面单根翻拣（urlencoded2 6F /
text-plain 6F / multipart 6F / submission-checks 3F / input-type-change 2F）。
入场甄别：

- urlencoded2/text-plain/multipart 各 6F = 18 案「characters not in
  encoding / lone surrogate」→ **归口 encoding 流 M1**（run-rules §9
  「25 号流不重复施工」，s146 已定）。
- submission-checks 3F 用 `document.createElement("frame")` → FRAME
  支持需 ~30 shim gate 点 + host 侧，3 keys 代价过高 → **否决**（本轮
  入场评估）。
- input-type-change-submit（C2 face）→ **取刀**。定向复现：text/
  search/tel 3 Pass 后文件级 Timeout，completion state
  `{"loaded":true,"pending":17,"phase":4,"tests":20}` = 第 4 案（url）
  `loadPromise(iframe)` 永不 resolve；s147 全量该文件仅 3 行结果 +
  1 文件 Timeout 对（s146 全量 3 行同形态）。

## 勘根（上游定谳 + 探针四案三分辨）

1. **上游基线**：上游 WPT 同路径文件存在
   （html/semantics/forms/the-input-element/input-type-change-submit.html，
   gh api 核对 1377 字节）且**无 .ini 期望文件** → 上游全绿；
   语料 20 案全断言导航取参 `n1=v2`。真浏览器不拦 url/email 案。
2. **spec 面定谳**：click handler 在派发窗口内改 `input.type="submit"`
   → 默认动作（input activation behavior §4.10.5.4）运行时 type 已是
   Submit Button state（barred from constraint validation，§4.10.21）
   → typeMismatch（§4.10.5.2.6，仅 email/url state 适用）不触发 →
   提交放行。shim 拦截 = 实现伪象，非 spec 语义。
3. **探针四案**（临时语料 zz-s148-probe-itcs，跑后已删）：
   - p1 detached `createElement('input')`（**handle 分支**）：type 改
     submit 后 `validity.valid` 翻 true——handle 路径视图 live；
   - p2 sel 路径 form 内 input（insertAdjacentHTML 产物，**sel 分支**）：
     click 后 `invalid` 事件仍派发——校验仍判 invalid；
   - p3 同形态 iframe 不导航——提交被中止实锤；
   - p4 无 type 变更 click：无 invalid 无导航（探针控制案设计误设——
     非 submit type 激活本就 no-op，反成激活门语义旁证）。
   - p1/p2 矛盾 → 视图差异定谳：同函数 `_validityState` 两分支行为
     分裂，唯一分歧点 = type 属性读取通道。
4. **单点归因**：`_validityState`（part03.js:11597 前态）type 读 sel
   分支走**纯快照** `__zw_get_attr`——R2995 latest-wins 层下 type IDL
   写（`input.type = 'submit'`）只落 lw 变更列表不落快照 → 校验时
   ty 恒旧值（url/email）→ typeMismatch → invalid 中止提交 →
   loadPromise 挂死 → 文件级 Timeout。**s136 同根第 2 处**（s136 修
   激活门：part04 两处 `.click()`/dispatchEvent 默认动作 type 读换
   lw 融合读；本轮补 validity 计算位）。
5. **观察记录（不改，挂账）**：挂死时 state pending:17 = 案 5-20 从未
   启动——shim promise_test 体**串行启动**（前案未决不启后案），上游
   语义为注册即启动（20 案并行）。单案挂死冻结尾部的结构面与 s147
   刀 B 拆账「文件级 Timeout 族」直接相关（见挂账）。

## 施工（part03.js 1 处，净 +9 行）

`_validityState` type 读 sel 分支换 `__zw_get_attr_lw` 融合读（consult
变更列表 → applied view 回落；s136 同形三目 + typeof 守卫）；handle
分支 `__zw_get_attr_handle` 本身 consult 当前批 mutations 不动。注释
引 spec §4.10.5.2.6 与 s136 同根先例。其余 part03 快照 type 读（3805
SVG script 加载、13612/13661/13680 checkbox/radio pre-click 链）非本
语料流 exercised，不触碰（精准修改）。

## 验证

- 定向 input-type-change-submit：**20/20 全 Pass**（修复前 3 Pass +
  文件级 Timeout；url/email 案从挂死翻 Pass）。
- 临时探针复跑：p3（导航取参 n1=v2）Pass、p2 不再派发 invalid（按
  设计挂起）——单点归因闭环；探针文件已删。
- **全量 join**（对 s147 /tmp/zw-s147-full.log，(key,state) 对
  sort -u）：s148 **7085P/812F/96T = 7993 对**（s147 7068P/812F/97T
  = 7977 对）；lost 5 / gained 21 全归因零真 P→F：
  - input-type-change-submit 文件 Timeout 对 → 17 子测试 Pass 对
    （本轮主刀）；
  - form-double-submit-input-type-change **2F→2P 同根连带**（该文件
    click handler 内 email/url→submit 同形态，stale typeMismatch 同
    根误拦——候选④ 面意外再收 2 案）；
  - modulepreload-referrerpolicy 2P→F：**全量负载 flake**——定向
    复跑 7/7 全 Pass 复绿；s124 轮在册 flake 先例同族（控制面
    「两 flake 页过滤重跑复绿」），与本地改动零机制关联（validity
    读面不触碰 module 加载/referrer 通道），保持观察。
- 门禁：clippy workspace --all-targets -D warnings 零告警；shim
  13-part 规范序拼接 node --check 过（**109736 行** = HEAD 109727 +
  9 净增；s147 账本 109701 为注释措辞整理前口径）；engine lib
  **3593/10F**（失败名单与 s147 逐字节相同——8 已知环境面 + R3045
  归口 net-api + details 在册间歇红；+1 pass = s147 rebase 并入的
  net-api 流新增测试）；runner bin **263/1F**（trend_csv_check 归口
  encoding 流不变；+5 pass 同 rebase 源）；零 .rs 改动（fmt 无面）。
  验证集对齐 s142/s145/s146/s147 同面先例（定向 + 全量 join + lib +
  runner bin + clippy，无 reftest 义务——行为面非渲染）。

## 挂账（本切片更新）

- **C2 input-type-change 面（候选④ input-type-change 项）收口**：
  面域 = 全文件 20 案（挂账原文「url/email 2F」系旧报法口径——近轮
  全量该文件实为 3 行 + 文件级 Timeout，本轮翻 20/20）。
- **form-double-submit-input-type-change 2F 连带收口**（同根连带，
  候选④ 面再减 2）。候选④ 余项全定谳：18 案归口 encoding（s146
  已定）、submission-checks FRAME 否决（本轮）、input-type-change 20
  案收口（本轮）——**候选④ 面关闭**。
- **新增观察挂账**：①shim promise_test 串行启动面（单案挂死冻结全
  文件尾部——上游注册即启动语义；Timeout 族与 per-test timeout 缺失
  面的共同结构根）；②modulepreload-referrerpolicy 全量负载 flake
  2 键（strict-origin/unsafe-url；定向复绿，s124 族观察先例）。
- 承袭不变：details 族 lib flake ≈40% 翻转率独立勘根；link-multiple
  / csp-blocked 文件级 Timeout 族（候选①）；无锚链碰撞理论面盘点
  （候选②）；with(window) 包合分支影子旁路面；影子名单非注册类
  API 不全；engine lib 9F 环境面；R102 live_aware pending 失明；
  待决三项（css/text 语料资源、green-2x2.png、parser form pointer
  缓行 2026-10-23）。

## 下一步（s149 候选）

①文件级 Timeout 族单根翻拣（link-multiple-load-events / csp-blocked，
刀 B 拆账余面 + 本轮观察记录①同构：per-test timeout 缺失面勘根——
上游 testharness 8s 默认 per-test 超时，挂死案翻 Fail 行、文件完成，
键集报法漂移解除面）；②无锚链碰撞理论面影响面盘点（无语料面，盘
点性质）；③encoding 流 M1 进度对账（不施工，仅边界对账）；④全量
基线目录面再翻拣（63F 目录基线面的其他文件族单根勘根）。
