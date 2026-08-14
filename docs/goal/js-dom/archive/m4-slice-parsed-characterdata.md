# M4 切片 R48 — parsed DOM CharacterData 编辑 + characterData record（SetChildText 全链）

**日期**: 2026-08-15
**里程碑**: M4 / DC-3（nodes MutationObserver）
**证据**: [../evidence/2026-08-15-r48-parsed-characterdata.json](../evidence/2026-08-15-r48-parsed-characterdata.json)

## 切片动机

MutationObserver-characterData 4P/12F——失败整簇 `appendData/insertData/deleteData/replaceData is not a function`：方法仅 handle-based 文本节点（createTextNode 所建）有；WPT 编辑的是 **parsed DOM 文本节点**（`<p id=n10>CHAN</p>.firstChild`，`_wrapNodeEntry` 普通快照对象无 handle）。

## 实现（host + shim 全链）

### host 侧

- `DomMutation::SetChildText { parent_selector, child_index, text }`：按父 selector 定位 + childNodes 索引（与 `__zw_child_nodes` JSON 同全节点序）替换子文本
- 回调 `__zw_set_child_text`（part callbacks.rs）

### shim 侧

- `_wrapNodeEntry` 文本/注释对象补 appendData/insertData/deleteData/replaceData/substringData + `data`/`nodeValue` setter——写经「父 sel + `__zwChildIndex`」（`_childNodeList` map 时盖章）；本地 `__nv`/`textContent`/`length` 同步（同块读不 stale）
- characterData record 发到父 id，`characterDataOldValue` 请求时写前捕获 oldValue

### observe 回落

`MO.observe(textNode)` 原对无 sel/handle 的普通对象 no-op——现回落**父元素 id**（文本节点 characterData 观测可投递；record.target 仍为文本节点）。

### 同轮 bug

初版 notify 参数序错（`(null, parentSel)` 对 `(sel, handle)` 签名——parentSel 落 handle 位生成 `h:` id）——probe 定位修正。

## 结果

| 项 | 前 | 后 |
|----|-----|-----|
| MutationObserver-characterData | 4P/12F | **18P/0F（100%）双路径** |
| dom/nodes polyfill | 2552P | **2566P（+14）** |
| dom/nodes native | 2522P | 2536P |
| Node-textContent | 17P | 17P 持平（setter 改动验证无回归）|

零回归：events 189P / collections 24P / traversal 9P / ranges 39P / classlist 1420P。

## R45–R48 MutationObserver 族累计

attributes 0→**38P/0F（100%）**、childList 10→15P/1F、characterData 4→**18P/0F（100%）**；dom/nodes 2507→**2566（四轮 +59）**。

## 验证门禁

- 单测 `test_parsed_text_characterdata_r48`（4 断言组）
- engine v8 2129 / quickjs 1415 全绿；quickjs 矩阵 14 crate 全绿
- clippy 双矩阵零警告，fmt 无 diff
