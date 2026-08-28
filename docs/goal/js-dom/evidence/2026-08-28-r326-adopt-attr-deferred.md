# R326 Evidence — adoptNode 的 attributes ownerDocument（负结果回退：detached doc 域分派链验证未收敛，降级 L2 主线）

**日期**: 2026-08-28
**切片**: M4——R326(a) 备档集巡检续（Node-mutation-adoptNode 1F）
**改动面**: 无（全部改动已回退，worktree clean，engine 2466 全绿——R325 后渲染流并行推进至 2466）

## 一、探针定位链（四轮）

WPT Node-mutation-adoptNode "Adopting an element into a different document updates
... the owner docs of its attributes"：`div(handle).attributes[0].ownerDocument` 在
`other_doc.body.appendChild(div)` 后应 === other_doc，实测仍主文档。

1. **Attr 无 ownerDocument**——`_zwMakeAttr` 补动态 getter（沿 ownerElement 读，adopt
   自动跟随）→ getter 生效但读到主文档。
2. **跨文档 appendChild 的 adopt 落表缺**——_zwParseEl 域补 sel/handle 子落
   `__zwAdoptDocBySel/Handle` 表（ownerDocument getter 的消费点）→ 表仍无键（L4=false）。
3. **目标 doc 引用误读**——body 字面量的 ownerDocument getter 有「主文档回落」，跨文档
   判定误为同文档；改显式印章（_zwOwnerDetDoc/_zwOwnerTree）→ 仍不生效。
4. **分步探针**（zz-r326-adopt2，assert 消息回显）：`od3=true`（body.ownerDocument ===
   other_doc ✓）而 L1/L2/L4 全 false——**body 的 appendChild 分派根本没走我改的两个域**
   （_zwParseEl 域与 R112 串行合并分支都补了落表仍 L4=false）——body 字面量的实际
   appendChild 来自未定位的第三条路径。

## 二、回退原因

执行路径的**第四条通道**未定位前，三处落表改动无法验证（WPT 仍红），且改动面横跨
Attr 构造、_zwParseEl、R112 串行合并三域——按「不留半成品」与深结构护栏整组回退。

## 三、降级记录（L2 主线待办）

sel/handle 子跨文档移动的 adopt 记账链路须先做**执行路径测绘**（body.appendChild 的
实际解析域——div 域、移动域、wire 域三面枚举），再落表。与 R324 的四环节链路同批，
随 L2 identity 双源统一专项处理。

## 四、基线

worktree clean，engine 2466 全绿（R325 后渲染流并行推进），dom sweep Fail set 恒等。

## 五、教训

1. **探针先行确认执行路径**（本轮计划里写了但没严格执行）——修改前先确认「这次调用
   实际走进哪个实现」，否则补丁落在死代码上。
2. **分步探针的顺序价值**：od3=true/L4=false 一条探针同时排除了「数据源缺」与「消费
   缺」两个假设，把问题收窄到「执行分派」——比逐个改代码快一个量级。
