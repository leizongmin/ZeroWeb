# M2 s134 · reset 按钮 click 激活接 form.reset()——reset-form 4F + reset.html 4F + commandfor 3F 收口（真翻红 0）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s133（form reset handle 身份枚举 + 清脏复位）

## 入场定刀位

s133 挂账「click 复位 6 面」入场：spec §4.10.5.4 reset button 激活
行为 = form.reset()（trusted reset 事件 + 控件复位），目标翻
reset-form.html input/textarea click 4 面。

## 勘根定谳

1. **激活通道拓扑（双链分离）**：proxy `.click()`（part04:5566 click()
   API 门，WPT runner 路径）默认动作臂只认 submit/image——reset 型
   **完全无处理（连事件都不派发）**；part03:17793 raw node 链（UA
   指针激活）派发无信任 reset 事件但无还原臂。s133 账本原记
   「part03:17793 click 通道」实为 raw 链——runner 断言面走的是
   proxy 链 no-op。两链与既有 submit 拓扑同构（raw 链裸派发、
   proxy 链走完整机制），raw 链本轮不动。
2. **共享出口单点**：`_zwSubmitBtnDefaultAction`（net-api M5-S8，
   part04:10836）已是 .click() API 门（R57）与 dispatchEvent 派发
   click 默认动作（R106 扩面）两臂共用的 form 反链 + 激活入口——
   加 `asReset` 参数即单点接线，form 反查（`.form` IDL 直取 +
   32 跳反链 fallback）零重复。
3. **爆炸半径勘根（先圈后施工）**：corpus 内 reset 按钮 click 断言
   面 4 文件 17 Fail + button-events 文件级 Timeout（t5/t7 挂起无
   行）。form-nameditem/constructing-form-data-set/
   form-elements-filter/form-requestsubmit 仅 markup/属性断言无
   click-on-reset 行为面，不卷入。
4. **复位语义全继承**：激活臂经 FORM proxy `frm.reset()` 进 R3048
   闭包——trusted reset 事件（s129，preventDefault 可取消）+
   s133 双身份枚举/清脏复位/R2998 两步式全部自动共享，零新复位
   逻辑。

## 施工（engine shim part04.js 5 处，无 Rust 变更）

1. `_zwSubmitBtnDefaultAction(sel, handle, asReset)`：尾部分支——
   asReset 走 `frm.reset()`（R3048），否则原 `_zwRunFormSubmit`。
2. R106 dispatchEvent 臂：`_r106is` 扩 `type=reset`（BUTTON/INPUT），
   透传 `_r106ty === 'reset'`。
3. .click() API 臂：`_subIsReset` 同门检测，disabled 守卫与 submit
   同清（R313 早退 + 分支内复检同款）。

## 验证

- 定向 4 文件：resetting-a-form **恰 4 F→P**（input/textarea ×
  button/input click；select ×3 R3909 既红同根、output ×2 保持
  Pass）；the-input-element/reset **恰 4 F→P**（form owner/disabled/
  form= 属性 ×2；form element pointer 面仍红=parser form pointer
  待决项）；button-click-resets-with-commandfor **3/3 F→P**；
  button-events **2 新键 Pass**（t5 type 切换复位 + t7 innermost
  reset——原挂起无行）。
- 域回归键级 join：the-input-element 1644 行恰 4 F→P 零他移；
  the-button-element 3 F→P + 2 新键零他移；resetting-a-form 14 行
  恰 4 F→P；constraints 920 行 / form-control-infrastructure
  121 行 / the-form-element 108 行**逐键相同零位移**。
- 全量 A/B × /tmp/s133-full.log 键级 join **恰 11 Fail→Pass +
  2 新键 Pass**，**真翻红 0、new non-Pass 0、gone 0**（6790P/1019F/
  103T → 6803P/1008F/103T，行数 7912→7914 自洽）。
- engine lib **3571/0**（s133 提交新增 1 案后基线）；wpt-runner bin
  **256/0**（76 ignored）；clippy（zero-engine --all-targets
  -D warnings）零告警；本轮零 Rust 变更，fmt 不适用。shim 拼接
  （13 part）node --check 过。无探针页；语料零改动。全量 runner
  以 corpus 已知 Fail 非零退出（make 错误 1，前轮同形非异常）。
  渲染面零依赖。

## 挂账（本切片更新）

- **click 复位 6 面收口移出候选**（input/textarea 4 面翻绿；
  select 2 面归 ②select-by-reset R3909 根）。reset 域既有异根
  更新：①select-by-reset 3 面（reset-form select ×3 + reset.html
  form element pointer 面前置）R3909/解析器 form pointer 双根
  既红；②reset.html form element pointer 面待决项缓行不变。
- **button-events 文件级 Timeout 收窄待续**：挂起测试 3→1
  （t5/t7 完成并 Pass），文件行 Timeout 仍在；「type=button
  shouldn't trigger submit or reset events」Fail 承袭（submit
  误派发异根，非本轮 reset 臂引入）。
- **part03 raw 链 reset 臂（UA 指针激活）仍无还原臂**：与 raw 链
  submit 裸派发同构的既有近似，corpus 无对应用例，不扩面。
- 承袭不变：form-nameditem 残余 10F、form-submission-0 186F、
  execution-timing 构成面、lazy-loading 26F、meta color-scheme
  21F、写流盖章断链、part02 setAttribute 绕 notify、041 nested
  write、deep clone 非 HTML ns、css-module corpus 门（待决）、
  www2 可达性、module-import-referrer TLA、dynamic import() with
  透传、classic 外链 SRI、143 write、defer 多写完成序、接口对象
  基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live
  树逗号列表 qSA 根缺口、渲染流面让位累积、s131 插入步缺口细化、
  radio 现象⑤ becomes connected / checked 属性增设 /
  defaultChecked 无独立 corpus 面（现象①覆盖）；**待决（须用户
  拍板）**：css/text 数据模块语料资源缺失、green-2x2.png 语料
  资源分发、parser form element pointer 4F 缓行（2026-10-23）。

## 过程教训

- **join 前先核对自己管道**：首轮 join 输出空集是 `sed
  's/^[^ ]* //'` 把 grep 多文件前缀连 OLD-/NEW- 状态标一起剥掉——
  全部行折叠成同元组后 count 恒 ≥2 被过滤，输出「零位移」假象。
  join 工具链本身也要先用已知有位移的数据集（如 reset.html）
  校验过再信。
- **账本转述数字≠log 行数**：s133 账本「the-input-element
  1512P/132F/3T」与 s133-full.log 实际 1644 行（1512P/129F/3T）
  有出入（转述时点口径差异）——域回归对账一律直接从两份 log 文件
  全行 join，不以账本回忆数字为基线。
- **激活行为接线先勘通道拓扑再选刀位**：同一「click reset 按钮」
  行为在 shim 里有 raw 链（part03）与 proxy 链（part04）两条平行
  实现；WPT 断言面只走 proxy 链。不看 runner 到达路径就改 raw 链
  会零收益还引入双派发风险——与 s132「先定谳 spec、再找共享出口」
  同款刀法。
