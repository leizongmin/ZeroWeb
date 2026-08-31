# R320 Evidence — events/MO 备档面假设复核（三域旧归因维持 + handlers-changed 归因精确化 + template 惰性语义资产化）

**日期**: 2026-08-28
**切片**: M4——R320(b) 备档面假设复核（R313「假设会过期」教训的系统性应用第二轮）
**改动面**: `part24.rs`（+2 正式测试：handlers-changed 基线序 + template 惰性语义；零生产代码改动）

## 一、逐域复核结论

| 域 | 旧归因 | 复核结果 |
|---|---|---|
| Event-dispatch-handlers-changed（1F） | 「target 阶段双 listener 拷贝语义——dispatch 循环深改」 | **归因精确化**：快照/removed-flag/once 机制均在（R111/R34/R35），基线四站序正确（探针断言 `0@1@parent,0@2@target,1@2@target,1@3@parent`）；真缺口 = **listener 内 swap（remove 自身 + add 新 handler）后，同站后续站点的快照循环仍消费新 add 的 handler**（3@3@parent 稳定复现）。疑点收敛到「add-then-later-station 的快照时序」，深结构备档维持 |
| MutationObserver-document（3F） | parse-time mutation 不产 record（html5ever 解析流无 hook 点） | 维持——`assert_unreached: document observer did not trigger` 形态不变 |
| event-global-onerror（1F） | window.event 跨 realm 恢复（R312 时代） | 维持——两 `[object Object]` 的 identity 差 = 跨 realm event 对象身份，frames[1].Function 构造的 handler 域 |
| Document-URL（1F） | —（新复核） | **环境基建**：`redirect.py` 需服务端重定向，runner 虚拟根无该通道——非 DOM 域 |
| Node-isConnected "Test with iframes"（1F） | iframe 连通域（R181 备档） | 维持 |
| remove-next-sibling-during-replace-with（1F） | —（新复核） | **部分推进**：template 内联 script 不执行已验证（探针 `tplRan=0`，spec 惰性文档片段语义 ✓，资产化为正式测试）；残余 = `replaceWith(content.cloneNode(true))` 后 `querySelector('script')` 返 null——克隆产物的 script 节点在查询视图缺，转模板/克隆域待归因 |

## 二、资产化

- `r320_handlers_changed_attribution`：dispatch 基线四站序回归（守护 R34/R35/R111 既有机制不再退化）
- `r320_template_script_inert_probe`：template 惰性语义（script 不执行 + content 结构）——spec HTML template 元素的行为锚点

## 三、A/B

| 项 | R319 | R320 | Δ |
|---|---|---|---|
| 全量 dom sweep | 54140P/58F/22T | **54140P/58F/22T** | Fail set 恒等（Timeout 双向漂移 1 例单跑 Pass）|
| engine --lib（v8/quickjs）| 2457/1460 | **2459**/1460 | +2（纯测试）|
| fmt / clippy（v8 guarded + quickjs）| — | 干净/0 | — |

## 四、教训

1. **探针断言须与实现语义解耦验证**：r320 探针首版把 swap 形态误当基线（重写时丢守卫），
   断言失败暴露了「3@3 稳定复现」这一真归因线索——测试失败本身成了证据。
2. **备档复核的产出形式**：假设维持时把归因精确化记录（handlers-changed 从「深改」收窄
   到「add-then-later-station 快照时序」），比笼统重跑更有下一轮价值。
