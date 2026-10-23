# M2 切片 44 — 跨代 handle proxy 孤儿 isConnected 假阴性 + activeElement light 锚误穿收口（R5247）（2026-10-23）

## 范围

s39 已知边界「focus-after-close 余 1F：Complex dialog usage focus 恢复红
（activeElement 得 div）」。

## 根因（六层钻透）

Complex 案跨 testdriver 多命令 execute_script，宿主 apply 换代发生在
show（捕获 prevFocused input proxy）与 close（触发 R4600 focusing steps）
之间：

1. 静态树节点的 handle 形 proxy 无 `_zwNodeParent` 反链记账（R91 只记 JS
   动态操作 child→容器），apply 换代后亦不补记；
2. isConnected getter 反链 miss → rect 探测兜底（runner 通道布局空）→
   **双假阴性 disconnected**；
3. R114 detached 臂误入 → prevFocused 焦点所有权解析为「detached」→
   document.activeElement 回落；
4. getter 的 R4600 focus-chain retarget 上行（对象域 parentNode）解析出
   **runner 投影容器形**（BND dump 实证：`_shadowHandles` 跨执行残留注册
   `__n1/__n3/__n5/__n8`，meta `hostSel:null`、hostHandle 指向换代后复用
   的活 handle）→ 误穿边界 retarget 投影 host → 断言得 DIV 非 INPUT；
5. host tagName 防线被复用活 handle 绕过（tagName 可读）——不可判据；
6. `hostSel==null` 判据同样不成立：页面自建 shadow 的动态 host
   （createElement 产物）meta.hostSel 亦为 null——与投影 root 不可区分。

## 修复（R5247，语义判据回归 spec）

- **part04 isConnected getter + part01 `_zwIsConnected`**：handle 形前置
  复核——`__zw_selector_for_handle`（宿主持久 handle→selector，不随换代
  清）反查 + `document.querySelector` 同一性核验（命中**同 handle** 且
  `__zw_contains('html', sel)==='1'` 才短路 true）。false/miss 落原全路径：
  `_zwRemovedHandles` 守门保持 remove 语义；shadow 树内锚宿主扁平 selector
  命中别元素（handle 不合）不短路——R91 反链 shadow 穿越权威判定保持；
  同步窗口反链真值优先不受损；消多实例 selector 歧义。part04 的复核须
  **前置**（反链循环含 R91 host rect 中途 return，后置不可达）。
- **part06 activeElement getter**：retarget 语义前提门——锚经同一性核验
  确认身处 light 树（可被文档查询命中同 handle）则直返锚。spec focus
  chain retargeting 仅当焦点元素身处 shadow 树才穿边界读 host，light 锚
  无边界可穿——该门以锚自身树域为判据，与 stale 注册形态（投影 root、
  跨执行残留、动态 host 均不可区分）解耦。host 解析空壳（tagName 不可
  读）兜底回落原锚。

## 验证

- focus-after-close 单案 **6/6 Pass**（Simple / moved in between / moved
  to shadow DOM root / removed→body / shadow DOM host / **Complex**）；
  文件级 Timeout 为既有 dont_scroll 挂账（focus 滚动面缺口，s39 起既有，
  独立定性）。
- interactive-elements 全域 98 files 482 subtest **420P/60F/2T 对 s43
  逐位 REG 0 / GAIN 1**（Fail/Timeout 清单 diff 仅 Complex 案一行翻绿）。
- zero-engine lib **3437P/0F**（3435 + 2 新契约 part76）。
- 契约翻转力实证（修前自然红）：撤 part04 前置复核 → 钉 1(a) 红
  （孤儿 isConnected 假阴性复活）；part06 light 门改无条件短路 → 钉
  2(b) 红（shadow 锚 retarget 被误杀）。

## 契约（part76）

- `is_connected_handle_orphan_identity_recheck_r5247`：identity 桥键值
  契约自定义（R331 QS 升格以桥值为 handle 物化），静态树升格 proxy 无
  反链记账——生产跨代孤儿的公开面同构；(a) 孤儿 isConnected true、
  (b) remove 后 false（快照未换代 contains 仍在档，removed 守门是唯一
  拦截）。
- `active_element_light_anchor_no_retarget_r5247`：(a) light 锚直返、
  (b) 真 attachShadow 后 shadow 锚 retarget host（R4600 既有面）、(c)
  双向切换稳定。
- stale `_shadowHandles` 投影注册的完整误穿形态公开面不可构造
  （attachShadow 注册键是新建 fragment handle，无法指向 light 锚父），
  自然红钉由 corpus 全域对账钉住（本 evidence GAIN=1）。

## 已知边界

- focus-after-close 文件级 Timeout（dont_scroll 案：focus 不滚 viewport
  → scroll 事件永不发 → promise 永挂）维持 s39 既有挂账，本切片不扩面。
- dialog-focus-previous-outside 3F（shadowroot outside/inside、slotted
  焦点恢复面）为 s39 既有挂账，不属本切片。

## 门禁

zero-engine lib 3437P/0F、clippy --workspace -D warnings（test-guard 包
裹）/fmt clean；零新增依赖；corpus runner release 二进制含 R5247 符号
（`_e5247al`/`_icSelR5247c`）。数据
[2026-10-23-m2s44-orphan-isconnected-retarget.json](2026-10-23-m2s44-orphan-isconnected-retarget.json)。
