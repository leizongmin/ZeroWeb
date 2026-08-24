# R219 Evidence — insertBefore 防递归 + insertNode endOffset 同步 + deep-clone 工厂化

**日期**: 2026-08-24
**切片**: M4——R218 记录的 12,x–15,x Maximum call stack 72F（页面上下文插桩定位：restoreIframe 的 refDoc documentElement 容器 insertNode 自递归，探针实测 depth 4198）
**改动面**: `part03.js`（Node.prototype.insertBefore own-property 防递归判定 + proxy 委托 + deep-clone Text/Comment 工厂化 + 三方法原型兜底 kill-switch 暂缓）+ `part06.js`（Range.insertNode element 容器 endOffset 同步）+ `part21.rs`（回归单测）

## 一、根因定位（页面上下文插桩）

- R219-sur-probe（assertNodesEqual 包装 + A/E 树签名 dump）：12,x 溢出点 =
  restoreIframe 的 `refDoc.documentElement.cloneNode(true)` 产物（iframe 子文档
  合成 docEl，原型链 HTMLHtmlElement.prototype → Node.prototype）作容器调
  `insertBefore` —— 旧实现 `typeof this.insertBefore === 'function'` 命中**原型
  方法自身** → 无限自递归（depth 4198 Maximum call stack）。
- 同款教训链：R126 removeChild / R127 replaceChild（own-property 判定防自递归）。

## 二、实现三件

1. **insertBefore own-property 防递归**（R126 同款判定）：
   - own `insertBefore`（proxy/_zwMEl/detached doc/工厂元素）直调；
   - **proxy 委托修正**：proxy 元素的 `insertBefore` 经 get trap 动态返回、非
     own property——own-property 判定单独会误判「无实现」走兜底绕过 host 桥
     （首版实测 surround -350P）。判据 = `__zwHandle` 经 get trap 可读 → 直调
     trap 版本；
   - 无实现回落 appendChild 兜底（R219 试过本地 splice 插入语义——绕过
     detached doc 的 `_tree` 查询树使 restoreIframe 后 sim 残留跨轮泄漏
     （surround rows 12–24 P2F 350+），已回退；本地插入须等 iframe
     contentDocument 每轮 fresh-doc 后再做）。
2. **Range.insertNode endOffset 同步**（part06，spec `dom-range-insertnode` 末步）：
   element 容器 collapsed 插入后 `setEnd(parent, newIndex+1)`——与 Text 分支的
   syncEnd209 同款；Range-insertNode 15,x「resulting range position」的
   endOffset expected 2 got 1 簇。
3. **deep-clone Text/Comment 工厂化**（part03 `_zwDeepCloneEl`）：旧裸对象
   `{nodeType,nodeName,...}` 缺方法面，common.js nextNode oracle 的
   `node.hasChildNodes()` 直接 TypeError（Range-extractContents「Returned
   fragment」簇）——改经 `_zwMText`/`_zwMComment` 重建（方法面 + 原型链全配）。

## 三、暂缓项（kill-switch 记录）

`Node.prototype.contains` / `compareDocumentPosition` / `hasChildNodes` 三原型
兜底已实现但经 `_r219ProtoMethods = false` 暂缓启用：启用后 sim
（common.js mySurroundContents）深入 iframe 子文档合成树，但 shim 的 iframe
contentDocument 是跨轮共享对象，restoreIframe 清理只动 doc 首末子——sim 的
部分变更（head 内 comment/PI 移入）跨轮残留使 mega-case 后续 subtest 树形态
与 host 分歧（surround 873→523P 实测 -350）。待「iframe contentDocument 每轮
fresh-doc」（R208 家族深项）后随同启用。

## 四、验证链

- **单文件**（vs R218 基线）：
  - insertNode **952P→1094P（+142）**——12–15,x Maximum call stack 72F 全消
    （12–15,x 现 142P）+ endOffset 同步 F2P；P2F 4（15,x head 容器——
    headEl 无 own 方法、插入不生效，fresh-doc 后收口）
  - cloneContents 149→**153P（+4）**；extractContents 98→**100P（+2）**
    （deep-clone 工厂化）
  - surroundContents 873→**854P（-19）**——sim 深入后的跨轮残留（已定位
    headEl 裸数组泄漏机理，fresh-doc 深项后回收）
  - deleteContents 56 不变
- **dom 子域**：nodes 12662→12661（-1）/ events 578→576（-2）/ traversal
  1595→1595 / collections 49→49
- **engine 单测**：2360 全绿（新增 `r219_insert_before_anti_recursion_and_fallback`
  ——plain 防递归 / proxy 桥委托 / own 实现直调三层断言）
- **fmt / clippy**：零 diff / 零警告
- **make test**：1F = window_surface_present_smoke XOpenDisplayFailed（无 X
  server 环境项，run-rules §10，与 R218 同款非回归）

## 五、净评估

约 **+126P 净增**（insertNode +142、clone +4、extract +2、surround -19、
nodes -1、events -2）。surround -19 的机理已完全定位（跨轮残留），与 +142 的
溢出消除同为「sim 深入」的正负两面——fresh-doc 深项落地后两侧同时回收。

## 六、commit

87923fe08
