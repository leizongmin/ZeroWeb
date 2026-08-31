# R302 Evidence — cross-realm MO 机制三件（构造器 + Function 绑定 + 回调异常上报；文件 1F 深化推进）

**日期**: 2026-08-27
**切片**: M4——R302(a) MO 剩余 4F 首件（cross-realm-callback）
**改动面**: `part05.js`（iframe win 补 MutationObserver 转发 + Function 绑定构造器印记 realm）+ `part01.js`（MO flush 回调异常「report the exception」）+ `part24.rs`（+1 单测）

## 一、三件修复（机制层，engine 单测验证）

WPT `MutationObserver-cross-realm-callback-report-exception`（`new frames[0]
.MutationObserver(new frames[1].Function('throw ...'))` → frame1.onerror 收报）：

1. **iframe win 补 `MutationObserver`**（part05 `_zwMakeIframeWin`——旧缺名
   "not a constructor" 直接 TypeError，测试第一关即挂）；
2. **`Function` 绑定构造器**（R187 Object/Proxy.revocable 同款）——产物印记
   `_zwRealmWin` + 入 `__zwRealmOf` 注册表（revocable 后反查用）；
3. **MO flush 回调异常上报**（part01 `_moFlush` 的 `catch (_e){}` 静默吞改为
   `_zwReportListenerError(err, realm)`——realm 从 callback 印记/注册表反查，
   R187 定向派发到该 realm win 的 error/onerror）。

engine 单测（r302）：假 realm win + 注册表印记回调 → `disp:error|onerrorHit`
全链验证 ✓。

## 二、文件级 1F 的剩余差距（深结构记档）

真实文件注入探针（fetch-dom-subset.sh 会重拉，注入后立即跑）逐层归因：

- `fnIs=function ✓ / cbStamped→reg=true ✓ / onerrorSet ✓`（机制件全通）；
- **最终断点**：`mo.observe(frames[0].document.body)` + `target.append('foo')`
  → **回调零命中**（`hits=` 空）——iframe 工厂 body 的 append 走工厂直插
  （part05 docEl.append/el.append 的 `childNodes.push`），**无 `_mo_notify`**；
  且工厂节点无 handle/sel → `_mo_id` null → **observe 静默丢弃**（R188 曾为
  document 打 'doc' 专用 id 同款问题——工厂元素域系统性不可观察）。

**归档**：iframe 工厂元素的可观察性（identity 方案 = 工厂节点稳定 id 贯通
observe/notify 两端）归 R220 深结构域（与 handle-append 视图桥、tree-order
同族）。本切片的机制件为该域解除后此文件立即转绿铺路。

## 三、验证

| 套件 | 基线 | R302 | Δ |
|---|---|---|---|
| MutationObserver-cross-realm | 1F（TypeError 崩） | 1F（**形态深化**：构造器/绑定/上报全通，剩 observe 工厂 body 域） | 质变 |
| MutationObserver 全族 | 114P/6F | 115P/6F | +1P（子测试解锁） |
| EventListener-cross-realm 族 | 18P/0F | 18P/0F | 持平（Function 绑定无扰动） |
| Text/Comment-constructor | 16P/16P | 同 | 持平（R295 消费方） |
| node-creation-realm | 13P/0F | 同 | 持平 |
| engine 单测 | 2439 | **2440**（r302：realm 上报全链） | +1 |
| make test | — | 1F = XOpenDisplayFailed 环境项 | 持平 |
| fmt / clippy | — | 干净 | — |

## 四、教训

**fetch-dom-subset.sh 每次重拉覆盖注入**：`make testharness-dom` 的 fetch 依赖
发现文件存在即跳过（`-s ${target}` 非空跳过）——**注入必须在 make 之前完成且
不经过 restore-then-run 顺序**；R299 的成功流程是 inject → make（fetch skip）→
restore。本轮两次 restore 先于 make 执行使探针落空，浪费两轮。
