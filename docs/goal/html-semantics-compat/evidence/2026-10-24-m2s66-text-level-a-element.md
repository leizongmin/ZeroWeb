# M2 切片 66（s66）——text-level-semantics 开片：the-a-element 两族全收（R5273）

日期：2026-10-24。text-level-semantics 开片（M1 基线 21P/13F），13F 两族
全收，域 **21P/13F → 34P/0F**（GAIN 13 / REG 0，域首次 exit 0）。

## 1. 13F 分诊（两族，全本流 shim 面）

- **a.text getter/setter 9F**：`a.text` IDL = textContent 精确委托
  （HTML §dom-a-text；option.text 的 strip-and-collapse **不适用**——
  a.text-getter-01 断言 `<a>a <!--b-->b c </a>` 的 .text 为全文 "a b c "）。
- **a-stringifier 4F**（stringifier 1/2/6/7）：WebIDL es-stringifier step 2
  品牌检查缺失——`a.toString.call(null/undefined/{})` 须 TypeError（null/
  undefined 非严格函数装箱 globalThis，instanceof 判 false 覆盖）。

## 2. 修复面（R5273，三刀）

1. **a.text getter**（part03 get-trap，OPTION/TITLE 读块邻位）——A tag 门
   `prop === 'text'` 返 textContent（wrap 后读，R5269 title.text 同口径）。
2. **a.text setter**（part04 set-trap，R3907/R5269 委托臂邻位）——A tag 门
   委托 `_makeProxy(...).textContent = value`（string replace all 同 spec
   步骤：childNodes.length===1、firstChild.data 同步、textContent 镜像）。
3. **a/area toString 品牌检查**（part03 get-trap 22795 臂 + 原型侧 6420 臂
   同步）——`this instanceof HTMLAnchorElement/HTMLAreaElement` 门，非
   platform object 抛 TypeError。**探针实证原型侧非实际供给面**：
   createElement 产物 toString 经 get-trap 供出（proxy 拦截全部 get，原型
   defineProperty 不达）——trap 臂才是修复点，原型臂保持一致防直装路径。
   String(a)/`new WebSocket(<a>)` ToPrimitive 走 @@toPrimitive 臂（22773）
   不受影响（this=proxy instanceof 真）。

### 共享路径根修：sel 形注册文本视图并入 overlay（part05）

a.text 委托暴露**预存共享缺陷**：textContent=（sel 形解析元素）写后同 turn
`childNodes.length===0` 而 `firstChild.data` 已见新值（探针 s66 钉定：直接
`a.textContent = "q"` 同样翻车，与 .text 委托无关）。根因链：setter 经
`_zwRegisterTextEl` 落 `_zwTextElsBySel` 注册表 + `_mo_notify` childList →
`_zwHCLiveInvalidate` 桶记 removed=旧 wrapped 子（overlay 剔除生效）+
added=注册文本节点（无 `_zwSelPendingParent` link → overlay 并入跳过）→
childNodes 视图只剩剔除（0 子）；firstChild 走 `_zwLocalChildNodes` 直读
注册表不受 overlay——两视图分裂。R5269 probe62「childLen=0」同根源（title
corpus 不读 sel 形 childNodes 未暴露）。

**修法**：`_zwOverlayPendingChildNodes` 入口注册表种子——`_zwTextElsBySel`
命中即以注册文本节点为基（整体替换语义生效）。后续 appendChild 的 handle
子经 bucket added（`_zwNodeParent` link）照常并入；removeChild 移除该节点
经 bucket removed identity 剔除；host apply 后快照虽含真文本子，种子恒取
注册单节点不双计（bucket added 同节点无 link 不再并入）。innerHTML 纯文本
路径（R294 已把 added 换成注册表同一节点 + R304 link）与种子 seen 相抵，
无双计。

## 验证

- text-level-semantics 域 **34P/0F**（exit 0；GAIN 13）
- 回归：links **119P/20F/4T 与 HEAD stash 二分逐值一致**（20F 预存，未开
  片域）；document-metadata **59P/59F/18T+1 NotRun** 逐值不变（overlay 种
  子触及 title 视图零回归）；tabular 130P/4F、interactive 439P/44F 逐值
  不变；zero-engine lib **3463P/0F**（s64 基线 3458 + oracle 流 +5）
- 探针 probe-s66 两断言绿后删除（gitignored 本地 corpus）
- 拼接 shim node --check 通过；fmt/clippy 不适用（diff 仅 JS shim）

## 未解决项

- links 域 20F+4T（htmlanchorelement attribute-getter-setter 14F + URL
  getter/setter 族 6F + target_blank/rel Timeout 4T）——未开片域预存，
  embedded-content 之后按控制面挂账序开片分诊
- text-level-semantics 域全绿，本域收口。下一域按控制面挂账序：
  embedded-content / scripting-1 / microdata
