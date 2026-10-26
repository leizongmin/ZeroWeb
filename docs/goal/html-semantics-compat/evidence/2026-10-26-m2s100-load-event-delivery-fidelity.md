# M2 切片 100：经典脚本 load/error 事件投递保真（runner 内联通道 + .py fixture 仿真 + attr 值面）

日期：2026-10-26。范围：`tests/wpt-runner/src/testharness.rs`（inline_local_scripts 投递保真 + fetch 通道 .py fixture）+ `crates/engine/src/pipeline/extract.rs`（attr_from_open_tag 值面保真）。上轮：[M2 切片 99](2026-10-26-m2s99-script-bool-idl-has-visibility.md)。本切片施工跨 429 断点：断点前已完成后半程验证复跑（终扫与断点前扫描 bitwise 一致）。

## 1. 入场核账与定谳

- s99 基线（scripting-1 口径 574 行）：314P/227F/15T + 18 NotRun。
- 挂账族两面根因：①投递面——parser 外链被 `inline_local_scripts` 内联改写时 `onload`/`onerror` 内容属性被丢弃（s94 保真只留 nonce/nomodule/type），且内联后无取回环节，spec「成功执行 fire load / 取回失败 fire error」随之丢失 → nomodule-set-on-synchronously「must run」`assert_true(loaded)` 假红、load-error-events-1 parser 臂 on* 全盲；②服务面——`load-error-events.py` 是 wptserve CGI 端点，语料 pull 面不含 .py，dynamic appendChild 臂取回落通用缺文件 404 → 200 组 error 路径误触发假红。
- 页面结构（语料 le1）：11 臂 = parser defer/sync/async × 200/404 + dynamic no-defer/async × 200/404 + no-src 内联臂；200/404 由 `?test=` 查询串决定（`*_load` → 200）。

## 2. 接线（三刀）

- ①inline_local_scripts 投递保真：`script_event_handler_attr` 提取并回写 onload/onerror（`&`→`&amp;`、`"`→`&quot;` 转义同 nonce 先例）；200 投递原脚本体 + load 投递尾，非 200 只投 error 投递尾（脚本体不执行——上游 404 体亦不执行）；投递尾经 `document.currentScript.dispatchEvent` 派发（on* 内容属性 handler 走 shim R2934 编译链，addEventListener 侦听者可达；事件不冒泡，window load 生命周期不受扰）。src 查询串先剥离再归一（`?test=` 由 fixture 面消费）。
- ②fetch 通道 .py fixture 仿真（`wpt_load_error_events_response`，fetch handler 与内联预处理共用）：`test` 参数 `^[a-zA-Z0-9_]+$` 门（缺席/非法 → 400 空体）；含 `_load` → 200 + `<test>.executed = true`；否则 → 404 + assert_unreached 体（本引擎侧 status 拒 eval）。路径精确匹配 `the-script-element/resources/load-error-events.py`；module 族变体路径不匹配（语义各异，维持通用 404，归 module 尾片）。
- ③extract `attr_from_open_tag` 值面从**原文**切片：旧版从 ASCII 小写定位镜像切值，`onLoad(t1)` → `onload(t1)` 失效、base64 nonce 同面受损；lower 镜像等字节长，`lower.len() - val.len()` 偏移两侧通用（多字节 UTF-8 序列 ASCII 小写化不变）。调用方四面（style nonce CSP、runner 内联 nonce/type、新增 handler 面）均为保真受益，无依赖小写化的消费方。

## 3. 验证

- 单测：runner bin **228/228**（+2：fixture 200/404/400/路径精确匹配四态钉 + 内联保真/投递尾形态钉）；zero-engine lib **3530/3530**。
- 定向：FILTER=load-error-events——le1 **11/11 全 Pass**（parser 三形态 × 200/404 + dynamic 四臂 + no-src 臂，页完成无 Timeout）、text-module 1F 为 s99 既有（module .py 变体，非本刀回归）；FILTER=nomodule——nomodule-set-on-synchronously **1F→1P**、nomodule-async 2/2 保持、external-module 1F 承袭（module 尾片）。
- 全子树终扫（scripting-1 口径 578 行）：**325P/221F/14T + 18 NotRun**，与断点前首轮扫描逐行一致（确定性双跑）；diff vs s99 恰 3 页 F/T→P（le1 2F+1T→11P 全清、module/load-error-events 2F→2P——parser module 臂 on* 保真连带、nomodule-sync 1F→1P）、**零 P→非P**（/tmp/n99.txt vs /tmp/n100s.txt）。
- execution-timing/122 行缺行归因：teardown 粒度方差（s93 先例）——单跑仍报既有 `scheduler: altering the type attribute` Fail（s95 同面），无隐藏翻绿/翻红。
- fmt clean；clippy `-D warnings`（engine + wpt-runner；systemd scope 本环境不可用，ulimit 24G + timeout shell 兜底，沿 s91 口径）；reftest/product-smoke 未跑（无 render/layout 面）。

## 4. 范围与挂账

- 触碰面：testharness.rs（fetch arm + fixture 函数 + inline 保真/投递尾 + 2 单测）、extract.rs（attr 值切片）；shim 零变更；语料不可改维持。
- load-error-events 族清账：classic 全臂 + nomodule-sync 销账；残余 module .py 变体页（text-module 1F、css-module/json-module 同族）归 module 尾片（.py 执行器前置）。
- 承袭：module 编译错误事件面（nomodule-set-on-external-module-script 1F）、116 frameset 文档语义、063 Timeout、charset 12F（.py）、svg-script-fragment-clone 3F、FIXME 组（bufTop 漂移/setAttribute('src')/type 门/children-changed 其余入口）、接口对象基建面。
