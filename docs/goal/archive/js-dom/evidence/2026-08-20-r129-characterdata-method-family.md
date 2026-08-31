# R129 — M4 nodes：CharacterData 方法族 spec 校验 + WebIDL 转换语义（44F→0F 全 100%，+47 净）

**日期**: 2026-08-20
**Driving WPT**: `dom/nodes/CharacterData-{appendChild,remove,appendData,deleteData,insertData,
replaceData,substringData,data}.html`（八文件 44F → 0F，157P/157P 双路径 100%）
**账本**: `tests/wpt-runner/imported-tests.txt`（R129 条目）

## 根因（四类）

1. **叶子节点 mutation 族无校验**（18F）：Text/Comment/PI 的 appendChild/insertBefore/
   replaceChild 落到元素分支静默执行——spec `dom-node-pre-insert`「parent 非
   Element/Document/DocumentFragment」须 HierarchyRequestError。
2. **WebIDL 参数语义**（~12F）：appendData 缺参须 TypeError；null/undefined 须显式
   String 转换（'null'/'undefined'）；data= 是 [LegacyNullToEmptyString]（null→''、
   undefined→'undefined'）——旧 `s == null ? '' : s` 把两者都吞成 ''。
3. **offset/count 校验方向错**（~10F）：只有 **offset** 越界（负或 > length）抛
   IndexSizeError；**count 是 unsigned long**（WebIDL 回绕：-1→2^32-1）后按 spec
   「count > length-offset → clamp」——负小值 wrap 后恒 clamp 到余量，**不抛**（WPT
   "small negative count" `deleteData(2,-1)` → 'te'）。
4. **方法存在性与 remove 空缺**（~9F）：`'remove' in textNode` 恒 false（has trap 不含
   get-trap 方法面）；text.remove() 对 handle 父 registry 不生效（parent.childNodes 残留）。

## 关键定位：has trap 双键覆盖

`_makeProxy` 的 Proxy handler 对象字面量**跨 shim part 拼接**——`has:` 键定义了**两次**
（part03 早期 FV 属性版 + part05 expando 版），JS 对象字面量重复键**后者胜**——part05 的
expando has 是唯一生效版。R129 首版把方法白名单加进 part03 的死键，探针实证
（`'validity' in t` true 来自 target own key 而非 trap）后定位。**教训**：拼接式对象字面量
的重复键是静默覆盖——同 handler 加 trap 前先 grep 全 part 确认无既有键。

## 修复面

| 层 | 修复 |
|----|------|
| part04 get trap | isText/isComment/isPI 的 appendChild/insertBefore/replaceChild 一律 HierarchyRequestError |
| part04 五方法 | appendData 缺参 TypeError；四方法 offset 越界 IndexSizeError；count `>>> 0` 回绕 + clamp 余量；data 参数显式 String() |
| part04 set trap | data/nodeValue = 的 LegacyNullToEmptyString（null→''、undefined→'undefined'） |
| part04 remove trap | CharacterData 分支：handle 父 registry 剔除 + `_zwNodeParent` 反链 + record + 迭代器通知 |
| part05 has trap（生效版） | expando 后接方法白名单（remove/appendData 族/data/length/cloneNode/...20 项）——`'remove' in t` 为 true |
| part03 has trap（死键） | 注释标记被覆盖事实（防后人再改死键） |
| part05 parsed-node setter | data/nodeValue 同 LegacyNullToEmptyString |

## A/B 验证

- **CharacterData 八文件**：44F→**0F（157P 双路径 100%）**。
- **dom/nodes 全量**：7902→**7949P（+47 净）**，fail 284→**237（逐文件 diff 零新增）**；
  native 6398→**6445P（+47 同步）**。
- **回归面**：events 419/27F、traversal 1595/9F、collections 48/0F、MO 105/10F、
  Element-classlist 1420/0F——与 R128 基线逐项一致零回归。
- **单测**：engine `test_character_data_method_family_r129`（7 断言组）。
- `make test` 全绿 66 套件（v8+quickjs 双矩阵）；fmt 无 diff；clippy 零警告。

## 教训

1. **拼接式 handler 字面量的重复键静默覆盖**——`has:`/`get:` 等同名 trap 键在跨 part
   拼接后只留最后一个；改 trap 行为先 grep `键名: function` 全 part 确认唯一或定位生效版。
2. **WebIDL 数值参数先查类型再写校验**——count 是 unsigned long（回绕语义）非 signed
   （抛错语义）；「负 count 抛 IndexSizeError」是想当然，spec 步骤是 wrap+clamp。
3. **DOMString 参数的 null/undefined 三态**：普通 DOMString（null→'null'）vs
   [LegacyNullToEmptyString]（null→''）——undefined 恒 'undefined'。`s == null ? '' : s`
   一刀切是两类用例都错的根源。
