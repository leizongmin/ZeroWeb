# DC-2 · 批八：html/syntax 解析簇 html5lib 语料接入（.dat 61 文件 + 投递改写属性透传）

2026-11-21。html5lib 驱动三件套（html5lib_url/write/write_single.html，170 变体键）
此前整批落「html5lib-runner setup」synthetic Fail（runType=undefined + fetch 404
双缺口），本批双缺口齐补：parsing/resources 61 个 .dat（pin 315976933 全集）显式
fetch_raw 拉取 + runner 投递改写透传 `data-run-type`。release runner 实测套件
Pass 67,269 → 67,387（**+118**）、Fail 10,384 → 10,656（+272）、Timeout 138 → 286
（+148），1,049 键零回归（所有差异为纯新增或键内转换）；对账口径净 **+118 Pass**，
覆盖分母键 **+170**。

## 变更（三文件）

1. `tests/wpt-runner/scripts/goals/61-wptfyi-html-syntax-dat.sh`（新）——
   parsing/resources 61 个 .dat 逐文件 fetch_raw（数据资产非 .html/.js 扩展，
   fetch_dir_html 扩展名分流不覆盖；清单按 pin 固定，逐文件幂等）。
2. `Makefile`——新增 fetch-wpt-html-syntax-dat target，链入
   testharness-html-syntax 依赖。
3. `tests/wpt-runner/src/testharness.rs`——投递改写 `open_tag_passthrough_attrs`：
   既有保真族与调度消费面之外的原标签属性逐字透传（data-run-type 面板——test.js
   读 `document.currentScript.dataset.runType`，丢弃即整批 synthetic Fail）；投递
   产物标记防御性列入名单（幂等重入不双发）。引号态切属性 span（值内 `>`/空白
   不封口）、自闭形孤立 `/` 折返消费。单元测试
   `inline_delivery_passthrough_attrs_keeps_data_run_type`（guard 包裹实测过）。

## 驱动执行形态（test.js 机制）

`<script src="resources/test.js" data-run-type>` 按 variant 查询参数
fetch(`resources/${file}.dat`)；`#document-fragment` 行恒走 innerHTML（根 = 元素，
normalize 在），document 行走页面 run-type（url = Blob URL iframe 载入 /
write = contentDocument.open+write）。

## 结果（release runner，test-guard 包裹，全量 before/after）

| 面 | 变更前 | 变更后 |
|---|---:|---:|
| Pass | 67,269 | 67,387（+118） |
| Fail | 10,384 | 10,656（+272） |
| Timeout | 138 | 286（+148） |
| 键 | 1,049 | 1,049（+0） |

- 零回归：1,049 共享键 0 键回退。非 html5lib 仅 3 键序列变化且全部纯 Pass 新增
  ——html/dom/render-blocking *.optional.html 6→14 / 8→16 / 4→8（+20）：投递
  改写此前丢弃 `blocking` 属性，透传后 render-blocking 语义恢复，搭车收账。
- html5lib 170 键真执行：540 实行（Pass 98 / Fail 442，全部 url 变体）+ 148 键
  页级 Timeout。url 变体 22 键完整跑完（完整键实行数 = 分母行数逐字对齐：
  foreign-fragment 66/66、tests4 9/9）、38 键中截断（特定输入行 onload 不再
  触发）；write/write_single 110 键全数行 1 挂起。
- Pass 98 全部为 fragment 行（innerHTML 路径真树比）——fragment 解析/序列化
  首次有真信号。

## 对账口径（summary-chrome @4b26db8978）

170 执行变体键分母 5,417 子测试（chrome 过 5,376）：此前整批 0 执行未入账，
本批后映射净 **+118 Pass**（98 html5lib + 20 render-blocking）；覆盖分母键
+170（parsing 通道 245 键中执行 190）。

## 分母分类账（parsing + fragments 262 键全量对平）

| 分类 | 键 | 子测试 | chrome 过 |
|---|---:|---:|---:|
| 已执行（本批面） | 190 | 7,773 | 7,731 |
| pin 缺失（315976933 无此文件） | 26 | 53 | 49 |
| scripted_foster01 驱动变体（.dat 晚于 pin） | 3 | 6 | 6 |
| 无扫描链（pin 在、非本套件键形） | 43 | 368 | 367 |

- 对平核验：190 + 26 + 3 + 43 = 262 键；7,773 + 53 + 6 + 368 = 8,200 子测试
  （parsing 8,166 + fragments 34）。
- pin 缺失 26 键明细：adoption-agency-reparenting 族 7 + custom-element-
  reparenting 族 3 + crashtests 3（0 子测试）+ foster-parenting 面 3 + parsing
  顶层散键 6 + parsing-html-fragments 3 + parsing-xml-fragments 1——上游晚于
  pin 新增，按批六先例（语料单 pin 不混版）不拉，随周跑另报。

## 战役图（442 F + 148 TO，刀位储备）

- **Node.normalize 类分裂（363 F）**——`_zwParseEl.prototype.normalize` 已实现
  （part02.js R189，委托 mutTree），url 变体 document 行根 = Blob-URL iframe
  `contentDocument`（另一文档类）无 normalize，serializeTree 首行即抛 → document
  行整批 Fail。刀位 = Node.normalize 上移共享 Node 原型
  （https://dom.spec.whatwg.org/#dom-node-normalize）——js_dom_shim 域与零-web
  流 WIP 同面，待碰头（同 master 卡点）。
- **document.write 行 1 挂起（110 键 TO）**——write/write_single 经
  `iframe.contentDocument.open/write/close` 写入任意 HTML 全数挂起——engine
  document open/write 管道域。
- **Blob-URL 文档载入挂起（38 键 TO）**——url 变体特定输入行（adoption-agency
  形居多）iframe.onload 不再触发，疑似解析器死循环；完整键证据表明载入前 8 行
  正常（Fail 也走完 onload → serializeTree 抛错路径）。
- **fragment 行真树差（79 F）**——innerHTML 路径 assert_equals 族（foreign
  content / 表单 container / 表格形），真解析差面，非缺口阻断。

## 套件时长注记

套件墙钟 93 min：148 页级 TO 各耗 ~30s 键级超时预算 ≈ 74 min。挂起面两刀
（normalize、document.write）落地后该面大部分消解，周跑预算需记此数。

[逐文件 JSON](2026-11-21-dc2-batch8-syntax-html5lib-dat.json) ·
[批七证据](2026-11-21-dc2-batch7-canvas-offscreen-main-faces.md)
