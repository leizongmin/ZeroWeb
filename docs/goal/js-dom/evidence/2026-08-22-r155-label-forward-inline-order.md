# R155 — LABEL 激活转发 + inline handler 时序 + oninput inline 执行

**日期**: 2026-08-22
**里程碑**: M4（WPT dom 上游基线扩展）
**commit**: `24702bb89`
**驱动用例**: `dom/events/Event-dispatch-single-activation-behavior.html`（132 subtest）

## 根因与修复（四件）

### ① node.click() 的祖先 LABEL 激活转发

`_zwMEl` 的 `node.click()` 只处理本节点 tag——非激活 target（LABEL 内 span）的 click 无任何
激活。补「上行最近激活元素」判定：parentNode 链找首个激活行为祖先，LABEL → 转发其内部
第一个 labelable 控件（input/button/select/textarea）的 click；遇其他激活元素（INPUT/
BUTTON/A/AREA/DETAILS/SUMMARY）止步（nearest 语义）。

### ② inline onclick 时序修正（LABEL 簇 11F 根因）

`node.click()` 旧版把 inline onclick 编译执行放在**最前**（INPUT 翻转前）→
`onclick="this.checked ? activated(this) : null"` 的 `this.checked` 恒 false → activated
永不调。spec 派发模型：inline handler 是 **listener**，在 pre-click activation（checked
翻转）之后触发。修=执行移到 INPUT 翻转账建立后、dispatchEvent 前（与 proxy 侧 R108 同序）。

### ③ inline oninput/onchange 执行（仅本地链）

`node.click()` post-activation 的 input/change 事件经本地派发（doc/docEl/body 三站
listener）——inline `oninput` 属性 handler 非 listener 不触发。修=本地链在派发前显式编译
执行 inline oninput/onchange。**proxy 路径不加**：R2934 泛型 on* 编译已把 inline 注册为
input/change listener，再加直执行会双触发（单测 acts:2 回归验证后删除）。

### ④ connected 判定宽容 fallback

`_zwClickActivationConnected` 对 clone 产物（无 `_zwNodeParent` 反链记账的 handle 元素）
返 false → input/change post 段被跳过。补三个 fallback：`document.contains(el)`、
parentNode 上行（sel 节点即 connected）、`_handleChildren` 反查宿主容器。注：checkbox@FORM
簇的 clone checkbox 三个 fallback 均未命中（proxy parentNode 也 null——反链缺失是结构性的），
12F 仍留（见未收）。

## A/B 验证

| 项 | 结果 |
|----|------|
| single-activation | **118P/14F**（vs R154 107P/25F：-11F；LABEL 簇 11F 全收） |
| 剩余 14F 归因 | checkbox/radio@FORM·LABEL·DETAILS 12 + radio@LABEL 1 + 1——clone checkbox（handle proxy 无 sel 无 `_zwNodeParent` 反链）的 connected 判定 + post input/change 链未通，**结构性缺口**：cloneNode deep 分支 innerHTML 重建的子 proxy 未建反链（源头修复方向：`_zwFragmentAdded` 同款 parentNode 重指或 appendChild 记账） |
| 全量 dom 套件 | **6290P/264F/18T**（vs R154 6279P/275F：净 +11P/-11F，fail 集合 diff 零新增） |
| `make test` | 66 套件全绿 |
| fmt / clippy | 零警告 |
| 单测 | r155 两件（template.content clone checkbox click 链含 inline oninput；createElement acts:1 防双触发回归）+ 既有 r154/r155b 全绿（2299 total） |

## 方法学沉淀

- **内联 WPT 诊断用例**是定位 testharness-vs-沙箱差异的最快路径：复制 WPT 用例结构到
  `zz-*.html` 临时文件，`assert_equals(x, 'MARKER', diag)` 强制失败输出诊断串（runner 打印
  assert 消息）。比沙箱单测猜测快一个量级。用完删除。
- **双路径 inline 语义**：proxy（有 sel/handle → listener 链覆盖 inline）与本地 `_zwMEl`
  （无 listener 基建 → 须显式执行）的 inline handler 执行策略不同——修复时须分辨 target
  形态，防双触发。

## 未收（记入 R156 候选）

- single-activation 剩 14F：clone checkbox 的 `_zwNodeParent` 反链结构性缺失——正解在
  cloneNode deep 分支给重建子建反链（或 `_zwFragmentAdded` parentNode 重指的 proxy 对称），
  属 clone/registry 深结构。
- Element-matches.html 整页 error（连续三轮未动——下轮优先）
- Attr-prefix 2F / MO-document 3F / realm·adopt 族
