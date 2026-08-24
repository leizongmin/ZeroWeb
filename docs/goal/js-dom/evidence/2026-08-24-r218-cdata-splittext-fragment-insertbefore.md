# R218 Evidence — CDATA splitText + detached fragment insertBefore

**日期**: 2026-08-24
**切片**: M4——R218(b) insertNode 916P 基线聚类的方法面两簇（12,x 栈溢出经 3 种探针复刻 clean——真 harness sim 流程未复现，转 R219 页面上下文插桩）
**改动面**: `part03.js`（splitText nt=4 补入 + CDATA 尾节点保型 + detached fragment insertBefore）+ `part21.rs`（回归单测）

## 一、916P 基线聚类

- `range.startContainer.splitText is not a function` 34F（6,x——CDATA startContainer：
  R209 的 splitText 只挂 nt=3）
- `parent_.insertBefore is not a function` 17F（38,x——range 落在 docfrag 容器：
  detached doc fragment 缺 insertBefore）
- `assert_throws_dom HRE` 86F + Maximum call stack 72F（12,x——三种探针复刻
  （单形态/双 iframe/50 轮循环）全 clean，真 harness 的 sim 流程未复现——R219
  页面上下文插桩）

## 二、实现三件

1. **splitText nt=4 补入**（spec CDATASection : Text——splitText 经继承可达）
2. **CDATA split 尾节点保型**：旧版尾节点恒 createTextNode 使 CDATA split 尾变
   Text；nt=4 时经源 doc createCDATASection 重建
3. **detached fragment insertBefore**（spec `dom-node-pre-insert`）：ref=null
   等价 append + ref 前插 + fragment 展平（递归 insertBefore 保持 ref 语义）

## 三、验证链

- **单文件**：insertNode **916P→952P（+36）**；surround 865→**873P（+8，
  CDATA splitText 连带）**；delete 56 / extract 98 / clone 149 不变
- **全量（polyfill）**：R217 基线 51589P/3449F/21T → **51631P/3405F/23T
  （净 +42P）**——F2P 57（insertNode 46 + surround 11）/ P2F 13（形态重分布）
- **全量（native 对照）**：**51634P/3405F/20T**——flips 仅 3（全部为既存
  harness flaky 的 Timeout↔Pass：query-target-in-load-event / EventListener-
  incumbent-global ×2）
- **engine 单测**：2358 全绿（新增
  `test_cdata_splittext_and_fragment_insertbefore_r218`——CDATA split 数据/保型/
  越界 + fragment ref 前插/尾插/展平五断言组）
- **fmt / clippy**：零 diff / 零警告
- **make test**：（见 master.md R218 行）

## 四、R219 靶点

1. 12,x Maximum call stack 72F：页面上下文插桩（真 harness 的 sim JS 流程——
   三种探针形态全 clean，溢出在 testharness 页面自身 JS 累积态）
2. insertNode HRE 86F 残余（952P 基线重新聚类）

## 五、commit

f7d3e6a4e
