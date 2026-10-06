# slice37 设计卡：Window named properties object 语义收口

状态：探针完成（Chrome 154 对照 + ZeroWeb 687b12ecd before 双实录 + WPT 组 31P/13F/2T 复现），
2026-10-06。

## 根因假设

现状 named access 三路安装全部写 globalThis own data property：

1. 动态面 `_zwNADefineNA`（part05:10728）`Object.defineProperty(globalThis, …, {w:true,e:false,c:true})`
2. 静态面 `_installNamedAccess`（part06:8245）`globalThis[id] = el`
3. R139 contentWindow 注册（part04:766）`globalThis[name] = entry.win`

WebIDL §3.7.4 要求 named property 位于 `Window.prototype` 之下的 WindowProperties
named properties object（NPO）原型层。缺口逐条对应 9F+1T：

- `Window`/`Window.prototype` 构造器面不存在 → prototype.html 2F、window-np 2F（"Window is not defined"）
- named property 不在原型层 → gsp descriptor/`in`/hasOwnProperty 面全空；own 安装直接覆写
  EventTarget.prototype 遮蔽名 → prototype.html 另 2F（window.named3 === element 而非 "shadowing"）
- window 级 delete 后无原型层兜底 → slice36 FIXME ⑥「delete 后再读不复活」（Chrome 实测应复活）
- iframe 名通道 lazy（仅 window 'load' 派发或 contentWindow 首读注册；runner 两点皆无）→
  window-np "Static name"/"duplicate property names" 缺值
- 子 realm（`_zwMakeIframeWin` 对象字面量，proto=Object.prototype）无独立 NPO 链 → cross-global-npo 1F

## 语义裁定（Chrome 154 headless 实测，repro/npo-probe.html + /tmp/s37/arb.mjs）

- NPO 描述符 = `{writable:true, enumerable:false, configurable:true}`。派发书「configurable:false」
  与 WPT/Chrome 均不符——以 WPT/Chrome 为准（探针 s2.gspDescNpid1 = w=true,e=false,c=true）。
- 遮蔽可见性 = **仅 EventTarget.prototype / Object.prototype 层 own 属性隐名**：
  - window own expando 不隐（prototype.html test 1）
  - Window.prototype own 不隐（test 2 + arb：iframe 名 shIfr 被 Window.prototype own 遮蔽仍 NPO own）
  - ET.prototype/Object.prototype own 隐 + 原型跌落（test 3/4）
  - iframe 名 "constructor" 因 EventTarget.prototype.constructor own 而隐（arb ctor.npoOwn=false）
- delete：window 级 sloppy/strict 均 ret=true 且再读经 NPO 复活（含 iframe 名）；NPO 级
  sloppy ret=false / strict TypeError（WebIDL §3.7.4.3 [[Delete]]→false 的 JS 观察面）
- `Object.getOwnPropertyNames(npo)` 不含 named prop（arb npoGpnHasBar=false），与
  hasOwnProperty 真 面并存 → 实现须空 target + trap，不可把值放 target own
- `globalThis.[[Prototype]]`：Chrome 不可变（WebIDL [Global] ImmutablePrototype，
  probe s5 THREW:TypeError）；ZeroWeb V8 全局可变（s5 settable+restored 实证）→ 允许接线。
  ImmutablePrototype 本切片不实现（目标用例不断言；残余申报）
- ZeroWeb before 链形 window→X→Object.prototype（X 贡献 0 可枚举 own，etcheck 实证）→
  X 脱链无 enumerable 损失

## 架构决策

**Proxy 空靶 NPO + 注册表 backing 收口**（双腿同一份 shim JS）：

1. part05 EventTarget 定义（:12654）后：ET.prototype 三方法（addEventListener/
   removeEventListener/dispatchEvent）重定义 enumerable:false（Chrome 同款；防 for-in(window)
   747→750 回归，etcheck 实证它们是仅有的可枚举 own）→ 构造 NPO → 构造 `Window` 函数 +
   `Window.prototype`（own constructor {w:true,e:false,c:true}，[[Prototype]]=NPO）→
   `Object.setPrototypeOf(globalThis, Window.prototype)`
2. NPO trap 面：`getOwnPropertyDescriptor`/`get` = backing（slice36 注册表改写目标）命中 →
   否则 iframe 名 live 扫描（首棵 contentWindow，经 R139 既存 `_zwLoadIframeEntry` 面，gen 缓存
   防热路径）→ 隐名时原型跌落（ET.prototype→Object.prototype 逐层 own 查找）；`set`/
   `defineProperty`/`deleteProperty`/`preventExtensions` → false；`setPrototypeOf` → 仅接受
   EventTarget.prototype；`ownKeys` → 空（对齐 Chrome gPN 面）；Symbol.toStringTag =
   "WindowProperties"
3. 安装面收口（读写面换 backing，语义/守卫/集合 live 维护全保）：
   `_zwNADefineNA`/`_zwNARegisterName`/`_zwNAUnregisterEl`/`_zwNAInstallCollection`（part05）、
   `_installNamedAccess`（part06）、R139 注册（part04）。`__zwNamedAccessInstalled` 登记账本
   仍留 globalThis（记账非安装面）
4. renderer `js_worker.rs` 快照臂 JS：换代回收/新 context 清扫/登记重扫的 `globalThis[k]` 读 +
   `delete globalThis[k]` 改走 backing 助手（`__zwNAGet`/`__zwNADelete`/`__zwNAOwnKeys`）——
   NPO 化后 own 不存在，不改则换代回收静默失效（跨文档残影）
5. 子 realm：`_zwMakeIframeWin` 尾部（return win 前）挂 per-child 链
   `win → childWinProto(own constructor) → childNPO(live 查询 child doc id/name) →
   EventTarget.prototype(共享主 realm)`（cross-global-npo.html 五层走查面）
6. slice36 FIXME ⑥ 申报更新：delete window.N → ret=true 且再读复活（Chrome 同款），
   责任转移说明随 manifest；NPO 级 delete strict/sloppy 双臂新钉（派发书「顺带闭合」项）

## 替代方案（弃）

- named property 装 configurable:true own 属性：反语义捷径（派发书禁止），descriptor 断言
  (true,false,true) 直接红
- 弃注册表改每读全量树查询：slice36 动态名面 31P 全量重测，pending 节点/树序面退化风险大
- V8 Rust 侧 named property interceptor：rusty_v8 蓝图 API 未在此 shim 层暴露，JS Proxy 全覆盖

## 不可退化项

- WPT named-access 组 before = 31P/13F/2T（gates/wpt-named-access-group-before.log，
  687b12ecd 复现 = slice36 终态）；slice36 动态名面 31P 不回退
- 静态安装面 16P 基线钉（js_dom_bridge_tests s27/s32 + js_worker s27/s28/s30/s32 钉族）全绿
- for-in(window) 计数守恒（before 747）
- basics "not enumerable"/existing-prop 用户值遮蔽/strict-mode-redefine（form name=location）
  三面不回退
- named-objects/nested-context/window-null-names（R139 通道三案）与 cross-origin 404 案：
  原样不追（iframe 名 live 扫描对其的附带影响如实记录）
