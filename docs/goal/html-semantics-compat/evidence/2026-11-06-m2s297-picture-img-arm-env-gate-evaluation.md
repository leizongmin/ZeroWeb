# M2 s297 — picture 父 IMG 臂 env 化独立评估销案（维持 s221 unconditional——全语料零红面归因 + env 门卫逐条零可观测增量 + ⓷ 三 EncodingError 红面复核成立）

日期：2026-11-06。切片号 s297（承 s290 挂账「picture 父 IMG 臂（s221）env
化为独立评估面」→ s291/s292/s293/s294/s295/s296 六轮维持）。纯评估零源码
改动切片（docs-only）。

## 勘根/评估

**候选源流**：s290 挂账更新——「picture 父 IMG 臂（s221）env 化为独立评估
面（current-request-microtask 时序面依赖现 unconditional 重验，本片不
动）」；s291 补定谳「picture 臂不设 ⓷（empty-picture srcset 重启→
EncodingError 断言面维持）」。本片独立评估收口。

**臂结构**（crates/engine/src/js_dom_shim/part03.js）：TreeMutated
picture 父 IMG 臂 :4018-4031 = unconditional SelectionQueue（记录半，
s221/s220）+ Revalidate（fetch 半）；Revalidate :3594 内部分流——picked →
RunUpdate（fetch 核自带 normative 守卫：s289 step-11 同 URL 在飞早退 /
s224 请求代纪 / s226 settle 陈旧守卫），null → NullSelection :3638
（s295 src 属性门 + 无属性无状态早退）。对照非 picture 插入臂
:3937-3992（s290 env 门卫 ⓵⓶⓷⓮⓯）+ s285 dormant 臂 :4004-4015。

**结构性论据**：s221「unconditional」非无守卫——下游 fetch 核 normative
守卫全覆盖（同 URL 在飞早退即 spec update-the-image-data step-11 面）；
s290 hook 层 env 门卫存在的唯一原因是非 picture 臂此前零重验（early
return→dormant，s284 收窄背景），可选 env 层（spec reacting-to-environment
-changes encouraged 可选算法，s290/s291 定性）必须落 hook 层；picture 臂走
relevant mutation → update-the-image-data normative 直路，可选层无语料钉
面不加。两臂不同门卫各自钉语料真面，非不一致。

**face 盘点**（s296-fulltree-b.log B 侧，picture 族全绿）：relevant-
mutations「inserted into picture」/「removed from picture」（:1255/:1256
——s221 臂直接钉面）；current-request-microtask 001/002（:1695/:1696，
s221 收口位）；decode-quick-attach picture 6 面（:851-856，含「srcset in
empty picture」「srcset in picture with source」「src in picture with
source not cached」EncodingError 钉面）；img-picture-ancestor 4/4
（:922-925，解析时态不入插入臂）；adoption picture 2 面（:770/:773）；
null-image-source「img with picture parent」（:1220）；picture-loading-
lazy 4 面（:1221-1224）。the-img-element 域 F/T 线全清点**无一 picture 族
红面**——env 化零可修目标。

**逐门卫增量分析**（若 env 化 picture 臂）：

- ⓵（srcset 空→deferral）：spec 判据「does not use srcset **or
  picture**」picture 父恒假——永不触发，no-op。
- ⓶（在飞册 truthy→skip）：quick-attach 面插入点 fetch 未启动（detached
  attr 变更只置 defer 标记 s291⓷，fetch 连接后才启）；新建 img 无在飞
  ——语料插入面零命中。
- ⓷（detached 延步标记→skip）：**实证三 EncodingError 面翻转**——
  quick-attach detached 建图 attr 已置标记→skip 重验→decode 捕获请求存
  活→resolve 非 EncodingError。s291 既有定谳复核成立，env 化必须排除
  ⓷。
- ⓮（选源 null→skip）：picture 父无 src/srcset img——现状 unconditional
  Revalidate→NullSelection→s295 无属性早退（零可见状态）；skip 同零可
  见。同结果。
- ⓯（同 last-selected→skip）：命中面 = 已 settle img 同选源重插/移动
  （adoption cached 型）——现状重启走引擎缓存 hit 可见面不变；skip 同结
  果。语料零断言钉其差异。

**净结论**：⓵⓶⓮⓯ 全集 = 全语料零可观测增量（A/B 门恒全同，无验证抓
手）；⓷ = 三红面回归。env 化 = 无 face 驱动的投机加固 + 连续完美 streak
风险，**销案**——维持 s221 unconditional。

## 挂账更新

- s290 以来七轮「picture 父 IMG 臂 env 化独立评估」**销案**（本账）。
- SOURCE 兄弟臂（:4032-4071，s226）不在候选文本范围（「picture 父 IMG
  臂」），本评估不及——下游同守卫集结构论据可承用，若后续立钉面再评。

## 下一步候选

1. 宿主 documentElement 物化缺口根修（s285 挂账 + s293 挂账②并案评估）。
2. plain-src 运行时 img 触发缺口根修（s293 挂账①——需先立钉面）。
3. PNG eXIf 方向标签支持（png crate exif_metadata 面——无语料消费面，
   s296 FIXME 已注记，低优先）。
4. css image-orientation 属性渲染面移交 rendering-compat 候选（s296 账）。

## 证据

- s296-fulltree-b.log B 侧 face 行号引用（本账盘点节）+ the-img-element
  域 F/T 线全清点（零 picture 族红面；F 线集中 current-pixel-density/
  audio lazy/canvas 异域）。
- 代码锚：part03.js :3937-3992（非 picture env 臂）/:4004-4015（s285
  dormant）/:4016-4031（picture IMG 臂）/:3594-3604（Revalidate 分流）/
  :3638-3668（NullSelection s295 门）。
- 账本链：s290（挂账立目）/s291（⓷ 排除定谳）→ 本账收口。
- 纯评估切片：零源码零日志新增，face 引用与代码锚即为证据。
