# M2-s86 分诊轮：scripting-1 开片分诊 + moving-between-documents 多文档面出账（runner 一刀）

日期：2026-10-25 ｜ 轮次：s86（开片分诊 + runner skip 规则一刀）｜ 前序：[s85](2026-10-25-m2s85-closedby-rootcause-falsified.md)

## 1. 入场核账：s85 下一步继承的挂账已被 s79/s80 消费

s85 记「下一步 testdriver-Actions 支持面」过时：s79 已证伪 s78 的
「hash-name-dynamic 5F=testdriver-Actions 面」注记（runner 自带 test_driver
Actions stub，点击链可达），s80 跨批 selector 迁移 journal 已收口
（3P/5F→8P/0F）。本轮 FILTER=hash-name-reference-dynamic 复跑 **8/8 Pass**
复证一致。连续三轮（s84 根因、s78 注记、s85 下一步）证明同一教训：
**挂账项开工前必须当拍复核红图**。

F 族 name-attribute 3F（s51 时代挂账）同样已被 s54 分诊 + 后续轮收口至
18P/0F。720-stretch 渲染展开 22F 为 layout 集成面（跨流协调）。由此按
DC-2 挂账序转 **scripting-1 开片**（2026-10-24 s67 轮记「embedded-content /
scripting-1 / microdata 按挂账序开片」的首个未消费大项；microdata 上游
404 空域 M1 已销项）。

## 2. scripting-1 当拍红图（make 目标 + FILTER=scripting-1，47min 全域跑）

当拍 **169P/335F/86T**（590 子测试；M1 基线 78P/247F/70T/395——语料 +195
后 P +91，通过率 19.7%→28.6%；模块底座已由 script-sandbox es_module +
R3093 动态 import fetch 链建起，M1「模块面页面级不执行」主簇已松动）。

失败族地图（335F）：

| 族 | F | 代表信号 |
|---|---|---|
| execution-timing | 94 | `scheduler: stylesheets blocking scripts` 12F 单簇——样式表阻塞脚本语义未实现（spec script scheduling 段） |
| module 基面 | 72 | instantiation-error 8F / evaluation-error 5F / referrer-policy-for-descendants 9F / single-evaluation / compilation-error |
| dynamic-import | 49 | `__zw_dynamic_import.call(...) must reject` 24F 单根——不可解析 specifier 须拒未拒；credentials mode 12F |
| css-module / json-module / text-module | 21/11/8 | 模块类型变体（text-module src 误加载、json 模块 MIME/导出面） |
| the-script-element 基面 | 65 | script-for-event 13F（legacy `for=`/`event=`）、charset 解码 9F、script-text setter、emptyish、supports、type-whitespace |
| import-meta | 4 | importmap 解析面 |

Timeout 地图（86T）：moving-between-documents 52 + ordering 1、
dynamic-import 11、execution-timing 8、module 5、base 4、microtasks 3、
fetch-src 2。

## 3. 本轮一刀：moving-between-documents 多文档面出账（R5286，runner skip 规则）

53 案（目录 52 + ordering 1 除外均为 helper 引用案）经
`resources/moving-between-documents-helper.js` 运行期
`createElement("iframe")` 开第二文档——主源无 iframe 字面，既有内容规则
（`<iframe`/`createElement('iframe')`）不可见，全部 **1P+1T**（首子测试过、
其后 iframe load 等待挂满超时网），单文档 runner 不可执行面，归
navigation-compat M3 多 frame 文档管线门控（the-iframe-element 排除先例）。

修复：`html_semantics_case_skipped` 增 helper 标记臂
（`moving-between-documents-helper.js` 精准命中，FIXME(navigation M3) 注记）
+ 契约单测（helper 案 skip / ordering 案不误伤）。

**验证**：FILTER=scripting-1 复跑 **169P/335F/86T → 117P/335F/34T**
（−52P/−52T 全落 helper 案，F 逐位不变，ordering 五案与
`moving-between-documents-during-evaluation.html`（createHTMLDocument 单文
档真实红面）零误伤）；墙钟 47min→24min（53×~30s 超时空转出清）。域账自此
按 **117P/335F/34T** 记。

## 4. 下一切片（按单根杠杆序，engine shim/script-sandbox 本流工作面）

1. **dynamic-import must-reject**（24F 单根 + credentials 12F）——
   script-sandbox es_module `__zw_dynamic_import` 拒绝语义（不可解析
   specifier TypeError）+ fetch credentials mode 传播。
2. **execution-timing stylesheets-blocking-scripts**（12F 单簇 + 毗邻）——
   样式表阻塞脚本调度语义（shim script scheduler × stylesheet ready 面）。
3. **the-script-element 基面批**（65F 分散小面）——script-for-event legacy
   门、charset 解码、script-text setter 等。
4. module 基面尾片（instantiation-error/evaluation-error 传播、referrer
   policy 9F）与 css/json/text 模块变体（40F）随后续轮次。

## 5. 范围与记账

- 生产面改动仅 `tests/wpt-runner/src/testharness_html_semantics.rs` 一处
  skip 规则臂 + 同文件契约单测；engine/webview/shim 零改动。
- 域账出账 53 案记「多文档面出账（navigation M3 门控）」，非通过率改善。
- 语料不可改维持（helper 为上游资产）。

## 6. 提交门禁记录：88 绿腿 + 两个预存红（均非本流改动可达）

`make test` 全量：88 腿绿后止于两处**预存红**，归因与处置（共享面碰头
纪律：记录 + 通知，不硬解）：

1. **quickjs `zero-renderer --lib` 红腿**：
   `page_scripts::tests::mutation_arm_does_not_apply_across_document_reset_
   window` 0.64s 确定性复现（同测 v8 变体绿；340P/1F）。本流依赖隔离实证：
   `cargo tree -p zero-renderer | grep wpt-runner` → 0。疑因 commit
   `48937037e`（2026-10-24 rAF 帧驱动默认开启，touch page_scripts.rs），
   文件现归 perf/renderer 流（P104 `195ab149a`）主动施工面，测试消息自述
   「时序假设失效须重审」——须所有方重审，不代修。
2. **zero-webview clippy::map_entry 硬错**：
   `crates/webview/src/video_registry.rs:262`（`contains_key`+`insert`），
   由 commit `9c6652fa3`（2026-10-25 bg02-media 基线 v2，media 流）引入，
   语义为刻意的重复登记幂等（注释自述保留既有 entry），但 `-D warnings`
   下挂 clippy 门禁——归 media 流修复（entry API 等价改写或 allow 注记）。

本流直接门禁全绿：runner 契约单测
`moving_between_documents_helper_cases_are_skipped ... ok`、runner 域
clippy 面、fmt 均通过。
