# M2 s136 · 激活门 type 读通道换 lw 融合读——button-events 9/9 全绿收口（t3 F→P + t2 新键 + 文件 Timeout 解除，真翻红 0）

日期：2026-10-29 · 切片：html-semantics M2 · 前序：s135（select reset 默认层捕获）

## 入场定刀位

控制面 s136 候选①：button-events 挂起残案 +「type=button shouldn't
trigger submit or reset events」submit 误派发异根勘根。s134 定谳该
Fail 为「submit 误派发异根非本轮 reset 臂引入」，同文件级 Timeout
挂起 1（t2 无行）。

## 勘根定谳

1. **断言面盘点**：t3 Fail = `assert_unreached: type=button
   shouldn't trigger submission`（submit 误派发，非 reset）；t2
   「The reset event must be fired when click a button in reset
   status」**无输出行 = pending**——文件级 Timeout 的根因（s134
   挂账「挂起 1」即 t2）。
2. **事件序列探针**（testharness 结构断言 events 数组）：p2
   （type=reset click）收到的是 `p1:submit`——onsubmit handler 为
   p1 残留（p2 只 armReset），即 **p2 自己的 click 误走 submit
   出口**；p3（type=button click）收到 `p3:submit` 同根。
3. **回调包装 TRACE 定谳单根**：页面 JS 包装
   `__zw_get_attr_lw/__zw_get_attr` 记录 click 门读取——IDL
   setter（`btn.type=X`）**推 SetAttr mutation，R2995 lw 层立即
   可见**（`__zw_get_attr_lw(#btn)="submit"/"reset"/"button"`），
   但 click 门 sel 分支读**纯快照 `__zw_get_attr` 恒空** →
   `_subIsBtn` 命中 `BUTTON && ty==''`（空 type 默认 submit）→
   恒走 submit 出口。t1/t4 碰巧对（期望恰为 submit）。
4. **handle 分支无坑**：`__zw_get_attr_handle`（callbacks.rs:1916）
   本身 consult 当前批 mutations（latest-wins），句柄形元素不受
   影响——修复面仅 sel 分支。
5. **探针 p5 反常已解释**：corpus 同构三连击（async_test）修复前
   Pass 与 p2/p3 Fail 并存——seq 实为 submit×3 时第三次 push 触发
   `t.done()`，50ms step_timeout 里的终断言在 test 完成后执行被
   harness 无效化（假 Pass）；非激活门行为差异。corpus 九子测试
   全绿为最终定谳面。

## 施工（engine shim part04.js 2 处，无 Rust 变更）

click() API 门（R57，~5648）与 R106 dispatchEvent 臂（~5540）的
type 读取 sel 分支换 R2995 lw 融合读（consult 变更列表
latest-wins → applied view 回落，与 `getAttribute` 同语义；
`typeof __zw_get_attr_lw === 'function'` 守卫照 part01:294 惯例，
读取失败回落 `__zw_get_attr` 纯快照）；handle 分支
`__zw_get_attr_handle` 不动。type 读取正确后既有出口判定全继承：
`_subIsBtn`/`_subIsReset`（s134）两臂自动收敛。

## 验证

- 定向：button-events.html **9/9 全绿**——t3 F→P + t2 新键 Pass
  （原 pending）+ **文件级 Timeout 行解除**（9 子测试全部完成）。
- 域回归键级 join（对 /tmp/s135-full.log）：resetting-a-form 14 /
  the-form-element 108 / form-control-infrastructure 121 /
  constraints 920 / the-input-element 1644 / the-select-element
  314 / the-option-element 108 / the-textarea-element 41
  **逐键零位移**（textarea 1 条 NEW 为 dup-key 副本假阳性，两份
  同 Pass）；the-button-element 恰 t2 新键 + t3 F→P + 文件
  Timeout gone。
- 全量 join：7914/7914 行守恒，**恰 1 Fail→Pass 真翻红 0、
  new non-Pass 0**（6805P/1005F/104T → 6808P/1004F/102T）。另
  一对 NEW/GONE = the-img-element src-then-lazy-load.html 文件级
  Timeout 波动**自然回落**（s135 记录的边界波动往返对冲净零，非
  本轮改动面）；其余 3 条 NEW 为 dup-key 副本假阳性。
- engine lib：3584 用例（并行流并入 +13 后新基线）默认并行间歇
  1F——失败测试 = `test_details_name_exclusivity_template_deep_entry`
  **与 s135 定谳完全同款**（本轮默认并行 2F/8P + 4 线程 0F/2P +
  失败名抓取确认）；延续 s135 挂账（负载敏感型待独立勘根），非
  本轮逻辑引入（改动为激活门 type 读通道，details 测试无 button
  click 路径）。runner bin **256/0**（76i）；clippy（zero-engine
  --all-targets -D warnings）零告警；零 Rust 变更 fmt 不适用；
  shim 拼接（13 part）node --check 过。探针页验后删；语料零改动。

## 挂账（本切片更新）

- **button-events 文件级 Timeout 解除（挂起清零）+ t3 收口 +
  t2 新键**——s134 挂账「收窄待续（挂起 1）」终结。
- **part03 raw 链（UA 指针激活）type 读通道未勘**：本轮只修
  proxy 链两臂（click() API 门 + R106），raw 链同构嫌疑在册
  （corpus 无断言面，不扩面）。
- **engine lib 默认并行 details flake**：第 2 个数据点（s136 同
  测试再间歇），挂账独立勘根不变。
- 承袭不变：reset.html form element pointer 面（parser form
  pointer 待决项缓行）、part03 raw 链 reset 臂无还原臂、
  form-nameditem 残余 10F、form-submission-0 186F、
  execution-timing 构成面、lazy-loading 26F、meta color-scheme
  21F、写流盖章断链、part02 setAttribute 绕 notify、041 nested
  write、deep clone 非 HTML ns、css-module corpus 门（待决）、
  www2 可达性、module-import-referrer TLA、dynamic import() with
  透传、classic 外链 SRI、143 write、defer 多写完成序、接口对象
  基建面、FIXME 组、/tmp/zw-hb*.txt、FileList 缺索引访问、live
  树逗号列表 qSA 根缺口、渲染流面让位累积、s131 插入步缺口细化、
  radio 现象⑤/checked 属性增设/defaultChecked 无独立 corpus 面、
  s135 已知限制 3 条；**待决（须用户拍板）**：css/text 数据模块
  语料资源缺失、green-2x2.png 语料资源分发、parser form element
  pointer 4F 缓行（2026-10-23）。

## 过程教训

- **Pass 行不带 assert 消息**：runner 只在 Fail 行输出消息——
  埋点转储用「故意断言失败 + 数据进 expected 值」通道带出；先
  验证通道再写埋点，别用 console.log（runner 吞）。
- **回调包装是零改动 TRACE 通道**：宿主回调经 globalThis 运行时
  解析（shim 内自由变量查找），页面 JS 包装即可记录门函数读取，
  不动 shim 拿到单根级证据——比 shim 内埋点快一轮编译。
- **async_test 的 done 后断言被无效化**：探针终断言放
  step_timeout 时，若同步路径已 `t.done()`，终断言可能整体失效
  成假 Pass——探针的 Pass 也要过一遍「断言真的执行了吗」再信。
- **IDL 属性写不等于快照可见**：运行时 IDL setter 推 mutation
  但 apply 轮末才落快照——激活/判定类门函数读属性一律用融合读
  （lw）或 consult 批（handle 形），纯快照通道只适用于解析期
  稳定属性。s135 select 读链三层序同款刀法。
