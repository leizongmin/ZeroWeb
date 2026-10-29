# M2 s144 · 刀 C re-entrant submission hang 收口——form.submit() 放行 + cannot-navigate 中止（hang 全消 + preventdefault 双键翻红 + 15 键新 Fail 归因 selector 键状态泄漏）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s143（查询视图已应用前缀重放双计修复）

## 入场定刀位

s143 下一步候选①：刀 C re-entrant submission hang。失败形态（s143 基
线）：form-double-submit-requestsubmit.html runTest2 首案
（cRS:T,cS:T,pD:T,pass:T）loadPromise 挂死 → 文件级 Timeout + 24 键
NotRun；form-double-submit-preventdefault 键 1 Fail（expected n1=v6
got v2）+ 键 2 Timeout；form-submission-algorithm 第 9/10 子测挂。

## 勘根（spec 原文 + 语料语义锚定）

1. **spec 4.10.22.3 form submission algorithm 原文取得**（web reader，
   2026-10-26 版）：步 5 整段（5.1 firing submission events flag 检
   查、5.2 置位、5.4 interactive validation、5.6 submit 事件派发、
   5.9 事后 cannot navigate 复查）仅在「submitted from `submit()`
   method is false」时执行——`form.submit()` 直接跳到 entry list 构
   造（submitter = form 自身哨兵，按钮不贡献 entry）+ 步 9 cannot
   navigate 复查 + 导航。步 9 原文：「Cannot navigate is run again as
   dispatching the formdata event in constructing the entry list could
   have changed the outcome。」步 1 cannot navigate（form 未连入文档
   等）在算法入口。
2. **旧守卫逆语义**：review m3a 给 submit() 加的 `_zwSubmitBusy` 短
   路（submit 派发窗口内 return）使 handler 内 submit() 无导航——
   runTest2 全部 cS:T 案的期望值（如 (T,T,T,*) 期望 n1=v2 = submit()
   时刻 entry）钉死 submit() 必须放行。且宿主 Enter 通道
   （HtmlUserAction::Submit）本就不设该守卫（m3b 已知边界）——JS 通
   道短路、宿主通道放行，同窗口双通道不一致；去守卫是消除不一致而
   非引入新行为。
3. **not-connected 中止缺失**：preventdefault 键 2（form.remove() 后
   第二次 submit() 不得覆盖首次导航）与 algorithm t10「Cannot
   navigate (after constructing the entry list)」（formdata handler
   内 e.target.remove()）共同钉死检查位次 = formdata 派发后、导航前
   （spec 步 9）；入口检查 = 步 1（submit() 通道无事件窗口，由此覆
   盖）。
4. **C3 假说证伪**：预判 runTest 4 Fail（pDRS:F,pDSB:T expected v1
   got v2）为「第二次提交 preventDefault 未生效」。探针 T2/T3 证伪：
   click 默认动作 submit 事件 + PD 生效（submitCount=1 无导航）、内
   层 requestSubmit firing 窗口被挡 + 外层 PD 生效（navCount=0）——
   4 键实为 selector 键状态泄漏（见下），非 PD 缺陷。

## 施工（part04.js 3 处）

- **form.submit() 方法**：去 `if (_zwSubmitBusy) return;` 短路，注释
  重写为 spec 步 5 门控语义 + runTest2 语料锚点；m3b 宿主通道双导航
  FIXME 保留（该边界不受本轮影响）。
- **_zwFormSubmitNavigate 函数顶**：cannot-navigate 检查（spec 步
  1/5.9——requestSubmit/click 通道覆盖 submit 事件窗口内的移除；
  submit() 通道覆盖入口）。cannot navigate 完整定义（node document
  fully active / salvageable）未实现（FIXME 挂账），以
  `document.contains` connectedness 近似（与 _zwRunFormSubmit 入口检
  查同型）。
- **formdata finally 块后、dialog 分支前**：cannot-navigate 复查
  （spec 步 9 原位——formdata handler 内移除 → 构造完成后中止导航）。

## 验证

- shim 13-part 规范序拼接 node --check 过（109568 行；rebase 后合并
  树复验过）。
- 目录定向：form-submission-0 全目录 256P/83F/1T。runTest2 从 1T+24
  未上报 → 16P/20F **全落定**（hang 消失）；preventdefault 双键全
  Pass；algorithm 残留 1T（t10，iframe.src 异步装载面，独立挂账）。
- **全量 join**（对 s143 基线 /tmp/zw-s143-full2.log，7994 行 =
  7038P/857F/99T）：**最终以 rebase 后合并树跑为准**（提交前 pull
  --rebase 并入并行流 6 提交，含真实 shim 代码面——EventTarget null
  listener 面 part03/04/06 + layout R5324 + encoding 流 goal 脚本，
  按 s142/s143 先例门禁全数合并树重出）——合并树 8018 行 =
  **7048P/872F/98T**（纯语料，探针已删）。**(key,state) 状态集 join
  7534/7534 零键丢失、状态位移恰 3 键全为改善、零 P→F**：
  preventdefault 键 1 Fail→Pass（submit() 放行——handler 内最后
  submit() 的 v6 entry 胜出）、键 2 Timeout→Pass（not-connected 中
  止——remove 后第二次 submit() 中止，首次导航保留）、runTest2 首案
  Timeout→Pass。解 hang 新可见 23 键（7 新 Pass + 16 新 Fail，16F 归
  泄漏面见下）。行数守恒：base-only −3（3 修复键旧行）+ only-merged
  +24（23 解 hang + csp-blocked 文件级键 NotRun→Timeout 报法漂移 1
  ——同一挂起完成回调两种报法，刀 B 时序方差非代码位移），
  7994−3+24 = 8018 精确闭合。rebase 前 pre-merge 树跑（含 6 探针键
  3P/3F）runTest2 24 键逐键与合并树相同——两树语料面一致。
- 门禁（合并树）：clippy 三 crate（zero-engine/webview/wpt-runner
  --all-targets -D warnings）零告警；engine lib **3592/9F**（总数
  3601，+1 并行流新测试）——9 名单 = 8 已知名单 + 
  test_request_signal_passthrough_r3045（归口 net-api 流不变），本
  轮 details 族 flake 回绿（pre-merge 树曾现 test_details_name_
  exclusivity_template_deep_entry、隔离复跑 Pass 实证在册间歇）；
  runner bin **257/1F**——唯一 F = trend_csv_check wpt-suites.csv 数
  据行 196 列错位（期望 15 列得 16），该行由刚并入的并行流提交
  f62bf89aa（encoding 流 31 号归档 + 36 号立项）引入，**归口该流**
  非（本切片 pre-merge 树曾 258/0F）；fmt diff 两文件均非本切片
  （js_dom_bridge.rs 7 处 net-api 区 + testharness.rs 3 处 layout 并
  行流，归口记录）；探针页验后删，语料零改动。过程注记：合并树首
  轮全量被 test-guard 瞬态误杀（runner 横幅后零测试行即报单进程
  4.31GB>4GB——重跑原样通过，与 s143 陈旧二进制误杀不同因，系统内
  存压力面采样尖峰嫌疑，复现再勘）。
- **二次 rebase 复验（最终树）**：推送时远端再前移（net-api M2-S11
  CORS preflight 收口 0f2f656f2 触碰 part01.js fetch 面 +101/−37 +
  rustls 0.23.46 bump），再变基后门禁关键项重出：shim 拼接
  node --check 过；全量 join 对 s143 基线**排除 flake 文件后状态位
  移仍恰 3 键全改善、零 P→F**，runTest2 16F+8P、preventdefault 双
  Pass 逐键复现（form 面与上轮合并树完全一致——CORS/part01 面改动
  未触及本切片语义）；本轮全量跑 textfieldselection/select-event.html
  文件级 Timeout 1 次（−38P），**定向复跑 270/270 全绿**——负载时
  序 flake（刀 B 邻域，与 csp-blocked 同型），非确定性回归，复现归
  口 runner/时序勘根。

## 新泄漏面定谳（探针 zz-probe-s144/s144b，验后删）——刀 D 立项

16 键 runTest2 新 Fail 单根：**元素状态按碰撞性 selector 字符串键存
储，跨 populateForm 代际泄漏**。

- 探针链：T1 实证 `.value` 写入泄漏（form3 input.value='YYY' →
  form4 全新 input.value 读出 'YYY'）；T5 实证属性 overlay 泄漏
  （form1 submitter.setAttribute('value','AAA') → form2 全新
  submitter.getAttribute('value') = 'AAA'）；T6 干净页复刻 runTest2
  (T,T,F,T) 形态无 v4——真实文件的 v4 全部来自前序键写入（runTest
  click listener 无条件 input.value='v2'、listener#2
  submitter.value='v4'，两文件表单结构全同 → selector 字符串碰撞）。
- 机制：insertAdjacentHTML 产物为 sel-identity（无 handle），
  `_elKey` 落 selector 字符串键；populateForm 在 body afterbegin 连
  环插入，前后代元素解析出同一 selector 字符串（s142 属性锚只覆盖
  iframe[name]/form[target]，input[name=n1] 等无唯一性保证）。脏值
  store（_inputValues）与宿主属性 overlay 双通道皆泄漏。
- runTest 4 键（pDSB:T）同根：前键遗留 n1='v2' 脏值泄入本键 entry
  （expected v1 got v2 与泄漏推演逐案吻合）。
- 修复归 stable_selector 注册制架构面（键身份化或代际作用域化），
  影响面可能超本目录，独立立项勘根（刀 D）。

## 挂账（本切片更新）

- **刀 D（新）**：selector 键元素状态跨代泄漏（上节；探针证据在
  案）。
- **刀 C 改记**：hang 已收口（本轮主刀）；残余 runTest2 16F + runTest
  4F 移交刀 D；algorithm t10「Cannot navigate (after constructing
  the entry list)」残留 1T——not-connected 检查已在位，残面 =
  iframe.src='/common/dummy.xhtml' 异步装载 + loadPromise 解析链
  （独立挂账）。
- C2 input-type-change url/email 2F（不变，基线既有，独立面）。
- 刀 B runner 多跑+首遍挂（不变）；accept-charset w1252 + `&#NNNN;`
  实体回退 + lone surrogate USV 18 案（不变）；「真实再物化」观察项
  （不变）；engine lib 9F 环境面（合并树 9 名单 = 8 已知 + R3045 归
  口 net-api 流；details 族 flake 在册间歇，pre-merge 树现/合并树
  绿）；runner trend_csv_check 行 196 列错位新面（并行流 f62bf89aa
  引入，归口该流）；其余承袭清单不变。
- **待决（须用户拍板）**：css/text 语料资源、green-2x2.png、parser
  form pointer 缓行（2026-10-23）。

## 下一步（s145 候选）

①刀 D selector 键状态泄漏面勘根（16+4 键，影响面可能超本目录）；
②accept-charset w1252 + 实体回退 + lone surrogate USV 18 案；
③algorithm t10 iframe.src 异步装载面；④刀 B runner 多跑机制勘根。
