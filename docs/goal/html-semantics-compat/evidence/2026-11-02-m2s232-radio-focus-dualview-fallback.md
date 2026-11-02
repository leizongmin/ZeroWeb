# M2 s232 — radio-focus 双视图分歧修复（键盘链动态锚宿主解析 miss 回落 activeElement 通道）

日期：2026-11-02。切片号 s232（承 s231）。

## 入场定刀位

s231 下一步候选首选：radio-focus-navigation-checked 双视图分歧修复——
勘根 s231 已闭环（Actions.send() activeElement 锚 → stub `selectorFor`
arm4 属性唯一化产 `input[checked]`（shim qSA 可见）→ 宿主 native doc
解析 miss → R347 重入队 20 次全 miss → reject；M2-S195 降级臂只在
selector 空时接管为设计盲区），本轮零新勘根直接施工。修复候选三向
（arm4 宿主可解析性复验 / IDL 写入投影 native doc / 键盘臂 miss 回落）
裁定 **(c) 键盘臂回落**：WebDriver 键盘事件目标 = 活动元素（spec 真），
降级臂非妥协而是语义本体；且不动指针步 R347 迟到物化重试语义（键盘步
迟到物化经回落同样按 spec 语义派到活动元素，无行为缺口）。

## 施工（+19 行：9 注释 + 10 代码，单文件单臂）

`tests/wpt-runner/src/testharness.rs` `apply_testdriver_command`：selector
推导后、M2-S195 降级臂前插回落块——op ∈ keydown/keyup 且
`command.selector.is_none()`（纯键盘链）且 `__zw_td_selector` 产出非空但
`page_node_ref_for_selector` 宿主解析 miss → selector 归 None 落降级臂
（live activeElement 通道 + dispatch_tab_focus）。显式 selector
（test_driver.click/send_keys 页面指定目标）与指针步语义不动。

## 验证

- solo：radio-focus-navigation-checked 5 子测试 **3×3 全 Pass**（修复前
  7/7 F 含 stash 纯态 3）。
- clippy `-p zero-wpt-runner --all-targets -D warnings` 干净；fmt 干净。
- 域 A/B：全树 (key,state) join（对比 s231-ab.log 基线，8098 键）——
  lost/gained 各恰 2 = 目标两键 Fail→Pass，**零其他状态翻转**（含
  typing/InsertText 族——降级臂跳字符落地面无回归现形）。全树
  7504P/516F/81T，对 s231 基线 7502P/518F/81T 差恰 +2P/-2F = 目标两键
  （日志 s232-ab.log；s234 轮勘误——原稿误抄 s231 分布数字）。
- 零插桩零探针（勘根 s231 闭环免仪器）。

## s231 漂移疑点闭合注记

s231 未定谳的「solo 恒 F（7/7）vs 三轮全树基线 P」漂移随本修复**结构性
闭合**：失败路径（宿主解析 miss → reject）已消除，该键现在 solo/全树
双 P（本轮 solo 3× + fulltree 双 P 实证）。漂移机制（疑 8-job 进程内
跨文件状态/分配序）不再可复现，不再追查。

## 承袭挂账（更新）

- **valueasnumber time 出界 wrap 预勘根完成**（下轮定刀候选）：setter
  time 臂（part04 `valueAsNumber` 分支）走 `new Date(vsN)`——vsN 超 JS
  Date 值域（±8.64e15 ms）落 Invalid Date → value 落空；域内负值
  （-3600000 → UTC 23:00）天然已过。浏览器语义 = 数字→time 串按 ms 对
  24h **双精度取模上卷**：node 实证 `((n%86400000)+86400000)%86400000`
  对 `2.7343337071894478e26` 得 39250944 ms → 精确复现 WPT 期望
  `"10:54:10.944"`。修复方向：time 臂 setter 构造 Date 前先 wrap（仅
  time；date/week/month 语义不同不动）。
- 散单余项签名已录（s231 账本）：email-set-value、input-type-change-value
  2F、input-type-button、inserted-or-removed、color.window.js 语料缺口。
- 其余承袭不动（keyboard 余 2F 车道挂账；table/label 簇用户门禁
  GB-20261031；parent-label 布局零盒 2F；checkable following-label flake
  族在册等）。

## 方法论教训

- **勘根闭环的挂账直接施工是零成本快路径**：s231 把机制、错误链、修复
  候选三向全记进账本，本轮从定刀到 A/B 全绿一气呵成，无探针无插桩——
  分轮切片时把勘根写透是给下轮的最好交接。
- **修复候选裁定先锚 spec 语义**：键盘事件目标 = 活动元素是 WebDriver
  spec 本体，降级臂即该语义——(c) 不是「妥协的兜底」而是「补全降级臂的
  触发面」；(a)/(b) 都在给近似层打补丁（(b) 还会加深 IDL 写属性这一
  非 spec 近似）。语义锚定让裁定时无需试错。
