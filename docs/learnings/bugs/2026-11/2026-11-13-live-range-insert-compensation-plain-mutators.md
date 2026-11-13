---
date: 2026-11-13
modules: engine
---

# plain 树 mutator 缺 R263 插入补偿致 live-range 塌缩

## 问题

document 作用域取宿主时，bold 700/900 壳部分选区 toggle-off 在 peel 切分
插壳后摘壳永不发生（corpus bold.html 恒红 12F）。元素作用域取宿主同页全绿。
d11 曾定谳为"混合树致 contained 集空转"，但克隆壳 plain 化对齐后 got 仍
re-wrap——形态对齐是必要非充分。

## 根因

两层叠加：

1. `__zwAdjustRangesForInsert`（R263，live-range 插入边界补偿）只在
   doc/handle 面接线，plain mutator `_zwMElInsertBefore` /
   `_zwMElAppendChild` 缺装。tc-scope 宇宙由宿主桥的 proxy insertBefore →
   宿主 mutation → 宿主侧选区自保补偿（探针实测 end offset=2），plain
   宇宙无此通道。
2. JS 侧 `_zwVaShiftEndpointsForInsert` 对 end 边界 (parent, kids) 做
   +1 时走 `setEnd(parent, 2)`——插入前 parent 只有 1 个孩子，撞 part06
   setEnd 校验（offset > nodeLength 抛 IndexSizeError），异常被静默
   catch，range 塌缩为 collapsed → `_zwValIsEffectivelyContained` 全
   false → setSelectionValue 的 clearValue/forceValue 零到达。

另有一个接线位序坑：首版把钩子调用放在 splice/push 之后、parentNode
defineProperty 之前——钩子以 `inserted.parentNode` 定位新父/新索引，
此时 parentNode 为空直接早退，补偿看似接了实则无效。

## 解决

- 两处调用点移到 parentNode 落位之后（`_zwForceParentLink` /
  defineProperty parentNode 之后），注释记录位序契约。
- 补偿写 `_endOffsetBase` 槽（R183 accessor 读出面）绕过 setEnd 校验：
  插入时刻的合法 end offset 可以超当前孩子数，setEnd 按当前树长拦截，
  accessor 槽直写不受限。
- peel 克隆壳工厂按被切壳形态分流（`__zwHandle === undefined` 时走
  cloneNode(false) 复制源壳含 style 归一），保持全 plain 链形态一致。

## 避免方式

- 给 mutator 加 range 补偿类钩子时，先确认钩子的定位依赖：
  以 `inserted.parentNode` 定位的钩子必须在 parentNode 落位后调用，
  否则静默早退——接线后用探针页实测补偿是否生效（CV end offset），
  不能只看"调用已插入"。
- 需要写入超当前树长的 range offset 时走 accessor 槽（`_endOffsetBase`），
  不走 setStart/setEnd（校验按当前树长）。
- 同一能力存在多宇宙面（handle 桥/plain mutator）时，修一面必须核对
  另一面是否同构接线；探针页按宿主获取方式（document.querySelector vs
  元素作用域 querySelector）分腿对照。
- face 全量断言级 F2P/P2F 对拍是方向性错误的快速反证（本例 +4 零回归）。
