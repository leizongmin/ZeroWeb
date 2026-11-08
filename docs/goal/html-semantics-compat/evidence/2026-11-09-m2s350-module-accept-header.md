# M2 切片 350：json/text 数据模块 Accept 头端点臂

**日期**: 2026-11-09
**目标对**: `json-module/http-accept-header.sub.html` + `text-module/http-accept-header.sub.html`（s349 基线 19F 中双 F——仅 Dynamic 子测报红）
**结果**: 双文件 4 键全 P（2 键 F→P + 2 键新增 P）；语料口径 **19F→17F**

## 上游语义（勘根）

[fetch a single module script](https://html.spec.whatwg.org/multipage/webappapis.html#fetch-a-single-module-script) 的 JSON modules 集成：json 型模块取回请求 Accept 头钉死 `application/json,*/*;q=0.5`；text modules 集成草案同形 `text/plain,*/*;q=0.5`。语料经 `http-accept-header-checker.py` 端点（上游 wptserve handler，**不在语料**）回显请求 Accept：json → `{"accept": "<值>"}` JSON module、text → 裸值文本 module；静态 `import x from "./…checker.py" with {type}` 与动态 `import("./…checker.py", { with: { type } })` 两臂同断言值。

## 旧红与根因

- 域跑双文件仅 Dynamic 子测 Fail：`TypeError: Failed to fetch module: './http-accept-header-checker.py'`；Static 子测**缺席日志**（静态 import 链断 → 首模块求值失败，test() 未注册）。
- 根因：端点资产缺失（同 s227 缺资产形态——上游 .py handler 不随语料分发）。本引擎静态/动态 `with {type}` 属性面（s107/s250 既有）与模块类型图收集均完好——s349 域跑 integrity/charset/referrer-policies 全 P 佐证；唯一缺口 = 端点回体仿真。

## 施工（runner 单面，engine 零改动）

`wpt_data_script_fetcher_with_stash` 新增端点臂（置于 `__zw_mtype` intent 臂**前**——s267 规则：.py 非 json/scripty MIME，intent 臂接管即 Err）：

- `the-script-element/json-module/http-accept-header-checker.py` → `{"accept": "application/json,*/*;q=0.5"}`；
- `the-script-element/text-module/http-accept-header-checker.py` → `text/plain,*/*;q=0.5`。

Accept 值 = 按端点目录归属的模块类型锚定的 spec 常量——fetcher 契约 `(page_url, src)` 无头通道，runner 车道以目录为类型锚（text 型取回报文另带 `__zw_mtype=text` 标记交叉核对；json 面目录即锚），**s268 compute_module_referer 服务端求值同法**。单测 `accept_header_checker_serves_module_type_anchors_s350`（两路径回体逐字节钉死）。

## Blast radius 普查

- 端点路径唯一（两 corpus 目录各一个 checker.py），arm 前置不扰既有臂（referrer-checker/serve-custom-response/module-fetch-tester/delayed-modulescript 路径互斥）。
- engine/webview/renderer 零改动——静态图收集、动态 import 属性校验、text/json 模块合成面全部复用既有通道。
- 真实 HTTP 车道（renderer/tab）Accept 头建模**不在本切片**（fetcher 契约无头通道，语料无案面）——挂账。

## 验证

- **solo 双文件 4 键全 P**（`FILTER=http-accept-header`）：json Static+Dynamic、text Static+Dynamic。
- **单测**：runner `--bins` **302P/0F**（+1 新测；余 +3 为并行 M3 流 s346/6a 承接）。
- **clippy** zero-wpt-runner `--all-targets -D warnings` 0 告警；`cargo fmt --all -- --check` 净。
- **域 join**（LC_ALL=C + `(key,state)` sort -u）：A=s349-b **19F/1100P/8T** vs B=s350-b **17F/1104P/8T**——det F→P=2（双文件 Dynamic 键逐键吻合）+新增 P 键=2（双文件 Static 键，A 面缺席非退面）+P→F=∅+F added-only ∅+T=8 同集（双向 comm ∅）。

## 挂账 / 下一步

- 真实 HTTP 车道模块 fetch Accept 头建模（fetcher 契约扩展，语料外）。
- import.defer 三案（module/dynamic-import/*-deferred-evaluation.tentative）= JS 语言特性面（V8 harmony/解析+deferred module 评估图），独立切片勘根。
- 域残余 17F 逐案勘根（execution-timing 6 案长尾、css-module relative-urls 2 案等）；T=8 长尾在册。
- testharness.rs 与并行 M3 流（SW fetcher）同文件不同区——已 pull--rebase 顺序合流，无冲突。

## 日志

- evidence/logs/s350-b-script-element.log（域跑 B 面）；s349 基线 A 面：s349-b-script-element.log（既有）
