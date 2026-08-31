# R327 Evidence — 执行路径测绘落地：R326 降级件精准重放（Node-mutation-adoptNode 全文件转绿；全量 54143P/56F，Fail set 恰 -1）

**日期**: 2026-08-28
**切片**: M4——R327(a) 执行路径测绘 + R326 降级件重放
**改动面**: `part03.js`（两处：_zwMakeAttr 的 ownerDocument 动态 getter + R112 串行合并分支的 adopt 落表）——**每处均有函数指纹探针确认执行路径**

## 一、测绘结论（指纹探针法，R263 source 指纹的正式化）

`String(body.appendChild)` 的特征片段定位（WPT 环境）：

| 特征 | 命中 | 结论 |
|---|---|---|
| `oTag`（R112 串行合并） | ✓ | body.appendChild = `_makeDetachedDocument` body 字面量 |
| `ensureTree`（通用回落） | ✓ | 同函数（两分支共存） |
| `_r307Anc`（_zwParseEl 域） | ✗ | 非 _zwParseEl 域 |
| `_r191adoptBody`（R191 adopt 块） | ✓ | adopt 落表在通用路径已有 |
| `stable`（doc.body 双读 identity） | ✓ | 静态绑定（非动态 getter） |

**R326 负结果的复盘**：当时的两处补丁位置**都正确**、路径**都可达**——失败的真因是
**stale binary**（cargo 不跟踪 include_str! 依赖，R326 轮的 build 恰好没含 engine 重编，
探针跑在旧 shim 上）+ L4 探针的 assert 串没含 l7 使「getter 是否在场」证据缺失。本轮
touch js_dom_bridge.rs 强制重编后一次三绿。

## 二、落地的两件

1. **R112 串行合并分支的 adopt 落表**：`other_doc.body.appendChild(handleDiv)` 命中
   串行合并早退（`return c` 在 R191 adopt 块之前）→ ownerDocument 印记缺失。补
   `__zwAdoptDocByHandle[handle] = doc`（与 R191 通用路径同源）。
2. **Attr 的 ownerDocument 动态 getter**：`_zwMakeAttr` 无 ownerDocument 字段。补
   `defineProperty` getter 沿 `ownerElement.ownerDocument` 读（adopt 落表重指元素后
   Attr 自动跟随，零维护）。

## 三、A/B

| 项 | R325/R326 基线 | R327 | Δ |
|---|---|---|---|
| Node-mutation-adoptNode | 1P/1F（三年备档） | **2P/0F 全文件 Pass** | +1 |
| **全量 dom sweep** | 54139P/56F/25T | **54143P/56F/22T** | Fail set 恰 -1（zz-r326 遗留探针清理后恒等）|
| engine --lib（v8/quickjs）| 2462/1460 | **2466/1464** | 渲染流并行推进 +4/+4（本切片零新增单测——WPT 资产锁定）|
| fmt / clippy（v8 guarded + quickjs）| — | 干净/0 | — |

## 四、教训（第三次同型事故，升级为硬性流程）

**include_str! 改动的验证前置**：改 js_dom_shim/*.js 后必须 `touch crates/engine/src/js_dom_bridge.rs`
并确认 build 输出**含 "Compiling zero-engine"** 再跑探针——"Finished" 不代表重编
（增量构建跳过未感知依赖的 crate）。R187（改 shim 须重建）、R308（stale binary 假象）、
本轮回放三案例同根因；探针结果与代码预期矛盾时，**第一动作是强制重编**而非追加假设。
