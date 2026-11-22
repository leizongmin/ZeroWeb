# DC-2 切片 5 刀③b① 证据：TT 属性 sink 六 setter 通道（2026-11-23）

## 交付面（part05/part03 四钩点 + part14 判定表升级）

1. **part05 `_zwSetAttributeNodeCore` 入口钩**（单点覆盖四通道）：fresh 元素
   `NamedNodeMap.setNamedItem/setNamedItemNS` 经 R122 identity 路由直入本核心
   （part03 `_attributesProxy` setNamedItem 臂），加上 part04 trap 直达的
   `Element.setAttributeNode/setAttributeNodeNS`——四通道同一钩点。**ordering
   语义**：sink 检查在 InUse/同属性早退**之前**（error-cases 页八行断言先抛
   TypeError）；detached attr 的 `attr.value=` 不查（IDL DOMString 转换后只在
   绑定时过 enforcement——语料主矩阵 `createAttributeNS → value=` 依赖此不抛）。
2. **part03 `_r122Set` 传播钩**（Attr.value/Node.nodeValue 通道）：sink 检查在
   传播 try **之外**（原 `_eA122` 吞 TypeError 根因——WPT Attr.value/
   Node.nodeValue 行「did not throw」）；管道返回值接回 `s`（default policy
   变换后写传播）+ 置位单次透传旗标（传播臂不再二次进管道）。
3. **part03 `Attr.prototype.textContent` accessor**（Node.textContent 通道，
   新挂）：spec dom-node-textcontent setter 的 attribute 特例（「If this is an
   attribute, then set an attribute value」）——与 value/nodeValue 同源
   `_r122Set`。配套三处 own 属性清理：`_zwMakeAttr` 不建 own textContent
   （own writable 拦截外部赋值使传播失效——value/nodeValue 同款约束）、
   `_r122Set` 内部直写改 data-only（防 setter 自递归）、`attrObj` 缓存同步去
   textContent 赋值（误触 setter 传播臂）。
4. **part14 判定表升级 `_zwTtAttrSinkData`**：返 `{type, sink}`——sink 名按
   spec 形（语料 default-policy 页实测）：事件处理器恒 `Element <local>`
   （svg g ondblclick → "Element ondblclick"）、`HTMLIFrameElement srcdoc`、
   `HTMLScriptElement src`、`SVGScriptElement href`。`_zwTtAttrSinkType` 保留
   为 type-only 包装。
5. **part14 default policy 回调参数序修正**：`(input, type, sink)`（spec
   process-value-with-default-policy；语料 createTrustedType 按
   (input, trustedTypeName, sinkName) 断言——刀③a 误序 (input, sink, type)
   在无 default policy 页不可见）。
6. **part14 单次透传旗标**（`__zwTtBypassNextSink`）：Attr.value setter 管道
   已变换后传播经 `oe.setAttribute` 会二次入管道——default policy 二次调用
   （语料 policy 回调 assert `seenSinkName === undefined` 二现崩）。置位/消费
   同一同步流，管道抛 TypeError 路径不置位，无跨调用残留。
7. **part04 两臂迁移**（setAttribute/setAttributeNS）：`__zwTtAttrSinkType` →
   `__zwTtAttrSinkData`（带 spec sink 名进 default policy 回调）。

## 探针（zz-knife3b-probe.html，临时件不入库）

刀前基线：pb3–pb11 九通道全不抛（六通道 + ordering 面现行为定谳）、pb1/pb2
（createAttributeNS 字段面 + attributes[0] Attr 形态——ownerElement 已是
proxy）、pb12/pb13（TT 实例透传 + 非 sink 不抛）基线过。刀后 13/13 全绿
（TypeError 全臂 + ordering 两臂 + 非 sink 写传播 + TT 透传）。

## 语料对账（set-attributes-require-* 三页 536 测试）

**287P/249F（刀③a 基线）→ 536P/0F（+249P/-249F 全清）**：

- no-default-policy 264P/0F（③a 时 142P/122F）——六通道矩阵行全翻。
- error-cases 8P/0F（③a 时 0P/8F）——ordering 八行（TT 检查先于 InUse/同
  属性早退）。
- default-policy 264P/0F（③a 时 145P/119F）——default policy 管道三修后全
  翻（参数序 + sink 名 + 单次调用）。

中途形态如实记档：钩点落地后 default-policy 页 153P/111F（管道返回值丢弃 +
传播二次调用两根因），修正后全清。

## 回归网（五证）

- zero-engine lib 全量 **4249P/3F 精确回基线**（3F = min_max + numeric_special
  + offset_path_m2 全预存挂账）✓——Attr 内部改造（textContent accessor 化 +
  own 属性清理）零回归。
- knife③a 探针 16P/0F 恒等 ✓、knife③b 探针 13P/0F ✓。
- san spot **323P 恒等**（F 328 同 ③a 记录值，页预算截断落位抖动域）。
- tt 非 enforcement 页（set-attributes-no-require）**202P/0F**（③a 时点记
  179P——+23P 为 textContent/value 写传播修复的附带减免，非 enforcement 面
  的 round-trip 行）。

## 挂账（3c+）

- **HTML sink 族（刀③b② 进行中）**：innerHTML/outerHTML/insertAdjacentHTML/
  document.write/ShadowRoot.innerHTML/iframe.srcdoc 属性反射通道/script.src
  属性反射通道/script.text|innerText|textContent（TrustedScript）。
  block-string-assignment 族基线 31P/82F（15 页可跑）。
- `block-string-assignment-to-attribute-via-attribute-node.html` rc=1 零输出：
  页内 `<script src="x">` 404 载入永不完成 → load 链阻断 → 结果不上报（已知
  engine 载入挂起面——与 wptfyi-html 批二十四 speculative TO 面同根因，该刀
  位收）。六通道就位后本页预期全翻，届时回补。
- script sink wrap 重设计（刀③③/3c）：block-string-assignment-to-*-setTimeout
  页族在册。
- DOMParser.parseFromString / Document.parseHTMLUnsafe / setHTMLUnsafe /
  Range.createContextualFragment（block-string 四页：API 缺失域，独立刀位）。
