# DC-2 · 切片 5 刀①（TrustedTypePolicyFactory 核心，2026-11-22）

日期：2026-11-22。基线：切片 4（`829c56fb1`，tt 通道基线 31P/304F/30T）。目标：按切片 4 census 刀序第①刀——`trustedTypes` 工厂核心面落 shim，破 wholesale "trustedTypes is not defined" 149 处。

## 实现（part12-trusted-types.js 新件 + js_dom_bridge.rs 拼接链尾挂接）

自包含 IIFE（part10/11 先例）、幂等守卫、零新 Rust 桥、零 sink 改动（innerHTML/setAttribute 等既有行为零变化——sink enforcement CSP 门为刀③，本刀对既有通道零回归面）。`window === globalThis`（part06:10289）单点注册覆盖 `trustedTypes` 与 `window.trustedTypes` 双访问形态。

- 接口对象五件（TrustedHTML/TrustedScript/TrustedScriptURL/TrustedTypePolicy/TrustedTypePolicyFactory）：WebIDL interface object 形态（函数+prototype，调用抛 Illegal constructor TypeError）；stringifier/valueOf 基座在 prototype。
- 实例品牌：WeakSet 三件套——is* 判定不看原型链（isXXX 语料：`Object.create(html)` instanceof 真 而 is* 假、`Object.assign` 克隆 is* 假实证）；数据槽 `__zwTtData` 不可枚举自有（克隆不可复制）。
- createPolicy 算法（https://w3c.github.io/webappsec-trusted-types/dist/spec/ #dom-trustedtypepolicyfactory-createpolicy）：回调缺席容忍（{} / null legal——unenforced 语料实证）；缺席回调方法调用抛 TypeError；回调返 undefined → TypeError、返 null → 空串、其余 String 化；多参原样透传（createXYZTests 三参/缺参实证）；unenforced 面重复名容忍、任意名（含空串/非 ASCII）合法（non-tt-policy-name 全 ASCII 码点循环实证）；'default' 名创建即置 defaultPolicy（defaultPolicy 语料实证）。defaultPolicy/emptyHTML/emptyScript 只读 accessor（赋值静默失败——constants/defaultPolicy 语料实证）。
- sink 映射表（#the-default-policy-helpers，探针首轮两案修正后定谳）：
  - getPropertyType：innerHTML/outerHTML **跨 elementNs 普适**（NSURI_ARRAY 全 ns 断言 TrustedHTML——首轮按 ns 门外判错、探针抓回）；script src/text/innerText/textContent、iframe srcdoc 在 elementNs 空/null/HTML 门外（属性名大小写敏感、标签 ASCII 折叠——'ſcript' 期望 null）。
  - getAttributeType：事件处理器**真实清单**判定（HTML spec #event-handlers-on-elements 固定表 ~80 项）× elementNs ∈ {空, HTML, SVG, MathML} × attrNs 必空 → TrustedScript（attributes.js：SVG g[ondblclick]/MathML mrow[onmousedown] 真、FOO ns/attrNs 非空/伪名 ondoesnotexist 假——on* 非前缀匹配系探针抓回的规格点）；iframe[srcdoc]/script[src] HTML 面；SVG 面 script[href]（attrNs 空或 xlink，qualified 'xlink:href' 不命中、elementNs 大小写敏感）。

## 判读注

- 探针轮询假完成事故：轮询条件「runner 进程不在」在构建期即成立（runner 尚未起跑）——零计数＝腿未跑非零案跑；改以 make 链 EXIT/Error 标记轮询。
- tentative 事件处理器全表页超时＝`<script type="module">` + .mjs import——runner 模块脚本通道缺面（非映射表域，记账不入刀）。

## 验证（守卫入口）

二进制锚定：引擎重链后 runner 二进制 md5 `f57e9d3e4d937885`；兄弟流 layout-engine WIP 恒在 A/B 两侧内部抵消（同切片 4 纪律）。

- 工厂探针腿（FILTER=TrustedTypePolicyFactory，首轮→修后复跑）：首轮 **160P/21F/3T** 抓回两映射案（innerHTML/outerHTML 非 ns 普适、on* 前缀匹配）；修后复跑 **163P/18F/3T**——工厂核心脸全绿（createXYZ 28/28、getPropertyType 28/28、getAttributeType 23/23、isXXX、constants、defaultPolicy、non-tt-policy-name、unenforced）。余 18F 全落刀③域（cspTests 簇 6＝CSP trusted-types 指令面、metadata 12＝require-trusted-types-for 面）+ namespace.html 1（页面级 getElementsByTagNameNS 微刀域）；3T＝cspTests 两页（CSP 面）+ event-handler tentative（模块脚本通道缺面）。
- tt 全腿：**943P/1012F/68T** vs 切片 4 基线 31P/304F/30T——**+912P、逐文件 P loss=0**（join 全 179 文件对账）；F 增量非回归而是计数域换形：基线页首引 trustedTypes 即抛＝页级单 F，现页真跑、失败下沉到子测粒度。F 内容归因（census 全落 enforcement/default-policy 域＝刀③）：set-attributes-require-* 254F（CSP 强制 sink 面）、Node-multiple-arguments 180F（script sink 面）、eval-function-constructor 64F、script-enforcement 64F、HTMLElement-generic 48F、default-policy 簇 56F+18F+9F、block-string-assignment-* 残余等；T 30→68（reporting/Worker 域页 + 页预算带，runner worker 通道域记账）。page-script-threw 从 149 处降至 1 处（namespace.html getElementsByTagNameNS 唯一残余）。
- san 全腿：**60P/503F/1T** vs 基线 39P/363F/1T——**+21P、逐文件 loss=0**（4 处 trustedTypes 交叉依赖页解封后跑得更深；Sanitizer 构造面仍缺＝刀②域，F 增量同计数域换形）。
- dom 回看 spot（FILTER=interface-objects）：**23P/0F 与切片 3 基线恒等**（23P）——新全局注册零碰撞旁证；零 sink 改动面下 dom/wc 通道无行为面变化（part12 纯新增 API，不改任何既有全局）。

净增记账：+933P（tt +912 / san +21）进覆盖面，累计 ≈78,903/107,971。

## 刀序剩余

- 刀② Sanitizer 构造面 + setHTML/setHTMLUnsafe 挂接（san 语料 wholesale 104 处）。
- 刀③ sink enforcement（require-trusted-types-for CSP 门）——cspTests 簇/metadata 12F/block-string-assignment-* 依赖面。
- 微刀候选：document getElementsByTagNameNS/getAttributeNode（namespace.html 页面级 1F，DOMParser 文档 ns 查询面）；runner 模块脚本通道（tentative 全表页）。
