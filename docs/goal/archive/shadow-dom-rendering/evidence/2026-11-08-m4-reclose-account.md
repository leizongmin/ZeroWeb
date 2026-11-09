# M4 复测收口账（第二会话独立复核）——净树余腿复核 + DC-4 数值门逐批 + B6 门值回填 + s331 机制层深挖

**日期**: 2026-11-08
**切片**: 45 号 M4（006 翻绿追加轮之后的收口轮）
**性质**: 独立会话第二源复核账——与同日
[M4 收口主账](2026-11-08-m4-closeout-numbers.md)（legs10/链 A/链 B/legs11b）
双源；结论全部同判。净新增：§1 workspace 腿 standalone-binary 前置口径、
§3 同二进制两跑逐位复现、§5 s331 td_debug 机制层实证（selector 身份
层钉层）。数字账以主账 §7 门禁链总账为准，本账为双源印证与机制深挖。
**基线**: 净树 `origin/main=03900b511` + 本轮 golden 资产 `49aaba12d`（产品树与追加轮 legs8/legs9 验证树零 delta）
**结论**: DC-1~DC-4 全部达成。DC-4 = 池内 19/20 + 1 具名范围外
（vw-units，iframe 载体面）逐批不回落；rendering-compat 全量 **725/725
rc=0**（本 clone 一手复跑）；头条 006 ✓0.07% 不回落；B6 门值实测回填
15P/86F 两跑复现。对端移交项 s331 复核 18F→18F 零改善——勘根改判
（见 §5），残余缺口归 selector 身份/指针寻址面（既有 hit-test 具名
挂账族近邻），不阻 DC-4。

## §1 净树余腿复核（make test 全腿，规范入口 + 逐腿补跑）

规范 `make guarded-test`（fmt 前置 + standalone binaries 前置齐全）跑至
workspace 腿后因该腿 fail-fast 中止；余腿按同阈值口径逐腿补跑
（test-guard --compile-first 包裹，`ZW_GUARD_RUNNER_PREFIX="xvfb-run -a"`
浏览器腿齐全）：

| 腿 | 结果 | 判定 |
|---|---|---|
| fmt --check | ✓（入口第一步通过） | ✓ |
| standalone binaries 构建 | ✓ | ✓ |
| workspace v8 | **3815P/6F** | 6F 与在册 HEAD 环境失败名单全同名
（img_runtime_deferred_settle_r5275 / svg_img_settle_density_r5277 /
test_fetch_filtered_response_type_matrix /
test_iframe_service_worker_messages_dispatch_to_iframe_container /
test_t8k_websocket_error_nonterminal / test_t8k_websocket_event_flow_open_message_close）——零 P→F。控制面挂账「workspace 腿带 standalone binary 构建前置重跑」收口 |
| quickjs clippy（13 crate -D warnings） | ✓（并行腿日志零 error；净树缓存复跑 Finished 无警告） | ✓ |
| renderer bin / renderer lib | legs9 同产品树账复用（此后产品树零 delta：仅 docs + golden 资产两 commit） | ✓ 复用 |
| browser bin（xvfb 前缀） | 主 harness 742P/1F（`default_actions_work_without_javascript`——R5384 stash 实证干净 HEAD 恒红，在册对端 zero-browser 账）+ 集成 9 二进制全绿含 **cdp_focus_governance 6P/0F**（M1 在册形状；R3b 挂账「净树复核」收口） | ✓ |
| quickjs crates | **775P/1F**：image_batch_e2e::renderer_publish_batch_image_fetch_reaches_concurrent_in_flight——断言 37≠36「多取回一张图」（并发重复取图计数抖动）；隔离复跑 **ok**（R5385 `same_origin_image_burst` 同族 discharge 先例） | ✓ |
| GPU 探测 + adapter-only | adapter 可用：gpu::renderer:: 96P/0F ✓、surface gpu/cpu 一致性 1P/0F ✓ | ✓ |

## §2 DC-4 数值门逐批账（`make reftest-upstream FILTER=shadow-dom`，本 clone 实测）

矩阵 **34 案 33P/1F**（strict 口径；其余 19 案为 FILTER 子串附带命中的
selection/css-counter-styles 邻域，全绿）：

| 批 | 门值 | 实测 | 注 |
|---|---|---|---|
| B1 slot 回退/分配 8 案 | 8/8 | **8/8 ✓** | 004 ✓（R4 remeasure 臂）、006 ✓0.07%（追加轮 append sel 项臂） |
| B2 布局失效/静态分配 5 案 | 5/5 | **5/5 ✓** | 3 案空白合法形态恒白守卫（近纯色可疑清单在册，合法） |
| B3 样式失效/继承 3 案 | 3/3 | **2/3** | vw-units ✗（2.08%）——M1 §4 定谳 iframe 载体面（runner 无嵌套文档几何管道），范围外具名挂账 |
| B4 `:host` 伪类族 3 案 | 3/3 | **3/3 ✓** | M1 翻绿维持 |
| B5 交互耦合 1 案 | 1/1 | **1/1 ✓** | nested-hover 静态面通过口径不变（真交互面具名挂账） |
| 邻域 selection/shadow-dom 9 案 | 9/9 | **9/9 ✓** | 零回落；另 crash-print 1/1、css-counter-styles 4/4 池外全绿 |

基线（M0 实测 16P/4F）→ 收口 19/20 + 1 具名范围外：004/006/focus-1 翻绿
维持，vw-units 恒红在册（载体面），逐批不回落 ✓。头条 006 solo ✓0.07%
不回落 ✓。

**池外注记**：追加轮矩阵所记 35 案 2F 含 `display-contents-shadow-dom-1`
（1.97%）——该文件在规范语料库（本 clone fetch 态）不存在、无 git 历史
（wpt-data 为 gitignored 拉取工件；对端 clone 调查期 ad hoc fetch 面）。
主账判读为「他域误中非本池分母（css/display 面）」与本账「FILTER 子串
附带命中，不在 20 案地址池」同判；其红相（display:contents × shadow
组合）留池外观察注记，不属 DC-4 分母。

## §3 B6 门值回填（testharness-shadow-dom 补充池 10 案 101 子测）

**实测 15P/86F，与基线记录总数逐位一致；同二进制两跑复现逐位一致**
（offsetParent-across 4P/18F、offsetTop 2P/1F、focus-within 1P、
scroll-restore 1P、scroll-to-fragment 7P/3F、caretPosition 0/14、
elementFromPoint 0/41、HighlightRegistry 0/4、offsetX/Y 0/3、accesskey 0/2）。
JSON 留档 `.acceptance/b6-m4-gate.json`（gitignored 面，两跑各一）。

- 几何命中簇（elementFromPoint/caretPosition/Highlight/offsetX-Y）零过 =
  runner 无输入注入/布局几何查询管道的既有预期口径，capability 挂账不变。
- accesskey helper `pressAccessKey` ReferenceError——静态等价挂账不变。
- 案内分布采主账对基线日志的订正（offsetParent 簇 18F/4P、
  scroll-to-fragment 7P/3F，R3b evidence 原誊 12F/10P、8P/2F 系誊写误差）
  ——本账两跑 JSON 与订正后分布一致，轮转以主账 §3 订正为细则基线。

## §4 rendering-compat 零回归 + 全量账

`make reftest` 全量 **rc=0，725/725（100%）零回归**——本 clone 一手复跑
（产品树与追加轮 legs8 验证树逐位同；golden 资产 commit 不触产品）。
workspace/quickjs/browser/renderer/GPU 腿账见 §1。

## §5 对端移交项 s331（dialog-popover-closedby-complex）勘根改判

移交口径（04fb3e133）：「s331 closedby-complex 18F 勘根 = R3 动态 slot
分配缺位，归 M3 变异应用臂 resolve_slots 挂接批一并验收」。R3/R3b/
append-sel 全落地后本净树复核：**18F→18F 零改善**（规范
`ZW_TESTHARNESS_RAF_FRAME_DRIVEN=1` 环境实测）。归因链（探针 +
td_debug 指针解析实证，探针页用后即删不入语料）：

1. **非 slot 变体 20/20 全过**——点击合成、light-dismiss、modal/modeless
   机制面健康；失败面精确锁定 slot 结构变体。
2. **slot 分配/渲染/布局面健康**：探针钉链——shim proxy 带 zwsh 复合
   selector（`zwsh6:#host1|#dA`）✓；读通道（getAttribute/textContent）
   经同一 selector 漏斗正常 ✓；帧对齐后 gBCR rect 表返回精确几何
   （`50,100,200,100` = 声明 fixed 位置）✓。006 翻绿 + B1 8/8 同证。
3. **残余缺口 = 指针寻址的 selector 身份层**：`ZW_TD_DEBUG=1` 实证
   resolve_pointer_target 对 slot 结构落点铸出**light 形态 CSS 选择器**
   （`dialog.dialogB > div:nth-child(1)`——扁平树父链、非 zwsh 命名
   空间），引擎 light 树解析不着 → 派发寻址失败 → dialog 不 open。
   s176 zwsh 通道本身健康（探针双向验证）；缺口在部分 shim 铸造路径
   对 slotted/shadow 嵌套内容产出扁平父链 light 选择器、未经 zwsh
   命名空间收敛。

**改判结论**：R3 承接面（分配新鲜度）已修且与本案例无因果关系；残余
18F 归 **hit-test/点击寻址 selector 身份面**——既有具名挂账
「hit-test 四面与 selection/Highlight registry 面交叠按通道归因定档」
的近邻；案例语料本体属对端 html-semantics goal 的 dialog/popover 账。
修复面（shim selector 铸造路径收敛 + 解析器信任策略）是共享面变更
（selector identity 触 WC/读面 A/B 广谱），留待专项切片按碰头条款
双方留痕后接，不并入本 goal 收口。主账同日净态重跑同判 18F 原样、
归因 hit-test/几何查询通道（closeout §8），两源独立收敛；本账 td_debug
实证把该通道缺口钉细到 selector 身份层（几何数据在、寻址通道断），
后续切片从该层入手。

## §6 语料与探针卫生

- 本轮探针页（zw-s331-probe.html 五版迭代）置于 wpt-data 仅运行期，
  账目落档后已删除；wpt-data gitignored 核实 ✓。DC-4 矩阵分母不受影响
  （html/semantics 语料域，非 shadow-dom 池）。
- B6 门值 JSON 两跑留 `.acceptance/`（gitignored），不尾随提交。

## §7 关联

- M4 收口主账（门禁链总账与 B6 案内订正以此为准）：
  [closeout-numbers](2026-11-08-m4-closeout-numbers.md)。
- 006 翻绿根因修复与门禁账：
  [追加轮 evidence](2026-11-08-m3-append-sel-channel.md)；
  R3b/R3/R4 账见各自 evidence。
- s331 移交原文：归档控制面 DC-4 段「对端移交」条目（04fb3e133 在册）。
- 残余缺口（selector 身份面）后续切片入口：§5 改判结论 +
  `resolve_pointer_target` s176 注释（testharness.rs:13526 附近）+
  part01.js s330 declarative 锚注释。
